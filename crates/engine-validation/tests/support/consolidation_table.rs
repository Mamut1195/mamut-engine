//! Fixtures derivados a mano en provenance/ENG-GEO-006.md; sin datos de motores.
use engine_geo::consolidation::Drainage;
#[derive(Clone, Copy)]
pub struct Fixture {
    pub name: &'static str,
    pub source: &'static str,
    pub citation: &'static str,
    pub drainage: Drainage,
    pub transformed_layers: bool,
    pub ramp: bool,
    pub unload: bool,
}
const fn row(
    name: &'static str,
    drainage: Drainage,
    transformed_layers: bool,
    ramp: bool,
    unload: bool,
) -> Fixture {
    Fixture {
        name, drainage, transformed_layers, ramp, unload,
        source: "Separación de variables y convolución propias; transformación multicapa x=∫dz/√cv, mv√cv constante. provenance/ENG-GEO-006.md.",
        citation: "K. Terzaghi (1943), Theoretical Soil Mechanics, primera edición, John Wiley & Sons, New York: teoría de consolidación unidimensional. Referencia conceptual; no se transcribe un ejemplo del libro.",
    }
}
pub const FIXTURES: &[Fixture] = &[
    row("Terzaghi-dren-superior", Drainage::Top, false, false, false),
    row(
        "Terzaghi-dren-inferior",
        Drainage::Bottom,
        false,
        false,
        false,
    ),
    row(
        "Terzaghi-doble-drenaje",
        Drainage::Double,
        false,
        false,
        false,
    ),
    row("Terzaghi-rampa", Drainage::Top, false, true, false),
    row(
        "Terzaghi-rampa-descarga",
        Drainage::Bottom,
        false,
        true,
        true,
    ),
    row(
        "multicapa-flujo-interfaz",
        Drainage::Top,
        true,
        false,
        false,
    ),
];

// Independent spectral fixture; all three layers have H/sqrt(cv)=1 but
// different mv*sqrt(cv), so the interface conditions remain heterogeneous.
pub const MULTILAYER_SOURCE: &str =
    "Autofunciones multicapa derivadas a mano, continuidad de u y k*u_z; provenance/ENG-GEO-006.md (revisión r32). Sin discretización FE en el oráculo.";
pub const MULTILAYER_CITATION: &str =
    "H. S. Carslaw y J. C. Jaeger (1959), Conduction of Heat in Solids, segunda edición, Oxford University Press: difusión en medios compuestos y separación de variables. Referencia general; fixture y derivación propios.";
// (H [m], cv [m²/s], mv [1/kPa], k [m/s]); gamma_w=10 kN/m³.
pub const MULTILAYER: [(f64, f64, f64, f64); 3] = [
    (0.5, 0.25, 0.002, 0.005),
    (1.5, 2.25, 0.001, 0.0225),
    (1.0, 1.0, 0.003, 0.03),
];
