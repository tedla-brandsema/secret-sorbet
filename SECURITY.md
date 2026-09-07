# Security Policy

Secret Sorbet handles credentials and therefore treats security reports as
first-class project work. The project is currently pre-release and has not
undergone an independent security audit.

## Supported Versions

There are no supported releases yet. Reports should be evaluated against the
latest commit on `main`. This policy will be updated with an explicit supported
version matrix before the first stable release.

## Reporting a Vulnerability

Use [GitHub private vulnerability reporting][report] to report a suspected
vulnerability. Do not open a public issue or discussion containing vulnerability
details.

If private vulnerability reporting is unavailable, open a public issue that
asks the maintainers to provide a private reporting channel. Do not include the
vulnerability, reproduction steps, credentials, or other sensitive information
in that issue.

Please include, when applicable:

- the affected commit or version and operating system
- the security impact and plausible attack scenario
- minimal reproduction steps or a proof of concept using synthetic secrets
- whether the issue has been disclosed anywhere else
- a suggested mitigation, if you have one

Never submit a real passphrase, generated password, account identifier, vault,
token, recovery code, or other credential. Replace sensitive values with
synthetic test data before sharing logs, screenshots, or reproduction material.

The maintainers will validate the report, coordinate remediation and disclosure
with the reporter, and use a GitHub security advisory when appropriate. Response
and remediation times depend on severity, complexity, and maintainer
availability; this pre-release project does not promise a fixed service level.
Please avoid public disclosure until a fix is available or a disclosure date has
been coordinated.

## Relevant Security Areas

Reports are especially useful when they concern:

- deterministic derivation, domain separation, or compatibility contracts
- memory-hard key derivation and parameter handling
- secret lifetime, copying, zeroization, or diagnostic disclosure
- command-line input, output, terminal, signal, and clipboard boundaries
- output-policy validation, bias, panics, or resource exhaustion
- dependency, build, release, or supply-chain integrity

Reports about documented limitations are welcome when they demonstrate a new or
materially greater impact. General feature requests and non-security defects
should use the public issue tracker after the repository is published.

[report]: https://github.com/tedla-brandsema/secret-sorbet/security/advisories/new
