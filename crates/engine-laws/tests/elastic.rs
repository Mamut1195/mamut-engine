//! In-house analytical benchmarks; memory only, suitable for wasm32 compilation.

use engine_contracts::{
    conformance::{check_section_law, check_uniaxial_law},
    SectionLaw, UniaxialLaw,
};
use engine_core::{ContractId, Dim, EResult, Real};
use engine_laws::{
    section::{
        ElasticFrame, ElasticFrameFactory, ElasticFrameParams, SectionFactory, ELASTIC_FRAME,
    },
    uniaxial::{Elastic, ElasticFactory, ElasticParams, UniaxialFactory, ELASTIC},
};

struct UniaxialFixture {
    source: &'static str,
    citation: &'static str,
    e: Real,
    strain: Real,
    stress: Real,
}

const UNIAXIAL_FIXTURES: &[UniaxialFixture] = &[
    UniaxialFixture {
        source: "Derivación analítica propia: sigma = E * eps; datos propios en SI.",
        citation: "provenance/ENG-MAT-001.md, Derivaciones propias y fixtures",
        e: 200.0,
        strain: 0.02,
        stress: 4.0,
    },
    UniaxialFixture {
        source: "Derivación analítica propia: compresión lineal; datos propios en SI.",
        citation: "provenance/ENG-MAT-001.md, Derivaciones propias y fixtures",
        e: 200e9,
        strain: -0.001,
        stress: -200e6,
    },
    UniaxialFixture {
        source: "Derivación analítica propia: origen sin precarga; datos propios en SI.",
        citation: "provenance/ENG-MAT-001.md, Derivaciones propias y fixtures",
        e: 200e9,
        strain: 0.0,
        stress: 0.0,
    },
];

struct SectionFixture {
    source: &'static str,
    citation: &'static str,
    dim: Dim,
    params: ElasticFrameParams,
    strain: &'static [Real],
    resultants: &'static [Real],
    diagonal: &'static [Real],
}

const SMALL_PARAMS: ElasticFrameParams = ElasticFrameParams {
    e: 200.0,
    g: 80.0,
    area: 3.0,
    iy: 2.0,
    iz: 5.0,
    j: 4.0,
    ay: Some(1.5),
    az: Some(1.2),
};

const SECTION_FIXTURES: &[SectionFixture] = &[
    SectionFixture {
        source: "Derivación analítica propia D2: N = EA * eps0; Mz = EIz * kz.",
        citation: "provenance/ENG-MAT-001.md, Derivaciones propias y fixtures",
        dim: Dim::D2,
        params: SMALL_PARAMS,
        strain: &[0.02, -0.03],
        resultants: &[12.0, -30.0],
        diagonal: &[600.0, 1000.0],
    },
    SectionFixture {
        source: "Derivación analítica propia D3: N, My, Mz, T; torsión de Saint-Venant.",
        citation: "provenance/ENG-MAT-001.md, Derivaciones propias y fixtures",
        dim: Dim::D3,
        params: SMALL_PARAMS,
        strain: &[0.02, -0.03, 0.04, -0.05],
        resultants: &[12.0, -12.0, 40.0, -16.0],
        diagonal: &[600.0, 400.0, 1000.0, 320.0],
    },
    SectionFixture {
        source: "Derivación analítica propia D3 con propiedades de escala estructural, SI.",
        citation: "provenance/ENG-MAT-001.md, Derivaciones propias y fixtures",
        dim: Dim::D3,
        params: ElasticFrameParams {
            e: 200e9,
            g: 80e9,
            area: 0.03,
            iy: 2e-5,
            iz: 5e-5,
            j: 4e-5,
            ay: None,
            az: None,
        },
        strain: &[0.002, -0.003, 0.004, -0.005],
        resultants: &[12e6, -12e3, 40e3, -16e3],
        diagonal: &[6e9, 4e6, 1e7, 3.2e6],
    },
];

