// SPDX-License-Identifier: AGPL-3.0-only

use std::{collections::BTreeMap, fs, path::Path};

use serde_json::{json, Value};

const REGISTRY: &str = include_str!("../../../docs/contracts/error-codes-1.json");
const LOCK: &str = include_str!("../../../docs/contracts/error-codes-1.lock");

// Ancla independiente del registro y del lock. Cada alta exige añadir aquí su
// definición completa, sin reescribir las anteriores. Se compara texto semántico
// exacto: sin hashes débiles ni dependencias nuevas. No generar en tiempo de test.
const FROZEN_DEFINITIONS: &[&str] = &[
    r#"{"id":"API-E-CONSOLIDATION-COST","exitCode":2,"outcome":"invalid_input","description":"Consolidation cost, segment count or substep count exceeds its budget.","introduced":"1.0.0","messageKey":"errors.API-E-CONSOLIDATION-COST"}"#,
    r#"{"id":"API-E-CONSOLIDATION-WORK","exitCode":2,"outcome":"invalid_input","description":"Consolidation work exceeds the permitted budget.","introduced":"1.0.0","messageKey":"errors.API-E-CONSOLIDATION-WORK"}"#,
    r#"{"id":"API-E-INPUT","exitCode":2,"outcome":"invalid_input","description":"Invalid JSON or engine input structure.","introduced":"1.0.0","messageKey":"errors.API-E-INPUT"}"#,
    r#"{"id":"API-E-SFC-OPTIONS","exitCode":2,"outcome":"invalid_input","description":"Invalid or incompatible advanced analysis options.","introduced":"1.0.0","messageKey":"errors.API-E-SFC-OPTIONS"}"#,
    r#"{"id":"API-I-CANCELLED","exitCode":5,"outcome":"cancelled","description":"Cooperative cancellation was requested.","introduced":"1.0.0","messageKey":"errors.API-I-CANCELLED"}"#,
    r#"{"id":"API-W-COMBINED-PRESCRIBED","exitCode":1,"outcome":"warning","description":"Combined prescribed displacements require review.","introduced":"1.0.0","messageKey":"errors.API-W-COMBINED-PRESCRIBED"}"#,
    r#"{"id":"API-W-DIAPHRAGM-RECOVERY","exitCode":1,"outcome":"warning","description":"Diaphragm recovery has limited domain test coverage.","introduced":"1.0.0","messageKey":"errors.API-W-DIAPHRAGM-RECOVERY"}"#,
    r#"{"id":"API-W-MASS-PROJECTION","exitCode":1,"outcome":"warning","description":"Mass conversion rejects negative load projections without a tolerance.","introduced":"1.0.0","messageKey":"errors.API-W-MASS-PROJECTION"}"#,
    r#"{"id":"API-W-PDELTA-INERTIA","exitCode":1,"outcome":"warning","description":"P-Delta block inertia stability diagnostics require review.","introduced":"1.0.0","messageKey":"errors.API-W-PDELTA-INERTIA"}"#,
    r#"{"id":"CLI-E-CANCELLED","exitCode":5,"outcome":"cancelled","description":"Execution was cancelled.","introduced":"1.0.0","messageKey":"errors.CLI-E-CANCELLED"}"#,
    r#"{"id":"CLI-E-DOCUMENT","exitCode":2,"outcome":"invalid_input","description":"Input document could not be read or parsed.","introduced":"1.0.0","messageKey":"errors.CLI-E-DOCUMENT"}"#,
    r#"{"id":"CLI-E-INTERNAL","exitCode":70,"outcome":"internal","description":"Execution or output I/O could not be completed.","introduced":"1.0.0","messageKey":"errors.CLI-E-INTERNAL"}"#,
    r#"{"id":"CLI-E-METHOD","exitCode":70,"outcome":"internal","description":"Method failed without a semantic classification.","introduced":"1.0.0","messageKey":"errors.CLI-E-METHOD"}"#,
    r#"{"id":"CLI-E-NOT-CONVERGED","exitCode":4,"outcome":"not_converged","description":"Execution did not converge or is unstable.","introduced":"1.0.0","messageKey":"errors.CLI-E-NOT-CONVERGED"}"#,
    r#"{"id":"CLI-E-TIMEOUT","exitCode":6,"outcome":"timeout","description":"Execution timed out.","introduced":"1.0.0","messageKey":"errors.CLI-E-TIMEOUT"}"#,
    r#"{"id":"CLI-E-UNSUPPORTED","exitCode":3,"outcome":"unsupported","description":"Operation is not supported.","introduced":"1.0.0","messageKey":"errors.CLI-E-UNSUPPORTED"}"#,
    r#"{"id":"CLI-E-USAGE","exitCode":2,"outcome":"usage","description":"Invalid command or arguments.","introduced":"1.0.0","messageKey":"errors.CLI-E-USAGE"}"#,
    r#"{"id":"CLI-E-VALIDATION","exitCode":2,"outcome":"invalid_input","description":"Input does not satisfy the operation contract.","introduced":"1.0.0","messageKey":"errors.CLI-E-VALIDATION"}"#,
    r#"{"id":"ENG-E-0001","exitCode":2,"outcome":"invalid_input","description":"Unknown unit.","introduced":"1.0.0","messageKey":"errors.ENG-E-0001"}"#,
    r#"{"id":"ENG-E-0002","exitCode":2,"outcome":"invalid_input","description":"Duplicate entity identifier.","introduced":"1.0.0","messageKey":"errors.ENG-E-0002"}"#,
    r#"{"id":"ENG-E-0003","exitCode":2,"outcome":"invalid_input","description":"Missing entity reference.","introduced":"1.0.0","messageKey":"errors.ENG-E-0003"}"#,
    r#"{"id":"ENG-E-0004","exitCode":2,"outcome":"invalid_input","description":"Node is not connected to any member.","introduced":"1.0.0","messageKey":"errors.ENG-E-0004"}"#,
    r#"{"id":"ENG-E-0005","exitCode":2,"outcome":"invalid_input","description":"Member has zero length.","introduced":"1.0.0","messageKey":"errors.ENG-E-0005"}"#,
    r#"{"id":"ENG-E-0006","exitCode":4,"outcome":"not_converged","description":"System is singular or unstable.","introduced":"1.0.0","messageKey":"errors.ENG-E-0006"}"#,
    r#"{"id":"ENG-E-0007","exitCode":4,"outcome":"not_converged","description":"Numerical procedure did not converge.","introduced":"1.0.0","messageKey":"errors.ENG-E-0007"}"#,
    r#"{"id":"ENG-E-0008","exitCode":2,"outcome":"invalid_input","description":"Non-finite input or arithmetic result.","introduced":"1.0.0","messageKey":"errors.ENG-E-0008"}"#,
    r#"{"id":"ENG-E-0009","exitCode":2,"outcome":"invalid_input","description":"Incompatible contract name or major version.","introduced":"1.0.0","messageKey":"errors.ENG-E-0009"}"#,
    r#"{"id":"ENG-E-0010","exitCode":2,"outcome":"invalid_input","description":"Invalid dimensions, sizes or operation parameters.","introduced":"1.0.0","messageKey":"errors.ENG-E-0010"}"#,
    r#"{"id":"ENG-E-0011","exitCode":2,"outcome":"invalid_input","description":"Degree of freedom is unavailable in this context.","introduced":"1.0.0","messageKey":"errors.ENG-E-0011"}"#,
    r#"{"id":"ENG-E-0012","exitCode":70,"outcome":"internal","description":"Engine encountered an inconsistent internal state.","introduced":"1.0.0","messageKey":"errors.ENG-E-0012"}"#,
    r#"{"id":"ENG-E-0013","exitCode":3,"outcome":"unsupported","description":"Component does not support the requested capability.","introduced":"1.0.0","messageKey":"errors.ENG-E-0013"}"#,
    r#"{"id":"ENG-E-0014","exitCode":4,"outcome":"not_converged","description":"Massless degrees of freedom have non-positive-definite stiffness.","introduced":"1.0.0","messageKey":"errors.ENG-E-0014"}"#,
];

