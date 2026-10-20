//! Fixtures portables: misma tabla analítica en engine-geo, validación y wasm.
use engine_core::{EResult, EngineError};
use engine_geo::{qz, tz};

const SOURCE: &str = "API RP 2GEO, Geotechnical and Foundation Design Considerations, 1.ª ed. abril 2011, Addendum 1 octubre 2014, reafirmada 2021";
const TZ_CITATION: &str =
    "§8.4.2, figura 2 y tabla 2; interpolación lineal y tracción: derivación propia ENG-GEO-004";
const QZ_CITATION: &str =
    "§8.4.3, figura 3; interpolación lineal y punta sin tracción: derivación propia ENG-GEO-004";
// Tolerancia solo de redondeo binary64: no hay discretización ni calibración.
const RELATIVE_TOLERANCE: f64 = 2e-14;

fn require(ok: bool, message: &str) -> EResult<()> {
    if ok {
        Ok(())
    } else {
        Err(EngineError::new("ENG-E-0012", message))
    }
}

fn check(actual: f64, expected: f64, citation: &str) -> EResult<()> {
    require(
        actual.is_finite()
            && (actual - expected).abs() <= RELATIVE_TOLERANCE * expected.abs().max(1.0),
        &format!("Obtenido {actual}, esperado {expected}; source={SOURCE}; citation={citation}"),
    )
}

fn params() -> tz::ApiTzParams {
    tz::ApiTzParams {
        tmax_compression_pa: Some(50_000.0),
        tmax_tension_pa: Some(30_000.0),
        zpeak_m: Some(0.01),
        plateau_z_m: Some(0.03),
    }
}

/// Cinco fixtures: arena, arcilla, punta API y dos tablas de usuario.
pub fn run_all() -> EResult<usize> {
    let sand = tz::api_sand(params())?;
    let clay = tz::api_clay(params(), Some(0.8))?;
    // Vértices esperados expresados directamente en SI, no desde el generador.
    let ascending = [
        (0.0, 0.0),
        (0.0016, 15_000.0),
        (0.0031, 25_000.0),
        (0.0057, 37_500.0),
        (0.008, 45_000.0),
        (0.01, 50_000.0),
    ];
    require(
        sand.points().len() == 13 && clay.points().len() == 15,
        "Número de vértices t-z",
    )?;
    for curve in [&sand, &clay] {
        for (i, &(z, t)) in ascending.iter().enumerate() {
            let vertex = curve.points()[curve.points().len() / 2 + i];
            check(vertex.0, z, TZ_CITATION)?;
            require(vertex.1 == t, "Ordenada exacta del vértice t-z")?;
            require(curve.at(vertex.0)? == t, "Evaluación exacta en vértice t-z")?;
            check(curve.at(z)?, t, TZ_CITATION)?;
            check(curve.at(-z)?, -t * 0.6, TZ_CITATION)?;
        }
    }
    // Medio de (0.0016,15000)-(0.0031,25000): (0.00235,20000).
    check(sand.at(0.00235)?, 20_000.0, TZ_CITATION)?;
    check(sand.at(-0.00235)?, -12_000.0, TZ_CITATION)?;
    check(sand.at(0.03)?, 50_000.0, TZ_CITATION)?;
    check(sand.at(1.0)?, 50_000.0, TZ_CITATION)?;
    check(sand.at(-1.0)?, -30_000.0, TZ_CITATION)?;
    // Medio pico-residual: (50000+40000)/2 = 45000 Pa.
    check(clay.at(0.015)?, 45_000.0, TZ_CITATION)?;
    check(clay.at(0.02)?, 40_000.0, TZ_CITATION)?;
    check(clay.at(0.03)?, 40_000.0, TZ_CITATION)?;
    check(clay.at(-0.015)?, -27_000.0, TZ_CITATION)?;
    check(clay.at(-1.0)?, -24_000.0, TZ_CITATION)?;
    check(clay.at(1.0)?, 40_000.0, TZ_CITATION)?;

    let tip = qz::api(qz::ApiQzParams {
        qp_n: Some(1_000_000.0),
        diameter_m: Some(1.0),
        plateau_z_m: Some(0.2),
    })?;
    let tip_vertices = [
        (-0.2, 0.0),
        (0.0, 0.0),
        (0.002, 250_000.0),
        (0.013, 500_000.0),
        (0.042, 750_000.0),
        (0.073, 900_000.0),
        (0.1, 1_000_000.0),
        (0.2, 1_000_000.0),
    ];
    require(tip.points() == tip_vertices, "Vértices q-z exactos")?;
    for &(z, q) in &tip_vertices {
        require(tip.at(z)? == q, "Evaluación exacta en vértice q-z")?;
    }
    // Medio de (0.002,250000)-(0.013,500000): Q = 375000 N.
    check(tip.at(0.0075)?, 375_000.0, QZ_CITATION)?;
    check(tip.at(1.0)?, 1_000_000.0, QZ_CITATION)?;
    for z in [-1.0, -0.1, -0.001] {
        check(tip.at(z)?, 0.0, QZ_CITATION)?;
    }

    // Derivación propia: interpolar una tabla artificial con ablandamiento.
    let custom = tz::user(
        Some(vec![(-0.02, -8.0), (0.0, 0.0), (0.01, 20.0), (0.02, 10.0)]),
        tz::UserTraction::Explicit,
    )?;
    check(custom.at(0.015)?, 15.0, "Derivación propia: (20+10)/2")?;
    check(custom.at(-0.01)?, -4.0, "Derivación propia: (-8+0)/2")?;
    let symmetric = qz::user(
        Some(vec![(0.0, 0.0), (0.01, 100.0)]),
        tz::UserTraction::Symmetric,
    )?;
    check(
        symmetric.at(-0.005)?,
        -50.0,
        "Derivación propia: reflexión e interpolación de tabla artificial",
    )?;
    Ok(5)
}
