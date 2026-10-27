//! `static.newton`: full Newton with explicit absolute load factors.
//!
//! K_total = K_lineal + Σ K_elemento(u, historia_aceptada). Variable elements
//! are additional contributions and must NOT also be included in K_lineal.
//! Each evaluation uses total u (including constant prescribed displacements) and the history
//! of the last accepted step. trial_state is committed only upon convergence.
//! There is no line search, bisection, regularization, or retry.
//!
//! R = λ F − F_int is measured AFTER applying each correction. Δu is the last
//! applied correction and energy is |ΔuᵀR| with that updated residual.
//! All active tolerances must be met. With force alone, a linear system
//! converges in one correction; also requiring a small correction
//! may require another iteration. SI scales are explicit, constant, and
//! positive; mixed force/moment and translation/rotation norms are
//! numerical indicators, not physical invariants under changes of units.
//!
//! Example: bilinear spring to ground, 16 N, final u = 0.4 m.
//! ```
//! use engine_analysis::newton::{NewtonElement, NewtonOptions, NewtonSystem, StaticNewton};
//! use engine_assembly::LinearSystem;
//! use engine_contracts::{CaseRef, ElementCtx, ElementParams, Status};
//! use engine_core::{Dim, Dof, DofIndex, NodeId};
//! use engine_elements_spring::{NonlinearSpring, SpringAxes, SpringConnection};
//! use engine_laws::uniaxial::{MultilinearElastic, MultilinearMode};
//! use engine_linalg::ReferenceBackend;
//! use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node};
//! use std::sync::Arc;
//! let mut model = Model::new();
//! let mut node = Node::new(NodeId(1), Dim::D2, [0.0; 3]);
//! node.dofs = vec![Dof::Ux];
//! model.nodes.push(node);
//! model.cases.push(LoadCase { name: "P".into(), loads: vec![NodalLoad {
//!     node: NodeId(1), values: vec![DofValue { dof: Dof::Ux, value: 16.0 }],
//! }] });
//! let law = MultilinearElastic::new(
//!     vec![(0.0, 0.0), (0.1, 10.0), (0.5, 18.0)], MultilinearMode::Both,
//! )?;
//! let spring = NonlinearSpring::new(Dim::D2, SpringConnection::Ground,
//!     Arc::new(law), &[Dof::Ux], SpringAxes::Global)?;
//! let params = ElementParams::default();
//! let ctx = ElementCtx { dim: Dim::D2, coords: &[[0.0; 3]], params: &params };
//! let element = NewtonElement::new(Box::new(spring), vec![DofIndex(0)], &ctx)?;
//! let linear = LinearSystem::from_model(ReferenceBackend, &model, &[], &[])?;
//! let mut system = NewtonSystem::new(linear, vec![element])?;
//! let result = StaticNewton::run(&mut system, &CaseRef("P".into()),
//!     &[0.25, 0.5, 1.0], NewtonOptions::default())?;
//! assert_eq!(result.status, Status::Completed);
//! assert!((result.free_displacements[0] - 0.4).abs() < 1e-12);
//! # Ok::<(), engine_core::EngineError>(())
//! ```

use crate::finite;
use engine_assembly::{assemble::assemble_vector, LinearSystem};
use engine_contracts::{
    CaseRef, Element, ElementCtx, ElementParams, ElementResponse, Status, SystemView,
};
use engine_core::{ContractId, Dim, DofIndex, EResult, EngineError, Real};
use engine_linalg::{LinAlgBackend, SymMatrix};

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}

/// Variable element with owned context and global element indices.
/// Order is node then dofs_per_node; checked against the map when joined.
pub struct NewtonElement {
    element: Box<dyn Element>,
    location: Vec<DofIndex>,
    dim: Dim,
    coords: Vec<[Real; 3]>,
    params: ElementParams,
}

impl NewtonElement {
    /// Copies the SI context and retains the pure element, without evaluating or accepting.
    /// ENG-E-0010 for empty contract/sizes; ENG-E-0008 for nonfinite coordinates.
    pub fn new(
        element: Box<dyn Element>,
        location: Vec<DofIndex>,
        ctx: &ElementCtx<'_>,
    ) -> EResult<Self> {
        let width = element.dofs_per_node().len();
        if !element
            .contract()
            .is_compatible_with(&ContractId::new("element", 1, 0))
            || width == 0
            || element.n_nodes() == 0
            || element.n_nodes().checked_mul(width) != Some(location.len())
            || ctx.coords.len() != element.n_nodes()
        {
            return Err(invalid("Newton element: invalid contract or sizes."));
        }
        for xyz in ctx.coords {
            finite(xyz)?;
        }
        Ok(Self {
            element,
            location,
            dim: ctx.dim,
            coords: ctx.coords.to_vec(),
            params: ctx.params.clone(),
        })
    }

