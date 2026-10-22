//! Documento `bench-model/1`. Las unidades son SI.
//!
//! `orientation.kind` es siempre `local_z_vector`: un vector global, no un ángulo.
//! `spectrum.direction` es un eje global del triedro Z hacia arriba.
//! `stories[].mass` es `{m, Izz}` en kg y kg·m².
//! `spectrum.Sa.unit` es la cadena `m/s^2`. La ordenada no está en g.

use serde::{Deserialize, Serialize};

/// Identificador del contrato de modelo de benchmark.
pub const SCHEMA: &str = "bench-model/1";

/// Eje global del triedro Z hacia arriba.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    /// Eje global X. El vector asociado es `[1, 0, 0]`.
    X,
    /// Eje global Y. El vector asociado es `[0, 1, 0]`.
    Y,
    /// Eje global Z, vertical hacia arriba. El vector asociado es `[0, 0, 1]`.
    Z,
}

impl Axis {
    /// Vector unitario global de este eje.
    #[must_use]
    pub const fn vector(self) -> [f64; 3] {
        match self {
            Self::X => [1.0, 0.0, 0.0],
            Self::Y => [0.0, 1.0, 0.0],
            Self::Z => [0.0, 0.0, 1.0],
        }
    }
}

/// Componente de desplazamiento o giro, en el orden de `results/1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dof {
    /// Índice 0 en `displacements`.
    Ux,
    /// Índice 1 en `displacements`.
    Uy,
    /// Índice 2 en `displacements`.
    Uz,
    /// Índice 3 en `displacements`.
    Rx,
    /// Índice 4 en `displacements`.
    Ry,
    /// Índice 5 en `displacements`.
    Rz,
}

impl Dof {
    /// Índice dentro del vector de seis componentes de `results/1`.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Ux => 0,
            Self::Uy => 1,
            Self::Uz => 2,
            Self::Rx => 3,
            Self::Ry => 4,
            Self::Rz => 5,
        }
    }
}

/// Columna, viga o barra de celosía.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberKind {
    /// Barra entre dos niveles, en la misma planta.
    Column,
    /// Barra horizontal dentro de un nivel.
    Beam,
    /// `truss3d.linear`. Solo aporta rigidez axial.
    Truss,
}

/// Análisis que cabe en un documento `engine-input/1` (un método por archivo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisKind {
    /// `linearStatic`.
    Static,
    /// `modal`, con `modal.n` modos.
    Modal,
    /// `responseSpectrum`.
    Spectrum,
    /// `pDelta`, con las opciones declaradas en `p_delta`.
    PDelta,
}

impl AnalysisKind {
    /// Nombre estable usado en rutas y en el índice.
    #[must_use]
    pub const fn file_key(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Modal => "modal",
            Self::Spectrum => "spectrum",
            Self::PDelta => "pdelta",
        }
    }

    /// Método de `engine-input/1`.
    #[must_use]
    pub const fn engine_method(self) -> &'static str {
        match self {
            Self::Static => "linearStatic",
            Self::Modal => "modal",
            Self::Spectrum => "responseSpectrum",
            Self::PDelta => "pDelta",
        }
    }

    pub(crate) fn parse(text: &str) -> Option<Self> {
        match text {
            "static" => Some(Self::Static),
            "modal" => Some(Self::Modal),
            "spectrum" => Some(Self::Spectrum),
            "pdelta" => Some(Self::PDelta),
            _ => None,
        }
    }
}

/// Citas que acompañan a cada modelo. No se ajustan por tamaño.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Geometría del pórtico.
    pub geometry: String,
    /// Hormigón y módulos.
    pub material: String,
    /// Constante de torsión.
    pub torsion: String,
    /// Definición de la masa de piso.
    pub mass: String,
    /// Espectro ya evaluado.
    pub spectrum: String,
}

