// SPDX-License-Identifier: Apache-2.0

//! execution-manifest/1 records of completed runs.
//!
//! `recordFingerprint` covers the entire serialized manifest, omitting only
//! that root field. Step inputs cover selection metadata and parameters;
//! outputs cover the JSON result returned by the method.

use mamut_canon::{fingerprint_value, Fingerprint};
use mamut_workflow::CompiledStep;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::method::Method;
use crate::ExecuteError;

/// Contract id used by this producer for its manifest document.
pub const EXECUTION_MANIFEST_ID: &str = "urn:mamut:workflow:execution-manifest:1.0.0";

/// Terminal statuses defined by execution-manifest/1.
///
/// This executor emits only `Completed`. Errors and cancellation return
/// [`ExecuteError`] without a manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Every step returned a result, in definition order.
    Completed,
    /// Numerical convergence was not reached.
    NotConverged,
    /// The numerical result was unstable.
    Unstable,
    /// Execution was cancelled.
    Cancelled,
    /// An operation failed.
    Failed,
    /// The numerical status could not be determined.
    Unknown,
}

/// Local workflow identity and exact version, as workflow-definition/1 references.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowReference {
    /// Compiled profile, or a deterministic synthetic id for an empty profile.
    pub id: String,
    /// Compiled numeric version represented as decimal text.
    pub version: String,
}

impl WorkflowReference {
    /// Map a compiled profile/version to this producer's local workflow identity.
    ///
    /// An empty profile uses `urn:mamut:workflow:anonymous:<hash>`, where hash
    /// is SHA-256 over JCS of `{"profile":"","version":<u32>}`.
    pub fn from_profile(profile: &str, version: u32) -> Result<Self, ExecuteError> {
        let id = if profile.is_empty() {
            let hash =
                fingerprint_of(&serde_json::json!({"profile": profile, "version": version}))?;
            format!("urn:mamut:workflow:anonymous:{}", hash.value)
        } else {
            profile.to_owned()
        };
        Ok(Self {
            id,
            version: version.to_string(),
        })
    }
}

/// Identity and exact version of the method that actually ran.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodReference {
    /// Resolved method id, including substitutions.
    pub id: String,
    /// Resolved numeric version represented as decimal text.
    pub version: String,
}

/// Named JSON value or aggregate scope fingerprint, as execution-manifest/1.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionFingerprint {
    /// Scope name, unique within its list.
    pub name: String,
    /// SHA-256 over JCS bytes of the available JSON value.
    pub fingerprint: Fingerprint,
}

/// One completed method invocation, as execution-manifest/1 StepExecution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StepExecution {
    /// Slot name, or `step-<zero-based index>` when the slot is empty.
    pub step_id: String,
    /// Method that actually ran.
    pub method: MethodReference,
    /// Terminal status of this invocation.
    pub status: ExecutionStatus,
    /// The `selection` scope covers slot, method, numeric version and parameters.
    pub inputs: Vec<ExecutionFingerprint>,
    /// The `result` scope covers the complete JSON result of [`crate::Method::run`].
    pub outputs: Vec<ExecutionFingerprint>,
}

/// Record of one completed run, conforming to execution-manifest/1.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecutionManifest {
    /// [`EXECUTION_MANIFEST_ID`], retained as this producer's deterministic id.
    pub id: String,
    /// Local workflow reference derived from the compiled profile/version.
    pub workflow: WorkflowReference,
    /// [`ExecutionStatus::Completed`] for this executor.
    pub status: ExecutionStatus,
    /// Empty: this executor does not record workflow-wide input JSON values.
    pub inputs: Vec<ExecutionFingerprint>,
    /// Empty: this executor does not record workflow-wide output JSON values.
    pub outputs: Vec<ExecutionFingerprint>,
    /// Invocations in definition order.
    pub steps: Vec<StepExecution>,
    /// JCS fingerprint of this manifest omitting only this root field.
    /// The executor always populates it; the contract permits its absence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_fingerprint: Option<Fingerprint>,
}

pub(crate) fn record_step<T: Method>(
    index: usize,
    step: &CompiledStep<T>,
    output: &Value,
) -> Result<StepExecution, ExecuteError> {
    if step.method_id.is_empty() {
        return Err(ExecuteError::Fingerprint {
            message: format!("step {index} has an empty method id"),
        });
    }
    let input = serde_json::json!({
        "capability_slot": step.capability_slot,
        "method_id": step.method_id,
        "method_version": step.method_version,
        "parameters": step.parameters,
    });
    Ok(StepExecution {
        step_id: if step.capability_slot.is_empty() {
            format!("step-{index}")
        } else {
            step.capability_slot.clone()
        },
        method: MethodReference {
            id: step.method_id.clone(),
            version: step.method_version.to_string(),
        },
        status: ExecutionStatus::Completed,
        inputs: vec![ExecutionFingerprint {
            name: "selection".to_owned(),
            fingerprint: fingerprint_of(&input)?,
        }],
        outputs: vec![ExecutionFingerprint {
            name: "result".to_owned(),
            fingerprint: fingerprint_of(output)?,
        }],
    })
}

pub(crate) fn seal(
    profile: &str,
    version: u32,
    steps: Vec<StepExecution>,
) -> Result<ExecutionManifest, ExecuteError> {
    let mut manifest = ExecutionManifest {
        id: EXECUTION_MANIFEST_ID.to_owned(),
        workflow: WorkflowReference::from_profile(profile, version)?,
        status: ExecutionStatus::Completed,
        inputs: Vec::new(),
        outputs: Vec::new(),
        steps,
        record_fingerprint: None,
    };
    let value = serde_json::to_value(&manifest).map_err(|error| ExecuteError::Fingerprint {
        message: error.to_string(),
    })?;
    manifest.record_fingerprint = Some(fingerprint_of(&value)?);
    Ok(manifest)
}

fn fingerprint_of(value: &Value) -> Result<Fingerprint, ExecuteError> {
    fingerprint_value(value).map_err(|error| ExecuteError::Fingerprint {
        message: error.to_string(),
    })
}
