//! SOURCE: derivación propia; CITATION: provenance/ENG-ANL-002.md.
//! Sin valores publicados, fuentes externas ni oráculos.
use engine_analysis::modal::{Modal, ModalResults};
use engine_assembly::{DofMap, ElementContribution, LinearSystem, PrescribedDof};
use engine_contracts::{Element, ElementCtx, ElementParams, MassKind, SystemView};
use engine_core::math;
use engine_core::{Dim, Dof, DofIndex, EntityRef, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_linalg::{DMat, ReferenceBackend, SparseMatrix};
use engine_loads::mass::{rigid_translation, system_total_mass};
use engine_model::{DofValue, Model, NodalMass, Node, Support};
use std::f64::consts::{PI, TAU};

fn close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}, tol={tol}");
}
fn support(model: &mut Model, node: u32, dofs: &[Dof]) {
    model.supports.push(Support {
        id: format!("s{node}"),
        node: NodeId(node),
        restrained: dofs.to_vec(),
    });
}
fn dot(matrix: &SparseMatrix, a: &[f64], b: &[f64]) -> f64 {
    let dense = matrix.to_dense().unwrap();
    (0..a.len())
        .map(|i| {
            (0..b.len())
                .map(|j| a[i] * dense.get(i, j).unwrap() * b[j])
                .sum::<f64>()
        })
        .sum()
}
fn orthogonal(system: &LinearSystem<ReferenceBackend>, result: &ModalResults, tol: f64) {
    for (i, a) in result.modes.iter().enumerate() {
        for (j, b) in result.modes.iter().enumerate() {
            close(
                dot(
                    system.global_mass(result.mass_kind),
                    &a.global_shape,
                    &b.global_shape,
                ),
                if i == j { 1.0 } else { 0.0 },
                tol,
            );
            close(
                dot(system.stiffness(), &a.global_shape, &b.global_shape),
                if i == j { a.eigenvalue } else { 0.0 },
                tol * a.eigenvalue.max(b.eigenvalue).max(1.0),
            );
        }
    }
}
fn complete(system: &LinearSystem<ReferenceBackend>, result: &ModalResults) {
    for (d, report) in result.directions.iter().enumerate() {
        let mut r = rigid_translation(system.dof_map(), report.direction).unwrap();
        for (v, prescribed) in r.iter_mut().zip(system.constraints().prescribed_values()) {
            if prescribed.is_some() {
                *v = 0.0;
            }
        }
        let exact = dot(system.global_mass(result.mass_kind), &r, &r);
        let sum = result
            .modes
            .iter()
            .map(|m| m.participation[d].effective_mass)
            .sum::<f64>();
        close(sum, exact, 2e-10 * exact.max(1.0));
        close(report.free_mass, exact, 2e-12 * exact.max(1.0));
        close(
            report.cumulative_effective_mass,
            sum,
            2e-12 * exact.max(1.0),
        );
        close(
            report.total_mass,
            system_total_mass(system, result.mass_kind, report.direction).unwrap(),
            1e-12,
        );
    }
}

// Energía V=½k Σ(q_j−q_{j−1})², q_0=0 y techo libre.
// El último piso tiene diagonal k, no 2k: NO es una cadena entre extremos fijos.
fn shear(n: usize, k: f64, m: f64) -> LinearSystem<ReferenceBackend> {
    let mut model = Model::new();
    for j in 0..=n {
        let mut node = Node::new(NodeId(j as u32 + 1), Dim::D2, [0.0, j as f64, 0.0]);
        node.dofs = vec![Dof::Ux, Dof::Uy];
        model.nodes.push(node);
        support(
            &mut model,
            j as u32 + 1,
            if j == 0 {
                &[Dof::Ux, Dof::Uy]
            } else {
                &[Dof::Uy]
            },
        );
        if j > 0 {
            model.masses.push(NodalMass {
                id: format!("m{j}"),
                node: NodeId(j as u32 + 1),
                values: vec![DofValue {
                    dof: Dof::Ux,
                    value: m,
                }],
            });
        }
    }
    let map = DofMap::new(&model.nodes).unwrap();
    let elements: Vec<_> = (1..=n)
        .map(|j| ElementContribution {
            location: [j, j + 1]
                .map(|id| map.global_index(NodeId(id as u32), Dof::Ux).unwrap())
                .to_vec(),
            stiffness: DMat::new(2, 2, vec![k, -k, -k, k]).unwrap(),
            mass_lumped: None,
            mass_consistent: None,
            geometric: None,
            loads: vec![],
        })
        .collect();
    LinearSystem::from_model(ReferenceBackend, &model, &elements, &[]).unwrap()
}

