use super::*;
use engine_core::{catalog, Severity};

fn m(value: Real) -> Parameter {
    Parameter::new(value, Provenance::Measured)
}

fn a(value: Real) -> Parameter {
    Parameter::new(value, Provenance::Assumed)
}

fn layer(thickness: Real, gamma: Real, saturated: Option<Real>) -> Layer {
    Layer::new(thickness, Some(m(gamma)), saturated.map(m))
}

fn profile(layers: Vec<Layer>, zw: Real) -> StratifiedProfile {
    StratifiedProfile::new(layers, Some(m(zw)), Some(m(10.0))).unwrap()
}

fn assert_stresses(profile: &dyn StressProfile, z: Real, total: Real, pore: Real, effective: Real) {
    let result = profile.stresses_at(z).unwrap();
    assert_eq!(result.depth_m, z);
    for (actual, expected) in [
        (result.total_vertical_kpa, total),
        (result.pore_pressure_kpa, pore),
        (result.effective_vertical_kpa, effective),
    ] {
        assert!(
            (actual - expected).abs() <= 1e-10,
            "{actual} != {expected} at z={z}"
        );
    }
}

fn assert_diagnostic<T: std::fmt::Debug>(result: GeoResult<T>, code: &str, field: &str) {
    let diagnostic = result.unwrap_err();
    let info = catalog::lookup(code).unwrap();
    assert_eq!(diagnostic.code, info.code);
    assert_eq!(diagnostic.message, info.message);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(diagnostic.entities.is_empty());
    let hint = diagnostic.hint.unwrap();
    assert!(hint.contains(field), "{hint} does not identify {field}");
    assert!(
        hint.contains("catálogo"),
        "{hint} omits the catalog limitation"
    );
}

// Fuente y cálculos propios: provenance/ENG-GEO-001.md, casos A–E.
// Los números siguientes son literales obtenidos a mano, no por el código probado.
#[test]
fn homogeneous_dry_profile_and_inclusive_endpoints() {
    let p = profile(vec![layer(10.0, 18.0, None)], 10.0);
    assert_stresses(&p, 0.0, 0.0, 0.0, 0.0);
    assert_stresses(&p, 4.0, 72.0, 0.0, 72.0);
    assert_stresses(&p, 10.0, 180.0, 0.0, 180.0);
    assert_eq!(p.total_depth_m(), 10.0);
    assert!(p.stresses_at(10.0).unwrap().assumed_parameters.is_empty());
}

#[test]
fn groundwater_at_surface_uses_saturated_weight_and_hydrostatic_pressure() {
    let p = profile(vec![layer(10.0, 18.0, Some(20.0))], 0.0);
    assert_stresses(&p, 0.0, 0.0, 0.0, 0.0);
    assert_stresses(&p, 4.0, 80.0, 40.0, 40.0);
    assert_stresses(&p, 10.0, 200.0, 100.0, 100.0);
    assert_eq!(p.total_vertical_stress_at(4.0).unwrap(), 80.0);
    assert_eq!(p.pore_pressure_at(4.0).unwrap(), 40.0);
    assert_eq!(p.effective_vertical_stress_at(4.0).unwrap(), 40.0);
}

#[test]
fn groundwater_crossing_a_layer_splits_the_integral() {
    let p = profile(vec![layer(10.0, 18.0, Some(20.0))], 3.0);
    assert_stresses(&p, 2.0, 36.0, 0.0, 36.0);
    assert_stresses(&p, 3.0, 54.0, 0.0, 54.0);
    assert_stresses(&p, 5.0, 94.0, 20.0, 74.0);
    assert_stresses(&p, 10.0, 194.0, 70.0, 124.0);
    assert_stresses(&p, 3.0 - 1e-6, 53.999982, 0.0, 53.999982);
    assert_stresses(&p, 3.0 + 1e-6, 54.000020, 0.000010, 54.000010);
}

