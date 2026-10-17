#![forbid(unsafe_code)]
//! Suite de benchmarks estructurales de derivación analítica propia.
//!
//! Crate de desarrollo: `fixtures` expone el contrato fixture/1 y su runner
//! portable vía engine-api. Las suites históricas viven en pruebas. Alcance: estática lineal,
//! modal, P-Delta elástico y espectro (73 fixtures analíticos propios).
//! ENG-GEO-002 añade seis fixtures de lecho Winkler en 2D/3D, ejecutados
//! en nativo y wasm; datos en `tests/support/winkler_table.rs`.
//! Datos y tolerancias en `tests/support/table.rs`, `dynamic_table.rs` y en
//! `provenance/ENG-VAL-001.md` del repositorio. Sin E/S, hilos ni fuentes externas.

pub mod fixtures;
