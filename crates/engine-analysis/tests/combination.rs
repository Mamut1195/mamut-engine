//! In-house analytical solutions: provenance/ENG-LOAD-002.md. No oracles.
use engine_analysis::{
    combination::{combine, envelope, EnvelopeInput, ResultId},
    results::{CaseResults, Field, MemberRecovery, MemberResult, NodeResult, StationResult},
    StaticLinear,
};
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem, PrescribedDof};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, MemberLoad};
use engine_core::math;
use engine_core::{Dim, Dof, EResult, EntityRef, MemberId, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_linalg::ReferenceBackend;
use engine_model::{CaseFactor, Combination, DofValue, LoadCase, Model, NodalLoad, Node, Support};

fn declaration(terms: &[(&str, f64)]) -> Combination {
    Combination {
        id: "C".into(),
        terms: terms
            .iter()
            .map(|&(case, factor)| CaseFactor {
                case: case.into(),
                factor,
            })
            .collect(),
    }
}

fn fixture(name: &str, value: f64) -> CaseResults {
    CaseResults {
        case: CaseRef(name.into()),
        factor: 1.0,
        nodes: vec![NodeResult {
            id: NodeId(7),
            displacements: std::array::from_fn(|i| value * (i + 1) as f64),
            prescribed_displacements: [None; 6],
            reactions: std::array::from_fn(|i| -value * (i + 1) as f64),
        }],
        members: vec![MemberResult {
            id: MemberId(13),
            end_forces_local: (1..=6).map(|i| value * i as f64).collect(),
            stations: [0.0, 0.5, 1.0]
                .into_iter()
                .map(|position| StationResult {
                    position,
                    forces_local: std::array::from_fn(|i| {
                        value * (i + 1) as f64 * (1.0 - 2.0 * position)
                    }),
                })
                .collect(),
            // This cache is ignored; the API must recompute the extremes.
            extremes: vec![],
        }],
    }
}

fn components(r: &CaseResults) -> Vec<f64> {
    r.nodes
        .iter()
        .flat_map(|n| n.displacements.into_iter().chain(n.reactions))
        .chain(r.members.iter().flat_map(|m| {
            m.end_forces_local
                .iter()
                .copied()
                .chain(m.stations.iter().flat_map(|s| s.forces_local))
        }))
        .collect()
}

fn bit_equal(actual: f64, expected: f64) {
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "{actual:?} != {expected:?}"
    );
}

fn close(actual: f64, expected: f64) {
    close_scaled(actual, expected, 1.0);
}

fn close_scaled(actual: f64, expected: f64, scale: f64) {
    let tolerance = 128.0 * f64::EPSILON * expected.abs().max(scale).max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}, tolerance {tolerance}"
    );
}

#[test]
fn weighted_sum_bits_negative_zero_and_already_scaled_case() {
    let a = fixture("A", 0.75);
    let mut b = fixture("B", -1.25);
    b.factor = 8.0; // Metadata, its values are already the scaled result.
    let c = fixture("Z", 1000.0);
    let cases = [a, b, c];
    for factors in [[2.0, -0.5, 0.0], [0.125, 4.0, -2.0], [1.3, -0.7, 0.0]] {
        let before = cases.clone();
        let output = combine(
            &declaration(&[("A", factors[0]), ("B", factors[1]), ("Z", factors[2])]),
            &cases,
        )
        .unwrap();
        assert_eq!(output.id, "C");
        assert_eq!(output.results.case.0, "C");
        assert_eq!(output.results.factor, 1.0);
        for (i, actual) in components(&output.results).into_iter().enumerate() {
            let mut expected = 0.0;
            for (r, factor) in cases.iter().zip(factors) {
                expected += factor * components(r)[i];
            }
            bit_equal(actual, expected);
        }
        assert_eq!(cases, before);
    }
    let zero = combine(&declaration(&[("A", 0.0)]), &cases).unwrap();
    assert!(components(&zero.results).iter().all(|&v| v == 0.0));
}

#[test]
fn extrema_are_recomputed_after_cancellation() {
    let a = fixture("A", 8.0);
    let mut b = fixture("B", 8.0);
    for s in &mut b.members[0].stations {
        s.forces_local = s.forces_local.map(|v| -v);
    }
    let output = combine(&declaration(&[("A", 1.0), ("B", 1.0)]), &[a, b]).unwrap();
    assert_eq!(output.results.members[0].extremes.len(), 6);
    for e in &output.results.members[0].extremes {
        assert_eq!(e.max.value, 0.0);
        assert_eq!(e.min.value, 0.0);
        assert_eq!(e.max.at_station, 0.0);
        assert_eq!(e.min.at_station, 0.0);
    }
}

