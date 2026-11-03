//! Fixed-end bar, uniform temperature. Own closed form, SI.
//! Gradient is not a case here.
//! source: derivación propia, N de tracción = -(E A α ΔT), f = -N b.
//! citation: provenance/ENG-LOAD-TEMP-972.md; sin cita externa.
use engine_assembly::Reaction;
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams};
use engine_core::{Dim, Dof, NodeId};
use engine_elements_truss::LinearTruss;
use engine_linalg::ReferenceBackend;
use engine_model::{LoadCase, Model, Node, Support};

/// Absolute newtons for the diagonal and spatial bars.
///
/// Aligned bars compare with equality: E, A, α and ΔT are dyadic and the
/// product 8*4*0.5*3 = 48 is exact. The 1e-12 N bound is the declared
/// tolerance for a rounded direction cosine. The provenance derives it
/// from the unit roundoff; it is larger than the first-order error of
/// these fixtures (under 1e-13 N).
const FORCE_TOL: f64 = 1e-12;

const SOURCE: &str = "analítico: derivación propia N=-(E A α ΔT) para barra empotrada-empotrada";
const CITATION: &str = "provenance/ENG-LOAD-TEMP-972.md; sin cita externa";

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= FORCE_TOL,
        "{actual} != {expected}; tol={FORCE_TOL}"
    );
}

fn bar() -> LinearTruss {
    LinearTruss::new(Dim::D2, 8.0, 4.0, 0.0).unwrap()
}

fn bar_dim(dim: Dim) -> LinearTruss {
    LinearTruss::new(dim, 8.0, 4.0, 0.0).unwrap()
}

fn ctx<'a>(dim: Dim, coords: &'a [[f64; 3]], params: &'a ElementParams) -> ElementCtx<'a> {
    ElementCtx {
        dim,
        coords,
        params,
    }
}

/// Hand magnitude E·A·α·ΔT. Not a call into the bar.
fn hand_magnitude() -> f64 {
    8.0 * 4.0 * 0.5 * 3.0
}

#[test]
fn citation_is_present_and_axial_force_is_exact() {
    assert!(!SOURCE.is_empty() && !CITATION.is_empty());
    let magnitude = hand_magnitude();
    assert_eq!(magnitude, 48.0);
    let params = ElementParams::default();
    let coords = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
    let thermal = bar()
        .fixed_end_uniform_temperature(&ctx(Dim::D2, &coords, &params), 0.5, 3.0, &[0.0, 0.5, 1.0])
        .unwrap();
    assert_eq!(thermal.axial_force, -magnitude);
    assert_eq!(thermal.joint_forces, vec![-48.0, 0.0, 48.0, 0.0]);
    assert_eq!(thermal.output.end_forces_local, vec![48.0, 0.0, -48.0, 0.0]);
    for station in &thermal.output.stations {
        assert_eq!(station.forces_local[0], -48.0);
        assert_eq!(&station.forces_local[1..], &[0.0; 5]);
    }
}

