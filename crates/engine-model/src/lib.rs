//! Modelo de dominio e inspección de entradas, escritos desde cero.
//!
//! **PROPUESTA ESTÁNDAR PENDIENTE DE APROBACIÓN DE JONATHAN.**
//! **Estado: DECISIÓN PROVISIONAL (J0/J2, 2026-10-10), revocable por Jonathan.** Ejes globales X, Y, Z diestros, Z vertical
//! hacia arriba. Los nodos D2 representan el plano X-Y (coordenada Z = 0), con
//! Ux, Uy y Rz; este plano es horizontal bajo la decisión provisional. D3 admite seis GDL.
//! El eje local x va de i a j; en D2, y = (-x_y, x_x, 0) y z = x × y.
//! En D3 se necesita una orientación adicional antes del cálculo; este modelo
//! de dominio no la deduce. Fuerzas y desplazamientos positivos siguen los ejes;
//! giros y momentos positivos siguen la mano derecha. Axial de tracción positivo.
//! En una sección con normal exterior +x, las resultantes positivas son
//! +N x, +Vy y, +Vz z y +T x, +My y, +Mz z; en la cara con normal -x se
//! invierten todas esas direcciones. En las caras de extremo del miembro,
//! i es la cara negativa y j la positiva. Por tanto, tracción positiva actúa
//! hacia -x en i y +x en j; cortante Vy positivo hacia -y en i y +y en j;
//! momento Mz positivo hacia -z en i y +z en j. Estos signos de resultantes
//! internas deben distinguirse de cargas nodales expresadas en ejes globales.
//! Véase también `docs/CONVENCION_EJES.md`.
//!
//! Cada nodo declara su dimensión; se permiten colecciones de nodos de distintas
//! dimensiones sin imponer compatibilidad entre miembros en este ticket. Todas
//! las magnitudes usan unidades coherentes elegidas por quien construye el modelo.
//! Las coordenadas siempre tienen tres componentes. No se comprueba aquí la
//! condición geométrica Z = 0 de D2, ni positividad física de propiedades o masas.
//! La inserción conserva entradas inválidas y duplicadas para diagnosticarlas.
//!
//! ```
//! use engine_model::*;
//! let mut modelo = Model::new();
//! modelo.insert_node(Node::new(NodeId(1), Dim::D2, [0.0, 0.0, 0.0]));
//! modelo.insert_node(Node::new(NodeId(2), Dim::D2, [2.0, 0.0, 0.0]));
//! modelo.insert_section(Section { id: "s".into(), properties: vec![] });
//! modelo.insert_law(Law { name: "lineal".into(), parameters: vec![] });
//! modelo.insert_member(Member {
//!     id: MemberId(1), nodes: [NodeId(1), NodeId(2)],
//!     section: "s".into(), law: "lineal".into(),
//! });
//! assert!(validate(&modelo).is_empty());
//! ```

#![deny(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};

pub use engine_core::{Diagnostic, Dim, Dof, EntityRef, MemberId, NodeId, Real};

/// Tolerancia absoluta de longitud, en la unidad de coordenadas del modelo.
///
/// Una distancia euclídea tridimensional menor o igual a 1e-12 se considera nula.
/// No se escala con el tamaño del modelo; convierta unidades de forma coherente.
pub const LENGTH_TOLERANCE: Real = 1e-12;

/// Parámetro genérico, sin interpretar su significado constitutivo.
#[derive(Clone, Debug)]
pub struct Parameter {
    /// Nombre propio del parámetro.
    pub name: String,
    /// Valor en las unidades coherentes del modelo.
    pub value: Real,
}

/// Valor asociado a una componente nodal (carga o masa).
#[derive(Clone, Debug)]
pub struct DofValue {
    /// Componente de desplazamiento o giro asociada.
    pub dof: Dof,
    /// Magnitud; para giros, la masa representa inercia rotacional.
    pub value: Real,
}

/// Nodo geométrico con dimensión y GDL declarados.
#[derive(Clone, Debug)]
pub struct Node {
    /// Identificador del nodo.
    pub id: NodeId,
    /// Dimensión que determina los GDL permitidos.
    pub dim: Dim,
    /// Coordenadas globales X, Y, Z.
    pub coordinates: [Real; 3],
    /// GDL declarados; se comprueba su pertenencia a la dimensión.
    pub dofs: Vec<Dof>,
}

