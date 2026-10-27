//! Elastic P-Delta with axial forces from the linear gravity case, without code factors.
//!
//! Kg is held constant at u_g: producers recover each member's axial force using
//! its original geometry. The lateral case is solved by Picard
//! iteration `K_red q_{i+1} = T_fᵀ(F − K u_0 − Kg u_i)`, equivalent to
//! `T_fᵀ(K + Kg)(T_f q + u_0) = T_fᵀF`. Axes, materials, and axial forces are not updated by the
//! lateral load; this is an initial-prestress approximation, not large-deformation analysis.
//! The LDLᵀ inertia of `T_fᵀ(K + Kg)T_f` is checked before iteration; a mechanism
//! still yields ENG-E-0006. A stable matrix may fail to converge with Picard iteration
//! (e.g. very large tension): this is reported without changing strategy or retrying.
//! The cases are independent: lateral u excludes u_g; prescribed displacements are not
//! scaled. The system state is neither accepted nor modified. See the derivation
//! and analytical tolerances in `provenance/ENG-ANL-003.md`.

use crate::{finite, StaticCaseResult, StaticLinear};
use engine_assembly::{GeometricProducer, LinearSystem, Reaction};
use engine_contracts::{CaseRef, Element, ElementCtx, SystemView};
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_linalg::{DMat, LinAlgBackend, SparseMatrix, SymMatrix, Triplet};
use std::{collections::BTreeSet, sync::Arc};

/// Explicit Picard iteration settings, without code-based values or retries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PDeltaOptions {
    /// Positive relative tolerance less than one, for change and residual.
    /// The residual also allows the roundoff floor documented in PDeltaIteration.
    pub tolerance: Real,
    /// Maximum number of lateral steps; positive. Gravity does not count as a step.
    pub max_iterations: usize,
    /// Positive scale in m for normalizing translations (does not modify equations).
    pub displacement_scale: Real,
    /// Positive scale in rad for normalizing rotations.
    pub rotation_scale: Real,
}

/// Physical/numerical termination of the method, distinct from input errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PDeltaOutcome {
    /// Relative change and per-DOF residual with roundoff floor meet the criterion.
    Completed,
    /// Reached exactly max_iterations; no state was accepted.
    NotConverged,
    /// Loss of stability from the LDLᵀ inertia of the total reduced stiffness.
    Unstable {
        /// Number of negative eigenvalues (includes signs from 2×2 blocks).
        negative_eigenvalues: usize,
    },
}

/// Report for each lateral step; there are no hidden iterations.
#[derive(Clone, Debug, PartialEq)]
pub struct PDeltaIteration {
    /// Step number, starting at one.
    pub iteration: usize,
    /// Free lateral displacement of this trial, m/rad.
    pub free_displacements: Vec<Real>,
    /// ‖Δu/s‖∞ / max(1, ‖u_nuevo/s‖∞, ‖u_previo/s‖∞).
    /// s is displacement_scale or rotation_scale according to the DOF.
    pub relative_displacement_change: Real,
    /// ‖T_fᵀ((K+Kg)u−F)‖∞; mixes forces and moments (N/N·m in SI).
    pub free_residual_norm: Real,
    /// Residual / max(1, ‖T_fᵀ((K+Kg)u)‖∞, ‖T_fᵀF‖∞).
    /// Mixed SI indicator; not a unit-invariant physical norm.
    /// Retained as a diagnostic; does not determine convergence.
    pub relative_free_residual: Real,
    /// max_i |T_fᵀ((K+Kg)u−F)_i| / (tolerance·ref_g + piso_i).
    /// ref_g = max_j(|(T_fᵀF)_j|, (|T_f|ᵀ|K+Kg||u|)_j) within the same group:
    /// translations (forces) or rotations (moments), without an absolute unit scale.
    /// piso_i = 64·m_i·eps·(|T_f|ᵀ|K+Kg||u|)_i, where m_i is the number of
    /// CSR entries in the global rows contributing to the projected row
    /// (includes prescribed DOFs and stored zeros), at least 1. Without diaphragms
    /// it matches the global row and preserves the previous criterion.
    /// eps = Real::EPSILON; the factor 64 covers prior accumulation and solution
    /// as well as the final product (derivation in ENG-ANL-003.md).
    /// Must be <= 1, together with the displacement-change criterion.
    /// Near buckling, the accepted residual corresponds to a displacement error
    /// amplified by ~1/(1−P/Pcr): the effective tolerance on u is thus relaxed.
    /// In practice the floor is active only with prescribed displacements;
    /// the force/moment references resolve the W14x48 stagnation.
    /// 0/0 is reported as 0; an underflowed denominator with a nonzero residual,
    /// as Real::MAX (does not converge).
    pub residual_convergence_ratio: Real,
}

