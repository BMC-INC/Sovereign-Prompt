Sovereign Prompt 0.4.0 updates the MCP SDK and protects every HTTP access path while preserving local stdio integration and the existing fifteen tools.

- Current Streamable HTTP at `/mcp`, with a bounded compatibility adapter for existing `/sse` clients.
- Exact Host/Origin validation, bounded requests and HTTP credentials bound to a user identity; another user's record IDs cannot grant access.
- Dashboard sessions use expiring HttpOnly cookies. Provider proxy authentication is separate from provider credentials.
- Anthropic Messages and OpenAI Chat Completions retain buffered and streaming responses and non-text message blocks.
- SQLite and PostgreSQL remain supported, including existing database files. PostgreSQL parameter binding and numeric decoding are repaired. Existing local stdio configurations keep their command, arguments and database setting.
- Each archive includes both `sovereign-prompt` and `sovereign-proxy`, license and security documentation. Verify downloads against `SHA256SUMS`.

For remote HTTP use, configure credentials, exact allowed hosts/origins and a TLS reverse proxy. Claude.ai connectors additionally depend on their configured remote gateway and authentication setup; a local stdio installation alone does not create a Claude.ai connector.

Read the repository's security and migration guidance before upgrading. macOS and Windows downloads are not notarized/code-signed installers.
