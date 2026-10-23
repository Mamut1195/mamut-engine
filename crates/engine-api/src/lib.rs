#![forbid(unsafe_code)]
//! Fachada portable de validación y análisis de `engine-input/1`.
//!
//! La única frontera serde/JSON del workspace; salida canónica `results/1`.
//! Consulte el README del crate para el alcance y las convenciones SI.
#![deny(missing_docs)]

mod analysis_input;
mod extensions;
mod geo_input;
mod input;
mod json;
mod limits;
pub use geo_input::{
    MAX_CONSOLIDATION_COST, MAX_CONSOLIDATION_SEGMENTS, MAX_CONSOLIDATION_SUBSTEPS,
    MAX_CONSOLIDATION_WORK,
};
mod methods;
pub use methods::{method_descriptor, method_descriptors, MethodDescriptor, MethodStatus};
mod query;
mod run;
pub use limits::{
    MAX_CASES, MAX_COMBINATIONS, MAX_INPUT_BYTES, MAX_JSON_ARRAY_ITEMS, MAX_JSON_DEPTH, MAX_LOADS,
    MAX_MEMBERS, MAX_NODES,
};
pub use query::ResultQuery;
pub use run::{exit_code, run, Outcome, Status};

use engine_core::{catalog, EntityRef, Severity};
use serde::Serialize;
use serde_json::{json, Value};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

