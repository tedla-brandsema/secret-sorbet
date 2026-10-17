# Portability and deterministic derivation implementation plan

This plan defines implementation scope and acceptance; creating it does not
implement or verify a portability guarantee. The [roadmap](../ROADMAP.md) owns
phase and slice progress, the checkbox legend, and completion evidence.
[TODO](../TODO.md) records accepted outstanding work; the
[documentation index](../README.md) explains repository conventions.
New plans belong in `docs/plans/`. Do not maintain progress checkboxes here.

The portability implementation delivery ends at Phase 2 (P01–P05). P06 and P07 are
deferred at the user's request; their decision/device evidence is not required
to close Phase 2. P08 remains the broader future portability release gate.
The separately authorized [v0.1.0 development source release](v0.1.0-release.md)
has its own verification and publication slices; it does not complete P08.

## Baseline evidence and scope

This section records the planning baseline at `7fd4ffc`, before implementation.
Current status and execution evidence live in the roadmap and
[support contract](../support.md).

The current [core](../../lib/core/src/lib.rs) uses `std`, heap allocation,
Argon2id v19 (65,536 KiB, three iterations, four lanes, 32-byte key), SHA-256
salt derivation, and an HMAC-SHA-512 stream. Context lengths and stream counters
use big-endian u64 bytes. The [dependency configuration](../../lib/core/Cargo.toml)
enables Argon2 `alloc`, `parallel`, and `zeroize`; runtime/thread requirements
must be checked rather than inferred from the lane count.

There is one [login vector](../../test-vectors/default-login-v1.json).
Its [harness](../../lib/pw/tests/compatibility_vectors.rs) reconstructs a login
from effective `variables`; it cannot establish raw-input trimming behavior.
The [CI configuration](../../.github/workflows/rust.yml) declares six native
jobs: Linux x86_64/ARM64, Windows x86_64/ARM64, and macOS Intel/Apple Silicon.
Configuration is evidence of intended coverage, not proof those jobs passed.
Validation during this documentation delivery covered the workspace build,
format check, clippy with warnings denied, workspace tests (48 tests), and both
workspace/fuzz-lock dependency audits: all passed on rustc 1.98.0, x86_64 Linux.
No remote CI runs were verified.

The [iOS](../../app/ios/README.md) and [Android](../../app/android/README.md)
directories are application placeholders. Existing architecture/application
proposals must remain labelled planned. Ignored, stale `graphify-out` material
is not authoritative and is outside this implementation plan's edits.

## Intended contract

Support policy will name tested targets and requirements: Rust `std`, usable heap
allocation, at least the fixed 64 MiB Argon2 working memory **plus** application,
allocator, stack, and parallel execution overhead, and a tested 32- or 64-bit
address space. Document actual dependency/thread requirements and supported
Rust toolchains. Do not promise every Rust target, `no_std`, allocation-free
operation, embedded execution, or mobile application availability.

For v1, identical input bytes and policy/order must retain identical output bytes.
Passphrase bytes are not normalized. Login service, username, and counter use
Rust Unicode-aware `trim`; service then uses ASCII-only lowercasing. Username
and counter retain case; non-ASCII service letters are not lowercased. Core
variable order, class order, labels, raw class bytes (including duplicates),
and explicit-allowed versus class-union mode enter the context. Deduplication
for selection does not make distinct context bytes interchangeable.

Visually identical NFC and NFD text may produce different v1 passwords. Rust's
Unicode whitespace tables can change with toolchains; pin the tested baseline
and fixtures for boundary characters, then review toolchain upgrades against
it. A changed trim result is a compatibility event, not an automatic vector
refresh. P06 is an explicit product decision about any *opt-in* NFC profile;
no normalization change is approved here and v1 must never silently change.

## Execution phases and dependencies

