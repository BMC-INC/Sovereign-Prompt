use anyhow::Result;
use tracing_subscriber::EnvFilter;

mod proxy;
mod rewrite;
mod upstream;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    proxy::run().await
}
