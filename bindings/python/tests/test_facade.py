"""ENG-PY-003: API pública instalada, referencias Rust y contratos sin red."""
from copy import deepcopy
from dataclasses import is_dataclass
import hashlib
import inspect
import json
import os
from pathlib import Path
import shutil
import subprocess
import typing
import unittest
from unittest.mock import patch

import mamut_engine as engine
from mamut_engine import facade, types

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "crates/engine-api/fixtures/input/valid"
SOURCE = (FIXTURES / "cantilever-linear-static.json").read_text()


class PublicFacadeTest(unittest.TestCase):
    def test_all_fixtures_match_independent_rust_and_roundtrip(self):
        references = os.environ.get("MAMUT_RUST_REFERENCES")
        self.assertTrue(references, "Defina MAMUT_RUST_REFERENCES; véase README")
        paths = sorted(FIXTURES.glob("*.json")) + sorted(
            (ROOT / "crates/engine-api/tests/fixtures/determinism").glob("*.json"))
        for path in paths:
            with self.subTest(fixture=path.name):
                model = engine.Model.from_json(path.read_text())
                self.assertTrue(is_dataclass(model))
                self.assertEqual(model.to_dict(), json.loads(path.read_text()))
                self.assertEqual(engine.Model.from_json(model.to_json()).to_dict(), model.data)
                events = []
                result = engine.Engine().run(model, progress=events.append)
                reference = (Path(references) / path.name).read_bytes()
                self.assertEqual(result.to_json().encode(), reference)
                self.assertEqual(hashlib.sha256(reference).digest(),
                                 hashlib.sha256(result.to_json().encode()).digest())
                self.assertEqual(result.input_sha256, model.input_sha256)
                self.assertEqual(result.version["engine_version"], engine.__version__)
                self.assertTrue(set(engine.version()["contracts"]) <=
                                set(result.version["contracts"]))
                self.assertEqual(result.data["nodes"], model.data["model"]["nodes"])
                self.assertEqual([m["id"] for m in result.data["members"]],
                                 [m["id"] for m in (model.data["model"]["members"]
                                     + model.data["model"].get("springs", []))])
                self.assertFalse(result.partial)
                self.assertTrue(events)
                self.assertEqual(engine.Results.from_json(result.to_json(), model=model).to_json(),
                                 result.to_json())
                self.assertEqual(engine.validate(model), {"diagnostics": []})

    def test_stale_checks_mutable_model_and_raw_dict(self):
        for source in (engine.Model.from_json(SOURCE), json.loads(SOURCE)):
            result = engine.run(source)
            data = source.data if isinstance(source, engine.Model) else source
            copy = result.to_dict()
            copy["cases"].clear()
            self.assertTrue(result.data["cases"])
            # Solo cambia formato/orden de claves: no cambia la identidad canónica.
            data["model"]["nodes"][0]["coordinates"][0] = -0.0
            # JCS iguala 0, -0.0 y 0.0 para la identidad del input.
            self.assertFalse(result.is_stale)
            data["cases"][0]["loads"][0]["values"][0]["value"] *= 2
            self.assertTrue(result.is_stale)
            for access in (result.to_json, result.to_dict, lambda: result.status,
                           lambda: result.version, lambda: result.diagnostics):
                with self.assertRaises(engine.StaleResultsError) as caught:
                    access()
                self.assertEqual(caught.exception.code, "PY-E-STALE")
            data["units"] = None
            self.assertTrue(result.is_stale)

    def test_roundtrip_preserves_signed_zero_floats_ids_and_array_order(self):
        data = json.loads(SOURCE)
        data["model"]["nodes"].reverse()
        data["model"]["nodes"][1]["coordinates"][0] = -0.0
        model = engine.Model.from_dict(data)
        result = engine.run(model)
        self.assertEqual([n["id"] for n in result.data["nodes"]], [2, 1])
        model.data = dict(reversed(list(model.data.items())))
        model.data["model"]["nodes"][1]["coordinates"][0] = 0.0
        self.assertFalse(result.is_stale)
        model.data["model"]["nodes"].reverse()
        self.assertTrue(result.is_stale)

    def test_preflight_never_passes_invalid_arguments_to_rust(self):
        bad = []
        for value in (float("nan"), float("inf"), -float("inf"), True, "1", None):
            data = json.loads(SOURCE)
            data["model"]["nodes"][0]["coordinates"][0] = value
            bad.append(data)
        for value in (-1, 2**32, True, 1.0):
            data = json.loads(SOURCE)
            data["model"]["nodes"][0]["id"] = value
            bad.append(data)
        for value in ([1], [1, 2, 3], "12"):
            data = json.loads(SOURCE)
            data["model"]["members"][0]["nodes"] = value
            bad.append(data)
        data = json.loads(SOURCE)
        data["model"]["nodes"] *= 129
        bad.append(data)
        data = json.loads(SOURCE)
        del data["units"]
        bad.append(data)
        for key, value in (("units", "kN-m"), ("axes", "Y-up"), ("axes", None)):
            data = json.loads(SOURCE)
            target = data if key == "units" else data["model"]
            target[key] = value
            bad.append(data)
        data = json.loads(SOURCE)
        del data["model"]["axes"]
        bad.append(data)
        cycle = {}; cycle["self"] = cycle
        bad += [cycle, 42, b"{}", "NaN", "{", '{"a":1,"a":2}',
                SOURCE.replace('200', '1e400'), SOURCE + " " * (8 * 1024 * 1024)]
        with patch.object(facade._native, "run") as native_run, \
             patch.object(facade._native, "validate") as native_validate, \
             patch.object(facade._native, "input_sha256") as fingerprint:
            for source in bad:
                for function in (engine.run, engine.validate):
                    with self.subTest(value=repr(source)[:80], function=function.__name__):
                        with self.assertRaises(engine.InvalidInputError) as caught:
                            function(source)
                        self.assertEqual(caught.exception.severity, "error")
                        self.assertTrue(caught.exception.details)
            native_run.assert_not_called()
            native_validate.assert_not_called()
            fingerprint.assert_not_called()
        for kwargs in ({"progress": 1}, {"cancel": object()}):
            with self.assertRaises(engine.InvalidInputError):
                engine.run(SOURCE, **kwargs)

    def test_jcs_safe_integer_boundaries_and_stale(self):
        for n in (2**53 - 1, -(2**53 - 1)):
            data = json.loads(SOURCE)
            # Carga en apoyo: verifica la frontera sin grandes desplazamientos.
            data["cases"][0]["loads"][0]["node"] = 1
            data["cases"][0]["loads"][0]["values"][0]["value"] = n
            for source in (data, json.dumps(data)):
                with self.subTest(n=n, source=type(source).__name__):
                    model = (engine.Model.from_dict(source) if isinstance(source, dict)
                             else engine.Model.from_json(source))
                    self.assertEqual(engine.validate(model), {"diagnostics": []})
                    result = engine.run(model)
                    self.assertEqual(result.input_sha256, model.input_sha256)
                    for unsafe in (2**53, -(2**53), 2**63 - 1, 2**63):
                        model.data["cases"][0]["loads"][0]["values"][0]["value"] = unsafe
                        self.assertTrue(result.is_stale)
                        with self.assertRaises(engine.StaleResultsError):
                            result.to_json()
                    model.data["cases"][0]["loads"][0]["values"][0]["value"] = n
                    self.assertFalse(result.is_stale)

    def test_unsafe_integers_are_rejected_before_ffi_for_dict_and_json(self):
        with patch.object(facade._native, "run") as native_run, \
             patch.object(facade._native, "validate") as native_validate, \
             patch.object(facade._native, "input_sha256") as fingerprint:
            for n in (2**53, -(2**53), 2**53 + 1, 2**63 - 1, 2**63,
                      -(2**63), 2**64 - 1, 10**100, -(10**100)):
                data = json.loads(SOURCE)
                data["cases"][0]["loads"][0]["values"][0]["value"] = n
                for source in (data, json.dumps(data)):
                    constructor = (engine.Model.from_dict if isinstance(source, dict)
                                   else engine.Model.from_json)
                    for function in (constructor, engine.run, engine.validate):
                        with self.subTest(n=n, function=function.__name__,
                                          source=type(source).__name__):
                            with self.assertRaises(engine.InvalidInputError) as caught:
                                function(source)
                            self.assertIn("9007199254740991", caught.exception.cause)
                            self.assertEqual(caught.exception.details["path"],
                                             "$.cases[0].loads[0].values[0].value")
            native_run.assert_not_called()
            native_validate.assert_not_called()
            fingerprint.assert_not_called()

    def test_bool_is_never_an_integer_id_or_counter(self):
        for value in (True, False):
            for path, source in (("id", SOURCE), ("mode_count",
                                 (FIXTURES / "modal-single-dof.json").read_text()),
                                 ("max_iterations", (FIXTURES / "p-delta.json").read_text())):
                data = json.loads(source)
                target = (data["model"]["nodes"][0] if path == "id"
                          else data["analyses"][0]["options"])
                target[path] = value
                for model in (data, json.dumps(data)):
                    with self.subTest(path=path, value=value, source=type(model).__name__):
                        with self.assertRaises(engine.InvalidInputError):
                            engine.validate(model)

    def test_optional_fields_and_results_extensions_survive(self):
        # Campos opcionales publicados después del modelo estático inicial.
        data = json.loads(SOURCE)
        data["model"]["prescribed"] = []
        data["model"]["diaphragms"] = []
        data["model"]["members"][0]["element"] = "frame2d.euler_elastic"
        data["cases"][0]["member_loads"] = []
        model = engine.Model.from_dict(data)
        self.assertEqual(engine.Model.from_json(model.to_json()).data, data)
        result = engine.run(model)
        spatial = engine.Model.from_json(
            (ROOT / "crates/engine-api/tests/fixtures/determinism/frame3d-inclined-static.json")
            .read_text())
        self.assertTrue(spatial.data["model"]["members"][0]["orientation"])
        self.assertEqual(engine.Model.from_json(spatial.to_json()).data, spatial.data)
        engine.run(spatial)
        document = result.to_dict()
        # Los límites de entrada no recortan las extensiones de results/1.
        document["future_optional"] = {"identity": ["a", 2], "large": [0] * 16385,
                                       "integer": 2**63}
        document["cases"][0]["future_optional"] = 3
        source = json.dumps(document)
        imported = engine.Results.from_json(source)
        self.assertEqual(imported.to_json(), source)
        self.assertEqual(imported.to_dict(), document)
        # La validación usa el schema instalado, sin whitelist duplicada.
        newer = deepcopy(facade._spec("engine-input/1"))
        newer["properties"]["optional_added"] = {"type": "string"}
        data["optional_added"] = "kept"
        with patch.object(facade, "_spec", return_value=newer):
            self.assertEqual(json.loads(facade._input(data)), data)

    def test_major_contract_and_missing_defaults_are_rejected(self):
        for contract in ("engine-input/2", "engine-input/1.1", "other/1"):
            data = json.loads(SOURCE)
            data["schema"] = contract
            with self.assertRaises(engine.Unsupported) as caught:
                engine.validate(data)
            self.assertEqual(caught.exception.code, "ENG-E-0009")
        result = engine.run(SOURCE).to_dict()
        result["schema"] = "results/2"
        with self.assertRaises(engine.Unsupported):
            engine.Results.from_json(json.dumps(result))
        data = json.loads((FIXTURES / "modal-single-dof.json").read_text())
        del data["analyses"][0]["options"]["mass_kind"]
        with self.assertRaises(engine.InvalidInputError):
            engine.run(data)
        self.assertEqual(engine.schema("engine-input", 1), engine.schema("engine-input/1"))

    def test_jcs_identity_and_stable_pdelta_match_current_rust(self):
        model = engine.Model.from_json(SOURCE)
        result = engine.run(model)
        # Regresión: SHA-256 del JSON compacto no es la huella JCS.
        model.data["model"]["nodes"][1]["coordinates"][0] = 4.0
        self.assertFalse(result.is_stale)
        self.assertEqual(result.input_sha256, model.input_sha256)
        self.assertNotEqual(model.input_sha256,
                            hashlib.sha256(model.to_json().encode()).hexdigest())
        pdelta = engine.Model.from_json((FIXTURES / "p-delta.json").read_text())
        self.assertEqual(pdelta.data["options"], {})
        self.assertEqual(engine.run(pdelta).status, "ok")
        descriptor = next(d for d in engine.capabilities()["method_descriptors"]
                          if d["workflow"]["method_id"] == "pDelta")
        self.assertEqual(descriptor["status"], "Stable")

    def test_errors_keep_diagnostics_and_are_never_success(self):
        data = json.loads(SOURCE)
        data["model"]["supports"] = []
        unsupported = json.loads(SOURCE)
        unsupported["model"]["members"][0]["element"] = "unavailable"
        no_convergence = json.loads((FIXTURES / "p-delta.json").read_text())
        no_convergence["analyses"][0]["options"]["max_iterations"] = 1
        token = engine.CancelToken(); token.cancel()
        for model, kwargs, cls, code in (
            (data, {}, engine.Unstable, "ENG-E-0006"),
            (unsupported, {}, engine.Unsupported, "ENG-E-0013"),
            (no_convergence, {}, engine.NotConverged, "ENG-E-0007"),
            (SOURCE, {"cancel": token}, engine.Cancelled, "API-I-CANCELLED"),
        ):
            with self.subTest(error=cls.__name__):
                with self.assertRaises(cls) as caught:
                    engine.run(model, **kwargs)
                error = caught.exception
                self.assertIsInstance(error, engine.EngineError)
                self.assertEqual(error.code, code)
                self.assertEqual(error.severity, "error")
                self.assertTrue(error.results.partial)
                self.assertEqual(error.results.data["cases"], [])
                self.assertEqual(error.results.to_json(), error.document)
                self.assertEqual(error.diagnostics, error.results.diagnostics)
                self.assertEqual(error.details["code"], code)
                original = json.loads(error.document)
                first = next(d for d in original["diagnostics"] if d["severity"] == "error")
                self.assertEqual((error.code, error.entities, error.severity, error.cause,
                                  error.details),
                                 (first["code"], first["entities"], first["severity"],
                                  first["cause"], first))
                if cls is engine.NotConverged:
                    self.assertIsNotNone(error.results.data["convergence"])
                    self.assertIn("residual_convergence_ratio",
                                  error.results.data["convergence"]["iterations"][0])
        invalid = json.loads(SOURCE)
        invalid["model"]["members"][0]["nodes"] = [1, 99]
        report = engine.validate(invalid)
        self.assertTrue(report["diagnostics"])
        with self.assertRaises(engine.InvalidInputError) as caught:
            engine.run(invalid)
        self.assertEqual(caught.exception.entities, report["diagnostics"][0]["entities"])

    def test_callbacks_are_python_and_mutation_marks_results_stale(self):
        model = engine.Model.from_json(SOURCE)
        def callback(event):
            self.assertIsInstance(event, dict)
            model.data["options"]["changed"] = True
        result = engine.run(model, progress=callback)
        self.assertTrue(result.is_stale)
        def fail(event):
            raise LookupError("callback")
        with self.assertRaises(LookupError):
            engine.run(SOURCE, progress=fail)
        unstable = json.loads(SOURCE)
        unstable["model"]["supports"] = []
        with self.assertRaises(engine.Unstable):
            engine.run(unstable, progress=fail)

    def test_imported_provenance_and_snapshot_cannot_be_silently_rebound(self):
        result = engine.run(SOURCE)
        other = json.loads(SOURCE)
        other["model"]["nodes"][1]["coordinates"][0] *= 2
        with self.assertRaises(engine.StaleResultsError):
            engine.Results.from_json(result.to_json(), model=other)
        imported = engine.Results.from_json(result.to_json())
        self.assertEqual(imported.input_sha256, result.input_sha256)
        with self.assertRaises(engine.StaleResultsError):
            imported.assert_current(other)

    def test_old_results_and_invalid_imported_shapes(self):
        result = engine.run((FIXTURES / "p-delta.json").read_text()).to_dict()
        del result["target"]
        del result["git_commit"]
        for iteration in result["convergence"]["iterations"]:
            del iteration["residual_convergence_ratio"]
        self.assertEqual(engine.Results.from_json(json.dumps(result)).data, result)
        for key, value in (("input_sha256", "g" * 64), ("status", "success"),
                           ("git_commit", "invalid")):
            bad = deepcopy(result)
            bad[key] = value
            with self.subTest(key=key), self.assertRaises(engine.InvalidInputError):
                engine.Results.from_json(json.dumps(bad))
        bad = deepcopy(result)
        bad["cases"][0]["nodes"][0]["displacements"] = [0.0]
        with self.assertRaises(engine.InvalidInputError):
            engine.Results.from_json(json.dumps(bad))

    def test_secondary_result_parser_cannot_mask_engine_error(self):
        unstable = json.loads(SOURCE)
        unstable["model"]["supports"] = []
        with patch.object(engine.Results, "from_json", side_effect=engine.Unsupported("future")):
            with self.assertRaises(engine.Unstable) as caught:
                engine.run(unstable)
        self.assertEqual(caught.exception.code, "ENG-E-0006")
        self.assertIsNone(caught.exception.results)
        self.assertEqual(caught.exception.diagnostics,
                         json.loads(caught.exception.document)["diagnostics"])

    def test_type_annotations_are_complete_and_resolve(self):
        for name in engine.__all__:
            obj = getattr(engine, name)
            if inspect.isfunction(obj):
                hints = typing.get_type_hints(obj)
                self.assertIn("return", hints, name)
                for parameter in inspect.signature(obj).parameters:
                    self.assertIn(parameter, hints, (name, parameter))
            elif inspect.isclass(obj) and obj.__module__.startswith("mamut_engine"):
                typing.get_type_hints(obj)
                for method_name, method in obj.__dict__.items():
                    if isinstance(method, (classmethod, staticmethod)):
                        method = method.__func__
                    if isinstance(method, property):
                        method = method.fget
                    if inspect.isfunction(method):
                        # dataclasses genera métodos auxiliares sin anotaciones
                        # (incluido __replace__ en Python 3.13).
                        original = inspect.unwrap(method)
                        if (original.__module__ == "dataclasses"
                                or original.__code__.co_filename == "<string>"):
                            continue
                        hints = typing.get_type_hints(method)
                        self.assertIn("return", hints, (name, method_name))
                        for parameter in inspect.signature(method).parameters:
                            if parameter not in ("self", "cls"):
                                self.assertIn(parameter, hints, (name, method_name, parameter))
        for obj in vars(types).values():
            if typing.is_typeddict(obj):
                self.assertTrue(typing.get_type_hints(obj))
        self.assertTrue((Path(engine.__file__).parent / "py.typed").is_file())

    def test_static_checker_when_available_offline(self):
        checker = shutil.which("mypy") or shutil.which("pyright")
        if checker is None:
            # La alternativa offline obligatoria se ejecuta en test_typing.py.
            return
        package = str(Path(engine.__file__).parent)
        args = [checker, package]
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        completed = subprocess.run(args, capture_output=True, text=True, env=env)
        self.assertEqual(completed.returncode, 0, completed.stdout + completed.stderr)


if __name__ == "__main__":
    unittest.main()
