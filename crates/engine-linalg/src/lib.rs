#![forbid(unsafe_code)]
//! Internal numerical boundary with custom types and core errors.
//!
//! `ReferenceBackend` is a simple implementation for tests and small
//! problems: cubic work and quadratic memory, without performance promises.
//! The `faer` feature reserves a stub: integration PENDING verification of the
//! 0.24.4 package license notices; see this crate's README.md.
//! Quantities retain the SI units of their rows and columns; this crate
//! does not convert units or know the mapping between global indices and nodes.
//! [`SymCscMatrix`] guarda solo el triángulo superior en CSC y rechaza, antes
//! de reservar, un orden o una estimación de memoria por encima del límite
//! que pasa el llamador. No elige el corte entre la ruta densa y la dispersa.
//! V1 uses the sequential reference, without explicit SIMD or implicit FMA.
//! Reduction order and the explicit Sturm FMA exception are documented
//! in `docs/DETERMINISM.md` (ENG-CORE-010). Scalar functions go through
//! `engine_core::math`: libm 0.2.16 (MIT) except sqrt, correctly rounded
//! IEEE-754 in std. The native/wasm cross-check verifies results and scalar bits.
//!
//! ```
//! use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend};
//! let a = DMat::new(2, 2, vec![2.0, 0.0, 0.0, 4.0])?;
//! let b = DMat::new(2, 1, vec![6.0, 8.0])?;
//! let x = ReferenceBackend.solve_dense(&a, &b)?;
//! assert_eq!(x.values(), &[3.0, 2.0]);
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

use engine_core::{DofIndex, EResult, EngineError, EntityRef, Real};
mod budget;
mod guyan;
mod matrix;
mod modes;
mod reference;
mod sparse;
mod sym_csc;
pub use budget::{dense_bytes, eigen_working_bytes, factor_bytes, sparse_bytes, MemoryBudget};
pub use guyan::{guyan_condense, guyan_condense_with_check, GuyanReduction};
pub use matrix::{DMat, SparsityPattern, SymMatrix};
pub use modes::{lowest_modes, sturm_count, LowestModes};
pub use reference::ReferenceBackend;
pub use sparse::{SparseMatrix, Triplet};
pub use sym_csc::{sym_csc_bytes, SymCscLimits, SymCscMatrix};
#[cfg(feature = "faer")]
mod faer_backend;
#[cfg(feature = "faer")]
pub use faer_backend::FaerBackend;

/// Numerical information for a symmetric factorization without regularization.
#[derive(Clone, Debug, PartialEq)]
pub struct FactorInfo {
    /// Counts (positive, negative, zero), if the backend provides them.
    /// The Sturm check uses the number of negatives in this inertia.
    pub inertia: Option<(usize, usize, usize)>,
    /// Minimum absolute pivot value divided by the maximum, if known.
    /// For 2×2 blocks, their two eigenvalues are used, not just the diagonal.
    /// It is a dimensionless indicator, not a condition number estimate.
    pub min_pivot_ratio: Option<Real>,
    /// Global indices of detected singular pivots, counted from zero.
    /// A singular factorization returns ENG-E-0006 with these entities;
    /// the future assembler must map them to node and physical component.
    pub singular_dofs: Vec<DofIndex>,
}

/// Reusable factor; solving neither refactorizes nor modifies the factor.
pub trait SymFactorization {
    /// Numerical factorization metadata.
    fn info(&self) -> &FactorInfo;
    /// Whether the factorization used 2×2 blocks to evaluate its inertia.
    /// None indicates the backend does not publish this data.
    fn uses_2x2_pivots(&self) -> Option<bool> {
        None
    }
    /// Solves all columns of `b` and replaces them with `x` in `Ax=b`.
    ///
    /// `b.rows()` must match the factor order. Zero columns is valid.
    /// ENG-E-0010 indicates incompatible size, ENG-E-0008 nonfinite data or
    /// result. On error, all of `b` is preserved, without partial writes.
    fn solve_in_place(&self, b: &mut DMat) -> EResult<()>;
    /// Cooperative substitution; by default, checks before the backend kernel.
    fn solve_in_place_with_check(
        &self,
        b: &mut DMat,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<()> {
        check()?;
        self.solve_in_place(b)
    }
    /// New solution with cooperative control, without modifying the original RHS.
    fn solve_with_check(&self, b: &DMat, check: &mut dyn FnMut() -> EResult<()>) -> EResult<DMat> {
        let mut x = b.clone();
        self.solve_in_place_with_check(&mut x, check)?;
        Ok(x)
    }
    /// Returns a new solution, preserving the original load matrix.
    fn solve(&self, b: &DMat) -> EResult<DMat> {
        let mut x = b.clone();
        self.solve_in_place(&mut x)?;
        Ok(x)
    }
}

/// All ordered dense eigenvalues and their eigenvectors as columns.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneralizedEigen {
    /// Ascending values of λ in `Kφ=λMφ`; units of K divided by M.
    pub eigenvalues: Vec<Real>,
    /// Normalized columns: `φᵢᵀMφⱼ=δᵢⱼ` within numerical precision.
    /// The component of greatest magnitude is positive; multiplicities may
    /// produce different bases of the same subspace depending on the backend.
    pub vectors: DMat,
}

