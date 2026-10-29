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

The current Cargo packages are version **0.1.0**. Historical feature announcements in the [changelog](CHANGELOG.md) use a separate sequence; they are not evidence of published binary releases or production certification.

## What you can evaluate

- Nine configurable heuristic categories, including vague wording, repetition, missing context, format signals, and sensitive-data patterns.
- Domain templates and three alternative refinements, with optional explanations.
- Token counts using `cl100k_base`, `o200k_base`, `p50k_base`, and `r50k_base` encodings.
- SQLite or PostgreSQL persistence for prompt history, output capture, ratings, governance status, and audit events.
- SHA-256 hashes and HMAC-SHA256 record authentication using a key you control.
- A browser dashboard for stored statistics and recent records.

Refinement can add structure and **increase** token counts. Token deltas are measurements of the selected encodings, not guarantees of lower API bills or better model answers. Cost reports use bundled estimates; verify them against your provider's current pricing and evaluate output quality on your own tasks.

## Quick start

Install a current stable Rust toolchain, then build from the repository root:

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

Clients must support the server's transport and tools. Compatibility with every named MCP product has not been certified.

## MCP tools

| Area | Tools |
| --- | --- |
| Analyze and refine | `optimize_prompt`, `count_tokens`, `list_templates` |
| Review usage | `get_stats`, `get_history`, `savings_report`, `team_report` |
| Capture feedback | `capture_output`, `rate_optimization`, `learning_insights` |
| Review governance | `governance_check`, `governance_approve`, `get_audit_trail` |
| Check integrity | `sign_optimization`, `verify_signature` |

Governance status is stored with a record. It is a review signal, not a complete data-loss-prevention or authorization system. The MCP optimizer may return a record marked rejected; callers must inspect that status before using the text.

## HTTP proxy

```bash
./target/release/sovereign-proxy
```

The proxy listens on `127.0.0.1:8788`. It exposes `/health`, `/v1/messages`, and `/v1/chat/completions`. Point a compatible HTTP client at this local endpoint while retaining the credentials required by your upstream provider. It does not implement the OpenAI Responses API or arbitrary provider routes.

| Environment variable | Default |
| --- | --- |
| `SOVEREIGN_PROXY_ADDR` | `127.0.0.1:8788` |
| `SOVEREIGN_UPSTREAM_ANTHROPIC` | `https://api.anthropic.com` |
| `SOVEREIGN_UPSTREAM_OPENAI` | `https://api.openai.com` |

The adapter optimizes the last user message's text, preserves other request fields and non-text blocks, and forwards the request. Critical sensitive-data matches or configured injection rejection return HTTP 403 before forwarding. Requests without user text pass through; this is not a whole-conversation policy gateway.

Streaming responses pass through without output capture. Non-streaming text responses are captured on a best-effort basis. Upstream credentials, prompts, and requests are sent to the configured provider; only the refinement engine itself operates without an LLM call.

## Configuration and storage

Copy [`sovereign-prompt/sovereign_prompt.toml`](sovereign-prompt/sovereign_prompt.toml) to the working directory, or set `SOVEREIGN_CONFIG_PATH` to its path. Injection modes are `warn` (default), `rewrite`, and `reject`. Review configuration loading logs: an invalid file currently falls back to defaults.

Both services accept `DATABASE_URL` for SQLite or PostgreSQL. `SOVEREIGN_DB_PATH` is the SQLite path fallback. Original prompts, refinements, and captured outputs are stored without application-level encryption; choose storage, access, backup, and retention controls appropriate to your data.

Set a private `SOVEREIGN_HMAC_KEY` before relying on record authentication. The bundled development key is public. HMAC verifies possession of a shared key; it does not establish independent signer identity or provide non-repudiation.

## Optional local Docker setup

```bash
docker compose up --build
```

The development compose file builds from the workspace root, starts PostgreSQL, and publishes the dashboard and legacy SSE endpoint on loopback only. Its database credentials and fallback signing key are development defaults. Stop with `docker compose down`; the named database volume persists.

Legacy SSE is available at `http://127.0.0.1:8790/sse` when `SOVEREIGN_MCP_TRANSPORT=sse`. The HTTP services do not provide complete authentication or tenant isolation. Do not expose them publicly without an authenticated gateway, host/origin validation, and a reviewed deployment design. Prefer stdio for local MCP use. See [security boundaries and dependency caveats](SECURITY.md).

## Development and validation

```bash
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Tests cover analysis, persistence, governance, tokenization, signatures, API-message rewriting, and proxy rejection behavior. Provider compatibility, real model quality, deployment isolation, and all release targets require separate integration validation.

[Architecture notes](sovereign-prompt/README.md) · [Contribution guide](CONTRIBUTING.md) · [Issues](https://github.com/BMC-INC/Sovereign-Prompt/issues)

## License

[MIT](LICENSE). Built by [ExecLayer / BMC-INC](https://github.com/BMC-INC).
