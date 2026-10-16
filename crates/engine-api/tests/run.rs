//! Oráculos SI derivados a mano, sin implementación ni datos de otros motores.
use engine_api::{canonical_json, exit_code, CancelToken, Outcome, Status};
use engine_core::math;
use serde_json::{json, Value};
mod support;
use support::checked_run as run;

fn input() -> Value {
    json!({"schema": "engine-input/1", "units": "SI", "options": {},
        "model": {"axes": "Z-up", "nodes": [
            {"id": 1, "dim": "D2", "coordinates": [0.0,0.0,0.0], "dofs": ["Ux","Uy","Rz"]},
            {"id": 2, "dim": "D2", "coordinates": [4.0,0.0,0.0], "dofs": ["Ux","Uy","Rz"]}],
            "members": [{"id": 7, "nodes": [1,2], "section": "s", "law": "elastic"}],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 2.0}, {"name": "I", "value": 3.0}]}],
            "laws": [{"name": "elastic", "parameters": [{"name": "E", "value": 200.0}, {"name": "rho", "value": 0.0}]}],
            "supports": [{"id": "fixed", "node": 1, "restrained": ["Ux","Uy","Rz"]}], "masses": []},
        "cases": [{"name": "D", "loads": [{"node": 2, "values": [{"dof": "Uy", "value": -12.0}]}]}],
        "combinations": [], "analyses": [{"method": "linearStatic"}]})
}
fn solve(v: &Value) -> (Outcome, Value) {
    let outcome = run(&v.to_string(), &CancelToken::new(), |_| {});
    let document = serde_json::from_str(&outcome.document).unwrap();
    (outcome, document)
}
fn number(v: &Value) -> f64 {
    v.as_f64().unwrap()
}
fn close(v: &Value, expected: f64) {
    let actual = number(v);
    assert!(
        (actual - expected).abs() < 1e-10 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}

#[test]
fn cantilever_tip_point_load_analytical_displacement_and_fixed_moment() {
    // Equilibrio: R=P, M=PL. Integrando EI y''=−P(L−x), y(0)=y'(0)=0:
    // y(L)=−PL³/(3EI). E=200, I=3, L=4, P=12.
    let (o, d) = solve(&input());
    assert_eq!(o.status, Status::Ok);
    assert_eq!(o.exit_code(), 0);
    close(
        &d["cases"][0]["nodes"][1]["displacements"][1],
        -12.0 * math::powi(4.0_f64, 3) / (3.0 * 200.0 * 3.0),
    );
    close(&d["cases"][0]["nodes"][0]["reactions"][1], 12.0);
    close(&d["cases"][0]["nodes"][0]["reactions"][5], 48.0);
    close(&d["cases"][0]["members"][0]["end_forces_local"][2], 48.0);
    assert_eq!(d["members"][0]["element"], "frame2d.euler_elastic");
    assert_eq!(d["nodes"][1]["coordinates"], json!([4.0, 0.0, 0.0]));
}

#[test]
fn simply_supported_uniform_load_analytical_peak_and_reactions() {
    // ΣF=0, simetría: R_i=R_j=wL/2. M(x)=w x(L−x)/2,
    // M'(x)=w(L/2−x)=0 en L/2, Mmax=wL²/8.
    let mut v = input();
    v["model"]["supports"] = json!([
        {"id": "pin", "node": 1, "restrained": ["Ux","Uy"]},
        {"id": "roller", "node": 2, "restrained": ["Uy"]}]);
    v["cases"] = json!([{"name": "D", "loads": [], "member_loads": [
        {"member": 7, "force_per_length": [0.0,-6.0,0.0], "moment_per_length": [0.0,0.0,0.0]}]}]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 0, "{}", o.document);
    for index in [0, 1] {
        close(
            &d["cases"][0]["nodes"][index]["reactions"][1],
            6.0 * 4.0 / 2.0,
        );
    }
    close(
        &d["cases"][0]["members"][0]["extremes"][5]["max"]["value"],
        6.0 * 4.0 * 4.0 / 8.0,
    );
    close(
        &d["cases"][0]["members"][0]["extremes"][5]["max"]["at_station"],
        0.5,
    );
}

#[test]
fn combination_12d_16l_superposes_all_fields_once_and_warns_prescribed_output() {
    let mut v = input();
    v["cases"].as_array_mut().unwrap().push(
        json!({"name": "L", "loads": [{"node": 2, "values": [{"dof": "Uy", "value": -5.0}]}]}),
    );
    v["combinations"] = json!([{"id": "ULS", "terms": [{"case": "D", "factor": 1.2}, {"case": "L", "factor": 1.6}]}]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 1);
    assert_eq!(d["status"], "warning");
    assert_eq!(d["diagnostics"][0]["code"], "API-W-COMBINED-PRESCRIBED");
    let c = &d["combinations"][0]["results"];
    for (array, fields) in [("displacements", 6), ("reactions", 6)] {
        for node in 0..2 {
            for component in 0..fields {
                close(
                    &c["nodes"][node][array][component],
                    1.2 * number(&d["cases"][0]["nodes"][node][array][component])
                        + 1.6 * number(&d["cases"][1]["nodes"][node][array][component]),
                );
            }
        }
    }
    for component in 0..6 {
        close(
            &c["members"][0]["end_forces_local"][component],
            1.2 * number(&d["cases"][0]["members"][0]["end_forces_local"][component])
                + 1.6 * number(&d["cases"][1]["members"][0]["end_forces_local"][component]),
        );
    }
    close(&c["nodes"][0]["reactions"][1], 1.2 * 12.0 + 1.6 * 5.0);
}

#[test]
fn deterministic_bytes_and_sha_with_recursive_key_permutation_and_negative_zero() {
    fn reverse_keys(v: &Value) -> String {
        match v {
            Value::Object(m) => format!(
                "{{{}}}",
                m.iter()
                    .rev()
                    .map(|(k, v)| format!("{}:{}", json!(k), reverse_keys(v)))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Value::Array(a) => format!(
                "[{}]",
                a.iter().map(reverse_keys).collect::<Vec<_>>().join(",")
            ),
            Value::Number(n) if n.is_f64() && n.as_f64() == Some(0.0) => "-0.0".into(),
            _ => v.to_string(),
        }
    }
    let v = input();
    let a = v.to_string();
    let b = reverse_keys(&v);
    let first = run(&a, &CancelToken::new(), |_| {});
    assert_eq!(first, run(&a, &CancelToken::new(), |_| {}));
    assert_eq!(first, run(&b, &CancelToken::new(), |_| {}));
    assert_eq!(first.document, canonical_json(&first.document).unwrap());
    let document: Value = serde_json::from_str(&first.document).unwrap();
    assert_eq!(
        document["input_sha256"],
        mamut_canon::fingerprint(&a).unwrap().value
    );
    for key in ["timestamp", "path"] {
        assert!(document.get(key).is_none());
    }
}

#[test]
fn invalid_nonfinite_schema_and_missing_references_exit_two() {
    let baseline = input();
    for bad in [
        "NaN",
        "Infinity",
        "-Infinity",
        "1e999",
        "-1e400",
        "18446744073709551616",
        "-9223372036854775809",
    ] {
        let o = run(
            &baseline.to_string().replace("-12.0", bad),
            &CancelToken::new(),
            |_| {},
        );
        assert_eq!(o.exit_code(), 2, "{}", o.document);
        let d: Value = serde_json::from_str(&o.document).unwrap();
        assert!(d["input_sha256"].is_null());
    }
    for (pointer, replacement) in [
        ("/schema", json!("engine-input/2")),
        ("/model/members/0/nodes", json!([1, 99])),
        (
            "/combinations",
            json!([{"id":"bad","terms":[{"case":"missing","factor":0.0}]}]),
        ),
    ] {
        let mut v = baseline.clone();
        *v.pointer_mut(pointer).unwrap() = replacement;
        let (o, d) = solve(&v);
        assert_eq!(o.exit_code(), 2, "{}", o.document);
        assert_eq!(d["cases"], json!([]));
        assert!(!d["diagnostics"].as_array().unwrap().is_empty());
    }
}

#[test]
fn cancelled_before_start_and_every_boundary_discards_partial_results() {
    let mut v = input();
    v["cases"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"L", "loads":[]}));
    for boundary in 0..=7 {
        let cancel = CancelToken::new();
        if boundary == 0 {
            cancel.cancel();
        }
        let mut events = Vec::new();
        let o = run(&v.to_string(), &cancel, |p| {
            events.push(p);
            if p.completed == boundary {
                cancel.cancel();
            }
        });
        assert_eq!(o.exit_code(), 5, "boundary {boundary}: {}", o.document);
        assert_eq!(events.len(), boundary);
        let d: Value = serde_json::from_str(&o.document).unwrap();
        for key in ["nodes", "members", "cases", "combinations"] {
            assert_eq!(d[key], json!([]));
        }
        assert_eq!(d["status"], "cancelled");
    }
    let mut events = Vec::new();
    let o = run(&v.to_string(), &CancelToken::new(), |p| events.push(p));
    assert_eq!(o.exit_code(), 0);
    assert_eq!(
        events.iter().map(|p| p.stage).collect::<Vec<_>>(),
        [
            "parse",
            "model",
            "assembly",
            "case",
            "case",
            "combinations",
            "serialization"
        ]
    );
    assert!(events
        .iter()
        .enumerate()
        .all(|(i, p)| p.operation == "run" && p.total == 7 && p.completed == i + 1));
}