/// Fallos de la frontera; los defectos topológicos se devuelven como diagnósticos.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "cause")]
pub enum ApiError {
    /// JSON, campos, tipos o convenciones de entrada inválidos.
    InvalidInput(String),
    /// Contrato de entrada desconocido o versión mayor no admitida.
    UnsupportedSchema(String),
    /// Documento de esquema no publicado por esta fachada.
    UnknownSchema(String),
    /// Código ausente del catálogo de engine-core.
    UnknownCode(String),
    /// Cancelación cooperativa solicitada entre etapas.
    Cancelled,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(s) => write!(f, "InvalidInput: {s}"),
            Self::UnsupportedSchema(s) => write!(f, "UnsupportedSchema: {s}"),
            Self::UnknownSchema(s) => write!(f, "UnknownSchema: {s}"),
            Self::UnknownCode(s) => write!(f, "UnknownCode: {s}"),
            Self::Cancelled => f.write_str("Cancelled"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Token reutilizable por referencia, sin runtime, hilos ni dependencia de SO.
/// Una vez cancelado permanece cancelado. Puede compartirse mediante `Arc`.
#[derive(Debug, Default)]
pub struct CancelToken(AtomicBool);

impl CancelToken {
    /// Crea un token inicialmente activo.
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }
    /// Solicita detenerse en el próximo punto de control.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    /// Consulta la solicitud sin ejecutar trabajo del motor.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
    fn check(&self) -> Result<(), ApiError> {
        if self.is_cancelled() {
            Err(ApiError::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// Progreso determinista al terminar una etapa, iteración o modo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Progress {
    /// Operación pública en curso, `validate` o `run`.
    pub operation: &'static str,
    /// Etapa finalizada; incluye gravity/pdeltaIteration/modal/spectrumMode/spectrumDirection.
    pub stage: &'static str,
    /// Etapas completadas (los casos cuentan individualmente).
    pub completed: usize,
    /// Total de etapas; P-Delta reserva su máximo y puede terminar antes.
    pub total: usize,
}

/// Diagnóstico serializable que conserva el catálogo y contexto del dominio.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Diagnostic {
    /// Código del dominio ENG-E-xxxx o código documentado local API-*.
    pub code: &'static str,
    /// `info`, `warning` o `error` según engine-core.
    pub severity: &'static str,
    /// Entidades `{kind, id}` en el orden del dominio.
    pub entities: Vec<Value>,
    /// Mensaje del diagnóstico del dominio.
    pub cause: String,
    /// Contexto o sugerencia original.
    pub hint: Option<String>,
}

impl From<engine_core::Diagnostic> for Diagnostic {
    fn from(d: engine_core::Diagnostic) -> Self {
        let entities = d
            .entities
            .into_iter()
            .map(|e| match e {
                EntityRef::Node(id) => json!({"kind": "node", "id": id.0}),
                EntityRef::Member(id) => json!({"kind": "member", "id": id.0}),
                EntityRef::Dof(id) => json!({"kind": "dof", "id": id.0}),
                EntityRef::Case(id) => json!({"kind": "case", "id": id}),
                EntityRef::Law(id) => json!({"kind": "law", "id": id}),
            })
            .collect();
        Self {
            code: d.code,
            severity: match d.severity {
                Severity::Info => "info",
                Severity::Warning => "warning",
                Severity::Error => "error",
            },
            entities,
            cause: d.message,
            hint: d.hint,
        }
    }
}

/// Versión del engine en este workspace y contratos reconocidos por la fachada.
pub fn version() -> Value {
    json!({"engine_version": env!("CARGO_PKG_VERSION"),
        "contracts": ["engine-input/1", "results/1"]})
}

/// Capacidades reales: estático, P-Delta, modal, espectro y combinaciones lineales.
/// `engine` indica presencia en engine-analysis; `run` indica soporte de la fachada.
/// `method_descriptors` publica estado, contratos y evidencia desde el registro SDK;
/// pDelta es Stable y se habilita por defecto. nonlinearStatic, soilProfile y
/// consolidation1d son Experimental hasta ENG-VAL-006; requieren
/// options.allow_experimental = true al ejecutar el job.
pub fn capabilities() -> Value {
    let mut capabilities = json!({
        "run": true,
        "operations": ["version", "capabilities", "schema", "explain", "validate", "run"],
        "schemas": {"engine-input/1": {"validate": true}, "results/1": {"emit": true},
            "validate-report/1": {"emit": true}},
        "validation": "topology",
        "combinations": {"run": true},
        "elements": ["frame2d.euler_elastic", "frame3d.euler_elastic", "truss2d.linear", "truss3d.linear", "frame2d.winkler", "frame3d.winkler", "spring.linear", "spring.nonlinear"],
        "laws": ["elastic", "steel.bilinear_kinematic", "multilinear_elastic", "elastic.cubic"],
        "end_releases": ["frame2d.euler_elastic", "frame3d.euler_elastic"],
        "methods": {
            "linearStatic": {"engine": true, "run": true},
            "nonlinearStatic": {"engine": true, "run": true},
            "soilProfile": {"engine": true, "run": true},
            "consolidation1d": {"engine": true, "run": true},
            "pDelta": {"engine": true, "run": true},
            "modal": {"engine": true, "run": true},
            "responseSpectrum": {"engine": true, "run": true}
        }
    });
    match method_descriptors() {
        Ok(descriptors) => {
            capabilities["method_descriptors"] = json!(descriptors);
        }
        Err(error) => {
            capabilities["run"] = json!(false);
            capabilities["registry_error"] = json!(error.to_string());
            capabilities["method_descriptors"] = json!([]);
        }
    }
    capabilities
}

// Registro único: añadir aquí un esquema lo expone y lo incluye en la publicación.
const SCHEMAS: &[(&str, &str)] = &[
    (
        "engine-input/1",
        include_str!("../schemas/engine-input-1.json"),
    ),
    ("results/1", include_str!("../schemas/results-1.json")),
    (
        "validate-report/1",
        include_str!("../schemas/validate-report-1.json"),
    ),
    (
        "capabilities/1",
        include_str!("../schemas/capabilities-1.json"),
    ),
    (
        "method-descriptor/1",
        include_str!("../schemas/method-descriptor-1.json"),
    ),
];

/// Enumera todos los kinds admitidos por `schema()`, en orden de registro.
pub fn schema_kinds() -> impl ExactSizeIterator<Item = &'static str> {
    SCHEMAS.iter().map(|(kind, _)| *kind)
}

/// Devuelve un documento JSON Schema estático (Draft 2020-12), sin E/S ni red.
/// Los kinds admitidos se enumeran mediante `schema_kinds()`.
pub fn schema(kind: &str) -> Result<&'static str, ApiError> {
    SCHEMAS
        .iter()
        .find_map(|(name, document)| (*name == kind).then_some(*document))
        .ok_or_else(|| ApiError::UnknownSchema(kind.into()))
}

/// Explicación española del catálogo del dominio y de códigos locales API-*.
pub fn explain(code: &str) -> Result<&'static str, ApiError> {
    run::explain(code)
        .or_else(|| catalog::explain(code))
        .ok_or_else(|| ApiError::UnknownCode(code.into()))
}

