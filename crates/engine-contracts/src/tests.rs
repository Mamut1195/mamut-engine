use crate::{conformance::*, examples::*, *};
use std::collections::BTreeMap;

fn samples() -> Vec<Vec<Real>> {
    vec![
        vec![0.0; 6],
        vec![0.001, -0.002, 0.01, 0.004, 0.003, -0.02],
        vec![-0.01, 0.02, 0.0, 0.03, -0.01, 0.0],
    ]
}
fn params_json() -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("E".into(), JsonValue::Number(200.0)),
        ("A".into(), JsonValue::Number(3.0)),
        ("density".into(), JsonValue::Number(2.0)),
    ]))
}
fn assert_clean(diagnostics: Vec<Diagnostic>) {
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}
fn detected(diagnostics: Vec<Diagnostic>, text: &str) {
    assert!(
        diagnostics
            .iter()
            .any(|d| d.hint.as_ref().is_some_and(|h| h.contains(text))),
        "No detectó {text}: {diagnostics:#?}"
    );
}

#[test]
fn every_physical_example_and_factory_passes_conformance() {
    assert_clean(check_uniaxial_law(
        &LinearElastic::new(200e9).unwrap(),
        &[-0.01, 0.0, 0.02],
        1e-6,
    ));
    assert_clean(check_section_law(
        &ElasticSection2D::new(600.0, 10.0).unwrap(),
        &[vec![0.0; 2], vec![0.01, 0.03], vec![-0.01, -0.03]],
        1e-6,
    ));
    let coords = [[1.0, 2.0, 0.0], [4.0, 6.0, 0.0]];
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    assert_clean(check_element(
        &LinearBar2D::new(200.0, 3.0, 2.0).unwrap(),
        &ctx,
        &samples(),
        1e-6,
    ));
    let nodes = [(NodeId(1), coords[0]), (NodeId(2), coords[1])];
    assert_clean(check_element_factory(
        &BarFactory,
        &params_json(),
        &ModelView {
            dim: Dim::D2,
            nodes: &nodes,
        },
        &ctx,
        &samples(),
        1e-6,
    ));
}

#[test]
fn analytic_hooke_section_and_ea_over_l() {
    // Derivación propia; referencia bibliográfica y limitación en la bitácora.
    assert_eq!(
        LinearElastic::new(200.0)
            .unwrap()
            .respond(0.02, &[])
            .unwrap()
            .stress,
        4.0
    );
    let section = ElasticSection2D::new(600.0, 10.0)
        .unwrap()
        .respond(&[0.02, -0.03], &[])
        .unwrap();
    assert_eq!(section.resultants, vec![12.0, -0.3]);
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &[[0.0; 3], [2.0, 0.0, 0.0]],
        params: &params,
    };
    let bar = LinearBar2D::new(200.0, 3.0, 2.0).unwrap();
    let r = bar
        .respond(&ctx, &[0.0, 0.0, 0.0, 0.01, 0.0, 0.0], &[])
        .unwrap();
    assert_eq!(r.tangent.get(0, 0), Some(300.0)); // EA/L = 200×3/2.
    assert_eq!(r.internal_force, vec![-3.0, 0.0, 0.0, 3.0, 0.0, 0.0]);
    let output = bar
        .recover(
            &ctx,
            &[0.0, 0.0, 0.0, 0.01, 0.0, 0.0],
            &[],
            &[],
            &[0.0, 0.5, 1.0],
        )
        .unwrap();
    assert_eq!(output.end_forces_local, vec![-3.0, 0.0, 0.0, 3.0, 0.0, 0.0]);
    assert!(output.stations.iter().all(|s| s.forces_local[0] == 3.0));
    for kind in [MassKind::Lumped, MassKind::Consistent] {
        let mass = bar.mass(&ctx, kind).unwrap().unwrap();
        for axis in [0, 1] {
            let total: Real = [axis, axis + 3]
                .iter()
                .flat_map(|i| [axis, axis + 3].into_iter().map(move |j| (*i, j)))
                .map(|(i, j)| mass.get(i, j).unwrap())
                .sum();
            assert_eq!(total, 12.0); // ρAL=2×3×2 por cada dirección.
        }
    }
}

