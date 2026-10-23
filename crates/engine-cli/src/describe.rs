// SPDX-License-Identifier: AGPL-3.0-only

//! Autodescripción y consultas de la fachada, sin E/S de archivos ni ejecución.
use std::io::Write;

use mamut_workflow_cli::{ExitCodes, FailureKind, MAX_DOCUMENT_BYTES};
use serde_json::{json, Value};

const INDEX: &str = include_str!("../../../schemas/index.json");

pub(crate) fn dispatch(
    args: &[String],
    stdout: &mut impl Write,
) -> Option<Result<(), FailureKind>> {
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["describe", "--json"] | ["--help" | "help" | "-h", "--json"] => {
            description().and_then(|value| write_json(stdout, &value))
        }
        ["--version"] => writeln!(
            stdout,
            "{} {}",
            crate::identity().engine_id,
            crate::identity().engine_version
        )
        .map_err(|_| FailureKind::Internal),
        ["--version", "--json"] => write_json(stdout, &engine_api::version()),
        ["capabilities"] => write_json(stdout, &engine_api::capabilities()),
        ["schema", id] => engine_api::schema(id)
            .map_err(|_| FailureKind::Usage)
            // No añadir newline ni volver a serializar: son los bytes de la API.
            .and_then(|text| {
                stdout
                    .write_all(text.as_bytes())
                    .map_err(|_| FailureKind::Internal)
            }),
        ["explain", code] => engine_api::explain(code)
            .map_err(|_| FailureKind::Usage)
            .and_then(|text| writeln!(stdout, "{text}").map_err(|_| FailureKind::Internal)),
        ["explain", code, "--json"] => engine_api::explain(code)
            .map_err(|_| FailureKind::Usage)
            .and_then(|text| write_json(stdout, &json!({"code":code,"explanation":text}))),
        ["--version" | "capabilities" | "schema" | "explain", ..] => Err(FailureKind::Usage),
        ["describe" | "--help" | "help" | "-h", rest @ ..] if rest.contains(&"--json") => {
            Err(FailureKind::Usage)
        }
        _ => return None,
    };
    Some(result)
}

fn write_json(stdout: &mut impl Write, value: &Value) -> Result<(), FailureKind> {
    writeln!(stdout, "{value}").map_err(|_| FailureKind::Internal)
}

fn flag(name: &str, value: &str, required: bool, description: &str) -> Value {
    json!({"name":name,"value":value,"required":required,"description":description})
}

fn command(
    name: &str,
    aliases: &[&str],
    arguments: &[&str],
    flags: Vec<Value>,
    output: &str,
    description: &str,
) -> Value {
    json!({"name":name,"aliases":aliases,"arguments":arguments,"flags":flags,
        "subcommands":[],"output":output,"description":description})
}

