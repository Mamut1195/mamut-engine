"""The v1 generator is deterministic and stays aligned with the schemas.

Denominator: every key of ``$defs``, plus the root document. Anonymous
``oneOf`` branches and inline objects are expansions of those definitions.
"""

from __future__ import annotations

import ast
import dataclasses
import importlib.util
import json
import re
import sys
import tokenize
import unittest
from pathlib import Path
from typing import Literal, get_args, get_origin, get_type_hints

REPOSITORY = Path(__file__).resolve().parents[3]
GENERATOR_PATH = REPOSITORY / "bindings" / "python" / "tools" / "gen_v1.py"
MODULE_PATH = REPOSITORY / "bindings" / "python" / "python" / "mamut_engine" / "v1.py"
SCHEMAS = (
    ("engine-input/1", ("engine-input", "1.json")),
    ("results/1", ("results", "1.json")),
)
_WORD_SI = re.compile(r"\bSI\b")
_WORD_FIX = re.compile(r"\bfix\b", re.IGNORECASE)


def load_generator():
    spec = importlib.util.spec_from_file_location("mamut_gen_v1", GENERATOR_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("no se pudo cargar el generador")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def load_generated():
    spec = importlib.util.spec_from_file_location(
        "mamut_engine_v1_under_test",
        MODULE_PATH,
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("no se pudo cargar el módulo generado")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def read_schema(relative_parts: tuple[str, ...]) -> dict[str, object]:
    path = REPOSITORY.joinpath("schemas", *relative_parts)
    document = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(document, dict):
        raise AssertionError("el esquema no es un objeto")
    return document


class GeneratorContractTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.generator = load_generator()
        cls.generated = load_generated()
        cls.source = MODULE_PATH.read_text(encoding="utf-8")

    def test_two_renders_have_identical_bytes(self) -> None:
        first = self.generator.render().encode("utf-8")
        second = self.generator.render().encode("utf-8")
        self.assertEqual(first, second)
        self.assertTrue(first.endswith(b"\n"))
        self.assertFalse(first.endswith(b"\n\n"))

    def test_committed_module_matches_the_schemas(self) -> None:
        rendered = self.generator.render().encode("utf-8")
        self.assertEqual(MODULE_PATH.read_bytes(), rendered)

    def test_definition_denominator_is_covered(self) -> None:
        covered = {entry.schema_name: entry for entry in self.generator.coverage()}
        self.assertEqual(tuple(covered), ("engine-input/1", "results/1"))
        for schema_name, relative in SCHEMAS:
            document = read_schema(relative)
            definitions = document["$defs"]
            self.assertIsInstance(definitions, dict)
            assert isinstance(definitions, dict)
            keys = tuple(definitions)
            entry = covered[schema_name]
            self.assertEqual(tuple(name for name, _symbol in entry.definition_symbols), keys)
            self.assertEqual(len(entry.definition_symbols), len(keys))
            root = getattr(self.generated, entry.root_symbol)
            self._assert_object(root, _root_schema(document))
            for key, symbol in entry.definition_symbols:
                body = definitions[key]
                self.assertIsInstance(body, dict)
                assert isinstance(body, dict)
                self._assert_definition(symbol, body)

    def test_units_constant_is_the_schema_constant_only(self) -> None:
        document = read_schema(("engine-input", "1.json"))
        properties = _root_schema(document)["properties"]
        self.assertIsInstance(properties, dict)
        assert isinstance(properties, dict)
        units = properties["units"]
        self.assertIsInstance(units, dict)
        assert isinstance(units, dict)
        self.assertEqual(units["const"], "SI")
        hints = get_type_hints(self.generated.EngineInputDocument)
        self.assertEqual(hints["units"], Literal["SI"])
        self.assertEqual(get_args(hints["units"]), ("SI",))

    def test_analysis_is_discriminated_by_method(self) -> None:
        document = read_schema(("engine-input", "1.json"))
        definitions = document["$defs"]
        assert isinstance(definitions, dict)
        analysis = definitions["analysis"]
        assert isinstance(analysis, dict)
        branches = analysis["oneOf"]
        assert isinstance(branches, list)
        classes = get_args(self.generated.Analysis)
        self.assertEqual(len(classes), len(branches))
        for cls, branch in zip(classes, branches, strict=True):
            self.assertIsInstance(branch, dict)
            assert isinstance(branch, dict)
            method = branch["properties"]
            assert isinstance(method, dict)
            constant = method["method"]
            assert isinstance(constant, dict)
            value = constant["const"]
            self.assertIsInstance(value, str)
            self.assertEqual(get_type_hints(cls)["method"], Literal[value])
            self._assert_object(cls, branch)

    def test_dataclasses_are_frozen_slots_and_keyword_only(self) -> None:
        found = False
        for value in vars(self.generated).values():
            if not isinstance(value, type) or not hasattr(value, "__dataclass_fields__"):
                continue
            found = True
            params = value.__dataclass_params__
            self.assertTrue(params.frozen, value)
            self.assertTrue(params.slots, value)
            self.assertTrue(params.kw_only, value)
            for field in dataclasses.fields(value):
                self.assertTrue(field.kw_only, field)
        self.assertTrue(found)

    def test_string_enums_reject_unknown_values(self) -> None:
        for value in vars(self.generated).values():
            if value is self.generated.StrEnum:
                continue
            if not isinstance(value, type) or not issubclass(value, self.generated.StrEnum):
                continue
            with self.assertRaises(ValueError):
                value("__not_a_schema_value__")

    def test_prose_does_not_assume_a_unit_system(self) -> None:
        tree = ast.parse(self.source)
        for node in ast.walk(tree):
            if isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef)):
                prose = ast.get_docstring(node)
                if prose is not None:
                    self.assertIsNone(_WORD_SI.search(prose), prose)
        with MODULE_PATH.open("rb") as stream:
            for token in tokenize.tokenize(stream.readline):
                if token.type == tokenize.COMMENT:
                    self.assertIsNone(_WORD_SI.search(token.string), token.string)

    def test_generated_text_has_no_machine_path_or_forbidden_word(self) -> None:
        for token in ("/home/", "/agent/", "/tmp/", "/usr/", "/workspace/"):
            self.assertNotIn(token, self.source)
            self.assertNotIn(token, GENERATOR_PATH.read_text(encoding="utf-8"))
        for line in self.source.splitlines():
            self.assertIsNone(_WORD_FIX.search(line), line)
        for line in GENERATOR_PATH.read_text(encoding="utf-8").splitlines():
            self.assertIsNone(_WORD_FIX.search(line), line)

    def test_installed_wheel_matches_and_carries_typing_markers(self) -> None:
        import mamut_engine
        import mamut_engine.v1 as installed

        package = Path(mamut_engine.__file__).resolve().parent
        self.assertTrue((package / "py.typed").is_file())
        self.assertTrue((package / "__init__.pyi").is_file())
        self.assertTrue((package / "_native.pyi").is_file())
        installed_bytes = Path(installed.__file__).read_bytes()
        self.assertEqual(installed_bytes, self.generator.render().encode("utf-8"))

    def _assert_definition(self, symbol: str, body: dict[str, object]) -> None:
        value = getattr(self.generated, symbol)
        if "enum" in body:
            values = body["enum"]
            self.assertIsInstance(values, list)
            assert isinstance(values, list)
            self.assertEqual([member.value for member in value], values)
            return
        if "oneOf" in body and "type" not in body:
            branches = body["oneOf"]
            self.assertIsInstance(branches, list)
            assert isinstance(branches, list)
            classes = get_args(value)
            self.assertEqual(len(classes), len(branches))
            for cls, branch in zip(classes, branches, strict=True):
                self.assertIsInstance(branch, dict)
                assert isinstance(branch, dict)
                if branch.get("type") == "object":
                    self._assert_object(cls, branch)
            return
        if body.get("type") == "object":
            self._assert_object(value, body)
            return
        if body.get("type") == "integer":
            self.assertIs(value, int)
            return
        if body.get("type") == "array":
            self.assertIs(get_origin(value), list)
            return
        self.fail(f"definición sin comprobación: {symbol}")

    def _assert_object(self, cls: type, body: dict[str, object]) -> None:
        properties = body["properties"]
        self.assertIsInstance(properties, dict)
        assert isinstance(properties, dict)
        required_value = body.get("required", [])
        self.assertIsInstance(required_value, list)
        assert isinstance(required_value, list)
        required = set(required_value)
        names = list(properties)
        expected = [name for name in names if name in required]
        expected.extend(name for name in names if name not in required)
        fields = dataclasses.fields(cls)
        self.assertEqual([field.name for field in fields], expected)
        for field in fields:
            property_schema = properties[field.name]
            self.assertIsInstance(property_schema, dict)
            assert isinstance(property_schema, dict)
            if field.name in required:
                self.assertIs(field.default, dataclasses.MISSING, field)
                continue
            if "default" in property_schema:
                self.assertEqual(field.default, property_schema["default"], field)
                continue
            self.assertIsNone(field.default, field)


def _root_schema(document: dict[str, object]) -> dict[str, object]:
    return {key: value for key, value in document.items() if key not in {"$defs", "$id", "$schema"}}


if __name__ == "__main__":
    unittest.main()
