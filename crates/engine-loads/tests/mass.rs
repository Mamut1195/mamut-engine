//! Fixtures analíticos propios; ninguna fuente externa ni oráculo.
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, MassKind};
use engine_core::{Dim, Dof, DofIndex, EResult, EntityRef, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_elements_truss::LinearTruss;
use engine_linalg::{DMat, ReferenceBackend, SparseMatrix, Triplet};
use engine_loads::mass::{
    assemble_nodal_mass_matrix, assemble_nodal_masses, mass_from_force, masses_from_case,
    masses_from_load_vector, masses_from_system_case, rigid_translation, system_total_mass,
    total_mass, validate_nodal_masses,
};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, NodalMass, Node, Support};

const SOURCE: &str = "analítico: energía cinética, rho A L, traslación rígida y F/g";
const CITATION: &str = "provenance/ENG-MASS-001.md, (1)-(3); ENG-MASS-001-FU.md, (1)-(5); propias";

#[test]
fn sum_of_members_and_shared_node_mass_in_both_kinds() -> EResult<()> {
    let bar = LinearTruss::new(Dim::D2, 200.0, 2.0, 3.0)?;
    let coords = [[0.0; 3], [4.0, 0.0, 0.0], [4.0, 2.0, 0.0]];
    let mut m = Model::new();
    for i in [2, 0, 1] {
        let mut node = Node::new(NodeId(i as u32 + 1), Dim::D2, coords[i]);
        node.dofs = bar.dofs_per_node().to_vec();
        m.nodes.push(node);
    }
    m.masses
        .push(nodal("compartida", 2, &[(Dof::Ux, 5.0), (Dof::Uy, 5.0)]));
    let first = contribution(&bar, Dim::D2, &coords[..2])?;
    let mut second = contribution(&bar, Dim::D2, &coords[1..])?;
    second.location = (2..6).map(DofIndex).collect();
    let system = LinearSystem::from_model(ReferenceBackend, &m, &[first, second], &[])?;
    for kind in [MassKind::Lumped, MassKind::Consistent] {
        for d in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            close(
                system_total_mass(&system, kind, d)?,
                5.0 + 3.0 * 2.0 * (4.0 + 2.0),
            );
        }
    }
    Ok(())
}

fn close(actual: f64, expected: f64) {
    assert!(!SOURCE.is_empty() && !CITATION.is_empty());
    assert!(
        (actual - expected).abs() <= 3e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}

fn model(dim: Dim, coords: [[f64; 3]; 2], dofs: &[Dof]) -> Model {
    let mut m = Model::new();
    // Entrada invertida para verificar que la masa sigue DofMap, no la lista.
    for i in [1, 0] {
        let mut node = Node::new(NodeId(i as u32 + 1), dim, coords[i]);
        node.dofs = dofs.to_vec();
        m.nodes.push(node);
    }
    m.supports.push(Support {
        id: "base".into(),
        node: NodeId(1),
        restrained: dofs.to_vec(),
    });
    m
}

fn nodal(id: &str, node: u32, values: &[(Dof, f64)]) -> NodalMass {
    NodalMass {
        id: id.into(),
        node: NodeId(node),
        values: values
            .iter()
            .map(|&(dof, value)| DofValue { dof, value })
            .collect(),
    }
}

fn contribution(
    element: &dyn Element,
    dim: Dim,
    coords: &[[f64; 3]],
) -> EResult<ElementContribution> {
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim,
        coords,
        params: &params,
    };
    let n = element.dofs_per_node().len() * 2;
    Ok(ElementContribution {
        location: (0..n).map(|i| DofIndex(i as u32)).collect(),
        stiffness: element.respond(&ctx, &vec![0.0; n], &[])?.tangent,
        mass_lumped: element.mass(&ctx, MassKind::Lumped)?,
        mass_consistent: element.mass(&ctx, MassKind::Consistent)?,
        geometric: None,
        loads: vec![],
    })
}

