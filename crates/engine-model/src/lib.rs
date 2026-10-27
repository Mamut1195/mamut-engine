#![forbid(unsafe_code)]
//! Domain model and input inspection, written from scratch.
//!
//! **APPROVED by Jonathan 2026-10-10 22:52: X-Y horizontal, Z vertical upward (the Engine governs).** Right-handed global X, Y, Z axes, Z vertically
//! upward. D2 nodes represent the X-Y plane (Z coordinate = 0), with
//! Ux, Uy, and Rz; this plane is horizontal. D3 supports six DOFs.
//! The local x axis runs from i to j; in D2, y = (-x_y, x_x, 0) and z = x × y.
//! In D3, an additional orientation is needed before computation; this domain
//! model does not infer it. Positive forces and displacements follow the axes;
//! positive rotations and moments follow the right-hand rule. Axial tension is positive.
//! On a section with outward normal +x, the positive resultants are
//! +N x, +Vy y, +Vz z and +T x, +My y, +Mz z; on the face with normal -x,
//! all these directions are reversed. On the member end faces,
//! i is the negative face and j the positive one. Thus, positive tension acts
//! toward -x at i and +x at j; positive shear Vy toward -y at i and +y at j;
//! positive moment Mz toward -z at i and +z at j. These signs of internal
//! resultants must be distinguished from nodal loads expressed in global axes.
//! The 2D elevation convention is still pending a decision. See also `docs/AXES_CONVENTION.md`.
//!
//! Each node declares its dimension; collections of nodes with different
//! dimensions are allowed without enforcing member compatibility in this ticket. All
//! quantities use consistent units chosen by the model builder.
//! Coordinates always have three components. Neither the D2 geometric
//! condition Z = 0 nor physical positivity of properties or masses is checked here.
//! Insertion preserves invalid and duplicate entries for diagnostics.
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

use engine_core::math;
use std::collections::{BTreeMap, BTreeSet};

pub use engine_core::{Diagnostic, Dim, Dof, EntityRef, MemberId, NodeId, Real};

/// Absolute length tolerance, in the model's coordinate unit.
///
/// A three-dimensional Euclidean distance less than or equal to 1e-12 is considered zero.
/// It does not scale with model size; convert units consistently.
pub const LENGTH_TOLERANCE: Real = 1e-12;

/// Generic parameter, without interpreting its constitutive meaning.
#[derive(Clone, Debug)]
pub struct Parameter {
    /// Custom parameter name.
    pub name: String,
    /// Value in the model's consistent units.
    pub value: Real,
}

/// Value associated with a nodal component (load or mass).
#[derive(Clone, Debug)]
pub struct DofValue {
    /// Associated displacement or rotation component.
    pub dof: Dof,
    /// Magnitude; for rotations, mass represents rotational inertia.
    pub value: Real,
}

/// Geometric node with declared dimension and DOFs.
#[derive(Clone, Debug)]
pub struct Node {
    /// Node identifier.
    pub id: NodeId,
    /// Dimension that determines the allowed DOFs.
    pub dim: Dim,
    /// Global X, Y, Z coordinates.
    pub coordinates: [Real; 3],
    /// Declared DOFs; membership in the dimension is checked.
    pub dofs: Vec<Dof>,
}

impl Node {
    /// Constructs a node with all DOFs of its dimension, without validating coordinates.
    pub fn new(id: NodeId, dim: Dim, coordinates: [Real; 3]) -> Self {
        Self {
            id,
            dim,
            coordinates,
            dofs: dim.dofs().to_vec(),
        }
    }
}

/// Oriented member between two nodes, with section and law by name.
#[derive(Clone, Debug)]
pub struct Member {
    /// Member identifier.
    pub id: MemberId,
    /// Ends i and j, in that order.
    pub nodes: [NodeId; 2],
    /// Section identifier.
    pub section: String,
    /// Constitutive law name.
    pub law: String,
}

/// Section with generic properties to be interpreted by future computation.
#[derive(Clone, Debug)]
pub struct Section {
    /// Custom section identifier.
    pub id: String,
    /// Properties, such as area or inertias, without physical requirements here.
    pub properties: Vec<Parameter>,
}

/// Declarative constitutive law; this crate does not evaluate its response.
#[derive(Clone, Debug)]
pub struct Law {
    /// Custom name, used as an identifier.
    pub name: String,
    /// Generic law parameters.
    pub parameters: Vec<Parameter>,
}

/// Nodal support through components constrained to zero displacement or rotation.
#[derive(Clone, Debug)]
pub struct Support {
    /// Custom support identifier.
    pub id: String,
    /// Support node.
    pub node: NodeId,
    /// Constrained DOFs.
    pub restrained: Vec<Dof>,
}

