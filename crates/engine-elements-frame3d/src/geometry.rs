//! Explicit right-handed triad; T global→local, Kglobal=Tᵀ Klocal T.
use crate::{finite, invalid, vector, Matrix};
use engine_contracts::ElementCtx;
use engine_core::math;
use engine_core::{Dim, EResult, EngineError, Real};
use engine_linalg::DMat;

pub(crate) struct Geometry {
    pub length: Real,
    axes: [[Real; 3]; 3],
}
/// Scaled normalization: accepts huge or tiny finite orientations.
/// Minimum sine of the angle between the orientation and local x axis.
pub(crate) const ORIENTATION_MIN_SIN: Real = 1e-8;
pub(crate) fn unit(v: [Real; 3]) -> EResult<[Real; 3]> {
    vector(&v, 3, "Orientation")?;
    let scale = v.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
    if scale == 0.0 {
        return Err(invalid("Orientation: zero vector."));
    }
    let v = v.map(|x| x / scale);
    let norm = math::hypot(math::hypot(v[0], v[1]), v[2]);
    Ok(v.map(|x| x / norm))
}
fn cross(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
impl Geometry {
    pub fn new(ctx: &ElementCtx<'_>, orientation: [Real; 3]) -> EResult<Self> {
        if ctx.dim != Dim::D3 || ctx.coords.len() != 2 {
            return Err(invalid("Spatial beam: requires D3 and two coordinates."));
        }
        for p in ctx.coords {
            vector(p, 3, "Coordinate (m)")?;
        }
        let mut delta = [0.0; 3];
        for (a, d) in delta.iter_mut().enumerate() {
            *d = finite(ctx.coords[1][a] - ctx.coords[0][a], "Coordinate difference")?;
        }
        let length = finite(
            math::hypot(math::hypot(delta[0], delta[1]), delta[2]),
            "L (m)",
        )?;
        if length <= 1e-12 {
            return Err(EngineError::new("ENG-E-0005", "Beam: L≤1e-12 m."));
        }
        let x = delta.map(|d| d / length);
        let hint = unit(orientation)?;
        // hint×x is equivalent to projecting hint normal to x and forming y=z×x.
        // `hint` is unit length, so |hint×x| = sin(angle). With sin ≤ 1e-8, the triad's
        // relative error would be ≥ ε/1e-8 ≈ 1e-8 and rigid rotations would leave
        // spurious forces (J4 r6): rejected. Reorthogonalization is also performed.
        let y = cross(hint, x);
        if math::hypot(math::hypot(y[0], y[1]), y[2]) <= ORIENTATION_MIN_SIN {
            return Err(invalid(
                "Orientation is parallel or nearly parallel to the local x axis (sine of angle ≤ 1e-8).",
            ));
        }
        let mut y = unit(y)?;
        // One Gram–Schmidt pass against x and renormalization: orthonormal to ~1e-16.
        let dot: Real = x.iter().zip(y).map(|(a, b)| a * b).sum();
        for (yi, xi) in y.iter_mut().zip(x) {
            *yi -= dot * xi;
        }
        let y = unit(y)?;
        let z = unit(cross(x, y))?;
        Ok(Self {
            length,
            axes: [x, y, z],
        })
    }
    fn transform(&self) -> Matrix {
        let mut t = [[0.0; 12]; 12];
        for start in [0, 3, 6, 9] {
            for (i, axis) in self.axes.iter().enumerate() {
                t[start + i][start..start + 3].copy_from_slice(axis);
            }
        }
        t
    }
    pub fn local_triplet(&self, v: [Real; 3]) -> EResult<[Real; 3]> {
        vector(&v, 3, "Global intensity")?;
        let mut out = [0.0; 3];
        for (r, axis) in out.iter_mut().zip(self.axes) {
            *r = finite(
                axis.iter().zip(v).map(|(a, v)| a * v).sum(),
                "Local intensity",
            )?;
        }
        Ok(out)
    }
    pub fn global_triplet(&self, v: [Real; 3]) -> EResult<[Real; 3]> {
        vector(&v, 3, "Local intensity")?;
        let mut out = [0.0; 3];
        for (i, r) in out.iter_mut().enumerate() {
            *r = finite(
                (0..3).map(|a| self.axes[a][i] * v[a]).sum(),
                "Global intensity",
            )?;
        }
        Ok(out)
    }
    pub fn local_vector(&self, u: &[Real]) -> EResult<[Real; 12]> {
        vector(u, 12, "u_e (m/rad)")?;
        multiply(&self.transform(), u)
    }
    pub fn global_vector(&self, f: &[Real; 12]) -> EResult<[Real; 12]> {
        crate::release::force(f, &self.transform())
    }
    pub fn global_matrix(&self, m: &Matrix) -> EResult<DMat> {
        let global = crate::release::project(m, &self.transform())?;
        DMat::new(12, 12, global.into_iter().flatten().collect())
    }
}
pub(crate) fn multiply(m: &Matrix, u: &[Real]) -> EResult<[Real; 12]> {
    let mut f = [0.0; 12];
    for (v, row) in f.iter_mut().zip(m) {
        *v = finite(
            row.iter().zip(u).map(|(k, u)| k * u).sum(),
            "Matrix/vector product",
        )?;
    }
    Ok(f)
}
