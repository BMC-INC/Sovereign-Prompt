# Changelog

All notable changes to SovereignPrompt will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