#[test]
fn distributed_load_recovery_satisfies_equilibrium_and_signs() {
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &[[0.0; 3], [2.0, 0.0, 0.0]],
        params: &params,
    };
    let bar = LinearBar2D::new(200.0, 3.0, 0.0).unwrap();
    let load = MemberLoad {
        force_per_length: [4.0, 0.0, 0.0],
        moment_per_length: [0.0; 3],
    };
    assert_eq!(
        bar.load_vector(&ctx, &load).unwrap(),
        vec![4.0, 0.0, 0.0, 4.0, 0.0, 0.0]
    );
    let r = bar
        .recover(&ctx, &[0.0; 6], &[], &[load], &[0.0, 0.5, 1.0])
        .unwrap();
    assert_eq!(r.end_forces_local, vec![-4.0, 0.0, 0.0, -4.0, 0.0, 0.0]);
    assert_eq!(
        r.stations
            .iter()
            .map(|s| s.forces_local[0])
            .collect::<Vec<_>>(),
        vec![4.0, 0.0, -4.0]
    );
    assert!(bar.mass(&ctx, MassKind::Lumped).unwrap().is_none());
    assert!(bar
        .geometric_stiffness(&ctx, &[0.0; 6], &[])
        .unwrap()
        .is_none());
}

#[test]
fn system_sink_analysis_control_examples_pass_their_kits() {
    let mut sys = SpringSystem::new(12.0).unwrap();
    assert_clean(check_system_view(
        &mut sys,
        &[vec![0.0], vec![0.1], vec![-0.2]],
        &CaseRef("example.case".into()),
        1e-6,
    ));
    assert_eq!(sys.accepted(), -0.2);
    let mut sink = MemorySink::default();
    assert_clean(check_result_sink(&mut sink));
    assert_eq!(sink.entries.len(), 1); // Entradas rechazadas no dejan escritura parcial.
    let ctl = RecordingControl::default();
    assert_clean(check_analysis_method(
        &PrescribedStep,
        &mut sys,
        &JsonValue::Array(vec![JsonValue::Number(0.25)]),
        &mut sink,
        &ctl,
    ));
    assert_eq!(sys.accepted(), 0.25);
    assert_eq!(sink.entries.len(), 2);
    assert_clean(check_run_control(&ctl.events()));
    ctl.cancel();
    assert_clean(check_analysis_method(
        &PrescribedStep,
        &mut sys,
        &JsonValue::Null,
        &mut sink,
        &ctl,
    ));
    assert_eq!(sys.accepted(), 0.25);
    assert_eq!(sink.entries.len(), 2);
}

struct BadUniaxial(u8);
impl UniaxialLaw for BadUniaxial {
    fn kind(&self) -> &'static str {
        "test.bad_law"
    }
    fn state_len(&self) -> usize {
        1
    }
    fn initial_state(&self) -> Vec<Real> {
        if self.0 == 4 {
            vec![]
        } else {
            vec![0.0]
        }
    }
    fn initial_tangent(&self) -> Real {
        2.0
    }
    fn respond(&self, x: Real, s: &[Real]) -> EResult<UniaxialResponse> {
        Ok(UniaxialResponse {
            stress: match self.0 {
                1 => Real::NAN,
                2 => Real::INFINITY,
                3 => 2.0 * x + s[0],
                _ => 2.0 * x,
            },
            tangent: if self.0 == 0 { 1.0 } else { 2.0 },
            trial_state: if self.0 == 5 { vec![0.0; 2] } else { vec![x] },
        })
    }
}
#[test]
fn kit_detects_wrong_derivative_nonfinite_and_accepted_history() {
    detected(
        check_uniaxial_law(&BadUniaxial(0), &[0.1], 1e-5),
        "error relativo",
    );
    for mode in [1, 2] {
        detected(
            check_uniaxial_law(&BadUniaxial(mode), &[0.1], 1e-5),
            "finitos",
        );
    }
    detected(
        check_uniaxial_law(&BadUniaxial(3), &[0.1], 1e-5),
        "Tensión aceptada",
    );
    for mode in [4, 5] {
        detected(
            check_uniaxial_law(&BadUniaxial(mode), &[0.1], 1e-5),
            "Longitud esperada",
        );
    }
}

