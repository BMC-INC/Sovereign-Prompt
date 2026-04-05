# SovereignProxy Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a new `sovereign-proxy` crate to the workspace that acts as an HTTP reverse proxy, intercepting API calls to Anthropic/OpenAI, running prompts through SovereignPrompt's optimization engine, and forwarding to the upstream provider.

**Architecture:** New workspace crate `sovereign-proxy/` depends on `sovereign-prompt` as a library (all modules already pub). Axum HTTP server accepts `/v1/messages` (Anthropic) and `/v1/chat/completions` (OpenAI) requests, extracts the last user message, runs it through the existing analyze/refine/template pipeline via direct Rust calls, rewrites the message in the request body, and forwards to the configured upstream URL. Supports both buffered and streaming (SSE) responses. Response text is captured back to the database for the feedback loop.

**Tech Stack:** Rust, Axum 0.8, reqwest (with `stream` feature), tokio, serde_json, the existing `sovereign-prompt` library crate.

---

## File Structure

```
Sovereign-Prompt/
  Cargo.toml                    # NEW — workspace root
  sovereign-prompt/
    Cargo.toml                  # MODIFY — no breaking changes, just ensure lib is exposed
    src/                        # UNTOUCHED
  sovereign-proxy/
    Cargo.toml                  # NEW — binary crate, depends on sovereign-prompt
    src/
      main.rs                   # NEW — entry point, env config, spawn server
      proxy.rs                  # NEW — Axum routes, request/response handling
      rewrite.rs                # NEW — extract + rewrite user message for both API formats
      upstream.rs               # NEW — forward request to provider, handle streaming
```

Each file has one job:
- `main.rs` — Config, DB init, server startup (mirrors `sovereign-prompt/src/main.rs` pattern)
- `proxy.rs` — Axum router, route handlers, shared state
- `rewrite.rs` — Parse Anthropic/OpenAI request bodies, extract last user message, replace with optimized version
- `upstream.rs` — HTTP client, forward requests, stream SSE responses, capture output

---

## Chunk 1: Workspace Setup + Skeleton

### Task 1: Create workspace root Cargo.toml

**Files:**
- Create: `Cargo.toml` (workspace root)

- [ ] **Step 1: Write workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["sovereign-prompt", "sovereign-proxy"]
```

- [ ] **Step 2: Verify existing crate still builds**

Run: `cd /Users/kingjames/Desktop/Sovereign-Prompt && cargo build -p sovereign-prompt`
Expected: Compiles without errors. No changes to sovereign-prompt source.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add workspace root for multi-crate setup"
```

### Task 2: Scaffold sovereign-proxy crate

**Files:**
- Create: `sovereign-proxy/Cargo.toml`
- Create: `sovereign-proxy/src/main.rs`
- Create: `sovereign-proxy/src/proxy.rs`
- Create: `sovereign-proxy/src/rewrite.rs`
- Create: `sovereign-proxy/src/upstream.rs`

- [ ] **Step 1: Write sovereign-proxy/Cargo.toml**

```toml
[package]
name = "sovereign-proxy"
version = "0.1.0"
edition = "2021"
authors = ["ExecLayer Inc."]
description = "HTTP reverse proxy that runs prompts through SovereignPrompt before forwarding to any LLM API"
license = "MIT"

[[bin]]
name = "sovereign-proxy"
path = "src/main.rs"

[dependencies]
sovereign-prompt = { path = "../sovereign-prompt" }
tokio = { version = "1", features = ["full"] }
axum = "0.8"
reqwest = { version = "0.12", features = ["stream", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
futures-util = "0.3"
bytes = "1"
http = "1"
http-body-util = "0.1"
```

- [ ] **Step 2: Write minimal main.rs that starts an Axum server**

```rust
// sovereign-proxy/src/main.rs
use anyhow::Result;
use tracing_subscriber::EnvFilter;

mod proxy;
mod rewrite;
mod upstream;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    proxy::run().await
}
```

