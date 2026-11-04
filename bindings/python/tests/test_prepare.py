"""prepare() matches run() displacements on an in-repo linear model."""
import unittest

import mamut_engine as engine

MODEL = {
    "schema": "engine-input/1",
    "units": "SI",
    "options": {},
    "model": {
        "axes": "Z-up",
        "nodes": [
            {"id": 10, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
            {"id": 4, "dim": "D2", "coordinates": [4.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
            {"id": 7, "dim": "D2", "coordinates": [2.0, 3.0, 0.0], "dofs": ["Ux", "Uy"]},
        ],
        "members": [
            {"id": 1, "nodes": [10, 4], "section": "s", "law": "steel", "element": "truss2d.linear"},
            {"id": 2, "nodes": [10, 7], "section": "s", "law": "steel", "element": "truss2d.linear"},
            {"id": 3, "nodes": [4, 7], "section": "s", "law": "steel", "element": "truss2d.linear"},
        ],
        "sections": [{"id": "s", "properties": [{"name": "A", "value": 0.01}]}],
        "laws": [{"name": "steel", "parameters": [
            {"name": "E", "value": 2.0e11}, {"name": "rho", "value": 0.0}]}],
        "supports": [
            {"id": "a", "node": 10, "restrained": ["Ux", "Uy"]},
            {"id": "b", "node": 4, "restrained": ["Ux", "Uy"]},
        ],
        "masses": [],
    },
    "cases": [
        {"name": "P", "loads": [{"node": 7, "values": [
            {"dof": "Ux", "value": 1000.0}, {"dof": "Uy", "value": -500.0}]}]},
        {"name": "Q", "loads": [{"node": 7, "values": [{"dof": "Uy", "value": -200.0}]}]},
    ],
    "combinations": [],
    "analyses": [{"method": "linearStatic"}],
}


class PrepareTest(unittest.TestCase):
    def test_displacements_match_run_and_repeat(self):
        results = engine.run(MODEL)
        prepared = engine.prepare(MODEL)
        first = prepared.displacements()
        self.assertEqual(first, prepared.displacements())
        self.assertEqual([node["id"] for node in first[0]["nodes"]], [4, 7, 10])
        for case, expected in zip(first, results.data["cases"], strict=True):
            self.assertEqual(case["case"], expected["case"])
            for node, expected_node in zip(case["nodes"], expected["nodes"], strict=True):
                self.assertEqual(node["id"], expected_node["id"])
                self.assertEqual(node["displacements"], expected_node["displacements"])

    def test_combinations_use_run(self):
        model = {
            **MODEL,
            "combinations": [{"id": "uls", "terms": [{"case": "P", "factor": 1.4}]}],
        }
        with self.assertRaises(engine.InvalidInputError) as caught:
            engine.prepare(model)
        self.assertIn("combinations", caught.exception.cause)

    def test_capabilities_lists_prepare(self):
        self.assertIn("prepare", engine.capabilities()["operations"])
