//! Embedded, locale-independent translations for stable diagnostic codes.

use crate::{ApiError, Diagnostic};
use std::{collections::BTreeMap, sync::OnceLock};

/// Explicit message language; the default never depends on the host locale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    /// English (the byte-compatible default).
    #[default]
    En,
    /// Spanish.
    Es,
    /// Portuguese (neutral/Brazilian terminology).
    Pt,
}

impl Lang {
    /// All supported languages in stable order.
    pub const ALL: [Self; 3] = [Self::En, Self::Es, Self::Pt];

    /// Returns the exact language identifier.
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Pt => "pt",
        }
    }

    /// Accepts only exact lowercase identifiers, without whitespace or regions.
    pub fn parse(code: &str) -> Result<Self, ApiError> {
        match code {
            "en" => Ok(Self::En),
            "es" => Ok(Self::Es),
            "pt" => Ok(Self::Pt),
            _ => Err(ApiError::InvalidInput(format!(
                "Unsupported language {code:?}"
            ))),
        }
    }
}

type Messages = BTreeMap<String, String>;

fn catalog(lang: Lang) -> Option<&'static Messages> {
    static EN: OnceLock<Option<Messages>> = OnceLock::new();
    static ES: OnceLock<Option<Messages>> = OnceLock::new();
    static PT: OnceLock<Option<Messages>> = OnceLock::new();
    let (slot, source) = match lang {
        Lang::En => (&EN, include_str!("../messages/en.json")),
        Lang::Es => (&ES, include_str!("../messages/es.json")),
        Lang::Pt => (&PT, include_str!("../messages/pt.json")),
    };
    slot.get_or_init(|| serde_json::from_str(source).ok())
        .as_ref()
}

/// Looks up an embedded message, returning `None` for unknown keys or invalid JSON.
pub fn message(lang: Lang, key: &str) -> Option<&'static str> {
    catalog(lang)?.get(key).map(String::as_str)
}

/// Returns catalog keys in alphabetical order; invalid catalogs return no keys.
pub fn message_keys(lang: Lang) -> Vec<&'static str> {
    catalog(lang).map_or_else(Vec::new, |messages| {
        messages.keys().map(String::as_str).collect()
    })
}

fn translated_cause(code: &str, cause: &str, lang: Lang) -> Option<&'static str> {
    if lang == Lang::En {
        return None;
    }
    let key = format!("diagnostic.{code}");
    if message(Lang::En, &key)? == cause {
        message(lang, &key)
    } else {
        None
    }
}

/// Translates only exact default diagnostic causes; preserves hints and context.
pub fn localize_diagnostic(diagnostic: &mut Diagnostic, lang: Lang) {
    if let Some(cause) = translated_cause(diagnostic.code, &diagnostic.cause, lang) {
        diagnostic.cause = cause.into();
    }
}

/// Translates exact default causes while preserving diagnostic order and data.
pub fn localize_diagnostics(mut diagnostics: Vec<Diagnostic>, lang: Lang) -> Vec<Diagnostic> {
    for diagnostic in &mut diagnostics {
        localize_diagnostic(diagnostic, lang);
    }
    diagnostics
}

/// Localizes `diagnostics[]` in a results or validation document using canonical JSON.
/// English returns the original bytes without parsing or serialization.
pub fn localize_document(document: &str, lang: Lang) -> Result<String, ApiError> {
    if lang == Lang::En {
        return Ok(document.into());
    }
    let mut value = crate::json::parse(document)?;
    if let Some(diagnostics) = value
        .get_mut("diagnostics")
        .and_then(serde_json::Value::as_array_mut)
    {
        for diagnostic in diagnostics {
            if let (Some(code), Some(cause)) =
                (diagnostic["code"].as_str(), diagnostic["cause"].as_str())
            {
                if let Some(cause) = translated_cause(code, cause, lang) {
                    diagnostic["cause"] = cause.into();
                }
            }
        }
    }
    crate::json::canonical_value(value)
}

