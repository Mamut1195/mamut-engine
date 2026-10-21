use super::{invalid, validate_damping};
use crate::finite;
use engine_core::math;
use engine_core::{EResult, EngineError, Real};

/// Combinación modal componente a componente, conservando signos antes de sumar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalCombination {
    /// √Σ R_n²; modos estadísticamente independientes.
    Srss,
    /// √Σ_iΣ_j ρ_ij R_i R_j con ζ de cada tabla modal.
    Cqc,
}

/// Combinación de envolventes direccionales con factores explícitos del usuario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionalCombination {
    /// √Σ_d (a_d R_d)².
    Srss,
    /// Σ_d a_d R_d. Es una envolvente ponderada, no un estado equilibrado.
    WeightedSum,
}

/// Correlación CQC Wilson–Der Kiureghian–Bayo (1981), fórmula de la petición.
///
/// r=ω_j/ω_i; numerador=8√(ζ_iζ_j)(ζ_i+rζ_j)r^(3/2);
/// denominador=(1−r²)²+4ζ_iζ_j r(1+r²)+4(ζ_i²+ζ_j²)r².
/// Reduce a la fórmula de ζ común. Se permutan i,j para usar r≤1 y evitar
/// desbordamientos; la expresión es simétrica. Devuelve 0≤ρ≤1.
/// Para frecuencias iguales y ζ_i=ζ_j (incluido cero), ρ=1 por convención;
/// con ambos ζ=0 y frecuencias distintas, ρ=0. Para frecuencias iguales y
/// ζ distintos, ρ=2√(ζ_iζ_j)/(ζ_i+ζ_j), cero si solo uno tiene ζ=0.
/// No finitos: ENG-E-0008; ω≤0 o ζ fuera de [0,1): ENG-E-0010.
pub fn cqc_correlation(omega_i: Real, omega_j: Real, zi: Real, zj: Real) -> EResult<Real> {
    validate_frequency(omega_i)?;
    validate_frequency(omega_j)?;
    validate_damping(zi)?;
    validate_damping(zj)?;
    if omega_i == omega_j {
        if zi == zj {
            return Ok(1.0);
        }
        let scale = zi.max(zj);
        let (a, b) = (zi / scale, zj / scale);
        return Ok(2.0 * math::sqrt(a) * math::sqrt(b) / (a + b));
    }
    let (high, low, zh, zl) = if omega_i > omega_j {
        (omega_i, omega_j, zi, zj)
    } else {
        (omega_j, omega_i, zj, zi)
    };
    let r = low / high;
    let root_product = math::sqrt(zh) * math::sqrt(zl);
    let denominator_root = math::hypot(
        math::hypot(
            1.0 - r * r,
            2.0 * root_product * math::sqrt(r * (1.0 + r * r)),
        ),
        2.0 * r * math::hypot(zh, zl),
    );
    let rho = 8.0
        * (root_product / denominator_root)
        * ((zh + r * zl) / denominator_root)
        * r
        * math::sqrt(r);
    finite(&[rho])?;
    // Solo redondeo en los límites matemáticos de una correlación.
    Ok(rho.clamp(0.0, 1.0))
}

pub(super) fn validate_frequency(omega: Real) -> EResult<()> {
    finite(&[omega])?;
    if omega <= 0.0 {
        return Err(invalid(
            "Se requieren frecuencias modales positivas; sin modos rígidos.",
        ));
    }
    Ok(())
}

pub(super) fn correlations(
    frequencies: &[Real],
    dampings: &[Real],
    method: ModalCombination,
) -> EResult<Vec<Vec<Real>>> {
    correlations_with_check(frequencies, dampings, method, &mut || Ok(()))
}

pub(super) fn correlations_with_check(
    frequencies: &[Real],
    dampings: &[Real],
    method: ModalCombination,
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<Vec<Vec<Real>>> {
    if frequencies.is_empty() || frequencies.len() != dampings.len() {
        return Err(invalid(
            "Modos vacíos o dimensiones de amortiguamiento incompatibles.",
        ));
    }
    for (&omega, &zeta) in frequencies.iter().zip(dampings) {
        validate_frequency(omega)?;
        validate_damping(zeta)?;
    }
    let mut rows = Vec::new();
    for (i, &omega) in frequencies.iter().enumerate() {
        check()?;
        let mut row = Vec::new();
        for (j, &other) in frequencies.iter().enumerate() {
            row.push(if i == j {
                1.0
            } else if method == ModalCombination::Srss {
                0.0
            } else {
                cqc_correlation(omega, other, dampings[i], dampings[j])?
            });
        }
        rows.push(row);
    }
    Ok(rows)
}

/// Combina un escalar firmado por modo (p. ej. una acción de miembro recuperada).
/// Dimensiones iguales no vacías; frecuencias en rad/s, ζ∈[0,1). Devuelve una
/// magnitud no negativa. Se recuperan acciones con `Element::recover` sobre
/// CADA desplazamiento modal antes de combinar; recuperar sobre la envolvente
/// combinada perdería los signos y las correlaciones de los modos.
pub fn combine_modal(
    values: &[Real],
    frequencies: &[Real],
    dampings: &[Real],
    method: ModalCombination,
) -> EResult<Real> {
    let rho = correlations(frequencies, dampings, method)?;
    if values.len() != frequencies.len() {
        return Err(invalid("Dimensiones de respuestas y modos incompatibles."));
    }
    quadratic(values, &rho)
}

pub(super) fn quadratic(values: &[Real], rho: &[Vec<Real>]) -> EResult<Real> {
    quadratic_with_check(values, rho, &mut || Ok(()))
}

pub(super) fn quadratic_with_check(
    values: &[Real],
    rho: &[Vec<Real>],
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<Real> {
    check()?;
    finite(values)?;
    let scale = values.iter().fold(0.0_f64, |s, v| s.max(v.abs()));
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut sum = 0.0;
    for (i, &a) in values.iter().enumerate() {
        check()?;
        for (j, &b) in values.iter().enumerate() {
            sum += rho[i][j] * (a / scale) * (b / scale);
        }
    }
    finite(&[sum])?;
    let tolerance = 64.0 * Real::EPSILON * math::powi(values.len() as Real, 2);
    if sum < -tolerance {
        return Err(EngineError::new(
            "ENG-E-0012",
            "CQC: forma cuadrática negativa.",
        ));
    }
    let result = scale * math::sqrt(sum.max(0.0));
    finite(&[result])?;
    Ok(result)
}

/// Combina escalares de direcciones con factores finitos no negativos.
/// ENG-E-0010 por listas vacías/incompatibles o factores negativos;
/// ENG-E-0008 por no finitos/desbordamiento. Ningún factor está incorporado.
pub fn combine_directions(
    values: &[Real],
    factors: &[Real],
    method: DirectionalCombination,
) -> EResult<Real> {
    if values.is_empty() || values.len() != factors.len() {
        return Err(invalid("Direcciones vacías o factores incompatibles."));
    }
    finite(values)?;
    finite(factors)?;
    if factors.iter().any(|&v| v < 0.0) {
        return Err(invalid(
            "Los factores direccionales deben ser no negativos.",
        ));
    }
    let mut result = 0.0_f64;
    for (&value, &factor) in values.iter().zip(factors) {
        let term = value * factor;
        finite(&[term])?;
        result = match method {
            DirectionalCombination::Srss => math::hypot(result, term),
            DirectionalCombination::WeightedSum => result + term,
        };
        finite(&[result])?;
    }
    Ok(result)
}
