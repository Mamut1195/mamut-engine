use engine_api::{localize_diagnostic, localize_diagnostics, localize_document, Lang};
use serde_json::{json, Value};

const INVALID: &str = include_str!("../fixtures/input/invalid/missing-reference.json");

#[test]
fn real_diagnostics_translate_only_default_causes_and_preserve_all_data() {
    let english = engine_api::validate(INVALID).unwrap();
    assert!(!english.is_empty());
    assert_eq!(localize_diagnostics(english.clone(), Lang::En), english);
    for lang in [Lang::Es, Lang::Pt] {
        let translated = localize_diagnostics(english.clone(), lang);
        let mut changed = 0;
        for (base, localized) in english.iter().zip(&translated) {
            let mut expected = base.clone();
            if let Some(default) =
                engine_api::message(Lang::En, &format!("diagnostic.{}", base.code))
            {
                if base.cause == default {
                    expected.cause =
                        engine_api::message(lang, &format!("diagnostic.{}", base.code))
                            .unwrap()
                            .into();
                    changed += 1;
                }
            }
            assert_eq!(localized, &expected);
            let mut specific = base.clone();
            specific.cause.push_str(" (entity 99)");
            let original = specific.clone();
            localize_diagnostic(&mut specific, lang);
            assert_eq!(specific, original);
        }
        assert!(changed > 0);
        let outcome = engine_api::run(INVALID, &engine_api::CancelToken::new(), |_| {});
        let localized = outcome.clone().localized(lang);
        assert_eq!(localized.status, outcome.status);
        let mut expected: Value = serde_json::from_str(&outcome.document).unwrap();
        expected["diagnostics"] = json!(translated);
        assert_eq!(
            localized.document,
            engine_api::canonical_json(&expected.to_string()).unwrap()
        );
        assert_eq!(outcome.clone().localized(Lang::En), outcome);
        let report = json!({"schema":"validate-report/1","diagnostics":english});
        let mut expected = report.clone();
        expected["diagnostics"] = json!(localize_diagnostics(english.clone(), lang));
        assert_eq!(
            localize_document(&report.to_string(), lang).unwrap(),
            engine_api::canonical_json(&expected.to_string()).unwrap()
        );
    }
}

#[test]
fn english_preserves_document_bytes_and_other_languages_reject_invalid_json() {
    let document = " { \"diagnostics\": [] } \n";
    assert_eq!(localize_document(document, Lang::En).unwrap(), document);
    for lang in [Lang::Es, Lang::Pt] {
        assert!(matches!(
            localize_document("not JSON", lang),
            Err(engine_api::ApiError::InvalidInput(_))
        ));
    }
}
