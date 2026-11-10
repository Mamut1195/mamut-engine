"""Temperatura permitida en pDelta y buckling.linear, y validate.

El alargamiento del caso lateral es δ = α·ΔT·L. Con α = 0.5 1/K, ΔT = 3 K
y L = 2 m, δ = 0.5·3·2 = 3.0 m. Kg de una cercha alineada no entra en el
grado axial, así que la gravedad no cambia ese Ux.

El pandeo arriostrado da λ = E·A·L / (Lv·P). Con E = 8 Pa, A = 4 m²,
L = 2 m, Lv = 1 m y P = 16 N, λ = 4. Una temperatura fuera de la referencia
no cambia ese factor.

validate devuelve diagnósticos y no un código de salida. ENG-E-0013 es el
código ya usado para una capacidad no soportada.
"""
import unittest

import mamut_engine as engine

ALPHA_PER_KELVIN = 0.5
DELTA_T_KELVIN = 3.0
LENGTH_M = 2.0
BRACE_M = 1.0
YOUNG_PA = 8.0
AREA_M2 = 4.0
COMPRESSION_N = 16.0
ABSOLUTE_TOLERANCE = 1e-9


def thermal_elongation_m():
    return ALPHA_PER_KELVIN * DELTA_T_KELVIN * LENGTH_M


def buckling_factor():
    return YOUNG_PA * AREA_M2 * LENGTH_M / (BRACE_M * COMPRESSION_N)


def temperature(member):
    return [{
        "member": member,
        "alpha_per_kelvin": ALPHA_PER_KELVIN,
        "delta_t_kelvin": DELTA_T_KELVIN,
    }]


def pdelta(gravity_temperature, lateral_temperature):
    gravity = {
        "name": "G",
        "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -COMPRESSION_N}]}],
    }
    lateral = {"name": "H", "loads": []}
    if gravity_temperature:
        gravity["temperature_loads"] = temperature(1)
    if lateral_temperature:
        lateral["temperature_loads"] = temperature(1)
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
                "id": 1, "nodes": [1, 2], "section": "s", "law": "elastic",
                "element": "truss2d.linear",
            }],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": AREA_M2}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": YOUNG_PA}, {"name": "rho", "value": 0.0},
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]},
            ],
            "masses": [],
        },
        "cases": [gravity, lateral],
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


def buckling(reference_temperature, other_temperature):
    reference = {
        "name": "R",
        "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -COMPRESSION_N}]}],
    }
    cases = [reference]
    if reference_temperature:
        reference["temperature_loads"] = temperature(1)
    if other_temperature:
        cases.append({
            "name": "T",
            "loads": [],
            "temperature_loads": temperature(2),
        })
    return {
        "schema": "engine-input/1",
        "units": "SI",
        "options": {"allow_experimental": True},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [LENGTH_M, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 3, "dim": "D2", "coordinates": [LENGTH_M, BRACE_M, 0.0], "dofs": ["Ux", "Uy"]},
            ],
            "members": [
                {"id": 1, "nodes": [1, 2], "section": "s", "law": "elastic", "element": "truss2d.linear"},
                {"id": 2, "nodes": [2, 3], "section": "s", "law": "elastic", "element": "truss2d.linear"},
            ],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": AREA_M2}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": YOUNG_PA}, {"name": "rho", "value": 0.0},
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "c", "node": 3, "restrained": ["Ux", "Uy"]},
            ],
            "masses": [],
        },
        "cases": cases,
        "combinations": [],
        "analyses": [{
            "method": "buckling.linear",
            "options": {"mode_count": 1, "reference": {"case": "R", "factor": 1.0}},
        }],
    }


class TemperaturePermittedTest(unittest.TestCase):
    def assert_within(self, got, want, label):
        delta = abs(got - want)
        self.assertLessEqual(
            delta,
            ABSOLUTE_TOLERANCE,
            f"{label}: got {got}, want {want}, |delta| {delta}",
        )

    def assert_validate(self, model, code):
        diagnostics = engine.validate(model)["diagnostics"]
        if code is None:
            self.assertFalse(any(item["severity"] == "error" for item in diagnostics), diagnostics)
        else:
            self.assertTrue(
                any(item["code"] == code and item["severity"] == "error" for item in diagnostics),
                diagnostics,
            )

    def test_pdelta_lateral_temperature_elongates_by_alpha_delta_t_length(self):
        elongation = thermal_elongation_m()
        self.assertEqual(elongation, 3.0)
        model = pdelta(False, True)
        self.assert_validate(model, None)
        result = engine.run(model)
        self.assertIn(result.status, ("ok", "warning"))
        node = next(item for item in result.data["cases"][0]["nodes"] if item["id"] == 2)
        self.assert_within(node["displacements"][0], elongation, "lateral Ux")
        for component, value in enumerate(node["displacements"][1:], start=1):
            self.assert_within(value, 0.0, f"component {component}")

    def test_buckling_temperature_outside_the_reference_keeps_the_factor(self):
        expected = buckling_factor()
        self.assertEqual(expected, 4.0)
        plain = buckling(False, False)
        heated = buckling(False, True)
        self.assert_validate(plain, None)
        self.assert_validate(heated, None)
        plain_factor = engine.run(plain).data["buckling"]["modes"][0]["eigenvalue"]
        heated_factor = engine.run(heated).data["buckling"]["modes"][0]["eigenvalue"]
        self.assert_within(plain_factor, expected, "reference factor")
        self.assert_within(heated_factor, plain_factor, "factor with temperature off the reference")

    def test_validate_rejects_reference_and_gravity_temperature(self):
        self.assert_validate(pdelta(True, False), "ENG-E-0013")
        self.assert_validate(buckling(True, False), "ENG-E-0013")


if __name__ == "__main__":
    unittest.main()
