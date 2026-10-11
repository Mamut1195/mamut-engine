use super::*;

fn valid_model(dim: Dim) -> Model {
    let mut model = Model::new();
    model
        .insert_node(Node::new(NodeId(1), dim, [0.0, 0.0, 0.0]))
        .insert_node(Node::new(NodeId(2), dim, [2.0, 0.0, 0.0]))
        .insert_section(Section {
            id: "sección-a".into(),
            properties: vec![Parameter {
                name: "área".into(),
                value: 0.01,
            }],
        })
        .insert_law(Law {
            name: "ley-a".into(),
            parameters: vec![Parameter {
                name: "módulo".into(),
                value: 2.0e11,
            }],
        })
        .insert_member(Member {
            id: MemberId(10),
            nodes: [NodeId(1), NodeId(2)],
            section: "sección-a".into(),
            law: "ley-a".into(),
        })
        .insert_support(Support {
            id: "apoyo-a".into(),
            node: NodeId(1),
            restrained: dim.dofs().to_vec(),
        })
        .insert_case(LoadCase {
            name: "caso-a".into(),
            loads: vec![NodalLoad {
                node: NodeId(2),
                values: vec![DofValue {
                    dof: Dof::Uy,
                    value: -10.0,
                }],
            }],
        })
        .insert_combination(Combination {
            id: "combinación-a".into(),
            terms: vec![CaseFactor {
                case: "caso-a".into(),
                factor: 1.5,
            }],
        })
        .insert_mass(NodalMass {
            id: "masa-a".into(),
            node: NodeId(2),
            values: vec![DofValue {
                dof: Dof::Ux,
                value: 3.0,
            }],
        });
    model
}

fn entity_key(entity: &EntityRef) -> String {
    match entity {
        EntityRef::Node(id) => format!("nodo:{}", id.0),
        EntityRef::Member(id) => format!("miembro:{}", id.0),
        EntityRef::Case(name) => format!("caso:{name}"),
        EntityRef::Law(name) => format!("ley:{name}"),
        EntityRef::Dof(_) => panic!("El modelo no emite referencias a índices globales"),
    }
}

fn check(diagnostic: &Diagnostic, code: &str, entity: &str, context: &str) {
    assert_eq!(diagnostic.code, code);
    assert_eq!(diagnostic.entities.len(), 1);
    assert_eq!(entity_key(&diagnostic.entities[0]), entity);
    assert!(diagnostic.hint.as_ref().unwrap().contains(context));
    assert_eq!(
        diagnostic.severity,
        engine_core::catalog::lookup(code).unwrap().severity
    );
    assert_eq!(
        diagnostic.message,
        engine_core::catalog::lookup(code).unwrap().message
    );
}

#[test]
fn valid_models_and_empty_model() {
    assert!(validate(&Model::new()).is_empty());
    for dim in [Dim::D2, Dim::D3] {
        let model = valid_model(dim);
        assert!(validate(&model).is_empty());
        assert_eq!(model.nodes[0].dofs, dim.dofs());
    }
}

#[test]
fn duplicate_ids_in_every_collection_are_preserved() {
    let mut model = valid_model(Dim::D2);
    model.insert_node(model.nodes[0].clone());
    model.insert_member(model.members[0].clone());
    model.insert_section(model.sections[0].clone());
    model.insert_law(model.laws[0].clone());
    model.insert_support(model.supports[0].clone());
    model.insert_case(model.cases[0].clone());
    model.insert_combination(model.combinations[0].clone());
    model.insert_mass(model.masses[0].clone());
    let diagnostics = validate(&model);
    let expected = [
        ("nodo:1", "Nodo 1"),
        ("miembro:10", "Miembro 10"),
        ("miembro:10", "sección-a"),
        ("ley:ley-a", "ley-a"),
        ("nodo:1", "apoyo-a"),
        ("caso:caso-a", "caso-a"),
        ("caso:caso-a", "combinación-a"),
        ("nodo:2", "masa-a"),
    ];
    assert_eq!(diagnostics.len(), expected.len());
    for (diagnostic, (entity, context)) in diagnostics.iter().zip(expected) {
        check(diagnostic, "ENG-E-0002", entity, context);
    }
    assert_eq!(model.nodes.len(), 3);
    assert_eq!(model.members.len(), 2);
}

