//! Multilayer 1D Terzaghi consolidation, linear elements, and consistent mass.
//!
//! Contract `consolidation/1`: z in m, positive downward; t in s; excess u and
//! Δσ in kPa, positive in compression. mv in 1/kPa, k in m/s, cv in m²/s, and γw in
//! kN/m³. Requires cv=k/(mv γw), with maximum relative discrepancy 1e-10.
//! No missing parameter is derived or substituted. Geometry is independent of the
//! geostatic profile: its thicknesses can be reused without requiring γ/γsat/zw,
//! which do not enter this saturated incremental problem.
//!
//! Conservative equation: mv u̇ − ((k/γw)u′)′ = mv Δσ̇. Shared nodes
//! impose continuity of u; flux is balanced weakly at interfaces.
//! Ideal drains u=0, other faces impermeable. Load uniform in z,
//! initially at rest: Δσ(0) is applied instantaneously, u(0)=Δσ(0) at interior
//! nodes and zero at drains. This interpolation has an initial error layer
//! of size h; s(0)=0 is not claimed exactly on a finite mesh.
//!
//! The table starts at t=0, interpolates linearly, and does not extrapolate. Time stepping
//! splits at all load breakpoints and output times. θ=0.5 by default;
//! θ<0.5 requires modal stability, checked using the Gershgorin bound for the pair
//! (K,M) documented in provenance/ENG-GEO-006.md. Crank–Nicolson oscillations
//! and negative results during unloading are not clipped.
//!
//! `U=s/[Δσ∫mv dz]` is the compressibility-weighted average degree. For Δσ=0,
//! returns None (even if residual pressure exists). With unloading, U may
//! leave `[0,1]`. s=∫mv(Δσ−u)dz is integrated exactly for the linear field.
//! The result retains all Assumed parameters, including load and γw.
//!
//! Reuses the engine-linalg factorization; its reference backend is
//! dense, O(n³) for factorization and O(n²) per solve. Assembly is CSR. Explicit
//! limits of this reference API: 2048 elements, 100000 outputs,
//! 8000000 output nodal values, and 1000000 substeps per interval.
//! ENG-E-0010 for range, mesh, stability, or
//! incompatible cv; 0003 for missing data; 0008 for nonfinite data/results.
use crate::{Diagnostic, GeoResult, Parameter, Provenance, Real};
use engine_core::{EngineError, Severity};
use engine_linalg::{
    DMat, LinAlgBackend, ReferenceBackend, SparseMatrix, SymFactorization, SymMatrix, Triplet,
};

/// Saturated layer; all physical data have explicit provenance.
#[derive(Clone, Debug)]
pub struct ConsolidationLayer {
    /// Positive thickness in m, contiguous with the preceding layer from z=0.
    pub thickness_m: Parameter,
    /// Positive cv in m²/s, required and checked against k/(mv γw).
    pub cv_m2_s: Option<Parameter>,
    /// Positive mv in 1/kPa, required.
    pub mv_per_kpa: Option<Parameter>,
    /// Positive hydraulic conductivity k in m/s, required.
    pub k_m_s: Option<Parameter>,
    /// Positive number of uniform elements in this layer.
    pub elements: usize,
}

/// Permeable faces; the others are impermeable, without prescribed flux.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drainage {
    /// Top drain at z=0.
    Top,
    /// Bottom drain at z=H.
    Bottom,
    /// Drains at z=0 and z=H.
    Double,
}

/// Point in a history of total stress increment uniform with depth.
#[derive(Clone, Copy, Debug)]
pub struct LoadPoint {
    /// Time in s, strictly increasing; the table starts exactly at 0.
    pub time_s: Real,
    /// Δσ in kPa and its provenance; required, may be zero or negative.
    pub delta_stress_kpa: Option<Parameter>,
}

