#![forbid(unsafe_code)]
//! `static.linear`, [`newton`] (`static.newton`), [`pdelta`], [`modal`], [`spectrum`], and [`buckling`] analysis, in SI units.
//!
//! `static.linear` solves independent cases with a single factorization
//! of K_red = T_fᵀ K T_f over the free independent DOFs. Recovers dependent
//! and prescribed DOFs with u = T_f q_f + u_0. Includes settlements without scaling them by the load factor; does not modify the
//! accepted or trial state. The `combination` module superposes results using
//! declarative model combinations; `StaticLinear` solves only cases.
//! With zero prescribed displacements, u(A+B)=u(A)+u(B); with settlements, subtract
//! the unloaded solution u(0) once. Does not incorporate Kg, iterations, or regularization.
//!
//! [`buckling`] (`buckling.linear`) solves the finite eigenvalues of
//! `K_ff + λ Kg_ff = 0` at the reference geometric stiffness. It does not
//! accept state. The module states the sign of Kg and the rejection of a
//! singular or indefinite reference metric.
//!
//! Right-handed global axes; D2 uses XY and Rz, D3 XYZ and Rx/Ry/Rz. Original
//! coordinates in m (D2 requires Z=0), translations in m, rotations in rad, forces
//! in N, and moments in N·m. Undeformed reference positions are used.
//! Equilibrium sums ALL physical global loads, including equivalent element
//! loads and loads applied at supports, plus reactions
//! from engine-assembly. The moment adds `(position-point) × force`, with the
//! right-hand sign, and nodal couples. Does not use the reduced RHS for this sum.
#![deny(missing_docs)]

pub mod buckling;
pub mod combination;
pub mod modal;
pub mod pdelta;
pub mod pile;
pub mod results;
pub mod spectrum;

use engine_assembly::{LinearSystem, Reaction};
use engine_contracts::CaseRef;
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_linalg::LinAlgBackend;

pub mod newton;

/// Stateless linear static analysis method.
pub struct StaticLinear;

/// Results in the same order as the requested list.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticResults {
    /// One independent result per case, without automatic acceptance.
    pub cases: Vec<StaticCaseResult>,
}

/// Physical solution and checks for a scaled case.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticCaseResult {
    /// Declared system case.
    pub case: CaseRef,
    /// Factor applied only to loads.
    pub factor: Real,
    /// Free independent displacements in reduced order, m/rad.
    pub free_displacements: Vec<Real>,
    /// Complete displacements in global order, including dependent and prescribed DOFs, m/rad.
    pub global_displacements: Vec<Real>,
    /// Reactions at all prescribed DOFs, N/N·m.
    pub reactions: Vec<Reaction>,
    /// Global force and moment equilibrium.
    pub equilibrium: EquilibriumReport,
    /// `‖T_fᵀ(K u-F)‖∞` in SI components (N/N·m).
    pub free_residual_norm: Real,
    /// Previous norm / max(1, ‖T_fᵀ K u‖∞, ‖T_fᵀ F‖∞).
    /// Numerical indicator of SI components; mixes forces and moments,
    /// so it is not a physical norm invariant under changes of units.
    pub relative_free_residual: Real,
}

/// Global equilibrium with component sums and moments including lever arms.
#[derive(Clone, Debug, PartialEq)]
pub struct EquilibriumReport {
    /// Physical sum per DOF `[Fx,Fy,Fz,Mx,My,Mz]`; couples WITHOUT force lever arms.
    pub applied_force_sum: [Real; 6],
    /// Sum of reactions per DOF, same order and convention.
    pub reaction_sum: [Real; 6],
    /// Sum of the two preceding vectors; the last three values are
    /// residual nodal couples, not the total equilibrium moment.
    pub residual_force: [Real; 6],
    /// Total residual moment `[Mx,My,Mz]`, including force lever arms, N·m.
    pub residual_moment: [Real; 3],
    /// Global moment reference point, m.
    pub moment_point: [Real; 3],
    /// Maximum of the six ratios: residual forces / max(1 N, sum of
    /// force magnitudes for that component) and total residual moments /
    /// max(1 N·m, sum of magnitudes of couples and cross-product terms for
    /// that component). Both loads and reactions contribute to the scales.
    /// A system without actions and in equilibrium has zero relative residual.
    pub relative_residual: Real,
}

impl StaticLinear {
    /// Public method identifier.
    pub const KIND: &'static str = "static.linear";