// Cantilever L=2, E=16, A=2, I=2, homogeneous supports. A: axial 48;
// B: transverse force -48 and, optionally, uniform -24. D: combined loads.
fn cantilever(factors: [f64; 2], with_uniform: bool) -> EResult<Vec<CaseResults>> {
    let mut model = Model::new();
    model.nodes = vec![
        Node::new(NodeId(1), Dim::D2, [0.0; 3]),
        Node::new(NodeId(2), Dim::D2, [2.0, 0.0, 0.0]),
    ];
    model.supports.push(Support {
        id: "empotramiento".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux, Dof::Uy, Dof::Rz],
    });
    for (name, axial, transverse) in [
        ("A", 48.0, 0.0),
        ("B", 0.0, -48.0),
        ("D", factors[0] * 48.0, factors[1] * -48.0),
    ] {
        model.cases.push(LoadCase {
            name: name.into(),
            loads: vec![NodalLoad {
                node: NodeId(2),
                values: vec![
                    DofValue {
                        dof: Dof::Ux,
                        value: axial,
                    },
                    DofValue {
                        dof: Dof::Uy,
                        value: transverse,
                    },
                ],
            }],
        });
    }
    let element = EulerFrame2D::new(16.0, 2.0, 2.0, 0.0, MomentReleases::default())?;
    let params = ElementParams::default();
    let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let map = DofMap::new(&model.nodes)?;
    let mut location = vec![];
    for node in [NodeId(1), NodeId(2)] {
        for &dof in element.dofs_per_node() {
            location.push(map.global_index(node, dof)?);
        }
    }
    let q = if with_uniform { -24.0 } else { 0.0 };
    let loads: Vec<_> = [0.0, q, factors[1] * q]
        .into_iter()
        .map(|q| MemberLoad {
            force_per_length: [0.0, q, 0.0],
            moment_per_length: [0.0; 3],
        })
        .collect();
    let contribution = ElementContribution {
        stiffness: element.respond(&ctx, &[0.0; 6], &[])?.tangent,
        location,
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: model
            .cases
            .iter()
            .zip(&loads)
            .map(|(case, load)| {
                Ok(ElementLoad {
                    case: CaseRef(case.name.clone()),
                    values: element.load_vector(&ctx, load)?,
                })
            })
            .collect::<EResult<_>>()?,
    };
    let mut system = LinearSystem::from_model(ReferenceBackend, &model, &[contribution], &[])?;
    let requested: Vec<_> = model
        .cases
        .iter()
        .map(|c| (CaseRef(c.name.clone()), 1.0))
        .collect();
    let solutions = StaticLinear::run(&mut system, &requested)?;
    let mut results = vec![];
    for (solution, load) in solutions.cases.iter().zip(&loads) {
        results.push(CaseResults::recover(
            solution,
            &system,
            &[MemberRecovery {
                id: MemberId(9),
                nodes: &[NodeId(1), NodeId(2)],
                element: &element,
                dim: Dim::D2,
                params: &params,
                committed: &[],
                loads: std::slice::from_ref(load),
                stations: None,
            }],
        )?);
    }
    Ok(results)
}

// Beam with zero rotations, uy(0)=0, uy(L)=δ; free ux(L) allows solving.
// v=δ(3(x/L)²−2(x/L)³) => M=6EIδ/L²(1−2x/L), V=12EIδ/L³.
fn fixed_end_settlement(name: &str, axial: f64, delta: f64) -> EResult<CaseResults> {
    let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
    let mut model = Model::new();
    model.nodes = vec![
        Node::new(NodeId(1), Dim::D2, coords[0]),
        Node::new(NodeId(2), Dim::D2, coords[1]),
    ];
    model.supports = vec![
        Support {
            id: "i".into(),
            node: NodeId(1),
            restrained: vec![Dof::Ux, Dof::Uy, Dof::Rz],
        },
        Support {
            id: "j".into(),
            node: NodeId(2),
            restrained: vec![Dof::Uy, Dof::Rz],
        },
    ];
    model.cases.push(LoadCase {
        name: name.into(),
        loads: vec![NodalLoad {
            node: NodeId(2),
            values: vec![DofValue {
                dof: Dof::Ux,
                value: axial,
            }],
        }],
    });
    let element = EulerFrame2D::new(16.0, 2.0, 2.0, 0.0, MomentReleases::default())?;
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let map = DofMap::new(&model.nodes)?;
    let mut location = vec![];
    for node in [NodeId(1), NodeId(2)] {
        for &dof in element.dofs_per_node() {
            location.push(map.global_index(node, dof)?);
        }
    }
    let contribution = ElementContribution {
        stiffness: element.respond(&ctx, &[0.0; 6], &[])?.tangent,
        location,
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    };
    let mut system = LinearSystem::from_model(
        ReferenceBackend,
        &model,
        &[contribution],
        &[PrescribedDof {
            node: NodeId(2),
            dof: Dof::Uy,
            value: delta,
        }],
    )?;
    let solutions = StaticLinear::run(&mut system, &[(CaseRef(name.into()), 1.0)])?;
    CaseResults::recover(
        &solutions.cases[0],
        &system,
        &[MemberRecovery {
            id: MemberId(9),
            nodes: &[NodeId(1), NodeId(2)],
            element: &element,
            dim: Dim::D2,
            params: &params,
            committed: &[],
            loads: &[],
            stations: None,
        }],
    )
}