struct HistoryLaw;
impl UniaxialLaw for HistoryLaw {
    fn kind(&self) -> &'static str {
        "test.history"
    }
    fn state_len(&self) -> usize {
        2
    }
    fn initial_state(&self) -> Vec<Real> {
        vec![0.0, 0.0]
    }
    fn initial_tangent(&self) -> Real {
        2.0
    }
    fn respond(&self, x: Real, s: &[Real]) -> EResult<UniaxialResponse> {
        vector_for_test(s, 2)?;
        let stress = s[1] + 2.0 * (x - s[0]);
        Ok(UniaxialResponse {
            stress,
            tangent: 2.0,
            trial_state: vec![x, stress],
        })
    }
}
fn vector_for_test(s: &[Real], n: usize) -> EResult<()> {
    size(s.len(), n)?;
    finite(s)
}
#[test]
fn kit_exercises_nonempty_history_on_a_committed_path() {
    assert_clean(check_uniaxial_law(
        &HistoryLaw,
        &[0.1, 0.3, -0.1, 0.3],
        1e-6,
    ));
}

struct BadSection(u8);
impl SectionLaw for BadSection {
    fn kind(&self) -> &'static str {
        "test.bad_section"
    }
    fn contract(&self) -> ContractId {
        ContractId::new("section", if self.0 == 2 { 2 } else { 1 }, 0)
    }
    fn dim(&self) -> Dim {
        Dim::D2
    }
    fn state_len(&self) -> usize {
        0
    }
    fn initial_state(&self) -> Vec<Real> {
        vec![]
    }
    fn respond(&self, e: &[Real], _: &[Real]) -> EResult<SectionResponse> {
        Ok(SectionResponse {
            resultants: if self.0 == 1 {
                vec![]
            } else {
                vec![e[0] + e[1], e[1]]
            },
            tangent: DMat::new(2, 2, vec![1.0, 1.0, 0.0, 1.0])?,
            trial_state: vec![],
        })
    }
}
#[test]
fn section_kit_detects_asymmetry_shape_and_contract_version() {
    detected(
        check_section_law(&BadSection(0), &[vec![0.1, 0.2]], 1e-5),
        "Simetría",
    );
    detected(
        check_section_law(&BadSection(1), &[vec![0.1, 0.2]], 1e-5),
        "Longitud esperada",
    );
    detected(
        check_section_law(&BadSection(2), &[vec![0.1, 0.2]], 1e-5),
        "Contrato",
    );
}

