//! Plantillas comentadas de los ocho traits; no son métodos productivos validados.
//!
//! Derivación propia: `σ=Eε`, `[N,Mz]=[EAε0,EIκz]`, barra k=EA/L y
//! F=k b(bᵀu), b=[-nx,-ny,0,nx,ny,0]. Véase provenance/ENG-CORE-007.md.
//! Cada instrucción comenta su función; el álgebra cerrada de estos ejemplos
//! no constituye un backend ni un solver.
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
use crate::*; // Reutiliza tipos de contrato y núcleo.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
}; // Control observable sin runtime externo.

fn positive(value: Real) -> EResult<Real> {
    finite(&[value])?; // Rechaza NaN e infinitos antes de comparar.
    if value <= 0.0 {
        return Err(invalid("Se requiere un parámetro positivo."));
    } // Exige rigidez física.
    Ok(value) // Conserva la magnitud SI validada.
}
fn vector(values: &[Real], len: usize) -> EResult<()> {
    size(values.len(), len)?; // Verifica longitud exacta del contrato.
    finite(values) // Verifica todas las componentes.
}

/// Material lineal sin historia, σ=Eε, rango de deformación finita cuyo producto
/// sea finito; E positivo en Pa. No representa plasticidad ni precarga.
pub struct LinearElastic {
    modulus: Real,
}
impl LinearElastic {
    /// Valida E > 0 Pa; ENG-E-0010 por rango y ENG-E-0008 por no finito.
    pub fn new(modulus: Real) -> EResult<Self> {
        Ok(Self {
            modulus: positive(modulus)?,
        }) // Guarda solo E validado.
    }
}
impl UniaxialLaw for LinearElastic {
    fn kind(&self) -> &'static str {
        "example.linear_elastic"
    } // Nombre propio de plantilla.
    fn state_len(&self) -> usize {
        0
    } // La elasticidad no tiene memoria.
    fn initial_state(&self) -> Vec<Real> {
        vec![]
    } // Historia vacía del origen.
    fn initial_tangent(&self) -> Real {
        self.modulus
    } // dσ/dε=E constante.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 0)?; // Impide historia incompatible.
        finite(&[strain])?; // Valida deformación total.
        let stress = self.modulus * strain; // Aplica la solución cerrada de Hooke.
        finite(&[stress])?; // Informa desbordamiento como error.
        Ok(UniaxialResponse {
            stress,
            tangent: self.modulus,
            trial_state: vec![],
        }) // No acepta estado.
    }
}

/// Sección elástica D2: N=EA ε0 y Mz=EI κz, sin acoplamiento ni historia.
/// Rigideces positivas EA en N, EI en N·m²; rango de productos finitos.
pub struct ElasticSection2D {
    axial: Real,
    bending: Real,
}
impl ElasticSection2D {
    /// Valida EA y EI positivos; errores de rango/no finitos como LinearElastic.
    pub fn new(axial: Real, bending: Real) -> EResult<Self> {
        Ok(Self {
            axial: positive(axial)?,
            bending: positive(bending)?,
        }) // Almacena las dos rigideces SI.
    }
}
impl SectionLaw for ElasticSection2D {
    fn kind(&self) -> &'static str {
        "example.elastic_section2d"
    } // Nombre de la sección de muestra.
    fn contract(&self) -> ContractId {
        ContractId::new("section", 1, 0)
    } // Declara compatibilidad v1.
    fn dim(&self) -> Dim {
        Dim::D2
    } // Dos deformaciones generalizadas.
    fn state_len(&self) -> usize {
        0
    } // Sin memoria material.
    fn initial_state(&self) -> Vec<Real> {
        vec![]
    } // Origen sin historia.
    fn respond(&self, e: &[Real], committed: &[Real]) -> EResult<SectionResponse> {
        vector(e, 2)?; // Espera ε0 y κz.
        vector(committed, 0)?; // Rechaza historia sobrante.
        let resultants = vec![self.axial * e[0], self.bending * e[1]]; // Calcula N y Mz.
        finite(&resultants)?; // Evita publicar desbordamientos.
        let tangent = DMat::new(2, 2, vec![self.axial, 0.0, 0.0, self.bending])?; // Derivada diagonal consistente.
        Ok(SectionResponse {
            resultants,
            tangent,
            trial_state: vec![],
        }) // Devuelve prueba pura.
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
            return Err(invalid("Densidad negativa."));
        } // Rechaza masa negativa.
        Ok(Self {
            modulus: positive(modulus)?,
            area: positive(area)?,
            density,
        }) // Guarda propiedades SI.
    }
    fn geometry(&self, ctx: &ElementCtx<'_>) -> EResult<(Real, [Real; 2])> {
        if ctx.dim != Dim::D2 {
            return Err(invalid("La barra de ejemplo requiere D2."));
        } // Limita plano.
        size(ctx.coords.len(), 2)?; // Exige los dos extremos.
        for p in ctx.coords {
            finite(p)?; // Valida las tres coordenadas.
            if p[2] != 0.0 {
                return Err(invalid("D2 requiere Z=0."));
            } // Respeta el plano aprobado.
        }
        let dx = ctx.coords[1][0] - ctx.coords[0][0]; // Diferencia X de i a j.
        let dy = ctx.coords[1][1] - ctx.coords[0][1]; // Diferencia Y de i a j.
        let length = dx.hypot(dy); // Longitud sin elevar coordenadas grandes al cuadrado.
        finite(&[length])?; // Rechaza geometría desbordada.
        if length <= 1e-12 {
            return Err(EngineError::new("ENG-E-0005", "Barra de longitud nula."));
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
                "La barra solo admite carga uniforme axial sin momentos.",
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
        let mut values = vec![0.0; 36]; // Reserva el bloque denso provisional.
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
            return Err(invalid("Estación fuera de [0,1]."));
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
            return Err(invalid("La fábrica requiere D2."));
        } // Valida capacidad geométrica.
        let JsonValue::Object(fields) = params else {
            return Err(invalid("Se esperaba objeto de propiedades."));
        }; // Comprueba tipo JSON.
        size(fields.len(), 3)?; // Rechaza propiedades adicionales.
        let number = |key| -> EResult<Real> {
            match fields.get(key) {
                Some(JsonValue::Number(v)) => Ok(*v),
                _ => Err(invalid("Propiedad numérica requerida ausente.")),
            }
        }; // Extrae solo números.
        Ok(Box::new(LinearBar2D::new(
            number("E")?,
            number("A")?,
            number("density")?,
        )?)) // Valida rango y construye objeto propietario.
    }
}

