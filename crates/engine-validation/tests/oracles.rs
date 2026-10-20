//! ENG-VAL-005: cobertura de metadatos y matriz, complementaria a los tests numéricos.
// Aquí se inspeccionan metadatos; los parámetros mecánicos se usan en sus suites.
#[allow(dead_code)]
#[path = "support/consolidation_table.rs"]
mod consolidation;
#[allow(dead_code)]
#[path = "support/dynamic_table.rs"]
mod dynamic;
#[allow(dead_code)]
#[path = "support/table.rs"]
mod linear;
#[allow(dead_code)]
#[path = "support/winkler_table.rs"]
mod winkler;

#[path = "support/py_table.rs"]
mod py;

use engine_validation::fixtures::{Fixture, REPOSITORY_FIXTURES};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

const MATRIX: &str = include_str!("../ORACULOS.md");
const MULTILAYER_CASES: &[&str] = &["multicapa-no-transformable"];

fn metadata(
    source: &str,
    citation: &str,
    justification: &str,
    limits: &[f64],
) -> Result<(), String> {
    for text in [source, citation, justification] {
        let lower = text.to_lowercase();
        if text.trim().is_empty()
            || lower
                .split(|c: char| !c.is_alphanumeric())
                .any(|word| matches!(word, "pendiente" | "pendientes"))
            || ["opensees", "stabileo", "calculix", "lapack"]
                .iter()
                .any(|name| lower.contains(name))
        {
            return Err("Fuente, cita o justificación ausente, pendiente o prohibida".into());
        }
    }
    if limits.is_empty() || limits.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return Err("Cada oráculo requiere límites finitos y positivos".into());
    }
    Ok(())
}

fn repository_fixture(text: &str) -> Result<(String, String), String> {
    // El contrato admite defaults para consumidores; el inventario V1 exige
    // declaración explícita en cada archivo versionado, sin cambiar el contrato.
    let fixture = Fixture::parse(text)?;
    let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let limits = value
        .get("tolerance")
        .ok_or("tolerance explícita ausente")?;
    let a = limits["absolute"].as_f64().ok_or("absolute ausente")?;
    let r = limits["relative"].as_f64().ok_or("relative ausente")?;
    let source = value["source"].to_string();
    metadata(
        &source,
        value["source"]["citation"]
            .as_str()
            .ok_or("citation ausente")?,
        limits["justification"]
            .as_str()
            .ok_or("justification ausente")?,
        &[a, r],
    )?;
    Ok((fixture.id().into(), tolerance(a, r)))
}

fn tolerance(absolute: f64, relative: f64) -> String {
    format!("A={absolute:e}; R={relative:e}")
}

fn geo_tolerance(t: consolidation::Tolerance) -> String {
    format!(
        "s/s∞<{:e}; Urel<{:e}; u/100<{:e}; qrel<{:e}",
        t.settlement_normalized, t.degree_relative, t.pressure_normalized, t.flux_relative
    )
}

fn matrix_rows(matrix: &str) -> impl Iterator<Item = &str> {
    matrix.lines().filter(|line| line.starts_with("| `"))
}

fn matrix_coverage(cases: &[&str], matrix: &str) -> Result<(), String> {
    let table_names: BTreeSet<_> = cases.iter().copied().collect();
    let matrix_names: Vec<_> = matrix_rows(matrix)
        .map(|row| row.split('|').nth(1).unwrap().trim().trim_matches('`'))
        .collect();
    let unique_matrix: BTreeSet<_> = matrix_names.iter().copied().collect();
    let missing: Vec<_> = table_names.difference(&unique_matrix).collect();
    let extra: Vec<_> = unique_matrix.difference(&table_names).collect();
    if !missing.is_empty()
        || !extra.is_empty()
        || cases.len() != table_names.len()
        || matrix_names.len() != unique_matrix.len()
    {
        return Err(format!(
            "Faltan en matriz / sobran en tablas-fixtures: {missing:?}; \
             sobran en matriz / faltan en tablas-fixtures: {extra:?}; \
             duplicados en tablas-fixtures: {}; duplicados en matriz: {}",
            cases.len() - table_names.len(),
            matrix_names.len() - unique_matrix.len()
        ));
    }
    Ok(())
}

// Anclas de los encabezados Markdown locales: minúsculas, espacios a guiones
// y puntuación eliminada (p. ej. «2.º orden» -> «2º-orden»).
fn heading_anchor(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || matches!(c, '-' | '_'))
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .collect()
}

fn derivation_link(cell: &str) -> Result<(&str, &str), String> {
    let link = cell
        .split_once("](../../provenance/")
        .and_then(|(_, rest)| rest.split_once(')'))
        .map(|(link, _)| link)
        .ok_or("Falta enlace Markdown a provenance")?;
    let (file, anchor) = link.split_once('#').ok_or("Falta ancla de derivación")?;
    if !file.ends_with(".md") || file.contains('/') || anchor.is_empty() {
        return Err(format!("Enlace de derivación inválido: {link}"));
    }
    Ok((file, anchor))
}

