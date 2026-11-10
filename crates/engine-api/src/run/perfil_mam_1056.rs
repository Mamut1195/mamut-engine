//! Medición MAM-1056. No corre en `cargo test` salvo el control de validez.
//! La corrida de tiempos está ignorada y hay que pedirla en `--release`.
use super::{assemble_members, build_members, validation, Failure};
use crate::json::{canonical_value, parse};
use crate::{fingerprint_value, run, CancelToken, Status};
use engine_analysis::modal::Modal;
use engine_analysis::results::{CaseResults, MemberRecovery};
use engine_analysis::StaticLinear;
use engine_assembly::LinearSystem;
use engine_contracts::{CaseRef, Element, MassKind};
use engine_core::math;
use engine_core::EngineError;
use engine_linalg::{DMat, ReferenceBackend};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const LINES: usize = 5;
const STORIES: usize = 9;
const SPAN_M: f64 = 6.0;
const STORY_HEIGHT_M: f64 = 3.0;
const REPEATS: usize = 5;
const MODAL_CAP: Duration = Duration::from_secs(8);

#[derive(Clone, Copy)]
enum Kind {
    Static,
    Modal,
    PDelta,
}

#[test]
fn portal_document_passes_validation() {
    for kind in [Kind::Static, Kind::Modal, Kind::PDelta] {
        let text = portal_document(kind);
        let document: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(document["model"]["nodes"].as_array().unwrap().len(), 250);
        assert_eq!(document["model"]["members"].as_array().unwrap().len(), 585);
        let diagnostics = crate::validate(&text).expect("documento válido");
        assert!(
            diagnostics.iter().all(|item| item.severity != "error"),
            "{diagnostics:?}"
        );
    }
}

#[test]
#[ignore = "MAM-1056: cinco repeticiones release del desglose denso"]
fn perfil_ruta_densa() {
    let static_json = portal_document(Kind::Static);
    let modal_json = portal_document(Kind::Modal);
    let mut rows = Vec::new();
    let mut quadratic_share = Vec::new();
    for repeat in 0..REPEATS {
        eprintln!("MAM-1056 desglose estático repetición {repeat}");
        let _ = io::stderr().flush();
        let share = time_static(&static_json, &mut rows);
        if let Some(share) = share {
            quadratic_share.push(share);
        }
        eprintln!("MAM-1056 desglose modal repetición {repeat}");
        let _ = io::stderr().flush();
        time_modal(&modal_json, &mut rows);
    }
    if quadratic_share.is_empty() {
        eprintln!("MAM-1056 ajuste_cuadratico DESCONOCIDO");
    } else {
        eprintln!(
            "MAM-1056 mediana fraccion_trabajo_cuadratico={:.4}",
            median(&quadratic_share)
        );
    }
    print_medians(&rows);
}

