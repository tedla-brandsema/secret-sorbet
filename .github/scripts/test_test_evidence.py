"""Regression coverage for native execution evidence without running expensive KDFs."""

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

            def run(command, check):
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
