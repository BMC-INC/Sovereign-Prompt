# Security boundaries

SovereignPrompt is intended for local evaluation and controlled integration. The source is available for review; that is not a security certification.

## Data flow

The analysis and refinement engine runs locally without a model API call. SQLite stores data locally; PostgreSQL uses the configured database connection. The optional HTTP proxy sends requests, credentials, and prompt content to the configured upstream provider. No analytics reporting service is built into the Rust services.

Original prompts, refined prompts, captured responses, user labels, and audit records are persisted without application-level encryption. There is no automatic retention policy. Protect the database, backups, process environment, and any signing key. In stdio and uncredentialed loopback mode, user labels remain local operator inputs. Authenticated HTTP requests bind the user identity to a configured API credential.

## HTTP services

The dashboard, proxy, and MCP endpoints default to loopback. Every HTTP request validates an exact allowed Host and, when present, the browser Origin. Non-loopback binds fail at startup without configured credentials and explicit allowed hosts. Keys are compared using constant-time HMAC verification. Invalid credential configuration fails startup.

`SOVEREIGN_API_KEY` binds one key to `SOVEREIGN_USER_ID`; `SOVEREIGN_API_KEYS` supports independent user/key pairs. Authenticated MCP and dashboard access cannot cross that identity's prompt IDs, history, reports, ratings, or audit records. Proxy records are attributed to the authenticated identity. Provider credentials remain separate from the local proxy key (`X-Sovereign-Key`), and local credentials/cookies are not forwarded upstream. Upstream redirects are not followed.

Stdio uses OS process access and retains its existing local operator behavior. Uncredentialed loopback HTTP also retains operator access for compatibility; loopback alone does not isolate other applications running on the same machine. Enable credentials to restrict access. Public HTTP listeners require an HTTPS reverse proxy and deployment-specific access/operating controls. The server does not implement OAuth, automatic key rotation, distributed quotas, or an identity-provider login flow.

The dashboard stores opaque sessions only in bounded server memory. Cookies are HttpOnly and SameSite=Strict, expire after one hour, and use Secure for HTTPS origins. Signing out revokes the session. Browser API keys are not kept in storage or URLs. Dashboard sockets have a bounded lifetime and recheck cookie-session validity while streaming.

Modern `/mcp` is stateless: every request authenticates independently and there is no cross-user server session to reuse. Legacy `/sse` sessions are bound to the authenticated principal; `/message` requests must carry that same identity. Compatibility sessions are capped at 64 per process, have bounded queues, expire after 30 minutes, and are removed on disconnect. MCP requests have 16 concurrent admission slots, a 45-second response timeout, a 1 MiB HTTP-body limit, and a 256 KiB tool-argument limit. HTTP bodies are read with a 15-second timeout. These limits are process controls, not a distributed denial-of-service protection service.

## Heuristic checks

Sensitive-data checks recognize specific SSN, credit-card, credential, and PII-reference patterns. Injection checks recognize configured wording patterns. Both can miss sensitive content and produce false positives. They do not replace access control, a complete DLP system, or review of model outputs.

The MCP optimizer records `approved`, `pending`, or `rejected` governance status. A returned optimization is not automatic approval to send the text elsewhere. The HTTP proxy rejects critical governance matches and configured injection rejections with HTTP 403. It checks the last user message's extracted text, not every system message, tool result, attachment, or prior turn. Requests without that text pass through.

## Integrity records

SHA-256 hashes detect changes to the values covered by the record. HMAC-SHA256 authenticates records to a shared key. Anyone holding that key can produce a valid tag; this is not public-key signing, independent proof of identity, or non-repudiation.

Set `SOVEREIGN_HMAC_KEY` before relying on HMAC verification. The built-in development key is publicly known. Protect and back up the key separately from the database.

## Dependencies and release limits

The workspace uses the current `rmcp` 3.8 SDK. The legacy HTTP+SSE compatibility adapter uses that SDK's maintained protocol implementation; the vulnerable 0.1 dependency is removed. SQLite/PostgreSQL driver packages are pinned together at 0.8.6 so the unused MySQL/RSA dependency graph is not included. CI runs an unfiltered `cargo audit` with no ignored advisories.

The audited October 2026 lockfile has zero reported vulnerabilities or warnings. This is a point-in-time dependency result, not a guarantee against unknown vulnerabilities. Run the test suite, Clippy, and an unfiltered audit when assessing a deployment. The core library denies unsafe Rust in its own code; that attribute does not certify dependencies or a deployed system. Release archives contain both binaries and SHA-256 download checksums; checksums do not provide a signed-installer trust chain.

## Reporting a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/BMC-INC/Sovereign-Prompt/security/advisories/new) if enabled. Otherwise contact a maintainer through a verified channel listed on the [organization profile](https://github.com/BMC-INC) to arrange private disclosure. Do not include credentials, private prompts, or exploit details in a public issue.
