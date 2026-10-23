//! Annotated templates for the eight traits; these are not validated production methods.
//!
//! Derivación propia: `σ=Eε`, `[N,Mz]=[EAε0,EIκz]`, barra k=EA/L y
//! F=k b(bᵀu), b=[-nx,-ny,0,nx,ny,0]. Véase provenance/ENG-CORE-007.md.
//! Each statement comments on its function; the closed-form algebra of these examples
//! does not constitute a backend or solver.
//!
//! ```
//! use engine_contracts::*;
//! use engine_contracts::examples::ElasticSection2D;
//! let s = ElasticSection2D::new(20.0, 3.0)?;
//! let r = s.respond(&[0.1, 0.2], &[])?;
//! assert_eq!(r.resultants[0], 2.0);
//! assert!((r.resultants[1] - 0.6).abs() < 1e-12);
//! # Ok::<(), EngineError>(())
//! ```
use crate::*; // Reuses contract and core types.
use engine_core::math;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
}; // Observable control without an external runtime.

fn positive(value: Real) -> EResult<Real> {
    finite(&[value])?; // Rejects NaN and infinities before comparing.
    if value <= 0.0 {
        return Err(invalid("A positive parameter is required."));
    } // Requires physical stiffness.
    Ok(value) // Preserves the validated SI magnitude.
}
fn vector(values: &[Real], len: usize) -> EResult<()> {
    size(values.len(), len)?; // Checks exact contract length.
    finite(values) // Checks all components.
}

/// Linear material without history, σ=Eε, finite strain range whose product
/// is finite; positive E in Pa. Does not represent plasticity or preload.
pub struct LinearElastic {
    modulus: Real,
}
impl LinearElastic {
    /// Validates E > 0 Pa; ENG-E-0010 for range and ENG-E-0008 for nonfinite values.
    pub fn new(modulus: Real) -> EResult<Self> {
        Ok(Self {
            modulus: positive(modulus)?,
        }) // Stores only validated E.
    }
}
impl UniaxialLaw for LinearElastic {
    fn kind(&self) -> &'static str {
        "example.linear_elastic"
    } // Custom template name.
    fn state_len(&self) -> usize {
        0
    } // Elasticity has no memory.
    fn initial_state(&self) -> Vec<Real> {
        vec![]
    } // Empty history at the origin.
    fn initial_tangent(&self) -> Real {
        self.modulus
    } // Constant dσ/dε=E.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 0)?; // Prevents incompatible history.
        finite(&[strain])?; // Validates total strain.
        let stress = self.modulus * strain; // Applies Hooke's closed-form solution.
        finite(&[stress])?; // Reports overflow as an error.
        Ok(UniaxialResponse {
            stress,
            tangent: self.modulus,
            trial_state: vec![],
        }) // Does not commit state.
    }
}

/// Elastic D2 section: N=EA ε0 and Mz=EI κz, without coupling or history.
/// Positive stiffnesses EA in N, EI in N·m²; finite product range.
pub struct ElasticSection2D {
    axial: Real,
    bending: Real,
}
impl ElasticSection2D {
    /// Validates positive EA and EI; range/nonfinite errors as in LinearElastic.
    pub fn new(axial: Real, bending: Real) -> EResult<Self> {
        Ok(Self {
            axial: positive(axial)?,
            bending: positive(bending)?,
        }) // Stores both SI stiffnesses.
    }
}
impl SectionLaw for ElasticSection2D {
    fn kind(&self) -> &'static str {
        "example.elastic_section2d"
    } // Sample section name.
    fn contract(&self) -> ContractId {
        ContractId::new("section", 1, 0)
    } // Declares v1 compatibility.
    fn dim(&self) -> Dim {
        Dim::D2
    } // Two generalized strains.
    fn state_len(&self) -> usize {
        0
    } // No material memory.
    fn initial_state(&self) -> Vec<Real> {
        vec![]
    } // Origin without history.
    fn respond(&self, e: &[Real], committed: &[Real]) -> EResult<SectionResponse> {
        vector(e, 2)?; // Expects ε0 and κz.
        vector(committed, 0)?; // Rejects excess history.
        let resultants = vec![self.axial * e[0], self.bending * e[1]]; // Computes N and Mz.
        finite(&resultants)?; // Avoids publishing overflows.
        let tangent = DMat::new(2, 2, vec![self.axial, 0.0, 0.0, self.bending])?; // Consistent diagonal derivative.
        Ok(SectionResponse {
            resultants,
            tangent,
            trial_state: vec![],
        }) // Returns a pure trial.
    }
}

