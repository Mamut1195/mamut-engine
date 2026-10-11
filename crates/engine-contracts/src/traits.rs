//! Fronteras v1; todos los errores reutilizan EngineError del núcleo.
use crate::*;

/// Ley uniaxial pura, thread-safe, contrato uniaxial/1.
///
/// # Entradas y unidades
/// Deformación total adimensional para material, o desplazamiento relativo m
/// para resorte; historia aceptada finita y con longitud state_len.
/// # Salidas e invariantes
/// Tensión Pa o fuerza N y su derivada consistente (error relativo FD < 1e-6).
/// Toda historia es plana, finita y propiedad del llamador. Repetir la entrada
/// aceptada reproduce la tensión aceptada. No modifica self ni committed;
/// cada kind declara su rango de validez y puntos no diferenciables.
/// # Errores
/// ENG-E-0008 por no finitos/desbordamiento; ENG-E-0010 por historia o rango
/// inválidos. Otros errores físicos deben declarar código y contexto del núcleo.
pub trait UniaxialLaw: Send + Sync {
    /// Nombre propio estable; sin unidades ni efectos laterales.
    fn kind(&self) -> &'static str;
    /// Identidad uniaxial/1; versión menor compatible.
    fn contract(&self) -> ContractId {
        ContractId::new("uniaxial", 1, 0)
    }
    /// Longitud exacta de la historia aceptada y de prueba.
    fn state_len(&self) -> usize;
    /// Historia inicial finita en las unidades documentadas por kind.
    fn initial_state(&self) -> Vec<Real>;
    /// Tangente inicial finita, Pa o N/m; consistente con el origen inicial.
    fn initial_tangent(&self) -> Real;
    /// Evalúa deformación TOTAL y estado ACEPTADO; devuelve respuesta de prueba.
    /// No acepta historia implícitamente; errores y unidades según este trait.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse>;
}

/// Ley de sección pura y thread-safe, contrato section/1.
///
/// # Entradas y unidades
/// D2 e=[ε0, κz]; D3 e=[ε0, κy, κz, θ′]; ε0 adimensional y resto m⁻¹.
/// Estado aceptado plano finito de longitud state_len.
/// # Salidas e invariantes
/// D2 s=[N, Mz]; D3 s=[N, My, Mz, T], N y N·m. Tangente simétrica v1,
/// derivada consistente de s respecto a e con error FD relativo < 1e-6.
/// No modifica self ni committed; al repetir e aceptado reproduce s aceptado.
/// Toda salida es finita y trial_state tiene longitud state_len.
/// # Errores
/// ENG-E-0010 por tamaños o rango inválidos; ENG-E-0008 por no finitos.
pub trait SectionLaw: Send + Sync {
    /// Nombre propio de la sección y su rango físico documentado.
    fn kind(&self) -> &'static str;
    /// Identidad section/1.
    fn contract(&self) -> ContractId;
    /// Dimensión que determina dos o cuatro deformaciones generalizadas.
    fn dim(&self) -> Dim;
    /// Longitud exacta del estado plano.
    fn state_len(&self) -> usize;
    /// Historia inicial finita; unidades definidas por kind.
    fn initial_state(&self) -> Vec<Real>;
    /// Responde a deformaciones TOTALES usando historia ACEPTADA sin mutarla.
    /// Devuelve resultantes, tangente y estado de prueba; errores según el trait.
    fn respond(&self, e: &[Real], committed: &[Real]) -> EResult<SectionResponse>;
}

