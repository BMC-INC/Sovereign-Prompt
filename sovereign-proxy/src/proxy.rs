use anyhow::Result;
use axum::{routing::get, Json, Router};
use std::net::SocketAddr;

pub async fn run() -> Result<()> {
    let app = Router::new()
        .route("/health", get(health));

    let addr: SocketAddr = std::env::var("SOVEREIGN_PROXY_ADDR")
        .ok()
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| "127.0.0.1:8788".parse().unwrap());

    tracing::info!("SovereignProxy listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "service": "sovereign-proxy"}))
}