struct BadElement {
    bar: LinearBar2D,
    mode: u8,
}
impl Element for BadElement {
    fn kind(&self) -> &'static str {
        "test.bad_element"
    }
    fn contract(&self) -> ContractId {
        self.bar.contract()
    }
    fn n_nodes(&self) -> usize {
        2
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        Dim::D2.dofs()
    }
    fn respond(&self, c: &ElementCtx<'_>, u: &[Real], s: &[Real]) -> EResult<ElementResponse> {
        let mut r = self.bar.respond(c, u, s)?;
        if self.mode == 0 {
            r.internal_force[0] += 1.0;
        } // Fuerza espuria incluso en cuerpo rígido.
        if self.mode == 1 {
            r.tangent = DMat::zeros(6, 6)?;
        } // Derivada falsa.
        if self.mode == 2 && (c.coords[1][1] - c.coords[0][1]).abs() > 1e-12 {
            r.internal_force.iter_mut().for_each(|v| *v *= 2.0);
            r.tangent = DMat::new(6, 6, r.tangent.values().iter().map(|v| v * 2.0).collect())?;
            // Pasa FD y modos, falla covarianza.
        }
        if self.mode == 4 {
            r.internal_force.pop();
        }
        Ok(r)
    }
    fn mass(&self, c: &ElementCtx<'_>, k: MassKind) -> EResult<Option<DMat>> {
        if self.mode == 3 {
            let mut m = vec![0.0; 36];
            m[1] = 1.0;
            return Ok(Some(DMat::new(6, 6, m)?));
        }
        self.bar.mass(c, k)
    }
    fn geometric_stiffness(
        &self,
        c: &ElementCtx<'_>,
        u: &[Real],
        s: &[Real],
    ) -> EResult<Option<DMat>> {
        self.bar.geometric_stiffness(c, u, s)
    }
    fn load_vector(&self, c: &ElementCtx<'_>, l: &MemberLoad) -> EResult<Vec<Real>> {
        self.bar.load_vector(c, l)
    }
    fn recover(
        &self,
        c: &ElementCtx<'_>,
        u: &[Real],
        s: &[Real],
        l: &[MemberLoad],
        x: &[Real],
    ) -> EResult<ElementOutput> {
        self.bar.recover(c, u, s, l, x)
    }
}
#[test]
fn element_kit_detects_rigid_derivative_rotation_mass_and_shape_defects() {
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &[[0.0; 3], [2.0, 0.0, 0.0]],
        params: &params,
    };
    for (mode, message) in [
        (0, "Modo rígido"),
        (1, "Tangente inconsistente"),
        (2, "Covarianza"),
        (3, "Simetría masa"),
        (4, "Longitud esperada"),
    ] {
        let bad = BadElement {
            bar: LinearBar2D::new(200.0, 3.0, 2.0).unwrap(),
            mode,
        };
        detected(check_element(&bad, &ctx, &samples(), 1e-6), message);
    }
}

#[test]
fn empty_samples_invalid_steps_and_wrong_inputs_never_pass() {
    let law = LinearElastic::new(2.0).unwrap();
    for h in [0.0, -1.0, Real::NAN, Real::INFINITY] {
        detected(check_uniaxial_law(&law, &[0.0], h), "incremento");
    }
    detected(check_uniaxial_law(&law, &[], 1e-6), "muestras");
    // T2 (J4): h finito pero absurdo (redondeo o truncamiento) tampoco pasa.
    for h in [1e-12, 1e-10, 0.1, 1.0, 1e6] {
        detected(check_uniaxial_law(&law, &[0.0], h), "fuera de");
        detected(
            check_section_law(&SpatialSection, &[vec![0.0; 4]], h),
            "fuera de",
        );
    }
    for h in [
        crate::conformance::FD_STEP_MIN,
        1e-6,
        crate::conformance::FD_STEP_MAX,
    ] {
        assert_clean(check_uniaxial_law(&law, &[0.1], h));
    }
    detected(
        check_section_law(
            &ElasticSection2D::new(2.0, 3.0).unwrap(),
            &[vec![0.0]],
            1e-6,
        ),
        "Longitud",
    );
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &[[0.0; 3]],
        params: &params,
    };
    detected(
        check_element(
            &LinearBar2D::new(2.0, 3.0, 0.0).unwrap(),
            &ctx,
            &samples(),
            1e-6,
        ),
        "Contexto",
    );
}