fn frozen_definitions() -> Value {
    json!({"codes": FROZEN_DEFINITIONS.iter().map(|entry|
        serde_json::from_str::<Value>(entry).unwrap()).collect::<Vec<_>>()})
}

fn unique_index(document: &Value) -> Option<BTreeMap<&str, &Value>> {
    let mut codes = BTreeMap::new();
    for definition in document["codes"].as_array().unwrap() {
        let id = definition["id"].as_str().unwrap();
        if codes.insert(id, definition).is_some() {
            return None;
        }
    }
    Some(codes)
}

fn index(document: &Value) -> BTreeMap<&str, &Value> {
    unique_index(document).expect("Código reutilizado")
}

fn compatible_with_baseline(registry: &Value, lock: &Value, baseline: &Value) -> bool {
    let (Some(current), Some(locked), Some(_)) = (
        unique_index(registry),
        unique_index(lock),
        unique_index(baseline),
    ) else {
        return false;
    };
    // También impide modificar/borrar registro y lock juntos. Una alta exige
    // extender explícitamente el ancla de revisión dentro de este test.
    lock["codes"] == baseline["codes"]
        && current.keys().eq(locked.keys())
        && locked.into_iter().all(|(id, frozen)| {
            current.get(id).is_some_and(|definition| {
                let mut definition = (*definition).clone();
                definition.as_object_mut().unwrap().remove("status");
                definition == *frozen
            })
        })
}