/// Lateral result and gravity used as prestress, without implicit state acceptance.
#[derive(Clone, Debug, PartialEq)]
pub struct PDeltaResult {
    /// Termination of the lateral analysis.
    pub outcome: PDeltaOutcome,
    /// Settings actually used, including scales and tolerance.
    pub options: PDeltaOptions,
    /// Prior linear gravity case; axial forces evaluated at its displacements.
    pub gravity: StaticCaseResult,
    /// Independent lateral case.
    pub lateral_case: CaseRef,
    /// Lateral load factor, without scaling prescribed displacements.
    pub lateral_factor: Real,
    /// Inertia (positive, negative, zero) of `T_fᵀ(K+Kg)T_f`, from LDLᵀ.
    pub inertia: (usize, usize, usize),
    /// 2×2 blocks used for inertia; None if the backend does not report this.
    pub inertia_uses_2x2: Option<bool>,
    /// Ordered reports; empty for Unstable (lateral load is not solved).
    pub iterations: Vec<PDeltaIteration>,
    /// Last global lateral trial, or None for Unstable.
    pub global_displacements: Option<Vec<Real>>,
    /// Reactions `(K+Kg)u−F` at prescribed DOFs from the last trial.
    /// Empty for Unstable. These are not reactions from the gravity+lateral combination.
    pub reactions: Vec<Reaction>,
}

/// P-Delta analysis with initial prestress and explicit Picard iteration.
pub struct PDelta;

impl PDelta {
    /// Public method identifier.
    pub const KIND: &'static str = "pdelta";

