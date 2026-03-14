use crate::analyzer::PromptAnalyzer;
use crate::config::{InjectionMode, SovereignConfig};
use crate::crypto::CryptoEngine;
use crate::db::Database;
use crate::governance::GovernancePolicy;
use crate::optimizer::PromptOptimizer;
use crate::templates::PromptTemplateLibrary;
use crate::tokenizer::{Tokenizer, DEFAULT_TOKEN_MODEL};
use crate::types::{AuditLogEntry, ModelTokenSummary, OptimizeResponse, PromptRecord};
use anyhow::Result;
use chrono::Utc;
use rmcp::model::*;
use rmcp::{ServerHandler, ServiceExt};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

type JsonObject = serde_json::Map<String, serde_json::Value>;

#[derive(Clone)]
pub struct SovereignPromptServer {
    db: Arc<Database>,
    tokenizer: Arc<Tokenizer>,
    crypto: Arc<CryptoEngine>,
    config: Arc<SovereignConfig>,
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
                            },
                            "explain_mode": {
                                "type": "boolean",
                                "description": "When true, returns detailed heuristic explanations for each of the 9 checks"
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
            Tool::new(
                "governance_check",
                "Validate a stored prompt against governance policies.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "prompt_id": {
                                "type": "string",
                                "description": "The prompt ID to validate"
                            }
                        },
                        "required": ["prompt_id"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "governance_approve",
                "Approve or reject a prompt optimization with governance tracking.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "prompt_id": {
                                "type": "string",
                                "description": "The prompt ID to approve/reject"
                            },
                            "actor": {
                                "type": "string",
                                "description": "The user or system approving/rejecting"
                            },
                            "status": {
                                "type": "string",
                                "description": "New status: approved or rejected"
                            }
                        },
                        "required": ["prompt_id", "actor", "status"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "get_audit_trail",
                "Retrieve the governance audit trail for a prompt.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "prompt_id": {
                                "type": "string",
                                "description": "The prompt ID to get audit trail for"
                            }
                        },
                        "required": ["prompt_id"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "sign_optimization",
                "Cryptographically sign a prompt optimization record.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "prompt_id": {
                                "type": "string",
                                "description": "The prompt ID to sign"
                            }
                        },
                        "required": ["prompt_id"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "verify_signature",
                "Verify the cryptographic signature and hash chain of a prompt record.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "prompt_id": {
                                "type": "string",
                                "description": "The prompt ID to verify"
                            }
                        },
                        "required": ["prompt_id"]
                    }))
                    .unwrap(),
                ),
            ),
            Tool::new(
                "savings_report",
                "Generate a cost savings report with token metrics, cost estimates across models, and daily trends.",
                Arc::new(
                    serde_json::from_value::<JsonObject>(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "user_id": {
                                "type": "string",
                                "description": "User ID to generate report for"
                            },
                            "period": {
                                "type": "string",
                                "description": "Time period: 7d, 30d, 90d, or all (default: 30d)"
                            }
                        },
                        "required": ["user_id"]
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

        let explain_mode = args
            .get("explain_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

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

        // Check injection mode
        let working_prompt = match self.config.injection.mode {
            InjectionMode::Reject => {
                // Check for injection patterns before proceeding
                let injection_feedback =
                    PromptAnalyzer::analyze_with_config(&prompt, &self.config.heuristics);
                let has_injection = injection_feedback
                    .iter()
                    .any(|f| f.category == "Security");
                if has_injection {
                    return Err(ErrorData::invalid_params(
                        "Prompt rejected: injection pattern detected. Injection mode is set to 'reject'.",
                        None,
                    ));
                }
                prompt.clone()
            }
            InjectionMode::Rewrite => {
                PromptOptimizer::strip_injection_patterns(&prompt)
            }
            InjectionMode::Warn => prompt.clone(),
        };

        let original_tokens = self
            .tokenizer
            .count_for_model(&token_model, &prompt)
            .unwrap_or_default();

        let (feedback, explanations) = if explain_mode {
            PromptAnalyzer::analyze_explained(&working_prompt, &self.config.heuristics)
        } else {
            let fb = PromptAnalyzer::analyze_with_config(&working_prompt, &self.config.heuristics);
            (fb, Vec::new())
        };

        let refined_base = PromptOptimizer::refine(&working_prompt, &feedback, &self.tokenizer);
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

        // Compute content hash
        let content_hash = CryptoEngine::compute_content_hash(&prompt, &refined);

        // Run governance check
        let policy = GovernancePolicy::current();
        let gov_feedback = GovernancePolicy::validate_prompt(&prompt);
        let approval_status = GovernancePolicy::determine_status(&gov_feedback);
        let governance_id = Uuid::new_v4().to_string();

        let mut record = PromptRecord::new_with_context(
            user_id.clone(),
            domain.clone(),
            token_model.clone(),
            prompt.clone(),
            original_tokens,
            refined.clone(),
            refined_tokens,
            feedback_json,
        );

        record.governance_id = Some(governance_id.clone());
        record.policy_version = Some(policy.version.clone());
        record.approval_status = Some(approval_status.clone());
        record.content_hash = Some(content_hash.clone());

        let prompt_id = record.id.clone();

        self.db
            .insert_prompt(&record)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        // Write audit log entry
        let audit = AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            prompt_id: prompt_id.clone(),
            action: "created".to_string(),
            actor: user_id,
            detail: serde_json::json!({
                "policy_version": policy.version,
                "approval_status": approval_status,
                "content_hash": content_hash,
            }),
            created_at: Utc::now(),
        };
        let _ = self.db.insert_audit_log(&audit).await;

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
            content_hash: Some(content_hash),
            governance_status: Some(approval_status),
        };

        let mut json_value = serde_json::to_value(&response)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if explain_mode {
            json_value["heuristic_explanations"] =
                serde_json::to_value(&explanations).unwrap_or_default();
        }

        let json = serde_json::to_string_pretty(&json_value)
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

        // Compute and store output hash
        let output_hash = CryptoEngine::compute_output_hash(&output);
        let _ = self.db.update_output_hash(&prompt_id, &output_hash).await;

        // Audit log
        let audit = AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            prompt_id: prompt_id.clone(),
            action: "captured".to_string(),
            actor: "system".to_string(),
            detail: serde_json::json!({
                "output_hash": output_hash,
                "output_token_count": output_tokens,
            }),
            created_at: Utc::now(),
        };
        let _ = self.db.insert_audit_log(&audit).await;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "status": "captured",
                "prompt_id": prompt_id,
                "token_model": token_model,
                "output_token_count": output_tokens,
                "output_hash": output_hash
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

    async fn handle_governance_check(&self, args: &JsonObject) -> Result<CallToolResult, ErrorData> {
        let prompt_id = args
            .get("prompt_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'prompt_id' parameter", None))?;

        let record = self
            .db
            .get_prompt_by_id(prompt_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
            .ok_or_else(|| ErrorData::invalid_params("prompt not found", None))?;

        let policy = GovernancePolicy::current();
        let gov_feedback = GovernancePolicy::validate_prompt(&record.original_prompt);
        let status = GovernancePolicy::determine_status(&gov_feedback);

        let json = serde_json::json!({
            "prompt_id": prompt_id,
            "policy_version": policy.version,
            "governance_feedback": gov_feedback,
            "recommended_status": status,
            "current_status": record.approval_status,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?,
        )]))
    }

    async fn handle_governance_approve(
        &self,
        args: &JsonObject,
    ) -> Result<CallToolResult, ErrorData> {
        let prompt_id = args
            .get("prompt_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'prompt_id' parameter", None))?
            .to_string();

        let actor = args
            .get("actor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'actor' parameter", None))?
            .to_string();

        let status = args
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'status' parameter", None))?
            .to_string();

        if status != "approved" && status != "rejected" {
            return Err(ErrorData::invalid_params(
                "status must be 'approved' or 'rejected'",
                None,
            ));
        }

        // Verify prompt exists
        self.db
            .get_prompt_by_id(&prompt_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
            .ok_or_else(|| ErrorData::invalid_params("prompt not found", None))?;

        self.db
            .update_approval_status(&prompt_id, &status)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let audit = AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            prompt_id: prompt_id.clone(),
            action: status.clone(),
            actor: actor.clone(),
            detail: serde_json::json!({
                "new_status": status,
            }),
            created_at: Utc::now(),
        };
        self.db
            .insert_audit_log(&audit)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "prompt_id": prompt_id,
                "status": status,
                "actor": actor,
            })
            .to_string(),
        )]))
    }

    async fn handle_get_audit_trail(
        &self,
        args: &JsonObject,
    ) -> Result<CallToolResult, ErrorData> {
        let prompt_id = args
            .get("prompt_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'prompt_id' parameter", None))?;

        let trail = self
            .db
            .get_audit_trail(prompt_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&trail)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    async fn handle_sign_optimization(
        &self,
        args: &JsonObject,
    ) -> Result<CallToolResult, ErrorData> {
        let prompt_id = args
            .get("prompt_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'prompt_id' parameter", None))?
            .to_string();

        let record = self
            .db
            .get_prompt_by_id(&prompt_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
            .ok_or_else(|| ErrorData::invalid_params("prompt not found", None))?;

        let content_hash = record
            .content_hash
            .ok_or_else(|| ErrorData::internal_error("no content hash on record", None))?;

        let signature = self.crypto.sign(&content_hash);
        let signed_at = Utc::now();

        self.db
            .update_signature(&prompt_id, &signature, &signed_at)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let audit = AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            prompt_id: prompt_id.clone(),
            action: "signed".to_string(),
            actor: "system".to_string(),
            detail: serde_json::json!({
                "content_hash": content_hash,
                "signature": signature,
            }),
            created_at: Utc::now(),
        };
        let _ = self.db.insert_audit_log(&audit).await;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "prompt_id": prompt_id,
                "content_hash": content_hash,
                "signature": signature,
                "signed_at": signed_at.to_rfc3339(),
            })
            .to_string(),
        )]))
    }

    async fn handle_verify_signature(
        &self,
        args: &JsonObject,
    ) -> Result<CallToolResult, ErrorData> {
        let prompt_id = args
            .get("prompt_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'prompt_id' parameter", None))?;

        let record = self
            .db
            .get_prompt_by_id(prompt_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
            .ok_or_else(|| ErrorData::invalid_params("prompt not found", None))?;

        let content_hash = record
            .content_hash
            .as_deref()
            .ok_or_else(|| ErrorData::internal_error("no content hash on record", None))?;

        let signature = record
            .signature
            .as_deref()
            .ok_or_else(|| ErrorData::internal_error("record is not signed", None))?;

        let sig_valid = self.crypto.verify(content_hash, signature);
        let chain_valid = CryptoEngine::verify_hash_chain(
            &record.original_prompt,
            &record.refined_prompt,
            content_hash,
            record.output.as_deref(),
            record.output_hash.as_deref(),
        );

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "prompt_id": prompt_id,
                "signature_valid": sig_valid,
                "hash_chain_valid": chain_valid,
                "content_hash": content_hash,
                "output_hash": record.output_hash,
                "signed_at": record.signed_at.map(|dt| dt.to_rfc3339()),
            })
            .to_string(),
        )]))
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

    async fn handle_savings_report(&self, args: &JsonObject) -> Result<CallToolResult, ErrorData> {
        let user_id = args
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::invalid_params("missing 'user_id' parameter", None))?
            .to_string();

        let period = args
            .get("period")
            .and_then(|v| v.as_str())
            .unwrap_or("30d")
            .to_string();

        let report = self
            .db
            .get_savings_report(&user_id, &period, None)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&report)
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
            "governance_check" => self.handle_governance_check(&args).await,
            "governance_approve" => self.handle_governance_approve(&args).await,
            "get_audit_trail" => self.handle_get_audit_trail(&args).await,
            "sign_optimization" => self.handle_sign_optimization(&args).await,
            "verify_signature" => self.handle_verify_signature(&args).await,
            "savings_report" => self.handle_savings_report(&args).await,
            _ => Err(ErrorData::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            )),
        }
    }
}

fn build_server(db: Arc<Database>, config: SovereignConfig) -> SovereignPromptServer {
    let tokenizer = Tokenizer::new().expect("failed to initialize tokenizer");
    let hmac_key = std::env::var("SOVEREIGN_HMAC_KEY")
        .unwrap_or_else(|_| "sovereign-prompt-dev-key-change-in-production".to_string());
    let crypto = Arc::new(CryptoEngine::new(hmac_key.as_bytes()));

    SovereignPromptServer {
        db,
        tokenizer: Arc::new(tokenizer),
        crypto,
        config: Arc::new(config),
    }
}

pub async fn run(db: Arc<Database>) -> Result<()> {
    let config = SovereignConfig::load();
    let server = build_server(db, config);

    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}

pub async fn run_sse(db: Arc<Database>, bind: SocketAddr) -> Result<()> {
    let config = SovereignConfig::load();
    let server = build_server(db, config);

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
