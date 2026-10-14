//! source: derivación propia; citation: provenance/ENG-ELM-005.md.
//! Usa la ley real `MultilinearElastic` de engine-laws (MAT-002) como
//! dev-dependency interna; sin copias ni adaptadores.
use engine_contracts::{
    conformance::check_element, Element, ElementCtx, ElementParams, MassKind, MemberLoad,
    UniaxialLaw, UniaxialResponse,
};
use engine_core::math;
use engine_core::{ContractId, Dim, Dof, EResult, EngineError, Real};
use engine_elements_spring::{NonlinearSpring, SpringAxes, SpringConnection};
use std::sync::Arc;

use engine_laws::uniaxial::{MultilinearElastic, MultilinearMode};

fn finite(value: Real, name: &str) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new("ENG-E-0008", name))
    }
}
fn vector(values: &[Real], n: usize, name: &str) -> EResult<()> {
    if values.len() != n {
        return Err(EngineError::new("ENG-E-0010", name));
    }
    for &v in values {
        finite(v, name)?;
    }
    Ok(())
}
fn close(a: Real, b: Real) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}
fn code<T>(r: EResult<T>, expected: &str) {
    assert_eq!(r.err().expect("Debe producir diagnóstico").code, expected);
}
fn bilinear() -> Arc<dyn UniaxialLaw> {
    Arc::new(
        MultilinearElastic::new(
            vec![
                (-0.5, -18.0),
                (-0.1, -10.0),
                (0.0, 0.0),
                (0.1, 10.0),
                (0.5, 18.0),
            ],
            MultilinearMode::Both,
        )
        .unwrap(),
    )
}
fn compression_gap() -> Arc<dyn UniaxialLaw> {
    Arc::new(
        MultilinearElastic::new(
            vec![(-1.25, -100.0), (-0.25, 0.0), (0.0, 0.0)],
            MultilinearMode::CompressionOnly,
        )
        .unwrap(),
    )
}
fn spring(connection: SpringConnection, law: Arc<dyn UniaxialLaw>) -> NonlinearSpring {
    NonlinearSpring::new(Dim::D2, connection, law, &[Dof::Ux], SpringAxes::Global).unwrap()
}
fn context<'a>(dim: Dim, coords: &'a [[Real; 3]], params: &'a ElementParams) -> ElementCtx<'a> {
    ElementCtx {
        dim,
        coords,
        params,
    }
}
fn zero_load() -> MemberLoad {
    MemberLoad {
        force_per_length: [0.0; 3],
        moment_per_length: [0.0; 3],
    }
}

#[test]
fn bilinear_force_and_tangent_closed_form() {
    let p = ElementParams::default();
    for connection in [SpringConnection::BetweenNodes, SpringConnection::Ground] {
        let s = spring(connection, bilinear());
        let coords = vec![[0.0; 3]; s.n_nodes()];
        let ctx = context(Dim::D2, &coords, &p);
        assert_eq!(s.kind(), "spring.nonlinear");
        assert_eq!(s.contract(), ContractId::new("element", 1, 0));
        assert_eq!(s.law().kind(), "multilinear_elastic");
        assert_eq!(s.state_len(), 0);
        for (delta, f, k) in [
            (-0.7, -22.0, 20.0),
            (-0.3, -14.0, 20.0),
            (-0.05, -5.0, 100.0),
            (0.0, 0.0, 100.0),
            (0.05, 5.0, 100.0),
            (0.3, 14.0, 20.0),
            (0.7, 22.0, 20.0),
        ] {
            // Se incluyen desplazamientos uniformes y coordenadas coincidentes.
            let u = if connection == SpringConnection::Ground {
                vec![delta]
            } else {
                vec![0.25, 0.25 + delta]
            };
            let r = s.respond(&ctx, &u, &[]).unwrap();
            assert!(r.trial_state.is_empty());
            let b = if s.n_nodes() == 1 {
                vec![1.0]
            } else {
                vec![-1.0, 1.0]
            };
            for i in 0..b.len() {
                close(r.internal_force[i], f * b[i]);
                for j in 0..b.len() {
                    close(r.tangent.get(i, j).unwrap(), k * b[i] * b[j]);
                }
            }
            let out = s
                .recover(&ctx, &u, &[], &[zero_load()], &[1.0, 0.0, 0.5])
                .unwrap();
            assert_eq!(out.end_forces_local, r.internal_force);
            for station in out.stations {
                close(station.forces_local[0], f);
            }
        }
    }
}

