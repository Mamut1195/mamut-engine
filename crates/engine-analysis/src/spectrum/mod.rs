//! `spectrum` response spectrum, exclusively tabulated by the user.
//!
//! Sa is pseudoacceleration in m/s² for the declared ζ of each table;
//! q_n,max=Γ_n Sa(T_n,ζ_n)/ω_n² and u_n=φ_n q_n,max. Global equivalent
//! forces are F_n=Mφ_n Γ_n Sa. K, M, and participation come from [`Modal`].
//! Modes must have ω>0; no response is assigned to rigid modes.
//!
//! R_n=(K u_n−F_n)_p at all supports, with zero prescribed perturbations.
//! Signed base shear is −Σ_apoyos R_traslacional·d̂. Consistent-mass couplings
//! to supports are included in F_p; for masses only at
//! free DOFs of a shear building, V_n=M*_n Sa_n. Static loads,
//! settlements, and M_fp r_p terms are not added to modal participation.
//!
//! SRSS/CQC combine EACH scalar after recovery, using its modal
//! sign. Their envelopes are not a simultaneous or equilibrated state:
//! combined base shear is computed from the shears per mode,
//! never by summing reaction envelopes. For members, extract u_e from
//! each `global_displacements`, use `Element::recover` without static loads, and
//! combine each scalar with [`combine_modal`]. The assembled system retains
//! no Element objects/contexts and has no complete results facade.
//!
//! The user chooses table(s), damping ratio(s), interpolation, extrapolation,
//! direction(s), factors, and combination. There are no codes, built-in spectra,
//! damping correction, gravity, or default factors.

mod combination;
mod table;

pub use combination::{
    combine_directions, combine_modal, cqc_correlation, DirectionalCombination, ModalCombination,
};
pub use table::{Extrapolation, Interpolation, SpectrumUnits, TabulatedSpectrum};

use crate::{
    finite,
    modal::{Modal, ModalResults},
};
use engine_assembly::{LinearSystem, Reaction};
use engine_contracts::MassKind;
use engine_core::math;
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_linalg::{LinAlgBackend, SparseMatrix};

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", format!("Spectrum: {message}"))
}

fn validate_damping(zeta: Real) -> EResult<()> {
    finite(&[zeta])?;
    if !(0.0..1.0).contains(&zeta) {
        return Err(invalid("Damping ζ must be in [0,1)."));
    }
    Ok(())
}

/// Declared excitation: one common table or one table per mode, in modal order.
#[derive(Clone, Copy, Debug)]
pub struct SpectrumDirection<'a> {
    /// User global direction, finite and nonzero; normalized.
    pub direction: [Real; 3],
    /// Length 1 (common ζ) or `mode_count` (ζ/tables per mode); never empty.
    pub spectra: &'a [TabulatedSpectrum],
}

/// Explicit user options; does not implement Default.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumOptions {
    /// Nonzero number of modes requested from [`Modal::run`].
    pub mode_count: usize,
    /// System mass type.
    pub mass_kind: MassKind,
    /// Combination of signed modal responses.
    pub modal_combination: ModalCombination,
    /// Combination of directional envelopes.
    pub directional_combination: DirectionalCombination,
    /// One finite nonnegative factor per direction, in the same order.
    pub directional_factors: Vec<Real>,
}

/// Displacements, reactions, and base shear of a mode or envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumResponse {
    /// Free displacements in reduced order, m/rad.
    pub free_displacements: Vec<Real>,
    /// Global displacements, m/rad; zero perturbations at supports.
    pub global_displacements: Vec<Real>,
    /// Reactions in global order, N/N·m; signed per mode, combined magnitudes.
    pub reactions: Vec<Reaction>,
    /// Base shear projected onto d̂, N; signed per mode, combined magnitude.
    pub base_shear: Real,
}

/// Signed peak modal response in one direction; does not represent a history.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumModeResponse {
    /// Damping ratio ζ declared in the table used.
    pub damping: Real,
    /// Tabulated/interpolated Sa(T_n,ζ_n), m/s².
    pub spectral_acceleration: Real,
    /// Peak modal coordinate Γ_n Sa/ω_n²; its sign depends on the sign of φ.
    /// Units of q depend on the mass normalization of φ; φq is in m/rad.
    pub modal_displacement: Real,
    /// Global equivalent forces Mφ_n Γ_n Sa, N/N·m, including supports.
    pub equivalent_forces: Vec<Real>,
    /// Signed physical response recovered before combination.
    pub response: SpectrumResponse,
}

