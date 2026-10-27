#![forbid(unsafe_code)]
//! Springs `spring.linear` and `spring.nonlinear`, contract element/1.
//!
//! [`NonlinearSpring`] retains a resolved `uniaxial/1` law. Uses unit length and
//! area: ε=δ, σ=F; the law table expresses displacement and force,
//! without automatically converting a material stress-strain curve.
//! Its history is exactly the law's history. `respond` only produces a trial;
//! the caller commits by replacing its accepted history with `trial_state`
//! after convergence, or reverts by discarding the trial and retaining the accepted history.
//! The tangent is (dF/dδ)bbᵀ, with a fixed axis and no geometric stiffness.
//! Breakpoints, gaps, prestress, and physical range depend on the selected law.
//! For rotations, the equivalent interpretation is ε=θ, σ=M, tangent in N·m/rad.
//!
//! In `spring.linear`, the direct API [`LinearSpring::new`] accepts one or more local DOFs
//! of the same family (translations or rotations), sharing a stiffness k≥0:
//! δ is the SUM of their relative displacements, energy
//! W=½kδ². This couples the DOFs; independent springs require separate
//! elements. Between nodes δ=c·(u_j−u_i); to ground δ=c·u_i. For one DOF,
//! `K=k[[1,−1],[−1,1]]` between nodes, `K=[k]` to ground. Internal force = ∂W/∂u.
//! Fixed global or explicit axes, never inferred from coordinates.
//! Normalized direction for local DOF Ux; a rotation uses rows as
//! local axes expressed in global axes. D2 is in XY, Z=0; D3 is right-handed.
//! Factory [`SpringFactory`] and its JSON schema accept exactly one local DOF
//! of the selected dimension. Empty lists or lists with more DOFs than the
//! dimension are rejected; multiple distinct valid DOFs produce ENG-E-0013.
//!
//! Translations in m, forces in N, k in N/m; rotations in rad, moments in N·m, k in N·m/rad.
//! Translations and rotations are not mixed without an explicit dimensional scale.
//! Validated properties in self; ctx.params is unused. Coincident nodes
//! are allowed: no physical length, span load, mass, or Kg.
//! Mass/Kg return None after validating inputs; zero load returns zero,
//! any nonzero span load is rejected. Recovery returns local actions
//! per node and constant resultants `[N,Vy,Vz,T,My,Mz]` at stations.
//! To ground, the sole node represents the moving end j; its action is +kδ.
//!
//! The axis is fixed: a rigid rotation changing its projection may deform a
//! spring between separated nodes. Uniform translations are zero modes.
//! A ground support removes the corresponding rigid mode. Covariance
//! requires rotating the spring axis as well; general kit limits are in the log.
//! ENG-E-0008: nonfinite values/overflow; ENG-E-0010: range, size, zero
//! direction, nonorthonormal rotation, unsupported DOF or load. No I/O, threads,
//! clock, or mutable state. Deterministic per target, without a bitwise promise
//! across targets or regularization of tiny stiffnesses.
//!
//! ```
//! use engine_elements_spring::{LinearSpring, SpringAxes, SpringConnection};
//! use engine_contracts::{Element, ElementCtx, ElementParams};
//! use engine_core::{Dim, Dof};
//! let spring = LinearSpring::new(Dim::D2, SpringConnection::Ground,
//!     200.0, &[Dof::Ux], SpringAxes::Global)?;
//! let params = ElementParams::default();
//! let coords = [[0.0; 3]];
//! let ctx = ElementCtx { dim: Dim::D2, coords: &coords, params: &params };
//! assert_eq!(spring.respond(&ctx, &[0.06], &[])?.internal_force, vec![12.0]);
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

mod factory;
mod geometric;
mod geometry;
mod loads;
mod mass;
mod nonlinear;
mod recover;
mod stiffness;

use engine_contracts::{Element, ElementCtx, ElementOutput, ElementResponse, MassKind, MemberLoad};
use engine_core::{ContractId, Dim, Dof, EResult, EngineError, Real};
use engine_linalg::DMat;
pub use factory::SpringFactory;
pub use geometry::SpringAxes;
pub use nonlinear::NonlinearSpring;

/// Spring connection; nodal order between ends is i, j.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpringConnection {
    /// Two nodes, equal and opposite actions.
    BetweenNodes,
    /// One moving node connected to stationary ground.
    Ground,
}

/// Rank-one linear spring with immutable SI properties.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearSpring {
    dim: Dim,
    connection: SpringConnection,
    k: Real,
    local: [Real; 6],
    global: [Real; 6],
    mask: usize,
}

impl LinearSpring {
    /// Constructs with finite k≥0 and distinct, nonempty DOFs of the same family.
    /// DOFs are summed with unit coefficient in the selected system.
    /// Direction requires only local Ux. Rotation expands active global DOFs
    /// to the entire family to preserve covariance; Global retains
    /// the selected subset. Output order: the canonical engine-core order.
    pub fn new(
        dim: Dim,
        connection: SpringConnection,
        k: Real,
        dofs: &[Dof],
        axes: SpringAxes,
    ) -> EResult<Self> {
        finite(k, "k")?;
        if k < 0.0 {
            return Err(invalid("Spring: k must be nonnegative."));
        }
        let (local, global, mask) = geometry::coupling(dim, dofs, axes)?;
        Ok(Self {
            dim,
            connection,
            k,
            local,
            global,
            mask,
        })
    }

    /// Validated spring dimension.
    pub fn dim(&self) -> Dim {
        self.dim
    }
    /// Connection between nodes or to ground.
    pub fn connection(&self) -> SpringConnection {
        self.connection
    }
    /// Validated stiffness k, N/m or N·m/rad according to the family.
    pub fn stiffness(&self) -> Real {
        self.k
    }

    fn size(&self) -> usize {
        self.n_nodes() * self.dofs_per_node().len()
    }
}

impl Element for LinearSpring {
    fn kind(&self) -> &'static str {
        "spring.linear"
    }
    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    }
    fn n_nodes(&self) -> usize {
        match self.connection {
            SpringConnection::BetweenNodes => 2,
            SpringConnection::Ground => 1,
        }
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        geometry::dofs(self.mask)
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        geometry::validate(self, ctx)?;
        let force = stiffness::force(self, u_e, committed)?;
        Ok(ElementResponse {
            internal_force: stiffness::vector(self, force, false)?,
            tangent: stiffness::matrix(self)?,
            trial_state: vec![],
        })
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        mass::matrix(self, ctx, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        geometric::matrix(self, ctx, u_e, committed)
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        geometry::validate(self, ctx)?;
        loads::validate(load)?;
        Ok(vec![0.0; self.size()])
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput> {
        recover::output(self, ctx, u_e, committed, loads, stations)
    }
}

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
fn finite(value: Real, name: &str) -> EResult<Real> {
    if !value.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            format!("{name}: non-finite input or result."),
        ));
    }
    Ok(value)
}
fn vector(values: &[Real], size: usize, name: &str) -> EResult<()> {
    if values.len() != size {
        return Err(invalid(&format!(
            "{name}: size {}, expected {size}.",
            values.len()
        )));
    }
    for &value in values {
        finite(value, name)?;
    }
    Ok(())
}