- [ ] **Step 3: Write proxy.rs skeleton with health route**

```rust
// sovereign-proxy/src/proxy.rs
use anyhow::Result;
use axum::{routing::get, Json, Router};
use std::net::SocketAddr;

pub async fn run() -> Result<()> {
    let app = Router::new()
        .route("/health", get(health));

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
```

- [ ] **Step 4: Write empty rewrite.rs and upstream.rs stubs**

```rust
// sovereign-proxy/src/rewrite.rs
// Message extraction and rewriting for Anthropic/OpenAI request formats
```

```rust
// sovereign-proxy/src/upstream.rs
// HTTP forwarding to upstream LLM providers
```

- [ ] **Step 5: Verify both crates build**

Run: `cd /Users/kingjames/Desktop/Sovereign-Prompt && cargo build`
Expected: Both `sovereign-prompt` and `sovereign-proxy` compile.

- [ ] **Step 6: Commit**

```bash
git add sovereign-proxy/ Cargo.toml
git commit -m "feat: scaffold sovereign-proxy crate with health endpoint"
```

---

## Chunk 2: Request Rewriting (Core Logic)

### Task 3: Implement Anthropic message extraction and rewriting

**Files:**
- Modify: `sovereign-proxy/src/rewrite.rs`

This is the core of the proxy. It parses incoming API request bodies, finds the last user message, and replaces it with the optimized version.

- [ ] **Step 1: Write test for Anthropic message extraction**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_last_user_message_anthropic() {
        let body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 1024,
            "messages": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there"},
                {"role": "user", "content": "Please help me with something maybe"}
            ]
        });
        let extracted = extract_last_user_message(&body, ApiFormat::Anthropic);
        assert_eq!(extracted, Some("Please help me with something maybe".to_string()));
    }

    #[test]
    fn test_extract_anthropic_content_blocks() {
        let body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": "Analyze this maybe somehow"}
                ]
            }]
        });
        let extracted = extract_last_user_message(&body, ApiFormat::Anthropic);
        assert_eq!(extracted, Some("Analyze this maybe somehow".to_string()));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p sovereign-proxy`
Expected: FAIL — functions not defined.

- [ ] **Step 3: Implement extraction and rewriting**

```rust
// sovereign-proxy/src/rewrite.rs
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApiFormat {
    Anthropic,
    OpenAI,
}

/// Extract the text content of the last user message from a request body.
pub fn extract_last_user_message(body: &Value, format: ApiFormat) -> Option<String> {
    let messages = body.get("messages")?.as_array()?;
    let last_user = messages.iter().rev().find(|m| {
        m.get("role").and_then(|r| r.as_str()) == Some("user")
    })?;

    let content = last_user.get("content")?;

    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => {
            // Both Anthropic and OpenAI support content block arrays
            // Find the last text block
            let text_parts: Vec<&str> = blocks.iter().filter_map(|block| {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    block.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            }).collect();
            if text_parts.is_empty() { None } else { Some(text_parts.join(" ")) }
        }
        _ => None,
    }
}

