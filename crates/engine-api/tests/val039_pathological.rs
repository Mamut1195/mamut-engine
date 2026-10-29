//! VAL-039: pathological inputs, exact registry codes and own equilibrium oracles.
//! Derivations, limits and excluded observations: provenance/ENG-VAL-039.md.
use engine_api::{CancelToken, Outcome, Status};
use engine_core::math;
use serde_json::{json, Value};
#[path = "support/val039.rs"]
mod models;
mod support;

fn solve(raw: &str) -> (Outcome, Value) {
    let outcome = support::checked_run(raw, &CancelToken::new(), |_| {});
    let document = serde_json::from_str(&outcome.document).unwrap();
    (outcome, document)
}

fn error(raw: &str, code: &str, status: Status, exit: i32) -> Value {
    let registry: Value =
        serde_json::from_str(include_str!("../../../docs/contracts/error-codes-1.json")).unwrap();
    let entry = registry["codes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == code)
        .unwrap();
    assert_eq!(entry["exitCode"], exit);
    let (outcome, document) = solve(raw);
    assert_eq!(outcome.status, status, "{}", outcome.document);
    assert_eq!(outcome.exit_code(), exit, "{}", outcome.document);
    assert_eq!(
        document["diagnostics"].as_array().unwrap().len(),
        1,
        "{document}"
    );
    let diagnostic = &document["diagnostics"][0];
    assert_eq!(diagnostic["code"], code, "{document}");
    assert_eq!(diagnostic["severity"], "error");
    assert!(!diagnostic["cause"].as_str().unwrap().is_empty());
    for field in [
        "nodes",
        "members",
        "cases",
        "combinations",
        "modes",
        "envelopes",
    ] {
        assert_eq!(document[field], json!([]), "{field}: {document}");
    }
    assert_eq!(outcome, solve(raw).0, "unstable diagnostic bytes");
    document
}

fn rejected_without_responses(raw: &str, outcome: &Outcome, document: &Value) {
    let registry: Value =
        serde_json::from_str(include_str!("../../../docs/contracts/error-codes-1.json")).unwrap();
    let diagnostics = document["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{document}");
    let code = &diagnostics[0]["code"];
    let entry = registry["codes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == *code)
        .unwrap_or_else(|| panic!("unregistered code: {document}"));
    assert_eq!(entry["exitCode"], outcome.exit_code());
    assert_ne!(outcome.exit_code(), 0);
    for field in [
        "nodes",
        "members",
        "cases",
        "combinations",
        "modes",
        "envelopes",
    ] {
        assert_eq!(document[field], json!([]), "{field}: {document}");
    }
    assert_eq!(*outcome, solve(raw).0, "rejection bytes");
}

fn number(value: &Value) -> f64 {
    let number = value
        .as_f64()
        .unwrap_or_else(|| panic!("expected finite number: {value}"));
    assert!(number.is_finite());
    number
}

fn close(actual: f64, expected: f64) {
    // Relative comparison remains meaningful for the tiny-length displacement.
    assert!(actual.is_finite() && expected.is_finite());
    let scale = actual.abs().max(expected.abs()).max(f64::MIN_POSITIVE);
    assert!(
        (actual - expected).abs() <= 1e-10 * scale,
        "{actual} != {expected}"
    );
}

fn success(input: &Value) -> Value {
    let (outcome, document) = solve(&input.to_string());
    assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
    assert_eq!(outcome.exit_code(), 0);
    assert_eq!(document["diagnostics"], json!([]));
    document
}

fn finite_array(value: &Value) {
    let values = value.as_array().unwrap();
    assert!(!values.is_empty());
    for value in values {
        number(value);
    }
}

fn equilibrium(input: &Value, document: &Value) {
    let results = document["cases"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    let nodes = results[0]["nodes"].as_array().unwrap();
    assert_eq!(
        nodes.len(),
        input["model"]["nodes"].as_array().unwrap().len()
    );
    let dofs = ["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"];
    let mut applied = [0.0; 3];
    let mut applied_moment = [0.0; 3];
    let mut scale: f64 = 0.0;
    for load in input["cases"][0]["loads"].as_array().unwrap() {
        let node = input["model"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["id"] == load["node"])
            .unwrap();
        let r = &node["coordinates"];
        for component in load["values"].as_array().unwrap() {
            let index = dofs
                .iter()
                .position(|dof| component["dof"] == *dof)
                .unwrap();
            let force = number(&component["value"]);
            assert!(index < 3, "these models apply only nodal forces");
            applied[index] += force;
            let mut f = [0.0; 3];
            f[index] = force;
            let moment = cross([number(&r[0]), number(&r[1]), number(&r[2])], f);
            for i in 0..3 {
                applied_moment[i] += moment[i];
            }
            scale += force.abs() + moment.iter().map(|v| v.abs()).sum::<f64>();
        }
    }
    assert!(scale > 0.0);
    for node in nodes {
        finite_array(&node["displacements"]);
        finite_array(&node["reactions"]);
        let r = input["model"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == node["id"])
            .unwrap();
        let r = &r["coordinates"];
        let f = [
            number(&node["reactions"][0]),
            number(&node["reactions"][1]),
            number(&node["reactions"][2]),
        ];
        let moment = cross([number(&r[0]), number(&r[1]), number(&r[2])], f);
        for i in 0..3 {
            applied[i] += f[i];
            applied_moment[i] += moment[i] + number(&node["reactions"][i + 3]);
        }
    }
    for residual in applied.into_iter().chain(applied_moment) {
        assert!(
            residual.abs() <= 1e-10 * scale,
            "global imbalance: {residual}, scale={scale}"
        );
    }
    let members = results[0]["members"].as_array().unwrap();
    assert_eq!(
        members.len(),
        input["model"]["members"].as_array().unwrap().len()
    );
    for member in members {
        finite_array(&member["end_forces_local"]);
        for station in member["stations"].as_array().unwrap() {
            number(&station["position"]);
            finite_array(&station["forces_local"]);
        }
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn orthogonal_bars(e1: f64, e2: f64) -> Value {
    let mut input = models::truss();
    input["model"]["nodes"].as_array_mut().unwrap().push(json!({
        "id":3,"dim":"D2","coordinates":[0.0,4.0,0.0],"dofs":["Ux","Uy"]}));
    input["model"]["members"]
        .as_array_mut()
        .unwrap()
        .push(json!({
        "id":8,"nodes":[1,3],"element":"truss2d.linear","section":"s","law":"other"}));
    input["model"]["laws"][0]["parameters"][0]["value"] = json!(e1);
    input["model"]["laws"].as_array_mut().unwrap().push(json!({
        "name":"other","parameters":[{"name":"E","value":e2},{"name":"rho","value":0.0}]}));
    input["model"]["supports"]
        .as_array_mut()
        .unwrap()
        .push(json!({
        "id":"guide3","node":3,"restrained":["Ux"]}));
    input["cases"][0]["loads"]
        .as_array_mut()
        .unwrap()
        .push(json!({
        "node":3,"values":[{"dof":"Uy","value":1.0}]}));
    input
}

fn series_bars(e1: f64, e2: f64) -> Value {
    let mut input = orthogonal_bars(e1, e2);
    input["model"]["nodes"][2]["coordinates"] = json!([8.0, 0.0, 0.0]);
    input["model"]["members"][1]["nodes"] = json!([2, 3]);
    input["model"]["supports"][2]["restrained"] = json!(["Uy"]);
    input["cases"][0]["loads"] = json!([{"node":3,"values":[{"dof":"Ux","value":1.0}]}]);
    input
}

#[test]
fn mechanisms_and_massless_mechanism_have_exact_codes_and_no_partial_results() {
    for (name, input, code) in models::unstable_models() {
        if name == "supercritical column" {
            continue;
        }
        error(&input.to_string(), code, Status::Unstable, 4);
    }
}

#[test]
fn topology_and_contract_errors_have_exact_codes_and_stable_diagnostics() {
    for (_, input, code) in models::topology_errors() {
        error(&input.to_string(), code, Status::Invalid, 2);
    }
}

#[test]
fn malformed_json_and_missing_fields_use_the_registered_input_structure_code() {
    for raw in [
        "{",
        "null {}",
        r#"{"schema":"engine-input/1","schema":"engine-input/1"}"#,
    ] {
        error(raw, "API-E-INPUT", Status::Invalid, 2);
    }
    for field in ["schema", "model", "cases", "analyses", "units"] {
        let mut input = models::cantilever();
        input.as_object_mut().unwrap().remove(field);
        error(&input.to_string(), "API-E-INPUT", Status::Invalid, 2);
    }
}

#[test]
fn h2_non_finite_load_tokens_report_eng_e_0008() {
    let source = include_str!("fixtures/val039/cantilever.json");
    assert!(source.contains("-12.0"));
    for token in ["NaN", "Infinity", "-Infinity", "1e999", "-1e999"] {
        let raw = source.replace("-12.0", token);
        let document = error(&raw, "ENG-E-0008", Status::Invalid, 2);
        assert!(document["input_sha256"].is_null(), "{token}: {document}");
    }
}

#[test]
fn h2_large_finite_load_is_not_non_finite() {
    let source = include_str!("fixtures/val039/cantilever.json");
    assert!(source.contains("-12.0"));
    for token in ["1e300", "-1e300"] {
        let raw = source.replace("-12.0", token);
        let (outcome, document) = solve(&raw);
        for diagnostic in document["diagnostics"].as_array().unwrap() {
            assert_ne!(diagnostic["code"], "ENG-E-0008", "{token}: {document}");
        }
        if outcome.exit_code() > 1 {
            rejected_without_responses(&raw, &outcome, &document);
        }
    }
}

#[test]
fn h2_non_finite_words_in_case_names_are_text() {
    let source = include_str!("fixtures/val039/cantilever.json");
    let mut input: Value = serde_json::from_str(source).unwrap();
    for text in ["NaN", "Infinity", "-Infinity", "-NaN"] {
        input["cases"][0]["name"] = json!(text);
        success(&input);
    }
}

#[test]
fn near_zero_lengths_have_defined_rejection_or_finite_equilibrium() {
    let mut input = models::cantilever();
    input["model"]["nodes"][1]["coordinates"][0] = json!(1e-12);
    error(&input.to_string(), "ENG-E-0005", Status::Invalid, 2);
    let length: f64 = 1e-9;
    input["model"]["nodes"][1]["coordinates"][0] = json!(length);
    let document = success(&input);
    equilibrium(&input, &document);
    // Integrating EI v''=-P(L-x) twice with v(0)=v'(0)=0 gives v(L)=-PL³/(3EI).
    close(
        number(&document["cases"][0]["nodes"][1]["displacements"][1]),
        -12.0 * math::powi(length, 3) / (3.0 * 200.0 * 3.0),
    );
    close(
        number(&document["cases"][0]["nodes"][0]["reactions"][5]),
        12.0 * length,
    );
}

#[test]
fn extreme_moduli_in_one_model_remain_finite_and_balanced() {
    for (e1, e2) in [(1e-6, 1e12), (1e12, 1e-6), (1.0, 1e6)] {
        let input = orthogonal_bars(e1, e2);
        let document = success(&input);
        equilibrium(&input, &document);
        // Each axial bar has EA delta/L=F; F=1, L=4, A=2.
        close(
            number(&document["cases"][0]["nodes"][1]["displacements"][0]),
            4.0 / (2.0 * e1),
        );
        close(
            number(&document["cases"][0]["nodes"][2]["displacements"][1]),
            4.0 / (2.0 * e2),
        );
    }
}

#[test]
fn extreme_moduli_in_series_have_balanced_response_or_clean_rejection() {
    for (e1, e2) in [(1e-6, 1e12), (1e12, 1e-6)] {
        let input = series_bars(e1, e2);
        let raw = input.to_string();
        let (outcome, document) = solve(&raw);
        if outcome.status == Status::Ok {
            equilibrium(&input, &document);
            close(
                number(&document["cases"][0]["nodes"][2]["displacements"][0]),
                4.0 / (2.0 * e1) + 4.0 / (2.0 * e2),
            );
        } else {
            // Finding H4 (provenance/ENG-VAL-039.md): the structure is physically
            // stable; absorption of the soft element diagonal during assembly
            // is reported as ill-conditioning, with no partial results.
            assert!(e1 < e2, "stiff-first order must solve: {document}");
            assert_eq!(document["diagnostics"][0]["code"], "ENG-E-0010");
            assert_eq!(outcome.status, Status::Invalid);
            assert_eq!(outcome.exit_code(), 2);
            rejected_without_responses(&raw, &outcome, &document);
        }
    }
}

#[test]
fn h4_soft_bar_next_to_support_reports_ill_conditioning_not_mechanism() {
    for (e1, e2) in [(1e-6, 1e12), (1e-3, 1e12), (1e-6, 1e9)] {
        let input = series_bars(e1, e2);
        let document = error(&input.to_string(), "ENG-E-0010", Status::Invalid, 2);
        let cause = document["diagnostics"][0]["cause"].as_str().unwrap();
        assert!(cause.contains("Ill-conditioned"), "{document}");
        assert!(cause.contains("node 2, DOF Ux (global 2)"), "{document}");
    }
}

#[test]
fn h4_reclassification_threshold_is_pinned_on_both_sides() {
    // Equal A and L give contrast = E1/E2 = k1/k2. With n=2 free DOFs,
    // 64 * EPSILON * max(n, 1) is about 2.842e-14.
    let e2 = 1e12;

    // Below the threshold, within 0.5..1 times its value.
    let input = series_bars(2.0e-14 * e2, e2);
    let document = error(&input.to_string(), "ENG-E-0010", Status::Invalid, 2);
    let cause = document["diagnostics"][0]["cause"].as_str().unwrap();
    assert!(cause.contains("Ill-conditioned"), "{document}");

    // Just above the threshold: residual H1 band, documented as an H4
    // limitation in provenance/ENG-VAL-039.md. Pin the threshold here:
    // doubling the tolerance would reclassify this case.
    let input = series_bars(2.85e-14 * e2, e2);
    let document = error(&input.to_string(), "ENG-E-0006", Status::Unstable, 4);
    let cause = document["diagnostics"][0]["cause"].as_str().unwrap();
    assert!(!cause.contains("Ill-conditioned"), "{document}");

    // Clearly above the threshold, recovery rejects the result instead.
    let input = series_bars(4.0e-14 * e2, e2);
    let document = error(&input.to_string(), "ENG-E-0010", Status::Invalid, 2);
    let cause = document["diagnostics"][0]["cause"].as_str().unwrap();
    assert!(!cause.contains("Ill-conditioned"), "{document}");
    assert!(cause.contains("Inconsistent recovery"), "{document}");
}

#[test]
fn true_mechanism_still_reports_singular_system() {
    for (e1, e2) in [(200.0, 200.0), (200.0, 2e6)] {
        let mut input = series_bars(e1, e2);
        // Node 3 has no transverse stiffness and its Uy remains free. The
        // positive axial diagonals retain both contributions at these contrasts.
        input["model"]["supports"].as_array_mut().unwrap().pop();
        let document = error(&input.to_string(), "ENG-E-0006", Status::Unstable, 4);
        assert!(
            !document["diagnostics"][0]["cause"]
                .as_str()
                .unwrap()
                .contains("Ill-conditioned"),
            "{document}"
        );
    }
}

#[test]
fn supercritical_pdelta_columns_report_instability_without_responses() {
    for ratio in [1.2, 2.0, 10.0] {
        let document = error(
            &models::supercritical_column(ratio).to_string(),
            "ENG-E-0006",
            Status::Unstable,
            4,
        );
        assert_eq!(document["convergence"]["status"], "unstable");
    }
}

#[test]
fn repeated_modes_in_orthogonal_directions_are_finite_and_deterministic() {
    let mut input = orthogonal_bars(200.0, 200.0);
    input["model"]["masses"] = json!([
        {"id":"m2","node":2,"values":[{"dof":"Ux","value":5.0}]},
        {"id":"m3","node":3,"values":[{"dof":"Uy","value":5.0}]}]);
    models::modal(&mut input, 2);
    let document = success(&input);
    let modes = document["modes"].as_array().unwrap();
    assert_eq!(modes.len(), 2);
    for mode in modes {
        // det(K-lambda M)=0; K=(EA/L)I, M=5I gives lambda=20 twice.
        close(number(&mode["eigenvalue"]), 200.0 * 2.0 / (4.0 * 5.0));
        close(
            number(&mode["frequency_hz"]),
            math::sqrt(20.0) / std::f64::consts::TAU,
        );
        assert!(number(&mode["period"]) > 0.0);
        for node in mode["shape"].as_array().unwrap() {
            finite_array(&node["displacements"]);
        }
    }
    for direction in 0..2 {
        let total: f64 = modes
            .iter()
            .map(|mode| number(&mode["participation"][direction]["effective_mass"]))
            .sum();
        close(total, 5.0);
    }
    let raw = input.to_string();
    let first = solve(&raw).0;
    for _ in 0..4 {
        assert_eq!(first, solve(&raw).0);
    }
}

#[test]
fn massless_axial_and_rotation_dofs_are_condensed_to_a_finite_mode() {
    let mut input = models::cantilever();
    input["model"]["masses"] = json!([
        {"id":"m","node":2,"values":[{"dof":"Uy","value":5.0}]}]);
    models::modal(&mut input, 1);
    input["analyses"][0]["options"]["directions"] = json!([[0.0, 1.0, 0.0]]);
    let document = success(&input);
    assert_eq!(document["modes"].as_array().unwrap().len(), 1);
    let mode = &document["modes"][0];
    // Eliminating the free rotation gives k=3EI/L³, lambda=k/m.
    close(
        number(&mode["eigenvalue"]),
        3.0 * 200.0 * 3.0 / (math::powi(4.0, 3) * 5.0),
    );
    assert!(number(&mode["frequency_hz"]) > 0.0);
    for node in mode["shape"].as_array().unwrap() {
        finite_array(&node["displacements"]);
    }
    let shape = &mode["shape"][1]["displacements"];
    close(number(&shape[5]), 3.0 / (2.0 * 4.0) * number(&shape[1]));
}

#[test]
fn extreme_si_loads_scale_displacements_reactions_and_member_forces() {
    let baseline = success(&models::truss());
    for scale in [1e-9, 1e9] {
        let mut input = models::truss();
        input["cases"][0]["loads"][0]["values"][0]["value"] = json!(scale);
        let document = success(&input);
        equilibrium(&input, &document);
        for node in 0..2 {
            for field in ["displacements", "reactions"] {
                for component in 0..6 {
                    close(
                        number(&document["cases"][0]["nodes"][node][field][component]) / scale,
                        number(&baseline["cases"][0]["nodes"][node][field][component]),
                    );
                }
            }
        }
        let forces = document["cases"][0]["members"][0]["end_forces_local"]
            .as_array()
            .unwrap();
        for (i, force) in forces.iter().enumerate() {
            close(
                number(force) / scale,
                number(&baseline["cases"][0]["members"][0]["end_forces_local"][i]),
            );
        }
    }
}
