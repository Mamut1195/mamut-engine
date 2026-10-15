// SPDX-License-Identifier: Apache-2.0

//! Workflow compiler.
//!
//! Resolution follows Hydrology's `WorkflowCompiler::compile_method`: the
//! selection's slot must be the step's slot, the method id must exist, the
//! version must be one of the versions registered for that slot, the
//! descriptor dimensions must equal the slot's canonical dimensions, each
//! consumed and produced contract must match the slot's contract at the same
//! major version, the parameter names must match the descriptor, and the
//! factory must report the same slot. The compiled steps are in definition
//! order. A contract mismatch returns [`CompileError`] and does not build a plan.

use serde::Serialize;

use crate::contract::ContractId;
use crate::descriptor::{
    MethodDescriptor, MethodSelectionV1, NumericParameters, QuantityDescriptor,
};
use crate::error::{CompileError, ContractMismatch};
use crate::registry::MethodCatalog;
use crate::workflow::{ProfileContract, SlotContract, WorkflowDefinitionV1};

/// One resolved step. The engine runs `instance`; it does not look up `method_id` again.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompiledStep<T> {
    /// Slot this step filled.
    pub capability_slot: String,
    /// Resolved method id.
    pub method_id: String,
    /// Resolved method version.
    pub method_version: u32,
    /// Parameter values that passed the shape check.
    pub parameters: NumericParameters,
    /// Value returned by the catalog factory.
    pub instance: T,
}

/// Plan accepted by an engine runtime.
///
/// Hydrology's compiled workflow owns a basin graph. This one owns the
/// resolved steps. The engine builds its own graph from them.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompiledWorkflow<T> {
    /// Profile copied from the definition.
    pub profile: String,
    /// Version copied from the definition.
    pub version: u32,
    /// Steps in definition order.
    pub steps: Vec<CompiledStep<T>>,
}

/// Resolves a [`WorkflowDefinitionV1`] against a [`MethodCatalog`] and a profile.
pub struct WorkflowCompiler<'a, C: MethodCatalog + ?Sized> {
    catalog: &'a C,
    profile: ProfileContract,
}

impl<'a, C: MethodCatalog + ?Sized> WorkflowCompiler<'a, C> {
    /// Borrow a catalog and keep the profile this compiler accepts.
    pub fn new(catalog: &'a C, profile: ProfileContract) -> Self {
        Self { catalog, profile }
    }

    /// Compile `definition`. The first failure is returned.
    pub fn compile(
        &self,
        definition: &WorkflowDefinitionV1,
    ) -> Result<CompiledWorkflow<C::Instance>, CompileError> {
        if definition.profile != self.profile.profile {
            return Err(CompileError::UnsupportedProfile {
                profile: definition.profile.clone(),
                expected: self.profile.profile.clone(),
            });
        }
        if definition.version != self.profile.version {
            return Err(CompileError::UnsupportedVersion {
                version: definition.version,
                expected: self.profile.version,
            });
        }
        if definition.steps.len() != self.profile.slots.len() {
            return Err(CompileError::StepCount {
                expected: self.profile.slots.len(),
                actual: definition.steps.len(),
            });
        }

        let mut steps = Vec::with_capacity(definition.steps.len());
        for (selection, slot) in definition.steps.iter().zip(self.profile.slots.iter()) {
            steps.push(self.compile_selection(selection, slot)?);
        }
        Ok(CompiledWorkflow {
            profile: definition.profile.clone(),
            version: definition.version,
            steps,
        })
    }