#[test]
fn axial_force_does_not_depend_on_length_or_direction() {
    let params = ElementParams::default();
    let short = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
    let long = [[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]];
    let reversed = [[2.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let element = bar();
    let n_short = element
        .fixed_end_uniform_temperature(&ctx(Dim::D2, &short, &params), 0.5, 3.0, &[])
        .unwrap();
    let n_long = element
        .fixed_end_uniform_temperature(&ctx(Dim::D2, &long, &params), 0.5, 3.0, &[])
        .unwrap();
    let n_reversed = element
        .fixed_end_uniform_temperature(&ctx(Dim::D2, &reversed, &params), 0.5, 3.0, &[])
        .unwrap();
    assert_eq!(n_short.axial_force, -48.0);
    assert_eq!(n_long.axial_force, n_short.axial_force);
    assert_eq!(n_reversed.axial_force, -48.0);
    assert_eq!(n_reversed.joint_forces, vec![48.0, 0.0, -48.0, 0.0]);
    let spatial = bar_dim(Dim::D3);
    let vertical = [[0.0, 0.0, 0.0], [0.0, 0.0, 2.0]];
    let up = spatial
        .fixed_end_uniform_temperature(&ctx(Dim::D3, &vertical, &params), 0.5, 3.0, &[0.25])
        .unwrap();
    assert_eq!(up.axial_force, -48.0);
    assert_eq!(up.joint_forces, vec![0.0, 0.0, -48.0, 0.0, 0.0, 48.0]);
    assert_eq!(up.output.stations[0].forces_local[0], -48.0);
    // Local slots stay axial: index 0 is -N and index `axes` is +N.
    assert_eq!(
        up.output.end_forces_local,
        vec![48.0, 0.0, 0.0, -48.0, 0.0, 0.0]
    );
}

#[test]
fn cooling_and_negative_alpha_flip_the_sign_and_zero_stays_zero() {
    let params = ElementParams::default();
    let coords = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
    let c = ctx(Dim::D2, &coords, &params);
    let element = bar();
    let cooling = element
        .fixed_end_uniform_temperature(&c, 0.5, -3.0, &[1.0])
        .unwrap();
    assert_eq!(cooling.axial_force, 48.0);
    assert_eq!(cooling.joint_forces, vec![48.0, 0.0, -48.0, 0.0]);
    let negative_alpha = element
        .fixed_end_uniform_temperature(&c, -0.5, 3.0, &[1.0])
        .unwrap();
    assert_eq!(negative_alpha.axial_force, cooling.axial_force);
    assert_eq!(negative_alpha.joint_forces, cooling.joint_forces);
    for (alpha, delta_t) in [(0.0, 3.0), (0.5, 0.0)] {
        let zero = element
            .fixed_end_uniform_temperature(&c, alpha, delta_t, &[0.0])
            .unwrap();
        assert_eq!(zero.axial_force, 0.0);
        assert_eq!(zero.joint_forces, vec![0.0; 4]);
        assert_eq!(zero.output.end_forces_local, vec![0.0; 4]);
    }
}

#[test]
fn diagonal_and_spatial_directions_match_the_hand_projection() {
    let params = ElementParams::default();
    let magnitude = hand_magnitude();
    // d = (3/5, 4/5). Components are the hand quotients, not `b()` of the bar.
    let fx = (magnitude * 3.0) / 5.0;
    let fy = (magnitude * 4.0) / 5.0;
    let coords = [[0.0, 0.0, 0.0], [3.0, 4.0, 0.0]];
    let thermal = bar()
        .fixed_end_uniform_temperature(&ctx(Dim::D2, &coords, &params), 0.5, 3.0, &[0.0, 1.0])
        .unwrap();
    assert_eq!(thermal.axial_force, -magnitude);
    let expected = [-fx, -fy, fx, fy];
    for (actual, component) in thermal.joint_forces.iter().zip(expected) {
        close(*actual, component);
    }
    assert_eq!(thermal.output.end_forces_local[0], magnitude);
    assert_eq!(thermal.output.end_forces_local[2], -magnitude);
    // (2, 3, 6), L = 7. Hand components (E A α ΔT) * component / 7.
    let spatial = [[0.0, 0.0, 0.0], [2.0, 3.0, 6.0]];
    let along = [2.0, 3.0, 6.0];
    let body = bar_dim(Dim::D3)
        .fixed_end_uniform_temperature(&ctx(Dim::D3, &spatial, &params), 0.5, 3.0, &[0.5])
        .unwrap();
    assert_eq!(body.axial_force, -magnitude);
    for axis in 0..3 {
        let component = (magnitude * along[axis]) / 7.0;
        close(body.joint_forces[axis], -component);
        close(body.joint_forces[3 + axis], component);
    }
    let mut sum = [0.0; 3];
    for axis in 0..3 {
        sum[axis] = body.joint_forces[axis] + body.joint_forces[3 + axis];
        close(sum[axis], 0.0);
    }
}

fn reaction(reactions: &[Reaction], node: NodeId, dof: Dof) -> f64 {
    reactions
        .iter()
        .find(|item| item.node == node && item.dof == dof)
        .unwrap()
        .value
}

fn restrained(dofs: &[Dof], free: Option<Dof>) -> Vec<Dof> {
    dofs.iter()
        .copied()
        .filter(|dof| Some(*dof) != free)
        .collect()
}

#[test]
fn fixed_axial_reactions_use_the_existing_load_channel() {
    // The existing assembler rejects a model with no free DOF.
    let locked = bar();
    let dofs = locked.dofs_per_node();
    let mut blocked = Model::new();
    for (i, coordinates) in [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]].iter().enumerate() {
        let mut node = Node::new(NodeId(i as u32 + 1), Dim::D2, *coordinates);
        node.dofs = dofs.to_vec();
        blocked.nodes.push(node);
        blocked.supports.push(Support {
            id: format!("s{i}"),
            node: NodeId(i as u32 + 1),
            restrained: dofs.to_vec(),
        });
    }
    blocked.cases.push(LoadCase {
        name: "T".into(),
        loads: vec![],
    });
    let refused = match LinearSystem::from_model(ReferenceBackend, &blocked, &[], &[]) {
        Err(error) => error,
        Ok(_) => panic!("a fully fixed model must be rejected"),
    };
    assert_eq!(refused.code, "ENG-E-0010");

    // Both axial ends stay fixed. One collinear DOF of a second, unloaded bar
    // is free so the assembler has an unknown. That unknown solves to 0.
    for (dim, coords, third, axial, transverse) in [
        (
            Dim::D2,
            [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            [4.0, 0.0, 0.0],
            Dof::Ux,
            Dof::Uy,
        ),
        (
            Dim::D3,
            [[0.0, 0.0, 0.0], [0.0, 0.0, 2.0]],
            [0.0, 0.0, 4.0],
            Dof::Uz,
            Dof::Ux,
        ),
    ] {
        let bar = bar_dim(dim);
        let dofs = bar.dofs_per_node();
        let params = ElementParams::default();
        let thermal = bar
            .fixed_end_uniform_temperature(&ctx(dim, &coords, &params), 0.5, 3.0, &[0.5])
            .unwrap();
        let mechanical = bar
            .recover(
                &ctx(dim, &coords, &params),
                &vec![0.0; 2 * dofs.len()],
                &[],
                &[],
                &[0.5],
            )
            .unwrap();
        assert_eq!(mechanical.stations[0].forces_local[0], 0.0);
        let mut model = Model::new();
        let points = [coords[0], coords[1], third];
        for (i, coordinates) in points.iter().enumerate() {
            let mut node = Node::new(NodeId(i as u32 + 1), dim, *coordinates);
            node.dofs = dofs.to_vec();
            model.nodes.push(node);
            let free = (i == 2).then_some(axial);
            model.supports.push(Support {
                id: format!("s{i}"),
                node: NodeId(i as u32 + 1),
                restrained: restrained(dofs, free),
            });
        }
        model.cases.push(LoadCase {
            name: "T".into(),
            loads: vec![],
        });
        let map = DofMap::new(&model.nodes).unwrap();
        let locate = |pair: [NodeId; 2]| {
            pair.into_iter()
                .flat_map(|node| {
                    let map = &map;
                    dofs.iter()
                        .map(move |&dof| map.global_index(node, dof).unwrap())
                })
                .collect()
        };
        let zero = vec![0.0; 2 * dofs.len()];
        let contribution = |pair: [NodeId; 2], coords: [[f64; 3]; 2], forces: Vec<f64>| {
            let ctx = ctx(dim, &coords, &params);
            ElementContribution {
                location: locate(pair),
                stiffness: bar.respond(&ctx, &zero, &[]).unwrap().tangent,
                mass_lumped: None,
                mass_consistent: None,
                geometric: None,
                loads: vec![ElementLoad {
                    case: CaseRef("T".into()),
                    values: forces,
                }],
            }
        };
        let heated = contribution([NodeId(1), NodeId(2)], coords, thermal.joint_forces);
        let idle = contribution(
            [NodeId(2), NodeId(3)],
            [coords[1], third],
            vec![0.0; 2 * dofs.len()],
        );
        let case = CaseRef("T".into());
        let mut system =
            LinearSystem::from_model(ReferenceBackend, &model, &[heated, idle], &[]).unwrap();
        let u = system.solve(&case, 1.0).unwrap();
        assert_eq!(u, vec![0.0]);
        let reactions = system.reactions(&u, &case, 1.0).unwrap();
        assert_eq!(reaction(&reactions, NodeId(1), axial), 48.0);
        assert_eq!(reaction(&reactions, NodeId(2), axial), -48.0);
        assert_eq!(reaction(&reactions, NodeId(1), transverse), 0.0);
        assert_eq!(reaction(&reactions, NodeId(2), transverse), 0.0);
    }
}

#[test]
fn nonfinite_overflow_and_geometry_use_existing_codes() {
    let params = ElementParams::default();
    let coords = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
    let element = bar();
    let code = |alpha, delta_t, stations: &[f64]| {
        element
            .fixed_end_uniform_temperature(
                &ctx(Dim::D2, &coords, &params),
                alpha,
                delta_t,
                stations,
            )
            .err()
            .expect("must fail")
            .code
    };
    assert_eq!(code(f64::NAN, 3.0, &[]), "ENG-E-0008");
    assert_eq!(code(0.5, f64::INFINITY, &[]), "ENG-E-0008");
    assert_eq!(code(1e308, 1e308, &[]), "ENG-E-0008");
    assert_eq!(code(0.5, 3.0, &[f64::NAN]), "ENG-E-0008");
    assert_eq!(code(0.5, 3.0, &[1.1]), "ENG-E-0010");
    let collapsed = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let short = element
        .fixed_end_uniform_temperature(&ctx(Dim::D2, &collapsed, &params), 0.5, 3.0, &[])
        .err()
        .expect("must fail");
    assert_eq!(short.code, "ENG-E-0005");
    let lifted = [[0.0, 0.0, 0.0], [2.0, 0.0, 1.0]];
    let spatial = element
        .fixed_end_uniform_temperature(&ctx(Dim::D2, &lifted, &params), 0.5, 3.0, &[])
        .err()
        .expect("must fail");
    assert_eq!(spatial.code, "ENG-E-0010");
}
