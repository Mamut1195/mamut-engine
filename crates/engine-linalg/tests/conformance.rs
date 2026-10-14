//! Valores esperados derivados de identidades y soluciones cerradas propias.
use engine_core::math;
use engine_core::{DofIndex, EngineError, EntityRef};
use engine_linalg::*;

fn mat(rows: usize, cols: usize, values: &[f64]) -> DMat {
    DMat::new(rows, cols, values.to_vec()).unwrap()
}
fn sym(n: usize, values: &[f64]) -> SymMatrix {
    SymMatrix::new(mat(n, n, values)).unwrap()
}
fn identity(n: usize) -> DMat {
    mat(
        n,
        n,
        &(0..n * n)
            .map(|p| if p / n == p % n { 1.0 } else { 0.0 })
            .collect::<Vec<_>>(),
    )
}
fn close(actual: f64, expected: f64, tol: f64) {
    assert!(
        (actual - expected).abs() < tol,
        "{actual} != {expected}, tol {tol}"
    );
}
fn residual(a: &DMat, x: &DMat, b: &DMat, tol: f64) {
    for col in 0..b.cols() {
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for i in 0..a.rows() {
            let ax = (0..a.cols())
                .map(|j| a.get(i, j).unwrap() * x.get(j, col).unwrap())
                .sum::<f64>();
            numerator += math::powi(ax - b.get(i, col).unwrap(), 2);
            denominator += math::powi(b.get(i, col).unwrap(), 2);
        }
        // RHS nula: norma absoluta; las cargas no nulas usan norma euclídea relativa.
        let error = if denominator == 0.0 {
            math::sqrt(numerator)
        } else {
            math::sqrt(numerator / denominator)
        };
        assert!(error < tol, "residual {error} >= {tol}");
    }
}
fn springs(n: usize) -> SymMatrix {
    sym(
        n,
        &(0..n * n)
            .map(|p| {
                let (i, j) = (p / n, p % n);
                if i == j {
                    2.0
                } else if i.abs_diff(j) == 1 {
                    -1.0
                } else {
                    0.0
                }
            })
            .collect::<Vec<_>>(),
    )
}
fn check_error<T>(result: Result<T, EngineError>, code: &str) -> EngineError {
    match result {
        Ok(_) => panic!("Se esperaba {code}"),
        Err(error) => {
            assert_eq!(error.code, code);
            assert!(!error.message.is_empty());
            error
        }
    }
}

