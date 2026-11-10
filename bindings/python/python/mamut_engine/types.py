"""DTO públicos de engine-input/1 y results/1, derivados de los schemas propios.

Las colecciones conservan identidad y orden. Campos opcionales se omiten; no
se rellenan constantes físicas. Los documentos de runtime preservan extensiones.
"""
from __future__ import annotations

from typing import Literal, TypeAlias, TypedDict

JSONValue: TypeAlias = "None | bool | int | float | str | list[JSONValue] | dict[str, JSONValue]"
JSONDocument: TypeAlias = dict[str, JSONValue]

class InputUniformLoad(TypedDict):
    """Campos requeridos del contrato InputUniformLoad."""
    member: InputId
    force_per_length: list[float]
    moment_per_length: list[float]

class InputUniform_temperature(TypedDict):
    """Campos requeridos del contrato InputUniform_temperature."""
    member: InputId
    alpha_per_kelvin: float
    delta_t_kelvin: float

class InputPrescribed(TypedDict):
    """Campos requeridos del contrato InputPrescribed."""
    node: InputId
    dof: InputDof
    value: float

class InputDiaphragm(TypedDict):
    """Campos requeridos del contrato InputDiaphragm."""
    master: InputId
    slaves: list[InputId]

InputId: TypeAlias = int

InputDof: TypeAlias = Literal['Ux', 'Uy', 'Uz', 'Rx', 'Ry', 'Rz']

InputMethod: TypeAlias = Literal['linearStatic', 'pDelta', 'modal', 'responseSpectrum', 'nonlinearStatic', 'soilProfile', 'consolidation1d', 'buckling.linear']

class InputNode(TypedDict):
    """Campos requeridos del contrato InputNode."""
    id: InputId
    dim: Literal['D2', 'D3']
    coordinates: list[float]
    dofs: list[InputDof]

class InputMemberRequired(TypedDict):
    """Campos requeridos del contrato InputMember."""
    id: InputId
    nodes: list[InputId]
    section: str
    law: str

class InputMember(InputMemberRequired, total=False):
    """Campos opcionales, sin valores físicos implícitos."""
    element: str
    orientation: list[float]
    release_i: bool | list[bool]
    release_j: bool | list[bool]
    winkler: InputWinkler

class InputParameter(TypedDict):
    """Campos requeridos del contrato InputParameter."""
    name: str
    value: float

class InputSection(TypedDict):
    """Campos requeridos del contrato InputSection."""
    id: str
    properties: list[InputParameter]

class InputLawRequired(TypedDict):
    """Campos requeridos del contrato InputLaw."""
    name: str
    parameters: list[InputParameter]

class InputLaw(InputLawRequired, total=False):
    """Selección opcional de ley uniaxial."""
    kind: Literal["elastic", "steel.bilinear_kinematic", "multilinear_elastic", "elastic.cubic"]
    points: list[list[float]]
    mode: Literal["both", "compression_only", "tension_only"]

class InputSupport(TypedDict):
    """Campos requeridos del contrato InputSupport."""
    id: str
    node: InputId
    restrained: list[InputDof]

class InputDofValue(TypedDict):
    """Campos requeridos del contrato InputDofValue."""
    dof: InputDof
    value: float

class InputMass(TypedDict):
    """Campos requeridos del contrato InputMass."""
    id: str
    node: InputId
    values: list[InputDofValue]

class InputLoad(TypedDict):
    """Campos requeridos del contrato InputLoad."""
    node: InputId
    values: list[InputDofValue]

class InputCaseRequired(TypedDict):
    """Campos requeridos del contrato InputCase."""
    name: str
    loads: list[InputLoad]

class InputCase(InputCaseRequired, total=False):
    """Campos opcionales, sin valores físicos implícitos."""
    member_loads: list[InputUniformLoad]
    temperature_loads: list[InputUniform_temperature]

class InputCombination(TypedDict):
    """Campos requeridos del contrato InputCombination."""
    id: str
    terms: list[InputTerm]

class InputTerm(TypedDict):
    """Campos requeridos del contrato InputTerm."""
    case: str
    factor: float

class InputBuckling_options(TypedDict):
    """Campos requeridos del contrato InputBuckling_options."""
    mode_count: int
    reference: InputTerm

InputAnalysis: TypeAlias = "LinearStaticAnalysis | PDeltaAnalysis | ModalAnalysis | SpectrumAnalysis | NonlinearStaticAnalysis | SoilProfileAnalysis | Consolidation1dAnalysis | BucklingLinearAnalysis"

