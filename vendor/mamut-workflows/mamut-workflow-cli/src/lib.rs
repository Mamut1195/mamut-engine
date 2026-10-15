// SPDX-License-Identifier: Apache-2.0

//! Reusable local CLI for external workflow engines.
//!
//! [`CliConfig`] injects real [`MethodProvider`] instances, identity, exit codes,
//! progress and cancellation. [`parse_args`], [`dispatch`] and [`run`] work with
//! caller-owned readers and writers. Default output preserves the reference CLI.

#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod config;
mod parser;

pub use config::{CliConfig, ExitCodes, FailureKind, EXIT_CANCELLED};
pub use mamut_workflow_execution::{
    cli_run_failure, cli_run_failure_with_identity, cli_run_success, cli_run_success_with_identity,
    compile_one_method, run_one_method, run_one_method_with, run_registered, run_registered_with,
    CancelFlag, Cancellation, CliIdentity, ExecuteError, ExecutionManifest, IdentityProvider,
    IgnoreProgress, Method, MethodCallError, MethodProvider, NeverCancel, ProgressEvent,
    ProgressListener, ProgressPhase, MAX_DOCUMENT_BYTES,
};
pub use parser::{parse_args, Command};

use std::{
    fs,
    io::{self, BufReader, Read, Write},
    str,
};

use mamut_workflow::{
    CompiledWorkflow, RegisteredCatalog, RegisteredMethod, WorkflowDocumentError,
};
use mamut_workflow_execution::{read_stdio_message, write_stdio_message, McpServer};
use serde::Serialize;
use serde_json::{json, Value};

/// CLI failure with semantic classification and a numeric code.
///
/// Parser failures carry default codes. Dispatch failures use the supplied
/// configuration. Run envelopes sanitize messages; other commands report them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliFailure {
    /// Numeric exit code.
    pub code: u8,
    /// Diagnostic for non-run commands.
    pub message: String,
    /// Semantic classification, retained when codes overlap.
    pub kind: FailureKind,
}

