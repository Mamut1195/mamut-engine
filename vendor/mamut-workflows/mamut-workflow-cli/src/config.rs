// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, io::Write};

use crate::{
    Cancellation, CliIdentity, IdentityProvider, IgnoreProgress, MethodProvider, NeverCancel,
    ProgressListener,
};

pub use mamut_workflow_execution::{ExitCodes, FailureKind, EXIT_CANCELLED};

/// Reusable CLI configuration, built from compatibility defaults.
///
/// Borrowed providers, listeners, writers, and cancellation tokens are supported.
/// Listeners and optional progress writers use interior mutability so [`crate::run`]
/// takes `&CliConfig`. A configuration is intended for sequential invocations.
pub struct CliConfig<'a> {
    /// Identity in run and MCP method-tool success and failure envelopes.
    pub identity: CliIdentity,
    /// Name in `describe` and usage text.
    pub describe_name: String,
    /// Advertised commands in `describe`.
    pub describe_commands: Vec<String>,
    /// Numeric exit table.
    pub exit_codes: ExitCodes,
    /// Enable NDJSON even when the command does not include `--progress ndjson`.
    pub progress_ndjson: bool,
    pub(crate) provider: Box<dyn MethodProvider + 'a>,
    pub(crate) progress: RefCell<Box<dyn ProgressListener + 'a>>,
    pub(crate) cancellation: Box<dyn Cancellation + 'a>,
    pub(crate) progress_writer: Option<RefCell<Box<dyn Write + 'a>>>,
}

impl Default for CliConfig<'_> {
    fn default() -> Self {
        Self {
            identity: CliIdentity::default(),
            describe_name: "mamut-workflow".into(),
            describe_commands: ["describe", "methods", "compile", "validate", "run", "mcp"]
                .into_iter()
                .map(String::from)
                .collect(),
            exit_codes: ExitCodes::default(),
            progress_ndjson: false,
            provider: Box::new(IdentityProvider),
            progress: RefCell::new(Box::new(IgnoreProgress)),
            cancellation: Box::new(NeverCancel),
            progress_writer: None,
        }
    }
}

impl<'a> CliConfig<'a> {
    /// Compatibility defaults; also the entry point for the builder methods.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Supply real engine method instances.
    #[must_use]
    pub fn with_provider(mut self, provider: impl MethodProvider + 'a) -> Self {
        self.provider = Box::new(provider);
        self
    }

    /// Set execution and MCP identity; initialize the describe/usage name to engine id.
    #[must_use]
    pub fn with_identity(mut self, identity: CliIdentity) -> Self {
        self.describe_name = identity.engine_id.clone();
        self.identity = identity;
        self
    }

    /// Set the advertised name and commands (does not add parser commands).
    #[must_use]
    pub fn with_describe(mut self, name: impl Into<String>, commands: Vec<String>) -> Self {
        self.describe_name = name.into();
        self.describe_commands = commands;
        self
    }

    /// Replace numeric exit codes without changing failure semantics.
    #[must_use]
    pub fn with_exit_codes(mut self, codes: ExitCodes) -> Self {
        self.exit_codes = codes;
        self
    }

    /// Observe events in execution order, alongside optional NDJSON output.
    #[must_use]
    pub fn with_progress(mut self, listener: impl ProgressListener + 'a) -> Self {
        self.progress = RefCell::new(Box::new(listener));
        self
    }

    /// Supply a cancellation token checked before each method call.
    #[must_use]
    pub fn with_cancellation(mut self, cancellation: impl Cancellation + 'a) -> Self {
        self.cancellation = Box::new(cancellation);
        self
    }

    /// Enable or disable progress NDJSON by default.
    #[must_use]
    pub fn with_ndjson(mut self, enabled: bool) -> Self {
        self.progress_ndjson = enabled;
        self
    }

    /// Redirect NDJSON from stderr to a caller-owned writer. Flushes each event.
    #[must_use]
    pub fn with_progress_writer(mut self, writer: impl Write + 'a) -> Self {
        self.progress_writer = Some(RefCell::new(Box::new(writer)));
        self
    }
}
