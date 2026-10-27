//! Fixed axes and linear kinematics: zero Kg, indicated by None.
use crate::{geometry, vector, LinearSpring};
use engine_contracts::ElementCtx;
use engine_core::{EResult, Real};
use engine_linalg::DMat;

pub(crate) fn matrix(
    s: &LinearSpring,
    ctx: &ElementCtx<'_>,
    u: &[Real],
    committed: &[Real],
) -> EResult<Option<DMat>> {
    geometry::validate(s, ctx)?;
    vector(u, s.size(), "u_e")?;
    vector(committed, 0, "Historia")?;
    Ok(None)
}