/// Altura y masa de un piso. La base no es un piso.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Story {
    /// Índice del piso, desde 1. El nivel 0 es la base y no aparece aquí.
    pub level: u32,
    /// Altura del piso, en m: distancia vertical al nivel inferior.
    pub h: f64,
    /// Masa traslacional e inercia vertical del piso.
    pub mass: StoryMass,
}

/// Masa de un piso. No es un escalar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryMass {
    /// Masa traslacional total del piso, en kg.
    pub m: f64,
    /// Inercia de masa alrededor del eje global Z que pasa por el nudo maestro
    /// del diafragma de este piso, en kg·m².
    #[serde(rename = "Izz")]
    pub izz: f64,
}

/// Ejes de la retícula, en m, estrictamente crecientes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    /// Coordenadas X de las líneas, en m.
    pub x: Vec<f64>,
    /// Coordenadas Y de las líneas, en m.
    pub y: Vec<f64>,
}

/// Nudo de la retícula, incluida la base (`story` = 0).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    /// Identificador. En los modelos oficiales es contiguo desde 1.
    pub id: u32,
    /// 0 en la base; después coincide con `stories[].level`.
    pub story: u32,
    /// Índice de la línea X, desde 0.
    pub ix: u32,
    /// Índice de la línea Y, desde 0.
    pub iy: u32,
    /// Coordenadas globales `[x, y, z]` en m. Z hacia arriba.
    pub coordinates: [f64; 3],
}

/// Referencia del eje local z. No es un ángulo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Orientation {
    /// Siempre `local_z_vector`.
    pub kind: String,
    /// Componentes globales del vector de referencia. No está normalizado aquí.
    pub vector: [f64; 3],
}

/// Barra entre dos nudos.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    /// Identificador, desde 1 en los modelos oficiales.
    pub id: u32,
    /// Columna, viga o celosía.
    pub kind: MemberKind,
    /// Nudo inicial y nudo final.
    pub nodes: [u32; 2],
    /// Clave de `sections`.
    pub section: String,
    /// Vector de referencia del eje local z. La celosía no lo lleva.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
}

/// Sección elástica. E y G viven aquí; el traductor los pasa a la ley.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    /// Clave referida por las barras.
    pub id: String,
    /// Módulo de Young, en Pa.
    #[serde(rename = "E")]
    pub young: f64,
    /// Módulo de corte, en Pa. Ausente en una celosía.
    #[serde(rename = "G", default, skip_serializing_if = "Option::is_none")]
    pub shear: Option<f64>,
    /// Área, en m².
    #[serde(rename = "A")]
    pub area: f64,
    /// Inercia respecto del eje local y, en m⁴. Ausente en una celosía.
    #[serde(rename = "Iy", default, skip_serializing_if = "Option::is_none")]
    pub iy: Option<f64>,
    /// Inercia respecto del eje local z, en m⁴. Ausente en una celosía.
    #[serde(rename = "Iz", default, skip_serializing_if = "Option::is_none")]
    pub iz: Option<f64>,
    /// Constante de torsión de Saint-Venant, en m⁴. Ausente en una celosía.
    #[serde(rename = "J", default, skip_serializing_if = "Option::is_none")]
    pub j: Option<f64>,
}

/// Diafragma rígido de un piso.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diaphragm {
    /// Mismo índice que `stories[].level`.
    pub level: u32,
    /// Cota global Z del plano, en m.
    pub z: f64,
    /// Nudo independiente del diafragma. `Izz` se mide respecto de este nudo.
    pub master: u32,
    /// Resto de nudos del mismo nivel. No incluye al maestro.
    pub slaves: Vec<u32>,
}

/// Apoyo de la base.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Restraint {
    /// Nudo apoyado.
    pub node: u32,
    /// Componentes impedidas.
    pub dofs: Vec<Dof>,
}

/// Valor de un grado de libertad.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DofValue {
    /// Componente.
    pub dof: Dof,
    /// Fuerza en N, momento en N·m, masa en kg o inercia en kg·m², según el uso.
    pub value: f64,
}

