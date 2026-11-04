// SPDX-License-Identifier: AGPL-3.0-only

//! `mamut-engine convert` for mamut-toml/1.
//!
//! The mapped subset is the structural core, the load collections, and the
//! analysis branches written in mamut-toml/1. Optional names follow U5 and
//! return on export only with idmap/1. U9 is the canonical export in
//! `docs/V1.md` section 13. U7, U10, and U11 stay open.
//! `spring.nonlinear` is accepted only with `nonlinearStatic`.

mod core;
mod order;

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use engine_api::MAX_INPUT_BYTES;
use mamut_workflow_cli::{ExitCodes, FailureKind};
use serde_json::{json, Value};

use crate::error;

/// Convert failure. `cause` is a fixed sentence and never repeats input text.
#[derive(Debug)]
pub(crate) struct Failure {
    /// Process classification. The diagnostic code may be more specific.
    pub kind: FailureKind,
    /// Registered code.
    pub code: &'static str,
    /// JSON pointer, or `None` when the reader did not produce a safe location.
    pub pointer: Option<String>,
    /// Fixed diagnostic cause.
    pub cause: &'static str,
}

impl Failure {
    /// Format or subset violation. Exit 2, `CLI-E-VALIDATION`.
    pub(crate) fn validation(pointer: Option<String>, cause: &'static str) -> Self {
        Self {
            kind: FailureKind::Validation,
            code: "CLI-E-VALIDATION",
            pointer,
            cause,
        }
    }

    /// Unreadable or syntactically invalid document. Exit 2, `CLI-E-DOCUMENT`.
    pub(crate) fn document(cause: &'static str) -> Self {
        Self {
            kind: FailureKind::Document,
            code: "CLI-E-DOCUMENT",
            pointer: None,
            cause,
        }
    }

    /// Unsupported format version. Exit 3, `CLI-E-UNSUPPORTED`.
    pub(crate) fn unsupported(cause: &'static str) -> Self {
        Self {
            kind: FailureKind::Unsupported,
            code: "CLI-E-UNSUPPORTED",
            pointer: Some(String::new()),
            cause,
        }
    }

    /// Non-finite number on convert or export. Exit 2, `ENG-E-0008`.
    pub(crate) fn nonfinite() -> Self {
        Self {
            kind: FailureKind::Validation,
            code: "ENG-E-0008",
            pointer: None,
            cause: "A number is not finite.",
        }
    }

    /// Output or internal failure. Exit 70, `CLI-E-INTERNAL`.
    pub(crate) fn internal(cause: &'static str) -> Self {
        Self {
            kind: FailureKind::Internal,
            code: "CLI-E-INTERNAL",
            pointer: None,
            cause,
        }
    }
}

/// Reads at most 8 MiB and requires UTF-8. The extra byte is the overflow check.
pub(crate) fn read_limited(mut reader: impl Read) -> Result<String, Failure> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take((MAX_INPUT_BYTES as u64) + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Failure::document("The input could not be read."))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(Failure::document("The document exceeds the size limit."));
    }
    String::from_utf8(bytes).map_err(|_| Failure::document("The document is not valid UTF-8."))
}

/// Handles `convert`. Returns `None` for every other command.
pub(crate) fn dispatch<R: Read, O: Write, E: Write>(
    args: &[String],
    operation: &str,
    lang: engine_api::Lang,
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
) -> Option<u8> {
    if args.first().map(String::as_str) != Some("convert") {
        return None;
    }
    Some(convert(args, operation, lang, stdin, stdout, stderr))
}

fn convert<R: Read, O: Write, E: Write>(
    args: &[String],
    operation: &str,
    lang: engine_api::Lang,
    stdin: &mut R,
    stdout: &mut O,
    stderr: &mut E,
) -> u8 {
    let parsed = match parse_convert_args(args) {
        Ok(parsed) => parsed,
        Err(kind) => return error::report_failure_in(kind, operation, None, lang, stderr),
    };
    let text = match read_source(&parsed.input, stdin) {
        Ok(text) => text,
        Err(failure) => return report(failure, operation, lang, stderr),
    };
    let to_toml = match parsed.to.as_deref() {
        Some("toml") => true,
        Some("json") => false,
        None => detect_json(&text),
        _ => unreachable!("parse_convert_args only accepts json or toml"),
    };
    if parsed.idmap.is_some() && !to_toml {
        return error::report_failure_in(FailureKind::Usage, operation, None, lang, stderr);
    }
    let idmap = match &parsed.idmap {
        Some(path) => match read_idmap(path) {
            Ok(document) => Some(document),
            Err(failure) => return report(failure, operation, lang, stderr),
        },
        None => None,
    };
    let body = if to_toml {
        match idmap.as_deref() {
            None => core::json_to_toml(&text),
            Some(document) => core::json_to_toml_with_idmap(&text, Some(document)),
        }
    } else {
        core::toml_to_json(&text)
    };
    let body = match body {
        Ok(body) => body,
        Err(failure) => return report(failure, operation, lang, stderr),
    };
    if stdout.write_all(body.as_bytes()).is_err() {
        return report(
            Failure::internal("The output could not be written."),
            operation,
            lang,
            stderr,
        );
    }
    ExitCodes::engine_v1().ok
}

