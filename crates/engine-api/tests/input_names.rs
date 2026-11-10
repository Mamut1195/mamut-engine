//! Public vocabulary guard; imports no code and queries no other engines.
use std::{collections::BTreeSet, fs, path::Path};

const FORBIDDEN: &str = include_str!("../../../clean-room/forbidden_identifiers.txt");
const COMMAND_KEYS: &[&str] = &["ops", "pattern", "constraints", "system", "test", "element"];

// Keywords of the JSON Schema 2020-12 dialect used by the contracts.
// Explicitly enumerated: an unknown key is never treated as a keyword.
const SCHEMA_KEYWORDS: &[&str] = &[
    "$schema",
    "$id",
    "$defs",
    "$ref",
    "type",
    "properties",
    "required",
    "additionalProperties",
    "oneOf",
    "anyOf",
    "allOf",
    "items",
    "prefixItems",
    "minItems",
    "maxItems",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "enum",
    "const",
    "pattern",
    "description",
    "title",
    "format",
    "if",
    "then",
    "else",
    "not",
    "default",
    "examples",
];

// Frozen by MAM-480. engine-input/2 repeats the same $defs names and adds none.
// They authorize neither camelCase fields nor new definitions and apply only
// directly under $defs of the indicated $id.
const LEGACY_DEFS: &[(&str, &[&str])] = &[
    (
        "urn:mamut:engine-input:1",
        &[
            "uniformLoad",
            "dofValue",
            "massSource",
            "pDeltaOptions",
            "modalOptions",
            "spectrumTable",
            "spectrumDirection",
            "spectrumOptions",
        ],
    ),
    (
        "urn:mamut:engine-input:2",
        &[
            "uniformLoad",
            "dofValue",
            "massSource",
            "pDeltaOptions",
            "modalOptions",
            "spectrumTable",
            "spectrumDirection",
            "spectrumOptions",
        ],
    ),
    (
        "urn:mamut:results:1",
        &[
            "nodeIdentity",
            "memberIdentity",
            "fieldExtremes",
            "spectrumResponse",
            "caseFactor",
            "pDeltaIteration",
            "pDeltaConvergence",
        ],
    ),
];

fn identifiers() -> Vec<&'static str> {
    let names: Vec<_> = FORBIDDEN
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert!(!names.is_empty(), "The clean-room list cannot be empty");
    names
}

fn normalized(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '_' | '-'))
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn forbidden_word(text: &str, names: &[&str]) -> Option<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        // Preserves the previous detection of hyphen-separated words in text.
        .chain(text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_'))
        .find(|word| {
            names
                .iter()
                .any(|name| normalized(word) == normalized(name))
        })
        .map(str::to_owned)
}

fn ascii_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn format_character(c: char) -> bool {
    // Unicode General_Category=Cf (15.1); explicit ranges avoid a new dependency.
    matches!(
        c,
        '\u{00ad}'
            | '\u{0600}'..='\u{0605}'
            | '\u{061c}'
            | '\u{06dd}'
            | '\u{070f}'
            | '\u{0890}'..='\u{0891}'
            | '\u{08e2}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206f}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
            | '\u{110bd}'
            | '\u{110cd}'
            | '\u{13430}'..='\u{1343f}'
            | '\u{1bca0}'..='\u{1bca3}'
            | '\u{1d173}'..='\u{1d17a}'
            | '\u{e0001}'
            | '\u{e0020}'..='\u{e007f}'
    )
}

fn cyrillic_letter(c: char) -> bool {
    // Cyrillic and its phonetic, modifier and supplementary letters.
    // Greek mathematical notation and other non-ASCII text stay valid.
    c.is_alphabetic()
        && matches!(
            c,
            '\u{0400}'..='\u{052f}'
                | '\u{1c80}'..='\u{1c8a}'
                | '\u{1d2b}'
                | '\u{1d78}'
                | '\u{a640}'..='\u{a69f}'
                | '\u{1e030}'..='\u{1e06d}'
        )
}

// Name tree: preserves duplicate keys and does not convert numbers to f64/u64.
// This also inspects negative fixtures with 1e400 and integer overflow.
// serde_json decodes strings/escapes and validates scalars with IgnoredAny;
// validate/run separately checks the validity of the fixture contract.
#[derive(Debug)]
enum Node {
    Object(Vec<(String, Node)>),
    Array(Vec<Node>),
    String(String),
    Scalar,
}

