//! `buckling.linear`: finite eigenvalues of `K + λ Kg = 0`.
//!
//! Uses `K_ff` and the reference `Kg_ff` already reduced by [`LinearSystem`].
//! The factor λ multiplies that reference geometric stiffness. With the
//! element convention that axial force is positive in tension, a compressive
//! reference makes `Kg` negative on the bending terms, and the physical
//! critical multipliers are the positive eigenvalues of
//! `K φ = λ (−Kg) φ`. Shapes returned here satisfy `φᵀ(−Kg)φ = 1` on the
//! free DOFs. Global shapes are homogeneous perturbations `u = T_f φ`, with
//! zeros at prescribed DOFs, including settlements.
//!
//! Zero rows of `−Kg` are condensed. A negative diagonal of `−Kg`, a zero
//! reference `Kg`, or an indefinite metric is rejected. There is no load
//! shift and no second eigensolver. Mechanisms of `K` remain explicit zero
//! factors. The accepted and trial states are left unchanged.
//!
//! Derivation and the four Euler cases: `provenance/ENG-ANL-006.md`.

use engine_assembly::LinearSystem;
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_linalg::LinAlgBackend;

/// Stateless linear buckling method.
pub struct BucklingLinear;

/// Critical multipliers of the reference geometric stiffness, lowest first.
#[derive(Clone, Debug, PartialEq)]
pub struct BucklingResults {
    /// Modes in ascending λ.
    pub modes: Vec<BucklingMode>,
    /// Returned modes whose factor is numerically zero: mechanisms or rigid modes.
    pub rigid_mode_count: usize,
}

/// One mode of `K φ + λ Kg φ = 0`.
#[derive(Clone, Debug, PartialEq)]
pub struct BucklingMode {
    /// Load factor λ of the reference `Kg`. Dimensionless with respect to that reference.
    pub eigenvalue: Real,
    /// φ in free reduced order, with `φᵀ(−Kg_ff)φ = 1`.
    pub free_shape: Vec<Real>,
    /// Homogeneous global shape `T_f φ`, in m and rad. Prescribed DOFs are zero.
    pub global_shape: Vec<Real>,
}

impl BucklingLinear {
    /// Public method identifier.
    pub const KIND: &'static str = "buckling.linear";

    /// Returns exactly `count` lowest finite buckling factors.
    ///
    /// Does not modify accepted or trial state. `count = 0` returns an empty
    /// list and does not require a definite geometric metric.
    /// ENG-E-0010: zero Kg, a tension metric, or `count` above the free order.
    /// ENG-E-0006 and ENG-E-0014 propagate from a singular metric or from the
    /// condensed elastic block. ENG-E-0007: incomplete spectrum. ENG-E-0008:
    /// nonfinite values. ENG-E-0012: incompatible backend dimensions.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        count: usize,
    ) -> EResult<BucklingResults> {
        Self::run_with_check(system, count, || Ok(()))
    }

    /// Cooperative variant. The callback is polled around the eigensolver and each mode.
    pub fn run_with_check<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        count: usize,
        mut check: impl FnMut() -> EResult<()>,
    ) -> EResult<BucklingResults> {
        check()?;
        let spectrum = system.buckling_modes_with_check(count, &mut check)?;
        let n = system.constraints().free_dofs().len();
        if spectrum.eigenvalues.len() != count
            || spectrum.vectors.rows() != n
            || spectrum.vectors.cols() != count
            || spectrum.rigid_mode_count > count
        {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Buckling analysis: incompatible spectrum dimensions.",
            ));
        }
        super::finite(&spectrum.eigenvalues)?;
        super::finite(spectrum.vectors.values())?;
        if spectrum
            .eigenvalues
            .windows(2)
            .any(|pair| pair[0] > pair[1])
        {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Buckling analysis: unordered spectrum.",
            ));
        }
        let mut modes = Vec::with_capacity(count);
        for column in 0..count {
            check()?;
            let free_shape: Vec<Real> = (0..n)
                .map(|row| spectrum.vectors.values()[row * count + column])
                .collect();
            let global_shape = global_shape(system, &free_shape)?;
            modes.push(BucklingMode {
                eigenvalue: spectrum.eigenvalues[column],
                free_shape,
                global_shape,
            });
        }
        Ok(BucklingResults {
            modes,
            rigid_mode_count: spectrum.rigid_mode_count,
        })
    }
}

fn global_shape<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    reduced: &[Real],
) -> EResult<Vec<Real>> {
    let mut global = vec![0.0; system.dof_map().len()];
    for (row, value) in global.iter_mut().enumerate() {
        for &(index, coefficient) in system
            .constraints()
            .free_transformation_row(DofIndex(row as u32))?
        {
            let Some(&component) = reduced.get(index.0 as usize) else {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Buckling analysis: transformation index is outside the free shape.",
                ));
            };
            let term = coefficient * component;
            super::finite(&[term])?;
            *value += term;
            super::finite(&[*value])?;
        }
    }
    Ok(global)
}
