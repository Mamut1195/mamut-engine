//! Sin tramo físico: solo se admite la carga nula del contrato.
use crate::{invalid, vector};
use engine_contracts::MemberLoad;
use engine_core::EResult;

pub(crate) fn validate(load: &MemberLoad) -> EResult<()> {
    vector(&load.force_per_length, 3, "Carga de tramo")?;
    vector(&load.moment_per_length, 3, "Momento de tramo")?;
    if load.force_per_length != [0.0; 3] || load.moment_per_length != [0.0; 3] {
        return Err(invalid(
            "Resorte: cargas de tramo no admitidas; use cargas nodales.",
        ));
    }
    Ok(())
}
