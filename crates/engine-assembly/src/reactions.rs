//! Recuperación física de reacciones sobre los nodos, positivas en ejes globales.
use crate::{assemble::multiply, finite, size, Constraints, DofMap};
use engine_core::{Dof, EResult, NodeId, Real};
use engine_linalg::SparseMatrix;

/// Reacción sobre un GDL prescrito (apoyo o desplazamiento impuesto); signo
/// positivo según el GDL global.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reaction {
    /// Nodo con GDL prescrito.
    pub node: NodeId,
    /// Componente prescrita (restringida o con valor impuesto).
    pub dof: Dof,
    /// Fuerza en N o momento en N·m: `(K u - F)_p`.
    pub value: Real,
}

pub(crate) fn recover(
    map: &DofMap,
    constraints: &Constraints,
    k: &SparseMatrix,
    u_free: &[Real],
    force: &[Real],
) -> EResult<Vec<Reaction>> {
    size(force.len(), map.len())?;
    finite(force)?;
    let u = constraints.expand(u_free)?;
    let internal = multiply(k, &u)?;
    let mut result = Vec::new();
    for (i, p) in constraints.prescribed_values().iter().enumerate() {
        if p.is_some() {
            let (node, dof) = map.label(engine_core::DofIndex(i as u32))?;
            let value = internal[i] - force[i];
            finite(&[value])?;
            result.push(Reaction { node, dof, value });
        }
    }
    Ok(result)
}