#[test]
fn compression_only_support_never_pulls_and_has_open_gap() {
    let p = ElementParams::default();
    let ctx = context(Dim::D2, &[[0.0; 3]], &p);
    let s = spring(SpringConnection::Ground, compression_gap());
    for (delta, f, k) in [
        (-1.5, -125.0, 100.0),
        (-0.5, -25.0, 100.0),
        (-0.25, 0.0, 0.0),
        (-0.125, 0.0, 0.0),
        (0.0, 0.0, 0.0),
        (0.2, 0.0, 0.0),
        (100.0, 0.0, 0.0),
    ] {
        let r = s.respond(&ctx, &[delta], &[]).unwrap();
        close(r.internal_force[0], f);
        close(r.tangent.get(0, 0).unwrap(), k);
        assert!(r.internal_force[0] <= 0.0);
    }
}

#[test]
fn manual_newton_ground_equilibrium_crosses_bilinear_corner() {
    let p = ElementParams::default();
    let ctx = context(Dim::D2, &[[0.0; 3]], &p);
    for (law, load, start, exact) in [
        (bilinear(), 16.0, 0.0, 0.4),
        (compression_gap(), -20.0, -1.0, -0.45),
    ] {
        let s = spring(SpringConnection::Ground, law);
        let mut accepted = s.initial_state();
        let original = accepted.clone();
        let mut u = start;
        let mut converged = false;
        for _ in 0..8 {
            let r = s.respond(&ctx, &[u], &accepted).unwrap();
            assert_eq!(accepted, original); // Ninguna iteración acepta la prueba.
            let residual = load - r.internal_force[0];
            if residual.abs() < 1e-10 {
                accepted = r.trial_state; // Commit explícito solo al converger.
                converged = true;
                break;
            }
            let tangent = r.tangent.get(0, 0).unwrap();
            assert!(tangent > 0.0);
            u += residual / tangent;
        }
        assert!(converged);
        close(u, exact);
        close(
            s.respond(&ctx, &[u], &accepted).unwrap().internal_force[0],
            load,
        );
    }
}

#[test]
fn elastic_loading_unloading_is_reversible() {
    let p = ElementParams::default();
    let ctx = context(Dim::D2, &[[0.0; 3]], &p);
    for law in [bilinear(), compression_gap()] {
        let s = spring(SpringConnection::Ground, law);
        let path = [0.0, -0.5, -0.3, 0.05, 0.3, 0.7, 0.3, 0.05, -0.3, -0.5, 0.0];
        let mut accepted = s.initial_state();
        for delta in path {
            let r = s.respond(&ctx, &[delta], &accepted).unwrap();
            assert_eq!(r, s.respond(&ctx, &[delta], &s.initial_state()).unwrap());
            accepted = r.trial_state;
        }
        close(
            s.respond(&ctx, &[0.0], &accepted).unwrap().internal_force[0],
            0.0,
        );
    }
}

