# Portability and v1 compatibility

This contract applies to the shared Rust libraries. A target being supported by
Rust does not establish Secret Sorbet compatibility or application availability.
The [roadmap](ROADMAP.md) records implementation progress; execution evidence
below determines verified coverage.

Version 0.1.0 is an initial-development source release. The execution evidence
below applies to the specific tested revision and does not establish universal
platform support or an external security audit. See the
[security policy](../SECURITY.md).

## Runtime and toolchain requirements

- Rust 1.98.0 is the tested baseline, pinned in `rust-toolchain.toml`. This is
  a reproducible baseline, not a claim about the minimum compiler version.
- The current libraries require `std`, heap allocation, and a 32-bit or 64-bit
  address space. The locked Argon2 dependency rejects other pointer widths.
- Argon2id v19 uses 65,536 KiB of working memory, three passes, four lanes, and
  a 32-byte key. Allow additional memory for the allocator, stacks, Rayon
  execution, input context, output buffers, and the calling application.
- Argon2's `parallel` feature uses Rayon. The runtime must support the dependency's
  synchronization and thread-pool initialization; lane count is a cryptographic
  parameter, not a promise about available CPU cores or actual worker count.
- Policy lengths and counts use `usize`. Inputs must be representable and fit
  memory on every target being compared. Extreme lengths can exhaust resources;
  ordinary typed validation errors do not make allocation or thread-pool failure
  recoverable. Resource exhaustion may abort or panic. Callers must bound input,
  output, and parallel requests for their deployment.