#[test]
fn examples_reject_nonfinite_overflow_wrong_shapes_geometry_and_loads() {
    for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        assert!(LinearElastic::new(value).is_err());
        assert!(LinearBar2D::new(1.0, 1.0, value).is_err());
        assert!(ElasticSection2D::new(1.0, value).is_err());
        assert!(LinearElastic::new(2.0)
            .unwrap()
            .respond(value, &[])
            .is_err());
    }
    assert!(LinearElastic::new(Real::MAX)
        .unwrap()
        .respond(2.0, &[])
        .is_err());
    assert!(LinearElastic::new(1.0)
        .unwrap()
        .respond(0.0, &[0.0])
        .is_err());
    assert!(ElasticSection2D::new(1.0, 1.0)
        .unwrap()
        .respond(&[0.0], &[])
        .is_err());
    assert!(LinearBar2D::new(0.0, 1.0, 1.0).is_err());
    assert!(LinearBar2D::new(1.0, -1.0, 1.0).is_err());
    assert!(LinearBar2D::new(1.0, 1.0, -1.0).is_err());
    let bar = LinearBar2D::new(1.0, 1.0, 0.0).unwrap();
    let params = ElementParams::default();
    for coords in [[[0.0; 3]; 2], [[0.0; 3], [1.0, 0.0, 1.0]]] {
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        assert!(bar.respond(&ctx, &[0.0; 6], &[]).is_err());
    }
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &[[0.0; 3], [1.0, 0.0, 0.0]],
        params: &params,
    };
    for load in [
        MemberLoad {
            force_per_length: [0.0, 1.0, 0.0],
            moment_per_length: [0.0; 3],
        },
        MemberLoad {
            force_per_length: [0.0; 3],
            moment_per_length: [0.0, 0.0, 1.0],
        },
    ] {
        assert!(bar.load_vector(&ctx, &load).is_err());
        assert!(bar.recover(&ctx, &[0.0; 6], &[], &[load], &[0.5]).is_err());
    }
    for x in [-0.1, 1.1, Real::NAN] {
        assert!(bar.recover(&ctx, &[0.0; 6], &[], &[], &[x]).is_err());
    }
}

#[test]
fn factory_validates_schema_and_dimension_without_json_dependency() {
    let model = ModelView {
        dim: Dim::D2,
        nodes: &[],
    };
    for invalid in [
        JsonValue::Null,
        JsonValue::Array(vec![]),
        JsonValue::Object(BTreeMap::new()),
    ] {
        assert!(BarFactory.build(&invalid, &model).is_err());
    }
    let JsonValue::Object(mut fields) = params_json() else {
        unreachable!()
    };
    fields.insert("E".into(), JsonValue::String("200".into()));
    assert!(BarFactory
        .build(&JsonValue::Object(fields.clone()), &model)
        .is_err());
    fields.insert("E".into(), JsonValue::Number(Real::NAN));
    assert!(BarFactory
        .build(&JsonValue::Object(fields), &model)
        .is_err());
    assert!(BarFactory
        .build(
            &params_json(),
            &ModelView {
                dim: Dim::D3,
                nodes: &[]
            }
        )
        .is_err());
}

struct BadSink;
impl ResultSink for BadSink {
    fn put(&mut self, _: ResultKey, _: ResultValue) -> EResult<()> {
        Ok(())
    }
}
#[test]
fn sink_and_progress_kits_detect_defective_implementations() {
    detected(check_result_sink(&mut BadSink), "no rechazada");
    for events in [
        vec![],
        vec![(Real::NAN, "nota".into())],
        vec![(1.1, "nota".into())],
        vec![(0.5, "a".into()), (0.4, "b".into())],
        vec![(0.5, "".into())],
    ] {
        assert!(!check_run_control(&events).is_empty());
    }
}

