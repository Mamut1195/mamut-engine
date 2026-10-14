//! Numeración global estable de los GDL declarados por los nodos.
use crate::{error, finite};
use engine_core::{Dim, Dof, DofIndex, EResult, EntityRef, NodeId, Real};
use engine_model::Node;
use std::collections::{BTreeMap, BTreeSet};

/// Mapa inmutable nodo/GDL → índice global (base cero).
///
/// Cada nodo declara un subconjunto no vacío de las componentes de su dimensión.
/// El orden de entrada no afecta la numeración; Rz es la tercera componente D2,
/// aunque su índice espacial en engine-core sea cinco. La numeración reducida
/// se obtiene con [`crate::Constraints::reduced_index`].
#[derive(Clone, Debug)]
pub struct DofMap {
    labels: Vec<(NodeId, Dof)>,
    indices: BTreeMap<(NodeId, usize), DofIndex>,
    nodes: BTreeSet<NodeId>,
    geometry: BTreeMap<NodeId, (Dim, [Real; 3])>,
}

impl DofMap {
    /// Numera nodos por id ascendente y GDL por el orden de Dim.
    ///
    /// ENG-E-0002 por nodo/GDL duplicado; ENG-E-0011 por componente incompatible;
    /// ENG-E-0010 por nodos/GDL vacíos o más índices que u32; ENG-E-0008 por
    /// coordenadas no finitas. No valida miembros ni propiedades de elementos.
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

    /// Número total de GDL, incluidos los restringidos.
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Indica si no hay GDL; un mapa construido correctamente nunca está vacío.
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Coordenadas globales originales en m para brazos de fuerzas (D2: XY,
    /// Z=0 por convención; D3: XYZ). No transforma ni proyecta coordenadas.
    /// ENG-E-0003 por nodo ausente.
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

    /// Busca índice global; ENG-E-0003 por nodo ausente, ENG-E-0011 por GDL ausente.
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

    /// Etiqueta física de un índice global; ENG-E-0010 si está fuera de rango.
    pub fn label(&self, index: DofIndex) -> EResult<(NodeId, Dof)> {
        self.labels.get(index.0 as usize).copied().ok_or_else(|| {
            let mut e = error("ENG-E-0010", "Índice global fuera de rango.");
            e.entities.push(EntityRef::Dof(index));
            e
        })
    }
}
