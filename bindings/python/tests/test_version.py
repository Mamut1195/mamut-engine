"""Smoke test del paquete instalado; solo requiere la biblioteca estándar."""

from importlib.metadata import version as distribution_version
import unittest

import mamut_engine
from mamut_engine import _native


class VersionTest(unittest.TestCase):
    def test_installed_version(self):
        engine_version = mamut_engine.version()
        self.assertIsInstance(engine_version, str)
        self.assertEqual(engine_version, "0.1.0")  # engine-core/Cargo.toml
        self.assertEqual(mamut_engine.__version__, engine_version)
        self.assertEqual(_native.version(), engine_version)
        self.assertEqual(distribution_version("mamut-engine"), engine_version)
        self.assertEqual(
            [name for name in dir(_native) if not name.startswith("_")],
            ["version"],
        )


if __name__ == "__main__":
    unittest.main()
