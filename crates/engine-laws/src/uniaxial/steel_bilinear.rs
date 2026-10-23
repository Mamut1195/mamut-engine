//! Plasticidad 1D con endurecimiento cinemático lineal; derivación propia.

use super::UniaxialFactory;
use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, EngineError, Real};

/// Descriptor del acero bilineal cinemático, método 0.1.0.
pub const STEEL_BILINEAR_KINEMATIC: LawDescriptor = LawDescriptor {
    kind: "steel.bilinear_kinematic",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// Parámetros SI de [`SteelBilinearKinematicFactory`], validados al construir.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SteelBilinearKinematicParams {
    /// Módulo elástico E en Pa, finito y estrictamente positivo.
    pub e: Real,
    /// Tensión de fluencia inicial fy en Pa, finita y estrictamente positiva.
    pub fy: Real,
    /// Razón adimensional de la pendiente posfluencia respecto de E: 0 ≤ b < 1.
    /// El módulo cinemático es H = bE/(1−b); b = 0 permite plasticidad perfecta.
    pub b: Real,
}

/// Fábrica tipada sin registro global, conforme a [`UniaxialFactory`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SteelBilinearKinematicFactory;

impl UniaxialFactory for SteelBilinearKinematicFactory {
    type Params = SteelBilinearKinematicParams;

    fn descriptor(&self) -> LawDescriptor {
        STEEL_BILINEAR_KINEMATIC
    }

    fn build(&self, params: Self::Params) -> EResult<Box<dyn UniaxialLaw>> {
        Ok(Box::new(SteelBilinearKinematic::new(
            params.e, params.fy, params.b,
        )?))
    }
}

/// Acero bilineal: retorno implícito exacto en 1D con endurecimiento cinemático.
///
/// ε es la deformación TOTAL adimensional, positiva en tracción; E, fy, σ,
/// α y la tangente están en Pa. La historia plana es `[ε_p, α]`, inicialmente
/// `[0, 0]`: deformación plástica y retroceso (back-stress). El llamador es dueño
/// del estado aceptado; `respond` devuelve una prueba independiente y no acepta
/// ni modifica historia. No hay daño, endurecimiento isotrópico ni acumulados.
///
/// Se define H = bE/(1−b), de modo que la pendiente posfluencia es bE.
/// Con la historia aceptada: σ_tr = E(ε−ε_p), ξ = σ_tr−α y f = |ξ|−fy.
/// Si f ≤ 0, la respuesta es elástica y su tangente es E. Si f > 0, s = sign(ξ),
/// Δγ = f/(E+H), ε_p' = ε_p + sΔγ, α' = α + sHΔγ y
/// σ = σ_tr−sEΔγ = α'+s fy. La tangente consistente es EH/(E+H) = bE.
/// La superficie |σ−α| ≤ fy se traslada: tras invertir, el recorrido elástico
/// en tensión siempre mide 2fy (en deformación, 2fy/E), efecto Bauschinger.
///
/// Se evalúan EΔγ = (1−b)f y HΔγ = bf, sin formar H ni E+H. Se obtiene
/// Δγ dividiendo EΔγ entre E y σ desde la superficie retornada, evitando
/// restar tensiones grandes para obtener una meseta pequeña. El rango numérico
/// exige diferencias, incrementos y resultados finitos; su desbordamiento se
/// diagnostica. La precisión de la historia está limitada por `Real` (f64).
/// En f = 0 se elige la rama elástica; allí no hay derivada bilateral única.
///
/// ```
/// use engine_contracts::UniaxialLaw;
/// use engine_laws::uniaxial::SteelBilinearKinematic;
/// let law = SteelBilinearKinematic::new(200.0, 1.0, 0.1)?;
/// let mut committed = law.initial_state();
/// let trial = law.respond(0.02, &committed)?;
/// assert!((trial.stress - 1.3).abs() < 1e-12);
/// committed = trial.trial_state; // aceptación explícita por el llamador
/// let unload = law.respond(0.015, &committed)?;
/// assert!((unload.stress - 0.3).abs() < 1e-12);
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SteelBilinearKinematic {
    params: SteelBilinearKinematicParams,
}

impl SteelBilinearKinematic {
    /// Construye con E y fy en Pa y b = pendiente posfluencia/E.
    ///
    /// # Errores
    /// `ENG-E-0008` por parámetros no finitos; `ENG-E-0010` por E ≤ 0,
    /// fy ≤ 0 o b fuera de [0, 1). No se necesita que H sea representable.
    pub fn new(e: Real, fy: Real, b: Real) -> EResult<Self> {
        positive(e, "E (Pa)")?;
        positive(fy, "fy (Pa)")?;
        finite(b, "b")?;
        if !(0.0..1.0).contains(&b) {
            return Err(EngineError::new("ENG-E-0010", "b must be in [0, 1)."));
        }
        Ok(Self {
            params: SteelBilinearKinematicParams { e, fy, b },
        })
    }

    /// Parámetros originales validados, inmutables.
    pub fn params(&self) -> &SteelBilinearKinematicParams {
        &self.params
    }
}

impl UniaxialLaw for SteelBilinearKinematic {
    fn kind(&self) -> &'static str {
        STEEL_BILINEAR_KINEMATIC.kind
    }

    fn contract(&self) -> ContractId {
        STEEL_BILINEAR_KINEMATIC.contract
    }

    fn state_len(&self) -> usize {
        2
    }

    fn initial_state(&self) -> Vec<Real> {
        vec![0.0, 0.0]
    }

    fn initial_tangent(&self) -> Real {
        self.params.e
    }

    /// Evalúa sin mutar la ley ni aceptar historia.
    ///
    /// `ENG-E-0010` por longitud distinta de dos; `ENG-E-0008` por ε o historia
    /// no finitas, o desbordamiento de diferencias, incrementos o resultados.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 2, "Accepted history [ε_p, α]")?;
        finite(strain, "ε")?;
        let SteelBilinearKinematicParams { e, fy, b } = self.params;
        // La longitud se ha validado antes de acceder a ambos componentes.
        let (plastic_strain, alpha) = (committed[0], committed[1]);
        let elastic_strain = finite(strain - plastic_strain, "ε−ε_p")?;
        let stress_trial = finite(e * elastic_strain, "σ_tr (Pa)")?;
        let xi = finite(stress_trial - alpha, "ξ (Pa)")?;
        let f = xi.abs() - fy;
        if f <= 0.0 {
            return Ok(UniaxialResponse {
                stress: stress_trial,
                tangent: e,
                trial_state: committed.to_vec(),
            });
        }
        let sign = xi.signum();
        let stress_correction = (1.0 - b) * f;
        let gamma = finite(stress_correction / e, "Δγ")?;
        let plastic_trial = finite(plastic_strain + sign * gamma, "Trial ε_p")?;
        let alpha_trial = finite(alpha + sign * (b * f), "Trial α (Pa)")?;
        let stress = finite(alpha_trial + sign * fy, "σ (Pa)")?;
        Ok(UniaxialResponse {
            stress,
            tangent: b * e,
            trial_state: vec![plastic_trial, alpha_trial],
        })
    }
}
