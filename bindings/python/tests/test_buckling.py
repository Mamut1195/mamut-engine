"""buckling.linear a través de la fachada. La página de Timoshenko-Gere sigue desconocida."""
import unittest

import mamut_engine as engine
from mamut_engine.arrays import export

CITATION = (
    "DESCONOCIDO. Timoshenko-Gere no fue consultado: sin pagina ni tabla. "
    "UNKNOWN. Derivacion propia en provenance/ENG-ANL-006.md."
)


def column():
    return {
        "schema": "engine-input/1",
        "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0, 0, 0], "dofs": ["Ux", "Uy", "Rz"]},
                {"id": 2, "dim": "D2", "coordinates": [4, 0, 0], "dofs": ["Ux", "Uy", "Rz"]},
            ],
            "members": [{"id": 1, "nodes": [1, 2], "section": "s", "law": "elastic"}],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 2}, {"name": "I", "value": 3}]}],
            "laws": [{"name": "elastic", "parameters": [{"name": "E", "value": 200}, {"name": "rho", "value": 0}]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]},
            ],
            "masses": [],
        },
        "cases": [{"name": "N", "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -1}]}]}],
        "combinations": [],
        "analyses": [{
            "method": "buckling.linear",
            "options": {"mode_count": 1, "reference": {"case": "N", "factor": 1}},
        }],
        "options": {"allow_experimental": True},
    }


class BucklingFacadeTest(unittest.TestCase):
    def test_page_citation_stays_unknown(self):
        self.assertIn("DESCONOCIDO", CITATION)
        self.assertIn("UNKNOWN", CITATION)

    def test_one_element_pinned_column_matches_the_library_factor(self):
        self.assertEqual(
            engine.capabilities()["methods"]["buckling.linear"],
            {"engine": True, "run": True},
        )
        result = engine.run(column())
        data = result.data
        self.assertEqual(data["modes"], [])
        self.assertEqual(data["buckling"]["rigid_mode_count"], 0)
        self.assertEqual(data["buckling"]["modes"][0]["normalization"], "negative_kg")
        factor = data["buckling"]["modes"][0]["eigenvalue"]
        self.assertLessEqual(abs(factor - 450.0), 1e-8, CITATION)
        blocks = export(data, numpy=False)
        self.assertIn(("buckling", "modes", 0, "shape"), [block.path for block in blocks.blocks])


if __name__ == "__main__":
    unittest.main()
