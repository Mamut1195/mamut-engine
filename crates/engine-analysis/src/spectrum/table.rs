use super::{invalid, validate_damping};
use crate::finite;
use engine_core::math;
use engine_core::{EResult, Real};

/// Unidades SI declaradas para TODOS los pares de entrada; sin conversión de g.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpectrumUnits {
    /// Abscisa T en s y aceleración espectral Sa en m/s².
    PeriodSecondsAccelerationMetersPerSecondSquared,
    /// Abscisa f en Hz y aceleración espectral Sa en m/s².
    FrequencyHertzAccelerationMetersPerSecondSquared,
}

/// Interpolación siempre en periodo T, también para entradas en frecuencia.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    /// Segmentos rectos en (T, Sa); admite T=0 y Sa=0.
    Linear,
    /// Segmentos rectos en (ln T, ln Sa); exige T>0 y Sa>0.
    LogLog,
}

/// Decisión obligatoria del usuario ante periodos fuera del intervalo tabulado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extrapolation {
    /// Fuera del intervalo cerrado devuelve ENG-E-0010.
    Error,
    /// Mesetas explícitas: Sa del primer/último punto al salir por cada extremo.
    Plateau,
}

/// Tabla del usuario para un único ζ declarado; no ajusta Sa por amortiguamiento.
///
/// Al menos dos pares finitos con abscisas estrictamente crecientes y Sa≥0.
/// Las frecuencias deben ser positivas; se invierten y reordenan a periodos.
/// No contiene espectros, factores normativos ni valores predeterminados.
#[derive(Clone, Debug, PartialEq)]
pub struct TabulatedSpectrum {
    points: Vec<(Real, Real)>,
    units: SpectrumUnits,
    damping: Real,
    interpolation: Interpolation,
    extrapolation: Extrapolation,
}

impl TabulatedSpectrum {
    /// Construye una tabla inmutable y validada en SI para ζ∈[0,1).
    /// ENG-E-0008 por no finitos/desbordamientos; ENG-E-0010 por tamaño,
    /// orden, signos o valores incompatibles con log-log. No ordena una tabla
    /// inválida ni interpola en frecuencia: f se convierte a T=1/f.
    pub fn new(
        pairs: &[(Real, Real)],
        units: SpectrumUnits,
        damping: Real,
        interpolation: Interpolation,
        extrapolation: Extrapolation,
    ) -> EResult<Self> {
        validate_damping(damping)?;
        if pairs.len() < 2 {
            return Err(invalid("Se requieren al menos dos pares espectrales."));
        }
        for &(x, sa) in pairs {
            finite(&[x, sa])?;
            if x < 0.0 || sa < 0.0 {
                return Err(invalid("Periodo/frecuencia y Sa deben ser no negativos."));
            }
        }
        if pairs.windows(2).any(|p| p[0].0 >= p[1].0) {
            return Err(invalid("Las abscisas deben ser estrictamente crecientes."));
        }
        let mut points = pairs.to_vec();
        if units == SpectrumUnits::FrequencyHertzAccelerationMetersPerSecondSquared {
            for (x, _) in &mut points {
                if *x == 0.0 {
                    return Err(invalid("La frecuencia tabulada debe ser positiva."));
                }
                *x = 1.0 / *x;
                finite(&[*x])?;
            }
            points.reverse();
        }
        if points.windows(2).any(|p| p[0].0 >= p[1].0) {
            return Err(invalid("Periodos convertidos no distinguibles."));
        }
        if interpolation == Interpolation::LogLog
            && points.iter().any(|&(t, sa)| t <= 0.0 || sa <= 0.0)
        {
            return Err(invalid("Log-log exige periodos y Sa positivos."));
        }
        Ok(Self {
            points,
            units,
            damping,
            interpolation,
            extrapolation,
        })
    }

    /// Pares internos (T en s, Sa en m/s²), en orden de periodo creciente.
    pub fn period_points(&self) -> &[(Real, Real)] {
        &self.points
    }

    /// Unidades declaradas de la tabla original.
    pub fn input_units(&self) -> SpectrumUnits {
        self.units
    }

    /// Amortiguamiento ζ adimensional de esta tabla, sin corrección automática.
    pub fn damping(&self) -> Real {
        self.damping
    }

    /// Interpolación elegida por el usuario.
    pub fn interpolation(&self) -> Interpolation {
        self.interpolation
    }

    /// Tratamiento explícito fuera de rango.
    pub fn extrapolation(&self) -> Extrapolation {
        self.extrapolation
    }

    /// Evalúa Sa(T,ζ_tabla) en m/s²; T en s, finito y no negativo.
    /// Conserva exactamente las ordenadas en puntos de tabla. Un T=0 fuera
    /// de una tabla log-log solo puede evaluarse con meseta explícita.
    pub fn acceleration(&self, period: Real) -> EResult<Real> {
        finite(&[period])?;
        if period < 0.0 {
            return Err(invalid("El periodo de consulta debe ser no negativo."));
        }
        // El constructor garantiza dos puntos y la tabla no permite mutaciones.
        let first = self.points[0];
        let last = self.points[self.points.len() - 1];
        if period < first.0 || period > last.0 {
            return match self.extrapolation {
                Extrapolation::Error => Err(invalid("Periodo fuera del espectro tabulado.")),
                Extrapolation::Plateau => Ok(if period < first.0 { first.1 } else { last.1 }),
            };
        }
        let upper = self.points.partition_point(|&(t, _)| t < period);
        let (t1, sa1) = self.points[upper];
        if t1 == period {
            return Ok(sa1);
        }
        let (t0, sa0) = self.points[upper - 1];
        let sa = match self.interpolation {
            Interpolation::Linear => {
                let a = (period - t0) / (t1 - t0);
                (1.0 - a) * sa0 + a * sa1
            }
            Interpolation::LogLog => {
                // ln_1p distingue periodos cercanos; la alternativa evita
                // desbordar el cociente para intervalos muy amplios.
                let a = log_distance(t0, period) / log_distance(t0, t1);
                math::exp((1.0 - a) * math::ln(sa0) + a * math::ln(sa1))
            }
        };
        finite(&[sa])?;
        Ok(sa)
    }
}

fn log_distance(low: Real, high: Real) -> Real {
    let relative = (high - low) / low;
    if relative.is_finite() {
        math::ln_1p(relative)
    } else {
        math::ln(high) - math::ln(low)
    }
}
