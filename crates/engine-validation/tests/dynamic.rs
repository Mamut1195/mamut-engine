//! Independent analytical oracles: provenance/ENG-VAL-001.md, dynamic extension.
#[path = "support/dynamic_harness.rs"]
mod harness;
#[path = "support/dynamic_table.rs"]
mod table;

use engine_analysis::{
    modal::{Modal, ModalResults},
    pdelta::{PDelta, PDeltaOptions, PDeltaOutcome},
    spectrum::{
        combine_modal, cqc_correlation, DirectionalCombination, Extrapolation, Interpolation,
        ModalCombination, Spectrum, SpectrumDirection, SpectrumOptions, SpectrumUnits,
        TabulatedSpectrum,
    },
    StaticLinear,
};
use engine_assembly::{DofMap, ElementContribution, GeometricProducer, LinearSystem};
use engine_contracts::{CaseRef, MassKind};
use engine_core::math;
use engine_core::{Dim, Dof, EResult, NodeId};
use engine_linalg::{DMat, ReferenceBackend};
use engine_model::{DofValue, Model, NodalMass, Node};
use harness::{
    beam, column, dot, load, missing, portal, require, run_group, shear, support, Check,
};
use std::f64::consts::PI;
use table::{Fixture, Parameters, Tolerance, MODAL, PDELTA, SPECTRUM};