/// Barra D2 de dos nodos, desplazamientos pequeños y sin pretensión.
///
/// E Pa, A m², densidad kg/m³ no negativa; k=EA/L. Incluye Ux,Uy,Rz
/// (Rz tiene rigidez y masa nulas). Solo cargas uniformes globales paralelas
/// al eje de la barra, sin momentos. Masa de traslación isotrópica. No ofrece Kg.
/// Las propiedades están en self; este ejemplo no requiere campos de ctx.params.
/// Longitud > 1e-12 m, productos finitos; no modela giros finitos ni flexión.
pub struct LinearBar2D {
    modulus: Real,
    area: Real,
    density: Real,
}
impl LinearBar2D {
    /// Valida E,A positivos y densidad no negativa; ENG-E-0010/ENG-E-0008.
    pub fn new(modulus: Real, area: Real, density: Real) -> EResult<Self> {
        finite(&[density])?; // Valida la masa volumétrica.
        if density < 0.0 {
            return Err(invalid("Negative density."));
        } // Rechaza masa negativa.
        Ok(Self {
            modulus: positive(modulus)?,
            area: positive(area)?,
            density,
        }) // Guarda propiedades SI.
    }
    fn geometry(&self, ctx: &ElementCtx<'_>) -> EResult<(Real, [Real; 2])> {
        if ctx.dim != Dim::D2 {
            return Err(invalid("Example truss requires D2."));
        } // Limita plano.
        size(ctx.coords.len(), 2)?; // Exige los dos extremos.
        for p in ctx.coords {
            finite(p)?; // Valida las tres coordenadas.
            if p[2] != 0.0 {
                return Err(invalid("D2 requires Z=0."));
            } // Respeta el plano aprobado.
        }
        let dx = ctx.coords[1][0] - ctx.coords[0][0]; // Diferencia X de i a j.
        let dy = ctx.coords[1][1] - ctx.coords[0][1]; // Diferencia Y de i a j.
        let length = math::hypot(dx, dy); // Longitud sin elevar coordenadas grandes al cuadrado.
        finite(&[length])?; // Rechaza geometría desbordada.
        if length <= 1e-12 {
            return Err(EngineError::new("ENG-E-0005", "Zero-length truss."));
        } // Detecta degeneración.
        Ok((length, [dx / length, dy / length])) // Devuelve L y dirección local x.
    }
    fn axial_load(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Real> {
        let (_, n) = self.geometry(ctx)?; // Obtiene dirección axial válida.
        finite(&load.force_per_length)?; // Valida carga distribuida.
        finite(&load.moment_per_length)?; // Valida momentos aunque no se admitan.
        let q = n[0] * load.force_per_length[0] + n[1] * load.force_per_length[1]; // Proyecta la fuerza axial.
        let transverse = -n[1] * load.force_per_length[0] + n[0] * load.force_per_length[1]; // Mide parte no axial.
        finite(&[q, transverse])?; // Impide proyecciones desbordadas.
        if transverse.abs() > 1e-12 * q.abs().max(1.0)
            || load.force_per_length[2] != 0.0
            || load.moment_per_length != [0.0; 3]
        {
            return Err(invalid(
                "Truss supports only uniform axial load without moments.",
            )); // Evita omitir cargas.
        }
        Ok(q) // Intensidad axial SI con signo.
    }
}
impl Element for LinearBar2D {
    fn kind(&self) -> &'static str {
        "example.linear_bar2d"
    } // Identificador propio.
    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    } // Contrato de elemento v1.
    fn n_nodes(&self) -> usize {
        2
    } // Extremos i y j.
    fn dofs_per_node(&self) -> &'static [Dof] {
        Dim::D2.dofs()
    } // Ux,Uy,Rz por nodo.
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        vector(u, 6)?; // Verifica desplazamientos/giros totales.
        vector(committed, 0)?; // Este ejemplo no tiene historia.
        let (length, n) = self.geometry(ctx)?; // Geometría de referencia.
        let b = [-n[0], -n[1], 0.0, n[0], n[1], 0.0]; // Operador de elongación axial.
        let k = self.modulus * self.area / length; // Rigidez analítica EA/L.
        let extension: Real = b.iter().zip(u).map(|(bi, ui)| bi * ui).sum(); // Elongación lineal bᵀu.
        let axial = k * extension; // Tracción N positiva.
        let internal_force: Vec<_> = b.iter().map(|bi| axial * bi).collect(); // Acciones globales en caras.
        finite(&internal_force)?; // Verifica salida finita.
        let values = b
            .iter()
            .flat_map(|bi| b.iter().map(move |bj| k * bi * bj))
            .collect(); // Derivada cerrada k bbᵀ.
        let tangent = DMat::new(6, 6, values)?; // Almacena sin backend numérico.
        Ok(ElementResponse {
            internal_force,
            tangent,
            trial_state: vec![],
        }) // Retorna prueba sin mutar nada.
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        let (length, _) = self.geometry(ctx)?; // Valida la geometría incluso sin masa.
        if self.density == 0.0 {
            return Ok(None);
        } // Ausencia física explícita.
        let total = self.density * self.area * length; // Masa total del miembro.
        let mut values = vec![0.0; 36]; // Reserva el bloque denso de elemento.
        for axis in 0..2 {
            for a in 0..2 {
                for b in 0..2 {
                    let coefficient = match kind {
                        MassKind::Lumped => {
                            if a == b {
                                0.5
                            } else {
                                0.0
                            }
                        }
                        MassKind::Consistent => {
                            if a == b {
                                1.0 / 3.0
                            } else {
                                1.0 / 6.0
                            }
                        }
                    }; // Integra formas lineales o concentra.
                    values[(3 * a + axis) * 6 + 3 * b + axis] = total * coefficient;
                    // Cada traslación conserva masa total.
                }
            }
        }
        Ok(Some(DMat::new(6, 6, values)?)) // Matriz simétrica, giros nulos.
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        self.respond(ctx, u, committed)?; // Valida contexto y estado sin aceptarlos.
        Ok(None) // La barra lineal de muestra no ofrece segundo orden.
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        let (length, n) = self.geometry(ctx)?; // Longitud y orientación global.
        let q = self.axial_load(ctx, load)?; // Rechaza cargas no admitidas.
        let f = q * length / 2.0; // Cada extremo recibe la mitad de la carga uniforme.
        let out = vec![f * n[0], f * n[1], 0.0, f * n[0], f * n[1], 0.0]; // Vector nodal global.
        finite(&out)?; // No publica fuerzas desbordadas.
        Ok(out) // Orden nodo y después GDL.
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput> {
        let response = self.respond(ctx, u, committed)?; // Calcula respuesta sin aceptar.
        let (length, n) = self.geometry(ctx)?; // Eje local y longitud.
        finite(stations)?; // Rechaza posiciones no finitas.
        if stations.iter().any(|x| !(0.0..=1.0).contains(x)) {
            return Err(invalid("Station is outside [0,1]."));
        } // Limita tramo.
        let axial = n[0] * response.internal_force[3] + n[1] * response.internal_force[4]; // N elástico por cara j.
        let mut q = 0.0; // Acumula únicamente cargas admitidas.
        for load in loads {
            q += self.axial_load(ctx, load)?;
        } // Suma intensidades axiales.
        let end_forces_local = vec![
            -axial - q * length / 2.0,
            0.0,
            0.0,
            axial - q * length / 2.0,
            0.0,
            0.0,
        ]; // Resta cargas equivalentes.
        finite(&end_forces_local)?; // Verifica acciones de extremo.
        let stations: Vec<_> = stations
            .iter()
            .map(|x| StationResult {
                position: *x,
                forces_local: [axial + q * length * (0.5 - x), 0.0, 0.0, 0.0, 0.0, 0.0],
            })
            .collect(); // Equilibrio dN/dx=-q.
        for station in &stations {
            finite(&station.forces_local)?;
        } // Verifica cada sección.
        Ok(ElementOutput {
            end_forces_local,
            stations,
        }) // Conserva orden solicitado.
    }
}