fn close(actual: Real, expected: Real) {
    assert!(
        (actual - expected).abs() <= 1e-12 + 1e-12 * expected.abs(),
        "actual={actual}, expected={expected}"
    );
}

fn code<T>(result: EResult<T>, expected: &str) {
    match result {
        Err(error) => assert_eq!(error.code, expected, "{error}"),
        Ok(_) => panic!("Expected {expected}"),
    }
}

#[test]
fn uniaxial_analytical_fixtures() {
    for f in UNIAXIAL_FIXTURES {
        assert!(!f.source.is_empty() && !f.citation.is_empty());
        let law = Elastic::new(f.e).unwrap();
        let r = law.respond(f.strain, &[]).unwrap();
        close(r.stress, f.stress);
        assert_eq!(r.tangent, f.e);
        assert_eq!(law.young_modulus(), f.e);
        assert_eq!(law.initial_tangent(), f.e);
        assert_eq!(law.state_len(), 0);
        assert!(law.initial_state().is_empty() && r.trial_state.is_empty());
    }
}

#[test]
fn section_analytical_fixtures_and_tangent() {
    for f in SECTION_FIXTURES {
        assert!(!f.source.is_empty() && !f.citation.is_empty());
        let law = ElasticFrame::new(f.dim, f.params).unwrap();
        assert_eq!(law.params(), &f.params);
        assert_eq!(law.dim(), f.dim);
        assert_eq!(law.state_len(), 0);
        assert!(law.initial_state().is_empty());
        let r = law.respond(f.strain, &[]).unwrap();
        assert!(r.trial_state.is_empty());
        assert_eq!(r.resultants.len(), f.resultants.len());
        for (&actual, &expected) in r.resultants.iter().zip(f.resultants) {
            close(actual, expected);
        }
        assert_eq!(r.tangent.rows(), f.diagonal.len());
        assert_eq!(r.tangent.cols(), f.diagonal.len());
        for (i, &d) in f.diagonal.iter().enumerate() {
            for j in 0..f.diagonal.len() {
                close(r.tangent.get(i, j).unwrap(), if i == j { d } else { 0.0 });
            }
        }
    }
}

#[test]
fn contract_conformance() {
    let law = Elastic::new(200e9).unwrap();
    let diagnostics = check_uniaxial_law(&law, &[0.0, 0.001, -0.002, 0.001, 0.0], 1e-7);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for f in SECTION_FIXTURES {
        let law = ElasticFrame::new(f.dim, f.params).unwrap();
        let samples = vec![
            vec![0.0; f.strain.len()],
            f.strain.to_vec(),
            f.strain.iter().map(|v| -v).collect(),
            f.strain.to_vec(),
            vec![0.0; f.strain.len()],
        ];
        let diagnostics = check_section_law(&law, &samples, 1e-7);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }
}

#[test]
fn every_tangent_component_matches_central_difference() {
    // Componentwise comparison: large EA cannot hide errors in EI/GJ.
    for f in SECTION_FIXTURES {
        let law = ElasticFrame::new(f.dim, f.params).unwrap();
        let base = law.respond(f.strain, &[]).unwrap();
        let h = 1e-6;
        for j in 0..f.strain.len() {
            let mut plus = f.strain.to_vec();
            let mut minus = plus.clone();
            plus[j] += h;
            minus[j] -= h;
            let rp = law.respond(&plus, &[]).unwrap();
            let rm = law.respond(&minus, &[]).unwrap();
            for i in 0..f.strain.len() {
                let fd = (rp.resultants[i] - rm.resultants[i]) / (2.0 * h);
                let exact = base.tangent.get(i, j).unwrap();
                if exact == 0.0 {
                    assert_eq!(fd, 0.0);
                } else {
                    assert!((fd / exact - 1.0).abs() < 1e-6);
                }
            }
        }
    }
}

