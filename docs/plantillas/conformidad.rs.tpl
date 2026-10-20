//! Adaptar importaciones, parámetros y muestras desde la ficha del método.
use engine_contracts::{conformance::check_uniaxial_law, UniaxialLaw};
use engine_core::EResult;
use engine_laws::uniaxial::{Metodo, MetodoFactory, MetodoParams, UniaxialFactory, METODO};

#[test]
fn conformidad_y_fabrica() -> EResult<()> {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<Metodo>();
    send_sync::<MetodoFactory>();
    let params = MetodoParams { e: 200.0 }; // Sustituir desde la ficha.
    let factory: &dyn UniaxialFactory<Params = MetodoParams> = &MetodoFactory;
    assert_eq!(factory.descriptor(), METODO);
    let law = factory.build(params)?;
    assert_eq!(law.kind(), METODO.kind);
    assert_eq!(law.contract(), METODO.contract);
    assert_eq!(law.initial_state().len(), law.state_len());
    // Trayectoria con tracción, descarga y compresión. Ajustar sin cruzar
    // puntos no diferenciables con ε ± h; justificar h en la ficha.
    let samples = [0.0, 0.001, 0.002, 0.001, -0.001, 0.0];
    let diagnostics = check_uniaxial_law(law.as_ref(), &samples, 1e-7);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    Ok(())
}

#[test]
fn prueba_descartada_no_muta_estado() -> EResult<()> {
    let law = Metodo::new(200.0)?; // Adaptar al constructor real.
    let committed = law.initial_state();
    let snapshot = committed.clone();
    let first = law.respond(0.001, &committed)?;
    law.respond(-0.002, &committed)?; // Descartar prueba.
    assert_eq!(snapshot, committed);
    assert_eq!(first, law.respond(0.001, &committed)?);
    let accepted = first.trial_state;
    let repeated = law.respond(0.001, &accepted)?.stress;
    // Misma holgura de aceptación que el kit: relativo más piso absoluto.
    let scale = first.stress.abs().max(repeated.abs());
    assert!((first.stress - repeated).abs() <= 1e-12 + 1e-10 * scale);
    Ok(())
}

// Añadir negativos específicos de cada parámetro, NaN/±∞ en entradas y
// componentes de historia, longitudes inválidas y overflow; comparar códigos
// ENG-E-0008/0010 con el catálogo. Probar fronteras y límites por separado.
