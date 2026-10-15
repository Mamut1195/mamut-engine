// SPDX-License-Identifier: Apache-2.0

use crate::CliFailure;
use std::slice::Iter;

/// Parsed local CLI command. Paths equal to `-` refer to the supplied reader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Print usage.
    Help,
    /// Describe this engine.
    Describe,
    /// List catalog methods.
    Methods {
        /// Catalog file path or `-`.
        catalog: String,
    },
    /// Compile the selected workflow.
    Compile {
        /// Catalog file path or `-`.
        catalog: String,
        /// Workflow file path or `-`.
        workflow: String,
    },
    /// Validate without execution.
    Validate {
        /// Catalog file path or `-`.
        catalog: String,
        /// Workflow file path or `-`.
        workflow: String,
    },
    /// Execute the selected workflow.
    Run {
        /// Catalog file path or `-`.
        catalog: String,
        /// Workflow file path or `-`.
        workflow: String,
        /// Emit progress JSON lines.
        progress_ndjson: bool,
    },
    /// Serve the local stdio MCP protocol.
    Mcp {
        /// Catalog file path or `-`.
        catalog: String,
    },
}

enum CommandName {
    Describe,
    Methods,
    Compile,
    Validate,
    Run,
    Mcp,
}

/// Parse arguments excluding the executable name. Errors use default exit codes.
/// [`run`](crate::run) remaps those codes using its configuration.
pub fn parse_args(args: &[String]) -> Result<Command, CliFailure> {
    let mut rest = args.iter();
    let Some(command) = rest.next() else {
        return Err(CliFailure::usage("missing command"));
    };
    let name = match command.as_str() {
        "help" | "--help" | "-h" => return Ok(Command::Help),
        "describe" => CommandName::Describe,
        "methods" => CommandName::Methods,
        "compile" => CommandName::Compile,
        "validate" => CommandName::Validate,
        "run" => CommandName::Run,
        "mcp" => CommandName::Mcp,
        other => return Err(CliFailure::usage(format!("unknown command `{other}`"))),
    };
    let mut catalog = None;
    let mut workflow = None;
    let mut progress_ndjson = false;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "help" | "--help" | "-h" => return Ok(Command::Help),
            "--catalog" => {
                if catalog.is_some() {
                    return Err(CliFailure::usage("--catalog was set twice"));
                }
                catalog = Some(required_value(&mut rest, "--catalog")?);
            }
            "--progress" => {
                if progress_ndjson {
                    return Err(CliFailure::usage("--progress was set twice"));
                }
                if required_value(&mut rest, "--progress")? != "ndjson" {
                    return Err(CliFailure::usage("--progress requires ndjson"));
                }
                progress_ndjson = true;
            }
            "--workflow" => {
                if workflow.is_some() {
                    return Err(CliFailure::usage("--workflow was set twice"));
                }
                workflow = Some(required_value(&mut rest, "--workflow")?);
            }
            other => return Err(CliFailure::usage(format!("unknown argument `{other}`"))),
        }
    }
    if catalog.as_deref() == Some("-") && workflow.as_deref() == Some("-") {
        return Err(CliFailure::usage(
            "--catalog and --workflow cannot both read stdin",
        ));
    }
    if progress_ndjson && !matches!(name, CommandName::Run) {
        return Err(CliFailure::usage("--progress is only supported by run"));
    }
    match name {
        CommandName::Describe => {
            if catalog.is_some() || workflow.is_some() {
                return Err(CliFailure::usage(
                    "describe does not take --catalog or --workflow",
                ));
            }
            Ok(Command::Describe)
        }
        CommandName::Methods => {
            if workflow.is_some() {
                return Err(CliFailure::usage("methods does not take --workflow"));
            }
            let catalog = catalog.ok_or_else(|| CliFailure::usage("methods requires --catalog"))?;
            Ok(Command::Methods { catalog })
        }
        CommandName::Compile => {
            let (catalog, workflow) = require_pair("compile", catalog, workflow)?;
            Ok(Command::Compile { catalog, workflow })
        }
        CommandName::Validate => {
            let (catalog, workflow) = require_pair("validate", catalog, workflow)?;
            Ok(Command::Validate { catalog, workflow })
        }
        CommandName::Run => {
            let (catalog, workflow) = require_pair("run", catalog, workflow)?;
            Ok(Command::Run {
                catalog,
                workflow,
                progress_ndjson,
            })
        }
        CommandName::Mcp => {
            if workflow.is_some() {
                return Err(CliFailure::usage("mcp does not take --workflow"));
            }
            let catalog = catalog.ok_or_else(|| CliFailure::usage("mcp requires --catalog"))?;
            Ok(Command::Mcp { catalog })
        }
    }
}

fn require_pair(
    command: &str,
    catalog: Option<String>,
    workflow: Option<String>,
) -> Result<(String, String), CliFailure> {
    let catalog =
        catalog.ok_or_else(|| CliFailure::usage(format!("{command} requires --catalog")))?;
    let workflow =
        workflow.ok_or_else(|| CliFailure::usage(format!("{command} requires --workflow")))?;
    Ok((catalog, workflow))
}

fn required_value(args: &mut Iter<'_, String>, flag: &str) -> Result<String, CliFailure> {
    match args.next() {
        Some(value) if !value.starts_with("--") && !value.is_empty() => Ok(value.clone()),
        _ => Err(CliFailure::usage(format!("missing value for {flag}"))),
    }
}
