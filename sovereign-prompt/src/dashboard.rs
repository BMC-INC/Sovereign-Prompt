use crate::auth::{HttpSecurity, HttpService, Principal};
use crate::db::Database;
use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::{header, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Extension;
use axum::{Json, Router};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::time::{interval, Duration};

#[derive(Clone)]
struct DashboardState {
    db: Arc<Database>,
    security: Arc<HttpSecurity>,
    sessions: Arc<Mutex<HashMap<String, (Principal, Instant)>>>,
    socket_slots: Arc<tokio::sync::Semaphore>,
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    limit: Option<i64>,
}

pub async fn run(db: Arc<Database>, addr: SocketAddr) -> anyhow::Result<()> {
    let security = Arc::new(HttpSecurity::from_env(addr, HttpService::Dashboard)?);
    let app = router(db, security);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn router(db: Arc<Database>, security: Arc<HttpSecurity>) -> Router {
    let state = DashboardState {
        db,
        security,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        socket_slots: Arc::new(tokio::sync::Semaphore::new(64)),
    };
    Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/session", get(current_session))
        .route("/api/stats/{user_id}", get(stats_handler))
        .route("/api/history/{user_id}", get(history_handler))
        .route("/ws/analytics/{user_id}", get(websocket_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            dashboard_guard,
        ))
        .with_state(state)
}

async fn dashboard_guard(
    State(state): State<DashboardState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    state.security.validate_origin_and_host(request.headers())?;
    let public = matches!(
        request.uri().path(),
        "/" | "/health" | "/api/login" | "/api/logout"
    );
    if !public {
        let principal = session_id(request.headers()).and_then(|id| {
            let sessions = state.sessions.lock().ok()?;
            sessions
                .get(id)
                .filter(|(_, expiry)| *expiry > Instant::now())
                .map(|(principal, _)| principal.clone())
        });
        let principal = match principal {
            Some(principal) => Some(principal),
            None => state.security.authenticate_headers(request.headers())?,
        };
        if let Some(principal) = principal {
            request.extensions_mut().insert(principal);
        }
    }
    let (parts, body) = request.into_parts();
    let body = tokio::time::timeout(
        Duration::from_secs(15),
        axum::body::to_bytes(body, crate::auth::MAX_REQUEST_BODY_BYTES),
    )
    .await
    .map_err(|_| StatusCode::REQUEST_TIMEOUT)?
    .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let mut response = next.run(Request::from_parts(parts, Body::from(body))).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    response
        .headers_mut()
        .insert("x-frame-options", "DENY".parse().unwrap());
    Ok(response)
}

#[derive(Deserialize)]
struct LoginRequest {
    token: String,
}

fn session_id(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("sovereign_session="))
}

async fn login(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Response, StatusCode> {
    let principal = state
        .security
        .authenticate_token(&body.token)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let id = uuid::Uuid::new_v4().to_string();
    {
        let mut sessions = state
            .sessions
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sessions.retain(|_, (_, expiry)| *expiry > Instant::now());
        if let Some(previous) = session_id(&headers) {
            sessions.remove(previous);
        }
        if sessions.len() >= 1024 {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        sessions.insert(
            id.clone(),
            (
                principal.clone(),
                Instant::now() + Duration::from_secs(3600),
            ),
        );
    }
    let secure = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|origin| origin.starts_with("https://"));
    let cookie = format!(
        "sovereign_session={id}; HttpOnly; SameSite=Strict; Path=/; Max-Age=3600{}",
        if secure { "; Secure" } else { "" }
    );
    Ok((
        [(header::SET_COOKIE, cookie)],
        Json(serde_json::json!({"user_id": principal.user_id})),
    )
        .into_response())
}

async fn current_session(principal: Option<Extension<Principal>>) -> Json<serde_json::Value> {
    Json(
        serde_json::json!({"user_id": principal.as_ref().map(|p| &p.0.user_id), "authenticated": principal.is_some()}),
    )
}

async fn logout(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
) -> Result<Response, StatusCode> {
    if let Some(id) = session_id(&headers) {
        state
            .sessions
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .remove(id);
    }
    Ok((
        [(
            header::SET_COOKIE,
            "sovereign_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
        )],
        StatusCode::NO_CONTENT,
    )
        .into_response())
}

fn authorize_user(
    user_id: &str,
    principal: Option<&Principal>,
) -> Result<(), (StatusCode, String)> {
    if principal.is_some_and(|principal| principal.user_id != user_id) {
        return Err((
            StatusCode::FORBIDDEN,
            "User does not match authenticated identity".into(),
        ));
    }
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn stats_handler(
    Path(user_id): Path<String>,
    State(state): State<DashboardState>,
    principal: Option<Extension<Principal>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    authorize_user(&user_id, principal.as_ref().map(|p| &p.0))?;
    let stats = state
        .db
        .get_user_stats(&user_id)
        .await
        .map_err(internal_error)?;
    Ok(Json(serde_json::json!(stats)))
}

async fn history_handler(
    Path(user_id): Path<String>,
    Query(query): Query<HistoryQuery>,
    State(state): State<DashboardState>,
    principal: Option<Extension<Principal>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    authorize_user(&user_id, principal.as_ref().map(|p| &p.0))?;
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    let history = state
        .db
        .get_recent_prompts(&user_id, limit)
        .await
        .map_err(internal_error)?;
    Ok(Json(serde_json::json!(history)))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(user_id): Path<String>,
    State(state): State<DashboardState>,
    principal: Option<Extension<Principal>>,
    headers: axum::http::HeaderMap,
) -> Result<Response, (StatusCode, String)> {
    authorize_user(&user_id, principal.as_ref().map(|p| &p.0))?;
    let permit = state
        .socket_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many analytics streams".into(),
            )
        })?;
    let session = session_id(&headers).map(String::from);
    Ok(ws
        .on_upgrade(move |socket| async move {
            let _permit = permit;
            analytics_socket(socket, state, user_id, session).await;
        })
        .into_response())
}