/// Nodal action for a case, expressed in global axes.
#[derive(Clone, Debug)]
pub struct NodalLoad {
    /// Node to which the load is applied.
    pub node: NodeId,
    /// Forces or moments associated with nodal components.
    pub values: Vec<DofValue>,
}

/// Load case identified by a custom name.
#[derive(Clone, Debug)]
pub struct LoadCase {
    /// Case name, used as an identifier.
    pub name: String,
    /// Nodal actions for the case.
    pub loads: Vec<NodalLoad>,
}

/// Term in a combination of cases.
#[derive(Clone, Debug)]
pub struct CaseFactor {
    /// Name of the referenced case.
    pub case: String,
    /// Multiplying factor for the case's actions.
    pub factor: Real,
}

/// Declarative linear combination of load cases.
#[derive(Clone, Debug)]
pub struct Combination {
    /// Custom combination identifier.
    pub id: String,
    /// Cases and their factors, preserved in input order.
    pub terms: Vec<CaseFactor>,
}

/// Masses and inertias concentrated at a node.
#[derive(Clone, Debug)]
pub struct NodalMass {
    /// Custom mass assignment identifier.
    pub id: String,
    /// Receiving node.
    pub node: NodeId,
    /// Mass or inertia values per nodal component.
    pub values: Vec<DofValue>,
}

/// Domain collections, without implicit validation or replacement of duplicates.
///
/// Identifiers are unique within each class, not across classes.
/// Public lists allow building and correcting entries before validation.
#[derive(Clone, Debug, Default)]
pub struct Model {
    /// Nodes in insertion order.
    pub nodes: Vec<Node>,
    /// Members in insertion order.
    pub members: Vec<Member>,
    /// Sections in insertion order.
    pub sections: Vec<Section>,
    /// Laws in insertion order.
    pub laws: Vec<Law>,
    /// Supports in insertion order.
    pub supports: Vec<Support>,
    /// Cases in insertion order.
    pub cases: Vec<LoadCase>,
    /// Combinations in insertion order.
    pub combinations: Vec<Combination>,
    /// Masses in insertion order.
    pub masses: Vec<NodalMass>,
}

