//! Adaptación de una ley uniaxial propietaria a la geometría del resorte.
use crate::{finite, geometry, invalid, loads, recover, stiffness, vector, LinearSpring};
use crate::{SpringAxes, SpringConnection};
use engine_contracts::{
    Element, ElementCtx, ElementOutput, ElementParams, ElementResponse, MassKind, MemberLoad,
    UniaxialLaw, UniaxialResponse,
};
use engine_core::{ContractId, Dim, Dof, EResult, Real};
use engine_linalg::DMat;
use std::sync::Arc;

/// Resorte de rango uno `spring.nonlinear` con ley `uniaxial/1` inmutable.
///
/// La ley se interpreta como fuerza-desplazamiento: ε=δ (m), σ=F (N),
/// longitud/área unitarias. Para la familia de giros: θ (rad) y M (N·m).
/// La geometría, cargas y recuperación siguen [`LinearSpring`]. El eje propio
/// debe rotarse junto al modelo. Invertirlo cambia el lado activo de una ley
/// asimétrica, por lo que no se presupone invariancia ante inversión de signo.
///
/// La historia pertenece al llamador. No hay aceptación implícita ni memoria
/// de pruebas en el elemento; commit y revert se describen en el crate.
///
/// ```
/// use std::sync::Arc;
/// use engine_contracts::{examples::LinearElastic, Element, ElementCtx, ElementParams};
/// use engine_core::{Dim, Dof};
/// use engine_elements_spring::{NonlinearSpring, SpringAxes, SpringConnection};
/// // La ley de ejemplo se interpreta aquí como F=100δ, en N y m.
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
/// accepted = trial.trial_state; // Commit explícito tras aceptar el paso.
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
    /// Construye con ley resuelta y geometría igual a [`LinearSpring::new`].
    /// Valida `uniaxial/1`, tamaño/finitud de historia inicial y tangente inicial.
    /// No impone signo a la tangente: el rango físico lo declara cada ley.
    /// ENG-E-0010 por contrato/tamaños/geometría; ENG-E-0008 por no finitos.
    pub fn new(
        dim: Dim,
        connection: SpringConnection,
        law: Arc<dyn UniaxialLaw>,
        dofs: &[Dof],
        axes: SpringAxes,
    ) -> EResult<Self> {
        let contract = law.contract();
        if contract.name != "uniaxial" || contract.major != 1 {
            return Err(invalid("Resorte no lineal: se requiere ley uniaxial/1."));
        }
        let initial = law.initial_state();
        vector(&initial, law.state_len(), "Historia inicial de la ley")?;
        finite(law.initial_tangent(), "Tangente inicial de la ley")?;
        Ok(Self {
            geometry: LinearSpring::new(dim, connection, 1.0, dofs, axes)?,
            law,
            initial,
        })
    }

    /// Obtiene la ley previamente resuelta de `ElementParams::law` y la conserva.
    /// ENG-E-0010 si falta. `ctx.params` no cambia la ley tras construir.
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
            .ok_or_else(|| invalid("Resorte no lineal: falta la ley resuelta."))?;
        Self::new(dim, connection, law, dofs, axes)
    }

    /// Dimensión validada del resorte.
    pub fn dim(&self) -> Dim {
        self.geometry.dim()
    }

    /// Conexión entre nodos o a tierra.
    pub fn connection(&self) -> SpringConnection {
        self.geometry.connection()
    }

    /// Ley constitutiva conservada, independiente de la vida del contexto.
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
        vector(committed, self.state_len(), "Historia aceptada")?;
        // La geometría auxiliar tiene k=1: su fuerza generalizada es δ.
        let delta = stiffness::force(&self.geometry, u, &[])?;
        let response = self.law.respond(delta, committed)?;
        finite(response.stress, "Fuerza de la ley")?;
        finite(response.tangent, "Tangente de la ley")?;
        vector(
            &response.trial_state,
            self.state_len(),
            "Historia de prueba",
        )?;
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
