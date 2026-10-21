"""ENG-PY-005: contratos de experimento sobre la wheel; pytest y unittest offline."""
from copy import deepcopy
import builtins
import hashlib
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
from tempfile import TemporaryDirectory, gettempdir
import time
import unittest
from unittest.mock import call, patch

import mamut_engine as engine
from mamut_engine import _native, experiments as ex
from mamut_engine import _experiment_workflows as workflows

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "crates/engine-api/fixtures/input/valid"
BASE = json.loads((FIXTURES / "cantilever-linear-static.json").read_text())
LOAD = ["cases", 0, "loads", 0, "values", 0, "value"]
MODULUS = ["model", "laws", 0, "parameters", 0, "value"]
CLI = os.environ.get("MAMUT_EXPERIMENT_CLI")


def profile():
    return {"schema": "mamut.experiment/1", "name": "voladizo", "version": 1, "seed": 42,
            "parameters": {"P": {"path": LOAD, "values": [-12, -24]}},
            "metrics": {"uy": {"path": ["cases", 0, "nodes", 1, "displacements", 1],
                                "unit": "m"}}}


class ExperimentTest(unittest.TestCase):
    def test_cartesian_order_and_originals_are_independent(self):
        data, spec = deepcopy(BASE), profile()
        spec["parameters"]["E"] = {"path": MODULUS, "values": [200, 400]}
        before, settings = deepcopy(data), deepcopy(spec)
        model = engine.Model.from_dict(data)
        rows = ex.sweep(model, spec)
        self.assertEqual(len(rows), 4)
        self.assertTrue(all(r.status == "ok" and r.error is None for r in rows))
        self.assertEqual([r.parameters for r in rows], [
            {"E": 200, "P": -12}, {"E": 200, "P": -24},
            {"E": 400, "P": -12}, {"E": 400, "P": -24}])
        self.assertEqual(data, before)
        self.assertEqual(spec, settings)
        data["cases"].clear()
        spec["metrics"].clear()
        self.assertEqual(rows[0].result.status, "ok")
        self.assertEqual(rows[0].provenance["profile"], settings)
        self.assertEqual(rows[0].provenance["units"], "SI")
        self.assertEqual(rows[0].provenance["engine_version"], engine.__version__)
        self.assertEqual(rows[0].provenance["git_commit"], rows[0].result.data["git_commit"])

    def test_replay_and_fingerprints_are_exact(self):
        a, b = ex.sweep(BASE, profile()), ex.sweep(json.dumps(BASE), profile())
        self.assertEqual(ex.comparison_table(a), ex.comparison_table(b))
        for row in a:
            self.assertEqual(row.input_sha256, row.result.input_sha256)
            self.assertEqual(row.result_sha256,
                             hashlib.sha256(row.result.to_json().encode()).hexdigest())
        self.assertNotEqual(a[0].input_sha256, a[1].input_sha256)
        self.assertNotEqual(a[0].result_sha256, a[1].result_sha256)
        self.assertEqual(a[0].result.to_json(), engine.run(BASE).to_json())

    def test_partial_failures_keep_typed_errors_and_continue(self):
        spec = profile()
        spec["parameters"] = {"E": {"path": MODULUS, "values": [200, -1, 400]}}
        rows = ex.sweep(BASE, spec)
        self.assertEqual([r.name for r in rows], [f"variant-{i:04d}" for i in range(3)])
        self.assertEqual(rows[0].status, "ok")
        self.assertIsInstance(rows[1].error, engine.InvalidInputError)
        self.assertNotEqual(rows[1].status, "ok")
        self.assertTrue(rows[1].result.partial)
        self.assertEqual(rows[1].error.document, rows[1].result.to_json())
        self.assertEqual(rows[1].error.diagnostics, rows[1].result.diagnostics)
        self.assertIsNotNone(rows[1].input_sha256)
        self.assertIsNotNone(rows[1].result_sha256)
        self.assertEqual(rows[2].status, "ok")
        with self.assertRaises(engine.InvalidInputError) as caught:
            rows[1].raise_for_error()
        self.assertIs(caught.exception, rows[1].error)
        rows[2].raise_for_error()

    def test_scenarios_include_unstable_and_malformed_without_stopping(self):
        unstable = deepcopy(BASE)
        unstable["model"]["supports"] = []
        spec = profile(); spec.pop("parameters")
        rows = ex.compare_scenarios({"b-unstable": unstable, "c-ok": BASE,
                                    "a-invalid": "{"}, spec)
        self.assertEqual([r.name for r in rows], ["a-invalid", "b-unstable", "c-ok"])
        self.assertIsInstance(rows[0].error, engine.InvalidInputError)
        self.assertIsNone(rows[0].input_sha256)
        self.assertIsInstance(rows[1].error, engine.Unstable)
        self.assertEqual(rows[1].status, "unstable")
        self.assertEqual(rows[2].status, "ok")

    def test_profile_method_substitution_records_requested_and_actual(self):
        data = json.loads((FIXTURES / "p-delta.json").read_text())
        original = deepcopy(data)
        spec = profile(); spec.pop("parameters"); spec.pop("metrics")
        pdelta = ex.sweep(data, spec)[0]
        spec["analyses"] = [{"method": "linearStatic"}]
        linear = ex.sweep(data, spec)[0]
        self.assertEqual(pdelta.provenance["methods"], ["pDelta"])
        self.assertEqual(linear.provenance["methods"], ["linearStatic"])
        self.assertEqual(linear.provenance["base_analyses"], original["analyses"])
        self.assertEqual(linear.provenance["requested_analyses"], spec["analyses"])
        self.assertNotEqual(linear.input_sha256, pdelta.input_sha256)
        self.assertEqual(data, original)
        self.assertEqual(linear.status, "ok")

    def test_nan_inf_and_non_numeric_parameters_fail_before_execution(self):
        for value in (float("nan"), float("inf"), -float("inf"), True, "12"):
            spec = profile(); spec["parameters"]["P"]["values"] = [value]
            with self.subTest(value=value), patch.object(ex, "run") as run:
                with self.assertRaises(engine.InvalidInputError) as caught:
                    ex.sweep(BASE, spec)
                self.assertEqual(caught.exception.code, "API-E-INPUT")
                run.assert_not_called()
        spec = profile(); spec["seed"] = float("nan")
        with self.assertRaises(engine.InvalidInputError):
            ex.sweep(BASE, spec)

    def test_sweeping_analysis_options_preserves_original_base_provenance(self):
        data = json.loads((FIXTURES / "p-delta.json").read_text())
        before = deepcopy(data)
        spec = profile(); spec.pop("metrics")
        spec["parameters"] = {"tolerance": {
            "path": ["analyses", 0, "options", "tolerance"], "values": [1e-8, 1e-9]}}
        rows = ex.sweep(data, spec)
        self.assertTrue(all(r.status == "ok" for r in rows))
        for row in rows:
            self.assertEqual(row.provenance["base_analyses"], before["analyses"])
            self.assertEqual(row.provenance["requested_analyses"][0]["options"]["tolerance"],
                             row.parameters["tolerance"])
        self.assertEqual(data, before)

    def test_paths_and_profile_versions_reject_ambiguity(self):
        for path in ([], [True], ["cases", -1], ["absent"], "cases[0]"):
            spec = profile(); spec["parameters"]["P"]["path"] = path
            with self.subTest(path=path), self.assertRaises(engine.InvalidInputError):
                ex.sweep(BASE, spec)
        spec = profile()
        spec["parameters"]["alias"] = deepcopy(spec["parameters"]["P"])
        with self.assertRaises(engine.InvalidInputError):
            ex.sweep(BASE, spec)
        for version in (True, 0, "1"):
            spec = profile(); spec["version"] = version
            with self.assertRaises(engine.InvalidInputError):
                ex.sweep(BASE, spec)
        spec = profile(); spec["schema"] = "mamut.experiment/2"
        with self.assertRaises(engine.Unsupported):
            ex.sweep(BASE, spec)

    def test_no_callbacks_or_registration_cross_the_ffi(self):
        descriptors = engine.capabilities()
        native_names = set(vars(_native))
        original = _native.run
        calls = []
        def spy(*args, **kwargs):
            calls.append(kwargs)
            self.assertIsNone(kwargs.get("progress"))
            self.assertFalse(any(callable(v) for v in args + tuple(kwargs.values())))
            return original(*args, **kwargs)
        with patch.object(_native, "run", side_effect=spy):
            ex.sweep(BASE, profile())
        self.assertEqual(len(calls), 2)
        self.assertEqual(set(vars(_native)), native_names)
        self.assertEqual(engine.capabilities(), descriptors)
        spec = profile(); spec["callback"] = lambda: None
        with self.assertRaises(engine.InvalidInputError):
            ex.sweep(BASE, spec)

    def test_unexpected_python_errors_propagate(self):
        with patch.object(ex, "run", side_effect=LookupError("usuario")):
            with self.assertRaisesRegex(LookupError, "usuario"):
                ex.sweep(BASE, profile())

    def test_errors_without_results_keep_their_category_and_continue(self):
        for category, status in (
            (engine.Cancelled, "cancelled"), (engine.Unsupported, "unsupported"),
            (engine.InternalEngineError, "internalError"), (engine.Unstable, "unstable"),
            (engine.NotConverged, "notConverged"), (engine.InvalidInputError, "invalid"),
            (engine.AnalysisError, "analysisError"), (engine.EngineError, "error"),
        ):
            error = category("fallo sin documento")
            with self.subTest(category=category.__name__), patch.object(
                    ex, "run", side_effect=[error, engine.run(BASE)]):
                failed, passed = ex.sweep(BASE, profile())
            self.assertEqual(failed.status, status)
            self.assertIs(failed.error, error)
            self.assertIsNone(failed.result)
            self.assertIsNone(failed.result_sha256)
            self.assertEqual(passed.status, "ok")

    def test_error_document_status_precedes_category_without_results(self):
        for document, status in (
            ('{"status":"notConverged"}', "notConverged"),
            ('{"status":"unstable"}', "unstable"),
            ('{', "cancelled"), ('[]', "cancelled"),
            ('{"status":"ok"}', "cancelled"),
        ):
            error = engine.Cancelled("fallo sin Results")
            error.document = document
            with self.subTest(document=document), patch.object(ex, "run", side_effect=error):
                row = ex.sweep(BASE, profile())[0]
            self.assertEqual(row.status, status)
            self.assertIs(row.error, error)
            self.assertIsNone(row.result)
            self.assertEqual(row.result_sha256, hashlib.sha256(document.encode()).hexdigest())

    def test_table_preserves_units_and_copies_nested_data(self):
        rows = ex.sweep(BASE, profile())
        table = ex.comparison_table(rows)
        self.assertEqual(table[0]["metrics"]["uy"]["unit"], "m")
        self.assertEqual(table[0]["metrics"]["uy"]["value"],
                         rows[0].result.data["cases"][0]["nodes"][1]["displacements"][1])
        table[0]["metrics"].clear()
        table[0]["provenance"]["profile"].clear()
        self.assertTrue(rows[0].metrics)
        self.assertTrue(rows[0].provenance["profile"])

    def test_bad_metric_marks_variant_without_losing_engine_result(self):
        spec = profile(); spec["metrics"]["uy"]["path"] = ["absent"]
        rows = ex.sweep(BASE, spec)
        self.assertTrue(all(r.status == "metricError" for r in rows))
        self.assertTrue(all(r.result.status == "ok" for r in rows))
        self.assertTrue(all(isinstance(r.error, engine.InvalidInputError) for r in rows))

    def test_discovery_is_the_rust_catalog_and_returns_copies(self):
        before = deepcopy(engine.capabilities())
        descriptors = deepcopy(ex.methods())
        self.assertEqual(descriptors, before["method_descriptors"])
        methods, capabilities = ex.methods(), engine.capabilities()
        # Tanto objetos anidados como los contenedores externos son independientes.
        methods[0]["id"] = "mutado"
        methods[0].clear()
        methods.clear()
        capabilities["method_descriptors"][0]["id"] = "mutado"
        capabilities["method_descriptors"][0].clear()
        capabilities["method_descriptors"].clear()
        capabilities.clear()
        self.assertEqual(ex.methods(), descriptors)
        self.assertEqual(engine.capabilities(), before)

    def test_no_numpy_is_required(self):
        original = builtins.__import__
        def without_numpy(name, *args, **kwargs):
            if name.split(".")[0] == "numpy":
                raise AssertionError("El experimento intentó importar NumPy")
            return original(name, *args, **kwargs)
        with patch("builtins.__import__", side_effect=without_numpy):
            self.assertTrue(all(r.status == "ok" for r in ex.sweep(BASE, profile())))


