//! DTO privados: ninguna dependencia serde atraviesa engine-model.
use crate::ApiError;
use engine_core::{Dim, Dof, MemberId, NodeId};
use engine_model as domain;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};
use serde_json::Value;

// Serde acepta structs posicionales como arrays; el contrato exige objetos JSON.
pub(crate) struct Object<T>(pub T);

impl<'de, T: DeserializeOwned> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        if !value.is_object() {
            return Err(serde::de::Error::custom("Se requiere un objeto JSON"));
        }
        serde_json::from_value(value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

// Las etiquetas son solo strings; se rechaza la forma externa {"D2": null}.
macro_rules! string_enum {
    ($name:ident { $($variant:ident => $label:literal),+ $(,)? }) => {
        enum $name { $($variant),+ }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                match String::deserialize(d)?.as_str() {
                    $($label => Ok(Self::$variant),)+
                    _ => Err(serde::de::Error::custom(concat!("Etiqueta inválida: ", stringify!($name)))),
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
    #[serde(rename = "options")]
    _options: Object<Options>,
}

string_enum!(Units { SI => "SI" });
string_enum!(Axes { ZUp => "Z-up" });
string_enum!(Dimension { D2 => "D2", D3 => "D3" });
string_enum!(Component { Ux => "Ux", Uy => "Uy", Uz => "Uz", Rx => "Rx", Ry => "Ry", Rz => "Rz" });

impl Component {
    fn dof(self) -> Dof {
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
struct Options {}

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
}

// Ausencia permitida; null explícito no pertenece al esquema del vector.
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
    pub methods: Vec<&'static str>,
    pub analyses: Vec<Analysis>,
    pub elements: Vec<(String, Option<[f64; 3]>)>,
    pub loads: Vec<Vec<(engine_core::MemberId, engine_contracts::MemberLoad)>>,
    pub prescribed: Vec<engine_assembly::PrescribedDof>,
    pub diaphragms: Vec<engine_assembly::RigidDiaphragm>,
}

pub(crate) fn model(value: Value) -> Result<domain::Model, ApiError> {
    Ok(decode(value)?.model)
}

pub(crate) fn decode(value: Value) -> Result<Execution, ApiError> {
    let Object(input): Object<Input> =
        serde_json::from_value(value).map_err(|e| ApiError::InvalidInput(e.to_string()))?;
    let Object(source) = input.model;
    let analyses: Vec<_> = input.analyses.into_iter().map(|Object(a)| a).collect();
    let methods = analyses.iter().map(Analysis::method).collect();
    let elements = source
        .members
        .iter()
        .map(|Object(m)| (m.element.clone(), m.orientation))
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
    let mut model = domain::Model::new();
    for Object(n) in source.nodes {
        let dim = match n.dim {
            Dimension::D2 => Dim::D2,
            Dimension::D3 => Dim::D3,
        };
        if dim == Dim::D2 && n.coordinates[2] != 0.0 {
            return Err(ApiError::InvalidInput(format!(
                "Nodo {}: D2 requiere Z = 0",
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
                    "Carga: miembro {} ausente",
                    id.0
                )));
            }
        }
    }
    Ok(Execution {
        model,
        methods,
        analyses,
        elements,
        loads,
        prescribed,
        diaphragms,
    })
}