fn verify_derivation(cell: &str) -> Result<(), String> {
    let (file, anchor) = derivation_link(cell)?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../provenance")
        .join(file);
    let note = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let exists = note.lines().any(|line| {
        let heading = line.trim_start_matches('#');
        heading.len() < line.len()
            && heading.starts_with(' ')
            && heading_anchor(heading.trim()) == anchor
    });
    if !exists {
        return Err(format!("Ancla inexistente: {file}#{anchor}"));
    }
    Ok(())
}

fn matrix_row(names: &mut BTreeSet<String>, name: &str, note: &str, tolerance: &str) {
    assert!(names.insert(name.into()), "Caso repetido: {name}");
    let prefix = format!("| `{name}` |");
    let rows: Vec<_> = matrix_rows(MATRIX)
        .filter(|line| line.starts_with(&prefix))
        .collect();
    assert_eq!(rows.len(), 1, "Falta fila única en ORACULOS.md: {name}");
    let cells: Vec<_> = rows[0].split('|').map(str::trim).collect();
    assert_eq!(cells.len(), 5, "Columnas inválidas: {name}");
    let (file, _) = derivation_link(cells[2]).unwrap_or_else(|e| panic!("{name}: {e}"));
    assert_eq!(file, note, "Nota de derivación incorrecta: {name}");
    assert_eq!(cells[3], tolerance, "Tolerancia desactualizada: {name}");
}

