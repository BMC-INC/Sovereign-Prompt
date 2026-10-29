# SovereignPrompt

Local prompt analysis, deterministic refinement, and an inspectable history of what changed before a model request.

[![CI](https://github.com/BMC-INC/Sovereign-Prompt/actions/workflows/ci.yml/badge.svg)](https://github.com/BMC-INC/Sovereign-Prompt/actions/workflows/ci.yml)
[![MIT license](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[Quick start](#quick-start) · [MCP tools](#mcp-tools) · [HTTP proxy](#http-proxy) · [Security](SECURITY.md) · [Contributing](CONTRIBUTING.md)

## Purpose and current status

SovereignPrompt helps developers examine prompt quality, compare token counts, and keep before-and-after records on infrastructure they control. It uses configurable rules and templates rather than calling another model to rewrite the prompt.

This repository contains two working Rust services intended for **local evaluation and controlled integration**:

| Component | Role |
| --- | --- |
| `sovereign-prompt` | MCP server with explicit analysis, optimization, history, feedback, and integrity tools; includes a local dashboard |
| `sovereign-proxy` | HTTP adapter for Anthropic Messages and OpenAI-compatible Chat Completions; refines the last user message before forwarding |

The repository also includes a separate promotional website in [`web/`](web/) and video source in [`sovereign-prompt-video/`](sovereign-prompt-video/). Neither is required to run the services.

The current Cargo packages are version **0.4.0**. Earlier release tags advanced through 0.3.0 while package metadata remained at 0.1.0; this release aligns both package versions with the release tag. See the [changelog](CHANGELOG.md) for project history.

## What you can evaluate

- Nine configurable heuristic categories, including vague wording, repetition, missing context, format signals, and sensitive-data patterns.
- Domain templates and three alternative refinements, with optional explanations.
- Token counts using `cl100k_base`, `o200k_base`, `p50k_base`, and `r50k_base` encodings.
- SQLite or PostgreSQL persistence for prompt history, output capture, ratings, governance status, and audit events.
- SHA-256 hashes and HMAC-SHA256 record authentication using a key you control.
- A browser dashboard for stored statistics and recent records.

Refinement can add structure and **increase** token counts. Token deltas are measurements of the selected encodings, not guarantees of lower API bills or better model answers. Cost reports use bundled estimates; verify them against your provider's current pricing and evaluate output quality on your own tasks.

## Quick start

Install a current stable Rust toolchain (Rust 1.88 or newer), then build from the repository root:

```bash
git clone https://github.com/BMC-INC/Sovereign-Prompt.git
cd Sovereign-Prompt
cargo build --workspace --locked --release
```

Both binaries are written to `target/release/`. The default database is `sovereign_prompt.db` in the process's working directory. Set `SOVEREIGN_DB_PATH` to a persistent absolute path if your client starts processes from different directories.

Start the local dashboard without an MCP session:

```bash
SOVEREIGN_DASHBOARD_ONLY=1 ./target/release/sovereign-prompt
```

Open [localhost:8787](http://localhost:8787). Stop with Ctrl+C before starting another instance using the same dashboard port.

### Connect an MCP client

The default transport is stdio. Use the full path to the built `sovereign-prompt` binary as your client's server command, with no arguments. For a client using an `mcpServers` JSON configuration, a binary installed on its executable search path can be configured as:

```json
{
  "mcpServers": {
    "sovereign-prompt": {
      "command": "sovereign-prompt"
    }
  }
}
```

Install that command with `cargo install --locked --path sovereign-prompt`, or replace the command with the actual full path printed by `realpath target/release/sovereign-prompt`. A graphical client's search path may differ from your terminal's.

Connection makes the tools available. It does **not** automatically rewrite every conversation message: the client must call `optimize_prompt` explicitly.

Start with:

```json
{"prompt":"Summarize these release notes in five bullets for a developer audience.","domain":"documentation","explain_mode":true}
```

Stdio keeps the existing no-argument executable command and the same fifteen tool names and input schemas. Regression tests negotiate MCP `2024-11-05`, `2025-03-26`, and `2025-11-25`, and verify supported-version fallback for a newer client request. For command-line clients, after building:

```bash
claude mcp add --transport stdio sovereign-prompt -- "$(realpath target/release/sovereign-prompt)"
codex mcp add sovereign-prompt -- "$(realpath target/release/sovereign-prompt)"
```

These commands register the server with the respective client; they do not make model calls. Keep an existing `SOVEREIGN_DB_PATH` setting when updating an installed binary. Claude Desktop uses the `mcpServers` configuration above. Remote Claude.ai integrations need an authenticated HTTPS endpoint and a compatible gateway; this repository does not provide an OAuth authorization server.

## MCP tools

| Area | Tools |
| --- | --- |
| Analyze and refine | `optimize_prompt`, `count_tokens`, `list_templates` |
| Review usage | `get_stats`, `get_history`, `savings_report`, `team_report` |
| Capture feedback | `capture_output`, `rate_optimization`, `learning_insights` |
| Review governance | `governance_check`, `governance_approve`, `get_audit_trail` |
| Check integrity | `sign_optimization`, `verify_signature` |

Governance status is stored with a record. It is a review signal, not a complete data-loss-prevention or authorization system. The MCP optimizer may return a record marked rejected; callers must inspect that status before using the text.

## HTTP MCP and access control

For clients that need HTTP, start the server with an explicit API key. Keep the generated key in your secret manager or protected process environment:

```bash
export SOVEREIGN_API_KEY="$(openssl rand -hex 32)"
export SOVEREIGN_USER_ID="developer"
SOVEREIGN_MCP_TRANSPORT=http ./target/release/sovereign-prompt
```

The current transport is `http://127.0.0.1:8790/mcp`. Existing HTTP+SSE clients can continue using `http://127.0.0.1:8790/sse`; `SOVEREIGN_MCP_TRANSPORT=sse` still enables that endpoint. Both paths use the same authentication and ownership checks. Send `Authorization: Bearer` with the configured key, or `X-API-Key`. The legacy session ID is not a credential.

An HTTP key binds requests to `SOVEREIGN_USER_ID` (default `anonymous`). A caller cannot select another user's history, reports, or prompt IDs. To configure multiple identities, set `SOVEREIGN_API_KEYS` to a JSON object mapping each user ID to its own unique secret. Keys must contain at least 32 bytes. Stdio and loopback HTTP with no configured keys retain the original local operator access; use credentials whenever other local software must not share that access.

All HTTP services validate exact Host authorities and browser Origins. Non-loopback binding requires credentials and `SOVEREIGN_ALLOWED_HOSTS`, a comma-separated list of exact `host:port` authorities. `SOVEREIGN_ALLOWED_ORIGINS` optionally permits additional exact HTTP(S) origins. Use an HTTPS reverse proxy for remote access; forward the configured Host, and do not enable unrestricted origins. See [security boundaries](SECURITY.md) for limits and session behavior.

The dashboard accepts the key through its password field and creates an opaque, one-hour session cookie with `HttpOnly` and `SameSite=Strict` (`Secure` for HTTPS origins). It does not put API keys in URLs or browser storage. Reload restores the authenticated identity; signing out revokes the session.

## HTTP proxy

```bash
./target/release/sovereign-proxy
```

The proxy listens on `127.0.0.1:8788`. It exposes `/health`, `/v1/messages`, and `/v1/chat/completions`. Point a compatible HTTP client at this local endpoint while retaining the credentials required by your upstream provider. It does not implement the OpenAI Responses API or arbitrary provider routes.

When HTTP authentication is configured, supply the local credential as **`X-Sovereign-Key`**. Provider `Authorization` and `X-API-Key` headers keep their upstream meaning. The local key and browser cookies are removed before forwarding. Stored proxy records belong to the authenticated identity; uncredentialed loopback use retains the `proxy` user label.

| Environment variable | Default |
| --- | --- |
| `SOVEREIGN_PROXY_ADDR` | `127.0.0.1:8788` |
| `SOVEREIGN_UPSTREAM_ANTHROPIC` | `https://api.anthropic.com` |
| `SOVEREIGN_UPSTREAM_OPENAI` | `https://api.openai.com` |

The adapter optimizes the last user message's text, preserves other request fields and non-text blocks, and forwards the request. Critical sensitive-data matches or configured injection rejection return HTTP 403 before forwarding. Requests without user text pass through; this is not a whole-conversation policy gateway.

Streaming responses pass through without output capture. Non-streaming text responses are captured on a best-effort basis. Upstream credentials, prompts, and requests are sent to the configured provider; only the refinement engine itself operates without an LLM call.

## Configuration and storage

Copy [`sovereign-prompt/sovereign_prompt.toml`](sovereign-prompt/sovereign_prompt.toml) to the working directory, or set `SOVEREIGN_CONFIG_PATH` to its path. Injection modes are `warn` (default), `rewrite`, and `reject`. Invalid or unreadable configuration, unknown fields, and a missing explicitly selected file stop startup. Defaults apply only when no explicit path is set and the default file is absent.

Both services accept `DATABASE_URL` for SQLite or PostgreSQL. `SOVEREIGN_DB_PATH` is the SQLite path fallback. Original prompts, refinements, and captured outputs are stored without application-level encryption; choose storage, access, backup, and retention controls appropriate to your data.

Set a private `SOVEREIGN_HMAC_KEY` before relying on record authentication. The bundled development key is public. HMAC verifies possession of a shared key; it does not establish independent signer identity or provide non-repudiation.

## Optional local Docker setup

```bash
export SOVEREIGN_API_KEY="$(openssl rand -hex 32)"
export SOVEREIGN_HMAC_KEY="$(openssl rand -hex 32)"
docker compose up --build
```

The development compose file builds from the workspace root, starts PostgreSQL, and publishes the dashboard and legacy SSE endpoint on loopback only. The compose file requires the two generated keys above; database credentials remain development defaults. Stop with `docker compose down`; the named database volume persists.

Current Streamable HTTP and the legacy SSE compatibility endpoint are both available through port 8790. Use the same configured API key for MCP and dashboard access. Prefer stdio for a local desktop client; use a reviewed HTTPS gateway for remote deployment.

## Development and validation

```bash
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo audit
```

Tests cover analysis, persistence, governance, tokenization, signatures, unchanged tool schemas, three stdio protocol versions and future-version fallback, authenticated HTTP/SSE, tenant isolation, and both provider formats with mocked JSON/streaming responses. These tests do not assess real model quality or certify every remote client, deployment, and release platform.

[Architecture notes](sovereign-prompt/README.md) · [Contribution guide](CONTRIBUTING.md) · [Issues](https://github.com/BMC-INC/Sovereign-Prompt/issues)

## License

[MIT](LICENSE). Built by [ExecLayer / BMC-INC](https://github.com/BMC-INC).
