//! Diez corridas por entrada; hash del results/1 completo, no del input.
#[path = "support/determinism.rs"]
mod fixtures;
use engine_api::{canonical_json, run, CancelToken, Status};
use serde_json::Value;

#[test]
fn v1_results_are_identical_in_ten_runs() {
    for &(name, input) in fixtures::FIXTURES {
        let mut first = None;
        for iteration in 0..10 {
            let outcome = run(input, &CancelToken::new(), |_| {});
            assert_eq!(outcome.status, Status::Ok, "{name}: {}", outcome.document);
            assert_eq!(outcome.document, canonical_json(&outcome.document).unwrap());
            let hash = mamut_canon::sha256_hex(outcome.document.as_bytes());
            if let Some((document, previous_hash)) = &first {
                assert_eq!(&hash, previous_hash, "{name}, corrida {iteration}");
                assert_eq!(&outcome.document, document, "{name}, corrida {iteration}");
            } else {
                first = Some((outcome.document, hash));
            }
        }
    }
}

#[test]
fn cross_target_comparison_checks_target_and_preserves_all_other_fields() {
    let outcome = run(fixtures::FIXTURES[0].1, &CancelToken::new(), |_| {});
    let mut original: Value = serde_json::from_str(&outcome.document).unwrap();
    let compared = fixtures::comparison_document(&outcome.document).unwrap();
    original.as_object_mut().unwrap().remove("target");
    assert_eq!(serde_json::from_str::<Value>(&compared).unwrap(), original);
    assert!(original.get("git_commit").is_some());
    assert!(original.get("input_sha256").is_some());
    assert!(fixtures::comparison_document(&original.to_string()).is_err());
    for wrong_target in [serde_json::json!("wrong-target"), Value::Null] {
        original["target"] = wrong_target;
        assert!(fixtures::comparison_document(&original.to_string()).is_err());
    }
}

#[test]
fn shared_native_wasm_suite_hashes_are_repeatable() {
    for required in [
        "scalar-sin-cos",
        "frame3d-inclined-static",
        "frame3d-inclined-modal",
    ] {
        assert!(
            (0..fixtures::case_count()).any(|index| fixtures::case_name(index) == Some(required))
        );
    }
    for index in 0..fixtures::case_count() {
        let name = fixtures::case_name(index).unwrap();
        let first = fixtures::case_hash(index).unwrap();
        for _ in 1..10 {
            assert_eq!(fixtures::case_hash(index).unwrap(), first, "{name}");
        }
    }
    assert!(fixtures::case_name(fixtures::case_count()).is_none());
    assert!(fixtures::case_hash(fixtures::case_count()).is_err());
}

#[test]
fn spatial_fixtures_exercise_static_recovery_and_rotated_modes() {
    for (name, field) in [
        ("frame3d-inclined-static", "cases"),
        ("frame3d-inclined-modal", "modes"),
    ] {
        let (_, input) = fixtures::FIXTURES
            .iter()
            .find(|(fixture, _)| *fixture == name)
            .unwrap();
        let outcome = run(input, &CancelToken::new(), |_| {});
        assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
        let document: Value = serde_json::from_str(&outcome.document).unwrap();
        let members = document["members"].as_array().unwrap();
        assert_eq!(members.len(), 2);
        assert!(members
            .iter()
            .all(|member| member["element"] == "frame3d.euler_elastic"));
        if field == "cases" {
            let case = &document["cases"][0];
            assert_eq!(case["members"].as_array().unwrap().len(), 2);
            for member in case["members"].as_array().unwrap() {
                let forces = member["end_forces_local"].as_array().unwrap();
                assert_eq!(forces.len(), 12);
                assert!(forces.iter().all(|v| v.as_f64().unwrap().abs() > 1e-12));
            }
        } else {
            let modes = document["modes"].as_array().unwrap();
            assert_eq!(modes.len(), 6);
            for mode in modes {
                assert!(mode["eigenvalue"].as_f64().unwrap() > 0.0);
                assert_eq!(mode["participation"].as_array().unwrap().len(), 3);
                assert_eq!(mode["shape"].as_array().unwrap().len(), 3);
            }
        }
        // Iy != Iz: cambiar la orientación debe cambiar resultados mecánicos,
        // además del input_sha256, en ambos análisis.
        let mut unrotated: Value = serde_json::from_str(input).unwrap();
        for member in unrotated["model"]["members"].as_array_mut().unwrap() {
            member["orientation"] = serde_json::json!([0.0, 0.0, 1.0]);
        }
        let changed = run(&unrotated.to_string(), &CancelToken::new(), |_| {});
        assert_eq!(changed.status, Status::Ok, "{}", changed.document);
        let changed: Value = serde_json::from_str(&changed.document).unwrap();
        assert_ne!(
            document[field], changed[field],
            "{name}: orientación ignorada"
        );
    }
}
