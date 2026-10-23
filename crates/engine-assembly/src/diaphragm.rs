//! Infinitesimal rigid XY diaphragm, normal +Z, exclusively D3.
//!
//! For r = x_esclavo − x_maestro: Ux_s = Ux_m − r_y Rz_m,
//! Uy_s = Uy_m + r_x Rz_m and Rz_s = Rz_m. Uz, Rx, and Ry retain
//! their independence; no artificial stiffness or penalty.
use crate::{error, finite, DofMap};
use engine_core::{Dim, Dof, DofIndex, EResult, EntityRef, NodeId, Real};
use std::collections::BTreeSet;

/// XY floor constraint on Ux, Uy, and Rz of each slave.
///
/// Master and slaves must declare those three D3 components and be at
/// the same elevation (absolute tolerance 1e-9 m). Does not allow D2, chains,
/// cycles, repeated masters, or shared slaves. Does not impose Uz/Rx/Ry.
#[derive(Clone, Debug)]
pub struct RigidDiaphragm {
    /// Independent node defining translations and rotation about +Z.
    pub master: NodeId,
    /// Dependent nodes; order only affects the force listing.
    pub slaves: Vec<NodeId>,
}

/// Diaphragm action on a node, positive in global axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiaphragmNodalForce {
    /// Node receiving the action.
    pub node: NodeId,
    /// [Fx, Fy, Mz] in N, N, N·m, relative to the node itself.
    pub values: [Real; 3],
}

/// Internal forces of a constraint, recovered from Ku − factor F.
///
/// At equilibrium, the action on each slave is its residual. The action
/// on the master is opposite to the sum of those residuals transferred
/// to the master: [ΣFx, ΣFy, Σ(Mz + r_x Fy − r_y Fx)] with opposite sign.
/// This resultant balances the diaphragm; it is neither a support reaction nor
/// the total external floor load. Away from equilibrium it is a residual-based
/// estimate, not a multiplier obtained from an equilibrated solution.
#[derive(Clone, Debug, PartialEq)]
pub struct DiaphragmForces {
    /// Resultant applied by the diaphragm to the master.
    pub master: DiaphragmNodalForce,
    /// Actions applied to slaves, in declaration order.
    pub slaves: Vec<DiaphragmNodalForce>,
}

pub(crate) type Relations = Vec<Vec<(DofIndex, Real)>>;

pub(crate) fn relations(map: &DofMap, diaphragms: &[RigidDiaphragm]) -> EResult<Relations> {
    let mut masters = BTreeSet::new();
    let mut slaves = BTreeSet::new();
    for d in diaphragms {
        if d.slaves.is_empty() || !masters.insert(d.master) {
            return Err(error("ENG-E-0010", "Empty diaphragm or duplicate master."));
        }
        for &slave in &d.slaves {
            if slave == d.master || !slaves.insert(slave) {
                let mut e = error("ENG-E-0010", "Slave equals master or duplicate slave.");
                e.entities.push(EntityRef::Node(slave));
                return Err(e);
            }
        }
    }
    if let Some(&node) = masters.intersection(&slaves).next() {
        let mut e = error(
            "ENG-E-0013",
            "Cycles and masters that are slaves of another diaphragm are not allowed.",
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
            return Err(error("ENG-E-0013", "XY diaphragm supports only D3."));
        }
        let mx = map.global_index(d.master, Dof::Ux)?;
        let my = map.global_index(d.master, Dof::Uy)?;
        let mz = map.global_index(d.master, Dof::Rz)?;
        for &slave in &d.slaves {
            let (dim, coords) = map.geometry(slave)?;
            if dim != Dim::D3 {
                return Err(error("ENG-E-0013", "XY diaphragm supports only D3."));
            }
            let [rx, ry, rz] = std::array::from_fn(|i| coords[i] - master[i]);
            finite(&[rx, ry, rz])?;
            if rz.abs() > 1e-9 {
                return Err(error(
                    "ENG-E-0010",
                    "Diaphragm nodes are not on an XY floor.",
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
