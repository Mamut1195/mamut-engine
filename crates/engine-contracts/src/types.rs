//! Datos de intercambio propios; sin parser JSON ni dependencia externa.
use crate::*;
use std::{collections::BTreeMap, sync::Arc};

/// Valor JSON mínimo construido en memoria, sin parser ni serializador.
///
/// Números finitos exigidos al validar cada consumidor; objetos ordenados y
/// sin claves duplicadas. Los esquemas se entregan como String JSON. La frontera
/// de serialización y el esquema completo de resultados quedan en otros tickets.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    /// Valor nulo.
    Null,
    /// Valor lógico.
    Bool(bool),
    /// Número real (su unidad la declara el campo).
    Number(Real),
    /// Texto Unicode.
    String(String),
    /// Secuencia ordenada.
    Array(Vec<JsonValue>),
    /// Objeto con claves únicas.
    Object(BTreeMap<String, JsonValue>),
}

/// Respuesta uniaxial de prueba, conservando intacto el estado aceptado.
#[derive(Clone, Debug, PartialEq)]
pub struct UniaxialResponse {
    /// Tensión en Pa, o fuerza en N para una ley de resorte.
    pub stress: Real,
    /// Derivada consistente: Pa, o N/m para un resorte.
    pub tangent: Real,
    /// Historia de prueba finita de longitud state_len; unidades según la ley.
    pub trial_state: Vec<Real>,
}
/// Respuesta de sección generalizada.
#[derive(Clone, Debug, PartialEq)]
pub struct SectionResponse {
    /// D2: [N, Mz]; D3: [N, My, Mz, T], en N y N·m.
    pub resultants: Vec<Real>,
    /// Derivada de resultantes respecto a deformaciones; cuadrada y simétrica v1.
    pub tangent: DMat,
    /// Historia plana finita, longitud declarada por la sección.
    pub trial_state: Vec<Real>,
}
/// Parámetros resueltos: propiedades SI y referencias constitutivas opcionales.
///
/// Cada kind documenta claves y requisitos; ninguna validación automática.
#[derive(Clone, Default)]
pub struct ElementParams {
    /// Propiedades escalares SI con nombres propios (por ejemplo área en m²).
    pub scalars: BTreeMap<String, Real>,
    /// Sección ya resuelta, si el elemento la necesita.
    pub section: Option<Arc<dyn SectionLaw>>,
    /// Ley ya resuelta, si el elemento la necesita.
    pub law: Option<Arc<dyn UniaxialLaw>>,
}
/// Vista mínima prestada para fábricas; no ensambla ni modifica engine-model.
pub struct ModelView<'a> {
    /// Dimensión que debe admitir el elemento solicitado.
    pub dim: Dim,
    /// Identificadores y coordenadas SI disponibles; únicos y finitos.
    pub nodes: &'a [(NodeId, [Real; 3])],
}
/// Contexto prestado de un elemento, independiente del estado de prueba.
pub struct ElementCtx<'a> {
    /// Dimensión global; D2 en X-Y.
    pub dim: Dim,
    /// Coordenadas m, por nodo; D2 requiere Z = 0.
    pub coords: &'a [[Real; 3]],
    /// Parámetros previamente resueltos; el consumidor verifica requisitos.
    pub params: &'a ElementParams,
}
/// Respuesta global de un elemento.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementResponse {
    /// Fuerzas N y momentos N·m, nodo y después GDL.
    pub internal_force: Vec<Real>,
    /// Derivada respecto a desplazamientos m y giros rad; simétrica v1.
    pub tangent: DMat,
    /// Historia finita de prueba, longitud state_len.
    pub trial_state: Vec<Real>,
}
/// Distribución de masa; traslaciones en kg y giros en kg·m².
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MassKind {
    /// Masa concentrada en nodos.
    Lumped,
    /// Masa consistente con la interpolación del elemento.
    Consistent,
}
/// Carga uniforme de tramo expresada en ejes globales.
///
/// Tipo mínimo v1: fuerzas por longitud en N/m y momentos por longitud en N.
/// Las implementaciones deben rechazar componentes no soportadas, nunca omitirlas.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberLoad {
    /// [qx, qy, qz] globales en N/m.
    pub force_per_length: [Real; 3],
    /// [mx, my, mz] globales en N·m/m.
    pub moment_per_length: [Real; 3],
}
/// Resultantes en una estación interior, cara positiva local.
#[derive(Clone, Debug, PartialEq)]
pub struct StationResult {
    /// Posición adimensional x/L en [0, 1].
    pub position: Real,
    /// [N, Vy, Vz, T, My, Mz], fuerzas N y momentos N·m.
    pub forces_local: [Real; 6],
}
/// Recuperación de resultados locales, incluyendo cargas de tramo soportadas.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementOutput {
    /// Acciones de extremo: nodo y GDL local, cara negativa i y positiva j.
    pub end_forces_local: Vec<Real>,
    /// Estaciones en el orden solicitado, con resultantes por cara positiva.
    pub stations: Vec<StationResult>,
}
/// Referencia nominal de caso, sin normalización ni caso implícito.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseRef(pub String);
/// Estado terminal del análisis, independiente de errores de entrada o E/S.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Todos los pasos solicitados fueron aceptados.
    Completed,
    /// Se conserva el último paso aceptado; sin reintentos ocultos.
    NotConverged,
    /// Pérdida de estabilidad detectada y documentada.
    Unstable,
    /// Cancelado cooperativamente; se conserva el último estado aceptado.
    Cancelled,
}
/// Tolerancias relativas adimensionales, finitas y positivas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerances {
    /// Criterio de fuerza relativa.
    pub force_rel: Real,
    /// Criterio de desplazamiento relativo.
    pub disp_rel: Real,
    /// Criterio de energía relativa.
    pub energy_rel: Real,
}
/// Informe explícito de un paso; las normas son relativas adimensionales.
#[derive(Clone, Debug, PartialEq)]
pub struct StepReport {
    /// Índice del paso contado desde uno.
    pub step: u32,
    /// Factor de carga adimensional.
    pub load_factor: Real,
    /// Número de iteraciones ejecutadas.
    pub iterations: u32,
    /// Norma relativa de desequilibrio de fuerzas.
    pub residual_norm: Real,
    /// Norma relativa del incremento de desplazamiento.
    pub displacement_norm: Real,
    /// Norma relativa del incremento de energía.
    pub energy_norm: Real,
    /// Si se cumplieron las tolerancias declaradas.
    pub converged: bool,
    /// Tolerancias efectivamente usadas, sin valores implícitos.
    pub tolerances: Tolerances,
}
/// Historia de convergencia ordenada por paso, incluyendo el intento fallido.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConvergenceReport {
    /// Informes de los intentos realizados; lista vacía si se cancela antes.
    pub steps: Vec<StepReport>,
}
/// Resultado terminal; EResult se reserva para fallos de entrada, contrato o E/S.
#[derive(Clone, Debug)]
pub struct AnalysisOutcome {
    /// Estado final, sin convertir falta de convergencia en éxito.
    pub status: Status,
    /// Normas y tolerancias de todos los intentos.
    pub report: ConvergenceReport,
    /// Diagnósticos del catálogo del núcleo, con contexto pertinente.
    pub diagnostics: Vec<Diagnostic>,
}
/// Capacidades que requiere un método antes de comenzar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Requirements {
    /// Necesita masa ensamblada.
    pub mass: bool,
    /// Necesita rigidez geométrica.
    pub geometric: bool,
    /// Necesita respuesta constitutiva no lineal.
    pub nonlinear: bool,
    /// Casos requeridos por nombre; vacía permite selección mediante ajustes.
    pub cases: Vec<CaseRef>,
}
/// Dirección de un campo en un bloque results/1; sin serialización en este ticket.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultKey {
    /// Bloque: nodes, members, modes, cases, combinations, envelopes,
    /// convergence, diagnostics o provenance.
    pub block: String,
    /// Identificador textual propio de entidad o caso; None para metadatos globales.
    pub entity: Option<String>,
    /// Nombre del campo; sus unidades y semántica se declaran en el productor.
    pub field: String,
}
/// Valor mínimo transportable por ResultSink; numéricos finitos en SI.
///
/// La procedencia completa y los filtros de results/1 corresponden a ENG-CORE-005.
#[derive(Clone, Debug)]
pub enum ResultValue {
    /// Valor escalar SI.
    Scalar(Real),
    /// Vector SI en el orden documentado por el campo.
    Vector(Vec<Real>),
    /// Texto, también útil para metadatos de procedencia.
    Text(String),
    /// Resultado local de miembro.
    Element(ElementOutput),
    /// Informe de convergencia.
    Convergence(ConvergenceReport),
    /// Diagnósticos con códigos del núcleo.
    Diagnostics(Vec<Diagnostic>),
}
