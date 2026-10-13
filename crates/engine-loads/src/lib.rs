//! Casos de carga y funciones puras en SI, sin factores normativos implícitos.
//!
//! Se reutilizan las cargas nodales del modelo y los desplazamientos prescritos
//! del ensamblador. Las cargas de tramo actúan sobre una viga Euler–Bernoulli 2D
//! sin liberaciones: plano XY, local x de i a j, local y=(-sin,cos), z=+Z.
//! [`equivalent_nodal_forces_local`] sigue el signo de la carga aplicada;
//! [`fixed_end_forces_local`] devuelve su opuesto, las reacciones de los
//! empotramientos sobre el miembro con desplazamientos nulos.
//! Ambas usan el orden `[N_i,V_i,M_i,N_j,V_j,M_j]`.
//!
//! Las funciones validan entradas y resultados: ENG-E-0008 para no finitos o
//! desbordamientos, ENG-E-0010 para rangos o componentes repetidas en un caso,
//! ENG-E-0011 para GDL ausentes, ENG-E-0003 para referencias rotas y ENG-E-0013
//! para conversiones no representables mediante el contrato uniforme.
//! Las listas públicas se vuelven a validar en cada operación.
//!
//! ```
//! use engine_loads::{Beam2D, LoadAxes, SpanLoad, equivalent_nodal_forces_local};
//! let beam = Beam2D::new(4.0, 0.0)?;
//! let load = SpanLoad::Uniform { q: [0.0, 6.0], axes: LoadAxes::Local };
//! let f = equivalent_nodal_forces_local(&beam, &load)?;
//! assert!((f[1] - 12.0).abs() < 1e-12);
//! assert!((f[2] - 8.0).abs() < 1e-12);
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

mod nodal;
mod span;
mod weight;

pub use engine_assembly::PrescribedDof as ImposedDisplacement;
pub use engine_model::{DofValue, LoadCase, NodalLoad};
pub use nodal::{assemble_imposed_displacements, assemble_nodal_loads, validate_case};
pub use span::{
    equivalent_nodal_forces_local, fixed_end_forces_local, to_member_load, Beam2D, LoadAxes,
    SpanLoad,
};
pub use weight::{self_weight, self_weight_per_length};

use engine_core::{catalog, EResult, EngineError, Real};

fn error(code: &'static str, context: &str) -> EngineError {
    // Todos los códigos utilizados pertenecen al catálogo; sin unwrap/expect.
    let message = match catalog::lookup(code) {
        Some(info) => format!("{} {context}", info.message),
        None => context.to_owned(),
    };
    EngineError::new(code, message)
}

fn finite(values: &[Real]) -> EResult<()> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(error(
            "ENG-E-0008",
            "Entrada o resultado de carga no finito.",
        ));
    }
    Ok(())
}

fn length(value: Real) -> EResult<()> {
    finite(&[value])?;
    if value <= 0.0 {
        return Err(error("ENG-E-0010", "La longitud debe ser positiva."));
    }
    Ok(())
}
