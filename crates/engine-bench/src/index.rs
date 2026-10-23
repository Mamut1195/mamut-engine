//! Índice `bench-index/1`. Las rutas usan `/` y son relativas a la raíz del repositorio.
//!
//! `sha256` es el hash de los bytes UTF-8 del archivo, con el salto LF final y sin BOM.
//! No es el `input_sha256` JCS que `results/1` calcula sobre el documento de entrada.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::error::BenchError;
use crate::generate::official_models;
use crate::model::{AnalysisKind, BenchModel};
use crate::translate::{render, translate, translate_variant};
use engine_api::{MAX_MEMBERS, MAX_NODES};

/// Ruta del índice dentro del repositorio.
pub const INDEX_PATH: &str = "crates/engine-bench/fixtures/index.json";
/// Prefijo de los modelos y de las entradas traducidas.
pub const FIXTURE_ROOT: &str = "crates/engine-bench/fixtures";
/// Esquema publicado de `bench-model/1`.
pub const BENCH_MODEL_SCHEMA_PATH: &str = "crates/engine-bench/schemas/bench-model-1.json";

/// Escribe los tres modelos, las nueve entradas traducidas y el índice.
pub fn write_fixtures(repo_root: &Path) -> Result<(), BenchError> {
    let index = repository_index()?;
    for model in official_models() {
        let entry = index
            .models
            .iter()
            .find(|item| item.id == model.id)
            .ok_or_else(|| BenchError::new("Index is missing the official model."))?;
        write_bytes(repo_root, &entry.path, &render(&model)?)?;
        for kind in [
            AnalysisKind::Static,
            AnalysisKind::Modal,
            AnalysisKind::Spectrum,
        ] {
            let file = engine_file(entry, kind)?;
            write_bytes(repo_root, &file.path, &translate(&model, kind)?)?;
        }
    }
    write_bytes(repo_root, INDEX_PATH, &render(&index)?)?;
    Ok(())
}

/// Índice en memoria, con los mismos bytes que se escriben en disco.
pub fn repository_index() -> Result<BenchIndex, BenchError> {
    let mut models = Vec::new();
    for model in official_models() {
        models.push(index_model(&model)?);
    }
    Ok(BenchIndex {
        schema: "bench-index/1".to_owned(),
        encoding: "utf-8".to_owned(),
        newline: "lf".to_owned(),
        bench_model_schema: BENCH_MODEL_SCHEMA_PATH.to_owned(),
        hash: HashContract {
            name: "sha256".to_owned(),
            encoding: "lowercase hexadecimal".to_owned(),
            subject: "bytes UTF-8 exactos del archivo, con el salto LF final y sin BOM".to_owned(),
            not_equal_to: "results/1.input_sha256, que es SHA-256 sobre JSON RFC 8785".to_owned(),
        },
        limits: Limits {
            contract: "engine-input/1".to_owned(),
            max_nodes: MAX_NODES,
            max_members: MAX_MEMBERS,
        },
        models,
    })
}

fn index_model(model: &BenchModel) -> Result<IndexModel, BenchError> {
    let path = format!("{FIXTURE_ROOT}/models/{}.json", model.id);
    let bench_bytes = render(model)?;
    let roof = roof_node(model)?;
    if roof == 0 {
        return Err(BenchError::new("Roof node cannot be 0."));
    }
    let node_index = (roof - 1) as usize;
    Ok(IndexModel {
        id: model.id.clone(),
        path,
        sha256: file_sha256(bench_bytes.as_bytes()),
        nodes: model.nodes.len(),
        members: model.members.len(),
        engine_input: EngineInputSet {
            linear_static: hashed_engine(model, AnalysisKind::Static)?,
            modal: hashed_engine(model, AnalysisKind::Modal)?,
            spectrum: hashed_engine(model, AnalysisKind::Spectrum)?,
        },
        roof_node: roof,
        pointers: pointers(roof, node_index),
    })
}

fn hashed_engine(model: &BenchModel, kind: AnalysisKind) -> Result<HashedFile, BenchError> {
    let text = translate(model, kind)?;
    Ok(HashedFile {
        path: format!(
            "{FIXTURE_ROOT}/engine-input/{}-{}.json",
            model.id,
            kind.file_key()
        ),
        sha256: file_sha256(text.as_bytes()),
    })
}

