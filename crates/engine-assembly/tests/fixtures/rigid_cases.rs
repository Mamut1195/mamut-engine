//! Derivación propia: dos apoyos elásticos y un diafragma XY excéntrico.
//! Fuente: provenance/ENG-ASM-002.md, integración con main (2026-10-13).
use engine_assembly::{DofMap, ElementContribution, LinearSystem, PrescribedDof, RigidDiaphragm};
use engine_contracts::CaseRef;
use engine_core::{Dim, Dof, EResult, NodeId};
use engine_linalg::{DMat, LinAlgBackend};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

const XY: [Dof; 3] = [Dof::Ux, Dof::Uy, Dof::Rz];
const POINTS: [[f64; 2]; 2] = [[1.0, 1.0], [3.0, 1.0]];

pub fn cases() -> Vec<(CaseRef, f64)> {
    [("A", 2.0), ("B", -1.0), ("cero", 0.0)]
        .into_iter()
        .map(|(name, factor)| (CaseRef(name.into()), factor))
        .collect()
}

pub fn system<B: LinAlgBackend>(backend: B, settlement: Option<f64>) -> EResult<LinearSystem<B>> {
    let mut model = Model::new();
    for (i, [x, y]) in POINTS.into_iter().enumerate() {
        let id = i as u32 + 1;
        // Resortes horizontales coplanares: cada anclaje coincide con su
        // esclavo para conservar también el equilibrio de momentos 3D.
        for (node, z, dofs) in [(id, 3.0, &XY[..2]), (id + 10, 3.0, &XY[..])] {
            let mut n = Node::new(NodeId(node), Dim::D3, [x, y, z]);
            n.dofs = dofs.to_vec();
            model.nodes.push(n);
        }
        model.supports.push(Support {
            id: format!("base{id}"),
            node: NodeId(id),
            restrained: XY[..2].to_vec(),
        });
    }
    let mut master = Node::new(NodeId(20), Dim::D3, [0.0, 0.0, 3.0]);
    master.dofs = XY.to_vec();
    model.nodes.push(master);
    for (name, node, values) in [("A", 11, [10.0, 20.0, 5.0]), ("B", 12, [-20.0, 10.0, -5.0])] {
        model.cases.push(LoadCase {
            name: name.into(),
            loads: vec![NodalLoad {
                node: NodeId(node),
                values: XY
                    .into_iter()
                    .zip(values)
                    .map(|(dof, value)| DofValue { dof, value })
                    .collect(),
            }],
        });
    }
    model.cases.push(LoadCase {
        name: "cero".into(),
        loads: vec![],
    });
    let map = DofMap::new(&model.nodes)?;
    let mut elements = Vec::new();
    for id in 1..=2 {
        for dof in &XY[..2] {
            elements.push(ElementContribution {
                location: vec![
                    map.global_index(NodeId(id), *dof)?,
                    map.global_index(NodeId(id + 10), *dof)?,
                ],
                stiffness: DMat::new(2, 2, vec![100.0, -100.0, -100.0, 100.0])?,
                mass_lumped: None,
                mass_consistent: None,
                geometric: None,
                loads: vec![],
            });
        }
    }
    let prescribed: Vec<_> = settlement
        .map(|value| PrescribedDof {
            node: NodeId(20),
            dof: Dof::Ux,
            value,
        })
        .into_iter()
        .collect();
    LinearSystem::from_model_with_diaphragms(
        backend,
        &model,
        &elements,
        &prescribed,
        &[RigidDiaphragm {
            master: NodeId(20),
            slaves: vec![NodeId(11), NodeId(12)],
        }],
    )
}

pub fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}

pub fn verify_solution<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    free: &[f64],
    case: &CaseRef,
    factor: f64,
    settlement: Option<f64>,
) -> EResult<()> {
    // K_ind = [[200,0,-200],[0,200,400],[-200,400,1200]].
    // Las cargas transportadas al maestro son (Fx,Fy,Mz).
    let [fx, fy, mz] = match case.0.as_str() {
        "A" => [10.0, 20.0, 15.0],
        "B" => [-20.0, 10.0, 45.0],
        _ => [0.0; 3],
    };
    let theta = match settlement {
        Some(a) => (factor * (mz - 2.0 * fy) + 200.0 * a) / 400.0,
        None => factor * (mz + fx - 2.0 * fy) / 200.0,
    };
    let ux = match settlement {
        Some(a) => a,
        None => factor * fx / 200.0 + theta,
    };
    let uy = factor * fy / 200.0 - 2.0 * theta;
    let global = system.constraints().expand(free)?;
    let value = |node, dof| -> EResult<f64> {
        Ok(global[system.dof_map().global_index(NodeId(node), dof)?.0 as usize])
    };
    for (dof, expected) in XY.into_iter().zip([ux, uy, theta]) {
        close(value(20, dof)?, expected);
    }
    for (i, [x, y]) in POINTS.into_iter().enumerate() {
        let id = i as u32 + 11;
        // Solución analítica y compatibilidad rígida con el maestro recuperado.
        close(value(id, Dof::Ux)?, ux - y * theta);
        close(value(id, Dof::Uy)?, uy + x * theta);
        close(value(id, Dof::Rz)?, theta);
        close(
            value(id, Dof::Ux)?,
            value(20, Dof::Ux)? - y * value(20, Dof::Rz)?,
        );
        close(
            value(id, Dof::Uy)?,
            value(20, Dof::Uy)? + x * value(20, Dof::Rz)?,
        );
        close(value(id, Dof::Rz)?, value(20, Dof::Rz)?);
        close(value(i as u32 + 1, Dof::Ux)?, 0.0);
        close(value(i as u32 + 1, Dof::Uy)?, 0.0);
    }
    let reactions = system.reactions(free, case, factor)?;
    assert_eq!(reactions.len(), if settlement.is_some() { 5 } else { 4 });
    for reaction in reactions {
        let expected = if reaction.node == NodeId(20) {
            assert_eq!(reaction.dof, Dof::Ux);
            // El maestro no tiene rigidez física: esta reacción procede de
            // los esclavos, no de una fila propia de K.
            200.0 * ux - 200.0 * theta - factor * fx
        } else {
            -100.0 * value(reaction.node.0 + 10, reaction.dof)?
        };
        close(reaction.value, expected);
    }
    Ok(())
}
