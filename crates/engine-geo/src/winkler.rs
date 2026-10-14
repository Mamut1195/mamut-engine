//! Lecho bilateral lineal de Winkler (1867); contexto clásico: Hetényi (1946).
//! Derivación propia en provenance/ENG-GEO-002.md; k en N/m², reacción k·v en N/m.
use engine_core::{EResult, EngineError, Real};

fn finite(v: Real) -> EResult<Real> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            "Winkler: entrada o resultado no finito.",
        ))
    }
}
/// Valida rigidez distribuida k≥0 finita (N/m²).
/// ENG-E-0010 por k negativo; ENG-E-0008 por NaN/infinito.
pub fn validate(k: Real) -> EResult<()> {
    finite(k)?;
    if k < 0.0 {
        return Err(EngineError::new(
            "ENG-E-0010",
            "Winkler: k debe ser no negativo (N/m²).",
        ));
    }
    Ok(())
}
fn length(l: Real) -> EResult<()> {
    finite(l)?;
    if l <= 1e-12 {
        return Err(EngineError::new(
            "ENG-E-0005",
            "Winkler: L debe superar 1e-12 m.",
        ));
    }
    Ok(())
}
/// ∫ k NᵀN dx en `[v_i,v′_i,v_j,v′_j]`, con Hermite cúbica.
/// Simétrica definida positiva si k>0; cero si k=0. L en m.
/// Errores de k según [`validate`], ENG-E-0005 por L≤1e-12,
/// ENG-E-0008 por no finitos/desbordamiento y ENG-E-0010 por subdesbordamiento.
pub fn matrix(k: Real, l: Real) -> EResult<[[Real; 4]; 4]> {
    validate(k)?;
    length(l)?;
    if k == 0.0 {
        return Ok([[0.0; 4]; 4]);
    }
    let a = finite(k * (l / 420.0))?;
    let b = finite(a * l)?;
    let c = finite(b * l)?;
    if a == 0.0 || b == 0.0 || c == 0.0 {
        return Err(EngineError::new(
            "ENG-E-0010",
            "Winkler: coeficiente positivo subdesbordado.",
        ));
    }
    let out = [
        [156.0 * a, 22.0 * b, 54.0 * a, -13.0 * b],
        [22.0 * b, 4.0 * c, 13.0 * b, -3.0 * c],
        [54.0 * a, 13.0 * b, 156.0 * a, -22.0 * b],
        [-13.0 * b, -3.0 * c, -22.0 * b, 4.0 * c],
    ];
    for row in out {
        for v in row {
            finite(v)?;
        }
    }
    Ok(out)
}
/// Integrales `[∫₀ˣ k v(s) ds, ∫₀ˣ (x−s) k v(s) ds]`, N y N·m.
/// d=`[v_i,v′_i,v_j,v′_j]` en m/rad; posición x/L en `[0,1]`.
/// La fuerza del suelo sobre la viga es −k v; estas integrales usan k v.
/// ENG-E-0010 por posición fuera del tramo; los demás errores como [`matrix`].
pub fn integrals(k: Real, l: Real, d: [Real; 4], position: Real) -> EResult<[Real; 2]> {
    validate(k)?;
    length(l)?;
    finite(position)?;
    for v in d {
        finite(v)?;
    }
    if !(0.0..=1.0).contains(&position) {
        return Err(EngineError::new(
            "ENG-E-0010",
            "Winkler: posición fuera de [0,1].",
        ));
    }
    if k == 0.0 {
        return Ok([0.0; 2]);
    }
    let b = finite(l * d[1])?;
    let c = finite(l * d[3])?;
    let a = [
        d[0],
        b,
        finite(-3.0 * d[0] - 2.0 * b + 3.0 * d[2] - c)?,
        finite(2.0 * d[0] + b - 2.0 * d[2] + c)?,
    ];
    let mut force = 0.0;
    let mut moment = 0.0;
    let mut power = position;
    for (j, value) in a.into_iter().enumerate() {
        let n = (j + 1) as Real;
        force = finite(force + value * power / n)?;
        moment = finite(moment + value * power * position / (n * (n + 1.0)))?;
        power *= position;
    }
    let kl = finite(k * l)?;
    Ok([finite(kl * force)?, finite(finite(kl * l)? * moment)?])
}
