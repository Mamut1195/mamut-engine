//! Contributions in global axes, assembled without transformation or finite element.
use crate::DofMap;
use crate::{error, finite, size};
use engine_contracts::CaseRef;
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_core::{EntityRef, NodeId};
use engine_linalg::{DMat, LinAlgBackend, SparseMatrix, Triplet};
use engine_model::Model;
use std::collections::{BTreeMap, BTreeSet};

/// Equivalent nodal forces of an element for a declared case.
#[derive(Clone, Debug)]
pub struct ElementLoad {
    /// Exact case, without name normalization or an implicit case.
    pub case: CaseRef,
    /// Global vector in location order: N for translations, N·m for rotations.
    pub values: Vec<Real>,
}

/// Input interface for a linear element contribution, without evaluating Element.
///
/// Each row/column follows location. The caller obtains these matrices from
/// Element or another producer and performs global transformations beforehand.
/// K/Kg: N/m, N, N·m according to row/column; masses: kg, kg·m, kg·m².
/// Matrices must be square, exactly symmetric, and finite.
/// None for mass/Kg means zero contribution, without inferring mass of another type.
/// Kg is a matrix already evaluated by the producer at its reference state
/// (u=0 unless `LinearSystem::set_geometric_reference` is used);
/// it remains constant unless replaced by a registered GeometricProducer
/// with exactly the same location, including its order.
#[derive(Clone, Debug)]
pub struct ElementContribution {
    /// Distinct global indices in vector and matrix order.
    pub location: Vec<DofIndex>,
    /// Global element stiffness K.
    pub stiffness: DMat,
    /// Global lumped mass, if the producer provides it.
    pub mass_lumped: Option<DMat>,
    /// Global consistent mass, if the producer provides it.
    pub mass_consistent: Option<DMat>,
    /// Global geometric stiffness at an explicit reference state.
    pub geometric: Option<DMat>,
    /// Equivalent forces per case; repeated contributions are summed in this order.
    pub loads: Vec<ElementLoad>,
}

pub(crate) fn location(order: usize, indices: &[DofIndex]) -> EResult<()> {
    if indices.is_empty() {
        return Err(error("ENG-E-0010", "Aporte sin GDL."));
    }
    let mut seen = BTreeSet::new();
    for &d in indices {
        if d.0 as usize >= order {
            return Err(error("ENG-E-0010", "Ubicación elemental fuera de rango."));
        }
        if !seen.insert(d) {
            return Err(error("ENG-E-0002", "GDL duplicado en ubicación elemental."));
        }
    }
    Ok(())
}

/// Symmetry tolerance of [`assemble_matrix`]: exact (0), without implicit physical
/// tolerance. ASM-002 may use [`assemble_matrix_with_tolerance`].
pub const EXACT_SYMMETRY: Real = 0.0;

/// Assembles symmetric blocks with exact symmetry ([`EXACT_SYMMETRY`]).
///
/// Equivalent to [`assemble_matrix_with_tolerance`] with tolerance 0.
pub fn assemble_matrix<B: LinAlgBackend>(
    backend: &mut B,
    order: usize,
    blocks: &[(&[DofIndex], &DMat)],
) -> EResult<SparseMatrix> {
    assemble_matrix_with_tolerance(backend, order, blocks, EXACT_SYMMETRY)
}