struct FailingSink;
impl ResultSink for FailingSink {
    fn put(&mut self, _: ResultKey, _: ResultValue) -> EResult<()> {
        Err(EngineError::new("ENG-E-0012", "Destino fallido."))
    }
}
#[test]
fn analysis_errors_and_cancel_preserve_last_accepted_state() {
    let mut sys = SpringSystem::new(2.0).unwrap();
    sys.accept(&[0.1]).unwrap();
    let ctl = RecordingControl::default();
    let mut sink = MemorySink::default();
    for settings in [
        JsonValue::Null,
        JsonValue::Array(vec![]),
        JsonValue::Array(vec![JsonValue::Number(Real::NAN)]),
    ] {
        assert!(PrescribedStep
            .run(&mut sys, &settings, &mut sink, &ctl)
            .is_err());
        assert_eq!(sys.accepted(), 0.1);
        assert!(sink.entries.is_empty());
    }
    let settings = JsonValue::Array(vec![JsonValue::Number(0.2)]);
    assert_eq!(
        PrescribedStep
            .run(&mut sys, &settings, &mut FailingSink, &ctl)
            .unwrap_err()
            .code,
        "ENG-E-0012"
    );
    assert_eq!(sys.accepted(), 0.2); // Falló E/S después de aceptar; rollback conserva ese último aceptado.
    ctl.cancel();
    assert_eq!(
        PrescribedStep
            .run(&mut sys, &settings, &mut sink, &ctl)
            .unwrap()
            .status,
        Status::Cancelled
    );
    assert_eq!(sys.accepted(), 0.2);
}

#[test]
fn matrices_only_store_valid_finite_sized_data() {
    assert!(DMat::new(2, 2, vec![0.0; 3]).is_err());
    assert!(DMat::new(1, 1, vec![Real::NAN]).is_err());
    assert!(DMat::zeros(usize::MAX, 2).is_err());
    assert!(SymMatrix::new(DMat::new(2, 2, vec![1.0, 2.0, 0.0, 1.0]).unwrap()).is_err());
    assert!(SymMatrix::new(DMat::zeros(1, 2).unwrap()).is_err());
    assert!(SparsityPattern::new(2, vec![true, true, false, true]).is_err());
    assert!(SparsityPattern::new(2, vec![true]).is_err());
    assert_eq!(DMat::zeros(1, 1).unwrap().get(1, 0), None);
}

// Formulación espacial independiente para cubrir los seis modos del kit genérico.
struct SpatialBar {
    locked_component: Option<usize>,
}
impl Element for SpatialBar {
    fn kind(&self) -> &'static str {
        "test.spatial_bar"
    }
    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    }
    fn n_nodes(&self) -> usize {
        2
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        Dim::D3.dofs()
    }
    fn respond(&self, c: &ElementCtx<'_>, u: &[Real], s: &[Real]) -> EResult<ElementResponse> {
        vector_for_test(u, 12)?;
        vector_for_test(s, 0)?;
        let delta: [Real; 3] = std::array::from_fn(|i| c.coords[1][i] - c.coords[0][i]);
        let length = delta[0].hypot(delta[1]).hypot(delta[2]);
        let mut b = [0.0; 12];
        for i in 0..3 {
            b[i] = -delta[i] / length;
            b[6 + i] = delta[i] / length;
        }
        let axial: Real = b.iter().zip(u).map(|(bi, ui)| bi * ui).sum();
        let mut force: Vec<_> = b.iter().map(|bi| bi * axial).collect();
        let mut k: Vec<_> = b
            .iter()
            .flat_map(|bi| b.iter().map(move |bj| bi * bj))
            .collect();
        if let Some(i) = self.locked_component {
            force[i] += u[i];
            k[12 * i + i] += 1.0;
        }
        Ok(ElementResponse {
            internal_force: force,
            tangent: DMat::new(12, 12, k)?,
            trial_state: vec![],
        })
    }
    fn mass(&self, _: &ElementCtx<'_>, _: MassKind) -> EResult<Option<DMat>> {
        Ok(None)
    }
    fn geometric_stiffness(
        &self,
        _: &ElementCtx<'_>,
        _: &[Real],
        _: &[Real],
    ) -> EResult<Option<DMat>> {
        Ok(None)
    }
    fn load_vector(&self, _: &ElementCtx<'_>, _: &MemberLoad) -> EResult<Vec<Real>> {
        Ok(vec![0.0; 12])
    }
    fn recover(
        &self,
        _: &ElementCtx<'_>,
        _: &[Real],
        _: &[Real],
        _: &[MemberLoad],
        x: &[Real],
    ) -> EResult<ElementOutput> {
        Ok(ElementOutput {
            end_forces_local: vec![0.0; 12],
            stations: x
                .iter()
                .map(|x| StationResult {
                    position: *x,
                    forces_local: [0.0; 6],
                })
                .collect(),
        })
    }
}
#[test]
fn spatial_kit_checks_each_of_six_rigid_modes_and_three_rotation_axes() {
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D3,
        coords: &[[1.0, -2.0, 3.0], [4.0, 6.0, -1.0]],
        params: &params,
    };
    let samples = vec![
        vec![0.0; 12],
        (0..12).map(|i| (i as Real) * 0.003 - 0.01).collect(),
    ];
    assert_clean(check_element(
        &SpatialBar {
            locked_component: None,
        },
        &ctx,
        &samples,
        1e-6,
    ));
    for component in 0..6 {
        detected(
            check_element(
                &SpatialBar {
                    locked_component: Some(component),
                },
                &ctx,
                &samples,
                1e-6,
            ),
            &format!("Modo rígido {component}"),
        );
    }
}

