"""Documentos tipados de los esquemas engine-input/1 y results/1.

Este módulo lo escribe el generador de la biblioteca estándar. No se edita
a mano. Si un esquema cambia, hay que volver a generarlo: el test del
repositorio compara los bytes.

Las dataclasses son frozen, slots y kw_only. Cada enum de cadenas es un
StrEnum cerrado. La unión Analysis está discriminada por method.

Un campo opcional vale None cuando el documento omite la propiedad. Si el
esquema permite null, None también representa ese null. None no inventa una
magnitud ni un valor que el esquema no haya escrito.
"""

from __future__ import annotations

import sys
from dataclasses import dataclass
from enum import Enum
from typing import Literal, TypeAlias, final

if sys.version_info >= (3, 11):
    from enum import StrEnum
else:

    class StrEnum(str, Enum):
        """Enumeración de cadenas cerrada para Python 3.10.

        La biblioteca estándar llama StrEnum a esta clase desde Python 3.11.
        El paquete declara Python 3.10, donde ese nombre todavía no existe.
        Esta clase no define una metaclase propia.
        """

        def __str__(self) -> str:
            value = self.value
            if isinstance(value, str):
                return value
            return str(value)


# engine-input/1
# --------------


@dataclass(frozen=True, slots=True, kw_only=True)
class InputUniformLoad:
    """Objeto ``uniformLoad`` del esquema engine-input/1."""

    member: InputId
    force_per_length: list[float]
    moment_per_length: list[float]


@dataclass(frozen=True, slots=True, kw_only=True)
class InputUniformTemperature:
    """Objeto ``uniform_temperature`` del esquema engine-input/1."""

    member: InputId
    alpha_per_kelvin: float
    delta_t_kelvin: float


@dataclass(frozen=True, slots=True, kw_only=True)
class InputPrescribed:
    """Objeto ``prescribed`` del esquema engine-input/1."""

    node: InputId
    dof: InputDof
    value: float


@dataclass(frozen=True, slots=True, kw_only=True)
class InputDiaphragm:
    """Objeto ``diaphragm`` del esquema engine-input/1."""

    master: InputId
    slaves: list[InputId]


# Alias de la definición ``id`` del esquema engine-input/1.
InputId: TypeAlias = int


@final
class InputDof(StrEnum):
    """Conjunto cerrado de cadenas. Definición ``dof`` del esquema engine-input/1."""

    UX = "Ux"
    UY = "Uy"
    UZ = "Uz"
    RX = "Rx"
    RY = "Ry"
    RZ = "Rz"


@final
class InputMethod(StrEnum):
    """Conjunto cerrado de cadenas. Definición ``method`` del esquema engine-input/1."""

    LINEAR_STATIC = "linearStatic"
    P_DELTA = "pDelta"
    MODAL = "modal"
    RESPONSE_SPECTRUM = "responseSpectrum"
    NONLINEAR_STATIC = "nonlinearStatic"
    SOIL_PROFILE = "soilProfile"
    CONSOLIDATION1D = "consolidation1d"
    BUCKLING_LINEAR = "buckling.linear"


