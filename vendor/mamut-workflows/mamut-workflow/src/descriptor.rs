// SPDX-License-Identifier: Apache-2.0

//! Method descriptors.
//!
//! Field for field, these are the generic records in Hydrology's
//! `MethodDescriptor`: id, numeric version, capability slot, input and output
//! quantities, and parameters. The slot, dimension, and unit are strings.
//! Hydrology keeps its `CapabilitySlot` and `Unit` enums in that engine and
//! maps them to these strings at the boundary.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::contract::ContractId;

/// Numeric parameter table. Keys are parameter names.
///
/// This is Hydrology's `NumericParameters`: an ordered map of `f64` values.
/// The compiler checks names, not ranges.
pub type NumericParameters = BTreeMap<String, f64>;

/// The numeric shape of one method parameter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterDescriptor {
    /// Parameter name.
    pub name: String,
    /// When true, a selection must include this name.
    pub required: bool,
    /// Dimension identity supplied by the engine. Compared as text.
    pub dimension: String,
    /// Unit symbol supplied by the engine. Compared as text.
    pub unit: String,
}

impl ParameterDescriptor {
    /// A required parameter.
    pub fn required(
        name: impl Into<String>,
        dimension: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            required: true,
            dimension: dimension.into(),
            unit: unit.into(),
        }
    }

    /// An optional parameter.
    pub fn optional(
        name: impl Into<String>,
        dimension: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            required: false,
            dimension: dimension.into(),
            unit: unit.into(),
        }
    }
}

/// One named method input or output and its unit contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantityDescriptor {
    /// Port name.
    pub name: String,
    /// Dimension identity. Two quantities match only when this string matches.
    pub dimension: String,
    /// Unit symbol recorded with the dimension.
    pub unit: String,
}

impl QuantityDescriptor {
    /// A quantity with an explicit dimension and unit.
    pub fn new(
        name: impl Into<String>,
        dimension: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            dimension: dimension.into(),
            unit: unit.into(),
        }
    }
}

/// Dimensional method boundary used for discovery and compiler validation.
///
/// Order is significant, as it is on Hydrology's static slices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodDimensions {
    /// Quantities the method consumes.
    pub inputs: Vec<QuantityDescriptor>,
    /// Quantities the method produces.
    pub outputs: Vec<QuantityDescriptor>,
}

/// Discoverable metadata for one registered method implementation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodDescriptor {
    /// Stable method id. Not a contract URN.
    pub method_id: String,
    /// Method version. An integer, separate from any contract semver.
    pub method_version: u32,
    /// Capability this method fills. The engine chooses the vocabulary.
    pub capability_slot: String,
    /// Inputs and outputs the compiler compares to the slot's canonical dimensions.
    pub dimensions: MethodDimensions,
    /// Contracts consumed, in the same order as [`MethodDimensions::inputs`].
    ///
    /// Substitution compares each id with the slot's contract at the same index
    /// by domain, name, and major version. Minor and patch may differ.
    pub consumes: Vec<ContractId>,
    /// Contracts produced, in the same order as [`MethodDimensions::outputs`].
    pub produces: Vec<ContractId>,
    /// Declared parameters.
    pub parameters: Vec<ParameterDescriptor>,
    /// Message the shared CLI uses when `run` reaches this method.
    ///
    /// Absent, the CLI returns a deterministic JSON identity for the step.
    /// Engines that supply their own [`crate::MethodFactory`] do not read this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// A versioned method selection in a capability slot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodSelectionV1 {
    /// Slot the caller claims this selection fills.
    pub capability_slot: String,
    /// Method to resolve.
    pub method_id: String,
    /// Exact version. The compiler does not guess a latest version.
    pub method_version: u32,
    /// Parameter values.
    pub parameters: NumericParameters,
}
