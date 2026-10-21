"""Contrato NumPy opcional, descubierto por el pipeline stdlib de la wheel."""
import gc
import json
import os
from pathlib import Path
import unittest

try:
    import numpy as np
except ModuleNotFoundError as exc:
    if exc.name != "numpy":
        raise
    np = None

import mamut_engine as engine

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "crates/engine-api/fixtures/input/valid"
PATHS = sorted(FIXTURES.glob("*.json")) + sorted(
    (ROOT / "crates/engine-api/tests/fixtures/determinism").glob("*.json"))


@unittest.skipUnless(np is not None, "NumPy opcional no instalado")
class NumPyContractTest(unittest.TestCase):
    def test_numpy_matches_rust_lists_and_owns_memory(self):
        for path in PATHS:
            with self.subTest(fixture=path.stem):
                self.check_numpy_matches_rust_lists_and_owns_memory(path)

    def check_numpy_matches_rust_lists_and_owns_memory(self, path):
        reference = Path(os.environ["MAMUT_RUST_REFERENCES"]) / path.name
        result = engine.run(path.read_text())
        original_json = result.to_json()
        snapshot = result.to_dict()
        self.assertEqual(original_json.encode(), reference.read_bytes())
        lists = result.to_arrays()
        arrays = result.to_numpy()
        self.assertEqual(arrays.document, lists.document)
        self.assertEqual(arrays.status, result.status)
        self.assertEqual(arrays.partial, result.partial)
        self.assertEqual(len(arrays.blocks), len(lists.blocks))
        for a, b in zip(arrays.blocks, lists.blocks):
            self.assertEqual(a.path, b.path)
            raw = arrays.document
            for key in a.path:
                raw = raw[key]
            nodes = raw if a.path[-1] == "shape" else raw["nodes"]
            for name in ("displacements", "reactions"):
                self.assertEqual(a.matrices[name].row_ids, tuple(n["id"] for n in nodes))
                self.assertEqual(a.matrices[name].values.tolist(), [n[name] for n in nodes])
            if a.path[-1] == "shape":
                self.assertEqual(a.matrices["displacements"].units,
                                 ("kg^-1/2",) * 3 + ("m^-1*kg^-1/2",) * 3)
            if isinstance(raw, dict) and "members" in raw:
                members = raw["members"]
                identities = {m["id"]: m["element"] for m in arrays.document["members"]}
                for name, matrix in a.matrices.items():
                    if name.startswith("end_forces_local/"):
                        rows = [m for m in members if m["id"] in matrix.row_ids]
                        self.assertTrue(all(identities[m["id"]] == name.split("/")[1]
                                            for m in rows))
                        self.assertEqual(matrix.row_ids, tuple(m["id"] for m in rows))
                        self.assertEqual(matrix.values.tolist(), [m["end_forces_local"] for m in rows])
                self.assertEqual(a.matrices["forces_local"].values.tolist(),
                                 [s["forces_local"] for m in members for s in m["stations"]])
            self.assertEqual(a.matrices.keys(), b.matrices.keys())
            for name, matrix in a.matrices.items():
                expected = b.matrices[name]
                self.assertEqual(matrix.row_ids, expected.row_ids)
                self.assertEqual(matrix.row_indices, expected.row_indices)
                self.assertEqual(matrix.dofs, expected.dofs)
                self.assertEqual(matrix.units, expected.units)
                self.assertEqual(matrix.frame, expected.frame)
                self.assertEqual(matrix.values.dtype, np.dtype(np.float64))
                self.assertEqual(matrix.values.shape, matrix.shape)
                self.assertEqual(matrix.shape, expected.shape)
                self.assertTrue(matrix.values.flags.c_contiguous)
                self.assertTrue(matrix.values.flags.owndata)
                self.assertIsNone(matrix.values.base)
                self.assertEqual(matrix.values.tolist(), expected.values)
                self.assertTrue(np.isfinite(matrix.values).all())

        again = result.to_numpy()
        self.assertIsNot(arrays, again)
        self.assertIsNot(arrays.document, again.document)
        for block, other, expected_block in zip(arrays.blocks, again.blocks, lists.blocks):
            self.assertIsNot(block, other)
            self.assertIsNot(block.matrices, other.matrices)
            for name, matrix in block.matrices.items():
                with self.subTest(path=block.path, matrix=name):
                    second = other.matrices[name]
                    self.assertIsNot(matrix, second)
                    self.assertIsNot(matrix.values, second.values)
                    self.assertFalse(np.shares_memory(matrix.values, second.values))
                    matrix.values[:] = 987
                    self.assertEqual(second.values.tolist(), expected_block.matrices[name].values)
        self.assertEqual(arrays.document, snapshot)
        self.assertEqual(again.document, snapshot)
        self.assertEqual(result.to_dict(), snapshot)
        self.assertEqual(result.to_json(), original_json)
        fresh = result.to_numpy()
        for block, other in zip(fresh.blocks, again.blocks):
            for name, matrix in block.matrices.items():
                self.assertEqual(matrix.values.tolist(), other.matrices[name].values.tolist())
        arrays.document["cases"].clear()
        arrays.document["modes"].clear()
        self.assertEqual(again.document, snapshot)
        self.assertEqual(result.to_dict(), snapshot)
        self.assertEqual(result.to_json().encode(), reference.read_bytes())
        if not arrays.blocks:
            # Los métodos geotécnicos conservan su campo completo en document;
            # no producen tablas de nodos estructurales artificiales.
            self.assertIn("geotechnical", arrays.document)
            self.assertEqual(arrays.document, snapshot)
            return
        saved = arrays.blocks[0].matrices["displacements"].values
        # La segunda exportación tampoco puede alterar una exportación posterior.
        again.blocks[0].matrices["displacements"].values[:] = -123
        self.assertEqual(result.to_numpy().blocks[0].matrices["displacements"].values.tolist(),
                         lists.blocks[0].matrices["displacements"].values)
        del result, arrays
        gc.collect()
        self.assertTrue((saved == 987).all())

    def test_numpy_empty_shapes_and_cancelled_state(self):
        document = engine.run(PATHS[0].read_text()).to_dict()
        # El schema admite un caso importado sin filas, aun con identidades.
        document["cases"] = [{"case": "empty", "factor": 1, "nodes": [], "members": []}]
        imported = engine.Results.from_json(json.dumps(document))
        result = imported.to_numpy()
        again = imported.to_numpy()
        for name, width in (("displacements", 6), ("reactions", 6),
                            ("forces_local", 6), ("positions", 1)):
            matrix = result.blocks[0].matrices[name]
            self.assertEqual(matrix.values.shape, (0, width))
            self.assertEqual(matrix.values.dtype, np.float64)
            self.assertTrue(matrix.values.flags.owndata)
            self.assertTrue(matrix.values.flags.c_contiguous)
            self.assertIsNot(matrix.values, again.blocks[0].matrices[name].values)
        token = engine.CancelToken()
        token.cancel()
        with self.assertRaises(engine.Cancelled) as caught:
            engine.run(PATHS[0].read_text(), cancel=token)
        failed = caught.exception.results.to_numpy()
        self.assertEqual(failed.status, "cancelled")
        self.assertTrue(failed.partial)
        self.assertEqual(failed.blocks, ())

    def test_numpy_inputs_do_not_bypass_json_contract(self):
        for bad in (np.empty((0,)), np.zeros((2, 2)),
                    np.array([np.nan, 0, 0]), np.array([np.inf, 0, 0])):
            with self.subTest(bad=bad), self.assertRaises(engine.InvalidInputError):
                source = json.loads(PATHS[0].read_text())
                source["model"]["nodes"][0]["coordinates"] = bad
                engine.run(source)


if __name__ == "__main__":
    unittest.main()