| Phase | Slices | Entry condition |
| --- | --- | --- |
| Safety and contract | P01, P02 | Review this plan and allocate owners |
| Portable evidence | P03, P04, P05 | P01 and checked-in P02 fixtures |
| Product checkpoint | P06 | P02 defines v1; P03 documents support limits |
| Mobile evidence | P07 | P02–P05; mobile harness/toolchain available |
| Release acceptance | P08 | P01–P07 accepted; required execution evidence complete |

P03 may be drafted alongside P02; its final support claims depend on P04/P05.
P06's decision record is required; implementing a new profile is conditional on
explicit approval. P07 is accepted future work, not silently deferred forever.
Each slice is a focused commit; split further when review scope warrants it.
Missing required execution keeps the owning slice and release phase partial;
record the exact limitation in TODO rather than treating documentation as a waiver.

The requested gaps map to P02 (contract/Unicode fixtures), P03 (support policy),
P04/P05 (desktop, 32-bit and endianness execution), P06 (normalization decision),
and P07 (mobile execution). Start with P01: its termination defect blocks safe
expansion of the policy test surface.

<a id="p01"></a>

### P01 — Bound encoder selection before broadening coverage

- **Depends on:** no implementation slices. **Owns:** `lib/core/src/encoder.rs`,
  `policy.rs`, `error.rs`, local/property tests, `fuzz/fuzz_targets/policy.rs`,
  and affected encoder rustdoc/support documentation.
- `choose_index` currently has rejection limit zero for `len > 256`; selecting
  a required-class position with more than 256 free positions never succeeds
  with an infinite byte source. Fix this before adding longer policy vectors.
- Recommend a larger-index sampler: preserve the exact existing single-byte
  selection for sizes ≤256; for larger sizes only, use fixed-width u64
  big-endian rejection sampling with checked size conversions and overflow-safe
  rejection arithmetic. Add meaningful 32/64-bit portability fixtures for that
  mapping. Do not add a global output-length cap.
- Preserve **all** previously successful outputs, including lengths >256 with
  zero total required counts. Newly defined cases are formerly nonterminating
  required-position selections. Change the algorithm/profile identifier only
  if established deterministic bytes change.
- Add a finite byte source with a strict read budget: an old-code regression
  must fail deterministically without hanging the test process. Cover free counts
  255/256/257, zero/minimum counts, exhausted/erroring sources, overflow-prone
  counts, and pinned long outputs with zero total required counts. Exercise
  larger-sampler rejection and conversion boundaries without giant allocations.
- Extend bounded policy fuzz inputs across the 256-position boundary; keep
  resource caps and a budgeted source, meaningful output invariants, and no
  Argon2 in the hot loop. Acceptance: regression, old vectors, policy fuzz,
  debug/release agree.
- **Commit:** `fix(core): bound free-position selection without changing v1 outputs`.

<a id="p02"></a>

### P02 — Pin synthetic byte contracts and raw-input vectors

- **Depends on:** P01 for long-policy cases. **Owns:** `test-vectors/`,
  `lib/pw/tests/compatibility_vectors.rs`, `lib/core/src/context.rs`, `kdf.rs`,
  `stream.rs` test modules, fixture generation tooling, and vector format docs.
- Keep existing vector compatibility while adding explicit case IDs, generation
  provenance, algorithm/profile/KDF metadata, expected bytes, and reviewed
  synthetic-only fixtures. Expected values must be checked in, never computed
  from the implementation under test as the assertion oracle.
- Add raw login fields separately from expected effective variable metadata;
  call `LoginInput` with raw values and compare effective variables and output.
  Include passphrase/username non-ASCII and emoji, multibyte lengths, NFC/NFD
  divergent pairs, ASCII service case equivalence, non-ASCII case distinction,
  Unicode edge whitespace/near-whitespace, and whitespace-only counter failures.
- Expand full derivation cases for counter changes, reordered core variables,
  class order/duplicates, explicit alphabets versus union mode, minimum counts,
  safe length boundaries, and stream consumption spanning multiple blocks.
  Keep costly end-to-end cases small and serialized/resource bounded on emulators.
