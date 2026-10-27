//! Nodal masses and inertias in SI and explicit force-to-mass conversion.
//!
//! Nodal mass is diagonal by DOF: kg for translations, kg·m² for rotations.
//! Contributions from different identifiers are summed, as in the assembler.
//! Element matrices come from `Element::mass`; they are incorporated through
//! `ElementContribution` and queried through `LinearSystem::global_mass`.
//! Consistent matrices are not replaced with diagonal sums.
//!
//! The conversion direction selects weight: m=(F·d̂)·factor/g_usuario
//! per node, after summing its forces. A negative projection is rejected;
//! an exactly zero projection produces zero mass. This scalar mass is
//! assigned to each declared translation of the node, without rotational inertias.
//! The direction does not define anisotropic mass. Moments do not contribute to weight.
//! There is no implicit gravity, code factor, or combination.
//!
//! Nonfinite values/overflow: ENG-E-0008; negative masses/factors, nonpositive
//! gravity, zero direction, negative projection, or underflow of
//! positive mass: ENG-E-0010.
//! Missing references/DOFs: ENG-E-0003/0011; repeated ids: ENG-E-0002.
//!
//! ```
//! use engine_assembly::DofMap;
//! use engine_core::{Dim, Dof, NodeId};
//! use engine_model::{Node, LoadCase, NodalLoad, DofValue};
//! use engine_loads::mass::{masses_from_case, assemble_nodal_mass_matrix, total_mass};
//! let node = Node::new(NodeId(1), Dim::D3, [0.0; 3]);
//! let map = DofMap::new(&[node])?;
//! let case = LoadCase { name: "peso".into(), loads: vec![NodalLoad {
//!     node: NodeId(1), values: vec![DofValue { dof: Dof::Uz, value: -120.0 }],
//! }] };
//! let masses = masses_from_case(&map, &case, 0.25, [0.0, 0.0, -1.0], 10.0)?;
//! let matrix = assemble_nodal_mass_matrix(&map, &masses)?;
//! assert!((total_mass(&map, &matrix, [1.0, 0.0, 0.0])? - 3.0).abs() < 1e-12);
//! # Ok::<(), engine_core::EngineError>(())
//! ```

use crate::{assemble_nodal_loads, error, finite};
use engine_assembly::{DofMap, LinearSystem};
use engine_contracts::{CaseRef, MassKind};
use engine_core::math;
use engine_core::{Dof, DofIndex, EResult, EntityRef, NodeId, Real};
use engine_linalg::{LinAlgBackend, SparseMatrix, Triplet};
use engine_model::{DofValue, LoadCase, NodalMass};
use std::collections::{BTreeMap, BTreeSet};

fn axis(dof: Dof) -> Option<usize> {
    match dof {
        Dof::Ux => Some(0),
        Dof::Uy => Some(1),
        Dof::Uz => Some(2),
        _ => None,
    }
}

fn zeros(n: usize) -> EResult<Vec<Real>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(n)
        .map_err(|_| error("ENG-E-0010", "Cannot allocate the mass vector."))?;
    values.resize(n, 0.0);
    Ok(values)
}

fn nonnegative(value: Real) -> EResult<()> {
    finite(&[value])?;
    if value < 0.0 {
        return Err(error("ENG-E-0010", "Negative mass or factor."));
    }
    Ok(())
}

fn unit(direction: [Real; 3]) -> EResult<[Real; 3]> {
    finite(&direction)?;
    let scale = direction.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
    if scale == 0.0 {
        return Err(error("ENG-E-0010", "Direction must be nonzero."));
    }
    let scaled = direction.map(|v| v / scale);
    let norm = math::hypot(math::hypot(scaled[0], scaled[1]), scaled[2]);
    Ok(scaled.map(|v| v / norm))
}

/// Validates references, DOFs, unique ids, finiteness, and nonnegative masses/inertias.
/// Also checks that the sum per DOF is representable. Repeated components
/// are summed, even within an assignment, as in LinearSystem.
pub fn validate_nodal_masses(map: &DofMap, masses: &[NodalMass]) -> EResult<()> {
    assemble_nodal_masses(map, masses).map(|_| ())
}

