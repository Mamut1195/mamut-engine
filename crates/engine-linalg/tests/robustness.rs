//! Robustez ante dimensiones y magnitudes extremas (hallazgos J4 de ENG-LA-007).
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SparseMatrix, SymMatrix, Triplet};

fn dmat(rows: usize, cols: usize, v: &[f64]) -> DMat {
    DMat::new(rows, cols, v.to_vec()).unwrap()
}

#[test]
fn dense_extreme_dimensions_return_eng_e_0010_without_panic() {
    for (r, c) in [
        (1, usize::MAX),
        (1, usize::MAX / 8),
        (usize::MAX / 8, 1),
        (usize::MAX, usize::MAX),
        (1 << 20, 1 << 20),
    ] {
        let e = DMat::zeros(r, c).unwrap_err();
        assert_eq!(e.code, "ENG-E-0010", "{r}x{c}");
        assert!(!e.message.is_empty());
    }
    assert!(DMat::zeros(0, usize::MAX).is_ok());
    assert!(DMat::zeros(3, 4).is_ok());
}

/// Filas que superan el límite de reserva por destino (`1 << 40` no compila en wasm32).
#[cfg(target_pointer_width = "64")]
const HUGE_ROWS: usize = 1usize << 40;
/// 2^29 filas × 4 B = 2 GiB > límite de 1 GiB en 32 bits.
#[cfg(not(target_pointer_width = "64"))]
const HUGE_ROWS: usize = 1usize << 29;

#[test]
fn sparse_extreme_dimensions_return_eng_e_0010_without_panic() {
    for rows in [usize::MAX, usize::MAX - 1, usize::MAX / 8, HUGE_ROWS] {
        let e = SparseMatrix::from_triplets(rows, 1, &[]).unwrap_err();
        assert_eq!(e.code, "ENG-E-0010", "rows {rows}");
    }
    let t = [Triplet {
        row: 0,
        col: 0,
        value: 1.0,
    }];
    assert!(SparseMatrix::from_triplets(2, usize::MAX, &t).is_ok());
    // Densificar una matriz con columnas enormes también falla con error.
    let s = SparseMatrix::from_triplets(2, usize::MAX / 8, &t).unwrap();
    assert_eq!(s.to_dense().unwrap_err().code, "ENG-E-0010");
}

#[test]
fn solve_dense_rhs_near_f64_max_is_finite() {
    let a = dmat(2, 2, &[0.5, 0.5, 0.5, -0.5]);
    let b = dmat(2, 1, &[1e308, 0.0]);
    let x = ReferenceBackend.solve_dense(&a, &b).unwrap();
    for v in x.values() {
        assert!((v / 1e308 - 1.0).abs() < 1e-12, "{v}");
    }
}

#[test]
fn solve_dense_columns_scale_independently() {
    let a = dmat(2, 2, &[2.0, 0.0, 0.0, 4.0]);
    let b = dmat(2, 2, &[2e300, 2.0, 4e300, 4.0]);
    let x = ReferenceBackend.solve_dense(&a, &b).unwrap();
    assert!((x.values()[0] / 1e300 - 1.0).abs() < 1e-12);
    assert!((x.values()[1] - 1.0).abs() < 1e-12);
    assert!((x.values()[2] / 1e300 - 1.0).abs() < 1e-12);
    assert!((x.values()[3] - 1.0).abs() < 1e-12);
}

#[test]
fn unrepresentable_solution_reports_eng_e_0008() {
    let a = dmat(1, 1, &[1e-300]);
    let b = dmat(1, 1, &[1e300]);
    assert_eq!(
        ReferenceBackend.solve_dense(&a, &b).unwrap_err().code,
        "ENG-E-0008"
    );
}

#[test]
fn ldlt_block_2x2_keeps_subnormal_rhs() {
    // diag(1, [[0, 1e-10], [1e-10, 0]]), b = [0, 1e-320, 0].
    let k = SymMatrix::new(dmat(
        3,
        3,
        &[1.0, 0.0, 0.0, 0.0, 0.0, 1e-10, 0.0, 1e-10, 0.0],
    ))
    .unwrap();
    let f = ReferenceBackend.factorize(&k).unwrap();
    let mut b = dmat(3, 1, &[0.0, 1e-320, 0.0]);
    f.solve_in_place(&mut b).unwrap();
    let x = b.values();
    assert_eq!(x[0], 0.0);
    assert_eq!(x[1], 0.0);
    assert!(
        (x[2] / 1e-310 - 1.0).abs() < 1e-3, /* 1e-320 es subnormal: ~0,05 % de resolución */
        "{}",
        x[2]
    );
}