#[test]
fn repeated_settlement_rejected_against_fixed_end_beam_oracle() -> EResult<()> {
    let delta = 0.125;
    let moment = 6.0 * 16.0 * 2.0 * delta / math::powi(2.0_f64, 2);
    let d = fixed_end_settlement("D", 48.0, delta)?;
    close(d.members[0].stations[0].forces_local[5], moment);
    assert_eq!(d.nodes[1].prescribed_displacements[1], Some(delta));
    for other_delta in [delta, -delta, 2.0 * delta] {
        let l = fixed_end_settlement("L", -16.0, other_delta)?;
        close(
            l.members[0].stations[0].forces_local[5],
            moment * other_delta / delta,
        );
        if other_delta == delta {
            // The previous implementation would produce 2.8 M and 2.8 δ.
            close(
                1.2 * d.members[0].stations[0].forces_local[5]
                    + 1.6 * l.members[0].stations[0].forces_local[5],
                2.8 * moment,
            );
        }
        for factors in [[1.2, 1.6], [1.0, 0.0], [0.0, 0.0], [1.0, -1.0]] {
            let error = combine(
                &declaration(&[("D", factors[0]), ("L", factors[1])]),
                &[d.clone(), l.clone()],
            )
            .unwrap_err();
            assert_eq!(error.code, "ENG-E-0010");
            assert!(
                matches!(error.entities.as_slice(), [EntityRef::Case(d), EntityRef::Case(l)] if d == "D" && l == "L")
            );
            assert!(error.message.contains("separate settlement case"));
        }
    }
    Ok(())
}

#[test]
fn dedicated_settlement_case_counted_once_with_explicit_factor() -> EResult<()> {
    let delta = 0.125;
    let moment = 6.0 * 16.0 * 2.0 * delta / math::powi(2.0_f64, 2);
    let cases = [
        fixed_end_settlement("D", 48.0, 0.0)?,
        fixed_end_settlement("L", -16.0, 0.0)?,
        fixed_end_settlement("S", 0.0, delta)?,
    ];
    for alpha in [1.0, 0.5, -1.0, 0.0] {
        let c = combine(
            &declaration(&[("D", 1.2), ("L", 1.6), ("S", alpha)]),
            &cases,
        )?;
        let axial = 1.2 * 48.0 - 1.6 * 16.0;
        close(
            c.results.nodes[1].displacements[0],
            axial * 2.0 / (16.0 * 2.0),
        );
        close(c.results.nodes[1].displacements[1], alpha * delta);
        assert_eq!(
            c.results.nodes[1].prescribed_displacements[1],
            Some(alpha * delta)
        );
        close(c.results.nodes[0].reactions[5], -alpha * moment);
        close(c.results.nodes[1].reactions[5], -alpha * moment);
        for s in &c.results.members[0].stations {
            close(s.forces_local[0], axial);
            close(
                s.forces_local[1],
                alpha * 12.0 * 16.0 * 2.0 * delta / math::powi(2.0_f64, 3),
            );
            close(s.forces_local[5], alpha * moment * (1.0 - 2.0 * s.position));
        }
        let direct = fixed_end_settlement("directo", axial, alpha * delta)?;
        for (actual, expected) in components(&c.results).into_iter().zip(components(&direct)) {
            close(actual, expected);
        }
        // A combined result retains prescribed displacements for another combination.
        let nested = combine(
            &declaration(&[("C", 1.0), ("S", 1.0)]),
            &[c.results.clone(), cases[2].clone()],
        );
        if alpha == 0.0 {
            close(nested?.results.nodes[1].displacements[1], delta);
        } else {
            assert_eq!(nested.unwrap_err().code, "ENG-E-0010");
        }
    }
    // A single term with settlement retains the full explicit scaling;
    // unreferenced cases must not trigger rejection.
    let lone = combine(&declaration(&[("S", 2.0)]), &cases)?;
    close(
        lone.results.members[0].stations[0].forces_local[5],
        2.0 * moment,
    );
    Ok(())
}

