# SovereignPrompt core

This crate provides the MCP server and the shared prompt-analysis engine. Start with the [project README](../README.md) for installation, use, and current limitations.

## Modules

| Module | Responsibility |
| --- | --- |
| `server.rs` | Fifteen MCP tools, argument handling, identity-scoped data access, and stdio startup |
| `http_mcp.rs` | Stateless Streamable HTTP and bounded legacy SSE compatibility |
| `auth.rs` | Shared HTTP credentials, principal binding, Host/Origin validation, and body limits |
| `analyzer.rs` | Configurable prompt heuristics and explanations |
| `optimizer.rs` | Text refinement, injection-pattern stripping, and alternative variants |
| `templates.rs` | Domain-specific structure and constraints |
| `tokenizer.rs` | Counts using the supported tiktoken encodings |
| `db.rs` | SQLite/PostgreSQL persistence, migrations, history, and reports |
| `governance.rs` | Sensitive-pattern findings and initial review status |
| `crypto.rs` | SHA-256 hashing and shared-key HMAC authentication |
| `dashboard.rs` | Local HTTP dashboard and live statistics |
| `config.rs` | TOML configuration and defaults |
| `types.rs` | Records, findings, and report types |

## Optimization flow

The MCP handler loads the request context, applies the configured injection mode, analyzes the text, refines it, applies a domain template, and counts the original and refined tokens. It creates variants, records governance findings, hashes the content, and persists the record and audit events.

Governance status is part of the result and does not prevent a caller from using rejected text. Consumers must inspect it. See the [security boundaries](../SECURITY.md).

## Develop from the workspace root

```bash
cargo test --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Integration and transport tests are in [`tests/`](tests/), including a captured pre-migration tool-schema contract and a stdio protocol compatibility and version-fallback matrix. The proxy has separate message-rewriting and rejection tests in its own crate.

The Dockerfile is stored here but expects the repository root as its build context:

```bash
docker build -f sovereign-prompt/Dockerfile .
```
