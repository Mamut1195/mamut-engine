//! Adapt imports, parameters and samples from the method sheet.
use engine_contracts::{conformance::check_uniaxial_law, UniaxialLaw};
use engine_core::EResult;
use engine_laws::uniaxial::{Metodo, MetodoFactory, MetodoParams, UniaxialFactory, METODO};

#[test]
fn conformidad_y_fabrica() -> EResult<()> {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<Metodo>();
    send_sync::<MetodoFactory>();
    let params = MetodoParams { e: 200.0 }; // Replace from the method sheet.
    let factory: &dyn UniaxialFactory<Params = MetodoParams> = &MetodoFactory;
    assert_eq!(factory.descriptor(), METODO);
    let law = factory.build(params)?;
    assert_eq!(law.kind(), METODO.kind);
    assert_eq!(law.contract(), METODO.contract);
    assert_eq!(law.initial_state().len(), law.state_len());
    // Path with tension, unloading and compression. Adjust without crossing
    // nondifferentiable points with ε ± h; justify h in the method sheet.
    let samples = [0.0, 0.001, 0.002, 0.001, -0.001, 0.0];
    let diagnostics = check_uniaxial_law(law.as_ref(), &samples, 1e-7);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    Ok(())
}

#[test]
fn prueba_descartada_no_muta_estado() -> EResult<()> {
    let law = Metodo::new(200.0)?; // Adapt to the actual constructor.
    let committed = law.initial_state();
    let snapshot = committed.clone();
    let first = law.respond(0.001, &committed)?;
    law.respond(-0.002, &committed)?; // Discard trial.
    assert_eq!(snapshot, committed);
    assert_eq!(first, law.respond(0.001, &committed)?);
    let accepted = first.trial_state;
    let repeated = law.respond(0.001, &accepted)?.stress;
    // Same acceptance allowance as the kit: relative plus absolute floor.
    let scale = first.stress.abs().max(repeated.abs());
    assert!((first.stress - repeated).abs() <= 1e-12 + 1e-10 * scale);
    Ok(())
}

// Add negatives specific to each parameter, NaN/±∞ in inputs and
// history components, invalid lengths and overflow; compare codes
// ENG-E-0008/0010 with the catalog. Test boundaries and limits separately.
