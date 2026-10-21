//! Engine scalar policy: libm 0.2.16 (MIT), without architecture-specific kernels.
//!
//! Transcendental functions use the same Rust implementation on native and wasm.
//! `sqrt` retains std: IEEE-754 requires correct rounding (ties to even).
//! Correct rounding for transcendental functions and portable NaN bits are not
//! guaranteed; callers validate domain and finiteness using engine errors.

/// Evaluates `sin` with the pinned libm implementation.
#[inline]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

/// Evaluates `cos` with the pinned libm implementation.
#[inline]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

/// Evaluates `tan` with the pinned libm implementation.
#[inline]
pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}

/// Evaluates `asin` with the pinned libm implementation.
#[inline]
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}

/// Evaluates `acos` with the pinned libm implementation.
#[inline]
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}

/// Evaluates `atan` with the pinned libm implementation.
#[inline]
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}

/// Evaluates `sinh` with the pinned libm implementation.
#[inline]
pub fn sinh(x: f64) -> f64 {
    libm::sinh(x)
}

/// Evaluates `cosh` with the pinned libm implementation.
#[inline]
pub fn cosh(x: f64) -> f64 {
    libm::cosh(x)
}

/// Evaluates `tanh` with the pinned libm implementation.
#[inline]
pub fn tanh(x: f64) -> f64 {
    libm::tanh(x)
}

/// Evaluates `asinh` with the pinned libm implementation.
#[inline]
pub fn asinh(x: f64) -> f64 {
    libm::asinh(x)
}

/// Evaluates `acosh` with the pinned libm implementation.
#[inline]
pub fn acosh(x: f64) -> f64 {
    libm::acosh(x)
}

/// Evaluates `atanh` with the pinned libm implementation.
#[inline]
pub fn atanh(x: f64) -> f64 {
    libm::atanh(x)
}

/// Evaluates `exp` with the pinned libm implementation.
#[inline]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// Evaluates `exp2` with the pinned libm implementation.
#[inline]
pub fn exp2(x: f64) -> f64 {
    libm::exp2(x)
}

/// Evaluates `exp_m1` with the pinned libm implementation.
#[inline]
pub fn exp_m1(x: f64) -> f64 {
    libm::expm1(x)
}

/// Evaluates `ln` with the pinned libm implementation.
#[inline]
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}

/// Evaluates `ln_1p` with the pinned libm implementation.
#[inline]
pub fn ln_1p(x: f64) -> f64 {
    libm::log1p(x)
}

/// Evaluates `log2` with the pinned libm implementation.
#[inline]
pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}

/// Evaluates `log10` with the pinned libm implementation.
#[inline]
pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}

/// Evaluates `cbrt` with the pinned libm implementation.
#[inline]
pub fn cbrt(x: f64) -> f64 {
    libm::cbrt(x)
}

/// Correctly rounded IEEE-754 square root, using std.
#[inline]
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// Sine and cosine; preserves the std API order (sine, cosine).
#[inline]
pub fn sin_cos(x: f64) -> (f64, f64) {
    libm::sincos(x)
}

/// Angle of (x, y), in radians, with the pinned libm implementation.
#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}

/// Euclidean norm without forming x*x+y*y, guarding against overflow/underflow.
#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}

/// Power with a real exponent, with the pinned libm implementation.
#[inline]
pub fn powf(x: f64, exponent: f64) -> f64 {
    libm::pow(x, exponent)
}

/// Integer power through libm::pow; every i32 is exact in binary64.
///
/// This procedure is also pinned for negative exponents and negative bases;
/// its rounding may differ from std::powi's multiplication procedure.
#[inline]
pub fn powi(x: f64, exponent: i32) -> f64 {
    libm::pow(x, f64::from(exponent))
}

/// Logarithm in a given base, with a fixed order of two logarithms and division.
#[inline]
pub fn log(x: f64, base: f64) -> f64 {
    ln(x) / ln(base)
}
