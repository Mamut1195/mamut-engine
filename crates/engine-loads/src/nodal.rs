//! Per-case validation and adaptation to assembler primitives.
use crate::{error, finite, ImposedDisplacement, LoadCase};
use engine_assembly::{assemble::assemble_vector, DofMap};
use engine_core::{DofIndex, EResult, EntityRef, Real};
use std::collections::BTreeSet;

fn nodal_indices(map: &DofMap, case: &LoadCase) -> EResult<(Vec<DofIndex>, Vec<Real>)> {
    let mut seen = BTreeSet::new();
    let mut indices = Vec::new();
    let mut values = Vec::new();
    for load in &case.loads {
        // A nonexistent node is rejected even if it contains no components.
        let exists = (0..map.len()).any(|i| {
            map.label(DofIndex(i as u32))
                .is_ok_and(|(node, _)| node == load.node)
        });
        if !exists {
            let mut e = error("ENG-E-0003", "Load node does not exist.");
            e.entities.push(EntityRef::Node(load.node));
            return Err(e);
        }
        for value in &load.values {
            finite(&[value.value])?;
            let index = map.global_index(load.node, value.dof)?;
            if !seen.insert(index) {
                return Err(error(
                    "ENG-E-0010",
                    "Duplicate load on the same node/DOF in the case.",
                ));
            }
            indices.push(index);
            values.push(value.value);
        }
    }
    Ok((indices, values))
}

/// Validates the actions of a case against the declared DOFs and their dimension.
/// A component cannot repeat across loads in the same case or across
/// prescribed displacements. Force and displacement may share a DOF.
/// Does not modify the model; adds the case reference to any error.
pub fn validate_case(
    map: &DofMap,
    case: &LoadCase,
    imposed: &[ImposedDisplacement],
) -> EResult<()> {
    (|| {
        nodal_indices(map, case)?;
        assemble_imposed_displacements(map, imposed)?;
        Ok(())
    })()
    .map_err(|mut e: engine_core::EngineError| {
        e.entities.push(EntityRef::Case(case.name.clone()));
        e
    })
}

/// Assembles the nodal loads of a case into a global vector in N/N·m.
/// Reuses [`DofMap`] and [`engine_assembly::assemble::assemble_vector`].
/// An empty list produces zeros; repeated components produce ENG-E-0010.
pub fn assemble_nodal_loads(map: &DofMap, case: &LoadCase) -> EResult<Vec<Real>> {
    (|| {
        let (indices, values) = nodal_indices(map, case)?;
        let mut result = vec![0.0; map.len()];
        if !indices.is_empty() {
            assemble_vector(&mut result, &indices, &values)?;
        }
        Ok(result)
    })()
    .map_err(|mut e: engine_core::EngineError| {
        e.entities.push(EntityRef::Case(case.name.clone()));
        e
    })
}

/// Global vector of prescribed translations in m and rotations in rad.
/// None distinguishes a free DOF from a prescribed zero value. The same values
/// can be passed directly to [`engine_assembly::Constraints::new`], since
/// [`ImposedDisplacement`] is the existing PrescribedDof type.
/// Rejects nonfinite values, missing references/DOFs, and duplicates (ENG-E-0010).
pub fn assemble_imposed_displacements(
    map: &DofMap,
    imposed: &[ImposedDisplacement],
) -> EResult<Vec<Option<Real>>> {
    let mut result = vec![None; map.len()];
    for p in imposed {
        finite(&[p.value])?;
        let index = map.global_index(p.node, p.dof)?.0 as usize;
        if result[index].replace(p.value).is_some() {
            let mut e = error("ENG-E-0010", "Duplicate imposed displacement in the case.");
            e.entities.push(EntityRef::Node(p.node));
            return Err(e);
        }
    }
    Ok(result)
}
