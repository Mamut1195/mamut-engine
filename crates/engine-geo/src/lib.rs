#![forbid(unsafe_code)]
//! Geotecnia: perfiles estratificados y consolidación incremental, desde cero.
//!
//! [`consolidation`] resuelve Terzaghi 1D multicapa por elementos finitos;
//! [`winkler`] aporta el lecho elástico. Las hipótesis que siguen describen
//! exclusivamente el perfil de tensiones geostáticas.
//! [`py`] genera curvas laterales estáticas en SI sin prefijos (N, m, Pa).
//! Las curvas axiales [`tz`] y [`qz`] usan SI sin prefijos: m, Pa y N; su z es
//! desplazamiento local.
//!
//! # Unidades, ejes e hipótesis
//!
//! Espesores y profundidades en m, pesos unitarios en kN/m³ y tensiones en kPa:
//! `(kN/m³) × m = kN/m² = kPa`. Son unidades SI con prefijos explícitos.
//! La API de perfiles usa kPa; para integrar con los contratos estructurales en Pa, el
//! llamador debe convertir mediante `engine_core::units::convert`. El catálogo
//! de unidades del núcleo no representa pesos unitarios; no se amplía aquí.
//!
//! En el perfil, la profundidad `z` es positiva hacia abajo: con Z global
//! hacia arriba, `z = Z_superficie − Z`. La compresión geotécnica es positiva;
//! al representar una tensión normal con tracción positiva debe invertirse su
//! signo. No se interpreta el plano D2 horizontal como sección vertical.
//!
//! Cada capa aporta γ por encima del nivel freático `zw` y γsat por debajo.
//! Se integra `σv = Σ γi hi`, dividiendo las capas que cruzan `zw`.
//! `u = γw max(z − zw, 0)` y `σ'v = σv − u`. No se vuelve a sumar el peso
//! del agua a γsat. Sin cohesión, capilaridad, succión, sobrecargas, flujo,
//! exceso de presión de poro, consolidación ni tensiones horizontales.
//! Se rechaza `zw < 0` (agua sobre el terreno); `zw >= espesor_total` produce
//! un perfil seco a efectos de este modelo. En el nivel freático, `u = 0`.
//!
//! # Datos explícitos y diagnósticos
//!
//! Cada γ, γsat, γw y zw tiene [`Provenance`] obligatorio, sin valores por
//! defecto. [`Layer`] permite representar datos ausentes; el constructor del
//! perfil exige γ en todas las capas y γsat en cada capa con tramo bajo `zw`.
//! También exige zw y γw, incluso en un perfil seco. Valida el perfil completo
//! antes de cualquier consulta y nunca sustituye γsat por γ.
//! Todos los pesos aportados deben ser positivos y finitos, sin imponer
//! `γsat >= γ` ni `γsat >= γw`; la tensión efectiva puede ser negativa y no
//! se recorta. Los resultados reportan todos los parámetros declarados
//! [`Provenance::Assumed`], aunque no intervengan a la profundidad consultada.
//!
//! El catálogo carece de códigos geotécnicos específicos. Se reutiliza
//! `ENG-E-0003` para datos ausentes, `ENG-E-0010` para rangos/geometría inválidos
//! y `ENG-E-0008` para NaN, infinitos o desbordamientos. Cada [`Diagnostic`]
//! conserva mensaje/gravedad del catálogo y detalla dato, causa y corrección en
//! `hint`. `entities` queda vacío porque el núcleo no tiene entidades de suelo.
//!
//! # Compatibilidad
//!
//! Crate 0.1.0: una modificación incompatible de la API pública incrementa la
//! versión menor según semver antes de 1.0. El contrato propio `stress-profile/1`
//! incrementa su versión mayor si cambian unidades, signos o requisitos del
//! trait; su versión menor admite extensiones compatibles.
//!
//! ```
//! use engine_geo::{Layer, Parameter, Provenance, StratifiedProfile, StressProfile};
//! let measured = |value| Parameter::new(value, Provenance::Measured);
//! let profile = StratifiedProfile::new(
//!     vec![Layer::new(10.0, Some(measured(18.0)), Some(measured(20.0)))],
//!     Some(measured(3.0)), // zw, m bajo el terreno
//!     Some(Parameter::new(10.0, Provenance::Assumed)), // γw, kN/m³
//! )?;
//! let interchangeable: &dyn StressProfile = &profile;
//! let stress = interchangeable.stresses_at(5.0)?;
//! assert_eq!(stress.total_vertical_kpa, 18.0 * 3.0 + 20.0 * 2.0);
//! assert_eq!(stress.pore_pressure_kpa, 20.0);
//! assert_eq!(stress.effective_vertical_kpa, 74.0);
//! assert_eq!(stress.assumed_parameters, [engine_geo::ParameterId::WaterUnitWeight]);
//! # Ok::<(), engine_core::Diagnostic>(())
//! ```

#![deny(missing_docs)]

pub use engine_core::{ContractId, Diagnostic, Real};

pub mod winkler;

pub mod py;
pub mod tz;
pub use tz::qz;