/// Suite única, genérica: mismo contrato y tolerancias para cada backend.
fn conformidad(backend: &mut impl LinAlgBackend) {
    let a = mat(3, 3, &[4.0, 1.0, -1.0, 2.0, 7.0, 1.0, 0.0, -2.0, 5.0]);
    let inverse = backend.solve_dense(&a, &identity(3)).unwrap();
    let product = backend.matmul(&a, &inverse).unwrap();
    for (x, y) in product.values().iter().zip(identity(3).values()) {
        close(*x, *y, 1e-12);
    }
    residual(&a, &inverse, &identity(3), 1e-12);
    let rectangular = mat(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert_eq!(
        backend.transpose(&rectangular).unwrap(),
        mat(3, 2, &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0])
    );
    assert_eq!(
        backend
            .matmul(&rectangular, &mat(3, 1, &[1.0, 0.0, -1.0]))
            .unwrap(),
        mat(2, 1, &[-2.0, -2.0])
    );

    let entries = [
        Triplet {
            row: 1,
            col: 1,
            value: 1.0,
        },
        Triplet {
            row: 0,
            col: 1,
            value: -1.0,
        },
        Triplet {
            row: 0,
            col: 0,
            value: 2.0,
        },
        Triplet {
            row: 1,
            col: 0,
            value: -1.0,
        },
        Triplet {
            row: 1,
            col: 1,
            value: 1.0,
        },
    ];
    let assembled = backend.assemble(2, 2, &entries).unwrap();
    assert_eq!(assembled.row_offsets(), &[0, 2, 4]);
    assert_eq!(assembled.column_indices(), &[0, 1, 0, 1]);
    assert_eq!(assembled.values(), &[2.0, -1.0, -1.0, 2.0]);
    let pattern = assembled.symmetric_pattern().unwrap();
    assert_eq!(pattern.try_entries().unwrap(), &[true, true, true, true]);
    let dense = SymMatrix::new(assembled.to_dense().unwrap()).unwrap();
    residual(
        dense.dense(),
        &backend
            .factorize(&dense)
            .unwrap()
            .solve(&identity(2))
            .unwrap(),
        &identity(2),
        1e-12,
    );

    let k = springs(8);
    // -x_(i-1)+2x_i-x_(i+1)=1, x_0=x_(n+1)=0 ⇒ x_i=i(n+1-i)/2.
    let loads = mat(
        8,
        3,
        &(0..8).flat_map(|_| [1.0, 2.0, 0.0]).collect::<Vec<_>>(),
    );
    let factor = backend.factorize(&k).unwrap();
    assert_eq!(factor.info().inertia, Some((8, 0, 0)));
    assert!(factor.info().singular_dofs.is_empty());
    let solved = factor.solve(&loads).unwrap();
    residual(k.dense(), &solved, &loads, 1e-12);
    for i in 0..8 {
        let exact = ((i + 1) * (8 - i)) as f64 / 2.0;
        close(solved.get(i, 0).unwrap(), exact, 1e-12);
        close(solved.get(i, 1).unwrap(), 2.0 * exact, 1e-12);
        assert_eq!(solved.get(i, 2), Some(0.0));
    }
    let info = factor.info().clone();
    let mut second = identity(8);
    factor.solve_in_place(&mut second).unwrap();
    residual(k.dense(), &second, &identity(8), 1e-12);
    assert_eq!(&info, factor.info());
    let mut wrong = DMat::zeros(7, 2).unwrap();
    let original = wrong.clone();
    check_error(factor.solve_in_place(&mut wrong), "ENG-E-0010");
    assert_eq!(wrong, original);
    let mut overflow = mat(8, 1, &[f64::MAX; 8]);
    let original = overflow.clone();
    check_error(factor.solve_in_place(&mut overflow), "ENG-E-0008");
    assert_eq!(overflow, original);

    for (a, signs) in [
        (
            sym(3, &[2.0, 0.0, 0.0, 0.0, -3.0, 0.0, 0.0, 0.0, 4.0]),
            (2, 1, 0),
        ),
        (sym(2, &[0.0, 1.0, 1.0, 0.0]), (1, 1, 0)),
        // Congruencia A=L D Lᵀ, L=[[1,0,0],[1,1,0],[2,-1,1]], D=diag(2,-3,4).
        (
            sym(3, &[2.0, 2.0, 4.0, 2.0, -1.0, 7.0, 4.0, 7.0, 9.0]),
            (2, 1, 0),
        ),
        // Permutación que coloca el bloque 2×2 fuera de las primeras posiciones.
        (
            sym(3, &[0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 1.0, 0.0, 0.0]),
            (2, 1, 0),
        ),
    ] {
        let f = backend.factorize(&a).unwrap();
        assert_eq!(f.info().inertia, Some(signs));
        let ratio = f.info().min_pivot_ratio.unwrap();
        assert!(ratio > 0.0 && ratio <= 1.0);
        residual(
            a.dense(),
            &f.solve(&identity(a.order())).unwrap(),
            &identity(a.order()),
            1e-12,
        );
    }

    for mass in [
        SymMatrix::new(identity(8)).unwrap(),
        sym(
            8,
            &identity(8)
                .values()
                .iter()
                .map(|v| 2.0 * v)
                .collect::<Vec<_>>(),
        ),
    ] {
        let modes = backend.generalized_eigen(&k, &mass).unwrap();
        let mscale = mass.dense().get(0, 0).unwrap();
        for col in 0..8 {
            // Fórmula clásica de Toeplitz tridiagonal, derivación propia por seno
            // en provenance/ENG-LA-007.md; cita bibliográfica precisa PENDIENTE.
            let exact =
                (2.0 - 2.0 * math::cos((col + 1) as f64 * std::f64::consts::PI / 9.0)) / mscale;
            close(modes.eigenvalues[col], exact, 2e-12);
            let v = mat(
                8,
                1,
                &(0..8)
                    .map(|i| modes.vectors.get(i, col).unwrap())
                    .collect::<Vec<_>>(),
            );
            let mv = backend.matmul(mass.dense(), &v).unwrap();
            let b = mat(
                8,
                1,
                &mv.values()
                    .iter()
                    .map(|x| x * modes.eigenvalues[col])
                    .collect::<Vec<_>>(),
            );
            residual(k.dense(), &v, &b, 1e-12);
            for other in 0..8 {
                let norm = (0..8)
                    .map(|i| modes.vectors.get(i, other).unwrap() * mv.get(i, 0).unwrap())
                    .sum::<f64>();
                close(norm, if col == other { 1.0 } else { 0.0 }, 2e-12);
            }
        }
    }
    // Masa no diagonal: K=3M ⇒ λ=3 (multiplicidad), prueba por subespacio.
    let m = sym(2, &[2.0, 0.5, 0.5, 1.0]);
    let k = sym(2, &[6.0, 1.5, 1.5, 3.0]);
    let modes = backend.generalized_eigen(&k, &m).unwrap();
    for &value in &modes.eigenvalues {
        close(value, 3.0, 1e-12);
    }
    for col in 0..2 {
        for other in 0..2 {
            let norm = (0..2)
                .flat_map(|i| {
                    (0..2).map({
                        let modes = &modes;
                        let m = &m;
                        move |j| {
                            modes.vectors.get(i, col).unwrap()
                                * m.dense().get(i, j).unwrap()
                                * modes.vectors.get(j, other).unwrap()
                        }
                    })
                })
                .sum::<f64>();
            close(norm, if col == other { 1.0 } else { 0.0 }, 1e-12);
        }
    }

    // det(K-λM)=12-10λ+(7/4)λ² ⇒ λ=12/7 y 4.
    let k_mixed = sym(2, &[4.0, 0.0, 0.0, 3.0]);
    let mixed = backend.generalized_eigen(&k_mixed, &m).unwrap();
    for (col, expected) in [12.0 / 7.0, 4.0].into_iter().enumerate() {
        close(mixed.eigenvalues[col], expected, 1e-12);
        let v = mat(
            2,
            1,
            &[
                mixed.vectors.get(0, col).unwrap(),
                mixed.vectors.get(1, col).unwrap(),
            ],
        );
        let mv = backend.matmul(m.dense(), &v).unwrap();
        let b = mat(
            2,
            1,
            &mv.values().iter().map(|x| x * expected).collect::<Vec<_>>(),
        );
        residual(k_mixed.dense(), &v, &b, 1e-12);
        for other in 0..2 {
            let norm = (0..2)
                .map(|i| mixed.vectors.get(i, other).unwrap() * mv.get(i, 0).unwrap())
                .sum::<f64>();
            close(norm, if other == col { 1.0 } else { 0.0 }, 1e-12);
        }
    }
    check_error(
        backend.generalized_eigen(&m, &sym(2, &[0.0, 1.0, 1.0, 0.0])),
        "ENG-E-0010",
    );
    let zero_modes = backend
        .generalized_eigen(
            &SymMatrix::zeros(3).unwrap(),
            &SymMatrix::new(identity(3)).unwrap(),
        )
        .unwrap();
    for value in zero_modes.eigenvalues {
        close(value, 0.0, 1e-12);
    }

    check_error(backend.matmul(&rectangular, &rectangular), "ENG-E-0010");
    check_error(
        backend.solve_dense(&rectangular, &identity(2)),
        "ENG-E-0010",
    );
    check_error(backend.solve_dense(&a, &identity(2)), "ENG-E-0010");
    check_error(
        backend.solve_dense(&mat(2, 2, &[1.0, 2.0, 2.0, 4.0]), &identity(2)),
        "ENG-E-0006",
    );
    let singular_matrix = sym(3, &[2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0]);
    let error = check_error(backend.factorize(&singular_matrix), "ENG-E-0006");
    assert!(error
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Dof(DofIndex(1)))));
    let mechanism = sym(2, &[1.0, -1.0, -1.0, 1.0]);
    let error = check_error(backend.factorize(&mechanism), "ENG-E-0006");
    assert!(!error.entities.is_empty());
    let error = check_error(
        backend.generalized_eigen(&singular_matrix, &singular_matrix),
        "ENG-E-0006",
    );
    assert!(!error.entities.is_empty());
    check_error(
        backend.generalized_eigen(&m, &sym(2, &[1.0, 0.0, 0.0, -1.0])),
        "ENG-E-0010",
    );
    check_error(
        backend.generalized_eigen(&m, &SymMatrix::zeros(3).unwrap()),
        "ENG-E-0010",
    );
    check_error(
        backend.assemble(
            2,
            2,
            &[Triplet {
                row: 2,
                col: 0,
                value: 1.0,
            }],
        ),
        "ENG-E-0010",
    );
    check_error(
        backend.assemble(
            2,
            2,
            &[Triplet {
                row: 0,
                col: 0,
                value: f64::NAN,
            }],
        ),
        "ENG-E-0008",
    );
    check_error(
        backend.assemble(
            2,
            2,
            &[Triplet {
                row: 0,
                col: 0,
                value: f64::MAX,
            }; 2],
        ),
        "ENG-E-0008",
    );
    check_error(
        backend.matmul(&mat(1, 1, &[f64::MAX]), &mat(1, 1, &[2.0])),
        "ENG-E-0008",
    );
    for scaling in [1e-100, 1e100] {
        let a = sym(2, &[2.0 * scaling, -scaling, -scaling, 2.0 * scaling]);
        let b = mat(2, 1, &[scaling, scaling]);
        residual(
            a.dense(),
            &backend.factorize(&a).unwrap().solve(&b).unwrap(),
            &b,
            1e-12,
        );
    }
    let empty = SymMatrix::zeros(0).unwrap();
    assert_eq!(
        backend.factorize(&empty).unwrap().info().inertia,
        Some((0, 0, 0))
    );
    assert_eq!(
        backend
            .generalized_eigen(&empty, &empty)
            .unwrap()
            .eigenvalues
            .len(),
        0
    );
    assert_eq!(
        backend
            .solve_dense(empty.dense(), &DMat::zeros(0, 2).unwrap())
            .unwrap()
            .cols(),
        2
    );
}

