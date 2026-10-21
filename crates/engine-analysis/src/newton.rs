//! `static.newton`: Newton completo con factores de carga absolutos explícitos.
//!
//! K_total = K_lineal + Σ K_elemento(u, historia_aceptada). Los elementos
//! variables son aportes adicionales y NO deben incluirse también en K_lineal.
//! Cada evaluación usa u total (incluidos prescritos constantes) y la historia
//! del último paso aceptado. Solo al converger se hace commit de trial_state.
//! No hay búsqueda de línea, bisección, regularización ni reintentos.
//!
//! R = λ F − F_int se mide DESPUÉS de aplicar cada corrección. Δu es la última
//! corrección aplicada y la energía es |ΔuᵀR| con ese residual actualizado.
//! Todas las tolerancias activas deben cumplirse. Con fuerza sola, un sistema
//! lineal converge en una corrección; exigir además una corrección pequeña
//! puede requerir otra iteración. Las escalas SI son explícitas, constantes y
//! positivas; las normas mixtas de fuerzas/momentos y traslaciones/giros son
//! indicadores numéricos, no invariantes físicos frente a cambios de unidades.
//!
//! Ejemplo: resorte bilineal a tierra, 16 N, u final = 0.4 m.
//! ```
//! use engine_analysis::newton::{NewtonElement, NewtonOptions, NewtonSystem, StaticNewton};
//! use engine_assembly::LinearSystem;
//! use engine_contracts::{CaseRef, ElementCtx, ElementParams, Status};
//! use engine_core::{Dim, Dof, DofIndex, NodeId};
//! use engine_elements_spring::{NonlinearSpring, SpringAxes, SpringConnection};
//! use engine_laws::uniaxial::{MultilinearElastic, MultilinearMode};
//! use engine_linalg::ReferenceBackend;
//! use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node};
//! use std::sync::Arc;
//! let mut model = Model::new();
//! let mut node = Node::new(NodeId(1), Dim::D2, [0.0; 3]);
//! node.dofs = vec![Dof::Ux];
//! model.nodes.push(node);
//! model.cases.push(LoadCase { name: "P".into(), loads: vec![NodalLoad {
//!     node: NodeId(1), values: vec![DofValue { dof: Dof::Ux, value: 16.0 }],
//! }] });
//! let law = MultilinearElastic::new(
//!     vec![(0.0, 0.0), (0.1, 10.0), (0.5, 18.0)], MultilinearMode::Both,
//! )?;
//! let spring = NonlinearSpring::new(Dim::D2, SpringConnection::Ground,
//!     Arc::new(law), &[Dof::Ux], SpringAxes::Global)?;
//! let params = ElementParams::default();
//! let ctx = ElementCtx { dim: Dim::D2, coords: &[[0.0; 3]], params: &params };
//! let element = NewtonElement::new(Box::new(spring), vec![DofIndex(0)], &ctx)?;
//! let linear = LinearSystem::from_model(ReferenceBackend, &model, &[], &[])?;
//! let mut system = NewtonSystem::new(linear, vec![element])?;
//! let result = StaticNewton::run(&mut system, &CaseRef("P".into()),
//!     &[0.25, 0.5, 1.0], NewtonOptions::default())?;
//! assert_eq!(result.status, Status::Completed);
//! assert!((result.free_displacements[0] - 0.4).abs() < 1e-12);
//! # Ok::<(), engine_core::EngineError>(())
//! ```

use crate::finite;
use engine_assembly::{assemble::assemble_vector, LinearSystem};
use engine_contracts::{
    CaseRef, Element, ElementCtx, ElementParams, ElementResponse, Status, SystemView,
};
use engine_core::{ContractId, Dim, DofIndex, EResult, EngineError, Real};
use engine_linalg::{LinAlgBackend, SymMatrix};

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}