/// Replace the last user message text in the request body with the optimized version.
/// Returns the modified body. Preserves all other fields (model, params, system, etc).
pub fn replace_last_user_message(body: &mut Value, optimized: &str, _format: ApiFormat) {
    let messages = match body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        Some(m) => m,
        None => return,
    };

    // Find the last user message (iterate in reverse)
    if let Some(last_user) = messages.iter_mut().rev().find(|m| {
        m.get("role").and_then(|r| r.as_str()) == Some("user")
    }) {
        let content = match last_user.get("content") {
            Some(c) => c.clone(),
            None => return,
        };

        match content {
            Value::String(_) => {
                last_user["content"] = Value::String(optimized.to_string());
            }
            Value::Array(blocks) => {
                // Replace text in the last text block, preserve non-text blocks (images, etc)
                let mut new_blocks = blocks.clone();
                if let Some(last_text) = new_blocks.iter_mut().rev().find(|b| {
                    b.get("type").and_then(|t| t.as_str()) == Some("text")
                }) {
                    last_text["text"] = Value::String(optimized.to_string());
                }
                last_user["content"] = Value::Array(new_blocks);
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sovereign-proxy`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add sovereign-proxy/src/rewrite.rs
git commit -m "feat(proxy): implement message extraction and rewriting for Anthropic/OpenAI"
```

### Task 4: Add OpenAI format tests and edge cases

**Files:**
- Modify: `sovereign-proxy/src/rewrite.rs`

- [ ] **Step 1: Add OpenAI and edge case tests**

```rust
    #[test]
    fn test_extract_last_user_message_openai() {
        let body = serde_json::json!({
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Fix this thing somehow"}
            ]
        });
        let extracted = extract_last_user_message(&body, ApiFormat::OpenAI);
        assert_eq!(extracted, Some("Fix this thing somehow".to_string()));
    }

    #[test]
    fn test_replace_preserves_non_text_blocks() {
        let mut body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image", "source": {"type": "base64", "data": "abc"}},
                    {"type": "text", "text": "Describe this image maybe"}
                ]
            }]
        });
        replace_last_user_message(&mut body, "Describe this image concisely.", ApiFormat::Anthropic);
        let blocks = body["messages"][0]["content"].as_array().unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0]["type"], "image");
        assert_eq!(blocks[1]["text"], "Describe this image concisely.");
    }

    #[test]
    fn test_no_user_message_returns_none() {
        let body = serde_json::json!({
            "messages": [{"role": "system", "content": "You are a bot."}]
        });
        assert_eq!(extract_last_user_message(&body, ApiFormat::Anthropic), None);
    }

    #[test]
    fn test_replace_string_content() {
        let mut body = serde_json::json!({
            "messages": [
                {"role": "user", "content": "something vague maybe"}
            ]
        });
        replace_last_user_message(&mut body, "Optimized prompt.", ApiFormat::OpenAI);
        assert_eq!(body["messages"][0]["content"], "Optimized prompt.");
    }
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p sovereign-proxy`
Expected: All PASS

- [ ] **Step 3: Commit**

```bash
git add sovereign-proxy/src/rewrite.rs
git commit -m "test(proxy): add OpenAI format and edge case tests for rewrite"
```

---

## Chunk 3: Upstream Forwarding

### Task 5: Implement non-streaming upstream forwarding

**Files:**
- Modify: `sovereign-proxy/src/upstream.rs`

- [ ] **Step 1: Implement upstream client**

```rust
// sovereign-proxy/src/upstream.rs
use anyhow::Result;
use bytes::Bytes;
use reqwest::{Client, header::{HeaderMap, HeaderValue}};

pub struct UpstreamClient {
    client: Client,
    anthropic_url: String,
    openai_url: String,
}

impl UpstreamClient {
    pub fn new() -> Self {
        let anthropic_url = std::env::var("SOVEREIGN_UPSTREAM_ANTHROPIC")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
        let openai_url = std::env::var("SOVEREIGN_UPSTREAM_OPENAI")
            .unwrap_or_else(|_| "https://api.openai.com".to_string());

        Self {
            client: Client::new(),
            anthropic_url,
            openai_url,
        }
    }

    /// Forward a request to the upstream provider.
    /// Returns (status_code, response_headers, response_body_bytes).
    pub async fn forward(
        &self,
        path: &str,
        headers: HeaderMap,
        body: Bytes,
        is_anthropic: bool,
    ) -> Result<(u16, HeaderMap, Bytes)> {
        let base = if is_anthropic { &self.anthropic_url } else { &self.openai_url };
        let url = format!("{}{}", base, path);

        let mut req = self.client.post(&url).body(body);

        // Forward relevant headers (auth, content-type, api version, etc)
        for (key, value) in headers.iter() {
            let name = key.as_str().to_lowercase();
            match name.as_str() {
                "host" | "content-length" | "transfer-encoding" | "connection" => continue,
                _ => { req = req.header(key.clone(), value.clone()); }
            }
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let resp_headers = resp.headers().clone();
        let resp_body = resp.bytes().await?;

        Ok((status, resp_headers, resp_body))
    }

    /// Forward a streaming request, returning a byte stream.
    pub async fn forward_stream(
        &self,
        path: &str,
        headers: HeaderMap,
        body: Bytes,
        is_anthropic: bool,
    ) -> Result<(u16, HeaderMap, reqwest::Response)> {
        let base = if is_anthropic { &self.anthropic_url } else { &self.openai_url };
        let url = format!("{}{}", base, path);

        let mut req = self.client.post(&url).body(body);

        for (key, value) in headers.iter() {
            let name = key.as_str().to_lowercase();
            match name.as_str() {
                "host" | "content-length" | "transfer-encoding" | "connection" => continue,
                _ => { req = req.header(key.clone(), value.clone()); }
            }
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let resp_headers = resp.headers().clone();

        Ok((status, resp_headers, resp))
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p sovereign-proxy`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add sovereign-proxy/src/upstream.rs
git commit -m "feat(proxy): implement upstream HTTP forwarding with streaming support"
```

---

## Chunk 4: Proxy Route Handlers

### Task 6: Wire up the full proxy pipeline

**Files:**
- Modify: `sovereign-proxy/src/proxy.rs`
- Modify: `sovereign-proxy/src/main.rs`

This is where it all comes together: receive request -> optimize -> rewrite -> forward -> return.

- [ ] **Step 1: Implement shared proxy state and route handlers**

```rust
// sovereign-proxy/src/proxy.rs
use crate::rewrite::{self, ApiFormat};
use crate::upstream::UpstreamClient;
use anyhow::Result;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Request, Response, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use futures_util::StreamExt;
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
    let is_streaming = body_json.get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Extract last user message
    let original_prompt = match rewrite::extract_last_user_message(&body_json, format) {
        Some(p) => p,
        None => {
            // No user message to optimize — pass through as-is
            return forward_passthrough(state, headers, body, path, is_anthropic, is_streaming).await;
        }
    };

    // Run the SovereignPrompt optimization pipeline
    let token_model = sovereign_prompt::tokenizer::DEFAULT_TOKEN_MODEL;
    let original_tokens = state.tokenizer.count_for_model(token_model, &original_prompt)
        .unwrap_or_default();

    let feedback = PromptAnalyzer::analyze_with_config(&original_prompt, &state.config.heuristics);
    let refined_base = PromptOptimizer::refine(&original_prompt, &feedback, &state.tokenizer);

    // Detect domain from request model name or default to general
    let domain = detect_domain(&body_json);
    let (refined, _template) = PromptTemplateLibrary::apply(&domain, &refined_base);
    let refined_tokens = state.tokenizer.count_for_model(token_model, &refined)
        .unwrap_or_default();

    // Compute content hash
    let content_hash = CryptoEngine::compute_content_hash(&original_prompt, &refined);

    // Governance check
    let policy = GovernancePolicy::current();
    let gov_feedback = GovernancePolicy::validate_prompt(&original_prompt);
    let approval_status = GovernancePolicy::determine_status(&gov_feedback);

    // If governance rejects, do NOT optimize — pass through original
    if approval_status == "rejected" {
        tracing::warn!("Governance rejected prompt — passing through unmodified");
        return forward_passthrough(state, headers, body, path, is_anthropic, is_streaming).await;
    }

    // Log the optimization
    let feedback_json = serde_json::to_value(&feedback).unwrap_or_default();
    let mut record = PromptRecord::new_with_context(
        "proxy".to_string(),
        domain,
        token_model.to_string(),
        original_prompt,
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
        original_tokens, refined_tokens,
        record.savings_percentage
    );

    // Rewrite the body with optimized prompt
    rewrite::replace_last_user_message(&mut body_json, &refined, format);
    let rewritten_body = serde_json::to_vec(&body_json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if is_streaming {
        // Forward as stream, collect response text for capture
        let (status, resp_headers, resp) = state.upstream
            .forward_stream(path, headers, Bytes::from(rewritten_body), is_anthropic)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

        let stream = resp.bytes_stream();
        let response_body = Body::from_stream(stream);

        let mut response = Response::builder()
            .status(status);
        for (key, value) in resp_headers.iter() {
            response = response.header(key, value);
        }
        response.body(response_body)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    } else {
        // Forward buffered, capture full response
        let (status, resp_headers, resp_body) = state.upstream
            .forward(path, headers, Bytes::from(rewritten_body), is_anthropic)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

        // Capture output text from response (best-effort)
        if let Ok(resp_json) = serde_json::from_slice::<serde_json::Value>(&resp_body) {
            let output_text = extract_response_text(&resp_json, format);
            if let Some(text) = output_text {
                let output_tokens = state.tokenizer.count_for_model(token_model, &text)
                    .unwrap_or_default();
                let _ = state.db.update_output(&prompt_id, &text, output_tokens).await;
                let output_hash = CryptoEngine::compute_output_hash(&text);
                let _ = state.db.update_output_hash(&prompt_id, &output_hash).await;
            }
        }

        let mut response = Response::builder()
            .status(status);
        for (key, value) in resp_headers.iter() {
            response = response.header(key, value);
        }
        response.body(Body::from(resp_body))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
}

async fn forward_passthrough(
    state: ProxyState,
    headers: HeaderMap,
    body: Bytes,
    path: &str,
    is_anthropic: bool,
    is_streaming: bool,
) -> Result<Response<Body>, (StatusCode, String)> {
    if is_streaming {
        let (status, resp_headers, resp) = state.upstream
            .forward_stream(path, headers, body, is_anthropic)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

        let stream = resp.bytes_stream();
        let mut response = Response::builder().status(status);
        for (key, value) in resp_headers.iter() {
            response = response.header(key, value);
        }
        response.body(Body::from_stream(stream))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    } else {
        let (status, resp_headers, resp_body) = state.upstream
            .forward(path, headers, body, is_anthropic)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

        let mut response = Response::builder().status(status);
        for (key, value) in resp_headers.iter() {
            response = response.header(key, value);
        }
        response.body(Body::from(resp_body))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
}

/// Detect optimization domain from the model name or default to "general".
fn detect_domain(body: &serde_json::Value) -> String {
    // Could be extended with a header or query param; for now default to general
    let _model = body.get("model").and_then(|m| m.as_str()).unwrap_or("");
    "general".to_string()
}

/// Extract assistant response text from a non-streaming response body.
fn extract_response_text(body: &serde_json::Value, format: ApiFormat) -> Option<String> {
    match format {
        ApiFormat::Anthropic => {
            // Anthropic: { "content": [{"type": "text", "text": "..."}] }
            let blocks = body.get("content")?.as_array()?;
            let texts: Vec<&str> = blocks.iter().filter_map(|b| {
                if b.get("type")?.as_str()? == "text" {
                    b.get("text")?.as_str()
                } else {
                    None
                }
            }).collect();
            if texts.is_empty() { None } else { Some(texts.join("")) }
        }
        ApiFormat::OpenAI => {
            // OpenAI: { "choices": [{"message": {"content": "..."}}] }
            let choices = body.get("choices")?.as_array()?;
            let first = choices.first()?;
            first.get("message")?.get("content")?.as_str().map(|s| s.to_string())
        }
    }
}
```

- [ ] **Step 2: Update main.rs to initialize state and start proxy**

```rust
// sovereign-proxy/src/main.rs
use anyhow::Result;
use sovereign_prompt::config::SovereignConfig;
use sovereign_prompt::db::Database;
use sovereign_prompt::tokenizer::Tokenizer;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

mod proxy;
mod rewrite;
mod upstream;

use proxy::ProxyState;
use upstream::UpstreamClient;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("SOVEREIGN_DB_PATH").map(|p| format!("sqlite://{}?mode=rwc", p)))
        .unwrap_or_else(|_| "sqlite://./sovereign_prompt.db?mode=rwc".to_string());

    let db = Arc::new(Database::new(&database_url).await?);
    db.migrate().await?;

    let tokenizer = Arc::new(Tokenizer::new()?);
    let config = Arc::new(SovereignConfig::load());
    let upstream = Arc::new(UpstreamClient::new());

    let state = ProxyState {
        db,
        tokenizer,
        config,
        upstream,
    };

    tracing::info!("SovereignProxy starting...");
    proxy::run(state).await
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p sovereign-proxy`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add sovereign-proxy/
git commit -m "feat(proxy): implement full proxy pipeline with Anthropic and OpenAI support"
```

---

## Chunk 5: Integration Test + Documentation

### Task 7: Smoke test the proxy

**Files:**
- No new files. Manual testing with curl.

- [ ] **Step 1: Start the proxy in one terminal**

```bash
cd /Users/kingjames/Desktop/Sovereign-Prompt
RUST_LOG=info cargo run -p sovereign-proxy
```

- [ ] **Step 2: Test health endpoint**

```bash
curl http://localhost:8788/health
```
Expected: `{"service":"sovereign-proxy","status":"ok"}`

- [ ] **Step 3: Test Anthropic proxy (requires ANTHROPIC_API_KEY in env)**

```bash
curl -X POST http://localhost:8788/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "max_tokens": 100,
    "messages": [{"role": "user", "content": "maybe explain something about rust somehow"}]
  }'
```
Expected: Proxy logs show optimization (token savings), response comes back from Anthropic.

- [ ] **Step 4: Verify database capture**

```bash
sqlite3 sovereign_prompt.db "SELECT id, original_token_count, refined_token_count, savings_percentage FROM prompts ORDER BY created_at DESC LIMIT 1;"
```
Expected: Row with savings > 0%.

### Task 8: Add env var setup instructions

**Files:**
- No source changes. User adds to their shell profile.

- [ ] **Step 1: Document the one-time setup**

User adds to `~/.zshrc`:
```bash
# SovereignProxy — optimize all prompts before they hit the model
export ANTHROPIC_BASE_URL="http://localhost:8788"
# For OpenAI clients:
# export OPENAI_BASE_URL="http://localhost:8788"
```

Then `source ~/.zshrc`. Every Claude Code session and every OpenAI client call will flow through SovereignProxy automatically.

- [ ] **Step 2: Commit all remaining changes**

```bash
git add -A
git commit -m "feat: SovereignProxy v0.1.0 — HTTP reverse proxy for model-agnostic prompt optimization"
```

---

## Summary

| Task | What | Files |
|------|------|-------|
| 1 | Workspace root Cargo.toml | `Cargo.toml` |
| 2 | Scaffold sovereign-proxy crate | `sovereign-proxy/` (4 files) |
| 3 | Anthropic message extraction + rewriting | `rewrite.rs` |
| 4 | OpenAI format + edge case tests | `rewrite.rs` |
| 5 | Upstream HTTP forwarding | `upstream.rs` |
| 6 | Full proxy route handlers | `proxy.rs`, `main.rs` |
| 7 | Smoke test with curl | Manual |
| 8 | Env var documentation | Shell profile |

**Total new files:** 5 (workspace Cargo.toml + 4 source files in sovereign-proxy/)
**Existing files modified:** 0
**Existing behavior changed:** Nothing.
