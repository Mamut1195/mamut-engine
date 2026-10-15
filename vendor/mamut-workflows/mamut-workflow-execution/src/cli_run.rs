// SPDX-License-Identifier: Apache-2.0

//! Data-only cli-run/1 envelopes shared by CLI and MCP method tools.

use serde_json::{json, Value};

use crate::ExecutionManifest;

/// Engine identity and operation used in cli-run/1 envelopes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliIdentity {
    /// Stable engine identifier (`engineId`).
    pub engine_id: String,
    /// Engine version (`engineVersion`).
    pub engine_version: String,
    /// Operation name.
    pub operation: String,
}

impl Default for CliIdentity {
    fn default() -> Self {
        Self {
            engine_id: "mamut-workflow".into(),
            engine_version: env!("CARGO_PKG_VERSION").into(),
            operation: "run".into(),
        }
    }
}

/// Wrap a completed execution with the reference identity and success code 0.
/// The manifest recordFingerprint stays in data; it is not a cli-run fingerprint.
#[must_use]
pub fn cli_run_success(manifest: &ExecutionManifest) -> Value {
    cli_run_success_with_identity(manifest, &CliIdentity::default(), 0)
}

/// Wrap a completed execution using the caller's identity and success code.
#[must_use]
pub fn cli_run_success_with_identity(
    manifest: &ExecutionManifest,
    identity: &CliIdentity,
    code: u8,
) -> Value {
    json!({
        "type": "result",
        "engineId": identity.engine_id,
        "engineVersion": identity.engine_version,
        "operation": identity.operation,
        "outcome": "ok",
        "exitCode": code,
        "data": manifest
    })
}

/// Wrap a failure using the reference CLI's numeric mapping.
/// Messages are sanitized summaries: paths and engine text are never copied.
/// This legacy helper preserves its original mapping, including the internal
/// fallback for unlisted codes. Configurable hosts use the explicit variant.
#[must_use]
pub fn cli_run_failure(code: u8) -> Value {
    let (outcome, message) = match code {
        2 => ("usage", "Invalid command or arguments"),
        3 => (
            "invalid_input",
            "Input document could not be read or parsed",
        ),
        4 => ("invalid_input", "Workflow does not match the catalog"),
        5 => (
            "unknown",
            "Method execution failed without a semantic classification",
        ),
        _ => ("internal", "Execution could not be completed"),
    };
    cli_run_failure_with_identity(&CliIdentity::default(), code, outcome, message)
}

/// Wrap a failure with explicit identity, numeric code and semantic outcome.
///
/// Callers must pass a public, sanitized message. Numeric codes alone cannot
/// determine semantics when an engine supplies a different exit table.
#[must_use]
pub fn cli_run_failure_with_identity(
    identity: &CliIdentity,
    code: u8,
    outcome: &str,
    message: &str,
) -> Value {
    json!({
        "type": "result",
        "engineId": identity.engine_id,
        "engineVersion": identity.engine_version,
        "operation": identity.operation,
        "outcome": outcome,
        "exitCode": code,
        "error": {"code": format!("WF-E-{code:04}"), "message": message}
    })
}
