"""Errores públicos propios; ninguna excepción nativa cruza la fachada."""
from __future__ import annotations

import mamut_engine

from .types import JSONDocument, JSONValue, ResultDiagnostic, ResultEntity


class EngineError(Exception):
    """Fallo con código estable, entidades ordenadas y diagnóstico original."""

    code: str
    entities: list[ResultEntity]
    severity: str
    details: JSONDocument
    cause: str
    document: str | None
    diagnostics: list[ResultDiagnostic]
    results: mamut_engine.Results | None

    def __init__(self, cause: str, *, code: str = "API-E-INPUT",
                 entities: list[ResultEntity] | None = None, severity: str = "error",
                 details: JSONDocument | None = None) -> None:
        super().__init__(cause)
        self.code = code
        self.entities = [] if entities is None else entities
        self.severity = severity
        self.details = {} if details is None else details
        self.cause = cause
        self.document = None
        self.diagnostics = []
        self.results = None


class InvalidInputError(EngineError):
    """Argumento, JSON o contrato de entrada inválido."""


class AnalysisError(EngineError):
    """El análisis no produjo una solución satisfactoria."""


class NotConverged(AnalysisError):
    """ENG-E-0007: se agotó la convergencia."""


class Unstable(AnalysisError):
    """ENG-E-0006: sistema inestable o matriz singular."""


class Cancelled(EngineError):
    """API-I-CANCELLED: cancelación cooperativa permanente."""


class Unsupported(EngineError):
    """ENG-E-0013 o ENG-E-0009: operación o contrato no soportado."""


class InternalEngineError(EngineError):
    """ENG-E-0012: fallo interno de Rust, contenido en la frontera."""


class StaleResultsError(EngineError):
    """PY-E-STALE: el modelo actual difiere de input_sha256."""


# Compatibilidad nominal; son clases Python, nunca alias de _native.
CancelledError = Cancelled
