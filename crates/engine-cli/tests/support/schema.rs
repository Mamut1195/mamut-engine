// SPDX-License-Identifier: AGPL-3.0-only

//! Subconjunto Draft 2020-12 del validador local de engine-api/tests/support.
//! Sin dependencias nuevas ni resolución remota; keywords desconocidas fallan.
use serde_json::Value;

pub fn assert_error_stream(bytes: &[u8], exit_code: i32) -> Value {
    let text = std::str::from_utf8(bytes).unwrap();
    assert!(text.ends_with('\n'), "Sobre sin terminador NDJSON: {text}");
    let mut errors = Vec::new();
    for line in text.lines() {
        let value: Value = serde_json::from_str(line).unwrap();
        if value["schema"] == "cli-error/1" {
            assert!(conforms(&value), "cli-error/1 fuera de schema: {value}");
            assert_registered(&value);
            assert_eq!(value["exitCode"], exit_code);
            errors.push(value);
        } else {
            assert!(errors.is_empty(), "Progreso tras el sobre terminal");
            assert!(
                matches!(value["phase"].as_str(), Some("started" | "finished")),
                "Emisor ajeno al contrato: {value}"
            );
        }
    }
    assert_eq!(
        errors.len(),
        1,
        "Debe haber un único error terminal: {text}"
    );
    errors.remove(0)
}

fn assert_registered(envelope: &Value) {
    let registry: Value = serde_json::from_str(include_str!(
        "../../../../docs/contracts/error-codes-1.json"
    ))
    .unwrap();
    let codes = registry["codes"].as_array().unwrap();
    let definition = codes
        .iter()
        .find(|c| c["id"] == envelope["code"])
        .expect("Código no registrado");
    assert_eq!(definition["exitCode"], envelope["exitCode"]);
    assert_eq!(definition["outcome"], envelope["outcome"]);
    assert_eq!(definition["messageKey"], envelope["messageKey"]);
    assert_eq!(envelope["error"]["code"], envelope["code"]);
    let messages: Value = serde_json::from_str(include_str!("../../messages/en.json")).unwrap();
    assert_eq!(
        envelope["error"]["message"],
        messages[definition["messageKey"].as_str().unwrap()]
    );
    for diagnostic in envelope["diagnostics"].as_array().unwrap() {
        assert!(
            codes.iter().any(|c| c["id"] == diagnostic["code"]),
            "Diagnóstico no registrado: {diagnostic}"
        );
    }
}

fn audit(schema: &Value) {
    if schema.is_boolean() {
        return;
    }
    for (key, value) in schema.as_object().unwrap() {
        match key.as_str() {
            "$schema" | "$id" | "title" | "description" | "type" | "required" | "enum"
            | "const" | "minItems" | "maxItems" | "minimum" | "maximum" => {}
            "$ref" => assert!(value.as_str().unwrap().starts_with("#/$defs/")),
            "pattern" => {
                matches_pattern(value.as_str().unwrap(), "");
            }
            "properties" | "$defs" => {
                for child in value.as_object().unwrap().values() {
                    audit(child);
                }
            }
            "oneOf" | "allOf" | "prefixItems" => {
                for child in value.as_array().unwrap() {
                    audit(child);
                }
            }
            "items" | "additionalProperties" | "if" | "then" | "else" => audit(value),
            _ => panic!("Keyword de schema no soportada: {key}"),
        }
    }
}

pub fn conforms(value: &Value) -> bool {
    let root: Value = serde_json::from_str(include_str!("../../schemas/cli-error-1.json")).unwrap();
    audit(&root);
    valid(&root, &root, value)
}

