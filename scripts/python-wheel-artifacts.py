#!/usr/bin/env python3
"""Produces deterministic wheel artifacts using only the standard library (CPython >=3.11)."""

import argparse
from email.parser import BytesParser
import hashlib
import json
from pathlib import Path
import platform
import re
import shutil
import sys
import tomllib
from urllib.parse import quote
import zipfile


# Bytes: also inspects binaries without decoding or ignoring UTF-8 errors.
# Left boundaries for drives and UNC avoid confusing https:// URLs
# with an s:/ drive or a //servidor/recurso path.
# UNC requires a server name to avoid confusing binary instructions.
SYSTEM_PATH_PREFIX = (
    rb"/(?:workspace|home|tmp|root|opt|Users|var)\b"
    rb"|(?<![A-Za-z0-9+.-])[A-Za-z]:[\\/]"
    rb"|(?<![A-Za-z0-9+.:/\\-])(?:\\\\[\w.-]+\\|//[\w.-]+/)[^\\/\s\x00]+"
    rb"|file://|path\+"
)
SYSTEM_PATH = re.compile(SYSTEM_PATH_PREFIX + rb"|~/")
# Only ~/ needs additional context: these two bytes are common in machine code.
# Left: start, space, single/double quote, =, :, or a byte outside printable ASCII
# (0x20..0x7e). Right: at least 2 bytes from [A-Za-z0-9._/-],
# terminated by end of data, NUL, or an explicit ASCII text separator.
# This also rejects ~/xs followed by 0xc8 in the real .so from #119.
# The other patterns retain their boundaries and existing strict rejection.
BINARY_SYSTEM_PATH = re.compile(
    SYSTEM_PATH_PREFIX
    + rb"|(?:^|(?<=[ '\"=:\x00-\x1f\x7f-\xff]))~/"
    rb"(?=[A-Za-z0-9._/-]{2,}(?:$|[\x00\t\r\n '\"=:;,()\[\]{}]))"
)


CONTRACT_STATUS = (
    "Fachada completa engine-api: validate/run/capabilities/schema/explain/version; "
    "entrada engine-input/1 y salida canónica results/1"
)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n")


def check_portable(value, *, binary=None):
    """Rejects paths; treats bytes with NUL or invalid UTF-8 as binary.

    Strings and UTF-8 bytes without NUL retain the strict text pattern.
    Does not discard or decode binary bytes with data loss.
    """
    if isinstance(value, dict):
        for key, item in value.items():
            check_portable(key)
            check_portable(item)
    elif isinstance(value, list):
        for item in value:
            check_portable(item)
    elif isinstance(value, str):
        check_portable(value.encode("utf-8"), binary=False)
        if re.search(r"(?:^|[\s(=])/(?!/)", value):
            raise ValueError(f"Ruta del sistema en artefacto: {value!r}")
    elif isinstance(value, bytes):
        if binary is None:
            binary = b"\x00" in value
            if not binary:
                try:
                    value.decode("utf-8")
                except UnicodeDecodeError:
                    binary = True
        match = (BINARY_SYSTEM_PATH if binary else SYSTEM_PATH).search(value)
        if match:
            raise ValueError(f"Ruta del sistema en artefacto, byte {match.start()}: {value[match.start():match.end() + 80]!r}")


def check_wheel_portable(wheel):
    """Inspects ALL ZIP members, including dist-info/RECORD and .so."""
    with zipfile.ZipFile(wheel) as archive:
        for member in archive.infolist():
            try:
                check_portable(member.filename)
                check_portable(archive.read(member))
            except ValueError as error:
                raise ValueError(f"Wheel {Path(wheel).name}, miembro {member.filename}: {error}") from error


def dependency_graph(metadata):
    packages = {p["id"]: p for p in metadata["packages"]}
    root = next(p for p in packages.values() if p["name"] == "mamut-engine-py")
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    pending, selected = [root["id"]], set()
    while pending:
        item = pending.pop()
        if item not in selected:
            selected.add(item)
            pending.extend(d["pkg"] for d in nodes[item]["deps"])
    # Does not reuse Cargo package IDs: local crates contain absolute paths.
    refs = {}
    for item in selected:
        p = packages[item]
        source = "cargo" if p["source"] is not None else "mamut-local"
        refs[item] = f"pkg:{source}/{quote(p['name'], safe='')}@{quote(p['version'], safe='')}"
    if len(set(refs.values())) != len(refs):
        raise ValueError("Ambiguous SBOM identities: review sources of duplicate crates")
    return packages, root, nodes, refs