fn compatible(registry: &Value, lock: &Value) -> bool {
    compatible_with_baseline(registry, lock, &frozen_definitions())
}

#[test]
fn frozen_codes_cannot_disappear_or_change_meaning_exit_outcome_key_or_version() {
    let registry: Value = serde_json::from_str(REGISTRY).unwrap();
    let lock: Value = serde_json::from_str(LOCK).unwrap();
    assert_eq!(registry["schema"], "error-codes/1");
    assert_eq!(lock["schema"], "error-codes-lock/1");
    assert!(lock["codes"].as_array().unwrap().len() >= 32);
    assert!(
        compatible(&registry, &lock),
        "Código borrado o definición congelada modificada"
    );
    let messages: Value = serde_json::from_str(include_str!("../messages/en.json")).unwrap();
    for (id, definition) in index(&registry) {
        assert_eq!(definition.as_object().unwrap().len(), 7);
        let allowed_outcomes: &[&str] = match definition["exitCode"].as_u64() {
            Some(1) => &["warning"],
            Some(2) => &["usage", "invalid_input"],
            Some(3) => &["unsupported"],
            Some(4) => &["not_converged"],
            Some(5) => &["cancelled"],
            Some(6) => &["timeout"],
            Some(70) => &["internal"],
            _ => panic!("Salida sin categoría: {definition}"),
        };
        assert!(allowed_outcomes.contains(&definition["outcome"].as_str().unwrap()));
        assert!(matches!(
            definition["status"].as_str(),
            Some("active" | "deprecated")
        ));
        let version: Vec<_> = definition["introduced"]
            .as_str()
            .unwrap()
            .split('.')
            .collect();
        assert_eq!(version.len(), 3);
        assert!(version
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())));
        assert_eq!(definition["messageKey"], format!("errors.{id}"));
        assert!(!definition["description"]
            .as_str()
            .unwrap()
            .trim()
            .is_empty());
        assert!(!messages[definition["messageKey"].as_str().unwrap()]
            .as_str()
            .unwrap()
            .trim()
            .is_empty());
    }
    // Cada alta queda congelada en este mismo cambio; ningún id queda sin lock.
    assert_eq!(
        index(&registry).keys().collect::<Vec<_>>(),
        index(&lock).keys().collect::<Vec<_>>()
    );
}

