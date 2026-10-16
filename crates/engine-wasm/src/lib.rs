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

/// Returns a JSON string literal containing the explanation for a diagnostic code.
#[wasm_bindgen]
pub fn explain(code: &str) -> Result<String, JsValue> {
    json(&engine_api::explain(code).map_err(api_error)?)
}

/// Validates engine-input/1 and returns canonical JSON diagnostics.
/// API errors throw serialized ApiError strings; callback exceptions are rethrown.
#[wasm_bindgen]
pub fn validate(
    input: &str,
    cancel: &CancelToken,
    progress: Option<Function>,
) -> Result<String, JsValue> {
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
    json(&result.map_err(api_error)?)
}

/// Runs engine-input/1 and returns exactly the canonical results/1 bytes from engine-api.
/// Invalid inputs and cancellation return results/1; callback exceptions are rethrown.
#[wasm_bindgen]
pub fn run(
    input: &str,
    cancel: &CancelToken,
    progress: Option<Function>,
) -> Result<String, JsValue> {
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
    Ok(outcome.document)
}
