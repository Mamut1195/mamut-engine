//! Fixtures derivados a mano en provenance/ENG-GEO-006.md; sin datos de motores.
use engine_geo::consolidation::Drainage;
/// Límites existentes: errores normalizados de asiento/presión y relativos de U/flujo.
#[derive(Clone, Copy)]
pub struct Tolerance {
    pub settlement_normalized: f64,
    pub degree_relative: f64,
    pub pressure_normalized: f64,
    pub flux_relative: f64,
    pub justification: &'static str,
}
pub const TOLERANCE: Tolerance = Tolerance {
    settlement_normalized: 0.0015,
    degree_relative: 0.005,
    pressure_normalized: 0.004,
    flux_relative: 0.03,
    justification: "Límites originales de malla/paso: asiento normalizado por s∞, presión por 100 kPa; U y flujo relativos a la serie independiente. provenance/ENG-GEO-006.md.",
};
#[derive(Clone, Copy)]
pub struct Fixture {
    pub name: &'static str,
    pub source: &'static str,
    pub citation: &'static str,
    pub drainage: Drainage,
    pub transformed_layers: bool,
    pub ramp: bool,
    pub unload: bool,
    pub tolerance: Tolerance,
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
        tolerance: TOLERANCE,
        source: "Separación de variables y convolución propias; transformación multicapa x=∫dz/√cv, mv√cv constante. provenance/ENG-GEO-006.md#serie-de-terzaghi-derivación-y-fixtures.",
        citation: if transformed_layers {
            "Derivación propia: [transformación multicapa y flujo](provenance/ENG-GEO-006.md#multicapa-con-flujo-no-nulo-derivación-propia)."
        } else {
            "Derivación propia: [serie por separación de variables y convolución de rampas](provenance/ENG-GEO-006.md#serie-de-terzaghi-derivación-y-fixtures)."
        },
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
    "Autofunciones multicapa derivadas a mano, continuidad de u y k*u_z; provenance/ENG-GEO-006.md#revisión-j4-r32-multicapa-no-transformable-por-autofunciones. Sin discretización FE en el oráculo.";
pub const MULTILAYER_CITATION: &str =
    "Derivación propia: [autofunciones multicapa, proyección y continuidad de presión/flujo](provenance/ENG-GEO-006.md#revisión-j4-r32-multicapa-no-transformable-por-autofunciones).";
// (H [m], cv [m²/s], mv [1/kPa], k [m/s]); gamma_w=10 kN/m³.
pub const MULTILAYER: [(f64, f64, f64, f64); 3] = [
    (0.5, 0.25, 0.002, 0.005),
    (1.5, 2.25, 0.001, 0.0225),
    (1.0, 1.0, 0.003, 0.03),
];
