# Secret Sorbet

Secret Sorbet is a deterministic password generator. The core promise is:

```text
same passphrase + same variables + same policy + same algorithm version
=> same output
```

The application should not need to store account passwords or the variables used
to generate them. Storage can be added as an optional convenience layer, but it is
not the core model.

## Rust Monorepo Layout

```text
lib/core      deterministic derivation and user-definable output policies
lib/pw        password-specific defaults, login variables, and sanity checks
app/cli       command-line password generator
app/desktop   Tauri 2 desktop app for Windows, macOS, and Linux
app/ios       native SwiftUI app
app/android   native Kotlin/Compose app
```

The repository uses Rust for the shared deterministic engine. Desktop should use
Tauri so it can call the Rust crates directly; iOS and Android should stay
native so platform keychain, biometric, and device integration remain natural.

The first app milestone is the `seso` CLI, which calls `lib/pw` and
produces the pinned compatibility vector without storing user inputs.

Run it with a hidden passphrase prompt:

```sh
cargo run --locked -p secret-sorbet-cli -- generate \
  --service example.com \
  --username alice@example.com
```

By default, `seso` offers to copy the password to the clipboard for two minutes
without displaying it. Pass `--print` to emit it explicitly on standard output.

See [`app/cli/README.md`](app/cli/README.md) for secure automation, output, and
diagnostic behavior.

## Determinism Contract

The Rust core avoids platform-dependent behavior by treating all inputs as bytes,
serializing variables and policies with explicit lengths, deriving key material
with Argon2id, and using fixed big-endian counters for deterministic
HMAC-SHA-512 output. The `secret-sorbet-core-v1` profile uses Argon2id with
64 MiB of memory, 3 iterations, 4 lanes, and a 256-bit derived key. Legacy
direct-passphrase HMAC derivation was removed before release and is not supported.

Compatibility should be guarded with test vectors. Every future implementation
or binding must produce the exact same output for the same vector.

Run the same quality gates locally:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo audit
cargo +nightly fuzz run policy -- -runs=10000
```

`cargo audit` requires the `cargo-audit` Cargo subcommand. The automated audit
also runs weekly so newly published RustSec advisories are detected without
waiting for a source change. The fuzz target requires `cargo-fuzz` 0.13.2 and a
nightly Rust toolchain; its generated corpus and crash artifacts are ignored.