class InputModelRequired(TypedDict):
    """Campos requeridos del contrato InputModel."""
    axes: Literal['Z-up']
    nodes: list[InputNode]
    members: list[InputMember]
    sections: list[InputSection]
    laws: list[InputLaw]
    supports: list[InputSupport]
    masses: list[InputMass]

class InputModel(InputModelRequired, total=False):
    """Campos opcionales, sin valores físicos implícitos."""
    prescribed: list[InputPrescribed]
    diaphragms: list[InputDiaphragm]
    springs: list[InputSpring]

class InputMassSource(TypedDict):
    """Campos requeridos del contrato InputMassSource."""
    method: Literal['masses_from_case', 'system_case', 'masses_from_system_case']
    case: str
    factor: float
    direction: list[float]
    gravity: float

class InputPDeltaOptions(TypedDict):
    """Campos requeridos del contrato InputPDeltaOptions."""
    gravity: InputTerm
    lateral: InputTerm
    tolerance: float
    max_iterations: int
    displacement_scale: float
    rotation_scale: float

class InputModalOptionsRequired(TypedDict):
    """Campos requeridos del contrato InputModalOptions."""
    mode_count: int
    mass_kind: Literal['lumped', 'consistent']
    directions: list[list[float]]

class InputModalOptions(InputModalOptionsRequired, total=False):
    """Campos opcionales, sin valores físicos implícitos."""
    mass_source: InputMassSource

class InputSpectrumTable(TypedDict):
    """Campos requeridos del contrato InputSpectrumTable."""
    pairs: list[list[float]]
    units: Literal['periodSecondsAccelerationMetersPerSecondSquared', 'frequencyHertzAccelerationMetersPerSecondSquared']
    damping: float
    interpolation: Literal['linear', 'logLog']
    extrapolation: Literal['error', 'plateau']

class InputSpectrumDirection(TypedDict):
    """Campos requeridos del contrato InputSpectrumDirection."""
    direction: list[float]
    spectra: list[InputSpectrumTable]

class InputSpectrumOptionsRequired(TypedDict):
    """Campos requeridos del contrato InputSpectrumOptions."""
    mode_count: int
    mass_kind: Literal['lumped', 'consistent']
    directions: list[InputSpectrumDirection]
    modal_combination: Literal['srss', 'cqc']
    directional_combination: Literal['srss', 'weightedSum']
    directional_factors: list[float]

class InputSpectrumOptions(InputSpectrumOptionsRequired, total=False):
    """Campos opcionales, sin valores físicos implícitos."""
    mass_source: InputMassSource

class LinearStaticAnalysis(TypedDict):
    """Selección explícita del análisis linearStatic."""
    method: Literal['linearStatic']

class PDeltaAnalysis(TypedDict):
    """Selección explícita del análisis pDelta."""
    method: Literal['pDelta']
    options: InputPDeltaOptions

class ModalAnalysis(TypedDict):
    """Selección explícita del análisis modal."""
    method: Literal['modal']
    options: InputModalOptions

class SpectrumAnalysis(TypedDict):
    """Selección explícita del análisis responseSpectrum."""
    method: Literal['responseSpectrum']
    options: InputSpectrumOptions

class InputOptions(TypedDict, total=False):
    """Opciones del registro; pDelta Stable no requiere consentimiento."""
    allow_experimental: bool

class ModelDocument(TypedDict):
    """Campos requeridos del contrato ModelDocument."""
    schema: Literal['engine-input/1']
    units: Literal['SI']
    model: InputModel
    cases: list[InputCase]
    combinations: list[InputCombination]
    analyses: list[InputAnalysis]
    options: InputOptions

class ResultNodeIdentity(TypedDict):
    """Campos requeridos del contrato ResultNodeIdentity."""
    id: ResultId
    dim: Literal['D2', 'D3']
    coordinates: list[float]
    dofs: list[ResultDof]

ResultDof: TypeAlias = Literal['Ux', 'Uy', 'Uz', 'Rx', 'Ry', 'Rz']

class ResultMemberIdentityRequired(TypedDict):
    """Campos requeridos del contrato ResultMemberIdentity."""
    id: ResultId
    nodes: list[ResultId]
    element: str
    section: str
    law: str

class ResultMemberIdentity(ResultMemberIdentityRequired, total=False):
    """Etiquetas opcionales de recuperación de resortes."""
    end_dofs: list[str]