fn session_active(state: &DashboardState, session: Option<&str>) -> bool {
    session.is_none_or(|id| {
        state.sessions.lock().ok().is_some_and(|sessions| {
            sessions
                .get(id)
                .is_some_and(|(_, expiry)| *expiry > Instant::now())
        })
    })
}

async fn analytics_socket(
    mut socket: WebSocket,
    state: DashboardState,
    user_id: String,
    session: Option<String>,
) {
    let db = &state.db;
    let started = Instant::now();
    let mut ticker = interval(Duration::from_secs(2));

    loop {
        ticker.tick().await;
        if started.elapsed() >= Duration::from_secs(1800)
            || !session_active(&state, session.as_deref())
        {
            let _ = socket.send(Message::Close(None)).await;
            break;
        }

        let stats = match db.get_user_stats(&user_id).await {
            Ok(stats) => stats,
            Err(error) => {
                tracing::error!("Dashboard database error: {}", error);
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({
                            "type": "error",
                            "message": "Unable to read analytics",
                        })
                        .to_string()
                        .into(),
                    ))
                    .await;
                break;
            }
        };

        let history = match db.get_recent_prompts(&user_id, 5).await {
            Ok(history) => history,
            Err(error) => {
                tracing::error!("Dashboard database error: {}", error);
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({
                            "type": "error",
                            "message": "Unable to read analytics",
                        })
                        .to_string()
                        .into(),
                    ))
                    .await;
                break;
            }
        };

        let payload = serde_json::json!({
            "type": "snapshot",
            "user_id": user_id,
            "stats": stats,
            "recent_history": history,
        });

        if socket
            .send(Message::Text(payload.to_string().into()))
            .await
            .is_err()
        {
            break;
        }
    }
}

fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    {
        tracing::error!("Dashboard database error: {}", error);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to read analytics".into(),
        )
    }
}

