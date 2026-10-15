// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use mamut_workflow::{
    CompileError, ContractId, MethodCatalog, MethodDescriptor, MethodDimensions, MethodProduct,
    MethodRegistry, MethodSelectionV1, NumericParameters, ParameterDescriptor, ProfileContract,
    QuantityDescriptor, SlotContract, WorkflowCompiler, WorkflowDefinitionV1,
};

const STAGE: &str = "urn:mamut:sequence:stage:1.0.0";
const INFLOW: &str = "urn:mamut:sequence:inflow:1.0.0";
const OUTFLOW: &str = "urn:mamut:sequence:outflow:1.0.0";
const INFLOW_MINOR: &str = "urn:mamut:sequence:inflow:1.2.0";
const OUTFLOW_MINOR: &str = "urn:mamut:sequence:outflow:1.4.1";
const VOLUME: &str = "urn:mamut:sequence:volume:1.0.0";
const INFLOW_NEXT_MAJOR: &str = "urn:mamut:sequence:inflow:2.0.0";

fn contract(urn: &str) -> ContractId {
    ContractId::parse(urn).unwrap()
}

fn contracts(urns: &[&str]) -> Vec<ContractId> {
    urns.iter().copied().map(contract).collect()
}

fn quantity(name: &str, dimension: &str, unit: &str) -> QuantityDescriptor {
    QuantityDescriptor::new(name, dimension, unit)
}

fn flow_dimensions() -> MethodDimensions {
    MethodDimensions {
        inputs: vec![quantity("in", "L3/T", "m3/s")],
        outputs: vec![quantity("out", "L3/T", "m3/s")],
    }
}

fn depth_dimensions() -> MethodDimensions {
    MethodDimensions {
        inputs: vec![quantity("in", "L", "m")],
        outputs: vec![quantity("out", "L", "m")],
    }
}

struct Boundary {
    dimensions: MethodDimensions,
    consumes: Vec<ContractId>,
    produces: Vec<ContractId>,
}

fn boundary(dimensions: MethodDimensions, input: &str, output: &str) -> Boundary {
    Boundary {
        dimensions,
        consumes: contracts(&[input]),
        produces: contracts(&[output]),
    }
}

fn descriptor(
    method_id: &str,
    version: u32,
    slot: &str,
    boundary: Boundary,
    parameters: Vec<ParameterDescriptor>,
) -> MethodDescriptor {
    MethodDescriptor {
        method_id: method_id.to_owned(),
        method_version: version,
        capability_slot: slot.to_owned(),
        dimensions: boundary.dimensions,
        consumes: boundary.consumes,
        produces: boundary.produces,
        parameters,
        failure: None,
    }
}

fn produce_a(_: &NumericParameters) -> Result<&'static str, String> {
    Ok("method-a")
}

fn produce_b(_: &NumericParameters) -> Result<&'static str, String> {
    Ok("method-b")
}

fn produce_c(_: &NumericParameters) -> Result<&'static str, String> {
    Ok("method-c")
}

fn produce_d(_: &NumericParameters) -> Result<&'static str, String> {
    Ok("method-d")
}

fn produce_e(_: &NumericParameters) -> Result<&'static str, String> {
    Ok("method-e")
}

fn produce_f(_: &NumericParameters) -> Result<&'static str, String> {
    Ok("method-f")
}

