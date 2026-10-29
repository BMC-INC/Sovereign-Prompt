//! Current Streamable HTTP plus bounded compatibility for legacy HTTP+SSE clients.
use crate::{
    auth::{self, HttpSecurity, HttpService, Principal, MAX_REQUEST_BODY_BYTES},
    config::SovereignConfig,
    db::Database,
    server::{build_server, SovereignPromptServer},
};
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Extension, Json, Router,
};
use rmcp::{
    transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
    ServiceExt,
};
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::{mpsc, Mutex, Semaphore},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const MAX_LEGACY_SESSIONS: usize = 64;
const SESSION_LIFETIME: Duration = Duration::from_secs(30 * 60);

#[derive(Clone)]
struct LegacySession {
    principal: Option<Principal>,
    incoming: mpsc::Sender<String>,
}

#[derive(Clone)]
struct LegacyState {
    server: SovereignPromptServer,
    sessions: Arc<Mutex<HashMap<String, LegacySession>>>,
    shutdown: CancellationToken,
}

/// Both HTTP paths pass through the same immutable credential and origin policy.
/// Modern HTTP is stateless, preventing cross-principal session reuse entirely.
pub fn router(db: Arc<Database>, security: Arc<HttpSecurity>, config: SovereignConfig) -> Router {
    router_with_shutdown(db, security, config, CancellationToken::new())
}

fn router_with_shutdown(
    db: Arc<Database>,
    security: Arc<HttpSecurity>,
    config: SovereignConfig,
    shutdown: CancellationToken,
) -> Router {
    let server = build_server(db, config);
    let modern_server = server.clone();
    let transport_config = StreamableHttpServerConfig::default()
        .with_cancellation_token(shutdown.clone())
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(MAX_REQUEST_BODY_BYTES)
        .with_allowed_hosts(security.allowed_hosts().iter().cloned());
    let modern = StreamableHttpService::new(
        move || Ok(modern_server.clone()),
        Arc::new(LocalSessionManager::default()),
        transport_config,
    );
    let legacy = LegacyState {
        server,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        shutdown,
    };
    Router::new()
        .route_service("/mcp", modern)
        .route("/sse", get(legacy_connect))
        .route("/message", post(legacy_message))
        .with_state(legacy)
        .layer(middleware::from_fn_with_state(security, auth::guard))
        .layer(middleware::from_fn_with_state(
            Arc::new(Semaphore::new(16)),
            admission,
        ))
}

async fn admission(
    State(slots): State<Arc<Semaphore>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let _permit = slots
        .try_acquire_owned()
        .map_err(|_| StatusCode::TOO_MANY_REQUESTS)?;
    tokio::time::timeout(Duration::from_secs(45), next.run(request))
        .await
        .map_err(|_| StatusCode::REQUEST_TIMEOUT)
}

async fn legacy_connect(
    State(state): State<LegacyState>,
    principal: Option<Extension<Principal>>,
) -> Result<impl IntoResponse, StatusCode> {
    let principal = principal.map(|Extension(principal)| principal);
    let id = Uuid::new_v4().to_string();
    let (incoming, mut messages) = mpsc::channel::<String>(16);
    let (events, outgoing) = mpsc::channel::<Result<Event, Infallible>>(16);
    {
        let mut sessions = state.sessions.lock().await;
        if sessions.len() >= MAX_LEGACY_SESSIONS {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        sessions.insert(
            id.clone(),
            LegacySession {
                principal: principal.clone(),
                incoming,
            },
        );
    }
    // The endpoint is same-origin and contains only an opaque session ID, never a credential.
    events
        .send(Ok(Event::default()
            .event("endpoint")
            .data(format!("/message?sessionId={id}"))))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tokio::spawn(async move {
        let (service_io, transport_io) = tokio::io::duplex(64 * 1024);
        let (read, mut write) = tokio::io::split(transport_io);
        let writer = tokio::spawn(async move {
            while let Some(message) = messages.recv().await {
                if write.write_all(message.as_bytes()).await.is_err()
                    || write.write_all(b"\n").await.is_err()
                {
                    break;
                }
            }
        });
        let event_sender = events.clone();
        let reader = tokio::spawn(async move {
            let mut lines = BufReader::new(read).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let event = Ok(Event::default().event("message").data(line));
                if !matches!(
                    tokio::time::timeout(Duration::from_secs(10), event_sender.send(event)).await,
                    Ok(Ok(()))
                ) {
                    break;
                }
            }
        });
        let service = state.server.with_principal(principal);
        tokio::select! {
            _ = state.shutdown.cancelled() => {},
            _ = events.closed() => {},
            _ = tokio::time::sleep(SESSION_LIFETIME) => {},
            _ = async {
                if let Ok(service) = service.serve(service_io).await { let _ = service.waiting().await; }
            } => {},
        }
        writer.abort();
        reader.abort();
        state.sessions.lock().await.remove(&id);
    });
    Ok(Sse::new(ReceiverStream::new(outgoing)).keep_alive(KeepAlive::default()))
}

#[derive(Deserialize)]
struct SessionQuery {
    #[serde(rename = "sessionId", alias = "session_id")]
    session_id: String,
}

async fn legacy_message(
    State(state): State<LegacyState>,
    Query(query): Query<SessionQuery>,
    principal: Option<Extension<Principal>>,
    Json(message): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    if !message.is_object() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let principal = principal.map(|Extension(principal)| principal);
    let session = state
        .sessions
        .lock()
        .await
        .get(&query.session_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    if session.principal != principal {
        return Err(StatusCode::NOT_FOUND);
    }
    session
        .incoming
        .try_send(message.to_string())
        .map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => StatusCode::TOO_MANY_REQUESTS,
            mpsc::error::TrySendError::Closed(_) => StatusCode::NOT_FOUND,
        })?;
    Ok(StatusCode::ACCEPTED)
}

pub async fn run(db: Arc<Database>, bind: SocketAddr) -> Result<()> {
    let security = Arc::new(HttpSecurity::from_env(bind, HttpService::Mcp)?);
    let shutdown = CancellationToken::new();
    let app = router_with_shutdown(db, security, SovereignConfig::load()?, shutdown.clone());
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(
        "MCP HTTP listening on {bind}: /mcp (Streamable HTTP), /sse (legacy compatibility)"
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            shutdown.cancel();
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use tower::ServiceExt;

    #[tokio::test]
    async fn legacy_sessions_are_bounded_and_shutdown_closes_open_streams() {
        let directory = tempfile::tempdir().unwrap();
        let db = Arc::new(
            Database::new(directory.path().join("sessions.db").to_str().unwrap())
                .await
                .unwrap(),
        );
        db.migrate().await.unwrap();
        let security = Arc::new(
            HttpSecurity::new(
                "127.0.0.1:8790".parse().unwrap(),
                HttpService::Mcp,
                Default::default(),
                vec![],
                vec![],
            )
            .unwrap(),
        );
        let shutdown = CancellationToken::new();
        let app = router_with_shutdown(db, security, SovereignConfig::default(), shutdown.clone());
        let mut connections = Vec::new();
        for _ in 0..MAX_LEGACY_SESSIONS {
            let request = Request::builder()
                .uri("/sse")
                .header("host", "localhost:8790")
                .body(Body::empty())
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            connections.push(response);
        }
        let request = Request::builder()
            .uri("/sse")
            .header("host", "localhost:8790")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.oneshot(request).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        shutdown.cancel();
        for response in connections {
            tokio::time::timeout(Duration::from_secs(5), to_bytes(response.into_body(), 1024))
                .await
                .unwrap()
                .unwrap();
        }
    }
}