/// Contract independent of the linear algebra provider.
///
/// Each method preserves its inputs and validates finiteness/dimensions. Does not modify
/// stiffness or mass through hidden shifts or regularization. Matrix
/// indices are the DOF numbering supplied by the caller.
/// Factorization/solve uses ENG-E-0006 for singularity with a DOF entity;
/// eigenvalues use ENG-E-0007 on nonconvergence. Mass must be positive definite:
/// singular mass is rejected with ENG-E-0006 without producing NaN; indefinite
/// mass with ENG-E-0010. Explicit massless DOF reduction is in [`guyan_condense`].
/// For symmetric factorization and mass (Cholesky), the matrix is first balanced with
/// `S A S` (Ruiz-type iterative balancing) to tolerate DOFs with very
/// different scales. This MITIGATES, but does not guarantee, invariance under diagonal rescaling
/// `D A D`: it is exact for diagonal matrices and reliable for coupled matrices with
/// moderate scaling, but is not promised for extreme ranges. A pivot is
/// numerically singular if its magnitude is
/// ≤ `64 ε max(n,1)` after dividing the (balanced) matrix by its entry of greatest magnitude.
/// LIMIT of `solve_in_place`: each column of `b` is normalized by its entry of
/// greatest magnitude; if entries in the same column span a range greater than ~1e300
/// (ratio < ~1e-300), the small component underflows and is lost
/// (contributes 0 to the solution). This is an f64 limit and is not reported as an error.
/// Therefore, solving very ill-conditioned matrices is not guaranteed. The
/// policy is numerical, dimensionless, and explicit; it is not a physical tolerance.
pub trait LinAlgBackend {
    /// Custom descriptive backend name, independent of the external format.
    fn name(&self) -> &'static str;
    /// Computes `a b`; ENG-E-0010 if `a.cols()!=b.rows()`.
    fn matmul(&mut self, a: &DMat, b: &DMat) -> EResult<DMat>;
    /// Cooperative product; by default, checks before the backend kernel.
    fn matmul_with_check(
        &mut self,
        a: &DMat,
        b: &DMat,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<DMat> {
        check()?;
        self.matmul(a, b)
    }
    /// Transposes a rectangular matrix, preserving its quantities.
    fn transpose(&mut self, a: &DMat) -> EResult<DMat>;
    /// Solves a square dense, possibly nonsymmetric system with multiple RHS.
    /// ENG-E-0010 for squareness/sizes and ENG-E-0006 for singularity.
    fn solve_dense(&mut self, a: &DMat, b: &DMat) -> EResult<DMat>;
    /// Cooperative solve; by default, checks before the backend kernel.
    fn solve_dense_with_check(
        &mut self,
        a: &DMat,
        b: &DMat,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<DMat> {
        check()?;
        self.solve_dense(a, b)
    }
    /// Assembles general triplets into CSR, summing duplicates in input order.
    ///
    /// Each triplet represents a single entry, without automatic mirroring or
    /// division by two. Both halves are supplied for symmetry. Explicit
    /// zeros are retained in the pattern. ENG-E-0010 for out-of-range indices
    /// and ENG-E-0008 for nonfinite values or sums.
    fn assemble(&mut self, rows: usize, cols: usize, entries: &[Triplet]) -> EResult<SparseMatrix> {
        SparseMatrix::from_triplets(rows, cols, entries)
    }
    /// Factorizes a dense symmetric matrix once, without regularization.
    /// The factor owns its data and may outlive the backend and matrix.
    fn factorize(&mut self, a: &SymMatrix) -> EResult<Box<dyn SymFactorization>>;
    /// Factorizes while querying a control that may interrupt with its own error.
    /// By default queries before factorizing; ReferenceBackend also queries
    /// between blocks of up to 16 columns, without altering numerical operations.
    fn factorize_with_check(
        &mut self,
        a: &SymMatrix,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<Box<dyn SymFactorization>> {
        check()?;
        self.factorize(a)
    }
    /// Inertia (positive, negative, zero) by LDLᵀ congruence.
    ///
    /// By default queries the factor; ENG-E-0013 if it does not provide inertia.
    /// The reference backend allows singular matrices for this count,
    /// with the same numerical pivot tolerance as `factorize`, but without
    /// providing a singular factor for solving. Other backends may
    /// return ENG-E-0006 if the shift coincides with an eigenvalue.
    fn symmetric_inertia(&mut self, a: &SymMatrix) -> EResult<(usize, usize, usize)> {
        self.factorize(a)?
            .info()
            .inertia
            .ok_or_else(|| unsupported("Backend does not provide inertia for the Sturm check."))
    }
    /// Solves the dense generalized problem with positive definite mass.
    /// ENG-E-0010 if orders differ; see the trait's mass policy.
    fn generalized_eigen(&mut self, k: &SymMatrix, m: &SymMatrix) -> EResult<GeneralizedEigen>;
    /// Cooperative eigenvalues; the reference checks within its loops.
    fn generalized_eigen_with_check(
        &mut self,
        k: &SymMatrix,
        m: &SymMatrix,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<GeneralizedEigen> {
        check()?;
        self.generalized_eigen(k, m)
    }
    /// Cooperative inertia; the reference checks during LDLᵀ.
    fn symmetric_inertia_with_check(
        &mut self,
        a: &SymMatrix,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<(usize, usize, usize)> {
        check()?;
        self.symmetric_inertia(a)
    }
    /// Lowest mass-normalized modes with a Sturm completeness check.
    /// See [`lowest_modes`] and [`LowestModes`] for tolerances and diagnostics.
    fn lowest_modes(&mut self, k: &SymMatrix, m: &SymMatrix, count: usize) -> EResult<LowestModes> {
        lowest_modes(self, k, m, count)
    }
    /// Cooperative mode selection, including normalization and the Sturm check.
    fn lowest_modes_with_check(
        &mut self,
        k: &SymMatrix,
        m: &SymMatrix,
        count: usize,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<LowestModes> {
        modes::lowest_modes_with_check(self, k, m, count, check)
    }
}

pub(crate) fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
/// Limit on an individual numerical storage allocation, per target.
///
/// Avoids capacity panics (`capacity overflow`) and out-of-memory
/// aborts for extreme dimensions; beyond this size the dense engine is
/// not the appropriate path (sparse solver, ENG-LA-008).
/// On 64-bit targets the cap is 8 GiB; on 32-bit targets (e.g.
/// wasm32) it is explicitly 1 GiB, because `isize::MAX` (≈2 GiB) would not limit it in practice.
#[cfg(target_pointer_width = "64")]
pub(crate) const MAX_ALLOC_BYTES: usize = 1 << 33;

// 8 GiB does not fit in 32-bit usize: explicit cap of 1 GiB.
#[cfg(not(target_pointer_width = "64"))]
pub(crate) const MAX_ALLOC_BYTES: usize = 1 << 30;

/// Creates a `Vec` of `len` copies of `fill` with fallible, bounded allocation.
/// ENG-E-0010 with a diagnostic (elements, bytes, limit) if it exceeds the limit or
/// allocation fails; never panics.
pub(crate) fn alloc_vec<T: Clone>(len: usize, fill: T) -> EResult<Vec<T>> {
    let bytes = len
        .checked_mul(std::mem::size_of::<T>())
        .filter(|&b| b <= MAX_ALLOC_BYTES)
        .ok_or_else(|| {
            invalid(&format!(
                "Linear algebra: allocation of {len} elements exceeds the limit of {MAX_ALLOC_BYTES} bytes."
            ))
        })?;
    let mut v = Vec::new();
    v.try_reserve_exact(len).map_err(|_| {
        invalid(&format!(
            "Linear algebra: could not allocate {bytes} bytes for {len} elements."
        ))
    })?;
    v.resize(len, fill);
    Ok(v)
}
pub(crate) fn unsupported(message: &str) -> EngineError {
    EngineError::new("ENG-E-0013", message)
}
pub(crate) fn size(actual: usize, expected: usize) -> EResult<()> {
    if actual != expected {
        return Err(invalid(&format!(
            "Linear algebra: size {actual}, expected {expected}."
        )));
    }
    Ok(())
}
pub(crate) fn finite(values: &[Real]) -> EResult<()> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Linear algebra: non-finite input or result.",
        ));
    }
    Ok(())
}
pub(crate) fn singular(operation: &str, index: usize) -> EngineError {
    let mut error = EngineError::new(
        "ENG-E-0006",
        format!("{operation}: singular pivot at DOF {index}."),
    );
    if let Ok(index) = u32::try_from(index) {
        error.entities.push(EntityRef::Dof(DofIndex(index)));
    }
    error
}

#[cfg(test)]
mod max_alloc_tests {
    use super::*;

    #[test]
    fn max_alloc_bytes_is_explicit_per_target() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(MAX_ALLOC_BYTES, 8usize << 30);
        #[cfg(not(target_pointer_width = "64"))]
        {
            assert_eq!(MAX_ALLOC_BYTES, 1usize << 30);
            assert!(MAX_ALLOC_BYTES < isize::MAX as usize);
        }
    }

    #[test]
    fn alloc_vec_rejects_above_limit() {
        let too_many = MAX_ALLOC_BYTES / std::mem::size_of::<f64>() + 1;
        assert!(alloc_vec(too_many, 0.0f64).is_err());
    }
}
