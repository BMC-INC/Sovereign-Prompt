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
