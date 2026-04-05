// sovereign-proxy/src/proxy.rs
use crate::rewrite::{self, ApiFormat};
use crate::upstream::UpstreamClient;
use anyhow::Result;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::header::HeaderMap;
use sovereign_prompt::analyzer::PromptAnalyzer;
use sovereign_prompt::config::SovereignConfig;
use sovereign_prompt::crypto::CryptoEngine;
use sovereign_prompt::db::Database;
use sovereign_prompt::governance::GovernancePolicy;
use sovereign_prompt::optimizer::PromptOptimizer;
use sovereign_prompt::templates::PromptTemplateLibrary;
use sovereign_prompt::tokenizer::Tokenizer;
use sovereign_prompt::types::{AuditLogEntry, PromptRecord};
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ProxyState {
    pub db: Arc<Database>,
    pub tokenizer: Arc<Tokenizer>,
    pub config: Arc<SovereignConfig>,
    pub upstream: Arc<UpstreamClient>,
}

pub async fn run(state: ProxyState) -> Result<()> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/messages", post(handle_anthropic))
        .route("/v1/chat/completions", post(handle_openai))
        .with_state(state);

    let addr: SocketAddr = std::env::var("SOVEREIGN_PROXY_ADDR")
        .ok()
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| "127.0.0.1:8788".parse().unwrap());

    tracing::info!("SovereignProxy listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "service": "sovereign-proxy"}))
}

async fn handle_anthropic(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, (StatusCode, String)> {
    handle_proxy(state, headers, body, "/v1/messages", ApiFormat::Anthropic).await
}

async fn handle_openai(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, (StatusCode, String)> {
    handle_proxy(state, headers, body, "/v1/chat/completions", ApiFormat::OpenAI).await
}

async fn handle_proxy(
    state: ProxyState,
    headers: HeaderMap,
    body: Bytes,
    path: &str,
    format: ApiFormat,
) -> Result<Response<Body>, (StatusCode, String)> {
    let is_anthropic = format == ApiFormat::Anthropic;

    // Parse the request body
    let mut body_json: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;

    // Check if streaming is requested
    let is_streaming = body_json
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Extract last user message
    let original_prompt = match rewrite::extract_last_user_message(&body_json, format) {
        Some(p) => p,
        None => {
            // No user message to optimize — pass through as-is
            return forward_passthrough(&state, headers, body, path, is_anthropic, is_streaming)
                .await;
        }
    };

    // Run the SovereignPrompt optimization pipeline
    let token_model = sovereign_prompt::tokenizer::DEFAULT_TOKEN_MODEL;
    let original_tokens = state
        .tokenizer
        .count_for_model(token_model, &original_prompt)
        .unwrap_or_default();

    let feedback = PromptAnalyzer::analyze_with_config(&original_prompt, &state.config.heuristics);
    let refined_base = PromptOptimizer::refine(&original_prompt, &feedback, &state.tokenizer);

    let domain = "general".to_string();
    let (refined, _template) = PromptTemplateLibrary::apply(&domain, &refined_base);
    let refined_tokens = state
        .tokenizer
        .count_for_model(token_model, &refined)
        .unwrap_or_default();

    // Compute content hash
    let content_hash = CryptoEngine::compute_content_hash(&original_prompt, &refined);

    // Governance check
    let policy = GovernancePolicy::current();
    let gov_feedback = GovernancePolicy::validate_prompt(&original_prompt);
    let approval_status = GovernancePolicy::determine_status(&gov_feedback);

    // If governance rejects, pass through original unmodified
    if approval_status == "rejected" {
        tracing::warn!("Governance rejected prompt — passing through unmodified");
        return forward_passthrough(&state, headers, body, path, is_anthropic, is_streaming).await;
    }

    // Log the optimization to database
    let feedback_json = serde_json::to_value(&feedback).unwrap_or_default();
    let mut record = PromptRecord::new_with_context(
        "proxy".to_string(),
        domain,
        token_model.to_string(),
        original_prompt.clone(),
        original_tokens,
        refined.clone(),
        refined_tokens,
        feedback_json,
    );
    record.governance_id = Some(Uuid::new_v4().to_string());
    record.policy_version = Some(policy.version.clone());
    record.approval_status = Some(approval_status);
    record.content_hash = Some(content_hash);
    let prompt_id = record.id.clone();
    let _ = state.db.insert_prompt(&record).await;

    // Audit log
    let audit = AuditLogEntry {
        id: Uuid::new_v4().to_string(),
        prompt_id: prompt_id.clone(),
        action: "proxy_optimized".to_string(),
        actor: "sovereign-proxy".to_string(),
        detail: serde_json::json!({"policy_version": policy.version}),
        created_at: chrono::Utc::now(),
    };
    let _ = state.db.insert_audit_log(&audit).await;

    tracing::info!(
        "Optimized prompt: {} -> {} tokens ({:.1}% saved)",
        original_tokens,
        refined_tokens,
        record.savings_percentage
    );

    // Rewrite the body with optimized prompt
    rewrite::replace_last_user_message(&mut body_json, &refined, format);
    let rewritten_body = serde_json::to_vec(&body_json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if is_streaming {
        // Stream through — forward SSE stream directly to client
        let (status, resp_headers, resp) = state
            .upstream
            .forward_stream(path, headers, Bytes::from(rewritten_body), is_anthropic)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

        let stream = resp.bytes_stream().map(|result| {
            result.map_err(|e| {
                axum::Error::new(std::io::Error::other(e.to_string()))
            })
        });
        let response_body = Body::from_stream(stream);

        let mut builder =
            Response::builder().status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK));
        for (key, value) in resp_headers.iter() {
            builder = builder.header(key, value);
        }
        builder
            .body(response_body)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    } else {
        // Buffered — capture full response text
        let (status, resp_headers, resp_body) = state
            .upstream
            .forward(path, headers, Bytes::from(rewritten_body), is_anthropic)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

        // Best-effort capture of output text from response
        if let Ok(resp_json) = serde_json::from_slice::<serde_json::Value>(&resp_body) {
            let output_text = extract_response_text(&resp_json, format);
            if let Some(text) = output_text {
                let output_tokens = state
                    .tokenizer
                    .count_for_model(token_model, &text)
                    .unwrap_or_default();
                let _ = state.db.update_output(&prompt_id, &text, output_tokens).await;
                let output_hash = CryptoEngine::compute_output_hash(&text);
                let _ = state
                    .db
                    .update_output_hash(&prompt_id, &output_hash)
                    .await;
            }
        }

        let mut builder =
            Response::builder().status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK));
        for (key, value) in resp_headers.iter() {
            builder = builder.header(key, value);
        }
        builder
            .body(Body::from(resp_body))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
}

