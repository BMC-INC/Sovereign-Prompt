mod analyzer;
mod db;
mod optimizer;
mod server;
mod tokenizer;
mod types;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present (silent fail if missing)
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db_path = std::env::var("SOVEREIGN_DB_PATH")
        .unwrap_or_else(|_| "./sovereign_prompt.db".to_string());

    let db = db::Database::new(&db_path).await?;
    db.migrate().await?;

    tracing::info!("SovereignPrompt MCP server starting...");

    tokio::select! {
        result = server::run(db) => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
                return Err(e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal, exiting gracefully...");
        }
    }

    Ok(())
}
