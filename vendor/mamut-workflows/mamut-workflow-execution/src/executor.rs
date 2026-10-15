// SPDX-License-Identifier: Apache-2.0

//! Sequential executor.
//!
//! Steps run in compiled order. There is no branch step and no loop step.
//! The executor checks cancellation before each call. A method error or a
//! cancellation returns immediately and does not build a manifest.

use mamut_workflow::CompiledWorkflow;

use crate::manifest::{record_step, seal};
use crate::method::Method;
use crate::observe::{Cancellation, ProgressEvent, ProgressListener, ProgressPhase};
use crate::ExecuteError;

/// Runs a [`CompiledWorkflow`] whose instances implement [`Method`].
pub struct Executor<P, C> {
    progress: P,
    cancellation: C,
}

impl<P, C> Executor<P, C> {
    /// Observer pair. `progress` receives step events. `cancellation` is read
    /// before each step.
    pub fn with_observers(progress: P, cancellation: C) -> Self {
        Self {
            progress,
            cancellation,
        }
    }

    /// Call each step in definition order.
    ///
    /// The returned manifest exists only when every step returned a result.
    pub fn run<T: Method>(
        &mut self,
        workflow: &mut CompiledWorkflow<T>,
    ) -> Result<crate::ExecutionManifest, ExecuteError>
    where
        P: ProgressListener,
        C: Cancellation,
    {
        let mut steps = Vec::with_capacity(workflow.steps.len());
        for (index, step) in workflow.steps.iter_mut().enumerate() {
            if self.cancellation.is_cancelled() {
                return Err(ExecuteError::Cancelled { index });
            }
            self.progress.on_progress(ProgressEvent {
                index,
                capability_slot: step.capability_slot.clone(),
                method_id: step.method_id.clone(),
                method_version: step.method_version,
                phase: ProgressPhase::Started,
            });
            let output = step
                .instance
                .run()
                .map_err(|message| ExecuteError::MethodFailed {
                    index,
                    method_id: step.method_id.clone(),
                    method_version: step.method_version,
                    message,
                })?;
            self.progress.on_progress(ProgressEvent {
                index,
                capability_slot: step.capability_slot.clone(),
                method_id: step.method_id.clone(),
                method_version: step.method_version,
                phase: ProgressPhase::Finished,
            });
            steps.push(record_step(index, step, &output)?);
        }
        seal(&workflow.profile, workflow.version, steps)
    }
}
