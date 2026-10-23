//! Contrato `fixture/1`, procedencia obligatoria y comparación portable.
//!
//! Cada comprobación usa `|actual-esperado| <= max(absolute, relative*|esperado|)`.
//! Los valores predeterminados son (1e-10, 1e-10) para linearStatic y
//! (1e-7, 1e-7) para otros métodos. No hay reloj ni métricas de tiempo.
use engine_api::{canonical_json, CancelToken};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

/// JSON Schema documental del contrato; `Fixture::parse` aplica sus invariantes.
pub const SCHEMA: &str = include_str!("../schemas/fixture-1.json");

/// Cinco casos propios; los bytes son los archivos versionados del repositorio.
pub const REPOSITORY_FIXTURES: &[(&str, &str)] = &[
    ("axial-bar", include_str!("../fixtures/axial-bar.json")),
    (
        "axial-series",
        include_str!("../fixtures/axial-series.json"),
    ),
    (
        "cantilever-point",
        include_str!("../fixtures/cantilever-point.json"),
    ),
    (
        "simply-supported-uniform",
        include_str!("../fixtures/simply-supported-uniform.json"),
    ),
    (
        "spatial-cantilever",
        include_str!("../fixtures/spatial-cantilever.json"),
    ),
];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Source {
    Analytical {
        citation: String,
        derivation: String,
    },
    Book {
        citation: String,
        author: String,
        title: String,
        edition: String,
        page: String,
    },
    Published {
        citation: String,
        author: String,
        title: String,
        publication: String,
        locator: String,
    },
}

impl Source {
    fn fields(&self) -> Vec<&str> {
        match self {
            Self::Analytical {
                citation,
                derivation,
            } => vec![citation, derivation],
            Self::Book {
                citation,
                author,
                title,
                edition,
                page,
            } => vec![citation, author, title, edition, page],
            Self::Published {
                citation,
                author,
                title,
                publication,
                locator,
            } => vec![citation, author, title, publication, locator],
        }
    }
}

/// Tolerancia efectiva, con piso absoluto en las unidades de la magnitud.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Tolerance {
    /// Error relativo máximo respecto al valor esperado.
    pub relative: f64,
    /// Piso absoluto del error admisible, también para ceros.
    pub absolute: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Override {
    relative: f64,
    absolute: f64,
    justification: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    pointer: String,
    value: f64,
}

/// Caso validado e inmutable; entrada engine-input/1 embebida, sin resolver rutas.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    schema: String,
    id: String,
    source: Source,
    input: Value,
    expected: Vec<Expected>,
    #[serde(default)]
    tolerance: Option<Override>,
    #[serde(default)]
    provenance: Option<Value>,
}

// Inspección recursiva, incluidas claves, listas y capitalización arbitraria.
fn forbidden_provenance(value: &Value) -> bool {
    fn forbidden(text: &str) -> bool {
        let text = text.to_lowercase();
        ["opensees", "stabileo", "calculix"]
            .iter()
            .any(|name| text.contains(name))
    }
    match value {
        Value::String(text) => forbidden(text),
        Value::Array(items) => items.iter().any(forbidden_provenance),
        Value::Object(fields) => fields
            .iter()
            .any(|(key, value)| forbidden(key) || forbidden_provenance(value)),
        _ => false,
    }
}

impl Fixture {
    /// Rechaza fuentes ausentes/vacías y procedencias de motores no permitidos.
    /// También rechaza campos desconocidos, claves duplicadas y números no finitos.
    pub fn parse(json: &str) -> Result<Self, String> {
        let canonical = canonical_json(json).map_err(|e| e.to_string())?;
        let value: Value = serde_json::from_str(&canonical).map_err(|e| e.to_string())?;
        for field in ["source", "provenance"] {
            if value.get(field).is_some_and(forbidden_provenance) {
                return Err(format!("{field}: prohibited engine provenance"));
            }
        }
        let source: Source =
            serde_json::from_value(value.get("source").cloned().ok_or("source is required")?)
                .map_err(|e| format!("source: {e}"))?;
        if source.fields().iter().any(|s| s.trim().is_empty()) {
            return Err("All source fields must be nonempty".into());
        }
        if value.get("tolerance").is_some_and(Value::is_null) {
            return Err("tolerance must be an object when declared".into());
        }
        let fixture: Self = serde_json::from_value(value).map_err(|e| e.to_string())?;
        if fixture.schema != "fixture/1" {
            return Err("schema fixture/1 is required".into());
        }
        if fixture.id.trim().is_empty() {
            return Err("id must be nonempty".into());
        }
        if fixture.input.get("schema").and_then(Value::as_str) != Some("engine-input/1") {
            return Err("input must be an embedded engine-input/1 input".into());
        }
        if fixture.expected.is_empty() {
            return Err("expected must contain at least one check".into());
        }
        let mut pointers = BTreeSet::new();
        for expected in &fixture.expected {
            let pointer = &expected.pointer;
            let valid_escapes = pointer
                .split('~')
                .skip(1)
                .all(|part| part.starts_with(['0', '1']));
            if !pointer.starts_with('/')
                || !valid_escapes
                || !pointers.insert(pointer)
                || !expected.value.is_finite()
            {
                return Err(
                    "expected requires unique valid JSON Pointers and finite values".into(),
                );
            }
        }
        if let Some(tolerance) = &fixture.tolerance {
            if tolerance.justification.trim().is_empty()
                || !tolerance.relative.is_finite()
                || tolerance.relative < 0.0
                || !tolerance.absolute.is_finite()
                || tolerance.absolute <= 0.0
            {
                return Err(
                    "tolerance requires relative >= 0, absolute > 0, and nonempty justification"
                        .into(),
                );
            }
        }
        Ok(fixture)
    }

