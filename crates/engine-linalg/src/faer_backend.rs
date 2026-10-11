//! Reserva explícita del adaptador faer: no incorpora librería ni operaciones.
use engine_core::{EResult, EngineError};

/// Adaptador PENDIENTE de aclarar los avisos de licencia de faer 0.24.4.
///
/// La prueba temporal con dependencia optional, `default-features=false` y
/// `features=["std","sparse-linalg"]` compiló con Rust 1.85.1 en nativo y
/// wasm; ambos backends pasaron la misma conformidad nativa. Sin embargo, el
/// paquete incluye `COPYING.EIGEN.MPL2` pese a declarar MIT. Sin acceso de red
/// no pudo verificarse el alcance de ese aviso mediante documentación oficial.
/// No se incorpora esa dependencia a la entrega: véase README.md del crate y
/// `provenance/ENG-LA-007.md`. Este stub no implementa [`crate::LinAlgBackend`]
/// ni sustituye silenciosamente faer por el backend de referencia.
#[derive(Debug)]
pub struct FaerBackend {
    _private: (),
}
impl FaerBackend {
    /// Siempre devuelve ENG-E-0009 con el motivo de integración pendiente.
    /// La feature reserva el nombre, sin habilitar un backend funcional.
    pub fn new() -> EResult<Self> {
        Err(EngineError::new(
            "ENG-E-0009",
            "Backend faer pendiente: verificar el alcance de COPYING.EIGEN.MPL2 en faer 0.24.4 antes de incorporar una dependencia exclusivamente permisiva.",
        ))
    }
}
