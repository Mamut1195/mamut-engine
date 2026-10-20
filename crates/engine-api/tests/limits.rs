//! Límites y rangos comprobados antes del árbol JSON y de callbacks del motor.
use engine_api::{
    canonical_json, input_sha256, run, validate, validate_with, ApiError, CancelToken, Status,
    MAX_CASES, MAX_COMBINATIONS, MAX_INPUT_BYTES, MAX_JSON_ARRAY_ITEMS, MAX_JSON_DEPTH, MAX_LOADS,
    MAX_MEMBERS, MAX_NODES,
};
use serde_json::Value;

fn invalid_before_parse(input: &str, expected: &str) {
    for error in [
        canonical_json(input).unwrap_err(),
        input_sha256(input).unwrap_err(),
        validate(input).unwrap_err(),
    ] {
        assert!(matches!(error, ApiError::InvalidInput(_)));
        assert!(error.to_string().contains(expected), "{error}");
    }
    let mut callbacks = 0;
    let outcome = run(input, &CancelToken::new(), |_| callbacks += 1);
    assert_eq!(callbacks, 0);
    assert_eq!(outcome.status, Status::Invalid);
    assert_eq!(outcome.exit_code(), 2);
    let document: Value = serde_json::from_str(&outcome.document).unwrap();
    assert!(document["input_sha256"].is_null());
    assert_eq!(document["cases"], serde_json::json!([]));
    assert_eq!(document["diagnostics"][0]["code"], "API-E-INPUT");
    assert!(document["diagnostics"][0]["cause"]
        .as_str()
        .unwrap()
        .contains(expected));
    assert!(validate_with(input, &CancelToken::new(), |_| callbacks += 1).is_err());
    assert_eq!(callbacks, 0);
}

