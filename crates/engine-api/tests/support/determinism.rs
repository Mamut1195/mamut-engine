//! Entradas SI propias, compartidas por repetibilidad y cruce nativo/wasm.
pub mod scalar_cases;

use engine_api::{canonical_json, run, CancelToken, Status};

// El SDK hashea los bytes sin canonicalizarlos. Decodificar hex conserva el
// mismo digest de 32 bytes que usaba sha2, incluido el ABI de prueba wasm.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let hex = mamut_canon::sha256_hex(bytes);
    std::array::from_fn(|i| {
        u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).expect("SHA-256 hexadecimal del SDK")
    })
}

// target es procedencia específica de la compilación: se valida antes de
// excluirlo exclusivamente del cruce. Los demás campos conservan sus bytes.
pub fn comparison_document(document: &str) -> Result<String, String> {
    let mut value: serde_json::Value = serde_json::from_str(document).map_err(|e| e.to_string())?;
    if value["target"] != env!("MAMUT_ENGINE_TARGET") {
        return Err("target no corresponde a la compilación".into());
    }
    value
        .as_object_mut()
        .ok_or("results/1 no es objeto")?
        .remove("target");
    canonical_json(&value.to_string()).map_err(|e| e.to_string())
}

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
        Ok(sha256(comparison_document(&outcome.document)?.as_bytes()))
    } else {
        let scalar = index.checked_sub(FIXTURES.len()).ok_or("Índice inválido")?;
        scalar_cases::hash(scalar)
    }
}