fn time_static(json: &str, rows: &mut Vec<(String, Vec<f64>)>) -> Option<f64> {
    let started = Instant::now();
    let value = parse(json).unwrap();
    add(
        rows,
        "estatico/lectura_parse",
        started.elapsed().as_secs_f64(),
    );
    let started = Instant::now();
    let hash = fingerprint_value(&value).unwrap();
    add(
        rows,
        "estatico/lectura_huella",
        started.elapsed().as_secs_f64(),
    );
    assert_eq!(hash.len(), 64);
    let started = Instant::now();
    let execution = crate::input::decode(value).unwrap();
    add(
        rows,
        "estatico/lectura_decode",
        started.elapsed().as_secs_f64(),
    );
    let started = Instant::now();
    let diagnostics = validation(&execution);
    add(rows, "estatico/validacion", started.elapsed().as_secs_f64());
    assert!(diagnostics.iter().all(|item| item.severity != "error"));
    let started = Instant::now();
    let members = must(build_members(&execution));
    add(
        rows,
        "estatico/construccion_elementos",
        started.elapsed().as_secs_f64(),
    );
    let started = Instant::now();
    let model_for_system =
        super::temperature::model_for_system(&execution.model, &members, &execution.temperature)
            .unwrap();
    let contributions = must(assemble_members(&execution.model, &members));
    add(
        rows,
        "estatico/ensamblaje_elemental",
        started.elapsed().as_secs_f64(),
    );
    let started = Instant::now();
    let mut system = LinearSystem::from_model_with_diaphragms(
        ReferenceBackend,
        model_for_system.as_ref(),
        &contributions,
        &execution.prescribed,
        &execution.diaphragms,
    )
    .unwrap();
    add(
        rows,
        "estatico/sistema_y_reduccion",
        started.elapsed().as_secs_f64(),
    );
    let order = system.reduced_stiffness().order();
    let total_dof = system.dof_map().len();
    assert_eq!(total_dof, 1_500);
    assert_eq!(order, 1_350);
    let stiffness = system.reduced_stiffness().clone();
    let mut stamps = Vec::new();
    let started = Instant::now();
    let factor = system
        .factorize_reduced_with_check(&stiffness, &mut || {
            stamps.push(Instant::now());
            Ok(())
        })
        .unwrap();
    let finished = Instant::now();
    add(
        rows,
        "estatico/factorizacion",
        started.elapsed().as_secs_f64(),
    );
    let blocks = factor.uses_2x2_pivots();
    eprintln!("MAM-1056 orden_libre={order} grados_totales={total_dof} pivotes_2x2={blocks:?}");
    let share = quadratic_share(order, &stamps, finished, blocks);
    let mut substitution = 0.0;
    let mut load_vector = 0.0;
    for case in &execution.model.cases {
        let started = Instant::now();
        let rhs = system
            .reduced_rhs(&CaseRef(case.name.clone()), 1.0)
            .unwrap();
        let load = DMat::new(order, 1, rhs).unwrap();
        load_vector += started.elapsed().as_secs_f64();
        let started = Instant::now();
        let solved = factor.solve_with_check(&load, &mut || Ok(())).unwrap();
        substitution += started.elapsed().as_secs_f64();
        let sink: f64 = solved.values().iter().sum();
        assert!(sink.is_finite());
    }
    add(rows, "estatico/segundo_miembro", load_vector);
    add(rows, "estatico/sustitucion", substitution);
    let cases: Vec<_> = execution
        .model
        .cases
        .iter()
        .map(|case| (CaseRef(case.name.clone()), 1.0))
        .collect();
    let started = Instant::now();
    let solved = StaticLinear::run(&mut system, &cases).unwrap();
    add(
        rows,
        "estatico/solver_una_factorizacion",
        started.elapsed().as_secs_f64(),
    );
    let started = Instant::now();
    for (index, solution) in solved.cases.iter().enumerate() {
        let heated = super::temperature::recovery_bars(&members, index).unwrap();
        let views: Vec<MemberRecovery<'_>> = members
            .iter()
            .zip(&heated)
            .map(|(member, bar)| {
                let element: &dyn Element = match bar {
                    Some(bar) => bar,
                    None => member.element.as_ref(),
                };
                MemberRecovery {
                    id: member.id,
                    nodes: &member.nodes,
                    element,
                    dim: member.dim,
                    params: &member.params,
                    committed: &[],
                    loads: &member.loads[index],
                    stations: None,
                }
            })
            .collect();
        let recovered = CaseResults::recover(solution, &system, &views).unwrap();
        assert_eq!(recovered.members.len(), members.len());
    }
    add(
        rows,
        "estatico/recuperacion",
        started.elapsed().as_secs_f64(),
    );
    record_production(rows, "estatico/run", json, Status::Ok);
    share
}

fn time_modal(json: &str, rows: &mut Vec<(String, Vec<f64>)>) {
    let value = parse(json).unwrap();
    let execution = crate::input::decode(value).unwrap();
    let started = Instant::now();
    let members = must(build_members(&execution));
    let model_for_system =
        super::temperature::model_for_system(&execution.model, &members, &execution.temperature)
            .unwrap();
    let contributions = must(assemble_members(&execution.model, &members));
    add(
        rows,
        "modal/construccion_y_ensamblaje",
        started.elapsed().as_secs_f64(),
    );
    let started = Instant::now();
    let mut system = LinearSystem::from_model_with_diaphragms(
        ReferenceBackend,
        model_for_system.as_ref(),
        &contributions,
        &execution.prescribed,
        &execution.diaphragms,
    )
    .unwrap();
    add(
        rows,
        "modal/sistema_y_reduccion",
        started.elapsed().as_secs_f64(),
    );
    assert!(!system.has_massless_dofs(MassKind::Lumped));
    let started = Instant::now();
    let mut calls = 0_u64;
    let mut intervals = Vec::new();
    let mut previous = Instant::now();
    let result = Modal::run_with_check(
        &mut system,
        1,
        MassKind::Lumped,
        &[[1.0, 0.0, 0.0]],
        |_| Ok(()),
        || {
            let now = Instant::now();
            if calls > 0 && intervals.len() < 100_000 {
                intervals.push(now.duration_since(previous).as_secs_f64());
            }
            calls += 1;
            previous = now;
            if started.elapsed() >= MODAL_CAP || calls >= 100_000 {
                Err(EngineError::new("ENG-E-0012", "perfil: corte de medicion"))
            } else {
                Ok(())
            }
        },
    );
    let elapsed = started.elapsed().as_secs_f64();
    match result {
        Ok(modes) => {
            add(rows, "modal/espectro_completo", elapsed);
            eprintln!(
                "MAM-1056 modal completo modos={} segundos={elapsed:.6}",
                modes.modes.len()
            );
        }
        Err(error) => {
            assert!(
                error.message.contains("corte de medicion"),
                "{}",
                error.message
            );
            add(rows, "modal/prefijo_cortado", elapsed);
            if intervals.is_empty() {
                eprintln!("MAM-1056 modal intervalo DESCONOCIDO llamadas={calls}");
            } else {
                eprintln!(
                    "MAM-1056 modal llamadas={calls} mediana_intervalo_s={:.6} espectro_total=DESCONOCIDO",
                    median(&intervals)
                );
            }
        }
    }
}