#[test]
fn multiple_layers_with_groundwater_at_interface() {
    let p = profile(
        vec![layer(2.0, 16.0, None), layer(3.0, 18.0, Some(21.0))],
        2.0,
    );
    assert_stresses(&p, 1.0, 16.0, 0.0, 16.0);
    assert_stresses(&p, 2.0, 32.0, 0.0, 32.0);
    assert_stresses(&p, 4.0, 74.0, 20.0, 54.0);
    assert_stresses(&p, 5.0, 95.0, 30.0, 65.0);
    assert_stresses(&p, 2.0 - 1e-6, 31.999984, 0.0, 31.999984);
    assert_stresses(&p, 2.0 + 1e-6, 32.000021, 0.000010, 32.000011);
}

#[test]
fn groundwater_below_profile_has_no_pore_pressure_or_saturated_requirement() {
    let p = profile(vec![layer(2.0, 16.0, None), layer(3.0, 18.0, None)], 8.0);
    assert_stresses(&p, 2.0, 32.0, 0.0, 32.0);
    assert_stresses(&p, 5.0, 86.0, 0.0, 86.0);
}

#[test]
fn continuity_at_dry_and_submerged_interfaces() {
    // Caso F: zw cruza la segunda de tres capas; tercera totalmente sumergida.
    let p = profile(
        vec![
            layer(2.0, 16.0, None),
            layer(3.0, 18.0, Some(21.0)),
            layer(4.0, 19.0, Some(22.0)),
        ],
        3.0,
    );
    assert_stresses(&p, 2.0, 32.0, 0.0, 32.0);
    assert_stresses(&p, 2.0 - 1e-6, 31.999984, 0.0, 31.999984);
    assert_stresses(&p, 2.0 + 1e-6, 32.000018, 0.0, 32.000018);
    assert_stresses(&p, 5.0, 92.0, 20.0, 72.0);
    assert_stresses(&p, 5.0 - 1e-6, 91.999979, 19.999990, 71.999989);
    assert_stresses(&p, 5.0 + 1e-6, 92.000022, 20.000010, 72.000012);
    assert_stresses(&p, 9.0, 180.0, 60.0, 120.0);
}

#[test]
fn out_of_range_depth_and_water_above_ground_are_rejected() {
    let p = profile(vec![layer(10.0, 18.0, None)], 12.0);
    for z in [-1.0, -Real::MIN_POSITIVE, 10.000001] {
        assert_diagnostic(p.stresses_at(z), "ENG-E-0010", "depth_m");
        assert_diagnostic(p.total_vertical_stress_at(z), "ENG-E-0010", "depth_m");
        assert_diagnostic(p.pore_pressure_at(z), "ENG-E-0010", "depth_m");
        assert_diagnostic(p.effective_vertical_stress_at(z), "ENG-E-0010", "depth_m");
    }
    assert_diagnostic(
        StratifiedProfile::new(
            vec![layer(2.0, 18.0, Some(20.0))],
            Some(m(-0.1)),
            Some(m(10.0)),
        ),
        "ENG-E-0010",
        "groundwater_depth_m",
    );
}

#[test]
fn empty_profile_and_nonpositive_thickness_or_weights_are_rejected() {
    assert_diagnostic(
        StratifiedProfile::new(vec![], Some(m(3.0)), Some(m(10.0))),
        "ENG-E-0010",
        "layers",
    );
    for value in [0.0, -0.0, -1.0] {
        for (input, field) in [
            (layer(value, 18.0, Some(20.0)), "thickness_m"),
            (layer(2.0, value, Some(20.0)), "gamma_kn_m3"),
            (layer(2.0, 18.0, Some(value)), "gamma_sat_kn_m3"),
        ] {
            assert_diagnostic(
                StratifiedProfile::new(vec![input], Some(m(0.0)), Some(m(10.0))),
                "ENG-E-0010",
                field,
            );
        }
        assert_diagnostic(
            StratifiedProfile::new(vec![layer(2.0, 18.0, None)], Some(m(3.0)), Some(m(value))),
            "ENG-E-0010",
            "water_unit_weight_kn_m3",
        );
    }
    // Un γsat aportado se valida aun cuando no se utilice.
    assert_diagnostic(
        StratifiedProfile::new(
            vec![layer(2.0, 18.0, Some(-1.0))],
            Some(m(3.0)),
            Some(m(10.0)),
        ),
        "ENG-E-0010",
        "gamma_sat_kn_m3",
    );
}

