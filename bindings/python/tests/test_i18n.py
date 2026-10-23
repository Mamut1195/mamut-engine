"""Shared catalog parity and explicit language selection across the Python API."""
from copy import deepcopy
import json
from pathlib import Path
import unittest
from unittest.mock import patch

import mamut_engine as engine
from mamut_engine import _native, facade

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "crates/engine-api/fixtures/input"
INVALID = (FIXTURES / "invalid/missing-reference.json").read_text()
VALID = (FIXTURES / "valid/cantilever-linear-static.json").read_text()
CATALOGS = {lang: json.loads((ROOT / f"crates/engine-api/messages/{lang}.json").read_text())
            for lang in ("en", "es", "pt")}


def localized(diagnostics, lang):
    """Build expected translations from the same embedded catalog source."""
    result = deepcopy(diagnostics)
    for diagnostic in result:
        key = f"diagnostic.{diagnostic['code']}"
        if key in CATALOGS["en"] and diagnostic["cause"] == CATALOGS["en"][key]:
            diagnostic["cause"] = CATALOGS[lang][key]
    return result


class I18nTest(unittest.TestCase):
    def test_real_invalid_diagnostics_and_run_exceptions_match_catalog(self):
        english = engine.validate(INVALID)["diagnostics"]
        self.assertTrue(english)
        with self.assertRaises(engine.EngineError) as caught:
            engine.run(INVALID)
        baseline = json.loads(caught.exception.document)
        for lang in ("en", "es", "pt"):
            with self.subTest(lang=lang):
                expected = localized(english, lang)
                self.assertEqual(engine.Engine.validate(INVALID, lang=lang)["diagnostics"], expected)
                self.assertEqual(_native.validate(INVALID, lang)["diagnostics"], expected)
                if lang != "en":
                    self.assertNotEqual(expected, english)
                for run in (lambda: engine.Engine.run(INVALID, lang=lang),
                            lambda: _native.run(INVALID, lang=lang)):
                    with self.assertRaises(Exception) as caught:
                        run()
                    error = caught.exception
                    self.assertEqual(error.diagnostics, expected)
                    self.assertEqual(error.cause, next(d["cause"] for d in expected if d["severity"] == "error"))
                    expected_document = deepcopy(baseline)
                    expected_document["diagnostics"] = expected
                    self.assertEqual(json.loads(error.document), expected_document)

    def test_valid_results_preserve_all_data_and_default_english_bytes(self):
        sources = ((VALID, False),
                   ((ROOT / "crates/engine-api/tests/fixtures/determinism/cantilever.json").read_text(), True))
        for source, warning in sources:
            model = json.loads(source)
            if warning:
                model["combinations"] = [{"id": "scaled", "terms": [{"case": "D", "factor": 2.0}]}]
            baseline = engine.run(model)
            if warning:
                self.assertTrue(baseline.diagnostics)
            self.assertEqual(engine.run(model, lang="en").to_json(), baseline.to_json())
            self.assertEqual(_native.run(json.dumps(model), lang=None), baseline.to_json())
            for lang in ("es", "pt"):
                expected = baseline.to_dict()
                expected["diagnostics"] = localized(expected["diagnostics"], lang)
                actual = engine.run(model, lang=lang)
                self.assertEqual(actual.to_dict(), expected)
                self.assertEqual(actual.to_json(), _native.run(json.dumps(model), lang=lang))
        self.assertEqual(_native.validate(INVALID), _native.validate(INVALID, None))
        self.assertEqual(engine.validate(INVALID), engine.validate(INVALID, lang="en"))

    def test_explain_uses_shared_catalog(self):
        for code in ("ENG-E-0004", "API-E-CONSOLIDATION-COST"):
            self.assertEqual(_native.explain(code), _native.explain(code, None))
            self.assertEqual(engine.explain(code), engine.explain(code, lang="en"))
            for lang in ("en", "es", "pt"):
                expected = CATALOGS[lang][f"explain.{code}"]
                self.assertEqual(engine.explain(code, lang=lang), expected)
                self.assertEqual(_native.explain(code, lang), expected)

    def test_facade_rejects_language_before_ffi(self):
        for lang in (None, 1, [], {}, "", "ES", "pt-BR", "es_EC", "en "):
            for operation, argument in ((engine.explain, "ENG-E-0004"),
                                        (engine.validate, INVALID), (engine.run, VALID)):
                with self.subTest(lang=lang, operation=operation.__name__):
                    with patch.object(facade, "_call") as native:
                        with self.assertRaises(engine.InvalidInputError):
                            operation(argument, lang=lang)
                        native.assert_not_called()

    def test_native_invalid_language_is_api_input_error(self):
        for lang in ("", "ES", "pt-BR", " en"):
            for operation, argument in ((_native.explain, "ENG-E-0004"),
                                        (_native.validate, INVALID), (_native.run, VALID)):
                with self.subTest(lang=lang, operation=operation.__name__):
                    with self.assertRaises(_native.InvalidInputError) as caught:
                        operation(argument, lang=lang)
                    self.assertEqual(caught.exception.code, "API-E-INPUT")
                    self.assertIn("Unsupported language", caught.exception.cause)


if __name__ == "__main__":
    unittest.main()