/// Nudo maestro del diafragma más alto.
pub fn roof_node(model: &BenchModel) -> Result<u32, BenchError> {
    model
        .diaphragms
        .iter()
        .max_by_key(|diaphragm| diaphragm.level)
        .map(|diaphragm| diaphragm.master)
        .ok_or_else(|| BenchError::new("Model has no roof diaphragm."))
}

fn pointers(roof: u32, node_index: usize) -> Pointers {
    Pointers {
        period: Pointer {
            result: "modal".to_owned(),
            json_pointer: "/modes/0/period".to_owned(),
            unit: "s".to_owned(),
            component: None,
            case: None,
            description: "Periodo en segundos del modo 1, el de menor autovalor, en results/1 del archivo modal.".to_owned(),
        },
        static_roof_displacement: Pointer {
            result: "static".to_owned(),
            json_pointer: format!("/cases/1/nodes/{node_index}/displacements/0"),
            unit: "m".to_owned(),
            component: Some("Ux".to_owned()),
            case: Some("lateral".to_owned()),
            description: format!(
                "Desplazamiento Ux en metros del nudo de techo {roof} en el caso lateral. \
                 results/1 ordena los nudos por id ascendente; el índice de este nudo es {node_index}. \
                 cases/0 es gravity y cases/1 es lateral. displacements/0 es Ux, displacements/1 es Uy y displacements/2 es Uz."
            ),
        },
        cqc_roof_displacement: Pointer {
            result: "spectrum".to_owned(),
            json_pointer: format!("/envelopes/1/nodes/{node_index}/displacements/0"),
            unit: "m".to_owned(),
            component: Some("Ux".to_owned()),
            case: None,
            description: format!(
                "Componente Ux en metros del nudo de techo {roof} en la envolvente final de responseSpectrum. \
                 envelopes/0 es la dirección global X combinada con CQC. envelopes/1 es la combinación \
                 direccional final, con direction null. Hay una sola dirección y el factor es 1. \
                 La magnitud CQC de cada componente es no negativa."
            ),
        },
    }
}

fn engine_file(entry: &IndexModel, kind: AnalysisKind) -> Result<&HashedFile, BenchError> {
    Ok(match kind {
        AnalysisKind::Static => &entry.engine_input.linear_static,
        AnalysisKind::Modal => &entry.engine_input.modal,
        AnalysisKind::Spectrum => &entry.engine_input.spectrum,
        AnalysisKind::PDelta => {
            return Err(BenchError::new(
                "The rc3d index does not version pDelta. Translate that analysis separately.",
            ));
        }
    })
}

fn write_bytes(repo_root: &Path, relative: &str, text: &str) -> Result<(), BenchError> {
    if relative.contains('\\') {
        return Err(BenchError::new("Index paths use forward slashes `/`."));
    }
    let path = repo_root.join(relative);
    let parent = path
        .parent()
        .ok_or_else(|| BenchError::new("Fixture path has no directory."))?;
    fs::create_dir_all(parent).map_err(|error| BenchError::new(error.to_string()))?;
    fs::write(&path, text).map_err(|error| BenchError::new(error.to_string()))?;
    Ok(())
}

/// SHA-256 hexadecimal en minúsculas de los bytes tal como están en disco.
#[must_use]
pub fn file_sha256(bytes: &[u8]) -> String {
    mamut_canon::sha256_hex(bytes)
}

/// Contrato del índice que consume el kit externo.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BenchIndex {
    /// `bench-index/1`.
    pub schema: String,
    /// Codificación de los archivos hasheados.
    pub encoding: String,
    /// Salto de línea de esos archivos.
    pub newline: String,
    /// Ruta del JSON Schema de bench-model/1.
    pub bench_model_schema: String,
    /// Cómo se calcula cada sha256.
    pub hash: HashContract,
    /// Límites de la entrada traducida.
    pub limits: Limits,
    /// Un elemento por modelo, en orden de tamaño.
    pub models: Vec<IndexModel>,
}

/// Función de hash de los archivos versionados.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HashContract {
    /// `sha256`.
    pub name: String,
    /// Hexadecimal en minúsculas.
    pub encoding: String,
    /// Bytes que entran en el hash.
    pub subject: String,
    /// Hash que no debe usarse en su lugar.
    pub not_equal_to: String,
}

