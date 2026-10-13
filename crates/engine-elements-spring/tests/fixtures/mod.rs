//! Fixtures SI propios: equilibrio F=kδ y suma de energías elásticas.
pub const SOURCE: &str = "analítico: equilibrio y energía, derivación propia";
pub const CITATION: &str = "provenance/ENG-ELM-004.md, Derivaciones propias";
pub const K1: f64 = 120.0; // N/m.
pub const K2: f64 = 80.0; // N/m.
pub const FORCE: f64 = 12.0; // N.

pub fn cited() {
    assert!(!SOURCE.is_empty() && !CITATION.is_empty());
}
