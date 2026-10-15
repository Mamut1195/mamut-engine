// SPDX-License-Identifier: Apache-2.0

//! Errors from registration and compilation.
//!
//! The variants follow Hydrology's compiler failures: unknown method,
//! unsupported version, slot mismatch, dimensional contract, and parameter
//! shape. There is no diagnostic schema crate here.

use std::fmt;

/// A descriptor was rejected before it entered the catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// `method_id` is empty.
    EmptyMethodId,
    /// `capability_slot` is empty.
    EmptyCapabilitySlot {
        /// Method that failed validation.
        method_id: String,
    },
    /// This slot, id, and version are already registered.
    Duplicate {
        /// Capability slot.
        capability_slot: String,
        /// Method id.
        method_id: String,
        /// Method version.
        method_version: u32,
    },
    /// A quantity name is empty.
    EmptyQuantityName {
        /// Method id.
        method_id: String,
    },
    /// A parameter name is empty or repeated.
    InvalidParameter {
        /// Method id.
        method_id: String,
        /// Parameter name, when there is one.
        parameter: String,
        /// What failed.
        reason: String,
    },
    /// Consumed or produced contract ids are not aligned with the quantities.
    ContractCount {
        /// Method id.
        method_id: String,
        /// `input` or `output`.
        direction: String,
        /// Number of quantities on that side.
        quantities: usize,
        /// Number of contract ids on that side.
        contracts: usize,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMethodId => f.write_str("method_id is empty"),
            Self::EmptyCapabilitySlot { method_id } => {
                write!(f, "method `{method_id}` has an empty capability slot")
            }
            Self::Duplicate {
                capability_slot,
                method_id,
                method_version,
            } => write!(
                f,
                "method `{method_id}` version {method_version} is already registered for slot `{capability_slot}`"
            ),
            Self::EmptyQuantityName { method_id } => {
                write!(f, "method `{method_id}` has a quantity with an empty name")
            }
            Self::InvalidParameter {
                method_id,
                parameter,
                reason,
            } => write!(f, "parameter `{parameter}` of method `{method_id}`: {reason}"),
            Self::ContractCount {
                method_id,
                direction,
                quantities,
                contracts,
            } => write!(
                f,
                "method `{method_id}` has {contracts} {direction} contracts and {quantities} {direction} quantities"
            ),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Compilation failed before a runnable plan existed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    /// `profile` is not the one this compiler was built for.
    UnsupportedProfile {
        /// Profile on the definition.
        profile: String,
        /// Profile expected by the compiler.
        expected: String,
    },
    /// Definition `version` is not the profile version.
    UnsupportedVersion {
        /// Version on the definition.
        version: u32,
        /// Version expected by the compiler.
        expected: u32,
    },
    /// The definition does not have one selection per profile slot.
    StepCount {
        /// Number of slots in the profile.
        expected: usize,
        /// Number of selections in the definition.
        actual: usize,
    },
    /// No descriptor uses this method id.
    UnknownMethod {
        /// Slot that asked for the method.
        capability_slot: String,
        /// Method id from the selection.
        method_id: String,
    },
    /// The method id exists for the slot, but not at this version.
    UnsupportedMethodVersion {
        /// Method id.
        method_id: String,
        /// Requested version.
        requested: u32,
        /// Registered versions, ascending, separated by `", "`.
        supported: String,
    },
    /// The selection or the descriptor names a different slot than the profile step.
    CapabilitySlotMismatch {
        /// Method id from the selection.
        method_id: String,
        /// Slot required by this step.
        expected: String,
        /// Slot on the selection or on the descriptor.
        actual: String,
    },
    /// The descriptor's inputs and outputs are not the slot's canonical dimensions.
    IncompatibleDimensions {
        /// Method id.
        method_id: String,
        /// Method version.
        method_version: u32,
        /// Slot whose canonical dimensions were required.
        capability_slot: String,
    },
    /// A consumed or produced contract is not the slot's contract at the same major.
    ///
    /// Compilation stops. No plan is returned. The details are boxed so this
    /// error stays the size of the other compile failures.
    IncompatibleContract(Box<ContractMismatch>),
    /// A required parameter is absent.
    MissingParameter {
        /// Parameter name.
        parameter: String,
        /// Method id.
        method_id: String,
        /// Method version.
        method_version: u32,
    },
    /// The selection names a parameter the descriptor does not declare.
    UnexpectedParameter {
        /// Parameter name.
        parameter: String,
        /// Method id.
        method_id: String,
        /// Method version.
        method_version: u32,
    },
    /// [`crate::MethodCatalog::create`] reported a different slot than the step.
    FactorySlotMismatch {
        /// Method id.
        method_id: String,
        /// Slot required by this step.
        expected: String,
        /// Slot reported by the factory.
        actual: String,
    },
    /// The factory failed.
    Factory {
        /// Method id.
        method_id: String,
        /// Factory message.
        message: String,
    },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProfile { profile, expected } => write!(
                f,
                "unsupported profile `{profile}`; expected `{expected}`"
            ),
            Self::UnsupportedVersion { version, expected } => write!(
                f,
                "unsupported WorkflowDefinition version {version}; expected {expected}"
            ),
            Self::StepCount { expected, actual } => write!(
                f,
                "workflow has {actual} selections; profile expects {expected}"
            ),
            Self::UnknownMethod {
                capability_slot,
                method_id,
            } => write!(
                f,
                "unknown method `{method_id}` for capability slot `{capability_slot}`"
            ),
            Self::UnsupportedMethodVersion {
                method_id,
                requested,
                supported,
            } => write!(
                f,
                "unsupported version {requested} for method `{method_id}`; supported versions: {supported}"
            ),
            Self::CapabilitySlotMismatch {
                method_id,
                expected,
                actual,
            } => write!(
                f,
                "method `{method_id}` is registered for capability slot `{actual}`, expected `{expected}`"
            ),
            Self::IncompatibleDimensions {
                method_id,
                method_version,
                capability_slot,
            } => write!(
                f,
                "method `{method_id}` version {method_version} declares a dimensional contract incompatible with capability slot `{capability_slot}`"
            ),
            Self::IncompatibleContract(mismatch) => write!(f, "{mismatch}"),
            Self::MissingParameter {
                parameter,
                method_id,
                method_version,
            } => write!(
                f,
                "required parameter `{parameter}` is missing for method `{method_id}` version {method_version}"
            ),
            Self::UnexpectedParameter {
                parameter,
                method_id,
                method_version,
            } => write!(
                f,
                "unexpected parameter `{parameter}` for method `{method_id}` version {method_version}"
            ),
            Self::FactorySlotMismatch {
                method_id,
                expected,
                actual,
            } => write!(
                f,
                "factory for method `{method_id}` returned capability slot `{actual}`, expected `{expected}`"
            ),
            Self::Factory { method_id, message } => {
                write!(f, "factory for method `{method_id}` failed: {message}")
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// Diagnostic for a method that cannot replace the slot's contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractMismatch {
    /// Method id.
    pub method_id: String,
    /// Method version.
    pub method_version: u32,
    /// Slot whose contracts were required.
    pub capability_slot: String,
    /// `input` or `output`.
    pub direction: String,
    /// Quantity name when both sides have a contract at this index.
    pub port: Option<String>,
    /// Contract the slot requires, or the slot's contract list when the counts differ.
    pub expected: String,
    /// Contract the method declares, or the method's contract list when the counts differ.
    pub actual: String,
}

impl fmt::Display for ContractMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.port {
            Some(port) => write!(
                f,
                "method `{}` version {} on slot `{}` {} `{port}` declares contract `{}`; slot requires `{}` at the same major version",
                self.method_id, self.method_version, self.capability_slot, self.direction, self.actual, self.expected
            ),
            None => write!(
                f,
                "method `{}` version {} on slot `{}` has {} contracts `{}`; slot requires `{}`",
                self.method_id, self.method_version, self.capability_slot, self.direction, self.actual, self.expected
            ),
        }
    }
}
