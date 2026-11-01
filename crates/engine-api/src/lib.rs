#![forbid(unsafe_code)]
//! Portable facade for validation and analysis of `engine-input/1`.
//!
//! The workspace's only serde/JSON boundary; canonical `results/1` output.
//! See the crate README for scope and SI conventions.
#![deny(missing_docs)]

mod analysis_input;
mod extensions;
mod geo_input;
/// Embedded translations shared by all public wrappers.
pub mod i18n;
mod input;
pub use i18n::{
    explain_in, localize_diagnostic, localize_diagnostics, localize_document, message,
    message_keys, Lang,
};
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

use engine_core::{EntityRef, Severity};
use serde::Serialize;
use serde_json::{json, Value};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

/// Boundary failures; topological defects are returned as diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "cause")]
pub enum ApiError {
    /// Invalid JSON, fields, types, or input conventions.
    InvalidInput(String),
    /// JSON numeric token that is not a finite binary64 value (overflow, NaN, Infinity).
    NonFiniteNumber(String),
    /// Unit system label not recognized by this contract (only "SI" is accepted).
    UnknownUnit(String),
    /// Unknown input contract or unsupported major version.
    UnsupportedSchema(String),
    /// Schema document not published by this facade.
    UnknownSchema(String),
    /// Code absent from the engine-core catalog.
    UnknownCode(String),
    /// Cooperative cancellation requested between stages.
    Cancelled,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(s) => write!(f, "InvalidInput: {s}"),
            Self::NonFiniteNumber(s) => write!(f, "NonFiniteNumber: {s}"),
            Self::UnknownUnit(s) => write!(f, "UnknownUnit: {s}"),
            Self::UnsupportedSchema(s) => write!(f, "UnsupportedSchema: {s}"),
            Self::UnknownSchema(s) => write!(f, "UnknownSchema: {s}"),
            Self::UnknownCode(s) => write!(f, "UnknownCode: {s}"),
            Self::Cancelled => f.write_str("Cancelled"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Token reusable by reference, without a runtime, threads, or OS dependency.
/// Once cancelled, it remains cancelled. Can be shared through `Arc`.
#[derive(Debug, Default)]
pub struct CancelToken(AtomicBool);

impl CancelToken {
    /// Creates an initially active token.
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }
    /// Requests a stop at the next checkpoint.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    /// Checks the request without performing engine work.
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

/// Deterministic progress upon completion of a stage, iteration, or mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Progress {
    /// Public operation in progress, `validate` or `run`.
    pub operation: &'static str,
    /// Completed stage; includes gravity/pdeltaIteration/modal/spectrumMode/spectrumDirection.
    pub stage: &'static str,
    /// Completed stages (cases count individually).
    pub completed: usize,
    /// Total stages; P-Delta reserves its maximum and may finish early.
    pub total: usize,
}

/// Serializable diagnostic preserving the domain catalog and context.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Diagnostic {
    /// Domain code ENG-E-xxxx or locally documented API-* code.
    pub code: &'static str,
    /// `info`, `warning`, or `error` as defined by engine-core.
    pub severity: &'static str,
    /// `{kind, id}` entities in domain order.
    pub entities: Vec<Value>,
    /// Domain diagnostic message.
    pub cause: String,
    /// Original context or hint.
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

/// Engine version in this workspace and contracts recognized by the facade.
pub fn version() -> Value {
    json!({"engine_version": env!("CARGO_PKG_VERSION"),
        "contracts": ["engine-input/1", "results/1"]})
}

/// Actual capabilities: static, P-Delta, modal, spectrum, and linear combinations.
/// `engine` indicates presence in engine-analysis; `run` indicates facade support.
/// `method_descriptors` publishes status, contracts, and evidence from the SDK registry;
/// pDelta is Stable and enabled by default. nonlinearStatic, soilProfile, and
/// consolidation1d are Experimental until ENG-VAL-006; they require
/// options.allow_experimental = true when running the job.
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

// Single registry: adding a schema here exposes it and includes it in publication.
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

/// Lists all kinds supported by `schema()`, in registry order.
pub fn schema_kinds() -> impl ExactSizeIterator<Item = &'static str> {
    SCHEMAS.iter().map(|(kind, _)| *kind)
}

/// Returns a static JSON Schema document (Draft 2020-12), without I/O or network access.
/// Supported kinds are listed by `schema_kinds()`.
pub fn schema(kind: &str) -> Result<&'static str, ApiError> {
    SCHEMAS
        .iter()
        .find_map(|(name, document)| (*name == kind).then_some(*document))
        .ok_or_else(|| ApiError::UnknownSchema(kind.into()))
}

/// Returns the unchanged English explanation for a stable domain or API code.
pub fn explain(code: &str) -> Result<&'static str, ApiError> {
    explain_in(code, Lang::En)
}

/// Deterministic compact JSON: recursively sorted keys and -0.0 → +0.0.
/// Preserves array order and round-trip f64 floats; this is not RFC 8785.
/// Rejects duplicate keys, NaN/Inf, overflow, and documents with trailing text.
/// Applies public input limits before building the JSON tree.
/// Lexical integers require |n| <= 2^53-1 (the JCS/binary64 safe range).
pub fn canonical_json(input_json: &str) -> Result<String, ApiError> {
    json::canonical(input_json)
}

/// RFC 8785 SHA-256 input fingerprint, identical to `results/1.input_sha256`.
/// Applies the same strict parsing and limits as `run`, without running physics.
/// Rejects integer literals outside ±(2^53-1) before converting to binary64.
/// Does not use the custom `canonical_json` serialization as the preimage.
pub fn input_sha256(input_json: &str) -> Result<String, ApiError> {
    fingerprint_value(&json::parse(input_json)?)
}

fn fingerprint_value(value: &Value) -> Result<String, ApiError> {
    mamut_canon::fingerprint_value(value)
        .map(|fingerprint| fingerprint.value)
        .map_err(|e| ApiError::InvalidInput(format!("RFC 8785 fingerprint: {e}")))
}

/// Validates the contract, conventions, and topology; does not check stability.
/// Parsing/contract failures are `Err`; model defects are `Ok` diagnostics.
pub fn validate(input_json: &str) -> Result<Vec<Diagnostic>, ApiError> {
    validate_with(input_json, &CancelToken::new(), |_| {})
}

/// Variant with cancellation and a synchronous callback between complete stages.
/// Checks before starting and before/after each callback, including the last one.
/// Parsing, mapping, and the domain validator are indivisible stages.
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