    /// Identificador estable del caso.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Tolerancia declarada o predeterminada según el método de la entrada.
    pub fn tolerance(&self) -> Tolerance {
        if let Some(tolerance) = &self.tolerance {
            return Tolerance {
                relative: tolerance.relative,
                absolute: tolerance.absolute,
            };
        }
        let linear = self
            .input
            .get("analyses")
            .and_then(Value::as_array)
            .is_some_and(|analyses| {
                !analyses.is_empty()
                    && analyses
                        .iter()
                        .all(|a| a.get("method").and_then(Value::as_str) == Some("linearStatic"))
            });
        let default = if linear { 1e-10 } else { 1e-7 };
        Tolerance {
            relative: default,
            absolute: default,
        }
    }
}

/// Adaptador enchufable: recibe engine-input/1 y devuelve results/1 o un error.
/// No interpreta valores esperados ni decide tolerancias.
pub trait Target {
    /// Nombre estable del target para los informes.
    fn name(&self) -> &str;
    /// Ejecuta una entrada sin modificarla.
    fn execute(&self, input: &str) -> Result<String, String>;
}

/// Target nativo en proceso, vía engine-api y sin procesos externos.
pub struct NativeTarget;
impl Target for NativeTarget {
    fn name(&self) -> &str {
        "native-api"
    }
    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(engine_api::run(input, &CancelToken::new(), |_| {}).document)
    }
}

/// Resultado de una magnitud, incluso cuando su ruta falta o no es numérica.
#[derive(Debug, Serialize)]
pub struct Comparison {
    /// JSON Pointer dentro de results/1.
    pub pointer: String,
    /// Oráculo analítico/publicado declarado por el fixture.
    pub expected: f64,
    /// Valor calculado; ausente cuando falta la magnitud o no es finita.
    pub actual: Option<f64>,
    /// Error absoluto; ausente si no es representable como f64 finito.
    pub absolute_error: Option<f64>,
    /// Límite efectivo de error.
    pub allowed_error: f64,
    /// Comparación satisfactoria.
    pub passed: bool,
}

/// Informe de un fixture en un target.
#[derive(Debug, Serialize)]
pub struct Row {
    /// Identificador del fixture (nombre de documento si no es válido).
    pub fixture: String,
    /// Target ejecutado.
    pub target: String,
    /// Fuente y procedencia declaradas, conservadas para auditoría.
    pub source: Option<Value>,
    /// Metadatos opcionales de procedencia.
    pub provenance: Option<Value>,
    /// Tolerancia efectiva; ausente si el documento no es válido.
    pub tolerance: Option<Tolerance>,
    /// Justificación de la tolerancia particular, si existe.
    pub tolerance_justification: Option<String>,
    /// Todas las magnitudes pasaron y el target terminó con status ok.
    pub passed: bool,
    /// Error de formato/ejecución; las discrepancias están en comparisons.
    pub error: Option<String>,
    /// Comprobaciones en el orden declarado por el fixture.
    pub comparisons: Vec<Comparison>,
}

/// Informe determinista; sin tiempos, rutas del anfitrión ni marcas de reloj.
#[derive(Debug, Serialize)]
pub struct Report {
    /// Versión de este informe.
    pub schema: &'static str,
    /// Filas ordenadas por identificador de fixture y nombre de target.
    pub rows: Vec<Row>,
}

impl Report {
    /// Exige al menos una ejecución y todas las filas satisfactorias.
    pub fn passed(&self) -> bool {
        !self.rows.is_empty() && self.rows.iter().all(|row| row.passed)
    }

