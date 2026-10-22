// SPDX-License-Identifier: AGPL-3.0-only

//! Sobre terminal del adaptador. No modifica documentos ni manifiestos del motor.

use std::io::Write;

use mamut_workflow_cli::{ExitCodes, FailureKind};
use serde_json::{json, Value};

/// Nombre público conocido; nunca refleja argumentos arbitrarios en stderr.
pub(crate) fn operation(args: &[String]) -> &str {
    let mut args = args.iter();
    let mut command = args.next();
    // Timeout is the only adapter option accepted before the command.
    while command.is_some_and(|arg| arg == "--timeout") {
        args.next();
        command = args.next();
    }
    match command.map(String::as_str) {
        Some("run") => "run",
        Some("validate") => "validate",
        Some("compile") => "compile",
        Some("methods") => "methods",
        Some("describe") => "describe",
        Some("help" | "--help" | "-h") | None => "help",
        Some("mcp") => "mcp",
        _ => "cli",
    }
}

fn classification(kind: FailureKind) -> (&'static str, &'static str) {
    match kind {
        FailureKind::Usage => ("CLI-E-USAGE", "usage"),
        FailureKind::Document => ("CLI-E-DOCUMENT", "invalid_input"),
        FailureKind::Validation => ("CLI-E-VALIDATION", "invalid_input"),
        FailureKind::Unsupported => ("CLI-E-UNSUPPORTED", "unsupported"),
        FailureKind::NotConverged => ("CLI-E-NOT-CONVERGED", "not_converged"),
        FailureKind::Cancelled => ("CLI-E-CANCELLED", "cancelled"),
        FailureKind::Timeout => ("CLI-E-TIMEOUT", "timeout"),
        FailureKind::Method => ("CLI-E-METHOD", "internal"),
        FailureKind::Internal => ("CLI-E-INTERNAL", "internal"),
    }
}

pub(crate) fn report_failure(
    kind: FailureKind,
    operation: &str,
    document: Option<&Value>,
    stderr: &mut impl Write,
) -> u8 {
    let exit_code = ExitCodes::engine_v1().code(kind);
    let (fallback, outcome) = classification(kind);
    let diagnostics = document
        .and_then(|document| document.get("diagnostics"))
        .and_then(Value::as_array)
        .map_or_else(Vec::new, Clone::clone);
    let primary = diagnostics.iter().find(|d| d["severity"] == "error");
    // Cancellation has one terminal CLI code across signals, markers and host
    // controls. Keep the API's original diagnostics and document below.
    let code = if kind == FailureKind::Cancelled {
        fallback
    } else {
        primary
            .and_then(|d| d["code"].as_str())
            .map_or(fallback, |code| code)
    };
    // Clave estable para MAM-836/MAM-841; solo texto base inglés, sin i18n.
    let message_key = format!("errors.{code}");
    let messages = serde_json::from_str::<Value>(include_str!("../messages/en.json"));
    let message = messages
        .as_ref()
        .ok()
        .and_then(|messages| messages[&message_key].as_str())
        .map_or("Execution could not be completed.", |message| message);
    let identity = super::identity();
    let mut envelope = json!({
        "schema": "cli-error/1",
        "type": "result",
        "engineId": identity.engine_id,
        "engineVersion": identity.engine_version,
        "operation": operation,
        "exitCode": exit_code,
        "outcome": outcome,
        "code": code,
        "messageKey": message_key,
        "error": {"code": code, "message": message},
        "diagnostics": diagnostics,
        // Las entidades del motor no identifican una ubicación JSON inequívoca.
        "pointer": null,
        "hint": primary.and_then(|d| d["hint"].as_str()),
    });
    if let Some(document) = document {
        envelope["engineResults"] = json!([document]);
    }
    if writeln!(super::EventWriter(stderr), "{envelope}").is_err() {
        ExitCodes::engine_v1().internal
    } else {
        exit_code
    }
}

#[cfg(test)]
#[path = "../tests/support/schema.rs"]
mod schema;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_failure_kind_emits_one_schema_valid_registered_envelope() {
        for kind in [
            FailureKind::Usage,
            FailureKind::Document,
            FailureKind::Validation,
            FailureKind::Unsupported,
            FailureKind::NotConverged,
            FailureKind::Cancelled,
            FailureKind::Timeout,
            FailureKind::Method,
            FailureKind::Internal,
        ] {
            for operation in ["run", "validate", "mcp", "cli"] {
                let mut stderr = Vec::new();
                let exit_code = report_failure(kind, operation, None, &mut stderr);
                let envelope = schema::assert_error_stream(&stderr, exit_code.into());
                assert_eq!(envelope["diagnostics"], json!([]));
                assert_eq!(envelope["pointer"], Value::Null);
                assert_eq!(envelope["hint"], Value::Null);
            }
        }
    }

    #[test]
    fn diagnostics_and_engine_document_are_preserved_with_first_error_as_primary() {
        let document = json!({"diagnostics":[
            {"code":"API-W-MASS-PROJECTION", "severity":"warning", "entities":[], "cause":"aviso", "hint":null},
            {"code":"ENG-E-0003", "severity":"error", "entities":[{"kind":"node", "id":99}], "cause":"Referencia ausente", "hint":"Revise los nodos"},
            {"code":"ENG-E-0002", "severity":"error", "entities":[], "cause":"Duplicado", "hint":null}
        ]});
        let mut stderr = Vec::new();
        report_failure(FailureKind::Validation, "run", Some(&document), &mut stderr);
        let envelope = schema::assert_error_stream(&stderr, 2);
        assert_eq!(envelope["code"], "ENG-E-0003");
        assert_eq!(envelope["hint"], "Revise los nodos");
        assert_eq!(envelope["diagnostics"], document["diagnostics"]);
        assert_eq!(envelope["engineResults"], json!([document]));
    }
}
