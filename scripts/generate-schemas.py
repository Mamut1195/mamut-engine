#!/usr/bin/env python3
"""Generates, checks, and archives locally; uses only the standard library."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_BASE = "https://schemas.mamut.systems/"
KIND = re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*/[1-9][0-9]*\Z")


def canonical(value):
    # This format only contains strings, integers, and collections (no floats).
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")


def base_url(value):
    parts = urlsplit(value)
    if (parts.scheme != "https" or not parts.netloc or parts.query
            or parts.fragment or parts.username or parts.password
            or not value.endswith("/") or not value.isascii()
            or any(c.isspace() for c in value)):
        raise ValueError("The base must be an ASCII HTTPS URL ending in / without a query or fragment")
    return value


def inventory(directory):
    """Inventories exactly, without following links or accepting special files."""
    if directory.is_symlink() or not directory.is_dir():
        raise ValueError(f"Missing or linked directory: {directory}")
    files, directories = set(), set()
    for path in directory.rglob("*"):
        name = path.relative_to(directory).as_posix()
        if path.is_symlink():
            raise ValueError(f"Link not allowed: {path}")
        if path.is_dir():
            directories.add(name)
        elif path.is_file():
            files.add(name)
        else:
            raise ValueError(f"Special file not allowed: {path}")
    return files, directories


def additional_documents(root):
    config = json.loads((root / "scripts/schema-sources.json").read_bytes())
    documents, sources = {}, set()
    for entry in config["additional_schemas"]:
        kind, source = entry["kind"], entry["source"]
        path = Path(source)
        if (not KIND.fullmatch(kind) or path.is_absolute() or ".." in path.parts
                or path.as_posix() != source or not source.startswith("crates/engine-cli/schemas/")):
            raise ValueError(f"Invalid additional entry: {entry}")
        if kind in documents or source in sources:
            raise ValueError("Duplicate additional registration")
        if any(p.is_symlink() for p in (root / path, *(root / path).parents)):
            raise ValueError(f"Linked source: {source}")
        documents[kind] = (root / path).read_bytes()
        sources.add(source)
    directory = root / "crates/engine-cli/schemas"
    if directory.exists() or directory.is_symlink():
        files, _ = inventory(directory)
        actual = {f"crates/engine-cli/schemas/{name}" for name in files}
        if actual != sources:
            raise ValueError("There are unregistered CLI schemas or missing additional sources")
    return documents


def build_tree(api, root, base):
    documents = {kind: text.encode("utf-8") for kind, text in api.items()}
    for kind, data in additional_documents(root).items():
        if kind in documents:
            raise ValueError(f"Duplicate kind across API and CLI: {kind}")
        documents[kind] = data
    return document_tree(documents, base)


def document_tree(documents, base):
    entries, tree = [], {}
    for kind, data in sorted(documents.items()):
        if not KIND.fullmatch(kind):
            raise ValueError(f"Invalid kind: {kind}")
        identifier, version = kind.split("/")
        schema_id = json.loads(data)["$id"]
        if not isinstance(schema_id, str) or not schema_id:
            raise ValueError(f"Invalid $id: {kind}")
        relative = kind + ".json"
        tree[relative] = data
        entries.append({"id": identifier, "version": int(version), "$id": schema_id,
                        "path": relative, "url": base + relative,
                        "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)})
    tree["index.json"] = canonical({"base_url": base_url(base), "schemas": entries})
    return tree


def check_tree(directory, tree):
    files, directories = inventory(directory)
    expected_dirs = {str(Path(name).parent) for name in tree if name != "index.json"}
    if files != set(tree) or directories != expected_dirs:
        raise ValueError("The schemas/ inventory differs from the registry (missing or orphaned files)")
    for name, data in tree.items():
        if (directory / name).read_bytes() != data:
            raise ValueError(f"Outdated bytes: schemas/{name}")


def verified_tree(directory):
    """Checks the index and bytes before packaging, without running Cargo."""
    inventory(directory)
    index_data = (directory / "index.json").read_bytes()
    index = json.loads(index_data)
    base = base_url(index["base_url"])
    documents = {}
    for entry in index["schemas"]:
        kind = f'{entry["id"]}/{entry["version"]}'
        if not KIND.fullmatch(kind) or kind in documents:
            raise ValueError("Invalid or duplicate kind in index")
        documents[kind] = (directory / (kind + ".json")).read_bytes()
    tree = document_tree(documents, base)
    if index_data != tree["index.json"]:
        raise ValueError("Noncanonical index or incorrect metadata")
    check_tree(directory, tree)
    return tree


def package(directory, output):
    tree = verified_tree(directory)
    output = output.resolve()
    if output == directory.resolve() or directory.resolve() in output.parents:
        raise ValueError("The output must be outside schemas/")
    output.mkdir(parents=True, exist_ok=True)
    target = output / "mamut-schemas.tar"
    if target.is_symlink():
        raise ValueError("The output archive cannot be a link")
    with tarfile.open(target, "w", format=tarfile.USTAR_FORMAT) as archive:
        for name, data in sorted(tree.items()):
            info = tarfile.TarInfo("schemas/" + name)
            info.size, info.mode, info.mtime = len(data), 0o644, 0
            info.uid, info.gid, info.uname, info.gname = 0, 0, "", ""
            archive.addfile(info, io.BytesIO(data))
    return target


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--package", type=Path, metavar="OUT_DIR")
    parser.add_argument("--api-stdin", action="store_true", help="Internal transport for the Cargo test")
    parser.add_argument("--base-url", help=f"Default when generating: {DEFAULT_BASE}")
    args = parser.parse_args()
    if args.write and ("CI" in os.environ or os.environ.get("GITHUB_ACTIONS")):
        raise ValueError("Regeneración bloqueada en CI; ejecutar --check")
    directory = ROOT / "schemas"
    if args.package:
        if args.api_stdin or args.base_url:
            raise ValueError("--package does not accept generation options")
        target = package(directory, args.package)
        print(f"{hashlib.sha256(target.read_bytes()).hexdigest()}  {target}")
        return
    if args.api_stdin:
        api = json.load(sys.stdin)
    else:
        result = subprocess.run(["cargo", "run", "--offline", "--quiet", "-p", "engine-api",
                                 "--example", "export_schemas"], cwd=ROOT, check=True,
                                stdout=subprocess.PIPE)
        api = json.loads(result.stdout)
    base = args.base_url
    if base is None:
        base = (json.loads((directory / "index.json").read_bytes())["base_url"]
                if args.check else DEFAULT_BASE)
    tree = build_tree(api, ROOT, base)
    if args.write:
        if directory.exists() or directory.is_symlink():
            files, directories = inventory(directory)
            expected_dirs = {str(Path(n).parent) for n in tree if n != "index.json"}
            if files - set(tree) or directories - expected_dirs:
                raise ValueError("Remove orphaned files manually before regenerating")
        directory.mkdir(exist_ok=True)
        for name, data in tree.items():
            path = directory / name
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(data)
    check_tree(directory, tree)
    print("schemas/: bytes, index, and inventory OK")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
