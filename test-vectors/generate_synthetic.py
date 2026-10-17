#!/usr/bin/env python3
"""Independent synthetic v1 oracle; never accepts user secrets or repository code."""

import argparse
import ctypes
import ctypes.util
import hashlib
import hmac
import json
from pathlib import Path
import struct
import unicodedata


ROOT = Path(__file__).resolve().parent
ALGORITHM = "secret-sorbet-core-v1"
KDF = {
    "algorithm": "argon2id",
    "version": 19,
    "memory_kib": 65536,
    "iterations": 3,
    "lanes": 4,
    "output_length": 32,
    "salt_derivation": "sha256-domain-context-v1",
    "salt_domain": "secret-sorbet-core-v1:argon2id:salt",
    "salt_length": 16,
}
# The Unicode White_Space set used by Rust str::trim. Do not use Python strip's
# larger default set (it includes U+001C..U+001F). Near whitespace is retained.
WHITESPACE = "\t\n\v\f\r \u0085\u00a0\u1680" + "".join(
    chr(n) for n in range(0x2000, 0x200B)
) + "\u2028\u2029\u202f\u205f\u3000"


def u64(value):
    return struct.pack(">Q", value)


def field(label, value):
    label, value = label.encode("utf-8"), value.encode("utf-8")
    return u64(len(label)) + label + u64(len(value)) + value


def context(variables, policy):
    out = field("version", ALGORITHM) + u64(policy["length"]) + u64(len(variables))
    for name, value in variables:
        out += field("var-name", name) + field("var-value", value)
    out += u64(len(policy["classes"]))
    for item in policy["classes"]:
        out += field("class-name", item["name"]) + field("class-chars", item["chars"])
        out += u64(item["min_count"])
    if policy["allowed"] is None:
        return out + field("allowed-class-union", "")
    return out + field("allowed-explicit", policy["allowed"])


def salt_digest(encoded):
    return hashlib.sha256(KDF["salt_domain"].encode() + u64(len(encoded)) + encoded).digest()