async fn forward_passthrough(
    state: &ProxyState,
    headers: HeaderMap,
    body: Bytes,
    path: &str,
    is_anthropic: bool,
    is_streaming: bool,
) -> Result<Response<Body>, (StatusCode, String)> {
    if is_streaming {
        let (status, resp_headers, resp) = state
            .upstream
            .forward_stream(path, headers, body, is_anthropic)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

        let stream = resp.bytes_stream().map(|result| {
            result.map_err(|e| {
                axum::Error::new(std::io::Error::other(e.to_string()))
            })
        });
        let mut builder =
            Response::builder().status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK));
        for (key, value) in resp_headers.iter() {
            builder = builder.header(key, value);
        }
        builder
            .body(Body::from_stream(stream))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    } else {
        let (status, resp_headers, resp_body) = state
            .upstream
            .forward(path, headers, body, is_anthropic)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

        let mut builder =
            Response::builder().status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK));
        for (key, value) in resp_headers.iter() {
            builder = builder.header(key, value);
        }
        builder
            .body(Body::from(resp_body))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
}

/// Extract assistant response text from a non-streaming response body.
fn extract_response_text(body: &serde_json::Value, format: ApiFormat) -> Option<String> {
    match format {
        ApiFormat::Anthropic => {
            // Anthropic: { "content": [{"type": "text", "text": "..."}] }
            let blocks = body.get("content")?.as_array()?;
            let texts: Vec<&str> = blocks
                .iter()
                .filter_map(|b| {
                    if b.get("type")?.as_str()? == "text" {
                        b.get("text")?.as_str()
                    } else {
                        None
                    }
                })
                .collect();
            if texts.is_empty() {
                None
            } else {
                Some(texts.join(""))
            }
        }
        ApiFormat::OpenAI => {
            // OpenAI: { "choices": [{"message": {"content": "..."}}] }
            let choices = body.get("choices")?.as_array()?;
            let first = choices.first()?;
            first
                .get("message")?
                .get("content")?
                .as_str()
                .map(|s| s.to_string())
        }
    }
}
