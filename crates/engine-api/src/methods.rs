//! Extensión de dominio sobre descriptores y fábricas de mamut-workflow.
use crate::{input::Execution, run, ApiError};
use mamut_workflow::{
    ContractId, MethodCatalog, MethodDimensions, MethodFactory, MethodRegistry, MethodSelectionV1,
    NumericParameters, ProfileContract, QuantityDescriptor, SlotContract, WorkflowCompiler,
    WorkflowDefinitionV1,
};
use serde::Serialize;

pub(crate) const INPUT_CONTRACT: &str = "urn:mamut:engine:engine-input:1.0.0";
pub(crate) const RESULTS_CONTRACT: &str = "urn:mamut:engine:results:1.0.0";
const SLOT: &str = "engine_method";
const PROFILE: &str = "engine.run";

/// Madurez del método en la fachada; no certifica todos los modelos posibles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MethodStatus {
    /// Requiere consentimiento explícito en las opciones del job.
    Experimental,
    /// Habilitado por defecto con evidencia de benchmarks dentro de su alcance.
    Stable,
    /// Cuenta con los fixtures analíticos indicados, dentro de su alcance.
    Validated,
    /// Se conserva por compatibilidad; su evidencia explica la sustitución.
    Deprecated,
}

/// Descriptor method-descriptor/1: registro SDK más evidencia del Engine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MethodDescriptor {
    /// Versión del documento de descubrimiento.
    pub schema: &'static str,
    /// Descriptor genérico registrado en mamut-workflow, sin duplicar su modelo.
    pub workflow: mamut_workflow::MethodDescriptor,
    /// Madurez y política de habilitación del método.
    pub status: MethodStatus,
    /// Contrato JSON consumido por el adaptador.
    pub input_schema: &'static str,
    /// Contrato JSON producido por el adaptador.
    pub output_schema: &'static str,
    /// Rutas de pruebas/fixtures propios que delimitan la validación.
    pub fixtures: Vec<&'static str>,
    /// Citas de derivaciones propias, con su sección; sin atribuciones externas.
    pub citations: Vec<&'static str>,
    /// Rutas de bitácoras clean-room del método y del registro.
    pub provenance: Vec<&'static str>,
}

pub(crate) struct EngineMethod {
    descriptor: MethodDescriptor,
    pub execute: Runner,
    pub budget: fn(&Execution) -> usize,
}

fn contract(text: &str) -> Result<ContractId, ApiError> {
    ContractId::parse(text).map_err(|e| ApiError::InvalidInput(e.to_string()))
}

fn dimensions() -> MethodDimensions {
    // Se transportan documentos completos. Las magnitudes SI y sus unidades
    // permanecen en engine-input/1 y results/1, no son puertos escalares del SDK.
    MethodDimensions {
        inputs: vec![QuantityDescriptor::new("document", "1", "1")],
        outputs: vec![QuantityDescriptor::new("document", "1", "1")],
    }
}

fn registry() -> Result<MethodRegistry<EngineMethod>, ApiError> {
    let mut registry = MethodRegistry::new();
    let factories: [MethodFactory<EngineMethod>; 8] = [
        linear_static,
        combinations,
        pdelta,
        modal,
        spectrum,
        nonlinear,
        soil,
        consolidation,
    ];
    for factory in factories {
        let method = factory(&NumericParameters::new()).map_err(ApiError::InvalidInput)?;
        registry
            .register(method.descriptor.workflow, factory)
            .map_err(|e| ApiError::InvalidInput(format!("Engine registry: {e}")))?;
    }
    Ok(registry)
}

/// Descriptores en el orden determinista del registro real de mamut-workflow.
pub fn method_descriptors() -> Result<Vec<MethodDescriptor>, ApiError> {
    let registry = registry()?;
    registry
        .descriptors()
        .into_iter()
        .map(|descriptor| {
            registry
                .create(descriptor, &NumericParameters::new())
                .map(|product| product.instance.descriptor)
                .map_err(|e| ApiError::InvalidInput(e.to_string()))
        })
        .collect()
}

/// Resuelve id y contrato URN (dominio, nombre y mayor), aplicando la política.
/// Versiones minor/patch compatibles siguen las reglas del compilador SDK.
/// No ejecuta cálculos ni convierte engine-input/1 en un formato de workflow.
pub fn method_descriptor(
    method_id: &str,
    input_contract: &str,
    allow_experimental: bool,
) -> Result<MethodDescriptor, ApiError> {
    Ok(resolve(method_id, input_contract, allow_experimental)?.descriptor)
}