- Pin canonical context bytes, salt digest/truncation, synthetic derived key,
  HMAC blocks/counter byte order, and encoder byte-consumption fixtures. Place
  private-stage tests inside core modules; do not expose key inspection APIs.
  Exercise length-prefix boundaries cheaply without giant allocations.
- Add invalid/empty/non-ASCII-policy/overflow tests at the lowest useful layer;
  distinguish core validation from login validation. Cover accepted empty-variable
  list behavior deliberately; do not invent stricter behavior as a fixture edit.
- Acceptance: pinned literals reviewed against the byte format, reproducible
  fixture instructions, strict schema validation, preserved old output, and
  deterministic seeded property tests. **Commit:**
  `test(core): pin synthetic v1 bytes and raw login compatibility vectors`.

<a id="p03"></a>

### P03 — Publish a precise support and Unicode baseline

- **Depends on:** P02; final evidence references P04/P05. **Owns:** root README,
  core/pw rustdoc, architecture/security notes, app READMEs, and a support matrix
  under `docs/` (coordinate shared documentation ownership with P08).
- Document the intended contract above, fixed KDF parameters, dependency/thread
  requirements, 32/64-bit limits, tested toolchain, runtime memory overhead, and
  safe failure/recovery guidance. Distinguish configured, built, executed, and
  verified; record evidence links and target triples rather than claiming parity.
- Explain byte-identical v1 inputs, Unicode trim and ASCII lowercase boundaries,
  NFC/NFD divergence, toolchain drift review, and preserving original recovery
  inputs. Do not call service validation full DNS/IDNA normalization.
- Acceptance: docs match source and fixtures; all platform guarantees cite actual
  execution evidence or are explicitly planned. **Commit:**
  `docs: define tested portability requirements and v1 Unicode behavior`.

<a id="p04"></a>

### P04 — Execute vectors on native desktop targets

- **Depends on:** P02. **Owns:** native jobs in `.github/workflows/rust.yml`,
  toolchain baseline configuration, and support-matrix evidence fields.
- Retain all six existing native configurations; resolve actual runner/triple
  mappings from job output. Execute checked-in full and intermediate fixtures,
  plus relevant workspace tests, in debug and release on each architecture.
- Record `rustc -Vv`, target triple, dependency lock identity, job/run links, and
  fixture revision. Bound concurrency for 64 MiB KDF tests; failures remain
  failures and runner unavailability is an evidence gap.
- Acceptance: every declared native target has passing execution evidence; matrix
  labels never imply an unrun platform passed. **Commit:**
  `ci: execute v1 compatibility fixtures across native desktop targets`.

<a id="p05"></a>

### P05 — Execute 32-bit and big-endian vectors under emulation

- **Depends on:** P01/P02; coordinate CI ownership after P04. **Owns:** emulated
  jobs/scripts in `.github/`, target setup documentation, support evidence.
- Add Linux `i686-unknown-linux-gnu`, `armv7-unknown-linux-gnueabihf`, and
  big-endian `s390x-unknown-linux-gnu` execution using reproducible toolchains,
  sysroots and emulation (e.g. QEMU). Verify runtime identity and actually run
  pinned end-to-end and intermediate fixture tests; compilation alone is not
  acceptance. Do not disable parallel dependencies just to claim current support.
- Budget emulator memory/time and KDF concurrency explicitly; run debug/release
  where supported and explain a narrowly scoped missing gate. Record word size,
  endianness, ABI, runtime, and limitations; emulation does not prove device UX.
- Acceptance: expected bytes match all three executed target suites, with logs
  and setup that another reviewer can reproduce. **Commit:**
  `ci: execute compatibility vectors on 32-bit and big-endian targets`.

<a id="p06"></a>

### P06 — Decide whether to add an opt-in normalized profile

- **Depends on:** P02/P03. **Owns:** a decision record under `docs/`, future
  profile proposal and recovery/migration documentation; no v1 code changes.
