//! Cumulative memory budget (ENG-LA-008).
//!
//! `alloc_vec` limits each individual allocation; many allowed allocations can
//! add up to more than is available (e.g. wasm32, 4 GiB of address space).
//! `MemoryBudget` tracks bytes charged by a caller and rejects
//! any charge exceeding the limit with ENG-E-0013 (unsupported capability).
//! This is explicit, deterministic accounting: it does not query the operating system,
//! use threads or global state; each caller owns its budget.
use crate::*;

/// Cumulative memory budget, in bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryBudget {
    limit: usize,
    used: usize,
    peak: usize,
}

const F64: usize = std::mem::size_of::<Real>();

/// Upper bound on heap storage RETAINED by an order-`n` factorization of the
/// reference backend: dense L, D, secondary diagonal, blocks
/// (reserved capacity n), permutation, scales, and factor header in its Box.
/// Excludes input and allocator internal metadata. `None` on overflow.
pub fn factor_bytes(n: usize) -> Option<usize> {
    let vectors =
        n.checked_mul(3 * std::mem::size_of::<f64>() + 2 * std::mem::size_of::<usize>())?;
    dense_bytes(n, n)?
        .checked_add(vectors)?
        .checked_add(crate::reference::FACTOR_HEADER_BYTES)
}

/// Bound on TRANSIENT work: balanced and the normalized copy a remain live
/// alongside L in LDLᵀ. Temporary balancing rows (n values) fit
/// within these two matrices for n>0; for n=0 they do not allocate. `None` on overflow.
fn factor_working_bytes(n: usize) -> Option<usize> {
    dense_bytes(n, n)?.checked_mul(2)
}

/// Upper bound on TRANSIENT work for order-`n` `generalized_eigen`
/// in the reference backend: nine dense n×n matrices (Cholesky, inverse,
/// products, Jacobi, and excess capacity when collecting modes), plus auxiliary
/// vectors and the LDLᵀ header for diagnosing mass rejected by Cholesky.
/// Excludes inputs and result (n² modes + n eigenvalues), which
/// `MemoryBudget::generalized_eigen` charges separately. Excludes allocator internal
/// metadata. `None` on overflow.
pub fn eigen_working_bytes(n: usize) -> Option<usize> {
    // Balancing, temporary rows, D, and secondary diagonal: four Real
    // vectors; blocks/permutation or indices/ordering workspace: two usize vectors.
    let auxiliary = n.checked_mul(4 * F64 + 2 * std::mem::size_of::<usize>())?;
    dense_bytes(n, n)?
        .checked_mul(9)?
        .checked_add(auxiliary)?
        .checked_add(crate::reference::FACTOR_HEADER_BYTES)
}

fn eigen_result_bytes(n: usize) -> Option<usize> {
    dense_bytes(n, n)?.checked_add(n.checked_mul(std::mem::size_of::<f64>())?)
}

/// Bytes of a dense `rows × cols` matrix of `Real`; `None` on overflow.
pub fn dense_bytes(rows: usize, cols: usize) -> Option<usize> {
    rows.checked_mul(cols)?.checked_mul(F64)
}

/// Approximate bytes of a CSR with `rows` rows and `nnz` entries
/// (offsets + column indices + values); `None` on overflow.
pub fn sparse_bytes(rows: usize, nnz: usize) -> Option<usize> {
    let offsets = rows
        .checked_add(1)?
        .checked_mul(std::mem::size_of::<usize>())?;
    let entries = nnz.checked_mul(std::mem::size_of::<usize>() + F64)?;
    offsets.checked_add(entries)
}

