use engine_assembly::{Constraints, DofMap};
use engine_core::{Dim, Dof, EResult, EntityRef, NodeId};
use engine_loads::{
    assemble_imposed_displacements, assemble_nodal_loads, validate_case, DofValue,
    ImposedDisplacement, LoadCase, NodalLoad,
};
use engine_model::{Node, Support};

fn case(values: Vec<DofValue>) -> LoadCase {
    LoadCase {
        name: "servicio".into(),
        loads: vec![NodalLoad {
            node: NodeId(10),
            values,
        }],
    }
}

#[test]
fn global_numbering_reuses_model_and_assembly() -> EResult<()> {
    let map = DofMap::new(&[
        Node::new(NodeId(20), Dim::D2, [1.0, 0.0, 0.0]),
        Node::new(NodeId(10), Dim::D2, [0.0; 3]),
    ])?;
    let mut load_case = case(vec![
        DofValue {
            dof: Dof::Rz,
            value: 3.0,
        },
        DofValue {
            dof: Dof::Ux,
            value: 7.0,
        },
    ]);
    load_case.loads.push(NodalLoad {
        node: NodeId(20),
        values: vec![DofValue {
            dof: Dof::Uy,
            value: -4.0,
        }],
    });
    let imposed = [
        ImposedDisplacement {
            node: NodeId(20),
            dof: Dof::Rz,
            value: 0.02,
        },
        ImposedDisplacement {
            node: NodeId(10),
            dof: Dof::Ux,
            value: 0.0,
        },
    ];
    validate_case(&map, &load_case, &imposed)?;
    assert_eq!(
        assemble_nodal_loads(&map, &load_case)?,
        vec![7.0, 0.0, 3.0, 0.0, -4.0, 0.0]
    );
    let u = assemble_imposed_displacements(&map, &imposed)?;
    assert_eq!(u, vec![Some(0.0), None, None, None, None, Some(0.02)]);
    let supports = [Support {
        id: "base".into(),
        node: NodeId(10),
        restrained: vec![Dof::Ux, Dof::Uy],
    }];
    let constraints = Constraints::new(&map, &supports, &imposed)?;
    assert_eq!(constraints.prescribed_values()[0], u[0]);
    assert_eq!(constraints.prescribed_values()[5], u[5]);
    assert_eq!(
        constraints.expand(&[1.0, 2.0, 3.0])?,
        vec![0.0, 0.0, 1.0, 2.0, 3.0, 0.02]
    );
    assert_eq!(
        assemble_nodal_loads(
            &map,
            &LoadCase {
                name: "vacío".into(),
                loads: vec![]
            }
        )?,
        vec![0.0; 6]
    );
    assert_eq!(assemble_imposed_displacements(&map, &[])?, vec![None; 6]);
    Ok(())
}

#[test]
fn dimension_and_declared_dofs_are_respected() -> EResult<()> {
    let mut node = Node::new(NodeId(10), Dim::D2, [0.0; 3]);
    node.dofs = vec![Dof::Ux];
    let map = DofMap::new(&[node])?;
    for dof in [Dof::Uy, Dof::Uz, Dof::Rz, Dof::Rx] {
        assert_eq!(
            assemble_nodal_loads(&map, &case(vec![DofValue { dof, value: 1.0 }]))
                .unwrap_err()
                .code,
            "ENG-E-0011"
        );
    }
    let map = DofMap::new(&[Node::new(NodeId(10), Dim::D3, [0.0; 3])])?;
    let spatial = case(
        Dim::D3
            .dofs()
            .iter()
            .map(|&dof| DofValue {
                dof,
                value: dof.index() as f64 + 1.0,
            })
            .collect(),
    );
    assert_eq!(
        assemble_nodal_loads(&map, &spatial)?,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
    );
    let imposed: Vec<_> = Dim::D3
        .dofs()
        .iter()
        .map(|&dof| ImposedDisplacement {
            node: NodeId(10),
            dof,
            value: 0.01,
        })
        .collect();
    assert_eq!(
        assemble_imposed_displacements(&map, &imposed)?,
        vec![Some(0.01); 6]
    );
    Ok(())
}

#[test]
fn duplicates_nonfinite_and_broken_references() -> EResult<()> {
    let map = DofMap::new(&[Node::new(NodeId(10), Dim::D2, [0.0; 3])])?;
    let value = DofValue {
        dof: Dof::Uy,
        value: 1.0,
    };
    let mut duplicate = case(vec![value.clone(), value.clone()]);
    assert_eq!(
        assemble_nodal_loads(&map, &duplicate).unwrap_err().code,
        "ENG-E-0010"
    );
    duplicate = case(vec![value.clone()]);
    duplicate.loads.push(NodalLoad {
        node: NodeId(10),
        values: vec![value.clone()],
    });
    assert_eq!(
        validate_case(&map, &duplicate, &[]).unwrap_err().code,
        "ENG-E-0010"
    );
    // La unicidad pertenece a cada caso, no a todo el modelo.
    let first = case(vec![value.clone()]);
    let mut second = case(vec![value]);
    second.name = "otro".into();
    assert_eq!(
        assemble_nodal_loads(&map, &first)?,
        assemble_nodal_loads(&map, &second)?
    );
    let imposed = ImposedDisplacement {
        node: NodeId(10),
        dof: Dof::Uy,
        value: 0.0,
    };
    assert_eq!(
        assemble_imposed_displacements(&map, &[imposed, imposed])
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            assemble_nodal_loads(
                &map,
                &case(vec![DofValue {
                    dof: Dof::Ux,
                    value
                }])
            )
            .unwrap_err()
            .code,
            "ENG-E-0008"
        );
        assert_eq!(
            assemble_imposed_displacements(&map, &[ImposedDisplacement { value, ..imposed }])
                .unwrap_err()
                .code,
            "ENG-E-0008"
        );
    }
    for values in [
        vec![],
        vec![DofValue {
            dof: Dof::Ux,
            value: 1.0,
        }],
    ] {
        let bad = LoadCase {
            name: "ausente".into(),
            loads: vec![NodalLoad {
                node: NodeId(99),
                values,
            }],
        };
        let e = assemble_nodal_loads(&map, &bad).unwrap_err();
        assert_eq!(e.code, "ENG-E-0003");
        assert!(e
            .entities
            .iter()
            .any(|entity| matches!(entity,EntityRef::Case(name) if name=="ausente")));
    }
    assert_eq!(
        assemble_imposed_displacements(
            &map,
            &[ImposedDisplacement {
                node: NodeId(99),
                ..imposed
            }]
        )
        .unwrap_err()
        .code,
        "ENG-E-0003"
    );
    assert_eq!(
        assemble_imposed_displacements(
            &map,
            &[ImposedDisplacement {
                dof: Dof::Uz,
                ..imposed
            }]
        )
        .unwrap_err()
        .code,
        "ENG-E-0011"
    );
    assert_eq!(
        assemble_nodal_loads(
            &map,
            &case(vec![DofValue {
                dof: Dof::Ux,
                value: 1e308
            }])
        )?,
        vec![1e308, 0.0, 0.0]
    );
    Ok(())
}
