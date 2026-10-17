"""Regression coverage for native and emulated evidence without expensive KDFs."""

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "test_evidence", Path(__file__).with_name("test-evidence.py")
)
EVIDENCE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EVIDENCE)
TARGET = "x86_64-unknown-linux-gnu"


class EvidenceTests(unittest.TestCase):
    def command_output(self, *command):
        if command == ("git", "status", "--porcelain", "--untracked-files=normal"):
            return ""
        if command == ("rustc", "-Vv"):
            return f"rustc 1.98.0\nhost: {TARGET}\nrelease: 1.98.0"
        if command[:3] == ("rustc", "--print", "cfg"):
            return '\n'.join([
                'target_arch="x86_64"', 'target_os="linux"',
                'target_pointer_width="64"', 'target_endian="little"',
            ])
        if command[0] == "git":
            return "a" * 40
        return "arch=x86_64\nos=linux\npointer_width=64\nendian=little"

    def execute(self, exit_codes):
        commands = []
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "metadata.json"

            def run(command, check, env=None):
                if command[0] == "cargo":
                    commands.append(command)
                    return SimpleNamespace(returncode=exit_codes.pop(0))
                self.assertTrue(check)
                return SimpleNamespace(returncode=0)

            arguments = ["test", "--target", TARGET, "--runner", "unit-test",
                         "--output", str(destination)]
            with patch.object(sys, "argv", arguments), \
                    patch.object(EVIDENCE, "output", side_effect=self.command_output), \
                    patch.object(EVIDENCE.subprocess, "run", side_effect=run), \
                    contextlib.redirect_stdout(io.StringIO()):
                code = EVIDENCE.main()
                metadata = json.loads(destination.read_text())
        return code, metadata, commands

    def test_success_records_exact_commands_and_fixture_identity(self):
        code, metadata, commands = self.execute([0, 0])
        self.assertEqual(code, 0)
        self.assertEqual([item["profile"] for item in metadata["tests"]],
                         ["debug", "release"])
        self.assertEqual(commands, [
            ["cargo", "test", "--locked", "--workspace", "--target", TARGET,
             "--", "--test-threads=1"],
            ["cargo", "test", "--locked", "--workspace", "--target", TARGET,
             "--release", "--", "--test-threads=1"],
        ])
        self.assertEqual(metadata["tested_head"], "a" * 40)
        self.assertEqual(metadata["runtime"]["pointer_width"], "64")
        self.assertEqual(metadata["cargo_lock_sha256"], EVIDENCE.digest(Path("Cargo.lock")))
        self.assertTrue(metadata["fixtures_sha256"])
        self.assertTrue(all(len(value) == 64 for value in metadata["fixtures_sha256"].values()))
        # Metadata must contain identities and outcomes, never fixture content.
        self.assertNotIn("passphrase", json.dumps(metadata))
        self.assertNotIn("derived_key", json.dumps(metadata))

    def test_dirty_checkout_is_rejected_before_probe_without_path_disclosure(self):
        for status in ("M  sensitive-staged-file", " M sensitive-tracked-file",
                       "?? sensitive-untracked-file"):
            with self.subTest(status_kind=status[:2]):
                arguments = ["test", "--target", TARGET, "--runner", "unit-test"]
                with patch.object(sys, "argv", arguments), \
                        patch.object(EVIDENCE, "output", return_value=status) as output, \
                        patch.object(EVIDENCE.subprocess, "run") as run:
                    with self.assertRaises(SystemExit) as error:
                        EVIDENCE.main()
                    self.assertIn("clean checkout", str(error.exception))
                    self.assertNotIn("sensitive", str(error.exception))
                    output.assert_called_once_with(
                        "git", "status", "--porcelain", "--untracked-files=normal"
                    )
                    run.assert_not_called()

    def test_failure_is_retained_and_propagated(self):
        code, metadata, _ = self.execute([1, 0])
        self.assertEqual(code, 1)
        self.assertEqual([item["exit_code"] for item in metadata["tests"]], [1, 0])

    def execute_emulated(self, exit_codes=(0, 0), failure=None):
        commands = []
        environments = []
        output_calls = []
        exit_codes = list(exit_codes)
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "metadata.json"
            library_directory = Path(directory) / "lib"
            library_directory.mkdir()
            (library_directory / "libc.so.6").write_bytes(b"synthetic C runtime")
            arguments = ["test", "--target", "s390x-unknown-linux-gnu", "--runner", "unit-test",
                         "--emulator", "qemu-s390x", "--linker", "s390x-linux-gnu-gcc",
                         "--sysroot", directory, "--system-package", "qemu-user",
                         "--output", str(destination)]

            def command_output(*command):
                output_calls.append(command)
                if command[0] == "qemu-s390x":
                    if command[1] == "--version":
                        return "qemu-s390x version synthetic-test"
                    if failure == "emulator":
                        raise EVIDENCE.subprocess.CalledProcessError(1, command)
                    width = "32" if failure == "identity" else "64"
                    return f"arch=s390x\nos=linux\npointer_width={width}\nendian=big"
                if command[0] == "s390x-linux-gnu-gcc":
                    return "s390x-linux-gnu-gcc synthetic-test"
                if command[0] == "dpkg-query":
                    return "qemu-user=synthetic-test"
                value = self.command_output(*command)
                if command[:3] == ("rustc", "--print", "cfg"):
                    value = value.replace('target_arch="x86_64"', 'target_arch="s390x"')
                    value = value.replace('target_endian="little"', 'target_endian="big"')
                    value += '\ntarget_env="gnu"\ntarget_abi=""'
                return value

            def run(command, check, env=None):
                if command[0] == "cargo":
                    commands.append(command)
                    environments.append(env)
                    return SimpleNamespace(returncode=exit_codes.pop(0))
                self.assertIn("linker=s390x-linux-gnu-gcc", command)
                self.assertTrue(check)
                return SimpleNamespace(returncode=0)

            with patch.object(sys, "argv", arguments), \
                    patch.object(EVIDENCE, "output", side_effect=command_output), \
                    patch.object(EVIDENCE.subprocess, "run", side_effect=run), \
                    contextlib.redirect_stdout(io.StringIO()):
                if failure:
                    expected = SystemExit if failure == "identity" else EVIDENCE.subprocess.CalledProcessError
                    with self.assertRaises(expected):
                        EVIDENCE.main()
                    self.assertEqual(commands, [])
                    self.assertFalse(destination.exists())
                    return
                code = EVIDENCE.main()
                metadata = json.loads(destination.read_text())
        return code, metadata, commands, environments, output_calls

    def test_emulation_records_runtime_setup_and_runs_both_shared_suites(self):
        code, metadata, commands, environments, output_calls = self.execute_emulated()
        self.assertEqual(code, 0)
        self.assertEqual(metadata["execution"], "emulated")
        self.assertEqual(metadata["runtime"]["endian"], "big")
        self.assertEqual(metadata["emulation_setup"]["system_packages"],
                         ["qemu-user=synthetic-test"])
        self.assertEqual(metadata["emulation_setup"]["linker"], "s390x-linux-gnu-gcc")
        self.assertEqual(metadata["target_abi"], {"target_env": "gnu", "target_abi": ""})
        self.assertEqual(metadata["emulation_setup"]["sysroot_runtime_sha256"], {
            "lib/libc.so.6": EVIDENCE.hashlib.sha256(b"synthetic C runtime").hexdigest()
        })
        self.assertEqual(["--release" in command for command in commands], [False, True])
        for command, environment in zip(commands, environments):
            self.assertEqual(command[:7], ["cargo", "test", "--locked", "-p",
                                           "secret-sorbet-core", "-p", "secret-sorbet-pw"])
            self.assertEqual(command[-2:], ["--", "--test-threads=1"])
            self.assertEqual(command[command.index("--target") + 1],
                             "s390x-unknown-linux-gnu")
            self.assertNotIn("--workspace", command)
            self.assertEqual(environment["CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER"],
                             "s390x-linux-gnu-gcc")
            self.assertTrue(environment["CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_RUNNER"].startswith(
                "qemu-s390x -L "))
        self.assertTrue(any(command[:2] == ("qemu-s390x", "-L") for command in output_calls))

    def test_emulated_cargo_failure_is_retained_and_propagated(self):
        code, metadata, _, _, _ = self.execute_emulated(exit_codes=(0, 1))
        self.assertEqual(code, 1)
        self.assertEqual([item["exit_code"] for item in metadata["tests"]], [0, 1])

    def test_failed_emulator_never_runs_tests_or_claims_evidence(self):
        self.execute_emulated(failure="emulator")

    def test_emulated_runtime_mismatch_never_runs_tests_or_claims_evidence(self):
        self.execute_emulated(failure="identity")

    def test_missing_sysroot_is_rejected_before_emulator_or_tests(self):
        arguments = ["test", "--target", "s390x-unknown-linux-gnu", "--runner", "unit-test",
                     "--emulator", "qemu-s390x", "--linker", "s390x-linux-gnu-gcc",
                     "--sysroot", "/missing-target-sysroot"]
        with patch.object(sys, "argv", arguments), \
                patch.object(EVIDENCE, "output", side_effect=self.command_output), \
                patch.object(EVIDENCE.Path, "is_dir", return_value=False), \
                patch.object(EVIDENCE.subprocess, "run") as run:
            with self.assertRaises(SystemExit) as error:
                EVIDENCE.main()
            self.assertIn("sysroot is unavailable", str(error.exception))
            run.assert_not_called()

    def test_incomplete_emulator_setup_is_rejected(self):
        with patch.object(sys, "argv", ["test", "--target", TARGET, "--runner", "unit-test",
                                        "--emulator", "qemu-i386"]), \
                patch.object(EVIDENCE, "output") as output, \
                contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit):
                EVIDENCE.main()
            output.assert_not_called()

    def assert_identity_rejected(self, target=TARGET, transform=lambda value: value):
        def changed_output(*command):
            return transform(self.command_output(*command))

        with patch.object(sys, "argv", ["test", "--target", target, "--runner", "unit-test"]), \
                patch.object(EVIDENCE, "output", side_effect=changed_output), \
                patch.object(EVIDENCE.subprocess, "run") as run:
            with self.assertRaises(SystemExit):
                EVIDENCE.main()
            self.assertFalse(any(call.args[0][0] == "cargo" for call in run.call_args_list))

    def test_wrong_native_target_is_rejected(self):
        self.assert_identity_rejected(target="aarch64-unknown-linux-gnu")

    def test_wrong_release_is_rejected(self):
        self.assert_identity_rejected(
            transform=lambda value: value.replace("release: 1.98.0", "release: 1.97.0")
        )

    def test_runtime_pointer_width_mismatch_is_rejected(self):
        self.assert_identity_rejected(
            transform=lambda value: value.replace("pointer_width=64", "pointer_width=32")
        )

    def test_runtime_endianness_mismatch_is_rejected(self):
        self.assert_identity_rejected(
            transform=lambda value: value.replace("endian=little", "endian=big")
        )


if __name__ == "__main__":
    unittest.main()