    fn compile_selection(
        &self,
        selection: &MethodSelectionV1,
        expected: &SlotContract,
    ) -> Result<CompiledStep<C::Instance>, CompileError> {
        if selection.capability_slot != expected.capability_slot {
            return Err(CompileError::CapabilitySlotMismatch {
                method_id: selection.method_id.clone(),
                expected: expected.capability_slot.clone(),
                actual: selection.capability_slot.clone(),
            });
        }

        let descriptors = self.catalog.descriptors();
        let same_id: Vec<&MethodDescriptor> = descriptors
            .into_iter()
            .filter(|descriptor| descriptor.method_id == selection.method_id)
            .collect();
        if same_id.is_empty() {
            return Err(CompileError::UnknownMethod {
                capability_slot: expected.capability_slot.clone(),
                method_id: selection.method_id.clone(),
            });
        }

        let same_slot: Vec<&MethodDescriptor> = same_id
            .iter()
            .copied()
            .filter(|descriptor| descriptor.capability_slot == expected.capability_slot)
            .collect();
        if same_slot.is_empty() {
            return Err(CompileError::CapabilitySlotMismatch {
                method_id: selection.method_id.clone(),
                expected: expected.capability_slot.clone(),
                actual: same_id[0].capability_slot.clone(),
            });
        }

        let descriptor = same_slot
            .iter()
            .copied()
            .find(|descriptor| descriptor.method_version == selection.method_version)
            .ok_or_else(|| {
                let mut versions: Vec<u32> = same_slot
                    .iter()
                    .map(|descriptor| descriptor.method_version)
                    .collect();
                versions.sort_unstable();
                versions.dedup();
                CompileError::UnsupportedMethodVersion {
                    method_id: selection.method_id.clone(),
                    requested: selection.method_version,
                    supported: versions
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                }
            })?;

        if descriptor.dimensions != expected.dimensions {
            return Err(CompileError::IncompatibleDimensions {
                method_id: descriptor.method_id.clone(),
                method_version: descriptor.method_version,
                capability_slot: expected.capability_slot.clone(),
            });
        }

        compatible_contracts(descriptor, expected)?;
        validate_parameter_shape(descriptor, &selection.parameters)?;
        let product = self.catalog.create(descriptor, &selection.parameters)?;
        if product.capability_slot != expected.capability_slot {
            return Err(CompileError::FactorySlotMismatch {
                method_id: selection.method_id.clone(),
                expected: expected.capability_slot.clone(),
                actual: product.capability_slot,
            });
        }

        Ok(CompiledStep {
            capability_slot: expected.capability_slot.clone(),
            method_id: descriptor.method_id.clone(),
            method_version: descriptor.method_version,
            parameters: selection.parameters.clone(),
            instance: product.instance,
        })
    }
}

fn compatible_contracts(
    descriptor: &MethodDescriptor,
    expected: &SlotContract,
) -> Result<(), CompileError> {
    check_ports(
        descriptor,
        &expected.capability_slot,
        "input",
        &expected.dimensions.inputs,
        &expected.consumes,
        &descriptor.consumes,
    )?;
    check_ports(
        descriptor,
        &expected.capability_slot,
        "output",
        &expected.dimensions.outputs,
        &expected.produces,
        &descriptor.produces,
    )
}

fn check_ports(
    descriptor: &MethodDescriptor,
    slot: &str,
    direction: &str,
    quantities: &[QuantityDescriptor],
    expected_contracts: &[ContractId],
    actual_contracts: &[ContractId],
) -> Result<(), CompileError> {
    if expected_contracts.len() != quantities.len() || actual_contracts.len() != quantities.len() {
        return Err(contract_mismatch(
            descriptor,
            slot,
            direction,
            None,
            join_contracts(expected_contracts),
            join_contracts(actual_contracts),
        ));
    }
    for (quantity, (expected_id, actual_id)) in quantities
        .iter()
        .zip(expected_contracts.iter().zip(actual_contracts.iter()))
    {
        if !actual_id.major_compatible(expected_id) {
            return Err(contract_mismatch(
                descriptor,
                slot,
                direction,
                Some(quantity.name.clone()),
                expected_id.urn(),
                actual_id.urn(),
            ));
        }
    }
    Ok(())
}

fn contract_mismatch(
    descriptor: &MethodDescriptor,
    slot: &str,
    direction: &str,
    port: Option<String>,
    expected: String,
    actual: String,
) -> CompileError {
    CompileError::IncompatibleContract(Box::new(ContractMismatch {
        method_id: descriptor.method_id.clone(),
        method_version: descriptor.method_version,
        capability_slot: slot.to_owned(),
        direction: direction.to_owned(),
        port,
        expected,
        actual,
    }))
}

fn join_contracts(contracts: &[ContractId]) -> String {
    contracts
        .iter()
        .map(ContractId::urn)
        .collect::<Vec<_>>()
        .join(", ")
}

fn validate_parameter_shape(
    descriptor: &MethodDescriptor,
    parameters: &NumericParameters,
) -> Result<(), CompileError> {
    for expected in &descriptor.parameters {
        if expected.required && !parameters.contains_key(&expected.name) {
            return Err(CompileError::MissingParameter {
                parameter: expected.name.clone(),
                method_id: descriptor.method_id.clone(),
                method_version: descriptor.method_version,
            });
        }
    }
    for supplied in parameters.keys() {
        if !descriptor
            .parameters
            .iter()
            .any(|expected| expected.name == *supplied)
        {
            return Err(CompileError::UnexpectedParameter {
                parameter: supplied.clone(),
                method_id: descriptor.method_id.clone(),
                method_version: descriptor.method_version,
            });
        }
    }
    Ok(())
}
