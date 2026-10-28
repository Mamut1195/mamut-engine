"""Checks MAM-835 Rust/FFI/facade byte parity and SFC-010 diagnostics."""
import json
import os
from pathlib import Path
import unittest

import mamut_engine as engine
from mamut_engine import _native

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "crates/engine-api/fixtures/input/valid"


class Sfc010Test(unittest.TestCase):
    def test_every_new_fixture_matches_rust_bytes_through_both_python_surfaces(self):
        references = os.environ.get("MAMUT_RUST_REFERENCES")
        self.assertTrue(references, "Set MAMUT_RUST_REFERENCES using facade_reference")
        paths = sorted(FIXTURES.glob("sfc010-*.json"))
        self.assertEqual(len(paths), 11)
        for path in paths:
            with self.subTest(fixture=path.name):
                source = path.read_text()
                expected = (Path(references) / path.name).read_bytes()
                self.assertEqual(_native.run(source).encode(), expected)
                result = engine.run(source)
                self.assertEqual(result.to_json().encode(), expected)
                self.assertEqual(engine.validate(source), {"diagnostics": []})
                self.assertEqual(engine.Results.from_json(result.to_json()).to_json().encode(),
                                 expected)
                self.assertEqual(result.to_arrays().document, result.data)

    def test_failed_newton_keeps_accepted_step_and_stable_diagnostic(self):
        value = json.loads((FIXTURES / "sfc010-steel.json").read_text())
        value["analyses"][0]["options"]["max_iterations"] = 1
        with self.assertRaises(engine.NotConverged) as caught:
            engine.run(value)
        error = caught.exception
        self.assertEqual(error.code, "ENG-E-0007")
        steps = error.results.data["convergence"]["steps"]
        self.assertEqual(len(steps), 2)
        self.assertTrue(steps[0]["converged"])
        self.assertFalse(steps[1]["converged"])
        self.assertIsNone(steps[1]["results"])
        self.assertEqual(error.results.data["cases"], [])
        self.assertIn("SFC-010", engine.explain("API-E-SFC-OPTIONS"))

    def test_experimental_discovery_and_explicit_job_opt_in(self):
        descriptors = {d["workflow"]["method_id"]: d
                       for d in engine.capabilities()["method_descriptors"]}
        for method in ("nonlinearStatic", "soilProfile", "consolidation1d"):
            self.assertEqual(descriptors[method]["status"], "Experimental")
        self.assertIn("ENG-VAL-006", engine.explain("API-E-SFC-OPTIONS"))
        for path in sorted(FIXTURES.glob("sfc010-*.json")):
            value = json.loads(path.read_text())
            if not value["options"].get("allow_experimental"):
                continue
            for options in ({}, {"allow_experimental": False}):
                with self.subTest(fixture=path.name, options=options):
                    value["options"] = options
                    with self.assertRaises(_native.InvalidInputError) as native_error:
                        _native.run(json.dumps(value))
                    self.assertIn("Experimental", str(native_error.exception))
                    self.assertIn("options.allow_experimental = true",
                                  str(native_error.exception))
                    with self.assertRaises(engine.InvalidInputError) as caught:
                        engine.run(value)
                    self.assertEqual(caught.exception.code, "API-E-INPUT")
