//! V1 boundaries; all errors reuse EngineError from the core.
use crate::*;

/// Pure, thread-safe uniaxial law, uniaxial/1 contract.
///
/// # Inputs and units
/// Dimensionless total strain for material, or relative displacement in m
/// for a spring; finite committed history of length state_len.
/// # Outputs and invariants
/// Stress in Pa or force in N and its consistent derivative (relative FD error < 1e-6).
/// All history is flat, finite, and owned by the caller. Repeating the committed
/// input reproduces the committed stress. Does not modify self or committed;
/// each kind declares its validity range and nondifferentiable points.
/// # Errors
/// ENG-E-0008 for nonfinite values/overflow; ENG-E-0010 for invalid history or
/// range. Other physical errors must declare a core code and context.
pub trait UniaxialLaw: Send + Sync {
    /// Stable custom name; without units or side effects.
    fn kind(&self) -> &'static str;
    /// uniaxial/1 identity; compatible minor version.
    fn contract(&self) -> ContractId {
        ContractId::new("uniaxial", 1, 0)
    }
    /// Exact length of committed and trial history.
    fn state_len(&self) -> usize;
    /// Finite initial history in the units documented by kind.
    fn initial_state(&self) -> Vec<Real>;
    /// Finite initial tangent, Pa or N/m; consistent with the initial origin.
    fn initial_tangent(&self) -> Real;
    /// Evaluates TOTAL strain and COMMITTED state; returns a trial response.
    /// Does not implicitly commit history; errors and units as specified by this trait.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse>;
}

/// Pure, thread-safe section law, section/1 contract.
///
/// # Inputs and units
/// D2 e=[ε0, κz]; D3 e=[ε0, κy, κz, θ′]; ε0 dimensionless and the rest m⁻¹.
/// Finite flat committed state of length state_len.
/// # Outputs and invariants
/// D2 s=[N, Mz]; D3 s=[N, My, Mz, T], N and N·m. Symmetric tangent v1,
/// consistent derivative of s with respect to e with relative FD error < 1e-6.
/// Does not modify self or committed; repeating committed e reproduces committed s.
/// All output is finite and trial_state has length state_len.
/// # Errors
/// ENG-E-0010 for invalid sizes or range; ENG-E-0008 for nonfinite values.
pub trait SectionLaw: Send + Sync {
    /// Custom section name and its documented physical range.
    fn kind(&self) -> &'static str;
    /// section/1 identity.
    fn contract(&self) -> ContractId;
    /// Dimension determining two or four generalized strains.
    fn dim(&self) -> Dim;
    /// Exact length of the flat state.
    fn state_len(&self) -> usize;
    /// Finite initial history; units defined by kind.
    fn initial_state(&self) -> Vec<Real>;
    /// Responds to TOTAL strains using COMMITTED history without mutating it.
    /// Returns resultants, tangent, and trial state; errors as specified by the trait.
    fn respond(&self, e: &[Real], committed: &[Real]) -> EResult<SectionResponse>;
}

