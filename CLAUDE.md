# SovereignPrompt — Claude Code Project Config

## SovereignPrompt Optimize Command — Full Behavior

When the user asks to optimize a prompt (or when using the `optimize_prompt` MCP tool), follow this exact five-step flow every time. No exceptions.

### Step 1 — Rewrite
Take the user's raw prompt and rewrite it to the strongest possible version. Infer intent from vague input. Never return bracket placeholders like `[file name]` or `[error message]`. If the user was vague, sharpen it based on what they clearly meant. If critical context is genuinely missing and cannot be inferred, ask one targeted question — never more than one.

### Step 2 — Execute
Run the optimized prompt and return the actual result. The user should get their answer, not a template. They came here to get something done, not to learn prompt engineering.

### Step 3 — Diff
After delivering the result, show a brief explanation of what was changed from the original prompt and why. Keep it tight — 2-4 bullet points max, not a lecture.

### Step 4 — Metrics Block
After the diff, display a compact metrics summary:
- **Token count:** original prompt tokens → optimized prompt tokens (delta saved)
- **Clarity score:** rate original vs optimized on a 1-10 scale for specificity, actionability, and structure
- **Risk flags:** if the original prompt had ambiguity that could cause hallucination, off-target responses, or wasted output tokens, call it out explicitly
- **Estimated output efficiency:** flag whether the optimized prompt is likely to produce a usable response in one pass vs the original potentially needing follow-ups

The metrics block should be compact — no more than 4-5 lines. This is SovereignPrompt proving its own value on every single call.

### Step 5 — Learn Block (Condensed)
Keep the teaching component but make it minimal. One or two sentences max explaining the general prompt pattern that was applied — e.g., "Vague verbs replaced with direct actions, added constraint to prevent scope creep." No tables, no lengthy breakdowns. Just enough that the user absorbs the principle over time without it eating output tokens. Think fortune-cookie-sized advice, not a lesson plan.

### Core Rule
The user gives a prompt, they get back a better prompt, the actual result, proof that the optimization mattered, and a one-liner they'll remember. Five deliverables every time. No exceptions.

## Verification
Always verify changes work by actually testing/checking live state — never claim something works without running the relevant command, loading the URL, or checking the build output.

## Communication Style
When the user asks a simple yes/no question, answer it directly first, then provide any additional context. Do not launch into multi-step explanations or shell commands.

## Shell Commands
For shell commands, always use single-line or properly escaped commands compatible with zsh. Never output multi-line commands that will break in the terminal.

## Project Stack
Primary languages: Rust, TypeScript, Markdown. When making changes, run the appropriate build/check command (`cargo build`, `cargo clippy`, `tsc`) before claiming completion.

## Architecture

- **Database:** Dual-backend via sqlx `Any` driver. SQLite (default, local) or Postgres (remote/Docker) selected by `DATABASE_URL` env var. `SOVEREIGN_DB_PATH` is the legacy SQLite-only fallback.
- **Transport:** stdio (default for Claude Code) or SSE (`SOVEREIGN_MCP_TRANSPORT=sse` for remote clients).
- **Dashboard:** Axum HTTP + WebSocket on port 8787 with `/health` endpoint.
- **Docker:** `docker-compose up --build` runs SovereignPrompt + Postgres 16. Dockerfile is in `sovereign-prompt/`.
- **CI/CD:** GitHub Actions — test, clippy, fmt, cargo audit. Release workflow builds binaries for Linux/macOS/Windows on tag push.
- **Auth module:** `src/auth.rs` exists as a utility for future per-user API key auth on SSE transport (not yet wired in — rmcp's SSE internals are private).

## Git Workflow
When the user says to push to main, push directly to main. Do not create PRs or branches unless explicitly asked. Check branch protection status first and inform the user if it blocks the push.

## Security & Secrets
Never ask the user to paste API keys or secrets directly in chat. Guide them to store secrets in environment variables (`.zshrc`, `.env`) and reference those in config.
