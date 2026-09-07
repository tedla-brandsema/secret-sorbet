# Repository Engineering Standard

This file applies to the entire repository. Logging, testing, and documentation
are first-class product work. A change is not complete when its implementation
works; the relevant observability, verification, and documentation must also be
production quality.

## General Expectations

- Prefer clear, maintainable designs over clever or speculative abstractions.
- Preserve security, deterministic behavior, portability, and user ownership.
- Treat warnings, flaky tests, misleading documentation, and unsafe diagnostic
  output as defects.
- Keep changes focused. Do not mix unrelated refactors into feature or bug-fix
  work.
- Never preserve an insecure implementation solely for hypothetical backwards
  compatibility. Any intentional compatibility contract must be explicit and
  protected by checked-in vectors or fixtures.

## Logging and Diagnostics

- Logging is a designed interface. Use structured, contextual events with
  stable names and appropriate levels; do not add noisy line-by-line tracing.
- Never place a passphrase, generated password, encryption or HMAC key, token,
  authenticator secret, recovery material, vault plaintext, or equivalent
  sensitive value in logs, diagnostic output, or errors. A generated credential
  may only be emitted through an explicit user-facing secret-output channel.
- Treat usernames, email addresses, service names, file paths, and provider
  identifiers as potentially sensitive. Omit, redact, or minimize them unless
  they are essential to an explicitly documented diagnostic.
- Error messages must explain the failed operation and safe remediation without
  revealing secret input. Preserve underlying causes when that is safe.
- Shared libraries return typed, contextual errors and must not initialize a
  process-wide logger. Each application owns logger configuration, destinations,
  retention, and redaction at its boundary.
- Keep user-facing command output separate from diagnostic logs. CLI diagnostics
  go to standard error; requested machine-readable output must remain stable and
  uncontaminated.
- New background work, storage operations, external calls, and security-relevant
  state transitions require enough safe telemetry to diagnose failures. Document
  what is intentionally not observable for security reasons.

## Testing

- Every behavior change requires tests at the lowest useful level. Every bug fix
  requires a regression test that fails without the fix.
- Use unit tests for local invariants, integration tests for crate and application
  boundaries, compatibility vectors for deterministic contracts, property tests
  for broad invariants, and fuzz targets for parsers or adversarial input spaces.
- Security boundaries require positive and negative tests, including invalid,
  empty, extreme, and overflow-prone inputs. Secret-redaction behavior must be
  tested wherever diagnostic representations are introduced.
- Tests must be deterministic, isolated, and repeatable. Avoid dependencies on
  network access, wall-clock timing, execution order, or unseeded randomness.
- Do not weaken assertions, reduce coverage, add broad ignores, or accept flaky
  behavior to make a gate pass. Fix the implementation or explain a narrowly
  scoped exception in code and external documentation.
- Any intentional change to deterministic derivation must update the algorithm
  identifier or compatibility contract as appropriate and regenerate checked-in
  vectors in the same change.
- New fuzz targets must have bounded resource use, assert meaningful invariants,
  and avoid expensive primitives in the hot loop unless that primitive is the
  subject of the fuzz campaign.
- Before handing off a change, run all relevant local gates. The repository-wide
  baseline is:

  ```sh
  cargo fmt --all -- --check
  cargo clippy --locked --workspace --all-targets -- -D warnings
  cargo test --locked --workspace
  cargo audit
  ```

  Run the applicable fuzz targets for changes that affect their input surface.

## Documentation

- Documentation inside and outside code is part of the implementation and must
  be updated in the same change as the behavior it describes.
- Public APIs require useful rustdoc covering purpose, important invariants,
  security implications, error behavior, and examples where they improve safe
  use. Document panics and unsafe requirements whenever applicable.
- Comments should explain reasoning, invariants, formats, and non-obvious safety
  constraints. Do not narrate syntax or leave stale commented-out code.
- Keep the README, files under `.docs/`, application documentation, examples,
  compatibility-vector metadata, and operational instructions consistent with
  the current implementation.
- Architecture and security documentation must distinguish implemented behavior,
  planned work, accepted risk, and externally audited claims. Never present a
  plan or assumption as a shipped guarantee.
- User-facing behavior must document safe usage, failure modes, recovery steps,
  compatibility impact, and migration requirements where relevant.
- Examples and commands must be executable and kept current. Prefer linking to a
  single authoritative explanation instead of duplicating details that can drift.

## Completion Review

Before considering work complete, verify:

1. Logging and errors are sufficient to diagnose failures without exposing
   sensitive data.
2. Tests cover the success path, relevant failures, boundary conditions, and any
   changed deterministic contract.
3. Rustdoc, comments, README content, architecture notes, examples, and operator
   instructions affected by the change are accurate.
4. Formatting, linting, tests, dependency auditing, and applicable fuzzing pass.

If one of these areas is not applicable, that should be evident from the scope of
the change rather than silently omitted.
