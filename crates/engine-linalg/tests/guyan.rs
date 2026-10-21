//! SOURCE: derivación propia del complemento de Schur; Guyan (1965).
//! Referencia independiente: eliminación densa solve_dense de reference.rs.
#[path = "../../engine-assembly/tests/fixtures/counting.rs"]
mod counting;
use engine_core::{DofIndex, EntityRef};
use engine_linalg::{guyan_condense, DMat, LinAlgBackend, ReferenceBackend, SymMatrix};

fn sym(n: usize, values: Vec<f64>) -> SymMatrix {
    SymMatrix::new(DMat::new(n, n, values).unwrap()).unwrap()
}

#[test]
fn finite_spectrum_dense_elimination_recovery_and_one_factor() {
    // Intercalados m=[0,2], s=[1,3]; Mmm acoplada, sin diagonalización.
    let k = sym(
        4,
        vec![
            10., 2., 1., 1., 2., 4., -1., 1., 1., -1., 8., 2., 1., 1., 2., 3.,
        ],
    );
    let m = sym(
        4,
        vec![
            2., 0., 0.25, 0., 0., 0., 0., 0., 0.25, 0., 1., 0., 0., 0., 0., 0.,
        ],
    );
    let mut backend = counting::CountingBackend::default();
    let reduction = guyan_condense(&mut backend, &k, &m).unwrap().unwrap();
    assert_eq!(reduction.massive(), &[0, 2]);
    assert_eq!(reduction.massless(), &[1, 3]);
    assert_eq!(backend.factorizations.get(), 1);
    assert_eq!(backend.solves.get(), 1); // dos RHS, un único solve del factor.

    // Eliminación explícita independiente mediante solver denso con pivoteo.
    let ss = DMat::new(2, 2, vec![4., 1., 1., 3.]).unwrap();
    let sm = DMat::new(2, 2, vec![2., -1., 1., 2.]).unwrap();
    let x = ReferenceBackend.solve_dense(&ss, &sm).unwrap();
    let t = DMat::new(
        4,
        2,
        vec![
            1.,
            0.,
            -x.values()[0],
            -x.values()[1],
            0.,
            1.,
            -x.values()[2],
            -x.values()[3],
        ],
    )
    .unwrap();
    let transpose = ReferenceBackend.transpose(&t).unwrap();
    let kt = ReferenceBackend.matmul(k.dense(), &t).unwrap();
    let dense_k = ReferenceBackend.matmul(&transpose, &kt).unwrap();
    let mt = ReferenceBackend.matmul(m.dense(), &t).unwrap();
    let dense_m = ReferenceBackend.matmul(&transpose, &mt).unwrap();
    // Simetría explícita de referencia para el contrato exacto de SymMatrix.
    let mut values = dense_k.values().to_vec();
    values[1] = values[2];
    let reference = ReferenceBackend
        .generalized_eigen(&sym(2, values), &SymMatrix::new(dense_m).unwrap())
        .unwrap();
    let actual = ReferenceBackend
        .lowest_modes(&reduction.stiffness, &reduction.mass, 2)
        .unwrap();
    for (&a, &b) in actual.eigenvalues.iter().zip(&reference.eigenvalues) {
        assert!((a / b - 1.).abs() < 2e-14);
    }
    let full = reduction.recover(&actual.vectors).unwrap();
    let explicit = ReferenceBackend.matmul(&t, &actual.vectors).unwrap();
    for (&a, &b) in full.values().iter().zip(explicit.values()) {
        assert!((a - b).abs() < 2e-15);
    }
    // Pares finitos del problema completo singular: Kφ=λMφ y φᵀMφ=1.
    for col in 0..2 {
        let mut norm = 0.;
        for i in 0..4 {
            let mut residual = 0.;
            for j in 0..4 {
                let v = full.values()[j * 2 + col];
                residual += (k.dense().values()[i * 4 + j]
                    - actual.eigenvalues[col] * m.dense().values()[i * 4 + j])
                    * v;
                norm += full.values()[i * 2 + col] * m.dense().values()[i * 4 + j] * v;
            }
            assert!(residual.abs() < 1e-13);
        }
        assert!((norm - 1.).abs() < 1e-14);
    }
    // Gram completo después de recover(): incluye las filas recuperadas y
    // los términos fuera de diagonal de M, sin renormalizar las columnas.
    for a in 0..2 {
        for b in 0..2 {
            let mut gram = 0.;
            for i in 0..4 {
                for j in 0..4 {
                    gram += full.values()[i * 2 + a]
                        * m.dense().values()[i * 4 + j]
                        * full.values()[j * 2 + b];
                }
            }
            assert!((gram - if a == b { 1. } else { 0. }).abs() < 1e-14);
        }
    }
    assert_eq!(backend.factorizations.get(), 1); // recuperación sin resolver.
    assert_eq!(backend.solves.get(), 1);
    assert!(reduction.recover(&DMat::zeros(3, 1).unwrap()).is_err());
}

