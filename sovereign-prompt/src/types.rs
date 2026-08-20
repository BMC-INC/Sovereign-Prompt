use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRecord {
    pub id: String,
    pub user_id: String,
    pub domain: String,
    pub token_model: String,
    pub original_prompt: String,
    pub original_token_count: i64,
    pub refined_prompt: String,
    pub refined_token_count: i64,
    pub savings_percentage: f64,
    pub analysis_feedback: serde_json::Value,
    pub output: Option<String>,
    pub output_token_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    // Governance fields
    pub governance_id: Option<String>,
    pub policy_version: Option<String>,
    pub approval_status: Option<String>,
    // Crypto fields
    pub content_hash: Option<String>,
    pub output_hash: Option<String>,
    pub signature: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
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
        Self::new_with_context(
            user_id,
            "general".to_string(),
            "cl100k_base".to_string(),
            original_prompt,
            original_token_count,
            refined_prompt,
            refined_token_count,
            analysis_feedback,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_context(
        user_id: String,
        domain: String,
        token_model: String,
        original_prompt: String,
        original_token_count: i64,
        refined_prompt: String,
        refined_token_count: i64,
        analysis_feedback: serde_json::Value,
    ) -> Self {
        let savings = if original_token_count > 0 {
            ((original_token_count - refined_token_count) as f64 / original_token_count as f64)
                * 100.0
        } else {
            0.0
        };

        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            domain,
            token_model,
            original_prompt,
            original_token_count,
            refined_prompt,
            refined_token_count,
            savings_percentage: savings,
            analysis_feedback,
            output: None,
            output_token_count: None,
            created_at: Utc::now(),
            governance_id: None,
            policy_version: None,
            approval_status: None,
            content_hash: None,
            output_hash: None,
            signature: None,
            signed_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeResponse {
    pub prompt_id: String,
    pub domain: String,
    pub token_model: String,
    pub original_prompt: String,
    pub original_token_count: i64,
    pub refined_prompt: String,
    pub refined_token_count: i64,
    pub savings_percentage: f64,
    pub token_counts_by_model: BTreeMap<String, ModelTokenSummary>,
    pub template: PromptTemplateSummary,
    pub feedback: Vec<FeedbackItem>,
    pub variants: Vec<PromptVariant>,
    pub content_hash: Option<String>,
    pub governance_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTokenSummary {
    pub original_token_count: i64,
    pub refined_token_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplateSummary {
    pub domain: String,
    pub template_name: String,
    pub strategy: String,
    pub constraints: Vec<String>,
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
pub struct AuditLogEntry {
    pub id: String,
    pub prompt_id: String,
    pub action: String,
    pub actor: String,
    pub detail: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: String,
    pub total_prompts: i64,
    pub total_tokens_saved: i64,
    pub average_savings_percentage: f64,
    pub top_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicExplanation {
    pub check_name: String,
    pub fired: bool,
    pub reason: Option<String>,
    pub matched_patterns: Vec<String>,
    pub threshold: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsReport {
    pub user_id: String,
    pub period: String,
    pub total_prompts: i64,
    pub total_original_tokens: i64,
    pub total_refined_tokens: i64,
    /// Per-prompt token difference (original - refined). Negative means
    /// structure was added; task-level savings are captured by
    /// `attempts_to_usable` and `cost_per_accepted_output` instead.
    pub total_token_delta: i64,
    pub average_token_delta_percentage: f64,
    /// Average prompts per positively-rated output (None until at least one
    /// prompt has a positive learning signal in the period).
    pub attempts_to_usable: Option<f64>,
    /// Fraction of prompts whose governance status was "approved".
    pub governance_clean_rate: Option<f64>,
    pub cost_estimates: Vec<CostEstimate>,
    pub top_issues: Vec<String>,
    pub daily_trend: Vec<DailyTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub model: String,
    pub rate_per_million: f64,
    pub original_cost: f64,
    pub refined_cost: f64,
    pub savings: f64,
    /// Refined-prompt cost divided by positively-rated outputs in the period.
    pub cost_per_accepted_output: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTrend {
    pub date: String,
    pub prompts: i64,
    pub token_delta: i64,
    pub token_delta_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSignal {
    pub id: String,
    pub prompt_id: String,
    pub signal: String,
    pub comment: Option<String>,
    pub actor: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningInsights {
    pub total_ratings: i64,
    pub positive_count: i64,
    pub negative_count: i64,
    pub positive_rate: f64,
    pub best_domains: Vec<String>,
    pub worst_issues: Vec<String>,
    pub avg_savings_positive: f64,
    pub avg_savings_negative: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamReport {
    pub team_members: Vec<String>,
    pub period: String,
    pub total_prompts: i64,
    pub total_token_delta: i64,
    pub average_token_delta_percentage: f64,
    pub attempts_to_usable: Option<f64>,
    pub cost_estimates: Vec<CostEstimate>,
    pub top_issues: Vec<String>,
    pub member_breakdown: Vec<MemberStats>,
    pub daily_trend: Vec<DailyTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberStats {
    pub user_id: String,
    pub total_prompts: i64,
    pub total_token_delta: i64,
    pub average_token_delta_percentage: f64,
}
