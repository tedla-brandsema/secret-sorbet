# Changelog

This changelog tracks notable changes. `Unreleased` contains implemented changes
awaiting release. Planned portability work is tracked in the
[roadmap](docs/ROADMAP.md).

## Unreleased

### Added

- A Rust core for deterministic password derivation using the versioned
  `secret-sorbet-core-v1` profile, Argon2id, and an HMAC-SHA-512 byte stream.
- Customizable output policies with ASCII character classes, allowed character
  sets, and minimum counts.
- Login password helpers with default login variables, input validation, and a
  revision counter for password rotation.
- The `seso` CLI, with hidden passphrase prompts, interactive input, and
  passphrase input from standard input for automation.
- Opt-in password printing and clipboard delivery with timed cleanup behavior;
  platform clipboard history protections depend on OS support and third-party
  clipboard managers.
- Safe CLI diagnostics on standard error, separated from requested password
  output.
- A checked-in compatibility vector, property tests, a policy fuzz target, and
  CI configuration for formatting, linting, tests, dependency audits, fuzz
  smoke checks, and six native desktop targets. Configured targets do not by
  themselves establish that cross-platform execution has been verified.
- Fifteen additional synthetic raw-login vectors and independent Python/libargon2
  reproduction, plus pinned context, salt, key, HMAC, Unicode, and policy-boundary
  contracts. The original v1 vector remains unchanged.
- Repository documentation conventions, an implementation plan, and a
  canonical TODO list linked to the roadmap.

### Changed

- Repository documentation lives under `docs/`, with implementation plans under
  `docs/plans/` and phase/slice progress in `docs/ROADMAP.md`.

### Security

- CLI diagnostics omit passphrases, generated passwords, service names,
  usernames, and counters.
- Passphrases held by login inputs use zeroizing memory and are redacted from
  their debug representation.
- Clipboard cleanup preserves newer contents when verification succeeds and
  clears the clipboard for safety when verification fails. Cleanup and history
  protection limits are described in the [CLI guide](app/cli/README.md).

### Fixed

- Custom policies longer than 256 characters can now place required character
  classes instead of looping indefinitely. Existing successful v1 outputs retain
  their byte mapping, including long policies with no required positions.
