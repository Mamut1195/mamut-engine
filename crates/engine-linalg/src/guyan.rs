//! ENG-LA-009: exact static elimination of zero-mass rows/columns.
use crate::{finite, singular, size, DMat, LinAlgBackend, SymMatrix};
use engine_core::{DofIndex, EResult, EngineError, EntityRef};

/// Schur complement and Guyan recovery transformation.
/// Indices preserve original order; small nonzero mass is not eliminated.
#[derive(Clone, Debug, PartialEq)]
pub struct GuyanReduction {
    /// Kmm − Kms Kss⁻¹ Ksm, exactly symmetric by construction.
    pub stiffness: SymMatrix,
    /// Mmm, without approximation or diagonalization.
    pub mass: SymMatrix,
    /// Original indices of rows with mass.
    massive: Vec<usize>,
    /// Original indices of exactly zero-mass rows/columns.
    massless: Vec<usize>,
    /// R = −Kss⁻¹ Ksm; φs = R φm. Solved with a single LDLᵀ factor.
    recovery: DMat,
}

impl GuyanReduction {
    /// Indices of DOFs with mass in original order.
    pub fn massive(&self) -> &[usize] {
        &self.massive
    }
    /// Indices of massless DOFs in original order.
    pub fn massless(&self) -> &[usize] {
        &self.massless
    }
    /// Transformation R = −Kss⁻¹ Ksm.
    pub fn recovery(&self) -> &DMat {
        &self.recovery
    }

    /// Recovers modal columns in the original space, without changing their mass
    /// norm (φᵀMφ = φmᵀMmmφm). ENG-E-0010 for incompatible shape.
    pub fn recover(&self, modes: &DMat) -> EResult<DMat> {
        self.recover_with_check(modes, &mut || Ok(()))
    }

    /// Cooperative recovery, checks per row without changing the reductions.
    pub fn recover_with_check(
        &self,
        modes: &DMat,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<DMat> {
        size(modes.rows(), self.massive.len())?;
        let count = modes.cols();
        let mut full = DMat::zeros(self.massive.len() + self.massless.len(), count)?;
        for (i, &row) in self.massive.iter().enumerate() {
            for col in 0..count {
                full.values[row * count + col] = modes.values()[i * count + col];
            }
        }
        for (i, &row) in self.massless.iter().enumerate() {
            check()?;
            for col in 0..count {
                let mut value = 0.0;
                for j in 0..self.massive.len() {
                    value += self.recovery.values()[i * self.massive.len() + j]
                        * modes.values()[j * count + col];
                }
                full.values[row * count + col] = value;
            }
        }
        finite(full.values())?;
        Ok(full)
    }
}

/// Partitions after supports/MPC and condenses only identically zero
/// mass. `None` preserves the caller's original path without copying matrices.
///
/// Factorizes Kss once with the internal backend; requires inertia (s,0,0).
/// ENG-E-0014: singular/nonpositive Kss, with DOFs of the massless block;
/// ENG-E-0006: negative mass diagonal or absence of DOFs with mass;
/// ENG-E-0010: incompatible orders; ENG-E-0008: nonfinite result;
/// ENG-E-0013: backend without inertia. Does not regularize K or M. Memory O(n²).
pub fn guyan_condense<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
) -> EResult<Option<GuyanReduction>> {
    guyan_condense_with_check(backend, k, m, &mut || Ok(()))
}

/// Cooperative condensation: checks per row and during factorization.
pub fn guyan_condense_with_check<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<Option<GuyanReduction>> {
    check()?;
    let n = k.order();
    size(m.order(), n)?;
    let mut has_massless = false;
    for i in 0..n {
        check()?;
        if m.dense().values()[i * n + i] < 0.0 {
            return Err(singular("Guyan: negative mass", i));
        }
        has_massless |= m.dense().values()[i * n..(i + 1) * n]
            .iter()
            .all(|&v| v == 0.0);
    }
    if !has_massless {
        return Ok(None);
    }
    let mut massive = Vec::new();
    let mut massless = Vec::new();
    for i in 0..n {
        check()?;
        if m.dense().values()[i * n..(i + 1) * n]
            .iter()
            .any(|&v| v != 0.0)
        {
            massive.push(i);
        } else {
            massless.push(i);
        }
    }
    if massive.is_empty() {
        return Err(singular(
            "Guyan: no free DOFs with mass remain",
            massless[0],
        ));
    }
    let block = |rows: &[usize],
                 cols: &[usize],
                 matrix: &SymMatrix,
                 check: &mut dyn FnMut() -> EResult<()>|
     -> EResult<DMat> {
        let mut result = DMat::zeros(rows.len(), cols.len())?;
        for (i, &row) in rows.iter().enumerate() {
            check()?;
            for (j, &col) in cols.iter().enumerate() {
                result.values[i * cols.len() + j] = matrix.dense().values()[row * n + col];
            }
        }
        Ok(result)
    };
    let kss = SymMatrix::new(block(&massless, &massless, k, check)?)?;
    let factor = backend.factorize_with_check(&kss, check).map_err(|e| {
        if e.code != "ENG-E-0006" {
            return e;
        }
        let index = e.entities.iter().find_map(|entity| match entity {
            EntityRef::Dof(d) => massless.get(d.0 as usize).copied(),
            _ => None,
        });
        massless_mechanism(match index {
            Some(i) => i,
            None => massless[0],
        })
    })?;
    let inertia = factor.info().inertia.ok_or_else(|| {
        crate::unsupported("Guyan requires Kss factor inertia to check positivity.")
    })?;
    if inertia != (massless.len(), 0, 0) {
        return Err(massless_mechanism(massless[0]));
    }
    let ksm = block(&massless, &massive, k, check)?;
    let mut recovery = factor.solve_with_check(&ksm, check)?;
    for value in &mut recovery.values {
        *value = -*value;
    }
    let mut condensed = block(&massive, &massive, k, check)?;
    let nm = massive.len();
    for i in 0..nm {
        check()?;
        for j in 0..=i {
            let mut value = condensed.values[i * nm + j];
            for s in 0..massless.len() {
                value += ksm.values()[s * nm + i] * recovery.values()[s * nm + j];
            }
            condensed.values[i * nm + j] = value;
            condensed.values[j * nm + i] = value;
        }
    }
    finite(condensed.values())?;
    Ok(Some(GuyanReduction {
        stiffness: SymMatrix::new(condensed)?,
        mass: SymMatrix::new(block(&massive, &massive, m, check)?)?,
        massive,
        massless,
        recovery,
    }))
}

fn massless_mechanism(index: usize) -> EngineError {
    let mut error = EngineError::new(
        "ENG-E-0014",
        format!("Guyan: massless mechanism; Kss is singular or not positive definite, DOF block {index}."),
    );
    if let Ok(index) = u32::try_from(index) {
        error.entities.push(EntityRef::Dof(DofIndex(index)));
    }
    error
}