    fn respond(&self, global: &[Real], state: &[Real]) -> EResult<ElementResponse> {
        let u = self
            .location
            .iter()
            .map(|i| global[i.0 as usize])
            .collect::<Vec<_>>();
        let response = self.element.respond(
            &ElementCtx {
                dim: self.dim,
                coords: &self.coords,
                params: &self.params,
            },
            &u,
            state,
        )?;
        if response.trial_state.len() != self.element.state_len() {
            return Err(invalid("Newton element: invalid trial history size."));
        }
        finite(&response.trial_state)?;
        finite(response.tangent.values())?;
        Ok(response)
    }
}

/// System with constant linear contributions and additional variable elements.
/// Owns the history; evaluations never mutate it. Initial displacement
/// is the accepted LinearSystem displacement and initial history comes from each element.
pub struct NewtonSystem<B> {
    linear: LinearSystem<B>,
    elements: Vec<NewtonElement>,
    accepted_states: Vec<Vec<Real>>,
    accepted_factor: Option<Real>,
}

struct Evaluation {
    tangent: SymMatrix,
    internal: Vec<Real>,
    states: Vec<Vec<Real>>,
}

impl<B: LinAlgBackend> NewtonSystem<B> {
    /// Joins contributions without duplicating their forces/tangents in the linear system.
    /// Nodal and equivalent loads are declared in LinearSystem. Validates
    /// nodes/DOFs/coordinates, initial history, and response at accepted u.
    /// Propagates element and assembly diagnostics without acceptance.
    pub fn new(linear: LinearSystem<B>, elements: Vec<NewtonElement>) -> EResult<Self> {
        let mut states = Vec::new();
        for e in &elements {
            let mut zero = vec![0.0; linear.dof_map().len()];
            assemble_vector(&mut zero, &e.location, &vec![0.0; e.location.len()])?;
            let width = e.element.dofs_per_node().len();
            for (node_position, indices) in e.location.chunks_exact(width).enumerate() {
                let (node, _) = linear.dof_map().label(indices[0])?;
                if linear.dof_map().coordinates(node)? != e.coords[node_position] {
                    return Err(invalid("Newton element: coordinates differ from the map."));
                }
                for (&index, &dof) in indices.iter().zip(e.element.dofs_per_node()) {
                    if linear.dof_map().label(index)? != (node, dof) {
                        return Err(invalid("Newton element: invalid node/DOF order."));
                    }
                }
            }
            let state = e.element.initial_state();
            if state.len() != e.element.state_len() {
                return Err(invalid("Invalid initial element history."));
            }
            finite(&state)?;
            states.push(state);
        }
        let mut result = Self {
            linear,
            elements,
            accepted_states: states,
            accepted_factor: None,
        };
        let u = result.linear.accepted_displacement().to_vec();
        result.evaluate(&u, &mut || Ok(()))?;
        Ok(result)
    }

    /// Linear system and its accepted/trial displacements, read-only.
    pub fn linear_system(&self) -> &LinearSystem<B> {
        &self.linear
    }

    /// Histories from the last accepted step, in variable-element order.
    pub fn accepted_element_states(&self) -> &[Vec<Real>] {
        &self.accepted_states
    }

    /// Last absolute factor accepted by Newton; None before the first step.
    pub fn accepted_factor(&self) -> Option<Real> {
        self.accepted_factor
    }

    fn evaluate(
        &mut self,
        free: &[Real],
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<Evaluation> {
        check()?;
        let global = self.linear.constraints().expand(free)?;
        let mut internal = self.linear.global_internal_force(free)?;
        let responses = self
            .elements
            .iter()
            .zip(&self.accepted_states)
            .map(|(e, state)| {
                check()?;
                e.respond(&global, state)
            })
            .collect::<EResult<Vec<_>>>()?;
        for (e, response) in self.elements.iter().zip(&responses) {
            check()?;
            assemble_vector(&mut internal, &e.location, &response.internal_force)?;
        }
        let blocks = self
            .elements
            .iter()
            .zip(&responses)
            .map(|(e, r)| (e.location.as_slice(), &r.tangent))
            .collect::<Vec<_>>();
        let tangent = self.linear.tangent_with_blocks(&blocks)?;
        Ok(Evaluation {
            tangent,
            internal,
            states: responses.into_iter().map(|r| r.trial_state).collect(),
        })
    }
}

/// Independent criteria; None disables a criterion, and meeting only one active
/// criterion is never enough. Tolerances are relative, finite, and strictly positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewtonOptions {
    /// Maximum number of corrections/factorizations per step; must be > 0.
    pub max_iterations: u32,
    /// Tolerance for ‖R_f‖∞ / force_scale.
    pub force_tolerance: Option<Real>,
    /// Tolerance for ‖Δu_f‖∞ / displacement_scale.
    pub displacement_tolerance: Option<Real>,
    /// Tolerance for |Δu_fᵀR_f| / energy_scale.
    pub energy_tolerance: Option<Real>,
    /// Positive SI force/moment scale, chosen by the caller.
    pub force_scale: Real,
    /// Positive SI translation/rotation scale, chosen by the caller.
    pub displacement_scale: Real,
    /// Positive SI work scale, chosen by the caller.
    pub energy_scale: Real,
}

