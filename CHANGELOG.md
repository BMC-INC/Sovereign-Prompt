# Changelog

All notable changes to SovereignPrompt will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.3.0] - 2026-03-22

### Added

- Dual-database support: SQLite (local default) + Postgres (remote/Docker) via sqlx `Any` driver
- `DATABASE_URL` env var for Postgres connection (backward compatible with `SOVEREIGN_DB_PATH`)
- Dockerfile (multi-stage Rust build) and docker-compose.yml with Postgres 16
- `/health` endpoint on dashboard server for Docker healthchecks
- Auth module (`src/auth.rs`) for future SSE API key authentication
- GitHub Actions CI: test, clippy, fmt, security audit
- GitHub Actions release workflow: auto-builds binaries for Linux/macOS/Windows on tag push
- MIT LICENSE file, CHANGELOG, CONTRIBUTING.md

### Changed

- Upgraded sqlx 0.7 → 0.8 (resolves RUSTSEC-2024-0363 and RUSTSEC-2026-0049 audit advisories)
- Upgraded axum 0.7 → 0.8 (unifies dependency tree with rmcp)
- Improved `.gitignore` with `target/`, `.DS_Store`, `*.db`, IDE files

### Fixed

- SQL injection vulnerability in `get_learning_insights` — refactored to parameterized queries
- Nullable column handling for sqlx `AnyRow` type strictness (`output`, `output_token_count`)
- `DATE()` SQL function replaced with `SUBSTR()` for cross-database compatibility

## [0.2.0] - 2026-03-01

### Added
- Configurable heuristics via `sovereign_prompt.toml` — toggle any of the 9 checks on/off
- Explain mode for full transparency into optimization decisions
- Savings reports with cost estimates across Claude Sonnet 4 / Opus 4 / GPT-4o / GPT-4o-mini
- Custom heuristic plugins — define your own regex-based checks in TOML config
- Learning feedback loop via `rate_optimization` and `learning_insights` tools
- Team-level analytics with `team_report` tool
- Security hardening: governance policy engine for PII/credential detection
- Injection handler modes: `warn`, `rewrite`, `reject`
- Narrated video walkthrough

### Changed
- Overhauled README layout with embedded GIFs and feature comparison table
- Improved ambiguous pronoun detection thresholds

## [0.1.0] - 2025-12-15

### Added
- Core MCP-native prompt optimization engine in pure Rust
- 9 heuristic checks: vagueness, redundancy, missing context, politeness, injection, task separation, output format, ambiguous pronouns, governance
- 15 MCP tools including `optimize_prompt`, `count_tokens`, `get_stats`, `get_history`
- 7 domain templates: general, backend, frontend, data, security, product, documentation
- 3 variant generation: precision, creative, concise
- Multi-model tokenization via tiktoken-rs (cl100k_base, o200k_base, p50k_base, r50k_base)
- SQLite persistence with full CRUD and schema migrations
- SHA-256 content hashing and HMAC-SHA256 cryptographic signing
- Cryptographic audit trail with tamper detection
- SSE transport support alongside stdio
- Live Axum dashboard with WebSocket streaming
- 63 integration tests
- `#![deny(unsafe_code)]` enforced at crate root
- Zero cloud dependencies, zero telemetry