#[test]
fn axis_rotation_covariance_force_tangent_and_local_recovery() {
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        for connection in [SpringConnection::Ground, SpringConnection::BetweenNodes] {
            let q = if dim == Dim::D2 {
                [[0.6, -0.8, 0.0], [0.8, 0.6, 0.0], [0.0, 0.0, 1.0]]
            } else {
                [[0.6, 0.0, -0.8], [0.0, 1.0, 0.0], [0.8, 0.0, 0.6]]
            };
            let rotate = |v: [Real; 3]| -> [Real; 3] {
                std::array::from_fn(|i| (0..3).map(|j| q[i][j] * v[j]).sum())
            };
            let d = if dim == Dim::D2 {
                [3.0, 4.0, 0.0]
            } else {
                [2.0, -3.0, 6.0]
            };
            let s = NonlinearSpring::new(
                dim,
                connection,
                compression_gap(),
                &[Dof::Ux],
                SpringAxes::Direction(d),
            )
            .unwrap();
            let rotated = NonlinearSpring::new(
                dim,
                connection,
                compression_gap(),
                &[Dof::Ux],
                SpringAxes::Direction(rotate(d)),
            )
            .unwrap();
            let width = s.dofs_per_node().len();
            let coords = vec![[0.0; 3]; s.n_nodes()];
            let ctx = context(dim, &coords, &p);
            for delta in [-0.5, -0.1, 0.3] {
                let mut u = vec![0.0; width * s.n_nodes()];
                let norm = math::sqrt(d.iter().map(|v| v * v).sum::<Real>());
                for a in 0..width {
                    u[width * (s.n_nodes() - 1) + a] = delta * d[a] / norm;
                }
                let ru: Vec<_> = u
                    .chunks_exact(width)
                    .flat_map(|v| {
                        let mut xyz = [0.0; 3];
                        xyz[..width].copy_from_slice(v);
                        rotate(xyz)[..width].to_vec()
                    })
                    .collect();
                let r = s.respond(&ctx, &u, &[]).unwrap();
                let rr = rotated.respond(&ctx, &ru, &[]).unwrap();
                for node in 0..s.n_nodes() {
                    for (i, row) in q.iter().enumerate().take(width) {
                        let expected: Real = (0..width)
                            .map(|a| row[a] * r.internal_force[node * width + a])
                            .sum();
                        close(rr.internal_force[node * width + i], expected);
                    }
                }
                for i in 0..u.len() {
                    for j in 0..u.len() {
                        let mut expected = 0.0;
                        for a in 0..width {
                            for b in 0..width {
                                expected += q[i % width][a]
                                    * r.tangent
                                        .get(i / width * width + a, j / width * width + b)
                                        .unwrap()
                                    * q[j % width][b];
                            }
                        }
                        close(rr.tangent.get(i, j).unwrap(), expected);
                    }
                }
                let out = s.recover(&ctx, &u, &[], &[], &[0.5]).unwrap();
                let rout = rotated.recover(&ctx, &ru, &[], &[], &[0.5]).unwrap();
                for (a, b) in out.end_forces_local.iter().zip(rout.end_forces_local) {
                    close(*a, b);
                }
            }
        }
    }
}

// Ley plástica analítica propia de pruebas: retorno a |F|≤10 N, k=100 N/m,
// historia = desplazamiento plástico. Hace observable el commit y el revert.
struct Plastic;
impl UniaxialLaw for Plastic {
    fn kind(&self) -> &'static str {
        "test.plastic"
    }
    fn state_len(&self) -> usize {
        1
    }
    fn initial_state(&self) -> Vec<Real> {
        vec![0.0]
    }
    fn initial_tangent(&self) -> Real {
        100.0
    }
    fn respond(&self, delta: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 1, "Historia plástica")?;
        finite(delta, "δ")?;
        let elastic = 100.0 * (delta - committed[0]);
        let (force, tangent, plastic) = if elastic.abs() > 10.0 {
            let force = elastic.signum() * 10.0;
            (force, 0.0, delta - force / 100.0)
        } else {
            (elastic, 100.0, committed[0])
        };
        Ok(UniaxialResponse {
            stress: force,
            tangent,
            trial_state: vec![plastic],
        })
    }
}