fn record_production(
    rows: &mut Vec<(String, Vec<f64>)>,
    prefix: &str,
    json: &str,
    expected: Status,
) {
    let mut marks = vec![("inicio", Instant::now())];
    let started = Instant::now();
    let outcome = run(json, &CancelToken::new(), |progress| {
        marks.push((progress.stage, Instant::now()));
    });
    add(
        rows,
        &format!("{prefix}/total"),
        started.elapsed().as_secs_f64(),
    );
    assert_eq!(
        outcome.status,
        expected,
        "{}",
        &outcome.document[..outcome.document.len().min(400)]
    );
    let document = parse(&outcome.document).unwrap();
    let started = Instant::now();
    let again = canonical_value(document).unwrap();
    add(
        rows,
        &format!("{prefix}/serializacion"),
        started.elapsed().as_secs_f64(),
    );
    assert_eq!(again, outcome.document);
    let mut seen = BTreeMap::<&str, usize>::new();
    for pair in marks.windows(2) {
        let stage = pair[1].0;
        let ordinal = seen.entry(stage).or_insert(0);
        add(
            rows,
            &format!("{prefix}/{stage}#{ordinal}"),
            pair[1].1.duration_since(pair[0].1).as_secs_f64(),
        );
        *ordinal += 1;
    }
    let _document_len = outcome.document.len();
}

fn must<T>(result: Result<T, Failure>) -> T {
    match result {
        Ok(value) => value,
        Err(failure) => panic!(
            "{}",
            failure
                .1
                .first()
                .map(|item| format!("{} {}", item.code, item.cause))
                .unwrap_or_else(|| "fallo sin diagnóstico".to_owned())
        ),
    }
}

fn quadratic_share(
    order: usize,
    stamps: &[Instant],
    finished: Instant,
    blocks: Option<bool>,
) -> Option<f64> {
    if blocks != Some(false) {
        eprintln!("MAM-1056 ajuste_de_fase DESCONOCIDO pivotes_2x2={blocks:?}");
        return None;
    }
    let expected = expected_checks(order);
    if stamps.len() != expected {
        eprintln!(
            "MAM-1056 ajuste_de_fase DESCONOCIDO marcas={} esperadas={expected}",
            stamps.len()
        );
        return None;
    }
    let mut instants = stamps.to_vec();
    instants.push(finished);
    let mut quad = Vec::new();
    let mut full = Vec::new();
    let mut dt = Vec::new();
    let width = 15_usize;
    for index in 1..instants.len() - 1 {
        let start_column = width * index;
        if start_column + width > order {
            break;
        }
        let mut work = 0.0;
        for column in start_column..start_column + width {
            let remaining = (order - column) as f64;
            work += remaining * remaining;
        }
        quad.push(work);
        full.push((width * order * order) as f64);
        dt.push(
            instants[index + 1]
                .duration_since(instants[index])
                .as_secs_f64(),
        );
    }
    let Some((quad_coef, full_coef, r2)) = fit_two(&quad, &full, &dt) else {
        eprintln!("MAM-1056 ajuste_de_fase DESCONOCIDO");
        return None;
    };
    let quad_time: f64 = quad.iter().map(|value| quad_coef * value).sum();
    let full_time: f64 = full.iter().map(|value| full_coef * value).sum();
    let denom = quad_time + full_time;
    let share = if denom == 0.0 { 0.0 } else { quad_time / denom };
    eprintln!(
        "MAM-1056 ajuste r2={r2:.4} fraccion_cuadratica={share:.4} coef_cuadratico={quad_coef:.3e} coef_repaso={full_coef:.3e}"
    );
    if r2 < 0.9 {
        eprintln!("MAM-1056 ajuste_de_fase DESCONOCIDO r2_bajo");
        return None;
    }
    Some(share)
}