/// Acción nodal de un patrón.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodalAction {
    /// Nudo cargado.
    pub node: u32,
    /// Componentes de la acción.
    pub values: Vec<DofValue>,
}

/// Patrones de carga nombrados. No son todavía casos de `engine-input/1`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoadPatterns {
    /// Peso propio, en N, con Z hacia arriba (la componente Uz es negativa).
    pub gravity: Vec<NodalAction>,
    /// Carga horizontal del mismo eje global que el espectro.
    pub lateral: Vec<NodalAction>,
    /// Casos con nombre propio. Si no está vacío, reemplaza a gravity y lateral.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<NamedPattern>,
}

/// Caso de carga con el nombre que usarán las combinaciones.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedPattern {
    /// Nombre del caso, por ejemplo `D` o `EX`.
    pub name: String,
    /// Acciones nodales de ese caso.
    pub actions: Vec<NodalAction>,
}

/// Término `caso × factor` de una combinación estática o de P-Δ.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseFactor {
    /// Nombre de un caso del documento.
    pub case: String,
    /// Factor, con signo. P-Δ y el espectro no reinterpretan este número.
    pub factor: f64,
}

/// Combinación lineal de casos. Solo viaja en el documento `linearStatic`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticCombination {
    /// Identificador de la combinación.
    pub id: String,
    /// Términos, en el orden declarado.
    pub terms: Vec<CaseFactor>,
}

/// Masa nodal explícita. Si el modelo la trae, es la única masa que se emite.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodalMass {
    /// Nudo que recibe la masa.
    pub node: u32,
    /// Componentes declarados. No se añaden otros.
    pub values: Vec<DofValue>,
}

/// Opciones de `pDelta`. No hay valores suplidos por el traductor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PDeltaSpec {
    /// Caso y factor de la carga que entra en la matriz geométrica.
    pub gravity: CaseFactor,
    /// Caso y factor del caso lateral que se resuelve.
    pub lateral: CaseFactor,
    /// Tolerancia relativa del Engine, en `(0, 1)`.
    pub tolerance: f64,
    /// Tope de iteraciones que admite el Engine.
    pub max_iterations: u32,
    /// Escala de desplazamientos, en m.
    pub displacement_scale: f64,
    /// Escala de giros, en rad.
    pub rotation_scale: f64,
}

/// Pedido modal. Los pórticos oficiales usan 12 modos; el contrato admite otro entero.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Modal {
    /// Número de modos, entre 1 y 16384.
    pub n: u32,
    /// `lumped` o `consistent`. Los modelos oficiales usan `lumped`.
    pub mass_kind: String,
    /// Direcciones globales en las que se informa la participación.
    pub directions: Vec<[f64; 3]>,
}

/// Eje global de la excitación espectral.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectrumDirection {
    /// Nombre del eje global.
    pub axis: Axis,
    /// Componentes globales. Deben ser el vector unitario de `axis`.
    pub vector: [f64; 3],
}

/// Tabla Sa ya evaluada.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaTable {
    /// Siempre `m/s^2`. No se admite `g`.
    pub unit: String,
    /// Pares `[T en s, Sa en m/s²]`, con T estrictamente creciente.
    pub pairs: Vec<[f64; 2]>,
}

/// Combinación de las direcciones espectrales.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectionalCombination {
    /// `weightedSum` o `srss`.
    pub method: String,
    /// Un factor no negativo por dirección, en el mismo orden.
    pub factors: Vec<f64>,
}

/// Una corrida espectral distinta dentro del mismo modelo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectrumVariant {
    /// Nombre estable, por ejemplo `srss-X`.
    pub id: String,
    /// Combinación modal: `cqc` o `srss`.
    pub method: String,
    /// Direcciones globales de esta corrida.
    pub directions: Vec<SpectrumDirection>,
    /// Combinación de esas direcciones y sus factores.
    pub directional_combination: DirectionalCombination,
}

