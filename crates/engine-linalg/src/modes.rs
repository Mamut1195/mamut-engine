//! Dense modes and completeness checking by inertia, without regularization.
use crate::{finite, invalid, size, DMat, LinAlgBackend, SymMatrix};
use engine_core::{EResult, EngineError, Real};

/// Lowest modes of `Kφ=λMφ`, with positive definite mass.
#[derive(Clone, Debug, PartialEq)]
pub struct LowestModes {
    /// Ascending eigenvalues; numerically zero values are represented as zero.
    pub eigenvalues: Vec<Real>,
    /// Column eigenvectors, `φᵢᵀMφⱼ=δᵢⱼ`, positive dominant sign.
    pub vectors: DMat,
    /// Number of returned modes with numerically zero stiffness.
    ///
    /// These are rigid motions or possible mechanisms (ENG-E-0006 in a static
    /// analysis). K and M alone cannot distinguish them: the caller must review
    /// supports, connectivity, and releases. They are explicitly allowed for
    /// semidefinite K; they are neither regularized nor hidden by removing columns.
    pub rigid_mode_count: usize,
}

/// Counts eigenvalues strictly below `sigma` by LDLᵀ of `K−sigma M`.
///
/// M is validated as positive definite through the internal Cholesky. By congruence,
/// `K−σM` has as many negative signs as `L⁻¹KL⁻ᵀ−σI`. Negative scalar pivots
/// count one; for blocks `[[d,e],[e,g]]`, `det=dg−e²<0` counts one,
/// `det>0` and `tr=d+g<0` counts two; `det>0` and `tr>0` counts zero. A zero block
/// counts zero negatives (if det=0, the only nonzero sign is that of the trace).
///
/// Uses [`LinAlgBackend::symmetric_inertia`]; the reference excludes zero pivots
/// according to its numerical tolerance. Strict equality in the spectrum is understood
/// at that resolution; near a root, exact floating-point classification is not
/// guaranteed. sigma is not shifted to hide singularities.
/// ENG-E-0010 for orders/indefinite mass, ENG-E-0006 for singular mass,
/// ENG-E-0008 for nonfinite sigma/result and ENG-E-0013 if inertia is unavailable.
pub fn sturm_count<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
    sigma: Real,
) -> EResult<usize> {
    size(k.order(), m.order())?;
    finite(&[sigma])?;
    crate::reference::cholesky(m)?;
    count_with_valid_mass(backend, k, m, sigma, &mut || Ok(()))
}

fn count_with_valid_mass<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
    sigma: Real,
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<usize> {
    check()?;
    let n = k.order();
    let mut shifted = DMat::zeros(n, n)?;
    for ((entry, &ki), &mi) in shifted
        .values
        .iter_mut()
        .zip(k.dense().values())
        .zip(m.dense().values())
    {
        // Explicit FMA: a single rounding reduces cancellation near an
        // eigenvalue. mul_add semantics are required whether implemented in hardware or
        // software; exception documented in docs/DETERMINISMO.md.
        *entry = (-sigma).mul_add(mi, ki);
    }
    finite(&shifted.values)?;
    let (positive, negative, null) =
        backend.symmetric_inertia_with_check(&SymMatrix::new(shifted)?, check)?;
    if positive
        .checked_add(negative)
        .and_then(|v| v.checked_add(null))
        != Some(n)
    {
        return Err(incomplete(
            "El backend devolvió una inercia incompatible con el orden.",
        ));
    }
    Ok(negative)
}

fn incomplete(message: &str) -> EngineError {
    EngineError::new("ENG-E-0007", message)
}