There is no automatic lower-memory KDF or alternative-algorithm fallback.
Changing KDF parameters would change passwords. `no_std`, embedded targets,
WebAssembly, and mobile application delivery are outside verified coverage.
Rust itself has [different target support tiers](https://doc.rust-lang.org/rustc/platform-support.html),
including targets without a full standard library.

## Exact input and output contract

The algorithm identifier remains `secret-sorbet-core-v1`. Inputs are Rust UTF-8
strings, not OS-native string layouts. For the same effective input bytes,
ordered variables, ordered policy, and algorithm version, successful derivations
must produce the same ASCII output bytes. Native endianness, locale, clock,
randomness, and CPU count do not enter derivation.

The core preserves variable order, names, and values without normalization.
Names, class names, class order, character order and duplicates, output length,
minimum counts, and implicit-versus-explicit allowed characters enter the
length-prefixed context. Identical effective alphabets alone are insufficient
to establish identical contexts. Lengths and HMAC counters are fixed-width
big-endian u64 values. The [fixture documentation](../test-vectors/README.md)
specifies synthetic byte fixtures and their independent generation.

The login helper prepares its fields as follows:

| Input | v1 preparation |
| --- | --- |
| Passphrase | Preserve exact UTF-8 bytes; no trimming, case folding, or NFC/NFD normalization |
| Service | Trim Unicode whitespace at both ends, then lowercase ASCII letters only |
| Username | Trim Unicode whitespace at both ends; preserve case |
| Counter | Trim Unicode whitespace at both ends; preserve the remaining text, including leading zeroes |

Service validation is a shape check (a dot and no whitespace), not DNS or IDNA
canonicalization. The core and login helper have different validation boundaries;
for example, an empty core variable list is accepted, but login requires a
nonempty service and username, and effective variable values cannot be empty.

Composed `é` and decomposed `e` plus U+0301 look alike but have different v1
bytes and can produce different passwords. U+200B and U+FEFF are not trimmed.
Python's `str.strip()` is not a substitute for Rust's whitespace rules. The
pinned Rust baseline reports Unicode 17.0.0; fixture tests guard whitespace and
near-whitespace boundaries. Treat a toolchain upgrade that changes effective
input as a compatibility event: investigate it instead of refreshing vectors.
NFC normalization and mobile verification are deferred by the user (P06/P07).

The CLI additionally removes one trailing LF or CRLF in stdin mode and rejects
embedded line breaks and inputs over 1024 UTF-8 bytes. See the
[CLI guide](../app/cli/README.md) for input and secret-output handling.

## Encoder selection and recovery

Selections among 1–256 values retain the original single-byte rejection mapping.
Larger free-position selections consume eight bytes per attempt in big-endian
order; accept a u64 draw when it is at least `2^64 mod range`, then take its
remainder. This defines previously nonterminating required-position selections
without changing any previously successful v1 output. Long outputs with zero
required positions retain their original mapping.

The encoder validates policies before reading its source and propagates source
errors. A custom source that only supplies rejected draws can run indefinitely;
untrusted sources must enforce a read budget. No fixed iteration limit silently
changes successful password derivation.

For recovery, retain the original passphrase bytes, service, username, counter,
full policy, and profile identifiers using the user's chosen secure process.
If a derived password differs, stop before rotating or replacing credentials.
Check those inputs and the algorithm/toolchain/fixture identity first. Do not
try normalization, changed KDF parameters, or a guessed counter as an automatic
migration. Existing compatibility fixtures must remain unchanged unless an
explicit versioned contract change is approved.

## Execution coverage

All nine suites below passed at `fd01cbc` with Rust 1.98.0 in
[run 39022373134](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134).
Native jobs execute the 68-test workspace; emulated jobs execute the 46-test
core/password suites. Both profiles include the 16 full login vectors and
private intermediate contracts. The native suites also passed at `7e7333b`.
No successful cross-build alone establishes execution parity.

| Mode | Target | Word size / byte order | Execution evidence |
| --- | --- | --- | --- |
| Native Linux | `x86_64-unknown-linux-gnu` | 64 / little | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567741) |
| Native Linux | `aarch64-unknown-linux-gnu` | 64 / little | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567684) |
| Native Windows | `x86_64-pc-windows-msvc` | 64 / little | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567705) |
| Native Windows | `aarch64-pc-windows-msvc` | 64 / little | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567766) |
| Native macOS | `x86_64-apple-darwin` | 64 / little | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567704) |
| Native macOS | `aarch64-apple-darwin` | 64 / little | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567720) |
| QEMU Linux | `i686-unknown-linux-gnu` | 32 / little | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567708) |
| QEMU Linux | `armv7-unknown-linux-gnueabihf` | 32 / little | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567702) |
| QEMU Linux | `s390x-unknown-linux-gnu` | 64 / big | [debug + release passed](https://github.com/tedla-brandsema/secret-sorbet/actions/runs/39022373134/job/117366567689) |

The [durable evidence snapshot](verification/phase-2.json) records the observed
run/job outcomes, successful execution-step conclusions, artifact IDs/digests,
source commits, and lock/fixture identities. Outcomes were checked through the
public GitHub REST API; the committed harness validates runtime identity before
executing the suites. Artifact archive downloads required authentication (HTTP
401), so payload contents were not independently inspected. The archives retain
actual compiler/runtime and system-package metadata for authenticated inspection;
CI artifact retention is 90 days. This distinction is preserved in the snapshot.

CI evidence records the actual tested commit, compiler, target runtime identity,
dependency lock hash, fixture hashes, and debug/release command exit status.
Only a complete passing suite establishes verified execution. Native execution,
emulation, and build-only results must be labelled separately. Emulator parity
does not establish device performance or production application behavior.

## Safe diagnostics

Libraries return errors and do not configure a global logger. Diagnose failures
using stage/case IDs, algorithm identifiers, target/runtime metadata, and command
status. Do not log passphrases, generated passwords, real context/key/stream
bytes, account identifiers, or policies associated with real accounts. Synthetic
fixtures contain intermediate bytes intentionally; they are public test data.
Production intermediates are intentionally not observable. `LoginInput` debug
output redacts the passphrase only; its other fields remain sensitive and must
not be logged as an input bundle. CLI diagnostics remain separate on stderr.