#[test]
fn freeze_guard_rejects_mutations_and_allows_deprecation_and_appending() {
    let registry: Value = serde_json::from_str(REGISTRY).unwrap();
    let lock: Value = serde_json::from_str(LOCK).unwrap();
    let mut deleted = registry.clone();
    deleted["codes"].as_array_mut().unwrap().remove(0);
    assert!(!compatible(&deleted, &lock));
    for (field, value) in [
        ("id", json!("CLI-E-REUSED")),
        ("exitCode", json!(70)),
        ("outcome", json!("internal")),
        ("description", json!("Different meaning.")),
        ("messageKey", json!("different.key")),
        ("introduced", json!("2.0.0")),
    ] {
        let mut changed = registry.clone();
        changed["codes"][0][field] = value;
        assert!(
            !compatible(&changed, &lock),
            "Mutación no detectada: {field}"
        );
    }
    let mut deprecated = registry.clone();
    deprecated["codes"][0]["status"] = json!("deprecated");
    assert!(compatible(&deprecated, &lock));
    // Deprecated conserva exactamente el mismo significado y sigue reservando id.
    for field in [
        "id",
        "description",
        "exitCode",
        "outcome",
        "messageKey",
        "introduced",
    ] {
        let mut reassigned = deprecated.clone();
        reassigned["codes"][0][field] = json!("reassigned");
        assert!(
            !compatible(&reassigned, &lock),
            "Deprecated reutilizado: {field}"
        );
    }
    for status in ["active", "deprecated"] {
        let mut reused = deprecated.clone();
        let mut addition = reused["codes"][0].clone();
        addition["status"] = json!(status);
        reused["codes"].as_array_mut().unwrap().push(addition);
        assert!(
            !compatible(&reused, &lock),
            "ID deprecated duplicado: {status}"
        );
    }
    // Alta nueva: añadir en registro, lock y ancla es una revisión explícita.
    let mut extended_lock = lock.clone();
    let mut baseline = frozen_definitions();
    let mut addition = registry["codes"][0].clone();
    addition["id"] = json!("CLI-E-NEW");
    addition["messageKey"] = json!("errors.CLI-E-NEW");
    deprecated["codes"]
        .as_array_mut()
        .unwrap()
        .push(addition.clone());
    addition.as_object_mut().unwrap().remove("status");
    extended_lock["codes"]
        .as_array_mut()
        .unwrap()
        .push(addition.clone());
    assert!(!compatible(&deprecated, &extended_lock));
    baseline["codes"].as_array_mut().unwrap().push(addition);
    assert!(compatible_with_baseline(
        &deprecated,
        &extended_lock,
        &baseline
    ));
}

#[test]
fn embedded_anchor_rejects_coordinated_registry_and_lock_edits() {
    let registry: Value = serde_json::from_str(REGISTRY).unwrap();
    let lock: Value = serde_json::from_str(LOCK).unwrap();
    for position in 0..registry["codes"].as_array().unwrap().len() {
        let mut deleted = registry.clone();
        let mut shortened = lock.clone();
        deleted["codes"].as_array_mut().unwrap().remove(position);
        shortened["codes"].as_array_mut().unwrap().remove(position);
        assert!(!compatible(&deleted, &shortened));
        for (field, value) in [
            ("id", json!("CLI-E-REUSED")),
            ("exitCode", json!(0)),
            ("outcome", json!("changed")),
            ("description", json!("Different meaning.")),
            ("messageKey", json!("different.key")),
            ("introduced", json!("2.0.0")),
        ] {
            let mut changed = registry.clone();
            let mut rewritten = lock.clone();
            changed["codes"][position][field] = value.clone();
            rewritten["codes"][position][field] = value;
            assert!(
                !compatible(&changed, &rewritten),
                "Edición coordinada: {position}/{field}"
            );
        }
    }
    let mut reordered = lock.clone();
    reordered["codes"].as_array_mut().unwrap().swap(0, 1);
    assert!(!compatible(&registry, &reordered));
}

// Conservative Rust literal lexer: ignore comments (including doc and nested
// comments), chars and cfg(test) items; include ordinary and raw strings.
struct Literal<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

