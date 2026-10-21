//! VAL-032: entradas reales, bloqueos del Engine explícitos y avisos revisados.
use engine_api::{CancelToken, Status};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

// #184: condensación modal pendiente. Los ocho bloqueos conservan estado y
// error exactos como centinela hasta revisar evidencia y documentación.
// #185 habilita E3-pdelta sin cambiar los fixtures ni sus tolerancias.
const BLOCKED: &[(&str, Status, &str, &str)] = &[
    ("E1-modal", Status::Unstable, "unstable", "ENG-E-0006"),
    ("E2-cqc-X", Status::Unstable, "unstable", "ENG-E-0006"),
    ("E2-cqc-Y", Status::Unstable, "unstable", "ENG-E-0006"),
    ("E2-srss-X", Status::Unstable, "unstable", "ENG-E-0006"),
    ("E2b-cqc-X", Status::Unstable, "unstable", "ENG-E-0006"),
    ("E2b-cqc-Y", Status::Unstable, "unstable", "ENG-E-0006"),
    (
        "E2b-cqc-XY-srss",
        Status::Unstable,
        "unstable",
        "ENG-E-0006",
    ),
    ("E4-modal", Status::Unstable, "unstable", "ENG-E-0006"),
];
const EXECUTABLE: &[&str] = &[
    "E3-lineal",
    "E3-pdelta",
    "E5-combinaciones",
    "E5b-combinaciones",
];
const COMBINATIONS: &[&str] = &[
    "C1_1.4D",
    "C2_1.2D+1.6L",
    "C5_1.2D+L+EX",
    "C7_0.9D+EX",
    "C5_1.2D+L-EX",
    "C7_0.9D-EX",
    "C5_1.2D+L+EY",
    "C7_0.9D+EY",
    "C5_1.2D+L-EY",
    "C7_0.9D-EY",
];
// Pares run/código, incluidos avisos previos al fallo modal. No se permiten
// avisos nuevos sin revisión de su causa en el documento results/1.
const WARNINGS: &[(&str, &str)] = &[
    ("E1-modal", "API-W-DIAPHRAGM-RECOVERY"),
    ("E2-cqc-X", "API-W-DIAPHRAGM-RECOVERY"),
    ("E2-cqc-Y", "API-W-DIAPHRAGM-RECOVERY"),
    ("E2-srss-X", "API-W-DIAPHRAGM-RECOVERY"),
    ("E2b-cqc-X", "API-W-DIAPHRAGM-RECOVERY"),
    ("E2b-cqc-Y", "API-W-DIAPHRAGM-RECOVERY"),
    ("E2b-cqc-XY-srss", "API-W-DIAPHRAGM-RECOVERY"),
    ("E4-modal", "API-W-DIAPHRAGM-RECOVERY"),
    ("E3-lineal", "API-W-DIAPHRAGM-RECOVERY"),
    ("E3-pdelta", "API-W-DIAPHRAGM-RECOVERY"),
    ("E5-combinaciones", "API-W-DIAPHRAGM-RECOVERY"),
    ("E5-combinaciones", "API-W-COMBINED-PRESCRIBED"),
    ("E5b-combinaciones", "API-W-COMBINED-PRESCRIBED"),
];

