"""ENG-PY-006: shared analytical matrix and independent Rust byte references."""

import hashlib
import json
import math
import os
from pathlib import Path
import sys
import unittest

import mamut_engine as engine
from mamut_engine import _native

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "crates/engine-api/tests/fixtures/parity"
MATRIX = json.loads((FIXTURES / "matrix.json").read_text())


def pointer(document, path):
    for part in path.lstrip("/").split("/"):
        key = part.replace("~1", "/").replace("~0", "~")
        document = document[int(key)] if isinstance(document, list) else document[key]
    return document


class ParityMatrixTest(unittest.TestCase):
    def test_same_matrix_matches_closed_forms_rust_bytes_and_typed_errors(self):
        references = os.environ.get("MAMUT_PARITY_REFERENCES")
        self.assertTrue(references, "Set MAMUT_PARITY_REFERENCES using the Rust parity_matrix test")
        self.assertEqual(MATRIX["schema"], "parity-matrix/1")
        self.assertEqual(len(MATRIX["cases"]), 3)
        for case in MATRIX["cases"]:
            source = (FIXTURES / case["input"]).read_text()
            expected = (Path(references) / case["input"]).read_bytes()
            expected_jcs_hash = (Path(references) / f"{case['input']}.jcs-sha256").read_text()
            model = json.loads(source)
            self.assertEqual(model["units"], "SI")
            self.assertEqual(model["model"]["axes"], "Z-up")
            for surface, run, error_type in (
                ("native", _native.run, _native.AnalysisError),
                ("facade", engine.run, engine.Unstable),
            ):
                with self.subTest(case=case["input"], surface=surface):
                    if case["diagnostic"]:
                        with self.assertRaises(error_type) as caught:
                            run(source)
                        error = caught.exception
                        self.assertEqual(error.code, case["diagnostic"])
                        self.assertTrue(error.entities)
                        actual = error.document
                        self.assertEqual(error.diagnostics, json.loads(actual)["diagnostics"])
                        if surface == "facade":
                            self.assertEqual(error.results.to_json(), actual)
                    else:
                        result = run(source)
                        actual = result if surface == "native" else result.to_json()
                        self.assertEqual(engine.validate(source), {"diagnostics": []})
                        self.assertEqual(engine.Results.from_json(actual).to_json(), actual)
                    encoded = actual.encode()
                    self.assertEqual(encoded, expected)
                    digest = hashlib.sha256(encoded).hexdigest()
                    self.assertEqual(digest, hashlib.sha256(expected).hexdigest())
                    # JCS fingerprints and raw deterministic-JSON hashes use
                    # different preimages; compare each to its Rust reference.
                    self.assertEqual(_native.input_sha256(actual), expected_jcs_hash)
                    self.assertEqual(_native.canonical_json(actual), actual)
                    document = json.loads(actual)
                    self.assertEqual(document["schema"], "results/1")
                    self.assertEqual(document["status"], case["status"])
                    self.assertEqual(document["input_sha256"],
                                     engine.Model.from_json(source).input_sha256)
                    if case["diagnostic"]:
                        first = next(d for d in document["diagnostics"] if d["severity"] == "error")
                        self.assertEqual(first["code"], case["diagnostic"])
                        self.assertEqual(document["cases"], [])
                        self.assertEqual(case["checks"], [])
                        continue
                    self.assertEqual(document["diagnostics"], [])
                    self.assertEqual(len(document["nodes"]), len(model["model"]["nodes"]))
                    for node, input_node in zip(document["nodes"], model["model"]["nodes"]):
                        self.assertEqual(node["id"], input_node["id"])
                        self.assertEqual(node["dofs"], input_node["dofs"])
                        self.assertEqual(node["dim"], "D2")
                    tolerance = (MATRIX["tolerance_factor"] * sys.float_info.epsilon
                                 * case["free_dofs"] * case["condition_bound"])
                    self.assertTrue(case["checks"])
                    for check in case["checks"]:
                        value = pointer(document, check["pointer"])
                        exact = check["fraction"][0] / check["fraction"][1]
                        bound = tolerance * max(abs(exact), check["scale"])
                        self.assertTrue(math.isfinite(value))
                        self.assertLessEqual(abs(value - exact), bound,
                                             f"{check['pointer']} {check['unit']}")
