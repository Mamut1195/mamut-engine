"""Paridad de prepare y run con temperatura uniforme.

Barra recta, homogénea y elástica, con el extremo i impedido y el extremo j
libre a lo largo de la barra. La deformación axial se parte en parte mecánica
y parte térmica, ε = ε_mecánica + α·ΔT. Sin fuerza exterior el axil es nulo,
así que el alargamiento del extremo libre es δ = α·ΔT·L.

Con α = 0.5 1/K, ΔT = 3 K y L = 2 m, δ = 0.5·3·2 = 3.0 m. Ese producto es el
Ux esperado del nudo libre. No procede de otro programa.

Una fuerza axial F = 16 N en el nudo libre, con E = 8 Pa y A = 4 m², añade
δ_mec = F·L/(E·A) = 16·2/(8·4) = 1.0 m. Fuerza y temperatura en el mismo caso
dan Ux = 4.0 m. prepare no acepta combinaciones; ese trabajo sigue en run.
"""
import unittest

import mamut_engine as engine

ALPHA_PER_KELVIN = 0.5
DELTA_T_KELVIN = 3.0
LENGTH_M = 2.0
YOUNG_PA = 8.0
AREA_M2 = 4.0
AXIAL_FORCE_N = 16.0
# Cota del solver en binary64. El producto α·ΔT·L de estos datos es exacto.
ABSOLUTE_TOLERANCE_M = 1e-12


def free_elongation_m():
    return ALPHA_PER_KELVIN * DELTA_T_KELVIN * LENGTH_M


def mechanical_elongation_m():
    return AXIAL_FORCE_N * LENGTH_M / (YOUNG_PA * AREA_M2)


def bar(cases, combinations):
    return {
        "schema": "engine-input/1",
        "units": "SI",
        "options": {},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [LENGTH_M, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
            ],
            "members": [{
                "id": 1,
                "nodes": [1, 2],
                "section": "s",
                "law": "elastic",
                "element": "truss2d.linear",
            }],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": AREA_M2}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": YOUNG_PA},
                {"name": "rho", "value": 0.0},
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]},
            ],
            "masses": [],
        },
        "cases": cases,
        "combinations": combinations,
        "analyses": [{"method": "linearStatic"}],
    }


def thermal(name, loads):
    return {
        "name": name,
        "loads": loads,
        "temperature_loads": [{
            "member": 1,
            "alpha_per_kelvin": ALPHA_PER_KELVIN,
            "delta_t_kelvin": DELTA_T_KELVIN,
        }],
    }


def force_at_free_end():
    return [{"node": 2, "values": [{"dof": "Ux", "value": AXIAL_FORCE_N}]}]


def ux_of(case, node_id):
    for node in case["nodes"]:
        if node["id"] == node_id:
            return node["displacements"][0]
    raise AssertionError(node_id)


class PrepareTemperatureTest(unittest.TestCase):
    def assert_within(self, got, want, label):
        delta = abs(got - want)
        self.assertLessEqual(
            delta,
            ABSOLUTE_TOLERANCE_M,
            f"{label}: got {got}, want {want}, |delta| {delta}, "
            f"tolerance {ABSOLUTE_TOLERANCE_M}",
        )

    def assert_prepare_matches_run(self, model):
        results = engine.run(model)
        prepared = engine.prepare(model).displacements()
        self.assertEqual(prepared, engine.prepare(model).displacements())
        self.assertEqual(len(prepared), len(results.data["cases"]))
        for case, expected in zip(prepared, results.data["cases"], strict=True):
            self.assertEqual(case["case"], expected["case"])
            self.assertEqual(len(case["nodes"]), len(expected["nodes"]))
            for node, expected_node in zip(case["nodes"], expected["nodes"], strict=True):
                self.assertEqual(node["id"], expected_node["id"])
                self.assertEqual(len(node["displacements"]), 6)
                self.assertEqual(len(expected_node["displacements"]), 6)
                for component, (got, want) in enumerate(
                    zip(node["displacements"], expected_node["displacements"], strict=True)
                ):
                    self.assert_within(
                        got,
                        want,
                        f"case {case['case']} node {node['id']} component {component}",
                    )

    def test_free_bar_elongation_matches_the_closed_form_and_run(self):
        elongation = free_elongation_m()
        self.assertEqual(elongation, 3.0)
        model = bar([thermal("T", [])], [])
        results = engine.run(model)
        self.assertEqual(results.status, "ok")
        self.assert_within(ux_of(results.data["cases"][0], 2), elongation, "run Ux")
        self.assert_prepare_matches_run(model)
        prepared = engine.prepare(model).displacements()
        self.assert_within(ux_of(prepared[0], 2), elongation, "prepare Ux")
        for node in prepared[0]["nodes"]:
            for component, value in enumerate(node["displacements"]):
                if node["id"] == 2 and component == 0:
                    continue
                self.assert_within(value, 0.0, f"node {node['id']} component {component}")

    def test_mechanical_load_plus_temperature_matches_run(self):
        elongation = free_elongation_m()
        mechanical = mechanical_elongation_m()
        self.assertEqual(mechanical, 1.0)
        combined = mechanical + elongation
        self.assertEqual(combined, 4.0)
        model = bar(
            [
                {"name": "F", "loads": force_at_free_end()},
                thermal("FT", force_at_free_end()),
            ],
            [],
        )
        results = engine.run(model)
        self.assertEqual(results.status, "ok")
        self.assert_within(ux_of(results.data["cases"][0], 2), mechanical, "run mechanical Ux")
        self.assert_within(ux_of(results.data["cases"][1], 2), combined, "run combined Ux")
        self.assert_prepare_matches_run(model)
        prepared = engine.prepare(model).displacements()
        by_name = {case["case"]: case for case in prepared}
        self.assert_within(ux_of(by_name["F"], 2), mechanical, "prepare mechanical Ux")
        self.assert_within(ux_of(by_name["FT"], 2), combined, "prepare combined Ux")

    def test_combination_stays_on_run(self):
        scaled = 2.0 * free_elongation_m()
        self.assertEqual(scaled, 6.0)
        model = bar(
            [thermal("T", [])],
            [{"id": "C", "terms": [{"case": "T", "factor": 2.0}]}],
        )
        with self.assertRaises(engine.InvalidInputError) as caught:
            engine.prepare(model)
        self.assertIn("combinations", caught.exception.cause)
        results = engine.run(model)
        self.assertIn(results.status, ("ok", "warning"))
        combined = results.data["combinations"][0]["results"]
        self.assert_within(ux_of(combined, 2), scaled, "run combination Ux")
