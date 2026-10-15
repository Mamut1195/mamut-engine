// SPDX-License-Identifier: Apache-2.0

//! Method registry and workflow compiler.
//!
//! The types follow the generic half of Hydrology's `workflow.rs` and
//! `registry.rs`: a descriptor, a catalog that can build a method, and a
//! compiler that resolves each selection before anything runs. This crate does
//! not dispatch on `method_id` inside an orchestrator, and it does not know
//! hydrology's capability enum or basin graph.
//!
//! The `mamut-workflow` binary and the stdio MCP server live in
//! `mamut-workflow-run`.

#![deny(missing_docs)]

mod catalog;
mod compiler;
mod contract;
mod descriptor;
mod error;
mod registry;
mod workflow;

pub use catalog::{
    CatalogDocument, CatalogError, RegisteredCatalog, RegisteredMethod, WorkflowDocumentError,
};
pub use compiler::{CompiledStep, CompiledWorkflow, WorkflowCompiler};
pub use contract::{ContractId, ContractIdError};
pub use descriptor::{
    MethodDescriptor, MethodDimensions, MethodSelectionV1, NumericParameters, ParameterDescriptor,
    QuantityDescriptor,
};
pub use error::{CompileError, ContractMismatch, RegistryError};
pub use registry::{MethodCatalog, MethodFactory, MethodProduct, MethodRegistry};
pub use workflow::{ProfileContract, SlotContract, WorkflowDefinitionV1};