def derive_key(passphrase, encoded):
    library = ctypes.util.find_library("argon2")
    if library is None:
        raise SystemExit("libargon2 is required; no files were written")
    argon = ctypes.CDLL(library)
    hash_raw = argon.argon2id_hash_raw
    hash_raw.argtypes = [ctypes.c_uint32, ctypes.c_uint32, ctypes.c_uint32,
                        ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p,
                        ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
    hash_raw.restype = ctypes.c_int
    password = passphrase.encode("utf-8")
    salt = salt_digest(encoded)[:KDF["salt_length"]]
    output = ctypes.create_string_buffer(KDF["output_length"])
    result = hash_raw(KDF["iterations"], KDF["memory_kib"], KDF["lanes"],
                      password, len(password), salt, len(salt), output, len(output))
    if result != 0:
        raise SystemExit("synthetic Argon2id reference derivation failed")
    return output.raw


def block(key, encoded, counter):
    return hmac.digest(key, encoded + u64(counter), "sha512")


def encode(key, encoded, policy):
    def source():
        counter = 0
        while True:
            yield from block(key, encoded, counter)
            counter += 1

    stream = source()

    def choose(length):
        if length <= 256:
            limit = 256 - 256 % length
            while True:
                value = next(stream)
                if value < limit:
                    return value % length
        threshold = (1 << 64) % length
        while True:
            value = int.from_bytes(bytes(next(stream) for _ in range(8)), "big")
            if value >= threshold:
                return value % length

    def unique(chars):
        return list(dict.fromkeys(chars))

    allowed = unique(policy["allowed"] if policy["allowed"] is not None else
                     "".join(item["chars"] for item in policy["classes"]))
    out = [None] * policy["length"]
    for item in policy["classes"]:
        chars = unique(item["chars"])
        for _ in range(item["min_count"]):
            free = [i for i, value in enumerate(out) if value is None]
            position = free[choose(len(free))]
            out[position] = chars[choose(len(chars))]
    for i, value in enumerate(out):
        if value is None:
            out[i] = allowed[choose(len(allowed))]
    return "".join(out)


def policy(length=20, classes=None, allowed=None):
    if classes is None:
        classes = [("lowercase", "abcdefghijklmnopqrstuvwxyz", 1),
                   ("uppercase", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", 1),
                   ("digit", "0123456789", 1),
                   ("special", "!@#$%^&*()-_+=<>?", 1)]
    return {"length": length,
            "classes": [dict(zip(("name", "chars", "min_count"), row)) for row in classes],
            "allowed": allowed}


def effective(raw):
    # ASCII-only folding; Unicode lower/casefold would change v1.
    service = raw["service"].strip(WHITESPACE).translate(
        str.maketrans("ABCDEFGHIJKLMNOPQRSTUVWXYZ", "abcdefghijklmnopqrstuvwxyz"))
    return [["purpose", "login"], ["service", service],
            ["username", raw["username"].strip(WHITESPACE)],
            ["counter", raw["counter"].strip(WHITESPACE)]]


def login_fixtures():
    base = dict(passphrase="Synthetic café 🔐", service="EXAMPLE.TEST", username="zoë🧪", counter="1")
    classes = [("letters", "ab", 3), ("digits", "12", 3)]
    definitions = [
        ("unicode-nfc", {}, policy()),
        ("ascii-case-and-unicode-edge-whitespace",
         dict(service=WHITESPACE + "example.test" + WHITESPACE,
              username=WHITESPACE + base["username"] + WHITESPACE,
              counter=WHITESPACE + "1" + WHITESPACE), policy()),
        ("passphrase-nfd", dict(passphrase=unicodedata.normalize("NFD", base["passphrase"])), policy()),
        ("username-nfd", dict(username=unicodedata.normalize("NFD", base["username"])), policy()),
        ("near-whitespace-and-nonascii-service-uppercase",
         dict(service="\u001c\u200bÄ.EXAMPLE\ufeff", username="\u180ezoë🧪\u200b\ufeff"), policy()),
        ("near-whitespace-and-nonascii-service-lowercase",
         dict(service="\u001c\u200bä.EXAMPLE\ufeff", username="\u180ezoë🧪\u200b\ufeff"), policy()),
        ("counter-rotation", dict(counter="2"), policy()),
        ("passphrase-whitespace-is-significant",
         dict(passphrase="\t " + base["passphrase"] + "\u00a0"), policy()),
        ("minimum-count-equals-length", {}, policy(6, classes)),
        ("class-order", {}, policy(6, list(reversed(classes)))),
        ("duplicate-characters-and-class-names", {},
         policy(6, [("same", "aabb", 3), ("same", "1122", 3)])),
        ("explicit-alphabet", {}, policy(6, classes, "ab12")),
        ("minimum-length", {}, policy(1, [("one", "x", 1)])),
        ("multiple-hmac-blocks-no-required-long", {}, policy(300, [], "ab0123")),
        ("required-position-above-byte-range", {}, policy(257, [("one", "X", 1)], "abX")),
    ]
    cases = []
    for case_id, changes, output_policy in definitions:
        raw = base | changes
        variables = effective(raw)
        encoded = context(variables, output_policy)
        cases.append(dict(id=case_id, raw=raw, effective_variables=variables,
                          policy=output_policy,
                          output=encode(derive_key(raw["passphrase"], encoded), encoded, output_policy)))
    return dict(schema_version=1, algorithm=ALGORITHM, profile="secret-sorbet-login-v1", kdf=KDF,
                provenance=dict(generator="test-vectors/generate_synthetic.py",
                                reference="Python 3 stdlib + libargon2 argon2id_hash_raw (v19)",
                                inputs="public synthetic test data only"), cases=cases)


def rust_bytes(name, value):
    rows = ["    " + ", ".join(f"0x{byte:02x}" for byte in value[i:i + 16]) + ","
            for i in range(0, len(value), 16)]
    return f"pub(crate) const {name}: &[u8] = &[\n" + "\n".join(rows) + "\n];\n"


def core_fixture():
    variables = [["名", "é🧪"], ["counter", "1"]]
    output_policy = policy(10, [("文字", "aab12", 2)])
    encoded = context(variables, output_policy)
    key = derive_key("Synthetic core 🔐", encoded)
    data = "// Generated by test-vectors/generate_synthetic.py; PUBLIC SYNTHETIC DATA ONLY.\n"
    data += "// secret-sorbet-core-v1; Argon2id v19, m=65536 KiB, t=3, p=4, 32 bytes.\n"
    data += "// Independent Python stdlib + libargon2 reference; see README.md.\n"
    for name, value in [("CONTEXT", encoded),
                        ("EXPLICIT_CONTEXT", context(variables, output_policy | {"allowed": "ab12"})),
                        ("SALT_DIGEST", salt_digest(encoded)),
                        ("SALT", salt_digest(encoded)[:16]), ("KEY", key),
                        ("HMAC_0", block(key, encoded, 0)),
                        ("HMAC_1", block(key, encoded, 1)),
                        ("HMAC_256", block(key, encoded, 256))]:
        data += rust_bytes(name, value)
    for name, values in [("OUTPUT", variables), ("REORDERED_OUTPUT", list(reversed(variables))),
                         ("EMPTY_VARIABLES_OUTPUT", [])]:
        ctx = context(values, output_policy)
        output = encode(key if name == "OUTPUT" else derive_key("Synthetic core 🔐", ctx), ctx, output_policy)
        data += f"pub(crate) const {name}: &str = {json.dumps(output)};\n"
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--check", action="store_true", help="compare without modifying fixtures")
    action.add_argument("--write", action="store_true", help="regenerate only new synthetic fixtures")
    args = parser.parse_args()
    # Check the independent implementation against the untouched original contract.
    legacy = json.loads((ROOT / "default-login-v1.json").read_text())[0]
    encoded = context(legacy["variables"], legacy["policy"])
    if encode(derive_key(legacy["passphrase"], encoded), encoded, legacy["policy"]) != legacy["output"]:
        raise SystemExit("independent oracle disagrees with the original v1 contract")
    files = {"synthetic-login-v1.json": json.dumps(login_fixtures(), ensure_ascii=False, indent=2) + "\n",
             "synthetic-core-v1.rs": core_fixture()}
    for name, expected in files.items():
        path = ROOT / name
        if args.write:
            path.write_text(expected, encoding="utf-8")
        elif not path.exists() or path.read_text(encoding="utf-8") != expected:
            raise SystemExit(f"synthetic fixture differs: {name}")
    print("Synthetic reference fixtures verified" if args.check else "Synthetic reference fixtures written")


if __name__ == "__main__":
    main()
