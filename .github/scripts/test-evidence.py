#!/usr/bin/env python3
"""Execute native or emulated suites and retain public build metadata, never fixture values."""

import argparse
import hashlib
import json
import os
import shlex
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib


def output(*command):
    return subprocess.check_output(command, text=True).strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True)
    parser.add_argument("--runner", required=True)
    parser.add_argument("--output", default="native-evidence.json")
    parser.add_argument("--emulator")
    parser.add_argument("--sysroot")
    parser.add_argument("--linker")
    parser.add_argument("--system-package", action="append", default=[])
    args = parser.parse_args()
    emulated = args.emulator is not None
    if any((args.emulator, args.sysroot, args.linker)) and not all(
            (args.emulator, args.sysroot, args.linker)):
        parser.error("emulated execution requires emulator, sysroot and linker together")
    if args.system_package and not emulated:
        parser.error("system package evidence requires emulated execution")
    # HEAD identifies the tested source only when tracked and untracked inputs are clean.
    # Capture status silently: filenames can contain private user or provider information.
    if output("git", "status", "--porcelain", "--untracked-files=normal"):
        raise SystemExit("execution evidence requires a clean checkout; commit or remove local changes")
    rustc = output("rustc", "-Vv")
    host = next(line.removeprefix("host: ") for line in rustc.splitlines()
                if line.startswith("host: "))
    baseline = tomllib.loads(Path("rust-toolchain.toml").read_text())["toolchain"]["channel"]
    release = next(line.removeprefix("release: ") for line in rustc.splitlines()
                   if line.startswith("release: "))
    # A target installation or a runner label alone is not native execution evidence.
    if release != baseline or (not emulated and host != args.target):
        raise SystemExit("compiler host or release differs from the declared baseline")
    runner_command = [args.emulator, "-L", args.sysroot] if emulated else []
    environment = os.environ.copy()
    setup = None
    if emulated:
        if not Path(args.sysroot).is_dir():
            raise SystemExit("emulator sysroot is unavailable; install the target C runtime")
        target_key = args.target.upper().replace("-", "_")
        environment[f"CARGO_TARGET_{target_key}_LINKER"] = args.linker
        environment[f"CARGO_TARGET_{target_key}_RUNNER"] = shlex.join(runner_command)
        setup = {
            "emulator_command": runner_command,
            "emulator_version": output(args.emulator, "--version"),
            "linker": args.linker,
            "linker_version": output(args.linker, "--version"),
            "sysroot": args.sysroot,
            "sysroot_runtime_sha256": {
                path.relative_to(args.sysroot).as_posix(): digest(path)
                for path in sorted(set(Path(args.sysroot).glob("lib/ld*.so*"))
                                   | set(Path(args.sysroot).glob("lib/libc.so.6")))
                if path.is_file()
            },
            "system_packages": output("dpkg-query", "-W", "-f=${binary:Package}=${Version}\n",
                                      *args.system_package).splitlines() if args.system_package else [],
            "cargo_environment": {key: value for key, value in environment.items()
                                  if key.startswith(f"CARGO_TARGET_{target_key}_")
                                  and key.endswith(("_LINKER", "_RUNNER"))},
        }
    cfg = output("rustc", "--print", "cfg", "--target", args.target).splitlines()
    with tempfile.TemporaryDirectory() as directory:
        source = Path(directory) / "identity.rs"
        binary = Path(directory) / ("identity.exe" if os.name == "nt" else "identity")
        source.write_text('''fn main() {
    println!("arch={}", std::env::consts::ARCH);
    println!("os={}", std::env::consts::OS);
    println!("pointer_width={}", usize::BITS);
    println!("endian={}", if cfg!(target_endian = "big") { "big" } else { "little" });
}
''', encoding="utf-8")
        probe_command = ["rustc", "--edition=2024", "--target", args.target,
                         str(source), "-o", str(binary)]
        if emulated:
            probe_command.extend(["-C", f"linker={args.linker}"])
        subprocess.run(probe_command, check=True)
        runtime = dict(line.split("=", 1)
                       for line in output(*runner_command, str(binary)).splitlines())
    for key in ("arch", "os", "pointer_width", "endian"):
        if f'target_{key}="{runtime[key]}"' not in cfg:
            raise SystemExit("executed runtime identity differs from the compiler target")
    server = os.environ.get("GITHUB_SERVER_URL", "https://github.com")
    repository = os.environ.get("GITHUB_REPOSITORY", "")
    run_id = os.environ.get("GITHUB_RUN_ID", "")
    evidence = {
        "schema_version": 1,
        "execution": "emulated" if emulated else "native",
        "emulation_setup": setup,
        "tested_head": output("git", "rev-parse", "HEAD"),
        "event_head": os.environ.get("GITHUB_SHA"),
        "runner_label": args.runner,
        "runner_os": os.environ.get("RUNNER_OS"),
        "runner_arch": os.environ.get("RUNNER_ARCH"),
        "runner_image": {key: os.environ.get(key) for key in ("ImageOS", "ImageVersion")},
        "target": args.target,
        "rustc_vv": rustc,
        "target_cfg": cfg,
        "target_abi": {key: next((line.split("=", 1)[1].strip('"') for line in cfg
                                   if line.startswith(key + "=")), None)
                       for key in ("target_env", "target_abi")},
        "runtime": runtime,
        "cargo_lock_sha256": digest(Path("Cargo.lock")),
        "fixtures_sha256": {path.as_posix(): digest(path)
                            for path in sorted(Path("test-vectors").rglob("*")) if path.is_file()},
        "run_url": f"{server}/{repository}/actions/runs/{run_id}" if run_id else None,
        "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        "job_key": os.environ.get("GITHUB_JOB"),
        "tests": [],
    }
    evidence_path = Path(args.output)

    def save():
        evidence_path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n",
                                 encoding="utf-8")

    save()
    failed = False
    for profile in ("debug", "release"):
        command = ["cargo", "test", "--locked"]
        if emulated:
            # CLI tests spawn target child binaries, requiring separate binfmt configuration.
            # The shared core/pw suites execute all full and intermediate fixtures directly.
            command.extend(["-p", "secret-sorbet-core", "-p", "secret-sorbet-pw"])
        else:
            command.append("--workspace")
        command.extend(["--target", args.target])
        if profile == "release":
            command.append("--release")
        # One harness at a time bounds concurrent 64 MiB KDF allocations, including CLI tests.
        command.extend(["--", "--test-threads=1"])
        print(f"Executing {profile} tests for {args.target}", flush=True)
        result = subprocess.run(command, check=False, env=environment)
        evidence["tests"].append({"profile": profile, "command": command,
                                  "exit_code": result.returncode})
        save()
        failed |= result.returncode != 0
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