/// Sistema mock de un único GDL Ux del nodo 1: F=k u, masa=1 kg, Kg=0.
/// Caso admitido "example.case", fuerza unitaria; aceptado observable en m.
pub struct SpringSystem {
    stiffness: Real,
    accepted: Real,
    pattern: SparsityPattern,
}
impl SpringSystem {
    /// Construye con k positivo N/m; errores de rango/no finitos.
    pub fn new(stiffness: Real) -> EResult<Self> {
        Ok(Self {
            stiffness: positive(stiffness)?,
            accepted: 0.0,
            pattern: SparsityPattern::new(1, vec![true])?,
        }) // Crea ecuación escalar sin resolverla.
    }
    /// Último desplazamiento aceptado m, para observar el mock.
    pub fn accepted(&self) -> Real {
        self.accepted
    } // Expone aceptación explícita.
}
impl SystemView for SpringSystem {
    fn n_free_dofs(&self) -> usize {
        1
    } // Una ecuación libre.
    fn pattern(&self) -> &SparsityPattern {
        &self.pattern
    } // Presta patrón fijo.
    fn tangent(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()> {
        vector(u, 1)?; // Valida estado total.
        size(out.order(), 1)?; // Evita sustituir tamaño incompatible.
        *out = SymMatrix::new(DMat::new(1, 1, vec![self.stiffness])?)?; // Sobrescribe con k.
        Ok(()) // Sin aceptación implícita.
    }
    fn internal_force(&mut self, u: &[Real], out: &mut [Real]) -> EResult<()> {
        vector(u, 1)?; // Valida entrada.
        size(out.len(), 1)?; // Valida salida antes de escribir.
        let f = self.stiffness * u[0]; // Fuerza de resorte analítica.
        finite(&[f])?; // Impide escritura parcial por desbordamiento.
        out[0] = f; // Sobrescribe la única fuerza.
        Ok(()) // Estado aceptado intacto.
    }
    fn mass(&mut self, _: MassKind, out: &mut SymMatrix) -> EResult<()> {
        size(out.order(), 1)?; // Valida orden.
        *out = SymMatrix::new(DMat::new(1, 1, vec![1.0])?)?; // Masa unitaria kg.
        Ok(()) // Misma masa en ambas distribuciones.
    }
    fn geometric(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()> {
        vector(u, 1)?; // Valida desplazamiento.
        size(out.order(), 1)?; // Valida orden.
        *out = SymMatrix::zeros(1)?; // Rigidez geométrica nula.
        Ok(()) // No ofrece segundo orden.
    }
    fn external_force(&self, case: &CaseRef, factor: Real, out: &mut [Real]) -> EResult<()> {
        size(out.len(), 1)?; // Verifica salida.
        finite(&[factor])?; // Verifica factor adimensional.
        if case.0 != "example.case" {
            return Err(EngineError::new("ENG-E-0003", "Caso ausente."));
        } // Rechaza caso desconocido.
        out[0] = factor; // Fuerza base 1 N por factor.
        Ok(()) // Sin efecto sobre historia.
    }
    fn accept(&mut self, u: &[Real]) -> EResult<()> {
        let mut force = [0.0]; // Prepara reevaluación atómica.
        self.internal_force(u, &mut force)?; // Valida exactamente u y su respuesta.
        self.accepted = u[0]; // Acepta solo después del éxito.
        Ok(()) // Nuevo punto aceptado explícito.
    }
    fn rollback(&mut self) {} // No guarda pruebas: conservar accepted basta.
    fn dof_label(&self, d: DofIndex) -> (NodeId, Dof) {
        assert_eq!(d.0, 0, "Índice fuera del sistema de ejemplo"); // Aplica precondición documentada.
        (NodeId(1), Dof::Ux) // Traduce ecuación a nodo/componente.
    }
}

/// Destino mock en memoria, conserva escrituras repetidas en orden de llegada.
#[derive(Default)]
pub struct MemorySink {
    /// Entradas emitidas, sin deduplicación ni serialización.
    pub entries: Vec<(ResultKey, ResultValue)>,
}
impl ResultSink for MemorySink {
    fn put(&mut self, key: ResultKey, value: ResultValue) -> EResult<()> {
        conformance::validate_result(&key, &value)?; // Valida toda la entrada antes de mutar.
        self.entries.push((key, value)); // Escritura atómica en memoria y ordenada.
        Ok(()) // Confirma recepción.
    }
}

/// Control mock observable: cancelación atómica y progreso protegido por mutex.
#[derive(Default)]
pub struct RecordingControl {
    cancelled: AtomicBool,
    events: Mutex<Vec<(Real, String)>>,
}
impl RecordingControl {
    /// Solicita cancelación cooperativa de forma persistente.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    } // Solo escribe señal.
    /// Copia eventos de progreso; recupera contenido si un observador envenenó mutex.
    pub fn events(&self) -> Vec<(Real, String)> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone() // Lee sin alterar cálculo.
    }
}
impl RunControl for RecordingControl {
    fn progress(&self, fraction: Real, note: &str) {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push((fraction, note.into())); // Guarda evento válido enviado por método.
    }
    fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    } // Observa solicitud sin consumirla.
}

