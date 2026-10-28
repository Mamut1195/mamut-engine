//! In-house cases: binary64 bit hash, without JSON or tolerances.
use engine_core::math;
use engine_loads::Beam2D;

pub const NAMES: &[&str] = &[
    "scalar-trig",
    "scalar-exp-log",
    "scalar-powers-norm",
    "load-rotation-bits",
    "scalar-sin-cos",
];

pub fn hash(index: usize) -> Result<[u8; 32], String> {
    let mut values = Vec::new();
    match index {
        0 => {
            for x in [-1.0_f64, -0.375, -0.0, 0.0, 1e-20, 0.375, 1.0] {
                let (sin, cos) = math::sin_cos(x);
                values.extend([
                    math::sin(x),
                    math::cos(x),
                    sin,
                    cos,
                    math::tan(x),
                    math::asin(x),
                    math::acos(x),
                    math::atan(x),
                    math::sinh(x),
                    math::cosh(x),
                    math::tanh(x),
                    math::asinh(x),
                    math::acosh(1.0 + x.abs()),
                    math::atanh(x * 0.5),
                ]);
            }
            for x in [1e20, -1e300, std::f64::consts::FRAC_PI_2] {
                let (sin, cos) = math::sin_cos(x);
                values.extend([math::sin(x), math::cos(x), sin, cos, math::tan(x)]);
            }
            for (y, x) in [(-0.0, -1.0), (0.0, -1.0), (-0.75, 0.375), (1e-300, 1e300)] {
                values.push(math::atan2(y, x));
            }
        }
        1 => {
            for x in [-700.0, -1e-20, -0.0, 0.0, 1e-20, 0.375, 700.0] {
                values.extend([math::exp(x), math::exp_m1(x), math::exp2(x)]);
            }
            for x in [
                f64::from_bits(1),
                f64::MIN_POSITIVE,
                0.375,
                1.0,
                1.0 + f64::EPSILON,
                1e300,
            ] {
                values.extend([
                    math::ln(x),
                    math::log2(x),
                    math::log10(x),
                    math::log(x, 3.0),
                ]);
            }
            for x in [-0.75, -1e-20, -0.0, 0.0, 1e-20, 0.375, 1e300] {
                values.push(math::ln_1p(x));
            }
        }
        2 => {
            values.push(math::sqrt(-0.0));
            for x in [
                -1e300,
                -0.375,
                -0.0,
                0.0,
                f64::from_bits(1),
                1e-300,
                0.375,
                1e300,
            ] {
                values.extend([math::cbrt(x), math::sqrt(x.abs()), math::hypot(x, x * 0.5)]);
            }
            for (x, exponent) in [(0.375, 1.5), (1e-200, 0.25), (1e200, -0.25), (-0.375, 3.0)] {
                values.push(math::powf(x, exponent));
            }
            for (x, exponent) in [
                (-0.375, 3),
                (-0.375, -3),
                (-0.0, 3),
                (0.0, 0),
                (-1.0, i32::MIN),
                (-1.0, i32::MAX),
            ] {
                values.push(math::powi(x, exponent));
            }
        }
        3 => {
            // Exercises sin_cos through a public engine path.
            for angle in [-0.375, 0.0, 0.375, 1e20] {
                let beam = Beam2D::new(3.75, angle).map_err(|e| e.to_string())?;
                values.extend(
                    beam.forces_to_global([1.25, -2.75, 0.5, -1.25, 2.75, -0.5])
                        .map_err(|e| e.to_string())?,
                );
            }
        }
        4 => {
            // Explicit (sine, cosine) pair: signed zeros, subnormals, and
            // argument reduction near quadrants and for large angles.
            for x in [
                -1e300,
                -1e20,
                -std::f64::consts::PI,
                -0.375,
                -f64::from_bits(1),
                -0.0,
                0.0,
                f64::from_bits(1),
                1e-20,
                0.375,
                f64::from_bits(std::f64::consts::FRAC_PI_2.to_bits() - 1),
                std::f64::consts::FRAC_PI_2,
                f64::from_bits(std::f64::consts::FRAC_PI_2.to_bits() + 1),
                std::f64::consts::PI,
                1e20,
                1e300,
            ] {
                let (sin, cos) = math::sin_cos(x);
                values.extend([sin, cos]);
            }
        }
        _ => return Err("Índice escalar inválido".into()),
    }
    if values.is_empty() || values.iter().any(|x| !x.is_finite()) {
        return Err("Caso escalar vacío o no finito".into());
    }
    let mut bytes = Vec::with_capacity(values.len() * 8);
    for x in values {
        bytes.extend_from_slice(&x.to_bits().to_be_bytes());
    }
    Ok(super::sha256(&bytes))
}
