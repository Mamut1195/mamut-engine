// SPDX-License-Identifier: Apache-2.0

//! Progress events and cancellation.
//!
//! Events carry the step identity and a phase. They have no timestamp.
//! Cancellation is a flag the caller sets; the executor reads it before each
//! step. Neither one is an input to the manifest hash.

use std::cell::Cell;
use std::rc::Rc;

/// Where a step is in its call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressPhase {
    /// `Method::run` is about to be called.
    Started,
    /// `Method::run` returned a result.
    Finished,
}

/// One progress notice for a compiled step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressEvent {
    /// Index in the compiled workflow, starting at 0.
    pub index: usize,
    /// Slot the step filled.
    pub capability_slot: String,
    /// Method id that will run, or that just ran.
    pub method_id: String,
    /// Method version.
    pub method_version: u32,
    /// Whether the call is starting or has finished.
    pub phase: ProgressPhase,
}

/// Receives [`ProgressEvent`]s in definition order.
pub trait ProgressListener {
    /// Observe one event.
    fn on_progress(&mut self, event: ProgressEvent);
}

/// Listener that discards events.
#[derive(Clone, Copy, Debug, Default)]
pub struct IgnoreProgress;

impl ProgressListener for IgnoreProgress {
    fn on_progress(&mut self, _event: ProgressEvent) {}
}

/// Answers whether the caller has asked the run to stop.
pub trait Cancellation {
    /// `true` when the executor must not start the next step.
    fn is_cancelled(&self) -> bool;
}

/// Cancellation token that is never set.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeverCancel;

impl Cancellation for NeverCancel {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Caller-owned flag shared with the executor.
///
/// [`CancelFlag::cancel`] can be called from a progress listener. The executor
/// sees it before the next step. There is no background thread.
#[derive(Clone, Debug, Default)]
pub struct CancelFlag {
    cancelled: Rc<Cell<bool>>,
}

impl CancelFlag {
    /// A flag that is not cancelled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ask the executor to stop before the next step.
    pub fn cancel(&self) {
        self.cancelled.set(true);
    }
}

impl Cancellation for CancelFlag {
    fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
}

impl<T: Cancellation + ?Sized> Cancellation for &T {
    fn is_cancelled(&self) -> bool {
        (**self).is_cancelled()
    }
}

impl<T: ProgressListener + ?Sized> ProgressListener for &mut T {
    fn on_progress(&mut self, event: ProgressEvent) {
        (**self).on_progress(event);
    }
}
