//! Comprueba la misma barrera usada por PyO3 sin enlazar un intérprete embebido.
#![cfg(not(target_arch = "wasm32"))]
#[path = "../src/unwind.rs"]
mod unwind;

#[test]
fn engine_callback_unwind_is_contained_and_next_run_succeeds() {
    let input = include_str!(
        "../../../crates/engine-api/fixtures/input/valid/cantilever-linear-static.json"
    );
    let token = engine_api::CancelToken::new();
    let failed = unwind::boundary(|| {
        engine_api::run(input, &token, |_| {
            std::panic::resume_unwind(Box::new("fallo simulado durante engine-api::run"));
        })
    });
    assert_eq!(failed, Err(()));
    let recovered = unwind::boundary(|| engine_api::run(input, &token, |_| {}));
    assert!(recovered.is_ok_and(|result| result.status == engine_api::Status::Ok));
}