#[test]
fn nonempty_history_trial_commit_and_revert_are_explicit() {
    let p = ElementParams::default();
    let ctx = context(Dim::D2, &[[0.0; 3]], &p);
    let s = spring(SpringConnection::Ground, Arc::new(Plastic));
    let mut accepted = s.initial_state();
    let initial = accepted.clone();
    let trial = s.respond(&ctx, &[0.2], &accepted).unwrap();
    close(trial.internal_force[0], 10.0);
    close(trial.trial_state[0], 0.1);
    assert_eq!(accepted, initial);
    assert_eq!(trial, s.respond(&ctx, &[0.2], &accepted).unwrap());
    // Revert: se descarta la prueba; δ=0 sigue sin fuerza ni plasticidad.
    close(
        s.respond(&ctx, &[0.0], &accepted).unwrap().internal_force[0],
        0.0,
    );
    accepted = trial.trial_state; // Commit.
    close(
        s.respond(&ctx, &[0.2], &accepted).unwrap().internal_force[0],
        10.0,
    );
    close(
        s.respond(&ctx, &[0.15], &accepted).unwrap().internal_force[0],
        5.0,
    );
    close(
        s.respond(&ctx, &[0.15], &initial).unwrap().internal_force[0],
        10.0,
    );
    let saved = accepted.clone();
    s.recover(&ctx, &[0.3], &accepted, &[], &[0.5]).unwrap();
    assert!(s
        .geometric_stiffness(&ctx, &[0.3], &accepted)
        .unwrap()
        .is_none());
    code(s.respond(&ctx, &[Real::NAN], &accepted), "ENG-E-0008");
    assert_eq!(accepted, saved);
    let mut external_copy = s.initial_state();
    external_copy[0] = 1.0;
    assert_eq!(s.initial_state(), initial);
}

#[test]
fn conformance_kit_accepts_nonempty_history_and_multilinear_elastic() {
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        let coords = [[0.0; 3]; 2];
        let ctx = context(dim, &coords, &p);
        for law in [
            Arc::new(Plastic) as Arc<dyn UniaxialLaw>,
            bilinear(),
            compression_gap(),
        ] {
            let s = NonlinearSpring::new(
                dim,
                SpringConnection::BetweenNodes,
                law,
                &[Dof::Ux],
                SpringAxes::Global,
            )
            .unwrap();
            let samples = [
                vec![-0.01, 0.01],
                vec![0.0, 0.15],
                vec![0.0, 0.12],
                vec![0.0, -0.03],
                vec![0.0, -0.4],
            ];
            let issues = check_element(&s, &ctx, &samples, 1e-7);
            assert!(issues.is_empty(), "{issues:?}");
        }
    }
}

#[test]
fn resolved_law_owned_by_element_and_auxiliary_operations() {
    let p = ElementParams {
        law: Some(bilinear()),
        ..ElementParams::default()
    };
    let s = NonlinearSpring::from_params(
        Dim::D2,
        SpringConnection::Ground,
        &p,
        &[Dof::Ux],
        SpringAxes::Global,
    )
    .unwrap();
    drop(p);
    let p = ElementParams {
        law: Some(compression_gap()),
        ..ElementParams::default()
    };
    let ctx = context(Dim::D2, &[[0.0; 3]], &p);
    close(
        s.respond(&ctx, &[0.05], &[]).unwrap().internal_force[0],
        5.0,
    );
    assert_eq!(s.dim(), Dim::D2);
    assert_eq!(s.connection(), SpringConnection::Ground);
    for kind in [MassKind::Lumped, MassKind::Consistent] {
        assert!(s.mass(&ctx, kind).unwrap().is_none());
    }
    assert_eq!(s.load_vector(&ctx, &zero_load()).unwrap(), vec![0.0]);
    assert!(s.geometric_stiffness(&ctx, &[0.05], &[]).unwrap().is_none());
    let rotational = NonlinearSpring::new(
        Dim::D2,
        SpringConnection::Ground,
        bilinear(),
        &[Dof::Rz],
        SpringAxes::Global,
    )
    .unwrap();
    close(
        rotational
            .respond(&ctx, &[0.3], &[])
            .unwrap()
            .internal_force[0],
        14.0,
    );
    let coupled = NonlinearSpring::new(
        Dim::D2,
        SpringConnection::Ground,
        bilinear(),
        &[Dof::Ux, Dof::Uy],
        SpringAxes::Global,
    )
    .unwrap();
    let r = coupled.respond(&ctx, &[0.1, 0.2], &[]).unwrap();
    for f in r.internal_force {
        close(f, 14.0);
    }
    for k in r.tangent.values() {
        close(*k, 20.0);
    }
}