fn numerical() -> Check {
    Check(Tolerance {
        absolute: 2e-9,
        relative: 2e-8,
        justification: "Contraste numérico f64 independiente de la aproximación de modelo/malla.",
    })
}
fn orthogonal(system: &LinearSystem<ReferenceBackend>, result: &ModalResults) -> EResult<()> {
    let c = numerical();
    for (i, a) in result.modes.iter().enumerate() {
        for (j, b) in result.modes.iter().enumerate() {
            c.close(
                "φᵢᵀMφⱼ",
                dot(
                    system.global_mass(result.mass_kind),
                    &a.global_shape,
                    &b.global_shape,
                )?,
                if i == j { 1.0 } else { 0.0 },
            )?;
            c.close(
                "φᵢᵀKφⱼ / √(λᵢλⱼ)",
                dot(system.stiffness(), &a.global_shape, &b.global_shape)?
                    / math::sqrt(a.eigenvalue * b.eigenvalue),
                if i == j { 1.0 } else { 0.0 },
            )?;
        }
    }
    Ok(())
}
fn shear_oracle(
    result: &ModalResults,
    stiffness: &[f64],
    masses: &[f64],
    c: &Check,
    variable: bool,
) -> EResult<()> {
    let n = masses.len();
    let mut cumulative = 0.0;
    for (i, mode) in result.modes.iter().enumerate() {
        let (lambda, raw) = if variable {
            // det(K−λM)=m1m2λ²−[m2(k1+k2)+m1k2]λ+k1k2.
            let [k1, k2] = [stiffness[0], stiffness[1]];
            let [m1, m2] = [masses[0], masses[1]];
            let b = m2 * (k1 + k2) + m1 * k2;
            let root = math::sqrt(b * b - 4.0 * m1 * m2 * k1 * k2);
            let lambda = (b + if i == 0 { -root } else { root }) / (2.0 * m1 * m2);
            (lambda, vec![1.0, (k1 + k2 - lambda * m1) / k2])
        } else {
            let theta = (2 * i + 1) as f64 * PI / (2 * n + 1) as f64;
            (
                4.0 * stiffness[0] / masses[0] * math::powi(math::sin(theta / 2.0), 2),
                (1..=n)
                    .map(|j| math::sin(j as f64 * theta))
                    .collect::<Vec<_>>(),
            )
        };
        c.close("ω analítica", mode.angular_frequency, math::sqrt(lambda))?;
        let norm = math::sqrt(raw.iter().zip(masses).map(|(v, m)| m * v * v).sum::<f64>());
        let sign = if mode.free_shape[0] * raw[0] < 0.0 {
            -1.0
        } else {
            1.0
        };
        for (&actual, &v) in mode.free_shape.iter().zip(&raw) {
            c.close("φ normalizada", actual, sign * v / norm)?;
        }
        let gamma = sign * raw.iter().zip(masses).map(|(v, m)| m * v).sum::<f64>() / norm;
        c.close("Γ por suma manual", mode.participation[0].factor, gamma)?;
        c.close(
            "M* por Γ²",
            mode.participation[0].effective_mass,
            gamma * gamma,
        )?;
        cumulative += gamma * gamma;
        c.close(
            "masa acumulada por modo",
            mode.participation[0].cumulative_effective_mass,
            cumulative,
        )?;
    }
    let total: f64 = masses.iter().sum();
    c.close("Σ M*=masa total", cumulative, total)?;
    let report = result.directions.first().ok_or_else(missing)?;
    c.close("masa total reportada", report.total_mass, total)?;
    c.close("masa libre reportada", report.free_mass, total)?;
    c.close(
        "masa acumulada reportada",
        report.cumulative_effective_mass,
        total,
    )?;
    c.close(
        "fracción acumulada=1",
        report.cumulative_mass_fraction.ok_or_else(missing)?,
        1.0,
    )
}
fn beta(cantilever: bool) -> [f64; 3] {
    if !cantilever {
        return [PI, 2.0 * PI, 3.0 * PI];
    }
    // Roots of cos β cosh β=−1 computed here, without published figures.
    std::array::from_fn(|i| {
        let (mut lo, mut hi) = (i as f64 * PI + 0.1, (i + 1) as f64 * PI);
        for _ in 0..55 {
            let mid = (lo + hi) / 2.0;
            let f = |v: f64| math::cos(v) * math::cosh(v) + 1.0;
            if f(lo) * f(mid) <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        (lo + hi) / 2.0
    })
}
fn validate_modal(f: &Fixture) -> EResult<()> {
    let c = Check(f.tolerance);
    match f.parameters {
        Parameters::Shear { n, k, m } | Parameters::Chain { n, k, m } => {
            let mut s = shear(
                &vec![k; n],
                &vec![m; n],
                matches!(f.parameters, Parameters::Chain { .. }),
            )?;
            let r = Modal::run(&mut s, n, MassKind::Lumped, &[[1.0, 0.0, 0.0]])?;
            shear_oracle(&r, &vec![k; n], &vec![m; n], &c, false)?;
            orthogonal(&s, &r)
        }
        Parameters::Variable { stiffness, masses } => {
            let mut s = shear(&stiffness, &masses, false)?;
            let r = Modal::run(&mut s, 2, MassKind::Consistent, &[[1.0, 0.0, 0.0]])?;
            shear_oracle(&r, &stiffness, &masses, &c, true)?;
            orthogonal(&s, &r)
        }
        Parameters::Beam {
            mesh,
            cantilever,
            angle,
        } => {
            let direction = [-math::sin(angle), math::cos(angle), 0.0];
            let mut s = beam(mesh, cantilever, angle, f.frame)?;
            let r = Modal::run(&mut s, 3, MassKind::Consistent, &[direction])?;
            let roots = beta(cantilever);
            let scale = math::sqrt(f.frame.e * f.frame.inertia / (f.frame.density * f.frame.area))
                / math::powi(f.frame.length, 2);
            for (mode, root) in r.modes.iter().zip(roots) {
                c.close(
                    "ω=β²√(EI/ρA)/L²",
                    mode.angular_frequency,
                    scale * root * root,
                )?;
            }
            orthogonal(&s, &r)?;
            if mesh == 12 {
                let mut coarse = beam(4, cantilever, angle, f.frame)?;
                let cr = Modal::run(&mut coarse, 3, MassKind::Consistent, &[direction])?;
                for ((fine, coarse), root) in r.modes.iter().zip(&cr.modes).zip(roots) {
                    let exact = scale * root * root;
                    require(
                        (fine.angular_frequency / exact - 1.0).abs()
                            < (coarse.angular_frequency / exact - 1.0).abs() / 10.0,
                        "Refinamiento 4→12: el error no disminuye más de 10 veces",
                    )?;
                }
            }
            if angle != 0.0 {
                let mut unrotated = beam(mesh, cantilever, 0.0, f.frame)?;
                let original =
                    Modal::run(&mut unrotated, 3, MassKind::Consistent, &[[0.0, 1.0, 0.0]])?;
                let tight = numerical();
                for (a, b) in original.modes.iter().zip(&r.modes) {
                    tight.close("ω invariante", b.angular_frequency, a.angular_frequency)?;
                    tight.close(
                        "M* invariante",
                        b.participation[0].effective_mass,
                        a.participation[0].effective_mass,
                    )?;
                    tight.close(
                        "|Γ| invariante",
                        b.participation[0].factor.abs(),
                        a.participation[0].factor.abs(),
                    )?;
                    let expected: Vec<_> = a
                        .global_shape
                        .chunks_exact(3)
                        .flat_map(|v| {
                            [
                                math::cos(angle) * v[0] - math::sin(angle) * v[1],
                                math::sin(angle) * v[0] + math::cos(angle) * v[1],
                                v[2],
                            ]
                        })
                        .collect();
                    let sign = if dot(
                        s.global_mass(MassKind::Consistent),
                        &expected,
                        &b.global_shape,
                    )? < 0.0
                    {
                        -1.0
                    } else {
                        1.0
                    };
                    for (&x, &y) in expected.iter().zip(&b.global_shape) {
                        tight.close("Qφ rotada", y, sign * x)?;
                    }
                }
            }
            Ok(())
        }
        _ => Err(missing()),
    }
}
fn options() -> PDeltaOptions {
    PDeltaOptions {
        tolerance: 1e-10,
        max_iterations: 2000,
        displacement_scale: 0.01,
        rotation_scale: 0.01,
    }
}
fn case(name: &str) -> (CaseRef, f64) {
    (CaseRef(name.into()), 1.0)
}
fn completed(r: &engine_analysis::pdelta::PDeltaResult) -> EResult<()> {
    require(
        r.outcome == PDeltaOutcome::Completed,
        "P-Delta no completado",
    )?;
    let last = r.iterations.last().ok_or_else(missing)?;
    require(
        last.relative_displacement_change <= r.options.tolerance
            && last.residual_convergence_ratio <= 1.0,
        "Iteración final fuera de tolerancia",
    )
}
fn validate_pdelta(f: &Fixture) -> EResult<()> {
    let c = Check(f.tolerance);
    let ei = f.frame.e * f.frame.inertia;
    let length = f.frame.length;
    let force = f.frame.force;
    match f.parameters {
        Parameters::Column { mesh, ratio } => {
            let pcr = PI * PI * ei / (4.0 * length * length);
            let mut s = column(mesh, ratio * pcr, f.frame)?;
            let r = PDelta::run(&mut s, &case("G"), &case("H"), options())?;
            if ratio > 1.0 {
                require(
                    r.outcome
                        == PDeltaOutcome::Unstable {
                            negative_eigenvalues: 1,
                        },
                    "P>Pcr debe ser Unstable",
                )?;
                return require(
                    r.global_displacements.is_none() && r.iterations.is_empty() && r.inertia.1 == 1,
                    "Unstable debe informar inercia y no desplazamientos/iteraciones",
                );
            }
            completed(&r)?;
            let u = r.global_displacements.as_ref().ok_or_else(missing)?;
            let tip = u[3 * mesh + 1];
            let linear = force * math::powi(length, 3) / (3.0 * ei);
            let exact = if ratio == 0.0 {
                linear
            } else {
                let z = PI * math::sqrt(ratio.abs()) / 2.0;
                linear
                    * 3.0
                    * if ratio > 0.0 {
                        (math::tan(z) - z) / math::powi(z, 3)
                    } else {
                        (z - math::tanh(z)) / math::powi(z, 3)
                    }
            };
            let mesh_check = Check(Tolerance { absolute: 1e-10, relative: 2e-6,
                justification: "Hermite, 16 elementos: contraste con solución continua exacta, separado del error de polo." });
            mesh_check.close("flecha continua exacta", tip, exact)?;
            if ratio > 0.0 {
                c.close(
                    "amplificación 1/(1−P/Pcr)",
                    tip / linear,
                    1.0 / (1.0 - ratio),
                )?;
            }
            if ratio < 0.0 {
                require(tip < linear && tip > 0.0, "Tracción debe rigidizar")?;
            }
            if ratio == 0.0 {
                let lr = StaticLinear::run(&mut s, &[case("H")])?;
                let lu = &lr.cases.first().ok_or_else(missing)?.global_displacements;
                for (&a, &b) in u.iter().zip(lu) {
                    numerical().close("P=0 reproduce lineal", a, b)?;
                }
            }
            // Independent refinement: same load and continuous solution.
            if ratio != 0.0 {
                let mut coarse = column(4, ratio * pcr, f.frame)?;
                let cr = PDelta::run(&mut coarse, &case("G"), &case("H"), options())?;
                completed(&cr)?;
                let ct = cr.global_displacements.as_ref().ok_or_else(missing)?[13];
                require(
                    (tip / exact - 1.0).abs() < (ct / exact - 1.0).abs() / 10.0,
                    "Columna 4→16 no converge hacia solución continua",
                )?;
            }
            Ok(())
        }
        Parameters::Portal { mesh, ratio, rigid } => {
            let n = 2.0;
            let pcr_floor = 12.0 * ei * n / math::powi(length, 2);
            let mut s = portal(mesh, ratio * pcr_floor, rigid, f.frame)?;
            let r = PDelta::run(&mut s, &case("G"), &case("H"), options())?;
            completed(&r)?;
            let u = r.global_displacements.as_ref().ok_or_else(missing)?;
            let linear = force * math::powi(length, 3) / (12.0 * ei * n);
            let z = math::sqrt(12.0 * ratio);
            let exact = linear * 12.0 * (2.0 * math::tan(z / 2.0) - z) / math::powi(z, 3);
            let mesh_check = Check(Tolerance { absolute: 1e-10, relative: 1e-4,
                justification: "12 elementos por columna y rigidez/EA×1e5: solución continua con giro superior cero." });
            for id in [mesh, 2 * mesh + 1] {
                c.close(
                    "deriva aproximada de piso",
                    u[3 * id] / linear,
                    1.0 / (1.0 - ratio),
                )?;
                mesh_check.close("deriva continua exacta del marco", u[3 * id], exact)?;
            }
            numerical().close("simetría de deriva", u[3 * mesh], u[3 * (2 * mesh + 1)])
        }
        Parameters::Iteration {
            k,
            ratio,
            force,
            tolerance,
            limit,
        } => {
            let mut model = Model::new();
            for id in 1..=2 {
                let mut node = Node::new(NodeId(id), Dim::D2, [id as f64, 0.0, 0.0]);
                node.dofs = vec![Dof::Ux];
                model.nodes.push(node);
            }
            support(&mut model, 1, &[Dof::Ux]);
            load(&mut model, "G", &[(2, Dof::Ux, -ratio * k)]);
            load(&mut model, "H", &[(2, Dof::Ux, force)]);
            let map = DofMap::new(&model.nodes)?;
            let location = vec![
                map.global_index(NodeId(1), Dof::Ux)?,
                map.global_index(NodeId(2), Dof::Ux)?,
            ];
            let entry = ElementContribution {
                location: location.clone(),
                stiffness: DMat::new(2, 2, vec![k, -k, -k, k])?,
                mass_lumped: None,
                mass_consistent: None,
                geometric: None,
                loads: vec![],
            };
            let mut s = LinearSystem::from_model(ReferenceBackend, &model, &[entry], &[])?;
            s.set_geometric_producers(vec![GeometricProducer {
                location,
                eval: Box::new(move |gravity| {
                    let kg = k * (gravity[1] - gravity[0]);
                    DMat::new(2, 2, vec![kg, -kg, -kg, kg])
                }),
            }])?;
            let settings = PDeltaOptions {
                tolerance,
                max_iterations: limit,
                displacement_scale: 1.0,
                rotation_scale: 1.0,
            };
            let r = PDelta::run(&mut s, &case("G"), &case("H"), settings)?;
            completed(&r)?;
            let mut expected_steps = 0;
            for step in 1..=limit {
                let residual = math::powi(ratio, step as i32);
                let change = force / k * math::powi(ratio, step as i32 - 1);
                if residual <= tolerance && change <= tolerance {
                    expected_steps = step;
                    break;
                }
            }
            require(
                r.iterations.len() == expected_steps,
                "Número de iteraciones no coincide con serie geométrica",
            )?;
            for report in &r.iterations {
                let step = report.iteration as i32;
                c.close(
                    "uᵢ=F/[k(1−r)](1−rⁱ)",
                    report.free_displacements[0],
                    force / (k * (1.0 - ratio)) * (1.0 - math::powi(ratio, step)),
                )?;
                c.close(
                    "residual relativo=rⁱ",
                    report.relative_free_residual,
                    math::powi(ratio, step),
                )?;
                c.close(
                    "cambio relativo=F/k rⁱ⁻¹",
                    report.relative_displacement_change,
                    force / k * math::powi(ratio, step - 1),
                )?;
            }
            let limited = PDelta::run(
                &mut s,
                &case("G"),
                &case("H"),
                PDeltaOptions {
                    max_iterations: 3,
                    ..settings
                },
            )?;
            require(
                limited.outcome == PDeltaOutcome::NotConverged && limited.iterations.len() == 3,
                "Límite de tres pasos debe devolver NotConverged",
            )
        }
        _ => Err(missing()),
    }
}
fn spectrum_table(
    points: &[(f64, f64)],
    damping: f64,
    interpolation: Interpolation,
) -> EResult<TabulatedSpectrum> {
    TabulatedSpectrum::new(
        points,
        SpectrumUnits::PeriodSecondsAccelerationMetersPerSecondSquared,
        damping,
        interpolation,
        Extrapolation::Error,
    )
}
fn spectrum_options(method: DirectionalCombination, factors: Vec<f64>) -> SpectrumOptions {
    SpectrumOptions {
        mode_count: 2,
        mass_kind: MassKind::Lumped,
        modal_combination: ModalCombination::Srss,
        directional_combination: method,
        directional_factors: factors,
    }
}
fn validate_spectrum(f: &Fixture) -> EResult<()> {
    let c = Check(f.tolerance);
    match f.parameters {
        Parameters::Spectrum { k, m, sa, damping } => {
            let mut s = shear(&[k, k], &[m, m], false)?;
            let spectra = [spectrum_table(
                &[(0.0, sa), (100.0, sa)],
                damping,
                Interpolation::Linear,
            )?];
            let r = Spectrum::run(
                &mut s,
                &[SpectrumDirection {
                    direction: [1.0, 0.0, 0.0],
                    spectra: &spectra,
                }],
                &spectrum_options(DirectionalCombination::WeightedSum, vec![1.0]),
            )?;
            let mut vb2 = 0.0;
            let mut displacement2 = [0.0; 2];
            for (i, response) in r.directions[0].modes.iter().enumerate() {
                // φ=(1,t)/√[m(1+t²)], t=(1±√5)/2; arbitrary sign.
                let t =
                    (1.0 + if i == 0 {
                        math::sqrt(5.0_f64)
                    } else {
                        -math::sqrt(5.0_f64)
                    }) / 2.0;
                let norm = math::sqrt(m * (1.0 + t * t));
                let gamma = m * (1.0 + t) / norm;
                let lambda = k / m * (2.0 - t);
                let effective_mass = gamma * gamma;
                c.close("Sa constante", response.spectral_acceleration, sa)?;
                c.close(
                    "|q|=|Γ|Sa/ω² manual",
                    response.modal_displacement.abs(),
                    (gamma * sa / lambda).abs(),
                )?;
                c.close(
                    "Vb,n=M*n Sa",
                    response.response.base_shear,
                    effective_mass * sa,
                )?;
                for (j, raw) in [1.0, t].iter().enumerate() {
                    let displacement = raw / norm * gamma * sa / lambda;
                    c.close(
                        "φq físico",
                        response.response.global_displacements[2 * (j + 1)],
                        displacement,
                    )?;
                    displacement2[j] += displacement * displacement;
                }
                vb2 += math::powi(effective_mass * sa, 2);
            }
            c.close(
                "Vb² SRSS=Σ(M*n Sa)²",
                math::powi(r.combined.base_shear, 2),
                vb2,
            )?;
            for (j, &d2) in displacement2.iter().enumerate() {
                c.close(
                    "desplazamiento SRSS",
                    r.combined.global_displacements[2 * (j + 1)],
                    math::sqrt(d2),
                )?;
            }
            Ok(())
        }
        Parameters::Interpolation {
            log,
            points,
            query,
            expected,
        } => {
            let spectrum = spectrum_table(
                &points,
                0.05,
                if log {
                    Interpolation::LogLog
                } else {
                    Interpolation::Linear
                },
            )?;
            for &(x, y) in &points {
                c.close("extremo de tabla", spectrum.acceleration(x)?, y)?;
            }
            c.close(
                "interpolación analítica",
                spectrum.acceleration(query)?,
                expected,
            )
        }
        Parameters::Cqc {
            frequencies: [wi, wj],
            damping: [zi, zj],
            responses: [a, b],
        } => {
            // WDKB 1981 using the requested formula, independent direct expansion.
            let r = wj / wi;
            let rho = 8.0 * math::sqrt(zi * zj) * (zi + r * zj) * math::powf(r, 1.5)
                / (math::powi(1.0 - r * r, 2)
                    + 4.0 * zi * zj * r * (1.0 + r * r)
                    + 4.0 * (zi * zi + zj * zj) * r * r);
            c.close("ρij manual WDKB", cqc_correlation(wi, wj, zi, zj)?, rho)?;
            c.close("ρji simétrica", cqc_correlation(wj, wi, zj, zi)?, rho)?;
            c.close("ρii=1", cqc_correlation(wi, wi, zi, zi)?, 1.0)?;
            let actual = combine_modal(&[a, b], &[wi, wj], &[zi, zj], ModalCombination::Cqc)?;
            c.close(
                "CQC firmada manual",
                actual,
                math::sqrt(a * a + b * b + 2.0 * rho * a * b),
            )?;
            if wj / wi >= 100.0 {
                let srss = combine_modal(&[a, b], &[wi, wj], &[zi, zj], ModalCombination::Srss)?;
                Check(Tolerance {
                    absolute: 1e-10,
                    relative: 2e-5,
                    justification:
                        "r=100, ζ=.05: ρ<2.1e-5; CQC tiende a SRSS, no igualdad exacta con ζ>0.",
                })
                .close("CQC≈SRSS separados", actual, srss)?;
                c.close(
                    "CQC=SRSS separados sin amortiguamiento",
                    combine_modal(&[a, b], &[wi, wj], &[0.0, 0.0], ModalCombination::Cqc)?,
                    srss,
                )?;
            }
            if wi == wj {
                c.close("modos coincidentes ρ=1", actual, (a + b).abs())?;
            }
            Ok(())
        }
        Parameters::Directional {
            mass,
            stiffness,
            srss,
            factors,
            accelerations,
        } => {
            let mut model = Model::new();
            for id in 1..=2 {
                let mut node = Node::new(NodeId(id), Dim::D2, [id as f64, 0.0, 0.0]);
                node.dofs = vec![Dof::Ux, Dof::Uy];
                model.nodes.push(node);
            }
            support(&mut model, 1, &[Dof::Ux, Dof::Uy]);
            model.masses.push(NodalMass {
                id: "m".into(),
                node: NodeId(2),
                values: vec![
                    DofValue {
                        dof: Dof::Ux,
                        value: mass,
                    },
                    DofValue {
                        dof: Dof::Uy,
                        value: mass,
                    },
                ],
            });
            let map = DofMap::new(&model.nodes)?;
            let mut location = Vec::new();
            for id in 1..=2 {
                for d in [Dof::Ux, Dof::Uy] {
                    location.push(map.global_index(NodeId(id), d)?);
                }
            }
            let [kx, ky] = stiffness;
            let entry = ElementContribution {
                location,
                stiffness: DMat::new(
                    4,
                    4,
                    vec![
                        kx, 0.0, -kx, 0.0, 0.0, ky, 0.0, -ky, -kx, 0.0, kx, 0.0, 0.0, -ky, 0.0, ky,
                    ],
                )?,
                mass_lumped: None,
                mass_consistent: None,
                geometric: None,
                loads: vec![],
            };
            let mut s = LinearSystem::from_model(ReferenceBackend, &model, &[entry], &[])?;
            let sx = [spectrum_table(
                &[(0.0, accelerations[0]), (10.0, accelerations[0])],
                0.05,
                Interpolation::Linear,
            )?];
            let sy = [spectrum_table(
                &[(0.0, accelerations[1]), (10.0, accelerations[1])],
                0.05,
                Interpolation::Linear,
            )?];
            let r = Spectrum::run(
                &mut s,
                &[
                    SpectrumDirection {
                        direction: [1.0, 0.0, 0.0],
                        spectra: &sx,
                    },
                    SpectrumDirection {
                        direction: [0.0, 1.0, 0.0],
                        spectra: &sy,
                    },
                ],
                &spectrum_options(
                    if srss {
                        DirectionalCombination::Srss
                    } else {
                        DirectionalCombination::WeightedSum
                    },
                    factors.to_vec(),
                ),
            )?;
            let [x, y] = [mass * accelerations[0], mass * accelerations[1]];
            c.close("Vx=m Sa,x", r.directions[0].combined.base_shear, x)?;
            c.close("Vy=m Sa,y", r.directions[1].combined.base_shear, y)?;
            c.close(
                "factores direccionales del usuario",
                r.combined.base_shear,
                if srss {
                    math::sqrt(math::powi(factors[0] * x, 2) + math::powi(factors[1] * y, 2))
                } else {
                    factors[0] * x + factors[1] * y
                },
            )?;
            c.close(
                "Ux combinado",
                r.combined.global_displacements[2],
                factors[0] * accelerations[0] * mass / kx,
            )?;
            c.close(
                "Uy combinado",
                r.combined.global_displacements[3],
                factors[1] * accelerations[1] * mass / ky,
            )
        }
        _ => Err(missing()),
    }
}
#[test]
fn suite_modal_analitica() {
    assert_eq!(MODAL.len(), 12);
    run_group(MODAL, validate_modal);
}
#[test]
fn suite_pdelta_analitica() {
    assert_eq!(PDELTA.len(), 10);
    run_group(PDELTA, validate_pdelta);
}
#[test]
fn suite_espectro_analitico() {
    assert_eq!(SPECTRUM.len(), 9);
    run_group(SPECTRUM, validate_spectrum);
}