#[test]
fn broken_references_report_the_owner_and_exact_target() {
    let mut model = valid_model(Dim::D2);
    model.members[0].nodes = [NodeId(91), NodeId(92)];
    model.members[0].section = "sección-ausente".into();
    model.members[0].law = "ley-ausente".into();
    model.supports[0].node = NodeId(93);
    model.cases[0].loads[0].node = NodeId(94);
    model.combinations[0].terms[0].case = "caso-ausente".into();
    model.masses[0].node = NodeId(95);
    let diagnostics = validate(&model);
    let broken: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == "ENG-E-0003")
        .collect();
    let expected = [
        ("miembro:10", "91"),
        ("miembro:10", "92"),
        ("miembro:10", "sección-ausente"),
        ("miembro:10", "ley-ausente"),
        ("nodo:93", "apoyo-a"),
        ("caso:caso-a", "94"),
        ("caso:caso-ausente", "combinación-a"),
        ("nodo:95", "masa-a"),
    ];
    assert_eq!(broken.len(), expected.len());
    for (diagnostic, (entity, context)) in broken.iter().zip(expected) {
        check(diagnostic, "ENG-E-0003", entity, context);
    }
    // Se acumulan también los huérfanos y no se evalúan GDL sin nodo conocido.
    assert_eq!(diagnostics.len(), 10);
    assert!(!diagnostics
        .iter()
        .any(|d| d.code == "ENG-E-0005" || d.code == "ENG-E-0011"));
}

#[test]
fn supports_loads_and_masses_do_not_connect_an_orphan() {
    let mut model = valid_model(Dim::D2);
    model.insert_node(Node::new(NodeId(3), Dim::D2, [4.0, 0.0, 0.0]));
    model.supports[0].node = NodeId(3);
    model.cases[0].loads[0].node = NodeId(3);
    model.masses[0].node = NodeId(3);
    let diagnostics = validate(&model);
    assert_eq!(diagnostics.len(), 1);
    check(&diagnostics[0], "ENG-E-0004", "nodo:3", "Nodo 3");
}

#[test]
fn length_tolerance_has_an_inclusive_absolute_boundary() {
    for length in [
        0.0,
        LENGTH_TOLERANCE / 2.0,
        LENGTH_TOLERANCE,
        LENGTH_TOLERANCE * 2.0,
    ] {
        let mut model = valid_model(Dim::D3);
        model.nodes[1].coordinates = [0.0, 0.0, length];
        let diagnostics = validate(&model);
        if length <= LENGTH_TOLERANCE {
            assert_eq!(diagnostics.len(), 1);
            check(&diagnostics[0], "ENG-E-0005", "miembro:10", "tolerancia");
        } else {
            assert!(diagnostics.is_empty());
        }
    }
    let mut model = valid_model(Dim::D2);
    model.members[0].nodes[1] = NodeId(1);
    let diagnostics = validate(&model);
    assert_eq!(diagnostics.len(), 2); // Nodo 2 huérfano y miembro de extremos idénticos.
    check(&diagnostics[1], "ENG-E-0005", "miembro:10", "Miembro 10");
}

#[test]
fn all_numeric_fields_reject_nan_and_both_infinities() {
    for invalid in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        let mut model = valid_model(Dim::D2);
        model.nodes[0].coordinates = [invalid; 3];
        model.sections[0].properties[0].value = invalid;
        model.laws[0].parameters[0].value = invalid;
        model.cases[0].loads[0].values[0].value = invalid;
        model.combinations[0].terms[0].factor = invalid;
        model.masses[0].values[0].value = invalid;
        let diagnostics = validate(&model);
        let expected = [
            ("nodo:1", "X"),
            ("nodo:1", "Y"),
            ("nodo:1", "Z"),
            ("miembro:10", "sección-a"),
            ("ley:ley-a", "módulo"),
            ("caso:caso-a", "carga 0"),
            ("caso:caso-a", "combinación-a"),
            ("nodo:2", "masa-a"),
        ];
        assert_eq!(diagnostics.len(), expected.len());
        for (diagnostic, (entity, context)) in diagnostics.iter().zip(expected) {
            check(diagnostic, "ENG-E-0008", entity, context);
        }
    }
}

