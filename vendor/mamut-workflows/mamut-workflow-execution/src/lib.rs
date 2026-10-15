// SPDX-License-Identifier: Apache-2.0

//! Deterministic executor for a compiled workflow.
//!
//! [`Executor`] calls each [`Method`] in definition order and, when every
//! call returns, seals an execution manifest. The manifest fingerprint is
//! [`mamut_canon::fingerprint_value`]: JCS bytes, then SHA-256. There is no
//! clock and no random source. A method error or a cancellation returns
//! [`ExecuteError`] and does not return a manifest.
//!
//! The workflow has no branch step and no loop step. This crate does not run
//! a hydrology simulation.
//!
//! Shared implementation used by the CLI and the `mamut-workflow-run` facade. Each
//! registered method is an MCP tool. A tool call runs that one method.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod cli_run;
mod executor;
mod exit_codes;
mod manifest;
mod mcp;
mod method;
mod observe;
mod registered;

#[cfg(test)]
mod tests;

pub use cli_run::{
    cli_run_failure, cli_run_failure_with_identity, cli_run_success, cli_run_success_with_identity,
    CliIdentity,
};
pub use executor::Executor;
pub use exit_codes::{ExitCodes, FailureKind, EXIT_CANCELLED};
pub use manifest::{
    ExecutionFingerprint, ExecutionManifest, ExecutionStatus, MethodReference, StepExecution,
    WorkflowReference, EXECUTION_MANIFEST_ID,
};
pub use mcp::{
    read_frame, read_stdio_message, write_frame, write_stdio_message, McpServer, StdioFraming,
};
pub use method::Method;
pub use observe::{
    CancelFlag, Cancellation, IgnoreProgress, NeverCancel, ProgressEvent, ProgressListener,
    ProgressPhase,
};
pub use registered::{
    compile_one_method, run_one_method, run_one_method_with, run_registered, run_registered_with,
    IdentityProvider, MethodCallError, MethodProvider,
};

/// Maximum UTF-8 document or MCP message body size in bytes (16 MiB).
///
/// CLI catalog/workflow reads inspect at most this limit plus one byte to detect
/// oversized input. MCP framing applies the same limit to each message body.
pub const MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;

use std::fmt;

impl Executor<IgnoreProgress, NeverCancel> {
    /// Executor that ignores progress and does not cancel.
    pub fn new() -> Self {
        Self::with_observers(IgnoreProgress, NeverCancel)
    }
}

impl Default for Executor<IgnoreProgress, NeverCancel> {
    fn default() -> Self {
        Self::new()
    }
}

/// The run stopped before a manifest existed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecuteError {
    /// The caller cancelled before this step started.
    Cancelled {
        /// Step index that did not start.
        index: usize,
    },
    /// [`Method::run`] returned an error. Later steps were not called.
    MethodFailed {
        /// Step index that failed.
        index: usize,
        /// Method id.
        method_id: String,
        /// Method version.
        method_version: u32,
        /// Message from the method.
        message: String,
    },
    /// The manifest document could not be fingerprinted.
    Fingerprint {
        /// Canonicalization or JSON message.
        message: String,
    },
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled { index } => {
                write!(f, "execution cancelled before step {index}")
            }
            Self::MethodFailed {
                index,
                method_id,
                method_version,
                message,
            } => write!(
                f,
                "method `{method_id}` version {method_version} failed at step {index}: {message}"
            ),
            Self::Fingerprint { message } => {
                write!(f, "execution manifest fingerprint: {message}")
            }
        }
    }
}

impl std::error::Error for ExecuteError {}
