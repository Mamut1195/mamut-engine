"""Transporte local al CLI SDK existente; no compila ni planifica en Python."""
from __future__ import annotations

from dataclasses import dataclass
from copy import deepcopy
import json
import math
import os
from pathlib import Path
import signal
import subprocess
from tempfile import TemporaryDirectory
import time
from typing import Any

from . import _validation as checks
from .errors import (
    AnalysisError, Cancelled, EngineError, InternalEngineError, InvalidInputError,
    NotConverged, Unstable, Unsupported,
)
from .facade import ModelLike, Results


@dataclass(frozen=True)
class CompiledWorkflow:
    """Snapshot del plan generado por Rust y sus documentos SDK originales.

    No contiene funciones Python. run vuelve a resolver esos mismos documentos
    con el compilador SDK porque el CLI no admite importar planes serializados.
    """

    catalog_json: str
    workflow_json: str
    plan_json: str
    executable: str

    @property
    def plan(self) -> dict[str, Any]:
        """Copia del plan original producido por compile en el SDK."""
        return json.loads(self.plan_json)


def _failure(code: int, envelope: dict[str, Any]) -> EngineError:
    cls: type[EngineError]
    documents = envelope.get("engineResults", [])
    document = next((d for d in documents if d.get("schema") == "results/1"), None)
    if document is not None:
        result = Results.from_json(json.dumps(document, allow_nan=False))
        first = next((d for d in result.diagnostics if d["severity"] == "error"), {})
        cls = {"unstable": Unstable, "notConverged": NotConverged,
               "unsupported": Unsupported, "cancelled": Cancelled,
               "internalError": InternalEngineError, "invalid": InvalidInputError}.get(
                   result.status, AnalysisError)
        error = cls(first.get("cause", "Fallo de workflow"),
                    code=first.get("code", "ENG-E-0012"),
                    entities=deepcopy(first.get("entities", [])), details=deepcopy(first))
        error.document = result.to_json()
        error.results = result
        error.diagnostics = result.diagnostics
    else:
        cls = {2: InvalidInputError, 3: Unsupported, 4: AnalysisError,
               5: Cancelled, 6: AnalysisError}.get(code, InternalEngineError)
        error = cls(envelope.get("message", "Fallo de workflow SDK"), code=f"SDK-EXIT-{code}")
    error.details["workflow_envelope"] = deepcopy(envelope)
    return error


def _terminate(process: subprocess.Popen[str], *, process_group: bool) -> None:
    """Termina el grupo POSIX; sin killpg solo se mata el proceso directo."""
    if process_group:
        # start_new_session hace que PID = PGID, incluso si el envoltorio ya salió.
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        deadline = time.monotonic() + 0.2
        while True:
            process.poll()  # Recoger al hijo aunque sus descendientes sigan vivos.
            try:
                os.killpg(process.pid, 0)
            except ProcessLookupError:
                break
            except OSError:
                # Una sonda denegada no debe ocultar el timeout original.
                remaining = 0
            else:
                remaining = deadline - time.monotonic()
            if remaining <= 0:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                break
            time.sleep(min(0.01, remaining))
    else:
        # Windows no ofrece killpg: conserva la terminación del proceso directo.
        try:
            process.kill()
        except ProcessLookupError:
            pass
    try:
        process.communicate(timeout=5)
    except (subprocess.TimeoutExpired, OSError):
        # Un descendiente fuera del grupo puede seguir reteniendo las tuberías.
        # La limpieza no debe sustituir el TimeoutExpired original del CLI.
        if process_group:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except OSError:
                pass
        try:
            process.kill()
        except OSError:
            pass
        for stream in (process.stdout, process.stderr):
            if stream is not None:
                try:
                    stream.close()
                except OSError:
                    pass
        try:
            process.wait(timeout=5)
        except (subprocess.TimeoutExpired, OSError):
            pass