#[test]
fn missing_method_options_exit_two_and_unknown_elements_exit_three() {
    for method in ["pDelta", "modal", "responseSpectrum"] {
        let mut v = input();
        v["analyses"] = json!([{"method":method}]);
        let (o, d) = solve(&v);
        assert_eq!(o.exit_code(), 2);
        assert_eq!(d["diagnostics"][0]["code"], "API-E-INPUT");
        assert_eq!(d["cases"], json!([]));
    }
    let mut v = input();
    v["model"]["members"][0]["element"] = json!("unknown");
    assert_eq!(solve(&v).0.exit_code(), 3);
}

#[test]
fn unstable_model_reports_mechanism_and_exit_four() {
    let mut v = input();
    v["model"]["supports"] = json!([]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 4, "{}", o.document);
    assert_eq!(o.status, Status::Unstable);
    assert_eq!(d["diagnostics"][0]["code"], "ENG-E-0006");
}

#[test]
fn prescribed_combination_scales_delta_with_warning_and_repeated_settlements_fail() {
    let mut v = input();
    v["model"]["prescribed"] = json!([{"node":1,"dof":"Uy","value":0.125}]);
    v["cases"][0]["loads"] = json!([]);
    v["combinations"] = json!([{"id":"settlement","terms":[{"case":"D","factor":2.0}]}]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 1, "{}", o.document);
    close(
        &d["cases"][0]["nodes"][0]["prescribed_displacements"][1],
        0.125,
    );
    close(
        &d["combinations"][0]["results"]["nodes"][0]["prescribed_displacements"][1],
        0.25,
    );
    v["cases"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"L","loads":[]}));
    v["combinations"][0]["terms"]
        .as_array_mut()
        .unwrap()
        .push(json!({"case":"L","factor":0.0}));
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 2);
    assert_eq!(d["diagnostics"][0]["code"], "ENG-E-0010");
    assert_eq!(
        d["diagnostics"][0]["entities"],
        json!([{"kind":"case","id":"D"},{"kind":"case","id":"L"}])
    );
}

