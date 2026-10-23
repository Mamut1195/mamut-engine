//! Geometría de referencia SI, sin orientación transversal arbitraria.
use crate::{finite, invalid};
use engine_contracts::ElementCtx;
use engine_core::math;
use engine_core::{Dim, EResult, EngineError, Real};

pub(crate) struct Geometry {
    pub axes: usize,
    pub length: Real,
    pub d: [Real; 3],
}
impl Geometry {
    pub fn new(dim: Dim, ctx: &ElementCtx<'_>) -> EResult<Self> {
        if ctx.dim != dim || ctx.coords.len() != 2 {
            return Err(invalid(
                "Truss: incompatible dimension or coordinate count.",
            ));
        }
        for p in ctx.coords {
            for &x in p {
                finite(x, "Coordinate (m)")?;
            }
            if dim == Dim::D2 && p[2] != 0.0 {
                return Err(invalid("D2 requires Z=0."));
            }
        }
        let axes = if dim == Dim::D2 { 2 } else { 3 };
        let mut d = [0.0; 3];
        let mut length: Real = 0.0;
        for (axis, value) in d.iter_mut().enumerate().take(axes) {
            *value = finite(
                ctx.coords[1][axis] - ctx.coords[0][axis],
                "Coordinate difference (m)",
            )?;
            length = math::hypot(length, *value);
        }
        finite(length, "Length (m)")?;
        if length <= 1e-12 {
            return Err(EngineError::new("ENG-E-0005", "Truss: L≤1e-12 m."));
        }
        for value in &mut d {
            *value /= length;
        }
        Ok(Self { axes, length, d })
    }
    pub fn b(&self) -> Vec<Real> {
        self.d[..self.axes]
            .iter()
            .map(|d| -d)
            .chain(self.d[..self.axes].iter().copied())
            .collect()
    }
}