/// Fábrica de LinearBar2D; propiedades numéricas E, A, density, sin referencias.
pub struct BarFactory;
impl ElementFactory for BarFactory {
    fn kind(&self) -> &'static str {
        "example.linear_bar2d"
    } // Coincide con producto.
    fn params_schema(&self) -> String {
        r#"{"type":"object","properties":{"E":{"type":"number","exclusiveMinimum":0},"A":{"type":"number","exclusiveMinimum":0},"density":{"type":"number","minimum":0}},"required":["E","A","density"],"additionalProperties":false}"#.into()
        // Esquema textual propio, E Pa, A m², density kg/m³.
    }
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>> {
        if model.dim != Dim::D2 {
            return Err(invalid("Factory requires D2."));
        } // Valida capacidad geométrica.
        let JsonValue::Object(fields) = params else {
            return Err(invalid("Expected a properties object."));
        }; // Comprueba tipo JSON.
        size(fields.len(), 3)?; // Rechaza propiedades adicionales.
        let number = |key| -> EResult<Real> {
            match fields.get(key) {
                Some(JsonValue::Number(v)) => Ok(*v),
                _ => Err(invalid("Missing required numeric property.")),
            }
        }; // Extrae solo números.
        Ok(Box::new(LinearBar2D::new(
            number("E")?,
            number("A")?,
            number("density")?,
        )?)) // Valida rango y construye objeto propietario.
    }
}