fn valid(root: &Value, schema: &Value, value: &Value) -> bool {
    if let Some(allowed) = schema.as_bool() {
        return allowed;
    }
    let rules = schema.as_object().unwrap();
    rules.iter().all(|(key, rule)| match key.as_str() {
        "$schema" | "$id" | "$defs" | "title" | "description" | "then" | "else" => true,
        "$ref" => {
            let reference = rule.as_str().unwrap().strip_prefix('#').unwrap();
            valid(
                root,
                root.pointer(reference).expect("Referencia local ausente"),
                value,
            )
        }
        "type" => match rule {
            Value::Array(types) => types.iter().any(|t| has_type(t.as_str().unwrap(), value)),
            Value::String(name) => has_type(name, value),
            _ => panic!("type inválido"),
        },
        "const" => equal_json(rule, value),
        "enum" => rule
            .as_array()
            .unwrap()
            .iter()
            .any(|v| equal_json(v, value)),
        "required" => value.as_object().is_none_or(|object| {
            rule.as_array()
                .unwrap()
                .iter()
                .all(|name| object.contains_key(name.as_str().unwrap()))
        }),
        "properties" => value.as_object().is_none_or(|object| {
            rule.as_object()
                .unwrap()
                .iter()
                .all(|(name, child)| object.get(name).is_none_or(|v| valid(root, child, v)))
        }),
        "additionalProperties" => value.as_object().is_none_or(|object| {
            object.iter().all(|(name, v)| {
                schema
                    .get("properties")
                    .is_some_and(|p| p.get(name).is_some())
                    || valid(root, rule, v)
            })
        }),
        "items" => value.as_array().is_none_or(|items| {
            let prefix = schema
                .get("prefixItems")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            items.iter().skip(prefix).all(|v| valid(root, rule, v))
        }),
        "prefixItems" => value.as_array().is_none_or(|items| {
            items
                .iter()
                .zip(rule.as_array().unwrap())
                .all(|(v, child)| valid(root, child, v))
        }),
        "minItems" => value
            .as_array()
            .is_none_or(|items| items.len() as u64 >= rule.as_u64().unwrap()),
        "maxItems" => value
            .as_array()
            .is_none_or(|items| items.len() as u64 <= rule.as_u64().unwrap()),
        "minimum" => value.as_f64().is_none_or(|n| n >= rule.as_f64().unwrap()),
        "maximum" => value.as_f64().is_none_or(|n| n <= rule.as_f64().unwrap()),
        "pattern" => value
            .as_str()
            .is_none_or(|s| matches_pattern(rule.as_str().unwrap(), s)),
        "allOf" => rule
            .as_array()
            .unwrap()
            .iter()
            .all(|child| valid(root, child, value)),
        "oneOf" => {
            rule.as_array()
                .unwrap()
                .iter()
                .filter(|child| valid(root, child, value))
                .count()
                == 1
        }
        "if" => {
            let branch = if valid(root, rule, value) {
                "then"
            } else {
                "else"
            };
            schema
                .get(branch)
                .is_none_or(|child| valid(root, child, value))
        }
        _ => panic!("Keyword de schema no soportada: {key}"),
    })
}

fn has_type(name: &str, value: &Value) -> bool {
    match name {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "number" => value.is_number(),
        "integer" => value.as_f64().is_some_and(|v| v.fract() == 0.0),
        "string" => value.is_string(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => panic!("Tipo de schema no soportado: {name}"),
    }
}

// JSON Schema compara números por valor (0 y 0.0 satisfacen el mismo const).
fn equal_json(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(_), Value::Number(_)) => a.as_f64() == b.as_f64(),
        _ => a == b,
    }
}

fn matches_pattern(expression: &str, value: &str) -> bool {
    match expression {
        "^[0-9a-f]{64}$" => {
            value.len() == 64
                && value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }
        "^[0-9a-fA-F]{40}([0-9a-fA-F]{24})?$" => {
            matches!(value.len(), 40 | 64) && value.bytes().all(|b| b.is_ascii_hexdigit())
        }
        "^(ENG-E-[0-9]{4}|API-[EWI]-[A-Z-]+|CLI-E-[A-Z-]+)$" => {
            if let Some(code) = value.strip_prefix("ENG-E-") {
                code.len() == 4 && code.bytes().all(|b| b.is_ascii_digit())
            } else if let Some(code) = value.strip_prefix("API-") {
                let bytes = code.as_bytes();
                bytes.len() >= 3
                    && matches!(bytes[0], b'E' | b'W' | b'I')
                    && bytes[1] == b'-'
                    && bytes[2..]
                        .iter()
                        .all(|b| b.is_ascii_uppercase() || *b == b'-')
            } else if let Some(code) = value.strip_prefix("CLI-E-") {
                !code.is_empty() && code.bytes().all(|b| b.is_ascii_uppercase() || b == b'-')
            } else {
                false
            }
        }
        "^(?:/(?:[^~/]|~[01])*)*$" => {
            if value.is_empty() {
                return true;
            }
            if !value.starts_with('/') {
                return false;
            }
            let mut chars = value.chars();
            while let Some(c) = chars.next() {
                if c == '~' && !matches!(chars.next(), Some('0' | '1')) {
                    return false;
                }
            }
            true
        }
        _ => panic!("Expresión de schema no soportada: {expression}"),
    }
}