#[test]
fn rejects_singular_and_indefinite_massless_block() {
    let m = sym(3, vec![1., 0., 0., 0., 0., 0., 0., 0., 0.]);
    // Singular, diagonal negativa y bloque invertible con pivote 2×2 indefinido.
    for (ss, dof) in [
        ([0., 0., 0., 1.], 1),
        ([1., 0., 0., 0.], 2),
        ([-1., 0., 0., 1.], 1),
        ([0., 1., 1., 0.], 1),
    ] {
        let k = sym(3, vec![2., 0., 0., 0., ss[0], ss[1], 0., ss[2], ss[3]]);
        let error = guyan_condense(&mut ReferenceBackend, &k, &m).unwrap_err();
        assert_eq!(error.code, "ENG-E-0014");
        assert!(
            matches!(error.entities.as_slice(), [EntityRef::Dof(DofIndex(index))] if *index == dof)
        );
        assert!(error.message.contains(&format!("bloque del GDL {dof}.")));
    }
}

#[test]
fn exact_zero_partition_negative_mass_empty_mass_and_noop() {
    let k = sym(2, vec![2., 0., 0., 3.]);
    let mut backend = counting::CountingBackend::default();
    // Una masa positiva diminuta sigue siendo masa; ninguna factorización.
    let m = sym(2, vec![1., 0., 0., 1e-200]);
    assert!(guyan_condense(&mut backend, &k, &m).unwrap().is_none());
    assert_eq!(backend.factorizations.get(), 0);
    // Diagonal cero pero fila no nula pertenece a m (el eigensolver rechazará
    // luego esta masa indefinida); jamás se elimina una masa acoplada no nula.
    assert!(
        guyan_condense(&mut backend, &k, &sym(2, vec![0., 1., 1., 0.]))
            .unwrap()
            .is_none()
    );
    for m in [sym(2, vec![1., 0., 0., -1.]), SymMatrix::zeros(2).unwrap()] {
        let error = guyan_condense(&mut backend, &k, &m).unwrap_err();
        assert_eq!(error.code, "ENG-E-0006");
        assert!(error
            .entities
            .iter()
            .any(|e| matches!(e, EntityRef::Dof(_))));
    }
    assert_eq!(
        guyan_condense(&mut backend, &k, &SymMatrix::zeros(1).unwrap())
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
}

#[test]
fn no_massless_rows_returns_none_and_preserves_inputs_bit_for_bit() {
    let k = sym(2, vec![3., -0., -0., 7.]);
    let m = sym(2, vec![2., 0.5, 0.5, 1.]);
    let bits = |matrix: &SymMatrix| {
        matrix
            .dense()
            .values()
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>()
    };
    let before = (bits(&k), bits(&m));
    let addresses = (k.dense().values().as_ptr(), m.dense().values().as_ptr());
    let mut backend = counting::CountingBackend::default();
    assert!(guyan_condense(&mut backend, &k, &m).unwrap().is_none());
    assert_eq!(before, (bits(&k), bits(&m)));
    assert_eq!(
        addresses,
        (k.dense().values().as_ptr(), m.dense().values().as_ptr())
    );
    assert_eq!(backend.factorizations.get(), 0);
    assert_eq!(backend.solves.get(), 0);
}
