//! Interpolación elástica por tramos, sin historia ni dependencias externas.

use super::UniaxialFactory;
use crate::{finite, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, EngineError, Real};

/// Descriptor de la ley multilineal elástica, método 0.1.0.
pub const MULTILINEAR_ELASTIC: LawDescriptor = LawDescriptor {
    kind: "multilinear_elastic",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// Lados habilitados; ε positiva es tracción y ε negativa es compresión.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MultilinearMode {
    /// Curva completa, sin gap.
    #[default]
    Both,
    /// σ y tangente nulas para ε > 0.
    CompressionOnly,
    /// σ y tangente nulas para ε < 0.
    TensionOnly,
}

/// Parámetros SI de [`MultilinearElasticFactory`], validados al construir.
#[derive(Clone, Debug, PartialEq)]
pub struct MultilinearElasticParams {
    /// Puntos (ε adimensional, σ en Pa), ordenados e incluyendo (0, 0).
    pub points: Vec<(Real, Real)>,
    /// Lados habilitados; no cambia ni omite la validación de puntos.
    pub mode: MultilinearMode,
}

/// Fábrica tipada sin registro global, conforme a [`UniaxialFactory`].
#[derive(Clone, Copy, Debug, Default)]
pub struct MultilinearElasticFactory;

impl UniaxialFactory for MultilinearElasticFactory {
    type Params = MultilinearElasticParams;

    fn descriptor(&self) -> LawDescriptor {
        MULTILINEAR_ELASTIC
    }

    fn build(&self, params: Self::Params) -> EResult<Box<dyn UniaxialLaw>> {
        Ok(Box::new(MultilinearElastic::new(
            params.points,
            params.mode,
        )?))
    }
}

/// Curva σ(ε) elástica no lineal, continua y sin estado, definida por puntos.
///
/// Cada tramo usa mᵢ = (σᵢ₊₁ − σᵢ)/(εᵢ₊₁ − εᵢ) ≥ 0 y
/// σ = σᵢ + mᵢ(ε − εᵢ). Fuera de la tabla se prolonga el tramo extremo
/// correspondiente: primero en compresión, último en tracción. Se permite
/// el origen en un extremo (tabla unilateral); la misma regla prolonga el
/// otro lado, salvo que el modo lo inhabilite. Para simetría impar se dan
/// puntos reflejados (−ε, −σ); puntos independientes permiten asimetría.
///
/// La tangente en el interior es mᵢ. En un vértice se elige el tramo derecho,
/// salvo el último punto (último tramo) y el origen en modo solo-compresión
/// (tramo izquierdo si existe). `initial_tangent()` coincide con esta elección
/// en ε = 0. En el gap, σ y tangente son cero; el origen conserva la tangente
/// del lado activo. En vértices con pendientes distintas y en la apertura del
/// gap no existe derivada bilateral. El kit `check_uniaxial_law` acepta esta
/// convención con muestras cuyo intervalo ε ± h sea diferenciable, como exige
/// su contrato; los vértices se verifican por separado, sin FD central.
///
/// σ y tangente están en Pa; ε es total y adimensional. No hay precarga,
/// plasticidad ni daño, y tanto la historia inicial como la de prueba son vacías.
/// El rango numérico exige operaciones intermedias y resultados finitos.
///
/// ```
/// use engine_contracts::UniaxialLaw;
/// use engine_laws::uniaxial::{MultilinearElastic, MultilinearMode};
/// let law = MultilinearElastic::new(
///     vec![(-1.0, -2.0), (0.0, 0.0), (1.0, 4.0), (2.0, 6.0)],
///     MultilinearMode::Both,
/// )?;
/// let response = law.respond(1.5, &[])?;
/// assert_eq!((response.stress, response.tangent), (5.0, 2.0));
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MultilinearElastic {
    params: MultilinearElasticParams,
    slopes: Vec<Real>,
}

