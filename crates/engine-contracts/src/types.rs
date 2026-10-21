//! Custom exchange data; without a JSON parser or external dependency.
use crate::*;
use std::{collections::BTreeMap, sync::Arc};

/// Minimal JSON value constructed in memory, without a parser or serializer.
///
/// Finite numbers required when each consumer validates; ordered objects and
/// no duplicate keys. Schemas are supplied as JSON String. The serialization
/// boundary and complete results schema are left to other tickets.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Real number (its unit is declared by the field).
    Number(Real),
    /// Unicode text.
    String(String),
    /// Ordered sequence.
    Array(Vec<JsonValue>),
    /// Object with unique keys.
    Object(BTreeMap<String, JsonValue>),
}

/// Trial uniaxial response, preserving the committed state intact.
#[derive(Clone, Debug, PartialEq)]
pub struct UniaxialResponse {
    /// Stress in Pa, or force in N for a spring law.
    pub stress: Real,
    /// Consistent derivative: Pa, or N/m for a spring.
    pub tangent: Real,
    /// Finite trial history of length state_len; units according to the law.
    pub trial_state: Vec<Real>,
}
/// Generalized section response.
#[derive(Clone, Debug, PartialEq)]
pub struct SectionResponse {
    /// D2: [N, Mz]; D3: [N, My, Mz, T], in N and N·m.
    pub resultants: Vec<Real>,
    /// Derivative of resultants with respect to strains; square and symmetric v1.
    pub tangent: DMat,
    /// Finite flat history, length declared by the section.
    pub trial_state: Vec<Real>,
}
/// Resolved parameters: SI properties and optional constitutive references.
///
/// Each kind documents keys and requirements; no automatic validation.
#[derive(Clone, Default)]
pub struct ElementParams {
    /// SI scalar properties with custom names (for example area in m²).
    pub scalars: BTreeMap<String, Real>,
    /// Already resolved section, if the element needs it.
    pub section: Option<Arc<dyn SectionLaw>>,
    /// Already resolved law, if the element needs it.
    pub law: Option<Arc<dyn UniaxialLaw>>,
}
/// Minimal borrowed view for factories; neither assembles nor modifies engine-model.
pub struct ModelView<'a> {
    /// Dimension that the requested element must support.
    pub dim: Dim,
    /// Available identifiers and SI coordinates; unique and finite.
    pub nodes: &'a [(NodeId, [Real; 3])],
}
/// Borrowed element context, independent of trial state.
pub struct ElementCtx<'a> {
    /// Global dimension; D2 in X-Y.
    pub dim: Dim,
    /// Coordinates in m, per node; D2 requires Z = 0.
    pub coords: &'a [[Real; 3]],
    /// Previously resolved parameters; the consumer checks requirements.
    pub params: &'a ElementParams,
}
/// Global element response.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementResponse {
    /// Forces in N and moments in N·m, node then DOF.
    pub internal_force: Vec<Real>,
    /// Derivative with respect to displacements in m and rotations in rad; symmetric v1.
    pub tangent: DMat,
    /// Finite trial history, length state_len.
    pub trial_state: Vec<Real>,
}
/// Mass distribution; translations in kg and rotations in kg·m².
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MassKind {
    /// Mass lumped at nodes.
    Lumped,
    /// Mass consistent with the element interpolation.
    Consistent,
}
/// Uniform span load expressed in global axes.
///
/// Minimal v1 type: forces per length in N/m and moments per length in N.
/// Implementations must reject unsupported components, never omit them.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberLoad {
    /// Global [qx, qy, qz] in N/m.
    pub force_per_length: [Real; 3],
    /// Global [mx, my, mz] in N·m/m.
    pub moment_per_length: [Real; 3],
}
/// Resultants at an interior station, local positive face.
#[derive(Clone, Debug, PartialEq)]
pub struct StationResult {
    /// Dimensionless position x/L in [0, 1].
    pub position: Real,
    /// [N, Vy, Vz, T, My, Mz], forces in N and moments in N·m.
    pub forces_local: [Real; 6],
}
/// Recovery of local results, including supported span loads.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementOutput {
    /// End actions: node and local DOF, negative face i and positive face j.
    pub end_forces_local: Vec<Real>,
    /// Stations in the requested order, with resultants per positive face.
    pub stations: Vec<StationResult>,
}
/// Nominal case reference, without normalization or an implicit case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseRef(pub String);
/// Terminal analysis state, independent of input or I/O errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// All requested steps were committed.
    Completed,
    /// The last committed step is preserved; without hidden retries.
    NotConverged,
    /// Loss of stability detected and documented.
    Unstable,
    /// Cooperatively cancelled; the last committed state is preserved.
    Cancelled,
}
/// Dimensionless, finite, positive relative tolerances.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerances {
    /// Relative force criterion.
    pub force_rel: Real,
    /// Relative displacement criterion.
    pub disp_rel: Real,
    /// Relative energy criterion.
    pub energy_rel: Real,
}
/// Explicit step report; norms are relative and dimensionless.
#[derive(Clone, Debug, PartialEq)]
pub struct StepReport {
    /// Step index counted from one.
    pub step: u32,
    /// Dimensionless load factor.
    pub load_factor: Real,
    /// Number of iterations executed.
    pub iterations: u32,
    /// Relative force imbalance norm.
    pub residual_norm: Real,
    /// Relative displacement increment norm.
    pub displacement_norm: Real,
    /// Relative energy increment norm.
    pub energy_norm: Real,
    /// Whether the declared tolerances were met.
    pub converged: bool,
    /// Tolerances actually used, without implicit values.
    pub tolerances: Tolerances,
}
/// Convergence history ordered by step, including the failed attempt.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConvergenceReport {
    /// Reports of attempts made; empty list if cancelled beforehand.
    pub steps: Vec<StepReport>,
}
/// Terminal result; EResult is reserved for input, contract, or I/O failures.
#[derive(Clone, Debug)]
pub struct AnalysisOutcome {
    /// Final state, without turning lack of convergence into success.
    pub status: Status,
    /// Norms and tolerances of all attempts.
    pub report: ConvergenceReport,
    /// Diagnostics from the core catalog, with relevant context.
    pub diagnostics: Vec<Diagnostic>,
}
/// Capabilities a method requires before starting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Requirements {
    /// Requires assembled mass.
    pub mass: bool,
    /// Requires geometric stiffness.
    pub geometric: bool,
    /// Requires nonlinear constitutive response.
    pub nonlinear: bool,
    /// Required cases by name; empty allows selection through settings.
    pub cases: Vec<CaseRef>,
}
/// Address of a field in a results/1 block; no serialization in this ticket.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultKey {
    /// Block: nodes, members, modes, cases, combinations, envelopes,
    /// convergence, diagnostics, or provenance.
    pub block: String,
    /// Custom text identifier for an entity or case; None for global metadata.
    pub entity: Option<String>,
    /// Field name; its units and semantics are declared by the producer.
    pub field: String,
}
/// Minimal value transportable by ResultSink; finite numerics in SI.
///
/// Full provenance and results/1 filters belong to ENG-CORE-005.
#[derive(Clone, Debug)]
pub enum ResultValue {
    /// SI scalar value.
    Scalar(Real),
    /// SI vector in the order documented by the field.
    Vector(Vec<Real>),
    /// Text, also useful for provenance metadata.
    Text(String),
    /// Local member result.
    Element(ElementOutput),
    /// Convergence report.
    Convergence(ConvergenceReport),
    /// Diagnostics with core codes.
    Diagnostics(Vec<Diagnostic>),
}
