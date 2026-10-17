# Secret Sorbet

Secret Sorbet is a deterministic password generator. The core promise is:

```text
same passphrase + same variables + same policy + same algorithm version
=> same output
```

The application should not need to store account passwords or the variables used
to generate them. Storage can be added as an optional convenience layer, but it is
not the core model.

## Compatibility matrix

Snapshot dated 2026-10-17: native targets passed the workspace tests; QEMU
targets passed the shared core/password suites (CLI integration tests are not
included). Both ran in debug and release profiles at revision
[`fd01cbc`](https://github.com/tedla-brandsema/secret-sorbet/commit/fd01cbc),
in [workflow run 39022373134](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134).
These are point-in-time results, not live status badges. See the
[support contract](docs/support.md#execution-coverage) for runtime requirements
and evidence details.

| Execution | Target | ISA | Result |
| --- | --- | --- | --- |
| Native Linux | `x86_64-unknown-linux-gnu` | x86-64 | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567741) |
| Native Linux | `aarch64-unknown-linux-gnu` | ARM64 | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567684) |
| Native Windows | `x86_64-pc-windows-msvc` | x86-64 | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567705) |
| Native Windows | `aarch64-pc-windows-msvc` | ARM64 | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567766) |
| Native macOS | `x86_64-apple-darwin` | x86-64 | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567704) |
| Native macOS | `aarch64-apple-darwin` | Apple silicon ARM64 | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567720) |
| QEMU Linux | `i686-unknown-linux-gnu` | x86, 32-bit | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567708) |
| QEMU Linux | `armv7-unknown-linux-gnueabihf` | ARMv7, 32-bit | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567702) |
| QEMU Linux | `s390x-unknown-linux-gnu` | s390x, big-endian | [✅ Passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567689) |

iOS and Android execution remains unverified and deferred; see the
[support contract](docs/support.md#runtime-and-toolchain-requirements).

The live Rust workflow status below follows pushes to `main` and reflects the
whole workflow, not only compatibility execution:

[![Rust workflow status](https://github.com/tedla-brandsema/secret-sorbet/actions/workflows/rust.yml/badge.svg?branch=main&event=push)](https://github.com/tedla-brandsema/secret-sorbet/actions/workflows/rust.yml?query=branch%3Amain+event%3Apush)

## Rust Monorepo Layout

```text
lib/core      deterministic derivation and user-definable output policies
lib/pw        password-specific defaults, login variables, and sanity checks
app/cli       command-line password generator
app/desktop   planned Tauri 2 app for Windows, macOS, and Linux
app/ios       planned native SwiftUI app
app/android   planned native Kotlin/Compose app
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

See [`docs/README.md`](docs/README.md) for architecture boundaries, contributor
commands, and documentation conventions. Implementation plans belong in
`docs/plans/`; [the roadmap](docs/ROADMAP.md) tracks phase and slice progress,
and `docs/TODO.md` records accepted unfinished work.

See [`CHANGELOG.md`](CHANGELOG.md) for changes awaiting the first release.

## Determinism Contract

The Rust core avoids platform-dependent behavior by treating all inputs as bytes,
serializing variables and policies with explicit lengths, deriving key material
with Argon2id, and using fixed big-endian counters for deterministic
HMAC-SHA-512 output. The `secret-sorbet-core-v1` profile uses Argon2id with
64 MiB of memory, 3 iterations, 4 lanes, and a 256-bit derived key. Legacy
direct-passphrase HMAC derivation was removed before release and is not supported.

The original login vector, fifteen additional synthetic raw-input vectors, and
private intermediate byte fixtures guard compatibility. An independent
Python/libargon2 implementation reproduces their expected values; ordinary Rust
tests use checked-in literals. Every future binding must pass these fixtures.

See the [support and v1 input contract](docs/support.md) for tested targets,
runtime requirements, exact Unicode/whitespace rules, safe recovery, and execution
evidence. Visually identical Unicode text can have different bytes and produce
different v1 passwords. No normalization is added silently.

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