/// Elemento variable con contexto propietario e índices globales elementales.
/// El orden es nodo y luego dofs_per_node; se comprueba contra el mapa al unirlo.
pub struct NewtonElement {
    element: Box<dyn Element>,
    location: Vec<DofIndex>,
    dim: Dim,
    coords: Vec<[Real; 3]>,
    params: ElementParams,
}

impl NewtonElement {
    /// Copia el contexto SI y conserva el elemento puro, sin evaluar ni aceptar.
    /// ENG-E-0010 por contrato/tamaños vacíos; ENG-E-0008 por coordenadas no finitas.
    pub fn new(
        element: Box<dyn Element>,
        location: Vec<DofIndex>,
        ctx: &ElementCtx<'_>,
    ) -> EResult<Self> {
        let width = element.dofs_per_node().len();
        if !element
            .contract()
            .is_compatible_with(&ContractId::new("element", 1, 0))
            || width == 0
            || element.n_nodes() == 0
            || element.n_nodes().checked_mul(width) != Some(location.len())
            || ctx.coords.len() != element.n_nodes()
        {
            return Err(invalid("Elemento Newton: contrato o tamaños inválidos."));
        }
        for xyz in ctx.coords {
            finite(xyz)?;
        }
        Ok(Self {
            element,
            location,
            dim: ctx.dim,
            coords: ctx.coords.to_vec(),
            params: ctx.params.clone(),
        })
    }

    fn respond(&self, global: &[Real], state: &[Real]) -> EResult<ElementResponse> {
        let u = self
            .location
            .iter()
            .map(|i| global[i.0 as usize])
            .collect::<Vec<_>>();
        let response = self.element.respond(
            &ElementCtx {
                dim: self.dim,
                coords: &self.coords,
                params: &self.params,
            },
            &u,
            state,
        )?;
        if response.trial_state.len() != self.element.state_len() {
            return Err(invalid(
                "Elemento Newton: tamaño de historia de prueba inválido.",
            ));
        }
        finite(&response.trial_state)?;
        finite(response.tangent.values())?;
        Ok(response)
    }
}

/// Sistema con aportes lineales constantes y elementos variables adicionales.
/// Posee la historia; las evaluaciones nunca la mutan. El desplazamiento inicial
/// es el aceptado de LinearSystem y la historia inicial procede de cada elemento.
pub struct NewtonSystem<B> {
    linear: LinearSystem<B>,
    elements: Vec<NewtonElement>,
    accepted_states: Vec<Vec<Real>>,
    accepted_factor: Option<Real>,
}

struct Evaluation {
    tangent: SymMatrix,
    internal: Vec<Real>,
    states: Vec<Vec<Real>>,
}

impl<B: LinAlgBackend> NewtonSystem<B> {
    /// Une los aportes sin duplicar sus fuerzas/tangentes en el sistema lineal.
    /// Las cargas nodales y equivalentes se declaran en LinearSystem. Valida
    /// nodos/GDL/coordenadas, historia inicial y respuesta en u aceptado.
    /// Propaga los diagnósticos del elemento y del ensamblado sin aceptación.
    pub fn new(linear: LinearSystem<B>, elements: Vec<NewtonElement>) -> EResult<Self> {
        let mut states = Vec::new();
        for e in &elements {
            let mut zero = vec![0.0; linear.dof_map().len()];
            assemble_vector(&mut zero, &e.location, &vec![0.0; e.location.len()])?;
            let width = e.element.dofs_per_node().len();
            for (node_position, indices) in e.location.chunks_exact(width).enumerate() {
                let (node, _) = linear.dof_map().label(indices[0])?;
                if linear.dof_map().coordinates(node)? != e.coords[node_position] {
                    return Err(invalid("Elemento Newton: coordenadas distintas del mapa."));
                }
                for (&index, &dof) in indices.iter().zip(e.element.dofs_per_node()) {
                    if linear.dof_map().label(index)? != (node, dof) {
                        return Err(invalid("Elemento Newton: orden nodo/GDL inválido."));
                    }
                }
            }
            let state = e.element.initial_state();
            if state.len() != e.element.state_len() {
                return Err(invalid("Historia inicial elemental inválida."));
            }
            finite(&state)?;
            states.push(state);
        }
        let mut result = Self {
            linear,
            elements,
            accepted_states: states,
            accepted_factor: None,
        };
        let u = result.linear.accepted_displacement().to_vec();
        result.evaluate(&u, &mut || Ok(()))?;
        Ok(result)
    }

