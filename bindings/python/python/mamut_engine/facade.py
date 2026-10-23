"""Fachada Python tipada: transporte, contratos y procedencia, sin física."""
from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass, field
from functools import lru_cache
from typing import Any, Callable, ParamSpec, TypeAlias, TypeVar, cast

from . import _native
from . import _validation as checks
from .arrays import ArrayResults, export
from .errors import (
    AnalysisError, Cancelled, EngineError, InternalEngineError, InvalidInputError,
    NotConverged, StaleResultsError, Unstable, Unsupported,
)
from .types import (
    Capabilities, JSONDocument, JSONValue, ModelDocument, Progress, ResultDiagnostic,
    ResultsDocument, ValidationReport, VersionInfo,
)

_P = ParamSpec("_P")
_T = TypeVar("_T")


def _translate(err: Any) -> EngineError:
    code: str = getattr(err, "code", "ENG-E-0012")
    cls = {"ENG-E-0006": Unstable, "ENG-E-0007": NotConverged,
           "ENG-E-0013": Unsupported, "ENG-E-0009": Unsupported,
           "API-I-CANCELLED": Cancelled, "ENG-E-0012": InternalEngineError}.get(code)
    if cls is None:
        cls = AnalysisError if isinstance(err, _native.AnalysisError) else InvalidInputError
    document: str | None = getattr(err, "document", None)
    diagnostics: list[ResultDiagnostic] = getattr(err, "diagnostics", [])
    first = next((d for d in diagnostics if d["severity"] == "error"), None)
    result = cls(getattr(err, "cause", str(err)), code=code,
                 entities=deepcopy(getattr(err, "entities", [])),
                 severity=first["severity"] if first else "error",
                 details=cast(JSONDocument, deepcopy(first)) if first else {})
    result.document = document
    result.diagnostics = deepcopy(diagnostics)
    if document is not None:
        try:
            result.results = Results.from_json(document)
        except EngineError:
            # La interpretación secundaria nunca oculta el fallo original.
            # document y diagnostics conservan toda la salida recibida.
            pass
    return result


def _call(function: Callable[_P, _T], *args: _P.args, **kwargs: _P.kwargs) -> _T:
    try:
        return function(*args, **kwargs)
    except _native.EngineError as err:
        raise _translate(err) from None


def version() -> VersionInfo:
    """Versión del motor y contratos, sin información local del entorno."""
    return cast(VersionInfo, _call(_native.version))


def capabilities() -> Capabilities:
    """Operaciones, elementos y métodos realmente disponibles en Rust."""
    return cast(Capabilities, _call(_native.capabilities))


def schema(name: str, version: int | None = None) -> JSONDocument:
    """Schema local: schema('engine-input', 1) o schema('engine-input/1')."""
    if not isinstance(name, str) or (version is not None and type(version) is not int):
        raise checks.invalid("schema requiere nombre str y versión int")
    if version is not None:
        if "/" in name:
            raise checks.invalid("use nombre sin versión cuando especifica version")
        name = f"{name}/{version}"
    return cast(JSONDocument, _call(_native.schema, name))


@lru_cache(maxsize=2)
def _spec(name: str) -> dict[str, Any]:
    return cast(dict[str, Any], schema(name))


def _language(lang: str) -> None:
    """Validate language before crossing the native boundary."""
    if not isinstance(lang, str) or lang not in ("en", "es", "pt"):
        raise checks.invalid('lang must be "en", "es", or "pt"')


def explain(code: str, *, lang: str = "en") -> str:
    """Explicación original del catálogo engine-api; código ausente es error."""
    if not isinstance(code, str):
        raise checks.invalid("code debe ser str")
    _language(lang)
    return _call(_native.explain, code, lang=lang)


