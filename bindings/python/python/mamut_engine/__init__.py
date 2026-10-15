"""Fachada nativa de MAMUT Engine; JSON de entrada y resultados canónicos."""

from ._native import (
    AnalysisError as AnalysisError,
    CancelledError as CancelledError,
    CancelToken as CancelToken,
    EngineError as EngineError,
    InternalEngineError as InternalEngineError,
    InvalidInputError as InvalidInputError,
    capabilities as capabilities,
    explain as explain,
    run as run,
    schema as schema,
    validate as validate,
    version as version,
)

__version__: str = version()["engine_version"]
__all__ = [
    "AnalysisError", "CancelledError", "CancelToken", "EngineError",
    "InternalEngineError", "InvalidInputError", "capabilities", "explain",
    "run", "schema", "validate", "version", "__version__",
]