struct SpatialSection;
impl SectionLaw for SpatialSection {
    fn kind(&self) -> &'static str {
        "test.spatial_section"
    }
    fn contract(&self) -> ContractId {
        ContractId::new("section", 1, 0)
    }
    fn dim(&self) -> Dim {
        Dim::D3
    }
    fn state_len(&self) -> usize {
        0
    }
    fn initial_state(&self) -> Vec<Real> {
        vec![]
    }
    fn respond(&self, e: &[Real], s: &[Real]) -> EResult<SectionResponse> {
        vector_for_test(e, 4)?;
        vector_for_test(s, 0)?;
        let mut k = vec![0.0; 16];
        let mut resultants = vec![];
        for i in 0..4 {
            let stiffness = (i + 1) as Real;
            k[4 * i + i] = stiffness;
            resultants.push(stiffness * e[i]);
        }
        Ok(SectionResponse {
            resultants,
            tangent: DMat::new(4, 4, k)?,
            trial_state: vec![],
        })
    }
}
#[test]
fn section_kit_supports_four_spatial_generalized_components() {
    assert_clean(check_section_law(
        &SpatialSection,
        &[vec![0.0; 4], vec![0.02, -0.03, 0.04, -0.05]],
        1e-6,
    ));
}

struct BadSystem {
    inner: SpringSystem,
    omit_force: bool,
}
impl SystemView for BadSystem {
    fn n_free_dofs(&self) -> usize {
        self.inner.n_free_dofs()
    }
    fn pattern(&self) -> &SparsityPattern {
        self.inner.pattern()
    }
    fn tangent(&mut self, _: &[Real], out: &mut SymMatrix) -> EResult<()> {
        *out = SymMatrix::zeros(1)?;
        Ok(())
    }
    fn internal_force(&mut self, u: &[Real], out: &mut [Real]) -> EResult<()> {
        if self.omit_force {
            Ok(())
        } else {
            self.inner.internal_force(u, out)
        }
    }
    fn mass(&mut self, k: MassKind, out: &mut SymMatrix) -> EResult<()> {
        self.inner.mass(k, out)
    }
    fn geometric(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()> {
        self.inner.geometric(u, out)
    }
    fn external_force(&self, c: &CaseRef, f: Real, out: &mut [Real]) -> EResult<()> {
        self.inner.external_force(c, f, out)
    }
    fn accept(&mut self, u: &[Real]) -> EResult<()> {
        self.inner.accept(u)
    }
    fn rollback(&mut self) {
        self.inner.rollback()
    }
    fn dof_label(&self, d: DofIndex) -> (NodeId, Dof) {
        self.inner.dof_label(d)
    }
}
#[test]
fn system_kit_detects_wrong_tangent_and_incomplete_output_overwrite() {
    for (omit_force, message) in [(false, "error relativo"), (true, "finitos")] {
        let mut sys = BadSystem {
            inner: SpringSystem::new(12.0).unwrap(),
            omit_force,
        };
        detected(
            check_system_view(
                &mut sys,
                &[vec![0.1]],
                &CaseRef("example.case".into()),
                1e-6,
            ),
            message,
        );
    }
}

struct BadAnalysis(u8);
impl AnalysisMethod for BadAnalysis {
    fn id(&self) -> &'static str {
        "test.bad_analysis"
    }
    fn version(&self) -> (u16, u16, u16) {
        (1, 0, 0)
    }
    fn contract(&self) -> ContractId {
        ContractId::new("analysis", 1, 0)
    }
    fn settings_schema(&self) -> String {
        "{}".into()
    }
    fn requires(&self) -> Requirements {
        Requirements::default()
    }
    fn run(
        &self,
        _: &mut dyn SystemView,
        _: &JsonValue,
        _: &mut dyn ResultSink,
        _: &dyn RunControl,
    ) -> EResult<AnalysisOutcome> {
        let step = StepReport {
            step: 1,
            load_factor: 1.0,
            iterations: 1,
            residual_norm: if self.0 == 1 { 1.0 } else { 0.0 },
            displacement_norm: 0.0,
            energy_norm: 0.0,
            converged: true,
            tolerances: Tolerances {
                force_rel: 1e-8,
                disp_rel: 1e-8,
                energy_rel: 1e-8,
            },
        };
        Ok(AnalysisOutcome {
            status: if self.0 == 2 {
                Status::NotConverged
            } else {
                Status::Completed
            },
            report: ConvergenceReport {
                steps: if self.0 == 0 { vec![] } else { vec![step] },
            },
            diagnostics: vec![],
        })
    }
}
#[test]
fn analysis_kit_detects_missing_norms_false_convergence_and_ignored_cancel() {
    let mut sys = SpringSystem::new(1.0).unwrap();
    let mut sink = MemorySink::default();
    let ctl = RecordingControl::default();
    for mode in 0..3 {
        assert!(!check_analysis_method(
            &BadAnalysis(mode),
            &mut sys,
            &JsonValue::Null,
            &mut sink,
            &ctl
        )
        .is_empty());
    }
    ctl.cancel();
    detected(
        check_analysis_method(&BadAnalysis(3), &mut sys, &JsonValue::Null, &mut sink, &ctl),
        "cancelación",
    );
}

