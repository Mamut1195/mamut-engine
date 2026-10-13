//! Fixtures analíticos propios; ninguna fuente externa ni oráculo.
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, MassKind};
use engine_core::{Dim, Dof, DofIndex, EResult, NodeId};
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
const CITATION: &str = "provenance/ENG-MASS-001.md, ecuaciones (1)-(4); derivaciones propias";

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
            // El signo opuesto no cancela masa entre nodos.
            NodalLoad {
                node: NodeId(2),
                values: vec![DofValue {
                    dof: Dof::Uz,
                    value: 80.0,
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
            values: vec![20.0, -40.0],
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
    // Fuerzas netas -100 y +40: 2.5 + 1 = 3.5 kg en cada dirección.
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
        mass_from_force([3.0, 4.0, 0.0], 0.25, [-4.0, 3.0, 0.0], 10.0)?,
        0.0,
    );
    close(
        mass_from_force([3.0, 4.0, 0.0], 0.0, [3.0, 4.0, 0.0], 10.0)?,
        0.0,
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
