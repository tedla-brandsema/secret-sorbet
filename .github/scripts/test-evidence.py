#!/usr/bin/env python3
"""Execute native suites and retain public build metadata, never fixture values."""

import argparse
import hashlib
import json
import os
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
    args = parser.parse_args()
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
    if host != args.target or release != baseline:
        raise SystemExit("compiler host or release differs from the declared native baseline")
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
        subprocess.run(["rustc", "--edition=2024", "--target", args.target,
                        str(source), "-o", str(binary)], check=True)
        runtime = dict(line.split("=", 1) for line in output(str(binary)).splitlines())
    for key in ("arch", "os", "pointer_width", "endian"):
        if f'target_{key}="{runtime[key]}"' not in cfg:
            raise SystemExit("executed runtime identity differs from the compiler target")
    server = os.environ.get("GITHUB_SERVER_URL", "https://github.com")
    repository = os.environ.get("GITHUB_REPOSITORY", "")
    run_id = os.environ.get("GITHUB_RUN_ID", "")
    evidence = {
        "schema_version": 1,
        "execution": "native",
        "tested_head": output("git", "rev-parse", "HEAD"),
        "event_head": os.environ.get("GITHUB_SHA"),
        "runner_label": args.runner,
        "runner_os": os.environ.get("RUNNER_OS"),
        "runner_arch": os.environ.get("RUNNER_ARCH"),
        "runner_image": {key: os.environ.get(key) for key in ("ImageOS", "ImageVersion")},
        "target": args.target,
        "rustc_vv": rustc,
        "target_cfg": cfg,
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
        command = ["cargo", "test", "--locked", "--workspace", "--target", args.target]
        if profile == "release":
            command.append("--release")
        # One harness at a time bounds concurrent 64 MiB KDF allocations, including CLI tests.
        command.extend(["--", "--test-threads=1"])
        print(f"Executing {profile} workspace tests for {args.target}", flush=True)
        result = subprocess.run(command, check=False)
        evidence["tests"].append({"profile": profile, "command": command,
                                  "exit_code": result.returncode})
        save()
        failed |= result.returncode != 0
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
