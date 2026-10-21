"""Experimentos externos al núcleo: copias, ejecución Rust y tablas sin física.

Los perfiles son datos JSON mamut.experiment/1. Las rutas son listas explícitas
con claves str e índices int, sin expresiones evaluables ni callbacks.
"""
from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
from hashlib import sha256
from itertools import product
import json
from typing import Any, Iterable, cast

from . import _validation as checks
from .facade import Model, ModelLike, Results, capabilities, run, version
from .errors import (
    AnalysisError, Cancelled, EngineError, InternalEngineError, InvalidInputError,
    NotConverged, Unstable, Unsupported,
)
from ._experiment_workflows import (
    CompiledWorkflow as CompiledWorkflow,
    compile_workflow as compile_workflow,
    run_workflow as run_workflow,
)

__all__ = ["VariantResult", "sweep", "compare_scenarios", "comparison_table",
           "methods", "CompiledWorkflow", "compile_workflow", "run_workflow"]


def _path(value: object) -> list[str | int]:
    if (type(value) is not list or not value
            or any(type(p) not in (str, int) or (type(p) is int and p < 0) for p in value)):
        raise checks.invalid("ruta: lista no vacía de claves str e índices int >= 0")
    return cast(list[str | int], value)


def _get(document: Any, path: list[str | int]) -> Any:
    value = document
    for token in path:
        if ((type(value) is dict and type(token) is str and token in value)
                or (type(value) is list and type(token) is int and token < len(value))):
            value = value[token]
        else:
            raise checks.invalid(f"ruta ausente o incompatible: {path}")
    return value


def _profile(document: dict[str, Any]) -> dict[str, Any]:
    # Reutiliza las comprobaciones JSON/NaN/Inf de la fachada, sin otra validación física.
    result = cast(dict[str, Any], checks.parse(checks.dumps(document)))
    checks.contract(result, "mamut.experiment/1")
    if (type(result.get("version")) is not int or result["version"] < 1
            or not isinstance(result.get("name"), str) or not result["name"]):
        raise checks.invalid("perfil requiere name no vacío y version int >= 1")
    if set(result) - {"schema", "version", "name", "seed", "parameters", "metrics", "analyses"}:
        raise checks.invalid("campo de perfil desconocido")
    if "seed" in result and type(result["seed"]) is not int:
        raise checks.invalid("seed debe ser int; el barrido no usa aleatoriedad")
    for key in ("parameters", "metrics"):
        if type(result.get(key, {})) is not dict:
            raise checks.invalid(f"{key} debe ser un objeto")
    for parameter in result.get("parameters", {}).values():
        if type(parameter) is not dict or set(parameter) != {"path", "values"}:
            raise checks.invalid("parámetro requiere path y values")
        _path(parameter["path"])
        values = parameter["values"]
        if (type(values) is not list or not values
                or any(type(v) not in (int, float) for v in values)):
            raise checks.invalid("values requiere números JSON finitos, sin bool")
    paths = [p["path"] for p in result.get("parameters", {}).values()]
    for i, path in enumerate(paths):
        if any(path == other[:len(path)] or other == path[:len(other)] for other in paths[i + 1:]):
            raise checks.invalid("rutas de parámetros solapadas")
    for metric in result.get("metrics", {}).values():
        if (type(metric) is not dict or set(metric) != {"path", "unit"}
                or type(metric["unit"]) is not str or not metric["unit"]):
            raise checks.invalid("métrica requiere path y unit declarada no vacía")
        _path(metric["path"])
    if "analyses" in result and type(result["analyses"]) is not list:
        raise checks.invalid("analyses debe ser una lista engine-input/1")
    return result


def _model(source: ModelLike) -> Model:
    if isinstance(source, Model):
        return Model.from_dict(source.to_dict())
    if isinstance(source, str):
        return Model.from_json(source)
    return Model.from_dict(deepcopy(source))


def methods() -> list[dict[str, Any]]:
    """Lee los descriptores del registro Rust, sin registrar ni sustituir fábricas."""
    return cast(list[dict[str, Any]], deepcopy(capabilities()["method_descriptors"]))


@dataclass(frozen=True)
class VariantResult:
    """Una variante identificada; error conserva la instancia tipada y su salida.

    result_sha256 cubre los bytes UTF-8 exactos de results/1, no una huella JCS.
    input_sha256 procede de Rust. Ningún registro declara un nuevo método oficial.
    """

    name: str
    parameters: dict[str, int | float]
    input_sha256: str | None
    result_sha256: str | None
    status: str
    result: Results | None
    error: EngineError | None
    metrics: dict[str, dict[str, Any]]
    provenance: dict[str, Any]

    def raise_for_error(self) -> None:
        """Permite al consumidor volver a lanzar el fallo original del lote."""
        if self.error is not None:
            raise self.error


def _error_status(error: EngineError) -> str:
    """Sin Results: estado de document; después categoría pública; fallo genérico error.

    Un documento ilegible no oculta la excepción original. Las subclases específicas
    preceden a AnalysisError; el código/diagnóstico original permanece en error.
    """
    if error.document is not None:
        try:
            document = json.loads(error.document)
        except (ValueError, TypeError):
            document = None
        if isinstance(document, dict) and document.get("status") in (
                "invalid", "unstable", "notConverged", "unsupported", "cancelled", "internalError"):
            return document["status"]
    for category, status in (
        (InvalidInputError, "invalid"), (Unstable, "unstable"),
        (NotConverged, "notConverged"), (Unsupported, "unsupported"),
        (Cancelled, "cancelled"), (InternalEngineError, "internalError"),
        (AnalysisError, "analysisError"),
    ):
        if isinstance(error, category):
            return status
    return "error"


