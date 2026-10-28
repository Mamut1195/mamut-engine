//! SOURCE: own derivation and formulas from request ENG-ANL-004.
//! CITATION: provenance/ENG-ANL-004.md. No published values or oracles.
use engine_analysis::spectrum::{
    combine_directions, combine_modal, cqc_correlation, DirectionalCombination, Extrapolation,
    Interpolation, ModalCombination, Spectrum, SpectrumDirection, SpectrumOptions, SpectrumUnits,
    TabulatedSpectrum,
};
use engine_assembly::{DofMap, ElementContribution, LinearSystem, PrescribedDof};
use engine_contracts::{Element, ElementCtx, ElementParams, MassKind, SystemView};
use engine_core::math;
use engine_core::{Dim, Dof, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_linalg::{DMat, ReferenceBackend};
use engine_model::{DofValue, Model, NodalMass, Node, Support};
use std::f64::consts::{PI, TAU};

const PERIOD_SI: SpectrumUnits = SpectrumUnits::PeriodSecondsAccelerationMetersPerSecondSquared;
const FREQUENCY_SI: SpectrumUnits = SpectrumUnits::FrequencyHertzAccelerationMetersPerSecondSquared;
fn close(a: f64, b: f64, tol: f64) {
    assert!(
        (a - b).abs() <= tol * b.abs().max(1.0),
        "{a} != {b}, tol={tol}"
    );
}
fn table(pairs: &[(f64, f64)], zeta: f64) -> TabulatedSpectrum {
    TabulatedSpectrum::new(
        pairs,
        PERIOD_SI,
        zeta,
        Interpolation::Linear,
        Extrapolation::Error,
    )
    .unwrap()
}
fn options(count: usize, factors: Vec<f64>) -> SpectrumOptions {
    SpectrumOptions {
        mode_count: count,
        mass_kind: MassKind::Lumped,
        modal_combination: ModalCombination::Srss,
        directional_combination: DirectionalCombination::WeightedSum,
        directional_factors: factors,
    }
}
fn support(model: &mut Model, node: u32, dofs: &[Dof]) {
    model.supports.push(Support {
        id: format!("s{node}"),
        node: NodeId(node),
        restrained: dofs.to_vec(),
    });
}
// V=½k Σ(q_j−q_{j−1})², q_0=0 and free roof, as in ENG-ANL-002.
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
fn interpolation_si_conversion_and_explicit_extrapolation() {
    let linear = table(&[(1.0, 2.0), (3.0, 6.0)], 0.05);
    assert_eq!(linear.input_units(), PERIOD_SI);
    assert_eq!(linear.interpolation(), Interpolation::Linear);
    assert_eq!(linear.extrapolation(), Extrapolation::Error);
    assert_eq!(linear.damping(), 0.05);
    for (t, sa) in [(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)] {
        assert_eq!(linear.acceleration(t).unwrap(), sa);
    }
    for t in [0.0, 0.9, 3.1, 1e200] {
        assert_eq!(linear.acceleration(t).unwrap_err().code, "ENG-E-0010");
    }
    let plateau = TabulatedSpectrum::new(
        &[(1.0, 2.0), (3.0, 6.0)],
        PERIOD_SI,
        0.05,
        Interpolation::Linear,
        Extrapolation::Plateau,
    )
    .unwrap();
    assert_eq!(plateau.acceleration(0.0).unwrap(), 2.0);
    assert_eq!(plateau.acceleration(1e200).unwrap(), 6.0);
    let log = TabulatedSpectrum::new(
        &[(1.0, 2.0), (4.0, 32.0)],
        PERIOD_SI,
        0.1,
        Interpolation::LogLog,
        Extrapolation::Plateau,
    )
    .unwrap();
    // Sa=2T², derived here; T=2 ⇒ Sa=8.
    close(log.acceleration(2.0).unwrap(), 8.0, 1e-14);
    assert_eq!(log.acceleration(0.0).unwrap(), 2.0);
    let frequency = TabulatedSpectrum::new(
        &[(0.25, 32.0), (1.0, 2.0)],
        FREQUENCY_SI,
        0.1,
        Interpolation::Linear,
        Extrapolation::Error,
    )
    .unwrap();
    assert_eq!(frequency.input_units(), FREQUENCY_SI);
    assert_eq!(frequency.period_points(), &[(1.0, 2.0), (4.0, 32.0)]);
    // Interpolates in T: a=(2−1)/(4−1), not in f.
    assert_eq!(frequency.acceleration(2.0).unwrap(), 12.0);
    assert_eq!(
        table(&[(0.0, 0.0), (1.0, 1.0)], 0.0)
            .acceleration(0.5)
            .unwrap(),
        0.5
    );
}

#[test]
fn invalid_tables_queries_and_damping_use_catalog() {
    for pairs in [
        vec![],
        vec![(1.0, 1.0)],
        vec![(1.0, 1.0), (1.0, 2.0)],
        vec![(2.0, 1.0), (1.0, 2.0)],
        vec![(-1.0, 1.0), (1.0, 2.0)],
        vec![(0.0, -1.0), (1.0, 2.0)],
    ] {
        assert_eq!(
            TabulatedSpectrum::new(
                &pairs,
                PERIOD_SI,
                0.05,
                Interpolation::Linear,
                Extrapolation::Error
            )
            .unwrap_err()
            .code,
            "ENG-E-0010"
        );
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for pairs in [[(bad, 1.0), (2.0, 2.0)], [(1.0, bad), (2.0, 2.0)]] {
            assert_eq!(
                TabulatedSpectrum::new(
                    &pairs,
                    PERIOD_SI,
                    0.05,
                    Interpolation::Linear,
                    Extrapolation::Error
                )
                .unwrap_err()
                .code,
                "ENG-E-0008"
            );
        }
        assert_eq!(
            table(&[(1.0, 1.0), (2.0, 2.0)], 0.0)
                .acceleration(bad)
                .unwrap_err()
                .code,
            "ENG-E-0008"
        );
    }
    for zeta in [-0.01, 1.0, 2.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            TabulatedSpectrum::new(
                &[(1.0, 1.0), (2.0, 2.0)],
                PERIOD_SI,
                zeta,
                Interpolation::Linear,
                Extrapolation::Error
            )
            .unwrap_err()
            .code,
            if zeta.is_finite() {
                "ENG-E-0010"
            } else {
                "ENG-E-0008"
            }
        );
    }
    for pairs in [[(0.0, 1.0), (2.0, 2.0)], [(1.0, 0.0), (2.0, 2.0)]] {
        assert_eq!(
            TabulatedSpectrum::new(
                &pairs,
                PERIOD_SI,
                0.0,
                Interpolation::LogLog,
                Extrapolation::Error
            )
            .unwrap_err()
            .code,
            "ENG-E-0010"
        );
    }
    assert_eq!(
        TabulatedSpectrum::new(
            &[(0.0, 1.0), (1.0, 1.0)],
            FREQUENCY_SI,
            0.0,
            Interpolation::Linear,
            Extrapolation::Error
        )
        .unwrap_err()
        .code,
        "ENG-E-0010"
    );
    assert_eq!(
        TabulatedSpectrum::new(
            &[(f64::from_bits(1), 1.0), (1.0, 1.0)],
            FREQUENCY_SI,
            0.0,
            Interpolation::Linear,
            Extrapolation::Error
        )
        .unwrap_err()
        .code,
        "ENG-E-0008"
    );
    assert_eq!(
        table(&[(1.0, 1.0), (2.0, 2.0)], 0.0)
            .acceleration(-1.0)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
}

#[test]
fn cqc_common_general_symmetry_diagonal_bounds_and_zero_damping() {
    for zeta in [0.0, 1e-200, 0.01, 0.05, 0.9] {
        assert_eq!(cqc_correlation(2.0, 2.0, zeta, zeta).unwrap(), 1.0);
        for r in [1e-12_f64, 0.01, 0.2, 0.9, 0.9999, 1.0, 1.01, 3.0, 1e12] {
            let actual = cqc_correlation(1.0, r, zeta, zeta).unwrap();
            assert!((0.0..=1.0).contains(&actual));
            assert_eq!(actual, cqc_correlation(r, 1.0, zeta, zeta).unwrap());
            if zeta == 0.0 {
                assert_eq!(actual, if r == 1.0 { 1.0 } else { 0.0 });
            } else if zeta > 1e-100 {
                let exact = 8.0 * math::powi(zeta, 2) * (1.0 + r) * math::powf(r, 1.5)
                    / (math::powi(1.0 - r * r, 2)
                        + 4.0 * math::powi(zeta, 2) * r * math::powi(1.0 + r, 2));
                close(actual, exact, 1e-14);
            }
        }
    }
    for (zi, zj) in [(0.02_f64, 0.07_f64), (0.0, 0.1), (0.4, 0.9)] {
        for r in [0.1_f64, 0.95, 1.0, 1.4, 12.0] {
            let exact = 8.0 * math::sqrt(zi * zj) * (zi + r * zj) * math::powf(r, 1.5)
                / (math::powi(1.0 - r * r, 2)
                    + 4.0 * zi * zj * r * (1.0 + r * r)
                    + 4.0 * (zi * zi + zj * zj) * r * r);
            let actual = cqc_correlation(1.0, r, zi, zj).unwrap();
            close(actual, exact, 1e-14);
            assert_eq!(actual, cqc_correlation(r, 1.0, zj, zi).unwrap());
            assert!((0.0..=1.0).contains(&actual));
        }
    }
    assert_eq!(cqc_correlation(1.0, 2.0, 0.0, 0.05).unwrap(), 0.0);
    assert_eq!(cqc_correlation(1e-300, 1e300, 0.05, 0.05).unwrap(), 0.0);
}

#[test]
fn cqc_tends_to_srss_far_modes_and_keeps_signed_cross_terms() {
    let values = [3.0, 4.0];
    let zeta = [0.05; 2];
    let frequencies = [1.0, 1000.0];
    let rho = cqc_correlation(1.0, 1000.0, 0.05, 0.05).unwrap();
    assert!(rho < 1e-6);
    let srss = combine_modal(&values, &frequencies, &zeta, ModalCombination::Srss).unwrap();
    let cqc = combine_modal(&values, &frequencies, &zeta, ModalCombination::Cqc).unwrap();
    assert_eq!(srss, 5.0);
    // Declared relative tolerance 1e−6 with r=1000 and ζ=0.05.
    assert!((cqc / srss - 1.0).abs() < 1e-6);
    let closer = cqc_correlation(1.0, 1.01, 0.05, 0.05).unwrap();
    assert!(closer > 0.9);
    close(
        combine_modal(&[3.0, -4.0], &[1.0, 1.01], &zeta, ModalCombination::Cqc).unwrap(),
        math::sqrt(25.0 - 24.0 * closer),
        1e-14,
    );
    assert_eq!(
        combine_modal(&[3.0, -3.0], &[1.0, 1.0], &[0.0; 2], ModalCombination::Cqc).unwrap(),
        0.0
    );
    close(
        combine_modal(&[1e200, 1e200], &[1.0, 2.0], &zeta, ModalCombination::Srss).unwrap(),
        math::sqrt(2.0_f64) * 1e200,
        1e-14,
    );
}

#[test]
fn shear_building_own_closed_form_constant_and_linear_tables_all_modes() {
    assert_eq!(Spectrum::KIND, "spectrum");
    for n in [1, 4, 9] {
        for linear in [false, true] {
            let (k, m) = (17.0_f64, 3.0_f64);
            let pairs = if linear {
                [(0.0, 1.0), (100.0, 21.0)]
            } else {
                [(0.0, 2.5), (100.0, 2.5)]
            };
            let spectra = [table(&pairs, 0.05)];
            let input = [SpectrumDirection {
                direction: [5.0, 0.0, 0.0],
                spectra: &spectra,
            }];
            let mut system = shear(n, k, m);
            let result = Spectrum::run(&mut system, &input, &options(n, vec![1.0])).unwrap();
            assert_eq!(result.directions[0].unit_direction, [1.0, 0.0, 0.0]);
            let mut basal_squared = 0.0;
            let mut mass_sum = 0.0;
            let mut displacement_squared = vec![0.0; n];
            for (index, actual) in result.directions[0].modes.iter().enumerate() {
                // θ=(2n−1)π/(2N+1), ω²=4k/m sin²(θ/2), φ_j=sin(jθ).
                let theta = (2 * index + 1) as f64 * PI / (2 * n + 1) as f64;
                let lambda = 4.0 * k / m * math::powi(math::sin(theta / 2.0), 2);
                let period = TAU / math::sqrt(lambda);
                let raw: Vec<_> = (1..=n).map(|j| math::sin(j as f64 * theta)).collect();
                let norm = math::sqrt(m * raw.iter().map(|v| v * v).sum::<f64>());
                let projection = m * raw.iter().sum::<f64>() / norm;
                let sign = if result.modal.modes[index].free_shape[0] * raw[0] < 0.0 {
                    -1.0
                } else {
                    1.0
                };
                let gamma = sign * projection;
                let sa = if linear { 1.0 + 0.2 * period } else { 2.5 };
                let q = gamma * sa / lambda;
                let effective_mass = projection * projection;
                let basal = effective_mass * sa;
                close(
                    result.modal.modes[index].participation[0].factor,
                    gamma,
                    3e-11,
                );
                close(actual.modal_displacement, q, 3e-11);
                close(actual.spectral_acceleration, sa, 3e-11);
                close(actual.response.base_shear, basal, 3e-11);
                close(actual.response.reactions[0].value, -basal, 3e-11);
                for (j, &value) in raw.iter().enumerate() {
                    let u = sign * value / norm * q;
                    close(actual.response.free_displacements[j], u, 3e-11);
                    close(actual.response.global_displacements[(j + 1) * 2], u, 3e-11);
                    close(
                        actual.equivalent_forces[(j + 1) * 2],
                        m * sign * value / norm * gamma * sa,
                        3e-11,
                    );
                    displacement_squared[j] += u * u;
                }
                for values in [
                    &actual.response.global_displacements,
                    &actual.equivalent_forces,
                ] {
                    assert_eq!(values[0], 0.0);
                    assert_eq!(values[1], 0.0);
                }
                mass_sum += effective_mass;
                basal_squared += basal * basal;
            }
            close(mass_sum, n as f64 * m, 3e-11);
            // Own derivation: V_b,SRSS²=Σ(M*_n Sa_n)², NOT Σ M*_n Sa_n.
            close(
                math::powi(result.directions[0].combined.base_shear, 2),
                basal_squared,
                3e-11,
            );
            close(
                math::powi(result.combined.base_shear, 2),
                basal_squared,
                3e-11,
            );
            for (u, square) in result
                .combined
                .free_displacements
                .iter()
                .zip(displacement_squared)
            {
                close(*u, math::sqrt(square), 3e-11);
            }
            assert_eq!(system.accepted_displacement(), vec![0.0; n]);
            assert_eq!(system.trial_displacement(), vec![0.0; n]);
        }
    }
}

#[test]
fn direction_normalization_reversal_oblique_and_user_factors() {
    let spectra = [table(&[(0.0, 2.0), (100.0, 2.0)], 0.05)];
    let inputs =
        [[1.0, 0.0, 0.0], [-4.0, 0.0, 0.0], [1.0, 1.0, 0.0]].map(|direction| SpectrumDirection {
            direction,
            spectra: &spectra,
        });
    let mut system = shear(4, 17.0, 3.0);
    let mut opt = options(4, vec![1.0, 0.3, 0.0]);
    let result = Spectrum::run(&mut system, &inputs, &opt).unwrap();
    for (a, b) in result.directions[0]
        .modes
        .iter()
        .zip(&result.directions[1].modes)
    {
        assert_eq!(a.modal_displacement, -b.modal_displacement);
        assert_eq!(a.response.base_shear, b.response.base_shear);
        for (ua, ub) in a
            .response
            .global_displacements
            .iter()
            .zip(&b.response.global_displacements)
        {
            assert_eq!(ua, &(-ub));
        }
    }
    for (a, b) in result.directions[0]
        .modes
        .iter()
        .zip(&result.directions[2].modes)
    {
        close(b.response.base_shear, a.response.base_shear * 0.5, 1e-14);
        close(
            b.response.free_displacements[0],
            a.response.free_displacements[0] / math::sqrt(2.0_f64),
            1e-14,
        );
    }
    let x = result.directions[0].combined.base_shear;
    close(result.combined.base_shear, 1.3 * x, 1e-14);
    opt.directional_combination = DirectionalCombination::Srss;
    let srss = Spectrum::run(&mut system, &inputs, &opt).unwrap();
    close(srss.combined.base_shear, math::sqrt(1.09_f64) * x, 1e-14);
    // Entirely user-supplied factors, without built-in 100%/30%.
    assert_eq!(
        combine_directions(
            &[10.0, 20.0],
            &[0.4, 0.7],
            DirectionalCombination::WeightedSum
        )
        .unwrap(),
        18.0
    );
}

#[test]
fn per_mode_damping_tables_cqc() {
    let spectra = [
        table(&[(0.0, 1.0), (100.0, 1.0)], 0.02),
        table(&[(0.0, 3.0), (100.0, 3.0)], 0.07),
    ];
    let mut system = shear(2, 17.0, 3.0);
    let inputs = [SpectrumDirection {
        direction: [1.0, 0.0, 0.0],
        spectra: &spectra,
    }];
    let mut opt = options(2, vec![1.0]);
    opt.modal_combination = ModalCombination::Cqc;
    let result = Spectrum::run(&mut system, &inputs, &opt).unwrap();
    let d = &result.directions[0];
    assert_eq!(d.modes[0].damping, 0.02);
    assert_eq!(d.modes[1].damping, 0.07);
    assert_eq!(d.modes[1].spectral_acceleration, 3.0);
    let rho = cqc_correlation(
        result.modal.modes[0].angular_frequency,
        result.modal.modes[1].angular_frequency,
        0.02,
        0.07,
    )
    .unwrap();
    let (a, b) = (
        d.modes[0].response.base_shear,
        d.modes[1].response.base_shear,
    );
    close(
        d.combined.base_shear,
        math::sqrt(a * a + b * b + 2.0 * rho * a * b),
        1e-14,
    );
    for i in 0..2 {
        let (a, b) = (
            d.modes[0].response.free_displacements[i],
            d.modes[1].response.free_displacements[i],
        );
        close(
            d.combined.free_displacements[i],
            math::sqrt(a * a + b * b + 2.0 * rho * a * b),
            1e-14,
        );
    }
}

#[test]
fn consistent_mass_support_coupling_settlement_and_member_recovery() {
    let mut model = Model::new();
    for j in 0..2 {
        model
            .nodes
            .push(Node::new(NodeId(j + 1), Dim::D2, [j as f64, 0.0, 0.0]));
    }
    support(&mut model, 1, Dim::D2.dofs());
    support(&mut model, 2, &[Dof::Uy, Dof::Rz]);
    let element = EulerFrame2D::new(12.0, 1.0, 1.0, 6.0, MomentReleases::default()).unwrap();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [1.0, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let contribution = ElementContribution {
        location: (0..6).map(engine_core::DofIndex).collect(),
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
    let before_accepted = system.accepted_displacement().to_vec();
    let before_trial = system.trial_displacement().to_vec();
    let spectra = [table(&[(0.0, 2.0), (100.0, 2.0)], 0.05)];
    let inputs = [SpectrumDirection {
        direction: [1.0, 0.0, 0.0],
        spectra: &spectra,
    }];
    let mut opt = options(1, vec![1.0]);
    opt.mass_kind = MassKind::Consistent;
    let result = Spectrum::run(&mut system, &inputs, &opt).unwrap();
    let mode = &result.directions[0].modes[0];
    // M=(ρAL/6)[[2,1],[1,2]]=[[2,1],[1,2]], Kff=12 y ω²=6.
    // Γ=√2, Sa=2, u_f=1/3, F_p=2, F_f=4, R_p=−4−2=−6.
    close(mode.response.free_displacements[0], 1.0 / 3.0, 1e-14);
    close(mode.equivalent_forces[0], 2.0, 1e-14);
    close(mode.equivalent_forces[3], 4.0, 1e-14);
    close(mode.response.reactions[0].value, -6.0, 1e-14);
    close(mode.response.base_shear, 6.0, 1e-14);
    assert_eq!(mode.response.global_displacements[0], 0.0);
    let recovered = element
        .recover(&ctx, &mode.response.global_displacements, &[], &[], &[0.5])
        .unwrap();
    close(recovered.stations[0].forces_local[0], 4.0, 1e-14);
    let member = combine_modal(
        &[recovered.stations[0].forces_local[0]],
        &[math::sqrt(6.0_f64)],
        &[0.05],
        ModalCombination::Cqc,
    )
    .unwrap();
    close(member, 4.0, 1e-14);
    assert_eq!(system.accepted_displacement(), before_accepted);
    assert_eq!(system.trial_displacement(), before_trial);
}

#[test]
fn input_errors_and_no_state_change_after_out_of_range() {
    let spectra = [table(&[(0.01, 1.0), (0.02, 1.0)], 0.05)];
    let mut system = shear(1, 17.0, 3.0);
    system.accept(&[0.4]).unwrap();
    let before = system.accepted_displacement().to_vec();
    let inputs = [SpectrumDirection {
        direction: [1.0, 0.0, 0.0],
        spectra: &spectra,
    }];
    assert_eq!(
        Spectrum::run(&mut system, &inputs, &options(1, vec![1.0]))
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    assert_eq!(system.accepted_displacement(), before);
    assert_eq!(
        Spectrum::run(&mut system, &inputs, &options(0, vec![1.0]))
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    assert_eq!(
        Spectrum::run(&mut system, &[], &options(1, vec![]))
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    for direction in [[0.0; 3], [f64::NAN, 0.0, 0.0]] {
        let bad = [SpectrumDirection {
            direction,
            spectra: &spectra,
        }];
        assert_eq!(
            Spectrum::run(&mut system, &bad, &options(1, vec![1.0]))
                .unwrap_err()
                .code,
            if direction[0].is_nan() {
                "ENG-E-0008"
            } else {
                "ENG-E-0010"
            }
        );
    }
    let empty = [SpectrumDirection {
        direction: [1.0, 0.0, 0.0],
        spectra: &[],
    }];
    assert_eq!(
        Spectrum::run(&mut system, &empty, &options(1, vec![1.0]))
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    for factors in [vec![], vec![-1.0], vec![f64::NAN], vec![1.0, 1.0]] {
        let code = if factors.iter().any(|v| v.is_nan()) {
            "ENG-E-0008"
        } else {
            "ENG-E-0010"
        };
        assert_eq!(
            Spectrum::run(&mut system, &inputs, &options(1, factors))
                .unwrap_err()
                .code,
            code
        );
    }
    let mut overflow = shear(1, 1e-10, 1.0);
    let huge = [table(&[(0.0, f64::MAX), (1e10, f64::MAX)], 0.05)];
    let inputs = [SpectrumDirection {
        direction: [1.0, 0.0, 0.0],
        spectra: &huge,
    }];
    assert_eq!(
        Spectrum::run(&mut overflow, &inputs, &options(1, vec![1.0]))
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
}

#[test]
fn combination_errors_are_fallible() {
    assert_eq!(
        combine_modal(&[], &[], &[], ModalCombination::Cqc)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    assert_eq!(
        combine_modal(&[1.0], &[1.0, 2.0], &[0.05; 2], ModalCombination::Srss)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    assert_eq!(
        combine_modal(&[1.0], &[1.0], &[], ModalCombination::Cqc)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    assert_eq!(
        combine_modal(&[f64::NAN], &[1.0], &[0.05], ModalCombination::Cqc)
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    for (omega, zeta, code) in [
        (0.0, 0.05, "ENG-E-0010"),
        (-1.0, 0.05, "ENG-E-0010"),
        (1.0, 1.0, "ENG-E-0010"),
        (f64::NAN, 0.05, "ENG-E-0008"),
        (1.0, f64::NAN, "ENG-E-0008"),
    ] {
        assert_eq!(
            cqc_correlation(omega, 1.0, zeta, 0.05).unwrap_err().code,
            code
        );
    }
    assert_eq!(
        combine_directions(&[f64::MAX], &[2.0], DirectionalCombination::Srss)
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
}

#[test]
fn multiple_supports_combine_basal_after_signed_sum() {
    // Two independent oscillators: m=(2,3), k=(18,48), ω=(3,4).
    let mut model = Model::new();
    for j in 0..4 {
        let mut node = Node::new(NodeId(j + 1), Dim::D2, [j as f64, 0.0, 0.0]);
        node.dofs = vec![Dof::Ux, Dof::Uy];
        model.nodes.push(node);
        support(
            &mut model,
            j + 1,
            if j % 2 == 0 {
                &[Dof::Ux, Dof::Uy]
            } else {
                &[Dof::Uy]
            },
        );
    }
    let map = DofMap::new(&model.nodes).unwrap();
    let elements: Vec<_> = [(1, 2, 18.0, 2.0), (3, 4, 48.0, 3.0)]
        .into_iter()
        .map(|(base, top, k, m)| {
            model.masses.push(NodalMass {
                id: format!("m{top}"),
                node: NodeId(top),
                values: vec![DofValue {
                    dof: Dof::Ux,
                    value: m,
                }],
            });
            ElementContribution {
                location: [base, top]
                    .map(|id| map.global_index(NodeId(id), Dof::Ux).unwrap())
                    .to_vec(),
                stiffness: DMat::new(2, 2, vec![k, -k, -k, k]).unwrap(),
                mass_lumped: None,
                mass_consistent: None,
                geometric: None,
                loads: vec![],
            }
        })
        .collect();
    let mut system = LinearSystem::from_model(ReferenceBackend, &model, &elements, &[]).unwrap();
    let spectra = [table(&[(0.0, 2.0), (10.0, 2.0)], 0.05)];
    let inputs = [SpectrumDirection {
        direction: [1.0, 0.0, 0.0],
        spectra: &spectra,
    }];
    let result = Spectrum::run(&mut system, &inputs, &options(2, vec![1.0])).unwrap();
    // V=(4,6) ⇒ V_SRSS=√52; summing support envelopes would give 10.
    close(result.combined.base_shear, math::sqrt(52.0_f64), 1e-14);
    let sum_envelopes: f64 = result
        .combined
        .reactions
        .iter()
        .filter(|r| r.dof == Dof::Ux)
        .map(|r| r.value)
        .sum();
    close(sum_envelopes, 10.0, 1e-14);
    assert!(sum_envelopes > result.combined.base_shear);
}

#[test]
fn rigid_modes_rejected_and_log_log_numerical_ranges() {
    let spectra = [table(&[(0.0, 1.0), (100.0, 1.0)], 0.05)];
    let inputs = [SpectrumDirection {
        direction: [1.0, 0.0, 0.0],
        spectra: &spectra,
    }];
    let mut system = shear(1, 0.0, 3.0);
    assert_eq!(
        Spectrum::run(&mut system, &inputs, &options(1, vec![1.0]))
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    // Distinct periods whose rounded ln would be identical.
    let low = 1e100_f64;
    let high = f64::from_bits(low.to_bits() + 4);
    let mid = f64::from_bits(low.to_bits() + 2);
    let near = TabulatedSpectrum::new(
        &[(low, 1.0), (high, 4.0)],
        PERIOD_SI,
        0.0,
        Interpolation::LogLog,
        Extrapolation::Error,
    )
    .unwrap();
    close(near.acceleration(mid).unwrap(), 2.0, 1e-14);
    let wide = TabulatedSpectrum::new(
        &[(1e-200, 1e-100), (1e200, 1e100)],
        PERIOD_SI,
        0.0,
        Interpolation::LogLog,
        Extrapolation::Error,
    )
    .unwrap();
    close(wide.acceleration(1.0).unwrap(), 1.0, 1e-14);
}
