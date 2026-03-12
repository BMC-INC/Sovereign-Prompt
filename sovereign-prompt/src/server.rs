use crate::analyzer::PromptAnalyzer;
use crate::db::Database;
use crate::optimizer::PromptOptimizer;
use crate::tokenizer::Tokenizer;
use crate::types::{OptimizeResponse, PromptRecord};
use anyhow::Result;
use rmcp::model::*;
use rmcp::{ServerHandler, ServiceExt};
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
        ]
    }

    async fn handle_optimize_prompt(
        &self,
        args: &JsonObject,
    ) -> Result<CallToolResult, ErrorData> {
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

        let original_tokens = self.tokenizer.count(&prompt);
        let feedback = PromptAnalyzer::analyze(&prompt);
        let refined = PromptOptimizer::refine(&prompt, &feedback, &self.tokenizer);
        let refined_tokens = self.tokenizer.count(&refined);
        let variants = PromptOptimizer::generate_variants(&refined, &self.tokenizer);

        let savings = if original_tokens > 0 {
            ((original_tokens - refined_tokens) as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        let feedback_json = serde_json::to_value(&feedback).unwrap_or_default();

        let record = PromptRecord::new(
            user_id,
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
            original_prompt: prompt,
            original_token_count: original_tokens,
            refined_prompt: refined,
            refined_token_count: refined_tokens,
            savings_percentage: savings,
            feedback,
            variants,
        };

        let json = serde_json::to_string_pretty(&response)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    async fn handle_capture_output(
        &self,
        args: &JsonObject,
    ) -> Result<CallToolResult, ErrorData> {
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

        let output_tokens = self.tokenizer.count(&output);

        self.db
            .update_output(&prompt_id, &output, output_tokens)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "status": "captured",
                "prompt_id": prompt_id,
                "output_token_count": output_tokens
            })
            .to_string(),
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
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
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
            _ => Err(ErrorData::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            )),
        }
    }
}

pub async fn run(db: Database) -> Result<()> {
    let tokenizer = Tokenizer::new()?;

    let server = SovereignPromptServer {
        db: Arc::new(db),
        tokenizer: Arc::new(tokenizer),
    };

    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
