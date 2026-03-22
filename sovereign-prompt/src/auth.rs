use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};

/// Extract the expected API key hash from the environment.
/// Returns `None` if `SOVEREIGN_API_KEY` is not set (auth disabled).
pub fn expected_key_hash() -> Option<String> {
    std::env::var("SOVEREIGN_API_KEY").ok().map(|key| {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    })
}

/// Axum middleware that validates API key from request headers.
/// Checks `Authorization: Bearer <key>` or `X-API-Key: <key>`.
/// If `SOVEREIGN_API_KEY` env var is not set, all requests pass through.
pub async fn require_api_key(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let Some(expected_hash) = expected_key_hash() else {
        return Ok(next.run(req).await);
    };

    let provided_key = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| req.headers().get("x-api-key").and_then(|v| v.to_str().ok()));

    match provided_key {
        Some(key) => {
            let mut hasher = Sha256::new();
            hasher.update(key.as_bytes());
            let provided_hash = hex::encode(hasher.finalize());
            if provided_hash == expected_hash {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
