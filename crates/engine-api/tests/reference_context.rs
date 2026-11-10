//! Broken references keep ENG-E-0003 and add optional context.
mod support;

use engine_api::{explain_in, localize_diagnostics, validate, CancelToken, Lang};
use serde_json::Value;
use support::checked_run;

const MISSING_NODE: &str = include_str!("../fixtures/input/invalid/missing-reference.json");

#[test]
fn results_name_the_missing_node_without_a_new_code() {
    let diagnostics = validate(MISSING_NODE).unwrap();
    let broken = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "ENG-E-0003")
        .expect("broken reference");
    assert!(broken.cause.contains("node"));
    assert!(broken.cause.contains("99"));
    assert!(broken.cause.contains("member 7"));
    assert!(broken.cause.contains("end 1"));
    assert!(broken.cause.contains("members[0].nodes[1]"));
    assert_eq!(broken.entity_kind.as_deref(), Some("node"));
    assert_eq!(broken.missing_id, Some(serde_json::json!(99)));
    assert_eq!(broken.referenced_by.as_deref(), Some("members[0].nodes[1]"));
    assert!(broken.suggestions.is_none());

    let orphan = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "ENG-E-0004")
        .expect("orphan node");
    assert!(orphan.entity_kind.is_none());
    assert!(orphan.missing_id.is_none());
    assert!(orphan.referenced_by.is_none());
    assert!(orphan.suggestions.is_none());

    let outcome = checked_run(MISSING_NODE, &CancelToken::new(), |_| {});
    let document: Value = serde_json::from_str(&outcome.document).unwrap();
    let published = document["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "ENG-E-0003")
        .expect("results diagnostic");
    assert_eq!(published["entity_kind"], "node");
    assert_eq!(published["missing_id"], 99);
    assert_eq!(published["referenced_by"], "members[0].nodes[1]");
    assert!(published.get("suggestions").is_none());
    assert!(published["cause"].as_str().unwrap().contains("end 1"));

    let spanish = explain_in("ENG-E-0003", Lang::Es).unwrap();
    assert!(spanish.contains("entidad"));
    let localized = localize_diagnostics(diagnostics, Lang::Es);
    let localized_cause = &localized
        .iter()
        .find(|diagnostic| diagnostic.code == "ENG-E-0003")
        .unwrap()
        .cause;
    assert!(localized_cause.starts_with("The referenced entity does not exist."));
    assert!(localized_cause.contains("Missing node 99"));
}
