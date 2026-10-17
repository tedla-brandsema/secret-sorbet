# Repository documentation

Store repository documentation under `docs/` and implementation plans under
`docs/plans/`. Keep application usage instructions beside each application.
`docs/ROADMAP.md` is the authoritative phase and slice progress tracker;
`docs/TODO.md` is the canonical backlog of accepted unfinished work. Plans define
scope and acceptance; code, tests, and recorded verification establish what is
shipped.

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

The workspace uses Rust edition 2024 and Cargo resolver 3. The repository pins
Rust 1.98.0 in `rust-toolchain.toml`, and CI installs that baseline explicitly.
This is a tested baseline, not a declared minimum supported Rust version.
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

The native CI harness executes serialized debug and release workspace tests on
six desktop targets and retains metadata with compiler/runtime identity, tested
commit, lockfile and fixture hashes, and command outcomes. It refuses dirty
checkouts. To reproduce it locally from a clean checkout, keep its output outside
the worktree so it does not make the next run dirty:

```sh
python3 -B -m unittest discover -s .github/scripts -p 'test_*.py'
python3 .github/scripts/test-evidence.py \
  --target x86_64-unknown-linux-gnu --runner local \
  --output /tmp/secret-sorbet-native-evidence.json
```

The second command is for a native x86-64 GNU Linux host; choose the host's actual
Rust target on other systems. Harness Python requires 3.11 or newer (`tomllib`);
CI uses Python 3.12. Metadata is retained for 90 days in GitHub Actions artifacts;
durable support claims must retain run links and fixture/lock identity in docs.

The emulated jobs run only the shared core/password suites: CLI integration tests
spawn child executables that would need separate binfmt setup. The harness
compiles and executes a target identity probe under QEMU before testing; a
cross-build alone is insufficient. To reproduce the s390x job on Ubuntu 24.04:

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends -y \
  qemu-user gcc-s390x-linux-gnu libc6-dev-s390x-cross
rustup target add --toolchain 1.98.0 s390x-unknown-linux-gnu
python3 .github/scripts/test-evidence.py \
  --target s390x-unknown-linux-gnu --runner ubuntu-24.04 \
  --emulator qemu-s390x --linker s390x-linux-gnu-gcc \
  --sysroot /usr/s390x-linux-gnu --system-package qemu-user \
  --system-package gcc-s390x-linux-gnu --system-package libc6-dev-s390x-cross \
  --output /tmp/secret-sorbet-s390x-evidence.json
```

The workflow defines equivalent i686 and ARMv7 configurations. It preserves
Argon2's parallel feature and serializes tests, with three emulated jobs at most
and a 45-minute limit per job. Setup metadata records installed package versions,
QEMU/compiler-driver versions, target ABI, and sysroot loader/libc hashes. Ubuntu
packages can change between runs; each run revalidates the pinned byte fixtures
and records the actual setup rather than claiming an immutable system image.

Use one lowercase Conventional Commit type per coherent reviewed slice, for
example `docs: define the portability plan`. Stage only intended files. Commit
and push only within the authorization given for the task; never rewrite accepted
history or discard unrelated work.

## Plans and progress

Use the [roadmap](ROADMAP.md) for phases, slice dependencies, current progress,
completion evidence, and the checkbox legend. The detailed
[portability plan](plans/portability-and-determinism.md) defines implementation
scope and checks; the [canonical TODO](TODO.md) records unfinished work and
deferrals.

Use stable slice IDs, explicit dependencies, file ownership, acceptance checks,
and documentation requirements. Apply the roadmap's checkbox legend there;
link to that progress from other documents rather than duplicating status.

Update the canonical TODO when work is deferred or accepted. A distinct roadmap
reconciliation may be committed separately after the slice hash is known. Update
the roadmap after every slice, including its review and verification evidence,
commit hash, phase progress, and next slice.

## Changelog and release preparation

Keep the root [CHANGELOG.md](../CHANGELOG.md) current under `Unreleased` as
reviewed slices land. Describe release-relevant behavior and documentation in
terms useful to users; include compatibility, security, migration, or recovery
impact where applicable. Group entries under Added, Changed, Deprecated,
Removed, Fixed, or Security as needed; omit empty categories. Keep planned work
in the roadmap until it is implemented and verified.

At P08, review the accumulated entries against the completed roadmap, actual
platform evidence, compatibility fixtures, and remaining limitations. After the
release gates pass and a release is authorized, move the entries into a section
with the chosen version and actual release date (`YYYY-MM-DD`), newest first,
and retain an `Unreleased` section for subsequent work. Keep Cargo versions,
release notes, and any approved tag consistent. Creating a changelog or completing
the roadmap does not authorize tagging, publishing packages, or creating a release.

## Generated state and security

Keep `target/`, `graphify-out/`, `fuzz/target/`, `fuzz/corpus/`, `fuzz/artifacts/`,
and Rust backup files ignored as specified by the root `.gitignore`. The existing
graph may contain historical paths and claims; confirm them against current
sources. Do not rewrite generated local state as part of a documentation rename.

Follow [AGENTS.md](../AGENTS.md) for secret handling, safe diagnostics, tests,
and documentation requirements, and [SECURITY.md](../SECURITY.md) for private
vulnerability reporting. Use synthetic inputs in fixtures and review evidence.
Destructive operations require explicit authorization.

Documentation delivery history is recorded in the roadmap alongside the
implementation phases.
