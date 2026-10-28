//! J4 round r6: nearly parallel orientation, torsion with rounding, extreme axial force.
mod fixtures;
use engine_contracts::{Element, ElementParams};
use engine_core::math;
use engine_elements_frame3d::EulerFrame3D;
use fixtures::*;

#[test]
fn nearly_parallel_orientation_is_rejected_not_accepted_with_spurious_forces() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [1.0, 2.0, 3.0]];
    let c = ctx(&coords, &p);
    for hint in [[1.0, 2.0 + 1e-11, 3.0], [1.0, 2.0, 3.0], [-1.0, -2.0, -3.0]] {
        let b = EulerFrame3D::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, hint, releases(0)).unwrap();
        let out = b.recover(&c, &[0.0; 12], &[], &[], &[0.0]);
        assert!(out.is_err(), "orientation {hint:?} must be rejected");
    }
}

#[test]
fn slightly_oblique_orientation_stays_orthonormal_rigid_rotation_gives_no_force() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [1.0, 2.0, 3.0]];
    let c = ctx(&coords, &p);
    // Small but accepted angle (sin≈3e-8): the triad must remain orthonormal.
    let hint = [1.0, 2.0 + 1e-7, 3.0];
    let b = EulerFrame3D::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, hint, releases(0)).unwrap();
    let x = [1.0, 2.0, 3.0].map(|v| v / math::sqrt(14f64));
    let mut u = [0.0; 12];
    for (i, v) in x.iter().enumerate() {
        u[3 + i] = 1e-3 * v;
        u[9 + i] = 1e-3 * v;
    }
    let out = b.recover(&c, &u, &[], &[], &[0.0, 0.5, 1.0]).unwrap();
    for f in out.end_forces_local.iter() {
        assert!(f.abs() < 1e-12, "spurious force {f}");
    }
}

#[test]
fn torsion_release_with_zero_local_moment_load_is_not_rejected_by_rounding() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [1.0, 7.0, 0.0]];
    let c = ctx(&coords, &p);
    let mut r = releases(0);
    r.i[0] = true;
    r.j[0] = true;
    let b = oriented(r, [0.0, 0.0, 1.0]);
    // Global distributed moment (7,-1,0): local mx = 0 mathematically.
    let v = b.load_vector(&c, &load([0.0; 3], [7.0, -1.0, 0.0]));
    assert!(v.is_ok(), "{v:?}");
    // Actual net torsion is still rejected.
    let bad = b.load_vector(&c, &load([0.0; 3], [1.0, 7.0, 0.0]));
    assert_eq!(bad.err().unwrap().code, "ENG-E-0006");
}

#[test]
fn axial_force_survives_tiny_e_times_huge_a() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [1.0, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let b = EulerFrame3D::new(
        1e-200,
        1.0,
        1e200,
        1.0,
        1.0,
        1.0,
        0.0,
        [0.0, 0.0, 1.0],
        releases(0),
    )
    .unwrap();
    let mut u = [0.0; 12];
    u[6] = 1e-130; // E·ε=1e-330 (0 in f64), N=A·E·ε=1e-130
    let kg = b.geometric_stiffness(&c, &u, &[]).unwrap().unwrap();
    let peak = kg.values().iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    assert!(peak > 0.0, "Kg was zeroed by underflow of E·ε");
}

#[test]
fn vertical_column_rejects_plus_z_and_accepts_another_vector() {
    let p = ElementParams::default();
    for coords in [[[0.0; 3], [0.0, 0.0, 3.0]], [[0.0, 0.0, 3.0], [0.0; 3]]] {
        let c = ctx(&coords, &p);
        let b = oriented(releases(0), [0.0, 0.0, 1.0]);
        assert!(b.recover(&c, &[0.0; 12], &[], &[], &[0.0]).is_err());
        let ok = oriented(releases(0), [1.0, 0.0, 0.0]);
        assert!(ok.recover(&c, &[0.0; 12], &[], &[], &[0.0]).is_ok());
    }
}

#[test]
fn torsion_tolerance_uses_moment_scale_not_force_scale() {
    let p = ElementParams::default();
    let length = 1e-3;
    let coords = [[0.0; 3], [length, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let q = load([0.0, 1e15, 0.0], [100.0, 0.0, 0.0]);
    // Without releases, obtaining the load checks that no prior ENG-E-0008 occurs.
    // The triad matches the global axes: f is also the local vector.
    let f = beam(releases(0)).load_vector(&c, &q).unwrap();
    assert!(f.iter().all(|v| v.is_finite()));
    let moments = [3, 4, 5, 9, 10, 11]
        .iter()
        .fold(0.0_f64, |a, &i| a.max(f[i].abs()));
    let forces = [0, 1, 2, 6, 7, 8]
        .iter()
        .fold(0.0_f64, |a, &i| a.max(f[i].abs()));
    let net = (f[3] + f[9]).abs();
    let tolerance = 1e-12 * moments.max(length * forces);
    let mixed_tolerance = 1e-12 * f.iter().fold(0.0_f64, |a, v| a.max(v.abs()));
    // Vy=qy·L/2=5e11 N; |Mz|=qy·L²/12≈8,333e7 N·m; T=mx·L=0,1 N·m.
    // Current scale=max(8,333e7,5e8)=5e8 N·m → threshold=5e-4 N·m.
    // Mixed scale=5e11 → threshold=0,5: would wrongly accept the torque 0,1.
    close(forces, 5e11);
    close(moments, 1e9 / 12.0);
    close(net, 0.1);
    close(tolerance, 5e-4);
    close(mixed_tolerance, 0.5);
    assert!(net > tolerance && net < mixed_tolerance);
    let mut r = releases(0);
    r.i[0] = true;
    r.j[0] = true;
    let b = oriented(r, [0.0, 0.0, 1.0]);
    let e = b.load_vector(&c, &q).unwrap_err();
    assert_eq!(e.code, "ENG-E-0006");
}

#[test]
fn torsion_tolerance_rejects_overflow_in_length_times_finite_forces() {
    let p = ElementParams::default();
    let length = 4.0_f64;
    let qx = 3e307_f64;
    let coords = [[0.0; 3], [length, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let q = load([qx, 0.0, 0.0], [0.0; 3]);
    assert!(length.is_finite() && qx.is_finite());
    // Axial force per node=qx·L/2=6e307 N, finite and without moments;
    // L·forces=2,4e308 N·m overflows although both inputs are finite.
    // Without double torsional release the scale is unused: the load is preserved.
    for mask in [0, 1, 8] {
        let f = beam(releases(mask)).load_vector(&c, &q).unwrap();
        assert!(f.iter().all(|v| v.is_finite()));
        assert_eq!(f[0], qx * (length / 2.0));
        assert_eq!(f[6], f[0]);
        for i in [1, 2, 3, 4, 5, 7, 8, 9, 10, 11] {
            assert_eq!(f[i], 0.0);
        }
        assert!(!(length * f[0]).is_finite());
    }
    let b = beam(releases(1 | 8));
    let e = b.load_vector(&c, &q).unwrap_err();
    assert_eq!(e.code, "ENG-E-0008");
}
