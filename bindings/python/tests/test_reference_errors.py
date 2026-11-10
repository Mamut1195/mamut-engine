"""ENG-E-0003 names the missing entity on the public exception."""
from __future__ import annotations

import json
from typing import Any
import unittest

import mamut_engine as engine


def _document() -> dict[str, Any]:
    """Small linearStatic model. The error text does not depend on its units."""
    return {
        "schema": "engine-input/1",
        "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]},
                {"id": 2, "dim": "D2", "coordinates": [2.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]},
            ],
            "members": [{"id": 7, "nodes": [1, 2], "section": "beam", "law": "elastic"}],
            "sections": [{"id": "beam", "properties": [{"name": "A", "value": 0.01}]}],
            "laws": [{"name": "elastic", "parameters": [{"name": "E", "value": 2.0e11}]}],
            "supports": [{"id": "base", "node": 1, "restrained": ["Ux", "Uy", "Rz"]}],
            "masses": [],
        },
        "cases": [{"name": "dead", "loads": [{"node": 2, "values": [{"dof": "Uy", "value": -10.0}]}]}],
        "combinations": [{"id": "uls", "terms": [{"case": "dead", "factor": 1.0}]}],
        "analyses": [{"method": "linearStatic"}],
        "options": {},
    }


class ReferenceErrorTest(unittest.TestCase):
    def _run(self, document: dict[str, Any]) -> engine.InvalidInputError:
        with self.assertRaises(engine.InvalidInputError) as caught:
            engine.run(json.dumps(document))
        error = caught.exception
        if not isinstance(error, engine.InvalidInputError):
            self.fail("the facade must raise InvalidInputError")
        return error

    def test_missing_member_node_is_named_on_the_exception(self) -> None:
        document = _document()
        document["model"]["nodes"] = [document["model"]["nodes"][0]]
        document["model"]["members"][0]["nodes"] = [1, 11]
        document["cases"][0]["loads"][0]["node"] = 1
        error = self._run(document)
        self.assertEqual(error.code, "ENG-E-0003")
        self.assertEqual(error.entity_kind, "node")
        self.assertEqual(error.missing_id, 11)
        self.assertEqual(error.referenced_by, "members[0].nodes[1]")
        self.assertEqual(error.suggestions, ["1"])
        message = str(error)
        self.assertIn("node", message)
        self.assertIn("11", message)
        self.assertIn("member 7", message)
        self.assertIn("end 1", message)

    def test_missing_section_is_named(self) -> None:
        document = _document()
        document["model"]["members"][0]["section"] = "beem"
        error = self._run(document)
        self.assertEqual(error.entity_kind, "section")
        self.assertEqual(error.missing_id, "beem")
        self.assertEqual(error.referenced_by, "members[0].section")
        self.assertEqual(error.suggestions, ["beam"])
        self.assertIn("section", str(error))
        self.assertIn("member 7", str(error))

    def test_missing_law_is_named(self) -> None:
        document = _document()
        document["model"]["members"][0]["law"] = "elastix"
        error = self._run(document)
        self.assertEqual(error.entity_kind, "law")
        self.assertEqual(error.missing_id, "elastix")
        self.assertEqual(error.referenced_by, "members[0].law")
        self.assertEqual(error.suggestions, ["elastic"])
        self.assertIn("law", str(error))
        self.assertIn("member 7", str(error))

    def test_missing_case_is_named(self) -> None:
        document = _document()
        document["combinations"][0]["terms"][0]["case"] = "deaf"
        error = self._run(document)
        self.assertEqual(error.entity_kind, "case")
        self.assertEqual(error.missing_id, "deaf")
        self.assertEqual(error.referenced_by, "combinations[0].terms[0].case")
        self.assertEqual(error.suggestions, ["dead"])
        message = str(error)
        self.assertIn("case", message)
        self.assertIn("deaf", message)
        self.assertIn("uls", message)
        self.assertIn("term 0", message)

    def test_missing_support_node_is_named(self) -> None:
        document = _document()
        document["model"]["supports"][0]["node"] = 12
        error = self._run(document)
        self.assertEqual(error.entity_kind, "node")
        self.assertEqual(error.missing_id, 12)
        self.assertEqual(error.referenced_by, "supports[0].node")
        self.assertEqual(error.suggestions, ["1", "2"])
        message = str(error)
        self.assertIn("node", message)
        self.assertIn("support base", message)

    def test_missing_load_node_is_named(self) -> None:
        document = _document()
        document["cases"][0]["loads"][0]["node"] = 12
        error = self._run(document)
        self.assertEqual(error.entity_kind, "node")
        self.assertEqual(error.missing_id, 12)
        self.assertEqual(error.referenced_by, "cases[0].loads[0].node")
        self.assertEqual(error.suggestions, ["1", "2"])
        message = str(error)
        self.assertIn("node", message)
        self.assertIn("case dead", message)
        self.assertIn("load 0", message)

    def test_spanish_catalog_explanation_remains_and_the_cause_stays_english(self) -> None:
        explanation = engine.explain("ENG-E-0003", lang="es")
        self.assertIn("entidad", explanation)
        document = _document()
        document["model"]["members"][0]["section"] = "beem"
        error = self._run(document)
        self.assertTrue(error.cause.startswith("The referenced entity does not exist."))
        self.assertIn("Missing section", error.cause)


if __name__ == "__main__":
    unittest.main()