impl Node {
    /// Construye un nodo con todos los GDL de su dimensión, sin validar coordenadas.
    pub fn new(id: NodeId, dim: Dim, coordinates: [Real; 3]) -> Self {
        Self {
            id,
            dim,
            coordinates,
            dofs: dim.dofs().to_vec(),
        }
    }
}

/// Miembro orientado entre dos nodos, con sección y ley por nombre.
#[derive(Clone, Debug)]
pub struct Member {
    /// Identificador del miembro.
    pub id: MemberId,
    /// Extremos i y j, en ese orden.
    pub nodes: [NodeId; 2],
    /// Identificador de la sección.
    pub section: String,
    /// Nombre de la ley constitutiva.
    pub law: String,
}

/// Sección con propiedades genéricas que interpretará el cálculo futuro.
#[derive(Clone, Debug)]
pub struct Section {
    /// Identificador propio de la sección.
    pub id: String,
    /// Propiedades, por ejemplo área o inercias, sin requisitos físicos aquí.
    pub properties: Vec<Parameter>,
}

/// Ley constitutiva declarativa; este crate no evalúa su respuesta.
#[derive(Clone, Debug)]
pub struct Law {
    /// Nombre propio, utilizado como identificador.
    pub name: String,
    /// Parámetros genéricos de la ley.
    pub parameters: Vec<Parameter>,
}

/// Apoyo nodal mediante componentes restringidas a desplazamiento o giro cero.
#[derive(Clone, Debug)]
pub struct Support {
    /// Identificador propio del apoyo.
    pub id: String,
    /// Nodo del apoyo.
    pub node: NodeId,
    /// GDL restringidos.
    pub restrained: Vec<Dof>,
}

/// Acción nodal de un caso, expresada en ejes globales.
#[derive(Clone, Debug)]
pub struct NodalLoad {
    /// Nodo al que se aplica la carga.
    pub node: NodeId,
    /// Fuerzas o momentos asociados a componentes nodales.
    pub values: Vec<DofValue>,
}

/// Caso de carga identificado por un nombre propio.
#[derive(Clone, Debug)]
pub struct LoadCase {
    /// Nombre del caso, utilizado como identificador.
    pub name: String,
    /// Acciones nodales del caso.
    pub loads: Vec<NodalLoad>,
}

/// Término de una combinación de casos.
#[derive(Clone, Debug)]
pub struct CaseFactor {
    /// Nombre del caso referenciado.
    pub case: String,
    /// Factor multiplicador de las acciones del caso.
    pub factor: Real,
}

/// Combinación lineal declarativa de casos de carga.
#[derive(Clone, Debug)]
pub struct Combination {
    /// Identificador propio de la combinación.
    pub id: String,
    /// Casos y sus factores, conservados en orden de entrada.
    pub terms: Vec<CaseFactor>,
}

/// Masas e inercias concentradas en un nodo.
#[derive(Clone, Debug)]
pub struct NodalMass {
    /// Identificador propio de la asignación de masa.
    pub id: String,
    /// Nodo receptor.
    pub node: NodeId,
    /// Valores de masa o inercia por componente nodal.
    pub values: Vec<DofValue>,
}

/// Colecciones de dominio, sin validación implícita ni sustitución de duplicados.
///
/// Los identificadores son únicos dentro de cada clase, no entre clases.
/// Las listas públicas permiten construir y corregir entradas antes de validar.
#[derive(Clone, Debug, Default)]
pub struct Model {
    /// Nodos en orden de inserción.
    pub nodes: Vec<Node>,
    /// Miembros en orden de inserción.
    pub members: Vec<Member>,
    /// Secciones en orden de inserción.
    pub sections: Vec<Section>,
    /// Leyes en orden de inserción.
    pub laws: Vec<Law>,
    /// Apoyos en orden de inserción.
    pub supports: Vec<Support>,
    /// Casos en orden de inserción.
    pub cases: Vec<LoadCase>,
    /// Combinaciones en orden de inserción.
    pub combinations: Vec<Combination>,
    /// Masas en orden de inserción.
    pub masses: Vec<NodalMass>,
}