#[test]
fn ldlt_rhs_near_f64_max_is_finite() {
    let k = SymMatrix::new(dmat(2, 2, &[0.5, 0.5, 0.5, -0.5])).unwrap();
    let f = ReferenceBackend.factorize(&k).unwrap();
    let mut b = dmat(2, 1, &[1e308, 0.0]);
    f.solve_in_place(&mut b).unwrap();
    for v in b.values() {
        assert!((v / 1e308 - 1.0).abs() < 1e-12, "{v}");
    }
}

fn mass_error(diag: &[f64]) -> (String, String) {
    let n = diag.len();
    let mut v = vec![0.0; n * n];
    for (i, d) in diag.iter().enumerate() {
        v[i * n + i] = *d;
    }
    let k = SymMatrix::new(
        DMat::new(n, n, {
            let mut e = vec![0.0; n * n];
            for i in 0..n {
                e[i * n + i] = 1.0;
            }
            e
        })
        .unwrap(),
    )
    .unwrap();
    let m = SymMatrix::new(DMat::new(n, n, v).unwrap()).unwrap();
    let e = ReferenceBackend.generalized_eigen(&k, &m).unwrap_err();
    (e.code.to_string(), e.message.clone())
}

#[test]
fn singular_mass_error_is_independent_of_dof_order() {
    // Singular con diagonal negativa: mismo código en cualquier permutación,
    // y el GDL culpable es el nulo (índice original).
    let (c1, m1) = mass_error(&[-1.0, 0.0]);
    let (c2, m2) = mass_error(&[0.0, -1.0]);
    assert_eq!(c1, "ENG-E-0006");
    assert_eq!(c1, c2);
    assert!(m1.contains("GDL 1"), "{m1}");
    assert!(m2.contains("GDL 0"), "{m2}");
    // Todas las permutaciones de {1, 0, -1}: siempre singular, GDL nulo.
    let base = [1.0, 0.0, -1.0];
    for p in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let d: Vec<f64> = p.iter().map(|&i| base[i]).collect();
        let zero = d.iter().position(|&x| x == 0.0).unwrap();
        let (c, m) = mass_error(&d);
        assert_eq!(c, "ENG-E-0006", "{d:?}");
        assert!(m.contains(&format!("GDL {zero}")), "{d:?}: {m}");
    }
    // Varios GDL nulos: se informa el de menor índice, sea cual sea el orden.
    for d in [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]] {
        let first = d.iter().position(|&x| x == 0.0).unwrap();
        let (c, m) = mass_error(&d);
        assert_eq!(c, "ENG-E-0006");
        assert!(m.contains(&format!("GDL {first}")), "{d:?}: {m}");
    }
    // Indefinida pero regular: ENG-E-0010 en cualquier orden.
    assert_eq!(mass_error(&[-1.0, 1.0]).0, "ENG-E-0010");
    assert_eq!(mass_error(&[1.0, -1.0]).0, "ENG-E-0010");
}

fn sym(n: usize, v: &[f64]) -> SymMatrix {
    SymMatrix::new(dmat(n, n, v)).unwrap()
}