    /// Sistema lineal y sus desplazamientos aceptados/de prueba, solo lectura.
    pub fn linear_system(&self) -> &LinearSystem<B> {
        &self.linear
    }

    /// Historias del último paso aceptado, en orden de elementos variables.
    pub fn accepted_element_states(&self) -> &[Vec<Real>] {
        &self.accepted_states
    }

    /// Último factor absoluto aceptado por Newton; None antes del primer paso.
    pub fn accepted_factor(&self) -> Option<Real> {
        self.accepted_factor
    }

    fn evaluate(
        &mut self,
        free: &[Real],
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<Evaluation> {
        check()?;
        let global = self.linear.constraints().expand(free)?;
        let mut internal = self.linear.global_internal_force(free)?;
        let responses = self
            .elements
            .iter()
            .zip(&self.accepted_states)
            .map(|(e, state)| {
                check()?;
                e.respond(&global, state)
            })
            .collect::<EResult<Vec<_>>>()?;
        for (e, response) in self.elements.iter().zip(&responses) {
            check()?;
            assemble_vector(&mut internal, &e.location, &response.internal_force)?;
        }
        let blocks = self
            .elements
            .iter()
            .zip(&responses)
            .map(|(e, r)| (e.location.as_slice(), &r.tangent))
            .collect::<Vec<_>>();
        let tangent = self.linear.tangent_with_blocks(&blocks)?;
        Ok(Evaluation {
            tangent,
            internal,
            states: responses.into_iter().map(|r| r.trial_state).collect(),
        })
    }
}

/// Criterios independientes; None desactiva un criterio, nunca basta con uno
/// de los activos. Las tolerancias son relativas, finitas y estrictamente positivas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewtonOptions {
    /// Número máximo de correcciones/factorizaciones por paso; debe ser > 0.
    pub max_iterations: u32,
    /// Tolerancia para ‖R_f‖∞ / force_scale.
    pub force_tolerance: Option<Real>,
    /// Tolerancia para ‖Δu_f‖∞ / displacement_scale.
    pub displacement_tolerance: Option<Real>,
    /// Tolerancia para |Δu_fᵀR_f| / energy_scale.
    pub energy_tolerance: Option<Real>,
    /// Escala SI positiva de fuerza/momento, elegida por el llamador.
    pub force_scale: Real,
    /// Escala SI positiva de traslación/giro, elegida por el llamador.
    pub displacement_scale: Real,
    /// Escala SI positiva de trabajo, elegida por el llamador.
    pub energy_scale: Real,
}

impl Default for NewtonOptions {
    fn default() -> Self {
        Self {
            max_iterations: 25,
            force_tolerance: Some(1e-10),
            displacement_tolerance: None,
            energy_tolerance: None,
            force_scale: 1.0,
            displacement_scale: 1.0,
            energy_scale: 1.0,
        }
    }
}

impl NewtonOptions {
    fn validate(&self) -> EResult<()> {
        let tolerances = [
            self.force_tolerance,
            self.displacement_tolerance,
            self.energy_tolerance,
        ];
        let scales = [self.force_scale, self.displacement_scale, self.energy_scale];
        finite(&scales)?;
        for tolerance in tolerances.into_iter().flatten() {
            finite(&[tolerance])?;
        }
        if self.max_iterations == 0
            || tolerances.iter().all(Option::is_none)
            || scales.iter().any(|s| *s <= 0.0)
            || tolerances.into_iter().flatten().any(|t| t <= 0.0)
        {
            return Err(invalid("Newton: máximo, criterios o escalas inválidos."));
        }
        Ok(())
    }
}

