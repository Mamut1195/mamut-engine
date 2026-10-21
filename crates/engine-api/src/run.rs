//! Ejecución y serialización, sin E/S.
pub(crate) mod advanced;
pub(crate) mod static_linear;
use crate::{input, ApiError, CancelToken, Diagnostic, Progress};
use engine_analysis::{
    combination::combine, results::CaseResults, results::MemberRecovery, StaticLinear,
};
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem};
use engine_contracts::{
    CaseRef, Element, ElementCtx, ElementFactory, ElementParams, JsonValue, ModelView,
};
use engine_core::{Dim, EngineError, EntityRef, Severity};
use engine_linalg::ReferenceBackend;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Estado estable de una ejecución, independiente del proceso anfitrión.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    /// Resultados completos sin avisos.
    Ok,
    /// Resultados completos con limitaciones explícitas.
    Warning,
    /// Entrada inválida, incluido contrato desconocido.
    Invalid,
    /// Capacidad solicitada no implementada.
    Unsupported,
    /// Solver sin convergencia.
    NotConverged,
    /// Sistema singular o mecanismo.
    Unstable,
    /// Cancelación entre etapas.
    Cancelled,
    /// Reservado; no se generan plazos en esta fachada.
    Timeout,
    /// Error interno comunicado mediante diagnóstico.
    InternalError,
}

/// Código de salida portable; no termina el proceso.
pub const fn exit_code(status: Status) -> i32 {
    match status {
        Status::Ok => 0,
        Status::Warning => 1,
        Status::Invalid => 2,
        Status::Unsupported => 3,
        Status::NotConverged | Status::Unstable => 4,
        Status::Cancelled => 5,
        Status::Timeout => 6,
        Status::InternalError => 70,
    }
}

/// Resultado de frontera, también para entrada inválida o cancelación.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Estado que también figura en el documento.
    pub status: Status,
    /// Bytes UTF-8 canónicos de `results/1`, sin relojes ni rutas.
    pub document: String,
}
impl Outcome {
    /// Código de salida para CLI/bindings.
    pub const fn exit_code(&self) -> i32 {
        exit_code(self.status)
    }
}

pub(crate) struct MethodContext<'a> {
    pub execution: &'a input::Execution,
    pub members: &'a [Member],
    pub contributions: &'a [ElementContribution],
    pub system: &'a mut LinearSystem<ReferenceBackend>,
    pub cancel: &'a CancelToken,
    pub progress: &'a mut dyn FnMut(Progress),
    pub document: &'a mut Value,
    pub total: usize,
    pub cases: &'a [CaseResults],
}

pub(crate) struct Failure(Status, Vec<Diagnostic>);
impl From<EngineError> for Failure {
    fn from(e: EngineError) -> Self {
        let status = match e.code {
            "ENG-E-0006" | "ENG-E-0014" => Status::Unstable,
            "ENG-E-0007" => Status::NotConverged,
            "ENG-E-0012" => Status::InternalError,
            "ENG-E-0013" => Status::Unsupported,
            _ => Status::Invalid,
        };
        Self(
            status,
            vec![Diagnostic::from(engine_core::Diagnostic {
                code: e.code,
                severity: Severity::Error,
                entities: e.entities,
                message: e.message,
                hint: None,
            })],
        )
    }
}
impl From<ApiError> for Failure {
    fn from(e: ApiError) -> Self {
        let (status, code) = match &e {
            ApiError::Cancelled => (Status::Cancelled, "API-I-CANCELLED"),
            ApiError::UnsupportedSchema(_) => (Status::Invalid, "ENG-E-0009"),
            _ => (Status::Invalid, "API-E-INPUT"),
        };
        Self(status, vec![diagnostic(code, "error", e.to_string())])
    }
}
fn diagnostic(code: &'static str, severity: &'static str, cause: String) -> Diagnostic {
    Diagnostic {
        code,
        severity,
        entities: vec![],
        cause,
        hint: None,
    }
}

pub(crate) struct Member {
    id: engine_core::MemberId,
    nodes: [engine_core::NodeId; 2],
    dim: Dim,
    element: Arc<dyn Element>,
    params: ElementParams,
    // Una única fuente para load_vector y MemberRecovery, sin cargas escaladas.
    loads: Vec<Vec<engine_contracts::MemberLoad>>,
}

