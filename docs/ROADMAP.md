# Implementation roadmap

This is the authoritative phase and slice progress tracker. The
[detailed plan](plans/portability-and-determinism.md) defines scope, ownership,
tests, documentation and acceptance. [TODO](TODO.md) records accepted outstanding
and deferred issue details. Those documents link here instead of maintaining
independent progress states. New detailed plans belong in `docs/plans/`.

## Status and update rules

- `[ ]` known future work.
- `[>]` active slice.
- `[x]` completed, reviewed, verified and committed, with actual commit hashes.
- `[~]` partially completed phase or work; state what remains.

On starting a slice, move the active marker to that slice and mark its phase
partial once work starts. Use `- [ ]` and `- [x]` checkbox items; replace the
marker with `[>]` or `[~]` when active or partial. Mark a phase complete only
when every required slice is complete. Missing required execution evidence keeps
started work and its phase partial; record the exact missing check/target in
TODO. Configured jobs and successful compilation do not establish execution.

Record review and verification evidence with each completed slice. After its
implementation commit hash is known, update this roadmap in a separate
reconciliation commit, recording the actual hash rather than guessing a future
hash. Keep the active marker current and reconcile TODO issue details as work
lands. Update the root [changelog](../CHANGELOG.md) with release-relevant changes
under `Unreleased` in the implementation slice. An unfinished reconciliation
must not imply a slice is complete.

The authorized delivery boundary is the end of Phase 2 (P01–P05), including
review, verification, commits, and pushes to `origin/main`. **Active: P04**.
P06 and P07 are deferred at the user's request; P08 remains outside this run.
Deferral does not establish Unicode-profile or mobile execution evidence.

## Phase 1 — Safety and byte contract

- [x] Phase 1: P01 and P02 are reviewed, verified, and committed.

<a id="p01"></a>

- [x] **P01 — Fix free-position selection nontermination.** Preserve all successful
  v1 outputs; add u64 big-endian sampling only above 256 and bounded regressions.
  Depends on: none. [Scope and acceptance](plans/portability-and-determinism.md#p01).
  Independently reviewed; formatting, Clippy, serial workspace tests, both
  dependency audits, six release encoder tests, and 10,000 policy fuzz runs passed.
  The finite 257-position regression fails against the previous implementation.
  Commit: `2cb25fe`.

<a id="p02"></a>

- [x] **P02 — Pin synthetic compatibility contracts.** Expand raw/effective login,
  intermediate byte, end-to-end, Unicode and invalid/boundary fixtures.
  Depends on: P01 for long-policy cases.
  [Scope and acceptance](plans/portability-and-determinism.md#p02).
  Independently reviewed and reproduced with Python/libargon2. Formatting,
  Clippy, and all 68 workspace tests passed in debug and release; the original
  v1 fixture is unchanged. Dependency lockfiles are unchanged from P01's audits.
  Commit: `8b24697`.

## Phase 2 — Portable execution evidence

- [~] Phase 2: P04 is active; P03's support documentation will incorporate
  execution evidence after P04/P05, as its dependencies require.

<a id="p03"></a>

- [ ] **P03 — Publish support requirements and v1 Unicode behavior.** Document
  std/heap, memory overhead, dependency/thread and 32/64-bit requirements.
  Depends on: P02; documentation may be drafted while execution is pending,
  with final evidence references completed after P04/P05 (no execution cycle).
  [Scope and acceptance](plans/portability-and-determinism.md#p03).
  Completion evidence: none. Commit: none.

<a id="p04"></a>

- [>] **P04 — Execute fixtures on native desktops.** Verify debug/release bytes
  on all six configured native desktop targets and retain execution evidence.
  Depends on: P02. [Scope and acceptance](plans/portability-and-determinism.md#p04).
  Completion evidence: none. Commit: none.

<a id="p05"></a>

- [ ] **P05 — Execute 32-bit and big-endian fixtures.** Run i686, ARMv7 and s390x
  under emulation with reproducible setup and explicit resource bounds.
  Depends on: P01/P02; coordinate CI ownership with P04.
  [Scope and acceptance](plans/portability-and-determinism.md#p05).
  Completion evidence: none. Commit: none.

## Phase 3 — Unicode product checkpoint

- [ ] Phase 3: complete P06's documented decision; no normalization change approved.

<a id="p06"></a>

- [ ] **P06 — Decide on an opt-in NFC profile.** Record approval, deferment or
  rejection, with versioning and recovery/migration constraints; preserve v1.
  Depends on: P02/P03. A new profile requires explicit approval and later scope.
  [Scope and acceptance](plans/portability-and-determinism.md#p06).
  Deferred by user for this Phase 2 delivery; no normalized profile is approved.
  Completion evidence: none. Commit: none.

## Phase 4 — Mobile core evidence

- [ ] Phase 4: complete P07's actual mobile execution, distinct from app delivery.

<a id="p07"></a>

- [ ] **P07 — Execute shared fixtures on iOS and Android.** Establish build checks,
  simulator/emulator execution and named physical-device evidence for claims.
  Depends on: P02–P05; availability of mobile harnesses/toolchains/devices.
  [Scope and acceptance](plans/portability-and-determinism.md#p07).
  Deferred by user for this Phase 2 delivery; no mobile execution is claimed.
  Completion evidence: none. Commit: none.

## Phase 5 — Release acceptance

- [ ] Phase 5: complete P08 after required execution evidence exists.

<a id="p08"></a>

- [ ] **P08 — Review and verify the portability release.** Complete local gates,
  independent review, safe diagnostics and consistent evidence-backed docs;
  prepare accurate changelog entries for the subsequent authorized release.
  Depends on: P01–P07; P06 requires a decision, not new-profile implementation.
  Missing required execution leaves this phase partial, not complete with a waiver.
  [Scope and acceptance](plans/portability-and-determinism.md#p08).
  Completion evidence: none. Commit: none.

## Documentation delivery

- [x] **D01 — Establish documentation layout and conventions.** Reviewed and
  verified directory/layout documentation; committed as `f19a75b`.
- [x] **D02 — Save the portability plan and canonical backlog.** Reviewed plan,
  links, scope and baseline checks; committed as `7fd4ffc`.
- [x] **Documentation reconciliation.** Recorded completed D01/D02 deliveries
  and their actual hashes; reviewed and verified; committed as `e87ab39`.
- [x] **D03 — Establish the authoritative roadmap.** Reviewed phase/slice
  checkboxes, the single progress legend, links and historical commit references;
  committed as `c7364ca`.
- [x] **D04 — Establish the release changelog.** Create root `CHANGELOG.md`,
  document per-slice maintenance, and connect final release-note review to P08.
  Depends on: D03. Independently reviewed; 44 local links/anchors, changelog
  structure, future-slice status and diff checks passed. Commit: `6dfae73`.

Completed documentation items do not establish portability implementation or
unverified remote execution.