@final
class InputNodeDim(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``dim`` de ``InputNode``."""

    D2 = "D2"
    D3 = "D3"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputNode:
    """Objeto ``node`` del esquema engine-input/1."""

    id: InputId
    dim: InputNodeDim
    coordinates: list[float]
    dofs: list[InputDof]


@dataclass(frozen=True, slots=True, kw_only=True)
class InputMember:
    """Objeto ``member`` del esquema engine-input/1."""

    id: InputId
    nodes: list[InputId]
    section: str
    law: str
    element: str | None = None
    orientation: list[float] | None = None
    release_i: bool | list[bool] | None = None
    release_j: bool | list[bool] | None = None
    winkler: InputWinkler | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputParameter:
    """Objeto ``parameter`` del esquema engine-input/1."""

    name: str
    value: float


@dataclass(frozen=True, slots=True, kw_only=True)
class InputSection:
    """Objeto ``section`` del esquema engine-input/1."""

    id: str
    properties: list[InputParameter]


@final
class InputLawKind(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``kind`` de ``InputLaw``."""

    ELASTIC = "elastic"
    STEEL_BILINEAR_KINEMATIC = "steel.bilinear_kinematic"
    MULTILINEAR_ELASTIC = "multilinear_elastic"
    ELASTIC_CUBIC = "elastic.cubic"


@final
class InputLawMode(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``mode`` de ``InputLaw``."""

    BOTH = "both"
    COMPRESSION_ONLY = "compression_only"
    TENSION_ONLY = "tension_only"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputLaw:
    """Objeto ``law`` del esquema engine-input/1."""

    name: str
    parameters: list[InputParameter]
    kind: InputLawKind | None = None
    points: list[list[float]] | None = None
    mode: InputLawMode | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputSupport:
    """Objeto ``support`` del esquema engine-input/1."""

    id: str
    node: InputId
    restrained: list[InputDof]


@dataclass(frozen=True, slots=True, kw_only=True)
class InputDofValue:
    """Objeto ``dofValue`` del esquema engine-input/1."""

    dof: InputDof
    value: float


@dataclass(frozen=True, slots=True, kw_only=True)
class InputMass:
    """Objeto ``mass`` del esquema engine-input/1."""

    id: str
    node: InputId
    values: list[InputDofValue]


@dataclass(frozen=True, slots=True, kw_only=True)
class InputLoad:
    """Objeto ``load`` del esquema engine-input/1."""

    node: InputId
    values: list[InputDofValue]


@dataclass(frozen=True, slots=True, kw_only=True)
class InputCase:
    """Objeto ``case`` del esquema engine-input/1."""

    name: str
    loads: list[InputLoad]
    member_loads: list[InputUniformLoad] | None = None
    temperature_loads: list[InputUniformTemperature] | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputCombination:
    """Objeto ``combination`` del esquema engine-input/1."""

    id: str
    terms: list[InputTerm]


@dataclass(frozen=True, slots=True, kw_only=True)
class InputTerm:
    """Objeto ``term`` del esquema engine-input/1."""

    case: str
    factor: float


@dataclass(frozen=True, slots=True, kw_only=True)
class LinearStaticAnalysis:
    """Rama de ``Analysis``. El campo method es la constante del esquema."""

    method: Literal["linearStatic"]


@dataclass(frozen=True, slots=True, kw_only=True)
class PDeltaAnalysis:
    """Rama de ``Analysis``. El campo method es la constante del esquema."""

    method: Literal["pDelta"]
    options: InputPDeltaOptions


@dataclass(frozen=True, slots=True, kw_only=True)
class ModalAnalysis:
    """Rama de ``Analysis``. El campo method es la constante del esquema."""

    method: Literal["modal"]
    options: InputModalOptions


@dataclass(frozen=True, slots=True, kw_only=True)
class ResponseSpectrumAnalysis:
    """Rama de ``Analysis``. El campo method es la constante del esquema."""

    method: Literal["responseSpectrum"]
    options: InputSpectrumOptions


@dataclass(frozen=True, slots=True, kw_only=True)
class NonlinearStaticAnalysis:
    """Rama de ``Analysis``. El campo method es la constante del esquema."""

    method: Literal["nonlinearStatic"]
    options: InputNewtonOptions


@dataclass(frozen=True, slots=True, kw_only=True)
class SoilProfileAnalysis:
    """Rama de ``Analysis``. El campo method es la constante del esquema."""

    method: Literal["soilProfile"]
    options: InputSoilOptions


@dataclass(frozen=True, slots=True, kw_only=True)
class Consolidation1dAnalysis:
    """Rama de ``Analysis``. El campo method es la constante del esquema."""

    method: Literal["consolidation1d"]
    options: InputConsolidationOptions


@dataclass(frozen=True, slots=True, kw_only=True)
class BucklingLinearAnalysis:
    """Rama de ``Analysis``. El campo method es la constante del esquema."""

    method: Literal["buckling.linear"]
    options: InputBucklingOptions


# Unión discriminada por method. Definición ``analysis`` del esquema engine-input/1.
Analysis: TypeAlias = (
    LinearStaticAnalysis
    | PDeltaAnalysis
    | ModalAnalysis
    | ResponseSpectrumAnalysis
    | NonlinearStaticAnalysis
    | SoilProfileAnalysis
    | Consolidation1dAnalysis
    | BucklingLinearAnalysis
)


@dataclass(frozen=True, slots=True, kw_only=True)
class InputModel:
    """Objeto ``model`` del esquema engine-input/1."""

    axes: Literal["Z-up"]
    nodes: list[InputNode]
    members: list[InputMember]
    sections: list[InputSection]
    laws: list[InputLaw]
    supports: list[InputSupport]
    masses: list[InputMass]
    prescribed: list[InputPrescribed] | None = None
    diaphragms: list[InputDiaphragm] | None = None
    springs: list[InputSpring] | None = None


@final
class InputMassSourceMethod(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``method`` de ``InputMassSource``."""

    MASSES_FROM_CASE = "masses_from_case"
    SYSTEM_CASE = "system_case"
    MASSES_FROM_SYSTEM_CASE = "masses_from_system_case"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputMassSource:
    """Objeto ``massSource`` del esquema engine-input/1."""

    method: InputMassSourceMethod
    case: str
    factor: float
    direction: list[float]
    gravity: float


@dataclass(frozen=True, slots=True, kw_only=True)
class InputPDeltaOptions:
    """Objeto ``pDeltaOptions`` del esquema engine-input/1."""

    gravity: InputTerm
    lateral: InputTerm
    tolerance: float
    max_iterations: int
    displacement_scale: float
    rotation_scale: float


@final
class InputModalOptionsMassKind(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``mass_kind`` de
    ``InputModalOptions``."""

    LUMPED = "lumped"
    CONSISTENT = "consistent"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputModalOptions:
    """Objeto ``modalOptions`` del esquema engine-input/1."""

    mode_count: int
    mass_kind: InputModalOptionsMassKind
    directions: list[list[float]]
    mass_source: InputMassSource | None = None


@final
class InputSpectrumTableUnits(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``units`` de
    ``InputSpectrumTable``."""

    PERIOD_SECONDS_ACCELERATION_METERS_PER_SECOND_SQUARED = (
        "periodSecondsAccelerationMetersPerSecondSquared"
    )
    FREQUENCY_HERTZ_ACCELERATION_METERS_PER_SECOND_SQUARED = (
        "frequencyHertzAccelerationMetersPerSecondSquared"
    )


@final
class InputSpectrumTableInterpolation(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``interpolation`` de
    ``InputSpectrumTable``."""

    LINEAR = "linear"
    LOG_LOG = "logLog"


@final
class InputSpectrumTableExtrapolation(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``extrapolation`` de
    ``InputSpectrumTable``."""

    ERROR = "error"
    PLATEAU = "plateau"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputSpectrumTable:
    """Objeto ``spectrumTable`` del esquema engine-input/1."""

    pairs: list[list[float]]
    units: InputSpectrumTableUnits
    damping: float
    interpolation: InputSpectrumTableInterpolation
    extrapolation: InputSpectrumTableExtrapolation


@dataclass(frozen=True, slots=True, kw_only=True)
class InputSpectrumDirection:
    """Objeto ``spectrumDirection`` del esquema engine-input/1."""

    direction: list[float]
    spectra: list[InputSpectrumTable]


@final
class InputSpectrumOptionsMassKind(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``mass_kind`` de
    ``InputSpectrumOptions``."""

    LUMPED = "lumped"
    CONSISTENT = "consistent"


@final
class InputSpectrumOptionsModalCombination(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``modal_combination`` de
    ``InputSpectrumOptions``."""

    SRSS = "srss"
    CQC = "cqc"


@final
class InputSpectrumOptionsDirectionalCombination(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``directional_combination`` de
    ``InputSpectrumOptions``."""

    SRSS = "srss"
    WEIGHTED_SUM = "weightedSum"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputSpectrumOptions:
    """Objeto ``spectrumOptions`` del esquema engine-input/1."""

    mode_count: int
    mass_kind: InputSpectrumOptionsMassKind
    directions: list[InputSpectrumDirection]
    modal_combination: InputSpectrumOptionsModalCombination
    directional_combination: InputSpectrumOptionsDirectionalCombination
    directional_factors: list[float]
    mass_source: InputMassSource | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputWinklerK:
    """Rama de la unión ``InputWinkler``."""

    k: float


@dataclass(frozen=True, slots=True, kw_only=True)
class InputWinklerKyKz:
    """Rama de la unión ``InputWinkler``."""

    ky: float
    kz: float


# Unión de la definición ``winkler`` del esquema engine-input/1.
InputWinkler: TypeAlias = InputWinklerK | InputWinklerKyKz


@final
class InputSpringKind(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``kind`` de ``InputSpring``."""

    SPRING_LINEAR = "spring.linear"
    SPRING_NONLINEAR = "spring.nonlinear"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputSpring:
    """Objeto ``spring`` del esquema engine-input/1."""

    id: InputId
    nodes: list[InputId]
    kind: InputSpringKind
    dof: InputDof
    k: float | None = None
    law: str | None = None
    direction: list[float] | None = None
    rotation: list[list[float]] | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputNewtonOptions:
    """Objeto ``newton_options`` del esquema engine-input/1."""

    case: str
    load_factors: list[float]
    tolerance: float
    max_iterations: int
    force_scale: float
    displacement_tolerance: float | None = None
    energy_tolerance: float | None = None
    displacement_scale: float | None = None
    energy_scale: float | None = None


@final
class InputGeoParameterProvenance(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``provenance`` de
    ``InputGeoParameter``."""

    MEASURED = "measured"
    ASSUMED = "assumed"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputGeoParameter:
    """Objeto ``geo_parameter`` del esquema engine-input/1."""

    value: float
    provenance: InputGeoParameterProvenance


@dataclass(frozen=True, slots=True, kw_only=True)
class InputSoilLayer:
    """Objeto ``soil_layer`` del esquema engine-input/1."""

    thickness_m: float
    gamma_kn_m3: InputGeoParameter | None = None
    gamma_sat_kn_m3: InputGeoParameter | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputSoilOptions:
    """Objeto ``soil_options`` del esquema engine-input/1."""

    layers: list[InputSoilLayer]
    depths_m: list[float]
    groundwater_depth_m: InputGeoParameter | None = None
    water_unit_weight_kn_m3: InputGeoParameter | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputConsolidationLayer:
    """Objeto ``consolidation_layer`` del esquema engine-input/1."""

    thickness_m: InputGeoParameter
    elements: int
    cv_m2_s: InputGeoParameter | None = None
    mv_per_kpa: InputGeoParameter | None = None
    k_m_s: InputGeoParameter | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputConsolidationLoad:
    """Objeto ``consolidation_load`` del esquema engine-input/1."""

    time_s: float
    delta_stress_kpa: InputGeoParameter | None = None


@final
class InputConsolidationOptionsDrainage(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``drainage`` de
    ``InputConsolidationOptions``."""

    TOP = "top"
    BOTTOM = "bottom"
    DOUBLE = "double"


@dataclass(frozen=True, slots=True, kw_only=True)
class InputConsolidationOptions:
    """Objeto ``consolidation_options`` del esquema engine-input/1."""

    layers: list[InputConsolidationLayer]
    drainage: InputConsolidationOptionsDrainage
    history: list[InputConsolidationLoad]
    output_times_s: list[float]
    theta: float
    max_step_s: float
    water_unit_weight_kn_m3: InputGeoParameter | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class InputBucklingOptions:
    """Objeto ``buckling_options`` del esquema engine-input/1."""

    mode_count: int
    reference: InputTerm


@dataclass(frozen=True, slots=True, kw_only=True)
class EngineInputDocumentOptions:
    """Objeto de la propiedad ``options`` en ``EngineInputDocument``."""

    allow_experimental: bool = False


@dataclass(frozen=True, slots=True, kw_only=True)
class EngineInputDocument:
    """Documento raíz del esquema engine-input/1."""

    schema: Literal["engine-input/1"]
    units: Literal["SI"]
    model: InputModel
    cases: list[InputCase]
    combinations: list[InputCombination]
    analyses: list[Analysis]
    options: EngineInputDocumentOptions


# results/1
# ---------


@final
class ResultNodeIdentityDim(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``dim`` de ``ResultNodeIdentity``."""

    D2 = "D2"
    D3 = "D3"


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultNodeIdentity:
    """Objeto ``nodeIdentity`` del esquema results/1."""

    id: ResultId
    dim: ResultNodeIdentityDim
    coordinates: list[float]
    dofs: list[ResultDof]


@final
class ResultDof(StrEnum):
    """Conjunto cerrado de cadenas. Definición ``dof`` del esquema results/1."""

    UX = "Ux"
    UY = "Uy"
    UZ = "Uz"
    RX = "Rx"
    RY = "Ry"
    RZ = "Rz"


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultMemberIdentity:
    """Objeto ``memberIdentity`` del esquema results/1."""

    id: ResultId
    nodes: list[ResultId]
    element: str
    section: str
    law: str
    end_dofs: list[str] | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultCombination:
    """Objeto ``combination`` del esquema results/1."""

    id: str
    results: ResultCase


# Alias de la definición ``id`` del esquema results/1.
ResultId: TypeAlias = int


# Alias de la definición ``vector6`` del esquema results/1.
ResultVector6: TypeAlias = list[float]


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultNode:
    """Objeto ``node`` del esquema results/1."""

    id: ResultId
    displacements: ResultVector6
    prescribed_displacements: list[float | None]
    reactions: ResultVector6


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultStation:
    """Objeto ``station`` del esquema results/1."""

    position: float
    forces_local: ResultVector6


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultExtreme:
    """Objeto ``extreme`` del esquema results/1."""

    value: float
    at_station: float


@final
class ResultFieldExtremesField(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``field`` de
    ``ResultFieldExtremes``."""

    N = "N"
    VY = "Vy"
    VZ = "Vz"
    T = "T"
    MY = "My"
    MZ = "Mz"


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultFieldExtremes:
    """Objeto ``fieldExtremes`` del esquema results/1."""

    field: ResultFieldExtremesField
    max: ResultExtreme
    min: ResultExtreme


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultMember:
    """Objeto ``member`` del esquema results/1."""

    id: ResultId
    end_forces_local: list[float]
    stations: list[ResultStation]
    extremes: list[ResultFieldExtremes]


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultCase:
    """Objeto ``case`` del esquema results/1."""

    case: str
    factor: float
    nodes: list[ResultNode]
    members: list[ResultMember]


@final
class ResultEntityKind(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``kind`` de ``ResultEntity``."""

    NODE = "node"
    MEMBER = "member"
    DOF = "dof"
    CASE = "case"
    LAW = "law"


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultEntity:
    """Objeto ``entity`` del esquema results/1."""

    kind: ResultEntityKind
    id: int | str


@final
class ResultDiagnosticSeverity(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``severity`` de
    ``ResultDiagnostic``."""

    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultDiagnostic:
    """Objeto ``diagnostic`` del esquema results/1."""

    code: str
    severity: ResultDiagnosticSeverity
    entities: list[ResultEntity]
    cause: str
    hint: str | None


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultParticipation:
    """Objeto ``participation`` del esquema results/1."""

    direction: list[float]
    total_mass: float
    free_mass: float
    factor: float
    effective_mass: float
    cumulative_effective_mass: float
    mass_fraction: float | None
    cumulative_mass_fraction: float | None


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultSpectrumResponse:
    """Objeto ``spectrumResponse`` del esquema results/1."""

    direction: list[float]
    damping: float
    spectral_acceleration: float
    modal_displacement: float
    equivalent_forces: list[float]
    nodes: list[ResultNode]
    base_shear: float


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultMode:
    """Objeto ``mode`` del esquema results/1."""

    mode: int
    eigenvalue: float
    angular_frequency: float
    frequency_hz: float
    period: float | None
    normalization: Literal["mass"]
    shape: list[ResultNode]
    participation: list[ResultParticipation]
    spectrum_responses: list[ResultSpectrumResponse]


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultEnvelope:
    """Objeto ``envelope`` del esquema results/1."""

    direction: None | list[float]
    factor: float | None
    nodes: list[ResultNode]
    base_shear: float


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultCaseFactor:
    """Objeto ``caseFactor`` del esquema results/1."""

    case: str
    factor: float


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultPDeltaIteration:
    """Objeto ``pDeltaIteration`` del esquema results/1."""

    iteration: int
    relative_displacement_change: float
    free_residual_norm: float
    relative_free_residual: float
    residual_convergence_ratio: float | None = None


@final
class ResultPDeltaConvergenceStatus(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``status`` de
    ``ResultPDeltaConvergence``."""

    OK = "ok"
    NOT_CONVERGED = "notConverged"
    UNSTABLE = "unstable"


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultPDeltaConvergence:
    """Objeto ``pDeltaConvergence`` del esquema results/1."""

    method: Literal["pDelta"]
    status: ResultPDeltaConvergenceStatus
    gravity: ResultCaseFactor
    lateral: ResultCaseFactor
    tolerance: float
    max_iterations: int
    displacement_scale: float
    rotation_scale: float
    inertia: list[int]
    inertia_uses_2x2: bool | None
    iterations: list[ResultPDeltaIteration]


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultNewtonStep:
    """Objeto ``newton_step`` del esquema results/1."""

    load_factor: float
    iterations: int
    converged: bool
    force_norm: float
    displacement_norm: float
    energy_norm: float
    relative_force_norm: float
    relative_displacement_norm: float
    relative_energy_norm: float
    results: None | ResultCase
    element_states: None | list[list[float]]


@final
class ResultNewtonConvergenceStatus(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``status`` de
    ``ResultNewtonConvergence``."""

    OK = "ok"
    NOT_CONVERGED = "notConverged"
    UNSTABLE = "unstable"


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultNewtonConvergence:
    """Objeto ``newton_convergence`` del esquema results/1."""

    method: Literal["nonlinearStatic"]
    case: str
    status: ResultNewtonConvergenceStatus
    accepted_factor: float | None
    tolerance: float
    max_iterations: int
    force_scale: float
    displacement_tolerance: float | None
    energy_tolerance: float | None
    displacement_scale: float
    energy_scale: float
    steps: list[ResultNewtonStep]
    failed_factor: float | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultSoilState:
    """Objeto ``soil_state`` del esquema results/1."""

    depth_m: float
    total_vertical_kpa: float
    pore_pressure_kpa: float
    effective_vertical_kpa: float
    assumed_parameters: list[str]


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultSoilResult:
    """Objeto ``soil_result`` del esquema results/1."""

    method: Literal["soilProfile"]
    states: list[ResultSoilState]


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultConsolidationState:
    """Objeto ``consolidation_state`` del esquema results/1."""

    time_s: float
    delta_stress_kpa: float
    excess_pore_pressure_kpa: list[float]
    settlement_m: float
    mean_degree: float | None


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultConsolidationResult:
    """Objeto ``consolidation_result`` del esquema results/1."""

    method: Literal["consolidation1d"]
    depths_m: list[float]
    assumed_parameters: list[str]
    states: list[ResultConsolidationState]


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultBucklingMode:
    """Objeto ``buckling_mode`` del esquema results/1."""

    mode: int
    eigenvalue: float
    normalization: Literal["negative_kg"]
    shape: list[ResultNode]


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultBucklingResult:
    """Objeto ``buckling_result`` del esquema results/1."""

    rigid_mode_count: int
    modes: list[ResultBucklingMode]


@final
class ResultsDocumentMethod(StrEnum):
    """Conjunto cerrado de cadenas de cada elemento de la propiedad ``methods`` de
    ``ResultsDocument``."""

    LINEAR_STATIC = "linearStatic"
    P_DELTA = "pDelta"
    MODAL = "modal"
    RESPONSE_SPECTRUM = "responseSpectrum"
    NONLINEAR_STATIC = "nonlinearStatic"
    SOIL_PROFILE = "soilProfile"
    CONSOLIDATION1D = "consolidation1d"
    BUCKLING_LINEAR = "buckling.linear"


@final
class ResultsDocumentStatus(StrEnum):
    """Conjunto cerrado de cadenas de la propiedad ``status`` de ``ResultsDocument``."""

    OK = "ok"
    INVALID = "invalid"
    UNSUPPORTED = "unsupported"
    CANCELLED = "cancelled"
    NOT_CONVERGED = "notConverged"
    UNSTABLE = "unstable"
    WARNING = "warning"
    TIMEOUT = "timeout"
    INTERNAL_ERROR = "internalError"


@dataclass(frozen=True, slots=True, kw_only=True)
class ResultsDocument:
    """Documento raíz del esquema results/1."""

    schema: Literal["results/1"]
    engine_version: str
    contracts: list[str]
    methods: list[ResultsDocumentMethod]
    input_sha256: str | None
    status: ResultsDocumentStatus
    nodes: list[ResultNodeIdentity]
    members: list[ResultMemberIdentity]
    modes: list[ResultMode]
    cases: list[ResultCase]
    envelopes: list[ResultEnvelope]
    combinations: list[ResultCombination]
    convergence:     (
        None
        | ResultPDeltaConvergence
        | ResultNewtonConvergence
    )
    diagnostics: list[ResultDiagnostic]
    git_commit: str | None = None
    target: str | None = None
    geotechnical: ResultSoilResult | ResultConsolidationResult | None = None
    buckling: ResultBucklingResult | None = None


__all__ = [
    "InputUniformLoad",
    "InputUniformTemperature",
    "InputPrescribed",
    "InputDiaphragm",
    "InputId",
    "InputDof",
    "InputMethod",
    "InputNode",
    "InputNodeDim",
    "InputMember",
    "InputParameter",
    "InputSection",
    "InputLaw",
    "InputLawKind",
    "InputLawMode",
    "InputSupport",
    "InputDofValue",
    "InputMass",
    "InputLoad",
    "InputCase",
    "InputCombination",
    "InputTerm",
    "Analysis",
    "LinearStaticAnalysis",
    "PDeltaAnalysis",
    "ModalAnalysis",
    "ResponseSpectrumAnalysis",
    "NonlinearStaticAnalysis",
    "SoilProfileAnalysis",
    "Consolidation1dAnalysis",
    "BucklingLinearAnalysis",
    "InputModel",
    "InputMassSource",
    "InputMassSourceMethod",
    "InputPDeltaOptions",
    "InputModalOptions",
    "InputModalOptionsMassKind",
    "InputSpectrumTable",
    "InputSpectrumTableUnits",
    "InputSpectrumTableInterpolation",
    "InputSpectrumTableExtrapolation",
    "InputSpectrumDirection",
    "InputSpectrumOptions",
    "InputSpectrumOptionsMassKind",
    "InputSpectrumOptionsModalCombination",
    "InputSpectrumOptionsDirectionalCombination",
    "InputWinkler",
    "InputWinklerK",
    "InputWinklerKyKz",
    "InputSpring",
    "InputSpringKind",
    "InputNewtonOptions",
    "InputGeoParameter",
    "InputGeoParameterProvenance",
    "InputSoilLayer",
    "InputSoilOptions",
    "InputConsolidationLayer",
    "InputConsolidationLoad",
    "InputConsolidationOptions",
    "InputConsolidationOptionsDrainage",
    "InputBucklingOptions",
    "EngineInputDocument",
    "EngineInputDocumentOptions",
    "ResultNodeIdentity",
    "ResultNodeIdentityDim",
    "ResultDof",
    "ResultMemberIdentity",
    "ResultCombination",
    "ResultId",
    "ResultVector6",
    "ResultNode",
    "ResultStation",
    "ResultExtreme",
    "ResultFieldExtremes",
    "ResultFieldExtremesField",
    "ResultMember",
    "ResultCase",
    "ResultEntity",
    "ResultEntityKind",
    "ResultDiagnostic",
    "ResultDiagnosticSeverity",
    "ResultParticipation",
    "ResultSpectrumResponse",
    "ResultMode",
    "ResultEnvelope",
    "ResultCaseFactor",
    "ResultPDeltaIteration",
    "ResultPDeltaConvergence",
    "ResultPDeltaConvergenceStatus",
    "ResultNewtonStep",
    "ResultNewtonConvergence",
    "ResultNewtonConvergenceStatus",
    "ResultSoilState",
    "ResultSoilResult",
    "ResultConsolidationState",
    "ResultConsolidationResult",
    "ResultBucklingMode",
    "ResultBucklingResult",
    "ResultsDocument",
    "ResultsDocumentMethod",
    "ResultsDocumentStatus",
]