struct ConvertArgs {
    input: String,
    to: Option<String>,
    idmap: Option<String>,
}

fn parse_convert_args(args: &[String]) -> Result<ConvertArgs, FailureKind> {
    let mut input = None;
    let mut to = None;
    let mut idmap = None;
    let mut rest = args[1..].iter();
    while let Some(arg) = rest.next() {
        if arg == "--"
            || arg.starts_with("--input=")
            || arg.starts_with("--to=")
            || arg.starts_with("--idmap=")
        {
            return Err(FailureKind::Usage);
        }
        if arg == "--input" {
            if input.is_some() {
                return Err(FailureKind::Usage);
            }
            match rest.next() {
                Some(value) if !value.is_empty() && (*value == "-" || !value.starts_with('-')) => {
                    input = Some(value.clone());
                }
                _ => return Err(FailureKind::Usage),
            }
        } else if arg == "--to" {
            if to.is_some() {
                return Err(FailureKind::Usage);
            }
            match rest.next() {
                Some(value) if value == "json" || value == "toml" => to = Some(value.clone()),
                _ => return Err(FailureKind::Usage),
            }
        } else if arg == "--idmap" {
            if idmap.is_some() {
                return Err(FailureKind::Usage);
            }
            match rest.next() {
                Some(value) if !value.is_empty() && !value.starts_with('-') => {
                    idmap = Some(value.clone());
                }
                _ => return Err(FailureKind::Usage),
            }
        } else {
            return Err(FailureKind::Usage);
        }
    }
    Ok(ConvertArgs {
        input: input.ok_or(FailureKind::Usage)?,
        to,
        idmap,
    })
}

fn read_idmap(path: &str) -> Result<String, Failure> {
    let mut file = File::open(Path::new(path))
        .map_err(|_| Failure::document("The identity map could not be read."))?;
    let mut bytes = Vec::new();
    Read::take(&mut file, (MAX_INPUT_BYTES as u64) + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Failure::document("The identity map could not be read."))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(Failure::document(
            "The identity map exceeds the size limit.",
        ));
    }
    String::from_utf8(bytes).map_err(|_| Failure::document("The identity map is not valid UTF-8."))
}

fn read_source(input: &str, stdin: &mut impl Read) -> Result<String, Failure> {
    if input == "-" {
        read_limited(stdin)
    } else {
        let file = File::open(Path::new(input))
            .map_err(|_| Failure::document("The input could not be read."))?;
        read_limited(file)
    }
}

fn detect_json(text: &str) -> bool {
    text.trim_start_matches([' ', '\t', '\n', '\r'])
        .starts_with('{')
}

fn report(
    failure: Failure,
    operation: &str,
    lang: engine_api::Lang,
    stderr: &mut impl Write,
) -> u8 {
    let document = json!({
        "diagnostics": [{
            "code": failure.code,
            "severity": "error",
            "entities": [],
            "cause": failure.cause,
            "hint": Value::Null,
        }]
    });
    error::report_located(
        failure.kind,
        operation,
        Some(&document),
        failure.pointer.as_deref(),
        false,
        lang,
        stderr,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_limit_stops_before_a_parse() {
        let mut bytes = vec![b' '; MAX_INPUT_BYTES];
        assert!(read_limited(bytes.as_slice()).is_ok());
        bytes.push(b' ');
        let error = read_limited(bytes.as_slice()).unwrap_err();
        assert_eq!(error.code, "CLI-E-DOCUMENT");
        assert_eq!(error.cause, "The document exceeds the size limit.");
    }

    #[test]
    fn invalid_utf8_does_not_become_a_string() {
        let error = read_limited(&[0xff][..]).unwrap_err();
        assert_eq!(error.code, "CLI-E-DOCUMENT");
        assert_eq!(error.cause, "The document is not valid UTF-8.");
    }
}