#[test]
fn nan_and_both_infinities_are_rejected_in_every_numeric_input() {
    let p = profile(vec![layer(2.0, 18.0, None)], 3.0);
    for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        assert_diagnostic(p.stresses_at(value), "ENG-E-0008", "depth_m");
        for (input, field) in [
            (layer(value, 18.0, Some(20.0)), "thickness_m"),
            (layer(2.0, value, Some(20.0)), "gamma_kn_m3"),
            (layer(2.0, 18.0, Some(value)), "gamma_sat_kn_m3"),
        ] {
            assert_diagnostic(
                StratifiedProfile::new(vec![input], Some(m(0.0)), Some(m(10.0))),
                "ENG-E-0008",
                field,
            );
        }
        assert_diagnostic(
            StratifiedProfile::new(vec![layer(2.0, 18.0, None)], Some(m(value)), Some(m(10.0))),
            "ENG-E-0008",
            "groundwater_depth_m",
        );
        assert_diagnostic(
            StratifiedProfile::new(vec![layer(2.0, 18.0, None)], Some(m(3.0)), Some(m(value))),
            "ENG-E-0008",
            "water_unit_weight_kn_m3",
        );
        assert_diagnostic(
            StratifiedProfile::new(
                vec![layer(2.0, 18.0, Some(value))],
                Some(m(3.0)),
                Some(m(10.0)),
            ),
            "ENG-E-0008",
            "gamma_sat_kn_m3",
        );
    }
}

#[test]
fn every_required_missing_parameter_is_identified_without_defaults() {
    for (input, field) in [
        (Layer::new(2.0, None, Some(m(20.0))), "gamma_kn_m3"),
        (layer(2.0, 18.0, None), "gamma_sat_kn_m3"),
    ] {
        assert_diagnostic(
            StratifiedProfile::new(vec![input], Some(m(0.0)), Some(m(10.0))),
            "ENG-E-0003",
            field,
        );
    }
    assert_diagnostic(
        StratifiedProfile::new(vec![layer(2.0, 18.0, None)], None, Some(m(10.0))),
        "ENG-E-0003",
        "groundwater_depth_m",
    );
    assert_diagnostic(
        StratifiedProfile::new(vec![layer(2.0, 18.0, None)], Some(m(3.0)), None),
        "ENG-E-0003",
        "water_unit_weight_kn_m3",
    );
    // γsat también es obligatorio cuando zw cruza una capa.
    assert_diagnostic(
        StratifiedProfile::new(vec![layer(2.0, 18.0, None)], Some(m(1.0)), Some(m(10.0))),
        "ENG-E-0003",
        "gamma_sat_kn_m3",
    );
    // Validación del perfil entero, aun si se pensaba consultar solo la capa 0.
    assert_diagnostic(
        StratifiedProfile::new(
            vec![layer(2.0, 16.0, None), layer(3.0, 18.0, None)],
            Some(m(2.0)),
            Some(m(10.0)),
        ),
        "ENG-E-0003",
        "layers[1].gamma_sat_kn_m3",
    );
    // γ sigue siendo obligatorio para una capa totalmente sumergida.
    assert_diagnostic(
        StratifiedProfile::new(
            vec![Layer::new(2.0, None, Some(m(20.0)))],
            Some(m(0.0)),
            Some(m(10.0)),
        ),
        "ENG-E-0003",
        "gamma_kn_m3",
    );
}