#[test]
fn bar_rho_a_l_each_direction_both_kinds_and_rotations() -> EResult<()> {
    let bar = LinearTruss::new(Dim::D3, 200.0, 2.0, 3.0)?;
    // Cada vector tiene longitud 4; rotación y traslación conservan rho A L=24.
    for end in [
        [4.0, 0.0, 0.0],
        [0.0, -4.0, 0.0],
        [0.0, 0.0, 4.0],
        [4.0 / 3.0, 8.0 / 3.0, 8.0 / 3.0],
    ] {
        let origin = [2.0, -3.0, 5.0];
        let coords = [origin, std::array::from_fn(|i| origin[i] + end[i])];
        let m = model(Dim::D3, coords, bar.dofs_per_node());
        let e = contribution(&bar, Dim::D3, &coords)?;
        let system = LinearSystem::from_model(ReferenceBackend, &m, &[e], &[])?;
        for kind in [MassKind::Lumped, MassKind::Consistent] {
            for direction in [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [-1.0, 2.0, 2.0],
            ] {
                close(system_total_mass(&system, kind, direction)?, 24.0);
            }
            let dense = system.global_mass(kind).to_dense()?;
            let (diagonal, off) = match kind {
                MassKind::Lumped => (12.0, 0.0),
                MassKind::Consistent => (8.0, 4.0),
            };
            for a in 0..3 {
                close(dense.get(a, a).unwrap(), diagonal);
                close(dense.get(a, a + 3).unwrap(), off);
            }
        }
    }
    Ok(())
}

#[test]
fn beam_consistent_closed_entries_and_invariant_total() -> EResult<()> {
    for releases in [
        MomentReleases::default(),
        MomentReleases { i: true, j: false },
        MomentReleases { i: true, j: true },
    ] {
        let beam = EulerFrame2D::new(200.0, 2.0, 3.0, 3.0, releases)?;
        for end in [[4.0, 0.0, 0.0], [0.0, 4.0, 0.0], [2.4, 3.2, 0.0]] {
            let coords = [[0.0; 3], end];
            let m = model(Dim::D2, coords, beam.dofs_per_node());
            let e = contribution(&beam, Dim::D2, &coords)?;
            let system = LinearSystem::from_model(ReferenceBackend, &m, &[e], &[])?;
            for kind in [MassKind::Lumped, MassKind::Consistent] {
                for direction in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [3.0, -4.0, 0.0]] {
                    close(system_total_mass(&system, kind, direction)?, 24.0);
                }
            }
            if releases == MomentReleases::default() && end[0] == 4.0 {
                let mass = system.global_mass(MassKind::Consistent).to_dense()?;
                close(mass.get(0, 0).unwrap(), 8.0);
                close(mass.get(0, 3).unwrap(), 4.0);
                close(mass.get(1, 1).unwrap(), 24.0 * 156.0 / 420.0);
                close(mass.get(1, 4).unwrap(), 24.0 * 54.0 / 420.0);
                close(mass.get(1, 2).unwrap(), 24.0 * 22.0 * 4.0 / 420.0);
                close(mass.get(2, 2).unwrap(), 24.0 * 4.0 * 16.0 / 420.0);
                // La traza traslacional consistente NO es la masa física.
                assert!((mass.get(1, 1).unwrap() + mass.get(4, 4).unwrap() - 24.0).abs() > 1.0);
            }
        }
    }
    Ok(())
}

#[test]
fn nodal_diagonal_plus_element_and_inertias() -> EResult<()> {
    let beam = EulerFrame2D::new(200.0, 2.0, 3.0, 3.0, MomentReleases::default())?;
    let coords = [[0.0; 3], [4.0, 0.0, 0.0]];
    let mut m = model(Dim::D2, coords, beam.dofs_per_node());
    m.masses = vec![
        nodal("A", 1, &[(Dof::Ux, 7.0), (Dof::Uy, 11.0), (Dof::Rz, 19.0)]),
        nodal("B", 2, &[(Dof::Ux, 5.0), (Dof::Uy, 2.0)]),
        nodal("C", 1, &[(Dof::Ux, 1.0), (Dof::Ux, 2.0)]),
    ];
    let map = DofMap::new(&m.nodes)?;
    validate_nodal_masses(&map, &m.masses)?;
    assert_eq!(
        assemble_nodal_masses(&map, &m.masses)?,
        vec![10.0, 11.0, 19.0, 5.0, 2.0, 0.0]
    );
    let matrix = assemble_nodal_mass_matrix(&map, &m.masses)?;
    close(total_mass(&map, &matrix, [1.0, 0.0, 0.0])?, 15.0);
    close(total_mass(&map, &matrix, [0.0, 1.0, 0.0])?, 13.0);
    let e = contribution(&beam, Dim::D2, &coords)?;
    let system = LinearSystem::from_model(ReferenceBackend, &m, &[e], &[])?;
    for kind in [MassKind::Lumped, MassKind::Consistent] {
        close(system_total_mass(&system, kind, [1.0, 0.0, 0.0])?, 39.0);
        close(system_total_mass(&system, kind, [0.0, 1.0, 0.0])?, 37.0);
        close(
            system_total_mass(&system, kind, [3.0, 4.0, 0.0])?,
            24.0 + 15.0 * 9.0 / 25.0 + 13.0 * 16.0 / 25.0,
        );
    }
    Ok(())
}