/// Límites documentados de `engine-input/1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Contrato que impone el límite.
    pub contract: String,
    /// `engine_api::MAX_NODES`.
    pub max_nodes: usize,
    /// `engine_api::MAX_MEMBERS`.
    pub max_members: usize,
}

/// Una fila del índice.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IndexModel {
    /// Identificador del modelo.
    pub id: String,
    /// Ruta del `bench-model/1`, relativa a la raíz y con `/`.
    pub path: String,
    /// SHA-256 del archivo `path`.
    pub sha256: String,
    /// Nudos, incluida la base.
    pub nodes: usize,
    /// Barras.
    pub members: usize,
    /// Entradas `engine-input/1` ya traducidas y versionadas.
    pub engine_input: EngineInputSet,
    /// Nudo maestro del diafragma de techo.
    pub roof_node: u32,
    /// Dónde leer periodo y desplazamientos de techo en `results/1`.
    pub pointers: Pointers,
}

/// Tres documentos porque `engine-input/1` admite un solo análisis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EngineInputSet {
    /// Caso estático.
    #[serde(rename = "static")]
    pub linear_static: HashedFile,
    /// Caso modal.
    pub modal: HashedFile,
    /// Caso espectral CQC.
    pub spectrum: HashedFile,
}

/// Ruta y hash de un archivo versionado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HashedFile {
    /// Ruta relativa a la raíz del repositorio, con `/`.
    pub path: String,
    /// SHA-256 de los bytes de `path`.
    pub sha256: String,
}

/// Punteros RFC 6901 sobre la salida `results/1` de cada análisis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pointers {
    /// Periodo del modo 1.
    pub period: Pointer,
    /// Desplazamiento de techo del caso lateral.
    pub static_roof_displacement: Pointer,
    /// Desplazamiento de techo de la envolvente CQC.
    pub cqc_roof_displacement: Pointer,
}

/// Un lugar concreto de la salida.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pointer {
    /// `static`, `modal` o `spectrum`: qué archivo de salida hay que abrir.
    pub result: String,
    /// Puntero JSON RFC 6901.
    pub json_pointer: String,
    /// Unidad SI del valor.
    pub unit: String,
    /// Componente, si el valor es una entrada de `displacements`.
    pub component: Option<String>,
    /// Nombre del caso estático, si aplica.
    pub case: Option<String>,
    /// Lectura prevista, para no inferir el índice.
    pub description: String,
}

/// Interfaz del binario `bench-prepare`.
pub fn cli(args: &[String]) -> Result<(), BenchError> {
    match args.first().map(String::as_str) {
        Some("generate") => {
            let root = flag(args, "--root")?.unwrap_or(".");
            write_fixtures(Path::new(root))
        }
        Some("translate") => translate_cli(args),
        _ => Err(BenchError::new(
            "Usage: bench-prepare generate --root DIR | translate --model FILE --analysis static|modal|spectrum|pdelta [--variant ID] --output FILE",
        )),
    }
}

fn translate_cli(args: &[String]) -> Result<(), BenchError> {
    let model_path = flag(args, "--model")?.ok_or_else(|| BenchError::new("Missing --model."))?;
    let analysis =
        flag(args, "--analysis")?.ok_or_else(|| BenchError::new("Missing --analysis."))?;
    let output = flag(args, "--output")?.ok_or_else(|| BenchError::new("Missing --output."))?;
    let kind = AnalysisKind::parse(analysis)
        .ok_or_else(|| BenchError::new("Analysis must be static, modal, spectrum, or pdelta."))?;
    let variant = flag(args, "--variant")?;
    let text =
        fs::read_to_string(model_path).map_err(|error| BenchError::new(error.to_string()))?;
    let model: BenchModel =
        serde_json::from_str(&text).map_err(|error| BenchError::new(error.to_string()))?;
    let translated = translate_variant(&model, kind, variant)?;
    if let Some(parent) = Path::new(output).parent() {
        fs::create_dir_all(parent).map_err(|error| BenchError::new(error.to_string()))?;
    }
    fs::write(output, translated).map_err(|error| BenchError::new(error.to_string()))
}

fn flag<'a>(args: &'a [String], name: &str) -> Result<Option<&'a str>, BenchError> {
    let mut values = args.windows(2).filter(|pair| pair[0] == name);
    let found = values.next().map(|pair| pair[1].as_str());
    if values.next().is_some() {
        return Err(BenchError::new(format!("{name} is duplicated.")));
    }
    Ok(found)
}
