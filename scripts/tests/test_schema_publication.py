#!/usr/bin/env python3
"""Runs offline tests, also executed by cargo test."""
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
TOOLS = runpy.run_path(str(ROOT / "scripts/generate-schemas.py"))


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        (self.root / "scripts").mkdir()
        self.config([])
        self.api = {"sample/1": '{\r\n  "$id": "urn:mamut:sample:1"\r\n}\r\n'}
        self.tree = TOOLS["build_tree"](self.api, self.root, TOOLS["DEFAULT_BASE"])
        self.directory = self.root / "schemas"
        for name, data in self.tree.items():
            path = self.directory / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)

    def config(self, entries):
        (self.root / "scripts/schema-sources.json").write_text(
            json.dumps({"additional_schemas": entries}), encoding="utf-8")

    def test_bytes_base_and_additional_registration(self):
        self.assertEqual(self.tree["sample/1.json"], self.api["sample/1"].encode())
        index = json.loads(self.tree["index.json"])
        entry = index["schemas"][0]
        self.assertEqual(entry["sha256"], hashlib.sha256(self.tree["sample/1.json"]).hexdigest())
        source = "crates/engine-cli/schemas/cli-error-1.json"
        path = self.root / source
        path.parent.mkdir(parents=True)
        path.write_bytes(b'{"$id":"urn:mamut:cli-error:1"}\n')
        with self.assertRaises(ValueError):
            TOOLS["build_tree"](self.api, self.root, TOOLS["DEFAULT_BASE"])
        self.config([{"kind": "cli-error/1", "source": source}])
        tree = TOOLS["build_tree"](self.api, self.root, "https://example.test/contracts/")
        self.assertEqual(tree["cli-error/1.json"], path.read_bytes())
        self.assertEqual(json.loads(tree["index.json"])["schemas"][0]["url"],
                         "https://example.test/contracts/cli-error/1.json")
        self.config([{"kind": "sample/1", "source": source}])
        with self.assertRaises(ValueError):
            TOOLS["build_tree"](self.api, self.root, TOOLS["DEFAULT_BASE"])

    def test_orphans_links_and_tampering_are_rejected(self):
        TOOLS["verified_tree"](self.directory)
        extra = self.directory / "extra"
        extra.mkdir()
        with self.assertRaises(ValueError):
            TOOLS["verified_tree"](self.directory)
        extra.rmdir()
        extra.write_bytes(b"extra")
        with self.assertRaises(ValueError):
            TOOLS["verified_tree"](self.directory)
        extra.unlink()
        extra.symlink_to(self.directory / "index.json")
        with self.assertRaises(ValueError):
            TOOLS["verified_tree"](self.directory)
        extra.unlink()
        document = self.directory / "sample/1.json"
        document.write_bytes(document.read_bytes() + b" ")
        with self.assertRaises(ValueError):
            TOOLS["verified_tree"](self.directory)
        document.write_bytes(self.tree["sample/1.json"])
        index_path = self.directory / "index.json"
        index = json.loads(index_path.read_bytes())
        index["schemas"][0]["bytes"] += 1
        index_path.write_bytes(TOOLS["canonical"](index))
        with self.assertRaises(ValueError):
            TOOLS["verified_tree"](self.directory)
        index_path.write_bytes(self.tree["index.json"])
        document.unlink()
        with self.assertRaises((ValueError, OSError)):
            TOOLS["verified_tree"](self.directory)

    def test_archive_is_stable_despite_source_metadata(self):
        first = TOOLS["package"](self.directory, self.root / "first").read_bytes()
        for path in self.directory.rglob("*.json"):
            os.utime(path, (123456789, 123456789))
            path.chmod(0o600)
        archive = TOOLS["package"](self.directory, self.root / "second")
        self.assertEqual(hashlib.sha256(first).digest(), hashlib.sha256(archive.read_bytes()).digest())
        with tarfile.open(archive) as stream:
            members = stream.getmembers()
            self.assertEqual([m.name for m in members], sorted("schemas/" + n for n in self.tree))
            for member in members:
                self.assertEqual((member.mtime, member.uid, member.gid, member.uname,
                                  member.gname, member.mode), (0, 0, 0, "", "", 0o644))
                self.assertEqual(stream.extractfile(member).read(), self.tree[member.name[8:]])
        with self.assertRaises(ValueError):
            TOOLS["package"](self.directory, self.directory / "out")

    def test_release_script_twice_from_another_directory(self):
        archives = []
        for name in ("one", "two"):
            output = self.root / name
            subprocess.run(["bash", str(ROOT / "scripts/package-schemas.sh"), str(output)],
                           cwd=self.root, check=True, stdout=subprocess.PIPE)
            archives.append(hashlib.sha256((output / "mamut-schemas.tar").read_bytes()).digest())
        self.assertEqual(*archives)

    def test_ci_cannot_regenerate(self):
        for variable in ("CI", "GITHUB_ACTIONS"):
            result = subprocess.run(["python3", str(ROOT / "scripts/generate-schemas.py"), "--write"],
                                    env={**os.environ, variable: "true"}, cwd=self.root,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b"CI", result.stderr)

    def test_empty_ci_cannot_regenerate(self):
        env = {key: value for key, value in os.environ.items() if key != "GITHUB_ACTIONS"}
        env["CI"] = ""
        result = subprocess.run(
            ["python3", str(ROOT / "scripts/generate-schemas.py"), "--write", "--api-stdin"],
            env=env, cwd=self.root, input=b"", stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"Regeneraci\xc3\xb3n bloqueada en CI; ejecutar --check", result.stderr)

    def test_invalid_kind_and_base(self):
        for base in ("http://example.test/", "https://example.test", "https://example.test/?a=1"):
            with self.assertRaises(ValueError):
                TOOLS["build_tree"](self.api, self.root, base)
        with self.assertRaises(ValueError):
            TOOLS["build_tree"]({"../1": self.api["sample/1"]}, self.root, TOOLS["DEFAULT_BASE"])


if __name__ == "__main__":
    unittest.main()
