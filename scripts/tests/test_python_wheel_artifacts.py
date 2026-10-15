"""Regresiones de portabilidad y selección del SBOM (issue #94)."""
import copy
from contextlib import redirect_stdout
import importlib.util
import io
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import zipfile

SPEC = importlib.util.spec_from_file_location("artifacts", Path(__file__).parents[1] / "python-wheel-artifacts.py")
artifacts = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(artifacts)


class SbomTest(unittest.TestCase):
    def metadata(self, prefix):
        binding_id = f"path+file://{prefix}/bindings/python#mamut-engine-py@0.1.0"
        engine_id = f"path+file://{prefix}/crates/engine-core#0.1.0"
        return {
            "packages": [
                {"id": binding_id, "name": "mamut-engine-py", "version": "0.1.0", "source": None},
                {"id": engine_id, "name": "engine-core", "version": "0.1.0", "source": None},
                {"id": "unrelated", "name": "unrelated", "version": "0.1.0", "source": None},
            ],
            "resolve": {"nodes": [
                {"id": binding_id, "deps": [{"pkg": engine_id}]},
                {"id": engine_id, "deps": []},
                {"id": "unrelated", "deps": []},
            ]},
        }

    def test_refs_independent_of_checkout_and_metadata_order(self):
        original = self.metadata("/workspace/repo")
        relocated = self.metadata("/home/user/repo")
        relocated["packages"].reverse()
        relocated["resolve"]["nodes"].reverse()
        first = sorted(artifacts.dependency_graph(original)[3].values())
        second = sorted(artifacts.dependency_graph(relocated)[3].values())
        self.assertEqual(first, second)
        self.assertEqual(len(first), 2)
        artifacts.check_portable(first)

    def test_reject_absolute_refs_recursively(self):
        for ref in ("path+file:///workspace/repo#engine@0.1.0", "/home/user/repo", "/tmp/build",
                    "/opt/build", "C:\\Users\\build", "prefix=/workspace/repo", "pkg:local/tmp/build"):
            with self.subTest(ref=ref), self.assertRaises(ValueError):
                artifacts.check_portable({"components": [{"bom-ref": ref}]})

    def test_accept_portable_refs_and_contracts(self):
        artifacts.check_portable({"bom-ref": "pkg:cargo/pyo3@0.25.1", "contracts": ["engine-input/1", "results/1"]})

    def test_reject_identity_collision(self):
        metadata = self.metadata("/tmp/repo")
        duplicate = copy.deepcopy(metadata["packages"][1])
        duplicate["id"] = "another-local-engine"
        metadata["packages"].append(duplicate)
        metadata["resolve"]["nodes"].append({"id": duplicate["id"], "deps": []})
        metadata["resolve"]["nodes"][0]["deps"].append({"pkg": duplicate["id"]})
        with self.assertRaises(ValueError):
            artifacts.dependency_graph(metadata)

    def test_deterministic_serialization(self):
        with tempfile.TemporaryDirectory() as directory:
            a, b = Path(directory) / "a", Path(directory) / "b"
            artifacts.write_json(a, {"version": 1, "bomFormat": "CycloneDX"})
            artifacts.write_json(b, {"bomFormat": "CycloneDX", "version": 1})
            self.assertEqual(a.read_bytes(), b.read_bytes())
            self.assertEqual(json.loads(a.read_text())["version"], 1)

    def fixture(self, directory, readme="README portable"):
        directory.mkdir()
        binding = directory / "bindings/python"
        binding.mkdir(parents=True)
        (binding / "pyproject.toml").write_text('[project]\nname = "mamut-engine"\n')
        for name in ("LICENSE", "NOTICE"):
            (binding / name).write_text("Fixture: no texto legal.\n")
        metadata = self.metadata(str(directory))
        for package in metadata["packages"]:
            package["license"] = "MIT" if package["name"] != "engine-core" else None
            package["manifest_path"] = str(directory / package["name"] / "Cargo.toml")
        metadata_path = directory / "metadata.json"
        artifacts.write_json(metadata_path, metadata)
        wheel = directory / "mamut_engine-0.1.0-cp310-abi3-linux_x86_64.whl"
        with zipfile.ZipFile(wheel, "w") as archive:
            info = "mamut_engine-0.1.0.dist-info"
            archive.writestr(f"{info}/METADATA", "Name: mamut-engine\nVersion: 0.1.0\nRequires-Python: >=3.10\nLicense-File: LICENSE\nLicense-File: NOTICE\n\n" + readme + "\n")
            archive.writestr(f"{info}/WHEEL", "Wheel-Version: 1.0\nTag: cp310-abi3-linux_x86_64\n")
            for name in ("LICENSE", "NOTICE"):
                archive.writestr(f"{info}/licenses/{name}", (binding / name).read_bytes())
        return SimpleNamespace(root=directory, metadata=metadata_path, wheel=wheel,
                               rustc_version="rustc fixture", maturin_version="maturin fixture")

    def test_generated_sbom_and_notices_survive_relocation(self):
        with tempfile.TemporaryDirectory() as directory:
            first, second = (self.fixture(Path(directory) / name) for name in ("a", "b"))
            for args in (first, second):
                artifacts.artifacts(args)
            for name in ("SBOM.cdx.json", "THIRD_PARTY_NOTICES"):
                self.assertEqual((first.root / name).read_bytes(), (second.root / name).read_bytes())
            sbom = json.loads((first.root / "SBOM.cdx.json").read_text())
            compatibility = json.loads((first.root / "COMPATIBILITY.json").read_text())
            status = compatibility["contract_status"]
            for operation in ("validate", "run", "capabilities", "schema", "explain", "version"):
                self.assertIn(operation, status)
            self.assertIn({"name": "mamut:contract_status", "value": status},
                          sbom["metadata"]["properties"])
            artifacts.check_portable(sbom)
            refs = {c["bom-ref"] for c in sbom["components"]}
            refs.add(sbom["metadata"]["component"]["bom-ref"])
            for dependency in sbom["dependencies"]:
                self.assertIn(dependency["ref"], refs)
                self.assertTrue(set(dependency["dependsOn"]) <= refs)
            self.assertIn("PENDIENTE", (first.root / "THIRD_PARTY_NOTICES").read_text())

    def test_reject_changed_legal_file(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.fixture(Path(directory) / "repo")
            (args.root / "bindings/python/NOTICE").write_text("Cambio no incluido en wheel")
            with self.assertRaisesRegex(ValueError, "Archivo legal distinto"):
                artifacts.artifacts(args)

    def test_reject_workspace_path_in_packaged_readme(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.fixture(Path(directory) / "repo", "export TARGET=/workspace/build")
            with self.assertRaisesRegex(ValueError, "dist-info/METADATA"):
                artifacts.artifacts(args)


class FacadeSmokeTest(unittest.TestCase):
    def run_smoke(self, *, engine_version="0.1.0", native_version="0.1.0",
                  contracts=None, binding_version="0.1.0", rust_tool=None):
        facade = {"engine_version": engine_version,
                  "contracts": contracts if contracts is not None else ["engine-input/1", "results/1"]}
        native = dict(facade, engine_version=native_version)
        module = SimpleNamespace(version=lambda: facade, __version__="0.1.0",
                                 _native=SimpleNamespace(version=lambda: native))
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "COMPATIBILITY.json"
            artifacts.write_json(manifest, {"engine_version": "0.1.0", "binding_version": "0.1.0",
                                            "contracts": ["engine-input/1", "results/1"]})
            output = io.StringIO()
            with patch.dict("sys.modules", {"mamut_engine": module}), \
                    patch("importlib.metadata.version", return_value=binding_version), \
                    patch.object(artifacts.shutil, "which", return_value=rust_tool), \
                    redirect_stdout(output):
                artifacts.smoke(manifest)
            return json.loads(output.getvalue())

    def test_dict_version_produces_scalar_engine_version_and_contracts(self):
        evidence = self.run_smoke()
        self.assertEqual(evidence["engine_version"], "0.1.0")
        self.assertEqual(evidence["contracts"], ["engine-input/1", "results/1"])
        self.assertTrue(evidence["native_version_call"])
        self.assertFalse(evidence["rust_on_path"])

    def test_reject_mismatched_versions_contracts_and_visible_rust(self):
        for changes, message in [
            ({"engine_version": "9.0.0"}, "Versión del engine"),
            ({"native_version": "9.0.0"}, "Versión del engine"),
            ({"contracts": ["engine-input/9"]}, "Contratos"),
            ({"binding_version": "9.0.0"}, "Versión de distribución"),
            ({"rust_tool": "rustc"}, "Herramienta Rust visible"),
        ]:
            with self.subTest(changes=changes), self.assertRaisesRegex(ValueError, message):
                self.run_smoke(**changes)


class WheelPortabilityTest(unittest.TestCase):
    def test_reject_absolute_paths_in_every_member(self):
        members = ("pkg-0.1.dist-info/METADATA", "pkg-0.1.dist-info/RECORD",
                   "pkg-0.1.dist-info/sboms/pkg.cyclonedx.json",
                   "pkg-0.1.dist-info/other/nested", "pkg/_native.abi3.so", "pkg/data")
        paths = (b"/workspace/repo", b"/home/user", b"/tmp/build", b"/root/.cargo",
                 b"file:///build", b"path+file:///workspace/repo", b"C:\\Users\\build", b"D:/build")
        with tempfile.TemporaryDirectory() as directory:
            wheel = Path(directory) / "synthetic.whl"
            for member in members:
                for path in paths:
                    with self.subTest(member=member, path=path):
                        with zipfile.ZipFile(wheel, "w") as archive:
                            archive.writestr("pkg/__init__.py", b"# portable\n")
                            # No texto válido: prueba búsqueda en bytes con NUL y sin UTF-8.
                            archive.writestr(member, b"\xff\x00prefix=" + path + b"\x00\xfe")
                        with self.assertRaisesRegex(ValueError, member):
                            artifacts.check_wheel_portable(wheel)

    def test_reject_absolute_member_name(self):
        with tempfile.TemporaryDirectory() as directory:
            wheel = Path(directory) / "synthetic.whl"
            with zipfile.ZipFile(wheel, "w") as archive:
                archive.writestr("/root/build/data", b"portable")
            with self.assertRaisesRegex(ValueError, "miembro /root/build/data"):
                artifacts.check_wheel_portable(wheel)

    def test_accept_portable_members_and_urls(self):
        with tempfile.TemporaryDirectory() as directory:
            wheel = Path(directory) / "synthetic.whl"
            with zipfile.ZipFile(wheel, "w") as archive:
                archive.writestr("pkg/_native.abi3.so", b"\xff\x00cargo-home/src/lib.rs\x00\xfe")
                archive.writestr("pkg-0.1.dist-info/METADATA", b"Home-page: https://example.org/\n")
                archive.writestr("pkg-0.1.dist-info/RECORD", b"pkg/_native.abi3.so,sha256=abcd,24\n")
            artifacts.check_wheel_portable(wheel)


if __name__ == "__main__":
    unittest.main()
