//! Shared test validator for the Draft 2020-12 subset.
//! No network; rejects unknown keywords and patterns.
use serde_json::Value;

pub fn conforms(value: &Value, root: &Value) -> bool {
    audit(root);
    valid(root, root, value)
}

fn audit(schema: &Value) {
    if schema.is_boolean() {
        return;
    }
    for (key, value) in schema.as_object().unwrap() {
        match key.as_str() {
            "$schema" | "$id" | "title" | "description" | "type" | "required" | "enum"
            | "const" | "minItems" | "maxItems" | "minimum" | "maximum" => {}
            "$ref" => {
                let reference = value.as_str().unwrap();
                assert!(
                    reference.starts_with("#/$defs/")
                        || reference == "urn:mamut:results:1"
                        || reference == "urn:mamut:results:1#/$defs/diagnostic"
                );
            }
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
            "items" | "contains" | "additionalProperties" | "if" | "then" | "else" => audit(value),
            _ => panic!("Unsupported schema keyword: {key}"),
        }
    }
}

fn valid(root: &Value, schema: &Value, value: &Value) -> bool {
    if let Some(allowed) = schema.as_bool() {
        return allowed;
    }
    let rules = schema.as_object().unwrap();
    rules.iter().all(|(key, rule)| match key.as_str() {
        "$schema" | "$id" | "$defs" | "title" | "description" | "then" | "else" => true,
        "$ref" => {
            let reference = rule.as_str().unwrap();
            if let Some(fragment) = reference.strip_prefix("urn:mamut:results:1") {
                let external: Value =
                    serde_json::from_str(engine_api::schema("results/1").unwrap()).unwrap();
                audit(&external);
                let child = if fragment.is_empty() {
                    &external
                } else {
                    external
                        .pointer(fragment.strip_prefix('#').unwrap())
                        .expect("Missing local external reference")
                };
                return valid(&external, child, value);
            }
            let reference = reference.strip_prefix('#').unwrap();
            valid(
                root,
                root.pointer(reference).expect("Missing local reference"),
                value,
            )
        }
        "type" => match rule {
            Value::Array(types) => types.iter().any(|t| has_type(t.as_str().unwrap(), value)),
            Value::String(name) => has_type(name, value),
            _ => panic!("invalid type"),
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
        "contains" => value
            .as_array()
            .is_none_or(|items| items.iter().any(|item| valid(root, rule, item))),
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
        _ => panic!("Unsupported schema keyword: {key}"),
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
        _ => panic!("Unsupported schema type: {name}"),
    }
}

// JSON Schema compares numbers by value (0 and 0.0 satisfy the same const).
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
        "^(ENG-E-[0-9]{4}|API-[EWI]-[A-Z-]+)$"
        | "^(ENG-E-[0-9]{4}|API-[EWI]-[A-Z-]+|CLI-E-[A-Z-]+)$" => {
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
                expression == "^(ENG-E-[0-9]{4}|API-[EWI]-[A-Z-]+|CLI-E-[A-Z-]+)$"
                    && !code.is_empty()
                    && code.bytes().all(|b| b.is_ascii_uppercase() || b == b'-')
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
        _ => panic!("Unsupported schema expression: {expression}"),
    }
}
