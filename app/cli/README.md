# Secret Sorbet CLI

The CLI is the first Secret Sorbet application. It provides local deterministic
password generation through the shared `secret-sorbet-pw` crate and stores nothing.

## Build and run

From the repository root:

```sh
cargo build --locked --release -p secret-sorbet-cli
cargo run --locked -p secret-sorbet-cli -- generate \
  --service example.com \
  --username alice@example.com
```

The default input mode prompts for the passphrase without echoing it. Secret Sorbet
does not accept a passphrase through a command-line argument or environment
option, because those values are commonly exposed through process inspection,
shell history, or inherited environments.

Interactive mode prompts for the service, username, and passphrase in that
order. Use either the short or long option:

```sh
seso generate -i
seso generate --interactive
```

`--interactive` cannot be combined with `--service`, `--username`, or
`--passphrase-stdin`. The service and username are visible while entered; the
passphrase remains hidden.

For automation, `--passphrase-stdin` reads up to one passphrase from standard
input and removes one trailing LF or CRLF line ending. Embedded line breaks are
rejected, and passphrases are limited to 1024 UTF-8 bytes to bound memory use.
The producer is responsible for protecting its end of the pipe. Do not put a
literal passphrase in shell history.

```sh
trusted-secret-command | seso generate \
  --service example.com \
  --username alice@example.com \
  --passphrase-stdin \
  --print
```

## Output and diagnostics

By default, the CLI asks `Copy to clipboard for 2 minutes? [Y/n]`. Enter or
`y` copies the password without displaying it. A terminal countdown keeps the
process in the foreground for two minutes. At expiry, Ctrl+C, SIGTERM, or SIGHUP,
the CLI clears the clipboard if it still contains the generated password and
confirms the cleanup before exiting. If the clipboard has since changed, that
newer content is preserved.

These guarantees apply to catchable termination signals. `SIGKILL`, sudden
power loss, an operating-system crash, or forceful process termination cannot
run cleanup; manually replace or clear the clipboard after such an event.

Secret Sorbet requests the strongest history protection exposed by its clipboard
backend: history exclusion on Linux and macOS, and exclusion from clipboard
monitoring on Windows so the value is not added to Windows clipboard history or
cloud clipboard. These controls prevent storage rather than trying to delete a
historical entry later.

Windows applies the exclusion to its OS-managed history and cloud clipboard.
Linux and macOS protections are conventions honored by compatible clipboard
managers, not operating-system enforcement. Independent third-party managers can
still ignore the platform metadata on any system and retain their own copies,
which the CLI cannot remove through a portable interface. The final cleanup
confirmation therefore refers only to the active clipboard.

Use `--print` for intentional display, redirection, or automation. In print mode,
stdout contains exactly the generated password followed by a newline and the
clipboard prompt is skipped. Terminal output and redirected files may retain the
password, so handle them accordingly.

Diagnostics go only to stderr. `--verbose` enables safe lifecycle events and
`--verbose --verbose` enables debug-level diagnostics. Diagnostic events never
contain the passphrase, generated password, service, username, or counter.
Third-party dependency logs are suppressed so backend fallback messages do not
pollute the user-facing diagnostic stream.

The process exits with status `0` on success, `1` on input or generation failure,
and clap uses status `2` for invalid command-line arguments.

## First-version scope

The initial command uses the default login policy and accepts:

- service domain
- username or email
- revision counter, defaulting to `1`
- a passphrase from a hidden prompt or standard input

Service and username are command-line options by default. With `-i` or
`--interactive`, all three mandatory inputs are collected by prompts instead.

Custom output policies, shell completions, and vault commands are not part of
this first version.