class ResultCombination(TypedDict):
    """Campos requeridos del contrato ResultCombination."""
    id: str
    results: ResultCase

ResultId: TypeAlias = int

ResultVector6: TypeAlias = list[float]

class ResultNode(TypedDict):
    """Campos requeridos del contrato ResultNode."""
    id: ResultId
    displacements: ResultVector6
    prescribed_displacements: list[float | None]
    reactions: ResultVector6

class ResultStation(TypedDict):
    """Campos requeridos del contrato ResultStation."""
    position: float
    forces_local: ResultVector6

class ResultExtreme(TypedDict):
    """Campos requeridos del contrato ResultExtreme."""
    value: float
    at_station: float

class ResultFieldExtremes(TypedDict):
    """Campos requeridos del contrato ResultFieldExtremes."""
    field: Literal['N', 'Vy', 'Vz', 'T', 'My', 'Mz']
    max: ResultExtreme
    min: ResultExtreme

class ResultMember(TypedDict):
    """Campos requeridos del contrato ResultMember."""
    id: ResultId
    end_forces_local: list[float]
    stations: list[ResultStation]
    extremes: list[ResultFieldExtremes]

class ResultCase(TypedDict):
    """Campos requeridos del contrato ResultCase."""
    case: str
    factor: float
    nodes: list[ResultNode]
    members: list[ResultMember]

class ResultEntity(TypedDict):
    """Campos requeridos del contrato ResultEntity."""
    kind: Literal['node', 'member', 'dof', 'case', 'law']
    id: int | str

class ResultDiagnosticRequired(TypedDict):
    """Campos requeridos del contrato ResultDiagnostic."""
    code: str
    severity: Literal['info', 'warning', 'error']
    entities: list[ResultEntity]
    cause: str
    hint: str | None

class ResultDiagnostic(ResultDiagnosticRequired, total=False):
    """Contexto opcional de una referencia ausente.

    Estas claves no están en los diagnósticos de otro código. Un documento
    anterior, sin ellas, sigue siendo válido.
    """
    entity_kind: str
    missing_id: int | str
    referenced_by: str
    suggestions: list[str]

class ResultParticipation(TypedDict):
    """Campos requeridos del contrato ResultParticipation."""
    direction: list[float]
    total_mass: float
    free_mass: float
    factor: float
    effective_mass: float
    cumulative_effective_mass: float
    mass_fraction: float | None
    cumulative_mass_fraction: float | None

class ResultSpectrumResponse(TypedDict):
    """Campos requeridos del contrato ResultSpectrumResponse."""
    direction: list[float]
    damping: float
    spectral_acceleration: float
    modal_displacement: float
    equivalent_forces: list[float]
    nodes: list[ResultNode]
    base_shear: float

class ResultMode(TypedDict):
    """Campos requeridos del contrato ResultMode."""
    mode: int
    eigenvalue: float
    angular_frequency: float
    frequency_hz: float
    period: float | None
    normalization: Literal['mass']
    shape: list[ResultNode]
    participation: list[ResultParticipation]
    spectrum_responses: list[ResultSpectrumResponse]

class ResultEnvelope(TypedDict):
    """Campos requeridos del contrato ResultEnvelope."""
    direction: None | list[float]
    factor: float | None
    nodes: list[ResultNode]
    base_shear: float

class ResultCaseFactor(TypedDict):
    """Campos requeridos del contrato ResultCaseFactor."""
    case: str
    factor: float

class ResultPDeltaIterationRequired(TypedDict):
    """Campos requeridos del contrato ResultPDeltaIteration."""
    iteration: int
    relative_displacement_change: float
    free_residual_norm: float
    relative_free_residual: float

class ResultPDeltaIteration(ResultPDeltaIterationRequired, total=False):
    """Criterio por GDL añadido en MAM-581; ausente en resultados anteriores."""
    residual_convergence_ratio: float

class ResultPDeltaConvergence(TypedDict):
    """Campos requeridos del contrato ResultPDeltaConvergence."""
    method: Literal['pDelta']
    status: Literal['ok', 'notConverged', 'unstable']
    gravity: ResultCaseFactor
    lateral: ResultCaseFactor
    tolerance: float
    max_iterations: int
    displacement_scale: float
    rotation_scale: float
    inertia: list[int]
    inertia_uses_2x2: bool | None
    iterations: list[ResultPDeltaIteration]

