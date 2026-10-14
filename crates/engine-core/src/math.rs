//! Política escalar del motor: libm 0.2.16 (MIT), sin kernels por arquitectura.
//!
//! Las trascendentes usan la misma implementación Rust en nativo y wasm.
//! `sqrt` conserva std: IEEE-754 exige redondeo correcto (empate a par).
//! No se prometen redondeo correcto para las trascendentes ni bits de NaN
//! portables; los llamadores validan dominio y finitud con los errores del motor.

/// Evalúa `sin` con la implementación fijada de libm.
#[inline]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

/// Evalúa `cos` con la implementación fijada de libm.
#[inline]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

/// Evalúa `tan` con la implementación fijada de libm.
#[inline]
pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}

/// Evalúa `asin` con la implementación fijada de libm.
#[inline]
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}

/// Evalúa `acos` con la implementación fijada de libm.
#[inline]
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}

/// Evalúa `atan` con la implementación fijada de libm.
#[inline]
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}

/// Evalúa `sinh` con la implementación fijada de libm.
#[inline]
pub fn sinh(x: f64) -> f64 {
    libm::sinh(x)
}

/// Evalúa `cosh` con la implementación fijada de libm.
#[inline]
pub fn cosh(x: f64) -> f64 {
    libm::cosh(x)
}

/// Evalúa `tanh` con la implementación fijada de libm.
#[inline]
pub fn tanh(x: f64) -> f64 {
    libm::tanh(x)
}

/// Evalúa `asinh` con la implementación fijada de libm.
#[inline]
pub fn asinh(x: f64) -> f64 {
    libm::asinh(x)
}

/// Evalúa `acosh` con la implementación fijada de libm.
#[inline]
pub fn acosh(x: f64) -> f64 {
    libm::acosh(x)
}

/// Evalúa `atanh` con la implementación fijada de libm.
#[inline]
pub fn atanh(x: f64) -> f64 {
    libm::atanh(x)
}

/// Evalúa `exp` con la implementación fijada de libm.
#[inline]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// Evalúa `exp2` con la implementación fijada de libm.
#[inline]
pub fn exp2(x: f64) -> f64 {
    libm::exp2(x)
}

/// Evalúa `exp_m1` con la implementación fijada de libm.
#[inline]
pub fn exp_m1(x: f64) -> f64 {
    libm::expm1(x)
}

/// Evalúa `ln` con la implementación fijada de libm.
#[inline]
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}

/// Evalúa `ln_1p` con la implementación fijada de libm.
#[inline]
pub fn ln_1p(x: f64) -> f64 {
    libm::log1p(x)
}

/// Evalúa `log2` con la implementación fijada de libm.
#[inline]
pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}

/// Evalúa `log10` con la implementación fijada de libm.
#[inline]
pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}

/// Evalúa `cbrt` con la implementación fijada de libm.
#[inline]
pub fn cbrt(x: f64) -> f64 {
    libm::cbrt(x)
}

/// Raíz cuadrada IEEE-754 correctamente redondeada, usando std.
#[inline]
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// Seno y coseno; conserva el orden (seno, coseno) de la API std.
#[inline]
pub fn sin_cos(x: f64) -> (f64, f64) {
    libm::sincos(x)
}

/// Ángulo de (x, y), en radianes, con la implementación fijada de libm.
#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}

/// Norma euclídea sin formar x*x+y*y, protegiendo overflow/underflow.
#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}

/// Potencia con exponente real, con la implementación fijada de libm.
#[inline]
pub fn powf(x: f64, exponent: f64) -> f64 {
    libm::pow(x, exponent)
}

/// Potencia entera a través de libm::pow; todo i32 es exacto en binary64.
///
/// Se fija este algoritmo también para exponentes negativos y bases negativas;
/// puede redondear distinto del algoritmo de multiplicaciones de std::powi.
#[inline]
pub fn powi(x: f64, exponent: i32) -> f64 {
    libm::pow(x, f64::from(exponent))
}

/// Logaritmo en una base dada, con orden fijo de dos logaritmos y división.
#[inline]
pub fn log(x: f64, base: f64) -> f64 {
    ln(x) / ln(base)
}