/// Explicit numerical controls, without assumed physical parameters.
#[derive(Clone, Copy, Debug)]
pub struct ConsolidationOptions {
    /// θ in `[0,1]`; 0 explicit, 0.5 Crank–Nicolson, 1 implicit Euler.
    pub theta: Real,
    /// Positive maximum step in s; reduced to reach breakpoints and outputs.
    pub max_step_s: Real,
}
impl ConsolidationOptions {
    /// Crank–Nicolson; the caller always supplies the maximum step.
    pub const fn new(max_step_s: Real) -> Self {
        Self {
            theta: 0.5,
            max_step_s,
        }
    }
}

/// Field at nodes common to all outputs and integrated quantities.
#[derive(Clone, Debug)]
pub struct ConsolidationState {
    /// Time in s.
    pub time_s: Real,
    /// Interpolated Δσ, kPa.
    pub delta_stress_kpa: Real,
    /// u in kPa, aligned with ConsolidationResult::depths_m.
    pub excess_pore_pressure_kpa: Vec<Real>,
    /// Settlement positive downward, m; exact integral of the FE field.
    pub settlement_m: Real,
    /// U=s/(Δσ∫mv dz); undefined when Δσ=0, without clipping.
    pub mean_degree: Option<Real>,
}

/// Outputs and complete provenance of the incremental calculation.
#[derive(Clone, Debug)]
pub struct ConsolidationResult {
    /// Nodal depths in m, including all interfaces.
    pub depths_m: Vec<Real>,
    /// States, exactly in the requested time order.
    pub states: Vec<ConsolidationState>,
    /// Paths of all parameters declared Assumed, without duplicates.
    pub assumed_parameters: Vec<String>,
}
impl ConsolidationResult {
    /// Interpolates u(z,t) linearly at a depth within the profile.
    /// Time is selected by output index; no space/time extrapolation.
    pub fn pressure_at(&self, state_index: usize, depth_m: Real) -> GeoResult<Real> {
        finite(depth_m, "depth_m")?;
        let state = self
            .states
            .get(state_index)
            .ok_or_else(|| invalid("Output index is out of range."))?;
        // Also validates public fields if the caller modifies the result.
        if self.depths_m.len() < 2 || state.excess_pore_pressure_kpa.len() != self.depths_m.len() {
            return Err(invalid("Result dimensions have changed."));
        }
        for (i, pair) in self.depths_m.windows(2).enumerate() {
            if depth_m >= pair[0] && depth_m <= pair[1] && pair[1] > pair[0] {
                let r = (depth_m - pair[0]) / (pair[1] - pair[0]);
                return finite(
                    (1.0 - r) * state.excess_pore_pressure_kpa[i]
                        + r * state.excess_pore_pressure_kpa[i + 1],
                    "u(z,t)",
                );
            }
        }
        Err(invalid("Depth is outside the profile."))
    }
}

#[derive(Clone, Debug)]
struct Element {
    h: Real,
    mv: Real,
    conductivity: Real, // k/γw
}