    /// JSON compacto con claves ordenadas y normalización de cero firmado.
    pub fn json(&self) -> Result<String, String> {
        let text = serde_json::to_string(self).map_err(|e| e.to_string())?;
        canonical_json(&text).map_err(|e| e.to_string())
    }

    /// Texto español por fixture y target, con detalle de cada comparación.
    pub fn text(&self) -> String {
        let mut text = String::new();
        for row in &self.rows {
            text.push_str(&format!(
                "{} / {}: {}\n",
                row.fixture,
                row.target,
                if row.passed { "OK" } else { "FAILED" }
            ));
            if let Some(error) = &row.error {
                text.push_str(&format!("  {error}\n"));
            }
            for c in &row.comparisons {
                text.push_str(&format!(
                    "  {}: expected={}, actual={}, error={}, limit={} [{}]\n",
                    c.pointer,
                    c.expected,
                    c.actual.map_or_else(|| "missing".into(), |v| v.to_string()),
                    c.absolute_error
                        .map_or_else(|| "unavailable".into(), |v| v.to_string()),
                    c.allowed_error,
                    if c.passed { "OK" } else { "FAILED" }
                ));
            }
        }
        text
    }
}

fn compare(fixture: &Fixture, target: &dyn Target) -> Result<Vec<Comparison>, String> {
    let input = canonical_json(&fixture.input.to_string()).map_err(|e| e.to_string())?;
    let output = target.execute(&input)?;
    let output = canonical_json(&output).map_err(|e| format!("Invalid results/1: {e}"))?;
    let value: Value = serde_json::from_str(&output).map_err(|e| e.to_string())?;
    if value["schema"] != "results/1" || value["status"] != "ok" {
        return Err(format!("results/1 with status ok is required: {output}"));
    }
    let tolerance = fixture.tolerance();
    fixture
        .expected
        .iter()
        .map(|expected| {
            let allowed = tolerance
                .absolute
                .max(tolerance.relative * expected.value.abs());
            if !allowed.is_finite() {
                return Err("Effective tolerance overflows f64".into());
            }
            let actual = value
                .pointer(&expected.pointer)
                .and_then(Value::as_f64)
                .filter(|v| v.is_finite());
            let difference = actual.map(|actual| (actual - expected.value).abs());
            Ok(Comparison {
                pointer: expected.pointer.clone(),
                expected: expected.value,
                actual,
                absolute_error: difference.filter(|v| v.is_finite()),
                allowed_error: allowed,
                passed: difference.is_some_and(|error| error <= allowed),
            })
        })
        .collect()
}

/// Ejecuta todos los documentos contra todos los targets; no interrumpe la suite
/// ante fallos. Valida la procedencia antes de invocar cualquier target.
/// IDs de fixture y nombres de target deben ser únicos y no vacíos.
pub fn run(documents: &[(&str, &str)], targets: &[&dyn Target]) -> Report {
    let mut rows = Vec::new();
    let mut ids = BTreeSet::new();
    let mut target_names = BTreeSet::new();
    let valid_targets = targets
        .iter()
        .all(|t| !t.name().trim().is_empty() && target_names.insert(t.name()));
    for &(name, document) in documents {
        let fixture = Fixture::parse(document).and_then(|fixture| {
            if !ids.insert(fixture.id.clone()) {
                return Err("Duplicate fixture id".into());
            }
            if !valid_targets {
                return Err("Empty or duplicate target names".into());
            }
            Ok(fixture)
        });
        for target in targets {
            let mut row = Row {
                fixture: name.into(),
                target: target.name().into(),
                source: None,
                provenance: None,
                tolerance: None,
                tolerance_justification: None,
                passed: false,
                error: None,
                comparisons: Vec::new(),
            };
            match &fixture {
                Err(error) => row.error = Some(error.clone()),
                Ok(fixture) => {
                    row.fixture = fixture.id.clone();
                    row.source = Some(serde_json::json!(&fixture.source));
                    row.provenance = fixture.provenance.clone();
                    row.tolerance = Some(fixture.tolerance());
                    row.tolerance_justification =
                        fixture.tolerance.as_ref().map(|t| t.justification.clone());
                    match compare(fixture, *target) {
                        Ok(comparisons) => {
                            row.passed = comparisons.iter().all(|c| c.passed);
                            row.comparisons = comparisons;
                        }
                        Err(error) => row.error = Some(error),
                    }
                }
            }
            rows.push(row);
        }
    }
    rows.sort_by(|a, b| (&a.fixture, &a.target).cmp(&(&b.fixture, &b.target)));
    Report {
        schema: "fixture-report/1",
        rows,
    }
}