#[test]
fn incoherent_prescribed_metadata_rejected_with_case_entity() -> EResult<()> {
    let a = fixed_end_settlement("A", 0.0, 0.125)?;
    for prescribed in [Some(0.25), Some(f64::NAN), Some(f64::INFINITY), None] {
        let mut b = fixed_end_settlement("B", 0.0, 0.0)?;
        b.nodes[1].prescribed_displacements[1] = prescribed;
        let error = combine(&declaration(&[("A", 1.0), ("B", 0.0)]), &[a.clone(), b]).unwrap_err();
        let code = if prescribed.is_some_and(|p| !p.is_finite()) {
            "ENG-E-0008"
        } else {
            "ENG-E-0010"
        };
        assert_eq!(error.code, code);
        if prescribed == Some(0.25) {
            assert_eq!(
                error.message,
                "Prescribed displacement is inconsistent with the nodal result."
            );
        }
        assert!(error
            .entities
            .iter()
            .any(|e| matches!(e, EntityRef::Case(name) if name == "B")));
    }
    Ok(())
}

#[test]
fn accumulation_follows_term_order_with_cancellation() -> EResult<()> {
    let mut cases = [fixture("A", 0.0), fixture("B", 0.0), fixture("C", 0.0)];
    for (case, value) in cases.iter_mut().zip([1e16, -1e16, 1.0]) {
        case.nodes[0].displacements.fill(value);
        case.nodes[0].reactions.fill(value);
        case.members[0].end_forces_local.fill(value);
        for station in &mut case.members[0].stations {
            station.forces_local.fill(value);
        }
    }
    cases.reverse(); // The repository does not define the accumulation order.
    for (terms, expected) in [
        ([("A", 1.0), ("B", 1.0), ("C", 1.0)], 1.0),
        ([("A", 1.0), ("C", 1.0), ("B", 1.0)], 0.0),
    ] {
        let output = combine(&declaration(&terms), &cases)?;
        for actual in components(&output.results) {
            bit_equal(actual, expected);
        }
        for extreme in &output.results.members[0].extremes {
            bit_equal(extreme.max.value, expected);
            bit_equal(extreme.min.value, expected);
        }
    }
    Ok(())
}

#[test]
fn superposition_matches_direct_static_linear_and_closed_form() -> EResult<()> {
    for factors in [[2.0, 0.5], [-0.5, 2.0], [0.0, -0.5], [1.3, -0.7]] {
        let cases = cantilever(factors, false)?;
        // Derived formulas: ux=P L/EA; uy=P L³/(3EI)+qL⁴/(8EI),
        // rz=P L²/(2EI)+qL³/(6EI). Without uniform load: A ux=3; B uy=-4, rz=-3.
        close(cases[0].nodes[1].displacements[0], 3.0);
        close(cases[1].nodes[1].displacements[1], -4.0);
        close(cases[1].nodes[1].displacements[5], -3.0);
        let combined = combine(
            &declaration(&[("A", factors[0]), ("B", factors[1])]),
            &cases,
        )?;
        for (index, (actual, direct)) in components(&combined.results)
            .into_iter()
            .zip(components(&cases[2]))
            .enumerate()
        {
            // With zero axial force, recovering N=−Ni can produce −0 versus
            // +0 from summing from zero. Zero is checked algebraically.
            if factors != [1.3, -0.7] && factors[0] != 0.0 {
                assert_eq!(
                    actual.to_bits(),
                    direct.to_bits(),
                    "factors={factors:?}, component={index}: {actual} != {direct}"
                );
            } else {
                close(actual, direct);
            }
        }
        for (actual, direct) in combined.results.members[0]
            .extremes
            .iter()
            .zip(&cases[2].members[0].extremes)
        {
            close(actual.max.value, direct.max.value);
            close(actual.min.value, direct.min.value);
            assert_eq!(actual.max.at_station, direct.max.at_station);
            assert_eq!(actual.min.at_station, direct.min.at_station);
        }
    }
    Ok(())
}

