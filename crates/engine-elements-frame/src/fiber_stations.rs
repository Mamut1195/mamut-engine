//! Frame bar `frame2d.fiber_stations`, contract element/1, method 0.1.0.
//!
//! The sheet is `docs/fichas/frame2d.fiber_stations.md`. A
//! `rectangle.fiber_bending` section is sampled at three stations. Virtual
//! work uses those samples. This bar does not replace `frame2d.euler_elastic`.
//!
//! Small-displacement Euler–Bernoulli kinematics, no shear strain, no rotary
//! inertia, no mass and no geometric stiffness. Nonzero span loads are
//! outside method 0.1.0. D2 only: local x from i to j, y = (−sin, cos),
//! z = +Z. Order `[ux, uy, rz]` per node. Positive N is tension. Positive
//! rotation is about +z and equals dv/dx. A positive rotation moves the
//! fiber at y toward −x by y·θ, so ε(y) = ε0 − y v″. The section uses
//! ε(y) = ε0 − κz y, therefore κz = v″. Method 0.1.0 is experimental.

use crate::geometry::Geometry;
use crate::{finite, invalid, unsupported, vector, Matrix};
use engine_contracts::{
    Element, ElementCtx, ElementOutput, ElementResponse, MassKind, MemberLoad, SectionLaw,
    SectionResponse, StationResult,
};
use engine_core::math;
use engine_core::{ContractId, Dof, EResult, EngineError, Real};
use engine_laws::section::RectangleFiberBending;
use engine_linalg::DMat;
use std::slice;

/// Number of integration stations in method 0.1.0.
///
/// The count is the one written in the method sheet. It is not a time or
/// memory budget, and it is not moved to match another program.
pub const FIBER_STATION_COUNT: usize = 3;

/// Upper count of stations accepted by `recover`.
///
/// This is the domain of method 0.1.0. It is not a time or memory budget.
pub const MAX_RECOVER_STATIONS: usize = 64;

/// Method 0.1.0 is experimental.
pub const FIBER_STATIONS_EXPERIMENTAL: bool = true;

const DOFS: [Dof; 3] = [Dof::Ux, Dof::Uy, Dof::Rz];

/// Two-node bar whose section is [`RectangleFiberBending`].
///
/// Experimental method 0.1.0. History is the section history at each
/// station, left to right. `respond`
/// does not mutate the bar or the committed history. The caller keeps a
/// trial only by replacing the committed slice.
///
/// ```
/// use engine_contracts::{Element, ElementCtx, ElementParams};
/// use engine_core::Dim;
/// use engine_elements_frame::FiberStationBar;
/// use engine_laws::section::{RectangleFiberBending, RectangleFiberBendingParams, SteelBar};
/// use engine_laws::uniaxial::{KentParkParams, SteelBilinearKinematicParams};
/// let psi = 4.4482216152605 / (0.0254 * 0.0254);
/// let section = RectangleFiberBending::new(
///     Dim::D2,
///     RectangleFiberBendingParams {
///         width: 0.25,
///         height: 0.5,
///         strip_count: 4,
///         concrete: KentParkParams {
///             fc: 3000.0 * psi,
///             rho_s: 0.0,
///             b_double_prime: 0.2,
///             s_h: 0.1,
///         },
///         steel: SteelBilinearKinematicParams {
///             e: 200_000_000_000.0,
///             fy: 500_000_000.0,
///             b: 0.0,
///         },
///         bars: vec![
///             SteelBar { area: 1.0 / 1024.0, y: -3.0 / 16.0 },
///             SteelBar { area: 1.0 / 1024.0, y: 3.0 / 16.0 },
///         ],
///     },
/// )?;
/// let bar = FiberStationBar::new(section);
/// let params = ElementParams::default();
/// let coords = [[0.0; 3], [4.0, 0.0, 0.0]];
/// let ctx = ElementCtx { dim: Dim::D2, coords: &coords, params: &params };
/// let response = bar.respond(&ctx, &[0.0; 6], &bar.initial_state())?;
/// assert_eq!(response.internal_force, vec![0.0; 6]);
/// assert_ne!(bar.kind(), "frame2d.euler_elastic");
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FiberStationBar {
    section: RectangleFiberBending,
}

struct Station {
    xi: Real,
    weight: Real,
}

struct Operators {
    axial: [Real; 6],
    curvature: [Real; 6],
}

impl FiberStationBar {
    /// Stores an already validated rectangular fiber section.
    pub fn new(section: RectangleFiberBending) -> Self {
        Self { section }
    }

    /// Section sampled at the stations. Immutable.
    pub fn section(&self) -> &RectangleFiberBending {
        &self.section
    }

    fn stride(&self) -> usize {
        self.section.state_len()
    }

