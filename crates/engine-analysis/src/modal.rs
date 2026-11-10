//! `modal` analysis: constant K, SI masses, and undamped free vibration.
//!
//! Uses K_ff and M_ff from [`LinearSystem`] and `lowest_modes`, without code factors.
//! Global modes are zero at supports, even with settlements: they are
//! perturbations about equilibrium, not static displacements.
//! Γ=φᵀM_ff r_f/(φᵀM_ffφ), M*= (φᵀM_ff r_f)² for mass-normalized φ.
//! The user direction is normalized with `rigid_translation`.
//!
//! With ALL modes, ΣM*=r_fᵀM_ff r_f. The global mass used as the denominator
//! of the fractions includes supports and may exceed this free mass.
//! M_fp r_p is not used to excite free DOFs. For a consistent matrix
//! with supports, the completeness identity is for M_ff, not the global mass.
//!
//! Guyan reduction (ENG-LA-009) condenses exactly zero rows/columns of M_ff after
//! supports and diaphragms; recovers φs before participation and spectrum calculations.
//! Singular/nonpositive Kss yields ENG-E-0014 with node/DOF; negative mass or absence
//! of free mass yields ENG-E-0006. Without zero rows, the numerical path is preserved.
//! Also propagates the multiplicity cutoff and dense limits from
//! [`engine_linalg::lowest_modes`]. Does not distinguish mechanisms from rigid modes.

use engine_assembly::LinearSystem;
use engine_contracts::MassKind;
use engine_core::math;
use engine_core::{EResult, EngineError, Real};
use engine_linalg::{LinAlgBackend, SparseMatrix};
use engine_loads::mass::{rigid_translation, total_mass};

/// Stateless linear free-vibration method.
pub struct Modal;

/// Selected spectrum and directional sums, in the requested order.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalResults {
    /// Mass type explicitly chosen by the user.
    pub mass_kind: MassKind,
    /// Modes in ascending frequency order.
    pub modes: Vec<ModalMode>,
    /// Totals for each direction, in input order.
    pub directions: Vec<ModalDirection>,
    /// Returned modes with numerically zero stiffness: rigid modes or mechanisms.
    pub rigid_mode_count: usize,
}

/// One mass-normalized mode, without damping.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalMode {
    /// λ=ω², in s⁻².
    pub eigenvalue: Real,
    /// Angular frequency ω, in rad/s.
    pub angular_frequency: Real,
    /// Frequency f=ω/(2π), in Hz.
    pub frequency_hz: Real,
    /// Period T=2π/ω, in s; None means infinite period (λ=0).
    pub period: Option<Real>,
    /// φ in reduced order, φᵀM_ffφ=1.
    pub free_shape: Vec<Real>,
    /// φ in global order, with zeros at prescribed DOFs.
    pub global_shape: Vec<Real>,
    /// Participation per direction, in input order.
    pub participation: Vec<ModalParticipation>,
}

/// Participation and effective mass of one mode for a unit direction.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalParticipation {
    /// Γ=φᵀM_ff r_f/(φᵀM_ffφ), with the sign of the mode shape.
    pub factor: Real,
    /// M*=(φᵀM_ff r_f)², in kg, with mass-normalized φ.
    pub effective_mass: Real,
    /// Sum of M* up to and including this mode, in kg.
    pub cumulative_effective_mass: Real,
    /// M*/directional global mass; None if the total mass is zero.
    pub mass_fraction: Option<Real>,
    /// Cumulative mass/directional global mass; None if the total mass is zero.
    pub cumulative_mass_fraction: Option<Real>,
}

/// Directional totals for the requested set of modes.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalDirection {
    /// Original user direction; evaluation uses its unit version.
    pub direction: [Real; 3],
    /// Global mass rᵀMr in kg, including supports (`engine_loads::mass::total_mass`).
    pub total_mass: Real,
    /// Mass r_fᵀM_ff r_f in kg: sum limit with all free modes.
    pub free_mass: Real,
    /// Sum of effective masses of the requested modes, in kg.
    pub cumulative_effective_mass: Real,
    /// Previous sum/global mass; None if the total mass is zero.
    pub cumulative_mass_fraction: Option<Real>,
}

impl Modal {
    /// Public method identifier.
    pub const KIND: &'static str = "modal";

