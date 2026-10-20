"""Frontera tabular sin depender de NumPy ni de pytest."""
from copy import deepcopy
from importlib.metadata import metadata
import json
from pathlib import Path
import subprocess
import sys
import unittest

import mamut_engine as engine
from mamut_engine.arrays import export

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "crates/engine-api/fixtures/input/valid"
SOURCE = (FIXTURES / "cantilever-linear-static.json").read_text()


class ArrayContractTest(unittest.TestCase):
    def test_installed_numpy_dependency_is_only_an_extra(self):
        info = metadata("mamut-engine")
        self.assertIn("numpy", info.get_all("Provides-Extra", []))
        requirements = info.get_all("Requires-Dist", [])
        self.assertEqual(len(requirements), 1)
        self.assertTrue(requirements[0].startswith("numpy>=1.23"))
        self.assertIn('extra == "numpy"', requirements[0].replace("'", '"'))

    def test_lazy_import_and_typed_missing_dependency_in_fresh_process(self):
        code = '''
import importlib.abc, sys
class NoNumPy(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname == "numpy" or fullname.startswith("numpy."):
            raise ModuleNotFoundError("NumPy ausente", name="numpy")
sys.meta_path.insert(0, NoNumPy())
import mamut_engine as e
assert "numpy" not in sys.modules
r = e.run(sys.argv[1])
assert r.to_arrays().status == "ok"
assert isinstance(r.to_arrays().blocks[0].matrices["displacements"].values, list)
assert "numpy" not in sys.modules
try:
    r.to_numpy()
except e.MissingNumPyError as err:
    assert err.code == "PY-E-NUMPY"
    assert "mamut-engine[numpy]" in str(err)
    assert isinstance(err.__cause__, ModuleNotFoundError)
    assert err.__cause__.name == "numpy"
    assert str(err.__cause__) == "NumPy ausente"
else:
    raise AssertionError("No se rechazó NumPy ausente")
assert r.to_dict()["cases"]
'''
        completed = subprocess.run([sys.executable, "-I", "-c", code, SOURCE],
                                   capture_output=True, text=True)
        self.assertEqual(completed.returncode, 0, completed.stdout + completed.stderr)

    def test_lists_preserve_ids_labels_units_and_snapshot(self):
        model = json.loads(SOURCE)
        # IDs no consecutivos y orden de identidades distinto del orden de filas.
        model["model"]["nodes"][0]["id"] = 4294967295
        model["model"]["members"][0]["nodes"][0] = 4294967295
        model["model"]["supports"][0]["node"] = 4294967295
        result = engine.run(model)
        exported = result.to_arrays()
        self.assertEqual(exported.document, result.to_dict())
        self.assertEqual(exported.blocks[0].path, ("cases", 0))
        matrices = exported.blocks[0].matrices
        u = matrices["displacements"]
        nodes = result.data["cases"][0]["nodes"]
        self.assertEqual(u.row_ids, tuple(n["id"] for n in nodes))
        self.assertEqual(u.values, [n["displacements"] for n in nodes])
        self.assertEqual(u.shape, (2, 6))
        self.assertEqual(u.dofs, ("Ux", "Uy", "Uz", "Rx", "Ry", "Rz"))
        self.assertEqual(u.units, ("m", "m", "m", "rad", "rad", "rad"))
        self.assertEqual(matrices["reactions"].units, ("N",) * 3 + ("N*m",) * 3)
        ends = matrices["end_forces_local/frame2d.euler_elastic"]
        self.assertEqual(ends.row_ids, (7,))
        self.assertEqual(ends.dofs, ("i.Ux", "i.Uy", "i.Rz", "j.Ux", "j.Uy", "j.Rz"))
        self.assertEqual(ends.units, ("N", "N", "N*m") * 2)
        self.assertEqual(ends.values[0], result.data["cases"][0]["members"][0]["end_forces_local"])
        stations = result.data["cases"][0]["members"][0]["stations"]
        self.assertEqual(matrices["forces_local"].row_ids, (7,) * len(stations))
        self.assertEqual(matrices["forces_local"].row_indices, tuple(range(len(stations))))
        self.assertEqual(matrices["positions"].values, [[s["position"]] for s in stations])

    def test_list_exports_are_independent_of_document_snapshot_and_each_other(self):
        result = engine.run(SOURCE)
        snapshot = result.to_dict()
        original = result.to_json()
        exported = result.to_arrays()
        again = result.to_arrays()
        self.assertIsNot(exported, again)
        self.assertIsNot(exported.document, again.document)
        for block, other in zip(exported.blocks, again.blocks):
            self.assertIsNot(block, other)
            self.assertIsNot(block.matrices, other.matrices)
            for name, matrix in block.matrices.items():
                with self.subTest(path=block.path, matrix=name):
                    second = other.matrices[name]
                    expected = deepcopy(second.values)
                    self.assertIsNot(matrix, second)
                    self.assertIsNot(matrix.values, second.values)
                    for row, other_row in zip(matrix.values, second.values):
                        self.assertIsNot(row, other_row)
                        row[:] = [987] * len(row)
                    matrix.values.append([123] * matrix.shape[1])
                    self.assertEqual(second.values, expected)
        self.assertEqual(exported.document, snapshot)
        self.assertEqual(again.document, snapshot)
        self.assertEqual(result.to_dict(), snapshot)
        self.assertEqual(result.to_json(), original)
        self.assertEqual(result.to_arrays(), again)
        exported.document["cases"].clear()
        self.assertEqual(again.document, snapshot)
        self.assertEqual(result.to_dict(), snapshot)
        self.assertEqual(result.to_json(), original)
        again.blocks[0].matrices["displacements"].values[0][0] = -123
        self.assertEqual(result.to_arrays().blocks[0].matrices["displacements"].values,
                         [n["displacements"] for n in snapshot["cases"][0]["nodes"]])

    def test_boolean_values_are_rejected_as_numbers(self):
        document = engine.run(SOURCE).to_dict()
        for field in ("displacements", "reactions", "end_forces_local",
                      "forces_local", "position"):
            for bad in (True, False):
                value = deepcopy(document)
                case = value["cases"][0]
                if field in ("displacements", "reactions"):
                    case["nodes"][0][field][0] = bad
                elif field == "end_forces_local":
                    case["members"][0][field][0] = bad
                else:
                    station = case["members"][0]["stations"][0]
                    if field == "position":
                        station[field] = bad
                    else:
                        station[field][0] = bad
                # Probar la frontera tabular sin que el schema rechace bool antes.
                with self.subTest(field=field, bad=bad), self.assertRaisesRegex(
                        engine.InvalidInputError, "se requieren valores f64 finitos"):
                    export(value, numpy=False)

    def test_stale_and_failed_state_are_preserved(self):
        model = engine.Model.from_json(SOURCE)
        result = engine.run(model)
        model.data["model"]["nodes"][1]["coordinates"][0] = 5
        for access in (result.to_arrays, result.to_numpy):
            with self.assertRaises(engine.StaleResultsError):
                access()
        token = engine.CancelToken()
        token.cancel()
        with self.assertRaises(engine.Cancelled) as caught:
            engine.run(SOURCE, cancel=token)
        exported = caught.exception.results.to_arrays()
        self.assertEqual(exported.status, "cancelled")
        self.assertTrue(exported.partial)
        self.assertEqual(exported.blocks, ())
        self.assertTrue(exported.document["diagnostics"])

    def test_empty_rows_and_invalid_shapes_finiteness_and_identities(self):
        document = engine.run(SOURCE).to_dict()
        empty = deepcopy(document)
        empty["cases"][0]["nodes"] = []
        empty["cases"][0]["members"] = []
        tables = engine.Results.from_json(json.dumps(empty)).to_arrays().blocks[0].matrices
        for name, width in (("displacements", 6), ("reactions", 6),
                            ("forces_local", 6), ("positions", 1)):
            self.assertEqual(tables[name].shape, (0, width))
            self.assertEqual(tables[name].values, [])
            self.assertEqual(tables[name].row_ids, ())
        for field in ("displacements", "reactions"):
            for bad in ([0], [[0]] * 6, [], [0] * 7,
                        [float("nan")] * 6, [float("inf")] * 6, [-float("inf")] * 6):
                value = deepcopy(document)
                value["cases"][0]["nodes"][0][field] = bad
                with self.subTest(field=field, bad=bad), self.assertRaises(engine.InvalidInputError):
                    engine.Results.from_json(json.dumps(value)).to_arrays()
        for bad in ([0], [], [0] * 12):
            value = deepcopy(document)
            value["cases"][0]["members"][0]["end_forces_local"] = bad
            with self.assertRaises(engine.InvalidInputError):
                engine.Results.from_json(json.dumps(value)).to_arrays()
        for field in ("nodes", "members"):
            value = deepcopy(document)
            value["cases"][0][field][0]["id"] = 111
            with self.assertRaises(engine.InvalidInputError):
                engine.Results.from_json(json.dumps(value)).to_arrays()
            value = deepcopy(document)
            value["cases"][0][field].append(deepcopy(value["cases"][0][field][0]))
            with self.assertRaises(engine.InvalidInputError):
                engine.Results.from_json(json.dumps(value)).to_arrays()
        value = deepcopy(document)
        value["cases"][0]["nodes"][0]["displacements"][0] = 10**1000
        with self.assertRaises(engine.InvalidInputError):
            engine.Results.from_json(json.dumps(value)).to_arrays()

    def test_mixed_element_widths_and_unknown_labels(self):
        # Fixture de transporte importado; no representa una solución física.
        document = engine.run(SOURCE).to_dict()
        for mid, kind, width in ((21, "truss2d.linear", 4), (22, "truss3d.linear", 6),
                                  (23, "frame3d.euler_elastic", 12)):
            identity = deepcopy(document["members"][0])
            identity.update(id=mid, element=kind)
            document["members"].append(identity)
            response = deepcopy(document["cases"][0]["members"][0])
            response.update(id=mid, end_forces_local=list(range(width)))
            document["cases"][0]["members"].append(response)
        result = engine.Results.from_json(json.dumps(document)).to_arrays()
        matrices = result.blocks[0].matrices
        for mid, kind, width in ((7, "frame2d.euler_elastic", 6), (21, "truss2d.linear", 4),
                                 (22, "truss3d.linear", 6), (23, "frame3d.euler_elastic", 12)):
            self.assertEqual(matrices[f"end_forces_local/{kind}"].shape, (1, width))
            self.assertEqual(matrices[f"end_forces_local/{kind}"].row_ids, (mid,))
        document["members"][0]["element"] = "future.element"
        with self.assertRaises(engine.Unsupported):
            engine.Results.from_json(json.dumps(document)).to_arrays()


if __name__ == "__main__":
    unittest.main()