class ResultsDocumentRequired(TypedDict):
    """Campos requeridos del contrato ResultsDocument."""
    schema: Literal['results/1']
    engine_version: str
    contracts: list[str]
    methods: list[Literal['linearStatic', 'pDelta', 'modal', 'responseSpectrum', 'nonlinearStatic', 'soilProfile', 'consolidation1d', 'buckling.linear']]
    input_sha256: str | None
    status: Literal['ok', 'invalid', 'unsupported', 'cancelled', 'notConverged', 'unstable', 'warning', 'timeout', 'internalError']
    nodes: list[ResultNodeIdentity]
    members: list[ResultMemberIdentity]
    modes: list[ResultMode]
    cases: list[ResultCase]
    envelopes: list[ResultEnvelope]
    convergence: None | ResultPDeltaConvergence | ResultNewton_convergence
    diagnostics: list[ResultDiagnostic]
    combinations: list[ResultCombination]

class ResultsDocument(ResultsDocumentRequired, total=False):
    """Procedencia opcional para admitir documentos results/1 anteriores."""
    git_commit: str | None
    target: str
    geotechnical: ResultSoil_result | ResultConsolidation_result
    buckling: ResultBuckling_result

class VersionInfo(TypedDict):
    """Versión del motor y contratos publicados."""
    engine_version: str
    contracts: list[str]

class MethodCapability(TypedDict):
    """Disponibilidad del método en el dominio y en run."""
    engine: bool
    run: bool

class MethodQuantity(TypedDict):
    """Magnitud declarada por el registro de métodos Rust."""
    name: str
    dimension: str
    unit: str

class MethodParameter(TypedDict):
    """Parámetro declarado por el registro de métodos Rust."""
    name: str
    required: bool
    dimension: str
    unit: str

class MethodDimensions(TypedDict):
    """Entradas y salidas del descriptor del método."""
    inputs: list[MethodQuantity]
    outputs: list[MethodQuantity]

class WorkflowMethodRequired(TypedDict):
    """Descriptor SDK transportado por engine-api, sin otra API de workflow."""
    method_id: str
    method_version: int
    capability_slot: str
    dimensions: MethodDimensions
    consumes: list[str]
    produces: list[str]
    parameters: list[MethodParameter]

class WorkflowMethod(WorkflowMethodRequired, total=False):
    """Contexto opcional de fallo publicado por el registro."""
    failure: str

class MethodDescriptor(TypedDict):
    """Contrato method-descriptor/1 y evidencia original del motor."""
    schema: Literal['method-descriptor/1']
    workflow: WorkflowMethod
    status: Literal['Experimental', 'Stable', 'Validated', 'Deprecated']
    input_schema: Literal['engine-input/1', 'results/1']
    output_schema: Literal['results/1']
    fixtures: list[str]
    citations: list[str]
    provenance: list[str]

class SchemaCapability(TypedDict, total=False):
    """Operaciones disponibles para un contrato."""
    validate: bool
    emit: bool

class CombinationCapability(TypedDict):
    """Disponibilidad del postproceso de combinaciones."""
    run: bool

class CapabilitiesRequired(TypedDict):
    """Capacidades reales del motor instalado."""
    run: bool
    operations: list[str]
    schemas: dict[str, SchemaCapability]
    validation: Literal['topology']
    combinations: CombinationCapability
    elements: list[str]
    methods: dict[str, MethodCapability]
    method_descriptors: list[MethodDescriptor]

class Capabilities(CapabilitiesRequired, total=False):
    """Error opcional si Rust no pudo construir su registro."""
    registry_error: str
    laws: list[str]
    end_releases: list[str]

class NodeDisplacement(TypedDict):
    """Desplazamiento nodal en el orden de results/1."""
    id: int
    displacements: list[float]

class CaseDisplacements(TypedDict):
    """Desplazamientos de un caso de carga."""
    case: str
    nodes: list[NodeDisplacement]

class Progress(TypedDict):
    """Evento diferido; completed y total cuentan etapas del motor."""
    operation: str
    stage: str
    completed: int
    total: int

class ValidationReport(TypedDict):
    """Diagnósticos topológicos, sin comprobar estabilidad."""
    diagnostics: list[ResultDiagnostic]

class InputWinklerK(TypedDict):
    """Campos requeridos de la rama winkler con k."""
    k: float


class InputWinklerKyKz(TypedDict):
    """Campos requeridos de la rama winkler con ky y kz."""
    ky: float
    kz: float


InputWinkler: TypeAlias = InputWinklerK | InputWinklerKyKz