fn description() -> Result<Value, FailureKind> {
    let identity = crate::identity();
    let codes = ExitCodes::engine_v1();
    let tables = codes.capabilities();
    let json_flag = || {
        flag(
            "--json",
            "",
            false,
            "Emitir JSON compacto con newline final.",
        )
    };
    let catalog = |required| {
        flag(
            "--catalog",
            "path|-",
            required,
            "Catálogo de métodos local o stdin.",
        )
    };
    let workflow = |required| {
        flag(
            "--workflow",
            "path|-",
            required,
            "Workflow local o stdin; requiere catálogo explícito.",
        )
    };
    let input = || {
        flag(
            "--input",
            "path|-",
            false,
            "engine-input/1; prevalece sobre MAMUT_ENGINE_INPUT. Una lectura por invocación.",
        )
    };
    Ok(json!({
        "schema":"cli-describe/1",
        "engineId":identity.engine_id,
        "engineVersion":identity.engine_version,
        "commands":[
            command("describe", &[], &[], vec![json_flag()], "legacy-description | cli-describe/1",
                "Sin --json conserva el JSON histórico de la base; con --json publica este contrato."),
            command("help", &["--help","-h"], &[], vec![json_flag()], "text | cli-describe/1",
                "Ayuda de la base; --json equivale a describe --json."),
            command("--version", &[], &[], vec![json_flag()], "text | version/1", "Versión real de engine-api."),
            command("capabilities", &[], &[], vec![], "capabilities/1", "Documento exacto de capacidades de engine-api, serializado en JSON."),
            command("schema", &[], &["id"], vec![], "json-schema", "Bytes exactos de engine_api::schema(id). Id desconocido: 2."),
            command("explain", &[], &["code"], vec![json_flag()], "text | explain/1", "Explicación de engine-api. Código desconocido: 2."),
            command("methods", &[], &[], vec![catalog(true)], "workflow-methods", "Lista los métodos del catálogo."),
            command("compile", &[], &[], vec![catalog(true),workflow(true)], "workflow-plan", "Compila un workflow sin ejecutarlo."),
            command("validate", &[], &[], vec![input(),catalog(false),workflow(false)], "validate-report/1 | workflow-validation | cli-run/1",
                "Con --input sin catálogo/workflow valida el modelo directamente. Con ambos documentos explícitos valida/compila solo el workflow. Sin documentos ni --input ejecuta el workflow de validación embebido."),
            command("run", &[], &[], vec![input(),catalog(false),workflow(false),flag("--progress","ndjson",false,"Eventos started/finished del workflow y líneas progress/1 en stderr.")], "cli-run/1",
                "Ejecuta engine.run con documentos embebidos si se omiten ambos; engineResults contiene results/1. Solo uno de catálogo/workflow es uso inválido."),
            command("mcp", &[], &[], vec![catalog(true)], "mcp-stdio", "Servidor MCP de la base con el proveedor del motor y transporte de entrada de compatibilidad.")
        ],
        "globalFlags":[
            flag("--timeout","seconds",false,"Finito, 0 < s <= 31536000; prevalece sobre MAMUT_ENGINE_TIMEOUT. Se valida para todos los comandos. Las consultas y validate --input directo no inician temporizador."),
            flag("--help","",false,"Ayuda de la base en los comandos de workflow y describe; también admite help y -h.")
        ],
        "environment":[
            {"name":"MAMUT_ENGINE_INPUT","value":"path","commands":["run","validate","mcp"],"description":"Ruta de compatibilidad; --input prevalece. El valor - es una ruta literal. MCP lee por llamada."},
            {"name":"MAMUT_ENGINE_TIMEOUT","value":"seconds","commands":["*"],"description":"Mismo rango que --timeout. Se valida incluso para consultas; el flag prevalece sobre una variable inválida."},
            {"name":"MAMUT_ENGINE_CANCEL_FILE","value":"path","commands":["run","validate","mcp"],"description":"La presencia del marcador solicita cancelación cooperativa. No se crea ni elimina."}
        ],
        "inputs":[
            {"id":"engine-input/1","format":"json","transport":"--input path|-; MAMUT_ENGINE_INPUT=path","maxBytes":engine_api::MAX_INPUT_BYTES,"description":"UTF-8, lectura acotada; stdin hasta EOF. No se admite más de un documento desde stdin."},
            {"id":"catalog","format":"json","transport":"--catalog path|-","maxBytes":MAX_DOCUMENT_BYTES,"description":"Catálogo de la base."},
            {"id":"workflow","format":"json","transport":"--workflow path|-","maxBytes":MAX_DOCUMENT_BYTES,"description":"Workflow de la base."}
        ],
        "outputs":[
            {"id":"cli-describe/1","format":"json","channel":"stdout","description":"Este contrato determinista, sin reloj ni rutas locales."},
            {"id":"version/1","format":"json","channel":"stdout","description":"engine_api::version(), sin campos añadidos."},
            {"id":"capabilities/1","format":"json","channel":"stdout","description":"engine_api::capabilities(), sin campos añadidos."},
            {"id":"explain/1","format":"json","channel":"stdout","description":"Objeto code, explanation."},
            {"id":"json-schema","format":"json","channel":"stdout","description":"Bytes originales de un esquema de engine-api; sin newline adicional."},
            {"id":"cli-run/1","format":"json","channel":"stdout","description":"Sobre del workflow con engineResults; results/1 para engine.run."},
            {"id":"validate-report/1","format":"json","channel":"stdout","description":"Validación directa del modelo: ok/0, warning/1, invalid/2."},
            {"id":"workflow-progress","format":"ndjson","channel":"stderr","description":"Con --progress ndjson: started/finished por paso, sin timestamps."},
            {"id":"progress/1","format":"ndjson","channel":"stderr","description":"Con --progress ndjson: una línea progress/1 por callback de engine_api::Progress (run), sin timestamps ni duraciones."},
            {"id":"cli-error/1","format":"json","channel":"stderr","description":"Sobre terminal sanitizado con código registrado; stdout vacío salvo fallo durante escritura."},
            {"id":"workflow-methods","format":"json","channel":"stdout","description":"Descriptores del catálogo de la base."},
            {"id":"workflow-plan","format":"json","channel":"stdout","description":"Plan compilado por la base."},
            {"id":"workflow-validation","format":"json","channel":"stdout","description":"Informe de validación/compilación del workflow de la base."},
            {"id":"legacy-description","format":"json","channel":"stdout","description":"describe sin --json, contrato histórico de la base."},
            {"id":"mcp-stdio","format":"mcp","channel":"stdio","description":"Frames MCP; conserva las respuestas y manifiestos de la base."},
            {"id":"text","format":"text","channel":"stdout","description":"Ayuda, versión o explicación sin --json."}
        ],
        "exitCodes":tables["exit_codes"],
        "exitCodeCategories":{
            "ok":codes.ok,"warning":codes.warning,"usage":codes.usage,
            "document":codes.document,"validation":codes.validation,"unsupported":codes.unsupported,
            "not_converged":codes.not_converged,"method":codes.method,
            "cancelled":codes.cancelled,"timeout":codes.timeout,"internal":codes.internal
        },
        "limits":{
            "MAX_INPUT_BYTES":engine_api::MAX_INPUT_BYTES,
            "MAX_NODES":engine_api::MAX_NODES,"MAX_MEMBERS":engine_api::MAX_MEMBERS,
            "MAX_CASES":engine_api::MAX_CASES,"MAX_COMBINATIONS":engine_api::MAX_COMBINATIONS,
            "MAX_LOADS":engine_api::MAX_LOADS,"MAX_JSON_DEPTH":engine_api::MAX_JSON_DEPTH,
            "MAX_JSON_ARRAY_ITEMS":engine_api::MAX_JSON_ARRAY_ITEMS,
            "MAX_CONSOLIDATION_WORK":engine_api::MAX_CONSOLIDATION_WORK,
            "MAX_CONSOLIDATION_COST":engine_api::MAX_CONSOLIDATION_COST,
            "MAX_CONSOLIDATION_SEGMENTS":engine_api::MAX_CONSOLIDATION_SEGMENTS,
            "MAX_CONSOLIDATION_SUBSTEPS":engine_api::MAX_CONSOLIDATION_SUBSTEPS,
            "MAX_DOCUMENT_BYTES":MAX_DOCUMENT_BYTES,"MAX_TIMEOUT_SECONDS":31536000
        },
        "argumentSyntax":"Flags separados de su valor por un espacio; sin --flag=value ni separador --. Flags repetidos inválidos. Solo un origen stdin entre input, catálogo y workflow.",
        "schemas":schema_inventory()?
    }))
}

