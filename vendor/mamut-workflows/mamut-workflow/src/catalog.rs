// SPDX-License-Identifier: Apache-2.0

//! Catalog document loaded by the CLI and the MCP server.
//!
//! [`MethodRegistry`](crate::MethodRegistry) stores a function pointer next to
//! each descriptor. A process that only has JSON cannot call those pointers.
//! [`RegisteredCatalog`] keeps the descriptors and the profile, and its
//! `create` returns the registered method identity. It does not run a method.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;

use crate::compiler::{CompiledWorkflow, WorkflowCompiler};
use crate::descriptor::MethodDescriptor;
use crate::error::{CompileError, RegistryError};
use crate::registry::{validate, MethodCatalog, MethodProduct};
use crate::workflow::{ProfileContract, WorkflowDefinitionV1};
use crate::NumericParameters;

/// JSON document an engine hands to the shared CLI or MCP server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogDocument {
    /// Profile whose slots the compiler expects, in step order.
    pub profile: ProfileContract,
    /// Methods already registered for that profile.
    pub methods: Vec<MethodDescriptor>,
}

/// Identity returned when a descriptor is compiled without an engine factory.
///
/// The step still carries the slot, id, version, and parameters. This value
/// records which registered descriptor `create` resolved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredMethod {
    /// Method id copied from the descriptor.
    pub method_id: String,
    /// Method version copied from the descriptor.
    pub method_version: u32,
    /// Slot copied from the descriptor.
    pub capability_slot: String,
}

/// Descriptors and profile loaded from a [`CatalogDocument`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredCatalog {
    profile: ProfileContract,
    methods: BTreeMap<(String, String, u32), MethodDescriptor>,
}

impl RegisteredCatalog {
    /// Validate `document` and index its methods by slot, id, and version.
    pub fn try_from_document(document: CatalogDocument) -> Result<Self, RegistryError> {
        let mut methods = BTreeMap::new();
        for descriptor in document.methods {
            validate(&descriptor)?;
            let key = (
                descriptor.capability_slot.clone(),
                descriptor.method_id.clone(),
                descriptor.method_version,
            );
            if methods.contains_key(&key) {
                return Err(RegistryError::Duplicate {
                    capability_slot: key.0,
                    method_id: key.1,
                    method_version: key.2,
                });
            }
            methods.insert(key, descriptor);
        }
        Ok(Self {
            profile: document.profile,
            methods,
        })
    }

    /// Parse a [`CatalogDocument`] and register its methods.
    pub fn from_json(text: &str) -> Result<Self, CatalogError> {
        let document = serde_json::from_str(text).map_err(CatalogError::json)?;
        Self::try_from_document(document).map_err(CatalogError::Registry)
    }

    /// Profile stored with the methods.
    pub fn profile(&self) -> &ProfileContract {
        &self.profile
    }

    /// JSON array of descriptors in registry order.
    pub fn methods_json(&self) -> Result<String, JsonError> {
        serde_json::to_string(&self.descriptors())
    }

    /// Compile `definition` with this catalog's profile.
    pub fn compile(
        &self,
        definition: &WorkflowDefinitionV1,
    ) -> Result<CompiledWorkflow<RegisteredMethod>, CompileError> {
        WorkflowCompiler::new(self, self.profile.clone()).compile(definition)
    }

    /// Parse a workflow document and [`Self::compile`] it.
    pub fn compile_json(
        &self,
        text: &str,
    ) -> Result<CompiledWorkflow<RegisteredMethod>, WorkflowDocumentError> {
        let definition = serde_json::from_str(text).map_err(WorkflowDocumentError::json)?;
        self.compile(&definition)
            .map_err(WorkflowDocumentError::Compile)
    }
}

impl MethodCatalog for RegisteredCatalog {
    type Instance = RegisteredMethod;

    fn descriptors(&self) -> Vec<&MethodDescriptor> {
        self.methods.values().collect()
    }

    fn create(
        &self,
        descriptor: &MethodDescriptor,
        _parameters: &NumericParameters,
    ) -> Result<MethodProduct<RegisteredMethod>, CompileError> {
        let key = (
            descriptor.capability_slot.clone(),
            descriptor.method_id.clone(),
            descriptor.method_version,
        );
        let Some(registered) = self.methods.get(&key) else {
            return Err(CompileError::UnknownMethod {
                capability_slot: descriptor.capability_slot.clone(),
                method_id: descriptor.method_id.clone(),
            });
        };
        Ok(MethodProduct {
            capability_slot: registered.capability_slot.clone(),
            instance: RegisteredMethod {
                method_id: registered.method_id.clone(),
                method_version: registered.method_version,
                capability_slot: registered.capability_slot.clone(),
            },
        })
    }
}

/// A catalog document could not be loaded.
#[derive(Debug)]
pub enum CatalogError {
    /// The text is not a catalog document.
    Json(String),
    /// A descriptor failed registration.
    Registry(RegistryError),
}