def _execute(name: str, model: Model, profile: dict[str, Any],
             parameters: dict[str, int | float], base_sha: str,
             base_analyses: object) -> VariantResult:
    provenance: dict[str, Any] = {
        "profile": deepcopy(profile), "base_input_sha256": base_sha,
        "base_analyses": deepcopy(base_analyses), "units": model.data["units"],
    }
    if "analyses" in profile:
        model.data["analyses"] = deepcopy(profile["analyses"])
    provenance["requested_analyses"] = deepcopy(model.data["analyses"])
    result = None
    error = None
    input_sha = None
    data: Any = {}
    metrics: dict[str, dict[str, Any]] = {}
    try:
        input_sha = model.input_sha256
        # No callback cruza la frontera FFI; Rust resuelve/compila los métodos oficiales.
        result = run(model)
        data = result.to_dict()
        for label, selector in profile.get("metrics", {}).items():
            value = _get(data, selector["path"])
            if type(value) not in (int, float):
                raise checks.invalid(f"métrica {label} debe seleccionar un escalar numérico")
            metrics[label] = {"value": value, "unit": selector["unit"]}
    except EngineError as caught:
        error = caught
        result = result if result is not None else caught.results
    # Las excepciones ajenas a EngineError se propagan: no se ocultan bugs del usuario.
    document = result.to_json() if result is not None else error.document if error else None
    data = result.to_dict() if result is not None else {}
    provenance.update(engine_version=data.get("engine_version", version()["engine_version"]),
                      git_commit=data.get("git_commit"),
                      target=data.get("target"), methods=deepcopy(data.get("methods", [])))
    status = result.status if result is not None else _error_status(error) if error else "invalid"
    if error is not None and status in ("ok", "warning"):
        status = "metricError"
    return VariantResult(name, deepcopy(parameters), input_sha,
                         sha256(document.encode("utf-8")).hexdigest() if document else None,
                         status, result, error, metrics, provenance)


def sweep(base: ModelLike, profile: dict[str, Any]) -> tuple[VariantResult, ...]:
    """Producto cartesiano determinista; nombres de parámetros ordenados.

    Cada ejecución recibe una copia. Un error Engine queda por variante y el
    lote continúa. Errores del perfil (incluido NaN/Inf) se lanzan antes del lote.
    """
    spec = _profile(profile)
    original = _model(base)
    base_sha = original.input_sha256
    parameters = spec.get("parameters", {})
    names = sorted(parameters)
    for parameter in parameters.values():
        if type(_get(original.data, parameter["path"])) not in (int, float):
            raise checks.invalid("el destino del parámetro debe ser numérico")
    outcomes = []
    for index, values in enumerate(product(*(parameters[n]["values"] for n in names))):
        model = _model(original)
        selected = dict(zip(names, values))
        for name, value in selected.items():
            path = parameters[name]["path"]
            _get(model.data, path[:-1])[path[-1]] = value
        outcomes.append(_execute(f"variant-{index:04d}", model, spec, selected, base_sha,
                                 original.data["analyses"]))
    return tuple(outcomes)


def compare_scenarios(scenarios: dict[str, ModelLike],
                      profile: dict[str, Any]) -> tuple[VariantResult, ...]:
    """Ejecuta escenarios por nombre ordenado; métricas son selecciones, sin cálculo."""
    spec = _profile(profile)
    if spec.get("parameters"):
        raise checks.invalid("compare_scenarios requiere perfil sin parámetros de barrido")
    if type(scenarios) is not dict or any(type(n) is not str or not n for n in scenarios):
        raise checks.invalid("escenarios requiere nombres str no vacíos")
    outcomes = []
    for name in sorted(scenarios):
        try:
            model = _model(scenarios[name])
            outcome = _execute(name, model, spec, {}, model.input_sha256, model.data["analyses"])
        except EngineError as error:
            outcome = VariantResult(name, {}, None, None, "invalid", None, error, {},
                                    {"profile": deepcopy(spec), "base_input_sha256": None,
                                     "engine_version": version()["engine_version"],
                                     "git_commit": None, "methods": []})
        outcomes.append(outcome)
    return tuple(outcomes)


def comparison_table(outcomes: Iterable[VariantResult]) -> list[dict[str, Any]]:
    """Filas independientes con métricas/unidades, estado, huellas y procedencia."""
    return [{"scenario": row.name, "status": row.status,
             "input_sha256": row.input_sha256, "result_sha256": row.result_sha256,
             "metrics": deepcopy(row.metrics), "provenance": deepcopy(row.provenance),
             "error": None if row.error is None else {
                 "type": type(row.error).__name__, "code": row.error.code,
                 "cause": row.error.cause, "entities": deepcopy(row.error.entities),
                 "diagnostics": deepcopy(row.error.diagnostics)}} for row in outcomes]