def artifacts(args):
    wheel, root_dir = Path(args.wheel), Path(args.root)
    check_wheel_portable(wheel)
    metadata = json.loads(Path(args.metadata).read_text())
    packages, binding, nodes, refs = dependency_graph(metadata)
    engine = next(p for p in packages.values() if p["name"] == "engine-core")
    project = tomllib.loads((root_dir / "bindings/python/pyproject.toml").read_text())["project"]
    with zipfile.ZipFile(wheel) as archive:
        info = next(n.rsplit("/", 1)[0] for n in archive.namelist() if n.endswith(".dist-info/METADATA"))
        pkg = BytesParser().parsebytes(archive.read(f"{info}/METADATA"))
        wheel_info = BytesParser().parsebytes(archive.read(f"{info}/WHEEL"))
        for name in ("LICENSE", "NOTICE"):
            if archive.read(f"{info}/licenses/{name}") != (root_dir / "bindings/python" / name).read_bytes():
                raise ValueError(f"Archivo legal distinto: {name}")
            if name not in pkg.get_all("License-File", []):
                raise ValueError(f"Missing License-File: {name}")
        if pkg["Version"] != binding["version"] or pkg["Name"] != project["name"]:
            raise ValueError("Wheel version/name does not match the manifest")
    components = []
    notices = ["THIRD_PARTY_NOTICES — inventario automático; PENDIENTE de revisión legal.",
               "Licencias declaradas en cargo metadata; no valida titularidad ni CLA.",
               "No sustituye textos de licencia ni una revisión previa a distribuir.",
               "Alcance conservador: dependencias transitivas Cargo de build/runtime de todos los destinos.", ""]
    for item in sorted(refs, key=refs.get):
        p = packages[item]
        component = {"type": "library", "bom-ref": refs[item], "name": p["name"], "version": p["version"]}
        if p["source"] is not None and p["source"].startswith("registry+"):
            component["purl"] = refs[item]
        if p["license"]:
            component["licenses"] = [{"expression": p["license"]}]
        else:
            component["properties"] = [{"name": "mamut:license-review", "value": "PENDIENTE: sin expresión declarada"}]
        components.append(component)
        notices.append(f"{p['name']} {p['version']}: {p['license'] or 'PENDIENTE (sin expresión declarada)'}")
    sbom = {
        "bomFormat": "CycloneDX", "specVersion": "1.5", "version": 1,
        "metadata": {"component": {"type": "application", "bom-ref": f"pkg:pypi/{project['name']}@{binding['version']}",
                                   "name": project["name"], "version": binding["version"]},
                     "properties": [{"name": "mamut:scope", "value": "Cargo closure, all targets, build and runtime; system libraries/tools excluded"},
                                    {"name": "mamut:contract_status", "value": CONTRACT_STATUS},
                                    {"name": "mamut:legal-review", "value": "PENDIENTE (licencias, titularidad y CLA)"}]},
        "components": components,
        "dependencies": [{"ref": f"pkg:pypi/{project['name']}@{binding['version']}", "dependsOn": [refs[binding["id"]]]}]
        + [{"ref": refs[item], "dependsOn": sorted({refs[d["pkg"]] for d in nodes[item]["deps"]})}
           for item in sorted(refs, key=refs.get)],
    }
    check_portable(sbom)
    write_json(wheel.parent / "SBOM.cdx.json", sbom)
    (wheel.parent / "THIRD_PARTY_NOTICES").write_text("\n".join(notices) + "\n")
    tags = wheel_info.get_all("Tag", [])
    if not tags:
        raise ValueError("Wheel without tags")
    compatibility = {
        "package": project["name"], "binding_version": binding["version"], "engine_version": engine["version"],
        "contracts": ["engine-input/1", "engine-input/2", "results/1"],
        "contract_status": CONTRACT_STATUS,
        "wheel": wheel.name, "sha256": digest(wheel.read_bytes()), "requires_python": pkg["Requires-Python"],
        "tags": [{"python": t.split("-")[0], "abi": t.split("-")[1], "platform": t.split("-")[2]} for t in tags],
        "tested": {"os": platform.system(), "machine": platform.machine(), "implementation": platform.python_implementation(),
                   "python": platform.python_version()},
        "abi3_status": "Tag declarado por PyO3; matriz entre intérpretes PENDIENTE",
        "source_date_epoch": 1767225600, "rustc": args.rustc_version, "maturin": args.maturin_version,
        "legal_review": "PENDIENTE (licencias, titularidad y CLA)",
    }
    check_portable(compatibility)
    write_json(wheel.parent / "COMPATIBILITY.json", compatibility)


