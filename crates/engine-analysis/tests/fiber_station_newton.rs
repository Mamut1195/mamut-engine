//! Voladizo axial de `frame2d.fiber_stations` con `StaticNewton`.
//!
//! Fuente: analítica. El axil de fluencia de las dos barras, con el hormigón
//! en tracción y por tanto sin tensión, es N★ = fy (A1 + A2). La carga final
//! es 3/2 de ese axil. La pendiente posterior a la fluencia es b E, con
//! b = 1/20, así que la tangente axial sigue siendo positiva.
//! Cita: docs/fichas/frame2d.fiber_stations.md, apartado del acero elástico
//! de `steel.bilinear_kinematic`, y crates/engine-analysis/src/newton.rs.

use engine_analysis::newton::{NewtonElement, NewtonOptions, NewtonSystem, StaticNewton};
use engine_assembly::{DofMap, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, Status};
use engine_core::{Dim, Dof, NodeId};
use engine_elements_frame::FiberStationBar;
use engine_laws::section::{RectangleFiberBending, RectangleFiberBendingParams, SteelBar};
use engine_laws::uniaxial::{KentParkParams, SteelBilinearKinematicParams};
use engine_linalg::ReferenceBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

fn yielded_bar() -> FiberStationBar {
    let psi = 4.4482216152605 / (0.0254 * 0.0254);
    FiberStationBar::new(
        RectangleFiberBending::new(
            Dim::D2,
            RectangleFiberBendingParams {
                width: 0.25,
                height: 0.5,
                strip_count: 4,
                concrete: KentParkParams {
                    fc: 3000.0 * psi,
                    rho_s: 0.0,
                    b_double_prime: 0.2,
                    s_h: 0.1,
                },
                steel: SteelBilinearKinematicParams {
                    e: 200_000_000_000.0,
                    fy: 500_000_000.0,
                    b: 0.05,
                },
                bars: vec![
                    SteelBar {
                        area: 1.0 / 1024.0,
                        y: -3.0 / 16.0,
                    },
                    SteelBar {
                        area: 1.0 / 1024.0,
                        y: 3.0 / 16.0,
                    },
                ],
            },
        )
        .expect("sección"),
    )
}

#[test]
fn static_newton_yields_the_bars_and_accepts_history() {
    let tensile = 1.5 * 500_000_000.0 * (2.0 / 1024.0);
    let mut model = Model::new();
    model.nodes.push(Node::new(NodeId(1), Dim::D2, [0.0; 3]));
    model
        .nodes
        .push(Node::new(NodeId(2), Dim::D2, [4.0, 0.0, 0.0]));
    model.supports.push(Support {
        id: "empotramiento".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux, Dof::Uy, Dof::Rz],
    });
    model.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![NodalLoad {
            node: NodeId(2),
            values: vec![DofValue {
                dof: Dof::Ux,
                value: tensile,
            }],
        }],
    });
    let map = DofMap::new(&model.nodes).expect("mapa");
    let mut location = Vec::new();
    for id in [1, 2] {
        for dof in [Dof::Ux, Dof::Uy, Dof::Rz] {
            location.push(map.global_index(NodeId(id), dof).expect("índice"));
        }
    }
    let bar = yielded_bar();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [4.0, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let element = NewtonElement::new(Box::new(bar), location, &ctx).expect("elemento");
    let linear = LinearSystem::from_model(ReferenceBackend, &model, &[], &[]).expect("sistema");
    let mut system = NewtonSystem::new(linear, vec![element]).expect("newton");
    let result = StaticNewton::run(
        &mut system,
        &CaseRef("P".into()),
        &[0.25, 0.5, 0.75, 1.0],
        NewtonOptions {
            max_iterations: 40,
            force_tolerance: Some(1e-8),
            displacement_tolerance: None,
            energy_tolerance: None,
            force_scale: tensile,
            displacement_scale: 1.0,
            energy_scale: 1.0,
        },
    )
    .expect("corrida");
    assert_eq!(result.status, Status::Completed);
    assert_eq!(result.accepted_factor, Some(1.0));
    let history = &result.element_states[0];
    assert!(
        history.iter().step_by(2).any(|plastic| plastic.abs() > 0.0),
        "la historia aceptada debe guardar deformación plástica"
    );
    let check = yielded_bar();
    let recovered = check
        .recover(&ctx, &result.global_displacements, history, &[], &[0.5])
        .expect("equilibrio");
    let axial = recovered.stations[0].forces_local[0];
    assert!(
        (axial - tensile).abs() <= 1e-6 * tensile.abs(),
        "axil {axial} frente a la carga {tensile}"
    );
    assert!(axial > 500_000_000.0 * (2.0 / 1024.0));
}