#[test]
fn combined_nodal_and_member_loads_match_direct_and_analytical_diagram() -> EResult<()> {
    for factors in [[2.0, 0.5], [-0.5, 2.0], [0.0, -0.5], [1.3, -0.7]] {
        let cases = cantilever(factors, true)?;
        close(cases[1].nodes[1].displacements[1], -5.5);
        close(cases[1].nodes[1].displacements[5], -4.0);
        let c = combine(
            &declaration(&[("A", factors[0]), ("B", factors[1])]),
            &cases,
        )?;
        // Exact equality with the input fields; the direct solve
        // allows rounding proportional to the scale of each member field.
        let a = components(&cases[0]);
        let b = components(&cases[1]);
        for (i, actual) in components(&c.results).into_iter().enumerate() {
            let mut expected = 0.0;
            expected += factors[0] * a[i];
            expected += factors[1] * b[i];
            bit_equal(actual, expected);
        }
        for (actual, direct) in c.results.nodes.iter().zip(&cases[2].nodes) {
            for (actual, direct) in actual
                .displacements
                .into_iter()
                .zip(direct.displacements)
                .chain(actual.reactions.into_iter().zip(direct.reactions))
            {
                close(actual, direct);
            }
        }
        let member = &c.results.members[0];
        let direct = &cases[2].members[0];
        let field_scale: [f64; 6] = std::array::from_fn(|i| {
            member
                .stations
                .iter()
                .chain(&direct.stations)
                .map(|s| s.forces_local[i].abs())
                .fold(1.0, f64::max)
        });
        let end_scale = member
            .end_forces_local
            .iter()
            .chain(&direct.end_forces_local)
            .map(|v| v.abs())
            .fold(1.0, f64::max);
        for (&actual, &direct) in member.end_forces_local.iter().zip(&direct.end_forces_local) {
            close_scaled(actual, direct, end_scale);
        }
        for (actual, direct) in member.stations.iter().zip(&direct.stations) {
            assert_eq!(actual.position, direct.position);
            for ((actual, direct), scale) in actual
                .forces_local
                .into_iter()
                .zip(direct.forces_local)
                .zip(field_scale)
            {
                close_scaled(actual, direct, scale);
            }
        }
        close(c.results.nodes[0].reactions[0], -48.0 * factors[0]);
        close(c.results.nodes[0].reactions[1], 96.0 * factors[1]);
        close(c.results.nodes[0].reactions[5], 144.0 * factors[1]);
        for s in &c.results.members[0].stations {
            let remaining = 2.0 * (1.0 - s.position);
            close_scaled(
                s.forces_local[0],
                48.0 * factors[0],
                48.0 * factors[0].abs(),
            );
            close(s.forces_local[1], factors[1] * (-48.0 - 24.0 * remaining));
            close_scaled(
                s.forces_local[5],
                factors[1] * (-48.0 * remaining - 12.0 * remaining * remaining),
                144.0 * factors[1].abs(),
            );
        }
        for ((actual, direct), scale) in member
            .extremes
            .iter()
            .zip(&direct.extremes)
            .zip(field_scale)
        {
            close_scaled(actual.max.value, direct.max.value, scale);
            close_scaled(actual.min.value, direct.min.value, scale);
            assert_eq!(actual.max.at_station, direct.max.at_station);
            assert_eq!(actual.min.at_station, direct.min.at_station);
        }
    }
    Ok(())
}