/// Assembles symmetric blocks through backend triplets.
///
/// blocks contains (global location, element matrix). Both halves
/// and explicit zeros are emitted, in block, row, and column order. Coincident
/// sums follow that deterministic order; no HashMap is iterated.
/// `symmetry_tolerance` is relative and nonnegative: each block requires
/// `|a_ij − a_ji| ≤ tol · máx|a|` for the block. With 0, symmetry is exact. With
/// tol > 0 the mean `(a_ij + a_ji)/2` is emitted in both halves, so the
/// assembled matrix is exactly symmetric.
/// ENG-E-0010 for sizes/indices/asymmetry or negative/nonfinite tolerance;
/// ENG-E-0002 for duplicate location; ENG-E-0008 for nonfinite data or
/// accumulations. Output preserves structural zeros to construct the pattern,
/// even under numerical cancellation.
pub fn assemble_matrix_with_tolerance<B: LinAlgBackend>(
    backend: &mut B,
    order: usize,
    blocks: &[(&[DofIndex], &DMat)],
    symmetry_tolerance: Real,
) -> EResult<SparseMatrix> {
    if !symmetry_tolerance.is_finite() || symmetry_tolerance < 0.0 {
        return Err(error(
            "ENG-E-0010",
            "Tolerancia de simetría negativa o no finita.",
        ));
    }
    let mut entries = Vec::new();
    for &(indices, matrix) in blocks {
        location(order, indices)?;
        size(matrix.rows(), indices.len())?;
        size(matrix.cols(), indices.len())?;
        let at = |i: usize, j: usize| matrix.values()[i * matrix.cols() + j];
        let scale = matrix.values().iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        for i in 0..indices.len() {
            for j in 0..i {
                let (a, b) = (at(i, j), at(j, i));
                // NaN or infinities are ALWAYS rejected (including ∞ ≤ ∞) and the difference
                // is compared without overflow: |a−b| ≤ |a|+|b| may be ∞ with finite a,b.
                // The difference is computed directly (exact for subnormals: 5e-324
                // versus 0 is asymmetric with tol=0); only on overflow are halves compared.
                let diff = (a - b).abs();
                let symmetric = a.is_finite()
                    && b.is_finite()
                    && (a == b
                        || if diff.is_finite() {
                            diff <= symmetry_tolerance * scale
                        } else {
                            (a / 2.0 - b / 2.0).abs() <= 0.5 * symmetry_tolerance * scale
                        });
                if !symmetric {
                    return Err(error("ENG-E-0010", "Matriz no simétrica."));
                }
            }
        }
        for (i, row) in indices.iter().enumerate() {
            for (j, col) in indices.iter().enumerate() {
                let (a, b) = (at(i, j), at(j, i));
                // a == b is preserved exactly (including subnormals); the mean uses the direct
                // sum and only falls back to halves if the sum overflows.
                let value = if a == b || symmetry_tolerance == 0.0 {
                    a
                } else if (a + b).is_finite() {
                    (a + b) * 0.5
                } else {
                    a / 2.0 + b / 2.0
                };
                entries.push(Triplet {
                    row: row.0 as usize,
                    col: col.0 as usize,
                    value,
                });
            }
        }
    }
    backend.assemble(order, order, &entries)
}

/// Adds a global element vector to out at its location (N/N·m).
///
/// ENG-E-0010 for size/indices, ENG-E-0002 for duplicate location,
/// ENG-E-0008 for nonfinite inputs or sums. On error out remains intact.
pub fn assemble_vector(out: &mut [Real], indices: &[DofIndex], values: &[Real]) -> EResult<()> {
    location(out.len(), indices)?;
    size(values.len(), indices.len())?;
    finite(out)?;
    finite(values)?;
    let sums: Vec<_> = indices
        .iter()
        .zip(values)
        .map(|(i, v)| out[i.0 as usize] + v)
        .collect();
    finite(&sums)?;
    for (i, v) in indices.iter().zip(sums) {
        out[i.0 as usize] = v;
    }
    Ok(())
}

pub(crate) fn multiply(matrix: &SparseMatrix, u: &[Real]) -> EResult<Vec<Real>> {
    size(u.len(), matrix.cols())?;
    finite(u)?;
    let mut result = vec![0.0; matrix.rows()];
    for (row, value) in result.iter_mut().enumerate() {
        for p in matrix.row_offsets()[row]..matrix.row_offsets()[row + 1] {
            *value += matrix.values()[p] * u[matrix.column_indices()[p]];
            finite(&[*value])?;
        }
    }
    Ok(result)
}

/// Global matrices and forces before applying the free/prescribed partition.
pub(crate) struct GlobalAssembly {
    pub k: SparseMatrix,
    pub lumped: SparseMatrix,
    pub consistent: SparseMatrix,
    pub kg: SparseMatrix,
    pub forces: BTreeMap<String, Vec<Real>>,
    pub nodal_forces: BTreeMap<String, Vec<Real>>,
}

