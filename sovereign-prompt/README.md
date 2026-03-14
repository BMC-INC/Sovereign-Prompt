<div align="center">

<br>

<img src="https://img.shields.io/badge/RUST-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
<img src="https://img.shields.io/badge/MCP-Native-6c5ce7?style=for-the-badge" alt="MCP Native" />
<img src="https://img.shields.io/badge/Zero%20Unsafe%20Code-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Zero Unsafe Code" />
<img src="https://img.shields.io/badge/63%20Tests-2ecc71?style=for-the-badge" alt="63 Tests" />

<br><br>

# sovereign-prompt

### Developer Reference

**The Rust crate behind SovereignPrompt. This document covers architecture, module responsibilities,<br>configuration, database schema, and everything you need to contribute or extend.**

For the product overview, see the [root README](../README.md).

<br>

</div>

---

## Architecture at a Glance

```
                    ┌──────────────────────────────────────────────┐
                    │              MCP Client Request              │
                    └──────────────────┬───────────────────────────┘
                                       │
                              ┌────────▼────────┐
                              │   server.rs      │  12 MCP tools, schema validation,
                              │   (entry point)  │  injection mode routing
                              └──┬──────┬──────┬─┘
                                 │      │      │
                    ┌────────────▼┐  ┌──▼───┐  ├──────────────┐
                    │ analyzer.rs │  │opt.rs│  │ templates.rs  │
                    │ 9 checks    │  │refine│  │ 7 domains     │
                    │ + explain   │  │strip │  │ + constraints │
                    └──────┬──────┘  └──┬───┘  └──────┬────────┘
                           │            │             │
                    ┌──────▼────────────▼─────────────▼──────┐
                    │            tokenizer.rs                 │
                    │   cl100k / o200k / p50k / r50k          │
                    └──────────────────┬──────────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
       ┌──────▼──────┐         ┌───────▼──────┐         ┌──────▼──────┐
       │  crypto.rs   │         │    db.rs      │         │governance.rs│
       │ SHA-256      │         │ SQLite CRUD   │         │ PII/cred    │
       │ HMAC-SHA256  │         │ audit trail   │         │ detection   │
       └──────────────┘         │ savings rpt   │         └─────────────┘
                                └───────────────┘
```

The entire pipeline is **deterministic** — no randomness, no LLM calls, no network egress. Same input always produces the same output. This makes it testable, auditable, and predictable.

---

## Module Breakdown

| Module | Lines | Responsibility |
|:-------|------:|:---------------|
| `server.rs` | ~600 | MCP `ServerHandler` impl. 15 tools with JSON schema validation. Routes injection modes (warn/rewrite/reject). Bridges analyzer, optimizer, templates, tokenizer, crypto, governance, and DB. |
| `analyzer.rs` | ~350 | 9 heuristic checks, each accepting `&HeuristicsConfig`. `analyze()` for backward compat, `analyze_with_config()` for runtime config, `analyze_explained()` for full transparency mode. |
| `optimizer.rs` | ~110 | Politeness stripping via compiled regex (`OnceLock`). Whitespace normalization. Format instruction injection. 3-variant generation (Precision/Creative/Concise). Injection pattern stripping for rewrite mode. |
| `config.rs` | ~120 | `SovereignConfig` deserialized from TOML. `HeuristicsConfig` with per-check toggles, 5 thresholds, custom pattern lists. `InjectionMode` enum (Warn/Rewrite/Reject). Loads from `SOVEREIGN_CONFIG_PATH` env or `./sovereign_prompt.toml`, falls back to sane defaults. |
| `db.rs` | ~500 | Async SQLite via `sqlx`. Schema migration with backward-compatible `ALTER TABLE` upgrades. CRUD for prompts, output capture, governance status, signatures. `get_savings_report()` with date filtering, daily aggregation, and multi-model cost estimates. |
| `types.rs` | ~210 | All data structures: `PromptRecord`, `OptimizeResponse`, `FeedbackItem`, `Severity`, `PromptVariant`, `AuditLogEntry`, `UserStats`, `HeuristicExplanation`, `SavingsReport`, `CostEstimate`, `DailyTrend`. |
| `templates.rs` | ~120 | 7 domain templates (general, backend, frontend, data, security, product, documentation). Each applies domain-specific constraints to refined prompts. |
| `tokenizer.rs` | ~50 | Wraps `tiktoken-rs`. Supports 4 models. `count()`, `count_for_model()`, `count_across_models()`. |
| `crypto.rs` | ~70 | `CryptoEngine` for HMAC-SHA256 signing/verification. Static methods for SHA-256 content hashing, output hashing, and hash chain verification. |
| `governance.rs` | ~110 | Policy v1.0.0 with regex-based detection: SSN, credit cards, credentials, PII references. `determine_status()` maps severity to approved/pending/rejected. |
| `dashboard.rs` | ~350 | Axum web server. Embedded HTML dashboard with dark theme. REST API (`/api/stats`, `/api/history`). WebSocket stream (`/ws/analytics`) updating every 2s. |
| `main.rs` | ~75 | Entry point. Loads `.env`, inits DB, spawns dashboard, selects transport (stdio/SSE/dashboard-only), handles graceful shutdown. |
| `lib.rs` | ~12 | `#![deny(unsafe_code)]` + module re-exports. |

