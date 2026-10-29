use axum::{
    body::{to_bytes, Body},
    http::{HeaderMap, Request, StatusCode},
};
use serde_json::{json, Value};
use sovereign_prompt::{
    auth::{HttpSecurity, HttpService},
    config::SovereignConfig,
    db::Database,
    http_mcp,
};
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio_stream::StreamExt;
use tower::ServiceExt;

const ALICE: &str = "alice-test-key-00000000000000000000";
const BOB: &str = "bob-test-key-0000000000000000000000";
fn security(service: HttpService) -> HttpSecurity {
    HttpSecurity::new(
        "127.0.0.1:8790".parse().unwrap(),
        service,
        BTreeMap::from([("alice".into(), ALICE.into()), ("bob".into(), BOB.into())]),
        vec![],
        vec![],
    )
    .unwrap()
}

#[test]
fn http_auth_validates_credentials_host_origin_and_proxy_header_separation() {
    let policy = security(HttpService::Mcp);
    let mut headers = HeaderMap::new();
    headers.insert("host", "localhost:8790".parse().unwrap());
    assert!(policy.validate_origin_and_host(&headers).is_ok());
    assert_eq!(
        policy.authenticate_headers(&headers).unwrap_err(),
        StatusCode::UNAUTHORIZED
    );
    headers.insert("authorization", format!("Bearer {ALICE}").parse().unwrap());
    assert_eq!(
        policy
            .authenticate_headers(&headers)
            .unwrap()
            .unwrap()
            .user_id,
        "alice"
    );
    headers.insert("origin", "https://evil.example".parse().unwrap());
    assert_eq!(
        policy.validate_origin_and_host(&headers),
        Err(StatusCode::FORBIDDEN)
    );
    headers.remove("origin");
    headers.insert("host", "evil.example:8790".parse().unwrap());
    assert_eq!(
        policy.validate_origin_and_host(&headers),
        Err(StatusCode::FORBIDDEN)
    );
    headers.insert("host", "localhost:8790".parse().unwrap());
    headers.append("host", "evil.example".parse().unwrap());
    assert_eq!(
        policy.validate_origin_and_host(&headers),
        Err(StatusCode::BAD_REQUEST)
    );

    let proxy = security(HttpService::Proxy);
    assert_eq!(
        proxy.authenticate_headers(&headers).unwrap_err(),
        StatusCode::UNAUTHORIZED
    );
    headers.insert("x-sovereign-key", BOB.parse().unwrap());
    assert_eq!(
        proxy
            .authenticate_headers(&headers)
            .unwrap()
            .unwrap()
            .user_id,
        "bob"
    );
    headers.insert("x-api-key", ALICE.parse().unwrap());
    assert_eq!(
        policy.authenticate_headers(&headers).unwrap_err(),
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn unsafe_http_startup_is_rejected_and_local_operator_mode_is_preserved() {
    let local = HttpSecurity::new(
        "127.0.0.1:8790".parse().unwrap(),
        HttpService::Mcp,
        BTreeMap::new(),
        vec![],
        vec![],
    )
    .unwrap();
    assert!(!local.requires_auth());
    assert!(local
        .authenticate_headers(&HeaderMap::new())
        .unwrap()
        .is_none());
    assert!(HttpSecurity::new(
        "0.0.0.0:8790".parse().unwrap(),
        HttpService::Mcp,
        BTreeMap::new(),
        vec![],
        vec![]
    )
    .is_err());
    assert!(HttpSecurity::new(
        "0.0.0.0:8790".parse().unwrap(),
        HttpService::Mcp,
        BTreeMap::from([("alice".into(), ALICE.into())]),
        vec![],
        vec![]
    )
    .is_err());
    assert!(HttpSecurity::new(
        "127.0.0.1:8790".parse().unwrap(),
        HttpService::Mcp,
        BTreeMap::from([("alice".into(), "short".into())]),
        vec![],
        vec![]
    )
    .is_err());
    assert!(HttpSecurity::new(
        "127.0.0.1:8790".parse().unwrap(),
        HttpService::Mcp,
        BTreeMap::from([("alice".into(), ALICE.into()), ("bob".into(), ALICE.into())]),
        vec![],
        vec![]
    )
    .is_err());
}

async fn app() -> (axum::Router, tempfile::TempDir, Arc<Database>) {
    let directory = tempfile::tempdir().unwrap();
    let url = format!(
        "sqlite://{}?mode=rwc",
        directory.path().join("test.db").display()
    );
    let db = Arc::new(Database::new(&url).await.unwrap());
    db.migrate().await.unwrap();
    (
        http_mcp::router(
            db.clone(),
            Arc::new(security(HttpService::Mcp)),
            SovereignConfig::default(),
        ),
        directory,
        db,
    )
}

fn request(path: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("host", "localhost:8790")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", "2025-03-26")
        .body(Body::from(body.to_string()))
        .unwrap()
}
async fn rpc(app: &axum::Router, token: &str, method: &str, params: Value) -> Value {
    let response = app
        .clone()
        .oneshot(request(
            "/mcp",
            token,
            json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}),
        ))
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice(&body).unwrap()
}
async fn tool(app: &axum::Router, token: &str, name: &str, arguments: Value) -> Value {
    rpc(
        app,
        token,
        "tools/call",
        json!({"name":name,"arguments":arguments}),
    )
    .await
}
fn payload(response: &Value) -> Value {
    assert_ne!(response["result"]["isError"], true, "{response}");
    serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[tokio::test]
async fn streamable_http_auth_sizes_and_tenant_access_are_enforced() {
    let (app, _directory, db) = app().await;
    let init = rpc(&app, ALICE, "initialize", json!({"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1"}})).await;
    assert_eq!(init["result"]["protocolVersion"], "2025-03-26");
    let tools = rpc(&app, ALICE, "tools/list", json!({})).await;
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 15);
    let created = payload(
        &tool(
            &app,
            ALICE,
            "optimize_prompt",
            json!({"prompt":"Please explain Rust ownership with a short code example"}),
        )
        .await,
    );
    let id = created["prompt_id"]
        .as_str()
        .or_else(|| created["id"].as_str())
        .expect("prompt ID");
    assert_eq!(
        db.get_prompt_by_id(id).await.unwrap().unwrap().user_id,
        "alice"
    );
    for name in [
        "capture_output",
        "governance_check",
        "governance_approve",
        "get_audit_trail",
        "sign_optimization",
        "verify_signature",
        "rate_optimization",
    ] {
        let denied = tool(&app, BOB, name, json!({"prompt_id":id,"output":"secret","actor":"alice","status":"approved","signal":"positive"})).await;
        assert_eq!(denied["result"]["isError"], true, "{name}: {denied}");
    }
    for name in [
        "get_history",
        "get_stats",
        "savings_report",
        "learning_insights",
        "optimize_prompt",
    ] {
        let denied = tool(
            &app,
            BOB,
            name,
            json!({"user_id":"alice","prompt":"impersonate"}),
        )
        .await;
        assert_eq!(denied["result"]["isError"], true, "{name}: {denied}");
    }
    let own = payload(&tool(&app, ALICE, "get_history", json!({"user_id":"alice"})).await);
    assert!(own.to_string().contains(id));
    let report = payload(&tool(&app, BOB, "team_report", json!({"user_ids":[]})).await);
    assert_eq!(report["total_prompts"], 0);
    let denied = tool(&app, BOB, "team_report", json!({"user_ids":["alice"]})).await;
    assert_eq!(denied["result"]["isError"], true);
    let huge = tool(
        &app,
        ALICE,
        "count_tokens",
        json!({"text":"x".repeat(256*1024)}),
    )
    .await;
    assert_eq!(huge["result"]["isError"], true);

    let unauthorized = app
        .clone()
        .oneshot(request("/mcp", "incorrect", json!({})))
        .await
        .unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
    let too_large = app
        .clone()
        .oneshot(request("/mcp", ALICE, json!({"x":"x".repeat(1024*1024)})))
        .await
        .unwrap();
    assert_eq!(too_large.status(), StatusCode::PAYLOAD_TOO_LARGE);
    for (header, value) in [
        ("host", "attacker.example"),
        ("origin", "https://attacker.example"),
    ] {
        let mut bad = request("/mcp", ALICE, json!({}));
        bad.headers_mut().insert(header, value.parse().unwrap());
        assert_eq!(
            app.clone().oneshot(bad).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
    }
}

