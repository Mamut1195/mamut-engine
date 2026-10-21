//! Ejecución y serialización, sin E/S.
pub(crate) mod advanced;
pub(crate) mod geo;
pub(crate) mod nonlinear;
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
    /// Cancelación cooperativa en un punto de control.
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

pub(crate) struct Failure(pub(crate) Status, pub(crate) Vec<Diagnostic>);
impl From<engine_core::Diagnostic> for Failure {
    fn from(d: engine_core::Diagnostic) -> Self {
        Self(Status::Invalid, vec![d.into()])
    }
}
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
    nodes: Vec<engine_core::NodeId>,
    variable: bool,
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
    let diagnostics = validation(&execution);
    if diagnostics.iter().any(|d| d.severity == "error") {
        return Err(Failure(
            if diagnostics.iter().any(|d| d.code == "ENG-E-0013") {
                Status::Unsupported
            } else {
                Status::Invalid
            },
            diagnostics,
        ));
    }
    document["diagnostics"] = json!(diagnostics);
    step(cancel, progress, "model", 2, total)?;
    if let crate::methods::Runner::Standalone(execute) = method.execute {
        execute(&execution, cancel, progress, document, total)?;
        return step(cancel, progress, "serialization", total, total);
    }
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
    identities(document, &execution, &members);
    method.run(MethodContext {
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

fn identities(document: &mut Value, execution: &input::Execution, members: &[Member]) {
    let model = &execution.model;
    // Identidades/geometría arriba; respuestas numéricas siempre dentro de su caso.
    document["nodes"] = json!(model.nodes.iter().map(|n| json!({"id": n.id.0, "dim": format!("{:?}", n.dim), "coordinates": n.coordinates, "dofs": n.dofs.iter().map(|d| format!("{d:?}")).collect::<Vec<_>>()})).collect::<Vec<_>>());
    let mut identities: Vec<_> = model.members.iter().zip(members).map(|(m, built)| json!({"id": m.id.0, "nodes": m.nodes.map(|n| n.0), "section": m.section, "law": m.law, "element": built.element.kind()})).collect();
    identities.extend(members.iter().skip(model.members.len()).zip(&execution.springs).map(|(m, s)| {
        let ends: &[&str] = if m.nodes.len() == 1 { &["j"] } else { &["i", "j"] };
        let labels: Vec<_> = ends.iter().flat_map(|end| m.element.dofs_per_node().iter().map(move |dof| format!("{end}.{dof:?}"))).collect();
        json!({"id": m.id.0, "nodes": m.nodes.iter().map(|n| n.0).collect::<Vec<_>>(), "section": "", "law": s.law.as_deref().unwrap_or(""), "element": m.element.kind(), "end_dofs": labels})
    }));
    document["members"] = json!(identities);
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
    for (m, beam) in model.members.iter().zip(&execution.elements) {
        let kind = &beam.kind;
        let orientation = &beam.orientation;
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
                "frame2d.winkler" => Box::new(engine_elements_frame::WinklerFactory),
                "frame3d.winkler" => Box::new(engine_elements_frame3d::WinklerFactory),
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
            for (key, release) in [
                ("release_i", &beam.release_i),
                ("release_j", &beam.release_j),
            ] {
                if let Some(release) = release {
                    let value = match release {
                        crate::extensions::Release::Plane(v) => JsonValue::Bool(*v),
                        crate::extensions::Release::Spatial(v) => {
                            JsonValue::Array(v.iter().map(|b| JsonValue::Bool(*b)).collect())
                        }
                    };
                    fields.insert(key.into(), value);
                }
            }
            if let Some(w) = &beam.winkler {
                for (key, value) in [("k", w.k), ("ky", w.ky), ("kz", w.kz)] {
                    if let Some(value) = value {
                        if fields
                            .insert(key.into(), JsonValue::Number(value))
                            .is_some()
                        {
                            return Err(crate::extensions::invalid("Propiedad Winkler duplicada."));
                        }
                    }
                }
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
                nodes: m.nodes.to_vec(),
                variable: false,
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
    for spring in &execution.springs {
        let (dim, element) = spring.build(execution)?;
        members.push(Member {
            id: engine_core::MemberId(spring.id),
            nodes: spring
                .nodes
                .iter()
                .map(|&n| engine_core::NodeId(n))
                .collect(),
            variable: spring.kind == "spring.nonlinear",
            dim,
            element: Arc::from(element),
            params: ElementParams::default(),
            loads: vec![vec![]; model.cases.len()],
        });
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
        .filter(|m| !m.variable)
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
        "API-E-SFC-OPTIONS" => Some("SFC-010: nonlinearStatic, soilProfile y consolidation1d son Experimental hasta la cobertura de ENG-VAL-006 (decisión de Jonathan vía J0 2026-10-20); requieren options.allow_experimental = true en la raíz del job. Revise compatibilidad entre método, resorte, ley, Winkler y liberaciones. Parámetros físicos y procedencia de suelo son explícitos; consulte docs/GUIA_METODOS.md."),
        "API-E-CONSOLIDATION-WORK" => Some("consolidation1d supera MAX_CONSOLIDATION_WORK = 50000000: suma de subpasos de toda la historia, partida en quiebres de carga y tiempos de salida, multiplicada por el número total de elementos. Reduzca historia/malla o aumente max_step_s. Se rechaza durante la validación, antes de integrar; consulte docs/GUIA_METODOS.md."),
        "API-E-CONSOLIDATION-COST" => Some("consolidation1d supera MAX_CONSOLIDATION_COST = 500000000 (Σ intervalos n³ + Σ subpasos n², n nodos libres: elementos con drenaje simple, elementos−1 con drenaje doble), MAX_CONSOLIDATION_SEGMENTS = 256 o MAX_CONSOLIDATION_SUBSTEPS = 100000. Los intervalos unen quiebres de carga y tiempos de salida de toda la historia, incluso después de la última salida; cada uno aporta max(1, ceil(intervalo/max_step_s)) subpasos. Aritmética u128 saturante, sin descontar factores reutilizados. Se rechaza en validación con invalid / salida 2; reduzca historia/malla/salidas o aumente max_step_s. Presupuesto calibrado en release con objetivo ≤5 s, sin garantía de plazo en otras máquinas; consulte docs/GUIA_METODOS.md y provenance/ENG-SFC-010.md, r3."),
        "API-E-INPUT" => Some("JSON o estructura engine-input/1 inválida. Revise campos, tipos y referencias de cargas de miembro. nonlinearStatic, soilProfile y consolidation1d son Experimental hasta ENG-VAL-006: requieren options.allow_experimental = true en la raíz del job."),
        "API-I-CANCELLED" => Some("Cancelación cooperativa solicitada entre etapas, antes de combine/factorize de cada intervalo de consolidación, entre bloques de hasta 16 columnas de su factorización de referencia o antes de cada subpaso. No se emiten respuestas parciales."),
        "API-W-COMBINED-PRESCRIBED" => Some("Seguimiento #85: los prescritos de una combinación son suma factor·δ, incluido el asentamiento único ponderado. Revise el factor explícito; varios casos de asentamiento no nulo se rechazan."),
        "API-W-DIAPHRAGM-RECOVERY" => Some("Seguimiento #84: la recuperación con diafragmas no tiene cobertura de pruebas del dominio. Revise equilibrio y resultados antes de usarlos."),
        "API-W-MASS-PROJECTION" => Some("#86: la conversión de cargas a masa rechaza cualquier proyección negativa, sin tolerancia; con gravedad no alineada a ejes puede rechazar por redondeo."),
        "API-W-PDELTA-INERTIA" => Some("#78: inercia LDLᵀ con bloques 2×2 usa copysign sin prueba confirmada; revisar el diagnóstico de estabilidad."),
        _ => None,
    }
}

/// Integra validación del modelo y las extensiones sin alterar el dominio.
pub(crate) fn validation(e: &input::Execution) -> Vec<Diagnostic> {
    let mut diagnostics: Vec<_> = engine_model::validate(&e.model).into_iter().filter(|d| {
        // Un nodo unido a un resorte ya tiene conectividad de frontera.
        !(d.code == "ENG-E-0004" && d.entities.iter().any(|entity| matches!(entity, EntityRef::Node(id) if e.springs.iter().any(|s| s.nodes.contains(&id.0)))))
    }).map(Diagnostic::from).collect();
    if diagnostics.iter().any(|d| d.severity == "error") {
        return diagnostics;
    }
    if let Err(error) = crate::extensions::validate(e) {
        diagnostics.extend(Failure::from(error).1);
        return diagnostics;
    }
    if let Err(error) = crate::geo_input::validate(e) {
        diagnostics.push(error.into());
        return diagnostics;
    }
    if e.elements.iter().any(|b| {
        b.winkler.is_some()
            || b.release_i.is_some()
            || b.release_j.is_some()
            || matches!(b.kind.as_str(), "frame2d.winkler" | "frame3d.winkler")
    }) {
        if let Err(failure) = build_members(e) {
            diagnostics.extend(failure.1);
        }
    }
    diagnostics
}
