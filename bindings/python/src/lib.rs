#![forbid(unsafe_code)]
//! Adaptador Python de engine-api, sin implementación numérica.
//!
//! El crate queda vacío en wasm32 y permanece fuera de default-members.

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

    // Cubre también conversión de resultados y entrega de callbacks. PyO3 conserva
    // además su barrera de unwind para el código generado de entrada al módulo.
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
                    return Err(InternalEngineError::new_err("Número JSON no representable"));
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
        Ok(err)
    }

    fn api_error(py: Python<'_>, err: ApiError) -> PyResult<PyErr> {
        let code = match &err {
            ApiError::UnsupportedSchema(_) => "ENG-E-0009",
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
                // Solo se parsea salida de engine-api, nunca entrada del usuario.
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

    /// Token atómico compartido; cancel() puede ejecutarse desde otro hilo Python.
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

    #[pyfunction]
    fn explain(py: Python<'_>, code: &str) -> PyResult<&'static str> {
        guarded(py, || match engine_api::explain(code) {
            Ok(text) => Ok(text),
            Err(err) => Err(api_error(py, err)?),
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

    #[pyfunction]
    fn validate(py: Python<'_>, input_json: &str) -> PyResult<Py<PyAny>> {
        guarded(py, || {
            match py.detach(|| engine_api::validate(input_json)) {
                Ok(diagnostics) => object(py, &json!({"diagnostics": diagnostics})),
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
        err.value(py).setattr("document", result.document)?;
        err.value(py)
            .setattr("diagnostics", object(py, &document["diagnostics"])?)?;
        Err(err)
    }

    /// Ejecuta sin GIL; entrega progreso acumulado después de salir del motor.
    #[pyfunction(signature = (input_json, progress=None, cancel=None))]
    fn run(
        py: Python<'_>,
        input_json: &str,
        progress: Option<Py<PyAny>>,
        cancel: Option<PyRef<'_, CancelToken>>,
    ) -> PyResult<String> {
        guarded(py, || {
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
            // Un error del motor tiene prioridad frente a excepciones del callback.
            let result = outcome(py, result);
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
        Ok(())
    }
}
