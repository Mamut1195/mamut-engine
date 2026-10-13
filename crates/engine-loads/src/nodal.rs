//! Validación por caso y adaptación a las primitivas del ensamblador.
use crate::{error, finite, ImposedDisplacement, LoadCase};
use engine_assembly::{assemble::assemble_vector, DofMap};
use engine_core::{DofIndex, EResult, EntityRef, Real};
use std::collections::BTreeSet;

fn nodal_indices(map: &DofMap, case: &LoadCase) -> EResult<(Vec<DofIndex>, Vec<Real>)> {
    let mut seen = BTreeSet::new();
    let mut indices = Vec::new();
    let mut values = Vec::new();
    for load in &case.loads {
        // Un nodo inexistente se rechaza incluso si no contiene componentes.
        let exists = (0..map.len()).any(|i| {
            map.label(DofIndex(i as u32))
                .is_ok_and(|(node, _)| node == load.node)
        });
        if !exists {
            let mut e = error("ENG-E-0003", "Nodo de carga inexistente.");
            e.entities.push(EntityRef::Node(load.node));
            return Err(e);
        }
        for value in &load.values {
            finite(&[value.value])?;
            let index = map.global_index(load.node, value.dof)?;
            if !seen.insert(index) {
                return Err(error(
                    "ENG-E-0010",
                    "Carga repetida sobre el mismo nodo/GDL en el caso.",
                ));
            }
            indices.push(index);
            values.push(value.value);
        }
    }
    Ok((indices, values))
}

/// Valida las acciones de un caso contra los GDL declarados y su dimensión.
/// Una componente no puede repetirse entre cargas del mismo caso, ni entre
/// desplazamientos impuestos. Fuerza y desplazamiento sí pueden compartir GDL.
/// No modifica el modelo; añade la referencia del caso a cualquier error.
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

/// Ensambla las cargas nodales de un caso a un vector global N/N·m.
/// Reutiliza [`DofMap`] y [`engine_assembly::assemble::assemble_vector`].
/// Una lista vacía produce ceros; componentes repetidas producen ENG-E-0010.
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

/// Vector global de traslaciones m y giros rad impuestos.
/// None distingue un GDL libre de un valor prescrito cero. Los mismos valores
/// pueden pasarse directamente a [`engine_assembly::Constraints::new`], pues
/// [`ImposedDisplacement`] es el tipo PrescribedDof existente.
/// Rechaza no finitos, referencias/GDL ausentes y duplicados (ENG-E-0010).
pub fn assemble_imposed_displacements(
    map: &DofMap,
    imposed: &[ImposedDisplacement],
) -> EResult<Vec<Option<Real>>> {
    let mut result = vec![None; map.len()];
    for p in imposed {
        finite(&[p.value])?;
        let index = map.global_index(p.node, p.dof)?.0 as usize;
        if result[index].replace(p.value).is_some() {
            let mut e = error("ENG-E-0010", "Desplazamiento impuesto repetido en el caso.");
            e.entities.push(EntityRef::Node(p.node));
            return Err(e);
        }
    }
    Ok(result)
}
