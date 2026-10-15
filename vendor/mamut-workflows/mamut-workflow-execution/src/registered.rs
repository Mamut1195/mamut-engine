// SPDX-License-Identifier: Apache-2.0

//! Run methods loaded from a [`RegisteredCatalog`](mamut_workflow::RegisteredCatalog).
//!
//! A provider supplies engine instances. The default provider returns the step
//! identity, or the descriptor's `failure` message. One registered method is
//! compiled as a one-step workflow and then passed to [`crate::Executor`].

use std::fmt;

use mamut_workflow::{
    CompiledStep, CompiledWorkflow, MethodCatalog, MethodDescriptor, MethodSelectionV1,
    NumericParameters, ProfileContract, RegisteredCatalog, RegisteredMethod, SlotContract,
    WorkflowCompiler, WorkflowDefinitionV1,
};
use serde_json::{json, Value};

use crate::{
    Cancellation, ExecuteError, ExecutionManifest, Executor, IgnoreProgress, Method, NeverCancel,
    ProgressListener,
};

/// One catalog method could not be run. No manifest is returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MethodCallError {
    /// The arguments do not match the method. The executor was not called.
    InvalidInput(String),
    /// [`Method::run`] failed.
    MethodFailed(String),
    /// The manifest could not be sealed.
    Internal(String),
}

impl fmt::Display for MethodCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) | Self::MethodFailed(message) | Self::Internal(message) => {
                f.write_str(message)
            }
        }
    }
}

impl std::error::Error for MethodCallError {}

struct IdentityMethod {
    failure: Option<String>,
    output: Value,
}

impl Method for IdentityMethod {
    fn run(&mut self) -> Result<Value, String> {
        match self.failure.clone() {
            Some(message) => Err(message),
            None => Ok(self.output.clone()),
        }
    }
}

/// Builds an engine method from a validated descriptor and compiled selection.
///
/// The step includes its numeric parameters. Factory errors stop execution and
/// are reported as method failures. The provider is invoked lazily at each step,
/// after the executor checks cancellation and emits its started event.
pub trait MethodProvider {
    /// Create the real engine instance for this selection.
    fn create(
        &self,
        descriptor: &MethodDescriptor,
        step: &CompiledStep<RegisteredMethod>,
    ) -> Result<Box<dyn Method>, String>;
}

impl<T: MethodProvider + ?Sized> MethodProvider for &T {
    fn create(
        &self,
        descriptor: &MethodDescriptor,
        step: &CompiledStep<RegisteredMethod>,
    ) -> Result<Box<dyn Method>, String> {
        (**self).create(descriptor, step)
    }
}

/// Reference provider: return selection identity or the descriptor's `failure`.
#[derive(Clone, Copy, Debug, Default)]
pub struct IdentityProvider;

impl MethodProvider for IdentityProvider {
    fn create(
        &self,
        descriptor: &MethodDescriptor,
        step: &CompiledStep<RegisteredMethod>,
    ) -> Result<Box<dyn Method>, String> {
        Ok(Box::new(IdentityMethod {
            failure: descriptor.failure.clone(),
            output: json!({
                "capability_slot": step.capability_slot,
                "method_id": step.method_id,
                "method_version": step.method_version,
                "parameters": step.parameters,
            }),
        }))
    }
}

/// Execute a workflow with the reference identity provider.
pub fn run_registered(
    catalog: &RegisteredCatalog,
    compiled: &CompiledWorkflow<RegisteredMethod>,
) -> Result<ExecutionManifest, ExecuteError> {
    run_registered_with(
        catalog,
        compiled,
        &IdentityProvider,
        IgnoreProgress,
        NeverCancel,
    )
}

