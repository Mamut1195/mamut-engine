"""API Python pura y tipada sobre engine-api. SI, ejes X-Y-Z, sin física Python."""
from .errors import (
    AnalysisError as AnalysisError,
    Cancelled as Cancelled,
    CancelledError as CancelledError,
    EngineError as EngineError,
    InternalEngineError as InternalEngineError,
    InvalidInputError as InvalidInputError,
    NotConverged as NotConverged,
    MissingNumPyError as MissingNumPyError,
    StaleResultsError as StaleResultsError,
    Unstable as Unstable,
    Unsupported as Unsupported,
)
from .arrays import (
    ArrayResults as ArrayResults,
    ResultBlock as ResultBlock,
    ResultMatrix as ResultMatrix,
)
from .facade import (
    CancelToken as CancelToken,
    Engine as Engine,
    Model as Model,
    ModelLike as ModelLike,
    PreparedLinear as PreparedLinear,
    Results as Results,
    capabilities as capabilities,
    explain as explain,
    prepare as prepare,
    run as run,
    schema as schema,
    validate as validate,
    version as version,
)
from .types import (
    Capabilities as Capabilities,
    CaseDisplacements as CaseDisplacements,
    ModelDocument as ModelDocument,
    NodeDisplacement as NodeDisplacement,
    Progress as Progress,
    ResultsDocument as ResultsDocument,
    ValidationReport as ValidationReport,
    VersionInfo as VersionInfo,
)

__version__: str
__all__ = [
    "ArrayResults", "ResultBlock", "ResultMatrix", "MissingNumPyError",
    "AnalysisError", "Cancelled", "CancelledError", "CancelToken", "Engine",
    "EngineError", "InternalEngineError", "InvalidInputError", "NotConverged",
    "StaleResultsError", "Unstable", "Unsupported", "Model", "ModelLike", "PreparedLinear",
    "Results", "Capabilities", "CaseDisplacements", "ModelDocument", "NodeDisplacement",
    "Progress", "ResultsDocument", "ValidationReport", "VersionInfo", "capabilities",
    "explain", "prepare", "run", "schema", "validate", "version", "__version__",
]