pub(crate) fn resolve(
    method_id: &str,
    input_contract: &str,
    allow_experimental: bool,
) -> Result<EngineMethod, ApiError> {
    let registry = registry()?;
    let profile = ProfileContract {
        profile: PROFILE.into(),
        version: 1,
        slots: vec![SlotContract {
            capability_slot: SLOT.into(),
            dimensions: dimensions(),
            consumes: vec![contract(input_contract)?],
            produces: vec![contract(RESULTS_CONTRACT)?],
        }],
    };
    let definition = WorkflowDefinitionV1 {
        profile: PROFILE.into(),
        version: 1,
        steps: vec![MethodSelectionV1 {
            capability_slot: SLOT.into(),
            method_id: method_id.into(),
            method_version: 1,
            parameters: NumericParameters::new(),
        }],
    };
    let plan = WorkflowCompiler::new(&registry, profile)
        .compile(&definition)
        .map_err(|e| ApiError::InvalidInput(format!("Method {method_id}: {e}")))?;
    let method = plan
        .steps
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::InvalidInput("Engine plan has no method".into()))?
        .instance;
    enforce_policy(&method.descriptor, allow_experimental)?;
    Ok(method)
}

fn enforce_policy(descriptor: &MethodDescriptor, allow_experimental: bool) -> Result<(), ApiError> {
    if descriptor.status == MethodStatus::Experimental && !allow_experimental {
        let method_id = &descriptor.workflow.method_id;
        return Err(ApiError::InvalidInput(format!(
            "Experimental method {method_id}: requires options.allow_experimental = true"
        )));
    }
    Ok(())
}