impl Default for NewtonOptions {
    fn default() -> Self {
        Self {
            max_iterations: 25,
            force_tolerance: Some(1e-10),
            displacement_tolerance: None,
            energy_tolerance: None,
            force_scale: 1.0,
            displacement_scale: 1.0,
            energy_scale: 1.0,
        }
    }
}

impl NewtonOptions {
    fn validate(&self) -> EResult<()> {
        let tolerances = [
            self.force_tolerance,
            self.displacement_tolerance,
            self.energy_tolerance,
        ];
        let scales = [self.force_scale, self.displacement_scale, self.energy_scale];
        finite(&scales)?;
        for tolerance in tolerances.into_iter().flatten() {
            finite(&[tolerance])?;
        }
        if self.max_iterations == 0
            || tolerances.iter().all(Option::is_none)
            || scales.iter().any(|s| *s <= 0.0)
            || tolerances.into_iter().flatten().any(|t| t <= 0.0)
        {
            return Err(invalid("Newton: invalid maximum, criteria, or scales."));
        }
        Ok(())
    }
}

/// Report of an attempted step; norms at the end of its last correction.
#[derive(Clone, Debug, PartialEq)]
pub struct NewtonStepReport {
    /// Requested absolute factor; not an increment relative to the previous step.
    pub load_factor: Real,
    /// Corrections performed; one factorization and solve per correction.
    pub iterations: u32,
    /// ‖R_f‖∞ in SI after updating u.
    pub force_norm: Real,
    /// ‖Δu_f‖∞ of the last correction in SI.
    pub displacement_norm: Real,
    /// |Δu_fᵀR_f| in SI with the updated residual.
    pub energy_norm: Real,
    /// Force norm divided by its scale.
    pub relative_force_norm: Real,
    /// Displacement norm divided by its scale.
    pub relative_displacement_norm: Real,
    /// Energy norm divided by its scale.
    pub relative_energy_norm: Real,
    /// All active tolerances satisfied.
    pub converged: bool,
}

/// Terminal result; vectors and histories are always the ACCEPTED ones.
#[derive(Clone, Debug, PartialEq)]
pub struct NewtonResults {
    /// Completed or NotConverged; infrastructure errors use EResult.
    pub status: Status,
    /// Requested case.
    pub case: CaseRef,
    /// Options and scales actually used.
    pub options: NewtonOptions,
    /// Attempts in order, including the failed step; no subsequent step is attempted.
    pub steps: Vec<NewtonStepReport>,
    /// Last accepted factor, or None if no step was accepted.
    pub accepted_factor: Option<Real>,
    /// Free displacements of the last accepted step, m/rad.
    pub free_displacements: Vec<Real>,
    /// Accepted global displacements with constant prescribed values, m/rad.
    pub global_displacements: Vec<Real>,
    /// Accepted histories, independent copy in element order.
    pub element_states: Vec<Vec<Real>>,
}

/// Static full Newton with load control and explicit commit per step.
pub struct StaticNewton;

impl StaticNewton {
    /// Method identifier.
    pub const KIND: &'static str = "static.newton";

    /// Runs exactly the list of factors λ_k on one case. These are absolute
    /// factors: [0.25, 0.5, 1] ends at F_total, not 1.75 F_total. Unloading,
    /// zero/negative factors, and explicit repetitions are allowed.
    /// Validates the entire list before acceptance. ENG-E-0010 for empty list/options,
    /// ENG-E-0008 for nonfinite values, ENG-E-0006 for singular tangent with node/DOF.
    /// On nonconvergence or error, discards trials and retains u/history of the
    /// last accepted step, without retries. Does not implement the analysis/1 facade.
    pub fn run<B: LinAlgBackend>(
        system: &mut NewtonSystem<B>,
        case: &CaseRef,
        factors: &[Real],
        options: NewtonOptions,
    ) -> EResult<NewtonResults> {
        Self::run_with(system, case, factors, options, || Ok(()))
    }