#[test]
fn envelope_governs_by_component_sign_station_and_kind() {
    let mut a = fixture("A", 1.0);
    let mut b = fixture("B", 1.0);
    a.nodes[0].displacements = [9.0, -8.0, 0.0, 1.0, -2.0, 3.0];
    b.nodes[0].displacements = [-7.0, 6.0, 0.0, -1.0, 2.0, -3.0];
    a.nodes[0].reactions = [-5.0, 4.0, 0.0, -1.0, 2.0, -3.0];
    b.nodes[0].reactions = [3.0, -2.0, 0.0, 1.0, -2.0, 3.0];
    a.members[0].stations[1].forces_local = [10.0, -20.0, 30.0, -40.0, 50.0, -60.0];
    b.members[0].stations[1].forces_local = [-11.0, 21.0, -31.0, 41.0, -51.0, 61.0];
    a.members[0].end_forces_local = vec![8.0, -9.0, 10.0, -11.0, 12.0, -13.0];
    b.members[0].end_forces_local = vec![-14.0, 15.0, -16.0, 17.0, -18.0, 19.0];
    let mut c = combine(
        &declaration(&[("A", 0.25), ("B", 0.25)]),
        &[a.clone(), b.clone()],
    )
    .unwrap();
    // Same name as a case, distinct identities.
    c.id = "A".into();
    let e = envelope(&[
        EnvelopeInput::Case(&a),
        EnvelopeInput::Combination(&c),
        EnvelopeInput::Case(&b),
    ])
    .unwrap();
    assert_eq!(
        e.sources,
        [
            ResultId::Case("A".into()),
            ResultId::Combination("A".into()),
            ResultId::Case("B".into())
        ]
    );
    let aid = ResultId::Case("A".into());
    let bid = ResultId::Case("B".into());
    let n = &e.nodes[0];
    for i in [0, 3, 5] {
        assert_eq!(n.displacements[i].max.source, aid);
        assert_eq!(n.displacements[i].min.source, bid);
        assert_eq!(n.reactions[i].max.source, bid);
        assert_eq!(n.reactions[i].min.source, aid);
    }
    for i in [1, 4] {
        assert_eq!(n.displacements[i].max.source, bid);
        assert_eq!(n.displacements[i].min.source, aid);
        assert_eq!(n.reactions[i].max.source, aid);
        assert_eq!(n.reactions[i].min.source, bid);
    }
    for i in 0..6 {
        let (max_id, min_id) = if i % 2 == 0 {
            (&aid, &bid)
        } else {
            (&bid, &aid)
        };
        let m = &e.members[0];
        let s = &m.stations[1].forces_local[i];
        assert_eq!(&s.max.source, max_id);
        assert_eq!(&s.min.source, min_id);
        assert_eq!(
            s.max.value,
            a.members[0].stations[1].forces_local[i].max(b.members[0].stations[1].forces_local[i])
        );
        assert_eq!(
            s.min.value,
            a.members[0].stations[1].forces_local[i].min(b.members[0].stations[1].forces_local[i])
        );
        assert_eq!(&m.end_forces_local[i].max.source, max_id);
        assert_eq!(&m.end_forces_local[i].min.source, min_id);
        assert_eq!(&m.extremes[i].max.source, max_id);
        assert_eq!(&m.extremes[i].min.source, min_id);
        assert_eq!(m.extremes[i].max.extreme.value, s.max.value);
        assert_eq!(m.extremes[i].min.extreme.value, s.min.value);
        assert_eq!(m.extremes[i].max.extreme.at_station, 0.5);
        assert_eq!(m.extremes[i].min.extreme.at_station, 0.5);
    }
    // A combination can also govern.
    let c = combine(&declaration(&[("A", -4.0)]), std::slice::from_ref(&a)).unwrap();
    let e = envelope(&[EnvelopeInput::Case(&a), EnvelopeInput::Combination(&c)]).unwrap();
    assert_eq!(
        e.nodes[0].displacements[0].min.source,
        ResultId::Combination("C".into())
    );
    assert_eq!(e.nodes[0].displacements[0].min.value, -36.0);
    assert_eq!(
        e.nodes[0].displacements[1].max.source,
        ResultId::Combination("C".into())
    );
}

#[test]
fn exact_ties_keep_first_source_and_first_station() {
    let a = fixture("Z", 0.0);
    let b = fixture("A", -0.0);
    for (first, second) in [(&a, &b), (&b, &a)] {
        let e = envelope(&[EnvelopeInput::Case(first), EnvelopeInput::Case(second)]).unwrap();
        let id = ResultId::Case(first.case.0.clone());
        for v in e.nodes[0]
            .displacements
            .iter()
            .chain(&e.nodes[0].reactions)
            .chain(&e.members[0].end_forces_local)
            .chain(e.members[0].stations.iter().flat_map(|s| &s.forces_local))
        {
            assert_eq!(v.max.source, id);
            assert_eq!(v.min.source, id);
        }
        for e in &e.members[0].extremes {
            assert_eq!(e.max.source, id);
            assert_eq!(e.min.source, id);
            assert_eq!(e.max.extreme.at_station, 0.0);
            assert_eq!(e.min.extreme.at_station, 0.0);
        }
    }
    let e = envelope(&[EnvelopeInput::Case(&a)]).unwrap();
    assert_eq!(e.members[0].extremes[5].field, Field::Mz);
}

fn rejects_pair(a: &CaseResults, b: &CaseResults, code: &str) {
    assert_eq!(
        combine(
            &declaration(&[("A", 1.0), ("B", 0.0)]),
            &[a.clone(), b.clone()]
        )
        .unwrap_err()
        .code,
        code
    );
    assert_eq!(
        envelope(&[EnvelopeInput::Case(a), EnvelopeInput::Case(b)])
            .unwrap_err()
            .code,
        code
    );
}