impl CatalogError {
    fn json(error: JsonError) -> Self {
        Self::Json(error.to_string())
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(message) => write!(f, "catalog document: {message}"),
            Self::Registry(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(_) => None,
            Self::Registry(error) => Some(error),
        }
    }
}

/// A workflow document could not be compiled.
#[derive(Debug)]
pub enum WorkflowDocumentError {
    /// The text is not a workflow definition.
    Json(String),
    /// The compiler rejected the definition.
    Compile(CompileError),
}

impl WorkflowDocumentError {
    fn json(error: JsonError) -> Self {
        Self::Json(error.to_string())
    }
}

impl fmt::Display for WorkflowDocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(message) => write!(f, "workflow document: {message}"),
            Self::Compile(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for WorkflowDocumentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(_) => None,
            Self::Compile(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RegisteredCatalog;

    const CATALOG: &str = r#"{
      "profile": {
        "profile": "sequence.v1",
        "version": 1,
        "slots": [
          {
            "capability_slot": "first",
            "dimensions": {
              "inputs": [{"name": "in", "dimension": "L", "unit": "m"}],
              "outputs": [{"name": "out", "dimension": "L", "unit": "m"}]
            },
            "consumes": ["urn:mamut:sequence:stage:1.0.0"],
            "produces": ["urn:mamut:sequence:stage:1.0.0"]
          },
          {
            "capability_slot": "second",
            "dimensions": {
              "inputs": [{"name": "in", "dimension": "L3/T", "unit": "m3/s"}],
              "outputs": [{"name": "out", "dimension": "L3/T", "unit": "m3/s"}]
            },
            "consumes": ["urn:mamut:sequence:inflow:1.0.0"],
            "produces": ["urn:mamut:sequence:outflow:1.0.0"]
          }
        ]
      },
      "methods": [
        {
          "method_id": "method-a",
          "method_version": 1,
          "capability_slot": "first",
          "dimensions": {
            "inputs": [{"name": "in", "dimension": "L", "unit": "m"}],
            "outputs": [{"name": "out", "dimension": "L", "unit": "m"}]
          },
          "consumes": ["urn:mamut:sequence:stage:1.0.0"],
          "produces": ["urn:mamut:sequence:stage:1.0.0"],
          "parameters": [{"name": "n", "required": true, "dimension": "1", "unit": "1"}]
        },
        {
          "method_id": "method-b",
          "method_version": 1,
          "capability_slot": "second",
          "dimensions": {
            "inputs": [{"name": "in", "dimension": "L3/T", "unit": "m3/s"}],
            "outputs": [{"name": "out", "dimension": "L3/T", "unit": "m3/s"}]
          },
          "consumes": ["urn:mamut:sequence:inflow:1.0.0"],
          "produces": ["urn:mamut:sequence:outflow:1.0.0"],
          "parameters": []
        },
        {
          "method_id": "method-c",
          "method_version": 1,
          "capability_slot": "second",
          "dimensions": {
            "inputs": [{"name": "in", "dimension": "L3/T", "unit": "m3/s"}],
            "outputs": [{"name": "out", "dimension": "L3/T", "unit": "m3/s"}]
          },
          "consumes": ["urn:mamut:sequence:inflow:1.2.0"],
          "produces": ["urn:mamut:sequence:outflow:1.4.1"],
          "parameters": []
        }
      ]
    }"#;

    fn workflow(second: &str) -> String {
        format!(
            r#"{{
              "profile": "sequence.v1",
              "version": 1,
              "steps": [
                {{
                  "capability_slot": "first",
                  "method_id": "method-a",
                  "method_version": 1,
                  "parameters": {{"n": 2.0}}
                }},
                {{
                  "capability_slot": "second",
                  "method_id": "{second}",
                  "method_version": 1,
                  "parameters": {{}}
                }}
              ]
            }}"#
        )
    }

    #[test]
    fn lists_methods_and_swaps_the_second_step() {
        let catalog = RegisteredCatalog::from_json(CATALOG).unwrap();
        let listed: Vec<serde_json::Value> =
            serde_json::from_str(&catalog.methods_json().unwrap()).unwrap();
        let ids: Vec<&str> = listed
            .iter()
            .map(|value| value["method_id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["method-a", "method-b", "method-c"]);

        let compiled = catalog.compile_json(&workflow("method-b")).unwrap();
        assert_eq!(compiled.steps[1].method_id, "method-b");
        assert_eq!(compiled.steps[1].instance.method_id, "method-b");
        let swapped = catalog.compile_json(&workflow("method-c")).unwrap();
        assert_eq!(swapped.steps[1].method_id, "method-c");
        assert_ne!(compiled, swapped);
    }

    #[test]
    fn rejects_a_duplicate_registration() {
        let mut catalog = serde_json::from_str::<serde_json::Value>(CATALOG).unwrap();
        let repeated = catalog["methods"][1].clone();
        catalog["methods"].as_array_mut().unwrap().push(repeated);
        let error = RegisteredCatalog::from_json(&catalog.to_string()).unwrap_err();
        assert!(error.to_string().contains("already registered"));
    }
}