@dataclass
class Model:
    """Documento engine-input/1 mutable y explícito, incluido el trabajo.

    data conserva ids, orden y campos opcionales. No añade unidades, gravedad,
    orientación, masas ni opciones físicas; Rust decide la semántica numérica.
    """

    data: ModelDocument

    def __post_init__(self) -> None:
        _input(self)

    @classmethod
    def from_json(cls, source: str) -> Model:
        """Lee JSON estricto sin coerciones ni claves duplicadas."""
        return cls(cast(ModelDocument, checks.parse(source)))

    @classmethod
    def from_dict(cls, document: ModelDocument) -> Model:
        """Conserva la referencia mutable para detectar cambios después de run."""
        return cls(document)

    def to_dict(self) -> ModelDocument:
        """Copia independiente sin perder campos opcionales u orden."""
        _input(self)
        return deepcopy(self.data)

    def to_json(self) -> str:
        """JSON canónico del motor: -0.0 → 0.0, f64, orden de arrays intacto."""
        return _canonical(_input(self))

    @property
    def input_sha256(self) -> str:
        """Huella RFC 8785/JCS de engine-api; no hashea to_json()."""
        return _fingerprint(_input(self))


ModelLike: TypeAlias = Model | ModelDocument | str


def _input(model: ModelLike) -> str:
    if isinstance(model, str):
        source = model
        document = checks.parse(model)
    else:
        document = cast(JSONDocument, model.data if isinstance(model, Model) else model)
        if type(document) is not dict:
            raise checks.invalid("model debe ser Model, documento dict o JSON str")
        source = checks.dumps(document)
    checks.contract(document, "engine-input/1")
    spec = _spec("engine-input/1")
    checks.structure(document, spec, spec)
    cases = cast(list[dict[str, Any]], document["cases"])
    if sum(len(c["loads"]) + len(c.get("member_loads", [])) for c in cases) > 16384:
        raise checks.invalid("máximo total 16384 registros de cargas", "$.cases")
    for analysis in cast(list[dict[str, Any]], document["analyses"]):
        options = analysis.get("options", {})
        if options.get("mode_count", 0) * len(options.get("directions", [])) > 16384:
            raise checks.invalid("mode_count * directions supera 16384", "$.analyses")
    return source


def _canonical(source: str) -> str:
    return _call(_native.canonical_json, source)


def _fingerprint(source: str) -> str:
    return _call(_native.input_sha256, source)


@dataclass(frozen=True)
class Results:
    """Snapshot results/1 con JSON original y vínculo al input de run.

    Los accesos comprueban stale; las copias exportadas son independientes.
    Los documentos incompletos pueden inspeccionarse desde EngineError.results,
    con partial=True, pero run nunca los devuelve como éxito.
    """

    _document: str = field(repr=False)
    _data: ResultsDocument = field(repr=False)
    _source: ModelLike | None = field(default=None, repr=False, compare=False)
    _fingerprint: str | None = field(default=None, repr=False, compare=False)

    @classmethod
    def from_json(cls, source: str, *, model: ModelLike | None = None) -> Results:
        """Importa results/1 preservando bytes y extensiones opcionales.

        Si se proporciona model, comprueba input_sha256 y vincula los accesos.
        Sin model solo conserva la procedencia declarada, no la autentica.
        """
        document = checks.parse(source, bounded=False)
        checks.contract(document, "results/1")
        spec = _spec("results/1")
        checks.structure(document, spec, spec, extensions=True)
        digest = document["input_sha256"]
        if digest is not None and (not isinstance(digest, str) or len(digest) != 64
                                   or any(c not in "0123456789abcdef" for c in digest)):
            raise checks.invalid("input_sha256 debe ser SHA-256 hexadecimal o null")
        result = cls(source, cast(ResultsDocument, document), model,
                     cast(str | None, digest))
        result.assert_current()
        return result

    @property
    def is_stale(self) -> bool:
        """True si cambió el input original, incluso si ahora es inválido."""
        if self._source is None:
            return False
        try:
            return _fingerprint(_input(self._source)) != self._fingerprint
        except EngineError:
            return True

    def assert_current(self, model: ModelLike | None = None) -> None:
        """Verifica el vínculo o un input externo; falla con PY-E-STALE."""
        stale = self.is_stale
        if model is not None:
            try:
                stale = stale or _fingerprint(_input(model)) != self._fingerprint
            except EngineError:
                stale = True
        if stale:
            raise StaleResultsError("Results pertenece a otro estado del input",
                                    code="PY-E-STALE",
                                    details={"input_sha256": self._fingerprint})

    @property
    def data(self) -> ResultsDocument:
        """Copia tipada del documento completo, tras verificar stale."""
        self.assert_current()
        return deepcopy(self._data)

    def to_dict(self) -> ResultsDocument:
        """Exporta todos los campos y su orden sin alterar el snapshot."""
        return self.data

    def to_json(self) -> str:
        """Devuelve exactamente el JSON recibido de Rust o de from_json."""
        self.assert_current()
        return self._document

    def to_arrays(self) -> ArrayResults:
        """Exporta tablas con listas independientes, IDs, GDL, unidades y estado."""
        return export(self.data, numpy=False)

    def to_numpy(self) -> ArrayResults:
        """Exporta copias ndarray float64 C; requiere mamut-engine[numpy]."""
        return export(self.data, numpy=True)

    @property
    def input_sha256(self) -> str | None:
        """Hash canónico del input; null si Rust no terminó de parsearlo."""
        self.assert_current()
        return self._data["input_sha256"]

    @property
    def version(self) -> VersionInfo:
        """Procedencia del productor del resultado, sin consultar otro motor."""
        self.assert_current()
        return {"engine_version": self._data["engine_version"],
                "contracts": list(self._data["contracts"])}

    @property
    def status(self) -> str:
        """Estado results/1 original, incluidos fallos importados."""
        self.assert_current()
        return self._data["status"]

    @property
    def partial(self) -> bool:
        """Marca documentos incompletos/fallidos, aunque no contengan respuestas."""
        return self.status not in ("ok", "warning")

    @property
    def diagnostics(self) -> list[ResultDiagnostic]:
        """Todos los diagnósticos originales, en orden y sin deduplicación."""
        self.assert_current()
        return deepcopy(self._data["diagnostics"])