#[tokio::test]
async fn legacy_sse_preserves_protocol_and_binds_session_to_authenticated_principal() {
    let (app, _directory, _) = app().await;
    let get = Request::builder()
        .uri("/sse")
        .header("host", "localhost:8790")
        .header("authorization", format!("Bearer {ALICE}"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(get).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let mut stream = response.into_body().into_data_stream();
    let event = tokio::time::timeout(Duration::from_secs(5), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let event = String::from_utf8(event.to_vec()).unwrap();
    let endpoint = event
        .lines()
        .find_map(|line| line.strip_prefix("data: "))
        .unwrap();
    let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"legacy-client","version":"1"}}});
    assert_eq!(
        app.clone()
            .oneshot(request(endpoint, BOB, initialize.clone()))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.clone()
            .oneshot(request(endpoint, ALICE, initialize))
            .await
            .unwrap()
            .status(),
        StatusCode::ACCEPTED
    );
    let event = tokio::time::timeout(Duration::from_secs(10), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let event = String::from_utf8(event.to_vec()).unwrap();
    let initialized: Value = serde_json::from_str(
        event
            .lines()
            .find_map(|line| line.strip_prefix("data: "))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(initialized["result"]["protocolVersion"], "2024-11-05");
    for message in [
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    ] {
        assert_eq!(
            app.clone()
                .oneshot(request(endpoint, ALICE, message))
                .await
                .unwrap()
                .status(),
            StatusCode::ACCEPTED
        );
    }
    let event = tokio::time::timeout(Duration::from_secs(10), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let event = String::from_utf8(event.to_vec()).unwrap();
    let listed: Value = serde_json::from_str(
        event
            .lines()
            .find_map(|line| line.strip_prefix("data: "))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(listed["result"]["tools"].as_array().unwrap().len(), 15);
    let endpoint = endpoint.to_string();
    drop(stream);
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        app.oneshot(request(
            &endpoint,
            ALICE,
            json!({"jsonrpc":"2.0","id":3,"method":"ping"})
        ))
        .await
        .unwrap()
        .status(),
        StatusCode::NOT_FOUND
    );
}
