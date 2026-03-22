<div align="center">

<br>

<img src="https://img.shields.io/badge/RUST-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
<img src="https://img.shields.io/badge/MCP-Native-6c5ce7?style=for-the-badge" alt="MCP Native" />
<img src="https://img.shields.io/badge/Zero%20Unsafe%20Code-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Zero Unsafe Code" />
<img src="https://img.shields.io/badge/License-MIT-00cec9?style=for-the-badge" alt="MIT License" />

<br>

<img src="https://img.shields.io/badge/63%20Tests%20Passing-2ecc71?style=for-the-badge" alt="63 Tests" />
<img src="https://img.shields.io/badge/15%20MCP%20Tools-fd79a8?style=for-the-badge" alt="15 MCP Tools" />
<img src="https://img.shields.io/badge/9%20Heuristic%20Checks-a29bfe?style=for-the-badge" alt="9 Heuristics" />
<img src="https://img.shields.io/badge/Local%20Only-No%20Cloud-ff7675?style=for-the-badge" alt="Local Only" />
<img src="https://img.shields.io/badge/Docker%20Ready-2496ED?style=for-the-badge&logo=docker&logoColor=white" alt="Docker Ready" />
<img src="https://img.shields.io/badge/SQLite%20%2B%20Postgres-336791?style=for-the-badge&logo=postgresql&logoColor=white" alt="SQLite + Postgres" />

<br><br>

# SovereignPrompt

### Your prompts are bleeding tokens. We stop the bleed.

**The first MCP-native prompt optimization engine. 9 heuristic checks, domain-aware refinement,<br>cryptographic audit trails, and real cost savings — all running locally in pure Rust.**

<br>

https://github.com/user-attachments/assets/ee2438e2-5f89-4dc3-8cd2-1f784a8844ab

<br>

[Get Started](#get-started) &#8226; [15 Tools](#the-toolbelt--15-mcp-tools) &#8226; [Security](./SECURITY.md) &#8226; [Developer Docs](./sovereign-prompt/README.md)

<br>

</div>

---

## Get Started

**One command to build. One config to connect. Done.**

```bash
git clone https://github.com/BMC-INC/Sovereign-Prompt.git
cd Sovereign-Prompt/sovereign-prompt
cargo build --release
```

Then tell your MCP client where to find the binary:

<details>
<summary><strong>Claude Desktop</strong></summary>

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "sovereign-prompt": {
      "command": "/absolute/path/to/target/release/sovereign-prompt"
    }
  }
}
```

</details>

<details>
<summary><strong>Claude Code</strong></summary>

Add to `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "sovereign-prompt": {
      "command": "/absolute/path/to/target/release/sovereign-prompt"
    }
  }
}
```

</details>

<details>
<summary><strong>Any MCP Client (SSE)</strong></summary>

```bash
SOVEREIGN_MCP_TRANSPORT=sse ./target/release/sovereign-prompt
```

Connect at `http://127.0.0.1:8790/sse`

</details>

That's it. 15 tools are live. Start with:

```
optimize_prompt: { "prompt": "your prompt here" }
```

---

## Why This Exists

You send 100 prompts a day. Most of them are 30-60% filler — politeness tokens the model ignores, vague language that degrades output quality, redundant phrasing that burns your budget. Multiply that across a team, and you're lighting money on fire.

SovereignPrompt sits between you and the model. Every prompt passes through 9 heuristic checks, gets stripped of waste, shaped by domain-specific templates, and measured across 4 tokenizer models — before a single token reaches the API. You get the refined prompt, three strategic variants, a full token savings breakdown, and a cryptographically signed audit trail.

