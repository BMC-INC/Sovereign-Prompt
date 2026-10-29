# Security boundaries

SovereignPrompt is intended for local evaluation and controlled integration. The source is available for review; that is not a security certification.

## Data flow

The analysis and refinement engine runs locally without a model API call. SQLite stores data locally; PostgreSQL uses the configured database connection. The optional HTTP proxy sends requests, credentials, and prompt content to the configured upstream provider. No analytics reporting service is built into the Rust services.

Original prompts, refined prompts, captured responses, user labels, and audit records are persisted without application-level encryption. There is no automatic retention policy. Protect the database, backups, process environment, and any signing key. User labels are not authenticated identities or tenant boundaries.

## HTTP services

The dashboard, proxy, and legacy MCP SSE endpoint default to loopback. The development Docker configuration publishes its ports on loopback. These services do not have complete built-in authentication, authorization, host/origin validation, or tenant isolation. The existence of `auth.rs` does not mean its middleware protects every route.

Prefer stdio for local MCP access. Public or shared deployments require a separately reviewed gateway and access policy; do not publish the default ports directly or expose them through an unrestricted tunnel.

## Heuristic checks

Sensitive-data checks recognize specific SSN, credit-card, credential, and PII-reference patterns. Injection checks recognize configured wording patterns. Both can miss sensitive content and produce false positives. They do not replace access control, a complete DLP system, or review of model outputs.

The MCP optimizer records `approved`, `pending`, or `rejected` governance status. A returned optimization is not automatic approval to send the text elsewhere. The HTTP proxy rejects critical governance matches and configured injection rejections with HTTP 403. It checks the last user message's extracted text, not every system message, tool result, attachment, or prior turn. Requests without that text pass through.

## Integrity records

SHA-256 hashes detect changes to the values covered by the record. HMAC-SHA256 authenticates records to a shared key. Anyone holding that key can produce a valid tag; this is not public-key signing, independent proof of identity, or non-repudiation.

Set `SOVEREIGN_HMAC_KEY` before relying on HMAC verification. The built-in development key is publicly known. Protect and back up the key separately from the database.

## Dependencies and release limits

The current workspace uses an older `rmcp` line. CI's audit step contains explicit exceptions for RUSTSEC-2023-0071 and RUSTSEC-2026-0189; a passing CI result does not mean there are no advisories. [RUSTSEC-2026-0189](https://rustsec.org/advisories/RUSTSEC-2026-0189.html) concerns HTTP transport host validation and recommends `rmcp` 1.4.0 or newer; stdio is unaffected by that advisory. Migration and HTTP exposure review remain release work.

Run the test suite, Clippy, and an unfiltered dependency audit when assessing a deployment. The core library denies unsafe Rust in its own code; that attribute does not certify the implementation or its dependencies.

## Reporting a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/BMC-INC/Sovereign-Prompt/security/advisories/new) if enabled. Otherwise contact a maintainer through a verified channel listed on the [organization profile](https://github.com/BMC-INC) to arrange private disclosure. Do not include credentials, private prompts, or exploit details in a public issue.