class InputSpringRequired(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    id: InputId
    nodes: list[InputId]
    kind: Literal['spring.linear', 'spring.nonlinear']
    dof: InputDof

class InputSpring(InputSpringRequired, total=False):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    k: float
    law: str
    direction: list[float]
    rotation: list[list[float]]


class InputNewton_optionsRequired(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    case: str
    load_factors: list[float]
    tolerance: float
    max_iterations: int
    force_scale: float

class InputNewton_options(InputNewton_optionsRequired, total=False):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    displacement_tolerance: float
    energy_tolerance: float
    displacement_scale: float
    energy_scale: float


class InputGeo_parameter(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    value: float
    provenance: Literal['measured', 'assumed']


class InputSoil_layerRequired(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    thickness_m: float

class InputSoil_layer(InputSoil_layerRequired, total=False):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    gamma_kn_m3: InputGeo_parameter
    gamma_sat_kn_m3: InputGeo_parameter


class InputSoil_optionsRequired(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    layers: list[InputSoil_layer]
    depths_m: list[float]

class InputSoil_options(InputSoil_optionsRequired, total=False):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    groundwater_depth_m: InputGeo_parameter
    water_unit_weight_kn_m3: InputGeo_parameter


class InputConsolidation_layerRequired(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    thickness_m: InputGeo_parameter
    elements: int

class InputConsolidation_layer(InputConsolidation_layerRequired, total=False):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    cv_m2_s: InputGeo_parameter
    mv_per_kpa: InputGeo_parameter
    k_m_s: InputGeo_parameter


class InputConsolidation_loadRequired(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    time_s: float

class InputConsolidation_load(InputConsolidation_loadRequired, total=False):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    delta_stress_kpa: InputGeo_parameter


class InputConsolidation_optionsRequired(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    layers: list[InputConsolidation_layer]
    drainage: Literal['top', 'bottom', 'double']
    history: list[InputConsolidation_load]
    output_times_s: list[float]
    theta: float
    max_step_s: float

class InputConsolidation_options(InputConsolidation_optionsRequired, total=False):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    water_unit_weight_kn_m3: InputGeo_parameter


class ResultNewton_step(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
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


class ResultNewton_convergenceRequired(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    method: Literal['nonlinearStatic']
    case: str
    status: Literal['ok', 'notConverged', 'unstable']
    accepted_factor: float | None
    tolerance: float
    max_iterations: int
    force_scale: float
    displacement_tolerance: float | None
    energy_tolerance: float | None
    displacement_scale: float
    energy_scale: float
    steps: list[ResultNewton_step]


class ResultNewton_convergence(ResultNewton_convergenceRequired, total=False):
    """Factor opcional de un intento con tangente singular."""
    failed_factor: float

class ResultSoil_state(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    depth_m: float
    total_vertical_kpa: float
    pore_pressure_kpa: float
    effective_vertical_kpa: float
    assumed_parameters: list[str]


class ResultSoil_result(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    method: Literal['soilProfile']
    states: list[ResultSoil_state]


class ResultConsolidation_state(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    time_s: float
    delta_stress_kpa: float
    excess_pore_pressure_kpa: list[float]
    settlement_m: float
    mean_degree: float | None


class ResultConsolidation_result(TypedDict):
    """Campos del contrato SFC-010; unidades según el esquema publicado."""
    method: Literal['consolidation1d']
    depths_m: list[float]
    assumed_parameters: list[str]
    states: list[ResultConsolidation_state]


class NonlinearStaticAnalysis(TypedDict):
    """Selección del análisis SFC-010."""
    method: Literal["nonlinearStatic"]
    options: InputNewton_options

class SoilProfileAnalysis(TypedDict):
    """Selección del análisis SFC-010."""
    method: Literal["soilProfile"]
    options: InputSoil_options

class Consolidation1dAnalysis(TypedDict):
    """Selección del análisis SFC-010."""
    method: Literal["consolidation1d"]
    options: InputConsolidation_options

class ResultBuckling_mode(TypedDict):
    """Un factor de K + lambda Kg = 0 y su forma global."""
    mode: int
    eigenvalue: float
    normalization: Literal["negative_kg"]
    shape: list[ResultNode]

class ResultBuckling_result(TypedDict):
    """Factores finitos más bajos. La página de Timoshenko-Gere sigue desconocida."""
    rigid_mode_count: int
    modes: list[ResultBuckling_mode]

class BucklingLinearAnalysis(TypedDict):
    """Selección explícita del análisis buckling.linear."""
    method: Literal["buckling.linear"]
    options: InputBuckling_options