fn production_literals(source: &str) -> Vec<Literal<'_>> {
    let bytes = source.as_bytes();
    let mut literals = Vec::new();
    let mut i = 0;
    let mut test_item = false;
    let mut depth = 0usize;
    while i < bytes.len() {
        if source[i..].starts_with("//") {
            i += source[i..].find('\n').unwrap_or(bytes.len() - i);
        } else if source[i..].starts_with("/*") {
            i += 2;
            let mut comments = 1;
            while i < bytes.len() && comments > 0 {
                if source[i..].starts_with("/*") {
                    comments += 1;
                    i += 2;
                } else if source[i..].starts_with("*/") {
                    comments -= 1;
                    i += 2;
                } else {
                    i += source[i..].chars().next().unwrap().len_utf8();
                }
            }
        } else if source[i..].starts_with("#[cfg(test)]") {
            test_item = true;
            i += "#[cfg(test)]".len();
        } else if bytes[i] == b'r' && matches!(bytes.get(i + 1), Some(b'#' | b'"')) {
            let opening = i;
            let start = i + 1;
            let mut quote = start;
            while bytes.get(quote) == Some(&b'#') {
                quote += 1;
            }
            if bytes.get(quote) != Some(&b'"') {
                i += 1;
                continue;
            }
            let terminator = format!("\"{}", &source[start..quote]);
            let end = quote
                + 1
                + source[quote + 1..]
                    .find(&terminator)
                    .expect("raw string sin cierre");
            if !test_item {
                literals.push(Literal {
                    text: &source[quote + 1..end],
                    start: opening,
                    end: end + terminator.len(),
                });
            }
            i = end + terminator.len();
        } else if bytes[i] == b'"' {
            let start = i + 1;
            i = start;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            if !test_item {
                literals.push(Literal {
                    text: &source[start..i],
                    start: start - 1,
                    end: i + 1,
                });
            }
            i += 1;
        } else if bytes[i] == b'\'' && bytes.get(i + 1) == Some(&b'\\') {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\'' {
                i += 1;
            }
            i += 1;
        } else if bytes[i] == b'\'' && source[i + 1..].chars().nth(1) == Some('\'') {
            i += 2 + source[i + 1..].chars().next().unwrap().len_utf8();
        } else {
            if test_item {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            test_item = false;
                        }
                    }
                    b';' if depth == 0 => test_item = false,
                    _ => {}
                }
            }
            i += source[i..].chars().next().unwrap().len_utf8();
        }
    }
    literals
}

// Match ^(ENG|API|CLI|WF)-[A-Z]- across all severity families. Prefixes
// count too: dynamic construction must not bypass the explicit registry.
fn code_family(text: &str) -> bool {
    let Some((domain, suffix)) = text.split_once('-') else {
        return false;
    };
    matches!(domain, "ENG" | "API" | "CLI" | "WF")
        && suffix
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_uppercase)
        && suffix.as_bytes().get(1) == Some(&b'-')
}

fn code_tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'))
        .filter(|code| code_family(code))
}

fn emitted_codes(source: &str) -> Vec<String> {
    let literals = production_literals(source);
    let mut codes: Vec<String> = literals
        .iter()
        .flat_map(|literal| code_tokens(literal.text).map(str::to_owned))
        .collect();
    // Adjacent literal arguments also cover concat!, format!, array concat/join
    // and string addition, even when the family itself is split ("EN", "G-E-").
    // Reject the assembled prefix rather than trying to evaluate dynamic Rust.
    for (start, literal) in literals.iter().enumerate() {
        if literal.text.is_empty() || code_family(literal.text) {
            continue;
        }
        for separator in ["", "-"] {
            let mut joined = literal.text.to_owned();
            for pair in literals[start..].windows(2) {
                let between = source[pair[0].end..pair[1].start].trim();
                if !matches!(between, "," | "+") {
                    break;
                }
                joined.push_str(separator);
                joined.push_str(pair[1].text);
                if code_family(&joined) {
                    let domain_end = joined.find('-').unwrap();
                    codes.push(joined[..domain_end + 3].to_owned());
                    break;
                }
                // No further fragments can repair a mismatched family prefix.
                if !["ENG-", "API-", "CLI-", "WF-"]
                    .iter()
                    .any(|family| family.starts_with(&joined) || joined.starts_with(family))
                {
                    break;
                }
            }
        }
    }
    codes
}

fn check_sources(path: &Path, registered: &BTreeMap<&str, &Value>, count: &mut usize) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_sources(&path, registered, count);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path).unwrap();
            for code in emitted_codes(&source) {
                assert!(
                    registered.contains_key(code.as_str()),
                    "Código sin registro en {}: {code}",
                    path.display()
                );
                *count += 1;
            }
        }
    }
}

