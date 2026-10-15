// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

/// Default cooperative cancellation code, distinct from 0, 2, 3, 4, 5 and 10.
pub const EXIT_CANCELLED: u8 = 6;

/// Semantic failure category, independent of an engine's numeric exit table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    /// Command or argument error.
    Usage,
    /// Unreadable or malformed input.
    Document,
    /// Workflow does not satisfy catalog contracts.
    Validation,
    /// Method returned an unclassified error, including factory errors.
    Method,
    /// Cooperative cancellation before a step.
    Cancelled,
    /// Unsupported operation (available to engine adapters).
    Unsupported,
    /// Numerical convergence failure (available to engine adapters).
    NotConverged,
    /// Reserved timeout category; this CLI does not start timers.
    Timeout,
    /// Internal execution or I/O failure.
    Internal,
}

/// Configurable numeric exit table. Semantics never depend on numeric equality.
///
/// Fields may share a code: Engine v1 uses 2 for all invalid inputs. The default
/// distinguishes usage (2), document (3), and validation (4) for compatibility.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExitCodes {
    /// Successful completion.
    pub ok: u8,
    /// Successful completion with warnings; reserved for engine adapters.
    pub warning: u8,
    /// Invalid arguments.
    pub usage: u8,
    /// Invalid document.
    pub document: u8,
    /// Invalid workflow.
    pub validation: u8,
    /// Unsupported operation.
    pub unsupported: u8,
    /// Did not converge.
    pub not_converged: u8,
    /// Unclassified method or factory error.
    pub method: u8,
    /// Cooperative cancellation.
    pub cancelled: u8,
    /// Reserved timeout.
    pub timeout: u8,
    /// Internal failure.
    pub internal: u8,
}

impl Default for ExitCodes {
    fn default() -> Self {
        Self {
            ok: 0,
            warning: 1,
            usage: 2,
            document: 3,
            validation: 4,
            unsupported: 5,
            not_converged: 5,
            method: 5,
            cancelled: EXIT_CANCELLED,
            timeout: 10,
            internal: 10,
        }
    }
}

impl ExitCodes {
    /// Engine v1: 0 ok, 1 warning, 2 invalid, 3 unsupported, 4 not converged,
    /// 5 cancelled, 6 timeout (reserved), 70 internal/unclassified method error.
    #[must_use]
    pub fn engine_v1() -> Self {
        Self {
            ok: 0,
            warning: 1,
            usage: 2,
            document: 2,
            validation: 2,
            unsupported: 3,
            not_converged: 4,
            method: 70,
            cancelled: 5,
            timeout: 6,
            internal: 70,
        }
    }

    /// Resolve a semantic category using this table.
    #[must_use]
    pub fn code(&self, kind: FailureKind) -> u8 {
        match kind {
            FailureKind::Usage => self.usage,
            FailureKind::Document => self.document,
            FailureKind::Validation => self.validation,
            FailureKind::Method => self.method,
            FailureKind::Cancelled => self.cancelled,
            FailureKind::Unsupported => self.unsupported,
            FailureKind::NotConverged => self.not_converged,
            FailureKind::Timeout => self.timeout,
            FailureKind::Internal => self.internal,
        }
    }

    /// Advertise numeric exit codes and emitted/reserved semantic tables as JSON.
    ///
    /// `exit_codes` retains the reference CLI's legacy numeric labels for default
    /// codes. Other tables use one label per number, with internal failures taking
    /// precedence over method failures and invalid workflows over other invalid
    /// inputs. `exit_code_categories` maps emitted categories to their codes,
    /// preserving all meanings when numbers overlap. For the default table these
    /// are ok, usage, document, validation, method, cancelled and internal. Warning,
    /// unsupported and not_converged are emitted only by adapters using nondefault
    /// tables. `reserved_exit_code_categories` lists categories not emitted by the
    /// selected profile. Timeout is always reserved; this API installs no timer.
    #[must_use]
    pub fn capabilities(&self) -> Value {
        let exit_codes = if *self == Self::default() {
            json!({"0":"success","2":"usage","3":"document","4":"validation","5":"method","10":"internal"})
        } else {
            let mut table = serde_json::Map::new();
            let ok_label = if *self == Self::engine_v1() {
                "ok"
            } else {
                "success"
            };
            for (code, label) in [
                (self.ok, ok_label),
                (self.warning, "warning"),
                (self.usage, "usage"),
                (self.document, "document"),
                (self.validation, "invalid"),
                (self.unsupported, "unsupported"),
                (self.not_converged, "not_converged"),
                (self.method, "method"),
                (self.cancelled, "cancelled"),
                (self.timeout, "timeout"),
                (self.internal, "internal"),
            ] {
                table.insert(code.to_string(), json!(label));
            }
            Value::Object(table)
        };
        let mut categories = json!({
            "ok": self.ok, "usage": self.usage, "document": self.document,
            "validation": self.validation, "method": self.method,
            "cancelled": self.cancelled, "internal": self.internal
        });
        let mut reserved = json!({"timeout": self.timeout});
        for (name, code) in [
            ("warning", self.warning),
            ("unsupported", self.unsupported),
            ("not_converged", self.not_converged),
        ] {
            if *self == Self::default() {
                reserved[name] = json!(code);
            } else {
                categories[name] = json!(code);
            }
        }
        json!({
            "exit_codes": exit_codes,
            "exit_code_categories": categories,
            "reserved_exit_code_categories": reserved
        })
    }
}