fn parse_string(text: &str) -> (String, &str) {
    let bytes = text.as_bytes();
    assert_eq!(bytes.first(), Some(&b'"'));
    let mut end = 1;
    while end < bytes.len() && bytes[end] != b'"' {
        if bytes[end] == b'\\' {
            end += 1;
        }
        end += 1;
    }
    assert!(end < bytes.len(), "Unclosed JSON string");
    end += 1;
    (serde_json::from_str(&text[..end]).unwrap(), &text[end..])
}

fn parse_node(text: &str) -> (Node, &str) {
    let text = text.trim_start();
    match text.as_bytes().first() {
        Some(b'"') => {
            let (value, rest) = parse_string(text);
            (Node::String(value), rest)
        }
        Some(b'{') | Some(b'[') => {
            let object = text.starts_with('{');
            let close = if object { '}' } else { ']' };
            let mut rest = text[1..].trim_start();
            let mut fields = Vec::new();
            let mut values = Vec::new();
            if !rest.starts_with(close) {
                loop {
                    let key = if object {
                        let (key, tail) = parse_string(rest);
                        rest = tail.trim_start().strip_prefix(':').unwrap().trim_start();
                        Some(key)
                    } else {
                        None
                    };
                    let (value, tail) = parse_node(rest);
                    if let Some(key) = key {
                        fields.push((key, value));
                    } else {
                        values.push(value);
                    }
                    rest = tail.trim_start();
                    if rest.starts_with(close) {
                        break;
                    }
                    rest = rest.strip_prefix(',').unwrap().trim_start();
                }
            }
            let node = if object {
                Node::Object(fields)
            } else {
                Node::Array(values)
            };
            (node, rest.strip_prefix(close).unwrap())
        }
        _ => {
            let end = text
                .find(|c: char| c.is_whitespace() || matches!(c, ',' | '}' | ']'))
                .unwrap_or(text.len());
            let _: serde::de::IgnoredAny = serde_json::from_str(&text[..end]).unwrap();
            (Node::Scalar, &text[end..])
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Context {
    Data,
    Schema,
    Names,
}

fn element_allowed(path: &[&str], context: Context, schema_id: &str) -> bool {
    match context {
        Context::Data => path == ["model", "members", "[]"],
        Context::Names => {
            ((schema_id == "urn:mamut:engine-input:1"
                || schema_id == "urn:mamut:engine-input:2")
                && path == ["$defs", "member", "properties"])
                // Contract exception PENDING: #88 also publishes the member kind
                // in results/1. Only this property, without extension.
                || (schema_id == "urn:mamut:results:1"
                    && path == ["$defs", "memberIdentity", "properties"])
        }
        Context::Schema => false,
    }
}

fn inspect<'a>(
    node: &'a Node,
    context: Context,
    schema_id: &str,
    path: &mut Vec<&'a str>,
    names: &[&str],
) -> Option<String> {
    match node {
        Node::Object(fields) => {
            for (key, value) in fields {
                if let Some(name) = forbidden_word(key, names) {
                    return Some(name);
                }
                let command = normalized(key);
                let regex_keyword = context == Context::Schema
                    && key == "pattern"
                    && matches!(value, Node::String(_));
                let member_selector = key == "element" && element_allowed(path, context, schema_id);
                if COMMAND_KEYS.contains(&command.as_str()) && !regex_keyword && !member_selector {
                    return Some(key.clone());
                }
                if context == Context::Schema {
                    if !SCHEMA_KEYWORDS.contains(&key.as_str()) {
                        return Some(key.clone());
                    }
                } else {
                    let legacy_def = context == Context::Names
                        && path == &["$defs"]
                        && LEGACY_DEFS
                            .iter()
                            .any(|(id, defs)| *id == schema_id && defs.contains(&key.as_str()));
                    if !ascii_name(key) && !legacy_def {
                        return Some(key.clone());
                    }
                }
                let next = if context == Context::Schema {
                    match key.as_str() {
                        "properties" | "$defs" => Context::Names,
                        "enum" | "const" | "default" | "examples" => Context::Data,
                        _ => Context::Schema,
                    }
                } else if context == Context::Names {
                    Context::Schema
                } else {
                    Context::Data
                };
                path.push(key);
                let violation = inspect(value, next, schema_id, path, names);
                path.pop();
                if violation.is_some() {
                    return violation;
                }
            }
            None
        }
        Node::Array(values) => {
            path.push("[]");
            let violation = values
                .iter()
                .find_map(|value| inspect(value, context, schema_id, path, names));
            path.pop();
            violation
        }
        Node::String(value) => forbidden_word(value, names).or_else(|| {
            value
                .chars()
                .any(|c| format_character(c) || cyrillic_letter(c))
                .then(|| value.clone())
        }),
        Node::Scalar => None,
    }
}

fn violation(text: &str, schema: bool) -> Option<String> {
    let names = identifiers();
    if let Some(name) = forbidden_word(text, &names) {
        return Some(name);
    }
    let (node, rest) = parse_node(text);
    assert!(rest.trim().is_empty(), "Content after the JSON document");
    let schema_id = match &node {
        Node::Object(fields) if schema => fields
            .iter()
            .find_map(|(key, value)| {
                if let ("$id", Node::String(id)) = (key.as_str(), value) {
                    Some(id.as_str())
                } else {
                    None
                }
            })
            .unwrap_or(""),
        _ => "",
    };
    let context = if schema {
        Context::Schema
    } else {
        Context::Data
    };
    inspect(&node, context, schema_id, &mut Vec::new(), &names)
}

fn check_tree(path: &Path, schema: bool) {
    let mut entries: Vec<_> = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "{path:?}: missing documents");
    for entry in entries {
        if entry.is_dir() {
            check_tree(&entry, schema);
        } else {
            let text = fs::read_to_string(&entry).unwrap();
            assert_eq!(violation(&text, schema), None, "{entry:?}");
        }
    }
}

#[test]
fn schemas_fixtures_and_public_input_field_names_are_clean() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    check_tree(&root.join("schemas"), true);
    check_tree(&root.join("fixtures"), false);
    for kind in ["engine-input/1", "engine-input/2", "results/1"] {
        let schema = engine_api::schema(kind).unwrap();
        assert_eq!(violation(schema, true), None, "{kind}");
        let _: serde_json::Value = serde_json::from_str(schema).unwrap();
    }
}

