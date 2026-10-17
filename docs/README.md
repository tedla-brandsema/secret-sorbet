# Repository documentation

Store repository documentation under `docs/` and implementation plans under
`docs/plans/`. Keep application usage instructions beside each application.
`docs/TODO.md` is the canonical backlog of accepted unfinished work. Plans describe
proposed changes; code, tests, and recorded verification establish what is shipped.

## Architecture boundaries

- `lib/core` owns deterministic derivation, canonical context encoding, the KDF,
  byte stream, and output policies. It returns errors and does not configure a
  process logger or integrate with platform services.
- `lib/pw` depends on `lib/core` and owns login defaults, input preparation, and
  validation. Applications should use this layer for login passwords.
- `app/cli` depends on `lib/pw` and owns terminal input, explicit secret output,
  clipboard handling, and diagnostic configuration.
- `app/desktop`, `app/ios`, `app/android`, and `app/browser` currently contain
  design notes. Their UIs and bindings are planned, not implemented or verified.
- `test-vectors` contains synthetic compatibility fixtures. Never replace a
  fixture with production credentials or regenerate expected values in a test.

Dependencies flow from applications to password helpers to the core. Platform
adapters must not independently reimplement derivation or change inputs without
an explicit compatibility contract.

## Contributor checks

The workspace uses Rust edition 2024 and Cargo resolver 3. CI installs the stable
Rust toolchain; the repository does not pin a compiler or declare an MSRV.
Edition 2024 alone does not establish the dependency graph's minimum compiler.
Run commands from the repository root and use the checked-in lockfiles.

```sh
cargo build --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo audit
cargo audit --no-fetch --file fuzz/Cargo.lock
```

`cargo fmt` defines Rust formatting. Install `rustfmt` and `clippy` for the selected
toolchain. CI installs `cargo-audit` 0.22.2. Auditing requires access to the RustSec
database and registry index; record access failures as unverified checks.

For changes to policy validation or encoding, also run:

```sh
cargo +nightly fuzz run policy -- -runs=10000
```

This requires nightly Rust and `cargo-fuzz` 0.13.2. Keep the fuzz hot loop bounded
and avoid invoking Argon2 there. Documentation-only changes require link, path,
and diff checks; report any baseline Rust checks actually performed.

Use one lowercase Conventional Commit type per coherent reviewed slice, for
example `docs: define the portability plan`. Stage only intended files. Commit
and push only within the authorization given for the task; never rewrite accepted
history or discard unrelated work.

## Plans and progress

The current implementation roadmap is
[portability and deterministic derivation](plans/portability-and-determinism.md).
Track its unfinished slices in the [canonical TODO](TODO.md).

Use stable slice IDs, explicit dependencies, file ownership, acceptance checks,
and documentation requirements. Record progress with these markers:

- `[x]` completed, reviewed, and committed; record its commit hash.
- `[>]` active slice.
- `[ ]` known future work.
- `[~]` partially completed phase.

Update the canonical TODO when work is deferred or accepted. A distinct roadmap
reconciliation may be committed separately after the slice hash is known.

## Generated state and security

Keep `target/`, `graphify-out/`, `fuzz/target/`, `fuzz/corpus/`, `fuzz/artifacts/`,
and Rust backup files ignored as specified by the root `.gitignore`. The existing
graph may contain historical paths and claims; confirm them against current
sources. Do not rewrite generated local state as part of a documentation rename.

Follow [AGENTS.md](../AGENTS.md) for secret handling, safe diagnostics, tests,
and documentation requirements, and [SECURITY.md](../SECURITY.md) for private
vulnerability reporting. Use synthetic inputs in fixtures and review evidence.
Destructive operations require explicit authorization.

## Documentation delivery

This ledger tracks the requested documentation setup, separately from future
portability implementation slices.

| Slice | Status | Outcome | Dependencies | Commit |
| --- | --- | --- | --- | --- |
| D01 | [x] | Rename the empty documentation directory and establish repository conventions | None | `f19a75b` |
| D02 | [>] | Review and save the portability plan and canonical backlog | D01 | Pending |