#[test]
fn val032_runs() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/validation-report/fixtures/val032");
    let mut paths: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension().and_then(|s| s.to_str()) == Some("json")
                && path.file_name().unwrap().to_string_lossy().starts_with('E')
        })
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 7, "se requieren los siete fixtures E*.json");
    let expected: BTreeSet<_> = BLOCKED
        .iter()
        .map(|(id, _, _, _)| *id)
        .chain(EXECUTABLE.iter().copied())
        .collect();
    let mut seen = BTreeSet::new();
    let mut observed_warnings = BTreeSet::new();
    for path in paths {
        let fixture: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        for run in fixture["runs"].as_array().unwrap() {
            let id = run["id"].as_str().unwrap();
            assert!(expected.contains(id), "run sin revisión: {id}");
            assert!(seen.insert(id.to_owned()), "run duplicado: {id}");
            let input = serde_json::to_string(&run["engine_input"]).unwrap();
            let outcome = engine_api::run(&input, &CancelToken::new(), |_| {});
            let result: Value = serde_json::from_str(&outcome.document).unwrap();
            assert_eq!(result["schema"], "results/1", "{id}");
            // Exportación opcional para revisar los documentos reales, sin
            // depender de archivos externos para determinar las expectativas.
            if let Some(dir) = std::env::var_os("VAL032_RESULTS_DIR") {
                fs::create_dir_all(&dir).unwrap();
                fs::write(
                    Path::new(&dir).join(format!("{id}.json")),
                    &outcome.document,
                )
                .unwrap();
                fs::write(Path::new(&dir).join(format!("{id}.input.json")), &input).unwrap();
            }
            assert_ne!(
                outcome.status,
                Status::Invalid,
                "{id}: {}",
                outcome.document
            );
            let diagnostics = result["diagnostics"].as_array().unwrap();
            let errors: Vec<_> = diagnostics
                .iter()
                .filter(|d| d["severity"] == "error")
                .map(|d| d["code"].as_str().unwrap())
                .collect();
            if let Some((_, status, serialized_status, code)) =
                BLOCKED.iter().find(|(blocked, _, _, _)| *blocked == id)
            {
                assert_eq!(outcome.status, *status, "{id}: revisar bloqueo #184");
                assert_eq!(result["status"], *serialized_status, "{id}");
                assert_eq!(errors, [*code], "{id}: {}", outcome.document);
            } else {
                assert!(
                    matches!(outcome.status, Status::Ok | Status::Warning),
                    "{id}: {}",
                    outcome.document
                );
                assert!(errors.is_empty(), "{id}: {errors:?}");
                assert_eq!(
                    result["status"],
                    if outcome.status == Status::Ok {
                        "ok"
                    } else {
                        "warning"
                    },
                    "{id}"
                );
            }
            let recovery_warnings = diagnostics
                .iter()
                .filter(|d| d["severity"] == "warning" && d["code"] == "API-W-DIAPHRAGM-RECOVERY")
                .count();
            assert_eq!(
                recovery_warnings,
                usize::from(WARNINGS.contains(&(id, "API-W-DIAPHRAGM-RECOVERY"))),
                "{id}: cantidad de avisos #84"
            );
            let mut warned_combinations = BTreeSet::new();
            for diagnostic in diagnostics.iter().filter(|d| d["severity"] == "warning") {
                let code = diagnostic["code"].as_str().unwrap();
                assert!(
                    WARNINGS.contains(&(id, code)),
                    "{id}: aviso no revisado: {diagnostic}"
                );
                observed_warnings.insert((id.to_owned(), code.to_owned()));
                if code == "API-W-COMBINED-PRESCRIBED" {
                    let combination = diagnostic["entities"][0]["id"].as_str().unwrap();
                    assert!(COMBINATIONS.contains(&combination), "{id}: {diagnostic}");
                    assert!(warned_combinations.insert(combination));
                }
            }
            if matches!(id, "E5-combinaciones" | "E5b-combinaciones") {
                assert_eq!(warned_combinations, COMBINATIONS.iter().copied().collect());
                for node in result["combinations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|c| c["results"]["nodes"].as_array().unwrap())
                {
                    // #85 se emite incluso para los prescritos nulos de base.
                    assert!(node["prescribed_displacements"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|value| value.is_null() || value.as_f64() == Some(0.0)));
                }
            }
            eprintln!("{id}: {:?}; {}", outcome.status, result["diagnostics"]);
        }
    }
    assert_eq!(
        seen.iter().map(String::as_str).collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(
        observed_warnings,
        WARNINGS
            .iter()
            .map(|(r, c)| (r.to_string(), c.to_string()))
            .collect()
    );
}
