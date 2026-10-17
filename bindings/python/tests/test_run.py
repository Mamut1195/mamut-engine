"""Integración sobre wheel: fachada Rust, errores, GIL y cancelación."""

import hashlib
import json
import math
import os
from pathlib import Path
import sys
import threading
import unittest

from mamut_engine import _native as engine, __version__

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "crates/engine-api/fixtures/input"
CANTILEVER = (FIXTURES / "valid/cantilever-linear-static.json").read_text()


class RunTest(unittest.TestCase):
    def test_cantilever_matches_rust_bytes_and_sha256(self):
        # La referencia se genera con cargo run, nunca mediante el binding.
        reference_path = os.environ.get("MAMUT_RUST_REFERENCE")
        self.assertTrue(reference_path, "Defina MAMUT_RUST_REFERENCE (véase README)")
        reference = Path(reference_path).read_bytes()
        self.assertEqual(engine.validate(CANTILEVER), {"diagnostics": []})
        for _ in range(3):
            actual = engine.run(CANTILEVER).encode()
            self.assertEqual(actual, reference)  # tolerancia de transporte: cero
            self.assertEqual(hashlib.sha256(actual).digest(), hashlib.sha256(reference).digest())
        document = json.loads(actual)
        self.assertEqual(document["status"], "ok")
        nodes = document["cases"][0]["nodes"]
        # EI y'' = -P(L-x) -> y(L)=-PL³/(3EI); equilibrio R=P, M=PL.
        # 1e-12 permite redondeo f64 del solve para este pequeño sistema bien condicionado.
        for actual, expected in [
            (nodes[1]["displacements"][1], -12 * 4**3 / (3 * 200 * 3)),
            (nodes[0]["reactions"][1], 12),
            (nodes[0]["reactions"][5], 48),
        ]:
            self.assertTrue(math.isclose(actual, expected, rel_tol=1e-12, abs_tol=1e-12))

    def test_validation_preserves_topology_diagnostics(self):
        data = json.loads(CANTILEVER)
        data["model"]["members"][0]["nodes"] = [1, 99]
        diagnostics = engine.validate(json.dumps(data))["diagnostics"]
        self.assertTrue(any(d["severity"] == "error" for d in diagnostics))
        with self.assertRaises(engine.InvalidInputError) as caught:
            engine.run(json.dumps(data))
        err = caught.exception
        first = next(d for d in diagnostics if d["severity"] == "error")
        self.assertEqual((err.code, err.entities, err.cause),
                         (first["code"], first["entities"], first["cause"]))
        self.assertTrue(err.entities)
        self.assertEqual(err.diagnostics, json.loads(err.document)["diagnostics"])
        self.assertEqual(json.loads(err.document)["cases"], [])

    def test_invalid_input_does_not_escape_ffi(self):
        bad_inputs = ["{", '{"schema":"engine-input/1","schema":"engine-input/1"}',
                      "NaN", '{"schema":"engine-input/9"}']
        for path in sorted((FIXTURES / "invalid").glob("*.json")):
            bad_inputs.append(path.read_text())
        for source in bad_inputs:
            for function in (engine.run, engine.validate):
                with self.subTest(function=function.__name__, source=source[:60]):
                    # validate retorna diagnósticos para defectos topológicos del dominio.
                    if function is engine.validate:
                        try:
                            result = function(source)
                        except engine.EngineError as err:
                            self.assertIsInstance(err.code, str)
                            self.assertIsInstance(err.entities, list)
                            self.assertIsInstance(err.cause, str)
                        else:
                            self.assertTrue(any(d["severity"] == "error" for d in result["diagnostics"]))
                    else:
                        with self.assertRaises(engine.EngineError) as caught:
                            function(source)
                        self.assertIn(json.loads(caught.exception.document)["status"],
                                      ["invalid", "unsupported"])
        # El proceso sigue vivo y el módulo vuelve a ejecutar un caso válido.
        self.assertEqual(json.loads(engine.run(CANTILEVER))["status"], "ok")

    def test_unstable_model_is_analysis_error(self):
        data = json.loads(CANTILEVER)
        data["model"]["supports"] = []
        with self.assertRaises(engine.AnalysisError) as caught:
            engine.run(json.dumps(data))
        self.assertEqual(caught.exception.code, "ENG-E-0006")
        self.assertEqual(json.loads(caught.exception.document)["status"], "unstable")

    def test_progress_is_ordered_and_deferred(self):
        events = []
        token = engine.CancelToken()

        def callback(event):
            events.append(event)
            token.cancel()  # Ya terminó el motor: no cambia el resultado.
            self.assertEqual(engine.version()["engine_version"], __version__)

        actual = engine.run(CANTILEVER, progress=callback, cancel=token)
        self.assertEqual(actual, engine.run(CANTILEVER))
        self.assertTrue(events)
        self.assertTrue(token.is_cancelled)
        self.assertEqual(events[0]["stage"], "parse")
        self.assertTrue(all(e["operation"] == "run" for e in events))
        completed = [e["completed"] for e in events]
        self.assertEqual(completed, sorted(completed))
        self.assertEqual(events[-1]["completed"], events[-1]["total"])

    def test_callback_errors_propagate_and_bad_callback_is_rejected(self):
        def fail(event):
            raise LookupError("callback")

        with self.assertRaisesRegex(LookupError, "callback"):
            engine.run(CANTILEVER, progress=fail)
        with self.assertRaises(TypeError):
            engine.run(CANTILEVER, progress=123)
        with self.assertRaises(TypeError):
            engine.run(CANTILEVER, cancel=lambda: True)
        # Un fallo del motor tiene prioridad sobre un fallo del callback diferido.
        with self.assertRaises(engine.InvalidInputError):
            engine.run('{"schema":"unknown"}', progress=fail)

    def test_pre_cancelled_token_is_permanent(self):
        token = engine.CancelToken()
        self.assertFalse(token.is_cancelled)
        token.cancel()
        token.cancel()
        for _ in range(2):
            events = []
            with self.assertRaises(engine.CancelledError) as caught:
                engine.run(CANTILEVER, events.append, token)
            self.assertEqual(caught.exception.code, "API-I-CANCELLED")
            document = json.loads(caught.exception.document)
            self.assertEqual(document["status"], "cancelled")
            self.assertEqual(document["cases"], [])
            self.assertEqual(events, [])

    def test_run_releases_gil_for_cancellation_from_python_thread(self):
        # Espacio JSON válido bajo MAX_INPUT_BYTES (8 MiB): mantiene trabajo
        # en Rust durante el parseo mientras el trabajador adquiere el GIL.
        # No cambia el caso ni depende de un timeout de cancelación.
        source = " " * (4 * 1024 * 1024) + CANTILEVER
        token = engine.CancelToken()
        gate = threading.Event()
        ready = threading.Event()

        def worker():
            ready.set()
            gate.wait()
            token.cancel()

        thread = threading.Thread(target=worker)
        thread.start()
        ready.wait()
        previous = sys.getswitchinterval()
        try:
            # Evita un cambio periódico de hilo Python entre gate.set() y run().
            # El trabajador puede cancelar al liberarse el GIL dentro del binding.
            sys.setswitchinterval(30)
            self.assertFalse(token.is_cancelled)
            gate.set()
            with self.assertRaises(engine.CancelledError) as caught:
                engine.run(source, cancel=token)
            self.assertEqual(json.loads(caught.exception.document)["cases"], [])
        finally:
            sys.setswitchinterval(previous)
            gate.set()
            thread.join(timeout=5)
        self.assertFalse(thread.is_alive())
        self.assertTrue(token.is_cancelled)

    def test_advanced_methods_and_warning_remain_api_results(self):
        for name in ("p-delta", "modal-single-dof", "response-spectrum", "beam-combination"):
            with self.subTest(name=name):
                source = (FIXTURES / "valid" / f"{name}.json").read_text()
                events = []
                result = json.loads(engine.run(source, progress=events.append))
                self.assertIn(result["status"], ["ok", "warning"])
                self.assertTrue(events)


if __name__ == "__main__":
    unittest.main()
