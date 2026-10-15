// SPDX-License-Identifier: Apache-2.0

//! Descriptor-backed method catalog.
//!
//! Hydrology's `MethodRegistry` stores a descriptor and a factory under
//! `(capability slot, method id, version)`. Discovery lists those descriptors
//! in that order. The factories here are plain function pointers; the concrete
//! hydrology methods stay in that engine.

use std::collections::BTreeMap;

use crate::descriptor::{MethodDescriptor, NumericParameters};
use crate::error::{CompileError, RegistryError};

/// Value returned by [`MethodCatalog::create`].
///
/// `capability_slot` is the slot the factory claims to have built. The compiler
/// rejects a product whose slot is not the slot it asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodProduct<T> {
    /// Slot reported by the factory.
    pub capability_slot: String,
    /// Engine-defined method value.
    pub instance: T,
}

/// Read-only catalog the compiler consults.
///
/// `Instance` is the engine's method value. Hydrology uses an enum of trait
/// objects; this crate does not.
pub trait MethodCatalog {
    /// What [`Self::create`] builds.
    type Instance;

    /// Descriptors from the actual registered catalog.
    fn descriptors(&self) -> Vec<&MethodDescriptor>;

    /// Build the implementation for an already-resolved descriptor.
    fn create(
        &self,
        descriptor: &MethodDescriptor,
        parameters: &NumericParameters,
    ) -> Result<MethodProduct<Self::Instance>, CompileError>;
}

/// Factory stored next to a descriptor.
///
/// The string error becomes [`CompileError::Factory`].
pub type MethodFactory<T> = fn(&NumericParameters) -> Result<T, String>;

struct RegisteredMethod<T> {
    descriptor: MethodDescriptor,
    factory: MethodFactory<T>,
}

/// In-memory catalog keyed by slot, method id, and version.
#[derive(Default)]
pub struct MethodRegistry<T> {
    methods: BTreeMap<(String, String, u32), RegisteredMethod<T>>,
}

impl<T> MethodRegistry<T> {
    /// Empty catalog.
    pub fn new() -> Self {
        Self {
            methods: BTreeMap::new(),
        }
    }

    /// Insert a descriptor and the function that builds its instance.
    pub fn register(
        &mut self,
        descriptor: MethodDescriptor,
        factory: MethodFactory<T>,
    ) -> Result<(), RegistryError> {
        validate(&descriptor)?;
        let key = (
            descriptor.capability_slot.clone(),
            descriptor.method_id.clone(),
            descriptor.method_version,
        );
        if self.methods.contains_key(&key) {
            return Err(RegistryError::Duplicate {
                capability_slot: key.0,
                method_id: key.1,
                method_version: key.2,
            });
        }
        self.methods.insert(
            key,
            RegisteredMethod {
                descriptor,
                factory,
            },
        );
        Ok(())
    }
}

impl<T> MethodCatalog for MethodRegistry<T> {
    type Instance = T;

    fn descriptors(&self) -> Vec<&MethodDescriptor> {
        self.methods
            .values()
            .map(|registered| &registered.descriptor)
            .collect()
    }

    fn create(
        &self,
        descriptor: &MethodDescriptor,
        parameters: &NumericParameters,
    ) -> Result<MethodProduct<T>, CompileError> {
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
        let instance =
            (registered.factory)(parameters).map_err(|message| CompileError::Factory {
                method_id: descriptor.method_id.clone(),
                message,
            })?;
        Ok(MethodProduct {
            capability_slot: descriptor.capability_slot.clone(),
            instance,
        })
    }
}