#[test]
fn reference_conformance() {
    conformidad(&mut ReferenceBackend);
}
#[cfg(feature = "faer")]
#[test]
fn faer_pending_is_explicit() {
    check_error(FaerBackend::new(), "ENG-E-0009");
}

#[test]
fn matrix_validation_and_sparse_storage() {
    check_error(
        SymMatrix::new(mat(2, 2, &[1.0, 2.0, 3.0, 4.0])),
        "ENG-E-0010",
    );
    check_error(SymMatrix::new(DMat::zeros(2, 3).unwrap()), "ENG-E-0010");
    for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        check_error(DMat::new(1, 1, vec![v]), "ENG-E-0008");
    }
    check_error(DMat::new(usize::MAX, 2, vec![]), "ENG-E-0010");
    check_error(DMat::new(2, 2, vec![1.0]), "ENG-E-0010");
    assert_eq!(DMat::zeros(0, 3).unwrap().get(0, 0), None);
    let sparse = SparseMatrix::from_triplets(
        100_000,
        100_000,
        &[Triplet {
            row: 99_999,
            col: 99_999,
            value: 0.0,
        }],
    )
    .unwrap();
    assert_eq!(sparse.values().len(), 1);
    let pattern = sparse.symmetric_pattern().unwrap();
    assert_eq!(pattern.coordinates(), &[(99_999, 99_999)]);
    let coords = SparsityPattern::from_coordinates(2, vec![(1, 0), (0, 1), (1, 0)]).unwrap();
    let mask = SparsityPattern::new(2, vec![false, true, true, false]).unwrap();
    assert_eq!(coords, mask);
    assert_eq!(coords.try_entries().unwrap(), mask.try_entries().unwrap());
    assert_eq!(
        coords.mask_iter().collect::<Vec<_>>(),
        vec![false, true, true, false]
    );
    assert!(coords.contains(0, 1) && !coords.contains(0, 0) && !coords.contains(9, 9));
    check_error(
        SparsityPattern::from_coordinates(2, vec![(1, 0)]),
        "ENG-E-0010",
    );
    check_error(
        SparsityPattern::from_coordinates(2, vec![(2, 2)]),
        "ENG-E-0010",
    );
    check_error(
        SparseMatrix::from_triplets(1, 2, &[])
            .unwrap()
            .symmetric_pattern(),
        "ENG-E-0010",
    );
}

