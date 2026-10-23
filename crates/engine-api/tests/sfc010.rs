//! MAM-835: oráculos analíticos propios y diagnósticos de las nuevas fronteras.
//! Citas, ecuaciones, unidades y tolerancias: provenance/ENG-SFC-010.md.
#[path = "support/sfc010.rs"]
mod fixtures;
mod support;
use engine_api::{canonical_json, explain, validate, CancelToken, Status};
use serde_json::{json, Value};
use support::checked_run as run;
fn input(name: &str) -> Value {
    serde_json::from_str(
        fixtures::FIXTURES
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1,
    )
    .unwrap()
}
fn result(value: &Value) -> Value {
    let o = run(&value.to_string(), &CancelToken::new(), |_| {});
    assert_eq!(o.status, Status::Ok, "{}", o.document);
    serde_json::from_str(&o.document).unwrap()
}
fn close(value: &Value, expected: f64) {
    let actual = value.as_f64().unwrap();
    assert!(
        (actual - expected).abs() <= 1e-10 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}
#[test]
fn all_new_fixtures_repeat_canonical_bytes_and_publish_only_requested_extensions() {
    for &(name, source) in fixtures::FIXTURES {
        assert!(validate(source).unwrap().is_empty(), "{name}");
        let first = run(source, &CancelToken::new(), |_| {});
        assert_eq!(first.status, Status::Ok, "{name}: {}", first.document);
        assert_eq!(canonical_json(&first.document).unwrap(), first.document);
        for _ in 0..3 {
            assert_eq!(
                run(source, &CancelToken::new(), |_| {}).document,
                first.document,
                "{name}"
            );
        }
        let d: Value = serde_json::from_str(&first.document).unwrap();
        assert_eq!(
            d.get("geotechnical").is_some(),
            matches!(name, "soil" | "consolidation")
        );
    }
}
#[test]
fn series_ground_between_and_spatial_springs_match_equilibrium() {
    let d = result(&input("spring-series"));
    close(&d["cases"][0]["nodes"][0]["displacements"][0], 0.12);
    close(&d["cases"][0]["nodes"][1]["displacements"][0], 0.18);
    close(&d["cases"][0]["members"][0]["end_forces_local"][0], 12.0);
    let d = result(&input("spring3d"));
    close(&d["cases"][0]["nodes"][0]["displacements"][0], 0.16);
    close(&d["cases"][0]["nodes"][0]["displacements"][1], 0.0);
    close(&d["cases"][0]["nodes"][0]["displacements"][2], 0.0);
    // La curva entre nodos aporta δ=0.4 además del resorte lineal δ=0.16.
    let mut v = input("multilinear");
    v["model"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":2,"dim":"D2","coordinates":[0,0,0],"dofs":["Ux"]}));
    v["model"]["springs"][0]["nodes"] = json!([1, 2]);
    v["model"]["springs"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":2,"nodes":[1],"kind":"spring.linear","dof":"Ux","k":100}));
    v["cases"][0]["loads"][0]["node"] = json!(2);
    let d = result(&v);
    close(&d["cases"][0]["nodes"][0]["displacements"][0], 0.16);
    close(&d["cases"][0]["nodes"][1]["displacements"][0], 0.56);
}
#[test]
fn nonlinear_laws_match_closed_force_displacement_and_steel_history() {
    for (name, expected) in [("multilinear", 0.4), ("cubic", 0.5)] {
        let d = result(&input(name));
        close(&d["cases"][0]["nodes"][0]["displacements"][0], expected);
        assert!(d["convergence"]["steps"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["converged"] == true));
    }
    let d = result(&input("steel"));
    let steps = d["convergence"]["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 6);
    for (step, expected) in steps.iter().zip([0.08, 0.4, 0.32, 0.24, 0.0, 0.08]) {
        close(&step["results"]["nodes"][0]["displacements"][0], expected);
        assert_eq!(step["element_states"][0].as_array().unwrap().len(), 2);
    }
    close(&d["convergence"]["steps"][1]["element_states"][0][0], 0.24);
    close(&d["convergence"]["steps"][1]["element_states"][0][1], 6.0);
}
#[test]
fn newton_adds_variable_spring_once_to_a_linear_beam_and_recovers_reactions() {
    let mut v: Value = serde_json::from_str(include_str!(
        "../fixtures/input/valid/cantilever-linear-static.json"
    ))
    .unwrap();
    v["model"]["laws"].as_array_mut().unwrap().push(json!({"name":"cubic","kind":"elastic.cubic","parameters":[{"name":"E","value":8},{"name":"C","value":16}]}));
    v["model"]["springs"] =
        json!([{"id":9,"nodes":[2],"kind":"spring.nonlinear","dof":"Uy","law":"cubic"}]);
    v["cases"][0]["loads"][0]["values"][0]["value"] = json!(-20.0625);
    v["analyses"] = json!([{"method":"nonlinearStatic","options":{"case":"D","load_factors":[0.5,1],"tolerance":1e-10,"max_iterations":25,"force_scale":20.0625,"displacement_tolerance":1e-10,"displacement_scale":1}}]);
    v["options"]["allow_experimental"] = json!(true);
    let d = result(&v);
    close(&d["cases"][0]["nodes"][1]["displacements"][1], -0.5);
    close(&d["cases"][0]["nodes"][0]["reactions"][1], 14.0625);
    close(&d["cases"][0]["nodes"][0]["reactions"][5], 56.25);
}
#[test]
fn winkler_constant_solution_and_end_releases_match_closed_beam_equilibrium() {
    for name in ["winkler2d", "winkler3d"] {
        let d = result(&input(name));
        for n in d["cases"][0]["nodes"].as_array().unwrap() {
            close(&n["displacements"][1], -2.0);
            close(&n["displacements"][5], 0.0);
            if name == "winkler3d" {
                close(&n["displacements"][2], -3.0);
                close(&n["displacements"][4], 0.0);
            }
        }
        for s in d["cases"][0]["members"][0]["stations"].as_array().unwrap() {
            close(&s["forces_local"][5], 0.0);
        }
    }
    for name in ["release2d", "release3d"] {
        let d = result(&input(name));
        for n in d["cases"][0]["nodes"].as_array().unwrap() {
            close(&n["reactions"][1], 40.0);
            if name == "release3d" {
                close(&n["reactions"][2], 120.0);
            }
        }
        let f = &d["cases"][0]["members"][0]["end_forces_local"];
        for i in if name == "release2d" {
            vec![2, 5]
        } else {
            vec![4, 5, 10, 11]
        } {
            close(&f[i], 0.0);
        }
    }
}
#[test]
fn soil_profile_preserves_hydrostatic_stress_and_parameter_provenance() {
    let d = result(&input("soil"));
    let s = &d["geotechnical"]["states"][3];
    close(&s["total_vertical_kpa"], 101.0);
    close(&s["pore_pressure_kpa"], 40.0);
    close(&s["effective_vertical_kpa"], 61.0);
    assert_eq!(s["assumed_parameters"], json!(["water_unit_weight_kn_m3"]));
    close(&d["geotechnical"]["states"][1]["pore_pressure_kpa"], 0.0);
}
#[test]
fn terzaghi_mean_degree_and_multilayer_drained_settlement_match_analytic_limits() {
    let v = input("consolidation");
    let d = result(&v);
    let g = &d["geotechnical"];
    assert_eq!(g["depths_m"].as_array().unwrap().len(), 41);
    assert!(g["assumed_parameters"]
        .as_array()
        .unwrap()
        .contains(&json!("history[0].delta_stress_kpa")));
    for s in g["states"].as_array().unwrap().iter().skip(1) {
        let tv = 0.1 * s["time_s"].as_f64().unwrap();
        let pi = std::f64::consts::PI;
        let u = 1.0
            - (0..200)
                .map(|m| {
                    let odd = (2 * m + 1) as f64;
                    8.0 / (pi * pi * odd * odd)
                        * engine_core::math::exp(-odd * odd * pi * pi * tv / 4.0)
                })
                .sum::<f64>();
        assert!(
            (s["mean_degree"].as_f64().unwrap() - u).abs() < 0.03,
            "{s}: U={u}"
        );
        assert!((s["settlement_m"].as_f64().unwrap() - 0.02 * u).abs() < 0.0006);
        close(&s["excess_pore_pressure_kpa"][0], 0.0);
    }
    let mut v = v;
    v["analyses"][0]["options"]["layers"][1]["mv_per_kpa"]["value"] = json!(0.002);
    v["analyses"][0]["options"]["layers"][1]["cv_m2_s"]["value"] = json!(0.05);
    let d = result(&v);
    let last = d["geotechnical"]["states"]
        .as_array()
        .unwrap()
        .last()
        .unwrap();
    assert!((last["settlement_m"].as_f64().unwrap() - 0.03).abs() < 1e-6);
}
#[test]
fn newton_stops_at_failed_step_preserves_accepted_history_and_cancels_between_steps() {
    let mut v = input("steel");
    v["analyses"][0]["options"]["max_iterations"] = json!(1);
    let o = run(&v.to_string(), &CancelToken::new(), |_| {});
    assert_eq!(o.status, Status::NotConverged);
    assert_eq!(o.exit_code(), 4);
    let d: Value = serde_json::from_str(&o.document).unwrap();
    let steps = d["convergence"]["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0]["converged"], true);
    assert_eq!(steps[1]["converged"], false);
    assert!(steps[1]["results"].is_null());
    close(&d["convergence"]["accepted_factor"], 0.5);
    assert_eq!(d["cases"], json!([]));
    let token = CancelToken::new();
    let mut count = 0;
    let o = run(&input("steel").to_string(), &token, |p| {
        if p.stage == "newtonStep" {
            count += 1;
            token.cancel();
        }
    });
    assert_eq!(count, 1);
    assert_eq!(o.status, Status::Cancelled);
    let d: Value = serde_json::from_str(&o.document).unwrap();
    assert!(d["convergence"].is_null());
    assert_eq!(d["cases"], json!([]));
}
fn rejects(v: &Value, code: &str) {
    let ds = validate(&v.to_string()).unwrap();
    assert!(ds.iter().any(|d| d.code == code), "{ds:?}");
    let o = run(&v.to_string(), &CancelToken::new(), |_| {});
    assert!(matches!(o.status, Status::Invalid | Status::Unsupported));
    let d: Value = serde_json::from_str(&o.document).unwrap();
    assert!(
        d["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == code),
        "{d}"
    );
    assert_eq!(d["cases"], json!([]));
    assert!(!explain(code).unwrap().is_empty());
}

fn rejects_exact(v: &Value, code: &str, status: Status, exit_code: i32) {
    let ds = validate(&v.to_string()).unwrap();
    assert_eq!(ds.iter().map(|d| d.code).collect::<Vec<_>>(), [code]);
    let o = run(&v.to_string(), &CancelToken::new(), |_| {});
    assert_eq!(o.status, status, "{}", o.document);
    assert_eq!(o.exit_code(), exit_code);
    let d: Value = serde_json::from_str(&o.document).unwrap();
    assert_eq!(d["status"], serde_json::to_value(status).unwrap());
    assert_eq!(d["diagnostics"].as_array().unwrap().len(), 1);
    assert_eq!(d["diagnostics"][0]["code"], code);
    assert_eq!(d["cases"], json!([]));
    assert!(!explain(code).unwrap().is_empty());
}

#[test]
fn combined_member_and_spring_limit_accepts_1024_and_rejects_1025() {
    let mut v = input("release2d");
    v["model"]["springs"] = json!((100..1123)
        .map(|id| json!({"id":id,"nodes":[2],"kind":"spring.linear","dof":"Uy","k":1}))
        .collect::<Vec<_>>());
    assert_eq!(v["model"]["members"].as_array().unwrap().len(), 1);
    assert_eq!(v["model"]["springs"].as_array().unwrap().len(), 1023);
    assert!(validate(&v.to_string()).unwrap().is_empty());
    let o = run(&v.to_string(), &CancelToken::new(), |_| {});
    assert_eq!(o.status, Status::Ok, "{}", o.document);
    v["model"]["springs"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":1123,"nodes":[2],"kind":"spring.linear","dof":"Uy","k":1}));
    rejects_exact(&v, "API-E-SFC-OPTIONS", Status::Invalid, 2);
}

#[test]
fn winkler_data_requires_winkler_beam_and_beams_reject_nonlinear_laws() {
    let mut v = input("winkler2d");
    v["model"]["members"][0]["element"] = json!("frame2d.euler_elastic");
    rejects_exact(&v, "API-E-SFC-OPTIONS", Status::Invalid, 2);
    let mut v = input("release2d");
    v["model"]["laws"][0] = input("steel")["model"]["laws"][0].clone();
    v["model"]["members"][0]["law"] = json!("law");
    rejects_exact(&v, "ENG-E-0013", Status::Unsupported, 3);
}

#[test]
fn newton_rejects_combinations_and_modal_spectrum_reject_springs() {
    let mut v = input("steel");
    v["combinations"] = json!([{"id":"C","terms":[{"case":"P","factor":1}]}]);
    rejects_exact(&v, "ENG-E-0013", Status::Unsupported, 3);
    for source in [
        include_str!("fixtures/determinism/modal-1dof.json"),
        include_str!("fixtures/determinism/spectrum-log-cqc.json"),
    ] {
        let reference: Value = serde_json::from_str(source).unwrap();
        let mut v = input("spring-series");
        v["analyses"] = reference["analyses"].clone();
        rejects_exact(&v, "ENG-E-0013", Status::Unsupported, 3);
    }
}

#[test]
fn consolidation_rejects_total_work_quickly_and_without_overflow() {
    use std::time::{Duration, Instant};
    assert_eq!(engine_api::MAX_CONSOLIDATION_WORK, 50_000_000);
    let mut v = input("consolidation");
    let options = &mut v["analyses"][0]["options"];
    options["history"] = json!([0, 800_000, 1_600_000]
        .map(|t| json!({"time_s":t,"delta_stress_kpa":{"value":20,"provenance":"measured"}})));
    options["max_step_s"] = json!(1);
    // Se cuenta toda la historia, incluso si solo se solicita la salida inicial.
    options["output_times_s"] = json!([0]);
    let source = v.to_string();
    let start = Instant::now();
    let diagnostics = validate(&source).unwrap();
    assert!(start.elapsed() < Duration::from_secs(2));
    assert_eq!(diagnostics[0].code, "API-E-CONSOLIDATION-WORK");
    rejects_exact(&v, "API-E-CONSOLIDATION-WORK", Status::Invalid, 2);
    // Un cociente infinito también debe rechazarse sin desbordar.
    v["analyses"][0]["options"]["max_step_s"] = json!(f64::MIN_POSITIVE);
    rejects_exact(&v, "API-E-CONSOLIDATION-WORK", Status::Invalid, 2);
}

#[test]
fn consolidation_retains_work_limit_independently_of_the_dense_budget() {
    let mut v = input("consolidation");
    let options = &mut v["analyses"][0]["options"];
    options["history"][1]["time_s"] = json!(1_250_000);
    options["output_times_s"] = json!([0, 625_000, 1_250_000]);
    options["max_step_s"] = json!(1);
    // La cota r2 exacta sigue permitiendo este trabajo; r3 lo rechaza por coste.
    rejects_exact(&v, "API-E-CONSOLIDATION-COST", Status::Invalid, 2);
    // ceil(0.5) + ceil(624999.5) + ceil(625000) añade un subpaso.
    v["analyses"][0]["options"]["output_times_s"] = json!([0, 0.5, 625_000, 1_250_000]);
    rejects_exact(&v, "API-E-CONSOLIDATION-WORK", Status::Invalid, 2);
}

fn dense_consolidation(elements: usize, segments: usize, steps: usize) -> Value {
    let mut v = input("consolidation");
    let o = &mut v["analyses"][0]["options"];
    let mut layer = o["layers"][0].clone();
    layer["elements"] = json!(elements);
    o["layers"] = json!([layer]);
    o["history"] = json!((0..=segments)
        .map(|t| json!({
            "time_s": t, "delta_stress_kpa": {"value":20 + t % 3,"provenance":"measured"}
        }))
        .collect::<Vec<_>>());
    o["output_times_s"] = json!([0, segments]);
    o["max_step_s"] = json!(1.0 / steps as f64);
    v
}

#[test]
fn dense_cost_rejects_large_meshes_below_the_r2_work_limit_in_under_two_seconds() {
    use std::time::{Duration, Instant};
    for (segments, steps) in [(1, 1), (24, 1), (1, 24_000)] {
        let v = dense_consolidation(2048, segments, steps);
        let source = v.to_string();
        let start = Instant::now();
        assert_eq!(
            validate(&source).unwrap()[0].code,
            "API-E-CONSOLIDATION-COST"
        );
        rejects_exact(&v, "API-E-CONSOLIDATION-COST", Status::Invalid, 2);
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    let description = explain("API-E-CONSOLIDATION-COST").unwrap();
    for text in ["500000000", "256", "100000", "free nodes", "u128"] {
        assert!(description.contains(text), "{description}");
    }
}

#[test]
fn dense_cost_counts_free_nodes_output_splits_and_the_exact_limit() {
    assert_eq!(engine_api::MAX_CONSOLIDATION_COST, 500_000_000);
    // n=100: 100³ + 49900·100² = 500000000 exactamente.
    let mut v = dense_consolidation(100, 1, 1);
    v["analyses"][0]["options"]["history"][1]["time_s"] = json!(49_900);
    v["analyses"][0]["options"]["output_times_s"] = json!([0, 49_900]);
    assert!(validate(&v.to_string()).unwrap().is_empty());
    v["analyses"][0]["options"]["output_times_s"] = json!([0, 0.5, 49_900]);
    rejects_exact(&v, "API-E-CONSOLIDATION-COST", Status::Invalid, 2);
    v["analyses"][0]["options"]["output_times_s"] = json!([0, 49_900]);
    v["analyses"][0]["options"]["layers"][0]["elements"] = json!(101);
    rejects_exact(&v, "API-E-CONSOLIDATION-COST", Status::Invalid, 2);
    v["analyses"][0]["options"]["drainage"] = json!("double");
    assert!(validate(&v.to_string()).unwrap().is_empty());
    // También se cobra la historia posterior a la última salida.
    v["analyses"][0]["options"]["output_times_s"] = json!([0]);
    v["analyses"][0]["options"]["history"][1]["time_s"] = json!(49_901);
    rejects_exact(&v, "API-E-CONSOLIDATION-COST", Status::Invalid, 2);
}

#[test]
fn dense_budget_bounds_history_output_intervals_and_small_system_substeps() {
    assert_eq!(engine_api::MAX_CONSOLIDATION_SEGMENTS, 256);
    assert_eq!(engine_api::MAX_CONSOLIDATION_SUBSTEPS, 100_000);
    assert!(validate(&dense_consolidation(1, 256, 1).to_string())
        .unwrap()
        .is_empty());
    rejects_exact(
        &dense_consolidation(1, 257, 1),
        "API-E-CONSOLIDATION-COST",
        Status::Invalid,
        2,
    );
    let mut v = dense_consolidation(1, 1, 1);
    v["analyses"][0]["options"]["history"][1]["time_s"] = json!(257);
    v["analyses"][0]["options"]["output_times_s"] = json!((0..=256).collect::<Vec<_>>());
    // 256 salidas positivas/iniciales más el tramo conservador posterior.
    rejects_exact(&v, "API-E-CONSOLIDATION-COST", Status::Invalid, 2);
    v["analyses"][0]["options"]["output_times_s"] = json!([0, 257]);
    v["analyses"][0]["options"]["history"][1]["time_s"] = json!(100_000);
    v["analyses"][0]["options"]["output_times_s"] = json!([0, 100_000]);
    assert!(validate(&v.to_string()).unwrap().is_empty());
    v["analyses"][0]["options"]["history"][1]["time_s"] = json!(100_001);
    v["analyses"][0]["options"]["output_times_s"] = json!([0]);
    for drainage in ["top", "double"] {
        v["analyses"][0]["options"]["drainage"] = json!(drainage);
        v["analyses"][0]["options"]["layers"][0]["elements"] =
            json!(if drainage == "double" { 2 } else { 1 });
        rejects_exact(&v, "API-E-CONSOLIDATION-COST", Status::Invalid, 2);
    }
}

#[test]
fn near_dense_cost_limit_completes_in_release_within_five_seconds() {
    let v = dense_consolidation(793, 1, 2);
    // 793³ + 2·793² = 499934955; un tercer subpaso supera el presupuesto.
    assert!(validate(&v.to_string()).unwrap().is_empty());
    let start = std::time::Instant::now();
    let d = result(&v);
    assert_eq!(d["geotechnical"]["depths_m"].as_array().unwrap().len(), 794);
    if !cfg!(debug_assertions) {
        assert!(start.elapsed() < std::time::Duration::from_secs(5));
    }
    rejects_exact(
        &dense_consolidation(793, 1, 3),
        "API-E-CONSOLIDATION-COST",
        Status::Invalid,
        2,
    );
}

#[test]
fn consolidation_cancels_a_long_valid_run_with_the_existing_diagnostic() {
    use std::{
        sync::Arc,
        thread,
        time::{Duration, Instant},
    };
    let mut v = input("consolidation");
    v["analyses"][0]["options"]["history"][1]["time_s"] = json!(100_000);
    v["analyses"][0]["options"]["output_times_s"] = json!([0, 100_000]);
    v["analyses"][0]["options"]["max_step_s"] = json!(1);
    let source = v.to_string();
    assert!(validate(&source).unwrap().is_empty());
    let token = Arc::new(CancelToken::new());
    // El callback de progreso model precede al despacho geotécnico.
    let mut worker = None;
    let start = Instant::now();
    let o = run(&source, &token, |p| {
        if p.stage == "model" {
            let token = Arc::clone(&token);
            worker = Some(thread::spawn(move || {
                thread::sleep(Duration::from_millis(20));
                token.cancel();
            }));
        }
    });
    worker.unwrap().join().unwrap();
    assert!(start.elapsed() < Duration::from_secs(2));
    assert_eq!(o.status, Status::Cancelled, "{}", o.document);
    assert_eq!(o.exit_code(), 5);
    let d: Value = serde_json::from_str(&o.document).unwrap();
    assert_eq!(d["status"], "cancelled");
    assert_eq!(d["diagnostics"][0]["code"], "API-I-CANCELLED");
    assert_eq!(d["cases"], json!([]));
    assert!(d.get("geotechnical").is_none());
}
#[test]
fn stable_diagnostics_cover_incompatible_elements_laws_newton_and_geo_data() {
    let mut v = input("spring-series");
    v["model"]["springs"][0]["id"] = json!(2);
    rejects(&v, "ENG-E-0002");
    let mut v = input("spring-series");
    v["model"]["springs"][0]["nodes"] = json!([99]);
    rejects(&v, "ENG-E-0003");
    let mut v = input("spring-series");
    v["model"]["springs"][0]["k"] = json!(-1);
    rejects(&v, "ENG-E-0010");
    let mut v = input("spring-series");
    v["model"]["springs"][0]["dof"] = json!("Uy");
    rejects(&v, "ENG-E-0011");
    let mut v = input("multilinear");
    v["analyses"] = json!([{"method":"linearStatic"}]);
    rejects(&v, "ENG-E-0013");
    let mut v = input("multilinear");
    v["model"]["laws"][0]["points"] = json!([[0, 0], [1, -1]]);
    rejects(&v, "ENG-E-0010");
    let mut v = input("steel");
    v["analyses"][0]["options"]["load_factors"] = json!([]);
    rejects(&v, "API-E-SFC-OPTIONS");
    let mut v = input("steel");
    v["analyses"][0]["options"]["displacement_tolerance"] = json!(1e-8);
    rejects(&v, "API-E-SFC-OPTIONS");
    let mut v = input("steel");
    v["analyses"][0]["options"]["case"] = json!("missing");
    rejects(&v, "ENG-E-0003");
    let mut v = input("steel");
    v["model"]["laws"][0]["parameters"][2]["value"] = json!(1);
    rejects(&v, "ENG-E-0010");
    let mut v = input("winkler2d");
    v["model"]["members"][0]["release_i"] = json!(true);
    rejects(&v, "API-E-SFC-OPTIONS");
    let mut v = input("release2d");
    v["model"]["members"][0]["release_i"] = json!([true, false, false]);
    rejects(&v, "ENG-E-0010");
    let mut v = input("soil");
    v["analyses"][0]["options"]
        .as_object_mut()
        .unwrap()
        .remove("water_unit_weight_kn_m3");
    rejects(&v, "ENG-E-0003");
    let mut v = input("soil");
    v["analyses"][0]["options"]["depths_m"] = json!([6]);
    rejects(&v, "ENG-E-0010");
    let mut v = input("consolidation");
    v["analyses"][0]["options"]["layers"][0]["cv_m2_s"]["value"] = json!(2);
    rejects(&v, "ENG-E-0010");
    let mut v = input("consolidation");
    v["analyses"][0]["options"]["history"][1]["time_s"] = json!(0);
    rejects(&v, "ENG-E-0010");
}
#[test]
fn nulls_unknown_fields_and_wrong_shapes_are_rejected_before_physics() {
    for (name, path, value) in [
        ("steel", vec!["model", "laws"], Value::Null),
        ("soil", vec!["options"], json!({"extra":true})),
    ] {
        let mut v = input(name);
        let mut p = &mut v;
        for key in path {
            p = &mut p[key];
        }
        *p = value;
        assert!(validate(&v.to_string()).is_err());
        assert_eq!(
            run(&v.to_string(), &CancelToken::new(), |_| {}).status,
            Status::Invalid
        );
    }
    let mut v = input("steel");
    v["model"]["springs"][0]["direction"] = Value::Null;
    assert!(validate(&v.to_string()).is_err());
}

#[test]
fn newton_reports_singular_tangent_without_accepting_the_failed_trial() {
    let mut v = input("steel");
    v["model"]["laws"][0]["parameters"][2]["value"] = json!(0);
    let o = run(&v.to_string(), &CancelToken::new(), |_| {});
    assert_eq!(o.status, Status::Unstable, "{}", o.document);
    let d: Value = serde_json::from_str(&o.document).unwrap();
    assert_eq!(d["convergence"]["status"], "unstable");
    close(&d["convergence"]["accepted_factor"], 0.5);
    close(&d["convergence"]["failed_factor"], 1.0);
    assert_eq!(d["convergence"]["steps"].as_array().unwrap().len(), 1);
    assert_eq!(d["cases"], json!([]));
}