/// Mock de aceptación de desplazamientos prescritos; NO resuelve equilibrio.
/// Ajustes: array SI de n_free_dofs desplazamientos/giros. Un paso sin iteraciones;
/// normas cero indican que no se ejecutó corrección iterativa. Tolerancias 1e-8.
pub struct PrescribedStep;
impl AnalysisMethod for PrescribedStep {
    fn id(&self) -> &'static str {
        "example.prescribed_step"
    } // Identificador de plantilla.
    fn version(&self) -> (u16, u16, u16) {
        (1, 0, 0)
    } // Semver explícita.
    fn contract(&self) -> ContractId {
        ContractId::new("analysis", 1, 0)
    } // Contrato v1.
    fn settings_schema(&self) -> String {
        r#"{"type":"array","items":{"type":"number"}}"#.into()
    } // Longitud depende del sistema.
    fn requires(&self) -> Requirements {
        Requirements::default()
    } // No requiere masa ni Kg.
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
                } // Atiende cancelación antes de evaluar.
                let JsonValue::Array(values) = settings else {
                    return Err(invalid("Se esperaba array de desplazamientos."));
                }; // Valida tipo.
                let u: EResult<Vec<_>> = values
                    .iter()
                    .map(|v| match v {
                        JsonValue::Number(x) => Ok(*x),
                        _ => Err(invalid("Desplazamiento no numérico.")),
                    })
                    .collect(); // Extrae números SI.
                let u = u?; // Propaga campos inválidos.
                vector(&u, sys.n_free_dofs())?; // Verifica longitud y finitud.
                ctl.progress(0.0, "Evaluando desplazamientos prescritos"); // Progreso inicial.
                let mut force = vec![0.0; u.len()]; // Prepara salida de fuerza.
                sys.internal_force(&u, &mut force)?; // Evalúa una prueba sin aceptarla.
                finite(&force)?; // Rechaza salida defectuosa del sistema.
                if ctl.cancelled() {
                    return Ok(cancelled());
                } // Evita aceptar tras cancelación.
                sys.accept(&u)?; // Fija exactamente el desplazamiento prescrito antes de publicarlo.
                sink.put(
                    ResultKey {
                        block: "cases".into(),
                        entity: Some("example.prescribed".into()),
                        field: "displacements".into(),
                    },
                    ResultValue::Vector(u.clone()),
                )?; // Emite desplazamientos ya aceptados.
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
                }; // Declara normas y criterios del mock.
                ctl.progress(1.0, "Paso prescrito aceptado"); // Finaliza progreso monótono.
                Ok(AnalysisOutcome {
                    status: Status::Completed,
                    report,
                    diagnostics: vec![],
                }) // Resultado terminal explícito.
            };
        let outcome = execute(sys, sink); // Ejecuta una sola estrategia declarada.
        if outcome.as_ref().is_err()
            || outcome
                .as_ref()
                .is_ok_and(|o| o.status == Status::Cancelled)
        {
            sys.rollback();
        } // Descarta pruebas al fallar o cancelar.
        outcome // Propaga outcome o error sin reintentos.
    }
}
fn cancelled() -> AnalysisOutcome {
    AnalysisOutcome {
        status: Status::Cancelled,
        report: ConvergenceReport::default(),
        diagnostics: vec![],
    } // No publica un paso no aceptado.
}