class CancelToken:
    """Token Python que posee el token atómico privado del motor.

    Cancelar es idempotente y permanente; otro hilo puede cancelar durante run.
    """

    def __init__(self) -> None:
        self._token = _native.CancelToken()

    def cancel(self) -> None:
        """Solicita cancelación cooperativa en el siguiente punto de control."""
        self._token.cancel()

    @property
    def is_cancelled(self) -> bool:
        """Consulta si se solicitó cancelar; nunca restablece el token."""
        return self._token.is_cancelled


def validate(model: ModelLike, *, lang: str = "en") -> ValidationReport:
    """Valida argumentos antes del FFI y delega la topología en Rust.

    Los defectos topológicos se devuelven como diagnósticos, igual que engine-api.
    No comprueba estabilidad ni ejecuta análisis.
    """
    _language(lang)
    return cast(ValidationReport, _call(_native.validate, _input(model), lang=lang))


def run(model: ModelLike, *, lang: str = "en", cancel: CancelToken | None = None,
        progress: Callable[[Progress], object] | None = None) -> Results:
    """Ejecuta Rust; solo ok/warning devuelven Results, con procedencia y stale.

    El progreso se entrega en orden al terminar Rust, también en fallos. Las
    excepciones del callback se propagan; un fallo del motor tiene prioridad.
    """
    _language(lang)
    if cancel is not None and not isinstance(cancel, CancelToken):
        raise checks.invalid("cancel debe ser CancelToken o None")
    if progress is not None and not callable(progress):
        raise checks.invalid("progress debe ser callable o None")
    source = _input(model)
    fingerprint = _fingerprint(source)
    document = _call(_native.run, source, progress=progress,
                     cancel=cancel._token if cancel is not None else None, lang=lang)
    result = Results.from_json(document)
    if result.input_sha256 != fingerprint:
        raise InternalEngineError("input_sha256 no coincide con el input ejecutado",
                                  code="ENG-E-0012")
    return Results(document, result._data, model, fingerprint)


class Engine:
    """Fachada sin estado; los métodos tienen el mismo contrato que las funciones."""

    validate = staticmethod(validate)
    run = staticmethod(run)
    capabilities = staticmethod(capabilities)
    schema = staticmethod(schema)
    explain = staticmethod(explain)
    version = staticmethod(version)