fn catalog() -> MethodRegistry<&'static str> {
    let mut registry = MethodRegistry::new();
    registry
        .register(
            descriptor(
                "method-a",
                1,
                "first",
                boundary(depth_dimensions(), STAGE, STAGE),
                vec![ParameterDescriptor::required("n", "1", "1")],
            ),
            produce_a,
        )
        .unwrap();
    registry
        .register(
            descriptor(
                "method-b",
                1,
                "second",
                boundary(flow_dimensions(), INFLOW, OUTFLOW),
                Vec::new(),
            ),
            produce_b,
        )
        .unwrap();
    registry
        .register(
            descriptor(
                "method-b",
                3,
                "second",
                boundary(flow_dimensions(), INFLOW, OUTFLOW),
                Vec::new(),
            ),
            produce_b,
        )
        .unwrap();
    registry
        .register(
            descriptor(
                "method-c",
                1,
                "second",
                boundary(flow_dimensions(), INFLOW_MINOR, OUTFLOW_MINOR),
                Vec::new(),
            ),
            produce_c,
        )
        .unwrap();
    registry
        .register(
            descriptor(
                "method-d",
                1,
                "second",
                boundary(depth_dimensions(), STAGE, STAGE),
                Vec::new(),
            ),
            produce_d,
        )
        .unwrap();
    registry
        .register(
            descriptor(
                "method-e",
                1,
                "second",
                boundary(flow_dimensions(), VOLUME, OUTFLOW),
                Vec::new(),
            ),
            produce_e,
        )
        .unwrap();
    registry
        .register(
            descriptor(
                "method-f",
                1,
                "second",
                boundary(flow_dimensions(), INFLOW_NEXT_MAJOR, OUTFLOW),
                Vec::new(),
            ),
            produce_f,
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
                dimensions: depth_dimensions(),
                consumes: contracts(&[STAGE]),
                produces: contracts(&[STAGE]),
            },
            SlotContract {
                capability_slot: "second".to_owned(),
                dimensions: flow_dimensions(),
                consumes: contracts(&[INFLOW]),
                produces: contracts(&[OUTFLOW]),
            },
        ],
    }
}

fn selection(
    slot: &str,
    method_id: &str,
    version: u32,
    parameters: NumericParameters,
) -> MethodSelectionV1 {
    MethodSelectionV1 {
        capability_slot: slot.to_owned(),
        method_id: method_id.to_owned(),
        method_version: version,
        parameters,
    }
}

fn two_step(second_method: &str, second_version: u32) -> WorkflowDefinitionV1 {
    let mut parameters = BTreeMap::new();
    parameters.insert("n".to_owned(), 2.0);
    WorkflowDefinitionV1 {
        profile: "sequence.v1".to_owned(),
        version: 1,
        steps: vec![
            selection("first", "method-a", 1, parameters),
            selection("second", second_method, second_version, BTreeMap::new()),
        ],
    }
}

#[test]
fn compiles_two_steps_and_swaps_the_second_method() {
    let registry = catalog();
    let compiler = WorkflowCompiler::new(&registry, profile());

    let compiled = compiler.compile(&two_step("method-b", 1)).unwrap();
    let again = compiler.compile(&two_step("method-b", 1)).unwrap();
    assert_eq!(compiled, again);
    assert_eq!(compiled.steps.len(), 2);
    assert_eq!(compiled.steps[0].method_id, "method-a");
    assert_eq!(compiled.steps[0].instance, "method-a");
    assert_eq!(compiled.steps[1].method_id, "method-b");
    assert_eq!(compiled.steps[1].instance, "method-b");

    let swapped = compiler.compile(&two_step("method-c", 1)).unwrap();
    assert_eq!(swapped.steps[0].method_id, "method-a");
    assert_eq!(swapped.steps[1].method_id, "method-c");
    assert_eq!(swapped.steps[1].instance, "method-c");
    assert_ne!(swapped, compiled);
}

#[test]
fn compiler_rejects_unknown_method_unsupported_version_and_bad_dimensions() {
    let registry = catalog();
    let compiler = WorkflowCompiler::new(&registry, profile());

    let unknown = compiler.compile(&two_step("missing", 1)).unwrap_err();
    assert!(matches!(unknown, CompileError::UnknownMethod { .. }));

    let version = compiler.compile(&two_step("method-b", 2)).unwrap_err();
    match version {
        CompileError::UnsupportedMethodVersion { supported, .. } => {
            assert_eq!(supported, "1, 3");
        }
        other => panic!("expected unsupported version, got {other}"),
    }

    let dimensions = compiler.compile(&two_step("method-d", 1)).unwrap_err();
    assert!(dimensions
        .to_string()
        .contains("dimensional contract incompatible"));
}