impl Model {
    /// Construye un modelo vacío, sin diagnósticos.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserta un nodo sin validar, conservando posibles duplicados.
    pub fn insert_node(&mut self, node: Node) -> &mut Self {
        self.nodes.push(node);
        self
    }
    /// Inserta un miembro sin validar sus referencias ni geometría.
    pub fn insert_member(&mut self, member: Member) -> &mut Self {
        self.members.push(member);
        self
    }
    /// Inserta una sección sin validar sus propiedades.
    pub fn insert_section(&mut self, section: Section) -> &mut Self {
        self.sections.push(section);
        self
    }
    /// Inserta una ley sin validar sus parámetros.
    pub fn insert_law(&mut self, law: Law) -> &mut Self {
        self.laws.push(law);
        self
    }
    /// Inserta un apoyo sin validar el nodo ni los GDL.
    pub fn insert_support(&mut self, support: Support) -> &mut Self {
        self.supports.push(support);
        self
    }
    /// Inserta un caso sin validar sus acciones.
    pub fn insert_case(&mut self, case: LoadCase) -> &mut Self {
        self.cases.push(case);
        self
    }
    /// Inserta una combinación sin validar casos ni factores.
    pub fn insert_combination(&mut self, combination: Combination) -> &mut Self {
        self.combinations.push(combination);
        self
    }
    /// Inserta una masa sin validar valores, nodo ni GDL.
    pub fn insert_mass(&mut self, mass: NodalMass) -> &mut Self {
        self.masses.push(mass);
        self
    }
}

fn report(out: &mut Vec<Diagnostic>, code: &str, entity: EntityRef, context: String) {
    let mut diagnostic = Diagnostic::from_code(code).expect("Código existente en engine-core");
    diagnostic.entities.push(entity);
    diagnostic.hint = Some(context);
    out.push(diagnostic);
}

fn duplicate<K: Ord>(
    seen: &mut BTreeSet<K>,
    key: K,
    out: &mut Vec<Diagnostic>,
    entity: EntityRef,
    context: String,
) {
    if !seen.insert(key) {
        report(out, "ENG-E-0002", entity, context);
    }
}

fn finite(out: &mut Vec<Diagnostic>, value: Real, entity: EntityRef, context: String) {
    if !value.is_finite() {
        report(out, "ENG-E-0008", entity, context);
    }
}

fn component(
    out: &mut Vec<Diagnostic>,
    node: Option<&&Node>,
    dof: Dof,
    entity: EntityRef,
    context: String,
) {
    if node.is_some_and(|node| !node.dim.dofs().contains(&dof)) {
        report(out, "ENG-E-0011", entity, context);
    }
}