#[test]
fn load_case_quarter_factor_minus_z_and_complete_system_case() -> EResult<()> {
    let coords = [[0.0; 3], [4.0, 0.0, 0.0]];
    let mut m = model(Dim::D3, coords, &[Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rz]);
    let case = LoadCase {
        name: "peso".into(),
        loads: vec![
            NodalLoad {
                node: NodeId(1),
                values: vec![
                    DofValue {
                        dof: Dof::Uz,
                        value: -120.0,
                    },
                    DofValue {
                        dof: Dof::Ux,
                        value: 30.0,
                    },
                    DofValue {
                        dof: Dof::Rz,
                        value: 999.0,
                    },
                ],
            },
            // Ambos nodos tienen peso en la dirección de gravedad.
            NodalLoad {
                node: NodeId(2),
                values: vec![DofValue {
                    dof: Dof::Uz,
                    value: -80.0,
                }],
            },
        ],
    };
    let map = DofMap::new(&m.nodes)?;
    let masses = masses_from_case(&map, &case, 0.25, [0.0, 0.0, -7.0], 10.0)?;
    for (mass, expected) in masses.iter().zip([3.0, 2.0]) {
        assert_eq!(mass.values.len(), 3);
        for value in &mass.values {
            close(value.value, expected);
        }
    }
    let matrix = assemble_nodal_mass_matrix(&map, &masses)?;
    for d in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]] {
        close(total_mass(&map, &matrix, d)?, 5.0);
    }
    m.cases.push(case.clone());
    m.masses = masses;
    let mut e = ElementContribution {
        location: vec![DofIndex(2), DofIndex(6)],
        stiffness: DMat::zeros(2, 2)?,
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![ElementLoad {
            case: CaseRef(case.name.clone()),
            values: vec![20.0, 40.0],
        }],
    };
    let system = LinearSystem::from_model(ReferenceBackend, &m, &[e.clone()], &[])?;
    let complete = masses_from_system_case(
        &system,
        &CaseRef(case.name.clone()),
        0.25,
        [0.0, 0.0, -1.0],
        10.0,
    )?;
    // Fuerzas netas -100 y -40: 2.5 + 1 = 3.5 kg en cada dirección.
    close(
        total_mass(
            &map,
            &assemble_nodal_mass_matrix(&map, &complete)?,
            [0.0, 0.0, 1.0],
        )?,
        3.5,
    );
    for kind in [MassKind::Lumped, MassKind::Consistent] {
        close(system_total_mass(&system, kind, [0.0, 0.0, 1.0])?, 5.0);
    }
    // Al añadir la fuente explícitamente se ensambla una vez, con ids únicos.
    m.masses = complete;
    e.loads.clear();
    let system = LinearSystem::from_model(ReferenceBackend, &m, &[e], &[])?;
    close(
        system_total_mass(&system, MassKind::Consistent, [1.0, 0.0, 0.0])?,
        3.5,
    );
    assert_eq!(
        masses_from_system_case(
            &system,
            &CaseRef("ausente".into()),
            0.25,
            [0.0, 0.0, -1.0],
            10.0
        )
        .unwrap_err()
        .code,
        "ENG-E-0003"
    );
    Ok(())
}

