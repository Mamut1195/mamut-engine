use engine_core::EResult;
use engine_geo::pile::PileDiscretization;

#[test]
fn irregular_mesh_integrates_constant_and_linear_fields() -> EResult<()> {
    let pile = PileDiscretization::new(vec![0.0, 2.0, 5.0, 9.0])?;
    assert_eq!(pile.depths_m(), &[0.0, 2.0, 5.0, 9.0]);
    assert_eq!(pile.tributaries_m(), &[1.0, 2.5, 3.5, 2.0]);
    let mut first_moment = 0.0;
    for (&depth, &weight) in pile.depths_m().iter().zip(pile.tributaries_m()) {
        first_moment += depth * weight;
    }
    assert_eq!(pile.tributaries_m().iter().sum::<f64>(), 9.0);
    assert_eq!(first_moment, 40.5);
    Ok(())
}

#[test]
fn rejects_invalid_geometry_and_nonfinite_depths() {
    for depths in [
        vec![],
        vec![0.0],
        vec![-1.0, 1.0],
        vec![0.0, 0.0],
        vec![1.0, 0.0],
        vec![0.0, f64::from_bits(1)],
    ] {
        assert_eq!(
            PileDiscretization::new(depths).unwrap_err().code,
            "ENG-E-0010"
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            PileDiscretization::new(vec![0.0, value]).unwrap_err().code,
            "ENG-E-0008"
        );
    }
}

#[test]
fn two_nodes_split_the_span() -> EResult<()> {
    let pile = PileDiscretization::new(vec![3.0, 7.0])?;
    assert_eq!(pile.tributaries_m(), &[2.0, 2.0]);
    Ok(())
}

#[test]
fn semi_infinite_lateral_field_has_balanced_tributary_reactions() -> EResult<()> {
    // EI=1, k=4, H=4 and head moment=2 give beta=1 and
    // y=exp(-z)(cos(z)+sin(z)). The exact integrals of -p and
    // -z*p over the semi-infinite domain are 4 and 2, respectively.
    let mut previous_force_error = f64::INFINITY;
    let mut previous_moment_error = f64::INFINITY;
    for divisions_per_m in [16, 32, 64] {
        let count = 16 * divisions_per_m;
        let depths: Vec<f64> = (0..=count)
            .map(|i| f64::from(i) / f64::from(divisions_per_m))
            .collect();
        let pile = PileDiscretization::new(depths)?;
        let mut force = 0.0;
        let mut moment = 0.0;
        for (&z, &weight) in pile.depths_m().iter().zip(pile.tributaries_m()) {
            let y = engine_core::math::exp(-z)
                * (engine_core::math::cos(z) + engine_core::math::sin(z));
            let reaction = 4.0 * y * weight;
            force += reaction;
            moment += z * reaction;
        }
        let force_error = (force - 4.0).abs();
        let moment_error = (moment - 2.0).abs();
        assert!(force_error < previous_force_error);
        assert!(moment_error < previous_moment_error);
        previous_force_error = force_error;
        previous_moment_error = moment_error;
    }
    assert!(previous_force_error < 0.001);
    assert!(previous_moment_error < 0.001);
    Ok(())
}
