#![forbid(unsafe_code)]
//! Python adapter for engine-api, without a numerical implementation.
//!
//! The crate is empty on wasm32 and remains outside default-members.

#[cfg(not(target_arch = "wasm32"))]
mod unwind;

#[cfg(not(target_arch = "wasm32"))]
mod python {
    use engine_api::{ApiError, Status};
    use pyo3::exceptions::{PyException, PyTypeError};
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList};
    use serde_json::{json, Value};
    use std::sync::Arc;

    pyo3::create_exception!(mamut_engine._native, EngineError, PyException);
    pyo3::create_exception!(mamut_engine._native, InvalidInputError, EngineError);
    pyo3::create_exception!(mamut_engine._native, CancelledError, EngineError);
    pyo3::create_exception!(mamut_engine._native, AnalysisError, EngineError);
    pyo3::create_exception!(mamut_engine._native, InternalEngineError, EngineError);

    // Also covers result conversion and callback delivery. PyO3 retains
    // its unwind barrier for the generated module entry code as well.
    fn guarded<T>(py: Python<'_>, f: impl FnOnce() -> PyResult<T>) -> PyResult<T> {
        match super::unwind::boundary(f) {
            Ok(result) => result,
            Err(_) => Err(engine_error(
                py,
                InternalEngineError::new_err("Fallo interno del motor"),
                "ENG-E-0012",
                &json!([]),
                "Fallo interno del motor (unwind interceptado)",
            )?),
        }
    }

    fn object(py: Python<'_>, value: &Value) -> PyResult<Py<PyAny>> {
        Ok(match value {
            Value::Null => py.None(),
            Value::Bool(v) => v.into_pyobject(py)?.to_owned().into_any().unbind(),
            Value::String(v) => v.into_pyobject(py)?.into_any().unbind(),
            Value::Number(n) => {
                if let Some(v) = n.as_u64() {
                    v.into_pyobject(py)?.into_any().unbind()
                } else if let Some(v) = n.as_i64() {
                    v.into_pyobject(py)?.into_any().unbind()
                } else if let Some(v) = n.as_f64() {
                    v.into_pyobject(py)?.into_any().unbind()
                } else {
                    let err = InternalEngineError::new_err("Número JSON no representable");
                    apply_reference(py, &err, &json!({}))?;
                    return Err(err);
                }
            }
            Value::Array(values) => {
                let list = PyList::empty(py);
                for v in values {
                    list.append(object(py, v)?)?;
                }
                list.into_any().unbind()
            }
            Value::Object(values) => {
                let dict = PyDict::new(py);
                for (key, v) in values {
                    dict.set_item(key, object(py, v)?)?;
                }
                dict.into_any().unbind()
            }
        })
    }

    fn engine_error(
        py: Python<'_>,
        err: PyErr,
        code: &str,
        entities: &Value,
        cause: &str,
    ) -> PyResult<PyErr> {
        let value = err.value(py);
        value.setattr("code", code)?;
        value.setattr("entities", object(py, entities)?)?;
        value.setattr("cause", cause)?;
        apply_reference(py, &err, &json!({}))?;
        Ok(err)
    }

    fn apply_reference(py: Python<'_>, err: &PyErr, diagnostic: &Value) -> PyResult<()> {
        let value = err.value(py);
        let kind = diagnostic
            .get("entity_kind")
            .filter(|item| item.is_string());
        value.setattr(
            "entity_kind",
            match kind {
                Some(item) => object(py, item)?,
                None => py.None(),
            },
        )?;
        let missing = diagnostic
            .get("missing_id")
            .filter(|item| item.is_string() || item.is_i64() || item.is_u64());
        value.setattr(
            "missing_id",
            match missing {
                Some(item) => object(py, item)?,
                None => py.None(),
            },
        )?;
        let path = diagnostic
            .get("referenced_by")
            .filter(|item| item.is_string());
        value.setattr(
            "referenced_by",
            match path {
                Some(item) => object(py, item)?,
                None => py.None(),
            },
        )?;
        let suggestions = PyList::empty(py);
        if let Some(items) = diagnostic.get("suggestions").and_then(Value::as_array) {
            for item in items {
                if let Some(text) = item.as_str() {
                    suggestions.append(text)?;
                }
            }
        }
        value.setattr("suggestions", suggestions)?;
        Ok(())
    }

    fn api_error(py: Python<'_>, err: ApiError) -> PyResult<PyErr> {
        let code = match &err {
            ApiError::UnsupportedSchema(_) => "ENG-E-0009",
            ApiError::UnknownUnit(_) => "ENG-E-0001",
            ApiError::Cancelled => "API-I-CANCELLED",
            _ => "API-E-INPUT",
        };
        let cause = err.to_string();
        let error = if matches!(err, ApiError::Cancelled) {
            CancelledError::new_err(cause.clone())
        } else {
            InvalidInputError::new_err(cause.clone())
        };
        engine_error(py, error, code, &json!([]), &cause)
    }

    fn parse_output(py: Python<'_>, document: &str) -> PyResult<Value> {
        serde_json::from_str(document)
            .map_err(|err| {
                // Only engine-api output is parsed, never user input.
                InternalEngineError::new_err(err.to_string())
            })
            .or_else(|err| {
                Err(engine_error(
                    py,
                    err,
                    "ENG-E-0012",
                    &json!([]),
                    "Salida JSON interna inválida",
                )?)
            })
    }

    /// Shared atomic token; cancel() can run from another Python thread.
    #[pyclass(frozen, module = "mamut_engine._native")]
    #[derive(Default)]
    struct CancelToken {
        inner: Arc<engine_api::CancelToken>,
    }

    #[pymethods]
    impl CancelToken {
        #[new]
        fn new() -> Self {
            Self::default()
        }

        fn cancel(&self) {
            self.inner.cancel();
        }

        #[getter]
        fn is_cancelled(&self) -> bool {
            self.inner.is_cancelled()
        }
    }

    #[pyfunction]
    fn version(py: Python<'_>) -> PyResult<Py<PyAny>> {
        guarded(py, || object(py, &engine_api::version()))
    }

    #[pyfunction]
    fn capabilities(py: Python<'_>) -> PyResult<Py<PyAny>> {
        guarded(py, || object(py, &engine_api::capabilities()))
    }

    #[pyfunction]
    fn schema(py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        guarded(py, || match engine_api::schema(name) {
            Ok(document) => object(py, &parse_output(py, document)?),
            Err(err) => Err(api_error(py, err)?),
        })
    }

    fn language(py: Python<'_>, lang: Option<&str>) -> PyResult<engine_api::Lang> {
        lang.map_or(Ok(engine_api::Lang::En), engine_api::Lang::parse)
            .or_else(|err| Err(api_error(py, err)?))
    }

    #[pyfunction(signature = (code, lang=None))]
    fn explain(py: Python<'_>, code: &str, lang: Option<&str>) -> PyResult<&'static str> {
        guarded(py, || {
            match engine_api::explain_in(code, language(py, lang)?) {
                Ok(text) => Ok(text),
                Err(err) => Err(api_error(py, err)?),
            }
        })
    }

    #[pyfunction]
    fn canonical_json(py: Python<'_>, input_json: &str) -> PyResult<String> {
        guarded(py, || {
            match py.detach(|| engine_api::canonical_json(input_json)) {
                Ok(document) => Ok(document),
                Err(err) => Err(api_error(py, err)?),
            }
        })
    }

    #[pyfunction]
    fn input_sha256(py: Python<'_>, input_json: &str) -> PyResult<String> {
        guarded(py, || {
            match py.detach(|| engine_api::input_sha256(input_json)) {
                Ok(digest) => Ok(digest),
                Err(err) => Err(api_error(py, err)?),
            }
        })
    }

    #[pyfunction(signature = (input_json, lang=None))]
    fn validate(py: Python<'_>, input_json: &str, lang: Option<&str>) -> PyResult<Py<PyAny>> {
        guarded(py, || {
            let lang = language(py, lang)?;
            match py.detach(|| engine_api::validate(input_json)) {
                Ok(diagnostics) => object(
                    py,
                    &json!({"diagnostics": engine_api::localize_diagnostics(diagnostics, lang)}),
                ),
                Err(err) => Err(api_error(py, err)?),
            }
        })
    }

    fn outcome(py: Python<'_>, result: engine_api::Outcome) -> PyResult<String> {
        if matches!(result.status, Status::Ok | Status::Warning) {
            return Ok(result.document);
        }
        let document = parse_output(py, &result.document)?;
        let diagnostic = document["diagnostics"]
            .as_array()
            .and_then(|items| items.iter().find(|d| d["severity"] == "error"))
            .cloned()
            .unwrap_or_else(
                || json!({"code": "ENG-E-0012", "entities": [], "cause": "Fallo sin diagnóstico"}),
            );
        let code = diagnostic["code"].as_str().unwrap_or("ENG-E-0012");
        let cause = diagnostic["cause"].as_str().unwrap_or("Fallo del motor");
        let err = match result.status {
            Status::Cancelled => CancelledError::new_err(cause.to_owned()),
            Status::Invalid => InvalidInputError::new_err(cause.to_owned()),
            Status::InternalError => InternalEngineError::new_err(cause.to_owned()),
            _ => AnalysisError::new_err(cause.to_owned()),
        };
        let err = engine_error(py, err, code, &diagnostic["entities"], cause)?;
        apply_reference(py, &err, &diagnostic)?;
        err.value(py).setattr("document", result.document)?;
        err.value(py)
            .setattr("diagnostics", object(py, &document["diagnostics"])?)?;
        Err(err)
    }

    /// Runs without the GIL; delivers accumulated progress after leaving the engine.
    #[pyfunction(signature = (input_json, progress=None, cancel=None, lang=None))]
    fn run(
        py: Python<'_>,
        input_json: &str,
        progress: Option<Py<PyAny>>,
        cancel: Option<PyRef<'_, CancelToken>>,
        lang: Option<&str>,
    ) -> PyResult<String> {
        guarded(py, || {
            let lang = language(py, lang)?;
            if progress
                .as_ref()
                .is_some_and(|callback| !callback.bind(py).is_callable())
            {
                return Err(PyTypeError::new_err("progress debe ser callable o None"));
            }
            let token = cancel.map(|c| Arc::clone(&c.inner)).unwrap_or_default();
            let collect = progress.is_some();
            let (result, events) = py.detach(|| {
                let mut events = Vec::new();
                let result = engine_api::run(input_json, &token, |event| {
                    if collect {
                        events.push(event);
                    }
                });
                (result, events)
            });
            // An engine error takes precedence over callback exceptions.
            let result = outcome(py, result.localized(lang));
            for event in events {
                if let Some(callback) = &progress {
                    if let Err(err) = callback.call1(py, (object(py, &json!(event))?,)) {
                        return match result {
                            Err(engine) => Err(engine),
                            Ok(_) => Err(err),
                        };
                    }
                }
            }
            result
        })
    }

    fn fault_error(py: Python<'_>, fault: engine_api::AnalysisFault) -> PyErr {
        let diagnostic = fault
            .diagnostics
            .iter()
            .find(|item| item.severity == "error")
            .or_else(|| fault.diagnostics.first());
        let code = diagnostic.map(|item| item.code).unwrap_or("ENG-E-0012");
        let cause = diagnostic
            .map(|item| item.cause.clone())
            .unwrap_or_else(|| "Fallo del motor".to_owned());
        let entities = diagnostic
            .map(|item| Value::Array(item.entities.clone()))
            .unwrap_or_else(|| json!([]));
        let err = match fault.status {
            Status::Cancelled => CancelledError::new_err(cause.clone()),
            Status::Invalid => InvalidInputError::new_err(cause.clone()),
            Status::InternalError => InternalEngineError::new_err(cause.clone()),
            _ => AnalysisError::new_err(cause.clone()),
        };
        let err = engine_error(py, err, code, &entities, &cause).unwrap_or_else(|err| err);
        if let Some(item) = diagnostic {
            if let Err(failure) = apply_reference(py, &err, &json!(item)) {
                return failure;
            }
        }
        err
    }

    /// Owns one factored linearStatic model. Not sendable: the factor stays on this thread.
    #[pyclass(unsendable, name = "PreparedLinear", module = "mamut_engine._native")]
    struct NativePrepared {
        inner: engine_api::PreparedLinear,
    }

    #[pymethods]
    impl NativePrepared {
        /// Solves again and returns `[{case, nodes: [{id, displacements}]}]`.
        fn displacements(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
            let cases = match self.inner.displacements() {
                Ok(cases) => cases,
                Err(fault) => return Err(fault_error(py, fault)),
            };
            let out = PyList::empty(py);
            for case in cases {
                let nodes = PyList::empty(py);
                for node in case.nodes {
                    let displacements = PyList::empty(py);
                    for value in node.displacements {
                        displacements.append(value)?;
                    }
                    let item = PyDict::new(py);
                    item.set_item("id", node.id)?;
                    item.set_item("displacements", displacements)?;
                    nodes.append(item)?;
                }
                let item = PyDict::new(py);
                item.set_item("case", &case.case)?;
                item.set_item("nodes", nodes)?;
                out.append(item)?;
            }
            Ok(out.into_any().unbind())
        }
    }

    /// Prepares one linearStatic model. Holds the GIL; the repeated solve is the hot path.
    #[pyfunction]
    fn prepare(py: Python<'_>, input_json: &str) -> PyResult<NativePrepared> {
        guarded(py, || {
            match engine_api::PreparedLinear::prepare(input_json, &engine_api::CancelToken::new()) {
                Ok(inner) => Ok(NativePrepared { inner }),
                Err(fault) => Err(fault_error(py, fault)),
            }
        })
    }

    #[pymodule]
    fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
        let py = module.py();
        module.add("EngineError", py.get_type::<EngineError>())?;
        module.add("InvalidInputError", py.get_type::<InvalidInputError>())?;
        module.add("CancelledError", py.get_type::<CancelledError>())?;
        module.add("AnalysisError", py.get_type::<AnalysisError>())?;
        module.add("InternalEngineError", py.get_type::<InternalEngineError>())?;
        module.add_class::<CancelToken>()?;
        module.add_function(wrap_pyfunction!(version, module)?)?;
        module.add_function(wrap_pyfunction!(capabilities, module)?)?;
        module.add_function(wrap_pyfunction!(schema, module)?)?;
        module.add_function(wrap_pyfunction!(explain, module)?)?;
        module.add_function(wrap_pyfunction!(canonical_json, module)?)?;
        module.add_function(wrap_pyfunction!(input_sha256, module)?)?;
        module.add_function(wrap_pyfunction!(validate, module)?)?;
        module.add_function(wrap_pyfunction!(run, module)?)?;
        module.add_function(wrap_pyfunction!(prepare, module)?)?;
        module.add_class::<NativePrepared>()?;
        Ok(())
    }
}
