"""buckling.linear y pDelta rechazan temperatura en la referencia de Kg.

Kg usa el axil mecánico y no resta αΔT. El código es ENG-E-0013, capacidad
no soportada. Con temperatura en la referencia de pandeo no se publica un
factor; el factor incorrecto de ese modelo era 300.
"""
import json
import unittest

import mamut_engine as engine

INCORRECT_FACTOR = 300.0


def publishes_number(value, target):
    if isinstance(value, bool) or value is None:
        return False
    if isinstance(value, (int, float)):
        return value == target
    if isinstance(value, list):
        return any(publishes_number(item, target) for item in value)
    if isinstance(value, dict):
        return any(publishes_number(item, target) for item in value.values())
    return False


def assert_rejected(test, model):
    with test.assertRaises(engine.Unsupported) as caught:
        engine.run(model)
    error = caught.exception
    test.assertEqual(error.code, "ENG-E-0013")
    document = json.loads(error.document)
    test.assertEqual(document["status"], "unsupported")
    test.assertTrue(any(item["code"] == "ENG-E-0013" for item in document["diagnostics"]))
    test.assertEqual(document["cases"], [])
    test.assertEqual(document["modes"], [])
    test.assertEqual(document["combinations"], [])
    test.assertEqual(document["envelopes"], [])
    test.assertIsNone(document["convergence"])
    test.assertNotIn("buckling", document)
    test.assertFalse(publishes_number(document, INCORRECT_FACTOR))
    test.assertTrue(error.results.partial)
    test.assertEqual(error.results.data["cases"], [])


def probe_buckling():
    return {
        "schema": "engine-input/1",
        "units": "SI",
        "options": {"allow_experimental": True},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [1.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 3, "dim": "D2", "coordinates": [4.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 4, "dim": "D2", "coordinates": [1.0, 1.0, 0.0], "dofs": ["Ux", "Uy"]},
            ],
            "members": [
                {"id": 1, "nodes": [1, 2], "section": "s", "law": "elastic", "element": "truss2d.linear"},
                {"id": 2, "nodes": [2, 3], "section": "s", "law": "elastic", "element": "truss2d.linear"},
                {"id": 3, "nodes": [2, 4], "section": "s", "law": "elastic", "element": "truss2d.linear"},
            ],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 4.0}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": 8.0}, {"name": "rho", "value": 0.0},
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "c", "node": 3, "restrained": ["Ux", "Uy"]},
                {"id": "d", "node": 4, "restrained": ["Ux", "Uy"]},
            ],
            "masses": [],
        },
        "cases": [{
            "name": "R",
            "loads": [],
            "temperature_loads": [{
                "member": 2, "alpha_per_kelvin": 0.5, "delta_t_kelvin": 0.01,
            }],
        }],
        "combinations": [],
        "analyses": [{
            "method": "buckling.linear",
            "options": {"mode_count": 1, "reference": {"case": "R", "factor": 1.0}},
        }],
    }


def pdelta_gravity():
    return {
        "schema": "engine-input/1",
        "units": "SI",
        "options": {},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [2.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
            ],
            "members": [{
                "id": 1, "nodes": [1, 2], "section": "s", "law": "elastic",
                "element": "truss2d.linear",
            }],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 4.0}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": 8.0}, {"name": "rho", "value": 0.0},
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]},
            ],
            "masses": [],
        },
        "cases": [
            {
                "name": "G",
                "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -1.0}]}],
                "temperature_loads": [{
                    "member": 1, "alpha_per_kelvin": 0.5, "delta_t_kelvin": 0.01,
                }],
            },
            {"name": "H", "loads": [{"node": 2, "values": [{"dof": "Uy", "value": 1.0}]}]},
        ],
        "combinations": [],
        "analyses": [{
            "method": "pDelta",
            "options": {
                "gravity": {"case": "G", "factor": 1.0},
                "lateral": {"case": "H", "factor": 1.0},
                "tolerance": 1e-8,
                "max_iterations": 20,
                "displacement_scale": 1.0,
                "rotation_scale": 1.0,
            },
        }],
    }


class TemperatureGeometricTest(unittest.TestCase):
    def test_buckling_reference_temperature_publishes_no_factor(self):
        assert_rejected(self, probe_buckling())

    def test_pdelta_gravity_temperature_publishes_no_factor(self):
        assert_rejected(self, pdelta_gravity())