/// Explains a stable domain or API code in the explicitly selected language.
pub fn explain_in(code: &str, lang: Lang) -> Result<&'static str, ApiError> {
    message(lang, &format!("explain.{code}")).ok_or_else(|| ApiError::UnknownCode(code.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn api_explanation_codes() -> Vec<&'static str> {
        // Read every arm of the legacy English reference, so an added API code
        // cannot silently escape translation coverage.
        include_str!("run.rs")
            .split("pub(crate) fn explain(code:")
            .nth(1)
            .unwrap()
            .split("\n}")
            .next()
            .unwrap()
            .lines()
            .filter_map(|line| line.trim().strip_prefix('"'))
            .filter_map(|line| line.split_once('"').map(|(code, _)| code))
            .filter(|code| code.starts_with("API-"))
            .collect()
    }

    #[test]
    fn catalogs_cover_exactly_the_registered_codes_and_preserve_english() {
        let registry: serde_json::Value =
            serde_json::from_str(include_str!("../../../docs/contracts/error-codes-1.json"))
                .unwrap();
        let mut expected = BTreeSet::new();
        for definition in registry["codes"].as_array().unwrap() {
            expected.insert(format!("errors.{}", definition["id"].as_str().unwrap()));
        }
        for info in engine_core::catalog::CODES {
            expected.insert(format!("diagnostic.{}", info.code));
            expected.insert(format!("explain.{}", info.code));
            assert_eq!(
                message(Lang::En, &format!("diagnostic.{}", info.code)),
                Some(info.message)
            );
            assert_eq!(
                crate::explain(info.code).unwrap(),
                engine_core::catalog::explain(info.code).unwrap()
            );
        }
        for code in api_explanation_codes() {
            expected.insert(format!("explain.{code}"));
            assert_eq!(crate::explain(code).ok(), crate::run::explain(code));
        }
        for lang in Lang::ALL {
            let keys = message_keys(lang);
            assert_eq!(
                keys.iter().copied().collect::<BTreeSet<_>>(),
                expected.iter().map(String::as_str).collect()
            );
            assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
            // Also enforce alphabetical order in the checked-in JSON itself.
            let source = match lang {
                Lang::En => include_str!("../messages/en.json"),
                Lang::Es => include_str!("../messages/es.json"),
                Lang::Pt => include_str!("../messages/pt.json"),
            };
            let source_keys: Vec<_> = source
                .lines()
                .filter_map(|line| line.trim().strip_prefix('"'))
                .map(|line| line.split('"').next().unwrap())
                .collect();
            assert_eq!(source_keys, keys);
            for key in keys {
                let text = message(lang, key).unwrap();
                assert!(!text.is_empty(), "{lang:?}: {key}");
                assert_eq!(text, text.trim(), "{lang:?}: {key}");
                if lang != Lang::En {
                    // No exceptions: every Spanish/Portuguese entry is translated.
                    assert_ne!(Some(text), message(Lang::En, key), "{lang:?}: {key}");
                }
            }
        }
    }

    #[test]
    fn translations_preserve_technical_identifiers_numbers_and_symbols() {
        let names = [
            "nonlinearStatic",
            "soilProfile",
            "consolidation1d",
            "options.allow_experimental",
            "true",
            "docs/GUIA_METODOS.md",
            "provenance/ENG-SFC-010.md",
            "max_step_s",
            "max(1, ceil(interval/max_step_s))",
            "u128",
            "Experimental",
            "Kss",
            "Guyan",
            "Winkler",
            "combine/factorize",
            "factor·δ",
            "LDLᵀ",
            "2×2",
            "≤5 s",
            "n³",
            "n²",
            "elements−1",
            "copysign",
        ];
        for key in message_keys(Lang::En) {
            let english = message(Lang::En, key).unwrap();
            let codes: Vec<_> = english
                .split(|c: char| !(c.is_ascii_alphanumeric() || "-_/.".contains(c)))
                .map(|word| word.trim_end_matches('.'))
                .filter(|word| {
                    word.starts_with("ENG-") || word.starts_with("API-") || word.starts_with("MAX_")
                })
                .collect();
            let numbers: Vec<_> = english
                .split(|c: char| !c.is_ascii_digit())
                .filter(|word| !word.is_empty())
                .collect();
            for lang in [Lang::Es, Lang::Pt] {
                let text = message(lang, key).unwrap();
                for token in codes
                    .iter()
                    .chain(numbers.iter())
                    .copied()
                    .chain(names.iter().copied().filter(|name| english.contains(name)))
                {
                    // 'elements' is prose; preserve the numerical expression and symbol.
                    let token = if token == "elements−1" {
                        "−1"
                    } else {
                        token
                    };
                    assert!(text.contains(token), "{lang:?}: {key} missing {token}");
                }
            }
        }
    }

    #[test]
    fn languages_are_exact_and_unknown_keys_and_codes_are_errors() {
        assert_eq!(Lang::default(), Lang::En);
        for lang in Lang::ALL {
            assert_eq!(Lang::parse(lang.code()), Ok(lang));
            assert_eq!(message(lang, "missing"), None);
            assert_eq!(
                explain_in("ENG-E-9999", lang),
                Err(ApiError::UnknownCode("ENG-E-9999".into()))
            );
        }
        for code in ["", "EN", "ES", "pt-BR", "es_EC", " en", "pt ", "de"] {
            assert_eq!(
                Lang::parse(code),
                Err(ApiError::InvalidInput(format!(
                    "Unsupported language {code:?}"
                )))
            );
        }
    }
}