impl Model {
    /// Constructs an empty model, without diagnostics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a node without validation, preserving possible duplicates.
    pub fn insert_node(&mut self, node: Node) -> &mut Self {
        self.nodes.push(node);
        self
    }
    /// Inserts a member without validating its references or geometry.
    pub fn insert_member(&mut self, member: Member) -> &mut Self {
        self.members.push(member);
        self
    }
    /// Inserts a section without validating its properties.
    pub fn insert_section(&mut self, section: Section) -> &mut Self {
        self.sections.push(section);
        self
    }
    /// Inserts a law without validating its parameters.
    pub fn insert_law(&mut self, law: Law) -> &mut Self {
        self.laws.push(law);
        self
    }
    /// Inserts a support without validating the node or DOFs.
    pub fn insert_support(&mut self, support: Support) -> &mut Self {
        self.supports.push(support);
        self
    }
    /// Inserts a case without validating its actions.
    pub fn insert_case(&mut self, case: LoadCase) -> &mut Self {
        self.cases.push(case);
        self
    }
    /// Inserts a combination without validating cases or factors.
    pub fn insert_combination(&mut self, combination: Combination) -> &mut Self {
        self.combinations.push(combination);
        self
    }
    /// Inserts a mass without validating values, node, or DOFs.
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

/// Inspects the complete model without modifying it and returns all diagnostics.
///
/// Deterministic order for the same input: nodes, members, sections, laws,
/// supports, cases, combinations, and masses; within each list, insertion order.
/// For each entity, duplication is reported first, then checks in field order.
/// Each subsequent repetition of an id produces a diagnostic. Ambiguous
/// references use the first declaration to continue the checks. A node
/// is orphaned if its id appears at no endpoint, even if that member has
/// other broken references. A support, load, or mass does not prevent orphan status.
/// Length is not evaluated with missing endpoints or nonfinite coordinates.
///
/// `entities` identifies the culprit: node, member, law, or case. For supports and
/// masses, their node is used (even if absent); for sections, the first member
/// referencing them and, if there is none, `Law(id_sección)` as a nominal reference.
/// For combinations, the case of the offending term is used, or the first in duplicate
/// errors; if there are no terms, `Case(id_combinación)` is a nominal reference.
/// These nominal references do not assert the existence of a law or case with the same name.
/// `hint` always includes the class and exact name/id of the original entity.
///
/// Duplicate identifiers:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// m.insert_law(Law { name: "ley".into(), parameters: vec![] });
/// m.insert_law(m.laws[0].clone());
/// let d = validate(&m);
/// assert_eq!(d[0].code, "ENG-E-0002");
/// assert!(matches!(&d[0].entities[0], EntityRef::Law(n) if n == "ley"));
/// ```
/// Broken references:
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
/// Orphan node:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// m.insert_node(Node::new(NodeId(3), Dim::D2, [0.0; 3]));
/// let d = validate(&m);
/// assert_eq!(d[0].code, "ENG-E-0004");
/// assert!(matches!(d[0].entities[0], EntityRef::Node(NodeId(3))));
/// ```
/// Zero length, including the tolerance boundary:
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
/// Nonfinite values:
/// ```
/// use engine_model::*;
/// let mut m = Model::new();
/// m.insert_law(Law { name: "ley".into(), parameters: vec![
///     Parameter { name: "rigidez".into(), value: f64::NAN }] });
/// let d = validate(&m);
/// assert_eq!(d[0].code, "ENG-E-0008");
/// assert!(matches!(&d[0].entities[0], EntityRef::Law(n) if n == "ley"));
/// ```
/// Invalid DOF:
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
        let context = format!("Node {}", node.id.0);
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
                format!("{context}, coordinate {}", ["X", "Y", "Z"][axis]),
            );
        }
        for dof in &node.dofs {
            component(
                &mut out,
                Some(&node),
                *dof,
                entity.clone(),
                format!("{context}, DOF {dof}"),
            );
        }
        if !connected.contains(&node.id) {
            report(&mut out, "ENG-E-0004", entity, context);
        }
    }
    let mut seen = BTreeSet::new();
    for member in &model.members {
        let entity = EntityRef::Member(member.id);
        let context = format!("Member {}", member.id.0);
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
                    format!("{context}, end {}: missing node {}", ["i", "j"][end], id.0),
                );
            }
        }
        if !sections.contains(member.section.as_str()) {
            report(
                &mut out,
                "ENG-E-0003",
                entity.clone(),
                format!("{context}, missing section {:?}", member.section),
            );
        }
        if !laws.contains(member.law.as_str()) {
            report(
                &mut out,
                "ENG-E-0003",
                entity.clone(),
                format!("{context}, missing law {:?}", member.law),
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
                    .fold(0.0_f64, |length, (a, b)| math::hypot(length, a - b));
                if distance <= LENGTH_TOLERANCE {
                    report(
                        &mut out,
                        "ENG-E-0005",
                        entity,
                        format!("{context}, length {distance} <= tolerance {LENGTH_TOLERANCE}"),
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
        let context = format!("Section {:?}", section.id);
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
                format!("{context}, property {:?}", parameter.name),
            );
        }
    }
    let mut seen = BTreeSet::new();
    for law in &model.laws {
        let entity = EntityRef::Law(law.name.clone());
        let context = format!("Law {:?}", law.name);
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
                format!("{context}, parameter {:?}", parameter.name),
            );
        }
    }
    let mut seen = BTreeSet::new();
    for support in &model.supports {
        let entity = EntityRef::Node(support.node);
        let context = format!("Support {:?}, node {}", support.id, support.node.0);
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
                format!("{context}, DOF {dof}"),
            );
        }
    }
    let mut seen = BTreeSet::new();
    for case in &model.cases {
        let entity = EntityRef::Case(case.name.clone());
        let context = format!("Case {:?}", case.name);
        duplicate(
            &mut seen,
            &case.name,
            &mut out,
            entity.clone(),
            context.clone(),
        );
        for (index, load) in case.loads.iter().enumerate() {
            let context = format!("{context}, load {index}, node {}", load.node.0);
            if !nodes.contains_key(&load.node) {
                report(&mut out, "ENG-E-0003", entity.clone(), context.clone());
            }
            for value in &load.values {
                let context = format!("{context}, DOF {}", value.dof);
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
        let context = format!("Combination {:?}", combination.id);
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
            let context = format!("{context}, term {index}, case {:?}", term.case);
            if !cases.contains(term.case.as_str()) {
                report(&mut out, "ENG-E-0003", entity.clone(), context.clone());
            }
            finite(&mut out, term.factor, entity, context);
        }
    }
    let mut seen = BTreeSet::new();
    for mass in &model.masses {
        let entity = EntityRef::Node(mass.node);
        let context = format!("Mass {:?}, node {}", mass.id, mass.node.0);
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
            let context = format!("{context}, DOF {}", value.dof);
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