fn expected_checks(order: usize) -> usize {
    let mut count = 1_usize;
    let mut column = 15_usize;
    while column < order {
        count += 1;
        column += 15;
    }
    count
}

fn fit_two(quad: &[f64], full: &[f64], dt: &[f64]) -> Option<(f64, f64, f64)> {
    if quad.len() != full.len() || quad.len() != dt.len() || quad.is_empty() {
        return None;
    }
    let mut qq = 0.0;
    let mut ff = 0.0;
    let mut qf = 0.0;
    let mut qd = 0.0;
    let mut fd = 0.0;
    for index in 0..dt.len() {
        qq += quad[index] * quad[index];
        ff += full[index] * full[index];
        qf += quad[index] * full[index];
        qd += quad[index] * dt[index];
        fd += full[index] * dt[index];
    }
    let det = qq * ff - qf * qf;
    if !det.is_finite() || det.abs() < 1e-18 {
        return None;
    }
    let quad_coef = (ff * qd - qf * fd) / det;
    let full_coef = (qq * fd - qf * qd) / det;
    let mean = dt.iter().sum::<f64>() / dt.len() as f64;
    let mut total = 0.0;
    let mut residual = 0.0;
    for index in 0..dt.len() {
        let predicted = quad_coef * quad[index] + full_coef * full[index];
        residual += (dt[index] - predicted) * (dt[index] - predicted);
        let delta = dt[index] - mean;
        total += delta * delta;
    }
    let r2 = if total == 0.0 {
        1.0
    } else {
        1.0 - residual / total
    };
    Some((quad_coef, full_coef, r2))
}