#[test]
fn invalid_inputs_and_wrong_history_lengths_return_diagnostics() {
    let p = ElementParams::default();
    code(
        NonlinearSpring::from_params(
            Dim::D2,
            SpringConnection::Ground,
            &p,
            &[Dof::Ux],
            SpringAxes::Global,
        ),
        "ENG-E-0010",
    );
    for (dofs, axes) in [
        (vec![], SpringAxes::Global),
        (vec![Dof::Ux, Dof::Ux], SpringAxes::Global),
        (vec![Dof::Ux, Dof::Rz], SpringAxes::Global),
        (vec![Dof::Uz], SpringAxes::Global),
        (vec![Dof::Ux], SpringAxes::Direction([0.0; 3])),
        (vec![Dof::Ux], SpringAxes::Direction([1.0, 0.0, 1.0])),
        (vec![Dof::Ux], SpringAxes::Rotation([[0.0; 3]; 3])),
    ] {
        code(
            NonlinearSpring::new(Dim::D2, SpringConnection::Ground, bilinear(), &dofs, axes),
            "ENG-E-0010",
        );
    }
    code(
        NonlinearSpring::new(
            Dim::D2,
            SpringConnection::Ground,
            bilinear(),
            &[Dof::Ux],
            SpringAxes::Direction([Real::NAN, 0.0, 0.0]),
        ),
        "ENG-E-0008",
    );
    let ctx = context(Dim::D2, &[[0.0; 3]], &p);
    for law in [bilinear(), Arc::new(Plastic) as Arc<dyn UniaxialLaw>] {
        let s = spring(SpringConnection::Ground, law);
        let accepted = s.initial_state();
        for bad in [vec![], vec![0.0], vec![0.0, 0.0]] {
            if bad.len() == s.state_len() {
                continue;
            }
            code(s.respond(&ctx, &[0.1], &bad), "ENG-E-0010");
            code(s.recover(&ctx, &[0.1], &bad, &[], &[]), "ENG-E-0010");
            code(s.geometric_stiffness(&ctx, &[0.1], &bad), "ENG-E-0010");
        }
        for u in [vec![], vec![0.0, 0.0]] {
            code(s.respond(&ctx, &u, &accepted), "ENG-E-0010");
        }
        for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
            code(s.respond(&ctx, &[value], &accepted), "ENG-E-0008");
            if s.state_len() > 0 {
                code(s.respond(&ctx, &[0.0], &[value]), "ENG-E-0008");
            }
        }
        for bad_ctx in [
            context(Dim::D3, &[[0.0; 3]], &p),
            context(Dim::D2, &[], &p),
            context(Dim::D2, &[[0.0, 0.0, 1.0]], &p),
        ] {
            code(s.respond(&bad_ctx, &[0.1], &accepted), "ENG-E-0010");
            code(s.mass(&bad_ctx, MassKind::Consistent), "ENG-E-0010");
        }
        code(
            s.respond(
                &context(Dim::D2, &[[Real::NAN, 0.0, 0.0]], &p),
                &[0.1],
                &accepted,
            ),
            "ENG-E-0008",
        );
        for position in [-0.1, 1.1] {
            code(
                s.recover(&ctx, &[0.1], &accepted, &[], &[position]),
                "ENG-E-0010",
            );
        }
        code(
            s.recover(&ctx, &[0.1], &accepted, &[], &[Real::NAN]),
            "ENG-E-0008",
        );
        let mut load = zero_load();
        load.force_per_length[0] = 1.0;
        code(s.load_vector(&ctx, &load), "ENG-E-0010");
        code(
            s.recover(&ctx, &[0.1], &accepted, &[load], &[]),
            "ENG-E-0010",
        );
        let mut load = zero_load();
        load.moment_per_length[2] = Real::NAN;
        code(s.load_vector(&ctx, &load), "ENG-E-0008");
    }
    let s = spring(SpringConnection::BetweenNodes, bilinear());
    code(
        s.respond(
            &context(Dim::D2, &[[0.0; 3]; 2], &p),
            &[-Real::MAX, Real::MAX],
            &[],
        ),
        "ENG-E-0008",
    );
    let s = spring(SpringConnection::Ground, bilinear());
    code(s.respond(&ctx, &[Real::MAX], &[]), "ENG-E-0008");
}