/// Elemento puro y thread-safe, contrato element/1.
///
/// # Entradas y unidades
/// Contexto con n_nodes coordenadas m y parámetros SI resueltos. u_e TOTAL
/// global, nodo y después dofs_per_node: traslaciones m y giros rad.
/// Estado ACEPTADO de longitud state_len. D2 exige X-Y y Z=0.
/// # Salidas e invariantes
/// Fuerzas N y momentos N·m globales; longitud n_nodes × dofs_per_node.len().
/// Tangente y masa cuadradas simétricas v1; tangente consistente FD < 1e-6.
/// Tres modos rígidos D2 y seis D3 sin fuerza interna para historia inicial sin
/// pretensión. Covarianza de fuerzas, tangente y masa al rotar el sistema global.
/// Respuesta finita y determinista; repetir el desplazamiento aceptado reproduce
/// fuerzas aceptadas. Nunca muta self, contexto ni historia.
/// # Errores
/// ENG-E-0010 por dimensión, tamaños, parámetros, cargas o estaciones no admitidos;
/// ENG-E-0005 por longitud nula; ENG-E-0008 por no finitos/desbordamiento.
/// Ausencia de masa o rigidez geométrica se indica con None, no con error.
pub trait Element: Send + Sync {
    /// Kind propio; documenta hipótesis, rango físico y cargas admitidas.
    fn kind(&self) -> &'static str;
    /// Identidad element/1.
    fn contract(&self) -> ContractId;
    /// Número de nodos en el contexto y en los vectores.
    fn n_nodes(&self) -> usize;
    /// Componentes activas por nodo, distintas y en el orden de los vectores.
    fn dofs_per_node(&self) -> &'static [Dof];
    /// Longitud exacta de historia; por defecto sin historia.
    fn state_len(&self) -> usize {
        0
    }
    /// Historia inicial finita; por defecto vacía.
    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }
    /// Evalúa u_e TOTAL y estado ACEPTADO; devuelve fuerza, tangente y prueba.
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse>;
    /// Devuelve masa (kg, kg·m y kg·m² según GDL) o None si no existe;
    /// valida contexto y devuelve errores definidos arriba.
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>>;
    /// Devuelve Kg global calculada con axial propio de u_e, o None si no soporta.
    /// Mismas unidades/tamaños que tangente; no acepta historia.
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>>;
    /// Convierte una carga de tramo a fuerzas/momentos nodales globales;
    /// longitud del vector igual a respond; rechaza componentes no soportadas.
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>>;
    /// Recupera acciones locales en caras de extremo y estaciones x/L en `[0,1]`,
    /// en el orden solicitado, incluyendo cargas de tramo. No modifica historia.
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput>;
}

/// Fábrica thread-safe de elementos, sin ensamblaje ni acceso al backend.
///
/// # Entradas, salidas y unidades
/// JSON mínimo en memoria, con propiedades SI según el esquema y ModelView
/// prestado de referencias únicas y coordenadas m. Devuelve elemento independiente
/// de la vida de la vista; no modifica el modelo.
/// # Invariantes y errores
/// Valida campos requeridos, tipos, rango, referencias y dimensión contra el
/// esquema declarado. Kind del elemento igual al de la fábrica, element/1.
/// ENG-E-0010 por esquema/dimensión, ENG-E-0003 por referencias ausentes,
/// ENG-E-0008 por no finitos. No realiza parsing JSON ni usa serde.
pub trait ElementFactory: Send + Sync {
    /// Nombre propio igual al del elemento construido.
    fn kind(&self) -> &'static str;
    /// JSON Schema como texto JSON válido; sin unidades implícitas.
    fn params_schema(&self) -> String;
    /// Valida parámetros SI y modelo prestado, y construye un elemento propietario.
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>>;
}

