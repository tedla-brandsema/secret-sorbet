# Public synthetic v1 compatibility fixtures

Every value here is public synthetic test data, including the original sample.
Never replace these inputs with a user's passphrase, account details, or output.
Pinned intermediate keys are confined to test builds; there is no production key
inspection API. Test diagnostics may contain these public fixture values.

`default-login-v1.json` is the original login contract. Its bytes and output are
preserved. `synthetic-login-v1.json` adds 15 independent reference cases without
changing the original file or the v1 algorithm identifier. `synthetic-core-v1.rs`
contains typed byte literals shared by private core tests. The Rust tests use
checked-in literals as their oracle; they never run the generator.

## Reproduce and verify

The reference implementation is `generate_synthetic.py`. It uses Python 3.9 or
newer, Python's standard library (`struct`, `hashlib`, `hmac`, `ctypes`, and JSON),
and the system `libargon2` C library's `argon2id_hash_raw` API, which implements
Argon2id version 19. It imports no project Rust implementation and needs no
Python packages, network, credentials, or random input. Its ASCII fold and
explicit Unicode White_Space set reproduce the documented v1 input rules;
Python's default `str.strip()` would incorrectly remove U+001C through U+001F.

Run these commands from the repository root:

```sh
python3 --version
python3 -c 'import ctypes.util; assert ctypes.util.find_library("argon2"), "system libargon2 is required"'
python3 test-vectors/generate_synthetic.py --check
cargo test --locked -p secret-sorbet-pw --test compatibility_vectors -- --test-threads=1
cargo test --locked -p secret-sorbet-core --lib -- --test-threads=1
```

To regenerate the two new fixture files intentionally:

```sh
python3 test-vectors/generate_synthetic.py --write
python3 test-vectors/generate_synthetic.py --check
git diff -- test-vectors/
```

Both generator modes first reproduce the unchanged original login output.
`--check` compares complete fixture contents without writing files. `--write`
only writes `synthetic-login-v1.json` and `synthetic-core-v1.rs`. It never writes
the original file. Normal `cargo test` needs no Python or `libargon2` installation.
Changing a fixture requires review of the actual byte contract, compatibility
impact, and algorithm identifier; a failing check is not a reason to refresh
expected outputs automatically. This independent reproduction is test evidence,
not an external security audit.

## JSON schema and input cases

The synthetic JSON object declares `schema_version`, `algorithm`, `profile`, the
complete current KDF profile, generation `provenance`, and `cases`. Each case has
a nonempty unique `id`, a `raw` object containing all four `LoginInput` fields,
literal `effective_variables`, a `policy`, and literal `output`. The passphrase
is passed verbatim. Service, username, and counter are passed raw to `LoginInput`;
the harness compares `variables()` with the separately pinned effective values.
It never reconstructs the raw login from those effective values.

Unknown object fields and missing declared fields fail schema tests, including
the explicit `allowed` field: `null` selects class union mode. Class order and
duplicate characters are preserved in policy metadata. The suite rejects empty
case lists and empty or duplicate IDs. The full derivation loop allows at most
16 cases and output lengths from 1 to 512, and executes serially within one test.
Use `--test-threads=1` to serialize the other expensive legacy/core unit tests as
well; each derivation uses a 64 MiB Argon2 memory region.

Coverage includes NFC/NFD passphrase and username divergence, emoji and UTF-8
multibyte text, verbatim passphrase whitespace, all Unicode White_Space edges,
ASCII service case equivalence, and retained non-ASCII service case. Near
whitespace U+001C, U+180E, U+200B, and U+FEFF is retained. Other cases pin counter
rotation, class order, duplicate class names and characters, minimum counts equal
to length, minimum length one, explicit alphabet versus union mode, 300 output
characters without requirements, and 257 characters with a required position.
The long cases consume multiple HMAC blocks.

Invalid/empty raw login fields, whitespace-only counters, invalid core fields,
non-ASCII and empty output charsets, missing required alphabet members, count
overflow, and invalid lengths are tested before expensive derivation, at their
respective validation boundaries. An empty core variable list is accepted and
has a pinned output. Password property tests have a fixed RNG seed and perform
no Argon2 work.

## Byte protocol pinned by private core tests

All integers below are unsigned, exactly eight bytes, and big endian, regardless
of the host's pointer width or byte order. A field is the concatenation of its
label's UTF-8 byte length, label bytes, value's UTF-8 byte length, and value bytes.
Lengths count bytes, not Unicode scalar values or displayed characters.

The canonical context contains, in order:

1. Field `version` with value `secret-sorbet-core-v1`, output length, and variable
   count.
2. For each variable in supplied order: fields `var-name` and `var-value`.
3. Class count; for each class in supplied order: fields `class-name` and
   `class-chars`, then its minimum count.
4. Field `allowed-class-union` with empty value, or `allowed-explicit` with its
   exact raw alphabet. Duplicate characters still affect context bytes even
   though the encoder deduplicates them for character selection.

The salt digest is SHA-256 of `secret-sorbet-core-v1:argon2id:salt`, context byte
length, and context bytes. Its first 16 bytes are the salt. Argon2id v19 uses
65,536 KiB memory, three iterations, four lanes, 32 output bytes, and the exact
passphrase UTF-8 bytes. HMAC-SHA-512 uses that key and hashes context bytes followed
by the eight-byte block counter, starting at zero. The private fixtures pin the
complete union and explicit contexts, salt digest, truncated salt, derived key,
blocks zero and one, and block 256 to make counter byte order unambiguous. Small
unit tests pin integer and length-prefix boundaries through `u64::MAX` without
large allocations. Separate pinned outputs cover reordered and empty variables.

For alphabet/position ranges up to 256, selection reads one byte, rejects values
at or above `256 - (256 % range)`, then takes the remainder. Larger free-position
ranges read eight bytes big endian, reject values below `2^64 % range`, and take
the remainder. No required position previously completed for ranges above 256;
the new large-range rule therefore extends v1 without changing existing output.
Required classes select a free position then a deduplicated class character;
remaining positions are filled in ascending order from the deduplicated allowed
alphabet. Encoder unit tests additionally pin finite-source byte consumption.

Progress and the acceptance record live in [P02 in the roadmap](../docs/ROADMAP.md#p02).
