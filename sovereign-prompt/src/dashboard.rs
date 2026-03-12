use crate::db::Database;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Clone)]
struct DashboardState {
    db: Arc<Database>,
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    limit: Option<i64>,
}

pub async fn run(db: Arc<Database>, addr: SocketAddr) -> anyhow::Result<()> {
    let state = DashboardState { db };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/stats/:user_id", get(stats_handler))
        .route("/api/history/:user_id", get(history_handler))
        .route("/ws/analytics/:user_id", get(websocket_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn stats_handler(
    Path(user_id): Path<String>,
    State(state): State<DashboardState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
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
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
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
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| analytics_socket(socket, state.db, user_id))
}

async fn analytics_socket(mut socket: WebSocket, db: Arc<Database>, user_id: String) {
    let mut ticker = interval(Duration::from_secs(2));

    loop {
        ticker.tick().await;

        let stats = match db.get_user_stats(&user_id).await {
            Ok(stats) => stats,
            Err(error) => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({
                            "type": "error",
                            "message": error.to_string(),
                        })
                        .to_string(),
                    ))
                    .await;
                break;
            }
        };

        let history = match db.get_recent_prompts(&user_id, 5).await {
            Ok(history) => history,
            Err(error) => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({
                            "type": "error",
                            "message": error.to_string(),
                        })
                        .to_string(),
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
            .send(Message::Text(payload.to_string()))
            .await
            .is_err()
        {
            break;
        }
    }
}

fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
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
    .grid { display: grid; gap: 14px; grid-template-columns: repeat(4, minmax(0,1fr)); margin-top: 14px; }
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
        <button id="connectBtn">Connect</button>
        <span class="status" id="status">offline</span>
      </div>
    </div>

    <div class="grid">
      <div class="card"><div class="label">Total Prompts</div><div class="value" id="totalPrompts">0</div></div>
      <div class="card"><div class="label">Tokens Saved</div><div class="value accent" id="tokensSaved">0</div></div>
      <div class="card"><div class="label">Avg Savings</div><div class="value green" id="avgSavings">0%</div></div>
      <div class="card"><div class="label">Stream Status</div><div class="value orange" id="streamState">idle</div></div>
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
      issues: document.getElementById("issues"),
      historyRows: document.getElementById("historyRows"),
    };

    function renderSnapshot(payload) {
      const stats = payload.stats || {};
      els.totalPrompts.textContent = stats.total_prompts ?? 0;
      els.tokensSaved.textContent = stats.total_tokens_saved ?? 0;
      const avg = Number(stats.average_savings_percentage ?? 0);
      els.avgSavings.textContent = avg.toFixed(1) + "%";
      els.streamState.textContent = "live";

      const issues = stats.top_issues || [];
      els.issues.innerHTML = issues.length
        ? issues.map((item) => `<span class="issue">${item}</span>`).join("")
        : `<span class="issue">No issues yet</span>`;

      const rows = payload.recent_history || [];
      els.historyRows.innerHTML = rows.map((row) => `
        <tr>
          <td><span class="pill">${row.id.slice(0, 8)}</span></td>
          <td>${row.domain || "general"}</td>
          <td>${row.token_model || "cl100k_base"}</td>
          <td>${row.original_token_count}</td>
          <td>${row.refined_token_count}</td>
          <td>${Number(row.savings_percentage || 0).toFixed(1)}%</td>
        </tr>
      `).join("");
    }

    function connect() {
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
    connect();
  </script>
</body>
</html>
"#;
