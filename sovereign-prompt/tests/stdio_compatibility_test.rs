use serde_json::{json, Value};
use std::{process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
};

async fn rpc(
    input: &mut ChildStdin,
    output: &mut tokio::io::Lines<BufReader<ChildStdout>>,
    value: Value,
) -> Value {
    input
        .write_all(format!("{value}\n").as_bytes())
        .await
        .unwrap();
    let line = tokio::time::timeout(Duration::from_secs(20), output.next_line())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    serde_json::from_str(&line).expect("stdout must contain only MCP JSON-RPC")
}

#[tokio::test]
async fn stdio_preserves_tool_contract_and_negotiates_client_protocol_versions() {
    let baseline: Value = serde_json::from_str(include_str!("fixtures/mcp_tools.json")).unwrap();
    for (client, version, expected) in [
        ("claude-desktop", "2024-11-05", "2024-11-05"),
        ("claude-code", "2025-03-26", "2025-03-26"),
        ("codex", "2025-11-25", "2025-11-25"),
        ("future-client", "2026-07-28", "2025-11-25"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_sovereign-prompt"))
            .current_dir(directory.path())
            .env("SOVEREIGN_MCP_TRANSPORT", "stdio")
            .env("SOVEREIGN_DASHBOARD_ADDR", "127.0.0.1:0")
            .env(
                "DATABASE_URL",
                format!(
                    "sqlite://{}?mode=rwc",
                    directory.path().join("test.db").display()
                ),
            )
            .env_remove("SOVEREIGN_CONFIG_PATH")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap()).lines();
        let initialized = rpc(&mut input, &mut output, json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":version,"capabilities":{},"clientInfo":{"name":client,"version":"compatibility-test"}}})).await;
        assert_eq!(
            initialized["result"]["protocolVersion"], expected,
            "{client}: {initialized}"
        );
        input
            .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
            .await
            .unwrap();
        let tools = rpc(
            &mut input,
            &mut output,
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        )
        .await;
        assert_eq!(
            tools["result"]["tools"], baseline,
            "{client}: tool contract changed"
        );
        let optimized = rpc(&mut input, &mut output, json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"optimize_prompt","arguments":{"user_id":"compatibility-user","prompt":"Please explain Rust ownership with an example"}}})).await;
        assert!(optimized.get("error").is_none(), "{client}: {optimized}");
        assert_ne!(
            optimized["result"]["isError"], true,
            "{client}: {optimized}"
        );
        assert!(optimized["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("refined_prompt"));
        child.kill().await.unwrap();
    }
}
