"""Contrato público de la wheel instalada; solo biblioteca estándar."""

from importlib.metadata import version as distribution_version
import unittest

import mamut_engine as engine
from mamut_engine import _native


class FacadeTest(unittest.TestCase):
    def test_installed_version(self):
        version = engine.version()
        self.assertEqual(version["engine_version"], distribution_version("mamut-engine"))
        self.assertEqual(engine.__version__, version["engine_version"])
        self.assertEqual(version["contracts"], ["engine-input/1", "results/1"])
        self.assertEqual(_native.version(), version)
        self.assertEqual(
            set(name for name in dir(_native) if not name.startswith("_")),
            set(engine.__all__) - {"__version__"},
        )

    def test_capabilities_and_schema(self):
        caps = engine.capabilities()
        self.assertTrue(caps["run"])
        self.assertTrue(caps["methods"]["linearStatic"]["run"])
        for name in ("engine-input/1", "results/1"):
            schema = engine.schema(name)
            self.assertIsInstance(schema, dict)
            self.assertIn("$schema", schema)
            self.assertIn("properties", schema)
        self.assertIsInstance(engine.explain("ENG-E-0006"), str)

    def test_lookup_errors_have_context(self):
        for function, argument in [(engine.schema, "missing"), (engine.explain, "missing")]:
            with self.subTest(function=function.__name__):
                with self.assertRaises(engine.InvalidInputError) as caught:
                    function(argument)
                self.assertIsInstance(caught.exception, engine.EngineError)
                self.assertEqual(caught.exception.code, "API-E-INPUT")
                self.assertEqual(caught.exception.entities, [])
                self.assertIn(argument, caught.exception.cause)


if __name__ == "__main__":
    unittest.main()