#[test]
fn invalid_physical_parameters_and_extension_references_are_rejected() {
    for (pointer, replacement) in [
        ("/model/laws/0/parameters/0/value", json!(-200.0)),
        ("/model/sections/0/properties", json!([])),
        ("/model/members/0/orientation", Value::Null),
        (
            "/model/prescribed",
            json!([{"node":99,"dof":"Uy","value":0.0}]),
        ),
        (
            "/model/prescribed",
            json!([{"node":1,"dof":"Uy","value":0.0},{"node":1,"dof":"Uy","value":0.0}]),
        ),
        (
            "/cases/0/member_loads",
            json!([{"member":99,"force_per_length":[0.0,1.0,0.0],"moment_per_length":[0.0,0.0,0.0]}]),
        ),
    ] {
        let mut v = input();
        let parts: Vec<_> = pointer.trim_start_matches('/').split('/').collect();
        if let Some(slot) = v.pointer_mut(pointer) {
            *slot = replacement;
        } else {
            let parent = format!("/{}", parts[..parts.len() - 1].join("/"));
            v.pointer_mut(&parent).unwrap()[parts[parts.len() - 1]] = replacement;
        }
        let (o, _) = solve(&v);
        assert_eq!(o.exit_code(), 2, "{pointer}: {}", o.document);
    }
}

#[test]
fn exit_code_mapping_is_complete_including_reserved_timeout() {
    for (status, code) in [
        (Status::Ok, 0),
        (Status::Warning, 1),
        (Status::Invalid, 2),
        (Status::Unsupported, 3),
        (Status::NotConverged, 4),
        (Status::Unstable, 4),
        (Status::Cancelled, 5),
        (Status::Timeout, 6),
        (Status::InternalError, 70),
    ] {
        assert_eq!(exit_code(status), code);
    }
}