---

## Data Flow — optimize_prompt

```
1. Parse args (prompt, user_id, domain, token_model, explain_mode)
2. Check injection mode:
   - Reject → scan for patterns, return error if found
   - Rewrite → strip injection patterns via regex
   - Warn → pass through unchanged
3. Count original tokens (selected model)
4. Run 9 heuristic checks against HeuristicsConfig
   - If explain_mode: also build HeuristicExplanation for each check
5. Refine prompt (strip politeness, normalize whitespace, add format hint)
6. Apply domain template (constraints injected)
7. Count refined tokens
8. Generate 3 variants with token counts
9. Count across all 4 tokenizer models (original + refined)
10. Compute SHA-256 content hash
11. Run governance check → determine approval status
12. Persist PromptRecord to SQLite
13. Write audit log entry
14. Return OptimizeResponse (+ heuristic_explanations if explain_mode)
```

---

## Configuration

The config file is optional. Without it, every default is production-ready.

```toml
[heuristics]
# Toggle individual checks
vagueness = true
redundancy = true
missing_context = true
politeness = true
injection = true
task_separation = true
output_format = true
ambiguous_pronouns = true
governance = true

# Thresholds
redundancy_word_repeat = 3     # Flag words repeated > N times
pronoun_threshold = 3          # Flag when pronoun count >= N
context_min_length = 50        # Min chars for "has context"
conjunction_threshold = 2      # Flag when conjunction count >= N
format_min_length = 30         # Min chars before flagging missing format

# Extend built-in patterns
# extra_vague_terms = ["unclear"]
# extra_injection_patterns = ["bypass safety"]
# extra_polite_terms = ["excuse me"]

[injection]
mode = "warn"  # "warn" | "rewrite" | "reject"
```

**Loading priority:** `SOVEREIGN_CONFIG_PATH` env var > `./sovereign_prompt.toml` > built-in defaults.

---

## SQLite Schema

Two tables with backward-compatible schema upgrades (`ALTER TABLE` with duplicate-column guards):

```sql
CREATE TABLE IF NOT EXISTS prompts (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL,
    domain              TEXT NOT NULL DEFAULT 'general',
    token_model         TEXT NOT NULL DEFAULT 'cl100k_base',
    original_prompt     TEXT NOT NULL,
    original_token_count INTEGER NOT NULL,
    refined_prompt      TEXT NOT NULL,
    refined_token_count INTEGER NOT NULL,
    savings_percentage  REAL NOT NULL,
    analysis_feedback   TEXT NOT NULL,       -- JSON array of FeedbackItem
    output              TEXT,                -- captured AI response
    output_token_count  INTEGER,
    created_at          TEXT NOT NULL,       -- RFC 3339
    governance_id       TEXT,                -- UUID linking governance context
    policy_version      TEXT,                -- e.g. "v1.0.0"
    approval_status     TEXT,                -- pending | approved | rejected
    content_hash        TEXT,                -- SHA-256(original || "||" || refined)
    output_hash         TEXT,                -- SHA-256(output)
    signature           TEXT,                -- HMAC-SHA256 hex signature
    signed_at           TEXT                 -- RFC 3339 signing timestamp
);

CREATE TABLE IF NOT EXISTS audit_log (
    id                  TEXT PRIMARY KEY,
    prompt_id           TEXT NOT NULL,
    action              TEXT NOT NULL,       -- created | approved | rejected | signed | captured
    actor               TEXT NOT NULL,       -- user_id or "system"
    detail              TEXT NOT NULL,       -- JSON metadata
    created_at          TEXT NOT NULL
);
```

