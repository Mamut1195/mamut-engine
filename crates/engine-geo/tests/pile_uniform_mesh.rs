//! Source: own trapezoidal quadrature derivation, provenance/ENG-GEO-005.md.
use engine_geo::pile::PileDiscretization;

#[test]
fn uniform_mesh_has_exact_end_and_integrates_affine_depth() {
    let mesh = PileDiscretization::uniform(8.0, 4).unwrap();
    assert_eq!(mesh.depths_m(), &[0.0, 2.0, 4.0, 6.0, 8.0]);
    assert_eq!(mesh.tributaries_m(), &[1.0, 2.0, 2.0, 2.0, 1.0]);
    let integral: f64 = mesh
        .depths_m()
        .iter()
        .zip(mesh.tributaries_m())
        .map(|(z, w)| z * w)
        .sum();
    assert_eq!(integral, 32.0);
}

#[test]
fn uniform_mesh_rejects_invalid_and_unrepresentable_inputs() {
    for (length, elements, code) in [
        (0.0, 4, "ENG-E-0010"),
        (1.0, 0, "ENG-E-0010"),
        (1.0, 4097, "ENG-E-0010"),
        (f64::NAN, 4, "ENG-E-0008"),
        (f64::from_bits(1), 2, "ENG-E-0010"),
    ] {
        assert_eq!(
            PileDiscretization::uniform(length, elements)
                .err()
                .map(|e| e.code),
            Some(code)
        );
    }
}