fn items(count: usize) -> String {
    std::iter::repeat_n("null", count)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn bytes_are_utf8_bytes_and_checked_before_any_json_syntax() {
    let at_limit = format!("\"{}\"", "é".repeat((MAX_INPUT_BYTES - 2) / 2));
    assert_eq!(at_limit.len(), MAX_INPUT_BYTES);
    assert!(canonical_json(&at_limit).is_ok());
    // Incluso texto inválido obtiene el error de tamaño, antes que el de sintaxis.
    invalid_before_parse(&"x".repeat(MAX_INPUT_BYTES + 1), "MAX_INPUT_BYTES");
}

#[test]
fn each_entity_limit_accepts_boundary_and_rejects_next_before_parsing() {
    for (prefix, suffix, name, limit) in [
        (r#"{"model":{"nodes":["#, "]}}", "MAX_NODES", MAX_NODES),
        (
            r#"{"model":{"members":["#,
            "]}}",
            "MAX_MEMBERS",
            MAX_MEMBERS,
        ),
        (r#"{"cases":["#, "]}", "MAX_CASES", MAX_CASES),
        (
            r#"{"combinations":["#,
            "]}",
            "MAX_COMBINATIONS",
            MAX_COMBINATIONS,
        ),
        (r#"{"cases":[{"loads":["#, "]}]}", "MAX_LOADS", MAX_LOADS),
        (
            r#"{"cases":[{"member_loads":["#,
            "]}]}",
            "MAX_LOADS",
            MAX_LOADS,
        ),
        (
            r#"{"auxiliary":["#,
            "]}",
            "MAX_JSON_ARRAY_ITEMS",
            MAX_JSON_ARRAY_ITEMS,
        ),
    ] {
        assert!(canonical_json(&format!("{prefix}{}{suffix}", items(limit))).is_ok());
        let oversized = format!("{prefix}{}{suffix}", items(limit + 1));
        invalid_before_parse(&oversized, name);
        // Sin cierres: la inspección debe detenerse al superar el límite, sin
        // intentar construir el árbol ni devolver primero el error de sintaxis.
        invalid_before_parse(&format!("{prefix}{}", items(limit + 1)), name);
    }
}

#[test]
fn loads_are_cumulative_across_cases_and_both_load_kinds() {
    let input = format!(
        r#"{{"cases":[{{"loads":[{}],"member_loads":[null]}},{{"loads":[null]}}]}}"#,
        items(MAX_LOADS - 1)
    );
    invalid_before_parse(&input, "MAX_LOADS");
    let boundary = input.replace(r#",{"loads":[null]}"#, "");
    assert!(canonical_json(&boundary).is_ok());
}

#[test]
fn depth_counts_arrays_and_objects_and_ignores_string_contents() {
    let boundary = format!(
        "{}0{}",
        "[".repeat(MAX_JSON_DEPTH),
        "]".repeat(MAX_JSON_DEPTH)
    );
    assert!(canonical_json(&boundary).is_ok());
    invalid_before_parse(&format!("[{boundary}]"), "MAX_JSON_DEPTH");
    let objects = format!(
        "{}null{}",
        r#"{"a":"#.repeat(MAX_JSON_DEPTH),
        "}".repeat(MAX_JSON_DEPTH)
    );
    assert!(canonical_json(&objects).is_ok());
    invalid_before_parse(&format!("[{objects}]"), "MAX_JSON_DEPTH");
    assert!(
        canonical_json(r#"{"text":"\\\"[{},123456789012345678901234567890]","empty":[]}"#).is_ok()
    );
}

#[test]
fn escaped_keys_enforce_limits_without_counting_member_node_references() {
    invalid_before_parse(
        &format!(
            r#"{{"mo\u0064el":{{"no\u0064es":[{}]}}}}"#,
            items(MAX_NODES + 1)
        ),
        "MAX_NODES",
    );
    let members = std::iter::repeat_n(r#"{"nodes":[1,2]}"#, MAX_MEMBERS)
        .collect::<Vec<_>>()
        .join(",");
    assert!(canonical_json(&format!(r#"{{"model":{{"members":[{members}]}}}}"#)).is_ok());
}

#[test]
fn integers_outside_jcs_safe_range_are_never_rounded_to_float() {
    for token in [
        "9007199254740992",
        "-9007199254740992",
        "9007199254740993",
        "9223372036854775807",
        "9223372036854775808",
        "18446744073709551615",
        "-9223372036854775808",
        "18446744073709551616",
        "-9223372036854775809",
        "-18446744073709551616",
        "99999999999999999999999999999999999999999999999999",
    ] {
        invalid_before_parse(token, "Entero JSON fuera de rango");
        // También se rechaza en un campo f64 antes de la conversión del DTO.
        invalid_before_parse(
            &format!(r#"{{"model":{{"nodes":[{{"coordinates":[{token},0,0]}}]}}}}"#),
            "Entero JSON fuera de rango",
        );
    }
    invalid_before_parse(&"9".repeat(10_000), "Entero JSON fuera de rango");
    for token in ["9007199254740991", "-9007199254740991"] {
        assert_eq!(canonical_json(token).unwrap(), token);
        assert!(input_sha256(token).is_ok());
    }
}

#[test]
fn safe_integer_boundaries_are_accepted_in_engine_input() {
    let mut input: Value = serde_json::from_str(include_str!(
        "../fixtures/input/valid/cantilever-linear-static.json"
    ))
    .unwrap();
    // Carga sobre el apoyo: prueba el parseo sin grandes desplazamientos ni
    // cancelación numérica en la recuperación del miembro.
    input["cases"][0]["loads"][0]["node"] = 1.into();
    for n in [9007199254740991_i64, -9007199254740991] {
        input["cases"][0]["loads"][0]["values"][0]["value"] = n.into();
        let source = input.to_string();
        assert!(validate(&source).unwrap().is_empty());
        let outcome = run(&source, &CancelToken::new(), |_| {});
        assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
        let document: Value = serde_json::from_str(&outcome.document).unwrap();
        assert_eq!(document["input_sha256"], input_sha256(&source).unwrap());
    }
    // bool es JSON válido, pero nunca un id ni contador entero.
    input["model"]["nodes"][0]["id"] = true.into();
    assert!(matches!(
        validate(&input.to_string()),
        Err(ApiError::InvalidInput(_))
    ));
    input["model"]["nodes"][0]["id"] = 1.into();
    input["analyses"] = serde_json::json!([{"method":"modal","options":{
        "mode_count":true,"mass_kind":"lumped","directions":[[1,0,0]]
    }}]);
    assert_eq!(
        run(&input.to_string(), &CancelToken::new(), |_| {}).status,
        Status::Invalid
    );
}

#[test]
fn float_overflow_is_invalid_including_negative_exponents_out_of_range() {
    for token in ["-1e400", "1e400", "1.7976931348623159e308"] {
        invalid_before_parse(token, "fuera del rango finito f64");
    }
    assert!(canonical_json("18446744073709551616.0").is_ok());
    assert_eq!(canonical_json("-0e10").unwrap(), "0.0");
}

#[test]
fn prior_cancellation_wins_over_oversized_input() {
    let input = "x".repeat(MAX_INPUT_BYTES + 1);
    let cancel = CancelToken::new();
    cancel.cancel();
    let mut callbacks = 0;
    assert_eq!(run(&input, &cancel, |_| callbacks += 1).exit_code(), 5);
    assert_eq!(
        validate_with(&input, &cancel, |_| callbacks += 1),
        Err(ApiError::Cancelled)
    );
    assert_eq!(callbacks, 0);
}
