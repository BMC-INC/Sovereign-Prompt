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

<br><br>

# SovereignPrompt

### Your prompts are bleeding tokens. We stop the bleed.

**The first MCP-native prompt optimization engine. 9 heuristic checks, domain-aware refinement,<br>cryptographic audit trails, and real cost savings — all running locally in pure Rust.**

<br>

https://github.com/BMC-INC/Sovereign-Prompt/releases/download/v0.2.0/SovereignPrompt.mp4

<br>

[Get Started](#get-started) &#8226; [How It Works](#how-it-works) &#8226; [12 Tools](#the-toolbelt--12-mcp-tools) &#8226; [Security](./SECURITY.md) &#8226; [Inner Docs](./sovereign-prompt/README.md)

<br>

</div>

---

## The Pitch

You send 100 prompts a day. Most of them are 30-60% filler — politeness tokens the model ignores, vague language that degrades output quality, redundant phrasing that burns your budget. Multiply that across a team, and you're lighting money on fire.

SovereignPrompt sits between you and the model. Every prompt passes through 9 heuristic checks, gets stripped of waste, shaped by domain-specific templates, and measured across 4 tokenizer models — before a single token reaches the API. You get the refined prompt, three strategic variants, a full token savings breakdown, and a cryptographically signed audit trail. All of it stored locally. Zero cloud. Zero telemetry.

This isn't a wrapper. It's an optimization engine that speaks [MCP](https://modelcontextprotocol.io) — the open protocol that Claude Desktop, Claude Code, Cursor, and every major AI client is adopting. Plug it in once, and every prompt you send through any MCP client gets optimized automatically.

---

## See It

<div align="center">

![How It Works](./sovereign-prompt/assets/how-it-works.gif)

*Request in. 9 checks run. Domain template applied. 4 tokenizers count. Optimized prompt out. Every step persisted.*

</div>

---

## The Bleed Is Real

```
"Hey, could you please kindly help me out and maybe possibly write
 something that sort of fixes the thing with the stuff? Thank you
 so much! Also additionally can you do the other thing as well?"
```

That prompt has **14 vague terms**, **5 politeness tokens**, **2 task-bundling conjunctions**, **no output format**, and **zero actionable context**. SovereignPrompt catches all of it in a single pass and tells you exactly what's wrong, why it matters, and what the fix looks like.

Turn on **explain mode** and you see every heuristic that fired, what it matched, and the threshold it triggered — full transparency into the optimization decisions.

---

## What Comes Out The Other Side

| What you send | What hits the model | Saved |
|:-------------|:-------------------|:-----:|
| "Please kindly help me write something that sort of fixes the bug in the code somehow" | "Fix the bug in the code. Be exact, technical, and minimal." | **~36%** |
| "Could you please maybe write something creative about nature, thank you so much" | "Write creative content about nature" | **~75%** |
| "Would you mind kindly helping me perhaps analyze the data and maybe find some patterns or something" | "Analyze the data and find patterns. Respond concisely and directly." | **~42%** |
| "Please could you help me plan out the tasks and then also break them down and additionally prioritize them" | "Plan tasks, break them down, prioritize. Respond concisely and directly." | **~48%** |

*Measured with `cl100k_base`. Your mileage varies — bloated prompts save more, clean prompts save less. That's the point.*

And for every optimization, you get **three variants** — **Precision** (tight, technical, minimal), **Creative** (broad, exploratory), and **Concise** (stripped to the bone) — each with its own token count so you can choose the right tool for the job.

---

## The Pipeline

<div align="center">

![Optimization Pipeline](./sovereign-prompt/assets/optimization-pipeline.gif)

</div>

Every prompt travels through a deterministic pipeline: **heuristic analysis** catches 9 categories of waste and risk. **Domain templates** apply field-specific constraints (backend, frontend, data, security, product, documentation). **Refinement** strips filler and normalizes structure. **Variant generation** gives you three angles. **Multi-model tokenization** counts across `cl100k_base`, `o200k_base`, `p50k_base`, and `r50k_base` simultaneously. The result is persisted, hashed, and ready to sign.

No LLM in the loop. No API calls. No latency. Pure deterministic Rust.

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

Every check can be toggled on or off individually. Thresholds are adjustable. You can add your own patterns. The injection handler has three modes: **warn** (flag it), **rewrite** (strip it), or **reject** (block it). All configured in `sovereign_prompt.toml`.

---

## The Toolbelt — 15 MCP Tools

| Tool | What It Does |
|:-----|:-------------|
| **`optimize_prompt`** | The core. Analyzes, refines, templates, generates variants, counts tokens across models. Optional `explain_mode` shows every heuristic decision. |
| **`savings_report`** | Your ROI in numbers. Token savings, cost estimates across Claude Sonnet 4 / Opus 4 / GPT-4o / GPT-4o-mini, daily trends, and top recurring issues — over any time period. |
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
| **`rate_optimization`** | Rate an optimization as positive or negative. Feeds the learning loop that tracks what works and what doesn't. |
| **`learning_insights`** | Get learning insights from rated optimizations — best domains, satisfaction rate, and actionable recommendations. |
| **`team_report`** | Team-level analytics aggregating savings across multiple users with per-member breakdowns and cost estimates. |

---

## Security Model

SovereignPrompt is **local-only by design**. Zero outbound network calls. Zero telemetry. Zero cloud dependencies. Your prompts never leave your machine.

- `#![deny(unsafe_code)]` enforced at the crate root — no escape hatches
- SHA-256 content hashing for tamper detection on every optimization
- HMAC-SHA256 signing with configurable keys for non-repudiable audit trails
- Governance policy engine catches sensitive data patterns before they go anywhere
- Injection detection with three configurable response modes

Full security documentation: [`SECURITY.md`](./SECURITY.md)

---

## Get Started

**Three steps. Under two minutes.**

### 1. Build

```bash
git clone https://github.com/BMC-INC/Sovereign-Prompt.git
cd Sovereign-Prompt/sovereign-prompt
cp .env.example .env
cargo build --release
```

### 2. Connect

<details>
<summary><strong>Claude Desktop</strong> — add to <code>claude_desktop_config.json</code></summary>

```json
{
  "mcpServers": {
    "sovereign-prompt": {
      "command": "/absolute/path/to/target/release/sovereign-prompt",
      "env": {
        "SOVEREIGN_DB_PATH": "/absolute/path/to/sovereign_prompt.db",
        "SOVEREIGN_CONFIG_PATH": "/absolute/path/to/sovereign_prompt.toml",
        "SOVEREIGN_HMAC_KEY": "your-secret-key",
        "RUST_LOG": "info"
      }
    }
  }
}
```

</details>

<details>
<summary><strong>Claude Code</strong> — add to <code>.mcp.json</code> in your project root</summary>

```json
{
  "mcpServers": {
    "sovereign-prompt": {
      "command": "/absolute/path/to/target/release/sovereign-prompt",
      "env": {
        "SOVEREIGN_DB_PATH": "./sovereign_prompt.db",
        "SOVEREIGN_CONFIG_PATH": "./sovereign_prompt.toml"
      }
    }
  }
}
```

</details>

<details>
<summary><strong>Any MCP Client</strong> — SSE transport for network-based clients</summary>

```bash
SOVEREIGN_MCP_TRANSPORT=sse SOVEREIGN_MCP_SSE_ADDR=127.0.0.1:8790 ./target/release/sovereign-prompt
```

SSE endpoint: `http://127.0.0.1:8790/sse`

</details>

### 3. Use

Every tool is available the moment the server connects. Start here:

```
optimize_prompt: { "prompt": "your prompt here" }
```

Want to see what the engine is thinking? Add `"explain_mode": true` — you'll get a full breakdown of every heuristic, what matched, what didn't, and why.

Want to know how much you've saved? Call `savings_report` with your user ID and a time range (`7d`, `30d`, `90d`, `all`). You'll get total token savings, cost estimates across major models, daily trends, and your most common prompt issues.

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

# Tune thresholds to your workflow
redundancy_word_repeat = 3
pronoun_threshold = 3
context_min_length = 50
conjunction_threshold = 2
format_min_length = 30

# Add your own patterns
# extra_vague_terms = ["thingy", "whatnot"]
# extra_injection_patterns = ["bypass safety"]
# extra_polite_terms = ["excuse me"]

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

Run the dashboard standalone:

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
| **Persistence** | SQLite via `sqlx` — async, zero compile-time DB |
| **Dashboard** | `axum` with WebSocket streaming |
| **Crypto** | `sha2` + `hmac` — SHA-256 hashing, HMAC-SHA256 signing |
| **Config** | `toml` — fully configurable heuristics and injection modes |
| **Runtime** | `tokio` — full async with graceful signal handling |
| **Tests** | 63 integration tests across all modules |

---

## Environment Variables

| Variable | Default | Description |
|:---------|:--------|:------------|
| `SOVEREIGN_DB_PATH` | `./sovereign_prompt.db` | SQLite database path |
| `SOVEREIGN_CONFIG_PATH` | `./sovereign_prompt.toml` | TOML config path |
| `SOVEREIGN_MCP_TRANSPORT` | `stdio` | Transport: `stdio` or `sse` |
| `SOVEREIGN_MCP_SSE_ADDR` | `127.0.0.1:8790` | SSE bind address |
| `SOVEREIGN_DASHBOARD_ADDR` | `127.0.0.1:8787` | Dashboard bind address |
| `SOVEREIGN_DASHBOARD_ONLY` | `false` | Dashboard-only mode |
| `SOVEREIGN_HMAC_KEY` | _(dev default)_ | HMAC signing key (**change in production**) |
| `RUST_LOG` | _(none)_ | Log level: `info`, `debug`, `trace` |

---

## Roadmap

- [x] 9-heuristic analysis engine with severity grading
- [x] Domain-specific template library (7 domains)
- [x] Multi-model token counting across 4 encodings
- [x] Configurable heuristics + injection modes via TOML
- [x] Explain mode — full heuristic transparency
- [x] Cost savings reports with multi-model estimates
- [x] Governance policy engine with PII/credential detection
- [x] Cryptographic audit trails (SHA-256 + HMAC-SHA256)
- [x] SSE transport + WebSocket dashboard
- [x] `#![deny(unsafe_code)]` + security documentation
- [x] Learning feedback loop — rate optimizations, get insights, improve over time
- [x] Custom heuristic plugins — define your own regex checks in TOML
- [x] Team-level analytics — aggregate savings across your organization

---

<div align="center">

<br>

Built by [**ExecLayer Inc.**](https://github.com/BMC-INC)

MIT License

*Your prompts. Your machine. Your sovereignty.*

<br>

</div>