#[test]
fn force_projection_rotation_and_user_gravity() -> EResult<()> {
    close(
        mass_from_force([3.0, 4.0, 0.0], 0.25, [3.0, 4.0, 0.0], 10.0)?,
        0.125,
    );
    close(
        mass_from_force([-4.0, 3.0, 0.0], 0.25, [-4.0, 3.0, 0.0], 10.0)?,
        0.125,
    );
    close(
        mass_from_force([3.0, 4.0, 0.0], 0.25, [3.0, 4.0, 0.0], 5.0)?,
        0.25,
    );
    close(
        // Componentes simétricas: la cancelación es cero exacto en f64.
        mass_from_force([4.0, 4.0, 0.0], 0.25, [-1.0, 1.0, 0.0], 10.0)?,
        0.0,
    );
    close(
        mass_from_force([3.0, 4.0, 0.0], 0.0, [3.0, 4.0, 0.0], 10.0)?,
        0.0,
    );
    Ok(())
}

fn rejects_suction<T: std::fmt::Debug>(result: EResult<T>, entities: &[EntityRef]) {
    // Oráculos derivados a mano en provenance/ENG-MASS-001-FU.md, (1)-(3).
    let error = result.unwrap_err();
    assert_eq!(error.code, "ENG-E-0010");
    assert_eq!(error.entities.len(), entities.len());
    for (actual, expected) in error.entities.iter().zip(entities) {
        match (actual, expected) {
            (EntityRef::Node(a), EntityRef::Node(b)) => assert_eq!(a, b),
            (EntityRef::Case(a), EntityRef::Case(b)) => assert_eq!(a, b),
            _ => panic!("Entidad inesperada: {actual:?}; se esperaba {expected:?}"),
        }
    }
}

#[test]
fn suction_force_projection_rejects_every_negative_sign() -> EResult<()> {
    // W=120 N, g=10 m/s²: m=W/g=12 kg; d se normaliza a -Z.
    close(
        mass_from_force([0.0, 0.0, -120.0], 1.0, [0.0, 0.0, -7.0], 10.0)?,
        12.0,
    );
    // Sin tolerancia: hasta el menor subnormal negativo debe rechazarse,
    // antes de multiplicar por un factor que pudiera llevarlo a cero.
    for upward in [120.0, f64::MIN_POSITIVE, f64::from_bits(1)] {
        for factor in [0.0, 0.25, 1.0] {
            rejects_suction(
                mass_from_force([0.0, 0.0, upward], factor, [0.0, 0.0, -1.0], 10.0),
                &[],
            );
        }
    }
    for zero in [0.0, -0.0] {
        close(
            mass_from_force([0.0, 0.0, zero], 1.0, [0.0, 0.0, -1.0], 10.0)?,
            0.0,
        );
    }
    Ok(())
}

#[test]
fn suction_load_vector_rejects_a_node_despite_gravitational_global_sum() -> EResult<()> {
    let m = model(Dim::D3, [[0.0; 3], [1.0; 3]], &[Dof::Ux, Dof::Uy, Dof::Uz]);
    let map = DofMap::new(&m.nodes)?;
    let mut forces = vec![0.0; map.len()];
    let z1 = map.global_index(NodeId(1), Dof::Uz)?.0 as usize;
    let z2 = map.global_index(NodeId(2), Dof::Uz)?.0 as usize;
    forces[z1] = -120.0;
    forces[z2] = 80.0;
    // El total -40 N no puede ocultar la proyección -80 N del nodo 2.
    for factor in [0.0, 0.25] {
        rejects_suction(
            masses_from_load_vector(&map, &forces, factor, [0.0, 0.0, -1.0], 10.0, "peso"),
            &[EntityRef::Node(NodeId(2))],
        );
    }
    forces[z2] = 0.0;
    let masses = masses_from_load_vector(&map, &forces, 0.25, [0.0, 0.0, -1.0], 10.0, "peso")?;
    assert_eq!(masses.len(), 2);
    for (mass, expected) in masses.iter().zip([3.0, 0.0]) {
        for value in &mass.values {
            close(value.value, expected);
        }
    }
    Ok(())
}