/// Returns exactly `count` lowest modes, mass-normalized.
///
/// Dense path for small models: computes the spectrum with
/// [`LinAlgBackend::generalized_eigen`] (Cholesky/Jacobi in the reference),
/// selects columns, and checks inertia with the same backend. O(n²) memory,
/// without a sparse solver or connection to MemoryBudget.
///
/// The check requires `Sturm(σ)==count`, with `σ=λ_count(1+1e−7)`. For zero
/// or very small values, `σ≥λ_count+16·tol_cero` is also imposed, where
/// `tol_cero=128 ε max(n,1) max|λ|`. If all of K is zero, σ=1e−7.
/// The floor avoids evaluating exactly at a rigid root; it does not modify K or M.
///
/// ENG-E-0007 indicates missing/excess modes, an incomplete spectrum, or a cut
/// within a multiplicity (includes clusters within tolerance).
/// The entire group must be requested; fewer are never returned silently.
/// Values λ<−tol_cero yield ENG-E-0010 with a diagnostic of indefinite stiffness or
/// instability; |λ|≤tol_cero are identified as zero-stiffness modes.
/// This is not a physical tolerance: very ill-conditioned spectra may require
/// another backend. Singular mass: ENG-E-0006; indefinite mass: ENG-E-0010.
/// `count>n` or different orders: ENG-E-0010. `count=0` returns n×0 and neither
/// factorizes nor requires positive M, although it validates compatible orders.
pub fn lowest_modes<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
    count: usize,
) -> EResult<LowestModes> {
    lowest_modes_with_check(backend, k, m, count, &mut || Ok(()))
}

pub(crate) fn lowest_modes_with_check<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
    count: usize,
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<LowestModes> {
    check()?;
    let n = k.order();
    size(n, m.order())?;
    if count > n {
        return Err(invalid(&format!(
            "Modos: count={count} excede el orden n={n}."
        )));
    }
    if count == 0 {
        return Ok(LowestModes {
            eigenvalues: Vec::new(),
            vectors: DMat::zeros(n, 0)?,
            rigid_mode_count: 0,
        });
    }
    crate::reference::cholesky_with_check(m, check)?;
    let all = backend.generalized_eigen_with_check(k, m, check)?;
    if all.eigenvalues.len() != n || all.vectors.rows() != n || all.vectors.cols() != n {
        return Err(incomplete(
            "MODOS FALTANTES: el backend no devolvió el espectro denso completo.",
        ));
    }
    finite(&all.eigenvalues)?;
    if all.eigenvalues.windows(2).any(|pair| pair[0] > pair[1]) {
        return Err(incomplete(
            "Modos: el backend devolvió autovalores sin ordenar.",
        ));
    }
    let scale = all.eigenvalues.iter().fold(0.0_f64, |s, v| s.max(v.abs()));
    let zero_tol = 128.0 * Real::EPSILON * (n as Real) * scale;
    if let Some(&value) = all.eigenvalues.iter().find(|&&v| v < -zero_tol) {
        return Err(invalid(&format!(
            "Modos: λ negativo {value}; K indefinida, posible inestabilidad. No es un modo rígido (tolerancia {zero_tol})."
        )));
    }
    let last = all.eigenvalues[count - 1].max(0.0);
    let sigma = if scale == 0.0 {
        1e-7
    } else {
        (last * (1.0 + 1e-7)).max(last + 16.0 * zero_tol)
    };
    finite(&[sigma, zero_tol])?;
    let found = count_with_valid_mass(backend, k, m, sigma, check)?;
    if found != count {
        return Err(incomplete(&format!(
            "MODOS FALTANTES / corte de multiplicidad: Sturm(σ={sigma})={found}, modos solicitados/devueltos={count}. Solicite el grupo completo o revise la convergencia del backend."
        )));
    }
    // Reapplies the backend normalization contract before selecting.
    let all = crate::reference::normalize_modes_with_check(all.eigenvalues, all.vectors, m, check)?;
    let mut eigenvalues = crate::alloc_vec(count, 0.0)?;
    let mut vectors = DMat::zeros(n, count)?;
    let mut rigid_mode_count = 0;
    for (col, eigenvalue) in eigenvalues.iter_mut().enumerate() {
        check()?;
        if all.eigenvalues[col].abs() <= zero_tol {
            rigid_mode_count += 1;
        } else {
            *eigenvalue = all.eigenvalues[col];
        }
        for row in 0..n {
            vectors.values[row * count + col] = all.vectors.values()[row * n + col];
        }
    }
    Ok(LowestModes {
        eigenvalues,
        vectors,
        rigid_mode_count,
    })
}