/// Frontera interna entre método de análisis y sistema ensamblado.
///
/// # Entradas, salidas y unidades
/// Vectores reducidos de n_free_dofs, desplazamientos m/giros rad, fuerzas N/
/// momentos N·m. Matrices cuadradas del mismo orden; masas kg/inercias kg·m².
/// Fuerzas externas por CaseRef y factor adimensional. El productor declara
/// el escalado necesario para normas relativas con componentes mixtas.
/// # Invariantes
/// Cada operación sobrescribe completamente out, nunca acumula. Tangente y masa
/// simétricas. Las evaluaciones usan el último estado aceptado y no lo aceptan
/// implícitamente. accept(u) reevaluará/aceptará exactamente u, de forma atómica;
/// rollback descarta toda prueba y conserva el último aceptado, sin fallar.
/// # Errores
/// ENG-E-0010 por tamaños/contexto; ENG-E-0008 por no finitos;
/// ENG-E-0003 por caso ausente; ENG-E-0012 por estado interno incoherente.
/// Un índice para dof_label debe ser válido; fuera de rango puede causar panic.
pub trait SystemView {
    /// Número de ecuaciones libres en el orden reducido.
    fn n_free_dofs(&self) -> usize;
    /// Patrón simétrico de ese orden; sin análisis numérico.
    fn pattern(&self) -> &SparsityPattern;
    /// Sobrescribe out con dF_int/du en u TOTAL, sin aceptar estado.
    fn tangent(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()>;
    /// Sobrescribe out con fuerzas internas globales reducidas para u TOTAL.
    fn internal_force(&mut self, u: &[Real], out: &mut [Real]) -> EResult<()>;
    /// Sobrescribe out con masa del tipo solicitado; cero si no hay masa.
    fn mass(&mut self, kind: MassKind, out: &mut SymMatrix) -> EResult<()>;
    /// Sobrescribe out con rigidez geométrica usando axial propio de u.
    fn geometric(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()>;
    /// Sobrescribe fuerzas del caso escaladas por factor; no modifica el sistema.
    fn external_force(&self, case: &CaseRef, factor: Real, out: &mut [Real]) -> EResult<()>;
    /// Acepta exactamente u y su historia reevaluada; si falla conserva aceptado.
    fn accept(&mut self, u: &[Real]) -> EResult<()>;
    /// Descarta prueba y recupera historia aceptada; infalible e idempotente.
    fn rollback(&mut self);
    /// Etiqueta nodo/GDL para índice global reducido válido; precondición d < n.
    fn dof_label(&self, d: DofIndex) -> (NodeId, Dof);
}

/// Método thread-safe, contrato analysis/1, sin acceso directo a nodos/elementos.
///
/// # Entradas y unidades
/// Sistema reducido, ajustes JSON mínimo con unidades SI y estrategias explícitas,
/// destino de resultados SI y control cooperativo. requires declara capacidades.
/// # Salidas e invariantes
/// Siempre reporta normas relativas y tolerancias usadas. NotConverged conserva
/// último estado aceptado, sin reintentos ocultos. Unstable indica pérdida de
/// estabilidad; Cancelled conserva último aceptado. Completed exige convergencia.
/// Mecanismos se diagnostican ENG-E-0006 con nodo/GDL. No publica pruebas como
/// estados aceptados. La procedencia results/1 completa se añade en la fachada.
/// # Errores
/// ENG-E-0010 por ajustes/capacidades inválidos; ENG-E-0008 por no finitos;
/// propaga errores de sistema y destino. En error o cancelación descarta pruebas.
/// La no convergencia se devuelve como outcome, no como error de infraestructura.
pub trait AnalysisMethod: Send + Sync {
    /// Identificador estable propio del método.
    fn id(&self) -> &'static str;
    /// Versión semántica mayor, menor y parche.
    fn version(&self) -> (u16, u16, u16);
    /// Identidad analysis/1.
    fn contract(&self) -> ContractId;
    /// JSON Schema textual de ajustes y estrategias explícitas.
    fn settings_schema(&self) -> String;
    /// Capacidades requeridas antes de ejecutar.
    fn requires(&self) -> Requirements;
    /// Ejecuta y emite resultados aceptados, normas, tolerancias y diagnósticos.
    /// Errores según este trait; respeta cancelled antes de aceptar cada paso.
    fn run(
        &self,
        sys: &mut dyn SystemView,
        settings: &JsonValue,
        sink: &mut dyn ResultSink,
        ctl: &dyn RunControl,
    ) -> EResult<AnalysisOutcome>;
}

/// Control cooperativo sin modificar el sistema.
///
/// Entrada: progreso adimensional finito en `[0,1]` y nota en español.
/// Salida: observación de cancelación; sin errores de infraestructura ni panic
/// para entradas válidas. El progreso debe ser monótono dentro de una ejecución.
/// El método consulta cancelled entre pasos y antes de aceptar; la observación
/// puede cambiar por intervención externa. No obliga a un modelo de hilos.
pub trait RunControl {
    /// Publica fracción finita `[0,1]` y nota; no altera estado de cálculo.
    fn progress(&self, fraction: Real, note: &str);
    /// Devuelve true si se solicitó detener; no acepta ni revierte el sistema.
    fn cancelled(&self) -> bool;
}

/// Destino de valores results/1, sin ejecutar cálculos.
///
/// Entrada: clave con bloque/campo propios no vacíos y valor finito SI; salida:
/// confirmación de escritura o error. Cada put es atómico; la política de claves
/// repetidas se documenta por implementación. Conserva orden de vectores y signos.
/// ENG-E-0010 por clave/valor inválido, ENG-E-0008 por no finitos; fallos E/S
/// se expresan mediante EngineError con contexto. No acepta historia del sistema.
pub trait ResultSink {
    /// Escribe un valor SI en su clave; propaga errores y no deja escritura parcial.
    fn put(&mut self, key: ResultKey, value: ResultValue) -> EResult<()>;
}
