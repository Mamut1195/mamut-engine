//! In-house SI fixtures from strength of materials and classical structural analysis.
pub const SOURCE: &str = "analítico: integración propia de EI v''''=q y equilibrio del pórtico";
pub const CITATION: &str = "provenance/ENG-ELM-002.md, Derivaciones propias; cita bibliográfica exacta de resistencia de materiales/análisis estructural clásico PENDIENTE";
pub const E: f64 = 200.0;
pub const A: f64 = 2.0;
pub const I: f64 = 3.0;
pub const RHO: f64 = 3.0;
pub const L: f64 = 4.0;
pub const W: f64 = 6.0;
pub const P: f64 = 12.0;

pub struct Fixture {
    pub id: &'static str,
    pub source: &'static str,
    pub citation: &'static str,
    pub displacement_tolerance: f64,
    pub force_tolerance: f64,
}
pub const STATIC: Fixture = Fixture {
    id: "frame2d.static.closed_forms",
    source: SOURCE,
    citation: CITATION,
    displacement_tolerance: 1e-12, // absolute m/rad, small SI fixtures.
    force_tolerance: 1e-9,         // absolute N and N·m, including equilibrium.
};
pub const BUCKLING: Fixture = Fixture {
    id: "frame2d.euler.pinned",
    source: "analítico: EI v''+P v=0, v(0)=v(L)=0 => Pcr=pi²EI/L²",
    citation: CITATION,
    displacement_tolerance: 1e-12,
    force_tolerance: 1e-9,
};
pub fn cited(f: &Fixture) {
    assert!(!f.id.is_empty() && !f.source.is_empty() && !f.citation.is_empty());
}