impl CliFailure {
    /// Construct a failure using the compatibility exit table.
    pub fn new(kind: FailureKind, message: impl Into<String>) -> Self {
        Self {
            code: ExitCodes::default().code(kind),
            kind,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new(FailureKind::Usage, message)
    }
    fn document(message: impl Into<String>) -> Self {
        Self::new(FailureKind::Document, message)
    }
    fn validation(message: impl Into<String>) -> Self {
        Self::new(FailureKind::Validation, message)
    }
    fn internal(message: impl Into<String>) -> Self {
        Self::new(FailureKind::Internal, message)
    }

    fn remap(mut self, config: &CliConfig<'_>) -> Self {
        self.code = config.exit_codes.code(self.kind);
        self
    }

    fn envelope(&self, config: &CliConfig<'_>) -> Value {
        let (outcome, message) = match self.kind {
            FailureKind::Usage => ("usage", "Invalid command or arguments"),
            FailureKind::Document => (
                "invalid_input",
                "Input document could not be read or parsed",
            ),
            FailureKind::Validation => ("invalid_input", "Workflow does not match the catalog"),
            FailureKind::Method => (
                "unknown",
                "Method execution failed without a semantic classification",
            ),
            FailureKind::Cancelled => ("cancelled", "Execution was cancelled"),
            FailureKind::Unsupported => ("unsupported", "Operation is not supported"),
            FailureKind::NotConverged => ("not_converged", "Execution did not converge"),
            FailureKind::Timeout => ("timeout", "Execution timed out"),
            FailureKind::Internal => ("internal", "Execution could not be completed"),
        };
        cli_run_failure_with_identity(&config.identity, self.code, outcome, message)
    }
}

impl std::fmt::Display for CliFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for CliFailure {}

/// Parse, dispatch and report a command. Arguments exclude the executable name.
///
/// stdout carries only the command result (or MCP frames). Failures and optional
/// NDJSON progress go to stderr; a configured progress writer overrides stderr.
/// Write failures return the configured internal code. No process is spawned.
pub fn run<R: Read, O: Write, E: Write>(
    args: &[String],
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
    config: &CliConfig<'_>,
) -> u8 {
    let result = parse_args(args)
        .map_err(|error| error.remap(config))
        .and_then(|command| dispatch(command, stdin, stdout, stderr, config));
    match result {
        Ok(()) => config.exit_codes.ok,
        Err(failure) => {
            let body = if args.first().is_some_and(|arg| arg == "run") {
                failure.envelope(config)
            } else {
                json!({"exit_code": failure.code, "message": failure.message})
            };
            if writeln!(stderr, "{body}").is_err() {
                config.exit_codes.internal
            } else {
                failure.code
            }
        }
    }
}

/// Execute an already parsed command with caller-owned streams.
///
/// This writes successful results and progress, returning failures to the caller
/// without printing an error envelope. Both parser and direct dispatch reject
/// two documents from stdin. All returned codes use `config.exit_codes`.
pub fn dispatch<R: Read, O: Write, E: Write>(
    command: Command,
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
    config: &CliConfig<'_>,
) -> Result<(), CliFailure> {
    dispatch_inner(command, stdin, stdout, stderr, config).map_err(|error| error.remap(config))
}

fn dispatch_inner<R: Read, O: Write, E: Write>(
    command: Command,
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
    config: &CliConfig<'_>,
) -> Result<(), CliFailure> {
    let body = match command {
        Command::Help => usage(config),
        Command::Describe => describe_json(config).to_string(),
        Command::Methods { catalog } => load_catalog(&catalog, stdin)?
            .methods_json()
            .map_err(|error| CliFailure::internal(error.to_string()))?,
        Command::Compile { catalog, workflow } | Command::Validate { catalog, workflow }
            if catalog == "-" && workflow == "-" =>
        {
            return Err(CliFailure::usage(
                "--catalog and --workflow cannot both read stdin",
            ))
        }
        Command::Compile { catalog, workflow } => {
            let catalog = load_catalog(&catalog, stdin)?;
            serialize(&compile_workflow(
                &catalog,
                &read_document(&workflow, stdin)?,
            )?)?
        }
        Command::Validate { catalog, workflow } => {
            let catalog = load_catalog(&catalog, stdin)?;
            let compiled = compile_workflow(&catalog, &read_document(&workflow, stdin)?)?;
            serialize(&validation_report(&compiled))?
        }
        Command::Run {
            catalog,
            workflow,
            progress_ndjson,
        } => {
            if catalog == "-" && workflow == "-" {
                return Err(CliFailure::usage(
                    "--catalog and --workflow cannot both read stdin",
                ));
            }
            let catalog = load_catalog(&catalog, stdin)?;
            let compiled = compile_workflow(&catalog, &read_document(&workflow, stdin)?)?;
            serialize(&execute_compiled(
                &catalog,
                &compiled,
                stderr,
                config,
                progress_ndjson,
            )?)?
        }
        Command::Mcp { catalog } => {
            let catalog = load_catalog(&catalog, stdin)?;
            serve(catalog, stdin, stdout, stderr, config)?;
            return Ok(());
        }
    };
    writeln!(stdout, "{body}").map_err(|error| CliFailure::internal(error.to_string()))
}

fn execute_compiled(
    catalog: &RegisteredCatalog,
    compiled: &CompiledWorkflow<RegisteredMethod>,
    stderr: &mut impl Write,
    config: &CliConfig<'_>,
    progress_ndjson: bool,
) -> Result<Value, CliFailure> {
    let mut configured_writer = config
        .progress_writer
        .as_ref()
        .map(|writer| writer.borrow_mut());
    let writer: &mut dyn Write = match configured_writer.as_mut() {
        Some(writer) => &mut ***writer,
        None => stderr,
    };
    let mut observer = NdjsonProgress {
        listener: &mut **config.progress.borrow_mut(),
        writer,
        enabled: progress_ndjson || config.progress_ndjson,
        error: None,
    };
    let result = run_registered_with(
        catalog,
        compiled,
        &*config.provider,
        &mut observer,
        &*config.cancellation,
    );
    if let Some(error) = observer.error {
        return Err(CliFailure::internal(error.to_string()));
    }
    let manifest = result.map_err(run_error)?;
    Ok(cli_run_success_with_identity(
        &manifest,
        &config.identity,
        config.exit_codes.ok,
    ))
}

struct NdjsonProgress<'a> {
    listener: &'a mut dyn ProgressListener,
    writer: &'a mut dyn Write,
    enabled: bool,
    error: Option<io::Error>,
}
impl ProgressListener for NdjsonProgress<'_> {
    fn on_progress(&mut self, event: ProgressEvent) {
        if self.enabled && self.error.is_none() {
            let phase = match event.phase {
                ProgressPhase::Started => "started",
                ProgressPhase::Finished => "finished",
            };
            let body = json!({"phase":phase, "index":event.index, "capability_slot":event.capability_slot,
                "method_id":event.method_id, "method_version":event.method_version});
            self.error = writeln!(self.writer, "{body}")
                .and_then(|()| self.writer.flush())
                .err();
        }
        self.listener.on_progress(event);
    }
}

fn usage(config: &CliConfig<'_>) -> String {
    let name = &config.describe_name;
    format!("{name} describe\n{name} methods --catalog <file>\n{name} compile --catalog <file> --workflow <file>\n{name} validate --catalog <file> --workflow <file>\n{name} run --catalog <file> --workflow <file>\n{name} mcp --catalog <file>")
}

