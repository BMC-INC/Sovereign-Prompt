# SovereignPrompt

Real-time MCP prompt optimization with token analysis, heuristic evaluation, and persistence.
Built by ExecLayer Inc.

https://github.com/user-attachments/assets/SovereignPrompt.mp4

## Features
- Real-time prompt analysis (clarity, redundancy, context, injection detection)
- Accurate token counting via tiktoken (cl100k_base)
- Token savings percentage on every optimization
- 3 prompt variants per request (Precision, Creative, Concise)
- Full persistence: prompts + outputs stored in SQLite
- User stats: total tokens saved, average savings, history
- MCP-native: works with any MCP-compatible client

## Tools Exposed
| Tool | Description |
|------|-------------|
| `optimize_prompt` | Analyze and refine a prompt, returns feedback + variants |
| `capture_output` | Store AI output against a prompt for learning |
| `get_stats` | User-level token savings stats |
| `get_history` | Recent prompt history |

## Setup
```bash
cp .env.example .env
cargo build --release
```

## MCP Config (claude_desktop_config.json)
```json
{
  "mcpServers": {
    "sovereign-prompt": {
      "command": "/path/to/sovereign-prompt",
      "env": {
        "SOVEREIGN_DB_PATH": "/path/to/sovereign_prompt.db"
      }
    }
  }
}
```

## ExecLayer Integration
SovereignPrompt is designed to connect with ExecLayer's SovereignClaw governance layer.
Cryptographic execution binding and audit trails for enterprise compliance coming in v0.2.