#[test]
fn shear_is_optional_and_does_not_change_section_response() {
    for dim in [Dim::D2, Dim::D3] {
        let with = ElasticFrame::new(dim, SMALL_PARAMS).unwrap();
        close(with.shear_rigidity_y().unwrap(), 120.0);
        close(with.shear_rigidity_z().unwrap(), 96.0);
        assert_eq!(with.axial_rigidity(), 600.0);
        assert_eq!(with.bending_rigidity_y(), 400.0);
        assert_eq!(with.bending_rigidity_z(), 1000.0);
        assert_eq!(with.torsional_rigidity(), 320.0);
        for (ay, az) in [(None, None), (Some(2.0), None), (None, Some(3.0))] {
            let other = ElasticFrame::new(
                dim,
                ElasticFrameParams {
                    ay,
                    az,
                    ..SMALL_PARAMS
                },
            )
            .unwrap();
            assert_eq!(other.shear_rigidity_y(), ay.map(|a| 80.0 * a));
            assert_eq!(other.shear_rigidity_z(), az.map(|a| 80.0 * a));
            let e = vec![0.02; if dim == Dim::D2 { 2 } else { 4 }];
            assert_eq!(
                with.respond(&e, &[]).unwrap(),
                other.respond(&e, &[]).unwrap()
            );
        }
    }
}

#[test]
fn uniaxial_rejects_invalid_modulus_and_inputs() {
    for value in [0.0, -0.0, -1.0] {
        code(Elastic::new(value), "ENG-E-0010");
    }
    for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        code(Elastic::new(value), "ENG-E-0008");
        code(
            Elastic::new(200.0).unwrap().respond(value, &[]),
            "ENG-E-0008",
        );
    }
    let law = Elastic::new(200.0).unwrap();
    for state in [vec![0.0], vec![0.0, 1.0], vec![Real::NAN]] {
        code(law.respond(0.0, &state), "ENG-E-0010");
    }
    code(law.respond(Real::MAX, &[]), "ENG-E-0008");
}

// Changes each property separately, including those inactive in D2.
fn changed(field: usize, value: Real) -> ElasticFrameParams {
    let mut p = SMALL_PARAMS;
    match field {
        0 => p.e = value,
        1 => p.g = value,
        2 => p.area = value,
        3 => p.iy = value,
        4 => p.iz = value,
        5 => p.j = value,
        6 => p.ay = Some(value),
        7 => p.az = Some(value),
        _ => panic!("Invalid test field"),
    }
    p
}

#[test]
fn every_section_parameter_must_be_positive_and_finite() {
    for dim in [Dim::D2, Dim::D3] {
        for field in 0..8 {
            for value in [0.0, -0.0, -1.0] {
                code(ElasticFrame::new(dim, changed(field, value)), "ENG-E-0010");
            }
            for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
                code(ElasticFrame::new(dim, changed(field, value)), "ENG-E-0008");
            }
        }
    }
}

#[test]
fn section_rejects_overflow_and_zero_underflow_in_rigidities() {
    for dim in [Dim::D2, Dim::D3] {
        for field in 0..8 {
            code(
                ElasticFrame::new(dim, changed(field, Real::MAX)),
                "ENG-E-0008",
            );
        }
        let tiny = Real::from_bits(1);
        let base = ElasticFrameParams {
            e: 0.5,
            g: 0.5,
            area: 1.0,
            iy: 1.0,
            iz: 1.0,
            j: 1.0,
            ay: Some(1.0),
            az: Some(1.0),
        };
        for field in 2..8 {
            let p = changed(field, tiny);
            let p = ElasticFrameParams {
                e: base.e,
                g: base.g,
                ..p
            };
            code(ElasticFrame::new(dim, p), "ENG-E-0010");
        }
    }
}

