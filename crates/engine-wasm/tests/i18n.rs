use engine_api::{CancelToken, Lang};

const INVALID: &str =
    include_str!("../../engine-api/fixtures/input/invalid/missing-reference.json");
const VALID: &str =
    include_str!("../../engine-api/fixtures/input/valid/cantilever-linear-static.json");

#[test]
fn localized_exports_match_engine_api_byte_for_byte() {
    for lang in Lang::ALL {
        for input in [INVALID, VALID] {
            let actual = engine_wasm::run(
                input,
                &engine_wasm::CancelToken::new(),
                None,
                Some(lang.code().into()),
            )
            .unwrap();
            let expected = engine_api::run(input, &CancelToken::new(), |_| {})
                .localized(lang)
                .document;
            assert_eq!(actual, expected);
            let actual = engine_wasm::validate(
                input,
                &engine_wasm::CancelToken::new(),
                None,
                Some(lang.code().into()),
            )
            .unwrap();
            let diagnostics =
                engine_api::localize_diagnostics(engine_api::validate(input).unwrap(), lang);
            let expected =
                engine_api::canonical_json(&serde_json::to_string(&diagnostics).unwrap()).unwrap();
            assert_eq!(actual, expected);
        }
        for code in ["ENG-E-0003", "API-E-SFC-OPTIONS"] {
            let actual = engine_wasm::explain(code, Some(lang.code().into())).unwrap();
            let expected = engine_api::canonical_json(
                &serde_json::to_string(engine_api::explain_in(code, lang).unwrap()).unwrap(),
            )
            .unwrap();
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn omitted_language_preserves_existing_english_bytes() {
    for input in [INVALID, VALID] {
        assert_eq!(
            engine_wasm::run(input, &engine_wasm::CancelToken::new(), None, None).unwrap(),
            engine_api::run(input, &CancelToken::new(), |_| {}).document
        );
        let expected = engine_api::canonical_json(
            &serde_json::to_string(&engine_api::validate(input).unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            engine_wasm::validate(input, &engine_wasm::CancelToken::new(), None, None).unwrap(),
            expected
        );
    }
    let expected = engine_api::canonical_json(
        &serde_json::to_string(engine_api::explain("ENG-E-0003").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(engine_wasm::explain("ENG-E-0003", None).unwrap(), expected);
}
