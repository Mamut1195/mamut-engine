#!/usr/bin/env python3
"""Python 3.11+, stdlib only; disposable verifier and real Cargo tamper tests.

Only named metadata, the checker and vendor-crates are copied. Source files are
handled as bytes, never displayed or imported. No actual vendor tree is changed.
Cargo lock bootstrap uses --offline; every build uses --frozen and an independent
empty CARGO_HOME and fresh CARGO_TARGET_DIR. Logs go to --log-dir or stdout.
"""
import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib

COPIES = ("Cargo.lock", "crates/engine-cli/Cargo.toml", ".cargo/config.toml",
          "provenance/MAM-938-vendor-inventory.json", "scripts/check-vendor-crates.py")


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def run(label, command, cwd, logs, env=None, reject=False, marker=b""):
    result = subprocess.run(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, check=False)
    if logs is None:
        sys.stdout.flush()
        sys.stdout.buffer.write(result.stdout)
        sys.stdout.buffer.flush()
    else:
        (logs / f"{label}.log").write_bytes(result.stdout)
    if reject:
        require(result.returncode != 0 and marker in result.stdout.lower(),
                f"{label}: expected checksum rejection was NOT established (exit {result.returncode})")
        print(f"PASS {label}: expected checksum failure observed", flush=True)
    else:
        require(result.returncode == 0, f"{label}: positive control failed (exit {result.returncode})")
        print(f"PASS {label}", flush=True)


@contextmanager
def mutation(path, replacement):
    original = path.read_bytes()
    try:
        path.write_bytes(replacement)
        yield
    finally:
        path.write_bytes(original)


def flipped(value):
    return ("0" if value[0] != "0" else "1") + value[1:]


def mutations(crate):
    checksum_path = crate / ".cargo-checksum.json"
    data = json.loads(checksum_path.read_bytes())
    source = "src/lib.rs"
    require(source in data["files"], "libm src/lib.rs must be a checksum-listed compilation input")
    yield "source", crate / source, (crate / source).read_bytes() + b"\n"
    file_data = json.loads(checksum_path.read_bytes())
    file_data["files"][source] = flipped(file_data["files"][source])
    yield "file_hash", checksum_path, (json.dumps(file_data, sort_keys=True) + "\n").encode("utf-8")
    package_data = json.loads(checksum_path.read_bytes())
    package_data["package"] = flipped(package_data["package"])
    yield "package_hash", checksum_path, (json.dumps(package_data, sort_keys=True) + "\n").encode("utf-8")


def cargo_environment(state, label):
    directory = state / label
    directory.mkdir()
    home = directory / "cargo-home"
    home.mkdir()
    target = directory / "target"
    require(not any(home.iterdir()) and not target.exists(), "Cargo state must start empty")
    allowed = ("HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "PATH", "TMPDIR", "TMP", "TEMP",
               "SYSTEMROOT", "SystemRoot", "WINDIR")
    if os.name == "nt":
        allowed += ("SYSTEMDRIVE", "USERPROFILE", "ProgramFiles", "ProgramFiles(x86)", "ProgramW6432",
                    "VSINSTALLDIR", "VCINSTALLDIR", "VCToolsInstallDir", "VCToolsVersion",
                    "WindowsSdkDir", "WindowsSDKVersion", "WindowsSdkBinPath",
                    "WindowsSdkVerBinPath", "UniversalCRTSdkDir", "UCRTVersion",
                    "LIB", "INCLUDE", "LIBPATH")
    env = {key: os.environ[key] for key in allowed if key in os.environ}
    env.update(CARGO_HOME=str(home), CARGO_TARGET_DIR=str(target),
               CARGO_NET_OFFLINE="true", CARGO_INCREMENTAL="0")
    return env


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).absolute().parents[1])
    parser.add_argument("--log-dir", type=Path)
    parser.add_argument("--cargo", default="cargo")
    args = parser.parse_args()
    root = args.root.absolute()
    logs = args.log_dir.absolute() if args.log_dir else None
    if logs is not None:
        require(not logs.resolve().is_relative_to(root.resolve()), "logs must be outside the checkout")
        logs.mkdir(parents=True, exist_ok=False)
    checker = root / "scripts/check-vendor-crates.py"
    run("original_positive", [sys.executable, "-I", str(checker), "--root", str(root)], root, logs)
    with tempfile.TemporaryDirectory(prefix="mam938-vendor-") as temporary:
        work = Path(temporary)
        require(not work.resolve().is_relative_to(root.resolve()), "temporary copies must be outside checkout")
        fixture = work / "fixture"
        fixture.mkdir()
        for name in COPIES:
            destination = fixture / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(root / name, destination)
        shutil.copytree(root / "vendor-crates", fixture / "vendor-crates", symlinks=True)
        verify = [sys.executable, "-I", str(fixture / "scripts/check-vendor-crates.py"),
                  "--root", str(fixture)]
        run("copy_positive", verify, fixture, logs)
        lock = tomllib.loads((fixture / "Cargo.lock").read_bytes().decode("utf-8"))
        locked = [package for package in lock["package"]
                  if package["name"] == "libm" and package.get("source")]
        require(len(locked) == 1, "Cargo probe requires one locked registry libm version")
        version = locked[0]["version"]
        candidates = []
        for directory in (fixture / "vendor-crates").iterdir():
            metadata = tomllib.loads((directory / "Cargo.toml").read_bytes().decode("utf-8"))["package"]
            if (metadata["name"], metadata["version"]) == ("libm", version):
                candidates.append(directory)
        require(len(candidates) == 1, "Cargo probe libm directory identity is ambiguous")
        crate = candidates[0]
        for label, path, replacement in mutations(crate):
            with mutation(path, replacement):
                run(f"verifier_{label}", verify, fixture, logs, reject=True, marker=b"mismatch")
        probe = work / "probe"
        (probe / "src").mkdir(parents=True)
        (probe / ".cargo").mkdir()
        shutil.copy2(fixture / ".cargo/config.toml", probe / ".cargo/config.toml")
        shutil.copytree(fixture / "vendor-crates", probe / "vendor-crates", symlinks=True)
        manifest = ('[package]\nname = "mam938-vendor-probe"\nversion = "0.0.0"\nedition = "2021"\n'
                    '[workspace]\n[dependencies]\nlibm = { version = "=' + version
                    + '", default-features = false }\n')
        (probe / "Cargo.toml").write_bytes(manifest.encode("utf-8"))
        (probe / "src/lib.rs").write_bytes(b"#![no_std]\npub fn probe(x: f64) -> f64 { libm::sqrt(x) }\n")
        state = work / "state"
        state.mkdir()
        run("cargo_lock_bootstrap", [args.cargo, "--offline", "generate-lockfile"], probe, logs,
            env=cargo_environment(state, "bootstrap"))
        command = [args.cargo, "--frozen", "check"]
        run("cargo_positive", command, probe, logs, env=cargo_environment(state, "positive"))
        for label, path, replacement in mutations(probe / "vendor-crates" / crate.name):
            with mutation(path, replacement):
                run(f"cargo_{label}", command, probe, logs,
                    env=cargo_environment(state, label), reject=True, marker=b"checksum")
    print("PASS disposable verifier and Cargo tamper controls", flush=True)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError, RuntimeError) as error:
        print(f"FAIL tamper tests: {error}; subsequent controls NOT RUN", file=sys.stderr)
        sys.exit(1)