type StructuralRunner = for<'a> fn(run::MethodContext<'a>) -> Result<(), run::Failure>;
type StandaloneRunner = fn(
    &Execution,
    &crate::CancelToken,
    &mut dyn FnMut(crate::Progress),
    &mut serde_json::Value,
    usize,
) -> Result<(), run::Failure>;
#[derive(Clone, Copy)]
pub(crate) enum Runner {
    Structural(StructuralRunner),
    Standalone(StandaloneRunner),
}
impl EngineMethod {
    pub fn run(&self, context: run::MethodContext<'_>) -> Result<(), run::Failure> {
        match self.execute {
            Runner::Structural(execute) => execute(context),
            Runner::Standalone(execute) => execute(
                context.execution,
                context.cancel,
                context.progress,
                context.document,
                context.total,
            ),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn builtin(
    id: &str,
    status: MethodStatus,
    input_schema: &'static str,
    execute: Runner,
    budget: fn(&Execution) -> usize,
    fixture: &'static str,
    citation: &'static str,
    provenance: &'static str,
) -> Result<EngineMethod, String> {
    let consumes = if input_schema == "engine-input/1" {
        INPUT_CONTRACT
    } else {
        RESULTS_CONTRACT
    };
    Ok(EngineMethod {
        descriptor: MethodDescriptor {
            schema: "method-descriptor/1",
            workflow: mamut_workflow::MethodDescriptor {
                method_id: id.into(),
                method_version: 1,
                capability_slot: SLOT.into(),
                dimensions: dimensions(),
                consumes: vec![ContractId::parse(consumes).map_err(|e| e.to_string())?],
                produces: vec![ContractId::parse(RESULTS_CONTRACT).map_err(|e| e.to_string())?],
                // Las opciones mecánicas están tipadas en el documento consumido;
                // no se duplican como NumericParameters del workflow.
                parameters: vec![],
                failure: None,
            },
            status,
            input_schema,
            output_schema: "results/1",
            fixtures: vec![fixture],
            citations: vec![citation],
            provenance: vec![
                provenance,
                "provenance/ENG-API-001.md",
                "provenance/ENG-CORE-006.md",
            ],
        },
        execute,
        budget,
    })
}

fn linear_static(_: &NumericParameters) -> Result<EngineMethod, String> {
    builtin(
        "linearStatic", MethodStatus::Validated, "engine-input/1",
        Runner::Structural(run::static_linear::execute), run::static_linear::budget,
        "crates/engine-api/fixtures/input/valid/cantilever-linear-static.json",
        "provenance/ENG-API-001.md · Incremento 2a: equilibrio e integración del voladizo y viga apoyada",
        "provenance/ENG-ANL-001.md",
    )
}
fn combinations(_: &NumericParameters) -> Result<EngineMethod, String> {
    builtin(
        "combinations", MethodStatus::Validated, "results/1",
        Runner::Structural(run::static_linear::combinations), run::static_linear::budget,
        "crates/engine-api/fixtures/input/valid/beam-combination.json",
        "provenance/ENG-API-001.md · Incremento 2a: superposición 1,2D+1,6L y restricciones de prescritos",
        "provenance/ENG-RES-001.md",
    )
}
fn pdelta(_: &NumericParameters) -> Result<EngineMethod, String> {
    builtin(
        "pDelta", MethodStatus::Stable, "engine-input/1",
        Runner::Structural(run::advanced::pdelta), run::advanced::pdelta_budget,
        "crates/engine-api/fixtures/input/valid/p-delta.json",
        "provenance/ENG-CORE-006.md · MAM-582: ENG-BENCH-002 (PR #132), provenance/ENG-BENCH-002.md; AISC 360-16 Comm. C2 casos 1 y 2; error ≤0.27%, tolerancias 3% M / 5% Δ; límites #78/#84",
        "provenance/ENG-ANL-003.md",
    )
}
fn modal(_: &NumericParameters) -> Result<EngineMethod, String> {
    builtin(
        "modal",
        MethodStatus::Validated,
        "engine-input/1",
        Runner::Structural(run::advanced::modal),
        run::advanced::modal_budget,
        "crates/engine-api/fixtures/input/valid/modal-single-dof.json",
        "provenance/ENG-API-001.md · Incremento 2c: oscilador axial, omega²=k/m y participación",
        "provenance/ENG-ANL-002.md",
    )
}
fn spectrum(_: &NumericParameters) -> Result<EngineMethod, String> {
    builtin(
        "responseSpectrum",
        MethodStatus::Validated,
        "engine-input/1",
        Runner::Structural(run::advanced::spectrum),
        run::advanced::spectrum_budget,
        "crates/engine-api/fixtures/input/valid/response-spectrum.json",
        "provenance/ENG-API-001.md · Incremento 2c: q=Gamma Sa/omega², u=Sa/omega² y V=m Sa",
        "provenance/ENG-ANL-004.md",
    )
}

fn nonlinear(_: &NumericParameters) -> Result<EngineMethod, String> {
    builtin(
        "nonlinearStatic",
        MethodStatus::Experimental,
        "engine-input/1",
        Runner::Structural(run::nonlinear::execute),
        run::nonlinear::budget,
        "crates/engine-api/fixtures/input/valid/sfc010-steel.json",
        "provenance/ENG-SFC-010.md · Equilibrio de resortes e historia cinemática",
        "provenance/ENG-SFC-010.md",
    )
}
fn soil(_: &NumericParameters) -> Result<EngineMethod, String> {
    let method = builtin(
        "soilProfile",
        MethodStatus::Experimental,
        "engine-input/1",
        Runner::Standalone(run::geo::execute),
        run::geo::budget,
        "crates/engine-api/fixtures/input/valid/sfc010-soil.json",
        "provenance/ENG-SFC-010.md · Perfil hidrostático estratificado",
        "provenance/ENG-GEO-001.md",
    )?;
    Ok(method)
}
fn consolidation(_: &NumericParameters) -> Result<EngineMethod, String> {
    let method = builtin(
        "consolidation1d",
        MethodStatus::Experimental,
        "engine-input/1",
        Runner::Standalone(run::geo::execute),
        run::geo::budget,
        "crates/engine-api/fixtures/input/valid/sfc010-consolidation.json",
        "provenance/ENG-SFC-010.md · Serie de Terzaghi U(Tv)",
        "provenance/ENG-GEO-006.md",
    )?;
    Ok(method)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experimental_policy_requires_opt_in_and_preserves_other_statuses() {
        let mut descriptor = pdelta(&NumericParameters::new()).unwrap().descriptor;
        descriptor.workflow.method_id = "testExperimental".into();
        descriptor.status = MethodStatus::Experimental;
        let error = enforce_policy(&descriptor, false).unwrap_err();
        assert!(matches!(error, ApiError::InvalidInput(_)));
        let cause = error.to_string();
        for expected in [
            "testExperimental",
            "Experimental",
            "options.allow_experimental = true",
        ] {
            assert!(cause.contains(expected), "{cause}");
        }
        assert!(enforce_policy(&descriptor, true).is_ok());
        for status in [
            MethodStatus::Stable,
            MethodStatus::Validated,
            MethodStatus::Deprecated,
        ] {
            descriptor.status = status;
            assert!(enforce_policy(&descriptor, false).is_ok());
            assert!(enforce_policy(&descriptor, true).is_ok());
        }
    }
}
