//! Suite de benchmarks estructurales de derivación analítica propia.
//!
//! Crate de desarrollo: los fixtures y el ejecutor viven en pruebas; no añade
//! dependencias de producción ni API al motor. Alcance: estática lineal V1.
//! Datos, ecuaciones y tolerancias en `tests/support/table.rs` y en
//! `provenance/ENG-VAL-001.md` del repositorio. Sin E/S, hilos ni fuentes externas.
