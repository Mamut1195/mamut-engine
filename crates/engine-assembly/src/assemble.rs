//! Aportes en ejes globales, ensamblados sin transformación ni elemento finito.
use crate::DofMap;
use crate::{error, finite, size};
use engine_contracts::CaseRef;
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_core::{EntityRef, NodeId};
use engine_linalg::{DMat, LinAlgBackend, SparseMatrix, SymMatrix, Triplet};
use engine_model::Model;
use std::collections::{BTreeMap, BTreeSet};

/// Fuerzas nodales equivalentes de un elemento para un caso declarado.
#[derive(Clone, Debug)]
pub struct ElementLoad {
    /// Caso exacto, sin normalización de nombre ni caso implícito.
    pub case: CaseRef,
    /// Vector global en el orden de location: N para traslaciones, N·m para giros.
    pub values: Vec<Real>,
}

/// Interfaz de entrada de un aporte elemental lineal, sin evaluar Element.
///
/// Cada fila/columna sigue location. El llamador obtiene estas matrices desde
/// Element u otro productor y realiza previamente las transformaciones globales.
/// K/Kg: N/m, N, N·m según fila/columna; masas: kg, kg·m, kg·m².
/// Las matrices deben ser cuadradas, simétricas exactamente y finitas.
/// None en masa/Kg significa aporte nulo, sin inferir una masa de otra clase.
/// Kg es una matriz ya evaluada por el productor en su estado de referencia;
/// este sistema lineal la mantiene constante y no calcula axiales ni P-Delta.
#[derive(Clone, Debug)]
pub struct ElementContribution {
    /// Índices globales distintos en el orden de los vectores y matrices.
    pub location: Vec<DofIndex>,
    /// Rigidez global elemental K.
    pub stiffness: DMat,
    /// Masa concentrada global, si el productor la proporciona.
    pub mass_lumped: Option<DMat>,
    /// Masa consistente global, si el productor la proporciona.
    pub mass_consistent: Option<DMat>,
    /// Rigidez geométrica global en un estado de referencia explícito.
    pub geometric: Option<DMat>,
    /// Fuerzas equivalentes por caso; los aportes repetidos se suman en este orden.
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

/// Ensambla bloques simétricos mediante tripletas del backend.
///
/// blocks contiene (ubicación global, matriz elemental). Se emiten ambas
/// mitades y ceros explícitos, en orden de bloques, fila y columna. Las sumas
/// coincidentes siguen ese orden determinista; no se itera HashMap.
/// ENG-E-0010 por tamaños/índices/asimetría; ENG-E-0002 por ubicación duplicada;
/// ENG-E-0008 por datos o acumulaciones no finitos. La salida conserva ceros
/// estructurales para construir el patrón, incluso en cancelaciones numéricas.
pub fn assemble_matrix<B: LinAlgBackend>(
    backend: &mut B,
    order: usize,
    blocks: &[(&[DofIndex], &DMat)],
) -> EResult<SparseMatrix> {
    let mut entries = Vec::new();
    for &(indices, matrix) in blocks {
        location(order, indices)?;
        size(matrix.rows(), indices.len())?;
        size(matrix.cols(), indices.len())?;
        SymMatrix::new(matrix.clone())?;
        for (i, row) in indices.iter().enumerate() {
            for (j, col) in indices.iter().enumerate() {
                entries.push(Triplet {
                    row: row.0 as usize,
                    col: col.0 as usize,
                    value: matrix.get(i, j).expect("Dimensiones validadas"),
                });
            }
        }
    }
    backend.assemble(order, order, &entries)
}

/// Suma un vector elemental global a out en su ubicación (N/N·m).
///
/// ENG-E-0010 por tamaño/índices, ENG-E-0002 por ubicación duplicada,
/// ENG-E-0008 por entradas o sumas no finitas. Ante error out queda intacto.
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

/// Matrices y fuerzas globales antes de aplicar la partición libre/prescrita.
pub(crate) struct GlobalAssembly {
    pub k: SparseMatrix,
    pub lumped: SparseMatrix,
    pub consistent: SparseMatrix,
    pub kg: SparseMatrix,
    pub forces: BTreeMap<String, Vec<Real>>,
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
        let force = forces.get_mut(&case.name).expect("Casos registrados");
        for load in &case.loads {
            // Verifica referencias incluso si la lista de componentes está vacía.
            check_node(map, load.node)?;
            for value in &load.values {
                assemble_vector(
                    force,
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
    })
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