const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <title>SovereignPrompt Analytics</title>
  <style>
    :root {
      --bg: #060a10;
      --panel: #0f1722;
      --panel-2: #151f2d;
      --text: #ebf4ff;
      --muted: #9fb3cc;
      --line: #273449;
      --accent: #20c2ff;
      --accent-2: #8be887;
      --warn: #ff9f5a;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "IBM Plex Sans", "Avenir Next", "Segoe UI", sans-serif;
      color: var(--text);
      background:
        radial-gradient(circle at 15% 10%, #123561 0%, transparent 36%),
        radial-gradient(circle at 85% 90%, #1d4d2f 0%, transparent 34%),
        linear-gradient(180deg, #05080e 0%, #090f17 100%);
      min-height: 100vh;
    }
    .wrap { max-width: 1100px; margin: 0 auto; padding: 32px 20px 40px; }
    .hero { display: flex; justify-content: space-between; align-items: end; gap: 20px; margin-bottom: 18px; }
    .title { font-size: clamp(30px, 5vw, 48px); margin: 0; letter-spacing: -1px; }
    .subtitle { color: var(--muted); margin-top: 8px; }
    .controls {
      display: flex; gap: 10px; align-items: center;
      background: color-mix(in srgb, var(--panel) 84%, transparent);
      border: 1px solid var(--line); border-radius: 14px; padding: 10px;
      backdrop-filter: blur(8px);
    }
    input {
      width: 220px; border: 1px solid var(--line); border-radius: 10px;
      background: var(--panel-2); color: var(--text); padding: 9px 12px;
    }
    button {
      border: none; border-radius: 10px; padding: 9px 12px; cursor: pointer;
      background: linear-gradient(140deg, var(--accent), #2d85ff); color: #001321;
      font-weight: 700;
    }
    .grid { display: grid; gap: 14px; grid-template-columns: repeat(5, minmax(0,1fr)); margin-top: 14px; }
    .card {
      background: color-mix(in srgb, var(--panel) 85%, transparent);
      border: 1px solid var(--line); border-radius: 14px; padding: 16px;
      backdrop-filter: blur(8px);
    }
    .label { color: var(--muted); font-size: 12px; text-transform: uppercase; letter-spacing: 1px; }
    .value { font-size: 31px; font-weight: 800; margin-top: 6px; }
    .accent { color: var(--accent); }
    .green { color: var(--accent-2); }
    .orange { color: var(--warn); }
    .section-title { margin: 18px 0 10px; color: var(--muted); text-transform: uppercase; letter-spacing: 1px; font-size: 12px; }
    .panel {
      background: color-mix(in srgb, var(--panel) 85%, transparent);
      border: 1px solid var(--line); border-radius: 14px;
      overflow: hidden;
    }
    table { width: 100%; border-collapse: collapse; font-size: 14px; }
    th, td { padding: 10px 12px; border-bottom: 1px solid #223044; text-align: left; }
    th { color: var(--muted); font-size: 12px; text-transform: uppercase; letter-spacing: 1px; }
    tr:last-child td { border-bottom: none; }
    .pill { font-size: 12px; padding: 3px 8px; border-radius: 999px; background: #183452; color: #7ec9ff; }
    .top-issues { display: flex; flex-wrap: wrap; gap: 8px; padding: 14px; }
    .issue { background: #162132; border: 1px solid #2b3f5b; border-radius: 999px; padding: 6px 10px; color: #c5d9ef; font-size: 13px; }
    .status { margin-left: 8px; font-size: 12px; color: var(--muted); }
    @media (max-width: 1060px) { .grid { grid-template-columns: repeat(3, minmax(0,1fr)); } }
    @media (max-width: 960px) { .grid { grid-template-columns: repeat(2, minmax(0,1fr)); } .hero { flex-direction: column; align-items: start; } }
    @media (max-width: 560px) { .grid { grid-template-columns: 1fr; } input { width: 100%; } .controls { width: 100%; } }
  </style>
</head>
<body>
  <div class="wrap">
    <div class="hero">
      <div>
        <h1 class="title">SovereignPrompt Live Analytics</h1>
        <div class="subtitle">WebSocket transport snapshots + REST history views.</div>
      </div>
      <div class="controls">
        <input id="userId" value="anonymous" aria-label="User ID" />
        <input id="apiKey" type="password" autocomplete="off" placeholder="API key (when enabled)" aria-label="API key" />
        <button id="connectBtn">Connect</button>
        <button id="logoutBtn">Sign out</button>
        <span class="status" id="status">offline</span>
      </div>
    </div>

    <div class="grid">
      <div class="card"><div class="label">Total Prompts</div><div class="value" id="totalPrompts">0</div></div>
      <div class="card"><div class="label">Tokens Saved</div><div class="value accent" id="tokensSaved">0</div></div>
      <div class="card"><div class="label">Avg Savings</div><div class="value green" id="avgSavings">0%</div></div>
      <div class="card"><div class="label">Stream Status</div><div class="value orange" id="streamState">idle</div></div>
      <div class="card"><div class="label">Governance</div><div class="value" id="govStatus" style="color:var(--accent-2)">--</div></div>
    </div>

    <div class="section-title">Top Issues</div>
    <div class="panel"><div class="top-issues" id="issues"></div></div>

    <div class="section-title">Recent Prompt History</div>
    <div class="panel">
      <table>
        <thead>
          <tr>
            <th>Prompt ID</th>
            <th>Domain</th>
            <th>Model</th>
            <th>Original</th>
            <th>Refined</th>
            <th>Savings</th>
            <th>Hash</th>
            <th>Signed</th>
          </tr>
        </thead>
        <tbody id="historyRows"></tbody>
      </table>
    </div>
  </div>

  <script>
    let socket = null;
    const els = {
      userId: document.getElementById("userId"),
      connectBtn: document.getElementById("connectBtn"),
      status: document.getElementById("status"),
      totalPrompts: document.getElementById("totalPrompts"),
      tokensSaved: document.getElementById("tokensSaved"),
      avgSavings: document.getElementById("avgSavings"),
      streamState: document.getElementById("streamState"),
      govStatus: document.getElementById("govStatus"),
      issues: document.getElementById("issues"),
      historyRows: document.getElementById("historyRows"),
    };

    function escapeHtml(value) {
      const node = document.createElement("span"); node.textContent = String(value ?? ""); return node.innerHTML;
    }

    function renderSnapshot(payload) {
      const stats = payload.stats || {};
      els.totalPrompts.textContent = stats.total_prompts ?? 0;
      els.tokensSaved.textContent = stats.total_tokens_saved ?? 0;
      const avg = Number(stats.average_savings_percentage ?? 0);
      els.avgSavings.textContent = avg.toFixed(1) + "%";
      els.streamState.textContent = "live";

      const recent = payload.recent_history || [];
      const latestStatus = recent.length > 0 && recent[0].approval_status ? recent[0].approval_status : "--";
      els.govStatus.textContent = latestStatus;

      const issues = stats.top_issues || [];
      els.issues.innerHTML = issues.length
        ? issues.map((item) => `<span class="issue">${escapeHtml(item)}</span>`).join("")
        : `<span class="issue">No issues yet</span>`;

      const rows = payload.recent_history || [];
      els.historyRows.innerHTML = rows.map((row) => `
        <tr>
          <td><span class="pill">${escapeHtml(String(row.id).slice(0, 8))}</span></td>
          <td>${escapeHtml(row.domain || "general")}</td>
          <td>${escapeHtml(row.token_model || "cl100k_base")}</td>
          <td>${Number(row.original_token_count)}</td>
          <td>${Number(row.refined_token_count)}</td>
          <td>${Number(row.savings_percentage || 0).toFixed(1)}%</td>
          <td>${row.content_hash ? `<span class="pill">${escapeHtml(String(row.content_hash).slice(0,8))}…</span>` : '<span style="color:var(--muted)">none</span>'}</td>
          <td>${row.signature ? '<span style="color:var(--accent-2)">&#x2713;</span>' : '<span style="color:var(--muted)">&mdash;</span>'}</td>
        </tr>
      `).join("");
    }

    async function connect() {
      const input = document.getElementById("apiKey");
      if (input.value) {
        const response = await fetch("/api/login", {method:"POST", headers:{"Content-Type":"application/json"}, body:JSON.stringify({token:input.value})});
        input.value = "";
        if (!response.ok) { els.status.textContent = "authentication failed"; return; }
        els.userId.value = (await response.json()).user_id;
      }
      const session = await fetch("/api/session");
      if (!session.ok) { els.status.textContent = "API key required"; return; }
      const identity = await session.json();
      if (identity.user_id) els.userId.value = identity.user_id;
      els.userId.disabled = identity.authenticated;
      const userId = encodeURIComponent(els.userId.value.trim() || "anonymous");
      if (socket) socket.close();

      const proto = location.protocol === "https:" ? "wss" : "ws";
      const url = `${proto}://${location.host}/ws/analytics/${userId}`;
      socket = new WebSocket(url);
      els.status.textContent = "connecting";
      els.streamState.textContent = "connecting";

      socket.onopen = () => {
        els.status.textContent = "connected";
      };

      socket.onmessage = (event) => {
        const payload = JSON.parse(event.data);
        if (payload.type === "snapshot") {
          renderSnapshot(payload);
        } else if (payload.type === "error") {
          els.status.textContent = "error";
          els.streamState.textContent = "error";
        }
      };

      socket.onclose = () => {
        els.status.textContent = "offline";
        els.streamState.textContent = "offline";
      };

      socket.onerror = () => {
        els.status.textContent = "error";
        els.streamState.textContent = "error";
      };
    }

    document.getElementById("connectBtn").addEventListener("click", connect);
    document.getElementById("logoutBtn").addEventListener("click", async () => {
      if (socket) socket.close();
      await fetch("/api/logout", {method:"POST"});
      document.getElementById("apiKey").value = "";
      els.historyRows.textContent = "";
      els.issues.textContent = "";
      for (const key of ["totalPrompts", "tokensSaved", "avgSavings", "govStatus"]) els[key].textContent = "--";
      els.userId.disabled = false;
      els.status.textContent = "signed out";
    });
    connect();
  </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn app() -> (Router, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            Database::new(dir.path().join("dashboard.db").to_str().unwrap())
                .await
                .unwrap(),
        );
        db.migrate().await.unwrap();
        let security = HttpSecurity::new(
            "127.0.0.1:8787".parse().unwrap(),
            HttpService::Dashboard,
            [
                ("alice".into(), "a".repeat(32)),
                ("bob".into(), "b".repeat(32)),
            ]
            .into(),
            vec![],
            vec![],
        )
        .unwrap();
        (router(db, Arc::new(security)), dir)
    }

    #[tokio::test]
    async fn rest_and_websocket_reject_untrusted_hosts_origins_and_other_users() {
        let (app, _dir) = app().await;
        for (path, host, origin, key, expected) in [
            (
                "/api/stats/alice",
                "127.0.0.1:8787",
                None,
                None,
                StatusCode::UNAUTHORIZED,
            ),
            (
                "/api/stats/alice",
                "evil.example",
                None,
                Some("a".repeat(32)),
                StatusCode::FORBIDDEN,
            ),
            (
                "/api/stats/alice",
                "127.0.0.1:8787",
                Some("https://evil.example"),
                Some("a".repeat(32)),
                StatusCode::FORBIDDEN,
            ),
            (
                "/api/stats/bob",
                "127.0.0.1:8787",
                None,
                Some("a".repeat(32)),
                StatusCode::FORBIDDEN,
            ),
            (
                "/api/history/bob",
                "127.0.0.1:8787",
                None,
                Some("a".repeat(32)),
                StatusCode::FORBIDDEN,
            ),
            (
                "/ws/analytics/bob",
                "127.0.0.1:8787",
                None,
                None,
                StatusCode::UNAUTHORIZED,
            ),
            (
                "/api/stats/alice",
                "127.0.0.1:8787",
                None,
                Some("a".repeat(32)),
                StatusCode::OK,
            ),
        ] {
            let mut request = Request::builder().uri(path).header("host", host);
            if let Some(origin) = origin {
                request = request.header("origin", origin);
            }
            if let Some(key) = key {
                request = request.header("authorization", format!("Bearer {key}"));
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), expected, "{path}, {host}, {origin:?}");
        }
    }

    #[tokio::test]
    async fn login_uses_expiring_opaque_cookie_and_logout_revokes_it() {
        let (app, _dir) = app().await;
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header("host", "127.0.0.1:8787")
                    .header("origin", "https://127.0.0.1:8787")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"token": "a".repeat(32)}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value = response.headers()[header::SET_COOKIE].to_str().unwrap();
        assert!(
            value.contains("HttpOnly")
                && value.contains("SameSite=Strict")
                && value.contains("Secure")
        );
        assert!(!value.contains(&"a".repeat(32)));
        let cookie = value.split(';').next().unwrap().to_string();
        for (path, status) in [
            ("/api/history/alice", StatusCode::OK),
            ("/api/history/bob", StatusCode::FORBIDDEN),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(path)
                        .header("host", "127.0.0.1:8787")
                        .header("cookie", &cookie)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), status);
        }
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/logout")
                    .header("host", "127.0.0.1:8787")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/history/alice")
                    .header("host", "127.0.0.1:8787")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn identity_check_preserves_local_operator_access() {
        assert!(authorize_user("legacy-user", None).is_ok());
        assert!(authorize_user(
            "bob",
            Some(&Principal {
                user_id: "alice".into()
            })
        )
        .is_err());
    }
}