#[test]
fn section_rejects_bad_strain_and_state_lengths() {
    for dim in [Dim::D2, Dim::D3] {
        let law = ElasticFrame::new(dim, SMALL_PARAMS).unwrap();
        let n = if dim == Dim::D2 { 2 } else { 4 };
        for len in 0..=5 {
            if len != n {
                code(law.respond(&vec![0.0; len], &[]), "ENG-E-0010");
            }
        }
        for state in [vec![0.0], vec![1.0, 2.0], vec![Real::NAN]] {
            code(law.respond(&vec![0.0; n], &state), "ENG-E-0010");
        }
        for index in 0..n {
            for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY, Real::MAX] {
                let mut e = vec![0.0; n];
                e[index] = value;
                code(law.respond(&e, &[]), "ENG-E-0008");
            }
        }
    }
}

#[test]
fn pure_repetition_and_return_to_origin() {
    let law = Elastic::new(200.0).unwrap();
    let original = law;
    let accepted = law.initial_state();
    let first = law.respond(-0.02, &accepted).unwrap();
    law.respond(0.03, &accepted).unwrap();
    assert_eq!(first, law.respond(-0.02, &first.trial_state).unwrap());
    assert_eq!(law, original);
    assert!(accepted.is_empty());
    assert_eq!(law.respond(0.0, &accepted).unwrap().stress, 0.0);
    for f in SECTION_FIXTURES {
        let law = ElasticFrame::new(f.dim, f.params).unwrap();
        let original = law.clone();
        let accepted = law.initial_state();
        let e = f.strain.to_vec();
        let first = law.respond(&e, &accepted).unwrap();
        let zero = vec![0.0; e.len()];
        assert_eq!(law.respond(&zero, &accepted).unwrap().resultants, zero);
        assert_eq!(first, law.respond(&e, &first.trial_state).unwrap());
        assert_eq!(e, f.strain);
        assert!(accepted.is_empty());
        assert_eq!(law, original);
    }
}

#[test]
fn factories_describe_and_build_contract_objects() {
    let uniaxial: &dyn UniaxialFactory<Params = ElasticParams> = &ElasticFactory;
    assert_eq!(uniaxial.descriptor(), ELASTIC);
    assert_eq!(ELASTIC.kind, "elastic");
    assert_eq!(ELASTIC.contract, ContractId::new("uniaxial", 1, 0));
    assert_eq!(ELASTIC.version, (0, 1, 0));
    let law = uniaxial.build(ElasticParams { e: 200.0 }).unwrap();
    assert_eq!(law.kind(), ELASTIC.kind);
    assert_eq!(law.contract(), ELASTIC.contract);
    assert_eq!(law.respond(0.02, &[]).unwrap().stress, 4.0);
    code(uniaxial.build(ElasticParams { e: -1.0 }), "ENG-E-0010");

    let section: &dyn SectionFactory<Params = ElasticFrameParams> = &ElasticFrameFactory;
    assert_eq!(section.descriptor(), ELASTIC_FRAME);
    assert_eq!(ELASTIC_FRAME.kind, "elastic_frame");
    assert_eq!(ELASTIC_FRAME.contract, ContractId::new("section", 1, 0));
    assert_eq!(ELASTIC_FRAME.version, (0, 1, 0));
    for dim in [Dim::D2, Dim::D3] {
        let law = section.build(dim, SMALL_PARAMS).unwrap();
        assert_eq!(law.kind(), ELASTIC_FRAME.kind);
        assert_eq!(law.contract(), ELASTIC_FRAME.contract);
        assert_eq!(law.dim(), dim);
        code(section.build(dim, changed(0, -1.0)), "ENG-E-0010");
    }
}

#[test]
fn laws_and_factories_are_send_sync_without_spawning_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Elastic>();
    assert_send_sync::<ElasticFrame>();
    assert_send_sync::<ElasticFactory>();
    assert_send_sync::<ElasticFrameFactory>();
    assert_send_sync::<Box<dyn UniaxialLaw>>();
    assert_send_sync::<Box<dyn SectionLaw>>();
}
