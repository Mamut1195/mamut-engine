//! Suite de benchmarks estructurales de derivación analítica propia.
//!
//! Crate de desarrollo: los fixtures y el ejecutor viven en pruebas; no añade
//! dependencias de producción ni API al motor. Alcance: estática lineal,
//! modal, P-Delta elástico y espectro (73 fixtures analíticos propios).
//! Datos y tolerancias en `tests/support/table.rs`, `dynamic_table.rs` y en
//! `provenance/ENG-VAL-001.md` del repositorio. Sin E/S, hilos ni fuentes externas.