/// JSON compacto determinista: claves ordenadas recursivamente y -0.0 → +0.0.
/// Conserva el orden de arrays y los floats f64 con round-trip; no es RFC 8785.
/// Rechaza claves duplicadas, NaN/Inf, overflow y documentos con texto sobrante.
/// Aplica los límites públicos de entrada antes de construir el árbol JSON.
/// Los enteros léxicos requieren |n| <= 2^53-1 (rango seguro de JCS/binary64).
pub fn canonical_json(input_json: &str) -> Result<String, ApiError> {
    json::canonical(input_json)
}

/// Huella SHA-256 RFC 8785 del input, idéntica a `results/1.input_sha256`.
/// Aplica el mismo parseo estricto y límites de `run`, sin ejecutar física.
/// Rechaza literales enteros fuera de ±(2^53-1) antes de convertirlos a binary64.
/// No usa la serialización propia de `canonical_json` como preimagen.
pub fn input_sha256(input_json: &str) -> Result<String, ApiError> {
    fingerprint_value(&json::parse(input_json)?)
}

fn fingerprint_value(value: &Value) -> Result<String, ApiError> {
    mamut_canon::fingerprint_value(value)
        .map(|fingerprint| fingerprint.value)
        .map_err(|e| ApiError::InvalidInput(format!("RFC 8785 fingerprint: {e}")))
}

/// Valida el contrato, las convenciones y la topología; no comprueba estabilidad.
/// Fallos de parseo/contrato son `Err`; defectos del modelo son diagnósticos `Ok`.
pub fn validate(input_json: &str) -> Result<Vec<Diagnostic>, ApiError> {
    validate_with(input_json, &CancelToken::new(), |_| {})
}

/// Variante con cancelación y callback síncrono entre etapas completas.
/// Consulta antes de empezar y antes/después de cada callback, incluido el último.
/// El parseo, mapeo y validador del dominio son etapas indivisibles.
pub fn validate_with(
    input_json: &str,
    cancel: &CancelToken,
    mut progress: impl FnMut(Progress),
) -> Result<Vec<Diagnostic>, ApiError> {
    cancel.check()?;
    let value = json::parse(input_json)?;
    let contract = value
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::InvalidInput("schema is required and must be text".into()))?;
    if contract != "engine-input/1" {
        return Err(ApiError::UnsupportedSchema(contract.into()));
    }
    checkpoint(cancel, &mut progress, "parse", 1)?;
    let execution = input::decode(value)?;
    checkpoint(cancel, &mut progress, "model", 2)?;
    let diagnostics = run::validation(&execution);
    checkpoint(cancel, &mut progress, "topology", 3)?;
    Ok(diagnostics)
}

fn checkpoint(
    cancel: &CancelToken,
    progress: &mut impl FnMut(Progress),
    stage: &'static str,
    completed: usize,
) -> Result<(), ApiError> {
    cancel.check()?;
    progress(Progress {
        operation: "validate",
        stage,
        completed,
        total: 3,
    });
    cancel.check()
}