#[test]
fn suction_case_uses_signed_net_projection_with_node_and_case() -> EResult<()> {
    let m = model(Dim::D2, [[0.0; 3], [1.0, 0.0, 0.0]], &[Dof::Ux, Dof::Uy]);
    let map = DofMap::new(&m.nodes)?;
    let mut case = LoadCase {
        name: "oblicuo".into(),
        loads: vec![NodalLoad {
            node: NodeId(1),
            values: vec![
                DofValue {
                    dof: Dof::Ux,
                    value: 3.0,
                },
                DofValue {
                    dof: Dof::Uy,
                    value: -4.0,
                },
            ],
        }],
    };
    // d̂=(3/5,4/5,0): q=(9-16)/5=-1.4 N, sin absolutos por componente.
    for factor in [0.0, 1.0] {
        rejects_suction(
            masses_from_case(&map, &case, factor, [3.0, 4.0, 0.0], 10.0),
            &[
                EntityRef::Node(NodeId(1)),
                EntityRef::Case(case.name.clone()),
            ],
        );
    }
    case.loads[0].values[0].value = 4.0;
    case.loads[0].values[1].value = -2.0;
    // Una componente opuesta sí se admite: q=(12-8)/5=0.8 N, m=0.08 kg.
    let masses = masses_from_case(&map, &case, 1.0, [3.0, 4.0, 0.0], 10.0)?;
    for value in &masses[0].values {
        close(value.value, 0.08);
    }
    Ok(())
}

#[test]
fn suction_system_case_sums_nodal_and_element_forces_before_projection() -> EResult<()> {
    let mut m = model(Dim::D3, [[0.0; 3], [1.0; 3]], &[Dof::Ux, Dof::Uy, Dof::Uz]);
    let case = CaseRef("neto".into());
    m.cases.push(LoadCase {
        name: case.0.clone(),
        loads: [(1, -120.0), (2, -80.0)]
            .into_iter()
            .map(|(node, value)| NodalLoad {
                node: NodeId(node),
                values: vec![DofValue {
                    dof: Dof::Uz,
                    value,
                }],
            })
            .collect(),
    });
    let map = DofMap::new(&m.nodes)?;
    let z2 = map.global_index(NodeId(2), Dof::Uz)?;
    // El aporte elemental ascendente se resta del peso nodal ANTES de proyectar:
    // -80+40=-40 N -> 4 kg; -80+80=0 -> 0 kg; -80+100=+20 N -> error.
    for (upward, expected) in [(40.0, Some(4.0)), (80.0, Some(0.0)), (100.0, None)] {
        let e = ElementContribution {
            location: vec![z2],
            stiffness: DMat::zeros(1, 1)?,
            mass_lumped: None,
            mass_consistent: None,
            geometric: None,
            loads: vec![ElementLoad {
                case: case.clone(),
                values: vec![upward],
            }],
        };
        let system = LinearSystem::from_model(ReferenceBackend, &m, &[e], &[])?;
        for factor in [0.0, 1.0] {
            let result = masses_from_system_case(&system, &case, factor, [0.0, 0.0, -1.0], 10.0);
            if let Some(mass2) = expected {
                let masses = result?;
                assert_eq!(masses.len(), 2);
                for (mass, value) in masses.iter().zip([12.0 * factor, mass2 * factor]) {
                    for component in &mass.values {
                        close(component.value, value);
                    }
                }
            } else {
                rejects_suction(
                    result,
                    &[EntityRef::Node(NodeId(2)), EntityRef::Case(case.0.clone())],
                );
            }
        }
    }
    Ok(())
}

#[test]
fn m13_negative_directional_total_with_positive_diagonal() -> EResult<()> {
    let m = model(Dim::D2, [[0.0; 3], [1.0, 0.0, 0.0]], &[Dof::Ux, Dof::Uy]);
    let map = DofMap::new(&m.nodes)?;
    let x1 = map.global_index(NodeId(1), Dof::Ux)?.0 as usize;
    let x2 = map.global_index(NodeId(2), Dof::Ux)?.0 as usize;
    // Oráculo propio (4), ENG-MASS-001-FU: [1,1] [[1,-2],[-2,1]] [1,1]ᵀ=-2 kg.
    let matrix = SparseMatrix::from_triplets(
        map.len(),
        map.len(),
        &[
            Triplet {
                row: x1,
                col: x1,
                value: 1.0,
            },
            Triplet {
                row: x1,
                col: x2,
                value: -2.0,
            },
            Triplet {
                row: x2,
                col: x1,
                value: -2.0,
            },
            Triplet {
                row: x2,
                col: x2,
                value: 1.0,
            },
        ],
    )?;
    assert_eq!(
        total_mass(&map, &matrix, [1.0, 0.0, 0.0]).unwrap_err().code,
        "ENG-E-0010"
    );
    Ok(())
}