impl MultilinearElastic {
    /// Valida al menos dos puntos finitos, ε estrictamente creciente y (0, 0).
    /// No ordena, refleja ni corrige datos; se valida también el lado inactivo.
    ///
    /// # Errores
    /// `ENG-E-0008`: no finitos o desbordamiento en diferencias/pendientes.
    /// `ENG-E-0010`: menos de dos puntos, orden incorrecto, ausencia del origen,
    /// pendiente negativa o pendiente positiva reducida a cero por
    /// subdesbordamiento. Las mesetas exactas (σᵢ₊₁ = σᵢ) son válidas.
    pub fn new(points: Vec<(Real, Real)>, mode: MultilinearMode) -> EResult<Self> {
        if points.len() < 2 {
            return Err(invalid("La curva requiere al menos dos puntos."));
        }
        let mut has_origin = false;
        for (i, &(strain, stress)) in points.iter().enumerate() {
            finite(strain, &format!("ε del punto {i}"))?;
            finite(stress, &format!("σ del punto {i} (Pa)"))?;
            has_origin |= strain == 0.0 && stress == 0.0;
        }
        if !has_origin {
            return Err(invalid("La curva debe contener el punto (0, 0)."));
        }
        let mut slopes = Vec::with_capacity(points.len() - 1);
        for (i, pair) in points.windows(2).enumerate() {
            let ((e0, s0), (e1, s1)) = (pair[0], pair[1]);
            if e1 <= e0 {
                return Err(invalid(format!(
                    "Tramo {i}: ε debe ser estrictamente creciente."
                )));
            }
            let de = finite(e1 - e0, &format!("Δε del tramo {i}"))?;
            let ds = finite(s1 - s0, &format!("Δσ del tramo {i} (Pa)"))?;
            if ds < 0.0 {
                return Err(invalid(format!(
                    "Tramo {i}: pendiente negativa no admitida."
                )));
            }
            let slope = finite(ds / de, &format!("Pendiente del tramo {i} (Pa)"))?;
            if ds > 0.0 && slope == 0.0 {
                return Err(invalid(format!(
                    "Tramo {i}: pendiente positiva reducida a cero por subdesbordamiento."
                )));
            }
            slopes.push(slope);
        }
        Ok(Self {
            params: MultilinearElasticParams { points, mode },
            slopes,
        })
    }

    /// Puntos y modo originales validados, inmutables.
    pub fn params(&self) -> &MultilinearElasticParams {
        &self.params
    }

    fn segment(&self, strain: Real) -> usize {
        let left_at_origin = strain == 0.0 && self.params.mode == MultilinearMode::CompressionOnly;
        let upper = self.params.points.partition_point(|&(e, _)| {
            if left_at_origin {
                e < strain
            } else {
                e <= strain
            }
        });
        // El constructor garantiza al menos dos puntos y un tramo.
        upper.saturating_sub(1).min(self.slopes.len() - 1)
    }
}

impl UniaxialLaw for MultilinearElastic {
    fn kind(&self) -> &'static str {
        MULTILINEAR_ELASTIC.kind
    }

    fn contract(&self) -> ContractId {
        MULTILINEAR_ELASTIC.contract
    }

    fn state_len(&self) -> usize {
        0
    }

    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }

    fn initial_tangent(&self) -> Real {
        self.slopes[self.segment(0.0)]
    }

    /// Evalúa sin mutar entradas. `ENG-E-0010` por historia no vacía;
    /// `ENG-E-0008` por ε no finita o desbordamiento en resta, producto o suma
    /// de interpolación/extrapolación. Se usa el extremo más cercano para
    /// reducir cancelación. Se validan ε e historia incluso en el gap.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 0, "Historia aceptada")?;
        finite(strain, "ε")?;
        if (self.params.mode == MultilinearMode::CompressionOnly && strain > 0.0)
            || (self.params.mode == MultilinearMode::TensionOnly && strain < 0.0)
        {
            return Ok(UniaxialResponse {
                stress: 0.0,
                tangent: 0.0,
                trial_state: Vec::new(),
            });
        }
        let i = self.segment(strain);
        let (e0, s0) = self.params.points[i];
        let (e1, s1) = self.params.points[i + 1];
        let tangent = self.slopes[i];
        let stress = if strain == e0 || tangent == 0.0 {
            s0
        } else if strain == e1 {
            s1
        } else {
            let midpoint = e0 * 0.5 + e1 * 0.5;
            let (anchor_strain, anchor_stress) = if strain <= midpoint {
                (e0, s0)
            } else {
                (e1, s1)
            };
            let delta = finite(strain - anchor_strain, "Δε de evaluación")?;
            let increment = finite(tangent * delta, "Incremento de σ (Pa)")?;
            finite(anchor_stress + increment, "σ interpolada/extrapolada (Pa)")?
        };
        Ok(UniaxialResponse {
            stress,
            tangent,
            trial_state: Vec::new(),
        })
    }
}

fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
