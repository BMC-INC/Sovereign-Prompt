use anyhow::Result;
use sovereign_prompt::auth::{HttpSecurity, HttpService};
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
        .with_writer(std::io::stderr)
        .init();

    let dashboard_addr = parse_bind("SOVEREIGN_DASHBOARD_ADDR", "127.0.0.1:8787")?;
    let mcp_transport = std::env::var("SOVEREIGN_MCP_TRANSPORT")
        .unwrap_or_else(|_| "stdio".to_string())
        .to_lowercase();
    if !matches!(mcp_transport.as_str(), "stdio" | "sse" | "http") {
        anyhow::bail!("SOVEREIGN_MCP_TRANSPORT must be stdio, sse, or http");
    }
    let mcp_sse_addr = parse_bind("SOVEREIGN_MCP_SSE_ADDR", "127.0.0.1:8790")?;
    let dashboard_only = match std::env::var("SOVEREIGN_DASHBOARD_ONLY").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("0" | "false" | "FALSE" | "no" | "NO") => false,
        Ok("1" | "true" | "TRUE" | "yes" | "YES") => true,
        _ => anyhow::bail!("SOVEREIGN_DASHBOARD_ONLY must be true or false"),
    };
    // Reject insecure explicit HTTP configuration before opening a database or
    // spawning an optional dashboard. Stdio continues to use OS process isolation.
    HttpSecurity::from_env(dashboard_addr, HttpService::Dashboard)?;
    if matches!(mcp_transport.as_str(), "sse" | "http") {
        HttpSecurity::from_env(mcp_sse_addr, HttpService::Mcp)?;
    }
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("SOVEREIGN_DB_PATH"))
        .unwrap_or_else(|_| "sqlite://./sovereign_prompt.db?mode=rwc".to_string());
    let db = Arc::new(db::Database::new(&database_url).await?);
    db.migrate().await?;
    if dashboard_only {
        tokio::select! {
            result = dashboard::run(db, dashboard_addr) => result?,
            _ = tokio::signal::ctrl_c() => {},
        }
        return Ok(());
    }

    let dashboard_db = db.clone();
    tokio::spawn(async move {
        if let Err(error) = dashboard::run(dashboard_db, dashboard_addr).await {
            tracing::warn!("Dashboard server stopped: {}", error);
        }
    });

    tracing::info!("SovereignPrompt MCP server starting...");
    tracing::info!("Dashboard available at http://{}", dashboard_addr);

    if matches!(mcp_transport.as_str(), "sse" | "http") {
        tracing::info!(
            "Starting MCP HTTP and compatibility SSE transports at http://{}",
            mcp_sse_addr
        );
        sovereign_prompt::http_mcp::run(db, mcp_sse_addr).await?;
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

fn parse_bind(name: &str, default: &str) -> Result<SocketAddr> {
    std::env::var(name)
        .unwrap_or_else(|_| default.into())
        .parse()
        .map_err(|error| anyhow::anyhow!("Invalid {name}: {error}"))
}
