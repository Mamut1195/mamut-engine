//! Own derivation: congruence, virtual work, and exact constraints.
#[allow(dead_code)]
#[path = "../../../engine-assembly/tests/fixtures/rigid_cases.rs"]
mod rigid_cases;

use super::{multiply, reduce, residual, sum};
use engine_assembly::{ElementContribution, LinearSystem};
use engine_contracts::{CaseRef, SystemView};
use engine_core::{Dim, Dof, DofIndex, EResult, NodeId};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SparseMatrix, Triplet};
use engine_model::{Model, Node};

#[test]
fn congruence_is_symmetric_and_matches_exact_lagrange_constraints() -> EResult<()> {
    for settlement in [None, Some(0.03)] {
        let system = rigid_cases::system(ReferenceBackend, settlement)?;
        let n = system.stiffness().rows();
        let nf = system.n_free_dofs();
        // Heterogeneous Kg: includes couplings and forces at slaves/master.
        let entries: Vec<_> = (0..n)
            .flat_map(|i| {
                (0..n).map(move |j| Triplet {
                    row: i,
                    col: j,
                    value: if i == j { (i + 1) as f64 } else { 0.1 },
                })
            })
            .collect();
        let kg = SparseMatrix::from_triplets(n, n, &entries)?;
        let projected = reduce(&system, &kg)?;
        let u0 = system.constraints().expand(&vec![0.0; nf])?;
        // Obtains T through displacement perturbations, without using the reduction
        // routine or the sparse rows implementing the projection.
        let columns: Vec<_> = (0..nf)
            .map(|j| {
                let mut q = vec![0.0; nf];
                q[j] = 1.0;
                Ok(system
                    .constraints()
                    .expand(&q)?
                    .iter()
                    .zip(&u0)
                    .map(|(a, b)| a - b)
                    .collect::<Vec<_>>())
            })
            .collect::<EResult<_>>()?;
        for (i, ti) in columns.iter().enumerate() {
            for (j, tj) in columns.iter().enumerate() {
                let ktj = multiply(&kg, tj)?;
                let expected: f64 = ti.iter().zip(ktj).map(|(a, b)| a * b).sum();
                rigid_cases::close(projected.dense().get(i, j).unwrap(), expected);
                assert_eq!(
                    projected.dense().get(i, j).unwrap(),
                    projected.dense().get(j, i).unwrap()
                );
            }
        }
        let total = sum(system.stiffness(), &kg)?;
        let force = system.global_force(&CaseRef("A".into()), 2.0)?;
        let au0 = multiply(&total, &u0)?;
        let rhs: Vec<_> = force.iter().zip(au0).map(|(f, a)| f - a).collect();
        let reduced = reduce(&system, &total)?;
        let q = ReferenceBackend.solve_dense(
            reduced.dense(),
            &DMat::new(nf, 1, system.constraints().project_force(&rhs)?)?,
        )?;
        let u = system.constraints().expand(q.values())?;

        // KKT system [A Cᵀ; C 0], with all physical DOFs unsupported:
        // C directly imposes u_sx=u_mx-y*rz, u_sy=u_my+x*rz, rz_s=rz_m.
        let physical: Vec<_> = (0..n)
            .filter(|&i| system.constraints().prescribed_values()[i].is_none())
            .collect();
        let slaves: Vec<_> = physical
            .iter()
            .copied()
            .filter(|&i| {
                !system
                    .constraints()
                    .free_dofs()
                    .contains(&DofIndex(i as u32))
            })
            .collect();
        let np = physical.len();
        let nk = np + slaves.len();
        let mut kkt = vec![0.0; nk * nk];
        let mut rhs_kkt = vec![0.0; nk];
        let dense = total.to_dense()?;
        for (i, &gi) in physical.iter().enumerate() {
            rhs_kkt[i] = rhs[gi];
            for (j, &gj) in physical.iter().enumerate() {
                kkt[i * nk + j] = dense.get(gi, gj).unwrap();
            }
        }
        for (c, &slave) in slaves.iter().enumerate() {
            let (node, dof) = system.dof_map().label(DofIndex(slave as u32))?;
            let [x, y] = if node == NodeId(11) {
                [1.0, 1.0]
            } else {
                [3.0, 1.0]
            };
            let relation = match dof {
                Dof::Ux => vec![(Dof::Ux, -1.0), (Dof::Rz, y)],
                Dof::Uy => vec![(Dof::Uy, -1.0), (Dof::Rz, -x)],
                Dof::Rz => vec![(Dof::Rz, -1.0)],
                _ => unreachable!(),
            };
            for (j, &global) in physical.iter().enumerate() {
                let mut value = f64::from(global == slave);
                for &(master_dof, coefficient) in &relation {
                    if global == system.dof_map().global_index(NodeId(20), master_dof)?.0 as usize {
                        value += coefficient;
                    }
                }
                kkt[(np + c) * nk + j] = value;
                kkt[j * nk + np + c] = value;
            }
        }
        let exact =
            ReferenceBackend.solve_dense(&DMat::new(nk, nk, kkt)?, &DMat::new(nk, 1, rhs_kkt)?)?;
        for (i, &global) in physical.iter().enumerate() {
            rigid_cases::close(u[global], u0[global] + exact.values()[i]);
        }
        let internal = multiply(&total, &u)?;
        let (norm, _, ratio) = residual(&system, &total, &u, &internal, &force, 1e-12)?;
        assert!(norm < 1e-10);
        assert!(ratio <= 1.0);
        // The physical slave residual contains constraint forces:
        // only their projected virtual work must vanish.
        assert!(physical
            .iter()
            .any(|&i| (internal[i] - force[i]).abs() > 1.0));
    }
    Ok(())
}

#[test]
fn no_diaphragm_reduction_is_the_original_free_submatrix() -> EResult<()> {
    let mut model = Model::new();
    let mut node = Node::new(NodeId(1), Dim::D2, [0.0; 3]);
    node.dofs = vec![Dof::Ux, Dof::Uy];
    model.nodes.push(node);
    let element = ElementContribution {
        location: vec![DofIndex(0), DofIndex(1)],
        stiffness: DMat::new(2, 2, vec![3.0, 1.0, 1.0, 4.0])?,
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    };
    let system = LinearSystem::from_model(ReferenceBackend, &model, &[element], &[])?;
    assert_eq!(
        reduce(&system, system.stiffness())?,
        *system.reduced_stiffness()
    );
    let force = vec![2.0, -7.0];
    assert_eq!(system.constraints().project_force(&force)?, force);
    assert_eq!(system.constraints().expand(&force)?, force);
    Ok(())
}