fn schema_inventory() -> Result<Vec<Value>, FailureKind> {
    let index: Value = serde_json::from_str(INDEX).map_err(|_| FailureKind::Internal)?;
    let entries = index["schemas"].as_array().ok_or(FailureKind::Internal)?;
    let mut inventory = Vec::with_capacity(entries.len());
    // El índice publicado contiene las huellas de los bytes originales, no de
    // una reserialización. --check y los tests verifican su correspondencia.
    for entry in entries {
        let id = entry["id"].as_str().ok_or(FailureKind::Internal)?;
        let version = entry["version"].as_u64().ok_or(FailureKind::Internal)?;
        let kind = format!("{id}/{version}");
        let from_api = engine_api::schema_kinds().any(|known| known == kind);
        if from_api {
            let text = engine_api::schema(&kind).map_err(|_| FailureKind::Internal)?;
            let schema: Value = serde_json::from_str(text).map_err(|_| FailureKind::Internal)?;
            if schema["$id"] != entry["$id"] {
                return Err(FailureKind::Internal);
            }
        }
        inventory.push(json!({"id":kind,"$id":entry["$id"],"sha256":entry["sha256"],"availableViaSchema":from_api}));
    }
    if engine_api::schema_kinds().any(|kind| !inventory.iter().any(|entry| entry["id"] == kind)) {
        return Err(FailureKind::Internal);
    }
    Ok(inventory)
}