fn describe_json(config: &CliConfig<'_>) -> Value {
    let tables = config.exit_codes.capabilities();
    json!({
        "name": config.describe_name, "version": config.identity.engine_version,
        "distribution_profile": "open-local", "network": false, "telemetry": false,
        "account_required": false, "ui_resources": false,
        "run_contract": "urn:mamut:tool:cli-run:1.0.0",
        "commands": config.describe_commands, "exit_codes": tables["exit_codes"],
        "exit_code_categories": tables["exit_code_categories"],
        "reserved_exit_code_categories": tables["reserved_exit_code_categories"]
    })
}

fn validation_report(compiled: &CompiledWorkflow<RegisteredMethod>) -> Value {
    json!({"valid":true, "profile":compiled.profile, "version":compiled.version,
        "steps":compiled.steps.iter().enumerate().map(|(index,step)| json!({
            "index":index, "capability_slot":step.capability_slot,
            "method_id":step.method_id, "method_version":step.method_version
        })).collect::<Vec<_>>()})
}

fn run_error(error: ExecuteError) -> CliFailure {
    match error {
        ExecuteError::MethodFailed { message, .. } => CliFailure::new(FailureKind::Method, message),
        ExecuteError::Cancelled { index } => CliFailure::new(
            FailureKind::Cancelled,
            format!("execution cancelled before step {index}"),
        ),
        ExecuteError::Fingerprint { message } => CliFailure::internal(message),
    }
}

fn compile_workflow(
    catalog: &RegisteredCatalog,
    text: &str,
) -> Result<CompiledWorkflow<RegisteredMethod>, CliFailure> {
    catalog.compile_json(text).map_err(|error| match error {
        WorkflowDocumentError::Json(message) => CliFailure::document(message),
        WorkflowDocumentError::Compile(error) => CliFailure::validation(error.to_string()),
    })
}
fn serialize(value: &impl Serialize) -> Result<String, CliFailure> {
    serde_json::to_string(value).map_err(|error| CliFailure::internal(error.to_string()))
}
fn load_catalog(path: &str, stdin: &mut impl Read) -> Result<RegisteredCatalog, CliFailure> {
    RegisteredCatalog::from_json(&read_document(path, stdin)?)
        .map_err(|error| CliFailure::document(error.to_string()))
}
fn read_document(path: &str, stdin: &mut impl Read) -> Result<String, CliFailure> {
    if path == "-" {
        read_bounded_document(stdin)
    } else {
        let file = fs::File::open(path)
            .map_err(|_| CliFailure::document("Input document could not be read"))?;
        read_bounded_document(file)
    }
}

fn read_bounded_document(reader: impl Read) -> Result<String, CliFailure> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_DOCUMENT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| CliFailure::document("Input document could not be read"))?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(CliFailure::document(
            "Input document exceeds the 16 MiB limit",
        ));
    }
    String::from_utf8(bytes).map_err(|_| CliFailure::document("Input document is not UTF-8"))
}

fn serve(
    catalog: RegisteredCatalog,
    stdin: &mut impl Read,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    config: &CliConfig<'_>,
) -> Result<(), CliFailure> {
    let mut reader = BufReader::new(stdin);
    let mut server = McpServer::new(catalog).with_exit_codes(config.exit_codes.clone());
    loop {
        let frame = read_stdio_message(&mut reader)
            .map_err(|error| CliFailure::internal(error.to_string()))?;
        let Some((frame, framing)) = frame else {
            return Ok(());
        };
        let message =
            str::from_utf8(&frame).map_err(|_| CliFailure::internal("frame is not utf-8"))?;
        let response = server.handle_with(
            message,
            &config.identity,
            |catalog, descriptor, parameters| {
                let result = parameters
                    .map_err(CliFailure::document)
                    .and_then(|parameters| {
                        compile_one_method(catalog, descriptor, parameters).map_err(|error| {
                            match error {
                                MethodCallError::InvalidInput(message) => {
                                    CliFailure::validation(message)
                                }
                                MethodCallError::MethodFailed(message) => {
                                    CliFailure::new(FailureKind::Method, message)
                                }
                                MethodCallError::Internal(message) => CliFailure::internal(message),
                            }
                        })
                    })
                    .and_then(|compiled| {
                        execute_compiled(catalog, &compiled, stderr, config, false)
                    });
                match result {
                    Ok(envelope) => envelope,
                    Err(error) => error.remap(config).envelope(config),
                }
            },
        );
        if let Some(response) = response {
            write_stdio_message(stdout, response.as_bytes(), framing)
                .map_err(|error| CliFailure::internal(error.to_string()))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_execution_errors_keep_code_ten_and_sanitize_details() {
        let failure = run_error(ExecuteError::Fingerprint {
            message: "private detail".into(),
        });
        assert_eq!(failure.code, 10);
        let report = failure.envelope(&CliConfig::default());
        assert_eq!(report, cli_run_failure(10));
        assert_eq!(report["outcome"], "internal");
        assert!(report.get("data").is_none());
        assert!(!report.to_string().contains("private detail"));
    }
}
