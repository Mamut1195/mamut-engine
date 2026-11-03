//! Finite, planar elastic pile with independent nonlinear soil springs.
//!
//! The computational X axis is depth, positive downward; Y is the chosen
//! positive lateral direction. This is a local analysis plane, not a claim
//! about the engine's global vertical axis. Axial displacement and load are
//! positive downward. Head force H and lateral displacement y are positive Y.
//! Head moment M is positive about +Z. The reported bending moment and shear
//! follow `M = -EI*y''` and `V = EI*y'''`, respectively, between springs.
//! Reported p is the soil force on the pile, opposite mobilized resistance.
//!
//! Each node receives its own supplied p-y and t-z curve; q-z acts only at
//! the tip. Curves extend their end segments according to the explicit policy.
//! Softening is rejected. See `provenance/ENG-GEO-005.md` for the numerical
//! model, recovery conventions, limitations, and unexecuted checks.

use crate::newton::{NewtonElement, NewtonOptions, NewtonStepReport, NewtonSystem, StaticNewton};
use engine_assembly::{DofMap, ElementContribution, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, Status, UniaxialLaw};
use engine_core::{Dim, Dof, EResult, EngineError, NodeId, Real};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_elements_spring::{NonlinearSpring, SpringAxes, SpringConnection};
use engine_geo::pile::{spring, PileDiscretization};
use engine_geo::{py::PyCurve, tz::qz::QzCurve, tz::TzCurve};
use engine_laws::uniaxial::{MultilinearElastic, MultilinearMode};
use engine_linalg::LinAlgBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};
use std::sync::Arc;

/// Rotational boundary condition at the head; translations remain free.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Head {
    /// Applied moment in N m, including zero for a moment-free head.
    Free {
        /// Applied positive-Z moment in N m.
        moment_nm: Real,
    },
    /// Zero head rotation; the moment is a reaction.
    Fixed,
}

/// Geometry, elastic section, soil tables and applied head actions in SI.
///
/// The two curve slices must contain one table per mesh node, in depth order.
/// The diameter is used only for shaft circumference; area and inertia are
/// independent supplied section properties. No soil or structural parameter is
/// inferred. The tip is otherwise free, with no imposed displacement.
pub struct PileInput<'a> {
    /// Mesh, starting at depth zero.
    pub mesh: &'a PileDiscretization,
    /// Young modulus in Pa.
    pub young_pa: Real,
    /// Cross-sectional area in m^2.
    pub area_m2: Real,
    /// Second moment of area in m^4.
    pub inertia_m4: Real,
    /// Shaft diameter in m.
    pub diameter_m: Real,
    /// Lateral p-y curves, p in N/m.
    pub lateral: &'a [PyCurve],
    /// Shaft t-z curves, t in Pa.
    pub shaft: &'a [TzCurve],
    /// Tip q-z curve, Q in N.
    pub tip: &'a QzCurve,
    /// Explicit end-segment extension policy for all curves.
    pub end_policy: spring::EndPolicy,
    /// Head rotational condition.
    pub head: Head,
    /// Head lateral force H in N.
    pub horizontal_n: Real,
    /// Head axial force, positive downward, in N.
    pub vertical_n: Real,
}

/// A section result immediately below a node, except at the tip where it is
/// immediately above. Nodal springs can cause jumps in shear and axial force.
#[derive(Clone, Debug, PartialEq)]
pub struct PileStation {
    /// Depth in m.
    pub depth_m: Real,
    /// Lateral displacement in m.
    pub y_m: Real,
    /// Axial displacement, positive downward, in m.
    pub settlement_m: Real,
    /// Bending moment in N m, `-EI*y''` on the selected element side.
    pub moment_nm: Real,
    /// Shear in N, `EI*y'''` on the selected element side.
    pub shear_n: Real,
    /// Nodal lateral soil force divided by tributary length, N/m.
    pub soil_force_n_per_m: Real,
}

/// An accepted load factor and its head response and depth profile.
#[derive(Clone, Debug, PartialEq)]
pub struct PileStep {
    /// Absolute multiplier of all supplied head loads.
    pub factor: Real,
    /// Applied downward head force at this factor, N.
    pub axial_load_n: Real,
    /// Head settlement in m, positive downward.
    pub head_settlement_m: Real,
    /// Accepted nodal profile in increasing depth order.
    pub profile: Vec<PileStation>,
}