#[test]
fn m24_z_direction_in_xy_model_rejected_by_mass_adapters() -> EResult<()> {
    let mut m = model(Dim::D2, [[0.0; 3], [1.0, 0.0, 0.0]], &[Dof::Ux, Dof::Uy]);
    let case = LoadCase {
        name: "xy".into(),
        loads: vec![],
    };
    m.cases.push(case.clone());
    let map = DofMap::new(&m.nodes)?;
    let system = LinearSystem::from_model(ReferenceBackend, &m, &[], &[])?;
    let z = [0.0, 0.0, 1.0];
    // Oráculo propio (5), ENG-MASS-001-FU: Uz no pertenece a {Ux,Uy}.
    for result in [
        masses_from_load_vector(&map, &vec![0.0; map.len()], 1.0, z, 10.0, "xy"),
        masses_from_case(&map, &case, 1.0, z, 10.0),
        masses_from_system_case(&system, &CaseRef(case.name.clone()), 1.0, z, 10.0),
    ] {
        assert_eq!(result.unwrap_err().code, "ENG-E-0011");
    }
    assert_eq!(
        total_mass(&map, &assemble_nodal_mass_matrix(&map, &[])?, z)
            .unwrap_err()
            .code,
        "ENG-E-0011"
    );
    Ok(())
}

