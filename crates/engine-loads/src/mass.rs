//! Masas e inercias nodales en SI y conversión explícita de fuerzas a masa.
//!
//! La masa nodal es diagonal por GDL: kg en traslaciones, kg·m² en giros.
//! Los aportes de distintos identificadores se suman, como en el ensamblador.
//! Las matrices elementales proceden de `Element::mass`; se incorporan con
//! `ElementContribution` y se consultan con `LinearSystem::global_mass`.
//! No se sustituyen matrices consistentes por sumas diagonales.
//!
//! La dirección de conversión selecciona el peso: m=(F·d̂)·factor/g_usuario
//! por nodo, después de sumar sus fuerzas. Una proyección negativa se rechaza;
//! una proyección exactamente cero produce masa cero. Esta masa escalar se
//! asigna a cada traslación declarada del nodo, sin inercias rotacionales.
//! La dirección no define una masa anisótropa. Los momentos no contribuyen al peso.
//! No hay gravedad, factores de norma ni combinaciones implícitos.
//!
//! No finitos/desbordamientos: ENG-E-0008; masas/factores negativos, gravedad
//! no positiva, dirección nula, proyección negativa o subdesbordamiento de
//! masa positiva: ENG-E-0010.
//! Referencias/GDL ausentes: ENG-E-0003/0011; ids repetidos: ENG-E-0002.
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

/// Valida referencias, GDL, ids únicos, finitud y masas/inercias no negativas.
/// También comprueba que la suma por GDL sea representable. Las componentes
/// repetidas se suman, incluso dentro de una asignación, igual que LinearSystem.
pub fn validate_nodal_masses(map: &DofMap, masses: &[NodalMass]) -> EResult<()> {
    assemble_nodal_masses(map, masses).map(|_| ())
}

/// Ensambla masas/inercias a la diagonal global por GDL, sin eliminar apoyos.
/// Una lista vacía devuelve ceros. No suma masa entre direcciones ni confunde
/// las inercias rotacionales con kg. Propaga los errores descritos en el módulo.
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

/// Ensambla únicamente las masas nodales a una matriz diagonal global CSR.
/// Ambos MassKind reciben esta misma contribución en LinearSystem; las masas
/// elementales se agregan allí, no se duplican en esta función.
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

/// Vector global de traslación rígida unitaria en una dirección normalizada.
/// Giros cero, incluidos los nodos apoyados. Cada nodo debe declarar las
/// traslaciones con componente no nula de la dirección (ENG-E-0011 si falta
/// alguna); así no se informa una masa total de una dirección truncada.
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

/// Masa total en kg por dirección: r_dᵀ M r_d sobre la matriz global completa.
/// Para masas nodales diagonales aporta Σm_n,a d̂_a²; para elementos de masa
/// traslacional isotrópica aporta Σrho·A·L. Las inercias nodales no contribuyen.
/// No usa la traza ni la suma de las diagonales de una matriz consistente.
/// ENG-E-0010 por tamaño o masa direccional negativa; ENG-E-0008 por desborde.
/// La matriz debe ser una masa física ya validada por su productor/ensamblador;
/// esta operación no certifica semidefinición positiva ni simetría.
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

/// Consulta la masa global de LinearSystem en la MassKind elegida por el usuario.
/// Incluye elementos y masas nodales; no reduce por apoyos ni modifica el sistema.
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

/// Convierte fuerza global N a kg: (F·d̂)·factor/g_usuario.
/// Factor no negativo y g positiva son obligatorios, ambos finitos. Admite
/// componentes de cualquier signo; normaliza la dirección sin desbordar su norma.
/// Proyección negativa: ENG-E-0010, incluso con factor cero, sin tolerancia.
/// Proyección exactamente cero (incluido -0.0) produce masa cero.
pub fn mass_from_force(
    force: [Real; 3],
    factor: Real,
    direction: [Real; 3],
    gravity: Real,
) -> EResult<Real> {
    conversion(factor, gravity)?;
    projected_mass(force, factor, unit(direction)?, gravity)
}

/// Convierte un vector global de fuerzas ya ensamblado a asignaciones nodales.
/// Proyecta la fuerza neta por nodo, sin compensar proyecciones entre nodos.
/// Momentos finitos se validan y no aportan masa.
/// Cada masa se asigna a todas las traslaciones declaradas, con id
/// `id_prefix:node:<id>`. El llamador debe elegir prefijos únicos al combinar
/// fuentes. No se añade masa de elemento ni se modifica el modelo.
/// ENG-E-0010 por tamaño o proyección nodal negativa; los errores de conversión
/// nodal incluyen Node. Propaga los errores de conversión y GDL.
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

/// Convierte las cargas nodales de un LoadCase a masas, con factor/dirección/g
/// explícitos. Prefijo `mass:case:<nombre>`. Valida el caso, incluidos sus
/// momentos y duplicados; no interpreta combinaciones ni cargas de tramo.
/// Rechaza proyección neta nodal negativa con ENG-E-0010 y entidades Node/Case.
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

/// Convierte un caso completo del sistema, incluidas fuerzas equivalentes de
/// elementos. Lee `global_force(case, 1)`; aplica el factor solicitado una sola
/// vez durante la conversión. No aplica apoyos ni cambia masas del sistema.
/// Suma aportes antes de proyectar; proyección neta nodal negativa produce
/// ENG-E-0010 con entidades Node/Case, incluso si el total del caso es gravitatorio.
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