/// Pure, thread-safe element, element/1 contract.
///
/// # Inputs and units
/// Context with n_nodes coordinates in m and resolved SI parameters. TOTAL
/// global u_e, node then dofs_per_node: translations in m and rotations in rad.
/// COMMITTED state of length state_len. D2 requires X-Y and Z=0.
/// # Outputs and invariants
/// Global forces in N and moments in N·m; length n_nodes × dofs_per_node.len().
/// Square symmetric tangent and mass v1; FD-consistent tangent < 1e-6.
/// Three D2 and six D3 rigid modes without internal force for initial history without
/// prestress in elements without external connection. A Winkler bed attached to the ground
/// resists transverse rigid motions by definition of its kind. Covariance of forces, tangent, and mass under rotation of
/// the global system.
/// Finite and deterministic response; repeating the committed displacement reproduces
/// committed forces. Never mutates self, context, or history.
/// # Errors
/// ENG-E-0010 for unsupported dimension, sizes, parameters, loads, or stations;
/// ENG-E-0005 for zero length; ENG-E-0008 for nonfinite values/overflow.
/// Absence of mass or geometric stiffness is indicated by None, not an error.
pub trait Element: Send + Sync {
    /// Custom kind; documents assumptions, physical range, and supported loads.
    fn kind(&self) -> &'static str;
    /// element/1 identity.
    fn contract(&self) -> ContractId;
    /// Number of nodes in the context and vectors.
    fn n_nodes(&self) -> usize;
    /// Active components per node, distinct and in vector order.
    fn dofs_per_node(&self) -> &'static [Dof];
    /// Exact history length; no history by default.
    fn state_len(&self) -> usize {
        0
    }
    /// Finite initial history; empty by default.
    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }
    /// Evaluates TOTAL u_e and COMMITTED state; returns force, tangent, and trial.
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse>;
    /// Returns mass (kg, kg·m, and kg·m² according to DOF) or None if absent;
    /// validates context and returns the errors defined above.
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>>;
    /// Returns global Kg computed with the axial force from u_e, or None if unsupported.
    /// Same units/sizes as tangent; does not commit history.
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>>;
    /// Converts a span load to global nodal forces/moments;
    /// vector length equal to respond; rejects unsupported components.
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>>;
    /// Recovers local actions at end faces and stations x/L in `[0,1]`,
    /// in the requested order, including span loads. Does not modify history.
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput>;
}

/// Thread-safe element factory, without assembly or backend access.
///
/// # Inputs, outputs, and units
/// Minimal in-memory JSON, with SI properties according to the schema and a borrowed
/// ModelView with unique references and coordinates in m. Returns an element independent
/// of the view's lifetime; does not modify the model.
/// # Invariants and errors
/// Validates required fields, types, range, references, and dimension against the
/// declared schema. Element kind equal to the factory kind, element/1.
/// ENG-E-0010 for schema/dimension, ENG-E-0003 for missing references,
/// ENG-E-0008 for nonfinite values. Does not parse JSON or use serde.
pub trait ElementFactory: Send + Sync {
    /// Custom name equal to that of the constructed element.
    fn kind(&self) -> &'static str;
    /// JSON Schema as valid JSON text; without implicit units.
    fn params_schema(&self) -> String;
    /// Validates SI parameters and borrowed model, and constructs an owned element.
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>>;
}