class WorkflowTimeoutTest(unittest.TestCase):
    def setUp(self):
        self.spec = profile(); self.spec.pop("parameters"); self.spec.pop("metrics")

    def operations(self, executable):
        compiled = ex.CompiledWorkflow("{}", "{}", "{}", executable)
        return (
            ("compile", lambda **kw: ex.compile_workflow({}, {}, executable=executable, **kw)),
            ("run", lambda **kw: ex.run_workflow(compiled, BASE, self.spec, **kw)),
        )

    def wait_for_pid(self, pid_file, *, subcase, started):
        waiting = time.monotonic()
        deadline = waiting + 5
        while True:
            try:
                text = pid_file.read_text(encoding="utf-8")
            except FileNotFoundError:
                text = ""
            if text.endswith("\n") and text[:-1].isdigit() and int(text) > 0:
                return int(text)
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                self.fail(msg=f"{subcase}: no apareció un PID completo en {pid_file}; "
                          f"contenido={text!r}, espera={time.monotonic() - waiting:.3f} s, "
                          f"elapsed={time.monotonic() - started:.3f} s")
            time.sleep(min(0.01, remaining))

    def pid_sleeper(self, directory):
        # El propio nieto publica tras exec, con SIGTERM por defecto; $! en el
        # shell solo prueba fork y puede adelantarse al restablecimiento de traps.
        sleeper = Path(directory) / "pid-sleeper.py"
        sleeper.write_text(
            "import os, sys, time\nfrom pathlib import Path\n"
            "Path(sys.argv[1]).write_text(str(os.getpid()) + '\\n', encoding='utf-8')\n"
            "time.sleep(60)\n", encoding="utf-8")
        return sleeper

    def pgid(self, pid, *, subcase, started):
        try:
            return os.getpgid(pid)
        except ProcessLookupError:
            self.fail(msg=f"{subcase}: PID={pid} desapareció antes de consultar su PGID, "
                      f"elapsed={time.monotonic() - started:.3f} s")

    def test_sleeping_cli_times_out_is_reaped_and_removes_temporary_inputs(self):
        with TemporaryDirectory() as directory:
            executable = Path(directory) / "sleeping-cli"
            executable.write_text(f"#!{sys.executable}\nimport time\ntime.sleep(60)\n",
                                  encoding="utf-8")
            executable.chmod(0o755)
            original_popen = subprocess.Popen
            for name, operation in self.operations(str(executable)):
                started = time.monotonic()
                subcase = f"sleeping/{name}"
                processes = []
                def start(*args, **kwargs):
                    process = original_popen(*args, **kwargs)
                    processes.append(process)
                    return process
                with patch.object(workflows.subprocess, "Popen", side_effect=start):
                    with self.assertRaises(engine.AnalysisError) as caught:
                        operation(timeout=0.2)
                error = caught.exception
                self.assertEqual(error.code, "PY-E-WORKFLOW-TIMEOUT")
                self.assertIsInstance(error.__cause__, subprocess.TimeoutExpired)
                self.assertEqual(len(processes), 1,
                                 msg=f"{subcase}: PID registrados={[p.pid for p in processes]}, "
                                     f"elapsed={time.monotonic() - started:.3f} s")
                self.assertIsNotNone(processes[0].returncode,
                                     msg=f"{subcase}: PID={processes[0].pid}, "
                                         f"elapsed={time.monotonic() - started:.3f} s")
                self.assertIsNotNone(processes[0].poll(),
                                     msg=f"{subcase}: PID={processes[0].pid}, "
                                         f"elapsed={time.monotonic() - started:.3f} s")
                command = error.__cause__.cmd
                temporary_root = Path(command[command.index("--catalog") + 1]).parent
                self.assertFalse(temporary_root.exists())

    def test_invalid_timeout_is_rejected_before_starting_cli(self):
        for name, operation in self.operations("/mamut/missing-cli"):
            for value in (0, -1, float("nan"), float("inf"), -float("inf"),
                          True, False, None, "1", 10**400):
                with self.subTest(operation=name, timeout=value), patch.object(workflows.subprocess, "Popen") as start:
                    with self.assertRaises(engine.InvalidInputError):
                        operation(timeout=value)
                    start.assert_not_called()

    def test_timeout_default_and_custom_value_reach_subprocess(self):
        for name, operation in self.operations("/mamut/simulated-cli"):
            for options, expected in (({}, 300), ({"timeout": 1.5}, 1.5)):
                started = time.monotonic()
                with self.subTest(operation=name, options=options), patch.object(
                        workflows.subprocess, "Popen") as start:
                    process = start.return_value.__enter__.return_value
                    process.communicate.return_value = ("{}", "")
                    process.returncode = 0
                    operation(**options)
                self.assertEqual(process.communicate.call_args_list, [call(timeout=expected)],
                                 msg=f"timeout-option/{name}/{options}: "
                                     f"elapsed={time.monotonic() - started:.3f} s")

    def test_transport_preserves_utf8_output_and_exit_codes(self):
        with TemporaryDirectory() as directory:
            executable = Path(directory) / "output-cli"
            for returncode in (0, 1, 3):
                executable.write_text(
                    f"#!{sys.executable}\nimport sys\n"
                    "sys.stdout.buffer.write('salida á\\r\\n'.encode('utf-8'))\n"
                    "sys.stderr.buffer.write(b'{\"message\":\"fallo\"}')\n"
                    f"sys.exit({returncode})\n", encoding="utf-8")
                executable.chmod(0o755)
                with self.subTest(returncode=returncode):
                    if returncode in (0, 1):
                        self.assertEqual(workflows._invoke(str(executable), "compile", "{}", "{}"),
                                         "salida á\n")
                    else:
                        with self.assertRaises(engine.Unsupported) as caught:
                            workflows._invoke(str(executable), "compile", "{}", "{}")
                        self.assertEqual(caught.exception.code, "SDK-EXIT-3")
                        self.assertEqual(caught.exception.details["workflow_envelope"],
                                         {"message": "fallo"})

    @unittest.skipUnless(os.name == "posix" and hasattr(os, "killpg"),
                         "Requiere grupos de procesos POSIX")
    def test_timeout_terminates_shell_and_grandchild(self):
        import shutil

        sleep = shutil.which("sleep", path=os.defpath)
        if sleep is None:
            self.skipTest("Requiere sleep disponible en las rutas del sistema")
        sleep = shlex.quote(sleep)
        with TemporaryDirectory() as directory:
            executable = Path(directory) / "wrapper-cli"
            pid_file = Path(directory) / "grandchild.pid"
            second_pid_file = Path(directory) / "second-grandchild.pid"
            ready_file = Path(directory) / "wrapper.pid"
            sleeper = (f"{shlex.quote(sys.executable)} -I -B "
                       f"{shlex.quote(str(self.pid_sleeper(directory)))}")
            # El trap recoge a los nietos: no depender de la recolección de init.
            # El segundo caso conserva el envoltorio vivo con las tuberías cerradas.
            for stubborn in (False, True):
                handler = f"wait; exec {sleep} 60" if stubborn else "wait; exit"
                executable.write_text(
                    f"#!/bin/sh\ntrap {shlex.quote(handler)} TERM\n"
                    f"{sleeper} {shlex.quote(str(pid_file))} &\n"
                    + ("exec >/dev/null 2>&1\n" if stubborn else "")
                    + f"{sleeper} {shlex.quote(str(second_pid_file))} &\n"
                    f"echo $$ > {shlex.quote(str(ready_file))}\nwait\n", encoding="utf-8")
                executable.chmod(0o755)
                for name, operation in self.operations(str(executable)):
                    subcase = f"shell/{name}/stubborn={stubborn}"
                    original_communicate = subprocess.Popen.communicate
                    processes, descendants, ready_at = [], [], []
                    started = time.monotonic()

                    def communicate(process, *args, **kwargs):
                        if not processes:
                            processes.append(process)
                            context = f"{subcase}/CLI-PID={process.pid}"
                            for child_file in (pid_file, second_pid_file):
                                pid = self.wait_for_pid(child_file, subcase=context, started=started)
                                descendants.append(pid)
                            wrapper = self.wait_for_pid(ready_file, subcase=context, started=started)
                            self.assertEqual(wrapper, process.pid,
                                             msg=f"{context}: PID publicado={wrapper}, "
                                                 f"elapsed={time.monotonic() - started:.3f} s")
                            # Presupuesto real de timeout + terminación, después
                            # de preparar el escenario; communicate no se simula.
                            ready_at.append(time.monotonic())
                        return original_communicate(process, *args, **kwargs)

                    with self.subTest(operation=name, stubborn=stubborn):
                        try:
                            with patch.object(
                                    workflows.subprocess.Popen, "communicate", autospec=True,
                                    side_effect=communicate):
                                with self.assertRaises(engine.AnalysisError) as caught:
                                    operation(timeout=0.2)
                            elapsed = time.monotonic() - ready_at[0]
                            self.assertLess(elapsed, 2,
                                            msg=f"{subcase}: la terminación del grupo tardó demasiado, "
                                                f"PID={processes[0].pid}, elapsed={elapsed:.3f} s, "
                                                f"setup={ready_at[0] - started:.3f} s")
                            error = caught.exception
                            self.assertEqual(error.code, "PY-E-WORKFLOW-TIMEOUT")
                            self.assertIsInstance(error.__cause__, subprocess.TimeoutExpired)
                            self.assertEqual(len(processes), 1,
                                             msg=f"{subcase}: PID registrados={[p.pid for p in processes]}, "
                                                 f"elapsed={time.monotonic() - started:.3f} s")
                            self.assertIsNotNone(processes[0].returncode,
                                                 msg=f"{subcase}: PID={processes[0].pid}, "
                                                     f"elapsed={time.monotonic() - started:.3f} s")
                            if stubborn:
                                self.assertEqual(processes[0].returncode, -signal.SIGKILL,
                                                 msg=f"{subcase}: PID={processes[0].pid}, "
                                                     f"elapsed={time.monotonic() - started:.3f} s")
                            command = error.__cause__.cmd
                            root = Path(command[command.index("--catalog") + 1]).parent
                            self.assertFalse(root.exists())
                            for pid in descendants:
                                deadline = time.monotonic() + 1
                                while True:
                                    try:
                                        os.kill(pid, 0)
                                    except ProcessLookupError:
                                        break
                                    self.assertLess(time.monotonic(), deadline,
                                                    msg=f"{subcase}: el nieto PID={pid} sigue existiendo "
                                                        f"tras el timeout, "
                                                        f"elapsed={time.monotonic() - started:.3f} s")
                                    time.sleep(0.01)
                        finally:
                            # No volver a esperar un archivo que nunca se publicó ni
                            # dejar procesos vivos cuando falla una aserción anterior.
                            for pid in descendants:
                                try:
                                    os.kill(pid, signal.SIGKILL)
                                except ProcessLookupError:
                                    pass
                            for process in processes:
                                if process.poll() is None:
                                    workflows._terminate(process, process_group=True)
                            pid_file.unlink(missing_ok=True)
                            second_pid_file.unlink(missing_ok=True)
                            ready_file.unlink(missing_ok=True)

    @unittest.skipUnless(os.name == "posix" and hasattr(os, "killpg"),
                         "Requiere grupos de procesos POSIX")
    def test_keyboard_interrupt_terminates_shell_and_grandchild(self):
        with TemporaryDirectory() as directory:
            executable = Path(directory) / "interrupted-cli"
            pid_file = Path(directory) / "grandchild.pid"
            sleeper = (f"{shlex.quote(sys.executable)} -I -B "
                       f"{shlex.quote(str(self.pid_sleeper(directory)))}")
            executable.write_text(
                "#!/bin/sh\ntrap 'wait; exit' TERM\n"
                f"{sleeper} {shlex.quote(str(pid_file))} &\nwait\n", encoding="utf-8")
            executable.chmod(0o755)
            original_communicate = subprocess.Popen.communicate
            for name, operation in self.operations(str(executable)):
                started = time.monotonic()
                subcase = f"interrupt/{name}"
                before = set(Path(gettempdir()).glob("mamut-experiment-*"))
                processes, descendants = [], []
                interruption = KeyboardInterrupt("Interrupción durante communicate")

                def interrupt(process, *args, **kwargs):
                    if not processes:
                        processes.append(process)
                        pid = self.wait_for_pid(pid_file,
                                                subcase=f"{subcase}/CLI-PID={process.pid}",
                                                started=started)
                        descendants.append(pid)
                        self.assertEqual(self.pgid(pid, subcase=subcase, started=started), process.pid,
                                         msg=f"{subcase}: PID={pid}, PGID esperado={process.pid}, "
                                             f"elapsed={time.monotonic() - started:.3f} s")
                        self.assertIsNone(process.poll(),
                                          msg=f"{subcase}: PID={process.pid}, "
                                              f"elapsed={time.monotonic() - started:.3f} s")
                        raise interruption
                    return original_communicate(process, *args, **kwargs)

                try:
                    with patch.object(workflows.subprocess.Popen, "communicate",
                                      autospec=True, side_effect=interrupt):
                        with self.assertRaises(KeyboardInterrupt) as caught:
                            operation(timeout=10)
                    self.assertIs(caught.exception, interruption)
                    self.assertEqual(len(processes), 1,
                                     msg=f"{subcase}: PID registrados={[p.pid for p in processes]}, "
                                         f"elapsed={time.monotonic() - started:.3f} s")
                    process = processes[0]
                    self.assertIsNotNone(process.poll(),
                                         msg=f"{subcase}: PID={process.pid}, "
                                             f"elapsed={time.monotonic() - started:.3f} s")
                    for pid in (process.pid, *descendants):
                        deadline = time.monotonic() + 1
                        while True:
                            try:
                                os.kill(pid, 0)
                            except ProcessLookupError:
                                break
                            self.assertLess(time.monotonic(), deadline,
                                            msg=f"{subcase}: PID={pid} sigue existiendo tras "
                                                f"KeyboardInterrupt, "
                                                f"elapsed={time.monotonic() - started:.3f} s")
                            time.sleep(0.01)
                    root = Path(process.args[process.args.index("--catalog") + 1]).parent
                    self.assertFalse(root.exists())
                    self.assertFalse(set(Path(gettempdir()).glob("mamut-experiment-*")) - before)
                finally:
                    # Recoger también los procesos si una regresión hace fallar el test.
                    for process in processes:
                        workflows._terminate(process, process_group=True)
                    for pid in descendants:
                        try:
                            os.kill(pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    pid_file.unlink(missing_ok=True)

    @unittest.skipUnless(os.name == "posix" and hasattr(os, "killpg"),
                         "Requiere grupos de procesos POSIX")
    def test_timeout_preserves_error_when_group_probe_is_denied(self):
        original_killpg = os.killpg

        def deny_probe(pid, sig):
            if sig == 0:
                raise PermissionError("Sonda del grupo denegada")
            return original_killpg(pid, sig)

        with TemporaryDirectory() as directory:
            executable = Path(directory) / "sleeping-cli"
            executable.write_text(f"#!{sys.executable}\nimport time\ntime.sleep(60)\n",
                                  encoding="utf-8")
            executable.chmod(0o755)
            for name, operation in self.operations(str(executable)):
                started = time.monotonic()
                subcase = f"denied-probe/{name}"
                with patch.object(workflows.os, "killpg", side_effect=deny_probe) as killpg:
                    with self.assertRaises(engine.AnalysisError) as caught:
                        operation(timeout=0.2)
                self.assertEqual(caught.exception.code, "PY-E-WORKFLOW-TIMEOUT")
                self.assertIsInstance(caught.exception.__cause__, subprocess.TimeoutExpired)
                pid = killpg.call_args_list[0].args[0]
                self.assertIn(call(pid, 0), killpg.call_args_list,
                              msg=f"{subcase}: PID={pid}, "
                                  f"elapsed={time.monotonic() - started:.3f} s")
                self.assertIn(call(pid, signal.SIGKILL), killpg.call_args_list,
                              msg=f"{subcase}: PID={pid}, "
                                  f"elapsed={time.monotonic() - started:.3f} s")

    @unittest.skipUnless(os.name == "posix" and hasattr(os, "killpg"),
                         "Requiere grupos de procesos POSIX")
    def test_timeout_closes_pipes_held_by_descendant_outside_group(self):
        import shutil

        setsid = shutil.which("setsid", path=os.defpath)
        if setsid is None:
            self.skipTest("Requiere setsid disponible en las rutas del sistema")
        setsid = str(Path(setsid).resolve())
        with TemporaryDirectory() as directory:
            executable = Path(directory) / "escaped-child-cli"
            pid_file = Path(directory) / "escaped.pid"
            sleeper = str(self.pid_sleeper(directory))
            executable.write_text(
                f"#!{sys.executable}\nimport subprocess, time\n"
                f"subprocess.Popen([{setsid!r}, {sys.executable!r}, '-I', '-B', "
                f"{sleeper!r}, {str(pid_file)!r}])\n"
                "time.sleep(60)\n", encoding="utf-8")
            executable.chmod(0o755)
            original_communicate = subprocess.Popen.communicate
            for name, operation in self.operations(str(executable)):
                subcase = f"escaped/{name}"
                processes, descendants, ready_at = [], [], []
                started = time.monotonic()

                def communicate(process, *args, **kwargs):
                    if not processes:
                        processes.append(process)
                        context = f"{subcase}/CLI-PID={process.pid}"
                        pid = self.wait_for_pid(pid_file, subcase=context, started=started)
                        descendants.append(pid)
                        self.assertEqual(self.pgid(pid, subcase=context, started=started), pid,
                                         msg=f"{context}: PID={pid}, setsid no se completó, "
                                             f"elapsed={time.monotonic() - started:.3f} s")
                        ready_at.append(time.monotonic())
                    return original_communicate(process, *args, **kwargs)

                try:
                    with patch.object(workflows.subprocess.Popen, "communicate",
                                      autospec=True, side_effect=communicate):
                        with self.assertRaises(engine.AnalysisError) as caught:
                            operation(timeout=0.2)
                    elapsed = time.monotonic() - ready_at[0]
                    pid = descendants[0]
                    self.assertEqual(caught.exception.code, "PY-E-WORKFLOW-TIMEOUT")
                    self.assertIsInstance(caught.exception.__cause__, subprocess.TimeoutExpired)
                    self.assertEqual(caught.exception.__cause__.timeout, 0.2,
                                     msg=f"{subcase}: elapsed={elapsed:.3f} s")
                    self.assertLess(elapsed, 8,
                                    msg=f"{subcase}: la limpieza del timeout quedó bloqueada, "
                                        f"PID={pid}, elapsed={elapsed:.3f} s, "
                                        f"setup={ready_at[0] - started:.3f} s")
                    self.assertEqual(len(processes), 1,
                                     msg=f"{subcase}: PID registrados={[p.pid for p in processes]}, "
                                         f"elapsed={time.monotonic() - started:.3f} s")
                    process = processes[0]
                    self.assertIsNotNone(process.returncode,
                                         msg=f"{subcase}: PID={process.pid}, "
                                             f"elapsed={time.monotonic() - started:.3f} s")
                    self.assertTrue(process.stdout.closed,
                                    msg=f"{subcase}: PID={process.pid}, "
                                        f"elapsed={time.monotonic() - started:.3f} s")
                    self.assertTrue(process.stderr.closed,
                                    msg=f"{subcase}: PID={process.pid}, "
                                        f"elapsed={time.monotonic() - started:.3f} s")
                    self.assertEqual(self.pgid(pid, subcase=subcase, started=started), pid,
                                     msg=f"{subcase}: PID={pid}, "
                                         f"elapsed={time.monotonic() - started:.3f} s")
                    self.assertNotEqual(self.pgid(pid, subcase=subcase, started=started), process.pid,
                                        msg=f"{subcase}: PID={pid}, PGID del CLI={process.pid}, "
                                            f"elapsed={time.monotonic() - started:.3f} s")
                    command = caught.exception.__cause__.cmd
                    root = Path(command[command.index("--catalog") + 1]).parent
                    self.assertFalse(root.exists())
                finally:
                    for pid in descendants:
                        try:
                            os.kill(pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    for process in processes:
                        if process.poll() is None:
                            workflows._terminate(process, process_group=True)
                    pid_file.unlink(missing_ok=True)


@unittest.skipUnless(CLI, "Defina MAMUT_EXPERIMENT_CLI con el binario Rust local")
class WorkflowTest(unittest.TestCase):
    def setUp(self):
        self.catalog = json.loads((ROOT / "crates/engine-cli/catalog.json").read_text())
        self.workflow = json.loads((ROOT / "crates/engine-cli/run.workflow.json").read_text())
        self.spec = profile(); self.spec.pop("parameters"); self.spec.pop("metrics")

    def test_real_sdk_plan_replay_manifest_and_substitution(self):
        compiled = ex.compile_workflow(self.catalog, self.workflow, executable=CLI)
        self.workflow["steps"][0]["method_id"] = "engine.validate"
        validation = ex.compile_workflow(self.catalog, self.workflow, executable=CLI)
        # Las mutaciones del input y de las copias del plan no cambian el snapshot.
        compiled.plan["steps"].clear()
        a = ex.run_workflow(compiled, BASE, self.spec)
        b = ex.run_workflow(compiled, BASE, self.spec)
        self.assertEqual(a, b)
        manifest = a["envelope"]["data"]
        self.assertEqual(manifest["steps"][0]["method"]["id"], "engine.run")
        self.assertTrue(manifest["recordFingerprint"])
        self.assertTrue(manifest["steps"][0]["outputs"])
        self.assertEqual(a["envelope"]["engineResults"][0], engine.run(BASE).data)
        c = ex.run_workflow(validation, BASE, self.spec)
        self.assertEqual(c["envelope"]["data"]["steps"][0]["method"]["id"], "engine.validate")
        self.assertNotEqual(manifest["recordFingerprint"], c["envelope"]["data"]["recordFingerprint"])
        self.assertEqual(c["profile"], self.spec)

    def test_sdk_multi_step_order_is_preserved(self):
        step = deepcopy(self.workflow["steps"][0]); step["method_id"] = "engine.validate"
        self.workflow["steps"].insert(0, step)
        self.catalog["profile"]["slots"] *= 2
        compiled = ex.compile_workflow(self.catalog, self.workflow, executable=CLI)
        result = ex.run_workflow(compiled, BASE, self.spec)
        self.assertEqual([s["method"]["id"] for s in result["envelope"]["data"]["steps"]],
                         ["engine.validate", "engine.run"])
        self.assertEqual(len(result["envelope"]["engineResults"]), 2)

    def test_sdk_errors_remain_typed_with_original_diagnostics(self):
        compiled = ex.compile_workflow(self.catalog, self.workflow, executable=CLI)
        data = deepcopy(BASE); data["model"]["supports"] = []
        with self.assertRaises(engine.Unstable) as caught:
            ex.run_workflow(compiled, data, self.spec)
        self.assertEqual(caught.exception.code, "ENG-E-0006")
        self.assertEqual(caught.exception.results.status, "unstable")
        self.assertEqual(caught.exception.diagnostics, caught.exception.results.diagnostics)
        self.assertIn("workflow_envelope", caught.exception.details)
        self.workflow["steps"][0]["method_id"] = "python.function"
        with self.assertRaises(engine.InvalidInputError):
            ex.compile_workflow(self.catalog, self.workflow, executable=CLI)

    def test_sdk_no_transport_silencing_and_profile_policy(self):
        with self.assertRaises(engine.InvalidInputError) as caught:
            ex.compile_workflow(self.catalog, self.workflow, executable="/mamut/missing-cli")
        self.assertIsInstance(caught.exception.__cause__, OSError)
        compiled = ex.compile_workflow(self.catalog, self.workflow, executable=CLI)
        self.spec["analyses"] = [{"method": "linearStatic"}]
        with self.assertRaises(engine.InvalidInputError):
            ex.run_workflow(compiled, BASE, self.spec)
        self.spec.pop("analyses")
        self.spec["metrics"] = profile()["metrics"]
        with self.assertRaises(engine.InvalidInputError):
            ex.run_workflow(compiled, BASE, self.spec)


if __name__ == "__main__":
    unittest.main()
