//! Pórtico de 250 nudos. El ensayo de tiempos está ignorado: `cargo test` no lo corre.
use engine_api::{run, CancelToken, Progress, Status};
use engine_bench::{
    portal_250, portal_250_counts, translate, AnalysisKind, PORTAL_250_ID, PORTAL_250_LINES,
    PORTAL_250_STORIES,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Write;
use std::time::Instant;

#[test]
fn portal_250_is_deterministic_and_translatable() {
    let model = portal_250();
    assert_eq!(model, portal_250());
    assert_eq!(model.id, PORTAL_250_ID);
    assert_eq!(
        (PORTAL_250_LINES, PORTAL_250_STORIES),
        (5, 9),
        "la retícula del perfil es 5×5 líneas y 9 pisos"
    );
    assert_eq!(portal_250_counts(), (250, 585, 1_350));
    assert_eq!(model.nodes.len(), 250);
    assert_eq!(model.members.len(), 585);
    assert!(model.diaphragms.is_empty());
    assert_eq!(model.nodal_masses.as_ref().map(Vec::len), Some(225));
    let static_json = translate(&model, AnalysisKind::Static).expect("estático traducible");
    let modal_json = translate(&model, AnalysisKind::Modal).expect("modal traducible");
    let pdelta_json = translate(&model, AnalysisKind::PDelta).expect("P-Delta traducible");
    for text in [&static_json, &modal_json, &pdelta_json] {
        let document: Value = serde_json::from_str(text).unwrap();
        assert_eq!(document["model"]["nodes"].as_array().unwrap().len(), 250);
        assert_eq!(document["model"]["members"].as_array().unwrap().len(), 585);
        assert!(document["model"]["diaphragms"]
            .as_array()
            .unwrap()
            .is_empty());
    }
    assert_eq!(
        serde_json::from_str::<Value>(&static_json).unwrap()["cases"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
#[ignore = "MAM-1056: cinco corridas release del generador portal_250"]
fn perfil_run_del_generador() {
    let model = portal_250();
    let static_json = translate(&model, AnalysisKind::Static).unwrap();
    let pdelta_json = translate(&model, AnalysisKind::PDelta).unwrap();
    let mut rows = Vec::new();
    for repeat in 0..5 {
        eprintln!("MAM-1056 generador estático repetición {repeat}");
        let _ = std::io::stderr().flush();
        record_run(&mut rows, "generador/estatico", &static_json, Status::Ok);
        eprintln!("MAM-1056 generador P-Delta repetición {repeat}");
        let _ = std::io::stderr().flush();
        record_run(&mut rows, "generador/pdelta", &pdelta_json, Status::Ok);
    }
    print_medians(&rows);
}

fn record_run(rows: &mut Vec<(String, Vec<f64>)>, prefix: &str, json: &str, expected: Status) {
    let mut marks = vec![("inicio", Instant::now())];
    let started = Instant::now();
    let outcome = run(json, &CancelToken::new(), |progress: Progress| {
        marks.push((progress.stage, Instant::now()));
    });
    let wall = started.elapsed().as_secs_f64();
    assert_eq!(
        outcome.status,
        expected,
        "{}",
        &outcome.document[..outcome.document.len().min(400)]
    );
    add(rows, &format!("{prefix}/total"), wall);
    let mut seen = BTreeMap::<&str, usize>::new();
    for pair in marks.windows(2) {
        let stage = pair[1].0;
        let ordinal = seen.entry(stage).or_insert(0);
        let name = format!("{prefix}/{stage}#{ordinal}");
        *ordinal += 1;
        add(
            rows,
            &name,
            pair[1].1.duration_since(pair[0].1).as_secs_f64(),
        );
    }
}

fn add(rows: &mut Vec<(String, Vec<f64>)>, name: &str, seconds: f64) {
    if let Some(row) = rows.iter_mut().find(|row| row.0 == name) {
        row.1.push(seconds);
    } else {
        rows.push((name.to_owned(), vec![seconds]));
    }
}

fn print_medians(rows: &[(String, Vec<f64>)]) {
    for (name, values) in rows {
        let mut sorted = values.clone();
        sorted.sort_by(f64::total_cmp);
        let median = sorted[sorted.len() / 2];
        eprintln!(
            "MAM-1056 mediana {name} repeticiones={} segundos={median:.6}",
            values.len()
        );
    }
}