/// Assembles masses/inertias on the global diagonal by DOF, without removing supports.
/// An empty list returns zeros. Does not sum mass across directions or confuse
/// rotational inertias with kg. Propagates the errors described in the module.
pub fn assemble_nodal_masses(map: &DofMap, masses: &[NodalMass]) -> EResult<Vec<Real>> {
    let mut result = zeros(map.len())?;
    let nodes = (0..map.len())
        .map(|i| map.label(DofIndex(i as u32)).map(|(node, _)| node))
        .collect::<EResult<BTreeSet<_>>>()?;
    let mut ids = BTreeSet::new();
    for mass in masses {
        (|| {
            if !nodes.contains(&mass.node) {
                return Err(error("ENG-E-0003", "Mass node does not exist."));
            }
            if !ids.insert(&mass.id) {
                return Err(error("ENG-E-0002", "Duplicate mass identifier."));
            }
            for value in &mass.values {
                nonnegative(value.value)?;
                let i = map.global_index(mass.node, value.dof)?.0 as usize;
                result[i] += value.value;
                finite(&[result[i]])?;
            }
            Ok(())
        })()
        .map_err(|mut e: engine_core::EngineError| {
            e.entities.push(EntityRef::Node(mass.node));
            e
        })?;
    }
    Ok(result)
}

/// Assembles only nodal masses into a global diagonal CSR matrix.
/// Both MassKind variants receive this same contribution in LinearSystem; element
/// masses are added there and are not duplicated in this function.
pub fn assemble_nodal_mass_matrix(map: &DofMap, masses: &[NodalMass]) -> EResult<SparseMatrix> {
    let diagonal = assemble_nodal_masses(map, masses)?;
    let entries: Vec<_> = diagonal
        .into_iter()
        .enumerate()
        .map(|(i, value)| Triplet {
            row: i,
            col: i,
            value,
        })
        .collect();
    SparseMatrix::from_triplets(map.len(), map.len(), &entries)
}

/// Global unit rigid-translation vector in a normalized direction.
/// Zero rotations, including supported nodes. Each node must declare the
/// translations corresponding to nonzero direction components (ENG-E-0011 if any
/// are missing); this avoids reporting total mass for a truncated direction.
pub fn rigid_translation(map: &DofMap, direction: [Real; 3]) -> EResult<Vec<Real>> {
    let d = unit(direction)?;
    let mut result = zeros(map.len())?;
    let mut nodes = BTreeSet::new();
    for (i, value) in result.iter_mut().enumerate() {
        let (node, dof) = map.label(DofIndex(i as u32))?;
        nodes.insert(node);
        if let Some(a) = axis(dof) {
            *value = d[a];
        }
    }
    for node in nodes {
        for (a, dof) in [Dof::Ux, Dof::Uy, Dof::Uz].into_iter().enumerate() {
            if d[a] != 0.0 {
                map.global_index(node, dof)?;
            }
        }
    }
    Ok(result)
}

/// Total mass in kg per direction: r_dᵀ M r_d over the complete global matrix.
/// For diagonal nodal masses, contributes Σm_n,a d̂_a²; for elements with
/// isotropic translational mass, contributes Σrho·A·L. Nodal inertias do not contribute.
/// Does not use the trace or the diagonal sum of a consistent matrix.
/// ENG-E-0010 for size or negative directional mass; ENG-E-0008 for overflow.
/// The matrix must be a physical mass already validated by its producer/assembler;
/// this operation does not certify positive semidefiniteness or symmetry.
pub fn total_mass(map: &DofMap, matrix: &SparseMatrix, direction: [Real; 3]) -> EResult<Real> {
    if matrix.rows() != map.len() || matrix.cols() != map.len() {
        return Err(error(
            "ENG-E-0010",
            "Mass matrix is incompatible with DOFs.",
        ));
    }
    let r = rigid_translation(map, direction)?;
    let mut total = 0.0;
    for (row, &r_row) in r.iter().enumerate() {
        for p in matrix.row_offsets()[row]..matrix.row_offsets()[row + 1] {
            let value = matrix.values()[p];
            if matrix.column_indices()[p] == row {
                nonnegative(value)?;
            }
            total += (value * r_row) * r[matrix.column_indices()[p]];
            finite(&[total])?;
        }
    }
    nonnegative(total)?;
    Ok(total)
}

/// Queries the global mass of LinearSystem for the user-selected MassKind.
/// Includes elements and nodal masses; does not reduce for supports or modify the system.
pub fn system_total_mass<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    kind: MassKind,
    direction: [Real; 3],
) -> EResult<Real> {
    total_mass(system.dof_map(), system.global_mass(kind), direction)
}

fn conversion(factor: Real, gravity: Real) -> EResult<()> {
    nonnegative(factor)?;
    finite(&[gravity])?;
    if gravity <= 0.0 {
        return Err(error("ENG-E-0010", "g_usuario must be positive (m/s²)."));
    }
    Ok(())
}