#[test]
fn pivot_threshold_is_invariant_to_diagonal_scaling() {
    // T5 (J4): matrices definidas positivas con GDL de escalas muy distintas
    // no son singulares; el veredicto no depende de D·A·D.
    for (d0, d1) in [
        (1e6, 1e-9),
        (1e8, 1e-6),
        (1e-9, 1e6),
        (1e12, 1e-12),
        (1.0, 1.0),
    ] {
        let m = sym(2, &[d0, 0.0, 0.0, d1]);
        let f = ReferenceBackend.factorize(&m).unwrap();
        let mut rhs = dmat(2, 1, &[d0, d1]);
        f.solve_in_place(&mut rhs).unwrap();
        for v in rhs.values() {
            assert!((v - 1.0).abs() < 1e-12, "{d0} {d1}: {v}");
        }
        let k = sym(2, &[2.0 * d0, 0.0, 0.0, 3.0 * d1]);
        let e = ReferenceBackend.generalized_eigen(&k, &m).unwrap();
        assert!((e.eigenvalues[0] - 2.0).abs() < 1e-12 && (e.eigenvalues[1] - 3.0).abs() < 1e-12);
    }
    // Acoplada: D A D de una A bien condicionada se factoriza y resuelve igual.
    let base = [4.0, 1.0, 1.0, 3.0];
    let x_ref = [1.0, 2.0];
    for (s0, s1) in [(1e4, 1e-5), (1e-6, 1e6), (1.0, 1.0)] {
        let a = [
            base[0] * s0 * s0,
            base[1] * s0 * s1,
            base[2] * s0 * s1,
            base[3] * s1 * s1,
        ];
        let f = ReferenceBackend.factorize(&sym(2, &a)).unwrap();
        // A_s x_s = b_s con x_s = x_ref/s: b = A_s x_s.
        let xs = [x_ref[0] / s0, x_ref[1] / s1];
        let mut b = dmat(
            2,
            1,
            &[a[0] * xs[0] + a[1] * xs[1], a[2] * xs[0] + a[3] * xs[1]],
        );
        f.solve_in_place(&mut b).unwrap();
        assert!((b.values()[0] / xs[0] - 1.0).abs() < 1e-9, "{s0} {s1}");
        assert!((b.values()[1] / xs[1] - 1.0).abs() < 1e-9, "{s0} {s1}");
    }
}

#[test]
fn scaling_does_not_hide_true_singularity() {
    // Singular de verdad (filas proporcionales) sigue dando ENG-E-0006 a
    // cualquier escala diagonal; nula también.
    for (s0, s1) in [(1.0, 1.0), (1e6, 1e-6), (1e-6, 1e6)] {
        let a = [s0 * s0, s0 * s1, s0 * s1, s1 * s1];
        let e = ReferenceBackend.factorize(&sym(2, &a)).err().unwrap();
        assert_eq!(e.code, "ENG-E-0006", "{s0} {s1}");
    }
    assert_eq!(
        ReferenceBackend
            .factorize(&sym(2, &[1e6, 0.0, 0.0, 0.0]))
            .err()
            .unwrap()
            .code,
        "ENG-E-0006"
    );
    assert_eq!(
        ReferenceBackend
            .generalized_eigen(
                &sym(2, &[1.0, 0.0, 0.0, 1.0]),
                &sym(2, &[1e6, 0.0, 0.0, 0.0])
            )
            .err()
            .unwrap()
            .code,
        "ENG-E-0006"
    );
}

