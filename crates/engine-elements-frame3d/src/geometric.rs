//! N∫(v′²+w′²)dx, second variation of transverse elongation.
use crate::{finite, stiffness::block, Matrix, PLANES};
use engine_core::{EResult, Real};

pub(crate) fn local_matrix(l: Real, n: Real) -> EResult<Matrix> {
    let a = finite((n / l) / 30.0, "N/(30L)")?;
    let b = finite(n / 30.0, "N/30")?;
    let c = finite(b * l, "NL/30")?;
    let bending = [
        [36.0 * a, 3.0 * b, -36.0 * a, 3.0 * b],
        [3.0 * b, 4.0 * c, -3.0 * b, -c],
        [-36.0 * a, -3.0 * b, 36.0 * a, -3.0 * b],
        [3.0 * b, -c, -3.0 * b, 4.0 * c],
    ];
    let mut m = [[0.0; 12]; 12];
    for (ids, sign) in PLANES {
        block(&mut m, ids, sign, bending)?;
    }
    Ok(m)
}
