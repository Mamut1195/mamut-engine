// SPDX-License-Identifier: Apache-2.0

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

use mamut_canon::{fingerprint, fingerprint_value, ALGORITHM, CANONICALIZATION};
use mamut_workflow::{
    CompiledStep, CompiledWorkflow, ContractId, MethodDescriptor, MethodDimensions, MethodRegistry,
    MethodSelectionV1, NumericParameters, ParameterDescriptor, ProfileContract, QuantityDescriptor,
    SlotContract, WorkflowCompiler, WorkflowDefinitionV1,
};
use serde_json::Value;

use crate::{
    CancelFlag, ExecuteError, Executor, Method, ProgressEvent, ProgressListener, ProgressPhase,
};

const STAGE: &str = "urn:mamut:sequence:stage:1.0.0";
const INFLOW: &str = "urn:mamut:sequence:inflow:1.0.0";
const OUTFLOW: &str = "urn:mamut:sequence:outflow:1.0.0";

fn contract(urn: &str) -> ContractId {
    ContractId::parse(urn).unwrap()
}

fn depth() -> MethodDimensions {
    MethodDimensions {
        inputs: vec![QuantityDescriptor::new("in", "L", "m")],
        outputs: vec![QuantityDescriptor::new("out", "L", "m")],
    }
}

fn flow() -> MethodDimensions {
    MethodDimensions {
        inputs: vec![QuantityDescriptor::new("in", "L3/T", "m3/s")],
        outputs: vec![QuantityDescriptor::new("out", "L3/T", "m3/s")],
    }
}

fn descriptor(
    method_id: &str,
    slot: &str,
    dimensions: MethodDimensions,
    input: &str,
    output: &str,
    parameters: Vec<ParameterDescriptor>,
) -> MethodDescriptor {
    MethodDescriptor {
        method_id: method_id.to_owned(),
        method_version: 1,
        capability_slot: slot.to_owned(),
        dimensions,
        consumes: vec![contract(input)],
        produces: vec![contract(output)],
        parameters,
        failure: None,
    }
}

#[derive(Clone)]
struct Echo;