/// Informe de un paso intentado; normas al final de su última corrección.
#[derive(Clone, Debug, PartialEq)]
pub struct NewtonStepReport {
    /// Factor absoluto solicitado; no es un incremento relativo al paso anterior.
    pub load_factor: Real,
    /// Correcciones ejecutadas; una factorización y resolución por corrección.
    pub iterations: u32,
    /// ‖R_f‖∞ en SI después de actualizar u.
    pub force_norm: Real,
    /// ‖Δu_f‖∞ de la última corrección en SI.
    pub displacement_norm: Real,
    /// |Δu_fᵀR_f| en SI con residual actualizado.
    pub energy_norm: Real,
    /// Norma de fuerza dividida por su escala.
    pub relative_force_norm: Real,
    /// Norma de desplazamiento dividida por su escala.
    pub relative_displacement_norm: Real,
    /// Norma de energía dividida por su escala.
    pub relative_energy_norm: Real,
    /// Todas las tolerancias activas satisfechas.
    pub converged: bool,
}

/// Resultado terminal; los vectores e historias siempre son los ACEPTADOS.
#[derive(Clone, Debug, PartialEq)]
pub struct NewtonResults {
    /// Completed o NotConverged; errores de infraestructura usan EResult.
    pub status: Status,
    /// Caso solicitado.
    pub case: CaseRef,
    /// Opciones y escalas efectivamente utilizadas.
    pub options: NewtonOptions,
    /// Intentos en orden, incluido el paso fallido; no se intenta ningún posterior.
    pub steps: Vec<NewtonStepReport>,
    /// Último factor aceptado, o None si ningún paso fue aceptado.
    pub accepted_factor: Option<Real>,
    /// Desplazamientos libres del último paso aceptado, m/rad.
    pub free_displacements: Vec<Real>,
    /// Desplazamientos globales aceptados con prescritos constantes, m/rad.
    pub global_displacements: Vec<Real>,
    /// Historias aceptadas, copia independiente en orden de elementos.
    pub element_states: Vec<Vec<Real>>,
}

/// Newton completo estático con control de carga y commit explícito por paso.
pub struct StaticNewton;

impl StaticNewton {
    /// Identificador del método.
    pub const KIND: &'static str = "static.newton";

    /// Ejecuta exactamente la lista de factores λ_k sobre un caso. Son factores
    /// absolutos: [0.25, 0.5, 1] termina en F_total, no en 1.75 F_total. Se
    /// permiten descarga, factores cero/negativos y repeticiones explícitas.
    /// Valida toda la lista antes de aceptar. ENG-E-0010 por lista vacía/opciones,
    /// ENG-E-0008 por no finitos, ENG-E-0006 por tangente singular con nodo/GDL.
    /// Ante no convergencia o error descarta pruebas y conserva u/historia del
    /// último paso aceptado, sin reintentos. No implementa la fachada analysis/1.
    pub fn run<B: LinAlgBackend>(
        system: &mut NewtonSystem<B>,
        case: &CaseRef,
        factors: &[Real],
        options: NewtonOptions,
    ) -> EResult<NewtonResults> {
        Self::run_with(system, case, factors, options, || Ok(()))
    }

    /// Variante cooperativa: consulta antes de cada paso, evaluación e iteración
    /// y antes de aceptar estado. Ante error conserva el último paso aceptado
    /// y revierte el estado de prueba, igual que [`Self::run`].
    pub fn run_with<B: LinAlgBackend>(
        system: &mut NewtonSystem<B>,
        case: &CaseRef,
        factors: &[Real],
        options: NewtonOptions,
        mut check: impl FnMut() -> EResult<()>,
    ) -> EResult<NewtonResults> {
        let result = Self::run_inner(system, case, factors, options, &mut check);
        system.linear.rollback();
        result
    }

