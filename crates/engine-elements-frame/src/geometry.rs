//! T convierte global a local; Kglobal=Tᵀ Klocal T.
use crate::{finite, invalid, vector, Matrix};
use engine_contracts::ElementCtx;
use engine_core::math;
use engine_core::{Dim, EResult, EngineError, Real};
use engine_linalg::DMat;

pub(crate) struct Geometry {
    pub length: Real,
    pub c: Real,
    pub s: Real,
}
impl Geometry {
    pub fn new(ctx: &ElementCtx<'_>) -> EResult<Self> {
        if ctx.dim != Dim::D2 || ctx.coords.len() != 2 {
            return Err(invalid("Viga: requiere D2 y dos coordenadas."));
        }
        for p in ctx.coords {
            vector(p, 3, "Coordenada (m)")?;
            if p[2] != 0.0 {
                return Err(invalid("D2 requiere Z=0."));
            }
        }
        let dx = finite(ctx.coords[1][0] - ctx.coords[0][0], "dx (m)")?;
        let dy = finite(ctx.coords[1][1] - ctx.coords[0][1], "dy (m)")?;
        let length = finite(math::hypot(dx, dy), "L (m)")?;
        if length <= 1e-12 {
            return Err(EngineError::new("ENG-E-0005", "Viga: L≤1e-12 m."));
        }
        Ok(Self {
            length,
            c: dx / length,
            s: dy / length,
        })
    }
    fn transform(&self) -> Matrix {
        let mut t = [[0.0; 6]; 6];
        for a in [0, 3] {
            t[a][a] = self.c;
            t[a][a + 1] = self.s;
            t[a + 1][a] = -self.s;
            t[a + 1][a + 1] = self.c;
            t[a + 2][a + 2] = 1.0;
        }
        t
    }
    pub fn local_vector(&self, u: &[Real]) -> EResult<[Real; 6]> {
        vector(u, 6, "u_e (m/rad)")?;
        multiply(&self.transform(), u)
    }
    pub fn global_vector(&self, f: &[Real; 6]) -> EResult<[Real; 6]> {
        crate::release::force(f, &self.transform())
    }
    pub fn global_matrix(&self, m: &Matrix) -> EResult<DMat> {
        let global = crate::release::project(m, &self.transform())?;
        DMat::new(6, 6, global.into_iter().flatten().collect())
    }
}
pub(crate) fn multiply(m: &Matrix, u: &[Real]) -> EResult<[Real; 6]> {
    let mut f = [0.0; 6];
    for (value, row) in f.iter_mut().zip(m) {
        *value = finite(
            row.iter().zip(u).map(|(k, u)| k * u).sum(),
            "Producto matriz/vector",
        )?;
    }
    Ok(f)
}
