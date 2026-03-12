<div align="center">

<br>

<img src="https://img.shields.io/badge/RUST-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
<img src="https://img.shields.io/badge/MCP-Native-6c5ce7?style=for-the-badge" alt="MCP Native" />
<img src="https://img.shields.io/badge/SQLite-Persistence-003B57?style=for-the-badge&logo=sqlite&logoColor=white" alt="SQLite" />
<img src="https://img.shields.io/badge/License-MIT-00cec9?style=for-the-badge" alt="MIT License" />
<img src="https://img.shields.io/badge/Tests-27%20Passing-2ecc71?style=for-the-badge" alt="Tests Passing" />

<br><br>

# SovereignPrompt

### Stop wasting tokens. Start shipping precision.

**Real-time MCP prompt optimization engine with heuristic analysis,<br>accurate token counting, and full persistence.**

Built by [ExecLayer Inc.](https://github.com/BMC-INC)

<br>

https://github.com/user-attachments/assets/SovereignPrompt.mp4

<br>

</div>

---

## The Problem

Every prompt you send to an LLM costs tokens. Most prompts are bloated with politeness filler, vague language, redundant phrasing, and missing structure. You're paying for noise.

```
"Hey, could you please kindly help me out and maybe possibly write
 something that sort of fixes the thing with the stuff? Thank you
 so much! Also additionally can you do the other thing as well?"
```

**That's 50+ tokens of pure waste.** SovereignPrompt catches it, strips it, and rewrites it before it ever hits the model.

---

## How It Works

```
                    ┌─────────────────────────────────────────┐
                    │           MCP Client Request             │
                    └──────────────────┬──────────────────────┘
                                       │
                                       ▼
                    ┌─────────────────────────────────────────┐
                    │         SovereignPrompt Server           │
                    │                                         │
                    │  ┌──────────┐  ┌──────────┐  ┌───────┐ │
                    │  │ Analyzer │→ │ Optimizer │→ │Tokenizer│
                    │  │ 8 checks │  │ refine +  │  │cl100k │ │
                    │  │          │  │ variants  │  │ _base │ │
                    │  └──────────┘  └──────────┘  └───────┘ │
                    │         │                        │      │
                    │         ▼                        ▼      │
                    │  ┌─────────────────────────────────┐    │
                    │  │     SQLite Persistence Layer     │    │
                    │  │  prompts · outputs · analytics   │    │
                    │  └─────────────────────────────────┘    │
                    └─────────────────────────────────────────┘
                                       │
                                       ▼
                    ┌─────────────────────────────────────────┐
                    │   Optimized Prompt + Feedback + Variants │
                    └─────────────────────────────────────────┘
```

---

## Features

<table>
<tr>
<td width="50%">

**Heuristic Prompt Analysis**
8 specialized checks run on every prompt — vagueness, redundancy, missing context, politeness tokens, prompt injection, task separation, output format, and ambiguous pronouns. Each returns severity-graded feedback with actionable suggestions.

</td>
<td width="50%">

**Accurate Token Counting**
Uses `tiktoken-rs` with the `cl100k_base` encoding (same tokenizer as GPT-4 / Claude). Every optimization reports exact before/after token counts and savings percentage.

</td>
</tr>
<tr>
<td width="50%">

**3 Prompt Variants Per Request**
Every optimization generates three tailored variants — **Precision** (technical, minimal), **Creative** (exploratory, multi-angle), and **Concise** (stripped to essentials) — each with its own token count.

</td>
<td width="50%">

**Full Persistence + Analytics**
Every prompt, refinement, and AI output is stored in SQLite. Query per-user stats including total tokens saved, average savings percentage, and top recurring issues.

</td>
</tr>
</table>

---

## MCP Tools

| Tool | Description | Key Parameters |
|:-----|:------------|:---------------|
| **`optimize_prompt`** | Analyze and refine a prompt. Returns feedback, refined version, token savings, and 3 variants. | `prompt` (required), `user_id` (optional) |
| **`capture_output`** | Store the AI's response against a prompt ID for output tracking and learning. | `prompt_id`, `output` |
| **`get_stats`** | Retrieve per-user optimization metrics — total prompts, tokens saved, average savings, top issues. | `user_id` |
| **`get_history`** | Fetch recent prompt history with full refinement data. | `user_id`, `limit` (max 50) |

---

## 8 Heuristic Checks

| Check | Severity | What It Catches |
|:------|:---------|:----------------|
| **Vagueness Detection** | Warning | `"something"`, `"stuff"`, `"kind of"`, `"maybe"`, `"whatever"` — 14 vague terms |
| **Redundancy Analysis** | Info | Words repeated 3+ times in a single prompt |
| **Missing Context** | Critical | Action verbs (`fix`, `update`, `change`) with insufficient detail (<50 chars) |
| **Politeness Tokens** | Info | `"please"`, `"kindly"`, `"could you"`, `"thank you"` — 7 filler patterns |
| **Prompt Injection** | Critical | `"ignore previous"`, `"forget everything"`, `"system:"`, `"jailbreak"` — 8 patterns |
| **Task Separation** | Warning | Multiple conjunctions (`"and then"`, `"additionally"`, `"as well as"`) indicating bundled tasks |
| **Output Format** | Info | No format signal detected (`json`, `list`, `table`, `code`, etc.) in prompts >30 chars |
| **Ambiguous Pronouns** | Warning | 3+ unresolved pronouns (`"it"`, `"this"`, `"they"`, `"those"`) in a single prompt |

---

## Optimization Pipeline

```
Input: "Please kindly help me fix the thing with the stuff"
                              │
                    ┌─────────▼──────────┐
                    │  Strip Politeness   │  ← case-insensitive regex
                    │  "help me fix the   │     with word boundaries
                    │   thing with the    │
                    │   stuff"            │
                    └─────────┬──────────┘
                              │
                    ┌─────────▼──────────┐
                    │ Normalize Whitespace│  ← collapse runs, trim
                    └─────────┬──────────┘
                              │
                    ┌─────────▼──────────┐
                    │ Append Format Hint  │  ← if no format signal
                    │ (when no critical   │     and length > 30
                    │  issues detected)   │
                    └─────────┬──────────┘
                              │
                    ┌─────────▼──────────┐
                    │ Generate 3 Variants │
                    │  ├─ Precision       │
                    │  ├─ Creative        │
                    │  └─ Concise         │
                    └────────────────────┘
```

---

## Tech Stack

| Component | Crate | Purpose |
|:----------|:------|:--------|
| **MCP Transport** | `rmcp 0.1` | Native Model Context Protocol server via stdio |
| **Tokenizer** | `tiktoken-rs 0.5` | `cl100k_base` BPE encoding for accurate token counts |
| **Database** | `sqlx 0.7` | Async SQLite with runtime queries, zero compile-time DB needed |
| **Runtime** | `tokio 1` | Full-featured async runtime with signal handling |
| **Serialization** | `serde 1` + `serde_json 1` | JSON serialization for MCP protocol and persistence |
| **Regex** | `regex 1` | Cached via `OnceLock` for politeness stripping and pronoun detection |
| **IDs** | `uuid 1` | V4 UUIDs for prompt record identifiers |
| **Logging** | `tracing 0.1` | Structured logging with env-filter support |
| **Environment** | `dotenvy 0.15` | `.env` file loading at startup |

---

## Project Structure

```
sovereign-prompt/
├── Cargo.toml
├── .env.example
├── .gitignore
├── src/
│   ├── main.rs          # Entry point — dotenv, DB init, signal handling
│   ├── lib.rs           # Library re-exports for integration tests
│   ├── server.rs        # MCP ServerHandler — 4 tools, schema definitions
│   ├── analyzer.rs      # 8 heuristic checks with cached regex
│   ├── optimizer.rs     # Politeness stripping, normalization, variant generation
│   ├── tokenizer.rs     # tiktoken cl100k_base wrapper
│   ├── types.rs         # PromptRecord, OptimizeResponse, FeedbackItem, UserStats
│   └── db.rs            # SQLite — migrations, CRUD, stats aggregation
└── tests/
    └── integration_test.rs  # 27 tests across all modules
```

---

## Quick Start

### 1. Build

```bash
git clone https://github.com/BMC-INC/Sovereign-Prompt.git
cd Sovereign-Prompt/sovereign-prompt
cp .env.example .env
cargo build --release
```

### 2. Configure MCP Client

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "sovereign-prompt": {
      "command": "/absolute/path/to/sovereign-prompt/target/release/sovereign-prompt",
      "env": {
        "SOVEREIGN_DB_PATH": "/absolute/path/to/sovereign_prompt.db",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 3. Run Tests

```bash
cargo test
```

```
running 27 tests
test analyzer_detects_vagueness ... ok
test analyzer_detects_politeness ... ok
test analyzer_detects_prompt_injection ... ok
test optimizer_strips_politeness ... ok
test optimizer_generates_three_variants ... ok
test db_insert_and_query_stats ... ok
test db_top_issues_populated ... ok
...
test result: ok. 27 passed; 0 failed
```

---

## SQLite Schema

```sql
CREATE TABLE IF NOT EXISTS prompts (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL,
    original_prompt     TEXT NOT NULL,
    original_token_count INTEGER NOT NULL,
    refined_prompt      TEXT NOT NULL,
    refined_token_count INTEGER NOT NULL,
    savings_percentage  REAL NOT NULL,
    analysis_feedback   TEXT NOT NULL,       -- JSON array
    output              TEXT,                -- captured AI response
    output_token_count  INTEGER,
    created_at          TEXT NOT NULL        -- RFC 3339
);
```

---

## Environment Variables

| Variable | Default | Description |
|:---------|:--------|:------------|
| `SOVEREIGN_DB_PATH` | `./sovereign_prompt.db` | Path to SQLite database file |
| `RUST_LOG` | _(none)_ | Tracing filter level (`info`, `debug`, `trace`) |

---

## Roadmap

- [ ] ExecLayer SovereignClaw governance integration
- [ ] Cryptographic execution binding and audit trails
- [ ] Prompt template library with per-domain optimization
- [ ] Multi-model token counting (o200k_base, etc.)
- [ ] WebSocket transport support
- [ ] Dashboard UI for analytics

---

<div align="center">

<br>

**SovereignPrompt** is built by [ExecLayer Inc.](https://github.com/BMC-INC)

MIT License

<br>

</div>
