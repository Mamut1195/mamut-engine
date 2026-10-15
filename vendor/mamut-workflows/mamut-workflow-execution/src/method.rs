// SPDX-License-Identifier: Apache-2.0

//! Method trait the executor calls.
//!
//! The compiled step already holds the instance the registry built. This trait
//! is that instance's run hook. It is not a hydrology method, and it does not
//! receive a basin graph.

use serde_json::Value;

/// One registered method the executor can call.
///
/// `run` returns the JSON result recorded on the manifest. The same instance
/// state must return the same JSON for the manifest hash to match.
pub trait Method {
    /// Run this step.
    ///
    /// The error string is the diagnostic. A failure stops the workflow before
    /// a manifest is built.
    fn run(&mut self) -> Result<Value, String>;
}

impl<T: Method + ?Sized> Method for Box<T> {
    fn run(&mut self) -> Result<Value, String> {
        (**self).run()
    }
}
