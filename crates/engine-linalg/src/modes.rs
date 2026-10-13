//! Modos densos y comprobación de completitud por inercia, sin regularización.
use crate::{finite, invalid, size, DMat, LinAlgBackend, SymMatrix};
use engine_core::{EResult, EngineError, Real};

/// Modos inferiores de `Kφ=λMφ`, con masa definida positiva.
#[derive(Clone, Debug, PartialEq)]
pub struct LowestModes {
    /// Autovalores ascendentes; los numéricamente nulos se representan como cero.
    pub eigenvalues: Vec<Real>,
    /// Autovectores columna, `φᵢᵀMφⱼ=δᵢⱼ`, signo dominante positivo.
    pub vectors: DMat,
    /// Número de modos devueltos con rigidez numéricamente nula.
    ///
    /// Son movimientos rígidos o posibles mecanismos (ENG-E-0006 en un análisis
    /// estático). K y M solas no permiten distinguirlos: el llamador debe revisar
    /// apoyos, conectividad y liberaciones. Se admiten explícitamente para K
    /// semidefinida; no se regularizan ni se ocultan eliminando columnas.
    pub rigid_mode_count: usize,
}

/// Cuenta autovalores estrictamente menores que `sigma` por LDLᵀ de `K−sigma M`.
///
/// M se valida definida positiva mediante el Cholesky propio. Por congruencia
/// `K−σM` tiene tantos signos negativos como `L⁻¹KL⁻ᵀ−σI`. Los pivotes escalares
/// negativos cuentan uno; para bloques `[[d,e],[e,g]]`, `det=dg−e²<0` cuenta uno,
/// `det>0` y `tr=d+g<0` cuenta dos; `det>0` y `tr>0` cuenta cero. Un bloque nulo
/// cuenta cero negativos (si det=0, el único signo no nulo es el de la traza).
///
/// Usa [`LinAlgBackend::symmetric_inertia`]; la referencia excluye pivotes nulos
/// según su tolerancia numérica. La igualdad estricta en el espectro se entiende
/// con esa resolución; cerca de una raíz no se garantiza clasificación exacta
/// en coma flotante. No se desplaza sigma para esconder singularidades.
/// ENG-E-0010 por órdenes/masa indefinida, ENG-E-0006 por masa singular,
/// ENG-E-0008 por sigma/resultado no finito y ENG-E-0013 si no hay inercia.
pub fn sturm_count<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
    sigma: Real,
) -> EResult<usize> {
    size(k.order(), m.order())?;
    finite(&[sigma])?;
    crate::reference::cholesky(m)?;
    count_with_valid_mass(backend, k, m, sigma)
}

fn count_with_valid_mass<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
    sigma: Real,
) -> EResult<usize> {
    let n = k.order();
    let mut shifted = DMat::zeros(n, n)?;
    for ((entry, &ki), &mi) in shifted
        .values
        .iter_mut()
        .zip(k.dense().values())
        .zip(m.dense().values())
    {
        // Un único redondeo reduce cancelación cerca de un autovalor.
        *entry = (-sigma).mul_add(mi, ki);
    }
    finite(&shifted.values)?;
    let (positive, negative, null) = backend.symmetric_inertia(&SymMatrix::new(shifted)?)?;
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

/// Devuelve exactamente `count` modos inferiores, normalizados a masa.
///
/// Camino denso para modelos pequeños: calcula el espectro con
/// [`LinAlgBackend::generalized_eigen`] (Cholesky/Jacobi en la referencia),
/// selecciona columnas y comprueba inercia con el mismo backend. Memoria O(n²),
/// sin solver disperso ni conexión a MemoryBudget.
///
/// El control exige `Sturm(σ)==count`, con `σ=λ_count(1+1e−7)`. Para valores
/// nulos o muy pequeños se impone además `σ≥λ_count+16·tol_cero`, donde
/// `tol_cero=128 ε max(n,1) max|λ|`. Si todo K es nulo, σ=1e−7.
/// El piso evita evaluar exactamente una raíz rígida; no modifica K ni M.
///
/// ENG-E-0007 indica modos faltantes/excedentes, espectro incompleto o un corte
/// dentro de una multiplicidad (incluye agrupaciones dentro de la tolerancia).
/// Se debe solicitar el grupo completo; nunca se devuelve silenciosamente menos.
/// Valores λ<−tol_cero dan ENG-E-0010 con diagnóstico de rigidez indefinida o
/// inestabilidad; |λ|≤tol_cero se identifican como modos de rigidez nula.
/// No es una tolerancia física: espectros muy mal condicionados pueden requerir
/// otro backend. Masa singular: ENG-E-0006; masa indefinida: ENG-E-0010.
/// `count>n` u órdenes distintos: ENG-E-0010. `count=0` devuelve n×0 y no
/// factoriza ni exige positividad de M, aunque valida órdenes compatibles.
pub fn lowest_modes<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
    count: usize,
) -> EResult<LowestModes> {
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
    crate::reference::cholesky(m)?;
    let all = backend.generalized_eigen(k, m)?;
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
    let found = count_with_valid_mass(backend, k, m, sigma)?;
    if found != count {
        return Err(incomplete(&format!(
            "MODOS FALTANTES / corte de multiplicidad: Sturm(σ={sigma})={found}, modos solicitados/devueltos={count}. Solicite el grupo completo o revise la convergencia del backend."
        )));
    }
    // Reaplica el contrato de normalización del backend antes de seleccionar.
    let all = crate::reference::normalize_modes(all.eigenvalues, all.vectors, m)?;
    let mut eigenvalues = crate::alloc_vec(count, 0.0)?;
    let mut vectors = DMat::zeros(n, count)?;
    let mut rigid_mode_count = 0;
    for (col, eigenvalue) in eigenvalues.iter_mut().enumerate() {
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
