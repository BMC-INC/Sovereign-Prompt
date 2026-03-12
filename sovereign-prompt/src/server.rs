use crate::analyzer::PromptAnalyzer;
use crate::db::Database;
use crate::optimizer::PromptOptimizer;
use crate::templates::PromptTemplateLibrary;
use crate::tokenizer::{Tokenizer, DEFAULT_TOKEN_MODEL};
use crate::types::{ModelTokenSummary, OptimizeResponse, PromptRecord};
use anyhow::Result;
use rmcp::model::*;
use rmcp::{ServerHandler, ServiceExt};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

type JsonObject = serde_json::Map<String, serde_json::Value>;

#[derive(Clone)]
pub struct SovereignPromptServer {
    db: Arc<Database>,
    tokenizer: Arc<Tokenizer>,
}

impl SovereignPromptServer {
    fn build_tool_list() -> Vec<Tool> {
        vec![
            Tool::new(
                "optimize_prompt",
                "Optimize a prompt. Returns refined version, token savings, feedback, and variants.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "user_id": {
                                "type": "string",
                                "description": "User ID (optional, defaults to anonymous)"
                            },
                            "domain": {
                                "type": "string",
                                "description": "Optimization domain template (general/backend/frontend/data/security/product/documentation)"
                            },
                            "token_model": {
                                "type": "string",
                                "description": "Tokenizer model to use for primary counts: cl100k_base, o200k_base, p50k_base, r50k_base"
                            },
                            "prompt": {
                                "type": "string",
                                "description": "The prompt to optimize"
                            }
                        },
                        "required": ["prompt"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "capture_output",
                "Capture the AI output for a previously optimized prompt to enable learning.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "prompt_id": {
                                "type": "string",
                                "description": "The prompt ID returned from optimize_prompt"
                            },
                            "output": {
                                "type": "string",
                                "description": "The AI output text to capture"
                            },
                            "token_model": {
                                "type": "string",
                                "description": "Optional tokenizer model override for output token count"
                            }
                        },
                        "required": ["prompt_id", "output"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "get_stats",
                "Get optimization stats for a user — total prompts, tokens saved, average savings.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "user_id": {
                                "type": "string",
                                "description": "User ID to fetch stats for"
                            }
                        },
                        "required": ["user_id"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "get_history",
                "Get recent prompt history for a user.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "user_id": {
                                "type": "string",
                                "description": "User ID"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Number of recent prompts to return (max 50)"
                            }
                        },
                        "required": ["user_id"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "list_templates",
                "List available per-domain prompt optimization templates.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "count_tokens",
                "Count tokens for text across supported models or a single specified model.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Text to tokenize"
                            },
                            "model": {
                                "type": "string",
                                "description": "Optional model: cl100k_base, o200k_base, p50k_base, r50k_base"
                            }
                        },
                        "required": ["text"]
                    }))
                    .unwrap(),
                ),
            ),
        ]
    }

    async fn handle_optimize_prompt(&self, args: &JsonObject) -> Result<CallToolResult, ErrorData> {
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'prompt' parameter", None))?
            .to_string();

        let user_id = args
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("anonymous")
            .to_string();

        let domain =
            PromptTemplateLibrary::normalize_domain(args.get("domain").and_then(|v| v.as_str()));
        let token_model = args
            .get("token_model")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_TOKEN_MODEL)
            .to_string();

        if !self.tokenizer.is_supported_model(&token_model) {
            return Err(ErrorData::invalid_params(
                format!(
                    "unsupported token_model '{}'. Supported: {}",
                    token_model,
                    self.tokenizer.available_models().join(", ")
                ),
                None,
            ));
        }

        let original_tokens = self
            .tokenizer
            .count_for_model(&token_model, &prompt)
            .unwrap_or_default();
        let feedback = PromptAnalyzer::analyze(&prompt);
        let refined_base = PromptOptimizer::refine(&prompt, &feedback, &self.tokenizer);
        let (refined, template) = PromptTemplateLibrary::apply(&domain, &refined_base);
        let refined_tokens = self
            .tokenizer
            .count_for_model(&token_model, &refined)
            .unwrap_or_default();
        let variants =
            PromptOptimizer::generate_variants_with_model(&refined, &self.tokenizer, &token_model);

        let original_by_model = self.tokenizer.count_across_models(&prompt);
        let refined_by_model = self.tokenizer.count_across_models(&refined);
        let token_counts_by_model: BTreeMap<String, ModelTokenSummary> = original_by_model
            .into_iter()
            .map(|(model, original)| {
                let refined = refined_by_model.get(&model).copied().unwrap_or_default();
                (
                    model,
                    ModelTokenSummary {
                        original_token_count: original,
                        refined_token_count: refined,
                    },
                )
            })
            .collect();

        let savings = if original_tokens > 0 {
            ((original_tokens - refined_tokens) as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        let feedback_json = serde_json::to_value(&feedback).unwrap_or_default();

        let record = PromptRecord::new_with_context(
            user_id,
            domain.clone(),
            token_model.clone(),
            prompt.clone(),
            original_tokens,
            refined.clone(),
            refined_tokens,
            feedback_json,
        );

        let prompt_id = record.id.clone();

        self.db
            .insert_prompt(&record)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response = OptimizeResponse {
            prompt_id,
            domain,
            token_model,
            original_prompt: prompt,
            original_token_count: original_tokens,
            refined_prompt: refined,
            refined_token_count: refined_tokens,
            savings_percentage: savings,
            token_counts_by_model,
            template,
            feedback,
            variants,
        };

        let json = serde_json::to_string_pretty(&response)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    async fn handle_capture_output(&self, args: &JsonObject) -> Result<CallToolResult, ErrorData> {
        let prompt_id = args
            .get("prompt_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'prompt_id' parameter", None))?
            .to_string();

        let output = args
            .get("output")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'output' parameter", None))?
            .to_string();

        let token_model = args
            .get("token_model")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_TOKEN_MODEL)
            .to_string();

        if !self.tokenizer.is_supported_model(&token_model) {
            return Err(ErrorData::invalid_params(
                format!(
                    "unsupported token_model '{}'. Supported: {}",
                    token_model,
                    self.tokenizer.available_models().join(", ")
                ),
                None,
            ));
        }

        let output_tokens = self
            .tokenizer
            .count_for_model(&token_model, &output)
            .unwrap_or_default();

        self.db
            .update_output(&prompt_id, &output, output_tokens)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "status": "captured",
                "prompt_id": prompt_id,
                "token_model": token_model,
                "output_token_count": output_tokens
            })
            .to_string(),
        )]))
    }

    async fn handle_list_templates(&self, _args: &JsonObject) -> Result<CallToolResult, ErrorData> {
        let json = serde_json::json!({
            "domains": PromptTemplateLibrary::available_domains(),
        });
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?,
        )]))
    }

    async fn handle_count_tokens(&self, args: &JsonObject) -> Result<CallToolResult, ErrorData> {
        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'text' parameter", None))?;

        let model = args.get("model").and_then(|v| v.as_str());

        let response = if let Some(model) = model {
            if !self.tokenizer.is_supported_model(model) {
                return Err(ErrorData::invalid_params(
                    format!(
                        "unsupported model '{}'. Supported: {}",
                        model,
                        self.tokenizer.available_models().join(", ")
                    ),
                    None,
                ));
            }

            serde_json::json!({
                "model": model,
                "token_count": self.tokenizer.count_for_model(model, text).unwrap_or_default(),
            })
        } else {
            serde_json::json!({
                "token_counts": self.tokenizer.count_across_models(text),
            })
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?,
        )]))
    }

    async fn handle_get_stats(&self, args: &JsonObject) -> Result<CallToolResult, ErrorData> {
        let user_id = args
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'user_id' parameter", None))?
            .to_string();

        let stats = self
            .db
            .get_user_stats(&user_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&stats)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    async fn handle_get_history(&self, args: &JsonObject) -> Result<CallToolResult, ErrorData> {
        let user_id = args
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'user_id' parameter", None))?
            .to_string();

        let limit = args
            .get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(10)
            .min(50);

        let history = self
            .db
            .get_recent_prompts(&user_id, limit)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&history)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

impl ServerHandler for SovereignPromptServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "SovereignPrompt".to_string(),
                version: "0.1.0".to_string(),
            },
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: Self::build_tool_list(),
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = request.arguments.unwrap_or_default();
        match request.name.as_ref() {
            "optimize_prompt" => self.handle_optimize_prompt(&args).await,
            "capture_output" => self.handle_capture_output(&args).await,
            "get_stats" => self.handle_get_stats(&args).await,
            "get_history" => self.handle_get_history(&args).await,
            "list_templates" => self.handle_list_templates(&args).await,
            "count_tokens" => self.handle_count_tokens(&args).await,
            _ => Err(ErrorData::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            )),
        }
    }
}

pub async fn run(db: Arc<Database>) -> Result<()> {
    let tokenizer = Tokenizer::new()?;

    let server = SovereignPromptServer {
        db,
        tokenizer: Arc::new(tokenizer),
    };

    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}

pub async fn run_sse(db: Arc<Database>, bind: SocketAddr) -> Result<()> {
    let tokenizer = Tokenizer::new()?;

    let server = SovereignPromptServer {
        db,
        tokenizer: Arc::new(tokenizer),
    };

    let ct = rmcp::transport::sse_server::SseServer::serve(bind)
        .await?
        .with_service(move || server.clone());

    tracing::info!("SovereignPrompt MCP SSE server listening on {}", bind);
    tracing::info!("SSE endpoint: http://{}/sse", bind);

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    tracing::info!("SSE transport shutdown complete.");
    Ok(())
}
