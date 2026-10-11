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
