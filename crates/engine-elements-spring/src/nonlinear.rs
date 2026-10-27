//! Adaptation of an owned uniaxial law to spring geometry.
use crate::{finite, geometry, invalid, loads, recover, stiffness, vector, LinearSpring};
use crate::{SpringAxes, SpringConnection};
use engine_contracts::{
    Element, ElementCtx, ElementOutput, ElementParams, ElementResponse, MassKind, MemberLoad,
    UniaxialLaw, UniaxialResponse,
};
use engine_core::{ContractId, Dim, Dof, EResult, Real};
use engine_linalg::DMat;
use std::sync::Arc;

/// Rank-one `spring.nonlinear` spring with an immutable `uniaxial/1` law.
///
/// The law is interpreted as force-displacement: ε=δ (m), σ=F (N),
/// unit length/area. For the rotation family: θ (rad) and M (N·m).
/// Geometry, loads, and recovery follow [`LinearSpring`]. The spring axis
/// must rotate with the model. Reversing it changes the active side of an
/// asymmetric law, so invariance under sign reversal is not assumed.
///
/// History belongs to the caller. There is no implicit acceptance or storage
/// of trials in the element; commit and revert are described in the crate.
///
/// ```
/// use std::sync::Arc;
/// use engine_contracts::{examples::LinearElastic, Element, ElementCtx, ElementParams};
/// use engine_core::{Dim, Dof};
/// use engine_elements_spring::{NonlinearSpring, SpringAxes, SpringConnection};
/// // The example law is interpreted here as F=100δ, in N and m.
/// let params = ElementParams {
///     law: Some(Arc::new(LinearElastic::new(100.0)?)),
///     ..ElementParams::default()
/// };
/// let spring = NonlinearSpring::from_params(Dim::D2, SpringConnection::Ground,
///     &params, &[Dof::Ux], SpringAxes::Global)?;
/// let coords = [[0.0; 3]];
/// let ctx = ElementCtx { dim: Dim::D2, coords: &coords, params: &params };
/// let mut accepted = spring.initial_state();
/// let trial = spring.respond(&ctx, &[0.05], &accepted)?;
/// assert_eq!(trial.internal_force, vec![5.0]);
/// accepted = trial.trial_state; // Explicit commit after accepting the step.
/// assert_eq!(spring.respond(&ctx, &[0.05], &accepted)?.internal_force, vec![5.0]);
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone)]
pub struct NonlinearSpring {
    geometry: LinearSpring,
    law: Arc<dyn UniaxialLaw>,
    initial: Vec<Real>,
}

impl NonlinearSpring {
    /// Constructs with a resolved law and geometry as in [`LinearSpring::new`].
    /// Validates `uniaxial/1`, initial history size/finiteness, and initial tangent.
    /// Imposes no sign on the tangent: each law declares its physical range.
    /// ENG-E-0010 for contract/sizes/geometry; ENG-E-0008 for nonfinite values.
    pub fn new(
        dim: Dim,
        connection: SpringConnection,
        law: Arc<dyn UniaxialLaw>,
        dofs: &[Dof],
        axes: SpringAxes,
    ) -> EResult<Self> {
        let contract = law.contract();
        if contract.name != "uniaxial" || contract.major != 1 {
            return Err(invalid("Nonlinear spring: uniaxial/1 law is required."));
        }
        let initial = law.initial_state();
        vector(&initial, law.state_len(), "Initial law history")?;
        finite(law.initial_tangent(), "Initial law tangent")?;
        Ok(Self {
            geometry: LinearSpring::new(dim, connection, 1.0, dofs, axes)?,
            law,
            initial,
        })
    }

    /// Retrieves the previously resolved law from `ElementParams::law` and retains it.
    /// ENG-E-0010 if missing. `ctx.params` does not change the law after construction.
    pub fn from_params(
        dim: Dim,
        connection: SpringConnection,
        params: &ElementParams,
        dofs: &[Dof],
        axes: SpringAxes,
    ) -> EResult<Self> {
        let law = params
            .law
            .clone()
            .ok_or_else(|| invalid("Nonlinear spring: missing resolved law."))?;
        Self::new(dim, connection, law, dofs, axes)
    }

    /// Validated spring dimension.
    pub fn dim(&self) -> Dim {
        self.geometry.dim()
    }

    /// Connection between nodes or to ground.
    pub fn connection(&self) -> SpringConnection {
        self.geometry.connection()
    }

    /// Retained constitutive law, independent of the context lifetime.
    pub fn law(&self) -> &dyn UniaxialLaw {
        self.law.as_ref()
    }

    fn trial(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[Real],
        committed: &[Real],
    ) -> EResult<UniaxialResponse> {
        geometry::validate(&self.geometry, ctx)?;
        vector(committed, self.state_len(), "Accepted history")?;
        // Auxiliary geometry has k=1: its generalized force is δ.
        let delta = stiffness::force(&self.geometry, u, &[])?;
        let response = self.law.respond(delta, committed)?;
        finite(response.stress, "Law force")?;
        finite(response.tangent, "Law tangent")?;
        vector(&response.trial_state, self.state_len(), "Trial history")?;
        Ok(response)
    }
}

impl Element for NonlinearSpring {
    fn kind(&self) -> &'static str {
        "spring.nonlinear"
    }
    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    }
    fn n_nodes(&self) -> usize {
        self.geometry.n_nodes()
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        self.geometry.dofs_per_node()
    }
    fn state_len(&self) -> usize {
        self.initial.len()
    }
    fn initial_state(&self) -> Vec<Real> {
        self.initial.clone()
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        let response = self.trial(ctx, u_e, committed)?;
        Ok(ElementResponse {
            internal_force: stiffness::vector(&self.geometry, response.stress, false)?,
            tangent: stiffness::matrix_with_tangent(&self.geometry, response.tangent)?,
            trial_state: response.trial_state,
        })
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        self.geometry.mass(ctx, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        self.trial(ctx, u_e, committed)?;
        Ok(None)
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        self.geometry.load_vector(ctx, load)
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
        member_loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput> {
        for load in member_loads {
            loads::validate(load)?;
        }
        let response = self.trial(ctx, u_e, committed)?;
        recover::output_with_force(&self.geometry, response.stress, member_loads, stations)
    }
}