pub mod consolidation;

mod profile;
pub use profile::StratifiedProfile;

/// Resultado geotécnico con un diagnóstico del catálogo y `hint` contextual.
pub type GeoResult<T> = Result<T, Diagnostic>;

/// Origen declarado de un dato, sin clasificación implícita ni valor por defecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Dato medido, según la declaración del llamador.
    Measured,
    /// Dato supuesto, que se reportará explícitamente en el resultado.
    Assumed,
}

/// Valor y origen inseparables; la unidad depende del campo que lo recibe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameter {
    /// Valor en las unidades explícitas del campo que lo recibe.
    pub value: Real,
    /// Origen explícito del valor; no certifica la calidad de una medición.
    pub provenance: Provenance,
}

impl Parameter {
    /// Declara un valor y su origen; la validación ocurre al construir el perfil.
    pub const fn new(value: Real, provenance: Provenance) -> Self {
        Self { value, provenance }
    }
}

/// Capa horizontal de espesor constante; las capas se ordenan desde el terreno.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    /// Espesor en m; debe ser finito y estrictamente positivo.
    pub thickness_m: Real,
    /// γ en kN/m³ sobre el nivel freático, requerido en todas las capas.
    pub gamma_kn_m3: Option<Parameter>,
    /// γsat en kN/m³, requerido si parte de la capa está bajo el nivel freático.
    pub gamma_sat_kn_m3: Option<Parameter>,
}

impl Layer {
    /// Declara una capa sin rellenar datos ausentes ni validarlos todavía.
    pub const fn new(
        thickness_m: Real,
        gamma_kn_m3: Option<Parameter>,
        gamma_sat_kn_m3: Option<Parameter>,
    ) -> Self {
        Self {
            thickness_m,
            gamma_kn_m3,
            gamma_sat_kn_m3,
        }
    }
}

/// Identidad de un parámetro supuesto; índices de capa desde cero, sin duplicados.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterId {
    /// γ de la capa indicada.
    LayerGamma {
        /// Posición de la capa desde el terreno, empezando en cero.
        layer_index: usize,
    },
    /// γsat de la capa indicada.
    LayerGammaSat {
        /// Posición de la capa desde el terreno, empezando en cero.
        layer_index: usize,
    },
    /// Profundidad zw del nivel freático.
    GroundwaterDepth,
    /// Peso unitario γw del agua.
    WaterUnitWeight,
}

/// Tensiones a una profundidad, con procedencia explícita del perfil completo.
#[derive(Clone, Debug, PartialEq)]
pub struct StressResult {
    /// Profundidad consultada en m desde el terreno, positiva hacia abajo.
    pub depth_m: Real,
    /// Tensión vertical total σv en kPa, compresión positiva.
    pub total_vertical_kpa: Real,
    /// Presión de poro u en kPa, hidrostática bajo zw y cero encima.
    pub pore_pressure_kpa: Real,
    /// Tensión efectiva σ'v = σv − u en kPa, sin recorte de valores negativos.
    pub effective_vertical_kpa: Real,
    /// Todos los parámetros aportados como Assumed, incluidos los no utilizados.
    ///
    /// Orden: γ, γsat por cada capa desde arriba, luego zw y γw.
    pub assumed_parameters: Vec<ParameterId>,
}

/// Perfil intercambiable, puro y thread-safe, contrato propio `stress-profile/1`.
///
/// Entrada: profundidad finita en m en `[0, total_depth_m()]`, sin extrapolar.
/// Salida: tensiones finitas en kPa con compresión positiva y reporte Assumed.
/// Errores según la documentación del crate; nunca omite un dato requerido.
/// Los métodos escalares son atajos; use `stresses_at` para obtener procedencia.
pub trait StressProfile: Send + Sync {
    /// Identidad del contrato; nombres o versiones mayores diferentes son incompatibles.
    fn contract(&self) -> ContractId {
        ContractId::new("stress-profile", 1, 0)
    }

    /// Profundidad máxima admitida en m, finita y positiva.
    fn total_depth_m(&self) -> Real;

    /// Calcula σv, u y σ'v, junto con los parámetros Assumed del perfil completo.
    fn stresses_at(&self, depth_m: Real) -> GeoResult<StressResult>;

    /// Calcula σv en kPa; propaga los diagnósticos de `stresses_at`.
    fn total_vertical_stress_at(&self, depth_m: Real) -> GeoResult<Real> {
        Ok(self.stresses_at(depth_m)?.total_vertical_kpa)
    }

    /// Calcula u en kPa; propaga los diagnósticos de `stresses_at`.
    fn pore_pressure_at(&self, depth_m: Real) -> GeoResult<Real> {
        Ok(self.stresses_at(depth_m)?.pore_pressure_kpa)
    }

    /// Calcula σ'v en kPa; propaga los diagnósticos de `stresses_at`.
    fn effective_vertical_stress_at(&self, depth_m: Real) -> GeoResult<Real> {
        Ok(self.stresses_at(depth_m)?.effective_vertical_kpa)
    }
}

#[cfg(test)]
mod tests;