    /// First StaticLinear(gravedad), then Kg(u_g) and the iterated lateral case.
    /// Producers must be registered to evaluate Kg outside their static
    /// reference (or the correct reference must be explicitly declared).
    /// ENG-E-0008: nonfinite values; ENG-E-0010: invalid settings; ENG-E-0003: missing
    /// case; ENG-E-0006: mechanism with node/DOF; ENG-E-0013: backend without
    /// inertia; ENG-E-0012: inconsistent diagnostics. No state changes.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        gravity: &(CaseRef, Real),
        lateral: &(CaseRef, Real),
        options: PDeltaOptions,
    ) -> EResult<PDeltaResult> {
        Self::run_with(system, gravity, lateral, options, |_| Ok(()))
    }

    /// Cooperative variant: callback after gravity (0) and each lateral iteration.
    /// A callback error stops the analysis without accepting state.
    pub fn run_with<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        gravity: &(CaseRef, Real),
        lateral: &(CaseRef, Real),
        options: PDeltaOptions,
        checkpoint: impl FnMut(usize) -> EResult<()>,
    ) -> EResult<PDeltaResult> {
        Self::run_with_check(system, gravity, lateral, options, checkpoint, || Ok(()))
    }

    /// Checks independent of progress; polls during LDLᵀ and at each iteration.
    pub fn run_with_check<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        gravity: &(CaseRef, Real),
        lateral: &(CaseRef, Real),
        options: PDeltaOptions,
        mut checkpoint: impl FnMut(usize) -> EResult<()>,
        mut check: impl FnMut() -> EResult<()>,
    ) -> EResult<PDeltaResult> {
        check()?;
        finite(&[
            gravity.1,
            lateral.1,
            options.tolerance,
            options.displacement_scale,
            options.rotation_scale,
        ])?;
        if options.tolerance <= 0.0
            || options.tolerance >= 1.0
            || options.max_iterations == 0
            || options.displacement_scale <= 0.0
            || options.rotation_scale <= 0.0
        {
            return Err(EngineError::new("ENG-E-0010", "P-Delta: invalid settings."));
        }
        // Also validate the lateral case before any factorization.
        let force = system.global_force(&lateral.0, lateral.1)?;
        let elastic_rhs = system.reduced_rhs(&lateral.0, lateral.1)?;
        let gravity = StaticLinear::run_about_with_check(
            system,
            std::slice::from_ref(gravity),
            [0.0; 3],
            &mut check,
        )?
        .cases
        .into_iter()
        .next()
        .ok_or_else(|| EngineError::new("ENG-E-0012", "P-Delta: missing gravity solution."))?;
        checkpoint(0)?;
        let kg = system.geometric_at(&gravity.free_displacements)?;
        let total = sum(system.stiffness(), &kg)?;
        let total_free = reduce(system, &total)?;
        let total_factor = system.factorize_reduced_with_check(&total_free, &mut check)?;
        let inertia = total_factor.info().inertia.ok_or_else(|| {
            EngineError::new(
                "ENG-E-0013",
                "P-Delta: backend does not provide LDLᵀ inertia.",
            )
        })?;
        if inertia
            .0
            .checked_add(inertia.1)
            .and_then(|s| s.checked_add(inertia.2))
            != Some(total_free.order())
        {
            return Err(EngineError::new(
                "ENG-E-0012",
                "P-Delta: inconsistent inertia.",
            ));
        }
        if inertia.2 != 0 {
            return Err(EngineError::new(
                "ENG-E-0006",
                "P-Delta: rigidez total singular.",
            ));
        }
        let mut result = PDeltaResult {
            outcome: if inertia.1 > 0 {
                PDeltaOutcome::Unstable {
                    negative_eigenvalues: inertia.1,
                }
            } else {
                PDeltaOutcome::NotConverged
            },
            options,
            gravity,
            lateral_case: lateral.0.clone(),
            lateral_factor: lateral.1,
            inertia,
            inertia_uses_2x2: total_factor.uses_2x2_pivots(),
            iterations: Vec::new(),
            global_displacements: None,
            reactions: Vec::new(),
        };
        if inertia.1 > 0 {
            return Ok(result);
        }
        // This factor only diagnosed stability; an eigensolver is never used.
        drop(total_factor);
        let elastic = system.reduced_stiffness().clone();
        let factor = system.factorize_reduced_with_check(&elastic, &mut check)?;
        let n = elastic.order();
        let mut previous = vec![0.0; n];
        let scales = (0..n)
            .map(|i| {
                let (_, dof) = system.try_dof_label(DofIndex(i as u32))?;
                Ok(if dof.index() < 3 {
                    options.displacement_scale
                } else {
                    options.rotation_scale
                })
            })
            .collect::<EResult<Vec<_>>>()?;
        for iteration in 1..=options.max_iterations {
            check()?;
            let old_global = system.constraints().expand(&previous)?;
            let geometric_force = multiply(&kg, &old_global)?;
            let projected_geometric = system.constraints().project_force(&geometric_force)?;
            let rhs = elastic_rhs
                .iter()
                .zip(projected_geometric)
                .map(|(f, geometric)| f - geometric)
                .collect();
            let rhs = DMat::new(n, 1, rhs)?;
            let solution = factor.solve_with_check(&rhs, &mut check)?;
            if solution.rows() != n || solution.cols() != 1 {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "P-Delta: invalid solution size.",
                ));
            }
            finite(solution.values())?;
            let current = solution.values().to_vec();
            let global = system.constraints().expand(&current)?;
            let internal = multiply(&total, &global)?;
            let (norm, relative, residual_ratio) = residual(
                system,
                &total,
                &global,
                &internal,
                &force,
                options.tolerance,
            )?;
            let change = displacement_change(&current, &previous, &scales)?;
            result.iterations.push(PDeltaIteration {
                iteration,
                free_displacements: current.clone(),
                relative_displacement_change: change,
                free_residual_norm: norm,
                relative_free_residual: relative,
                residual_convergence_ratio: residual_ratio,
            });
            result.reactions = reactions(system, &internal, &force)?;
            result.global_displacements = Some(global);
            checkpoint(iteration)?;
            if change <= options.tolerance && residual_ratio <= 1.0 {
                result.outcome = PDeltaOutcome::Completed;
                break;
            }
            previous = current;
        }
        Ok(result)
    }
}

