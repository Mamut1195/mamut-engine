//! Stable global numbering of DOFs declared by nodes.
use crate::{error, finite};
use engine_core::{Dim, Dof, DofIndex, EResult, EntityRef, NodeId, Real};
use engine_model::Node;
use std::collections::{BTreeMap, BTreeSet};

/// Immutable node/DOF → global index map (zero-based).
///
/// Each node declares a nonempty subset of its dimension's components.
/// Input order does not affect numbering; Rz is the third D2 component,
/// although its spatial index in engine-core is five. Reduced numbering
/// is obtained with [`crate::Constraints::reduced_index`].
#[derive(Clone, Debug)]
pub struct DofMap {
    labels: Vec<(NodeId, Dof)>,
    indices: BTreeMap<(NodeId, usize), DofIndex>,
    nodes: BTreeSet<NodeId>,
    geometry: BTreeMap<NodeId, (Dim, [Real; 3])>,
}

impl DofMap {
    /// Numbers nodes by ascending id and DOFs by Dim order.
    ///
    /// ENG-E-0002 for duplicate node/DOF; ENG-E-0011 for incompatible component;
    /// ENG-E-0010 for empty nodes/DOFs or more indices than u32; ENG-E-0008 for
    /// nonfinite coordinates. Does not validate members or element properties.
    pub fn new(nodes: &[Node]) -> EResult<Self> {
        if nodes.is_empty() {
            return Err(error("ENG-E-0010", "No hay nodos que numerar."));
        }
        let mut ordered = BTreeMap::new();
        for node in nodes {
            let result = (|| {
                if ordered.insert(node.id, node).is_some() {
                    return Err(error("ENG-E-0002", "Nodo duplicado en numeración."));
                }
                finite(&node.coordinates)?;
                if node.dofs.is_empty() {
                    return Err(error("ENG-E-0010", "Nodo sin GDL declarados."));
                }
                let mut seen = BTreeSet::new();
                for &dof in &node.dofs {
                    if !node.dim.dofs().contains(&dof) {
                        return Err(error("ENG-E-0011", format!("GDL {dof} incompatible.")));
                    }
                    if !seen.insert(dof.index()) {
                        return Err(error("ENG-E-0002", format!("GDL {dof} duplicado.")));
                    }
                }
                Ok(())
            })();
            result.map_err(|mut e| {
                e.entities.push(EntityRef::Node(node.id));
                e
            })?;
        }
        let mut result = Self {
            labels: Vec::new(),
            indices: BTreeMap::new(),
            nodes: ordered.keys().copied().collect(),
            geometry: ordered
                .iter()
                .map(|(&id, n)| (id, (n.dim, n.coordinates)))
                .collect(),
        };
        for (id, node) in ordered {
            for &dof in node.dim.dofs() {
                if node.dofs.contains(&dof) {
                    let index = u32::try_from(result.labels.len())
                        .map_err(|_| error("ENG-E-0010", "Numeración excede u32."))?;
                    result.indices.insert((id, dof.index()), DofIndex(index));
                    result.labels.push((id, dof));
                }
            }
        }
        Ok(result)
    }

    /// Total number of DOFs, including constrained ones.
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Indicates whether there are no DOFs; a correctly constructed map is never empty.
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Original global coordinates in m for force arms (D2: XY,
    /// Z=0 by convention; D3: XYZ). Does not transform or project coordinates.
    /// ENG-E-0003 for a missing node.
    pub fn coordinates(&self, node: NodeId) -> EResult<[Real; 3]> {
        self.geometry
            .get(&node)
            .map(|&(_, coordinates)| coordinates)
            .ok_or_else(|| {
                let mut e = error("ENG-E-0003", "Nodo sin coordenadas en el mapa.");
                e.entities.push(EntityRef::Node(node));
                e
            })
    }

    pub(crate) fn contains_node(&self, node: NodeId) -> bool {
        self.nodes.contains(&node)
    }

    pub(crate) fn geometry(&self, node: NodeId) -> EResult<(Dim, [Real; 3])> {
        self.geometry.get(&node).copied().ok_or_else(|| {
            let mut e = error("ENG-E-0003", "Nodo de diafragma inexistente.");
            e.entities.push(EntityRef::Node(node));
            e
        })
    }

    /// Looks up global index; ENG-E-0003 for a missing node, ENG-E-0011 for a missing DOF.
    pub fn global_index(&self, node: NodeId, dof: Dof) -> EResult<DofIndex> {
        self.indices
            .get(&(node, dof.index()))
            .copied()
            .ok_or_else(|| {
                let mut e = error(
                    if self.nodes.contains(&node) {
                        "ENG-E-0011"
                    } else {
                        "ENG-E-0003"
                    },
                    format!("Nodo {}, GDL {dof}.", node.0),
                );
                e.entities.push(EntityRef::Node(node));
                e
            })
    }

    /// Physical label of a global index; ENG-E-0010 if out of range.
    pub fn label(&self, index: DofIndex) -> EResult<(NodeId, Dof)> {
        self.labels.get(index.0 as usize).copied().ok_or_else(|| {
            let mut e = error("ENG-E-0010", "Índice global fuera de rango.");
            e.entities.push(EntityRef::Dof(index));
            e
        })
    }
}