/// Attempt reports and accepted load-settlement/profile points.
#[derive(Clone, Debug, PartialEq)]
pub struct PileResults {
    /// Completed or NotConverged; singular tangents are returned as errors.
    pub status: Status,
    /// Attempt reports, including a failed final attempt.
    pub reports: Vec<NewtonStepReport>,
    /// Accepted steps only; no failed trial is included.
    pub accepted: Vec<PileStep>,
}

/// Assemble and solve the finite pile using the supplied linear algebra backend.
///
/// Factors are absolute, nonempty, finite and applied in order. Newton options
/// and their scales are explicit. On nonconvergence the failed trial is
/// discarded, subsequent factors are not attempted, and accepted steps are
/// returned with `Status::NotConverged`. Solver and element errors propagate;
/// a singular tangent does not return partial results. There is no automatic
/// subdivision, retry, line search, or geometric stiffness.
pub fn solve<B: LinAlgBackend>(
    backend: B,
    input: &PileInput<'_>,
    factors: &[Real],
    options: NewtonOptions,
) -> EResult<PileResults> {
    let depths = input.mesh.depths_m();
    let weights = input.mesh.tributaries_m();
    if depths.len() > 4097 || depths.first() != Some(&0.0) {
        return Err(invalid(
            "mesh must start at zero and have at most 4097 nodes",
        ));
    }
    if input.lateral.len() != depths.len() || input.shaft.len() != depths.len() {
        return Err(invalid("one lateral and shaft curve is required per node"));
    }
    if factors.is_empty() {
        return Err(invalid("at least one load factor is required"));
    }
    finite(input.diameter_m)?;
    finite(input.horizontal_n)?;
    finite(input.vertical_n)?;
    if input.diameter_m <= 0.0 {
        return Err(invalid("diameter must be positive"));
    }
    let moment = match input.head {
        Head::Free { moment_nm } => finite(moment_nm)?,
        Head::Fixed => 0.0,
    };
    for &factor in factors {
        finite(factor)?;
        finite(factor * input.horizontal_n)?;
        finite(factor * input.vertical_n)?;
        finite(factor * moment)?;
    }
    let circumference = finite(std::f64::consts::PI * input.diameter_m)?;
    if circumference <= 0.0 {
        return Err(invalid("circumference underflows"));
    }
    let beam = EulerFrame2D::new(
        input.young_pa,
        input.area_m2,
        input.inertia_m4,
        0.0,
        MomentReleases::default(),
    )?;
    let mut model = Model::new();
    for (i, &depth) in depths.iter().enumerate() {
        model
            .nodes
            .push(Node::new(node_id(i)?, Dim::D2, [depth, 0.0, 0.0]));
    }
    if input.head == Head::Fixed {
        model.supports.push(Support {
            id: "pile-head-rotation".into(),
            node: node_id(0)?,
            restrained: vec![Dof::Rz],
        });
    }
    let case = CaseRef("pile-head".into());
    model.cases.push(LoadCase {
        name: case.0.clone(),
        loads: vec![NodalLoad {
            node: node_id(0)?,
            values: vec![
                DofValue {
                    dof: Dof::Ux,
                    value: input.vertical_n,
                },
                DofValue {
                    dof: Dof::Uy,
                    value: input.horizontal_n,
                },
                DofValue {
                    dof: Dof::Rz,
                    value: moment,
                },
            ],
        }],
    });
    let map = DofMap::new(&model.nodes)?;
    let params = ElementParams::default();
    let mut contributions = Vec::with_capacity(depths.len() - 1);
    for i in 0..depths.len() - 1 {
        let nodes = [node_id(i)?, node_id(i + 1)?];
        let coords = [map.coordinates(nodes[0])?, map.coordinates(nodes[1])?];
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        let mut location = Vec::with_capacity(6);
        for node in nodes {
            for &dof in beam.dofs_per_node() {
                location.push(map.global_index(node, dof)?);
            }
        }
        let response = beam.respond(&ctx, &[0.0; 6], &[])?;
        contributions.push(ElementContribution {
            location,
            stiffness: response.tangent,
            mass_lumped: None,
            mass_consistent: None,
            geometric: None,
            loads: Vec::new(),
        });
    }
    let mut elements = Vec::with_capacity(depths.len() * 2 + 1);
    let mut lateral_laws = Vec::with_capacity(depths.len());
    for (i, &weight) in weights[..depths.len()].iter().enumerate() {
        let lateral = law(spring::lateral(
            &input.lateral[i],
            weight,
            input.end_policy,
        )?)?;
        let shaft = law(spring::shaft(
            &input.shaft[i],
            circumference,
            weight,
            input.end_policy,
        )?)?;
        elements.push(binding(
            &map,
            node_id(i)?,
            Dof::Uy,
            lateral.clone(),
            &params,
        )?);
        elements.push(binding(&map, node_id(i)?, Dof::Ux, shaft, &params)?);
        lateral_laws.push(lateral);
    }
    let tip = law(spring::tip(input.tip, input.end_policy)?)?;
    elements.push(binding(
        &map,
        node_id(depths.len() - 1)?,
        Dof::Ux,
        tip,
        &params,
    )?);
    let linear = LinearSystem::from_model(backend, &model, &contributions, &[])?;
    let mut system = NewtonSystem::new(linear, elements)?;
    let mut reports = Vec::with_capacity(factors.len());
    let mut accepted = Vec::with_capacity(factors.len());
    let mut status = Status::Completed;
    for &factor in factors {
        let result = StaticNewton::run(&mut system, &case, &[factor], options)?;
        reports.extend(result.steps);
        if result.status != Status::Completed {
            status = result.status;
            break;
        }
        let u = &result.global_displacements;
        let mut profile = Vec::with_capacity(depths.len());
        for i in 0..depths.len() {
            let node = node_id(i)?;
            let ux = u[map.global_index(node, Dof::Ux)?.0 as usize];
            let uy = u[map.global_index(node, Dof::Uy)?.0 as usize];
            let span = i.min(depths.len() - 2);
            let nodes = [node_id(span)?, node_id(span + 1)?];
            let coords = [map.coordinates(nodes[0])?, map.coordinates(nodes[1])?];
            let ctx = ElementCtx {
                dim: Dim::D2,
                coords: &coords,
                params: &params,
            };
            let mut element_u = Vec::with_capacity(6);
            for n in nodes {
                for &dof in beam.dofs_per_node() {
                    element_u.push(u[map.global_index(n, dof)?.0 as usize]);
                }
            }
            let position = if i == depths.len() - 1 { 1.0 } else { 0.0 };
            let recovered = beam.recover(&ctx, &element_u, &[], &[], &[position])?;
            let station = recovered.stations.first().ok_or_else(|| {
                EngineError::new("ENG-E-0012", "Pile: missing frame recovery station.")
            })?;
            let resistance = lateral_laws[i].respond(uy, &[])?.stress;
            profile.push(PileStation {
                depth_m: depths[i],
                y_m: uy,
                settlement_m: ux,
                moment_nm: finite(-station.forces_local[5])?,
                shear_n: finite(-station.forces_local[1])?,
                soil_force_n_per_m: finite(-resistance / weights[i])?,
            });
        }
        let head_settlement_m = profile
            .first()
            .ok_or_else(|| invalid("empty profile"))?
            .settlement_m;
        accepted.push(PileStep {
            factor,
            axial_load_n: finite(factor * input.vertical_n)?,
            head_settlement_m,
            profile,
        });
    }
    Ok(PileResults {
        status,
        reports,
        accepted,
    })
}

fn law(points: Vec<(Real, Real)>) -> EResult<Arc<dyn UniaxialLaw>> {
    Ok(Arc::new(MultilinearElastic::new(
        points,
        MultilinearMode::Both,
    )?))
}

fn binding(
    map: &DofMap,
    node: NodeId,
    dof: Dof,
    law: Arc<dyn UniaxialLaw>,
    params: &ElementParams,
) -> EResult<NewtonElement> {
    let spring = NonlinearSpring::new(
        Dim::D2,
        SpringConnection::Ground,
        law,
        &[dof],
        SpringAxes::Global,
    )?;
    let coords = [map.coordinates(node)?];
    NewtonElement::new(
        Box::new(spring),
        vec![map.global_index(node, dof)?],
        &ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params,
        },
    )
}

fn node_id(index: usize) -> EResult<NodeId> {
    let id = u32::try_from(index + 1).map_err(|_| invalid("node identifier overflow"))?;
    Ok(NodeId(id))
}

fn finite(value: Real) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            "Pile: nonfinite input or result.",
        ))
    }
}

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", format!("Pile: {message}."))
}