#[test]
fn every_production_emitter_in_workspace_uses_registered_codes() {
    let registry: Value = serde_json::from_str(REGISTRY).unwrap();
    let registered = index(&registry);
    let mut count = 0;
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for entry in fs::read_dir(crates).unwrap() {
        let source = entry.unwrap().path().join("src");
        if source.is_dir() {
            check_sources(&source, &registered, &mut count);
        }
    }
    assert!(
        count > 100,
        "El inventario de emisores quedó vacío o incompleto"
    );
}

#[test]
fn source_inventory_handles_comments_raw_escaped_strings_and_test_items() {
    let source = r####"
        // "ENG-E-9999"
        /* /* "CLI-E-UNKNOWN" */ "API-E-UNKNOWN" */
        let quote = '"'; let letter = 'ñ'; let escape = '\'';
        emit("CLI-E-NEW"); emit(r###"API-E-NEW"###);
        const DOMAIN_CODE: &str = "ENG-E-0014";
        static LOCAL_CODE: &str = "API-E-DEFINED";
        static CATALOG: &[CodeInfo] = &[CodeInfo { code: "ENG-E-0013" }];
        report(DOMAIN_CODE); report(LOCAL_CODE); report(CATALOG[0].code);
        emit("{\"code\":\"ENG-E-0012\"}");
        #[cfg(test)] mod tests { emit("ENG-E-9999"); }
        emit("CLI-E-INTERNAL");
        #[cfg(test)] #[path = "test.rs"] mod tests;
        emit("API-E-INPUT");
    "####;
    assert_eq!(
        emitted_codes(source),
        [
            "CLI-E-NEW",
            "API-E-NEW",
            "ENG-E-0014",
            "API-E-DEFINED",
            "ENG-E-0013",
            "ENG-E-0012",
            "CLI-E-INTERNAL",
            "API-E-INPUT"
        ]
    );
    let registry: Value = serde_json::from_str(REGISTRY).unwrap();
    assert!(!index(&registry).contains_key(emitted_codes("emit(\"CLI-E-NEW\")")[0].as_str()));
}

#[test]
fn inventory_rejects_unregistered_macro_fragments_joins_and_severity_families() {
    let registry: Value = serde_json::from_str(REGISTRY).unwrap();
    let registered = index(&registry);
    // Mutate a registered emitter into each construction that used to escape
    // coverage. A prefix remains unregistered even if its final code is known.
    assert!(emitted_codes(r#"emit("ENG-E-0001");"#)
        .iter()
        .all(|code| registered.contains_key(code.as_str())));
    for mutation in [
        r#"emit(concat!("ENG-E-", "0099"));"#,
        r#"emit(format!("ENG-E-{n:04}"));"#,
        r#"emit("ENG-W-0001");"#,
        r#"emit(concat!("ENG", "-E-", "0099"));"#,
        r#"emit(concat!("EN", "G", "-E-", "0099"));"#,
        r#"emit(format!("{}{}", "CLI-W-", "UNKNOWN"));"#,
        r#"emit(["API", "-W-", "UNKNOWN"].join(""));"#,
        r#"emit(["WF", "E", "0099"].join("-"));"#,
        r#"emit("API-W-");"#,
        r#"emit("CLI-W-UNKNOWN");"#,
        r#"emit("WF-I-UNKNOWN");"#,
        r#"emit("ENG-Z-0099");"#,
        r#"emit(concat!("ENG-E-", "0001"));"#,
    ] {
        let codes = emitted_codes(mutation);
        assert!(
            codes
                .iter()
                .any(|code| !registered.contains_key(code.as_str())),
            "Unregistered construction escaped inventory: {mutation}; found {codes:?}"
        );
    }
    assert!(emitted_codes(r#"emit("API-not-a-code"); emit("ENG-E");"#).is_empty());
}

#[test]
#[should_panic(expected = "Código reutilizado")]
fn duplicate_ids_are_rejected_even_with_different_definitions() {
    let mut registry: Value = serde_json::from_str(REGISTRY).unwrap();
    let mut reused = registry["codes"][0].clone();
    reused["description"] = json!("Different meaning.");
    registry["codes"].as_array_mut().unwrap().push(reused);
    index(&registry);
}