impl MemoryBudget {
    /// Budget with `limit` bytes.
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            used: 0,
            peak: 0,
        }
    }
    /// Budget equal to the target's per-allocation limit (`MAX_ALLOC_BYTES`:
    /// 8 GiB on 64-bit, 1 GiB on 32-bit).
    pub fn for_target() -> Self {
        Self::new(MAX_ALLOC_BYTES)
    }
    /// Total limit in bytes.
    pub fn limit(&self) -> usize {
        self.limit
    }
    /// Currently charged bytes.
    pub fn used(&self) -> usize {
        self.used
    }
    /// Maximum bytes charged simultaneously since creation.
    pub fn peak(&self) -> usize {
        self.peak
    }
    /// Bytes still available.
    pub fn remaining(&self) -> usize {
        self.limit - self.used
    }
    /// Charges `bytes`. ENG-E-0013 if it exceeds the limit (or overflows); on error
    /// `used` remains intact (`peak` does not change because the charge was rejected).
    pub fn charge(&mut self, bytes: usize) -> EResult<()> {
        match self.used.checked_add(bytes) {
            Some(total) if total <= self.limit => {
                self.used = total;
                self.peak = self.peak.max(total);
                Ok(())
            }
            _ => Err(unsupported(&format!(
                "Presupuesto de memoria excedido: {} usados + {bytes} solicitados > límite {} bytes.",
                self.used, self.limit
            ))),
        }
    }
    /// Releases `bytes` (saturating at 0: excess release never yields negative usage).
    /// Charges have no identity: a caller's double release subtracts from other
    /// live charges (undercounting); release only what the caller itself charged.
    pub fn release(&mut self, bytes: usize) {
        self.used = self.used.saturating_sub(bytes);
    }
    /// Releases storage of a dense matrix charged to this budget.
    pub fn release_dense(&mut self, m: &DMat) {
        if let Some(b) = dense_bytes(m.rows(), m.cols()) {
            self.release(b);
        }
    }
    /// Factorizes `a` after charging the budget (retained + work). Success: only
    /// the retained factorization remains charged (`factor_bytes(n)`; free it with
    /// `release` when dropping it); error or caught panic: `used` intact.
    /// Error order: shape/size overflow → budget (ENG-E-0013) → backend.
    /// The bound covers numerical storage of the reference backend;
    /// other backends and error message construction are not bounded.
    pub fn factorize<B: LinAlgBackend>(
        &mut self,
        backend: &mut B,
        a: &SymMatrix,
    ) -> EResult<Box<dyn SymFactorization>> {
        let overflow = || invalid("Tamaño matricial desbordado.");
        let n = a.order();
        let retained = factor_bytes(n).ok_or_else(overflow)?;
        let total = retained
            .checked_add(factor_working_bytes(n).ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
        self.charge(total)?;
        let mut guard = ReleaseOnDrop {
            budget: self,
            bytes: total,
        };
        let result = backend.factorize(a);
        if result.is_ok() {
            guard.bytes = total - retained;
        }
        drop(guard);
        result
    }
    /// Budgeted generalized eigenvalues: charges the working set
    /// (`eigen_working_bytes(n)`) plus result; success: only result bytes remain
    /// charged (n² + n `f64` values); error or panic: `used` intact.
    /// The bound covers numerical storage of the reference backend;
    /// other backends and error message construction are not bounded.
    pub fn generalized_eigen<B: LinAlgBackend>(
        &mut self,
        backend: &mut B,
        k: &SymMatrix,
        m: &SymMatrix,
    ) -> EResult<GeneralizedEigen> {
        if k.order() != m.order() {
            return backend.generalized_eigen(k, m);
        }
        let overflow = || invalid("Tamaño matricial desbordado.");
        let n = k.order();
        let work = eigen_working_bytes(n).ok_or_else(overflow)?;
        let result_bytes = eigen_result_bytes(n).ok_or_else(overflow)?;
        let total = work.checked_add(result_bytes).ok_or_else(overflow)?;
        self.charge(total)?;
        let mut guard = ReleaseOnDrop {
            budget: self,
            bytes: total,
        };
        let result = backend.generalized_eigen(k, m);
        if result.is_ok() {
            guard.bytes = work;
        }
        drop(guard);
        result
    }
    /// Creates a zero matrix and charges it; the caller frees it with `release_dense`.
    /// ENG-E-0013 if it does not fit the budget; ENG-E-0010 for overflow/allocation.
    pub fn zeros(&mut self, rows: usize, cols: usize) -> EResult<DMat> {
        let bytes =
            dense_bytes(rows, cols).ok_or_else(|| invalid("Tamaño matricial desbordado."))?;
        self.charge(bytes)?;
        DMat::zeros(rows, cols).inspect_err(|_| self.release(bytes))
    }
    /// Solves `A X = B` with a backend, charging the working set
    /// (copy of A, B, and X: `n² + 2·n·m` elements). Transient work is
    /// released on completion; solution X remains charged until `release_dense`.
    /// ENG-E-0013 if the working set does not fit; backend errors are
    /// propagated and leave `used` as it was (`peak` preserves the maximum reached
    /// by the working set, which was actually charged); a caught panic
    /// also leaves no charge in `used`.
    pub fn solve_dense<B: LinAlgBackend>(
        &mut self,
        backend: &mut B,
        a: &DMat,
        b: &DMat,
    ) -> EResult<DMat> {
        // Error order independent of budget: first operand shape
        // (validated by the backend without allocating anything), then size
        // overflow, and finally budget (ENG-E-0013).
        if a.rows() != a.cols() || b.rows() != a.rows() {
            return backend.solve_dense(a, b);
        }
        let overflow = || invalid("Tamaño matricial desbordado.");
        let a_bytes = dense_bytes(a.rows(), a.cols()).ok_or_else(overflow)?;
        let b_bytes = dense_bytes(b.rows(), b.cols()).ok_or_else(overflow)?;
        let transient = a_bytes.checked_add(b_bytes).ok_or_else(overflow)?;
        let total = transient.checked_add(b_bytes).ok_or_else(overflow)?;
        self.charge(total)?;
        // RAII guard: if the backend panics (and the caller catches it
        // with `catch_unwind`), the charge is released during unwinding; `used` does not remain
        // charged.
        let mut guard = ReleaseOnDrop {
            budget: self,
            bytes: total,
        };
        let result = backend.solve_dense(a, b);
        if result.is_ok() {
            // Success: only transient work is released; X remains charged.
            guard.bytes = transient;
        }
        drop(guard);
        result
    }
}

/// Releases `bytes` from the budget on destruction (also during panic unwinding).
struct ReleaseOnDrop<'a> {
    budget: &'a mut MemoryBudget,
    bytes: usize,
}
impl Drop for ReleaseOnDrop<'_> {
    fn drop(&mut self) {
        self.budget.release(self.bytes);
    }
}
