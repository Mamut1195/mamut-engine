//! Second variation N ∫ v′² dx; original derivation with Hermite shapes.
use crate::{finite, Matrix};
use engine_core::{EResult, Real};

pub(crate) fn local_matrix(l: Real, n: Real) -> EResult<Matrix> {
    let a = finite((n / l) / 30.0, "N/(30L)")?;
    let b = finite(n / 30.0, "N/30")?;
    let c = finite(b * l, "NL/30")?;
    let block = [
        [36.0 * a, 3.0 * b, -36.0 * a, 3.0 * b],
        [3.0 * b, 4.0 * c, -3.0 * b, -c],
        [-36.0 * a, -3.0 * b, 36.0 * a, -3.0 * b],
        [3.0 * b, -c, -3.0 * b, 4.0 * c],
    ];
    let mut m = [[0.0; 6]; 6];
    let ids = [1, 2, 4, 5];
    for (i, &r) in ids.iter().enumerate() {
        for (j, &s) in ids.iter().enumerate() {
            m[r][s] = finite(block[i][j], "Kg")?;
        }
    }
    Ok(m)
}