// Ley deliberadamente inválida para comprobar la frontera del elemento.
struct BadLaw(u8);
impl UniaxialLaw for BadLaw {
    fn kind(&self) -> &'static str {
        "test.bad"
    }
    fn contract(&self) -> ContractId {
        match self.0 {
            0 => ContractId::new("section", 1, 0),
            1 => ContractId::new("uniaxial", 2, 0),
            _ => ContractId::new("uniaxial", 1, 0),
        }
    }
    fn state_len(&self) -> usize {
        1
    }
    fn initial_state(&self) -> Vec<Real> {
        match self.0 {
            2 => vec![],
            3 => vec![Real::NAN],
            _ => vec![0.0],
        }
    }
    fn initial_tangent(&self) -> Real {
        if self.0 == 4 {
            Real::INFINITY
        } else {
            1.0
        }
    }
    fn respond(&self, _: Real, _: &[Real]) -> EResult<UniaxialResponse> {
        if self.0 == 9 {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Fallo constitutivo de prueba",
            ));
        }
        Ok(UniaxialResponse {
            stress: if self.0 == 5 { Real::NAN } else { 1.0 },
            tangent: if self.0 == 6 { Real::INFINITY } else { 1.0 },
            trial_state: match self.0 {
                7 => vec![],
                8 => vec![Real::NAN],
                _ => vec![0.0],
            },
        })
    }
}

#[test]
fn malformed_law_initial_state_and_trial_outputs_are_rejected() {
    for (mode, expected) in [
        (0, "ENG-E-0010"),
        (1, "ENG-E-0010"),
        (2, "ENG-E-0010"),
        (3, "ENG-E-0008"),
        (4, "ENG-E-0008"),
    ] {
        code(
            NonlinearSpring::new(
                Dim::D2,
                SpringConnection::Ground,
                Arc::new(BadLaw(mode)),
                &[Dof::Ux],
                SpringAxes::Global,
            ),
            expected,
        );
    }
    let p = ElementParams::default();
    let ctx = context(Dim::D2, &[[0.0; 3]], &p);
    for (mode, expected) in [
        (5, "ENG-E-0008"),
        (6, "ENG-E-0008"),
        (7, "ENG-E-0010"),
        (8, "ENG-E-0008"),
        (9, "ENG-E-0012"),
    ] {
        let s = spring(SpringConnection::Ground, Arc::new(BadLaw(mode)));
        code(s.respond(&ctx, &[0.1], &[0.0]), expected);
        code(s.recover(&ctx, &[0.1], &[0.0], &[], &[]), expected);
        code(s.geometric_stiffness(&ctx, &[0.1], &[0.0]), expected);
    }
}
