"""API Python pura y tipada sobre engine-api. SI, ejes X-Y-Z, sin física Python."""
from .errors import (
    AnalysisError as AnalysisError,
    Cancelled as Cancelled,
    CancelledError as CancelledError,
    EngineError as EngineError,
    InternalEngineError as InternalEngineError,
    InvalidInputError as InvalidInputError,
    NotConverged as NotConverged,
    StaleResultsError as StaleResultsError,
    Unstable as Unstable,
    Unsupported as Unsupported,
)
from .facade import (
    CancelToken as CancelToken,
    Engine as Engine,
    Model as Model,
    ModelLike as ModelLike,
    Results as Results,
    capabilities as capabilities,
    explain as explain,
    run as run,
    schema as schema,
    validate as validate,
    version as version,
)
from .types import (
    Capabilities as Capabilities,
    ModelDocument as ModelDocument,
    Progress as Progress,
    ResultsDocument as ResultsDocument,
    ValidationReport as ValidationReport,
    VersionInfo as VersionInfo,
)

__version__: str
__all__ = [
    "AnalysisError", "Cancelled", "CancelledError", "CancelToken", "Engine",
    "EngineError", "InternalEngineError", "InvalidInputError", "NotConverged",
    "StaleResultsError", "Unstable", "Unsupported", "Model", "ModelLike", "Results",
    "Capabilities", "ModelDocument", "Progress", "ResultsDocument",
    "ValidationReport", "VersionInfo", "capabilities", "explain", "run", "schema",
    "validate", "version", "__version__",
]
