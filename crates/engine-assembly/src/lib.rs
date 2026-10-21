#![forbid(unsafe_code)]
//! Deterministic linear assembly, supports, and imposed displacements in SI.
//!
//! Contributions are received in global axes: this crate implements neither finite elements
//! nor local transformations. D2 follows X-Y with Ux, Uy, Rz; D3 follows the
//! right-handed triad with Z upward. Translations in m, rotations in rad, forces in N, and
//! moments in N·m. K and Kg relate those forces to those displacements;
//! M has entries in kg, kg·m, or kg·m² according to the coupled components.
//!
//! [`DofMap`] numbers declared DOFs by ascending node and component of
//! [`engine_core::Dim`]; [`Constraints`] assigns reduced indices preserving
//! that order. DOF subsets are allowed for axial problems.
//! [`RigidDiaphragm`] transforms slave Ux/Uy/Rz in the D3 XY floor plane
//! through a master; Uz/Rx/Ry remain independent. Normal +Z and infinitesimal
//! rotations; D2 and other planes are unsupported. Declared with
//! [`LinearSystem::from_model_with_diaphragms`], without altering the Model domain.
//! [`Constraints::transformation`] exposes T, including prescribed independent DOFs;
//! expansion is u = T_f q_f + u_0 and force projection is T_fᵀ.
//! K, M, and Kg are reduced by congruence T_fᵀ A T_f. Support reactions and
//! master/slave actions are recovered from the global residual Ku − factor F.
//!
//! [`LinearSystem`] implements [`engine_contracts::SystemView`]: physical internal
//! force `T_fᵀ K u`, external `factor T_fᵀ F`, and tangent `T_fᵀ K T_f`.
//! For a direct solve, [`LinearSystem::reduced_rhs`] returns
//! `T_fᵀ(factor F − K u_0)`. Without diaphragms this equals
//! `factor F_f − K_fp u_p`. It is not subtracted again in SystemView.
//!
//! Global matrices are CSR; reduced matrices are dense under the current
//! engine-linalg SymMatrix contract. Solving is delegated to the backend, without
//! regularization or threads. Scalability of the reference solver is not promised.
//!
//! Boundary example: two axial DOFs, K supplied by the caller.
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
pub mod diaphragm;
pub mod dofmap;
pub mod reactions;
pub mod system;

pub use assemble::{ElementContribution, ElementLoad};
pub use constraints::{Constraints, PrescribedDof};
pub use diaphragm::{DiaphragmForces, DiaphragmNodalForce, RigidDiaphragm};
pub use dofmap::DofMap;
pub use reactions::Reaction;
pub use system::{GeometricEval, GeometricProducer, LinearSystem};

use engine_core::{EResult, EngineError, Real};

fn error(code: &'static str, message: impl Into<String>) -> EngineError {
    let mut result = match EngineError::from_code(code) {
        Some(e) => e,
        None => EngineError::new("ENG-E-0012", "Código de error no registrado."),
    };
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
