#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! JSON facade over engine-api. Callbacks run synchronously at API checkpoints.

use js_sys::Function;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Monotonic cooperative cancellation token; safe to cancel from a progress callback.
#[wasm_bindgen]
#[derive(Default)]
pub struct CancelToken(engine_api::CancelToken);

#[wasm_bindgen]
impl CancelToken {
    /// Creates an active token. Allocate one token per request.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Cancels at the next engine-api checkpoint, never inside a solver step.
    pub fn cancel(&self) {
        self.0.cancel();
    }

    /// Reports whether cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }
}

fn json(value: &impl Serialize) -> Result<String, JsValue> {
    let text =
        serde_json::to_string(value).map_err(|error| JsValue::from_str(&error.to_string()))?;
    engine_api::canonical_json(&text).map_err(api_error)
}

fn api_error(error: engine_api::ApiError) -> JsValue {
    match serde_json::to_string(&error) {
        Ok(text) => JsValue::from_str(&text),
        Err(error) => JsValue::from_str(&error.to_string()),
    }
}

fn language(lang: Option<String>) -> Result<engine_api::Lang, JsValue> {
    parse_language(lang.as_deref()).map_err(api_error)
}

fn parse_language(lang: Option<&str>) -> Result<engine_api::Lang, engine_api::ApiError> {
    lang.map_or(Ok(engine_api::Lang::En), engine_api::Lang::parse)
}

fn notify(callback: &Option<Function>, progress: engine_api::Progress) -> Result<(), JsValue> {
    if let Some(callback) = callback {
        callback.call1(&JsValue::UNDEFINED, &JsValue::from_str(&json(&progress)?))?;
    }
    Ok(())
}

/// Returns canonical JSON describing engine and contract versions.
#[wasm_bindgen]
pub fn version() -> Result<String, JsValue> {
    json(&engine_api::version())
}

/// Returns the engine-api capability document as canonical JSON.
#[wasm_bindgen]
pub fn capabilities() -> Result<String, JsValue> {
    json(&engine_api::capabilities())
}

/// Returns the published JSON Schema string for a contract name.
#[wasm_bindgen]
pub fn schema(kind: &str) -> Result<String, JsValue> {
    engine_api::schema(kind)
        .map(str::to_owned)
        .map_err(api_error)
}

/// Returns a JSON string literal explaining a code; omitted language defaults to English.
#[wasm_bindgen]
pub fn explain(code: &str, lang: Option<String>) -> Result<String, JsValue> {
    json(&engine_api::explain_in(code, language(lang)?).map_err(api_error)?)
}

/// Validates engine-input/1 and returns localized canonical JSON diagnostics.
/// The optional language accepts only en, es, or pt; omission selects English.
/// API errors throw serialized ApiError strings; callback exceptions are rethrown.
#[wasm_bindgen]
pub fn validate(
    input: &str,
    cancel: &CancelToken,
    progress: Option<Function>,
    lang: Option<String>,
) -> Result<String, JsValue> {
    let lang = language(lang)?;
    let mut callback_error = None;
    let result = engine_api::validate_with(input, &cancel.0, |event| {
        if let Err(error) = notify(&progress, event) {
            callback_error = Some(error);
            cancel.cancel();
        }
    });
    if let Some(error) = callback_error {
        return Err(error);
    }
    json(&engine_api::localize_diagnostics(
        result.map_err(api_error)?,
        lang,
    ))
}

/// Runs engine-input/1 and returns canonical results/1 bytes from engine-api.
/// The optional language accepts only en, es, or pt; omission preserves English bytes.
/// Invalid inputs and cancellation return results/1; callback exceptions are rethrown.
#[wasm_bindgen]
pub fn run(
    input: &str,
    cancel: &CancelToken,
    progress: Option<Function>,
    lang: Option<String>,
) -> Result<String, JsValue> {
    let lang = language(lang)?;
    let mut callback_error = None;
    let outcome = engine_api::run(input, &cancel.0, |event| {
        if let Err(error) = notify(&progress, event) {
            callback_error = Some(error);
            cancel.cancel();
        }
    });
    if let Some(error) = callback_error {
        return Err(error);
    }
    Ok(outcome.localized(lang).document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_languages_use_the_existing_serialized_api_error_contract() {
        // JsValue construction requires a JS runtime. Exercise the shared parser
        // and the exact serialization used by api_error on the native target.
        for lang in ["", "ES", "pt-BR", "es_EC", " en", "pt "] {
            let error = parse_language(Some(lang)).unwrap_err();
            assert_eq!(
                error,
                engine_api::ApiError::InvalidInput(format!("Unsupported language {lang:?}"))
            );
            assert_eq!(
                serde_json::to_value(error).unwrap(),
                serde_json::json!({
                    "kind":"InvalidInput", "cause":format!("Unsupported language {lang:?}")
                })
            );
        }
        assert_eq!(parse_language(None), Ok(engine_api::Lang::En));
    }
}