/// Execute a compiled workflow with an engine provider and caller observers.
///
/// Cancellation is checked before each method call. Failure or cancellation
/// returns no manifest. Factory errors use the same step metadata as run errors.
pub fn run_registered_with<P: ProgressListener, C: Cancellation>(
    catalog: &RegisteredCatalog,
    compiled: &CompiledWorkflow<RegisteredMethod>,
    provider: &dyn MethodProvider,
    progress: P,
    cancellation: C,
) -> Result<ExecutionManifest, ExecuteError> {
    let descriptors = catalog.descriptors();
    let mut steps = Vec::with_capacity(compiled.steps.len());
    for (index, step) in compiled.steps.iter().enumerate() {
        let failure = |message| ExecuteError::MethodFailed {
            index,
            method_id: step.method_id.clone(),
            method_version: step.method_version,
            message,
        };
        let descriptor = descriptors
            .iter()
            .find(|descriptor| {
                descriptor.method_id == step.method_id
                    && descriptor.method_version == step.method_version
                    && descriptor.capability_slot == step.capability_slot
            })
            .ok_or_else(|| failure("compiled method is absent from catalog".into()))?;
        let instance = ProvidedMethod {
            provider,
            descriptor,
            step,
        };
        steps.push(CompiledStep {
            capability_slot: step.capability_slot.clone(),
            method_id: step.method_id.clone(),
            method_version: step.method_version,
            parameters: step.parameters.clone(),
            instance,
        });
    }
    let mut runnable = CompiledWorkflow {
        profile: compiled.profile.clone(),
        version: compiled.version,
        steps,
    };
    Executor::with_observers(progress, cancellation).run(&mut runnable)
}

// Resolve factories only when a step actually runs: cancelled steps must not
// construct engine instances or let later factory failures preempt cancellation.
struct ProvidedMethod<'a> {
    provider: &'a dyn MethodProvider,
    descriptor: &'a MethodDescriptor,
    step: &'a CompiledStep<RegisteredMethod>,
}

impl Method for ProvidedMethod<'_> {
    fn run(&mut self) -> Result<Value, String> {
        self.provider.create(self.descriptor, self.step)?.run()
    }
}

/// Compile `descriptor` as one step and run it.
///
/// The profile has that single slot, using the method's own dimensions and
/// contracts. This is not a multi-slot workflow and it does not compare lines.
pub fn run_one_method(
    catalog: &RegisteredCatalog,
    descriptor: &MethodDescriptor,
    parameters: NumericParameters,
) -> Result<ExecutionManifest, MethodCallError> {
    run_one_method_with(catalog, descriptor, parameters, &IdentityProvider)
}

/// Compile and execute one method using a caller's engine provider.
pub fn run_one_method_with(
    catalog: &RegisteredCatalog,
    descriptor: &MethodDescriptor,
    parameters: NumericParameters,
    provider: &dyn MethodProvider,
) -> Result<ExecutionManifest, MethodCallError> {
    let compiled = compile_one_method(catalog, descriptor, parameters)?;
    run_registered_with(catalog, &compiled, provider, IgnoreProgress, NeverCancel).map_err(
        |error| match error {
            ExecuteError::MethodFailed { message, .. } => MethodCallError::MethodFailed(message),
            ExecuteError::Cancelled { index } => {
                MethodCallError::Internal(format!("execution cancelled before step {index}"))
            }
            ExecuteError::Fingerprint { message } => MethodCallError::Internal(message),
        },
    )
}

/// Compile a single catalog method with its own one-slot profile.
///
/// Hosts can pass the resulting plan to [`run_registered_with`] to apply their
/// provider, progress, and cancellation policy to a single method tool.
pub fn compile_one_method(
    catalog: &RegisteredCatalog,
    descriptor: &MethodDescriptor,
    parameters: NumericParameters,
) -> Result<CompiledWorkflow<RegisteredMethod>, MethodCallError> {
    let profile = one_slot_profile(catalog, descriptor);
    let definition = WorkflowDefinitionV1 {
        profile: profile.profile.clone(),
        version: profile.version,
        steps: vec![MethodSelectionV1 {
            capability_slot: descriptor.capability_slot.clone(),
            method_id: descriptor.method_id.clone(),
            method_version: descriptor.method_version,
            parameters,
        }],
    };
    WorkflowCompiler::new(catalog, profile)
        .compile(&definition)
        .map_err(|error| MethodCallError::InvalidInput(error.to_string()))
}

fn one_slot_profile(catalog: &RegisteredCatalog, descriptor: &MethodDescriptor) -> ProfileContract {
    ProfileContract {
        profile: catalog.profile().profile.clone(),
        version: catalog.profile().version,
        slots: vec![SlotContract {
            capability_slot: descriptor.capability_slot.clone(),
            dimensions: descriptor.dimensions.clone(),
            consumes: descriptor.consumes.clone(),
            produces: descriptor.produces.clone(),
        }],
    }
}
