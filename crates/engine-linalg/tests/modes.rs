//! Independently derived closed-form solutions: spring chain and matrices with known spectrum.
use engine_core::math;
use engine_core::EResult;
use engine_linalg::{
    lowest_modes, sturm_count, DMat, GeneralizedEigen, LinAlgBackend, LowestModes,
    ReferenceBackend, SymFactorization, SymMatrix,
};

fn symmetric(n: usize, values: Vec<f64>) -> SymMatrix {
    SymMatrix::new(DMat::new(n, n, values).unwrap()).unwrap()
}

fn diagonal(values: &[f64]) -> SymMatrix {
    let n = values.len();
    let mut entries = vec![0.0; n * n];
    for (i, &value) in values.iter().enumerate() {
        entries[i * n + i] = value;
    }
    symmetric(n, entries)
}

fn chain(n: usize, spring: f64, mass: f64) -> (SymMatrix, SymMatrix, Vec<f64>) {
    let mut k = vec![0.0; n * n];
    for i in 0..n {
        k[i * n + i] = 2.0 * spring;
        if i > 0 {
            k[i * n + i - 1] = -spring;
            k[(i - 1) * n + i] = -spring;
        }
    }
    let exact = (1..=n)
        .map(|mode| {
            let angle = mode as f64 * std::f64::consts::PI / (2.0 * (n + 1) as f64);
            4.0 * spring / mass * math::powi(math::sin(angle), 2)
        })
        .collect();
    (symmetric(n, k), diagonal(&vec![mass; n]), exact)
}

fn check_modes(k: &SymMatrix, m: &SymMatrix, modes: &LowestModes) {
    let n = k.order();
    let cols = modes.eigenvalues.len();
    assert_eq!((modes.vectors.rows(), modes.vectors.cols()), (n, cols));
    for col in 0..cols {
        let lambda = modes.eigenvalues[col];
        let mut residual = 0.0;
        let mut magnitude = 0.0;
        for row in 0..n {
            let kv: f64 = (0..n)
                .map(|j| k.dense().values()[row * n + j] * modes.vectors.values()[j * cols + col])
                .sum();
            let mv: f64 = (0..n)
                .map(|j| m.dense().values()[row * n + j] * modes.vectors.values()[j * cols + col])
                .sum();
            residual += math::powi(kv - lambda * mv, 2);
            magnitude += math::powi(kv, 2) + math::powi(lambda * mv, 2);
        }
        assert!(math::sqrt(residual) <= 2e-11 * math::sqrt(magnitude).max(1.0));
        for other in 0..cols {
            let mut dot = 0.0;
            for i in 0..n {
                for j in 0..n {
                    dot += modes.vectors.values()[i * cols + col]
                        * m.dense().values()[i * n + j]
                        * modes.vectors.values()[j * cols + other];
                }
            }
            let expected = if other == col { 1.0 } else { 0.0 };
            assert!((dot - expected).abs() < 2e-12, "M-orthogonality: {dot}");
        }
    }
}

#[test]
fn fixed_end_spring_chain_matches_closed_form_and_sturm() {
    for n in [1, 4, 9] {
        let (k, m, exact) = chain(n, 17.0, 2.3);
        let mut backend = ReferenceBackend;
        let modes = lowest_modes(&mut backend, &k, &m, n).unwrap();
        for (&value, &expected) in modes.eigenvalues.iter().zip(&exact) {
            assert!((value / expected - 1.0).abs() < 2e-12);
        }
        check_modes(&k, &m, &modes);
        assert_eq!(modes.rigid_mode_count, 0);
        assert_eq!(sturm_count(&mut backend, &k, &m, -1.0).unwrap(), 0);
        for count in 0..=n {
            let sigma = if count == 0 {
                exact[0] * 0.5
            } else if count == n {
                exact[n - 1] * 1.1
            } else {
                (exact[count - 1] + exact[count]) * 0.5
            };
            assert_eq!(sturm_count(&mut backend, &k, &m, sigma).unwrap(), count);
            let subset = backend.lowest_modes(&k, &m, count).unwrap();
            assert_eq!(subset.eigenvalues.len(), count);
            check_modes(&k, &m, &subset);
        }
    }
}