/// Owned adapter for frame2d, bars, or other elements that provide Kg.
/// Captures coordinates, parameters, and accepted history without accepting new trials.
/// Each evaluation delegates to Element::geometric_stiffness, using the element's own
/// axial forces (positive in tension). Does not estimate N from the total model load.
/// ENG-E-0010 for sizes; ENG-E-0002 for repeated DOFs; ENG-E-0008 for nonfinite
/// values; ENG-E-0013 if the element does not support Kg. The system validates the
/// global range upon registration through set_geometric_producers.
pub fn geometric_producer(
    element: Arc<dyn Element>,
    ctx: &ElementCtx<'_>,
    location: Vec<DofIndex>,
    committed: Vec<Real>,
) -> EResult<GeometricProducer> {
    if element.n_nodes().checked_mul(element.dofs_per_node().len()) != Some(location.len())
        || committed.len() != element.state_len()
    {
        return Err(EngineError::new(
            "ENG-E-0010",
            "P-Delta: invalid element sizes.",
        ));
    }
    if location.iter().collect::<BTreeSet<_>>().len() != location.len() {
        return Err(EngineError::new(
            "ENG-E-0002",
            "P-Delta: duplicate element DOF.",
        ));
    }
    finite(&committed)?;
    let evaluate = |ctx: &ElementCtx<'_>, u: &[Real]| {
        element
            .geometric_stiffness(ctx, u, &committed)?
            .ok_or_else(|| EngineError::new("ENG-E-0013", "P-Delta: element does not provide Kg."))
    };
    evaluate(ctx, &vec![0.0; location.len()])?;
    let dim = ctx.dim;
    let coords = ctx.coords.to_vec();
    let params = ctx.params.clone();
    Ok(GeometricProducer {
        location,
        eval: Box::new(move |u| {
            let ctx = ElementCtx {
                dim,
                coords: &coords,
                params: &params,
            };
            element
                .geometric_stiffness(&ctx, u, &committed)?
                .ok_or_else(|| {
                    EngineError::new("ENG-E-0013", "P-Delta: element does not provide Kg.")
                })
        }),
    })
}

fn sum(a: &SparseMatrix, b: &SparseMatrix) -> EResult<SparseMatrix> {
    let entries = [a, b]
        .into_iter()
        .flat_map(|m| {
            (0..m.rows()).flat_map(move |row| {
                (m.row_offsets()[row]..m.row_offsets()[row + 1]).map(move |p| Triplet {
                    row,
                    col: m.column_indices()[p],
                    value: m.values()[p],
                })
            })
        })
        .collect::<Vec<_>>();
    SparseMatrix::from_triplets(a.rows(), a.cols(), &entries)
}

fn reduce<B: LinAlgBackend>(system: &LinearSystem<B>, matrix: &SparseMatrix) -> EResult<SymMatrix> {
    let mut entries = Vec::new();
    for global in 0..matrix.rows() {
        for p in matrix.row_offsets()[global]..matrix.row_offsets()[global + 1] {
            for &(row, a) in system
                .constraints()
                .free_transformation_row(DofIndex(global as u32))?
            {
                for &(col, b) in system
                    .constraints()
                    .free_transformation_row(DofIndex(matrix.column_indices()[p] as u32))?
                {
                    // T_fᵀ (K+Kg) T_f; accumulating and mirroring only one half
                    // preserves exact symmetry, as in engine-assembly.
                    if row >= col {
                        let value = a * matrix.values()[p] * b;
                        finite(&[value])?;
                        entries.push(Triplet {
                            row: row.0 as usize,
                            col: col.0 as usize,
                            value,
                        });
                    }
                }
            }
        }
    }
    let n = system.n_free_dofs();
    let dense = SparseMatrix::from_triplets(n, n, &entries)?.to_dense()?;
    let mut values = dense.values().to_vec();
    for row in 0..n {
        for col in 0..row {
            values[col * n + row] = values[row * n + col];
        }
    }
    SymMatrix::new(DMat::new(n, n, values)?)
}