#[test]
fn legal_contract_swap_compiles_and_illegal_swap_produces_no_plan() {
    let registry = catalog();
    let compiler = WorkflowCompiler::new(&registry, profile());

    let descriptors = registry.descriptors();
    let method_b = descriptors
        .iter()
        .find(|descriptor| descriptor.method_id == "method-b" && descriptor.method_version == 1)
        .unwrap();
    let method_c = descriptors
        .iter()
        .find(|descriptor| descriptor.method_id == "method-c")
        .unwrap();
    assert_eq!(method_b.dimensions, method_c.dimensions);
    assert_ne!(method_b.consumes, method_c.consumes);
    assert!(method_c.consumes[0].major_compatible(&method_b.consumes[0]));
    assert!(method_c.produces[0].major_compatible(&method_b.produces[0]));

    let original = compiler.compile(&two_step("method-b", 1)).unwrap();
    let swapped = compiler.compile(&two_step("method-c", 1)).unwrap();
    assert_eq!(original.steps[1].method_id, "method-b");
    assert_eq!(swapped.steps[1].method_id, "method-c");
    assert_eq!(swapped.steps[1].instance, "method-c");

    let other_contract = compiler.compile(&two_step("method-e", 1));
    assert!(other_contract.is_err());
    let other_contract = other_contract.unwrap_err();
    assert_eq!(
        other_contract.to_string(),
        "method `method-e` version 1 on slot `second` input `in` declares contract `urn:mamut:sequence:volume:1.0.0`; slot requires `urn:mamut:sequence:inflow:1.0.0` at the same major version"
    );

    let next_major = compiler.compile(&two_step("method-f", 1));
    assert!(next_major.is_err());
    assert_eq!(
        next_major.unwrap_err().to_string(),
        "method `method-f` version 1 on slot `second` input `in` declares contract `urn:mamut:sequence:inflow:2.0.0`; slot requires `urn:mamut:sequence:inflow:1.0.0` at the same major version"
    );
}

#[test]
fn compiler_rejects_slot_mismatch_parameter_shape_and_profile() {
    let registry = catalog();
    let compiler = WorkflowCompiler::new(&registry, profile());
    let mut definition = two_step("method-b", 1);

    definition.steps[1].capability_slot = "first".to_owned();
    assert!(matches!(
        compiler.compile(&definition),
        Err(CompileError::CapabilitySlotMismatch { .. })
    ));

    definition = two_step("method-a", 1);
    assert!(matches!(
        compiler.compile(&definition),
        Err(CompileError::CapabilitySlotMismatch { .. })
    ));

    definition = two_step("method-b", 1);
    definition.steps[0].parameters.clear();
    assert!(compiler
        .compile(&definition)
        .unwrap_err()
        .to_string()
        .contains("required parameter"));

    definition = two_step("method-b", 1);
    definition.steps[0]
        .parameters
        .insert("extra".to_owned(), 1.0);
    assert!(compiler
        .compile(&definition)
        .unwrap_err()
        .to_string()
        .contains("unexpected parameter"));

    definition = two_step("method-b", 1);
    definition.profile = "event.v1".to_owned();
    assert!(matches!(
        compiler.compile(&definition),
        Err(CompileError::UnsupportedProfile { .. })
    ));

    definition = two_step("method-b", 1);
    definition.version = 2;
    assert!(matches!(
        compiler.compile(&definition),
        Err(CompileError::UnsupportedVersion { .. })
    ));
}

struct WrongSlotCatalog {
    descriptors: Vec<MethodDescriptor>,
}

impl MethodCatalog for WrongSlotCatalog {
    type Instance = ();

    fn descriptors(&self) -> Vec<&MethodDescriptor> {
        self.descriptors.iter().collect()
    }

    fn create(
        &self,
        _descriptor: &MethodDescriptor,
        _parameters: &NumericParameters,
    ) -> Result<MethodProduct<()>, CompileError> {
        Ok(MethodProduct {
            capability_slot: "other".to_owned(),
            instance: (),
        })
    }
}

#[test]
fn compiler_rejects_a_factory_that_reports_another_slot() {
    let registry = catalog();
    let owned: Vec<MethodDescriptor> = registry.descriptors().into_iter().cloned().collect();
    let catalog = WrongSlotCatalog { descriptors: owned };
    let error = WorkflowCompiler::new(&catalog, profile())
        .compile(&two_step("method-b", 1))
        .unwrap_err();
    assert!(matches!(error, CompileError::FactorySlotMismatch { .. }));
}
