//! Physical recovery of reactions on nodes, positive in global axes.
use crate::{assemble::multiply, finite, size, Constraints, DofMap};
use engine_core::{Dof, EResult, NodeId, Real};
use engine_linalg::SparseMatrix;

/// Reaction on a prescribed DOF (support or imposed displacement); sign
/// positive according to the global DOF.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reaction {
    /// Node with a prescribed DOF.
    pub node: NodeId,
    /// Prescribed component (constrained or with an imposed value).
    pub dof: Dof,
    /// Force in N or moment in N·m: `(Tᵀ(K u - F))_p`.
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
    let residual: Vec<_> = internal.iter().zip(force).map(|(k, f)| k - f).collect();
    let projected = constraints.project_independent(&residual)?;
    let mut result = Vec::new();
    for (i, p) in constraints.prescribed_values().iter().enumerate() {
        if p.is_some() {
            let (node, dof) = map.label(engine_core::DofIndex(i as u32))?;
            let value = projected[i];
            finite(&[value])?;
            result.push(Reaction { node, dof, value });
        }
    }
    Ok(result)
}
