//! ENG-LA-009-FU: cuatro regresiones aisladas con oráculos diagonales propios.
use engine_core::EResult;
use engine_linalg::{
    sturm_count, DMat, GeneralizedEigen, LinAlgBackend, ReferenceBackend, SymFactorization,
    SymMatrix,
};

fn diagonal(values: &[f64]) -> SymMatrix {
    let n = values.len();
    let mut entries = vec![0.0; n * n];
    for (i, &value) in values.iter().enumerate() {
        entries[i * n + i] = value;
    }
    SymMatrix::new(DMat::new(n, n, entries).unwrap()).unwrap()
}

#[derive(Default)]
struct ContractBackend {
    swap_first_pair: bool,
    inertia: Option<(usize, usize, usize)>,
}

impl LinAlgBackend for ContractBackend {
    fn name(&self) -> &'static str {
        "test.lowest_modes_contract"
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

    fn symmetric_inertia(&mut self, a: &SymMatrix) -> EResult<(usize, usize, usize)> {
        match self.inertia {
            Some(inertia) => Ok(inertia),
            None => ReferenceBackend.symmetric_inertia(a),
        }
    }

    fn generalized_eigen(&mut self, k: &SymMatrix, m: &SymMatrix) -> EResult<GeneralizedEigen> {
        let mut result = ReferenceBackend.generalized_eigen(k, m)?;
        if self.swap_first_pair {
            result.eigenvalues.swap(0, 1);
            // Se permutan también las columnas: los pares propios siguen siendo
            // válidos; solo se viola el orden ascendente exigido al backend.
            let n = k.order();
            let mut values = result.vectors.values().to_vec();
            for row in values.chunks_mut(n) {
                row.swap(0, 1);
            }
            result.vectors = DMat::new(n, n, values)?;
        }
        Ok(result)
    }
}

#[test]
fn anti_revert_rejects_unsorted_eigenpairs() {
    // K=diag(1,2,4), M=I: raíces 1,2,4 y vectores canónicos.
    let k = diagonal(&[1.0, 2.0, 4.0]);
    let m = diagonal(&[1.0; 3]);
    assert_eq!(
        ReferenceBackend
            .lowest_modes(&k, &m, 3)
            .unwrap()
            .eigenvalues,
        vec![1.0, 2.0, 4.0]
    );
    let mut backend = ContractBackend {
        swap_first_pair: true,
        ..Default::default()
    };
    // [2,1,4] conserva la raíz mayor al final: con count=n el control de Sturm
    // pasaría, por lo que solo la guarda de orden puede rechazar el resultado.
    let error = backend.lowest_modes(&k, &m, 3).unwrap_err();
    assert_eq!(error.code, "ENG-E-0007");
    assert!(error.message.contains("sin ordenar"), "{error:?}");
}

#[test]
fn anti_revert_rejects_inertia_whose_sum_differs_from_order() {
    let k = diagonal(&[1.0, 2.0, 4.0]);
    let m = diagonal(&[1.0; 3]);
    // En sigma=2, K-sigma*M=diag(-1,0,2): inercia (1,1,1).
    assert_eq!(sturm_count(&mut ReferenceBackend, &k, &m, 2.0).unwrap(), 1);
    // En lowest_modes(count=1), la inercia correcta es (2,1,0).
    assert_eq!(
        ReferenceBackend
            .lowest_modes(&k, &m, 1)
            .unwrap()
            .eigenvalues,
        vec![1.0]
    );
    // Los primeros cuatro casos conservan el conteo negativo 1; sin validar la
    // suma, ambos caminos los aceptarían. Cubren déficit, exceso y desbordes.
    // El último suma n con wrapping_add: solo la suma comprobada lo rechaza
    // como inercia incompatible, antes de comprobar el conteo de modos.
    let n = k.order();
    for inertia in [
        (0, 1, 0),
        (2, 1, 1),
        (usize::MAX, 1, 0),
        (2, 1, usize::MAX),
        (usize::MAX, n + 1, 0),
    ] {
        let mut backend = ContractBackend {
            inertia: Some(inertia),
            ..Default::default()
        };
        let errors = [
            backend.lowest_modes(&k, &m, 1).unwrap_err(),
            sturm_count(&mut backend, &k, &m, 2.0).unwrap_err(),
        ];
        for error in errors {
            assert_eq!(error.code, "ENG-E-0007");
            assert!(error.message.contains("inercia incompatible"), "{error:?}");
        }
    }
}

#[test]
fn anti_revert_relative_shift_includes_nearby_distinct_root() {
    // Raíces s y s*(1+5e-8), distintas pero dentro del margen relativo 1e-7.
    // El piso 16*tol_cero (~9.1e-13*s) por sí solo no alcanza la segunda raíz.
    for scale in [1e-12, 1.0, 1e12] {
        let exact = [scale, scale * (1.0 + 5e-8)];
        let k = diagonal(&exact);
        let m = diagonal(&[1.0; 2]);
        let midpoint = (exact[0] + exact[1]) / 2.0;
        assert_eq!(
            sturm_count(&mut ReferenceBackend, &k, &m, midpoint).unwrap(),
            1
        );
        let error = ReferenceBackend.lowest_modes(&k, &m, 1).unwrap_err();
        assert_eq!(error.code, "ENG-E-0007");
        assert!(
            error.message.contains("corte de multiplicidad"),
            "{error:?}"
        );
        assert!(error.message.contains(")=2"), "{error:?}");
        let modes = ReferenceBackend.lowest_modes(&k, &m, 2).unwrap();
        for (&value, expected) in modes.eigenvalues.iter().zip(exact) {
            assert!((value / expected - 1.0).abs() < 1e-14);
        }
    }
}

#[test]
fn anti_revert_rigid_modes_are_exactly_zero_for_both_signs() {
    // Raíces -1e-15, +1e-15, 1; tol_cero=128*eps*3 > 1e-15.
    // Los ceros exactos como entrada no detectarían la falta de puesta a cero.
    let exact = [-1e-15, 1e-15, 1.0];
    let k = diagonal(&exact);
    let m = diagonal(&[1.0; 3]);
    let raw = ReferenceBackend.generalized_eigen(&k, &m).unwrap();
    assert!(raw.eigenvalues[0] < 0.0);
    assert!(raw.eigenvalues[1] > 0.0);
    for count in [2, 3] {
        let modes = ReferenceBackend.lowest_modes(&k, &m, count).unwrap();
        assert_eq!(modes.rigid_mode_count, 2);
        assert_eq!(&modes.eigenvalues[..2], &[0.0, 0.0]);
        if count == 3 {
            assert_eq!(modes.eigenvalues[2], 1.0);
        }
        // Para matrices diagonales y M=I los vectores son los ejes canónicos.
        for row in 0..3 {
            for col in 0..count {
                let expected = if row == col { 1.0 } else { 0.0 };
                assert_eq!(modes.vectors.values()[row * count + col], expected);
            }
        }
    }
}