#[test]
fn inconsistent_stations_sizes_topology_and_duplicates() {
    let a = fixture("A", 1.0);
    let b = fixture("B", 2.0);
    for variant in 0..12 {
        let mut bad = b.clone();
        match variant {
            0 => bad.members[0].stations[1].position = 0.5000000000000001,
            1 => {
                bad.members[0].stations.remove(1);
            }
            2 => bad.members[0].stations[1].position = 0.0,
            3 => bad.members[0].stations.swap(0, 1),
            4 => bad.members[0].stations[1].position = -0.1,
            5 => bad.members[0].stations.clear(),
            6 => {
                bad.members[0].end_forces_local.pop();
            }
            7 => bad.nodes[0].id = NodeId(99),
            8 => bad.members[0].id = MemberId(99),
            9 => bad.nodes.push(bad.nodes[0].clone()),
            10 => bad.members.push(bad.members[0].clone()),
            _ => bad.members[0].stations[0].position = 0.1,
        }
        rejects_pair(&a, &bad, "ENG-E-0010");
    }
    let mut b = b;
    b.members[0].stations[1].position = 0.25;
    let err = combine(&declaration(&[("A", 1.0), ("B", 1.0)]), &[a, b]).unwrap_err();
    assert!(err
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Member(MemberId(13)))));
}

#[test]
fn invalid_lists_missing_case_and_nonfinite_factors() {
    let a = fixture("A", 1.0);
    assert_eq!(
        combine(&declaration(&[]), std::slice::from_ref(&a))
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    assert_eq!(
        combine(&declaration(&[("A", 1.0)]), &[]).unwrap_err().code,
        "ENG-E-0010"
    );
    assert_eq!(
        combine(
            &declaration(&[("A", 1.0), ("A", 0.0)]),
            std::slice::from_ref(&a)
        )
        .unwrap_err()
        .code,
        "ENG-E-0010"
    );
    assert_eq!(
        combine(&declaration(&[("A", 1.0)]), &[a.clone(), a.clone()])
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    for factor in [1.0, 0.0] {
        let err = combine(
            &declaration(&[("ausente", factor)]),
            std::slice::from_ref(&a),
        )
        .unwrap_err();
        assert_eq!(err.code, "ENG-E-0003");
        assert!(matches!(err.entities.as_slice(), [EntityRef::Case(name)] if name == "ausente"));
    }
    for factor in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            combine(&declaration(&[("A", factor)]), std::slice::from_ref(&a))
                .unwrap_err()
                .code,
            "ENG-E-0008"
        );
    }
    assert_eq!(envelope(&[]).unwrap_err().code, "ENG-E-0010");
    assert_eq!(
        envelope(&[EnvelopeInput::Case(&a), EnvelopeInput::Case(&a)])
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    let c = combine(&declaration(&[("A", 1.0)]), &[a]).unwrap();
    assert_eq!(
        envelope(&[
            EnvelopeInput::Combination(&c),
            EnvelopeInput::Combination(&c)
        ])
        .unwrap_err()
        .code,
        "ENG-E-0010"
    );
}

#[test]
fn nonfinite_data_are_rejected_even_with_zero_factor() {
    let a = fixture("A", 1.0);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for variant in 0..6 {
            let mut b = fixture("B", 2.0);
            match variant {
                0 => b.nodes[0].displacements[5] = value,
                1 => b.nodes[0].reactions[5] = value,
                2 => b.members[0].end_forces_local[5] = value,
                3 => b.members[0].stations[1].forces_local[5] = value,
                4 => b.members[0].stations[1].position = value,
                _ => b.factor = value,
            }
            rejects_pair(&a, &b, "ENG-E-0008");
        }
    }
}