    fn run_inner<B: LinAlgBackend>(
        system: &mut NewtonSystem<B>,
        case: &CaseRef,
        factors: &[Real],
        options: NewtonOptions,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<NewtonResults> {
        options.validate()?;
        if factors.is_empty() {
            return Err(invalid("Newton: lista de factores vacía."));
        }
        finite(factors)?;
        // Valida también los productos λF antes del primer commit.
        for &factor in factors {
            check()?;
            system.linear.global_force(case, factor)?;
        }
        let mut steps = Vec::new();
        let mut status = Status::Completed;
        for &factor in factors {
            check()?;
            let force = system.linear.global_force(case, factor)?;
            let mut u = system.linear.accepted_displacement().to_vec();
            let mut evaluation = system.evaluate(&u, check)?;
            let mut residual = free_residual(system, &force, &evaluation.internal)?;
            for iteration in 1..=options.max_iterations {
                check()?;
                let delta = system.linear.solve_correction_with_check(
                    &evaluation.tangent,
                    &residual,
                    check,
                )?;
                for (value, correction) in u.iter_mut().zip(&delta) {
                    *value += correction;
                }
                finite(&u)?;
                evaluation = system.evaluate(&u, check)?;
                residual = free_residual(system, &force, &evaluation.internal)?;
                let report = report(factor, iteration, &residual, &delta, options)?;
                if report.converged {
                    check()?;
                    // Toda la respuesta y la historia se validaron en exactamente u.
                    // accept lineal es fallible/atómico; después solo se mueven buffers.
                    system.linear.accept(&u)?;
                    system.accepted_states = evaluation.states;
                    system.accepted_factor = Some(factor);
                    steps.push(report);
                    break;
                }
                if iteration == options.max_iterations {
                    steps.push(report);
                    status = Status::NotConverged;
                    break;
                }
            }
            if status == Status::NotConverged {
                break;
            }
        }
        let free_displacements = system.linear.accepted_displacement().to_vec();
        Ok(NewtonResults {
            status,
            case: case.clone(),
            options,
            steps,
            accepted_factor: system.accepted_factor,
            global_displacements: system.linear.constraints().expand(&free_displacements)?,
            free_displacements,
            element_states: system.accepted_states.clone(),
        })
    }
}

fn free_residual<B: LinAlgBackend>(
    system: &NewtonSystem<B>,
    force: &[Real],
    internal: &[Real],
) -> EResult<Vec<Real>> {
    let result = system
        .linear
        .constraints()
        .free_dofs()
        .iter()
        .map(|i| force[i.0 as usize] - internal[i.0 as usize])
        .collect::<Vec<_>>();
    finite(&result)?;
    Ok(result)
}

fn report(
    factor: Real,
    iterations: u32,
    residual: &[Real],
    delta: &[Real],
    options: NewtonOptions,
) -> EResult<NewtonStepReport> {
    let norm = |v: &[Real]| v.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    let mut work = 0.0;
    for (&d, &r) in delta.iter().zip(residual) {
        let term = d * r;
        finite(&[term])?;
        work += term;
        finite(&[work])?;
    }
    let force_norm = norm(residual);
    let displacement_norm = norm(delta);
    let energy_norm = work.abs();
    let relative_force_norm = force_norm / options.force_scale;
    let relative_displacement_norm = displacement_norm / options.displacement_scale;
    let relative_energy_norm = energy_norm / options.energy_scale;
    finite(&[
        relative_force_norm,
        relative_displacement_norm,
        relative_energy_norm,
    ])?;
    let satisfied = |t: Option<Real>, value: Real| t.is_none_or(|tol| value <= tol);
    let converged = satisfied(options.force_tolerance, relative_force_norm)
        && satisfied(options.displacement_tolerance, relative_displacement_norm)
        && satisfied(options.energy_tolerance, relative_energy_norm);
    Ok(NewtonStepReport {
        load_factor: factor,
        iterations,
        force_norm,
        displacement_norm,
        energy_norm,
        relative_force_norm,
        relative_displacement_norm,
        relative_energy_norm,
        converged,
    })
}