    fn integrate(
        &self,
        length: Real,
        local: &[Real; 6],
        committed: &[Real],
    ) -> EResult<([Real; 6], Matrix, Vec<Real>)> {
        let stations = stations(length)?;
        let mut force = [0.0; 6];
        let mut tangent = [[0.0; 6]; 6];
        let mut trial = Vec::with_capacity(self.state_len());
        let stride = self.stride();
        for (index, station) in stations.iter().enumerate() {
            let start = index * stride;
            let history = committed
                .get(start..start + stride)
                .ok_or_else(|| invalid("Committed history: size does not match the stations."))?;
            let ops = operators(length, station.xi)?;
            let deformation = [axial_strain(length, local)?, curvature(&ops, local)?];
            let response = self.section.respond(&deformation, history)?;
            let (axial, moment) = resultants(&response)?;
            add_force(&mut force, station.weight, &ops, axial, moment)?;
            add_tangent(&mut tangent, station.weight, &ops, &response)?;
            if response.trial_state.len() != stride {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Section history length changed at a station.",
                ));
            }
            trial.extend_from_slice(&response.trial_state);
        }
        Ok((force, tangent, trial))
    }

    fn history_index(&self, length: Real, position: Real) -> EResult<usize> {
        let stations = stations(length)?;
        let mut best = 0usize;
        let mut best_gap = Real::INFINITY;
        for (index, station) in stations.iter().enumerate() {
            let gap = finite((station.xi - position).abs(), "station distance")?;
            if gap < best_gap {
                best = index;
                best_gap = gap;
            }
        }
        Ok(best)
    }
}

fn stations(length: Real) -> EResult<[Station; FIBER_STATION_COUNT]> {
    let side = finite(math::sqrt(finite(3.0 / 5.0, "3/5")?), "sqrt(3/5)")?;
    let end = finite(5.0 / 9.0, "end weight")?;
    // Equation (3): the central weight is what makes the three weights sum to 2.
    let mid = finite(2.0 - end - end, "central weight")?;
    let half = finite(length / 2.0, "L/2 (m)")?;
    let map = |t: Real, weight: Real| -> EResult<Station> {
        Ok(Station {
            xi: finite((t + 1.0) / 2.0, "ξ")?,
            weight: finite(half * weight, "station weight (m)")?,
        })
    };
    Ok([map(-side, end)?, map(0.0, mid)?, map(side, end)?])
}

fn operators(length: Real, xi: Real) -> EResult<Operators> {
    let inverse = finite(1.0 / length, "1/L (m⁻¹)")?;
    let length_sq = finite(length * length, "L² (m²)")?;
    // κz = v″. These are ∂v″/∂q from the Hermite functions in the sheet.
    let bend_i = finite((-6.0 + 12.0 * xi) / length_sq, "v_i curvature operator")?;
    let theta_i = finite((-4.0 + 6.0 * xi) * inverse, "θ_i curvature operator")?;
    let bend_j = finite((6.0 - 12.0 * xi) / length_sq, "v_j curvature operator")?;
    let theta_j = finite((-2.0 + 6.0 * xi) * inverse, "θ_j curvature operator")?;
    Ok(Operators {
        axial: [-inverse, 0.0, 0.0, inverse, 0.0, 0.0],
        curvature: [0.0, bend_i, theta_i, 0.0, bend_j, theta_j],
    })
}

fn axial_strain(length: Real, local: &[Real; 6]) -> EResult<Real> {
    let elongation = finite(local[3] - local[0], "elongation (m)")?;
    finite(elongation / length, "ε0")
}

fn curvature(ops: &Operators, local: &[Real; 6]) -> EResult<Real> {
    let mut value = 0.0;
    for (&operator, &dof) in ops.curvature.iter().zip(local) {
        value = finite(value + operator * dof, "κz (m⁻¹)")?;
    }
    Ok(value)
}

fn resultants(response: &SectionResponse) -> EResult<(Real, Real)> {
    if response.resultants.len() != 2 {
        return Err(EngineError::new(
            "ENG-E-0012",
            "Section returned the wrong number of resultants.",
        ));
    }
    Ok((
        finite(response.resultants[0], "N (N)")?,
        finite(response.resultants[1], "Mz (N·m)")?,
    ))
}

fn add_force(
    force: &mut [Real; 6],
    weight: Real,
    ops: &Operators,
    axial: Real,
    moment: Real,
) -> EResult<()> {
    for ((slot, axial_op), curvature_op) in force.iter_mut().zip(ops.axial).zip(ops.curvature) {
        let work = finite(axial_op * axial + curvature_op * moment, "virtual work")?;
        *slot = finite(*slot + weight * work, "internal force")?;
    }
    Ok(())
}