/// Ejecuta linearStatic, pDelta, modal o responseSpectrum con opciones explícitas.
/// Consulta cancelación antes de empezar y antes/después de cada callback.
/// Parseo, modelo validado, ensamblado, cada caso, combinaciones y serialización
/// son indivisibles. El callback es síncrono y debe retornar normalmente.
/// Un fallo elimina todos los resultados parciales; input_sha256 es null si no
/// se completó el parseo/huella RFC 8785 con mamut-canon. Los métodos iterativos
/// consultan entre pasos/modos.
pub fn run(input_json: &str, cancel: &CancelToken, mut progress: impl FnMut(Progress)) -> Outcome {
    let mut document = json!({
        "schema": "results/1", "engine_version": env!("CARGO_PKG_VERSION"),
        "git_commit": option_env!("MAMUT_ENGINE_BUILD_COMMIT"),
        "target": env!("MAMUT_ENGINE_TARGET"),
        "contracts": ["engine-input/1", "results/1", "element/1", "system/1"],
        "methods": [], "input_sha256": null, "status": "ok",
        "nodes": [], "members": [], "cases": [], "combinations": [],
        "modes": [], "envelopes": [], "convergence": null, "diagnostics": []
    });
    let result = execute(input_json, cancel, &mut progress, &mut document);
    let status = match result {
        Ok(()) => {
            if document["diagnostics"]
                .as_array()
                .is_some_and(|d| d.iter().any(|d| d["severity"] == "warning"))
            {
                Status::Warning
            } else {
                Status::Ok
            }
        }
        Err(Failure(status, diagnostics)) => {
            for key in [
                "nodes",
                "members",
                "cases",
                "combinations",
                "modes",
                "envelopes",
            ] {
                document[key] = json!([]);
            }
            if !matches!(status, Status::NotConverged | Status::Unstable) {
                document["convergence"] = Value::Null;
            }
            for d in diagnostics {
                push_diagnostic(&mut document, d);
            }
            status
        }
    };
    document["status"] = json!(status);
    // Value contiene solo tipos serializables y números finitos del dominio.
    match crate::json::canonical_value(document) {
        Ok(document) => Outcome { status, document },
        Err(_) => Outcome {
            status: Status::InternalError,
            document: internal_document(),
        },
    }
}

fn internal_document() -> String {
    // Sobre constante válido incluso si fallara la serialización de la salida.
    format!(concat!("{{\"cases\":[],\"combinations\":[],\"contracts\":[\"engine-input/1\",\"results/1\"],",
        "\"convergence\":null,\"diagnostics\":[{{\"cause\":\"Error al serializar results/1\",\"code\":\"ENG-E-0012\",",
        "\"entities\":[],\"hint\":null,\"severity\":\"error\"}}],\"engine_version\":\"{}\",\"envelopes\":[],",
        "\"git_commit\":{},\"input_sha256\":null,\"members\":[],\"methods\":[],\"modes\":[],\"nodes\":[],\"schema\":\"results/1\",\"status\":\"internalError\",\"target\":\"{}\"}}"),
        env!("CARGO_PKG_VERSION"),
        option_env!("MAMUT_ENGINE_BUILD_COMMIT").map_or_else(|| "null".into(), |s| format!("\"{s}\"")),
        env!("MAMUT_ENGINE_TARGET"))
}

fn step(
    cancel: &CancelToken,
    progress: &mut dyn FnMut(Progress),
    stage: &'static str,
    completed: usize,
    total: usize,
) -> Result<(), Failure> {
    cancel.check()?;
    progress(Progress {
        operation: "run",
        stage,
        completed,
        total,
    });
    cancel.check()?;
    Ok(())
}

fn execute(
    input_json: &str,
    cancel: &CancelToken,
    progress: &mut dyn FnMut(Progress),
    document: &mut Value,
) -> Result<(), Failure> {
    cancel.check()?;
    let value = crate::json::parse(input_json)?;
    document["input_sha256"] = json!(crate::fingerprint_value(&value)?);
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::InvalidInput("schema obligatorio".into()))?;
    if schema != "engine-input/1" {
        return Err(ApiError::UnsupportedSchema(schema.into()).into());
    }
    let execution = input::decode(value)?;
    if execution.methods.len() != 1 {
        return Err(ApiError::InvalidInput("Se requiere exactamente un análisis".into()).into());
    }
    let method = crate::methods::resolve(
        execution.methods[0],
        crate::methods::INPUT_CONTRACT,
        execution.allow_experimental,
    )?;
    let total = (method.budget)(&execution);
    step(cancel, progress, "parse", 1, total)?;
    let model = &execution.model;
    document["methods"] = json!(execution.methods);
    let diagnostics: Vec<_> = engine_model::validate(model)
        .into_iter()
        .map(Diagnostic::from)
        .collect();
    if diagnostics.iter().any(|d| d.severity == "error") {
        return Err(Failure(Status::Invalid, diagnostics));
    }
    document["diagnostics"] = json!(diagnostics);
    step(cancel, progress, "model", 2, total)?;
    let members = build_members(&execution)?;
    let contributions = assemble_members(model, &members)?;
    let mut system = LinearSystem::from_model_with_diaphragms(
        ReferenceBackend,
        model,
        &contributions,
        &execution.prescribed,
        &execution.diaphragms,
    )?;
    if !execution.diaphragms.is_empty() {
        push_warning(document, "API-W-DIAPHRAGM-RECOVERY", "#84: recuperación con diafragmas sin cobertura de pruebas del dominio; revisar equilibrio y resultados.".into());
    }
    step(cancel, progress, "assembly", 3, total)?;
    identities(document, model, &members);
    (method.execute)(MethodContext {
        execution: &execution,
        members: &members,
        contributions: &contributions,
        system: &mut system,
        cancel,
        progress,
        document,
        total,
        cases: &[],
    })?;
    crate::json::canonical_value(document.clone())
        .map_err(|_| EngineError::new("ENG-E-0012", "Error al serializar results/1"))?;
    step(cancel, progress, "serialization", total, total)
}

