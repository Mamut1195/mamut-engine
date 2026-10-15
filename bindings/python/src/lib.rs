#![forbid(unsafe_code)]
//! Adaptador Python mínimo: expone únicamente la versión del núcleo Rust.
//!
//! En `wasm32` este crate queda vacío: PyO3 y el módulo Python solo se
//! compilan para destinos nativos. Permanece como miembro del workspace
//! para permitir sus comprobaciones completas en WebAssembly.

#[cfg(not(target_arch = "wasm32"))]
mod python {
    use pyo3::prelude::*;

    /// Devuelve la versión del núcleo; no declara capacidades de análisis.
    #[pyfunction]
    fn version() -> &'static str {
        engine_core::VERSION
    }

    /// Registra la única función disponible en el scaffold.
    #[pymodule]
    fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_function(wrap_pyfunction!(version, module)?)?;
        Ok(())
    }
}