#[test]
fn invalid_masses_references_dofs_and_accumulation() -> EResult<()> {
    let m = model(
        Dim::D2,
        [[0.0; 3], [1.0, 0.0, 0.0]],
        &[Dof::Ux, Dof::Uy, Dof::Rz],
    );
    let map = DofMap::new(&m.nodes)?;
    for value in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let masses = [nodal("bad", 1, &[(Dof::Ux, value)])];
        let expected = if value.is_finite() {
            "ENG-E-0010"
        } else {
            "ENG-E-0008"
        };
        assert_eq!(
            validate_nodal_masses(&map, &masses).unwrap_err().code,
            expected
        );
        assert_eq!(
            assemble_nodal_mass_matrix(&map, &masses).unwrap_err().code,
            expected
        );
    }
    for (masses, code) in [
        (vec![nodal("a", u32::MAX, &[])], "ENG-E-0003"),
        (vec![nodal("a", 1, &[(Dof::Uz, 1.0)])], "ENG-E-0011"),
        (vec![nodal("a", 1, &[]), nodal("a", 2, &[])], "ENG-E-0002"),
        (
            vec![
                nodal("a", 1, &[(Dof::Ux, f64::MAX)]),
                nodal("b", 1, &[(Dof::Ux, f64::MAX)]),
            ],
            "ENG-E-0008",
        ),
    ] {
        assert_eq!(assemble_nodal_masses(&map, &masses).unwrap_err().code, code);
    }
    assert_eq!(
        rigid_translation(&map, [0.0, 0.0, 1.0]).unwrap_err().code,
        "ENG-E-0011"
    );
    assert_eq!(
        total_mass(
            &map,
            &SparseMatrix::from_triplets(1, 1, &[])?,
            [1.0, 0.0, 0.0]
        )
        .unwrap_err()
        .code,
        "ENG-E-0010"
    );
    let negative = SparseMatrix::from_triplets(
        map.len(),
        map.len(),
        &[Triplet {
            row: 0,
            col: 0,
            value: -1.0,
        }],
    )?;
    assert_eq!(
        total_mass(&map, &negative, [1.0, 0.0, 0.0])
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    let overflow = SparseMatrix::from_triplets(
        map.len(),
        map.len(),
        &[
            Triplet {
                row: 0,
                col: 0,
                value: f64::MAX,
            },
            Triplet {
                row: 3,
                col: 3,
                value: f64::MAX,
            },
        ],
    )?;
    assert_eq!(
        total_mass(&map, &overflow, [1.0, 0.0, 0.0])
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    Ok(())
}

#[test]
fn conversion_invalid_entries_and_extremes_without_panics() -> EResult<()> {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for result in [
            mass_from_force([bad, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0], 10.0),
            mass_from_force([1.0; 3], bad, [1.0; 3], 10.0),
            mass_from_force([1.0; 3], 1.0, [bad, 0.0, 0.0], 10.0),
            mass_from_force([1.0; 3], 1.0, [1.0; 3], bad),
        ] {
            assert_eq!(result.unwrap_err().code, "ENG-E-0008");
        }
    }
    for result in [
        mass_from_force([1.0; 3], -0.25, [1.0; 3], 10.0),
        mass_from_force([1.0; 3], 1.0, [0.0; 3], 10.0),
        mass_from_force([1.0; 3], 1.0, [1.0; 3], 0.0),
        mass_from_force([1.0; 3], 1.0, [1.0; 3], -10.0),
    ] {
        assert_eq!(result.unwrap_err().code, "ENG-E-0010");
    }
    close(
        mass_from_force([1.0, 0.0, 0.0], 1.0, [f64::MAX, 0.0, 0.0], 1.0)?,
        1.0,
    );
    close(
        mass_from_force([1.0, 0.0, 0.0], 1.0, [f64::from_bits(1), 0.0, 0.0], 1.0)?,
        1.0,
    );
    assert_eq!(
        mass_from_force([f64::MAX, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0], 1.0)
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    assert_eq!(
        mass_from_force([1.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0], f64::from_bits(1))
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    assert_eq!(
        mass_from_force([f64::from_bits(1), 0.0, 0.0], 0.25, [1.0, 0.0, 0.0], 10.0)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    let m = model(Dim::D3, [[0.0; 3], [1.0; 3]], &[Dof::Ux, Dof::Uy, Dof::Uz]);
    let map = DofMap::new(&m.nodes)?;
    assert_eq!(
        masses_from_load_vector(&map, &[], 1.0, [1.0, 0.0, 0.0], 10.0, "a")
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    let mut force = vec![0.0; map.len()];
    force[0] = f64::NAN;
    assert_eq!(
        masses_from_load_vector(&map, &force, 0.0, [1.0, 0.0, 0.0], 10.0, "a")
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    let empty = LoadCase {
        name: "vacío".into(),
        loads: vec![],
    };
    let masses = masses_from_case(&map, &empty, 0.25, [0.0, 0.0, -1.0], 10.0)?;
    assert_eq!(assemble_nodal_masses(&map, &masses)?, vec![0.0; map.len()]);
    assert_eq!(assemble_nodal_masses(&map, &[])?, vec![0.0; map.len()]);
    let zero = assemble_nodal_mass_matrix(&map, &[])?;
    close(total_mass(&map, &zero, [f64::MAX; 3])?, 0.0);
    Ok(())
}

#[test]
fn element_zero_density_and_invalid_inputs() -> EResult<()> {
    for (rho, code) in [
        (-1.0, "ENG-E-0010"),
        (f64::NAN, "ENG-E-0008"),
        (f64::INFINITY, "ENG-E-0008"),
    ] {
        assert_eq!(
            LinearTruss::new(Dim::D3, 200.0, 2.0, rho).unwrap_err().code,
            code
        );
        assert_eq!(
            EulerFrame2D::new(200.0, 2.0, 3.0, rho, MomentReleases::default())
                .unwrap_err()
                .code,
            code
        );
    }
    let bar = LinearTruss::new(Dim::D3, 200.0, 2.0, 0.0)?;
    let coords = [[0.0; 3], [4.0, 0.0, 0.0]];
    let m = model(Dim::D3, coords, bar.dofs_per_node());
    let e = contribution(&bar, Dim::D3, &coords)?;
    assert!(e.mass_lumped.is_none() && e.mass_consistent.is_none());
    let system = LinearSystem::from_model(ReferenceBackend, &m, &[e], &[])?;
    for kind in [MassKind::Lumped, MassKind::Consistent] {
        close(system_total_mass(&system, kind, [1.0, 0.0, 0.0])?, 0.0);
    }
    Ok(())
}