#[test]
fn invalid_dofs_in_nodes_supports_loads_and_masses() {
    for dof in [Dof::Uz, Dof::Rx, Dof::Ry] {
        let mut model = valid_model(Dim::D2);
        model.nodes[0].dofs.push(dof);
        model.supports[0].restrained.push(dof);
        model.cases[0].loads[0].values[0].dof = dof;
        model.masses[0].values[0].dof = dof;
        let diagnostics = validate(&model);
        let expected = [
            ("nodo:1", "Nodo 1"),
            ("nodo:1", "apoyo-a"),
            ("caso:caso-a", "caso-a"),
            ("nodo:2", "masa-a"),
        ];
        assert_eq!(diagnostics.len(), expected.len());
        for (diagnostic, (entity, context)) in diagnostics.iter().zip(expected) {
            check(diagnostic, "ENG-E-0011", entity, context);
            assert!(diagnostic.hint.as_ref().unwrap().contains(dof.label()));
        }
    }
    let mut model = valid_model(Dim::D3);
    model.cases[0].loads[0].values = Dim::D3
        .dofs()
        .iter()
        .map(|dof| DofValue {
            dof: *dof,
            value: 1.0,
        })
        .collect();
    model.masses[0].values = model.cases[0].loads[0].values.clone();
    assert!(validate(&model).is_empty());
}

#[test]
fn nominal_references_for_unattached_sections_and_empty_combinations() {
    let mut model = Model::new();
    model.insert_section(Section {
        id: "sección-sola".into(),
        properties: vec![Parameter {
            name: "área".into(),
            value: Real::NAN,
        }],
    });
    model.insert_section(model.sections[0].clone());
    model.insert_combination(Combination {
        id: "combinación-vacía".into(),
        terms: vec![],
    });
    model.insert_combination(model.combinations[0].clone());
    let diagnostics = validate(&model);
    assert_eq!(diagnostics.len(), 4);
    check(
        &diagnostics[0],
        "ENG-E-0008",
        "ley:sección-sola",
        "sección-sola",
    );
    check(
        &diagnostics[1],
        "ENG-E-0002",
        "ley:sección-sola",
        "sección-sola",
    );
    check(
        &diagnostics[2],
        "ENG-E-0008",
        "ley:sección-sola",
        "sección-sola",
    );
    check(
        &diagnostics[3],
        "ENG-E-0002",
        "caso:combinación-vacía",
        "combinación-vacía",
    );
}

#[test]
fn repeated_validation_is_deterministic_and_uses_first_duplicate_node() {
    let mut model = valid_model(Dim::D2);
    let mut duplicate_node = model.nodes[1].clone();
    duplicate_node.coordinates = [0.0; 3];
    duplicate_node.dofs.push(Dof::Uz);
    model.insert_node(duplicate_node);
    model.combinations[0].terms.push(CaseFactor {
        case: "caso-ausente".into(),
        factor: Real::NAN,
    });
    let snapshot = |diagnostics: Vec<Diagnostic>| -> Vec<_> {
        diagnostics
            .into_iter()
            .map(|d| {
                (
                    d.code,
                    d.message,
                    d.entities.iter().map(entity_key).collect::<Vec<_>>(),
                    d.hint,
                )
            })
            .collect()
    };
    let first = validate(&model);
    assert_eq!(
        first.iter().map(|d| d.code).collect::<Vec<_>>(),
        ["ENG-E-0002", "ENG-E-0011", "ENG-E-0003", "ENG-E-0008"]
    );
    check(&first[0], "ENG-E-0002", "nodo:2", "Nodo 2");
    check(&first[1], "ENG-E-0011", "nodo:2", "Uz");
    check(
        &first[2],
        "ENG-E-0003",
        "caso:caso-ausente",
        "combinación-a",
    );
    check(
        &first[3],
        "ENG-E-0008",
        "caso:caso-ausente",
        "combinación-a",
    );
    assert_eq!(snapshot(first), snapshot(validate(&model)));
    assert_eq!(model.nodes.len(), 3);
}

#[test]
fn huge_and_tiny_finite_coordinates_do_not_create_false_zero_lengths() {
    let mut model = valid_model(Dim::D3);
    model.nodes[1].coordinates = [1e200; 3];
    assert!(validate(&model).is_empty());
    model.nodes[0].coordinates = [-Real::MAX, 0.0, 0.0];
    model.nodes[1].coordinates = [Real::MAX, 0.0, 0.0];
    assert!(validate(&model).is_empty());
    model.nodes[0].coordinates = [0.0; 3];
    model.nodes[1].coordinates = [1e-200; 3];
    let diagnostics = validate(&model);
    assert_eq!(diagnostics.len(), 1);
    check(&diagnostics[0], "ENG-E-0005", "miembro:10", "Miembro 10");
}