#[test]
fn equilibrated_ldlt_underflow_case_is_accurate_or_rejected() {
    // J4 r3 (a): A=[[1e-200,1e-200],[1e-200,1]], b=[0,7.41e-224]. x1 ≈ 7.41e-224 y
    // x0 = −x1 (de la fila 0). Antes (equilibrado sin cuidado) x0 salía −9.88e-224
    // (33 % de error) y en main daba ENG-E-0006.
    let a = sym(2, &[1e-200, 1e-200, 1e-200, 1.0]);
    for b1 in [7.41e-224, 1e-224, 3.0e-100] {
        let b = dmat(2, 1, &[0.0, b1]);
        match ReferenceBackend.factorize(&a) {
            Err(e) => assert_eq!(e.code, "ENG-E-0006"),
            Ok(f) => {
                let mut x = b.clone();
                match f.solve_in_place(&mut x) {
                    Err(e) => assert!(["ENG-E-0006", "ENG-E-0008"].contains(&e.code), "{e}"),
                    Ok(()) => {
                        let (x0, x1) = (x.values()[0], x.values()[1]);
                        // Fila 0: x0 = −x1; fila 1: x1(1−1e-200) = b1 ⇒ x1 ≈ b1.
                        let want1 = b1 / (1.0 - 1e-200);
                        assert!((x1 / want1 - 1.0).abs() < 1e-10, "{b1}: x1 {x1} vs {want1}");
                        assert!(
                            (x0 / -want1 - 1.0).abs() < 1e-9,
                            "{b1}: x0 {x0} vs {}",
                            -want1
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn equilibrated_ldlt_rhs_near_f64_max_keeps_working() {
    // J4 r3 (b): A=[[.5,.5],[.5,-.5]], b=[1.3e308, 0] devolvía ENG-E-0008
    // (b·s desbordaba antes de normalizar); la solución es [1.3e308, 1.3e308].
    let a = sym(2, &[0.5, 0.5, 0.5, -0.5]);
    let f = ReferenceBackend.factorize(&a).unwrap();
    let mut x = dmat(2, 1, &[1.3e308, 0.0]);
    f.solve_in_place(&mut x).unwrap();
    for v in x.values() {
        assert!((v / 1.3e308 - 1.0).abs() < 1e-12, "{v}");
    }
}

#[test]
fn equilibration_improves_coupled_scaling_without_promising_invariance() {
    // J4 r3 (c): A acoplada y D·A·D. Se acepta en un rango amplio de D (Ruiz
    // iterativo), sin prometer invariancia exacta fuera de él.
    let base = [2.0, 1.9, 0.0, 1.9, 2.0, 1.0, 0.0, 1.0, 1.001];
    for e in [1.0, 1e2, 1e4, 1e6, 1e8] {
        let d = [e, 1.0 / e, 1.0];
        let a: Vec<f64> = (0..9).map(|k| base[k] * (d[k / 3] * d[k % 3])).collect();
        let f = ReferenceBackend.factorize(&sym(3, &a));
        assert!(f.is_ok(), "e = {e}: {:?}", f.err());
    }
    let two = [1.0, 0.9, 0.9, 1.0];
    for e in [1.0, 1e6, 1e10, 1e14] {
        let d = [e, 1.0];
        let a: Vec<f64> = (0..4).map(|k| two[k] * (d[k / 2] * d[k % 2])).collect();
        assert!(ReferenceBackend.factorize(&sym(2, &a)).is_ok(), "e = {e}");
    }
}

#[test]
fn generalized_eigen_with_extreme_mass_scaling() {
    // K=diag(1e6,1e3), M=diag(1e8,1e-6): λ = [1e6/1e8, 1e3/1e-6] = [0.01, 1e9].
    let k = sym(2, &[1e6, 0.0, 0.0, 1e3]);
    let m = sym(2, &[1e8, 0.0, 0.0, 1e-6]);
    let e = ReferenceBackend.generalized_eigen(&k, &m).unwrap();
    assert!((e.eigenvalues[0] / 0.01 - 1.0).abs() < 1e-12);
    assert!((e.eigenvalues[1] / 1e9 - 1.0).abs() < 1e-12);
}

#[test]
fn empty_factorization_solve_with_huge_rhs_columns_terminates() {
    // Preexistente (J4): factorize(0×0) + solve_in_place(B 0×usize::MAX) no terminaba.
    let a = SymMatrix::new(DMat::zeros(0, 0).unwrap()).unwrap();
    let f = ReferenceBackend.factorize(&a).unwrap();
    let mut b = DMat::zeros(0, usize::MAX).unwrap();
    f.solve_in_place(&mut b).unwrap();
    assert_eq!((b.rows(), b.cols()), (0, usize::MAX));
    // Filas incompatibles siguen siendo error.
    let mut bad = DMat::zeros(1, 3).unwrap();
    assert!(f.solve_in_place(&mut bad).is_err());
}

#[test]
fn rhs_spanning_more_than_1e300_loses_the_small_component_documented_limit() {
    // Documenta el límite de #36: b=[1e300, 1e-30] → el componente pequeño se pierde
    // (ver doc de LinAlgBackend). La solución grande sigue siendo correcta.
    let a = SymMatrix::new(dmat(2, 2, &[2.0, 0.0, 0.0, 2.0])).unwrap();
    let f = ReferenceBackend.factorize(&a).unwrap();
    let mut b = dmat(2, 1, &[1e300, 1e-30]);
    f.solve_in_place(&mut b).unwrap();
    assert_eq!(b.values()[0], 5e299);
    assert!(b.values()[1] == 0.0 || (b.values()[1] - 5e-31).abs() <= 5e-31);
}