pub(crate) fn validate(descriptor: &MethodDescriptor) -> Result<(), RegistryError> {
    if descriptor.method_id.is_empty() {
        return Err(RegistryError::EmptyMethodId);
    }
    if descriptor.capability_slot.is_empty() {
        return Err(RegistryError::EmptyCapabilitySlot {
            method_id: descriptor.method_id.clone(),
        });
    }
    for quantity in descriptor
        .dimensions
        .inputs
        .iter()
        .chain(descriptor.dimensions.outputs.iter())
    {
        if quantity.name.is_empty() {
            return Err(RegistryError::EmptyQuantityName {
                method_id: descriptor.method_id.clone(),
            });
        }
    }
    if descriptor.consumes.len() != descriptor.dimensions.inputs.len() {
        return Err(RegistryError::ContractCount {
            method_id: descriptor.method_id.clone(),
            direction: "input".to_owned(),
            quantities: descriptor.dimensions.inputs.len(),
            contracts: descriptor.consumes.len(),
        });
    }
    if descriptor.produces.len() != descriptor.dimensions.outputs.len() {
        return Err(RegistryError::ContractCount {
            method_id: descriptor.method_id.clone(),
            direction: "output".to_owned(),
            quantities: descriptor.dimensions.outputs.len(),
            contracts: descriptor.produces.len(),
        });
    }
    let mut names: Vec<&str> = Vec::with_capacity(descriptor.parameters.len());
    for parameter in &descriptor.parameters {
        if parameter.name.is_empty() {
            return Err(RegistryError::InvalidParameter {
                method_id: descriptor.method_id.clone(),
                parameter: String::new(),
                reason: "the name is empty".to_owned(),
            });
        }
        if names.contains(&parameter.name.as_str()) {
            return Err(RegistryError::InvalidParameter {
                method_id: descriptor.method_id.clone(),
                parameter: parameter.name.clone(),
                reason: "the name is repeated".to_owned(),
            });
        }
        names.push(parameter.name.as_str());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MethodCatalog, MethodRegistry};
    use crate::contract::ContractId;
    use crate::descriptor::{
        MethodDescriptor, MethodDimensions, ParameterDescriptor, QuantityDescriptor,
    };
    use crate::error::RegistryError;

    fn quantity(name: &str) -> QuantityDescriptor {
        QuantityDescriptor::new(name, "L", "m")
    }

    fn contract() -> ContractId {
        ContractId::parse("urn:mamut:sequence:length:1.0.0").unwrap()
    }

    fn descriptor(slot: &str, method_id: &str, version: u32) -> MethodDescriptor {
        MethodDescriptor {
            method_id: method_id.to_owned(),
            method_version: version,
            capability_slot: slot.to_owned(),
            dimensions: MethodDimensions {
                inputs: vec![quantity("in")],
                outputs: vec![quantity("out")],
            },
            consumes: vec![contract()],
            produces: vec![contract()],
            parameters: vec![ParameterDescriptor::required("n", "1", "1")],
            failure: None,
        }
    }

    fn factory(_: &crate::NumericParameters) -> Result<&'static str, String> {
        Ok("built")
    }

    #[test]
    fn discovery_follows_slot_then_id_then_version() {
        let mut registry = MethodRegistry::new();
        registry
            .register(descriptor("second", "b", 2), factory)
            .unwrap();
        registry
            .register(descriptor("first", "a", 1), factory)
            .unwrap();
        registry
            .register(descriptor("second", "b", 1), factory)
            .unwrap();
        let ids: Vec<(&str, u32)> = registry
            .descriptors()
            .iter()
            .map(|descriptor| (descriptor.method_id.as_str(), descriptor.method_version))
            .collect();
        assert_eq!(ids, vec![("a", 1), ("b", 1), ("b", 2)]);
        assert!(registry
            .register(descriptor("first", "a", 1), factory)
            .is_err());
    }

    #[test]
    fn registration_rejects_a_contract_count_mismatch() {
        let mut broken = descriptor("first", "a", 1);
        broken.consumes.clear();
        let error = MethodRegistry::new().register(broken, factory).unwrap_err();
        assert!(matches!(
            error,
            RegistryError::ContractCount {
                direction,
                quantities: 1,
                contracts: 0,
                ..
            } if direction == "input"
        ));
    }
}
