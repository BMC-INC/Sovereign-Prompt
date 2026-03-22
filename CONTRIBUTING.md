# Contributing to SovereignPrompt

Thanks for your interest in contributing to SovereignPrompt. This document covers the process for contributing to this project.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/Sovereign-Prompt.git
   cd Sovereign-Prompt/sovereign-prompt
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run the tests:
   ```bash
   cargo test
   ```

## Development

### Prerequisites

- Rust (stable, 2021 edition)
- SQLite development libraries

### Project Structure

```
sovereign-prompt/
├── src/
│   ├── main.rs          # Entry point, transport selection
│   ├── server.rs        # 15 MCP tool handlers
│   ├── analyzer.rs      # 9 heuristic checks
│   ├── optimizer.rs     # Prompt refinement and variants
│   ├── config.rs        # TOML configuration
│   ├── templates.rs     # Domain templates
│   ├── tokenizer.rs     # Multi-model token counting
│   ├── crypto.rs        # SHA-256, HMAC-SHA256
│   ├── governance.rs    # PII/credential detection
│   ├── dashboard.rs     # Axum WebSocket dashboard
│   ├── types.rs         # Data structures
│   └── db.rs            # SQLite persistence
└── tests/
    └── integration_test.rs  # 63 integration tests
```

### Code Standards

- **Zero unsafe code** — `#![deny(unsafe_code)]` is enforced at the crate root
- **Zero warnings** — `cargo clippy -- -D warnings` must pass
- **Formatted** — `cargo fmt --check` must pass
- **All tests pass** — `cargo test` must pass

### Before Submitting

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## Pull Requests

1. Create a feature branch from `main`
2. Make your changes
3. Ensure all checks pass (see above)
4. Write a clear PR description explaining the **why**, not just the **what**
5. Keep PRs focused — one feature or fix per PR

## Reporting Issues

- Use GitHub Issues
- Include steps to reproduce
- Include your OS, Rust version, and MCP client

## Security

If you discover a security vulnerability, **do not** open a public issue. See [SECURITY.md](./SECURITY.md) for responsible disclosure instructions.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](./LICENSE).