/// Validated immutable consolidation profile, without default parameters.
#[derive(Clone, Debug)]
pub struct ConsolidationProfile {
    depths: Vec<Real>,
    elements: Vec<Element>,
    drainage: Drainage,
    assumed: Vec<String>,
}
fn diagnostic(code: &'static str, hint: impl Into<String>) -> Diagnostic {
    let mut d = match Diagnostic::from_code(code) {
        Some(d) => d,
        None => Diagnostic::new(
            "ENG-E-0012",
            Severity::Error,
            "Unregistered diagnostic code.",
        ),
    };
    d.hint = Some(hint.into());
    d
}
fn algebra(e: EngineError) -> Diagnostic {
    if e.code == "API-I-CANCELLED" {
        return Diagnostic::new(e.code, Severity::Info, e.message);
    }
    let mut d = diagnostic(e.code, e.message);
    d.entities = e.entities;
    d
}
fn invalid(hint: &str) -> Diagnostic {
    diagnostic("ENG-E-0010", hint)
}
fn finite(v: Real, field: &str) -> GeoResult<Real> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(diagnostic(
            "ENG-E-0008",
            format!("{field}: non-finite data or result."),
        ))
    }
}
fn positive(v: Real, field: &str) -> GeoResult<Real> {
    finite(v, field)?;
    if v > 0.0 {
        Ok(v)
    } else {
        Err(invalid(&format!(
            "{field}: must be positive and representable."
        )))
    }
}
fn parameter(p: Option<Parameter>, field: &str, assumed: &mut Vec<String>) -> GeoResult<Real> {
    let p = p.ok_or_else(|| {
        diagnostic(
            "ENG-E-0003",
            format!(
                "Missing {field}; provide a value and Provenance, with no implicit substitution."
            ),
        )
    })?;
    finite(p.value, field)?;
    if p.provenance == Provenance::Assumed {
        assumed.push(field.into());
    }
    Ok(p.value)
}
impl ConsolidationProfile {
    /// Validates all layers, k/mv/cv/γw, provenance, mesh, and arithmetic.
    /// Does not require geostatic stress profile data that do not enter this problem.
    pub fn new(
        layers: Vec<ConsolidationLayer>,
        water_unit_weight_kn_m3: Option<Parameter>,
        drainage: Drainage,
    ) -> GeoResult<Self> {
        let mut assumed = Vec::new();
        let water = positive(
            parameter(
                water_unit_weight_kn_m3,
                "water_unit_weight_kn_m3",
                &mut assumed,
            )?,
            "γw",
        )?;
        let count = layers
            .iter()
            .try_fold(0usize, |n, l| n.checked_add(l.elements))
            .ok_or_else(|| invalid("Element count overflow."))?;
        if layers.is_empty() || count > 2048 || count == 0 {
            return Err(invalid(
                "Layers and between 1 and 2048 elements in total are required.",
            ));
        }
        if count == 1 && drainage == Drainage::Double {
            return Err(invalid(
                "Double drainage requires at least two elements (one free node).",
            ));
        }
        let mut depths = Vec::with_capacity(count + 1);
        let mut elements = Vec::with_capacity(count);
        depths.push(0.0);
        let mut top = 0.0;
        for (i, layer) in layers.into_iter().enumerate() {
            let prefix = format!("layers[{i}]");
            let thickness = positive(
                parameter(
                    Some(layer.thickness_m),
                    &format!("{prefix}.thickness_m"),
                    &mut assumed,
                )?,
                "espesor",
            )?;
            let cv = positive(
                parameter(layer.cv_m2_s, &format!("{prefix}.cv_m2_s"), &mut assumed)?,
                "cv",
            )?;
            let mv = positive(
                parameter(
                    layer.mv_per_kpa,
                    &format!("{prefix}.mv_per_kpa"),
                    &mut assumed,
                )?,
                "mv",
            )?;
            let k = positive(
                parameter(layer.k_m_s, &format!("{prefix}.k_m_s"), &mut assumed)?,
                "k",
            )?;
            if layer.elements == 0 {
                return Err(invalid("Each layer requires at least one element."));
            }
            let conductivity = positive(k / water, "k/γw")?;
            let derived_cv = positive(conductivity / mv, "k/(γw mv)")?;
            if (finite(cv / derived_cv, "cv / derived cv")? - 1.0).abs() > 1e-10 {
                return Err(invalid(&format!(
                    "{prefix}: cv is incompatible with k/(γw mv); maximum relative discrepancy is 1e-10."
                )));
            }
            let bottom = finite(top + thickness, "accumulated depth")?;
            for e in 1..=layer.elements {
                let z = if e == layer.elements {
                    bottom
                } else {
                    finite(
                        top + thickness * (e as Real / layer.elements as Real),
                        "z nodal",
                    )?
                };
                let previous = depths[depths.len() - 1];
                let h = positive(z - previous, "element length")?;
                positive(mv * h / 6.0, "element mass")?;
                positive(conductivity / h, "element diffusion")?;
                elements.push(Element {
                    h,
                    mv,
                    conductivity,
                });
                depths.push(z);
            }
            top = bottom;
        }
        Ok(Self {
            depths,
            elements,
            drainage,
            assumed,
        })
    }

    fn free_range(&self) -> (usize, usize) {
        let first = usize::from(self.drainage != Drainage::Bottom);
        let end = self.depths.len() - usize::from(self.drainage != Drainage::Top);
        (first, end)
    }