/// Internal boundary between analysis method and assembled system.
///
/// # Inputs, outputs, and units
/// Reduced vectors of n_free_dofs, displacements in m/rotations in rad, forces in N/
/// moments in N·m. Square matrices of the same order; masses in kg/inertias in kg·m².
/// External forces by CaseRef and dimensionless factor. The producer declares
/// the scaling needed for relative norms with mixed components.
/// # Invariants
/// Each operation completely overwrites out, never accumulates. Symmetric tangent and
/// mass. Evaluations use the last committed state and do not commit it
/// implicitly. accept(u) will reevaluate/commit exactly u, atomically;
/// rollback discards every trial and preserves the last committed state, without failing.
/// # Errors
/// ENG-E-0010 for sizes/context; ENG-E-0008 for nonfinite values;
/// ENG-E-0003 for a missing case; ENG-E-0012 for inconsistent internal state.
/// An index for dof_label must be valid; out of range may cause a panic.
pub trait SystemView {
    /// Number of free equations in reduced order.
    fn n_free_dofs(&self) -> usize;
    /// Symmetric pattern of that order; without numerical analysis.
    fn pattern(&self) -> &SparsityPattern;
    /// Overwrites out with dF_int/du at TOTAL u, without committing state.
    fn tangent(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()>;
    /// Overwrites out with reduced global internal forces for TOTAL u.
    fn internal_force(&mut self, u: &[Real], out: &mut [Real]) -> EResult<()>;
    /// Overwrites out with mass of the requested type; zero if there is no mass.
    fn mass(&mut self, kind: MassKind, out: &mut SymMatrix) -> EResult<()>;
    /// Overwrites out with geometric stiffness using the axial force from u.
    fn geometric(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()>;
    /// Overwrites case forces scaled by factor; does not modify the system.
    fn external_force(&self, case: &CaseRef, factor: Real, out: &mut [Real]) -> EResult<()>;
    /// Commits exactly u and its reevaluated history; preserves committed state on failure.
    fn accept(&mut self, u: &[Real]) -> EResult<()>;
    /// Discards trial and restores committed history; infallible and idempotent.
    fn rollback(&mut self);
    /// Node/DOF label for a valid reduced global index; precondition d < n.
    fn dof_label(&self, d: DofIndex) -> (NodeId, Dof);
}

/// Thread-safe method, analysis/1 contract, without direct access to nodes/elements.
///
/// # Inputs and units
/// Reduced system, minimal JSON settings with SI units and explicit strategies,
/// SI result sink and cooperative control. requires declares capabilities.
/// # Outputs and invariants
/// Always reports relative norms and tolerances used. NotConverged preserves the
/// last committed state, without hidden retries. Unstable indicates loss of
/// stability; Cancelled preserves the last committed state. Completed requires convergence.
/// Mechanisms are diagnosed as ENG-E-0006 with node/DOF. Does not publish trials as
/// committed states. Full results/1 provenance is added in the facade.
/// # Errors
/// ENG-E-0010 for invalid settings/capabilities; ENG-E-0008 for nonfinite values;
/// propagates system and sink errors. Discards trials on error or cancellation.
/// Nonconvergence is returned as an outcome, not an infrastructure error.
pub trait AnalysisMethod: Send + Sync {
    /// Stable custom method identifier.
    fn id(&self) -> &'static str;
    /// Semantic major, minor, and patch version.
    fn version(&self) -> (u16, u16, u16);
    /// analysis/1 identity.
    fn contract(&self) -> ContractId;
    /// Textual JSON Schema for settings and explicit strategies.
    fn settings_schema(&self) -> String;
    /// Capabilities required before execution.
    fn requires(&self) -> Requirements;
    /// Executes and emits committed results, norms, tolerances, and diagnostics.
    /// Errors as specified by this trait; respects cancelled before committing each step.
    fn run(
        &self,
        sys: &mut dyn SystemView,
        settings: &JsonValue,
        sink: &mut dyn ResultSink,
        ctl: &dyn RunControl,
    ) -> EResult<AnalysisOutcome>;
}

/// Cooperative control without modifying the system.
///
/// Input: finite dimensionless progress in `[0,1]` and note in Spanish.
/// Output: cancellation observation; no infrastructure errors or panic
/// for valid inputs. Progress must be monotonic within a run.
/// The method queries cancelled between steps and before committing; the observation
/// may change through external intervention. Does not require a threading model.
pub trait RunControl {
    /// Publishes finite fraction `[0,1]` and note; does not alter computation state.
    fn progress(&self, fraction: Real, note: &str);
    /// Returns true if stopping was requested; neither commits nor rolls back the system.
    fn cancelled(&self) -> bool;
}

/// Sink for results/1 values, without executing computations.
///
/// Input: key with nonempty custom block/field and finite SI value; output:
/// write confirmation or error. Each put is atomic; the policy for repeated keys
/// is documented per implementation. Preserves vector order and signs.
/// ENG-E-0010 for invalid key/value, ENG-E-0008 for nonfinite values; I/O failures
/// are expressed through EngineError with context. Does not commit system history.
pub trait ResultSink {
    /// Writes an SI value at its key; propagates errors and leaves no partial write.
    fn put(&mut self, key: ResultKey, value: ResultValue) -> EResult<()>;
}