/// Mock system with a single Ux DOF at node 1: F=k u, mass=1 kg, Kg=0.
/// Supported case "example.case", unit force; observable committed state in m.
pub struct SpringSystem {
    stiffness: Real,
    accepted: Real,
    pattern: SparsityPattern,
}
impl SpringSystem {
    /// Constructs with positive k in N/m; range/nonfinite errors.
    pub fn new(stiffness: Real) -> EResult<Self> {
        Ok(Self {
            stiffness: positive(stiffness)?,
            accepted: 0.0,
            pattern: SparsityPattern::new(1, vec![true])?,
        }) // Creates a scalar equation without solving it.
    }
    /// Last committed displacement in m, for observing the mock.
    pub fn accepted(&self) -> Real {
        self.accepted
    } // Exposes explicit acceptance.
}
impl SystemView for SpringSystem {
    fn n_free_dofs(&self) -> usize {
        1
    } // One free equation.
    fn pattern(&self) -> &SparsityPattern {
        &self.pattern
    } // Borrows a fixed pattern.
    fn tangent(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()> {
        vector(u, 1)?; // Validates total state.
        size(out.order(), 1)?; // Avoids replacing an incompatible size.
        *out = SymMatrix::new(DMat::new(1, 1, vec![self.stiffness])?)?; // Overwrites with k.
        Ok(()) // No implicit acceptance.
    }
    fn internal_force(&mut self, u: &[Real], out: &mut [Real]) -> EResult<()> {
        vector(u, 1)?; // Validates input.
        size(out.len(), 1)?; // Validates output before writing.
        let f = self.stiffness * u[0]; // Analytical spring force.
        finite(&[f])?; // Prevents partial writes on overflow.
        out[0] = f; // Overwrites the single force.
        Ok(()) // Committed state intact.
    }
    fn mass(&mut self, _: MassKind, out: &mut SymMatrix) -> EResult<()> {
        size(out.order(), 1)?; // Validates order.
        *out = SymMatrix::new(DMat::new(1, 1, vec![1.0])?)?; // Unit mass in kg.
        Ok(()) // Same mass in both distributions.
    }
    fn geometric(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()> {
        vector(u, 1)?; // Validates displacement.
        size(out.order(), 1)?; // Validates order.
        *out = SymMatrix::zeros(1)?; // Zero geometric stiffness.
        Ok(()) // Does not offer second order.
    }
    fn external_force(&self, case: &CaseRef, factor: Real, out: &mut [Real]) -> EResult<()> {
        size(out.len(), 1)?; // Checks output.
        finite(&[factor])?; // Checks dimensionless factor.
        if case.0 != "example.case" {
            return Err(EngineError::new("ENG-E-0003", "Missing case."));
        } // Rejects unknown case.
        out[0] = factor; // Base force of 1 N times factor.
        Ok(()) // No effect on history.
    }
    fn accept(&mut self, u: &[Real]) -> EResult<()> {
        let mut force = [0.0]; // Prepares atomic reevaluation.
        self.internal_force(u, &mut force)?; // Validates exactly u and its response.
        self.accepted = u[0]; // Commits only after success.
        Ok(()) // New explicitly committed point.
    }
    fn rollback(&mut self) {} // Does not store trials: preserving accepted suffices.
    fn dof_label(&self, d: DofIndex) -> (NodeId, Dof) {
        assert_eq!(d.0, 0, "Índice fuera del sistema de ejemplo"); // Applies documented precondition.
        (NodeId(1), Dof::Ux) // Maps equation to node/component.
    }
}

/// In-memory mock sink, preserves repeated writes in arrival order.
#[derive(Default)]
pub struct MemorySink {
    /// Emitted entries, without deduplication or serialization.
    pub entries: Vec<(ResultKey, ResultValue)>,
}
impl ResultSink for MemorySink {
    fn put(&mut self, key: ResultKey, value: ResultValue) -> EResult<()> {
        conformance::validate_result(&key, &value)?; // Validates the entire input before mutating.
        self.entries.push((key, value)); // Atomic and ordered in-memory write.
        Ok(()) // Confirms receipt.
    }
}

/// Observable mock control: atomic cancellation and mutex-protected progress.
#[derive(Default)]
pub struct RecordingControl {
    cancelled: AtomicBool,
    events: Mutex<Vec<(Real, String)>>,
}
impl RecordingControl {
    /// Persistently requests cooperative cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    } // Only writes the signal.
    /// Copies progress events; recovers contents if an observer poisoned the mutex.
    pub fn events(&self) -> Vec<(Real, String)> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone() // Reads without altering computation.
    }
}
impl RunControl for RecordingControl {
    fn progress(&self, fraction: Real, note: &str) {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push((fraction, note.into())); // Stores a valid event sent by the method.
    }
    fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    } // Observes request without consuming it.
}

