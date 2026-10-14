//! Entradas SI propias, compartidas por repetibilidad y cruce nativo/wasm.
pub mod scalar_cases;

use engine_api::{run, CancelToken, Status};
use sha2::{Digest, Sha256};

pub const FIXTURES: &[(&str, &str)] = &[
    (
        "cantilever",
        include_str!("../fixtures/determinism/cantilever.json"),
    ),
    (
        "inclined-cantilever",
        include_str!("../fixtures/determinism/inclined-cantilever.json"),
    ),
    (
        "modal-1dof",
        include_str!("../fixtures/determinism/modal-1dof.json"),
    ),
    (
        "pdelta",
        include_str!("../fixtures/determinism/pdelta.json"),
    ),
    (
        "modal-coupled",
        include_str!("../fixtures/determinism/modal-coupled.json"),
    ),
    (
        "spectrum-log-cqc",
        include_str!("../fixtures/determinism/spectrum-log-cqc.json"),
    ),
    (
        "pdelta-inclined",
        include_str!("../fixtures/determinism/pdelta-inclined.json"),
    ),
    (
        "modal-inclined",
        include_str!("../fixtures/determinism/modal-inclined.json"),
    ),
    (
        "spectrum-inclined",
        include_str!("../fixtures/determinism/spectrum-inclined.json"),
    ),
    (
        "frame3d-inclined-static",
        include_str!("../fixtures/determinism/frame3d-inclined-static.json"),
    ),
    (
        "frame3d-inclined-modal",
        include_str!("../fixtures/determinism/frame3d-inclined-modal.json"),
    ),
];

pub fn case_count() -> usize {
    FIXTURES.len() + scalar_cases::NAMES.len()
}

pub fn case_name(index: usize) -> Option<&'static str> {
    if let Some(&(name, _)) = FIXTURES.get(index) {
        Some(name)
    } else {
        scalar_cases::NAMES
            .get(index.checked_sub(FIXTURES.len())?)
            .copied()
    }
}

pub fn case_hash(index: usize) -> Result<[u8; 32], String> {
    if let Some(&(name, input)) = FIXTURES.get(index) {
        let outcome = run(input, &CancelToken::new(), |_| {});
        if outcome.status != Status::Ok {
            return Err(format!("{name}: {}", outcome.document));
        }
        Ok(Sha256::digest(outcome.document.as_bytes()).into())
    } else {
        let scalar = index.checked_sub(FIXTURES.len()).ok_or("Índice inválido")?;
        scalar_cases::hash(scalar)
    }
}