#[test]
fn legacy_definitions_are_frozen_by_mam_480() {
    let expected: &[(&str, &[&str])] = &[
        (
            "urn:mamut:engine-input:1",
            &[
                "uniformLoad",
                "dofValue",
                "massSource",
                "pDeltaOptions",
                "modalOptions",
                "spectrumTable",
                "spectrumDirection",
                "spectrumOptions",
            ],
        ),
        (
            "urn:mamut:engine-input:2",
            &[
                "uniformLoad",
                "dofValue",
                "massSource",
                "pDeltaOptions",
                "modalOptions",
                "spectrumTable",
                "spectrumDirection",
                "spectrumOptions",
            ],
        ),
        (
            "urn:mamut:results:1",
            &[
                "nodeIdentity",
                "memberIdentity",
                "fieldExtremes",
                "spectrumResponse",
                "caseFactor",
                "pDeltaIteration",
                "pDeltaConvergence",
            ],
        ),
    ];
    assert_eq!(
        LEGACY_DEFS, expected,
        "frozen contract IDs, order and names"
    );
    let names: Vec<_> = LEGACY_DEFS.iter().flat_map(|(_, defs)| *defs).collect();
    assert_eq!(names.len(), 23);
    assert_eq!(
        names.iter().collect::<BTreeSet<_>>().len(),
        15,
        "engine-input/2 repeats the existing names and adds none"
    );
}

#[test]
fn frozen_legacy_definitions_exist_in_the_published_schemas() {
    for (kind, &(id, names)) in ["engine-input/1", "engine-input/2", "results/1"]
        .into_iter()
        .zip(LEGACY_DEFS)
    {
        let schema: serde_json::Value =
            serde_json::from_str(engine_api::schema(kind).unwrap()).unwrap();
        assert_eq!(schema["$id"], id, "{kind}");
        let defs = schema["$defs"].as_object().expect("published root $defs");
        for name in names {
            assert!(defs.contains_key(*name), "{kind}: missing /$defs/{name}");
        }
        let non_snake_case: BTreeSet<_> = defs
            .keys()
            .map(String::as_str)
            .filter(|name| !ascii_name(name))
            .collect();
        assert_eq!(
            non_snake_case,
            names.iter().copied().collect(),
            "{kind}: no other non-snake_case definitions"
        );
    }
}