/// Mock for committing prescribed displacements; does NOT solve equilibrium.
/// Settings: SI array of n_free_dofs displacements/rotations. One step without iterations;
/// zero norms indicate no iterative correction was performed. Tolerances 1e-8.
pub struct PrescribedStep;
impl AnalysisMethod for PrescribedStep {
    fn id(&self) -> &'static str {
        "example.prescribed_step"
    } // Template identifier.
    fn version(&self) -> (u16, u16, u16) {
        (1, 0, 0)
    } // Explicit semver.
    fn contract(&self) -> ContractId {
        ContractId::new("analysis", 1, 0)
    } // V1 contract.
    fn settings_schema(&self) -> String {
        r#"{"type":"array","items":{"type":"number"}}"#.into()
    } // Length depends on the system.
    fn requires(&self) -> Requirements {
        Requirements::default()
    } // Requires neither mass nor Kg.
    fn run(
        &self,
        sys: &mut dyn SystemView,
        settings: &JsonValue,
        sink: &mut dyn ResultSink,
        ctl: &dyn RunControl,
    ) -> EResult<AnalysisOutcome> {
        let execute =
            |sys: &mut dyn SystemView, sink: &mut dyn ResultSink| -> EResult<AnalysisOutcome> {
                if ctl.cancelled() {
                    return Ok(cancelled());
                } // Handles cancellation before evaluating.
                let JsonValue::Array(values) = settings else {
                    return Err(invalid("Expected a displacement array."));
                }; // Validates type.
                let u: EResult<Vec<_>> = values
                    .iter()
                    .map(|v| match v {
                        JsonValue::Number(x) => Ok(*x),
                        _ => Err(invalid("Nonnumeric displacement.")),
                    })
                    .collect(); // Extracts SI numbers.
                let u = u?; // Propagates invalid fields.
                vector(&u, sys.n_free_dofs())?; // Checks length and finiteness.
                ctl.progress(0.0, "Evaluating prescribed displacements"); // Initial progress.
                let mut force = vec![0.0; u.len()]; // Prepares force output.
                sys.internal_force(&u, &mut force)?; // Evaluates a trial without committing it.
                finite(&force)?; // Rejects faulty system output.
                if ctl.cancelled() {
                    return Ok(cancelled());
                } // Avoids committing after cancellation.
                sys.accept(&u)?; // Sets exactly the prescribed displacement before publishing it.
                sink.put(
                    ResultKey {
                        block: "cases".into(),
                        entity: Some("example.prescribed".into()),
                        field: "displacements".into(),
                    },
                    ResultValue::Vector(u.clone()),
                )?; // Emits already committed displacements.
                let report = ConvergenceReport {
                    steps: vec![StepReport {
                        step: 1,
                        load_factor: 1.0,
                        iterations: 0,
                        residual_norm: 0.0,
                        displacement_norm: 0.0,
                        energy_norm: 0.0,
                        converged: true,
                        tolerances: Tolerances {
                            force_rel: 1e-8,
                            disp_rel: 1e-8,
                            energy_rel: 1e-8,
                        },
                    }],
                }; // Declares mock norms and criteria.
                ctl.progress(1.0, "Prescribed step accepted"); // Completes monotonic progress.
                Ok(AnalysisOutcome {
                    status: Status::Completed,
                    report,
                    diagnostics: vec![],
                }) // Explicit terminal result.
            };
        let outcome = execute(sys, sink); // Executes a single declared strategy.
        if outcome.as_ref().is_err()
            || outcome
                .as_ref()
                .is_ok_and(|o| o.status == Status::Cancelled)
        {
            sys.rollback();
        } // Discards trials on failure or cancellation.
        outcome // Propagates outcome or error without retries.
    }
}
fn cancelled() -> AnalysisOutcome {
    AnalysisOutcome {
        status: Status::Cancelled,
        report: ConvergenceReport::default(),
        diagnostics: vec![],
    } // Does not publish an uncommitted step.
}