#[test]
fn pattern_dense_mask_is_fallible_for_extreme_orders() {
    let huge = SparsityPattern::from_coordinates(usize::MAX, vec![(0, 0)]);
    // from_coordinates rechaza orden² desbordado (ENG-E-0010) y por tanto no hay patrón.
    check_error(huge, "ENG-E-0010");
    // Orden grande sin desbordar pero con máscara densa sobre el límite.
    // Orden² > límite por destino sin desbordar usize: 2^40 B (> 8 GiB) en 64 bits; 1,6e9 B (> 1 GiB) en 32 bits.
    #[cfg(target_pointer_width = "64")]
    let order = 1usize << 20;
    #[cfg(not(target_pointer_width = "64"))]
    let order = 40_000usize;
    let p = SparsityPattern::from_coordinates(order, vec![(0, 0), (order - 1, order - 1)]).unwrap();
    assert_eq!(p.coordinates().len(), 2);
    check_error(p.try_entries(), "ENG-E-0013");
    // Las vías perezosas funcionan sin materializar.
    assert!(p.contains(order - 1, order - 1) && !p.contains(0, 1));
    assert_eq!(
        p.mask_iter().take(3).collect::<Vec<_>>(),
        vec![true, false, false]
    );
}

#[test]
fn pattern_dense_mask_small_is_cached_and_correct() {
    let p = SparsityPattern::from_coordinates(3, vec![(0, 0), (2, 1), (1, 2)]).unwrap();
    let a = p.try_entries().unwrap();
    assert_eq!(
        a,
        &[true, false, false, false, false, true, false, true, false]
    );
    assert!(std::ptr::eq(a, p.try_entries().unwrap()));
    assert_eq!(p.mask_iter().collect::<Vec<_>>(), a);
}
