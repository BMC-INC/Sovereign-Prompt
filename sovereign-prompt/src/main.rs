use anyhow::Result;
use sovereign_prompt::{dashboard, db, server};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present (silent fail if missing)
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("SOVEREIGN_DB_PATH").map(|p| format!("sqlite://{}?mode=rwc", p)))
        .unwrap_or_else(|_| "sqlite://./sovereign_prompt.db?mode=rwc".to_string());

    let db = Arc::new(db::Database::new(&database_url).await?);
    db.migrate().await?;

    let dashboard_addr = std::env::var("SOVEREIGN_DASHBOARD_ADDR")
        .ok()
        .and_then(|addr| addr.parse::<SocketAddr>().ok())
        .unwrap_or_else(|| "127.0.0.1:8787".parse().unwrap());
    let mcp_transport = std::env::var("SOVEREIGN_MCP_TRANSPORT")
        .unwrap_or_else(|_| "stdio".to_string())
        .to_lowercase();
    let mcp_sse_addr = std::env::var("SOVEREIGN_MCP_SSE_ADDR")
        .ok()
        .and_then(|addr| addr.parse::<SocketAddr>().ok())
        .unwrap_or_else(|| "127.0.0.1:8790".parse().unwrap());
    let dashboard_only = std::env::var("SOVEREIGN_DASHBOARD_ONLY")
        .ok()
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);

    let dashboard_db = db.clone();
    tokio::spawn(async move {
        if let Err(error) = dashboard::run(dashboard_db, dashboard_addr).await {
            tracing::warn!("Dashboard server stopped: {}", error);
        }
    });

    tracing::info!("SovereignPrompt MCP server starting...");
    tracing::info!("Dashboard available at http://{}", dashboard_addr);

    if dashboard_only {
        tracing::info!("Running in dashboard-only mode.");
        tokio::signal::ctrl_c().await?;
        tracing::info!("Received shutdown signal, exiting gracefully...");
        return Ok(());
    }

    if mcp_transport == "sse" {
        tracing::info!("Starting MCP SSE transport at http://{}", mcp_sse_addr);
        server::run_sse(db, mcp_sse_addr).await?;
        return Ok(());
    }

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
