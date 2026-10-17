"""Consistencia offline de stubs instalados y DTO frente a los contratos Rust.

Complementa get_type_hints cuando no hay mypy/pyright. Comprueba exports,
firmas PyO3 y campos requeridos/opcionales; no sustituye un checker completo.
"""
import ast
import inspect
from pathlib import Path
import typing
import unittest

import mamut_engine as engine
from mamut_engine import _native, types


class TypingContractTest(unittest.TestCase):
    def test_public_stub_matches_runtime_exports_and_reexports(self):
        package = Path(engine.__file__).parent
        runtime = ast.parse((package / "__init__.py").read_text())
        stub = ast.parse((package / "__init__.pyi").read_text())

        def exports(tree):
            result = {}
            for node in tree.body:
                if isinstance(node, ast.ImportFrom):
                    for alias in node.names:
                        result[alias.asname or alias.name] = (node.module, alias.name)
            return result

        self.assertEqual(exports(stub), exports(runtime))
        self.assertEqual(set(exports(stub)), set(engine.__all__) - {"__version__"})
        declared = next(n for n in stub.body if isinstance(n, ast.Assign)
                        and any(isinstance(t, ast.Name) and t.id == "__all__" for t in n.targets))
        self.assertEqual(ast.literal_eval(declared.value), engine.__all__)
        version = next(n for n in stub.body if isinstance(n, ast.AnnAssign)
                       and n.target.id == "__version__")
        self.assertEqual(ast.unparse(version.annotation), "str")
        self.assertIsInstance(engine.__version__, str)

    def test_native_stub_matches_actual_function_signatures(self):
        stub = ast.parse((Path(engine.__file__).parent / "_native.pyi").read_text())
        functions = {n.name: n for n in stub.body if isinstance(n, ast.FunctionDef)}
        classes = {n.name: n for n in stub.body if isinstance(n, ast.ClassDef)}
        expected = {name for name in vars(_native) if not name.startswith("_")}
        self.assertEqual(set(functions) | set(classes), expected)
        for name, node in functions.items():
            with self.subTest(function=name):
                signature = inspect.signature(getattr(_native, name))
                self.assertEqual(list(signature.parameters),
                                 [p.arg for p in node.args.args])
                self.assertFalse(node.args.posonlyargs or node.args.kwonlyargs)
                self.assertTrue(all(p.kind == inspect.Parameter.POSITIONAL_OR_KEYWORD
                                    for p in signature.parameters.values()))
                self.assertTrue(all(p.annotation is not None for p in node.args.args))
                self.assertIsNotNone(node.returns)
                defaults = [p.default for p in signature.parameters.values()
                            if p.default is not inspect.Parameter.empty]
                self.assertEqual(defaults, [ast.literal_eval(d) for d in node.args.defaults])
        for name in classes:
            self.assertTrue(inspect.isclass(getattr(_native, name)))
        for name in ("EngineError", "InvalidInputError", "CancelledError",
                     "AnalysisError", "InternalEngineError"):
            self.assertTrue(issubclass(getattr(_native, name), Exception))

    def assert_fields(self, dto, schema):
        self.assertEqual(set(typing.get_type_hints(dto)), set(schema["properties"]), dto)
        self.assertEqual(dto.__required_keys__, set(schema.get("required", [])), dto)
        self.assertEqual(dto.__optional_keys__,
                         set(schema["properties"]) - set(schema.get("required", [])), dto)

    def test_typed_documents_follow_required_and_optional_schema_fields(self):
        for name, prefix, root in (("engine-input/1", "Input", types.ModelDocument),
                                   ("results/1", "Result", types.ResultsDocument)):
            spec = engine.schema(name)
            self.assert_fields(root, spec)
            for key, definition in spec["$defs"].items():
                if "properties" in definition:
                    with self.subTest(schema=name, definition=key):
                        self.assert_fields(getattr(types, prefix + key[0].upper() + key[1:]),
                                           definition)
        input_spec = engine.schema("engine-input/1")
        self.assert_fields(types.InputOptions, input_spec["properties"]["options"])
        for branch, dto in zip(input_spec["$defs"]["analysis"]["oneOf"],
                               (types.LinearStaticAnalysis, types.PDeltaAnalysis,
                                types.ModalAnalysis, types.SpectrumAnalysis)):
            self.assert_fields(dto, branch)
        methods = engine.schema("method-descriptor/1")
        self.assert_fields(types.MethodDescriptor, methods)
        for key, dto in (("workflow", types.WorkflowMethod),
                         ("quantity", types.MethodQuantity),
                         ("parameter", types.MethodParameter)):
            self.assert_fields(dto, methods["$defs"][key])
        self.assert_fields(types.MethodDimensions,
                           methods["$defs"]["workflow"]["properties"]["dimensions"])
        caps = engine.schema("capabilities/1")
        self.assert_fields(types.Capabilities, caps)
        for key, dto in (("schemas", types.SchemaCapability),
                         ("methods", types.MethodCapability)):
            self.assert_fields(dto, caps["properties"][key]["additionalProperties"])
        self.assert_fields(types.CombinationCapability, caps["properties"]["combinations"])


if __name__ == "__main__":
    unittest.main()
