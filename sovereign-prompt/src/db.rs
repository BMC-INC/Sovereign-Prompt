use crate::types::{AuditLogEntry, CostEstimate, DailyTrend, PromptRecord, SavingsReport, UserStats};
use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self> {
        let url = format!("sqlite://{}?mode=rwc", path);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS prompts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                domain TEXT NOT NULL DEFAULT 'general',
                token_model TEXT NOT NULL DEFAULT 'cl100k_base',
                original_prompt TEXT NOT NULL,
                original_token_count INTEGER NOT NULL,
                refined_prompt TEXT NOT NULL,
                refined_token_count INTEGER NOT NULL,
                savings_percentage REAL NOT NULL,
                analysis_feedback TEXT NOT NULL,
                output TEXT,
                output_token_count INTEGER,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        // Backward-compatible schema upgrades for existing SQLite files.
        self.add_column_if_missing(
            "ALTER TABLE prompts ADD COLUMN domain TEXT NOT NULL DEFAULT 'general'",
        )
        .await?;
        self.add_column_if_missing(
            "ALTER TABLE prompts ADD COLUMN token_model TEXT NOT NULL DEFAULT 'cl100k_base'",
        )
        .await?;

        // Governance columns
        self.add_column_if_missing("ALTER TABLE prompts ADD COLUMN governance_id TEXT")
            .await?;
        self.add_column_if_missing("ALTER TABLE prompts ADD COLUMN policy_version TEXT")
            .await?;
        self.add_column_if_missing("ALTER TABLE prompts ADD COLUMN approval_status TEXT")
            .await?;

        // Crypto columns
        self.add_column_if_missing("ALTER TABLE prompts ADD COLUMN content_hash TEXT")
            .await?;
        self.add_column_if_missing("ALTER TABLE prompts ADD COLUMN output_hash TEXT")
            .await?;
        self.add_column_if_missing("ALTER TABLE prompts ADD COLUMN signature TEXT")
            .await?;
        self.add_column_if_missing("ALTER TABLE prompts ADD COLUMN signed_at TEXT")
            .await?;

        // Audit log table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                prompt_id TEXT NOT NULL,
                action TEXT NOT NULL,
                actor TEXT NOT NULL,
                detail TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_prompt_id ON audit_log(prompt_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON audit_log(created_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_prompts_user_id ON prompts(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_prompts_created_at ON prompts(created_at)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn add_column_if_missing(&self, query: &str) -> Result<()> {
        let result = sqlx::query(query).execute(&self.pool).await;
        if let Err(err) = result {
            let msg = err.to_string().to_lowercase();
            if !msg.contains("duplicate column name") {
                return Err(err.into());
            }
        }
        Ok(())
    }

    pub async fn insert_prompt(&self, record: &PromptRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO prompts (
                id, user_id, domain, token_model, original_prompt, original_token_count,
                refined_prompt, refined_token_count, savings_percentage,
                analysis_feedback, output, output_token_count, created_at,
                governance_id, policy_version, approval_status,
                content_hash, output_hash, signature, signed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&record.id)
        .bind(&record.user_id)
        .bind(&record.domain)
        .bind(&record.token_model)
        .bind(&record.original_prompt)
        .bind(record.original_token_count)
        .bind(&record.refined_prompt)
        .bind(record.refined_token_count)
        .bind(record.savings_percentage)
        .bind(record.analysis_feedback.to_string())
        .bind(&record.output)
        .bind(record.output_token_count)
        .bind(record.created_at.to_rfc3339())
        .bind(&record.governance_id)
        .bind(&record.policy_version)
        .bind(&record.approval_status)
        .bind(&record.content_hash)
        .bind(&record.output_hash)
        .bind(&record.signature)
        .bind(record.signed_at.map(|dt| dt.to_rfc3339()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_output(
        &self,
        prompt_id: &str,
        output: &str,
        output_token_count: i64,
    ) -> Result<()> {
        sqlx::query("UPDATE prompts SET output = ?, output_token_count = ? WHERE id = ?")
            .bind(output)
            .bind(output_token_count)
            .bind(prompt_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user_stats(&self, user_id: &str) -> Result<UserStats> {
        let row = sqlx::query(
            "SELECT
                COUNT(*) as total_prompts,
                COALESCE(SUM(original_token_count - refined_token_count), 0) as total_tokens_saved,
                COALESCE(AVG(savings_percentage), 0.0) as avg_savings
            FROM prompts
            WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Aggregate top issues from recent feedback
        let feedback_rows = sqlx::query(
            "SELECT analysis_feedback FROM prompts WHERE user_id = ? ORDER BY created_at DESC LIMIT 100",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut category_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for frow in &feedback_rows {
            let feedback_str: String = frow.get("analysis_feedback");
            if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&feedback_str) {
                for item in items {
                    if let Some(cat) = item.get("category").and_then(|c| c.as_str()) {
                        *category_counts.entry(cat.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut top: Vec<(String, usize)> = category_counts.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        let top_issues: Vec<String> = top
            .into_iter()
            .take(5)
            .map(|(cat, count)| format!("{} ({})", cat, count))
            .collect();

        Ok(UserStats {
            user_id: user_id.to_string(),
            total_prompts: row.get::<i64, _>("total_prompts"),
            total_tokens_saved: row.get::<i64, _>("total_tokens_saved"),
            average_savings_percentage: row.get::<f64, _>("avg_savings"),
            top_issues,
        })
    }

    pub async fn get_recent_prompts(&self, user_id: &str, limit: i64) -> Result<Vec<PromptRecord>> {
        let rows =
            sqlx::query("SELECT * FROM prompts WHERE user_id = ? ORDER BY created_at DESC LIMIT ?")
                .bind(user_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?;

        let records = rows
            .into_iter()
            .map(|r| {
                let feedback_str: String = r.get("analysis_feedback");
                PromptRecord {
                    id: r.get("id"),
                    user_id: r.get("user_id"),
                    domain: r.get("domain"),
                    token_model: r.get("token_model"),
                    original_prompt: r.get("original_prompt"),
                    original_token_count: r.get("original_token_count"),
                    refined_prompt: r.get("refined_prompt"),
                    refined_token_count: r.get("refined_token_count"),
                    savings_percentage: r.get("savings_percentage"),
                    analysis_feedback: serde_json::from_str(&feedback_str).unwrap_or_default(),
                    output: r.get("output"),
                    output_token_count: r.get("output_token_count"),
                    created_at: r
                        .get::<String, _>("created_at")
                        .parse::<chrono::DateTime<chrono::Utc>>()
                        .unwrap_or_default(),
                    governance_id: r.try_get("governance_id").ok().flatten(),
                    policy_version: r.try_get("policy_version").ok().flatten(),
                    approval_status: r.try_get("approval_status").ok().flatten(),
                    content_hash: r.try_get("content_hash").ok().flatten(),
                    output_hash: r.try_get("output_hash").ok().flatten(),
                    signature: r.try_get("signature").ok().flatten(),
                    signed_at: r
                        .try_get::<Option<String>, _>("signed_at")
                        .ok()
                        .flatten()
                        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
                }
            })
            .collect();

        Ok(records)
    }

    pub async fn get_prompt_by_id(&self, prompt_id: &str) -> Result<Option<PromptRecord>> {
        let row = sqlx::query("SELECT * FROM prompts WHERE id = ?")
            .bind(prompt_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => {
                let feedback_str: String = r.get("analysis_feedback");
                Ok(Some(PromptRecord {
                    id: r.get("id"),
                    user_id: r.get("user_id"),
                    domain: r.get("domain"),
                    token_model: r.get("token_model"),
                    original_prompt: r.get("original_prompt"),
                    original_token_count: r.get("original_token_count"),
                    refined_prompt: r.get("refined_prompt"),
                    refined_token_count: r.get("refined_token_count"),
                    savings_percentage: r.get("savings_percentage"),
                    analysis_feedback: serde_json::from_str(&feedback_str).unwrap_or_default(),
                    output: r.get("output"),
                    output_token_count: r.get("output_token_count"),
                    created_at: r
                        .get::<String, _>("created_at")
                        .parse::<chrono::DateTime<chrono::Utc>>()
                        .unwrap_or_default(),
                    governance_id: r.try_get("governance_id").ok().flatten(),
                    policy_version: r.try_get("policy_version").ok().flatten(),
                    approval_status: r.try_get("approval_status").ok().flatten(),
                    content_hash: r.try_get("content_hash").ok().flatten(),
                    output_hash: r.try_get("output_hash").ok().flatten(),
                    signature: r.try_get("signature").ok().flatten(),
                    signed_at: r
                        .try_get::<Option<String>, _>("signed_at")
                        .ok()
                        .flatten()
                        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn update_approval_status(
        &self,
        prompt_id: &str,
        status: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE prompts SET approval_status = ? WHERE id = ?")
            .bind(status)
            .bind(prompt_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_signature(
        &self,
        prompt_id: &str,
        signature: &str,
        signed_at: &chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        sqlx::query("UPDATE prompts SET signature = ?, signed_at = ? WHERE id = ?")
            .bind(signature)
            .bind(signed_at.to_rfc3339())
            .bind(prompt_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_output_hash(&self, prompt_id: &str, output_hash: &str) -> Result<()> {
        sqlx::query("UPDATE prompts SET output_hash = ? WHERE id = ?")
            .bind(output_hash)
            .bind(prompt_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_audit_log(&self, entry: &AuditLogEntry) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_log (id, prompt_id, action, actor, detail, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&entry.id)
        .bind(&entry.prompt_id)
        .bind(&entry.action)
        .bind(&entry.actor)
        .bind(entry.detail.to_string())
        .bind(entry.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_savings_report(
        &self,
        user_id: &str,
        period: &str,
        cost_rates: Option<&[(String, f64)]>,
    ) -> Result<SavingsReport> {
        let days = match period {
            "7d" => 7,
            "30d" => 30,
            "90d" => 90,
            "all" => 36500, // ~100 years
            _ => 30,
        };

        let date_filter = chrono::Utc::now() - chrono::Duration::days(days);
        let date_str = date_filter.to_rfc3339();

        // Aggregate totals
        let row = sqlx::query(
            "SELECT
                COUNT(*) as total_prompts,
                COALESCE(SUM(original_token_count), 0) as total_original,
                COALESCE(SUM(refined_token_count), 0) as total_refined,
                COALESCE(SUM(original_token_count - refined_token_count), 0) as total_saved,
                COALESCE(AVG(savings_percentage), 0.0) as avg_savings
            FROM prompts
            WHERE user_id = ? AND created_at >= ?",
        )
        .bind(user_id)
        .bind(&date_str)
        .fetch_one(&self.pool)
        .await?;

        let total_prompts: i64 = row.get("total_prompts");
        let total_original: i64 = row.get("total_original");
        let total_refined: i64 = row.get("total_refined");
        let total_saved: i64 = row.get("total_saved");
        let avg_savings: f64 = row.get("avg_savings");

        // Default cost rates per 1M input tokens
        let default_rates: Vec<(String, f64)> = vec![
            ("Claude Sonnet 4".to_string(), 3.00),
            ("Claude Opus 4".to_string(), 15.00),
            ("GPT-4o".to_string(), 2.50),
            ("GPT-4o-mini".to_string(), 0.15),
        ];
        let rates = cost_rates.unwrap_or(&default_rates);

        let cost_estimates: Vec<CostEstimate> = rates
            .iter()
            .map(|(model, rate)| {
                let original_cost = (total_original as f64 / 1_000_000.0) * rate;
                let refined_cost = (total_refined as f64 / 1_000_000.0) * rate;
                CostEstimate {
                    model: model.clone(),
                    rate_per_million: *rate,
                    original_cost,
                    refined_cost,
                    savings: original_cost - refined_cost,
                }
            })
            .collect();

        // Top issues
        let feedback_rows = sqlx::query(
            "SELECT analysis_feedback FROM prompts WHERE user_id = ? AND created_at >= ? ORDER BY created_at DESC LIMIT 100",
        )
        .bind(user_id)
        .bind(&date_str)
        .fetch_all(&self.pool)
        .await?;

        let mut category_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for frow in &feedback_rows {
            let feedback_str: String = frow.get("analysis_feedback");
            if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&feedback_str) {
                for item in items {
                    if let Some(cat) = item.get("category").and_then(|c| c.as_str()) {
                        *category_counts.entry(cat.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        let mut top: Vec<(String, usize)> = category_counts.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        let top_issues: Vec<String> = top
            .into_iter()
            .take(5)
            .map(|(cat, count)| format!("{} ({})", cat, count))
            .collect();

        // Daily trend
        let trend_rows = sqlx::query(
            "SELECT
                DATE(created_at) as day,
                COUNT(*) as prompts,
                COALESCE(SUM(original_token_count - refined_token_count), 0) as tokens_saved,
                COALESCE(AVG(savings_percentage), 0.0) as avg_savings
            FROM prompts
            WHERE user_id = ? AND created_at >= ?
            GROUP BY DATE(created_at)
            ORDER BY day ASC",
        )
        .bind(user_id)
        .bind(&date_str)
        .fetch_all(&self.pool)
        .await?;

        let daily_trend: Vec<DailyTrend> = trend_rows
            .into_iter()
            .map(|r| DailyTrend {
                date: r.get("day"),
                prompts: r.get("prompts"),
                tokens_saved: r.get("tokens_saved"),
                savings_percentage: r.get("avg_savings"),
            })
            .collect();

        Ok(SavingsReport {
            user_id: user_id.to_string(),
            period: period.to_string(),
            total_prompts,
            total_original_tokens: total_original,
            total_refined_tokens: total_refined,
            total_tokens_saved: total_saved,
            average_savings_percentage: avg_savings,
            cost_estimates,
            top_issues,
            daily_trend,
        })
    }

    pub async fn get_audit_trail(&self, prompt_id: &str) -> Result<Vec<AuditLogEntry>> {
        let rows =
            sqlx::query("SELECT * FROM audit_log WHERE prompt_id = ? ORDER BY created_at ASC")
                .bind(prompt_id)
                .fetch_all(&self.pool)
                .await?;

        let entries = rows
            .into_iter()
            .map(|r| {
                let detail_str: String = r.get("detail");
                AuditLogEntry {
                    id: r.get("id"),
                    prompt_id: r.get("prompt_id"),
                    action: r.get("action"),
                    actor: r.get("actor"),
                    detail: serde_json::from_str(&detail_str).unwrap_or_default(),
                    created_at: r
                        .get::<String, _>("created_at")
                        .parse::<chrono::DateTime<chrono::Utc>>()
                        .unwrap_or_default(),
                }
            })
            .collect();

        Ok(entries)
    }
}