fn portal_document(kind: Kind) -> String {
    let per_level = LINES * LINES;
    let young = 4_700.0e6 * math::sqrt(28.0);
    let shear = young / (2.0 * 1.20);
    let (column_a, column_iy, column_iz, column_j) = rectangle(0.70, 0.70);
    let (beam_a, beam_iy, beam_iz, beam_j) = rectangle(0.30, 0.70);
    let plan = (LINES - 1) as f64 * SPAN_M;
    let area = plan * plan;
    let mass = area * (0.15 * 2_400.0 + 3_000.0 / 9.80665);
    let share = mass / per_level as f64;
    let rotary = mass * SPAN_M * SPAN_M / (12.0 * per_level as f64);
    let dofs = ["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"];
    let mut nodes = Vec::new();
    let mut supports = Vec::new();
    let mut masses = Vec::new();
    let mut gravity = Vec::new();
    let mut lateral = Vec::new();
    for level in 0..=STORIES {
        for iy in 0..LINES {
            for ix in 0..LINES {
                let id = node_id(level, ix, iy);
                nodes.push(json!({
                    "id": id,
                    "dim": "D3",
                    "coordinates": [ix as f64 * SPAN_M, iy as f64 * SPAN_M, level as f64 * STORY_HEIGHT_M],
                    "dofs": dofs
                }));
                if level == 0 {
                    supports.push(json!({
                        "id": format!("support-{id}"),
                        "node": id,
                        "restrained": dofs
                    }));
                } else {
                    masses.push(json!({
                        "id": format!("mass-{id}"),
                        "node": id,
                        "values": dofs.iter().enumerate().map(|(index, dof)| json!({
                            "dof": dof,
                            "value": if index < 3 { share } else { rotary }
                        })).collect::<Vec<_>>()
                    }));
                    gravity.push(json!({
                        "node": id,
                        "values": [{"dof": "Uz", "value": -share * 9.80665}]
                    }));
                    lateral.push(json!({
                        "node": id,
                        "values": [{"dof": "Ux", "value": 0.10 * share * 9.80665}]
                    }));
                }
            }
        }
    }
    let mut members = Vec::new();
    let mut next = 1_u32;
    for level in 0..STORIES {
        for iy in 0..LINES {
            for ix in 0..LINES {
                members.push(member(
                    next,
                    node_id(level, ix, iy),
                    node_id(level + 1, ix, iy),
                    "column",
                    [1.0, 0.0, 0.0],
                ));
                next += 1;
            }
        }
    }
    for level in 1..=STORIES {
        for iy in 0..LINES {
            for ix in 0..LINES - 1 {
                members.push(member(
                    next,
                    node_id(level, ix, iy),
                    node_id(level, ix + 1, iy),
                    "beam",
                    [0.0, 0.0, 1.0],
                ));
                next += 1;
            }
        }
        for ix in 0..LINES {
            for iy in 0..LINES - 1 {
                members.push(member(
                    next,
                    node_id(level, ix, iy),
                    node_id(level, ix, iy + 1),
                    "beam",
                    [0.0, 0.0, 1.0],
                ));
                next += 1;
            }
        }
    }
    let cases = match kind {
        Kind::Static | Kind::PDelta => vec![
            json!({"name": "G", "loads": gravity}),
            json!({"name": "EX", "loads": lateral}),
        ],
        Kind::Modal => Vec::new(),
    };
    let analysis = match kind {
        Kind::Static => json!({"method": "linearStatic"}),
        Kind::Modal => json!({
            "method": "modal",
            "options": {"mode_count": 1, "mass_kind": "lumped", "directions": [[1.0, 0.0, 0.0]]}
        }),
        Kind::PDelta => json!({
            "method": "pDelta",
            "options": {
                "gravity": {"case": "G", "factor": 1.0},
                "lateral": {"case": "EX", "factor": 1.0},
                "tolerance": 1e-8,
                "max_iterations": 30,
                "displacement_scale": 0.01,
                "rotation_scale": 0.001
            }
        }),
    };
    let document = json!({
        "schema": "engine-input/1",
        "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": nodes,
            "members": members,
            "sections": [
                section("column", column_a, column_iy, column_iz, column_j),
                section("beam", beam_a, beam_iy, beam_iz, beam_j)
            ],
            "laws": [{
                "name": "elastic-column",
                "parameters": [
                    {"name": "E", "value": young},
                    {"name": "G", "value": shear},
                    {"name": "rho", "value": 0.0}
                ]
            }],
            "supports": supports,
            "masses": masses,
            "diaphragms": []
        },
        "cases": cases,
        "combinations": [],
        "analyses": [analysis],
        "options": {}
    });
    document.to_string()
}

fn node_id(level: usize, ix: usize, iy: usize) -> u32 {
    (level * LINES * LINES + iy * LINES + ix + 1) as u32
}

fn member(id: u32, start: u32, end: u32, section_id: &str, orientation: [f64; 3]) -> Value {
    json!({
        "id": id,
        "nodes": [start, end],
        "section": section_id,
        "law": "elastic-column",
        "element": "frame3d.euler_elastic",
        "orientation": orientation
    })
}

fn section(id: &str, area: f64, iy: f64, iz: f64, torsion: f64) -> Value {
    json!({
        "id": id,
        "properties": [
            {"name": "A", "value": area},
            {"name": "Iy", "value": iy},
            {"name": "Iz", "value": iz},
            {"name": "J", "value": torsion}
        ]
    })
}

fn rectangle(width_y: f64, depth_z: f64) -> (f64, f64, f64, f64) {
    let area = width_y * depth_z;
    let iy = width_y * math::powi(depth_z, 3) / 12.0;
    let iz = depth_z * math::powi(width_y, 3) / 12.0;
    let (long, short) = if width_y >= depth_z {
        (width_y, depth_z)
    } else {
        (depth_z, width_y)
    };
    let ratio = short / long;
    let torsion = long
        * math::powi(short, 3)
        * (1.0 / 3.0 - 0.21 * ratio * (1.0 - math::powi(ratio, 4) / 12.0));
    (area, iy, iz, torsion)
}

fn add(rows: &mut Vec<(String, Vec<f64>)>, name: &str, seconds: f64) {
    if let Some(row) = rows.iter_mut().find(|row| row.0 == name) {
        row.1.push(seconds);
    } else {
        rows.push((name.to_owned(), vec![seconds]));
    }
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[sorted.len() / 2]
}

fn print_medians(rows: &[(String, Vec<f64>)]) {
    for (name, values) in rows {
        eprintln!(
            "MAM-1056 mediana {name} repeticiones={} segundos={:.6}",
            values.len(),
            median(values)
        );
    }
}
