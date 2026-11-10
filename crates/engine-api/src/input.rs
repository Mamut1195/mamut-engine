//! Private DTOs: no serde dependency crosses engine-model.
use crate::ApiError;
use engine_core::units::catalog::CatalogUnit;
use engine_core::{Dim, Dof, MemberId, NodeId};
use engine_model as domain;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};
use serde_json::Value;

// Serde accepts positional structs as arrays; the contract requires JSON objects.
pub(crate) struct Object<T>(pub T);

impl<'de, T: DeserializeOwned> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        if !value.is_object() {
            return Err(serde::de::Error::custom("A JSON object is required"));
        }
        serde_json::from_value(value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

// Tags are strings only; the external form {"D2": null} is rejected.
macro_rules! string_enum {
    ($name:ident { $($variant:ident => $label:literal),+ $(,)? }) => {
        #[derive(Clone, Copy)]
        pub(crate) enum $name { $($variant),+ }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                match String::deserialize(d)?.as_str() {
                    $($label => Ok(Self::$variant),)+
                    _ => Err(serde::de::Error::custom(concat!("Invalid tag: ", stringify!($name)))),
                }
            }
        }
    };
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    #[serde(rename = "schema")]
    _schema: String,
    #[serde(rename = "units")]
    _units: Units,
    model: Object<Model>,
    cases: Vec<Object<Case>>,
    combinations: Vec<Object<Combination>>,
    analyses: Vec<Object<Analysis>>,
    options: Object<Options>,
}

string_enum!(Units { SI => "SI", US => "US" });
string_enum!(Axes { ZUp => "Z-up" });
string_enum!(Dimension { D2 => "D2", D3 => "D3" });
string_enum!(Component { Ux => "Ux", Uy => "Uy", Uz => "Uz", Rx => "Rx", Ry => "Ry", Rz => "Rz" });

impl Component {
    pub(crate) fn dof(self) -> Dof {
        match self {
            Self::Ux => Dof::Ux,
            Self::Uy => Dof::Uy,
            Self::Uz => Dof::Uz,
            Self::Rx => Dof::Rx,
            Self::Ry => Dof::Ry,
            Self::Rz => Dof::Rz,
        }
    }
}