    fn matrices(
        &self,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> GeoResult<(SparseMatrix, SparseMatrix, Vec<Real>)> {
        let (first, end) = self.free_range();
        let n = end - first;
        let mut mass = Vec::with_capacity(4 * self.elements.len());
        let mut diffusion = Vec::with_capacity(4 * self.elements.len());
        let mut source = vec![0.0; n];
        for (e, el) in self.elements.iter().enumerate() {
            check_cancelled(cancelled).map_err(algebra)?;
            let c = el.mv * el.h / 6.0;
            let d = el.conductivity / el.h;
            for a in 0..2 {
                let row = e + a;
                if row < first || row >= end {
                    continue;
                }
                source[row - first] = finite(
                    source[row - first] + el.mv * el.h / 2.0,
                    "accumulated element load",
                )?;
                for b in 0..2 {
                    let col = e + b;
                    if col < first || col >= end {
                        continue;
                    }
                    mass.push(Triplet {
                        row: row - first,
                        col: col - first,
                        value: c * if a == b { 2.0 } else { 1.0 },
                    });
                    diffusion.push(Triplet {
                        row: row - first,
                        col: col - first,
                        value: d * if a == b { 1.0 } else { -1.0 },
                    });
                }
            }
        }
        Ok((
            SparseMatrix::from_triplets(n, n, &mass).map_err(algebra)?,
            SparseMatrix::from_triplets(n, n, &diffusion).map_err(algebra)?,
            source,
        ))
    }

    /// Integrates with the engine-linalg reference backend.
    pub fn solve(
        &self,
        history: &[LoadPoint],
        output_times_s: &[Real],
        options: ConsolidationOptions,
    ) -> GeoResult<ConsolidationResult> {
        self.solve_with_cancel(history, output_times_s, options, &mut || false)
    }

    /// Integrates while checking the closure before each interval, factorization,
    /// block of up to 16 columns, and substep, without changing the
    /// time partition or arithmetic of `solve`.
    /// If it returns true, terminates with API-I-CANCELLED and no partial results.
    pub fn solve_with_cancel(
        &self,
        history: &[LoadPoint],
        output_times_s: &[Real],
        options: ConsolidationOptions,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> GeoResult<ConsolidationResult> {
        self.solve_with_backend_and_cancel::<_, true>(
            &mut ReferenceBackend,
            history,
            output_times_s,
            options,
            cancelled,
        )
    }

    /// Integrates with an existing backend; implements no elimination or solver of its own.
    /// An error does not modify the caller's profile, history, or previous outputs.
    pub fn solve_with_backend<B: LinAlgBackend>(
        &self,
        backend: &mut B,
        history: &[LoadPoint],
        output_times_s: &[Real],
        options: ConsolidationOptions,
    ) -> GeoResult<ConsolidationResult> {
        self.solve_with_backend_and_cancel::<_, false>(
            backend,
            history,
            output_times_s,
            options,
            &mut || false,
        )
    }

    fn solve_with_backend_and_cancel<B: LinAlgBackend, const REUSE_FACTOR: bool>(
        &self,
        backend: &mut B,
        history: &[LoadPoint],
        output_times_s: &[Real],
        options: ConsolidationOptions,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> GeoResult<ConsolidationResult> {
        finite(options.theta, "theta")?;
        positive(options.max_step_s, "max_step_s")?;
        if !(0.0..=1.0).contains(&options.theta) {
            return Err(invalid("theta must be in [0,1]."));
        }
        if history.len() < 2
            || history.len() > 100000
            || output_times_s.is_empty()
            || output_times_s.len() > 100000
        {
            return Err(invalid(
                "2..100000 load points and 1..100000 output times are required.",
            ));
        }
        if output_times_s
            .len()
            .checked_mul(self.depths.len())
            .is_none_or(|n| n > 8000000)
        {
            return Err(invalid("Output exceeds the limit of 8000000 nodal values."));
        }
        let mut assumed = self.assumed.clone();
        let mut loads = Vec::with_capacity(history.len());
        for (i, point) in history.iter().enumerate() {
            check_cancelled(cancelled).map_err(algebra)?;
            finite(point.time_s, "load time")?;
            if (i == 0 && point.time_s != 0.0) || (i > 0 && point.time_s <= history[i - 1].time_s) {
                return Err(invalid(
                    "History must start at 0 and have strictly increasing times.",
                ));
            }
            loads.push(parameter(
                point.delta_stress_kpa,
                &format!("history[{i}].delta_stress_kpa"),
                &mut assumed,
            )?);
        }
        for (i, &t) in output_times_s.iter().enumerate() {
            check_cancelled(cancelled).map_err(algebra)?;
            finite(t, "output time")?;
            if t < 0.0
                || t > history[history.len() - 1].time_s
                || (i > 0 && t <= output_times_s[i - 1])
            {
                return Err(invalid(
                    "Output times must be strictly increasing within the history, without extrapolation.",
                ));
            }
        }
        let (m, k, source) = self.matrices(cancelled)?;
        let (first, end) = self.free_range();
        let n = end - first;
        let mut u = vec![loads[0]; n];
        let mut states = Vec::with_capacity(output_times_s.len());
        let mut time = 0.0;
        let mut stress = loads[0];
        let mut segment = 0;
        let mut cached_factor: Option<(u64, Box<dyn SymFactorization>)> = None;
        for &target in output_times_s {
            check_cancelled(cancelled).map_err(algebra)?;
            while time < target {
                while segment + 1 < history.len() - 1 && time >= history[segment + 1].time_s {
                    segment += 1;
                }
                let stop = target.min(history[segment + 1].time_s);
                let span = positive(stop - time, "time interval")?;
                let steps_real =
                    finite((span / options.max_step_s).ceil(), "substep count")?.max(1.0);
                if steps_real > 1000000.0 {
                    return Err(invalid("More than 1000000 substeps in an interval."));
                }
                let steps = steps_real as usize;
                let dt = positive(span / steps as Real, "dt")?;
                check_cancelled(cancelled).map_err(algebra)?;
                check_stability(&m, &k, options.theta, dt)?;
                // ReferenceBackend only: M, K, and theta are immutable; the same
                // bitwise dt produces the same matrix and deterministic LDLᵀ.
                // Caller backends retain all their calls.
                if !REUSE_FACTOR
                    || cached_factor
                        .as_ref()
                        .is_none_or(|(bits, _)| *bits != dt.to_bits())
                {
                    let a = combine(&m, &k, options.theta * dt)?;
                    let factor = backend
                        .factorize_with_check(&a, &mut || check_cancelled(cancelled))
                        .map_err(algebra)?;
                    cached_factor = Some((dt.to_bits(), factor));
                }
                let factor = &cached_factor
                    .as_ref()
                    .ok_or_else(|| invalid("Missing time factorization."))?
                    .1;
                let load_span = positive(
                    history[segment + 1].time_s - history[segment].time_s,
                    "load interval",
                )?;
                let mut previous_time = time;
                for step in 1..=steps {
                    if cancelled() {
                        return Err(Diagnostic::new(
                            "API-I-CANCELLED",
                            Severity::Info,
                            "Consolidation cancelled at a substep.",
                        ));
                    }
                    let next_time = if step == steps {
                        stop
                    } else {
                        time + dt * step as Real
                    };
                    if next_time <= previous_time {
                        return Err(invalid("Time step is lost in f64; check the scales."));
                    }
                    let r = (next_time - history[segment].time_s) / load_span;
                    let next_stress = finite(
                        (1.0 - r) * loads[segment] + r * loads[segment + 1],
                        "interpolated Δσ",
                    )?;
                    let increment = finite(next_stress - stress, "Δσ increment")?;
                    let mut rhs = vec![0.0; n];
                    for row in 0..n {
                        let mut value = source[row] * increment;
                        for p in m.row_offsets()[row]..m.row_offsets()[row + 1] {
                            value += (m.values()[p] - (1.0 - options.theta) * dt * k.values()[p])
                                * u[m.column_indices()[p]];
                        }
                        rhs[row] = finite(value, "Time RHS")?;
                    }
                    let solution = factor
                        .solve_with_check(&DMat::new(n, 1, rhs).map_err(algebra)?, &mut || {
                            check_cancelled(cancelled)
                        })
                        .map_err(algebra)?;
                    u.copy_from_slice(solution.values());
                    stress = next_stress;
                    previous_time = next_time;
                }
                time = stop;
            }
            let mut full = vec![0.0; self.depths.len()];
            full[first..end].copy_from_slice(&u);
            let mut settlement = 0.0;
            let mut compliance = 0.0;
            for (e, el) in self.elements.iter().enumerate() {
                check_cancelled(cancelled).map_err(algebra)?;
                let c = finite(el.mv * el.h, "mv h")?;
                compliance = finite(compliance + c, "∫mv dz")?;
                settlement = finite(
                    settlement + c * (stress - 0.5 * full[e] - 0.5 * full[e + 1]),
                    "s(t)",
                )?;
            }
            let mean_degree = if stress == 0.0 {
                None
            } else {
                let ultimate = finite(stress * compliance, "s∞")?;
                if ultimate == 0.0 {
                    return Err(invalid("s∞ underflow; check the scales."));
                }
                Some(finite(settlement / ultimate, "U(t)")?)
            };
            states.push(ConsolidationState {
                time_s: target,
                delta_stress_kpa: stress,
                excess_pore_pressure_kpa: full,
                settlement_m: settlement,
                mean_degree,
            });
        }
        Ok(ConsolidationResult {
            depths_m: self.depths.clone(),
            states,
            assumed_parameters: assumed,
        })
    }
}

fn check_cancelled(cancelled: &mut dyn FnMut() -> bool) -> Result<(), EngineError> {
    if cancelled() {
        Err(EngineError::new(
            "API-I-CANCELLED",
            "Consolidation cancelled before or during factorization.",
        ))
    } else {
        Ok(())
    }
}

fn combine(m: &SparseMatrix, k: &SparseMatrix, scale: Real) -> GeoResult<SymMatrix> {
    let entries: Vec<_> = (0..m.rows())
        .flat_map(|row| {
            (m.row_offsets()[row]..m.row_offsets()[row + 1]).map(move |p| Triplet {
                row,
                col: m.column_indices()[p],
                value: m.values()[p] + scale * k.values()[p],
            })
        })
        .collect();
    SymMatrix::new(
        SparseMatrix::from_triplets(m.rows(), m.cols(), &entries)
            .map_err(algebra)?
            .to_dense()
            .map_err(algebra)?,
    )
    .map_err(algebra)
}

// λmax(M⁻¹K) ≤ max row absolute sum(K) / min row(diag(M)−sum offdiag(M)).
// M is symmetric strictly diagonally dominant for these linear elements.
fn check_stability(m: &SparseMatrix, k: &SparseMatrix, theta: Real, dt: Real) -> GeoResult<()> {
    if theta >= 0.5 {
        return Ok(());
    }
    let mut lower = Real::INFINITY;
    let mut upper: Real = 0.0;
    for row in 0..m.rows() {
        let mut margin = 0.0;
        let mut ksum = 0.0;
        for p in m.row_offsets()[row]..m.row_offsets()[row + 1] {
            margin += if m.column_indices()[p] == row {
                m.values()[p]
            } else {
                -m.values()[p].abs()
            };
            ksum += k.values()[p].abs();
        }
        lower = lower.min(margin);
        upper = upper.max(ksum);
    }
    positive(lower, "lower bound of M")?;
    positive(upper, "upper bound of K")?;
    let limit = positive(
        (2.0 / (1.0 - 2.0 * theta)) * (lower / upper),
        "stable dt limit",
    )?;
    if dt > limit {
        return Err(invalid(&format!("θ={theta}: dt={dt} s exceeds the sufficient stability bound {limit} s; reduce max_step_s.")));
    }
    Ok(())
}