Indices on `user_id`, `created_at`, `prompt_id`, and `action` for query performance.

---

## Project Structure

```
sovereign-prompt/
├── Cargo.toml                 # 15 deps + 1 dev-dep
├── .env.example
├── sovereign_prompt.toml      # Example config — all defaults shown
├── src/
│   ├── main.rs                # Entry: dotenv, DB, config, dashboard, transport
│   ├── lib.rs                 # #![deny(unsafe_code)] + mod re-exports
│   ├── server.rs              # 12 MCP tools + ServerHandler impl
│   ├── analyzer.rs            # 9 heuristic checks + explain mode
│   ├── optimizer.rs           # Refinement engine + injection stripping
│   ├── config.rs              # TOML config: HeuristicsConfig, InjectionMode
│   ├── templates.rs           # 7 domain templates
│   ├── tokenizer.rs           # Multi-model token counting
│   ├── crypto.rs              # SHA-256 + HMAC-SHA256
│   ├── governance.rs          # PII/credential detection + policy engine
│   ├── dashboard.rs           # Axum + WebSocket analytics
│   ├── types.rs               # All data structures
│   └── db.rs                  # SQLite: migrations, CRUD, savings reports
├── assets/
│   ├── how-it-works.gif
│   └── optimization-pipeline.gif
└── tests/
    └── integration_test.rs    # 63 tests
```

---

## Running Tests

```bash
cargo test
```

```
running 63 tests
test config_disabled_check_skips_analysis ... ok
test config_custom_threshold_changes_behavior ... ok
test config_custom_patterns_detected ... ok
test config_injection_rewrite_strips_patterns ... ok
test config_injection_reject_mode_detects ... ok
test explain_mode_returns_all_nine_explanations ... ok
test explain_mode_fired_accuracy ... ok
test explain_mode_with_config_interaction ... ok
test db_savings_report_query ... ok
test db_savings_report_cost_calculation ... ok
test db_savings_report_empty_state ... ok
...
test result: ok. 63 passed; 0 failed
```

Test coverage spans: tokenizer (4), analyzer (10), optimizer (6), templates (1), types (3), database (11), crypto (7), governance (5), config (5), explain mode (3), savings report (3).

```bash
cargo clippy -- -D warnings   # Zero warnings enforced
```

---

## Environment Variables

| Variable | Default | Description |
|:---------|:--------|:------------|
| `SOVEREIGN_DB_PATH` | `./sovereign_prompt.db` | SQLite database path |
| `SOVEREIGN_CONFIG_PATH` | `./sovereign_prompt.toml` | TOML config path |
| `SOVEREIGN_MCP_TRANSPORT` | `stdio` | Transport: `stdio` or `sse` |
| `SOVEREIGN_MCP_SSE_ADDR` | `127.0.0.1:8790` | SSE bind address |
| `SOVEREIGN_DASHBOARD_ADDR` | `127.0.0.1:8787` | Dashboard bind address |
| `SOVEREIGN_DASHBOARD_ONLY` | `false` | Dashboard-only mode (no MCP transport) |
| `SOVEREIGN_HMAC_KEY` | _(dev default)_ | HMAC signing key |
| `RUST_LOG` | _(none)_ | `info`, `debug`, `trace` |

---

<div align="center">

<br>

Built by [**ExecLayer Inc.**](https://github.com/BMC-INC) &#8226; MIT License

<br>

</div>