fn identities(document: &mut Value, model: &engine_model::Model, members: &[Member]) {
    // Identidades/geometría arriba; respuestas numéricas siempre dentro de su caso.
    document["nodes"] = json!(model.nodes.iter().map(|n| json!({"id": n.id.0, "dim": format!("{:?}", n.dim), "coordinates": n.coordinates, "dofs": n.dofs.iter().map(|d| format!("{d:?}")).collect::<Vec<_>>()})).collect::<Vec<_>>());
    document["members"] = json!(model.members.iter().zip(members).map(|(m, built)| json!({"id": m.id.0, "nodes": m.nodes.map(|n| n.0), "section": m.section, "law": m.law, "element": built.element.kind()})).collect::<Vec<_>>());
}

fn push_warning(document: &mut Value, code: &'static str, cause: String) {
    push_diagnostic(document, diagnostic(code, "warning", cause));
}
fn push_diagnostic(document: &mut Value, diagnostic: Diagnostic) {
    if let Some(d) = document["diagnostics"].as_array_mut() {
        d.push(json!(diagnostic));
    }
}

fn build_members(execution: &input::Execution) -> Result<Vec<Member>, Failure> {
    let model = &execution.model;
    let map = DofMap::new(&model.nodes)?;
    let mut members = Vec::new();
    for (m, (kind, orientation)) in model.members.iter().zip(&execution.elements) {
        let build = || -> Result<Member, EngineError> {
            let dimension = |id| {
                model
                    .nodes
                    .iter()
                    .find(|n| n.id == id)
                    .map(|n| n.dim)
                    .ok_or_else(|| EngineError::new("ENG-E-0003", "Nodo ausente"))
            };
            let dim = dimension(m.nodes[0])?;
            if dimension(m.nodes[1])? != dim {
                return Err(EngineError::new(
                    "ENG-E-0010",
                    "Dimensiones de extremos distintas",
                ));
            }
            let kind = if kind.is_empty() {
                match dim {
                    Dim::D2 => "frame2d.euler_elastic",
                    Dim::D3 => "frame3d.euler_elastic",
                }
            } else {
                kind.as_str()
            };
            let factory: Box<dyn ElementFactory> = match kind {
                "frame2d.euler_elastic" => Box::new(engine_elements_frame::FrameFactory),
                "frame3d.euler_elastic" => Box::new(engine_elements_frame3d::FrameFactory),
                "truss2d.linear" => Box::new(engine_elements_truss::TrussFactory::new(Dim::D2)),
                "truss3d.linear" => Box::new(engine_elements_truss::TrussFactory::new(Dim::D3)),
                _ => {
                    return Err(EngineError::new(
                        "ENG-E-0013",
                        format!("Elemento {kind} no soportado"),
                    ))
                }
            };
            let section = model
                .sections
                .iter()
                .find(|s| s.id == m.section)
                .ok_or_else(|| EngineError::new("ENG-E-0003", "Sección ausente"))?;
            let law = model
                .laws
                .iter()
                .find(|l| l.name == m.law)
                .ok_or_else(|| EngineError::new("ENG-E-0003", "Ley ausente"))?;
            let mut fields = BTreeMap::new();
            let mut params = ElementParams::default();
            for p in section.properties.iter().chain(&law.parameters) {
                if fields
                    .insert(p.name.clone(), JsonValue::Number(p.value))
                    .is_some()
                {
                    return Err(EngineError::new(
                        "ENG-E-0002",
                        format!("Parámetro {} duplicado", p.name),
                    ));
                }
                params.scalars.insert(p.name.clone(), p.value);
            }
            if let Some(o) = orientation {
                fields.insert(
                    "orientation".into(),
                    JsonValue::Array(o.iter().map(|v| JsonValue::Number(*v)).collect()),
                );
            }
            let coords = m
                .nodes
                .iter()
                .map(|n| map.coordinates(*n))
                .collect::<Result<Vec<_>, _>>()?;
            let nodes = m.nodes.iter().copied().zip(coords).collect::<Vec<_>>();
            let element = factory.build(
                &JsonValue::Object(fields),
                &ModelView { dim, nodes: &nodes },
            )?;
            let loads = execution
                .loads
                .iter()
                .map(|loads| {
                    loads
                        .iter()
                        .filter(|(id, _)| *id == m.id)
                        .map(|(_, l)| l.clone())
                        .collect()
                })
                .collect();
            Ok(Member {
                id: m.id,
                nodes: m.nodes,
                dim,
                element: Arc::from(element),
                params,
                loads,
            })
        };
        members.push(build().map_err(|mut e| {
            e.entities.push(EntityRef::Member(m.id));
            e
        })?);
    }
    Ok(members)
}