#[test]
fn equal_storey_shear_closed_form_all_modes_and_participation() {
    for n in [1, 4, 9] {
        for kind in [MassKind::Lumped, MassKind::Consistent] {
            let (k, m) = (17.0, 3.0);
            let mut system = shear(n, k, m);
            let result = Modal::run(
                &mut system,
                n,
                kind,
                &[[5.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
            )
            .unwrap();
            assert_eq!(Modal::KIND, "modal");
            assert_eq!(result.mass_kind, kind);
            assert_eq!(result.rigid_mode_count, 0);
            for (index, mode) in result.modes.iter().enumerate() {
                let theta = (2 * index + 1) as f64 * PI / (2 * n + 1) as f64;
                let omega = 2.0 * math::sqrt(k / m) * math::sin(theta / 2.0);
                close(mode.angular_frequency, omega, 3e-12 * omega);
                close(mode.frequency_hz, omega / TAU, 3e-12 * omega);
                close(mode.period.unwrap(), TAU / omega, 3e-12 * TAU / omega);
                let raw: Vec<_> = (1..=n).map(|j| math::sin(j as f64 * theta)).collect();
                let norm = math::sqrt(m * raw.iter().map(|v| v * v).sum::<f64>());
                let sign = if mode.free_shape[0] * raw[0] < 0.0 {
                    -1.0
                } else {
                    1.0
                };
                for (actual, value) in mode.free_shape.iter().zip(&raw) {
                    close(*actual, sign * value / norm, 3e-12);
                }
                let projection = m * raw.iter().sum::<f64>() / norm * sign;
                close(mode.participation[0].factor, projection, 3e-12);
                close(
                    mode.participation[0].effective_mass,
                    math::powi(projection, 2),
                    3e-11,
                );
                close(mode.participation[2].factor, -projection, 3e-12);
                close(
                    mode.participation[2].effective_mass,
                    math::powi(projection, 2),
                    3e-11,
                );
                assert_eq!(mode.participation[1].mass_fraction, None);
                assert_eq!(mode.participation[1].effective_mass, 0.0);
                let cumulative: f64 = result.modes[..=index]
                    .iter()
                    .map(|m| m.participation[0].effective_mass)
                    .sum();
                close(
                    mode.participation[0].cumulative_effective_mass,
                    cumulative,
                    1e-12,
                );
                close(
                    mode.participation[0].mass_fraction.unwrap(),
                    math::powi(projection, 2) / (n as f64 * m),
                    3e-12,
                );
                close(
                    mode.participation[0].cumulative_mass_fraction.unwrap(),
                    cumulative / (n as f64 * m),
                    3e-12,
                );
            }
            close(
                result.directions[0].cumulative_mass_fraction.unwrap(),
                1.0,
                3e-12,
            );
            assert!(result
                .modes
                .windows(2)
                .all(|p| p[0].eigenvalue < p[1].eigenvalue));
            orthogonal(&system, &result, 3e-12);
            complete(&system, &result);
            let partial = Modal::run(&mut system, 1, kind, &[[1.0, 0.0, 0.0]]).unwrap();
            close(
                partial.modes[0].eigenvalue,
                result.modes[0].eigenvalue,
                1e-12,
            );
            close(
                partial.directions[0].cumulative_effective_mass,
                result.modes[0].participation[0].effective_mass,
                1e-12,
            );
        }
    }
}

fn beam(elements: usize, cantilever: bool, angle: f64) -> LinearSystem<ReferenceBackend> {
    let mut model = Model::new();
    for j in 0..=elements {
        let x = j as f64 / elements as f64;
        model.nodes.push(Node::new(
            NodeId(j as u32 + 1),
            Dim::D2,
            [x * math::cos(angle), x * math::sin(angle), 0.0],
        ));
    }
    if cantilever {
        support(&mut model, 1, Dim::D2.dofs());
    } else {
        support(&mut model, 1, &[Dof::Ux, Dof::Uy]);
        support(&mut model, elements as u32 + 1, &[Dof::Ux, Dof::Uy]);
    }
    // I=1e−4 hace que los tres primeros modos sean flexionales incluso con
    // GDL axiales presentes. E=ρ=A=L=1 ⇒ ω=0.01·(βL)².
    let element = EulerFrame2D::new(1.0, 1.0, 1e-4, 1.0, MomentReleases::default()).unwrap();
    let map = DofMap::new(&model.nodes).unwrap();
    let params = ElementParams::default();
    let contributions: Vec<_> = (1..=elements)
        .map(|j| {
            let ids = [NodeId(j as u32), NodeId(j as u32 + 1)];
            let coords = ids.map(|id| map.coordinates(id).unwrap());
            let ctx = ElementCtx {
                dim: Dim::D2,
                coords: &coords,
                params: &params,
            };
            ElementContribution {
                location: ids
                    .iter()
                    .flat_map(|&id| {
                        Dim::D2
                            .dofs()
                            .iter()
                            .map(|&d| map.global_index(id, d).unwrap())
                            .collect::<Vec<_>>()
                    })
                    .collect(),
                stiffness: element.respond(&ctx, &[0.0; 6], &[]).unwrap().tangent,
                mass_lumped: element.mass(&ctx, MassKind::Lumped).unwrap(),
                mass_consistent: element.mass(&ctx, MassKind::Consistent).unwrap(),
                geometric: None,
                loads: vec![],
            }
        })
        .collect();
    LinearSystem::from_model(ReferenceBackend, &model, &contributions, &[]).unwrap()
}

// Raíces derivadas aquí de cos β cosh β + 1=0; bisección de intervalos propios,
// sin cifras de raíces publicadas ni resultados de otro motor.
fn cantilever_beta() -> [f64; 3] {
    std::array::from_fn(|i| {
        let mut low = i as f64 * PI + 0.1;
        let mut high = (i + 1) as f64 * PI;
        for _ in 0..55 {
            let mid = (low + high) / 2.0;
            let f = |x: f64| math::cos(x) * math::cosh(x) + 1.0;
            if f(low) * f(mid) <= 0.0 {
                high = mid;
            } else {
                low = mid;
            }
        }
        (low + high) / 2.0
    })
}
#[test]
fn frame_beams_converge_to_own_closed_form_solution() {
    for cantilever in [false, true] {
        let beta = if cantilever {
            cantilever_beta()
        } else {
            [PI, 2.0 * PI, 3.0 * PI]
        };
        let mut coarse = beam(4, cantilever, 0.0);
        let mut fine = beam(12, cantilever, 0.0);
        let c = Modal::run(&mut coarse, 3, MassKind::Consistent, &[[0.0, 1.0, 0.0]]).unwrap();
        let f = Modal::run(&mut fine, 3, MassKind::Consistent, &[[0.0, 1.0, 0.0]]).unwrap();
        for (i, root) in beta.iter().enumerate() {
            let exact = 0.01 * math::powi(*root, 2);
            let ce = (c.modes[i].angular_frequency / exact - 1.0).abs();
            let fe = (f.modes[i].angular_frequency / exact - 1.0).abs();
            assert!(
                fe < 3e-4 && fe < ce / 10.0,
                "cantilever={cantilever}, modo={i}, errores={ce},{fe}"
            );
        }
        orthogonal(&fine, &f, 2e-9);
        let count = fine.constraints().free_dofs().len();
        let all = Modal::run(
            &mut fine,
            count,
            MassKind::Consistent,
            &[[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
        )
        .unwrap();
        complete(&fine, &all);
        orthogonal(&fine, &all, 2e-9);
        assert!(all.directions[0].free_mass < all.directions[0].total_mass);
        assert!(all.directions[0].cumulative_mass_fraction.unwrap() < 1.0);
    }
}

#[test]
fn rotated_model_direction_and_modal_shapes_are_invariant() {
    let angle: f64 = 0.7;
    let (c, s) = (math::cos(angle), math::sin(angle));
    for cantilever in [false, true] {
        let mut horizontal = beam(6, cantilever, 0.0);
        let mut rotated = beam(6, cantilever, angle);
        let a = Modal::run(&mut horizontal, 3, MassKind::Consistent, &[[0.0, 1.0, 0.0]]).unwrap();
        let b = Modal::run(&mut rotated, 3, MassKind::Consistent, &[[-s, c, 0.0]]).unwrap();
        for (ma, mb) in a.modes.iter().zip(&b.modes) {
            close(ma.eigenvalue, mb.eigenvalue, 2e-10 * ma.eigenvalue);
            close(
                ma.participation[0].effective_mass,
                mb.participation[0].effective_mass,
                2e-10,
            );
            close(
                ma.participation[0].factor.abs(),
                mb.participation[0].factor.abs(),
                2e-10,
            );
            let expected: Vec<_> = ma
                .global_shape
                .chunks_exact(3)
                .flat_map(|q| [c * q[0] - s * q[1], s * q[0] + c * q[1], q[2]])
                .collect();
            let sign = if dot(
                rotated.global_mass(MassKind::Consistent),
                &expected,
                &mb.global_shape,
            ) < 0.0
            {
                -1.0
            } else {
                1.0
            };
            for (ea, &eb) in expected.iter().zip(&mb.global_shape) {
                close(sign * ea, eb, 3e-9);
            }
        }
        close(
            a.directions[0].total_mass,
            b.directions[0].total_mass,
            1e-12,
        );
        close(a.directions[0].free_mass, b.directions[0].free_mass, 1e-12);
    }
}

#[test]
fn singular_mass_propagates_node_and_global_dof_without_reduction() {
    let mut system = beam(1, true, 0.0);
    let error = Modal::run(&mut system, 1, MassKind::Lumped, &[[0.0, 1.0, 0.0]]).unwrap_err();
    assert_eq!(error.code, "ENG-E-0006");
    assert!(error.message.contains("Nodo 2") && error.message.contains("GDL Rz"));
    assert!(error
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Node(NodeId(2)))));
    assert!(error
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Dof(DofIndex(5)))));
}