/// Inspecciona el modelo completo sin modificarlo y devuelve todos los diagnósticos.
///
/// Orden determinista para la misma entrada: nodos, miembros, secciones, leyes,
/// apoyos, casos, combinaciones y masas; dentro de cada lista, orden de inserción.
/// En cada entidad se emite primero duplicidad, luego controles en orden de campos.
/// Cada repetición posterior de un id produce un diagnóstico. Las referencias
/// ambiguas usan la primera declaración para continuar los controles. Un nodo
/// es huérfano si su id no aparece en ningún extremo, aunque ese miembro tenga
/// otras referencias rotas. Un apoyo, carga o masa no evita la condición de huérfano.
/// No se evalúa longitud con extremos ausentes o coordenadas no finitas.
///
/// `entities` identifica al culpable: nodo, miembro, ley o caso. Para apoyos y
/// masas se usa su nodo (incluso ausente); para secciones, el primer miembro que
/// las referencia y, si no hay ninguno, `Law(id_sección)` como referencia nominal.
/// Para combinaciones se usa el caso del término culpable o el primero en errores
/// de duplicidad; si no hay términos, `Case(id_combinación)` es referencia nominal.
/// Estas referencias nominales no afirman existencia de una ley o caso homónimo.
/// `hint` siempre incluye la clase y el nombre/id exacto de la entidad original.
///
/// Identificadores duplicados:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// m.insert_law(Law { name: "ley".into(), parameters: vec![] });
/// m.insert_law(m.laws[0].clone());
/// let d = validate(&m);
/// assert_eq!(d[0].code, "ENG-E-0002");
/// assert!(matches!(&d[0].entities[0], EntityRef::Law(n) if n == "ley"));
/// ```
/// Referencias rotas:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// m.insert_member(Member { id: MemberId(7), nodes: [NodeId(1), NodeId(2)],
///     section: "s".into(), law: "l".into() });
/// let d = validate(&m);
/// assert_eq!(d.len(), 4);
/// assert!(d.iter().all(|d| d.code == "ENG-E-0003"
///     && matches!(d.entities[0], EntityRef::Member(MemberId(7)))));
/// ```
/// Nodo huérfano:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// m.insert_node(Node::new(NodeId(3), Dim::D2, [0.0; 3]));
/// let d = validate(&m);
/// assert_eq!(d[0].code, "ENG-E-0004");
/// assert!(matches!(d[0].entities[0], EntityRef::Node(NodeId(3))));
/// ```
/// Longitud nula, incluyendo el límite de tolerancia:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// m.insert_node(Node::new(NodeId(1), Dim::D2, [0.0; 3]));
/// m.insert_node(Node::new(NodeId(2), Dim::D2, [LENGTH_TOLERANCE, 0.0, 0.0]));
/// m.insert_member(Member { id: MemberId(4), nodes: [NodeId(1), NodeId(2)],
///     section: "s".into(), law: "l".into() });
/// let d = validate(&m);
/// let d = d.iter().find(|d| d.code == "ENG-E-0005").unwrap();
/// assert!(matches!(d.entities[0], EntityRef::Member(MemberId(4))));
/// ```
/// Valores no finitos:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// m.insert_law(Law { name: "ley".into(), parameters: vec![
///     Parameter { name: "rigidez".into(), value: f64::NAN }] });
/// let d = validate(&m);
/// assert_eq!(d[0].code, "ENG-E-0008");
/// assert!(matches!(&d[0].entities[0], EntityRef::Law(n) if n == "ley"));
/// ```
/// GDL inválido:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// let mut n = Node::new(NodeId(8), Dim::D2, [0.0; 3]);
/// n.dofs.push(Dof::Uz);
/// m.insert_node(n);
/// let d = validate(&m);
/// let d = d.iter().find(|d| d.code == "ENG-E-0011").unwrap();
/// assert!(matches!(d.entities[0], EntityRef::Node(NodeId(8))));
/// ```
pub fn validate(model: &Model) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut nodes = BTreeMap::new();
    for node in &model.nodes {
        nodes.entry(node.id).or_insert(node);
    }
    let sections: BTreeSet<_> = model.sections.iter().map(|s| s.id.as_str()).collect();
    let laws: BTreeSet<_> = model.laws.iter().map(|l| l.name.as_str()).collect();
    let cases: BTreeSet<_> = model.cases.iter().map(|c| c.name.as_str()).collect();
    let connected: BTreeSet<_> = model.members.iter().flat_map(|m| m.nodes).collect();
    let mut seen = BTreeSet::new();
    for node in &model.nodes {
        let entity = EntityRef::Node(node.id);
        let context = format!("Nodo {}", node.id.0);
        duplicate(
            &mut seen,
            node.id,
            &mut out,
            entity.clone(),
            context.clone(),
        );
        for (axis, value) in node.coordinates.iter().enumerate() {
            finite(
                &mut out,
                *value,
                entity.clone(),
                format!("{context}, coordenada {}", ["X", "Y", "Z"][axis]),
            );
        }
        for dof in &node.dofs {
            component(
                &mut out,
                Some(&node),
                *dof,
                entity.clone(),
                format!("{context}, GDL {dof}"),
            );
        }
        if !connected.contains(&node.id) {
            report(&mut out, "ENG-E-0004", entity, context);
        }
    }
    let mut seen = BTreeSet::new();
    for member in &model.members {
        let entity = EntityRef::Member(member.id);
        let context = format!("Miembro {}", member.id.0);
        duplicate(
            &mut seen,
            member.id,
            &mut out,
            entity.clone(),
            context.clone(),
        );
        for (end, id) in member.nodes.iter().enumerate() {
            if !nodes.contains_key(id) {
                report(
                    &mut out,
                    "ENG-E-0003",
                    entity.clone(),
                    format!(
                        "{context}, extremo {}: nodo {} ausente",
                        ["i", "j"][end],
                        id.0
                    ),
                );
            }
        }
        if !sections.contains(member.section.as_str()) {
            report(
                &mut out,
                "ENG-E-0003",
                entity.clone(),
                format!("{context}, sección {:?} ausente", member.section),
            );
        }
        if !laws.contains(member.law.as_str()) {
            report(
                &mut out,
                "ENG-E-0003",
                entity.clone(),
                format!("{context}, ley {:?} ausente", member.law),
            );
        }
        if let (Some(a), Some(b)) = (nodes.get(&member.nodes[0]), nodes.get(&member.nodes[1])) {
            if a.coordinates
                .iter()
                .chain(&b.coordinates)
                .all(|x| x.is_finite())
            {
                let distance = a
                    .coordinates
                    .iter()
                    .zip(b.coordinates)
                    .fold(0.0_f64, |length, (a, b)| length.hypot(a - b));
                if distance <= LENGTH_TOLERANCE {
                    report(
                        &mut out,
                        "ENG-E-0005",
                        entity,
                        format!("{context}, longitud {distance} <= tolerancia {LENGTH_TOLERANCE}"),
                    );
                }
            }
        }
    }
    let mut seen = BTreeSet::new();
    for section in &model.sections {
        let entity = model
            .members
            .iter()
            .find(|m| m.section == section.id)
            .map_or_else(
                || EntityRef::Law(section.id.clone()),
                |m| EntityRef::Member(m.id),
            );
        let context = format!("Sección {:?}", section.id);
        duplicate(
            &mut seen,
            &section.id,
            &mut out,
            entity.clone(),
            context.clone(),
        );
        for parameter in &section.properties {
            finite(
                &mut out,
                parameter.value,
                entity.clone(),
                format!("{context}, propiedad {:?}", parameter.name),
            );
        }
    }
    let mut seen = BTreeSet::new();
    for law in &model.laws {
        let entity = EntityRef::Law(law.name.clone());
        let context = format!("Ley {:?}", law.name);
        duplicate(
            &mut seen,
            &law.name,
            &mut out,
            entity.clone(),
            context.clone(),
        );
        for parameter in &law.parameters {
            finite(
                &mut out,
                parameter.value,
                entity.clone(),
                format!("{context}, parámetro {:?}", parameter.name),
            );
        }
    }
    let mut seen = BTreeSet::new();
    for support in &model.supports {
        let entity = EntityRef::Node(support.node);
        let context = format!("Apoyo {:?}, nodo {}", support.id, support.node.0);
        duplicate(
            &mut seen,
            &support.id,
            &mut out,
            entity.clone(),
            context.clone(),
        );
        if !nodes.contains_key(&support.node) {
            report(&mut out, "ENG-E-0003", entity.clone(), context.clone());
        }
        for dof in &support.restrained {
            component(
                &mut out,
                nodes.get(&support.node),
                *dof,
                entity.clone(),
                format!("{context}, GDL {dof}"),
            );
        }
    }
    let mut seen = BTreeSet::new();
    for case in &model.cases {
        let entity = EntityRef::Case(case.name.clone());
        let context = format!("Caso {:?}", case.name);
        duplicate(
            &mut seen,
            &case.name,
            &mut out,
            entity.clone(),
            context.clone(),
        );
        for (index, load) in case.loads.iter().enumerate() {
            let context = format!("{context}, carga {index}, nodo {}", load.node.0);
            if !nodes.contains_key(&load.node) {
                report(&mut out, "ENG-E-0003", entity.clone(), context.clone());
            }
            for value in &load.values {
                let context = format!("{context}, GDL {}", value.dof);
                finite(&mut out, value.value, entity.clone(), context.clone());
                component(
                    &mut out,
                    nodes.get(&load.node),
                    value.dof,
                    entity.clone(),
                    context,
                );
            }
        }
    }
    let mut seen = BTreeSet::new();
    for combination in &model.combinations {
        let context = format!("Combinación {:?}", combination.id);
        let entity = EntityRef::Case(
            combination
                .terms
                .first()
                .map_or(&combination.id, |t| &t.case)
                .clone(),
        );
        duplicate(
            &mut seen,
            &combination.id,
            &mut out,
            entity,
            context.clone(),
        );
        for (index, term) in combination.terms.iter().enumerate() {
            let entity = EntityRef::Case(term.case.clone());
            let context = format!("{context}, término {index}, caso {:?}", term.case);
            if !cases.contains(term.case.as_str()) {
                report(&mut out, "ENG-E-0003", entity.clone(), context.clone());
            }
            finite(&mut out, term.factor, entity, context);
        }
    }
    let mut seen = BTreeSet::new();
    for mass in &model.masses {
        let entity = EntityRef::Node(mass.node);
        let context = format!("Masa {:?}, nodo {}", mass.id, mass.node.0);
        duplicate(
            &mut seen,
            &mass.id,
            &mut out,
            entity.clone(),
            context.clone(),
        );
        if !nodes.contains_key(&mass.node) {
            report(&mut out, "ENG-E-0003", entity.clone(), context.clone());
        }
        for value in &mass.values {
            let context = format!("{context}, GDL {}", value.dof);
            finite(&mut out, value.value, entity.clone(), context.clone());
            component(
                &mut out,
                nodes.get(&mass.node),
                value.dof,
                entity.clone(),
                context,
            );
        }
    }
    out
}

#[cfg(test)]
mod tests;