fn projected_mass(force: [Real; 3], factor: Real, d: [Real; 3], gravity: Real) -> EResult<Real> {
    finite(&force)?;
    let projection = force.iter().zip(d).map(|(f, a)| f * a).sum::<Real>();
    finite(&[projection])?;
    if projection < 0.0 {
        return Err(error(
            "ENG-E-0010",
            "Net force has a projection opposite to the gravity direction.",
        ));
    }
    if factor == 0.0 {
        return Ok(0.0);
    }
    let weighted = projection * factor;
    let mass = weighted / gravity;
    finite(&[projection, weighted, mass])?;
    if projection > 0.0 && (weighted == 0.0 || mass == 0.0) {
        return Err(error("ENG-E-0010", "Positive mass lost to underflow."));
    }
    Ok(mass)
}

/// Converts global force in N to kg: (F·d̂)·factor/g_usuario.
/// A nonnegative factor and positive g are required, both finite. Accepts
/// components of either sign; normalizes the direction without overflowing its norm.
/// Negative projection: ENG-E-0010, even with a zero factor, without tolerance.
/// An exactly zero projection (including -0.0) produces zero mass.
pub fn mass_from_force(
    force: [Real; 3],
    factor: Real,
    direction: [Real; 3],
    gravity: Real,
) -> EResult<Real> {
    conversion(factor, gravity)?;
    projected_mass(force, factor, unit(direction)?, gravity)
}

/// Converts an already assembled global force vector to nodal assignments.
/// Projects the net force per node, without offsetting projections between nodes.
/// Finite moments are validated and do not contribute mass.
/// Each mass is assigned to all declared translations, with id
/// `id_prefix:node:<id>`. The caller must choose unique prefixes when combining
/// sources. No element mass is added and the model is not modified.
/// ENG-E-0010 for size or negative nodal projection; nodal conversion errors
/// include Node. Propagates conversion and DOF errors.
pub fn masses_from_load_vector(
    map: &DofMap,
    forces: &[Real],
    factor: Real,
    direction: [Real; 3],
    gravity: Real,
    id_prefix: &str,
) -> EResult<Vec<NodalMass>> {
    conversion(factor, gravity)?;
    let d = unit(direction)?;
    rigid_translation(map, direction)?;
    if forces.len() != map.len() {
        return Err(error(
            "ENG-E-0010",
            "Force vector is incompatible with DOFs.",
        ));
    }
    finite(forces)?;
    let mut nodes = BTreeMap::<NodeId, ([Real; 3], Vec<Dof>)>::new();
    for (i, &force) in forces.iter().enumerate() {
        let (node, dof) = map.label(DofIndex(i as u32))?;
        let entry = nodes.entry(node).or_default();
        if let Some(a) = axis(dof) {
            entry.0[a] = force;
            entry.1.push(dof);
        }
    }
    nodes
        .into_iter()
        .map(|(node, (force, dofs))| {
            let mass = projected_mass(force, factor, d, gravity).map_err(|mut e| {
                e.entities.push(EntityRef::Node(node));
                e
            })?;
            Ok(NodalMass {
                id: format!("{id_prefix}:node:{}", node.0),
                node,
                values: dofs
                    .into_iter()
                    .map(|dof| DofValue { dof, value: mass })
                    .collect(),
            })
        })
        .collect()
}

/// Converts the nodal loads of a LoadCase to masses, with explicit factor/direction/g.
/// Prefix `mass:case:<nombre>`. Validates the case, including its
/// moments and duplicates; does not interpret combinations or span loads.
/// Rejects negative net nodal projection with ENG-E-0010 and Node/Case entities.
pub fn masses_from_case(
    map: &DofMap,
    case: &LoadCase,
    factor: Real,
    direction: [Real; 3],
    gravity: Real,
) -> EResult<Vec<NodalMass>> {
    let forces = assemble_nodal_loads(map, case)?;
    masses_from_load_vector(
        map,
        &forces,
        factor,
        direction,
        gravity,
        &format!("mass:case:{}", case.name),
    )
    .map_err(|mut e| {
        e.entities.push(EntityRef::Case(case.name.clone()));
        e
    })
}

/// Converts a complete system case, including equivalent forces from
/// elements. Reads `global_force(case, 1)`; applies the requested factor only
/// once during conversion. Does not apply supports or change system masses.
/// Sums contributions before projecting; negative net nodal projection produces
/// ENG-E-0010 with Node/Case entities, even if the case total is gravitational.
pub fn masses_from_system_case<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    case: &CaseRef,
    factor: Real,
    direction: [Real; 3],
    gravity: Real,
) -> EResult<Vec<NodalMass>> {
    let forces = system.global_force(case, 1.0)?;
    masses_from_load_vector(
        system.dof_map(),
        &forces,
        factor,
        direction,
        gravity,
        &format!("mass:case:{}", case.0),
    )
    .map_err(|mut e| {
        e.entities.push(EntityRef::Case(case.0.clone()));
        e
    })
}
