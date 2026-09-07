# Secret Sorbet Desktop App

This directory is reserved for the Tauri 2 desktop application.

The desktop app should target Windows, macOS, and Linux while calling the Rust
crates under `lib/` directly.

The intended split is:

- `lib/core`: deterministic derivation and user-definable output policies
- `lib/pw`: password-specific defaults and validation
- `app/desktop`: Tauri 2 shell and desktop-specific integrations

First milestone:

```text
desktop UI input -> lib/pw -> pinned test-vector output
```