- Obtain an explicit product checkpoint on whether NFC is useful, which fields
  it would cover, normalization order versus trimming/case rules, dependency and
  Unicode-version policy, and how users select and retain profile identifiers.
- If approved later, scope separate implementation slices with a new explicit
  profile/version, pinned NFC/NFD vectors, old v1 recovery, credential rotation,
  backups/export metadata, and migration UX. Never reinterpret old saved inputs
  or claim a recovery path without tests. Approval, rejection, or deferment must
  record why; each is a valid decision outcome and makes no product behavior
  change in this slice.
- Acceptance: reviewed decision with owner and follow-up scope; approval cannot
  be inferred from this plan. **Commit:**
  `docs: record Unicode normalization product decision and migration constraints`.

<a id="p07"></a>

### P07 — Establish mobile core execution evidence

- **Depends on:** P02–P05. **Owns:** isolated mobile test harnesses, mobile CI,
  `app/ios/README.md`, `app/android/README.md`, and mobile support evidence.
- Build core/pw for named iOS simulator/device and Android ABI targets first;
  label those build checks. Add a minimal simulator/emulator runner to execute
  the same synthetic fixtures, then physical-device execution for claimed device
  support. Record exact OS/ABI/toolchain, thread behavior, memory constraints,
  lifecycle failures, and debug/release results where supported.
- Keep this separate from shipping native apps or selecting a production FFI.
  Any test bridge must preserve UTF-8 bytes and profile/policy metadata; test
  boundary errors/redaction. Never transfer real credentials into test artifacts.
- Acceptance: actual mobile fixture execution is linked per target and clearly
  separated from build-only coverage and placeholder application plans.
- **Commit:** `test(mobile): execute shared derivation fixtures on iOS and Android`.

<a id="p08"></a>

### P08 — Review evidence and close the portability release gate

- **Depends on:** P01–P07 (P06 requires a decision, not mandatory new-profile code).
  **Owns:** final documentation consistency, evidence/release checklist and TODO
  updates, and root `CHANGELOG.md`; fixes return to the slice owning the affected
  code/CI.
- An independent reviewer checks compatibility bytes, support claims, seeded
  tests, resource bounds, error/redaction behavior, and executable instructions.
  Reconcile architecture/app plans with shipped behavior; do not edit stale
  generated graph material to manufacture evidence.
- Review `CHANGELOG.md` against completed slices and actual platform evidence;
  capture compatibility, recovery, security, and known limitations. Prepare the
  accumulated `Unreleased` notes for a later authorized release, following
  [changelog maintenance](../README.md#changelog-and-release-preparation).
  Do not assign a release date, tag, or publication status before release.
- Required local gates: `cargo fmt --all -- --check`,
  `cargo clippy --locked --workspace --all-targets -- -D warnings`,
  `cargo test --locked --workspace`, `cargo test --locked --workspace --release`,
  and `cargo audit`; retain the CI audit of `fuzz/Cargo.lock`. Run affected policy
  fuzz targets (current smoke gate: `cargo +nightly fuzz run policy -- -runs=10000`).
- Libraries return safe typed errors and never initialize a global logger. CI
  may report stage/case IDs, target/toolchain and outcome, but never log real
  passphrases, outputs, context/key/stream bytes or user/provider identifiers.
  Synthetic fixtures may contain intermediate bytes; prevent real-secret
  assertion dumps and document intentionally unobservable internals. New
  diagnostics require positive/negative redaction tests; CLI stderr and explicit
  secret-output channels retain their documented separation.
- Acceptance: gates pass, independent review recorded, docs and fixtures agree,
  limitations are explicit, and each completed slice is reviewed, verified, and
  committed before marking it complete in the roadmap.
  Missing any required platform execution leaves that slice and this release
  phase partial in the roadmap, with the exact remaining requirement in TODO;
  it is not a waiver.
- **Commit:** `docs: record reviewed portability evidence and remaining limits`.
