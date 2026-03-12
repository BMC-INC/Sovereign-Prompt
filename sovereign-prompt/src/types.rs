use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRecord {
    pub id: String,
    pub user_id: String,
    pub original_prompt: String,
    pub original_token_count: i64,
    pub refined_prompt: String,
    pub refined_token_count: i64,
    pub savings_percentage: f64,
    pub analysis_feedback: serde_json::Value,
    pub output: Option<String>,
    pub output_token_count: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl PromptRecord {
    pub fn new(
        user_id: String,
        original_prompt: String,
        original_token_count: i64,
        refined_prompt: String,
        refined_token_count: i64,
        analysis_feedback: serde_json::Value,
    ) -> Self {
        let savings = if original_token_count > 0 {
            ((original_token_count - refined_token_count) as f64
                / original_token_count as f64)
                * 100.0
        } else {
            0.0
        };

        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            original_prompt,
            original_token_count,
            refined_prompt,
            refined_token_count,
            savings_percentage: savings,
            analysis_feedback,
            output: None,
            output_token_count: None,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeResponse {
    pub prompt_id: String,
    pub original_prompt: String,
    pub original_token_count: i64,
    pub refined_prompt: String,
    pub refined_token_count: i64,
    pub savings_percentage: f64,
    pub feedback: Vec<FeedbackItem>,
    pub variants: Vec<PromptVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackItem {
    pub category: String,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariant {
    pub label: String,
    pub prompt: String,
    pub token_count: i64,
    pub use_case: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: String,
    pub total_prompts: i64,
    pub total_tokens_saved: i64,
    pub average_savings_percentage: f64,
    pub top_issues: Vec<String>,
}