#[test]
fn guard_detects_identifiers_escaped_keys_and_nested_command_keys() {
    for name in identifiers() {
        for spelling in [name.to_owned(), name.to_ascii_uppercase(), normalized(name)] {
            for value in [
                serde_json::json!({spelling.clone(): 0}),
                serde_json::json!({"description": format!("({spelling})")}),
            ] {
                assert_eq!(violation(&value.to_string(), false), Some(spelling.clone()));
            }
        }
        let description = serde_json::json!({"description": format!("prefix-{name}")});
        assert_eq!(
            violation(&description.to_string(), false),
            Some(name.into())
        );
        let escaped = name
            .bytes()
            .map(|byte| format!("\\u{byte:04x}"))
            .collect::<String>();
        assert_eq!(
            violation(&format!("{{\"{escaped}\":0}}"), false),
            Some(name.into())
        );
    }
    for &key in COMMAND_KEYS {
        assert_eq!(
            violation(
                &serde_json::json!({"model": [{key: []}]}).to_string(),
                false
            ),
            Some(key.into())
        );
    }
    assert_eq!(
        violation(r#"{"model":{"\u006f\u0070\u0073":[]}}"#, false),
        Some("ops".into())
    );
    assert_eq!(
        violation(
            r#"{"name":"system","id":"fixed","model":{"members":[{"element":"truss2d.linear"}]},"value":1e400,"integer":18446744073709551616}"#,
            false
        ),
        None
    );
}

#[test]
fn schema_regex_keyword_does_not_allow_public_or_fixture_command_keys() {
    let regex = r#"{"type":"string","pattern":"^[a-z]+$"}"#;
    assert_eq!(violation(regex, true), None);
    assert_eq!(violation(regex, false), Some("pattern".into()));
    assert_eq!(violation(r#"{"pattern":{}}"#, true), Some("pattern".into()));
    for key in COMMAND_KEYS {
        let field = serde_json::json!({"properties": {(*key): {"type": "string"}}});
        assert_eq!(violation(&field.to_string(), true), Some((*key).into()));
    }
}

fn rejects_key(key: &str) {
    let fixture = serde_json::json!({"nested": [{key: 0}]}).to_string();
    let property = serde_json::json!({"properties": {key: {"type": "string"}}}).to_string();
    let definition = serde_json::json!({"$defs": {key: {"type": "string"}}}).to_string();
    let keyword = serde_json::json!({key: "value"}).to_string();
    let escaped = key
        .encode_utf16()
        .map(|unit| format!("\\u{unit:04x}"))
        .collect::<String>();
    let escaped_fixture = format!("{{\"{escaped}\":0}}");
    for (text, schema) in [
        (fixture, false),
        (property, true),
        (definition, true),
        (keyword, true),
        (escaped_fixture, false),
    ] {
        assert_eq!(violation(&text, schema), Some(key.into()), "{text}");
    }
}

macro_rules! negative_key_test {
    ($test:ident, $key:literal) => {
        #[test]
        fn $test() {
            rejects_key($key);
        }
    };
}

negative_key_test!(rejects_uppercase_ops, "OPS");
negative_key_test!(rejects_titlecase_ops, "Ops");
negative_key_test!(rejects_titlecase_pattern, "Pattern");
negative_key_test!(rejects_titlecase_transform, "GeomTransf");
negative_key_test!(rejects_lowercase_transform, "geomtransf");
negative_key_test!(rejects_lowercase_material, "uniaxialmaterial");
negative_key_test!(rejects_underscore_transform, "geom_transf");
negative_key_test!(rejects_cyrillic_ops, "оps");
negative_key_test!(rejects_cyrillic_transform, "geomTrаnsf");
negative_key_test!(rejects_zero_width_space, "ops\u{200b}");
negative_key_test!(rejects_trailing_space, "ops ");
negative_key_test!(rejects_hyphen_transform, "geom-transf");
negative_key_test!(rejects_underscore_command, "o_p_s");
negative_key_test!(rejects_hyphen_command, "o-p-s");
negative_key_test!(rejects_unlisted_camelcase, "newField");

#[test]
fn rejects_unknown_schema_keyword() {
    assert_eq!(
        violation(r#"{"unknown_keyword":"value"}"#, true),
        Some("unknown_keyword".into())
    );
    assert_eq!(
        violation(r#"{"properties":{"field":{"unknown_keyword":0}}}"#, true),
        Some("unknown_keyword".into())
    );
    assert_eq!(violation(r#"{"unknown_keyword":0}"#, false), None);
    assert_eq!(
        violation(
            r#"{"properties":{"unknown_keyword":{"type":"number"}}}"#,
            true
        ),
        None
    );
}

#[test]
fn element_is_rejected_outside_the_member_paths() {
    for fixture in [
        r#"{"element":"truss2d.linear"}"#,
        r#"{"model":{"element":"truss2d.linear"}}"#,
        r#"{"model":{"members":{"element":"truss2d.linear"}}}"#,
        r#"{"model":{"members":[{"nested":{"element":"truss2d.linear"}}]}}"#,
        r#"{"nested":{"model":{"members":[{"element":"truss2d.linear"}]}}}"#,
        r#"{"model":{"nodes":[{"element":"truss2d.linear"}]}}"#,
        r#"{"element":0,"element":1}"#,
    ] {
        assert_eq!(
            violation(fixture, false),
            Some("element".into()),
            "{fixture}"
        );
    }
    for (id, member) in [
        ("urn:mamut:engine-input:1", "member"),
        ("urn:mamut:engine-input:2", "member"),
        ("urn:mamut:results:1", "memberIdentity"),
    ] {
        let schema = serde_json::json!({"$id": id, "$defs": {member: {"properties": {"element": {"type": "string"}}}}});
        assert_eq!(violation(&schema.to_string(), true), None);
        let misplaced = serde_json::json!({"$id": id, "$defs": {"node": {"properties": {"element": {"type": "string"}}}}});
        assert_eq!(
            violation(&misplaced.to_string(), true),
            Some("element".into())
        );
    }
    assert_eq!(
        violation(
            r#"{"$defs":{"member":{"properties":{"element":{"type":"string"}}}}}"#,
            true
        ),
        Some("element".into())
    );
}

#[test]
fn legacy_definition_exceptions_are_scoped_to_the_published_contracts() {
    for &(id, defs) in LEGACY_DEFS {
        for &name in defs {
            assert!(!ascii_name(name));
            let schema = serde_json::json!({"$id": id, "$defs": {name: {"type": "object"}}});
            assert_eq!(violation(&schema.to_string(), true), None);
            rejects_key(name);
            let wrong_contract = serde_json::json!({"$id": "urn:mamut:future:1", "$defs": {name: {"type": "object"}}});
            assert_eq!(
                violation(&wrong_contract.to_string(), true),
                Some(name.into())
            );
            let property = serde_json::json!({"$id": id, "properties": {name: {"type": "string"}}});
            assert_eq!(violation(&property.to_string(), true), Some(name.into()));
            let nested_def = serde_json::json!({"$id": id, "$defs": {"nested": {"$defs": {name: {"type": "object"}}}}});
            assert_eq!(violation(&nested_def.to_string(), true), Some(name.into()));
        }
    }
}

fn rejects_string_value(value: &str) {
    // Test decoded scalars as well as JSON Unicode escapes, including surrogate pairs.
    let escaped = value
        .encode_utf16()
        .map(|unit| format!("\\u{unit:04x}"))
        .collect::<String>();
    for encoded in [
        serde_json::to_string(value).unwrap(),
        format!("\"{escaped}\""),
    ] {
        for (text, schema) in [
            (
                format!(r#"{{"nested":{{"description":{encoded}}}}}"#),
                false,
            ),
            (format!(r#"{{"nested":[{encoded}]}}"#), false),
            (
                format!(r#"{{"properties":{{"field":{{"description":{encoded}}}}}}}"#),
                true,
            ),
            (format!(r#"{{"examples":[{encoded}]}}"#), true),
        ] {
            assert_eq!(violation(&text, schema), Some(value.into()), "{text}");
        }
    }
}

#[test]
fn string_values_reject_cyrillic_confusables() {
    rejects_string_value("descripti\u{043e}n");
    for c in [
        '\u{0501}',
        '\u{1c80}',
        '\u{1d2b}',
        '\u{1d78}',
        '\u{a641}',
        '\u{1e030}',
    ] {
        rejects_string_value(&format!("a{c}b"));
    }
}

#[test]
fn descriptions_allow_greek_mathematical_notation() {
    for text in [
        r#"{"description":"δ/γ/φ"}"#,
        r#"{"description":"\u03b4/\u03b3/\u03c6"}"#,
    ] {
        assert_eq!(violation(text, true), None, "{text}");
        assert_eq!(violation(text, false), None, "{text}");
    }
}

#[test]
fn string_values_reject_zero_width_and_bidi_characters() {
    rejects_string_value("a\u{200b}b");
    rejects_string_value("\u{202e}text");
    for c in ['\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}']
        .into_iter()
        .chain('\u{202a}'..='\u{202e}')
        .chain('\u{2066}'..='\u{2069}')
    {
        rejects_string_value(&format!("a{c}b"));
    }
}

#[test]
fn string_values_reject_other_format_characters() {
    for c in [
        '\u{00ad}',
        '\u{0600}',
        '\u{061c}',
        '\u{06dd}',
        '\u{070f}',
        '\u{0890}',
        '\u{08e2}',
        '\u{180e}',
        '\u{200e}',
        '\u{200f}',
        '\u{2061}',
        '\u{206f}',
        '\u{fff9}',
        '\u{110bd}',
        '\u{110cd}',
        '\u{13430}',
        '\u{1bca0}',
        '\u{1d173}',
        '\u{e0001}',
        '\u{e0020}',
        '\u{e007f}',
    ] {
        rejects_string_value(&format!("a{c}b"));
    }
}

#[test]
fn string_values_allow_other_non_ascii_text() {
    for value in ["descripción", "café", "m² · m³ − 1", "Mᵀ", "\u{03e2}"] {
        let text = serde_json::json!({"description": value}).to_string();
        assert_eq!(violation(&text, true), None, "{text}");
        assert_eq!(violation(&text, false), None, "{text}");
    }
}

#[test]
fn keyword_spellings_do_not_exempt_data_keys() {
    assert_eq!(violation(r#"{"minItems":1}"#, true), None);
    assert_eq!(
        violation(r#"{"minItems":1}"#, false),
        Some("minItems".into())
    );
    assert_eq!(
        violation(r#"{"properties":{"minItems":{"type":"number"}}}"#, true),
        Some("minItems".into())
    );
    assert_eq!(
        violation(r#"{"const":{"minItems":1}}"#, true),
        Some("minItems".into())
    );
    assert_eq!(
        violation(r#"{"properties":{"properties":{"type":"string"}}}"#, true),
        None
    );
    assert_eq!(
        violation(r#"{"bad key":0,"bad key":1}"#, false),
        Some("bad key".into())
    );
}

#[test]
fn escaped_unicode_keys_cannot_bypass_the_guard() {
    for key in [
        "OPS",
        "оps",
        "geomTrаnsf",
        "ops\u{200b}",
        "ops ",
        "geom_transf",
    ] {
        let escaped = key
            .chars()
            .map(|c| format!("\\u{:04x}", u32::from(c)))
            .collect::<String>();
        for (text, schema) in [
            (format!("{{\"nested\":[{{\"{escaped}\":0}}]}}"), false),
            (format!("{{\"properties\":{{\"{escaped}\":{{}}}}}}"), true),
            (format!("{{\"$defs\":{{\"{escaped}\":{{}}}}}}"), true),
        ] {
            assert_eq!(violation(&text, schema), Some(key.into()), "{text}");
        }
    }
    assert_eq!(
        violation(r#"{"properties":{"\u043eps":{}},"properties":{}}"#, true),
        Some("оps".into())
    );
}

#[test]
fn ascii_data_key_grammar_has_no_implicit_exceptions() {
    for key in [
        "",
        "_x",
        "2d",
        "a-b",
        "name space",
        "café",
        "a\nb",
        "a\u{200b}",
    ] {
        rejects_key(key);
    }
    for key in ["a", "node2", "some_value"] {
        assert_eq!(
            violation(&serde_json::json!({key: 0}).to_string(), false),
            None
        );
        assert_eq!(
            violation(
                &serde_json::json!({"properties": {key: {}}}).to_string(),
                true
            ),
            None
        );
    }
}

#[test]
fn separator_normalization_also_applies_to_text_and_member_selectors() {
    for name in identifiers() {
        for separator in ["_", "-"] {
            let spelling = name
                .chars()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(separator);
            let text = serde_json::json!({"description": spelling.clone()}).to_string();
            assert_eq!(violation(&text, false), Some(spelling));
        }
    }
    for key in ["Element", "ELEMENT", "element_", "e_le_ment"] {
        let text = serde_json::json!({"model": {"members": [{key: "truss2d.linear"}]}}).to_string();
        assert_eq!(violation(&text, false), Some(key.into()));
    }
}