impl Method for Echo {
    fn run(&mut self) -> Result<Value, String> {
        Ok(serde_json::from_str(r#"{"b":1,"a":2}"#).unwrap())
    }
}

fn produce_echo(_: &NumericParameters) -> Result<Echo, String> {
    Ok(Echo)
}

fn registry() -> MethodRegistry<Echo> {
    let mut registry = MethodRegistry::new();
    registry
        .register(
            descriptor(
                "method-a",
                "first",
                depth(),
                STAGE,
                STAGE,
                vec![ParameterDescriptor::required("n", "1", "1")],
            ),
            produce_echo,
        )
        .unwrap();
    registry
        .register(
            descriptor("method-b", "second", flow(), INFLOW, OUTFLOW, Vec::new()),
            produce_echo,
        )
        .unwrap();
    registry
        .register(
            descriptor("method-c", "second", flow(), INFLOW, OUTFLOW, Vec::new()),
            produce_echo,
        )
        .unwrap();
    registry
}

fn profile() -> ProfileContract {
    ProfileContract {
        profile: "sequence.v1".to_owned(),
        version: 1,
        slots: vec![
            SlotContract {
                capability_slot: "first".to_owned(),
                dimensions: depth(),
                consumes: vec![contract(STAGE)],
                produces: vec![contract(STAGE)],
            },
            SlotContract {
                capability_slot: "second".to_owned(),
                dimensions: flow(),
                consumes: vec![contract(INFLOW)],
                produces: vec![contract(OUTFLOW)],
            },
        ],
    }
}

fn definition(second: &str, n: f64) -> WorkflowDefinitionV1 {
    let mut parameters = BTreeMap::new();
    parameters.insert("n".to_owned(), n);
    WorkflowDefinitionV1 {
        profile: "sequence.v1".to_owned(),
        version: 1,
        steps: vec![
            MethodSelectionV1 {
                capability_slot: "first".to_owned(),
                method_id: "method-a".to_owned(),
                method_version: 1,
                parameters,
            },
            MethodSelectionV1 {
                capability_slot: "second".to_owned(),
                method_id: second.to_owned(),
                method_version: 1,
                parameters: BTreeMap::new(),
            },
        ],
    }
}

fn run_compiled(second: &str, n: f64) -> crate::ExecutionManifest {
    let registry = registry();
    let mut compiled = WorkflowCompiler::new(&registry, profile())
        .compile(&definition(second, n))
        .unwrap();
    Executor::new().run(&mut compiled).unwrap()
}

#[test]
fn the_same_compiled_workflow_hashes_the_same_manifest() {
    let registry = registry();
    let compiler = WorkflowCompiler::new(&registry, profile());
    let compiled = compiler.compile(&definition("method-b", 2.0)).unwrap();
    let mut first = compiled.clone();
    let mut second = compiled.clone();

    let left = Executor::new().run(&mut first).unwrap();
    let right = Executor::new().run(&mut second).unwrap();
    assert_eq!(left, right);
    assert_eq!(
        left.record_fingerprint.as_ref().unwrap().algorithm,
        ALGORITHM
    );
    assert_eq!(
        left.record_fingerprint.as_ref().unwrap().canonicalization,
        CANONICALIZATION
    );
    assert_eq!(left.record_fingerprint.as_ref().unwrap().value.len(), 64);
    assert_eq!(left.status, crate::ExecutionStatus::Completed);
    assert_eq!(left.steps[0].method.id, "method-a");
    assert_eq!(left.steps[1].method.id, "method-b");

    let reordered = fingerprint(r#"{"a":2,"b":1}"#).unwrap();
    let ordered = fingerprint(r#"{"b":1,"a":2}"#).unwrap();
    assert_eq!(reordered.value, ordered.value);
    assert_eq!(left.steps[0].outputs[0].fingerprint.value, reordered.value);
    assert_eq!(left.steps[1].outputs[0].fingerprint.value, ordered.value);

    let mut document = serde_json::to_value(&left).unwrap();
    document
        .as_object_mut()
        .unwrap()
        .remove("recordFingerprint");
    assert_eq!(
        left.record_fingerprint.as_ref().unwrap().value,
        fingerprint_value(&document).unwrap().value
    );

    let other_input = run_compiled("method-b", 3.0);
    assert_ne!(
        left.record_fingerprint.as_ref().unwrap().value,
        other_input.record_fingerprint.as_ref().unwrap().value
    );

    let substituted = run_compiled("method-c", 2.0);
    assert_eq!(substituted.steps[1].method.id, "method-c");
    assert_ne!(
        left.record_fingerprint.as_ref().unwrap().value,
        substituted.record_fingerprint.as_ref().unwrap().value
    );
}

struct Recording {
    events: Rc<RefCell<Vec<ProgressEvent>>>,
}

impl ProgressListener for Recording {
    fn on_progress(&mut self, event: ProgressEvent) {
        self.events.borrow_mut().push(event);
    }
}

#[test]
fn progress_events_do_not_change_the_manifest_hash() {
    let registry = registry();
    let compiled = WorkflowCompiler::new(&registry, profile())
        .compile(&definition("method-b", 2.0))
        .unwrap();
    let mut quiet_workflow = compiled.clone();
    let mut watched_workflow = compiled;
    let quiet = Executor::new().run(&mut quiet_workflow).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let watched = Executor::with_observers(
        Recording {
            events: events.clone(),
        },
        crate::NeverCancel,
    )
    .run(&mut watched_workflow)
    .unwrap();
    assert_eq!(
        quiet.record_fingerprint.as_ref().unwrap().value,
        watched.record_fingerprint.as_ref().unwrap().value
    );
    let phases: Vec<ProgressPhase> = events.borrow().iter().map(|event| event.phase).collect();
    assert_eq!(
        phases,
        vec![
            ProgressPhase::Started,
            ProgressPhase::Finished,
            ProgressPhase::Started,
            ProgressPhase::Finished,
        ]
    );
}

struct Counting {
    label: &'static str,
    fail: bool,
    calls: Rc<Cell<u32>>,
}

impl Method for Counting {
    fn run(&mut self) -> Result<Value, String> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            Err(format!("{} failed", self.label))
        } else {
            Ok(serde_json::json!({"ok": true}))
        }
    }
}

fn counting_step(method_id: &str, method: Counting) -> CompiledStep<Counting> {
    CompiledStep {
        capability_slot: "slot".to_owned(),
        method_id: method_id.to_owned(),
        method_version: 1,
        parameters: BTreeMap::new(),
        instance: method,
    }
}

fn counting_workflow(methods: Vec<Counting>, ids: &[&str]) -> CompiledWorkflow<Counting> {
    CompiledWorkflow {
        profile: "sequence.v1".to_owned(),
        version: 1,
        steps: ids
            .iter()
            .zip(methods)
            .map(|(method_id, method)| counting_step(method_id, method))
            .collect(),
    }
}

#[test]
fn a_method_failure_stops_the_run_without_a_manifest() {
    let calls = Rc::new(Cell::new(0));
    let mut workflow = counting_workflow(
        vec![
            Counting {
                label: "first",
                fail: false,
                calls: calls.clone(),
            },
            Counting {
                label: "second",
                fail: true,
                calls: calls.clone(),
            },
            Counting {
                label: "third",
                fail: false,
                calls: calls.clone(),
            },
        ],
        &["method-a", "method-b", "method-c"],
    );
    let error = Executor::new().run(&mut workflow).unwrap_err();
    assert!(matches!(error, ExecuteError::MethodFailed { .. }));
    assert_eq!(
        error.to_string(),
        "method `method-b` version 1 failed at step 1: second failed"
    );
    assert_eq!(calls.get(), 2);
}

#[test]
fn cancellation_before_a_step_returns_no_manifest() {
    let calls = Rc::new(Cell::new(0));
    let flag = CancelFlag::new();
    let listener = CancelAfterFirst { flag: flag.clone() };
    let mut workflow = counting_workflow(
        vec![
            Counting {
                label: "first",
                fail: false,
                calls: calls.clone(),
            },
            Counting {
                label: "second",
                fail: false,
                calls: calls.clone(),
            },
        ],
        &["method-a", "method-b"],
    );
    let error = Executor::with_observers(listener, flag)
        .run(&mut workflow)
        .unwrap_err();
    assert_eq!(error, ExecuteError::Cancelled { index: 1 });
    assert_eq!(calls.get(), 1);
}

struct CancelAfterFirst {
    flag: CancelFlag,
}

impl ProgressListener for CancelAfterFirst {
    fn on_progress(&mut self, event: ProgressEvent) {
        if event.phase == ProgressPhase::Finished && event.index == 0 {
            self.flag.cancel();
        }
    }
}

#[test]
fn an_empty_method_reference_cannot_produce_a_manifest() {
    let registry = registry();
    let mut compiled = WorkflowCompiler::new(&registry, profile())
        .compile(&definition("method-b", 2.0))
        .unwrap();
    compiled.steps[0].method_id.clear();
    let error = Executor::new().run(&mut compiled).unwrap_err();
    assert!(matches!(error, ExecuteError::Fingerprint { .. }));
}