#[test]
fn result_validation_covers_nested_nonfinite_ranges_and_report_consistency() {
    let key = ResultKey {
        block: "members".into(),
        entity: Some("1".into()),
        field: "forces".into(),
    };
    for (position, force) in [(0.5, Real::INFINITY), (-0.1, 0.0), (Real::NAN, 0.0)] {
        assert!(validate_result(
            &key,
            &ResultValue::Element(ElementOutput {
                end_forces_local: vec![0.0],
                stations: vec![StationResult {
                    position,
                    forces_local: [force; 6]
                }]
            })
        )
        .is_err());
    }
    let mut step = StepReport {
        step: 1,
        load_factor: 1.0,
        iterations: 1,
        residual_norm: 0.0,
        displacement_norm: 0.0,
        energy_norm: 0.0,
        converged: true,
        tolerances: Tolerances {
            force_rel: 1e-8,
            disp_rel: 1e-8,
            energy_rel: 1e-8,
        },
    };
    step.tolerances.force_rel = 0.0;
    assert!(validate_result(
        &key,
        &ResultValue::Convergence(ConvergenceReport {
            steps: vec![step.clone()]
        })
    )
    .is_err());
    step.tolerances.force_rel = 1e-8;
    step.residual_norm = 1.0;
    assert!(validate_result(
        &key,
        &ResultValue::Convergence(ConvergenceReport { steps: vec![step] })
    )
    .is_err());
}
