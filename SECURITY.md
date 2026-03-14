# Security Policy

## Architecture

SovereignPrompt is a **local-only** MCP server. It runs entirely on the user's machine with zero outbound network calls.

- **No telemetry.** No usage data, prompts, or analytics are ever sent externally.
- **No cloud dependencies.** The server operates over `stdio` or a local SSE transport — both bound to `127.0.0.1` by default.
- **SQLite persistence** is file-local. The database path is configurable via `SOVEREIGN_DB_PATH`.

## Data Handling

| Data Type | Storage | Retention |
|:----------|:--------|:----------|
| Original prompts | SQLite (local) | Until user deletes DB file |
| Refined prompts | SQLite (local) | Until user deletes DB file |
| AI output (if captured) | SQLite (local) | Until user deletes DB file |
| HMAC signatures | SQLite (local) | Until user deletes DB file |
| Audit trail | SQLite (local) | Until user deletes DB file |
| Configuration | TOML file (local) | User-managed |

No data leaves the local filesystem. There are no API keys, no external service calls, and no network egress.

## Cryptographic Integrity

- **Content hashing**: SHA-256 over `original_prompt || "||" || refined_prompt` ensures tamper detection.
- **Output hashing**: SHA-256 over captured AI output for end-to-end verification.
- **HMAC-SHA256 signing**: Records can be cryptographically signed using a configurable key (`SOVEREIGN_HMAC_KEY`).
- **Hash chain verification**: The `verify_signature` tool validates both content hash and output hash integrity.

**Important:** Change the default HMAC key in production. The built-in dev key is for local development only.

## Prompt Injection Detection

SovereignPrompt detects 8 prompt injection patterns:

- `ignore previous`, `ignore all`, `disregard`
- `forget everything`, `new instruction`
- `system:`, `assistant:`, `jailbreak`

Injection handling is configurable via `sovereign_prompt.toml`:

| Mode | Behavior |
|:-----|:---------|
| `warn` (default) | Flag in analysis feedback |
| `rewrite` | Strip injection patterns before refinement |
| `reject` | Return an error immediately |

## Governance Policy

Built-in governance checks detect sensitive data patterns:

- SSN patterns (`\d{3}-\d{2}-\d{4}`)
- Credit card numbers (`\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}`)
- Credential patterns (`api_key`, `secret_key`, `password`)
- PII references (`social security`, `date of birth`, `passport number`, `bank account`)

Prompts containing critical governance violations are automatically rejected. Warning-level findings are flagged as pending for review.

## `#![deny(unsafe_code)]`

The crate enforces `#![deny(unsafe_code)]` at the library root. No `unsafe` blocks are permitted anywhere in the codebase.

## CI Recommendations

```yaml
# Add to your CI pipeline
- cargo audit          # Check dependencies for known vulnerabilities
- cargo clippy -- -D warnings  # Zero warnings policy
- cargo test           # All integration tests pass
```

## Supported Versions

| Version | Supported |
|:--------|:----------|
| 0.1.x | Yes |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public issue.
2. Email security concerns to the maintainers via the [ExecLayer Inc.](https://github.com/BMC-INC) GitHub organization.
3. Include a description of the vulnerability, steps to reproduce, and potential impact.
4. We will acknowledge receipt within 48 hours and provide a timeline for a fix.