use crate::analysis_input::Analysis;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {
    #[serde(default)]
    allow_experimental: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Model {
    #[serde(rename = "axes")]
    _axes: Axes,
    nodes: Vec<Object<Node>>,
    members: Vec<Object<Member>>,
    sections: Vec<Object<Section>>,
    laws: Vec<Object<Law>>,
    supports: Vec<Object<Support>>,
    masses: Vec<Object<Mass>>,
    #[serde(default)]
    prescribed: Vec<Object<Prescribed>>,
    #[serde(default)]
    diaphragms: Vec<Object<Diaphragm>>,
    #[serde(default)]
    springs: Vec<Object<crate::extensions::Spring>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Node {
    id: u32,
    dim: Dimension,
    coordinates: [f64; 3],
    dofs: Vec<Component>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    id: u32,
    nodes: [u32; 2],
    section: String,
    law: String,
    #[serde(default)]
    element: String,
    #[serde(default, deserialize_with = "orientation")]
    orientation: Option<[f64; 3]>,
    #[serde(default, deserialize_with = "crate::extensions::optional")]
    release_i: Option<crate::extensions::Release>,
    #[serde(default, deserialize_with = "crate::extensions::optional")]
    release_j: Option<crate::extensions::Release>,
    #[serde(default, deserialize_with = "crate::extensions::optional")]
    winkler: Option<Object<crate::extensions::Winkler>>,
}

// Absence is allowed; explicit null is outside the vector schema.
fn orientation<'de, D: Deserializer<'de>>(d: D) -> Result<Option<[f64; 3]>, D::Error> {
    <[f64; 3]>::deserialize(d).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    name: String,
    value: f64,
}

impl Parameter {
    fn domain(self) -> domain::Parameter {
        domain::Parameter {
            name: self.name,
            value: self.value,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Section {
    id: String,
    properties: Vec<Object<Parameter>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Law {
    name: String,
    parameters: Vec<Object<Parameter>>,
    #[serde(default, deserialize_with = "crate::extensions::nonempty")]
    kind: String,
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default, deserialize_with = "crate::extensions::nonempty")]
    mode: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Support {
    id: String,
    node: u32,
    restrained: Vec<Component>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DofValue {
    dof: Component,
    value: f64,
}

impl DofValue {
    fn domain(self) -> domain::DofValue {
        domain::DofValue {
            dof: self.dof.dof(),
            value: self.value,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Load {
    node: u32,
    values: Vec<Object<DofValue>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    loads: Vec<Object<Load>>,
    #[serde(default)]
    member_loads: Vec<Object<UniformLoad>>,
    #[serde(default)]
    temperature_loads: Vec<Object<UniformTemperature>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Term {
    case: String,
    factor: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Combination {
    id: String,
    terms: Vec<Object<Term>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mass {
    id: String,
    node: u32,
    values: Vec<Object<DofValue>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniformLoad {
    member: u32,
    force_per_length: [f64; 3],
    moment_per_length: [f64; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniformTemperature {
    member: u32,
    alpha_per_kelvin: f64,
    delta_t_kelvin: f64,
}

/// One uniform temperature change on a member, without a gradient.
pub(crate) struct TemperatureLoad {
    pub member: MemberId,
    pub alpha_per_kelvin: f64,
    pub delta_t_kelvin: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Prescribed {
    node: u32,
    dof: Component,
    value: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Diaphragm {
    master: u32,
    slaves: Vec<u32>,
}

pub(crate) struct Execution {
    pub model: domain::Model,
    pub allow_experimental: bool,
    pub methods: Vec<&'static str>,
    pub analyses: Vec<Analysis>,
    pub elements: Vec<crate::extensions::Beam>,
    pub springs: Vec<crate::extensions::Spring>,
    pub laws: Vec<crate::extensions::Law>,
    pub loads: Vec<Vec<(engine_core::MemberId, engine_contracts::MemberLoad)>>,
    pub temperature: Vec<Vec<TemperatureLoad>>,
    pub prescribed: Vec<engine_assembly::PrescribedDof>,
    pub diaphragms: Vec<engine_assembly::RigidDiaphragm>,
}

pub(crate) fn supported_input_schema(schema: &str) -> bool {
    matches!(schema, "engine-input/1" | "engine-input/2")
}

pub(crate) fn decode(value: Value) -> Result<Execution, ApiError> {
    let schema = value.get("schema").and_then(Value::as_str).unwrap_or("");
    let united_states = match schema {
        "engine-input/1" => {
            if let Some(label) = value.get("units").and_then(Value::as_str) {
                if label != "SI" {
                    return Err(ApiError::UnknownUnit(
                        "Unknown unit system; only \"SI\" is accepted".into(),
                    ));
                }
            }
            false
        }
        "engine-input/2" => match value.get("units") {
            Some(Value::String(label)) if label == "SI" => false,
            Some(Value::String(label)) if label == "US" => true,
            Some(Value::String(label)) => {
                return Err(ApiError::UnknownUnit(format!(
                    "The specified unit is not recognized: «{label}»."
                )));
            }
            None => {
                return Err(ApiError::InvalidInput(
                    "Missing required field units.".into(),
                ));
            }
            Some(_) => {
                return Err(ApiError::InvalidInput("Field units must be text.".into()));
            }
        },
        _ => {
            return Err(ApiError::UnsupportedSchema(schema.into()));
        }
    };
    let Object(mut input): Object<Input> =
        serde_json::from_value(value).map_err(|e| ApiError::InvalidInput(e.to_string()))?;
    if united_states {
        convert_united_states(&mut input)?;
    }
    let Object(source) = input.model;
    let analyses: Vec<_> = input.analyses.into_iter().map(|Object(a)| a).collect();
    let methods = analyses.iter().map(Analysis::method).collect();
    let elements = source
        .members
        .iter()
        .map(|Object(m)| crate::extensions::Beam {
            kind: m.element.clone(),
            orientation: m.orientation,
            release_i: m.release_i.clone(),
            release_j: m.release_j.clone(),
            winkler: m.winkler.as_ref().map(|Object(w)| w.clone()),
        })
        .collect();
    let loads: Vec<Vec<_>> = input
        .cases
        .iter()
        .map(|Object(c)| {
            c.member_loads
                .iter()
                .map(|Object(l)| {
                    (
                        engine_core::MemberId(l.member),
                        engine_contracts::MemberLoad {
                            force_per_length: l.force_per_length,
                            moment_per_length: l.moment_per_length,
                        },
                    )
                })
                .collect()
        })
        .collect();
    let temperature: Vec<Vec<_>> = input
        .cases
        .iter()
        .map(|Object(c)| {
            c.temperature_loads
                .iter()
                .map(|Object(t)| TemperatureLoad {
                    member: MemberId(t.member),
                    alpha_per_kelvin: t.alpha_per_kelvin,
                    delta_t_kelvin: t.delta_t_kelvin,
                })
                .collect()
        })
        .collect();
    let prescribed = source
        .prescribed
        .into_iter()
        .map(|Object(p)| engine_assembly::PrescribedDof {
            node: NodeId(p.node),
            dof: p.dof.dof(),
            value: p.value,
        })
        .collect();
    let diaphragms = source
        .diaphragms
        .into_iter()
        .map(|Object(d)| engine_assembly::RigidDiaphragm {
            master: NodeId(d.master),
            slaves: d.slaves.into_iter().map(NodeId).collect(),
        })
        .collect();
    let laws = source
        .laws
        .iter()
        .map(|Object(l)| crate::extensions::Law {
            name: l.name.clone(),
            kind: l.kind.clone(),
            parameters: l
                .parameters
                .iter()
                .map(|Object(p)| (p.name.clone(), p.value))
                .collect(),
            points: l.points.clone(),
            mode: l.mode.clone(),
        })
        .collect();
    let springs = source.springs.into_iter().map(|Object(s)| s).collect();
    let mut model = domain::Model::new();
    for Object(n) in source.nodes {
        let dim = match n.dim {
            Dimension::D2 => Dim::D2,
            Dimension::D3 => Dim::D3,
        };
        if dim == Dim::D2 && n.coordinates[2] != 0.0 {
            return Err(ApiError::InvalidInput(format!(
                "Node {}: D2 requires Z = 0",
                n.id
            )));
        }
        model.insert_node(domain::Node {
            id: NodeId(n.id),
            dim,
            coordinates: n.coordinates,
            dofs: n.dofs.into_iter().map(Component::dof).collect(),
        });
    }
    model.members = source
        .members
        .into_iter()
        .map(|Object(m)| domain::Member {
            id: MemberId(m.id),
            nodes: m.nodes.map(NodeId),
            section: m.section,
            law: m.law,
        })
        .collect();
    model.sections = source
        .sections
        .into_iter()
        .map(|Object(s)| domain::Section {
            id: s.id,
            properties: s
                .properties
                .into_iter()
                .map(|Object(p)| p.domain())
                .collect(),
        })
        .collect();
    model.laws = source
        .laws
        .into_iter()
        .map(|Object(l)| domain::Law {
            name: l.name,
            parameters: l
                .parameters
                .into_iter()
                .map(|Object(p)| p.domain())
                .collect(),
        })
        .collect();
    model.supports = source
        .supports
        .into_iter()
        .map(|Object(s)| domain::Support {
            id: s.id,
            node: NodeId(s.node),
            restrained: s.restrained.into_iter().map(Component::dof).collect(),
        })
        .collect();
    model.masses = source
        .masses
        .into_iter()
        .map(|Object(m)| domain::NodalMass {
            id: m.id,
            node: NodeId(m.node),
            values: m.values.into_iter().map(|Object(v)| v.domain()).collect(),
        })
        .collect();
    model.cases = input
        .cases
        .into_iter()
        .map(|Object(c)| domain::LoadCase {
            name: c.name,
            loads: c
                .loads
                .into_iter()
                .map(|Object(l)| domain::NodalLoad {
                    node: NodeId(l.node),
                    values: l.values.into_iter().map(|Object(v)| v.domain()).collect(),
                })
                .collect(),
        })
        .collect();
    model.combinations = input
        .combinations
        .into_iter()
        .map(|Object(c)| domain::Combination {
            id: c.id,
            terms: c
                .terms
                .into_iter()
                .map(|Object(t)| domain::CaseFactor {
                    case: t.case,
                    factor: t.factor,
                })
                .collect(),
        })
        .collect();
    for case_loads in &loads {
        for (id, _) in case_loads {
            if !model.members.iter().any(|m| m.id == *id) {
                return Err(ApiError::InvalidInput(format!(
                    "Load: missing member {}",
                    id.0
                )));
            }
        }
    }
    for case_loads in &temperature {
        for load in case_loads {
            if !model.members.iter().any(|m| m.id == load.member) {
                return Err(ApiError::InvalidInput(format!(
                    "Load: missing member {}",
                    load.member.0
                )));
            }
        }
    }
    Ok(Execution {
        allow_experimental: input.options.0.allow_experimental,
        model,
        methods,
        analyses,
        elements,
        springs,
        laws,
        loads,
        temperature,
        prescribed,
        diaphragms,
    })
}

fn translation(dof: Component) -> bool {
    matches!(dof, Component::Ux | Component::Uy | Component::Uz)
}

fn scale(value: f64, factor: f64) -> Result<f64, ApiError> {
    if !value.is_finite() {
        return Err(ApiError::NonFiniteNumber(value.to_string()));
    }
    let scaled = value * factor;
    if !scaled.is_finite() {
        return Err(ApiError::NonFiniteNumber(scaled.to_string()));
    }
    Ok(scaled)
}

fn scale_slot(slot: &mut Option<f64>, factor: f64) -> Result<(), ApiError> {
    if let Some(value) = slot.as_mut() {
        *value = scale(*value, factor)?;
    }
    Ok(())
}

fn section_factor(
    name: &str,
    ksi: f64,
    area: f64,
    inertia: f64,
    density: f64,
) -> Result<f64, ApiError> {
    Ok(match name {
        "E" | "G" => ksi,
        "A" | "Ax" | "Ay" | "Az" | "As" | "Asy" | "Asz" => area,
        "I" | "Ix" | "Iy" | "Iz" | "J" => inertia,
        "rho" => density,
        "nu" => 1.0,
        "k" | "ky" | "kz" => ksi,
        other => {
            return Err(ApiError::InvalidInput(format!(
                "US boundary has no conversion for section parameter «{other}»."
            )));
        }
    })
}

fn convert_law(law: &mut Law, ksi: f64, density: f64) -> Result<(), ApiError> {
    for parameter in &mut law.parameters {
        let factor = match parameter.0.name.as_str() {
            "E" | "G" | "fy" | "C" => ksi,
            "rho" => density,
            "b" => 1.0,
            other => {
                return Err(ApiError::InvalidInput(format!(
                    "US boundary has no conversion for law parameter «{other}»."
                )));
            }
        };
        parameter.0.value = scale(parameter.0.value, factor)?;
    }
    for point in &mut law.points {
        point[1] = scale(point[1], ksi)?;
    }
    Ok(())
}

fn convert_analysis(
    analysis: &mut Analysis,
    inch: f64,
    kip: f64,
    moment: f64,
) -> Result<(), ApiError> {
    match analysis {
        Analysis::LinearStatic {}
        | Analysis::SoilProfile { .. }
        | Analysis::Consolidation { .. }
        | Analysis::Buckling { .. } => {}
        Analysis::PDelta { options } => {
            options.0.displacement_scale = scale(options.0.displacement_scale, inch)?;
        }
        Analysis::Modal { options } => {
            if let Some(source) = options.0.mass_source.as_mut() {
                source.0.gravity = scale(source.0.gravity, inch)?;
            }
        }
        Analysis::Spectrum { options } => {
            if let Some(source) = options.0.mass_source.as_mut() {
                source.0.gravity = scale(source.0.gravity, inch)?;
            }
        }
        Analysis::Nonlinear { options } => {
            options.0.force_scale = scale(options.0.force_scale, kip)?;
            scale_slot(&mut options.0.displacement_scale, inch)?;
            scale_slot(&mut options.0.energy_scale, moment)?;
        }
    }
    Ok(())
}

/// Rewrites a kip-in document into the SI quantities the kernel already uses.
fn convert_united_states(input: &mut Input) -> Result<(), ApiError> {
    let kip = CatalogUnit::Kip.si_factor().to_f64();
    let inch = CatalogUnit::Inch.si_factor().to_f64();
    let ksi = CatalogUnit::Ksi.si_factor().to_f64();
    let moment = CatalogUnit::KipInch.si_factor().to_f64();
    let per_fahrenheit = CatalogUnit::PerFahrenheit.si_factor().to_f64();
    let delta_fahrenheit = CatalogUnit::Fahrenheit.si_factor().to_f64();
    let area = inch * inch;
    let inertia = area * area;
    let per_length = kip / inch;
    let mass = kip / inch;
    let density = kip / inertia;
    let model = &mut input.model.0;
    for node in &mut model.nodes {
        for coordinate in &mut node.0.coordinates {
            *coordinate = scale(*coordinate, inch)?;
        }
    }
    for section in &mut model.sections {
        for property in &mut section.0.properties {
            let factor = section_factor(property.0.name.as_str(), ksi, area, inertia, density)?;
            property.0.value = scale(property.0.value, factor)?;
        }
    }
    for law in &mut model.laws {
        convert_law(&mut law.0, ksi, density)?;
    }
    for item in &mut model.masses {
        for value in &mut item.0.values {
            let factor = if translation(value.0.dof) {
                mass
            } else {
                moment
            };
            value.0.value = scale(value.0.value, factor)?;
        }
    }
    for prescribed in &mut model.prescribed {
        let factor = if translation(prescribed.0.dof) {
            inch
        } else {
            1.0
        };
        prescribed.0.value = scale(prescribed.0.value, factor)?;
    }
    for member in &mut model.members {
        if let Some(winkler) = member.0.winkler.as_mut() {
            scale_slot(&mut winkler.0.k, ksi)?;
            scale_slot(&mut winkler.0.ky, ksi)?;
            scale_slot(&mut winkler.0.kz, ksi)?;
        }
    }
    for spring in &mut model.springs {
        if let Some(stiffness) = spring.0.k.as_mut() {
            let factor = if translation(spring.0.dof) {
                per_length
            } else {
                moment
            };
            *stiffness = scale(*stiffness, factor)?;
        }
    }
    for case in &mut input.cases {
        for load in &mut case.0.loads {
            for value in &mut load.0.values {
                let factor = if translation(value.0.dof) {
                    kip
                } else {
                    moment
                };
                value.0.value = scale(value.0.value, factor)?;
            }
        }
        for load in &mut case.0.member_loads {
            for component in &mut load.0.force_per_length {
                *component = scale(*component, per_length)?;
            }
            for component in &mut load.0.moment_per_length {
                *component = scale(*component, kip)?;
            }
        }
        for load in &mut case.0.temperature_loads {
            load.0.alpha_per_kelvin = scale(load.0.alpha_per_kelvin, per_fahrenheit)?;
            load.0.delta_t_kelvin = scale(load.0.delta_t_kelvin, delta_fahrenheit)?;
        }
    }
    for analysis in &mut input.analyses {
        convert_analysis(&mut analysis.0, inch, kip, moment)?;
    }
    Ok(())
}
