//! Boundary between engine-input/2 and the SI kernel.
use engine_api::{run, validate, ApiError, CancelToken};
use engine_core::units::catalog::CatalogUnit;
use serde_json::{json, Value};

fn factor(unit: CatalogUnit) -> f64 {
    unit.si_factor().to_f64()
}

fn truss(
    schema: &str,
    units: &str,
    length: f64,
    area: f64,
    modulus: f64,
    density: f64,
    load: f64,
) -> Value {
    json!({
        "schema": schema,
        "units": units,
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [length, 0.0, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [{
                "id": 1, "nodes": [1, 2], "section": "s", "law": "steel",
                "element": "truss2d.linear"
            }],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": area}]}],
            "laws": [{"name": "steel", "parameters": [
                {"name": "E", "value": modulus},
                {"name": "rho", "value": density}
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]}
            ],
            "masses": []
        },
        "cases": [{"name": "P", "loads": [{"node": 2, "values": [{"dof": "Ux", "value": load}]}]}],
        "combinations": [],
        "analyses": [{"method": "linearStatic"}],
        "options": {}
    })
}

fn without_fingerprint(document: &Value) -> Value {
    let mut copy = document.clone();
    if let Some(object) = copy.as_object_mut() {
        object.remove("input_sha256");
    }
    copy
}

#[test]
fn version_one_still_rejects_a_non_si_label() {
    let mut document = truss("engine-input/1", "US", 1.0, 1.0, 1.0, 1.0, 1.0);
    document["units"] = json!("US");
    let error = validate(&document.to_string()).unwrap_err();
    assert_eq!(
        error,
        ApiError::UnknownUnit("Unknown unit system; only \"SI\" is accepted".into())
    );
}

#[test]
fn version_two_requires_an_explicit_system() {
    let mut document = truss("engine-input/2", "SI", 1.0, 1.0, 1.0, 1.0, 1.0);
    document.as_object_mut().unwrap().remove("units");
    match validate(&document.to_string()) {
        Err(ApiError::InvalidInput(message)) => {
            assert!(
                message.contains("units"),
                "the missing-field error must name units: {message}"
            );
            assert!(message.contains("Missing required field"), "{message}");
        }
        other => panic!("expected a rejection that names units, got {other:?}"),
    }
    document["units"] = json!("kip");
    let error = validate(&document.to_string()).unwrap_err();
    match error {
        ApiError::UnknownUnit(message) => {
            assert!(message.contains("«kip»"), "{message}");
            assert_eq!(
                run(&document.to_string(), &CancelToken::new(), |_| {}).exit_code(),
                2
            );
        }
        other => panic!("expected unknown unit, got {other:?}"),
    }
    document["units"] = json!(1);
    match validate(&document.to_string()) {
        Err(ApiError::InvalidInput(message)) => assert!(message.contains("units"), "{message}"),
        other => panic!("expected a rejection that names units, got {other:?}"),
    }
}

#[test]
fn si_documents_match_across_versions_and_us_round_trips() {
    let inch = factor(CatalogUnit::Inch);
    let kip = factor(CatalogUnit::Kip);
    let ksi = factor(CatalogUnit::Ksi);
    let area = inch * inch;
    let density = kip / (inch * inch * inch * inch);
    let length_us = 120.0;
    let area_us = 8.0;
    let modulus_us = 29_000.0;
    let density_us = 0.25;
    let load_us = 2.5;
    let si = truss(
        "engine-input/1",
        "SI",
        length_us * inch,
        area_us * area,
        modulus_us * ksi,
        density_us * density,
        load_us * kip,
    );
    let same = truss(
        "engine-input/2",
        "SI",
        length_us * inch,
        area_us * area,
        modulus_us * ksi,
        density_us * density,
        load_us * kip,
    );
    let customary = truss(
        "engine-input/2",
        "US",
        length_us,
        area_us,
        modulus_us,
        density_us,
        load_us,
    );
    let left = run(&si.to_string(), &CancelToken::new(), |_| {});
    let middle = run(&same.to_string(), &CancelToken::new(), |_| {});
    let right = run(&customary.to_string(), &CancelToken::new(), |_| {});
    assert_eq!(left.exit_code(), 0, "{}", left.document);
    assert_eq!(middle.exit_code(), 0, "{}", middle.document);
    assert_eq!(right.exit_code(), 0, "{}", right.document);
    let left_body = without_fingerprint(&serde_json::from_str(&left.document).unwrap());
    let middle_body = without_fingerprint(&serde_json::from_str(&middle.document).unwrap());
    let right_body = without_fingerprint(&serde_json::from_str(&right.document).unwrap());
    assert_eq!(left_body, middle_body);
    // Contract: relative tolerance. Bit equality, if it occurs, is not required.
    assert_eq!(
        numbers_within(&right_body, &left_body, US_SI_RELATIVE_TOLERANCE),
        Ok(()),
        "US versus SI"
    );
    let back = length_us * inch / inch;
    let relative = ((back - length_us) / length_us).abs();
    assert!(relative <= 1e-15, "{relative}");
}

/// Relative bound for a US result against the same model already written in SI.
/// Boundary conversion rounds in binary64, so the documents are not required to match bit for bit.
const US_SI_RELATIVE_TOLERANCE: f64 = 1e-12;

fn numbers_within(actual: &Value, expected: &Value, tolerance: f64) -> Result<(), String> {
    match (actual, expected) {
        (Value::Number(left), Value::Number(right)) => {
            let left = left.as_f64().ok_or_else(|| format!("non-f64 {left}"))?;
            let right = right.as_f64().ok_or_else(|| format!("non-f64 {right}"))?;
            if right == 0.0 {
                if left == 0.0 {
                    Ok(())
                } else {
                    Err(format!("{left} against zero"))
                }
            } else if ((left - right) / right).abs() <= tolerance {
                Ok(())
            } else {
                Err(format!("{left} against {right}"))
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                return Err(format!("length {} against {}", left.len(), right.len()));
            }
            for (item, reference) in left.iter().zip(right) {
                numbers_within(item, reference, tolerance)?;
            }
            Ok(())
        }
        (Value::Object(left), Value::Object(right)) => {
            if left.len() != right.len() {
                return Err(format!("keys {} against {}", left.len(), right.len()));
            }
            for (key, reference) in right {
                let Some(item) = left.get(key) else {
                    return Err(format!("missing {key}"));
                };
                if key == "input_sha256" {
                    continue;
                }
                numbers_within(item, reference, tolerance)?;
            }
            Ok(())
        }
        _ if actual == expected => Ok(()),
        _ => Err(format!("{actual} against {expected}")),
    }
}

#[test]
fn unknown_us_parameter_names_the_field() {
    let mut document = truss("engine-input/2", "US", 10.0, 1.0, 1.0, 1.0, 1.0);
    document["model"]["sections"][0]["properties"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "zz", "value": 1.0}));
    let error = validate(&document.to_string()).unwrap_err();
    match error {
        ApiError::InvalidInput(message) => assert!(message.contains("«zz»"), "{message}"),
        other => panic!("expected a named rejection, got {other:?}"),
    }
}
