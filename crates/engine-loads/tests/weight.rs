mod fixtures;
use engine_core::EResult;
use engine_loads::{self_weight, self_weight_per_length};
use fixtures::close;

#[test]
fn total_line_weight_and_member_sum() -> EResult<()> {
    let gravity = [1.0, -2.0, -9.81];
    let mut total = [0.0; 3];
    let mut mass = 0.0;
    for (rho, area, l) in [(7850.0, 0.02, 4.0), (2400.0, 0.3, 7.0), (0.0, 1.0, 2.0)] {
        mass += rho * area * l;
        let force = self_weight(rho, area, l, gravity)?;
        let line = self_weight_per_length(rho, area, gravity)?;
        for i in 0..3 {
            close(force[i], rho * area * l * gravity[i]);
            close(line[i] * l, force[i]);
            total[i] += force[i];
        }
    }
    for i in 0..3 {
        close(total[i], mass * gravity[i]);
    }
    assert_eq!(self_weight(1.0, 1.0, 1.0, [0.0; 3])?, [0.0; 3]);
    assert_eq!(self_weight(0.0, 1e308, 1e308, gravity)?, [0.0; 3]);
    assert_eq!(self_weight(1e308, 1.0, 1e-300, [1.0; 3])?, [1e8; 3]);
    Ok(())
}

#[test]
fn invalid_properties_and_overflow() {
    for (rho, area, l) in [
        (-1.0, 1.0, 1.0),
        (1.0, 0.0, 1.0),
        (1.0, -1.0, 1.0),
        (1.0, 1.0, 0.0),
        (1.0, 1.0, -1.0),
    ] {
        assert_eq!(
            self_weight(rho, area, l, [0.0; 3]).unwrap_err().code,
            "ENG-E-0010"
        );
    }
    for (rho, area, l, g) in [
        (f64::NAN, 1.0, 1.0, [0.0; 3]),
        (1.0, f64::INFINITY, 1.0, [0.0; 3]),
        (1.0, 1.0, f64::NAN, [0.0; 3]),
        (1.0, 1.0, 1.0, [f64::NEG_INFINITY; 3]),
        (1e308, 1e308, 1.0, [1.0; 3]),
        (1e308, 1.0, 1e308, [1.0; 3]),
        (1.0, 1.0, 2.0, [1e308; 3]),
    ] {
        assert_eq!(self_weight(rho, area, l, g).unwrap_err().code, "ENG-E-0008");
    }
    for (rho, area, g, code) in [
        (-1.0, 1.0, [1.0; 3], "ENG-E-0010"),
        (1.0, 0.0, [1.0; 3], "ENG-E-0010"),
        (f64::NAN, 1.0, [1.0; 3], "ENG-E-0008"),
        (1.0, 1.0, [f64::NAN; 3], "ENG-E-0008"),
        (1e308, 1e308, [1.0; 3], "ENG-E-0008"),
    ] {
        assert_eq!(self_weight_per_length(rho, area, g).unwrap_err().code, code);
    }
}
