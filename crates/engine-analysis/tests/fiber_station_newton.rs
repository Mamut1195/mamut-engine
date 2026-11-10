//! Voladizo de `frame2d.fiber_stations` con `StaticNewton`.
//!
//! Fuente: analítica. Apartados 13 a 16 de
//! `docs/fichas/frame2d.fiber_stations.md`. El axil de fluencia de las dos
//! barras, con el hormigón en tracción y por tanto sin tensión, es
//! N★ = fy (A1 + A2). La carga final axial es 3/2 de ese axil. Con b = 1/20
//! la forma cerrada es Ux = 11/100 m, ε_p = 19/800 y α = fy/2. El momento
//! de punta ancla Ux, v y θ con los apartados 15 y 16 de esa ficha, además
//! del equilibrio, la simetría de las estaciones y la historia nula. Esos
//! esperados se redujeron en la ficha antes de este archivo.

use engine_analysis::newton::{
    NewtonElement, NewtonOptions, NewtonResults, NewtonSystem, StaticNewton,
};
use engine_assembly::{DofMap, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, Status};
use engine_core::{Dim, Dof, NodeId};
use engine_elements_frame::FiberStationBar;
use engine_laws::section::{RectangleFiberBending, RectangleFiberBendingParams, SteelBar};
use engine_laws::uniaxial::{KentParkParams, SteelBilinearKinematicParams};
use engine_linalg::ReferenceBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

const LENGTH: f64 = 4.0;
const ELASTIC_MODULUS: f64 = 200_000_000_000.0;
const YIELD_STRESS: f64 = 500_000_000.0;
const HARDENING: f64 = 0.05;

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
                    e: ELASTIC_MODULUS,
                    fy: YIELD_STRESS,
                    b: HARDENING,
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

