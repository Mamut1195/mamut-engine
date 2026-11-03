#!/usr/bin/env python3
"""Python 3.11+, stdlib only. Default: read-only directory integrity verification.

--write-inventory is an explicit supplier snapshot operation, not validation of
archive identity. A matching Cargo package checksum field is metadata; proving
registry archive identity additionally requires hashing cached .crate archives
against Cargo.lock in the supplier phase. Never import or display vendor source.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import sys
import tomllib

INVENTORY = "provenance/MAM-938-vendor-inventory.json"
SOURCES = {"registry+https://github.com/rust-lang/crates.io-index",
           "sparse+https://index.crates.io/"}
PIN = {"version": "=1.1.8+spec-1.1.0", "default-features": False,
       "features": ["std", "serde", "parse", "display"]}
CONFIG = {"source": {"crates-io": {"replace-with": "vendored-sources"},
                     "vendored-sources": {"directory": "vendor-crates"}}}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def node(path, directory=False):
    info = path.lstat()
    require(not stat.S_ISLNK(info.st_mode)
            and not (getattr(info, "st_file_attributes", 0) & 0x400),
            f"symlink/junction/reparse point rejected: {path}")
    valid = stat.S_ISDIR(info.st_mode) if directory else stat.S_ISREG(info.st_mode)
    require(valid, f"unexpected file type: {path}")


def digest(path):
    node(path)
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def toml_file(path):
    node(path)
    return tomllib.loads(path.read_bytes().decode("utf-8"))


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON entry: {key}")
        result[key] = value
    return result


def json_file(path):
    node(path)
    return json.loads(path.read_bytes(), object_pairs_hook=unique_pairs)


def checksum(value):
    require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value),
            "invalid SHA-256 metadata")
    return value


def relative(value):
    require(isinstance(value, str) and value and "\\" not in value
            and ":" not in value and "\x00" not in value,
            "invalid relative checksum/license path")
    require(not PurePosixPath(value).is_absolute()
            and all(part not in ("", ".", "..") for part in value.split("/")),
            "absolute/traversal/ambiguous checksum or license path rejected")
    return value


def scan(base):
    files, directories, pending = {}, set(), [base]
    while pending:
        current = pending.pop()
        node(current, True)
        directories.add(current.relative_to(base).as_posix())
        for path in sorted(current.iterdir(), key=lambda item: item.name):
            info = path.lstat()
            if stat.S_ISDIR(info.st_mode):
                node(path, True)
                pending.append(path)
            else:
                files[path.relative_to(base).as_posix()] = digest(path)
    return files, directories


def recompute(root):
    node(root, True)
    for name in (".cargo", "crates", "crates/engine-cli", "provenance",
                 "vendor-crates"):
        node(root / name, True)
    manifest = toml_file(root / "crates/engine-cli/Cargo.toml")
    require(manifest.get("dependencies", {}).get("toml") == PIN,
            "engine-cli toml pin/default-features/explicit features mismatch")
    require(toml_file(root / ".cargo/config.toml") == CONFIG,
            "vendored source configuration mismatch")
    lock = toml_file(root / "Cargo.lock")
    packages = lock.get("package")
    require(isinstance(packages, list), "missing locked packages")
    locked, identities = {}, set()
    for package in packages:
        name, version, source = package["name"], package["version"], package.get("source")
        require(isinstance(name, str) and isinstance(version, str), "invalid lock identity")
        identity = (name, version, source)
        require(identity not in identities, "duplicate locked package entry")
        identities.add(identity)
        if source is None:
            continue
        require(source in SOURCES, "non-crates.io registry/git locked source rejected")
        key = (name, version)
        require(key not in locked, "ambiguous locked registry identity")
        locked[key] = (source, checksum(package["checksum"]))
    require({version for name, version in locked if name == "toml"}
            == {"1.1.8+spec-1.1.0"}, "locked toml metadata version mismatch")
    seen, records, all_directories = set(), [], ["vendor-crates"]
    file_count = 0
    for crate in sorted((root / "vendor-crates").iterdir(), key=lambda item: item.name):
        node(crate, True)
        metadata = toml_file(crate / "Cargo.toml")["package"]
        key = (metadata["name"], metadata["version"])
        require(key in locked and key not in seen, "extra/duplicate vendored package")
        seen.add(key)
        source, locked_hash = locked[key]
        cargo_checksums = json_file(crate / ".cargo-checksum.json")
        require(isinstance(cargo_checksums, dict)
                and {"files", "package"} <= set(cargo_checksums)
                and set(cargo_checksums) <= {"$comment", "files", "package"}
                and ("$comment" not in cargo_checksums
                     or isinstance(cargo_checksums["$comment"], str)),
                "invalid checksum document")
        package_hash = checksum(cargo_checksums["package"])
        require(package_hash == locked_hash, "package checksum mismatch")
        declared = cargo_checksums["files"]
        require(isinstance(declared, dict), "invalid checksum file map")
        expected = {relative(name): checksum(value) for name, value in declared.items()}
        require(".cargo-checksum.json" not in expected, "self-referential checksum rejected")
        files, directories = scan(crate)
        required_files = set(expected) | {".cargo-checksum.json"}
        require(set(files) == required_files, f"missing/extra vendor files: {crate.name}")
        required_directories = {"."}
        for name in required_files:
            required_directories.update(parent.as_posix() for parent in PurePosixPath(name).parents)
        require(directories == required_directories,
                f"missing/extra vendor directories: {crate.name}")
        for name, expected_hash in expected.items():
            require(files[name] == expected_hash, f"Cargo file checksum mismatch: {crate.name}/{name}")
        license_value = metadata.get("license")
        require(license_value is None or isinstance(license_value, str), "invalid license metadata")
        require(not ("license-file" in metadata and "license_file" in metadata),
                "ambiguous license-file metadata")
        license_file = metadata.get("license-file", metadata.get("license_file"))
        license_key = None
        if license_file is not None:
            require(isinstance(license_file, str) and "\\" not in license_file,
                    "invalid declared license-file")
            license_key = relative(PurePosixPath(license_file).as_posix())
            require(license_key in files, "declared license-file missing")
        prefix = crate.relative_to(root).as_posix()
        license_files = {f"{prefix}/{name}": value for name, value in sorted(files.items())
                         if re.match(r"^(licen[cs]e|copying|notice)",
                                     PurePosixPath(name).name, re.IGNORECASE)
                         or name == license_key}
        records.append({"directory": prefix, "name": key[0], "version": key[1],
                        "source": source, "lock_checksum": locked_hash,
                        "package_checksum": package_hash, "license": license_value,
                        "license_file": license_file,
                        "license_file_sha256": files.get(license_key),
                        "license_files": license_files, "cargo_toml": f"{prefix}/Cargo.toml",
                        "files": {f"{prefix}/{name}": value for name, value in sorted(files.items())}})
        all_directories.extend(prefix if name == "." else f"{prefix}/{name}"
                               for name in sorted(directories))
        file_count += len(files)
    require(seen == set(locked), "missing vendored registry packages")
    return {"schema": "mamut.vendor-inventory/1", "cargo_lock": {
        "path": "Cargo.lock", "sha256": digest(root / "Cargo.lock"),
        "package_count": len(packages), "registry_package_count": len(locked)},
        "package_count": len(records), "file_count": file_count,
        "directory_count": len(all_directories), "directories": sorted(all_directories),
        "packages": records}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).absolute().parents[1])
    parser.add_argument("--write-inventory", action="store_true",
                        help="SUPPLIER ONLY: explicitly replace the inventory snapshot")
    args = parser.parse_args()
    root = Path(os.path.abspath(args.root))
    inventory = recompute(root)
    destination = root / INVENTORY
    if args.write_inventory:
        if destination.exists() or destination.is_symlink():
            node(destination)
        destination.write_bytes((json.dumps(inventory, indent=2, sort_keys=True) + "\n").encode("utf-8"))
        print("SUPPLIER snapshot written; archive identity is not established by this operation")
    else:
        require(json_file(destination) == inventory, "committed inventory mismatch")
        print(f"PASS vendor integrity: {inventory['package_count']} packages, {inventory['file_count']} files")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"FAIL vendor integrity: {error}", file=sys.stderr)
        sys.exit(1)