#[test]
fn assumed_parameters_report_all_provided_data_in_stable_order() {
    let p = StratifiedProfile::new(
        vec![
            Layer::new(2.0, Some(a(16.0)), Some(a(20.0))),
            Layer::new(3.0, Some(m(18.0)), Some(a(21.0))),
        ],
        Some(a(2.0)),
        Some(a(10.0)),
    )
    .unwrap();
    let expected = vec![
        ParameterId::LayerGamma { layer_index: 0 },
        ParameterId::LayerGammaSat { layer_index: 0 }, // aportado, aunque no usado
        ParameterId::LayerGammaSat { layer_index: 1 },
        ParameterId::GroundwaterDepth,
        ParameterId::WaterUnitWeight,
    ];
    for z in [0.0, 1.0, 5.0] {
        assert_eq!(p.stresses_at(z).unwrap().assumed_parameters, expected);
    }
    assert_stresses(&p, 5.0, 95.0, 30.0, 65.0);
    // Modificar el resultado no modifica el perfil.
    p.stresses_at(0.0).unwrap().assumed_parameters.clear();
    assert_eq!(p.stresses_at(0.0).unwrap().assumed_parameters, expected);
    let all_measured = profile(vec![layer(2.0, 18.0, Some(20.0))], 0.0);
    assert!(all_measured
        .stresses_at(2.0)
        .unwrap()
        .assumed_parameters
        .is_empty());
    // Assumed también identifica γ de una capa totalmente sumergida (caso G).
    let submerged = StratifiedProfile::new(
        vec![Layer::new(2.0, Some(a(18.0)), Some(m(20.0)))],
        Some(m(0.0)),
        Some(m(10.0)),
    )
    .unwrap();
    assert_stresses(&submerged, 2.0, 40.0, 20.0, 20.0);
    assert_eq!(
        submerged.stresses_at(2.0).unwrap().assumed_parameters,
        [ParameterId::LayerGamma { layer_index: 0 }]
    );
}

#[test]
fn saturated_weight_need_not_exceed_gamma_and_effective_stress_is_not_clamped() {
    // Caso H, deliberadamente fuera de valores físicos usuales: no se imponen.
    let p = profile(vec![layer(2.0, 18.0, Some(8.0))], 0.0);
    assert_stresses(&p, 2.0, 16.0, 20.0, -4.0);
}

#[test]
fn arithmetic_overflow_and_unrepresentable_layer_geometry_are_rejected() {
    assert_diagnostic(
        StratifiedProfile::new(
            vec![layer(Real::MAX, 1.0, None), layer(Real::MAX, 1.0, None)],
            Some(m(Real::MAX)),
            Some(m(10.0)),
        ),
        "ENG-E-0008",
        "bottom_m",
    );
    assert_diagnostic(
        StratifiedProfile::new(
            vec![layer(1e20, 1.0, None), layer(1.0, 1.0, None)],
            Some(m(1e21)),
            Some(m(10.0)),
        ),
        "ENG-E-0010",
        "thickness_m",
    );
    let total_overflow = profile(vec![layer(2.0, Real::MAX, None)], 3.0);
    assert_stresses(&total_overflow, 0.0, 0.0, 0.0, 0.0);
    assert_diagnostic(total_overflow.stresses_at(2.0), "ENG-E-0008", "σv");
    let saturated_overflow = profile(vec![layer(2.0, 1.0, Some(Real::MAX))], 0.0);
    assert_diagnostic(saturated_overflow.stresses_at(2.0), "ENG-E-0008", "σv");
    let sum_overflow = profile(vec![layer(1.0, 1e308, None), layer(1.0, 1e308, None)], 3.0);
    assert_diagnostic(sum_overflow.stresses_at(2.0), "ENG-E-0008", "layers[1]");
    let pore_overflow = StratifiedProfile::new(
        vec![layer(2.0, 1.0, Some(1.0))],
        Some(m(0.0)),
        Some(m(Real::MAX)),
    )
    .unwrap();
    assert_diagnostic(pore_overflow.stresses_at(2.0), "ENG-E-0008", "u (kPa)");
}

#[test]
fn trait_is_interchangeable_and_contract_is_versioned() {
    let p = profile(vec![layer(2.0, 18.0, None)], 3.0);
    let interchangeable: Box<dyn StressProfile> = Box::new(p.clone());
    assert_eq!(
        interchangeable.contract(),
        ContractId::new("stress-profile", 1, 0)
    );
    assert!(interchangeable
        .contract()
        .is_compatible_with(&ContractId::new("stress-profile", 1, 7)));
    assert!(!interchangeable
        .contract()
        .is_compatible_with(&ContractId::new("stress-profile", 2, 0)));
    assert_eq!(
        interchangeable.stresses_at(2.0).unwrap(),
        p.stresses_at(2.0).unwrap()
    );
}