fn add_tangent(
    tangent: &mut Matrix,
    weight: Real,
    ops: &Operators,
    response: &SectionResponse,
) -> EResult<()> {
    let entry = |row: usize, col: usize| -> EResult<Real> {
        response
            .tangent
            .get(row, col)
            .ok_or_else(|| EngineError::new("ENG-E-0012", "Section tangent entry is missing."))
    };
    let stiffness = [[entry(0, 0)?, entry(0, 1)?], [entry(1, 0)?, entry(1, 1)?]];
    let rows = [ops.axial, ops.curvature];
    for row in 0..6 {
        for col in row..6 {
            let first = finite(
                stiffness[0][0] * rows[0][col] + stiffness[0][1] * rows[1][col],
                "tangent column",
            )?;
            let second = finite(
                stiffness[1][0] * rows[0][col] + stiffness[1][1] * rows[1][col],
                "tangent column",
            )?;
            let value = finite(
                rows[0][row] * first + rows[1][row] * second,
                "material tangent",
            )?;
            tangent[row][col] = finite(tangent[row][col] + weight * value, "tangent")?;
            if row != col {
                tangent[col][row] = tangent[row][col];
            }
        }
    }
    Ok(())
}

fn reject_loads(loads: &[MemberLoad]) -> EResult<()> {
    for load in loads {
        vector(&load.force_per_length, 3, "q (N/m)")?;
        vector(&load.moment_per_length, 3, "m (N)")?;
        let nonzero = load
            .force_per_length
            .iter()
            .chain(load.moment_per_length.iter())
            .any(|component| *component != 0.0);
        if nonzero {
            return Err(unsupported(
                "frame2d.fiber_stations: a nonzero span load is outside method 0.1.0.",
            ));
        }
    }
    Ok(())
}

impl Element for FiberStationBar {
    fn kind(&self) -> &'static str {
        "frame2d.fiber_stations"
    }

    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    }

    fn n_nodes(&self) -> usize {
        2
    }

    fn dofs_per_node(&self) -> &'static [Dof] {
        &DOFS
    }

    fn state_len(&self) -> usize {
        self.stride().saturating_mul(FIBER_STATION_COUNT)
    }

    fn initial_state(&self) -> Vec<Real> {
        let one = self.section.initial_state();
        let mut state = Vec::with_capacity(one.len() * FIBER_STATION_COUNT);
        for _ in 0..FIBER_STATION_COUNT {
            state.extend_from_slice(&one);
        }
        state
    }

    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        vector(committed, self.state_len(), "Committed history")?;
        let geometry = Geometry::new(ctx)?;
        let local = geometry.local_vector(u_e)?;
        let (force, tangent, trial_state) = self.integrate(geometry.length, &local, committed)?;
        Ok(ElementResponse {
            internal_force: geometry.global_vector(&force)?.to_vec(),
            tangent: geometry.global_matrix(&tangent)?,
            trial_state,
        })
    }

    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        let _ = Geometry::new(ctx)?;
        match kind {
            MassKind::Lumped | MassKind::Consistent => Ok(None),
        }
    }

    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        vector(committed, self.state_len(), "Committed history")?;
        vector(u_e, 6, "u_e (m/rad)")?;
        let _ = Geometry::new(ctx)?;
        Ok(None)
    }

    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        let _ = Geometry::new(ctx)?;
        reject_loads(slice::from_ref(load))?;
        Ok(vec![0.0; 6])
    }

    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations_requested: &[Real],
    ) -> EResult<ElementOutput> {
        vector(committed, self.state_len(), "Committed history")?;
        if stations_requested.len() > MAX_RECOVER_STATIONS {
            return Err(invalid(
                "frame2d.fiber_stations: station count must be from 0 to 64.",
            ));
        }
        let geometry = Geometry::new(ctx)?;
        let local = geometry.local_vector(u_e)?;
        reject_loads(loads)?;
        let (force, _, _) = self.integrate(geometry.length, &local, committed)?;
        let mut stations_out = Vec::new();
        stations_out
            .try_reserve_exact(stations_requested.len())
            .map_err(|_| invalid("Could not allocate stations."))?;
        let shear = finite(-force[1], "Vy (N)")?;
        let stride = self.stride();
        for &position in stations_requested {
            finite(position, "Station x/L")?;
            if !(0.0..=1.0).contains(&position) {
                return Err(invalid("Station is outside [0, 1]."));
            }
            let index = self.history_index(geometry.length, position)?;
            let start = index * stride;
            let history = committed
                .get(start..start + stride)
                .ok_or_else(|| invalid("Committed history: size does not match the stations."))?;
            let ops = operators(geometry.length, position)?;
            let deformation = [
                axial_strain(geometry.length, &local)?,
                curvature(&ops, &local)?,
            ];
            let response = self.section.respond(&deformation, history)?;
            let (axial, moment) = resultants(&response)?;
            stations_out.push(StationResult {
                position,
                forces_local: [axial, shear, 0.0, 0.0, 0.0, moment],
            });
        }
        Ok(ElementOutput {
            end_forces_local: force.to_vec(),
            stations: stations_out,
        })
    }
}
