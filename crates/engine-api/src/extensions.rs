//! Fontanería SFC-010: propiedades de frontera, sin algoritmos constitutivos.
use crate::{
    analysis_input::Analysis,
    input::{Component, Execution, Object},
};
use engine_contracts::{Element, UniaxialLaw};
use engine_core::{Dim, EResult, EngineError, NodeId};
use engine_elements_spring::{LinearSpring, NonlinearSpring, SpringAxes, SpringConnection};
use engine_laws::uniaxial::{
    CubicElastic, Elastic, MultilinearElastic, MultilinearMode, SteelBilinearKinematic,
};
use serde::{de::DeserializeOwned, Deserialize, Deserializer};
use std::{collections::BTreeSet, sync::Arc};

pub(crate) fn optional<'de, T: DeserializeOwned, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

pub(crate) fn nonempty<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let value = String::deserialize(d)?;
    if value.is_empty() {
        return Err(serde::de::Error::custom("Empty tag is not allowed."));
    }
    Ok(value)
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum Release {
    Plane(bool),
    Spatial([bool; 3]),
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Winkler {
    #[serde(default, deserialize_with = "optional")]
    pub k: Option<f64>,
    #[serde(default, deserialize_with = "optional")]
    pub ky: Option<f64>,
    #[serde(default, deserialize_with = "optional")]
    pub kz: Option<f64>,
}
pub(crate) struct Beam {
    pub kind: String,
    pub orientation: Option<[f64; 3]>,
    pub release_i: Option<Release>,
    pub release_j: Option<Release>,
    pub winkler: Option<Winkler>,
}

pub(crate) struct Law {
    pub name: String,
    pub kind: String,
    pub parameters: Vec<(String, f64)>,
    pub points: Vec<[f64; 2]>,
    pub mode: String,
}
impl Law {
    pub fn build(&self) -> EResult<Arc<dyn UniaxialLaw>> {
        let allowed: &[&str] = match self.kind.as_str() {
            "" | "elastic" => &["E", "rho", "G"],
            "elastic.cubic" => &["E", "C"],
            "steel.bilinear_kinematic" => &["E", "fy", "b"],
            "multilinear_elastic" => &[],
            _ => return Err(EngineError::new("ENG-E-0013", "Unsupported law type.")),
        };
        let mut seen = BTreeSet::new();
        for (key, _) in &self.parameters {
            if !seen.insert(key) {
                return Err(EngineError::new("ENG-E-0002", "Duplicate law parameter."));
            }
            if !allowed.contains(&key.as_str()) {
                return Err(invalid("Law parameter is not allowed."));
            }
        }
        if self.kind != "multilinear_elastic" && (!self.points.is_empty() || !self.mode.is_empty())
        {
            return Err(invalid("points/mode apply only to multilinear_elastic."));
        }
        let number = |key| {
            self.parameters
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| *v)
                .ok_or_else(|| {
                    EngineError::new("ENG-E-0003", format!("Law {}: missing {key}.", self.name))
                })
        };
        Ok(match self.kind.as_str() {
            "" | "elastic" => Arc::new(Elastic::new(number("E")?)?),
            "elastic.cubic" => Arc::new(CubicElastic::new(number("E")?, number("C")?)?),
            "steel.bilinear_kinematic" => Arc::new(SteelBilinearKinematic::new(
                number("E")?,
                number("fy")?,
                number("b")?,
            )?),
            "multilinear_elastic" => {
                let mode = match self.mode.as_str() {
                    "" | "both" => MultilinearMode::Both,
                    "compression_only" => MultilinearMode::CompressionOnly,
                    "tension_only" => MultilinearMode::TensionOnly,
                    _ => return Err(invalid("Unknown multilinear mode.")),
                };
                Arc::new(MultilinearElastic::new(
                    self.points.iter().map(|p| (p[0], p[1])).collect(),
                    mode,
                )?)
            }
            _ => return Err(invalid("Unresolved law.")),
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Spring {
    pub id: u32,
    pub nodes: Vec<u32>,
    pub kind: String,
    pub dof: Component,
    #[serde(default, deserialize_with = "optional")]
    pub k: Option<f64>,
    #[serde(default, deserialize_with = "optional")]
    pub law: Option<String>,
    #[serde(default, deserialize_with = "optional")]
    pub direction: Option<[f64; 3]>,
    #[serde(default, deserialize_with = "optional")]
    pub rotation: Option<[[f64; 3]; 3]>,
}
impl Spring {
    pub fn build(&self, execution: &Execution) -> EResult<(Dim, Box<dyn Element>)> {
        let connection = match self.nodes.len() {
            1 => SpringConnection::Ground,
            2 if self.nodes[0] != self.nodes[1] => SpringConnection::BetweenNodes,
            _ => return Err(invalid("Spring: one or two distinct nodes are required.")),
        };
        let mut dim = None;
        for &id in &self.nodes {
            let node = execution
                .model
                .nodes
                .iter()
                .find(|n| n.id == NodeId(id))
                .ok_or_else(|| EngineError::new("ENG-E-0003", "Spring: missing node."))?;
            if dim.is_some_and(|d| d != node.dim) {
                return Err(invalid("Spring: dimensions differ."));
            }
            dim = Some(node.dim);
        }
        let dim = dim.ok_or_else(|| invalid("Spring has no dimension."))?;
        let axes = match (self.direction, self.rotation) {
            (None, None) => SpringAxes::Global,
            (Some(d), None) => SpringAxes::Direction(d),
            (None, Some(r)) => SpringAxes::Rotation(r),
            _ => {
                return Err(invalid(
                    "Spring: direction and rotation are mutually exclusive.",
                ))
            }
        };
        let dof = self.dof.dof();
        let element: Box<dyn Element> = match self.kind.as_str() {
            "spring.linear" if self.law.is_none() => Box::new(LinearSpring::new(
                dim,
                connection,
                self.k
                    .ok_or_else(|| EngineError::new("ENG-E-0003", "Linear spring: missing k."))?,
                &[dof],
                axes,
            )?),
            "spring.nonlinear" if self.k.is_none() => {
                let law = execution
                    .laws
                    .iter()
                    .find(|l| Some(&l.name) == self.law.as_ref())
                    .ok_or_else(|| EngineError::new("ENG-E-0003", "Spring: missing law."))?;
                Box::new(NonlinearSpring::new(
                    dim,
                    connection,
                    law.build()?,
                    &[dof],
                    axes,
                )?)
            }
            "spring.linear" | "spring.nonlinear" => return Err(invalid(
                "Linear spring requires k without law; nonlinear spring requires law without k.",
            )),
            _ => return Err(EngineError::new("ENG-E-0013", "Unsupported spring type.")),
        };
        for &id in &self.nodes {
            let node = execution
                .model
                .nodes
                .iter()
                .find(|n| n.id == NodeId(id))
                .ok_or_else(|| EngineError::new("ENG-E-0003", "Spring: missing node."))?;
            if element
                .dofs_per_node()
                .iter()
                .any(|d| !node.dofs.contains(d))
            {
                return Err(EngineError::new(
                    "ENG-E-0011",
                    "Spring: missing active global DOF.",
                ));
            }
        }
        Ok((dim, element))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Newton {
    pub case: String,
    pub load_factors: Vec<f64>,
    pub tolerance: f64,
    pub max_iterations: u32,
    pub force_scale: f64,
    #[serde(default, deserialize_with = "optional")]
    pub displacement_tolerance: Option<f64>,
    #[serde(default, deserialize_with = "optional")]
    pub energy_tolerance: Option<f64>,
    #[serde(default, deserialize_with = "optional")]
    pub displacement_scale: Option<f64>,
    #[serde(default, deserialize_with = "optional")]
    pub energy_scale: Option<f64>,
}
impl Newton {
    pub fn options(&self) -> EResult<engine_analysis::newton::NewtonOptions> {
        if self.load_factors.is_empty()
            || self.load_factors.len() > 16384
            || self.max_iterations == 0
            || self.max_iterations > 16384
        {
            return Err(invalid("Newton: steps or maximum is outside [1,16384]."));
        }
        if self.load_factors.iter().any(|v| !v.is_finite()) {
            return Err(EngineError::new("ENG-E-0008", "Newton: non-finite factor."));
        }
        for v in [
            Some(self.tolerance),
            Some(self.force_scale),
            self.displacement_tolerance,
            self.energy_tolerance,
            self.displacement_scale,
            self.energy_scale,
        ]
        .into_iter()
        .flatten()
        {
            if !v.is_finite() || v <= 0.0 {
                return Err(invalid(
                    "Newton: positive finite tolerances/scales are required.",
                ));
            }
        }
        if (self.displacement_tolerance.is_some() && self.displacement_scale.is_none())
            || (self.energy_tolerance.is_some() && self.energy_scale.is_none())
        {
            return Err(invalid(
                "Newton: each additional criterion requires its explicit scale.",
            ));
        }
        Ok(engine_analysis::newton::NewtonOptions {
            max_iterations: self.max_iterations,
            force_tolerance: Some(self.tolerance),
            force_scale: self.force_scale,
            displacement_tolerance: self.displacement_tolerance,
            energy_tolerance: self.energy_tolerance,
            displacement_scale: self.displacement_scale.unwrap_or(1.0),
            energy_scale: self.energy_scale.unwrap_or(1.0),
        })
    }
}

pub(crate) fn invalid(message: &str) -> EngineError {
    EngineError::new("API-E-SFC-OPTIONS", message)
}

pub(crate) fn validate(e: &Execution) -> EResult<()> {
    let method = e.methods.first().copied().unwrap_or("");
    if e.springs.len() + e.model.members.len() > crate::MAX_MEMBERS {
        return Err(invalid("Maximum of 1024 members and springs in total."));
    }
    if !e.springs.is_empty() && !matches!(method, "linearStatic" | "nonlinearStatic") {
        return Err(EngineError::new(
            "ENG-E-0013",
            "Springs: supported analyses are linearStatic/nonlinearStatic.",
        ));
    }
    let mut ids: BTreeSet<_> = e.model.members.iter().map(|m| m.id.0).collect();
    for s in &e.springs {
        if !ids.insert(s.id) {
            return Err(EngineError::new(
                "ENG-E-0002",
                "Duplicate member/spring ID.",
            ));
        }
        if s.kind == "spring.nonlinear" && method != "nonlinearStatic" {
            return Err(EngineError::new(
                "ENG-E-0013",
                "spring.nonlinear requires nonlinearStatic.",
            ));
        }
        s.build(e).map_err(|mut error| {
            error
                .entities
                .push(engine_core::EntityRef::Member(engine_core::MemberId(s.id)));
            error
        })?;
    }
    for l in &e.laws {
        // Las leyes históricas sin kind conservan exactamente la validación anterior.
        if !l.kind.is_empty() || !l.points.is_empty() || !l.mode.is_empty() {
            l.build()?;
        }
    }
    for (member, beam) in e.model.members.iter().zip(&e.elements) {
        if let Some(law) = e.laws.iter().find(|l| l.name == member.law) {
            if !law.kind.is_empty() && law.kind != "elastic" {
                return Err(EngineError::new(
                    "ENG-E-0013",
                    "Beams require an elastic law; nonlinear laws are connected to springs.",
                ));
            }
        }
        if beam.winkler.is_some()
            && !matches!(beam.kind.as_str(), "frame2d.winkler" | "frame3d.winkler")
        {
            return Err(invalid("winkler requires a Winkler beam kind."));
        }
        if (beam.release_i.is_some() || beam.release_j.is_some())
            && !matches!(
                beam.kind.as_str(),
                "" | "frame2d.euler_elastic" | "frame3d.euler_elastic"
            )
        {
            return Err(invalid("Releases are allowed only on elastic Euler beams."));
        }
    }
    if let Some(Analysis::Nonlinear { options: Object(o) }) = e.analyses.first() {
        o.options()?;
        if !e.model.cases.iter().any(|c| c.name == o.case) {
            return Err(EngineError::new("ENG-E-0003", "Newton: missing case."));
        }
        if !e.model.combinations.is_empty() || !e.diaphragms.is_empty() {
            return Err(EngineError::new(
                "ENG-E-0013",
                "Newton: combinations and diaphragms are not supported.",
            ));
        }
    }
    Ok(())
}