#[test]
fn diaphragm_output_always_has_explicit_recovery_warning() {
    // Smoke de frontera; no se declara cerrada la cobertura de dominio de #84.
    let mut v = input();
    v["model"]["nodes"] = json!([
        {"id":1,"dim":"D3","coordinates":[0.0,0.0,0.0],"dofs":["Ux","Uy","Uz","Rx","Ry","Rz"]},
        {"id":2,"dim":"D3","coordinates":[4.0,0.0,0.0],"dofs":["Ux","Uy","Uz","Rx","Ry","Rz"]}]);
    v["model"]["supports"][0]["restrained"] = json!(["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"]);
    v["model"]["members"][0]["orientation"] = json!([0.0, 0.0, 1.0]);
    v["model"]["sections"][0]["properties"] = json!([
        {"name":"A","value":2.0},{"name":"J","value":3.0},{"name":"Iy","value":3.0},{"name":"Iz","value":3.0}]);
    v["model"]["laws"][0]["parameters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"G","value":80.0}));
    v["model"]["diaphragms"] = json!([{"master":1,"slaves":[2]}]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 1, "{}", o.document);
    assert_eq!(d["diagnostics"][0]["code"], "API-W-DIAPHRAGM-RECOVERY");
    assert_eq!(d["cases"][0]["members"].as_array().unwrap().len(), 1);
    close(&d["cases"][0]["nodes"][0]["reactions"][1], 12.0);
}

#[test]
fn axial_truss_selection_uses_its_own_dofs_and_analytical_stiffness() {
    let mut v = input();
    for n in v["model"]["nodes"].as_array_mut().unwrap() {
        n["dofs"] = json!(["Ux", "Uy"]);
    }
    v["model"]["members"][0]["element"] = json!("truss2d.linear");
    v["model"]["sections"][0]["properties"] = json!([{"name":"A","value":2.0}]);
    v["model"]["supports"] = json!([
        {"id":"pin","node":1,"restrained":["Ux","Uy"]},
        {"id":"guide","node":2,"restrained":["Uy"]}]);
    v["cases"][0]["loads"][0]["values"] = json!([{"dof":"Ux","value":12.0}]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 0, "{}", o.document);
    // EA δ/L = P → δ=PL/EA.
    close(
        &d["cases"][0]["nodes"][1]["displacements"][0],
        12.0 * 4.0 / (200.0 * 2.0),
    );
    close(
        &d["cases"][0]["members"][0]["stations"][0]["forces_local"][0],
        12.0,
    );
}

#[test]
fn spatial_frame_cantilever_uses_z_load_and_y_bending_inertia() {
    let mut v = input();
    for node in v["model"]["nodes"].as_array_mut().unwrap() {
        node["dim"] = json!("D3");
        node["dofs"] = json!(["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"]);
    }
    v["model"]["supports"][0]["restrained"] = json!(["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"]);
    v["model"]["members"][0]["orientation"] = json!([0.0, 0.0, 1.0]);
    v["model"]["sections"][0]["properties"] = json!([
        {"name":"A","value":2.0}, {"name":"J","value":3.0},
        {"name":"Iy","value":3.0}, {"name":"Iz","value":5.0}]);
    v["model"]["laws"][0]["parameters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"G","value":80.0}));
    v["cases"][0]["loads"][0]["values"][0]["dof"] = json!("Uz");
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 0, "{}", o.document);
    assert_eq!(d["members"][0]["element"], "frame3d.euler_elastic");
    // EI_y z'' = −P(L−x), z(0)=z'(0)=0 → z(L)=−PL³/(3EI_y).
    close(
        &d["cases"][0]["nodes"][1]["displacements"][2],
        -12.0 * 64.0 / (3.0 * 200.0 * 3.0),
    );
    close(&d["cases"][0]["nodes"][0]["reactions"][2], 12.0);
    // r × F = +PL e_y; el apoyo ejerce −PL e_y.
    close(&d["cases"][0]["nodes"][0]["reactions"][4], -48.0);
    close(&d["cases"][0]["members"][0]["end_forces_local"][4], -48.0);
}

#[test]
fn spatial_truss_selection_recovers_axial_analytical_response() {
    let mut v = input();
    for node in v["model"]["nodes"].as_array_mut().unwrap() {
        node["dim"] = json!("D3");
        node["dofs"] = json!(["Ux", "Uy", "Uz"]);
    }
    v["model"]["members"][0]["element"] = json!("truss3d.linear");
    v["model"]["sections"][0]["properties"] = json!([{"name":"A","value":2.0}]);
    v["model"]["supports"] = json!([
        {"id":"pin","node":1,"restrained":["Ux","Uy","Uz"]},
        {"id":"guide","node":2,"restrained":["Uy","Uz"]}]);
    v["cases"][0]["loads"][0]["values"] = json!([{"dof":"Ux","value":12.0}]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 0, "{}", o.document);
    assert_eq!(d["members"][0]["element"], "truss3d.linear");
    // EA δ/L = P → δ=PL/EA; equilibrio axial R=−P.
    close(
        &d["cases"][0]["nodes"][1]["displacements"][0],
        12.0 * 4.0 / (200.0 * 2.0),
    );
    close(&d["cases"][0]["nodes"][0]["reactions"][0], -12.0);
    close(
        &d["cases"][0]["members"][0]["stations"][0]["forces_local"][0],
        12.0,
    );
}

#[test]
fn recovery_includes_every_member_in_a_two_span_cantilever() {
    let mut v = input();
    v["model"]["nodes"][1]["coordinates"] = json!([2.0, 0.0, 0.0]);
    v["model"]["nodes"].as_array_mut().unwrap().push(json!({
        "id":3,"dim":"D2","coordinates":[4.0,0.0,0.0],"dofs":["Ux","Uy","Rz"]}));
    v["model"]["members"].as_array_mut().unwrap().push(json!({
        "id":8,"nodes":[2,3],"section":"s","law":"elastic"}));
    v["cases"][0]["loads"][0]["node"] = json!(3);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 0, "{}", o.document);
    assert_eq!(d["cases"][0]["members"].as_array().unwrap().len(), 2);
    close(
        &d["cases"][0]["nodes"][2]["displacements"][1],
        -12.0 * math::powi(4.0_f64, 3) / (3.0 * 200.0 * 3.0),
    );
    close(&d["cases"][0]["nodes"][0]["reactions"][5], 48.0);
    let selected = o
        .query(&engine_api::ResultQuery {
            node_ids: Some(&[engine_core::NodeId(3), engine_core::NodeId(1)]),
            member_ids: Some(&[engine_core::MemberId(8)]),
            fields: None,
        })
        .unwrap();
    support::assert_results(&selected);
    assert_eq!(
        selected["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [1, 3]
    );
    assert_eq!(selected["members"].as_array().unwrap().len(), 1);
    assert_eq!(selected["members"][0]["id"], 8);
    assert_eq!(selected["members"][0]["nodes"], json!([2, 3]));
    assert_eq!(
        selected["cases"][0]["members"],
        json!([d["cases"][0]["members"][1]])
    );
}

#[test]
fn combinations_share_stations_when_individual_critical_points_differ() {
    let mut v = input();
    v["model"]["supports"] = json!([
        {"id":"pin","node":1,"restrained":["Ux","Uy"]},
        {"id":"roller","node":2,"restrained":["Uy"]}]);
    let load =
        json!({"member":7,"force_per_length":[0.0,-6.0,0.0],"moment_per_length":[0.0,0.0,0.0]});
    v["cases"] = json!([
        {"name":"D","loads":[],"member_loads":[load]},
        {"name":"L","loads":[{"node":1,"values":[{"dof":"Rz","value":12.0}]}],"member_loads":[load]}]);
    v["combinations"] =
        json!([{"id":"C","terms":[{"case":"D","factor":1.2},{"case":"L","factor":1.6}]}]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 1, "{}", o.document);
    let stations_a = d["cases"][0]["members"][0]["stations"].as_array().unwrap();
    let stations_b = d["cases"][1]["members"][0]["stations"].as_array().unwrap();
    assert!(stations_a.len() > 5); // Incluye la raíz distinta del segundo caso.
    assert_eq!(
        stations_a
            .iter()
            .map(|s| &s["position"])
            .collect::<Vec<_>>(),
        stations_b
            .iter()
            .map(|s| &s["position"])
            .collect::<Vec<_>>()
    );
    for (index, station) in d["combinations"][0]["results"]["members"][0]["stations"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        close(
            &station["forces_local"][5],
            1.2 * number(&stations_a[index]["forces_local"][5])
                + 1.6 * number(&stations_b[index]["forces_local"][5]),
        );
    }
}

#[test]
fn local_diagnostics_have_public_explanations() {
    for code in [
        "API-E-INPUT",
        "API-I-CANCELLED",
        "API-W-COMBINED-PRESCRIBED",
        "API-W-DIAPHRAGM-RECOVERY",
    ] {
        assert!(!engine_api::explain(code).unwrap().is_empty());
    }
}