#[test]
fn modal_kernel_control_interrupts_every_checkpoint_without_changing_bits() {
    let (k, m, _) = chain(4, 17.0, 2.3);
    let expected = ReferenceBackend.lowest_modes(&k, &m, 4).unwrap();
    let mut checks = 0;
    let actual = ReferenceBackend
        .lowest_modes_with_check(&k, &m, 4, &mut || {
            checks += 1;
            Ok(())
        })
        .unwrap();
    assert!(checks > 20, "Must check inside the kernels: {checks}");
    assert_eq!(
        actual
            .eigenvalues
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        expected
            .eigenvalues
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        actual
            .vectors
            .values()
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        expected
            .vectors
            .values()
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>()
    );
    for stop in 1..=checks {
        let mut calls = 0;
        let error = ReferenceBackend
            .lowest_modes_with_check(&k, &m, 4, &mut || {
                calls += 1;
                if calls == stop {
                    Err(engine_core::EngineError::new("TEST-CANCEL", "Cancelado"))
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert_eq!(error.code, "TEST-CANCEL", "Checkpoint {stop}");
        assert_eq!(calls, stop);
    }
}

#[test]
fn strict_sturm_at_roots_and_two_by_two_pivot() {
    let m = diagonal(&[1.0; 4]);
    let k = diagonal(&[0.0, 2.0, 2.0, 5.0]);
    for (sigma, expected) in [
        (-1.0, 0),
        (0.0, 0),
        (1.0, 1),
        (2.0, 1),
        (3.0, 3),
        (5.0, 3),
        (6.0, 4),
    ] {
        assert_eq!(
            sturm_count(&mut ReferenceBackend, &k, &m, sigma).unwrap(),
            expected
        );
    }
    // Zero diagonals require the 2×2 block: λ=±2, det<0, tr=0.
    let k = symmetric(2, vec![0.0, 2.0, 2.0, 0.0]);
    let m = diagonal(&[1.0, 1.0]);
    assert_eq!(ReferenceBackend.symmetric_inertia(&k).unwrap(), (1, 1, 0));
    for (sigma, expected) in [(-3.0, 0), (-2.0, 0), (0.0, 1), (2.0, 1), (3.0, 2)] {
        assert_eq!(
            sturm_count(&mut ReferenceBackend, &k, &m, sigma).unwrap(),
            expected
        );
    }
    // Negative and positive definite inertia (trace signs with det>0).
    for (diag, inertia) in [(-3.0, (0, 2, 0)), (3.0, (2, 0, 0))] {
        let k = symmetric(2, vec![diag, 2.0, 2.0, diag]);
        assert_eq!(ReferenceBackend.symmetric_inertia(&k).unwrap(), inertia);
    }
}

#[test]
fn mixed_block_inertia_preserves_null_modes_and_factor_contract() {
    // Indefinite 2×2 block with nonzero trace, one negative and one zero mode.
    // Permutations require sharing the count between 1×1/2×2 pivots.
    let entries = [
        0.5, 2.0, 0.0, 0.0, 2.0, -0.25, 0.0, 0.0, 0.0, 0.0, -3.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];
    for order in [[0, 1, 2, 3], [3, 2, 1, 0], [2, 0, 3, 1]] {
        for scale in [1e-200, 1.0, 1e200] {
            let mut values = vec![0.0; 16];
            for i in 0..4 {
                for j in 0..4 {
                    values[4 * i + j] = scale * entries[4 * order[i] + order[j]];
                }
            }
            let k = symmetric(4, values);
            let mut backend = ReferenceBackend;
            assert_eq!(backend.symmetric_inertia(&k).unwrap(), (1, 2, 1));
            assert_eq!(
                sturm_count(&mut backend, &k, &diagonal(&[1.0; 4]), 0.0).unwrap(),
                2
            );
            assert_eq!(backend.factorize(&k).err().unwrap().code, "ENG-E-0006");
        }
    }
    // Without the zero mode, FactorInfo and the Sturm query agree.
    let k = symmetric(3, vec![0.5, 2.0, 0.0, 2.0, -0.25, 0.0, 0.0, 0.0, -3.0]);
    let factor = ReferenceBackend.factorize(&k).unwrap();
    assert_eq!(factor.info().inertia, Some((1, 2, 0)));
    assert_eq!(ReferenceBackend.symmetric_inertia(&k).unwrap(), (1, 2, 0));
    let ratio = factor.info().min_pivot_ratio.unwrap();
    assert!(ratio > 0.0 && ratio <= 1.0);
}

#[test]
fn rigid_modes_are_returned_and_negative_stiffness_is_diagnosed() {
    let k = symmetric(2, vec![3.0, -3.0, -3.0, 3.0]);
    let m = diagonal(&[2.0, 2.0]);
    let modes = ReferenceBackend.lowest_modes(&k, &m, 1).unwrap();
    assert_eq!(modes.eigenvalues, vec![0.0]);
    assert_eq!(modes.rigid_mode_count, 1);
    check_modes(&k, &m, &modes);
    let modes = ReferenceBackend.lowest_modes(&k, &m, 2).unwrap();
    assert!((modes.eigenvalues[1] - 3.0).abs() < 1e-12);
    check_modes(&k, &m, &modes);
    let null = SymMatrix::zeros(2).unwrap();
    let modes = ReferenceBackend.lowest_modes(&null, &m, 2).unwrap();
    assert_eq!(modes.rigid_mode_count, 2);
    check_modes(&null, &m, &modes);
    assert_eq!(
        ReferenceBackend
            .lowest_modes(&null, &m, 1)
            .unwrap_err()
            .code,
        "ENG-E-0007"
    );
    let error = ReferenceBackend
        .lowest_modes(&diagonal(&[-1.0, 3.0]), &m, 1)
        .unwrap_err();
    assert_eq!(error.code, "ENG-E-0010");
    assert!(error.message.contains("negative"));
    // Singular inertia can be queried, but not solved as a factor.
    assert_eq!(ReferenceBackend.symmetric_inertia(&k).unwrap(), (1, 0, 1));
    assert_eq!(
        ReferenceBackend.factorize(&k).err().unwrap().code,
        "ENG-E-0006"
    );
}

#[test]
fn cutting_repeated_eigenvalues_reports_missing_modes() {
    let k = diagonal(&[1.0, 2.0, 2.0, 4.0]);
    let m = diagonal(&[1.0; 4]);
    let error = ReferenceBackend.lowest_modes(&k, &m, 2).unwrap_err();
    assert_eq!(error.code, "ENG-E-0007");
    assert!(error.message.contains("MISSING MODES"));
    assert!(error.message.contains("=3"));
    for count in [1, 3, 4] {
        let modes = ReferenceBackend.lowest_modes(&k, &m, count).unwrap();
        check_modes(&k, &m, &modes);
    }
}

#[test]
fn mass_scaling_preserves_shapes_and_scales_eigenvalues() {
    let (k, m, _) = chain(7, 13.0, 3.0);
    let original = ReferenceBackend.lowest_modes(&k, &m, 3).unwrap();
    for scale in [1e-6_f64, 7.0, 1e6] {
        let scaled_m = symmetric(7, m.dense().values().iter().map(|v| v * scale).collect());
        let scaled = ReferenceBackend.lowest_modes(&k, &scaled_m, 3).unwrap();
        check_modes(&k, &scaled_m, &scaled);
        for (&a, &b) in original.eigenvalues.iter().zip(&scaled.eigenvalues) {
            assert!((b * scale / a - 1.0).abs() < 2e-12);
        }
        for (&a, &b) in original
            .vectors
            .values()
            .iter()
            .zip(scaled.vectors.values())
        {
            assert!((a - b * math::sqrt(scale)).abs() < 2e-12);
        }
    }
}

// The default method remains compatible with a backend that only
// implements the original methods. Omission and incomplete size are simulated.
struct FaultyBackend {
    truncate: bool,
}
impl LinAlgBackend for FaultyBackend {
    fn name(&self) -> &'static str {
        "test.faulty"
    }
    fn matmul(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        ReferenceBackend.matmul(a, b)
    }
    fn transpose(&mut self, a: &DMat) -> EResult<DMat> {
        ReferenceBackend.transpose(a)
    }
    fn solve_dense(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        ReferenceBackend.solve_dense(a, b)
    }
    fn factorize(&mut self, a: &SymMatrix) -> EResult<Box<dyn SymFactorization>> {
        ReferenceBackend.factorize(a)
    }
    fn generalized_eigen(&mut self, k: &SymMatrix, m: &SymMatrix) -> EResult<GeneralizedEigen> {
        let mut result = ReferenceBackend.generalized_eigen(k, m)?;
        if self.truncate {
            result.eigenvalues.pop();
        } else {
            result.eigenvalues.rotate_left(1);
            let n = result.eigenvalues.len();
            result.eigenvalues[n - 1] = result.eigenvalues[n - 2];
        }
        Ok(result)
    }
}

#[test]
fn incomplete_backend_is_rejected_by_size_and_independent_sturm_count() {
    let k = diagonal(&[1.0, 2.0, 3.0, 4.0]);
    let m = diagonal(&[1.0; 4]);
    for truncate in [false, true] {
        let mut backend = FaultyBackend { truncate };
        let backend: &mut dyn LinAlgBackend = &mut backend;
        let error = backend.lowest_modes(&k, &m, 2).unwrap_err();
        assert_eq!(error.code, "ENG-E-0007");
        assert!(error.message.contains("MISSING MODES"));
    }
}

#[test]
fn invalid_inputs_and_empty_requests_use_catalog_errors() {
    let k = diagonal(&[1.0, 2.0]);
    let m = diagonal(&[1.0, 1.0]);
    assert_eq!(
        ReferenceBackend.lowest_modes(&k, &m, 3).unwrap_err().code,
        "ENG-E-0010"
    );
    assert_eq!(
        ReferenceBackend
            .lowest_modes(&k, &diagonal(&[1.0]), 0)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    for (mass, code) in [
        (diagonal(&[0.0, 1.0]), "ENG-E-0006"),
        (diagonal(&[-1.0, 1.0]), "ENG-E-0010"),
    ] {
        assert_eq!(
            ReferenceBackend
                .lowest_modes(&k, &mass, 1)
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(
            sturm_count(&mut ReferenceBackend, &k, &mass, 1.5)
                .unwrap_err()
                .code,
            code
        );
    }
    for sigma in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            sturm_count(&mut ReferenceBackend, &k, &m, sigma)
                .unwrap_err()
                .code,
            "ENG-E-0008"
        );
    }
    let huge = diagonal(&[f64::MAX, 1.0]);
    assert_eq!(
        sturm_count(&mut ReferenceBackend, &huge, &huge, -2.0)
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    assert_eq!(
        DMat::new(1, 1, vec![f64::NAN]).unwrap_err().code,
        "ENG-E-0008"
    );
    assert_eq!(
        SymMatrix::new(DMat::new(2, 2, vec![1.0, 1.0, 0.0, 1.0]).unwrap())
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    let empty = ReferenceBackend
        .lowest_modes(&k, &SymMatrix::zeros(2).unwrap(), 0)
        .unwrap();
    assert!(empty.eigenvalues.is_empty());
    assert_eq!((empty.vectors.rows(), empty.vectors.cols()), (2, 0));
    let empty_matrix = SymMatrix::zeros(0).unwrap();
    assert!(ReferenceBackend
        .lowest_modes(&empty_matrix, &empty_matrix, 0)
        .unwrap()
        .eigenvalues
        .is_empty());
    assert_eq!(
        sturm_count(&mut ReferenceBackend, &empty_matrix, &empty_matrix, 0.0).unwrap(),
        0
    );
}