    /// Computes exactly `count` modes and participation in each direction.
    /// Does not modify accepted or trial states. `count=0` returns empty
    /// modal lists; preserves the validation semantics of `lowest_modes`.
    /// ENG-E-0010: empty direction list, zero direction, count>n;
    /// ENG-E-0008: nonfinite values/overflows; ENG-E-0011: direction with missing
    /// DOFs; ENG-E-0012: incompatible backend result. Propagates the
    /// other mass/spectrum diagnostics described in the module.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        count: usize,
        kind: MassKind,
        directions: &[[Real; 3]],
    ) -> EResult<ModalResults> {
        Self::run_with(system, count, kind, directions, |_| Ok(()))
    }

    /// Cooperative variant: callback after the eigensolver (0) and each mode.
    /// Does not accept state; to also check the eigensolver use `run_with_check`.
    pub fn run_with<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        count: usize,
        kind: MassKind,
        directions: &[[Real; 3]],
        checkpoint: impl FnMut(usize) -> EResult<()>,
    ) -> EResult<ModalResults> {
        Self::run_with_check(system, count, kind, directions, checkpoint, || Ok(()))
    }

    /// Checks independent of progress, including the reference eigensolver.
    /// Polls per direction and mode without adding progress events.
    pub fn run_with_check<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        count: usize,
        kind: MassKind,
        directions: &[[Real; 3]],
        mut checkpoint: impl FnMut(usize) -> EResult<()>,
        mut check: impl FnMut() -> EResult<()>,
    ) -> EResult<ModalResults> {
        check()?;
        if directions.is_empty() {
            return Err(EngineError::new(
                "ENG-E-0010",
                "Modal analysis: missing directions.",
            ));
        }
        let free = system.constraints().free_dofs();
        let mut translations = Vec::new();
        let mut totals = Vec::new();
        for &direction in directions {
            check()?;
            let mut r = rigid_translation(system.dof_map(), direction)?;
            let total = total_mass(system.dof_map(), system.global_mass(kind), direction)?;
            for (value, prescribed) in r.iter_mut().zip(system.constraints().prescribed_values()) {
                if prescribed.is_some() {
                    *value = 0.0;
                }
            }
            let free_mass = product(system.global_mass(kind), &r, &r)?;
            totals.push(ModalDirection {
                direction,
                total_mass: total,
                free_mass,
                cumulative_effective_mass: 0.0,
                cumulative_mass_fraction: fraction(0.0, total)?,
            });
            translations.push(r);
        }
        let n = free.len();
        let condensed = system.has_massless_dofs(kind)?;
        let spectrum = system.lowest_modes_with_check(kind, count, &mut check)?;
        if spectrum.eigenvalues.len() != count
            || spectrum.vectors.rows() != n
            || spectrum.vectors.cols() != count
            || spectrum.rigid_mode_count > count
        {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Modal analysis: incompatible spectrum dimensions.",
            ));
        }
        super::finite(&spectrum.eigenvalues)?;
        super::finite(spectrum.vectors.values())?;
        if spectrum.eigenvalues.iter().any(|&v| v < 0.0)
            || spectrum.eigenvalues.windows(2).any(|v| v[0] > v[1])
        {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Modal analysis: negative or unordered spectrum.",
            ));
        }
        let free = system.constraints().free_dofs();
        checkpoint(0)?;
        let matrix = system.global_mass(kind);
        let mut modes = Vec::new();
        for (col, &eigenvalue) in spectrum.eigenvalues.iter().enumerate() {
            check()?;
            let mut global = vec![0.0; system.dof_map().len()];
            for (row, index) in free.iter().enumerate() {
                global[index.0 as usize] = spectrum.vectors.values()[row * count + col];
            }
            if condensed && !system.constraints().diaphragms().is_empty() {
                // Homogeneous perturbation T_f φ: never add settlements.
                // Only after Guyan reduction: the existing path without condensation preserves
                // its results and hashes, including the pending scope of
                // API-W-DIAPHRAGM-RECOVERY (#84). Extending it requires another change.
                let mut expanded = vec![0.0; global.len()];
                for (row, value) in expanded.iter_mut().enumerate() {
                    for &(index, coefficient) in system
                        .constraints()
                        .free_transformation_row(engine_core::DofIndex(row as u32))?
                    {
                        *value +=
                            coefficient * spectrum.vectors.values()[index.0 as usize * count + col];
                    }
                }
                global = expanded;
            }
            let norm = product(matrix, &global, &global)?;
            if norm <= 0.0 {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Modal analysis: mode shape has no positive mass norm.",
                ));
            }
            for value in &mut global {
                *value /= math::sqrt(norm);
            }
            super::finite(&global)?;
            let denominator = product(matrix, &global, &global)?;
            if denominator <= 0.0 {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Modal analysis: nonpositive normalized mass norm.",
                ));
            }
            let mut participation = Vec::new();
            for (r, total) in translations.iter().zip(&mut totals) {
                check()?;
                let projection = product(matrix, &global, r)?;
                let factor = projection / denominator;
                let effective_mass = projection * projection;
                total.cumulative_effective_mass += effective_mass;
                super::finite(&[factor, effective_mass, total.cumulative_effective_mass])?;
                total.cumulative_mass_fraction =
                    fraction(total.cumulative_effective_mass, total.total_mass)?;
                participation.push(ModalParticipation {
                    factor,
                    effective_mass,
                    cumulative_effective_mass: total.cumulative_effective_mass,
                    mass_fraction: fraction(effective_mass, total.total_mass)?,
                    cumulative_mass_fraction: total.cumulative_mass_fraction,
                });
            }
            let angular_frequency = math::sqrt(eigenvalue);
            let frequency_hz = angular_frequency / std::f64::consts::TAU;
            let period = if eigenvalue == 0.0 {
                None
            } else {
                Some(std::f64::consts::TAU / angular_frequency)
            };
            super::finite(&[angular_frequency, frequency_hz])?;
            if let Some(t) = period {
                super::finite(&[t])?;
            }
            modes.push(ModalMode {
                eigenvalue,
                angular_frequency,
                frequency_hz,
                period,
                free_shape: free.iter().map(|i| global[i.0 as usize]).collect(),
                global_shape: global,
                participation,
            });
            checkpoint(col + 1)?;
        }
        Ok(ModalResults {
            mass_kind: kind,
            modes,
            directions: totals,
            rigid_mode_count: spectrum.rigid_mode_count,
        })
    }
}

fn fraction(value: Real, total: Real) -> EResult<Option<Real>> {
    if total == 0.0 {
        return Ok(None);
    }
    let fraction = value / total;
    super::finite(&[fraction])?;
    Ok(Some(fraction))
}

fn product(matrix: &SparseMatrix, a: &[Real], b: &[Real]) -> EResult<Real> {
    let mut sum = 0.0;
    for (row, &value) in a.iter().enumerate() {
        for p in matrix.row_offsets()[row]..matrix.row_offsets()[row + 1] {
            let term = (value * matrix.values()[p]) * b[matrix.column_indices()[p]];
            super::finite(&[term])?;
            sum += term;
            super::finite(&[sum])?;
        }
    }
    Ok(sum)
}