    /// Cooperative variant: checks before each step, evaluation, and iteration
    /// and before accepting state. On error, retains the last accepted step
    /// and reverts trial state, like [`Self::run`].
    pub fn run_with<B: LinAlgBackend>(
        system: &mut NewtonSystem<B>,
        case: &CaseRef,
        factors: &[Real],
        options: NewtonOptions,
        mut check: impl FnMut() -> EResult<()>,
    ) -> EResult<NewtonResults> {
        let result = Self::run_inner(system, case, factors, options, &mut check);
        system.linear.rollback();
        result
    }

    fn run_inner<B: LinAlgBackend>(
        system: &mut NewtonSystem<B>,
        case: &CaseRef,
        factors: &[Real],
        options: NewtonOptions,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<NewtonResults> {
        options.validate()?;
        if factors.is_empty() {
            return Err(invalid("Newton: empty factor list."));
        }
        finite(factors)?;
        // Also validates λF products before the first commit.
        for &factor in factors {
            check()?;
            system.linear.global_force(case, factor)?;
        }
        let mut steps = Vec::new();
        let mut status = Status::Completed;
        for &factor in factors {
            check()?;
            let force = system.linear.global_force(case, factor)?;
            let mut u = system.linear.accepted_displacement().to_vec();
            let mut evaluation = system.evaluate(&u, check)?;
            let mut residual = free_residual(system, &force, &evaluation.internal)?;
            for iteration in 1..=options.max_iterations {
                check()?;
                let delta = system.linear.solve_correction_with_check(
                    &evaluation.tangent,
                    &residual,
                    check,
                )?;
                for (value, correction) in u.iter_mut().zip(&delta) {
                    *value += correction;
                }
                finite(&u)?;
                evaluation = system.evaluate(&u, check)?;
                residual = free_residual(system, &force, &evaluation.internal)?;
                let report = report(factor, iteration, &residual, &delta, options)?;
                if report.converged {
                    check()?;
                    // The entire response and history were validated at exactly u.
                    // Linear accept is fallible/atomic; only buffers are moved afterward.
                    system.linear.accept(&u)?;
                    system.accepted_states = evaluation.states;
                    system.accepted_factor = Some(factor);
                    steps.push(report);
                    break;
                }
                if iteration == options.max_iterations {
                    steps.push(report);
                    status = Status::NotConverged;
                    break;
                }
            }
            if status == Status::NotConverged {
                break;
            }
        }
        let free_displacements = system.linear.accepted_displacement().to_vec();
        Ok(NewtonResults {
            status,
            case: case.clone(),
            options,
            steps,
            accepted_factor: system.accepted_factor,
            global_displacements: system.linear.constraints().expand(&free_displacements)?,
            free_displacements,
            element_states: system.accepted_states.clone(),
        })
    }
}

fn free_residual<B: LinAlgBackend>(
    system: &NewtonSystem<B>,
    force: &[Real],
    internal: &[Real],
) -> EResult<Vec<Real>> {
    let result = system
        .linear
        .constraints()
        .free_dofs()
        .iter()
        .map(|i| force[i.0 as usize] - internal[i.0 as usize])
        .collect::<Vec<_>>();
    finite(&result)?;
    Ok(result)
}

fn report(
    factor: Real,
    iterations: u32,
    residual: &[Real],
    delta: &[Real],
    options: NewtonOptions,
) -> EResult<NewtonStepReport> {
    let norm = |v: &[Real]| v.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    let mut work = 0.0;
    for (&d, &r) in delta.iter().zip(residual) {
        let term = d * r;
        finite(&[term])?;
        work += term;
        finite(&[work])?;
    }
    let force_norm = norm(residual);
    let displacement_norm = norm(delta);
    let energy_norm = work.abs();
    let relative_force_norm = force_norm / options.force_scale;
    let relative_displacement_norm = displacement_norm / options.displacement_scale;
    let relative_energy_norm = energy_norm / options.energy_scale;
    finite(&[
        relative_force_norm,
        relative_displacement_norm,
        relative_energy_norm,
    ])?;
    let satisfied = |t: Option<Real>, value: Real| t.is_none_or(|tol| value <= tol);
    let converged = satisfied(options.force_tolerance, relative_force_norm)
        && satisfied(options.displacement_tolerance, relative_displacement_norm)
        && satisfied(options.energy_tolerance, relative_energy_norm);
    Ok(NewtonStepReport {
        load_factor: factor,
        iterations,
        force_norm,
        displacement_norm,
        energy_norm,
        relative_force_norm,
        relative_displacement_norm,
        relative_energy_norm,
        converged,
    })
}
