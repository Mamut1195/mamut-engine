//! Diafragma infinitesimal rígido XY, normal +Z, exclusivamente D3.
//!
//! Para r = x_esclavo − x_maestro: Ux_s = Ux_m − r_y Rz_m,
//! Uy_s = Uy_m + r_x Rz_m y Rz_s = Rz_m. Uz, Rx y Ry conservan
//! su independencia; no hay rigidez artificial ni penalización.
use crate::{error, finite, DofMap};
use engine_core::{Dim, Dof, DofIndex, EResult, EntityRef, NodeId, Real};
use std::collections::BTreeSet;

/// Restricción de planta XY sobre Ux, Uy y Rz de cada esclavo.
///
/// Maestro y esclavos deben declarar esas tres componentes D3 y estar a
/// la misma cota (tolerancia absoluta 1e-9 m). No admite D2, cadenas,
/// ciclos, maestros repetidos ni esclavos compartidos. No impone Uz/Rx/Ry.
#[derive(Clone, Debug)]
pub struct RigidDiaphragm {
    /// Nodo independiente que define traslaciones y giro alrededor de +Z.
    pub master: NodeId,
    /// Nodos dependientes; el orden solo afecta al listado de fuerzas.
    pub slaves: Vec<NodeId>,
}

/// Acción del diafragma sobre un nodo, positiva en ejes globales.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiaphragmNodalForce {
    /// Nodo receptor de la acción.
    pub node: NodeId,
    /// [Fx, Fy, Mz] en N, N, N·m, respecto al propio nodo.
    pub values: [Real; 3],
}

/// Fuerzas internas de una restricción, recuperadas de Ku − factor F.
///
/// En equilibrio, la acción sobre cada esclavo es su residuo. La acción
/// sobre el maestro es la opuesta a la suma de esos residuos transportados
/// al maestro: [ΣFx, ΣFy, Σ(Mz + r_x Fy − r_y Fx)] con signo contrario.
/// Esta resultante equilibra el diafragma; no es una reacción de apoyo ni
/// la carga externa total del piso. Fuera de equilibrio es una estimación
/// por residuos, y no un multiplicador obtenido de una solución equilibrada.
#[derive(Clone, Debug, PartialEq)]
pub struct DiaphragmForces {
    /// Resultante aplicada por el diafragma sobre el maestro.
    pub master: DiaphragmNodalForce,
    /// Acciones aplicadas sobre los esclavos, en orden de declaración.
    pub slaves: Vec<DiaphragmNodalForce>,
}

pub(crate) type Relations = Vec<Vec<(DofIndex, Real)>>;

pub(crate) fn relations(map: &DofMap, diaphragms: &[RigidDiaphragm]) -> EResult<Relations> {
    let mut masters = BTreeSet::new();
    let mut slaves = BTreeSet::new();
    for d in diaphragms {
        if d.slaves.is_empty() || !masters.insert(d.master) {
            return Err(error("ENG-E-0010", "Diafragma vacío o maestro repetido."));
        }
        for &slave in &d.slaves {
            if slave == d.master || !slaves.insert(slave) {
                let mut e = error("ENG-E-0010", "Esclavo igual al maestro o esclavo repetido.");
                e.entities.push(EntityRef::Node(slave));
                return Err(e);
            }
        }
    }
    if let Some(&node) = masters.intersection(&slaves).next() {
        let mut e = error(
            "ENG-E-0013",
            "No se admiten ciclos ni maestros esclavos de otro diafragma.",
        );
        e.entities.push(EntityRef::Node(node));
        return Err(e);
    }
    let mut result: Relations = (0..map.len())
        .map(|i| vec![(DofIndex(i as u32), 1.0)])
        .collect();
    for d in diaphragms {
        let (dim, master) = map.geometry(d.master)?;
        if dim != Dim::D3 {
            return Err(error("ENG-E-0013", "El diafragma XY solo admite D3."));
        }
        let mx = map.global_index(d.master, Dof::Ux)?;
        let my = map.global_index(d.master, Dof::Uy)?;
        let mz = map.global_index(d.master, Dof::Rz)?;
        for &slave in &d.slaves {
            let (dim, coords) = map.geometry(slave)?;
            if dim != Dim::D3 {
                return Err(error("ENG-E-0013", "El diafragma XY solo admite D3."));
            }
            let [rx, ry, rz] = std::array::from_fn(|i| coords[i] - master[i]);
            finite(&[rx, ry, rz])?;
            if rz.abs() > 1e-9 {
                return Err(error(
                    "ENG-E-0010",
                    "Los nodos del diafragma no están en una planta XY.",
                ));
            }
            for (dof, terms) in [
                (Dof::Ux, vec![(mx, 1.0), (mz, -ry)]),
                (Dof::Uy, vec![(my, 1.0), (mz, rx)]),
                (Dof::Rz, vec![(mz, 1.0)]),
            ] {
                result[map.global_index(slave, dof)?.0 as usize] =
                    terms.into_iter().filter(|(_, a)| *a != 0.0).collect();
            }
        }
    }
    Ok(result)
}
