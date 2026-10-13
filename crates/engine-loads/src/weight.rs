//! Peso propio con gravedad explícita, sin imponer un eje vertical al modelo.
use crate::{error, finite, length};
use engine_core::{EResult, Real};

fn properties(density: Real, area: Real, gravity: [Real; 3]) -> EResult<()> {
    finite(&[density, area])?;
    finite(&gravity)?;
    if density < 0.0 || area <= 0.0 {
        return Err(error("ENG-E-0010", "Se requiere rho ≥ 0 y A > 0."));
    }
    Ok(())
}

/// Fuerza total global rho·A·L·g en N; g se aporta en m/s².
/// Exige rho≥0, A>0, L>0 y magnitudes finitas; no aplica factores normativos.
/// ENG-E-0008 por desbordamiento de resultados/productos intermedios.
/// Para varios miembros, la fuerza total es la suma de sus vectores.
pub fn self_weight(
    density: Real,
    area: Real,
    member_length: Real,
    gravity: [Real; 3],
) -> EResult<[Real; 3]> {
    properties(density, area, gravity)?;
    length(member_length)?;
    let line_mass = density * area;
    let mass = line_mass * member_length;
    finite(&[line_mass, mass])?;
    let result = gravity.map(|g| mass * g);
    finite(&result)?;
    Ok(result)
}

/// Fuerza por unidad de longitud global rho·A·g en N/m.
/// Misma validación de propiedades que [`self_weight`]; permite gravedad nula
/// o fuera del plano XY. El consumidor 2D debe rechazar componentes Z no nulas.
pub fn self_weight_per_length(density: Real, area: Real, gravity: [Real; 3]) -> EResult<[Real; 3]> {
    properties(density, area, gravity)?;
    let line_mass = density * area;
    finite(&[line_mass])?;
    let result = gravity.map(|g| line_mass * g);
    finite(&result)?;
    Ok(result)
}