fn solve(loads: Vec<DofValue>, force_scale: f64) -> NewtonResults {
    let mut model = Model::new();
    model.nodes.push(Node::new(NodeId(1), Dim::D2, [0.0; 3]));
    model
        .nodes
        .push(Node::new(NodeId(2), Dim::D2, [LENGTH, 0.0, 0.0]));
    model.supports.push(Support {
        id: "empotramiento".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux, Dof::Uy, Dof::Rz],
    });
    model.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![NodalLoad {
            node: NodeId(2),
            values: loads,
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
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let element = NewtonElement::new(Box::new(bar), location, &ctx).expect("elemento");
    let linear = LinearSystem::from_model(ReferenceBackend, &model, &[], &[]).expect("sistema");
    let mut system = NewtonSystem::new(linear, vec![element]).expect("newton");
    StaticNewton::run(
        &mut system,
        &CaseRef("P".into()),
        &[0.25, 0.5, 0.75, 1.0],
        NewtonOptions {
            max_iterations: 40,
            force_tolerance: Some(1e-8),
            displacement_tolerance: None,
            energy_tolerance: None,
            force_scale,
            displacement_scale: 1.0,
            energy_scale: 1.0,
        },
    )
    .expect("corrida")
}

fn at(displacement: &[f64], dof: Dof) -> f64 {
    let map_nodes = [
        Node::new(NodeId(1), Dim::D2, [0.0; 3]),
        Node::new(NodeId(2), Dim::D2, [LENGTH, 0.0, 0.0]),
    ];
    let map = DofMap::new(&map_nodes).expect("mapa");
    let index = map.global_index(NodeId(2), dof).expect("índice");
    displacement[index.0 as usize]
}

#[test]
fn static_newton_yields_the_bars_and_accepts_history() {
    let area = 2.0 / 1024.0;
    let tensile = 1.5 * YIELD_STRESS * area;
    let result = solve(
        vec![DofValue {
            dof: Dof::Ux,
            value: tensile,
        }],
        tensile,
    );
    assert_eq!(result.status, Status::Completed);
    assert_eq!(result.accepted_factor, Some(1.0));

    let ux = at(&result.global_displacements, Dof::Ux);
    let uy = at(&result.global_displacements, Dof::Uy);
    let rotation = at(&result.global_displacements, Dof::Rz);
    assert!((ux - 11.0 / 100.0).abs() <= 3e-9, "Ux {ux} frente a 11/100");
    assert!(uy.abs() <= 1e-9, "Uy {uy} rompe la simetría axial");
    assert!(
        rotation.abs() <= 1e-9,
        "θ {rotation} rompe la simetría axial"
    );

    let history = &result.element_states[0];
    assert_eq!(history.len(), 12);
    let plastic = 19.0 / 800.0;
    let alpha = YIELD_STRESS / 2.0;
    for station in 0..3 {
        for bar in 0..2 {
            let base = station * 4 + bar * 2;
            let gap_plastic = (history[base] - plastic).abs();
            let gap_alpha = (history[base + 1] - alpha).abs();
            assert!(
                gap_plastic <= 1e-9,
                "ε_p {gap_plastic} en estación {station}, barra {bar}"
            );
            assert!(
                gap_alpha <= 10.0,
                "α {gap_alpha} en estación {station}, barra {bar}"
            );
        }
    }

    let check = yielded_bar();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let recovered = check
        .recover(&ctx, &result.global_displacements, history, &[], &[0.5])
        .expect("equilibrio");
    let axial = recovered.stations[0].forces_local[0];
    assert!(
        (axial - tensile).abs() <= 1e-6 * tensile.abs(),
        "axil {axial} frente a la carga {tensile}"
    );
    assert!(axial > YIELD_STRESS * area);
}

#[test]
fn static_newton_tip_moment_keeps_equilibrium_sign_and_symmetry() {
    let moment = 1_000.0;
    let result = solve(
        vec![DofValue {
            dof: Dof::Rz,
            value: moment,
        }],
        moment,
    );
    assert_eq!(result.status, Status::Completed);
    assert_eq!(result.accepted_factor, Some(1.0));

    let ux = at(&result.global_displacements, Dof::Ux);
    let uy = at(&result.global_displacements, Dof::Uy);
    let rotation = at(&result.global_displacements, Dof::Rz);
    // Apartado 16: Ux = ε0 L, θ = κ L, v = κ L²/2.
    assert!(
        (ux - 2.0966394932968644e-5).abs() <= 1e-11,
        "Ux {ux} frente a 2.0966394932968644e-5"
    );
    assert!(
        (uy - 3.5890067627056873e-4).abs() <= 1e-10,
        "Uy {uy} frente a 3.5890067627056873e-4"
    );
    assert!(
        (rotation - 1.7945033813528436e-4).abs() <= 3e-11,
        "θ {rotation} frente a 1.7945033813528436e-4"
    );

    let history = &result.element_states[0];
    assert_eq!(history, &vec![0.0; 12]);

    let check = yielded_bar();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let recovered = check
        .recover(
            &ctx,
            &result.global_displacements,
            history,
            &[],
            &[0.0, 0.5, 1.0],
        )
        .expect("equilibrio");
    let gap = 1e-4;
    let expected_force = [0.0, 0.0, -moment, 0.0, 0.0, moment];
    for (actual, expected) in recovered.end_forces_local.iter().zip(expected_force) {
        assert!(
            (actual - expected).abs() <= gap,
            "fuerza nodal {actual} frente a {expected}"
        );
    }
    assert_eq!(recovered.stations.len(), 3);
    for station in &recovered.stations {
        let axial = station.forces_local[0];
        let shear = station.forces_local[1];
        let station_moment = station.forces_local[5];
        assert!(axial.abs() <= gap, "N {axial} en x/L {}", station.position);
        assert!(shear.abs() <= gap, "Vy {shear} en x/L {}", station.position);
        assert!(
            (station_moment - moment).abs() <= gap,
            "Mz {station_moment} en x/L {}",
            station.position
        );
    }
}
