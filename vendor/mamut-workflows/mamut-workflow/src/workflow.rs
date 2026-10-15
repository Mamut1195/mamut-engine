// SPDX-License-Identifier: Apache-2.0

//! Workflow definitions.
//!
//! Hydrology's `WorkflowDefinitionV1` is an `event.v1` document: control,
//! basin elements, and hyetographs. Those stay in the engine. The shared
//! definition keeps the part the compiler resolves: `profile`, `version`, and
//! one [`MethodSelectionV1`] per step.
//!
//! A [`ProfileContract`] supplies the expected capability slot, the canonical
//! dimensions, and the consumed and produced contracts for each step. The
//! dimensions play the role of `CapabilitySlot::canonical_dimensions` without
//! embedding Hydrology's enum. The contracts are the substitution rule.

use serde::{Deserialize, Serialize};

use crate::contract::ContractId;
use crate::descriptor::{MethodDimensions, MethodSelectionV1};

/// Canonical dimensions and contracts required of every method placed in one step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotContract {
    /// Slot name the selection must use.
    pub capability_slot: String,
    /// Inputs and outputs the descriptor must match exactly.
    pub dimensions: MethodDimensions,
    /// Contracts the step consumes, in the same order as [`MethodDimensions::inputs`].
    pub consumes: Vec<ContractId>,
    /// Contracts the step produces, in the same order as [`MethodDimensions::outputs`].
    pub produces: Vec<ContractId>,
}

/// Profile the compiler accepts.
///
/// Hydrology hardcodes `event.v1` at version 1. An engine passes that profile,
/// or its own, when it builds the compiler.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileContract {
    /// Profile name, compared to [`WorkflowDefinitionV1::profile`].
    pub profile: String,
    /// Profile version, compared to [`WorkflowDefinitionV1::version`].
    pub version: u32,
    /// Expected slots, in step order.
    pub slots: Vec<SlotContract>,
}

/// Versioned workflow definition: a profile plus ordered method selections.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowDefinitionV1 {
    /// Profile name.
    pub profile: String,
    /// Definition version.
    pub version: u32,
    /// One selection per [`ProfileContract`] slot, in the same order.
    pub steps: Vec<MethodSelectionV1>,
}