#[test]
fn extreme_finite_values_overflow_and_subnormal_without_panics() {
    let mut a = fixture("A", 0.0);
    a.nodes[0].displacements[0] = f64::MAX;
    let mut b = a.clone();
    b.case.0 = "B".into();
    b.nodes[0].displacements[0] = -f64::MAX;
    let e = envelope(&[EnvelopeInput::Case(&a), EnvelopeInput::Case(&b)]).unwrap();
    assert_eq!(e.nodes[0].displacements[0].max.value, f64::MAX);
    assert_eq!(e.nodes[0].displacements[0].min.value, -f64::MAX);
    let cases = [a, b];
    for terms in [
        vec![("A", 2.0)],
        vec![("A", 1.0), ("B", -1.0)],
        vec![("A", 2.0), ("B", 2.0)],
    ] {
        assert_eq!(
            combine(&declaration(&terms), &cases).unwrap_err().code,
            "ENG-E-0008"
        );
    }
    let r = combine(&declaration(&[("A", 1.0), ("B", 1.0)]), &cases).unwrap();
    assert_eq!(r.results.nodes[0].displacements[0], 0.0);
    let mut tiny = fixture("A", 0.0);
    tiny.nodes[0].displacements[0] = f64::from_bits(1);
    let r = combine(&declaration(&[("A", 2.0)]), &[tiny]).unwrap();
    bit_equal(r.results.nodes[0].displacements[0], f64::from_bits(2));
    let mut zero = fixture("Z", 0.0);
    zero.members.clear();
    assert!(combine(
        &declaration(&[("Z", f64::MAX)]),
        std::slice::from_ref(&zero)
    )
    .is_ok());
    assert!(envelope(&[EnvelopeInput::Case(&zero)]).is_ok());
    zero.nodes.clear();
    assert_eq!(
        envelope(&[EnvelopeInput::Case(&zero)]).unwrap_err().code,
        "ENG-E-0010"
    );
}

#[test]
fn match_entities_by_id_and_sort_output() {
    let mut a = fixture("A", 1.0);
    let mut node = a.nodes[0].clone();
    node.id = NodeId(2);
    node.displacements = [4.0; 6];
    a.nodes.push(node);
    let mut m = a.members[0].clone();
    m.id = MemberId(1);
    m.end_forces_local.fill(10.0);
    for station in &mut m.stations {
        station.forces_local.fill(20.0);
    }
    a.members.push(m);
    let mut b = a.clone();
    b.case.0 = "B".into();
    b.nodes[0].displacements.fill(6.0);
    b.nodes[0].reactions.fill(7.0);
    b.nodes[1].displacements.fill(8.0);
    b.nodes[1].reactions.fill(9.0);
    b.members[0].end_forces_local.fill(30.0);
    b.members[1].end_forces_local.fill(40.0);
    for station in &mut b.members[0].stations {
        station.forces_local.fill(50.0);
    }
    for station in &mut b.members[1].stations {
        station.forces_local.fill(60.0);
    }
    b.nodes.reverse();
    b.members.reverse();
    let c = combine(
        &declaration(&[("A", 2.0), ("B", -0.5)]),
        &[a.clone(), b.clone()],
    )
    .unwrap();
    assert_eq!(c.results.nodes[0].id, NodeId(2));
    assert_eq!(c.results.nodes[0].displacements, [4.0; 6]);
    assert_eq!(
        c.results.nodes[0].reactions,
        a.nodes[1].reactions.map(|r| 2.0 * r - 4.5)
    );
    assert_eq!(c.results.nodes[1].id, NodeId(7));
    assert_eq!(
        c.results.nodes[1].displacements,
        a.nodes[0].displacements.map(|u| 2.0 * u - 3.0)
    );
    assert_eq!(
        c.results.nodes[1].reactions,
        a.nodes[0].reactions.map(|r| 2.0 * r - 3.5)
    );
    assert_eq!(c.results.members[0].id, MemberId(1));
    assert_eq!(c.results.members[0].end_forces_local, [0.0; 6]);
    assert_eq!(c.results.members[1].id, MemberId(13));
    assert_eq!(
        c.results.members[1].end_forces_local,
        a.members[0]
            .end_forces_local
            .iter()
            .map(|&f| 2.0 * f - 15.0)
            .collect::<Vec<_>>()
    );
    for station in &c.results.members[0].stations {
        assert_eq!(station.forces_local, [10.0; 6]);
    }
    for (actual, source) in c.results.members[1]
        .stations
        .iter()
        .zip(&a.members[0].stations)
    {
        assert_eq!(
            actual.forces_local,
            source.forces_local.map(|f| 2.0 * f - 25.0)
        );
    }
    let e = envelope(&[EnvelopeInput::Case(&a), EnvelopeInput::Case(&b)]).unwrap();
    assert_eq!(e.nodes[0].id, NodeId(2));
    assert_eq!(e.members[0].id, MemberId(1));
    assert_eq!(e.nodes[0].displacements[0].max.value, 8.0);
    assert_eq!(e.nodes[0].displacements[0].min.value, 4.0);
    assert_eq!(e.members[0].end_forces_local[0].max.value, 40.0);
    assert_eq!(e.members[0].end_forces_local[0].min.value, 10.0);
    assert_eq!(e.members[1].stations[0].forces_local[0].max.value, 50.0);
    assert_eq!(e.members[1].stations[0].forces_local[0].min.value, 1.0);
}