fn multiply(matrix: &SparseMatrix, u: &[Real]) -> EResult<Vec<Real>> {
    let mut result = vec![0.0; matrix.rows()];
    for (row, value) in result.iter_mut().enumerate() {
        for p in matrix.row_offsets()[row]..matrix.row_offsets()[row + 1] {
            let term = matrix.values()[p] * u[matrix.column_indices()[p]];
            finite(&[term])?;
            *value += term;
            finite(&[*value])?;
        }
    }
    Ok(result)
}

fn displacement_change(current: &[Real], previous: &[Real], scales: &[Real]) -> EResult<Real> {
    let mut change: Real = 0.0;
    let mut denominator: Real = 1.0;
    for ((u, old), scale) in current.iter().zip(previous).zip(scales) {
        let a = u / scale;
        let b = old / scale;
        let delta = a - b;
        finite(&[a, b, delta])?;
        change = change.max(delta.abs());
        denominator = denominator.max(a.abs()).max(b.abs());
    }
    Ok(change / denominator)
}

fn residual<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    matrix: &SparseMatrix,
    u: &[Real],
    internal: &[Real],
    force: &[Real],
    tolerance: Real,
) -> EResult<(Real, Real, Real)> {
    let mut norm: Real = 0.0;
    let mut scale: Real = 1.0;
    let mut references: [Real; 2] = [0.0; 2];
    let mut rows = Vec::with_capacity(system.n_free_dofs());
    let projected_internal = system.constraints().project_force(internal)?;
    let projected_force = system.constraints().project_force(force)?;
    let mut absolute_products = vec![0.0; system.n_free_dofs()];
    let mut term_counts = vec![0usize; system.n_free_dofs()];
    for global in 0..matrix.rows() {
        let transformation = system
            .constraints()
            .free_transformation_row(DofIndex(global as u32))?;
        if transformation.is_empty() {
            continue;
        }
        let start = matrix.row_offsets()[global];
        let end = matrix.row_offsets()[global + 1];
        let mut absolute_product = 0.0;
        for p in start..end {
            // Includes prescribed DOFs; |T_f|ᵀ |K+Kg| |u| bounds the projection.
            absolute_product += (matrix.values()[p] * u[matrix.column_indices()[p]]).abs();
            finite(&[absolute_product])?;
        }
        for &(free, coefficient) in transformation {
            let i = free.0 as usize;
            absolute_products[i] += coefficient.abs() * absolute_product;
            term_counts[i] += (end - start).max(1);
            finite(&[absolute_products[i]])?;
        }
    }
    for free in 0..system.n_free_dofs() {
        let value = projected_internal[free] - projected_force[free];
        finite(&[value])?;
        norm = norm.max(value.abs());
        scale = scale
            .max(projected_internal[free].abs())
            .max(projected_force[free].abs());
        let (_, dof) = system.try_dof_label(DofIndex(free as u32))?;
        let group = usize::from(dof.index() >= 3);
        let absolute_product = absolute_products[free];
        references[group] = references[group]
            .max(projected_force[free].abs())
            .max(absolute_product);
        let floor = 64.0 * term_counts[free].max(1) as Real * Real::EPSILON * absolute_product;
        finite(&[floor])?;
        rows.push((value.abs(), group, floor));
    }
    let mut convergence_ratio: Real = 0.0;
    for (value, group, floor) in rows {
        let bound = tolerance * references[group] + floor;
        finite(&[bound])?;
        let ratio = if value == 0.0 {
            0.0
        } else if bound == 0.0 {
            Real::MAX
        } else {
            value / bound
        };
        finite(&[ratio])?;
        convergence_ratio = convergence_ratio.max(ratio);
    }
    Ok((norm, norm / scale, convergence_ratio))
}

fn reactions<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    internal: &[Real],
    force: &[Real],
) -> EResult<Vec<Reaction>> {
    let mut result = Vec::new();
    for (i, prescribed) in system.constraints().prescribed_values().iter().enumerate() {
        if prescribed.is_some() {
            let (node, dof) = system.dof_map().label(DofIndex(i as u32))?;
            let value = internal[i] - force[i];
            finite(&[value])?;
            result.push(Reaction { node, dof, value });
        }
    }
    Ok(result)
}

#[cfg(test)]
#[path = "../tests/unit/pdelta_projection.rs"]
mod projection_tests;