    /// Solves all cases with ONE factorization and moments about the origin.
    /// ENG-E-0010 for an empty/duplicate list; ENG-E-0003 for a missing case;
    /// ENG-E-0008 for nonfinite values; ENG-E-0006 for a mechanism with node/DOF in
    /// the message and entities; ENG-E-0012 for an invalid backend diagnostic.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        cases: &[(CaseRef, Real)],
    ) -> EResult<StaticResults> {
        Self::run_about(system, cases, [0.0; 3])
    }

    /// Like [`Self::run`], with moments about a finite global point in m.
    /// Does not alter or accept system state, even on errors.
    pub fn run_about<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        cases: &[(CaseRef, Real)],
        point: [Real; 3],
    ) -> EResult<StaticResults> {
        Self::run_about_with_check(system, cases, point, &mut || Ok(()))
    }

    /// Cooperative variant per case and during the reference factorization.
    pub fn run_about_with_check<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        cases: &[(CaseRef, Real)],
        point: [Real; 3],
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<StaticResults> {
        check()?;
        finite(&point)?;
        let solutions = system.solve_cases_with_check(cases, check)?;
        let mut results = Vec::new();
        for ((case, factor), free) in cases.iter().zip(solutions) {
            check()?;
            let global = system.constraints().expand(&free)?;
            let reactions = system.reactions(&free, case, *factor)?;
            let force = system.global_force(case, *factor)?;
            let (free_residual_norm, relative_free_residual) =
                free_residual(system, &global, &force)?;
            let equilibrium = equilibrium(system, &force, &reactions, point)?;
            results.push(StaticCaseResult {
                case: case.clone(),
                factor: *factor,
                free_displacements: free,
                global_displacements: global,
                reactions,
                equilibrium,
                free_residual_norm,
                relative_free_residual,
            });
        }
        Ok(StaticResults { cases: results })
    }
}

fn finite(values: &[Real]) -> EResult<()> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Analysis: non-finite input or result.",
        ));
    }
    Ok(())
}

fn free_residual<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    global: &[Real],
    force: &[Real],
) -> EResult<(Real, Real)> {
    let k = system.stiffness();
    let mut internal = vec![0.0; global.len()];
    for (row, value) in internal.iter_mut().enumerate() {
        for p in k.row_offsets()[row]..k.row_offsets()[row + 1] {
            let term = k.values()[p] * global[k.column_indices()[p]];
            finite(&[term])?;
            *value += term;
            finite(&[*value])?;
        }
    }
    let residual: Vec<_> = internal.iter().zip(force).map(|(k, f)| k - f).collect();
    let residual = system.constraints().project_force(&residual)?;
    let internal = system.constraints().project_force(&internal)?;
    let external = system.constraints().project_force(force)?;
    let mut norm: Real = 0.0;
    let mut scale: Real = 1.0;
    for ((residual, internal), external) in residual.into_iter().zip(internal).zip(external) {
        finite(&[residual])?;
        norm = norm.max(residual.abs());
        scale = scale.max(internal.abs()).max(external.abs());
    }
    Ok((norm, norm / scale))
}

#[derive(Default)]
struct Sum {
    components: [Real; 6],
    moments: [Real; 3],
    scales: [Real; 6],
}

impl Sum {
    fn add(&mut self, axis: usize, value: Real, arm: [Real; 3]) -> EResult<()> {
        self.components[axis] += value;
        if axis < 3 {
            self.scales[axis] += value.abs();
            // r × F for a force with only an axis component.
            let mut f = [0.0; 3];
            f[axis] = value;
            let moment = [
                arm[1] * f[2] - arm[2] * f[1],
                arm[2] * f[0] - arm[0] * f[2],
                arm[0] * f[1] - arm[1] * f[0],
            ];
            for (i, m) in moment.iter().enumerate() {
                self.moments[i] += m;
                self.scales[i + 3] += m.abs();
            }
        } else {
            self.moments[axis - 3] += value;
            self.scales[axis] += value.abs();
        }
        finite(&self.components)?;
        finite(&self.moments)?;
        finite(&self.scales)
    }
}

fn equilibrium<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    force: &[Real],
    reactions: &[Reaction],
    point: [Real; 3],
) -> EResult<EquilibriumReport> {
    let map = system.dof_map();
    let mut applied = Sum::default();
    let mut reaction = Sum::default();
    for (i, &value) in force.iter().enumerate() {
        let (node, dof) = map.label(DofIndex(i as u32))?;
        let coordinates = map.coordinates(node)?;
        let arm = std::array::from_fn(|i| coordinates[i] - point[i]);
        finite(&arm)?;
        applied.add(dof.index(), value, arm)?;
    }
    for r in reactions {
        let coordinates = map.coordinates(r.node)?;
        let arm = std::array::from_fn(|i| coordinates[i] - point[i]);
        finite(&arm)?;
        reaction.add(r.dof.index(), r.value, arm)?;
    }
    let residual_force = std::array::from_fn(|i| applied.components[i] + reaction.components[i]);
    let residual_moment = std::array::from_fn(|i| applied.moments[i] + reaction.moments[i]);
    finite(&residual_force)?;
    finite(&residual_moment)?;
    let mut relative_residual: Real = 0.0;
    for i in 0..6 {
        let scale = applied.scales[i] + reaction.scales[i];
        finite(&[scale])?;
        let residual = if i < 3 {
            residual_force[i]
        } else {
            residual_moment[i - 3]
        };
        relative_residual = relative_residual.max(residual.abs() / scale.max(1.0));
    }
    Ok(EquilibriumReport {
        applied_force_sum: applied.components,
        reaction_sum: reaction.components,
        residual_force,
        residual_moment,
        moment_point: point,
        relative_residual,
    })
}