fn assemble_members(
    model: &engine_model::Model,
    members: &[Member],
) -> Result<Vec<ElementContribution>, Failure> {
    let map = DofMap::new(&model.nodes)?;
    members
        .iter()
        .map(|m| {
            let assemble = || -> Result<ElementContribution, EngineError> {
                let coords = m
                    .nodes
                    .iter()
                    .map(|n| map.coordinates(*n))
                    .collect::<Result<Vec<_>, _>>()?;
                let ctx = ElementCtx {
                    dim: m.dim,
                    coords: &coords,
                    params: &m.params,
                };
                let mut location = Vec::new();
                for &node in &m.nodes {
                    for &dof in m.element.dofs_per_node() {
                        location.push(map.global_index(node, dof)?);
                    }
                }
                let mut loads = Vec::new();
                for (case, member_loads) in model.cases.iter().zip(&m.loads) {
                    for load in member_loads {
                        loads.push(ElementLoad {
                            case: CaseRef(case.name.clone()),
                            values: m.element.load_vector(&ctx, load)?,
                        });
                    }
                }
                Ok(ElementContribution {
                    stiffness: m
                        .element
                        .respond(&ctx, &vec![0.0; location.len()], &[])?
                        .tangent,
                    location,
                    mass_lumped: None,
                    mass_consistent: None,
                    geometric: None,
                    loads,
                })
            };
            assemble().map_err(|mut e| {
                e.entities.push(EntityRef::Member(m.id));
                Failure::from(e)
            })
        })
        .collect()
}

fn case_json(case: &CaseResults) -> Value {
    json!({"case": case.case.0, "factor": case.factor,
        "nodes": case.nodes.iter().map(|n| json!({"id": n.id.0, "displacements": n.displacements, "prescribed_displacements": n.prescribed_displacements, "reactions": n.reactions})).collect::<Vec<_>>(),
        "members": case.members.iter().map(|m| json!({"id": m.id.0, "end_forces_local": m.end_forces_local,
            "stations": m.stations.iter().map(|s| json!({"position": s.position, "forces_local": s.forces_local})).collect::<Vec<_>>(),
            "extremes": m.extremes.iter().map(|e| json!({"field": format!("{:?}", e.field), "max": {"value": e.max.value, "at_station": e.max.at_station}, "min": {"value": e.min.value, "at_station": e.min.at_station}})).collect::<Vec<_>>() })).collect::<Vec<_>>()})
}

pub(crate) fn explain(code: &str) -> Option<&'static str> {
    match code {
        "API-E-INPUT" => Some("JSON o estructura engine-input/1 inválida. Revise campos, tipos y referencias de cargas de miembro."),
        "API-I-CANCELLED" => Some("Cancelación cooperativa solicitada entre etapas. No se emiten respuestas parciales."),
        "API-W-COMBINED-PRESCRIBED" => Some("Seguimiento #85: los prescritos de una combinación son suma factor·δ, incluido el asentamiento único ponderado. Revise el factor explícito; varios casos de asentamiento no nulo se rechazan."),
        "API-W-DIAPHRAGM-RECOVERY" => Some("Seguimiento #84: la recuperación con diafragmas no tiene cobertura de pruebas del dominio. Revise equilibrio y resultados antes de usarlos."),
        "API-W-MASS-PROJECTION" => Some("#86: la conversión de cargas a masa rechaza cualquier proyección negativa, sin tolerancia; con gravedad no alineada a ejes puede rechazar por redondeo."),
        "API-W-PDELTA-INERTIA" => Some("#78: inercia LDLᵀ con bloques 2×2 usa copysign sin prueba confirmada; revisar el diagnóstico de estabilidad."),
        _ => None,
    }
}