#[test]
fn mass_kind_changes_axial_frequency_and_supports_have_zero_shape() {
    let mut model = Model::new();
    for j in 0..2 {
        model
            .nodes
            .push(Node::new(NodeId(j + 1), Dim::D2, [j as f64, 0.0, 0.0]));
    }
    support(&mut model, 1, Dim::D2.dofs());
    support(&mut model, 2, &[Dof::Uy, Dof::Rz]);
    let element = EulerFrame2D::new(1.0, 1.0, 1.0, 1.0, MomentReleases::default()).unwrap();
    let coords = [[0.0; 3], [1.0, 0.0, 0.0]];
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let contribution = ElementContribution {
        location: (0..6).map(DofIndex).collect(),
        stiffness: element.respond(&ctx, &[0.0; 6], &[]).unwrap().tangent,
        mass_lumped: element.mass(&ctx, MassKind::Lumped).unwrap(),
        mass_consistent: element.mass(&ctx, MassKind::Consistent).unwrap(),
        geometric: None,
        loads: vec![],
    };
    let prescribed = [PrescribedDof {
        node: NodeId(1),
        dof: Dof::Ux,
        value: 0.2,
    }];
    let mut system =
        LinearSystem::from_model(ReferenceBackend, &model, &[contribution], &prescribed).unwrap();
    system.accept(&[0.7]).unwrap();
    for (kind, lambda, mass) in [
        (MassKind::Lumped, 2.0, 0.5),
        (MassKind::Consistent, 3.0, 1.0 / 3.0),
    ] {
        let r = Modal::run(&mut system, 1, kind, &[[1.0, 0.0, 0.0]]).unwrap();
        close(r.modes[0].eigenvalue, lambda, 1e-12);
        close(r.directions[0].free_mass, mass, 1e-12);
        close(r.directions[0].total_mass, 1.0, 1e-12);
        assert_eq!(&r.modes[0].global_shape[..3], &[0.0; 3]);
        complete(&system, &r);
        assert_eq!(system.accepted_displacement(), &[0.7]);
        assert_eq!(system.trial_displacement(), &[0.7]);
    }
}