/// Response per direction with traceability of tables and combinations.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumDirectionResult {
    /// Global unit direction used.
    pub unit_direction: [Real; 3],
    /// Input tables, including their interpolation/extrapolation choices.
    pub spectra: Vec<TabulatedSpectrum>,
    /// Responses per mode, in the same order as `modal.modes`.
    pub modes: Vec<SpectrumModeResponse>,
    /// SRSS or CQC envelope per component.
    pub combined: SpectrumResponse,
}

/// Complete result; accepted and trial states are not modified.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumResults {
    /// Modes and participations from ENG-ANL-002 for these same directions.
    pub modal: ModalResults,
    /// Declared options and applied factors, without code interpretation.
    pub options: SpectrumOptions,
    /// Directional results, in input order.
    pub directions: Vec<SpectrumDirectionResult>,
    /// Directional envelope; base shear combines the projections of each direction.
    /// This is neither a projection onto a new direction nor simultaneous equilibrium.
    pub combined: SpectrumResponse,
}

/// Stateless linear spectral method without a default spectrum.
pub struct Spectrum;

impl Spectrum {
    /// Public method identifier.
    pub const KIND: &'static str = "spectrum";

    /// Computes peak modal responses, envelopes, and directional combination.
    /// ENG-E-0010: empty modes/directions/tables, ω=0, zero direction,
    /// invalid factors, or period outside the table with Extrapolation::Error.
    /// ENG-E-0008: nonfinite values/overflow. Propagates errors from the modal
    /// module and assembler. Does not modify or accept system states.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        directions: &[SpectrumDirection<'_>],
        options: &SpectrumOptions,
    ) -> EResult<SpectrumResults> {
        Self::run_with(system, directions, options, |_, _| Ok(()))
    }

    /// Cooperative variant: modal, spectrumMode, and spectrumDirection stages.
    /// Callback after each mode/direction; `run_with_check` controls the kernels.
    /// An error stops analysis without accepting state.
    pub fn run_with<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        directions: &[SpectrumDirection<'_>],
        options: &SpectrumOptions,
        checkpoint: impl FnMut(&'static str, usize) -> EResult<()>,
    ) -> EResult<SpectrumResults> {
        Self::run_with_check(system, directions, options, checkpoint, || Ok(()))
    }

    /// Control independent of progress, including the eigensolver and CQC.
    pub fn run_with_check<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        directions: &[SpectrumDirection<'_>],
        options: &SpectrumOptions,
        mut checkpoint: impl FnMut(&'static str, usize) -> EResult<()>,
        mut check: impl FnMut() -> EResult<()>,
    ) -> EResult<SpectrumResults> {
        check()?;
        if options.mode_count == 0 || directions.is_empty() {
            return Err(invalid("Nonempty modes and directions are required."));
        }
        // Validates factors even if all responses are zero.
        combine_directions(
            &vec![0.0; directions.len()],
            &options.directional_factors,
            options.directional_combination,
        )?;
        for input in directions {
            if input.spectra.len() != 1 && input.spectra.len() != options.mode_count {
                return Err(invalid("Declare one shared table or one table per mode."));
            }
        }
        let input_directions: Vec<_> = directions.iter().map(|d| d.direction).collect();
        let modal = Modal::run_with_check(
            system,
            options.mode_count,
            options.mass_kind,
            &input_directions,
            |n| checkpoint("modal", n),
            &mut check,
        )?;
        let frequencies: Vec<_> = modal.modes.iter().map(|m| m.angular_frequency).collect();
        let mut results = Vec::new();
        for (d, input) in directions.iter().enumerate() {
            check()?;
            let unit_direction = unit(input.direction)?;
            let dampings: Vec<_> = (0..options.mode_count)
                .map(|n| input.spectra[if input.spectra.len() == 1 { 0 } else { n }].damping())
                .collect();
            let rho = combination::correlations_with_check(
                &frequencies,
                &dampings,
                options.modal_combination,
                &mut check,
            )?;
            let mut modes = Vec::new();
            for (n, mode) in modal.modes.iter().enumerate() {
                check()?;
                let table = &input.spectra[if input.spectra.len() == 1 { 0 } else { n }];
                let period = mode
                    .period
                    .ok_or_else(|| invalid("Rigid mode has no finite period."))?;
                let sa = table.acceleration(period)?;
                let acceleration = mode.participation[d].factor * sa;
                let q = acceleration / mode.eigenvalue;
                finite(&[acceleration, q])?;
                let global: Vec<_> = mode.global_shape.iter().map(|v| v * q).collect();
                finite(&global)?;
                let mut force =
                    multiply(system.global_mass(options.mass_kind), &mode.global_shape)?;
                for value in &mut force {
                    *value *= acceleration;
                }
                finite(&force)?;
                let internal = multiply(system.stiffness(), &global)?;
                let mut reactions = Vec::new();
                let mut base_shear = 0.0;
                for (i, prescribed) in system.constraints().prescribed_values().iter().enumerate() {
                    if prescribed.is_some() {
                        let (node, dof) = system.dof_map().label(DofIndex(i as u32))?;
                        let value = internal[i] - force[i];
                        finite(&[value])?;
                        if dof.index() < 3 {
                            base_shear -= value * unit_direction[dof.index()];
                            finite(&[base_shear])?;
                        }
                        reactions.push(Reaction { node, dof, value });
                    }
                }
                let free_displacements = system
                    .constraints()
                    .free_dofs()
                    .iter()
                    .map(|i| global[i.0 as usize])
                    .collect();
                modes.push(SpectrumModeResponse {
                    damping: table.damping(),
                    spectral_acceleration: sa,
                    modal_displacement: q,
                    equivalent_forces: force,
                    response: SpectrumResponse {
                        free_displacements,
                        global_displacements: global,
                        reactions,
                        base_shear,
                    },
                });
                checkpoint("spectrumMode", d * options.mode_count + n + 1)?;
            }
            let responses: Vec<_> = modes.iter().map(|m| &m.response).collect();
            let combined = combine_responses(&responses, |v| {
                combination::quadratic_with_check(v, &rho, &mut check)
            })?;
            results.push(SpectrumDirectionResult {
                unit_direction,
                spectra: input.spectra.to_vec(),
                modes,
                combined,
            });
            checkpoint("spectrumDirection", d + 1)?;
        }
        let responses: Vec<_> = results.iter().map(|d| &d.combined).collect();
        let combined = combine_responses(&responses, |v| {
            check()?;
            combine_directions(
                v,
                &options.directional_factors,
                options.directional_combination,
            )
        })?;
        Ok(SpectrumResults {
            modal,
            options: options.clone(),
            directions: results,
            combined,
        })
    }
}

fn unit(direction: [Real; 3]) -> EResult<[Real; 3]> {
    finite(&direction)?;
    let scale = direction.iter().fold(0.0_f64, |s, v| s.max(v.abs()));
    if scale == 0.0 {
        return Err(invalid("Zero excitation direction."));
    }
    let d = direction.map(|v| v / scale);
    let norm = math::hypot(math::hypot(d[0], d[1]), d[2]);
    Ok(d.map(|v| v / norm))
}

fn multiply(matrix: &SparseMatrix, vector: &[Real]) -> EResult<Vec<Real>> {
    if matrix.cols() != vector.len() {
        return Err(EngineError::new(
            "ENG-E-0012",
            "Incompatible matrix product.",
        ));
    }
    finite(vector)?;
    let mut result = vec![0.0; matrix.rows()];
    for (i, out) in result.iter_mut().enumerate() {
        for p in matrix.row_offsets()[i]..matrix.row_offsets()[i + 1] {
            let term = matrix.values()[p] * vector[matrix.column_indices()[p]];
            finite(&[term])?;
            *out += term;
            finite(&[*out])?;
        }
    }
    Ok(result)
}

fn combine_responses(
    responses: &[&SpectrumResponse],
    mut combine: impl FnMut(&[Real]) -> EResult<Real>,
) -> EResult<SpectrumResponse> {
    let first = responses
        .first()
        .ok_or_else(|| invalid("Empty responses."))?;
    let mut global = Vec::new();
    for i in 0..first.global_displacements.len() {
        global.push(combine(
            &responses
                .iter()
                .map(|r| r.global_displacements[i])
                .collect::<Vec<_>>(),
        )?);
    }
    let mut free = Vec::new();
    for i in 0..first.free_displacements.len() {
        free.push(combine(
            &responses
                .iter()
                .map(|r| r.free_displacements[i])
                .collect::<Vec<_>>(),
        )?);
    }
    let mut reactions = Vec::new();
    for (i, reaction) in first.reactions.iter().enumerate() {
        let value = combine(
            &responses
                .iter()
                .map(|r| r.reactions[i].value)
                .collect::<Vec<_>>(),
        )?;
        reactions.push(Reaction { value, ..*reaction });
    }
    let base_shear = combine(&responses.iter().map(|r| r.base_shear).collect::<Vec<_>>())?;
    Ok(SpectrumResponse {
        free_displacements: free,
        global_displacements: global,
        reactions,
        base_shear,
    })
}