This isn't a wrapper. It's an optimization engine that speaks [MCP](https://modelcontextprotocol.io) — the open protocol that Claude Desktop, Claude Code, Cursor, and every major AI client is adopting. Plug it in once, and every prompt you send through any MCP client gets optimized automatically.

No LLM in the loop. No API calls. No cloud. No latency. Pure deterministic Rust.

| Feature | SovereignPrompt | Other MCP Prompt Tools |
|:--------|:---------------:|:----------------------:|
| Local-only, zero cloud | **Yes** | Usually sends to LLM |
| Real token savings % with reports | **Yes** | No |
| Cryptographic audit trail | **Yes** | No |
| Built-in PII / governance engine | **Yes** | No |
| Live dashboard | **Yes** | No |
| Custom heuristic plugins | **Yes** | No |
| Team-level analytics | **Yes** | No |
| Learning feedback loop | **Yes** | No |
| Pure Rust (sub-5ms) | **Yes** | Python / TS |

---

## How It Works

<div align="center">

![How SovereignPrompt Works](./sovereign-prompt/assets/how-it-works.gif)

</div>

Prompt in. 9 heuristic checks run. Domain template applied. 4 tokenizers count simultaneously. Optimized prompt, 3 variants, and full analytics out. Every step persisted and hashable.

---

## What Comes Out The Other Side

| What you send | What hits the model | Saved |
|:-------------|:-------------------|:-----:|
| "Please kindly help me write something that sort of fixes the bug in the code somehow" | "Fix the bug in the code. Be exact, technical, and minimal." | **~36%** |
| "Could you please maybe write something creative about nature, thank you so much" | "Write creative content about nature" | **~75%** |
| "Would you mind kindly helping me perhaps analyze the data and maybe find some patterns or something" | "Analyze the data and find patterns. Respond concisely and directly." | **~42%** |
| "Please could you help me plan out the tasks and then also break them down and additionally prioritize them" | "Plan tasks, break them down, prioritize. Respond concisely and directly." | **~48%** |

Every optimization generates **three variants** — **Precision** (tight, technical, minimal), **Creative** (broad, exploratory), and **Concise** (stripped to the bone) — each with its own token count so you can choose the right tool for the job.

Turn on **explain mode** and you see every heuristic that fired, what it matched, and the threshold it triggered — full transparency into the optimization decisions.

---

## The Pipeline

<div align="center">

![Optimization Pipeline](./sovereign-prompt/assets/optimization-pipeline.gif)

</div>

Every prompt travels through a deterministic pipeline: **heuristic analysis** catches 9 categories of waste and risk. **Domain templates** apply field-specific constraints (backend, frontend, data, security, product, documentation). **Refinement** strips filler and normalizes structure. **Variant generation** gives you three angles. **Multi-model tokenization** counts across `cl100k_base`, `o200k_base`, `p50k_base`, and `r50k_base` simultaneously. The result is persisted, hashed, and ready to sign.

---

## 9 Lines of Defense

Every prompt runs through 9 heuristic checks, each tunable via a single TOML config file:

| Check | Severity | Why It Matters |
|:------|:--------:|:---------------|
| **Vagueness Detection** | `warn` | Vague prompts produce vague outputs. 14 built-in patterns catch the worst offenders. |
| **Redundancy Analysis** | `info` | Saying the same thing three times doesn't make the model listen harder. It just costs more. |
| **Missing Context** | `crit` | "Fix the bug" is not a prompt. It's a wish. This check demands specifics before you waste a call. |
| **Politeness Tokens** | `info` | The model doesn't care if you say please. 7 patterns stripped, zero feelings hurt. |
| **Prompt Injection** | `crit` | 8 injection patterns detected. Configurable response: warn, rewrite on the fly, or reject outright. |
| **Task Separation** | `warn` | Bundled tasks degrade accuracy. This flags prompts trying to do too much at once. |
| **Output Format** | `info` | No format signal means unpredictable structure. Caught early, fixed automatically. |
| **Ambiguous Pronouns** | `warn` | "Fix it and send it there" — fix what? Send where? Pronouns without referents are token-expensive ambiguity. |
| **Governance Policy** | `crit` | SSN patterns, credit card numbers, API keys, PII references — caught and flagged before they leave your machine. |

Every check can be toggled on or off. Thresholds are adjustable. You can add your own patterns. Define **custom heuristic plugins** with regex patterns right in the TOML config. The injection handler has three modes: **warn** (flag it), **rewrite** (strip it), or **reject** (block it).

---

## The Toolbelt — 15 MCP Tools

| Tool | What It Does |
|:-----|:-------------|
| **`optimize_prompt`** | The core. Analyzes, refines, templates, generates variants, counts tokens across models. Optional `explain_mode` shows every heuristic decision. |
| **`savings_report`** | Your ROI in numbers. Token savings, cost estimates across Claude Sonnet 4 / Opus 4 / GPT-4o / GPT-4o-mini, daily trends, and top recurring issues — over any time period. |
| **`rate_optimization`** | Rate an optimization as positive or negative. Feeds the learning loop that tracks what works and what doesn't. |
| **`learning_insights`** | Get learning insights from rated optimizations — best domains, satisfaction rate, and actionable recommendations. |
| **`team_report`** | Team-level analytics aggregating savings across multiple users with per-member breakdowns and cost estimates. |
| **`capture_output`** | Store the AI's actual response against an optimized prompt. Enables output hashing for end-to-end integrity verification. |
| **`count_tokens`** | Count tokens for any text across all 4 supported tokenizer models, or target a single model. |
| **`get_stats`** | Per-user aggregate metrics: total prompts, tokens saved, average savings percentage, top issues by frequency. |
| **`get_history`** | Fetch recent optimizations with full before/after data, domain, model, and governance status. |
| **`list_templates`** | See available domain templates: general, backend, frontend, data, security, product, documentation. |
| **`governance_check`** | Run governance validation against a stored prompt — SSN, credit card, credential, and PII pattern detection. |
| **`governance_approve`** | Approve or reject a prompt with actor tracking and full audit trail. |
| **`get_audit_trail`** | Every action on every prompt — creation, approval, rejection, signing, output capture — with timestamps and actors. |
| **`sign_optimization`** | Cryptographically sign a prompt record with HMAC-SHA256. Tamper-evident, verifiable, non-repudiable. |
| **`verify_signature`** | Verify the signature and full hash chain — content hash, output hash, and cryptographic binding. |

---

## Security

SovereignPrompt is **local-only by design**. Zero outbound network calls. Zero telemetry. Zero cloud dependencies. Your prompts never leave your machine.

- `#![deny(unsafe_code)]` enforced at the crate root — no escape hatches
- SHA-256 content hashing for tamper detection on every optimization
- HMAC-SHA256 signing with configurable keys for non-repudiable audit trails
- Governance policy engine catches sensitive data patterns before they go anywhere
- Injection detection with three configurable response modes

Full security documentation: [`SECURITY.md`](./SECURITY.md)

---

## Configure Everything

Drop a `sovereign_prompt.toml` next to the binary (or point `SOVEREIGN_CONFIG_PATH` at it):

```toml
[heuristics]
vagueness = true
redundancy = true
missing_context = true
politeness = true
injection = true
task_separation = true
output_format = true
ambiguous_pronouns = true
governance = true

# Tune thresholds
redundancy_word_repeat = 3
pronoun_threshold = 3
context_min_length = 50

# Custom heuristic plugins — your own regex-based checks
# [[heuristics.custom_checks]]
# name = "jargon_detector"
# pattern = "(?i)(synergy|leverage|paradigm)"
# severity = "warning"
# message = "Corporate jargon detected"
# suggestion = "Use plain, direct language"

[injection]
mode = "warn"   # "warn" | "rewrite" | "reject"
```

No config file? Every default is sane. The engine runs at full strength out of the box.

---

## Live Dashboard

A built-in Axum dashboard streams analytics over WebSockets at `http://127.0.0.1:8787`. Total prompts, tokens saved, average savings, governance status — all updating in real time.

```bash
SOVEREIGN_DASHBOARD_ONLY=1 ./target/release/sovereign-prompt
```

---

## Under the Hood

| | |
|:--|:--|
| **Language** | Rust (2021 edition), `#![deny(unsafe_code)]` |
| **Protocol** | MCP via `rmcp` — stdio (default) or SSE transport |
| **Tokenization** | `tiktoken-rs` — cl100k_base, o200k_base, p50k_base, r50k_base |
| **Persistence** | SQLite or Postgres via `sqlx` Any driver — set `DATABASE_URL` for Postgres |
| **Dashboard** | `axum` with WebSocket streaming |
| **Crypto** | `sha2` + `hmac` — SHA-256 hashing, HMAC-SHA256 signing |
| **Config** | `toml` — fully configurable heuristics, plugins, and injection modes |
| **Runtime** | `tokio` — full async with graceful signal handling |
| **Tests** | 63 integration tests across all modules |

---

## Environment Variables

| Variable | Default | Description |
|:---------|:--------|:------------|
| `DATABASE_URL` | _(none)_ | Database URL — `postgres://...` for Postgres, `sqlite://...` for SQLite. Takes priority over `SOVEREIGN_DB_PATH`. |
| `SOVEREIGN_DB_PATH` | `./sovereign_prompt.db` | SQLite database path (used when `DATABASE_URL` is not set) |
| `SOVEREIGN_CONFIG_PATH` | `./sovereign_prompt.toml` | TOML config path |
| `SOVEREIGN_MCP_TRANSPORT` | `stdio` | Transport: `stdio` or `sse` |
| `SOVEREIGN_MCP_SSE_ADDR` | `127.0.0.1:8790` | SSE bind address |
| `SOVEREIGN_DASHBOARD_ADDR` | `127.0.0.1:8787` | Dashboard bind address |
| `SOVEREIGN_DASHBOARD_ONLY` | `false` | Dashboard-only mode |
| `SOVEREIGN_HMAC_KEY` | _(dev default)_ | HMAC signing key (**change in production**) |
| `RUST_LOG` | _(none)_ | Log level: `info`, `debug`, `trace` |

---

## Docker Deployment

Run SovereignPrompt with Postgres in one command:

```bash
docker-compose up --build
```

This starts:

- **SovereignPrompt** in SSE mode on `http://localhost:8790/sse`
- **Postgres 16** with automatic schema migration
- **Dashboard** on `http://localhost:8787`
- **Health check** at `http://localhost:8787/health`

For custom configuration:

```bash
SOVEREIGN_HMAC_KEY=your-production-key docker-compose up --build
```

Or run standalone with Docker:

```bash
docker build -t sovereign-prompt ./sovereign-prompt
docker run -p 8790:8790 -p 8787:8787 \
  -e DATABASE_URL=postgres://user:pass@host:5432/db \
  -e SOVEREIGN_MCP_TRANSPORT=sse \
  sovereign-prompt
```

---

<div align="center">

<br>

Built by [**ExecLayer Inc.**](https://github.com/BMC-INC)

MIT License

*Your prompts. Your machine. Your sovereignty.*

<br>

</div>