#[test]
fn invalid_requests_zero_count_and_rigid_period() {
    let mut system = shear(3, 17.0, 3.0);
    for (count, dirs, code) in [
        (4, vec![[1.0, 0.0, 0.0]], "ENG-E-0010"),
        (1, vec![], "ENG-E-0010"),
        (1, vec![[0.0; 3]], "ENG-E-0010"),
        (1, vec![[f64::NAN, 0.0, 0.0]], "ENG-E-0008"),
        (1, vec![[f64::INFINITY, 0.0, 0.0]], "ENG-E-0008"),
        (1, vec![[0.0, 0.0, 1.0]], "ENG-E-0011"),
    ] {
        assert_eq!(
            Modal::run(&mut system, count, MassKind::Lumped, &dirs)
                .unwrap_err()
                .code,
            code
        );
    }
    let empty = Modal::run(&mut system, 0, MassKind::Lumped, &[[1.0, 0.0, 0.0]]).unwrap();
    assert!(empty.modes.is_empty());
    assert_eq!(empty.directions[0].cumulative_mass_fraction, Some(0.0));
    let mut rigid = shear(1, 0.0, 2.0);
    let r = Modal::run(&mut rigid, 1, MassKind::Lumped, &[[1.0, 0.0, 0.0]]).unwrap();
    assert_eq!(r.rigid_mode_count, 1);
    assert_eq!(r.modes[0].period, None);
    assert_eq!(r.modes[0].angular_frequency, 0.0);
    assert_eq!(r.modes[0].frequency_hz, 0.0);
    complete(&rigid, &r);
    let mut multiple = shear(2, 0.0, 2.0);
    let error = Modal::run(&mut multiple, 1, MassKind::Lumped, &[[1.0, 0.0, 0.0]]).unwrap_err();
    assert_eq!(error.code, "ENG-E-0007");
    assert!(error.message.contains("multiplicidad"));
}