/// Espectro de respuesta ya evaluado. El motor no reconstruye la norma.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spectrum {
    /// Etiqueta de zona sísmica.
    pub zone: String,
    /// Factor de zona Z, adimensional.
    #[serde(rename = "Z")]
    pub zone_factor: f64,
    /// Etiqueta del perfil de suelo.
    pub soil: String,
    /// Factor de reducción de respuesta R, adimensional.
    #[serde(rename = "R")]
    pub response_modification: f64,
    /// η, adimensional.
    pub eta: f64,
    /// Coeficiente de perfil Fa, adimensional.
    #[serde(rename = "Fa")]
    pub fa: f64,
    /// Coeficiente de perfil Fd, adimensional.
    #[serde(rename = "Fd")]
    pub fd: f64,
    /// Coeficiente de perfil Fs, adimensional.
    #[serde(rename = "Fs")]
    pub fs: f64,
    /// ζ de la tabla y de CQC, adimensional, en `[0, 1)`.
    pub zeta: f64,
    /// Gravedad usada al pasar de g a m/s². No es una ordenada espectral.
    pub g: f64,
    /// Exponente de la rama descendente.
    pub r: f64,
    /// Una dirección global. Incompatible con `directions`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<SpectrumDirection>,
    /// Varias direcciones globales, en orden. Incompatible con `direction`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub directions: Vec<SpectrumDirection>,
    /// Cómo se combinan las direcciones de la corrida por defecto.
    pub directional_combination: DirectionalCombination,
    /// Corridas adicionales. La tabla Sa y ζ siguen siendo los del espectro.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<SpectrumVariant>,
    /// Interpolación de la tabla que recibe el motor.
    pub interpolation: String,
    /// Extrapolación de la tabla que recibe el motor.
    pub extrapolation: String,
    /// Ordenadas ya evaluadas, en m/s².
    #[serde(rename = "Sa")]
    pub sa: SaTable,
}

/// Combinación modal. No es una combinación estática de casos.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Combination {
    /// `cqc` o `srss`.
    pub method: String,
    /// Mismo ζ que `spectrum.zeta`.
    pub zeta: f64,
}

/// Modelo completo `bench-model/1`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchModel {
    /// Siempre `bench-model/1`.
    pub schema: String,
    /// Identificador estable, por ejemplo `rc3d-3s`.
    pub id: String,
    /// Siempre `SI`.
    pub units: String,
    /// Citas de geometría, material, masa y espectro.
    pub source: Source,
    /// Pisos, sin contar la base.
    pub stories: Vec<Story>,
    /// Líneas X e Y.
    pub grid: Grid,
    /// Nudos, incluida la base.
    pub nodes: Vec<Node>,
    /// Columnas y vigas.
    pub members: Vec<Member>,
    /// Secciones referidas por las barras.
    pub sections: Vec<Section>,
    /// Un diafragma por piso.
    pub diaphragms: Vec<Diaphragm>,
    /// Apoyos.
    pub restraints: Vec<Restraint>,
    /// Gravedad y lateral.
    pub load_patterns: LoadPatterns,
    /// Pedido modal.
    pub modal: Modal,
    /// Espectro y tabla Sa.
    pub spectrum: Spectrum,
    /// Combinación modal de la corrida espectral por defecto, y ζ.
    pub combination: Combination,
    /// Masas nodales explícitas. `None` conserva el reparto tributario de los oficiales.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nodal_masses: Option<Vec<NodalMass>>,
    /// Opciones de `pDelta`. Ausente: ese análisis no se puede traducir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_delta: Option<PDeltaSpec>,
    /// Combinaciones estáticas. Solo salen en el documento `linearStatic`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub static_combinations: Vec<StaticCombination>,
}

/// Token de unidad de Sa. Comparación exacta, sin símbolo de g.
pub const SA_UNIT: &str = "m/s^2";

/// Único `orientation.kind` admitido. Queda en la caja: el esquema publica el literal.
pub(crate) const ORIENTATION_KIND: &str = "local_z_vector";
