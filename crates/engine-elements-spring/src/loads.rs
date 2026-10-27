//! No physical span: only the contract's zero load is accepted.
use crate::{invalid, vector};
use engine_contracts::MemberLoad;
use engine_core::EResult;

pub(crate) fn validate(load: &MemberLoad) -> EResult<()> {
    vector(&load.force_per_length, 3, "Span load")?;
    vector(&load.moment_per_length, 3, "Span moment")?;
    if load.force_per_length != [0.0; 3] || load.moment_per_length != [0.0; 3] {
        return Err(invalid(
            "Spring: span loads are not allowed; use nodal loads.",
        ));
    }
    Ok(())
}