def _invoke(executable: str, operation: str, catalog: str, workflow: str,
            model_json: str | None = None, *, timeout: float = 300) -> str:
    if type(timeout) not in (int, float):
        raise checks.invalid("timeout debe ser un número finito > 0, en segundos")
    try:
        valid_timeout = math.isfinite(timeout) and timeout > 0
    except OverflowError:
        valid_timeout = False
    if not valid_timeout:
        raise checks.invalid("timeout debe ser un número finito > 0, en segundos")
    with TemporaryDirectory(prefix="mamut-experiment-") as directory:
        root = Path(directory)
        (root / "catalog.json").write_text(catalog, encoding="utf-8")
        (root / "workflow.json").write_text(workflow, encoding="utf-8")
        env = dict(os.environ)
        # No heredar otro input ni un marcador de cancelación de una sesión ajena.
        env.pop("MAMUT_ENGINE_INPUT", None)
        env.pop("MAMUT_ENGINE_CANCEL_FILE", None)
        if model_json is not None:
            (root / "model.json").write_text(model_json, encoding="utf-8")
            env["MAMUT_ENGINE_INPUT"] = str(root / "model.json")
        try:
            process_group = os.name == "posix" and hasattr(os, "killpg")
            with subprocess.Popen(
                [executable, operation, "--catalog", str(root / "catalog.json"),
                 "--workflow", str(root / "workflow.json")],
                stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, encoding="utf-8", env=env, start_new_session=process_group,
            ) as process:
                try:
                    stdout, stderr = process.communicate(timeout=timeout)
                except subprocess.TimeoutExpired:
                    _terminate(process, process_group=process_group)
                    raise
                except BaseException:
                    _terminate(process, process_group=process_group)
                    raise
                completed = subprocess.CompletedProcess(
                    process.args, process.returncode, stdout, stderr)
        except subprocess.TimeoutExpired as exc:
            raise AnalysisError("Se agotó el tiempo del CLI local",
                                code="PY-E-WORKFLOW-TIMEOUT") from exc
        except OSError as exc:
            raise InvalidInputError("No se puede ejecutar el CLI local", code="PY-E-WORKFLOW") from exc
    if completed.returncode not in (0, 1):
        try:
            envelope = json.loads(completed.stderr)
        except (ValueError, TypeError) as exc:
            raise InternalEngineError("Respuesta inválida del CLI", code="PY-E-WORKFLOW") from exc
        raise _failure(completed.returncode, envelope)
    return completed.stdout


def compile_workflow(catalog: dict[str, Any], workflow: dict[str, Any], *,
                     executable: str, timeout: float = 300) -> CompiledWorkflow:
    """Compila con el CLI local; timeout limita la espera en segundos (finito > 0)."""
    catalog_json, workflow_json = checks.dumps(catalog), checks.dumps(workflow)
    plan = _invoke(executable, "compile", catalog_json, workflow_json, timeout=timeout)
    return CompiledWorkflow(catalog_json, workflow_json, plan, executable)


def run_workflow(compiled: CompiledWorkflow, model: ModelLike,
                 profile: dict[str, Any], *, timeout: float = 300) -> dict[str, Any]:
    """Ejecuta solo métodos registrados en Rust y conserva el sobre/manifiesto SDK.

    profile no admite barrido, métricas ni reemplazo analyses en esta vía: las selecciones
    oficiales están en compiled.plan y el modelo. El perfil completo acompaña al
    sobre sin alterar su recordFingerprint ni las huellas de sus pasos.
    timeout limita la espera del CLI en segundos (finito > 0; por defecto 300).
    """
    from .experiments import _model, _profile

    spec = _profile(profile)
    if spec.get("parameters") or spec.get("metrics") or "analyses" in spec:
        raise checks.invalid("workflow requiere perfil sin parameters, metrics ni analyses")
    snapshot = _model(model)
    source = _invoke(compiled.executable, "run", compiled.catalog_json,
                     compiled.workflow_json, snapshot.to_json(), timeout=timeout)
    envelope = json.loads(source)
    return {"profile": spec, "input_sha256": snapshot.input_sha256,
            "compiled_plan": compiled.plan, "envelope": envelope}
