//! ENG-LA-009: eliminación estática exacta de filas/columnas de masa nulas.
use crate::{finite, singular, size, DMat, LinAlgBackend, SymMatrix};
use engine_core::{DofIndex, EResult, EngineError, EntityRef};

/// Complemento de Schur y transformación de recuperación de Guyan.
/// Los índices conservan el orden original; no se elimina masa pequeña no nula.
#[derive(Clone, Debug, PartialEq)]
pub struct GuyanReduction {
    /// Kmm − Kms Kss⁻¹ Ksm, exactamente simétrica por construcción.
    pub stiffness: SymMatrix,
    /// Mmm, sin aproximar ni diagonalizar.
    pub mass: SymMatrix,
    /// Índices originales de las filas con masa.
    massive: Vec<usize>,
    /// Índices originales de las filas/columnas de masa exactamente nulas.
    massless: Vec<usize>,
    /// R = −Kss⁻¹ Ksm; φs = R φm. Se resuelve con un único factor LDLᵀ.
    recovery: DMat,
}

impl GuyanReduction {
    /// Índices de GDL con masa en el orden original.
    pub fn massive(&self) -> &[usize] {
        &self.massive
    }
    /// Índices de GDL sin masa en el orden original.
    pub fn massless(&self) -> &[usize] {
        &self.massless
    }
    /// Transformación R = −Kss⁻¹ Ksm.
    pub fn recovery(&self) -> &DMat {
        &self.recovery
    }

    /// Recupera columnas modales en el espacio original, sin cambiar su norma
    /// de masa (φᵀMφ = φmᵀMmmφm). ENG-E-0010 por forma incompatible.
    pub fn recover(&self, modes: &DMat) -> EResult<DMat> {
        size(modes.rows(), self.massive.len())?;
        let count = modes.cols();
        let mut full = DMat::zeros(self.massive.len() + self.massless.len(), count)?;
        for (i, &row) in self.massive.iter().enumerate() {
            for col in 0..count {
                full.values[row * count + col] = modes.values()[i * count + col];
            }
        }
        for (i, &row) in self.massless.iter().enumerate() {
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

/// Particiona después de apoyos/MPC y condensa únicamente masa idénticamente
/// nula. `None` conserva el camino original del llamador sin copiar matrices.
///
/// Factoriza Kss una vez con el backend propio; exige inercia (s,0,0).
/// ENG-E-0014: Kss singular/no positiva, con GDL del bloque sin masa;
/// ENG-E-0006: diagonal de masa negativa o ausencia de GDL con masa;
/// ENG-E-0010: órdenes incompatibles; ENG-E-0008: resultado no finito;
/// ENG-E-0013: backend sin inercia. No regulariza K ni M. Memoria O(n²).
pub fn guyan_condense<B: LinAlgBackend + ?Sized>(
    backend: &mut B,
    k: &SymMatrix,
    m: &SymMatrix,
) -> EResult<Option<GuyanReduction>> {
    let n = k.order();
    size(m.order(), n)?;
    let mut has_massless = false;
    for i in 0..n {
        if m.dense().values()[i * n + i] < 0.0 {
            return Err(singular("Guyan: masa negativa", i));
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
            "Guyan: no quedan GDL libres con masa",
            massless[0],
        ));
    }
    let block = |rows: &[usize], cols: &[usize], matrix: &SymMatrix| -> EResult<DMat> {
        let mut result = DMat::zeros(rows.len(), cols.len())?;
        for (i, &row) in rows.iter().enumerate() {
            for (j, &col) in cols.iter().enumerate() {
                result.values[i * cols.len() + j] = matrix.dense().values()[row * n + col];
            }
        }
        Ok(result)
    };
    let kss = SymMatrix::new(block(&massless, &massless, k)?)?;
    let factor = backend.factorize(&kss).map_err(|e| {
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
        crate::unsupported("Guyan requiere inercia del factor Kss para comprobar positividad.")
    })?;
    if inertia != (massless.len(), 0, 0) {
        return Err(massless_mechanism(massless[0]));
    }
    let ksm = block(&massless, &massive, k)?;
    let mut recovery = factor.solve(&ksm)?;
    for value in &mut recovery.values {
        *value = -*value;
    }
    let mut condensed = block(&massive, &massive, k)?;
    let nm = massive.len();
    for i in 0..nm {
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
        mass: SymMatrix::new(block(&massive, &massive, m)?)?,
        massive,
        massless,
        recovery,
    }))
}

fn massless_mechanism(index: usize) -> EngineError {
    let mut error = EngineError::new(
        "ENG-E-0014",
        format!("Guyan: mecanismo sin masa; Kss singular o no definida positiva, bloque del GDL {index}."),
    );
    if let Ok(index) = u32::try_from(index) {
        error.entities.push(EntityRef::Dof(DofIndex(index)));
    }
    error
}