pub(crate) fn assemble_model<B: LinAlgBackend>(
    backend: &mut B,
    map: &DofMap,
    model: &Model,
    elements: &[ElementContribution],
) -> EResult<GlobalAssembly> {
    let n = map.len();
    let mut forces = BTreeMap::new();
    for case in &model.cases {
        if forces.insert(case.name.clone(), vec![0.0; n]).is_some() {
            let mut e = error("ENG-E-0002", "Caso duplicado.");
            e.entities.push(EntityRef::Case(case.name.clone()));
            return Err(e);
        }
    }
    let mut nodal_forces = forces.clone();
    for element in elements {
        non_negative_diagonal(&element.stiffness, "Rigidez elemental negativa.")?;
        for mass in [&element.mass_lumped, &element.mass_consistent]
            .into_iter()
            .flatten()
        {
            non_negative_diagonal(mass, "Masa elemental negativa.")?;
        }
    }
    let blocks: Vec<_> = elements
        .iter()
        .map(|e| (e.location.as_slice(), &e.stiffness))
        .collect();
    let k = assemble_matrix(backend, n, &blocks)?;
    let mut lumped = assemble_matrix(
        backend,
        n,
        &elements
            .iter()
            .filter_map(|e| e.mass_lumped.as_ref().map(|m| (e.location.as_slice(), m)))
            .collect::<Vec<_>>(),
    )?;
    let mut consistent = assemble_matrix(
        backend,
        n,
        &elements
            .iter()
            .filter_map(|e| {
                e.mass_consistent
                    .as_ref()
                    .map(|m| (e.location.as_slice(), m))
            })
            .collect::<Vec<_>>(),
    )?;
    let kg = assemble_matrix(
        backend,
        n,
        &elements
            .iter()
            .filter_map(|e| e.geometric.as_ref().map(|m| (e.location.as_slice(), m)))
            .collect::<Vec<_>>(),
    )?;
    for element in elements {
        for load in &element.loads {
            let force = forces
                .get_mut(&load.case.0)
                .ok_or_else(|| missing_case(&load.case))?;
            assemble_vector(force, &element.location, &load.values)?;
        }
    }
    for case in &model.cases {
        let force = forces
            .get_mut(&case.name)
            .ok_or_else(|| missing_case(&CaseRef(case.name.clone())))?;
        let nodal_force = nodal_forces
            .get_mut(&case.name)
            .ok_or_else(|| missing_case(&CaseRef(case.name.clone())))?;
        for load in &case.loads {
            // Checks references even if the component list is empty.
            check_node(map, load.node)?;
            for value in &load.values {
                assemble_vector(
                    force,
                    &[map.global_index(load.node, value.dof)?],
                    &[value.value],
                )?;
                assemble_vector(
                    nodal_force,
                    &[map.global_index(load.node, value.dof)?],
                    &[value.value],
                )?;
            }
        }
    }
    let mut mass_ids = BTreeSet::new();
    let mut nodal_mass = Vec::new();
    for mass in &model.masses {
        check_node(map, mass.node)?;
        if !mass_ids.insert(&mass.id) {
            return Err(error("ENG-E-0002", "Identificador de masa duplicado."));
        }
        for value in &mass.values {
            if value.value < 0.0 {
                let mut e = error("ENG-E-0010", "Masa nodal negativa.");
                e.entities.push(EntityRef::Node(mass.node));
                return Err(e);
            }
            let i = map.global_index(mass.node, value.dof)?.0 as usize;
            nodal_mass.push(Triplet {
                row: i,
                col: i,
                value: value.value,
            });
        }
    }
    for matrix in [&mut lumped, &mut consistent] {
        let mut entries = triplets(matrix);
        entries.extend_from_slice(&nodal_mass);
        *matrix = backend.assemble(n, n, &entries)?;
    }
    Ok(GlobalAssembly {
        k,
        lumped,
        consistent,
        kg,
        forces,
        nodal_forces,
    })
}

/// K and M are positive semidefinite: a negative diagonal contradicts this.
fn non_negative_diagonal(matrix: &DMat, message: &str) -> EResult<()> {
    for i in 0..matrix.rows().min(matrix.cols()) {
        if matrix.get(i, i).is_some_and(|v| v < 0.0) {
            return Err(error("ENG-E-0010", message));
        }
    }
    Ok(())
}

fn check_node(map: &DofMap, node: NodeId) -> EResult<()> {
    if map.contains_node(node) {
        Ok(())
    } else {
        let mut e = error("ENG-E-0003", "Nodo de acción/masa inexistente.");
        e.entities.push(EntityRef::Node(node));
        Err(e)
    }
}

pub(crate) fn triplets(matrix: &SparseMatrix) -> Vec<Triplet> {
    (0..matrix.rows())
        .flat_map(|row| {
            (matrix.row_offsets()[row]..matrix.row_offsets()[row + 1]).map(move |p| Triplet {
                row,
                col: matrix.column_indices()[p],
                value: matrix.values()[p],
            })
        })
        .collect()
}

pub(crate) fn missing_case(case: &CaseRef) -> EngineError {
    let mut e = error("ENG-E-0003", format!("Caso {} ausente.", case.0));
    e.entities.push(EntityRef::Case(case.0.clone()));
    e
}
