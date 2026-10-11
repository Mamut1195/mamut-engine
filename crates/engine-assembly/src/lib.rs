//! Ensamblaje lineal determinista, apoyos y desplazamientos impuestos en SI.
//!
//! Los aportes se reciben en ejes globales: no se implementan elementos finitos
//! ni transformaciones locales. D2 sigue X-Y con Ux, Uy, Rz; D3 sigue la terna
//! diestra con Z hacia arriba. Traslaciones en m, giros en rad, fuerzas en N y
//! momentos en N·m. K y Kg relacionan esas fuerzas con esos desplazamientos;
//! M tiene entradas kg, kg·m o kg·m² según las componentes acopladas.
//!
//! [`DofMap`] numera los GDL declarados por nodo ascendente y componente de
//! [`engine_core::Dim`]; [`Constraints`] asigna los índices reducidos conservando
//! ese orden. Se permiten subconjuntos de GDL para problemas axiales.
//! [`LinearSystem`] implementa [`engine_contracts::SystemView`]: fuerza interna
//! física `(K u)_f = K_ff u_f + K_fp u_p`, externa `factor F_f` y tangente K_ff.
//! Para un solve directo, [`LinearSystem::reduced_rhs`] devuelve
//! `factor F_f - K_fp u_p`. No se resta nuevamente en SystemView.
//!
//! Las matrices globales son CSR; las reducidas son densas por el contrato
//! actual SymMatrix de engine-linalg. El solve se delega al backend, sin
//! regularización ni hilos. No se promete escalabilidad del solver de referencia.
//!
//! Ejemplo de frontera: dos GDL axiales, K suministrada por el llamador.
//! ```
//! use engine_assembly::{ElementContribution, LinearSystem};
//! use engine_contracts::CaseRef;
//! use engine_core::{Dim, Dof, DofIndex, NodeId};
//! use engine_linalg::{DMat, ReferenceBackend};
//! use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};
//! let mut model = Model::new();
//! for id in 1..=2 {
//!     let mut node = Node::new(NodeId(id), Dim::D2, [id as f64, 0.0, 0.0]);
//!     node.dofs = vec![Dof::Ux];
//!     model.nodes.push(node);
//! }
//! model.supports.push(Support {
//!     id: "base".into(), node: NodeId(1), restrained: vec![Dof::Ux],
//! });
//! model.cases.push(LoadCase { name: "P".into(), loads: vec![NodalLoad {
//!     node: NodeId(2), values: vec![DofValue { dof: Dof::Ux, value: 10.0 }],
//! }] });
//! let contribution = ElementContribution {
//!     location: vec![DofIndex(0), DofIndex(1)],
//!     stiffness: DMat::new(2, 2, vec![100.0, -100.0, -100.0, 100.0])?,
//!     mass_lumped: None, mass_consistent: None, geometric: None, loads: vec![],
//! };
//! let mut system = LinearSystem::from_model(ReferenceBackend, &model, &[contribution], &[])?;
//! let case = CaseRef("P".into());
//! let u = system.solve(&case, 1.0)?;
//! assert!((u[0] - 0.1).abs() < 1e-12); // m: P/k
//! let reactions = system.reactions(&u, &case, 1.0)?;
//! assert!((reactions[0].value + 10.0).abs() < 1e-12); // N
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

pub mod assemble;
pub mod constraints;
pub mod dofmap;
pub mod reactions;
pub mod system;

pub use assemble::{ElementContribution, ElementLoad};
pub use constraints::{Constraints, PrescribedDof};
pub use dofmap::DofMap;
pub use reactions::Reaction;
pub use system::{GeometricEval, GeometricProducer, LinearSystem};

use engine_core::{EResult, EngineError, Real};

fn error(code: &'static str, message: impl Into<String>) -> EngineError {
    let mut result = EngineError::from_code(code).expect("Código del catálogo engine-core");
    result.message.push(' ');
    result.message.push_str(&message.into());
    result
}

fn size(actual: usize, expected: usize) -> EResult<()> {
    if actual != expected {
        return Err(error(
            "ENG-E-0010",
            format!("Tamaño {actual}; esperado {expected}."),
        ));
    }
    Ok(())
}

fn finite(values: &[Real]) -> EResult<()> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(error(
            "ENG-E-0008",
            "Entrada o resultado de ensamblaje no finito.",
        ));
    }
    Ok(())
}
