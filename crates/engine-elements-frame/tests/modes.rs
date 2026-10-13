//! Vibración Euler–Bernoulli: esperados cerrados, sin oráculos ni fuentes externas.
use engine_contracts::{Element, ElementCtx, ElementParams, MassKind};
use engine_core::Dim;
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};

// Ensambla exclusivamente uy/rz: la viga horizontal desacopla flexión y axial.
// E=ρ=A=I=L=1 ⇒ ω=β²; f=ω/(2π). Masa consistente, sin inercia rotatoria.
fn beam_matrices(elements: usize, cantilever: bool) -> (SymMatrix, SymMatrix) {
    let beam = EulerFrame2D::new(1.0, 1.0, 1.0, 1.0, MomentReleases::default()).unwrap();
    let params = ElementParams::default();
    let total = 2 * (elements + 1);
    let free: Vec<_> = (0..total)
        .filter(|&dof| {
            if cantilever {
                dof >= 2 // Empotramiento: uy=rz=0 en x=0.
            } else {
                dof != 0 && dof != 2 * elements // Apoyos: uy=0 en ambos extremos.
            }
        })
        .collect();
    let n = free.len();
    let mut stiffness = vec![0.0; n * n];
    let mut mass = vec![0.0; n * n];
    for element in 0..elements {
        let coords = [
            [element as f64 / elements as f64, 0.0, 0.0],
            [(element + 1) as f64 / elements as f64, 0.0, 0.0],
        ];
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        let k = beam.respond(&ctx, &[0.0; 6], &[]).unwrap().tangent;
        let m = beam.mass(&ctx, MassKind::Consistent).unwrap().unwrap();
        let global = [
            2 * element,
            2 * element + 1,
            2 * element + 2,
            2 * element + 3,
        ];
        let local = [1, 2, 4, 5];
        for i in 0..4 {
            for j in 0..4 {
                if let (Some(row), Some(col)) = (
                    free.iter().position(|&dof| dof == global[i]),
                    free.iter().position(|&dof| dof == global[j]),
                ) {
                    stiffness[row * n + col] += k.get(local[i], local[j]).unwrap();
                    mass[row * n + col] += m.get(local[i], local[j]).unwrap();
                }
            }
        }
    }
    (
        SymMatrix::new(DMat::new(n, n, stiffness).unwrap()).unwrap(),
        SymMatrix::new(DMat::new(n, n, mass).unwrap()).unwrap(),
    )
}

fn frequencies(elements: usize, cantilever: bool) -> Vec<f64> {
    let (k, m) = beam_matrices(elements, cantilever);
    let modes = ReferenceBackend.lowest_modes(&k, &m, 3).unwrap();
    assert_eq!(modes.rigid_mode_count, 0);
    modes
        .eigenvalues
        .iter()
        .map(|lambda| lambda.sqrt() / (2.0 * std::f64::consts::PI))
        .collect()
}

fn check_refinement(cantilever: bool, beta: [f64; 3]) {
    let coarse = frequencies(4, cantilever);
    let fine = frequencies(12, cantilever);
    for mode in 0..3 {
        let exact = beta[mode].powi(2) / (2.0 * std::f64::consts::PI);
        let coarse_error = (coarse[mode] / exact - 1.0).abs();
        let fine_error = (fine[mode] / exact - 1.0).abs();
        // Tolerancia relativa 3e−4 en frecuencia con 12 elementos Hermite.
        // La malla de 4 debe mejorar al menos ×10: se verifica refinamiento,
        // no solo coincidencia con una tolerancia arbitraria.
        assert!(
            fine_error < 3e-4,
            "modo {}: f={}, exacta={exact}, error={fine_error}",
            mode + 1,
            fine[mode]
        );
        assert!(
            fine_error < coarse_error / 10.0,
            "refinamiento modo {}: gruesa={coarse_error}, fina={fine_error}",
            mode + 1
        );
    }
}

#[test]
fn simply_supported_beam_converges_to_pi_n() {
    let pi = std::f64::consts::PI;
    check_refinement(false, [pi, 2.0 * pi, 3.0 * pi]);
}

#[test]
fn cantilever_converges_to_closed_form_roots() {
    // Raíces de cos(β) cosh(β)=−1; los valores 1.8751, 4.6941, 7.8548
    // solicitados se conservan con más cifras para no dominar el error de malla.
    check_refinement(
        true,
        [
            1.875_104_068_711_961,
            4.694_091_132_974_175,
            7.854_757_438_237_612,
        ],
    );
}