#[test]
fn inventario_cubre_todas_las_tablas_y_fixtures() {
    let cases: Vec<_> = linear::FIXTURES
        .iter()
        .map(|f| f.name)
        .chain(dynamic::MODAL.iter().map(|f| f.name))
        .chain(dynamic::PDELTA.iter().map(|f| f.name))
        .chain(dynamic::SPECTRUM.iter().map(|f| f.name))
        .chain(winkler::FIXTURES.iter().map(|f| f.name))
        .chain(consolidation::FIXTURES.iter().map(|f| f.name))
        .chain(MULTILAYER_CASES.iter().copied())
        .chain(py::FIXTURES.iter().map(|f| f.name))
        .chain(REPOSITORY_FIXTURES.iter().map(|(id, _)| *id))
        .collect();
    let expected_count = linear::FIXTURES.len()
        + dynamic::MODAL.len()
        + dynamic::PDELTA.len()
        + dynamic::SPECTRUM.len()
        + winkler::FIXTURES.len()
        + consolidation::FIXTURES.len()
        + MULTILAYER_CASES.len()
        + py::FIXTURES.len()
        + REPOSITORY_FIXTURES.len();
    matrix_coverage(&cases, MATRIX).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(matrix_rows(MATRIX).count(), expected_count);
    let mut names = BTreeSet::new();
    for f in linear::FIXTURES {
        let t = f.tolerance;
        metadata(
            f.source,
            f.citation,
            t.justification,
            &[t.absolute, t.relative],
        )
        .unwrap_or_else(|e| panic!("{}: {e}", f.name));
        matrix_row(
            &mut names,
            f.name,
            "ENG-VAL-001.md",
            &tolerance(t.absolute, t.relative),
        );
    }
    for f in dynamic::MODAL
        .iter()
        .chain(dynamic::PDELTA)
        .chain(dynamic::SPECTRUM)
    {
        let t = f.tolerance;
        metadata(
            f.source,
            f.citation,
            t.justification,
            &[t.absolute, t.relative],
        )
        .unwrap_or_else(|e| panic!("{}: {e}", f.name));
        matrix_row(
            &mut names,
            f.name,
            "ENG-VAL-001.md",
            &tolerance(t.absolute, t.relative),
        );
    }
    for f in winkler::FIXTURES {
        metadata(f.source, f.citation, f.justification, &[1e-8, f.bound])
            .unwrap_or_else(|e| panic!("{}: {e}", f.name));
        matrix_row(
            &mut names,
            f.name,
            "ENG-GEO-002.md",
            &tolerance(1e-8, f.bound),
        );
    }
    for f in consolidation::FIXTURES {
        let t = f.tolerance;
        metadata(
            f.source,
            f.citation,
            t.justification,
            &[
                t.settlement_normalized,
                t.degree_relative,
                t.pressure_normalized,
                t.flux_relative,
            ],
        )
        .unwrap_or_else(|e| panic!("{}: {e}", f.name));
        matrix_row(&mut names, f.name, "ENG-GEO-006.md", &geo_tolerance(t));
    }
    let t = consolidation::TOLERANCE;
    metadata(
        consolidation::MULTILAYER_SOURCE,
        consolidation::MULTILAYER_CITATION,
        t.justification,
        &[
            t.settlement_normalized,
            t.degree_relative,
            t.pressure_normalized,
            t.flux_relative,
        ],
    )
    .unwrap();
    for name in MULTILAYER_CASES {
        matrix_row(&mut names, name, "ENG-GEO-006.md", &geo_tolerance(t));
    }
    for f in py::FIXTURES {
        assert_eq!(f.source.kind, "analytical");
        assert!(f.depth_m >= 0.0 && !f.checks.is_empty());
        metadata(
            f.source.derivation,
            f.source.citation,
            py::JUSTIFICATION,
            &[py::ABSOLUTE, py::RELATIVE],
        )
        .unwrap();
        matrix_row(
            &mut names,
            f.name,
            "ENG-GEO-003.md",
            &tolerance(py::ABSOLUTE, py::RELATIVE),
        );
    }
    for (name, text) in REPOSITORY_FIXTURES {
        let (id, tolerance) = repository_fixture(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(&id, name);
        matrix_row(&mut names, &id, "ENG-CORE-009.md", &tolerance);
    }
    assert_eq!(
        names.len(),
        expected_count,
        "Casos sin verificar en la guardia"
    );
}

#[test]
fn cada_fila_enlaza_un_ancla_existente_de_su_derivacion() {
    for row in matrix_rows(MATRIX) {
        let cells: Vec<_> = row.split('|').map(str::trim).collect();
        verify_derivation(cells[2]).unwrap_or_else(|e| panic!("{}: {e}", cells[1]));
    }
}

#[test]
fn guardia_detecta_enlaces_de_derivacion_rotos() {
    for cell in [
        "[Propia](../../provenance/ENG-VAL-001.md)",
        "[Propia](../../provenance/ENG-VAL-001.md#)",
        "[Propia](../../provenance/ENG-VAL-001.md#seccion-inexistente)",
        "[Propia](../../provenance/nota-inexistente.md#voladizos)",
    ] {
        assert!(verify_derivation(cell).is_err(), "Enlace aceptado: {cell}");
    }
    assert!(verify_derivation("[Propia](../../provenance/ENG-VAL-001.md#voladizos)").is_ok());
}

#[test]
fn guardia_informa_casos_faltantes_y_sobrantes_en_ambos_lados() {
    let matrix = "| `comun` | fuente | tolerancia |\n| `solo-matriz` | fuente | tolerancia |";
    let error = matrix_coverage(&["comun", "solo-tablas"], matrix).unwrap_err();
    assert!(error.contains("Faltan en matriz / sobran en tablas-fixtures: [\"solo-tablas\"]"));
    assert!(error.contains("sobran en matriz / faltan en tablas-fixtures: [\"solo-matriz\"]"));
    assert!(matrix_coverage(&["comun", "comun", "solo-matriz"], matrix).is_err());
    assert!(matrix_coverage(&["comun"], "| `comun` |\n| `comun` |").is_err());
}

#[test]
fn guardia_rechaza_fuentes_o_limites_incompletos() {
    let source = "Derivación propia independiente: u=PL/EA";
    let citation = "provenance/ENG-VAL-001.md";
    let reason = "Margen de redondeo f64";
    assert!(metadata(source, citation, reason, &[1e-9]).is_ok());
    for (s, c, j) in [
        (" ", citation, reason),
        (source, "", reason),
        (source, citation, ""),
    ] {
        assert!(metadata(s, c, j, &[1e-9]).is_err());
    }
    for forbidden in [
        "OpenSees",
        "oPeNsEeS",
        "STABILEO",
        "CalculiX",
        "LAPACK",
        "PENDIENTE",
        "pEnDiEnTe",
    ] {
        assert!(metadata(forbidden, citation, reason, &[1e-9]).is_err());
        assert!(metadata(source, &format!("{citation}; {forbidden}"), reason, &[1e-9]).is_err());
    }
    for limits in [&[][..], &[0.0], &[-1.0], &[f64::NAN], &[f64::INFINITY]] {
        assert!(metadata(source, citation, reason, limits).is_err());
    }
}

#[test]
fn fixtures_rechazan_justificacion_vacia_o_en_blanco_y_cita_pendiente() {
    for (name, text) in REPOSITORY_FIXTURES {
        for justification in ["", " ", "\t\n "] {
            let mut value: Value = serde_json::from_str(text).unwrap();
            value["tolerance"]["justification"] = Value::String(justification.into());
            assert!(
                repository_fixture(&value.to_string()).is_err(),
                "{name}: justificación aceptada: {justification:?}"
            );
        }
        for pending in ["PENDIENTE", "pEnDiEnTe"] {
            let mut value: Value = serde_json::from_str(text).unwrap();
            value["source"]["citation"] = Value::String(format!(
                "Derivación propia; provenance/ENG-CORE-009.md; cotejo {pending}"
            ));
            assert!(
                repository_fixture(&value.to_string()).is_err(),
                "{name}: cita pendiente aceptada"
            );
        }
    }
}

#[test]
fn fixtures_versionados_rechazan_oraculo_o_tolerancia_ausente() {
    for (_, text) in REPOSITORY_FIXTURES {
        assert!(repository_fixture(text).is_ok());
        for field in ["source", "expected", "tolerance"] {
            let mut value: Value = serde_json::from_str(text).unwrap();
            value.as_object_mut().unwrap().remove(field);
            assert!(
                repository_fixture(&value.to_string()).is_err(),
                "Falta {field}"
            );
        }
        let mut value: Value = serde_json::from_str(text).unwrap();
        value["source"]["citation"] = Value::String("OpenSees como única referencia".into());
        assert!(repository_fixture(&value.to_string()).is_err());
    }
}
