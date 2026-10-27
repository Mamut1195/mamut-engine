//! Ideal massless spring, validating the context even with None output.
use crate::{geometry, LinearSpring};
use engine_contracts::{ElementCtx, MassKind};
use engine_core::EResult;
use engine_linalg::DMat;

pub(crate) fn matrix(
    s: &LinearSpring,
    ctx: &ElementCtx<'_>,
    _kind: MassKind,
) -> EResult<Option<DMat>> {
    geometry::validate(s, ctx)?;
    Ok(None)
}