def compare(first, second):
    a, b = Path(first), Path(second)
    ha, hb = digest(a.read_bytes()), digest(b.read_bytes())
    if a.name != b.name or ha != hb:
        with zipfile.ZipFile(a) as za, zipfile.ZipFile(b) as zb:
            for name in sorted(set(za.namelist()) | set(zb.namelist())):
                def details(archive):
                    if name not in archive.namelist():
                        return None
                    info = archive.getinfo(name)
                    return {"sha256": digest(archive.read(name)), "timestamp": info.date_time,
                            "compression": info.compress_type, "attributes": info.external_attr,
                            "extra": info.extra.hex(), "comment": info.comment.hex()}
                left, right = details(za), details(zb)
                if left != right:
                    print(f"Difference {name}: {left} / {right}", file=sys.stderr)
        left_bytes, right_bytes = a.read_bytes(), b.read_bytes()
        offset = next((i for i, (x, y) in enumerate(zip(left_bytes, right_bytes)) if x != y), min(len(left_bytes), len(right_bytes)))
        print(f"ZIP: sizes {len(left_bytes)}/{len(right_bytes)}, first differing byte {offset}", file=sys.stderr)
        raise ValueError(f"Different wheels: {a.name} {ha} / {b.name} {hb}")
    print(json.dumps({"independent_builds": 2, "wheel": a.name, "sha256": ha, "identical": True}, sort_keys=True, indent=2))


def smoke(manifest):
    from importlib.metadata import version
    import mamut_engine
    from mamut_engine import _native

    expected = json.loads(Path(manifest).read_text())
    for tool in ("rustc", "cargo", "rustup"):
        if shutil.which(tool) is not None:
            raise ValueError(f"Herramienta Rust visible en PATH: {tool}")
    actual = mamut_engine.version()  # Real call to Rust through PyO3.
    if not (actual["engine_version"] == mamut_engine.__version__ == expected["engine_version"]
            and actual == _native.version()):
        raise ValueError("Versión del engine inesperada")
    if actual["contracts"] != expected["contracts"]:
        raise ValueError("Contratos de la fachada inesperados")
    if version("mamut-engine") != expected["binding_version"]:
        raise ValueError("Versión de distribución inesperada")
    print(json.dumps({"engine_version": actual["engine_version"], "contracts": actual["contracts"], "binding_version": version("mamut-engine"),
                      "python": platform.python_version(), "rust_on_path": False, "native_version_call": True}, sort_keys=True, indent=2))


def checksums(directory):
    directory = Path(directory)
    lines = [f"{digest(p.read_bytes())}  {p.name}\n" for p in sorted(directory.iterdir()) if p.is_file() and p.name != "SHA256SUMS"]
    (directory / "SHA256SUMS").write_text("".join(lines))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest="command", required=True)
    artifact = subs.add_parser("artifacts")
    for key in ("metadata", "wheel", "root", "rustc-version", "maturin-version"):
        artifact.add_argument(f"--{key}", required=True)
    comp = subs.add_parser("compare")
    comp.add_argument("first")
    comp.add_argument("second")
    subs.add_parser("smoke").add_argument("manifest")
    subs.add_parser("checksums").add_argument("directory")
    subs.add_parser("check-wheel").add_argument("wheel")
    args = parser.parse_args()
    if args.command == "artifacts":
        artifacts(args)
    elif args.command == "compare":
        compare(args.first, args.second)
    elif args.command == "smoke":
        smoke(args.manifest)
    elif args.command == "check-wheel":
        check_wheel_portable(args.wheel)
    else:
        checksums(args.directory)


if __name__ == "__main__":
    main()
