use crate::types::{PromptRecord, UserStats};
use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};

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

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_prompts_user_id ON prompts(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_prompts_created_at ON prompts(created_at)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn insert_prompt(&self, record: &PromptRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO prompts (
                id, user_id, original_prompt, original_token_count,
                refined_prompt, refined_token_count, savings_percentage,
                analysis_feedback, output, output_token_count, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&record.id)
        .bind(&record.user_id)
        .bind(&record.original_prompt)
        .bind(record.original_token_count)
        .bind(&record.refined_prompt)
        .bind(record.refined_token_count)
        .bind(record.savings_percentage)
        .bind(record.analysis_feedback.to_string())
        .bind(&record.output)
        .bind(record.output_token_count)
        .bind(record.created_at.to_rfc3339())
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
        sqlx::query(
            "UPDATE prompts SET output = ?, output_token_count = ? WHERE id = ?",
        )
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

    pub async fn get_recent_prompts(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<PromptRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM prompts WHERE user_id = ? ORDER BY created_at DESC LIMIT ?",
        )
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
                    original_prompt: r.get("original_prompt"),
                    original_token_count: r.get("original_token_count"),
                    refined_prompt: r.get("refined_prompt"),
                    refined_token_count: r.get("refined_token_count"),
                    savings_percentage: r.get("savings_percentage"),
                    analysis_feedback: serde_json::from_str(&feedback_str)
                        .unwrap_or_default(),
                    output: r.get("output"),
                    output_token_count: r.get("output_token_count"),
                    created_at: r
                        .get::<String, _>("created_at")
                        .parse::<chrono::DateTime<chrono::Utc>>()
                        .unwrap_or_default(),
                }
            })
            .collect();

        Ok(records)
    }
}
