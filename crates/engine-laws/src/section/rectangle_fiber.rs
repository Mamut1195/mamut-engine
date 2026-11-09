//! Rectangular fiber section `rectangle.fiber_bending`, contract `section/1`.
//!
//! The sheet is `docs/fichas/rectangle.fiber_bending.md`. Concrete strips use
//! `concrete.kent_park`. Steel bars use `steel.bilinear_kinematic`. The gross
//! concrete rectangle is kept, and each bar is added at its own centroid.

use super::SectionFactory;
use crate::uniaxial::{
    KentPark, KentParkParams, SteelBilinearKinematic, SteelBilinearKinematicParams,
};
use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{DMat, SectionLaw, SectionResponse, UniaxialLaw};
use engine_core::{ContractId, Dim, EResult, EngineError, Real};

/// Rectangular fiber section descriptor, method 0.1.0.
pub const RECTANGLE_FIBER_BENDING: LawDescriptor = LawDescriptor {
    kind: "rectangle.fiber_bending",
    contract: ContractId::new("section", 1, 0),
    version: (0, 1, 0),
};

/// Upper end of the strip count for method 0.1.0.
///
/// This is the domain written in the method sheet. It does not approve a
/// time or memory budget.
const MAX_STRIPS: usize = 64;
/// Upper end of the bar count for method 0.1.0.
///
/// This is the domain written in the method sheet. It does not approve a
/// time or memory budget.
const MAX_BARS: usize = 32;
/// Upper end of the constant-axial update count for method 0.1.0.
const MAX_UPDATES: usize = 32;

/// One steel bar, area in m² and centroid y in m.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SteelBar {
    /// Bar area in m², finite and strictly positive.
    pub area: Real,
    /// Centroid y in m, measured from the rectangle centroid. |y| ≤ h/2.
    pub y: Real,
}

/// SI description of [`RectangleFiberBending`], validated on construction.
#[derive(Clone, Debug, PartialEq)]
pub struct RectangleFiberBendingParams {
    /// Width b along local z, in m, finite and strictly positive.
    pub width: Real,
    /// Height h along local y, in m, finite and strictly positive.
    pub height: Real,
    /// Equal concrete strips through the height. Integer from 1 to 64.
    pub strip_count: usize,
    /// Shared concrete law. Every strip uses these parameters.
    pub concrete: KentParkParams,
    /// Shared steel law. Every bar uses these parameters.
    pub steel: SteelBilinearKinematicParams,
    /// Bars in state order. Count from 1 to 32. Area is added to the gross
    /// concrete area and is not removed from it.
    pub bars: Vec<SteelBar>,
}

/// One curvature station at a requested axial force.
#[derive(Clone, Debug, PartialEq)]
pub struct CurvaturePoint {
    /// Axial strain ε0 that meets the residual limit.
    pub axial_strain: Real,
    /// Curvature κz in m⁻¹, as requested.
    pub curvature: Real,
    /// Integrated axial force in N. Tension is positive.
    pub axial_force: Real,
    /// Integrated moment Mz in N·m.
    pub moment: Real,
    /// Trial history at this strain pair. The caller decides whether to keep it.
    pub trial_state: Vec<Real>,
}

/// Typed factory for [`RectangleFiberBending`], without a global registry.
#[derive(Clone, Copy, Debug, Default)]
pub struct RectangleFiberBendingFactory;

impl SectionFactory for RectangleFiberBendingFactory {
    type Params = RectangleFiberBendingParams;

    fn descriptor(&self) -> LawDescriptor {
        RECTANGLE_FIBER_BENDING
    }

    fn build(&self, dim: Dim, params: Self::Params) -> EResult<Box<dyn SectionLaw>> {
        Ok(Box::new(RectangleFiberBending::new(dim, params)?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Strip {
    area: Real,
    y: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Share {
    force: Real,
    moment: Real,
    axial: Real,
    couple: Real,
    bending: Real,
}

/// Rectangle of concrete strips and steel bars, D2 only.
///
/// e = [ε0, κz] and s = [N, Mz]. ε(y) = ε0 − κz y. N is the sum of σ A and
/// Mz is the sum of (−y) σ A, concrete strips first and then bars. The
/// tangent uses the same order. History is the steel histories concatenated
/// in bar order; concrete stores nothing.
///
/// Positive N is tension. Positive κz produces positive Mz on a symmetric
/// elastic section. The constant-axial search holds κz and solves ε0.
///
/// ```
/// use engine_contracts::SectionLaw;
/// use engine_core::Dim;
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
///             SteelBar { area: 1.0 / 1024.0, y: -0.1875 },
///             SteelBar { area: 1.0 / 1024.0, y: 0.1875 },
///         ],
///     },
/// )?;
/// let response = section.respond(&[0.0, 0.0], &section.initial_state())?;
/// assert_eq!(response.resultants, vec![0.0, 0.0]);
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct RectangleFiberBending {
    params: RectangleFiberBendingParams,
    concrete: KentPark,
    steel: SteelBilinearKinematic,
    strips: Vec<Strip>,
}

impl RectangleFiberBending {
    /// Validates the rectangle, builds the strips, and constructs both laws.
    ///
    /// # Errors
    /// `ENG-E-0010` for D3, a size that is not strictly positive, a strip or
    /// bar count outside the method domain, or a bar centroid outside the
    /// rectangle. `ENG-E-0008` for a nonfinite size or an overflowed product.
    /// Concrete and steel parameter errors propagate from those laws.
    pub fn new(dim: Dim, params: RectangleFiberBendingParams) -> EResult<Self> {
        match dim {
            Dim::D2 => Self::from_params(params),
            Dim::D3 => Err(EngineError::new(
                "ENG-E-0010",
                "D3 is outside rectangle.fiber_bending; torsion and a second curvature are not defined.",
            )),
        }
    }

    fn from_params(params: RectangleFiberBendingParams) -> EResult<Self> {
        let width = positive(params.width, "width (m)")?;
        let height = positive(params.height, "height (m)")?;
        if !(1..=MAX_STRIPS).contains(&params.strip_count) {
            return Err(EngineError::new(
                "ENG-E-0010",
                "strip_count must be from 1 to 64.",
            ));
        }
        if !(1..=MAX_BARS).contains(&params.bars.len()) {
            return Err(EngineError::new(
                "ENG-E-0010",
                "bar count must be from 1 to 32.",
            ));
        }
        let concrete = KentPark::new(
            params.concrete.fc,
            params.concrete.rho_s,
            params.concrete.b_double_prime,
            params.concrete.s_h,
        )?;
        let steel = SteelBilinearKinematic::new(params.steel.e, params.steel.fy, params.steel.b)?;
        let count = finite(params.strip_count as Real, "strip_count")?;
        let dy = positive(
            finite(height / count, "strip height (m)")?,
            "strip height (m)",
        )?;
        let half = finite(height / 2.0, "half height (m)")?;
        let mut strips = Vec::with_capacity(params.strip_count);
        for i in 0..params.strip_count {
            let index = finite((i as Real) + 0.5, "strip index")?;
            let y = finite(index * dy - half, "strip centroid y (m)")?;
            let area = positive(finite(width * dy, "strip area (m²)")?, "strip area (m²)")?;
            strips.push(Strip { area, y });
        }
        for bar in &params.bars {
            positive(bar.area, "steel area (m²)")?;
            let y = finite(bar.y, "steel y (m)")?;
            if y.abs() > half {
                return Err(EngineError::new(
                    "ENG-E-0010",
                    "steel y (m) must lie inside the rectangle.",
                ));
            }
        }
        Ok(Self {
            params,
            concrete,
            steel,
            strips,
        })
    }

    /// Original parameters, immutable.
    pub fn params(&self) -> &RectangleFiberBendingParams {
        &self.params
    }

    /// Moment at one curvature while the axial force stays near a target.
    ///
    /// κz and `committed` are held. ε0 starts at `axial_strain_guess` and
    /// follows ε0 ← ε0 − (N − N★) / (∂N/∂ε0). The derivative is the section
    /// tangent. The search returns when |N − N★| is at most `residual_limit`.
    ///
    /// # Errors
    /// `ENG-E-0008` for a nonfinite argument or an overflowed update.
    /// `ENG-E-0010` for a residual limit that is not strictly positive, an
    /// update count outside 1..=32, or a history the section rejects.
    /// `ENG-E-0006` when the axial tangent is zero and the residual is still
    /// above the limit. `ENG-E-0007` when the updates are exhausted.
    pub fn moment_at_constant_axial(
        &self,
        axial_force: Real,
        curvature: Real,
        axial_strain_guess: Real,
        committed: &[Real],
        residual_limit: Real,
        iteration_limit: usize,
    ) -> EResult<CurvaturePoint> {
        finite(axial_force, "target axial force (N)")?;
        finite(curvature, "κz (m⁻¹)")?;
        finite(axial_strain_guess, "axial strain guess")?;
        positive(residual_limit, "residual limit (N)")?;
        if !(1..=MAX_UPDATES).contains(&iteration_limit) {
            return Err(EngineError::new(
                "ENG-E-0010",
                "update count must be from 1 to 32.",
            ));
        }
        let mut axial_strain = axial_strain_guess;
        for _ in 0..iteration_limit {
            let response = self.respond(&[axial_strain, curvature], committed)?;
            let (axial, moment) = two_resultants(&response)?;
            let residual = finite(axial - axial_force, "axial residual (N)")?;
            if residual.abs() <= residual_limit {
                return Ok(CurvaturePoint {
                    axial_strain,
                    curvature,
                    axial_force: axial,
                    moment,
                    trial_state: response.trial_state,
                });
            }
            let stiffness = response
                .tangent
                .get(0, 0)
                .ok_or_else(|| EngineError::new("ENG-E-0012", "Axial tangent entry is missing."))?;
            if stiffness == 0.0 {
                return Err(EngineError::new(
                    "ENG-E-0006",
                    "Axial tangent is zero, so the axial strain is not determined.",
                ));
            }
            let step = finite(residual / stiffness, "axial strain increment")?;
            axial_strain = finite(axial_strain - step, "updated axial strain")?;
        }
        Err(EngineError::new(
            "ENG-E-0007",
            "Constant-axial curvature point did not meet the residual limit.",
        ))
    }
}

fn two_resultants(response: &SectionResponse) -> EResult<(Real, Real)> {
    if response.resultants.len() != 2 {
        return Err(EngineError::new(
            "ENG-E-0012",
            "Section returned the wrong number of resultants.",
        ));
    }
    Ok((response.resultants[0], response.resultants[1]))
}

fn fiber_strain(axial_strain: Real, curvature: Real, y: Real) -> EResult<Real> {
    let lever = finite(curvature * y, "κz·y")?;
    finite(axial_strain - lever, "fiber strain")
}

fn share(area: Real, y: Real, stress: Real, tangent: Real) -> EResult<Share> {
    let force = finite(stress * area, "fiber force (N)")?;
    let moment = finite((-y) * force, "fiber moment (N·m)")?;
    let axial = finite(tangent * area, "fiber axial stiffness (N)")?;
    let couple = finite((-y) * axial, "fiber coupling (N·m)")?;
    let bending = finite((-y) * couple, "fiber bending stiffness (N·m²)")?;
    Ok(Share {
        force,
        moment,
        axial,
        couple,
        bending,
    })
}

struct Sums {
    force: Real,
    moment: Real,
    axial: Real,
    couple: Real,
    bending: Real,
}

impl Sums {
    fn new() -> Self {
        Self {
            force: 0.0,
            moment: 0.0,
            axial: 0.0,
            couple: 0.0,
            bending: 0.0,
        }
    }

    fn add(&mut self, share: Share) -> EResult<()> {
        self.force = finite(self.force + share.force, "N (N)")?;
        self.moment = finite(self.moment + share.moment, "Mz (N·m)")?;
        self.axial = finite(self.axial + share.axial, "dN/dε0 (N)")?;
        self.couple = finite(self.couple + share.couple, "dN/dκz (N·m)")?;
        self.bending = finite(self.bending + share.bending, "dMz/dκz (N·m²)")?;
        Ok(())
    }
}

impl SectionLaw for RectangleFiberBending {
    fn kind(&self) -> &'static str {
        RECTANGLE_FIBER_BENDING.kind
    }

    fn contract(&self) -> ContractId {
        RECTANGLE_FIBER_BENDING.contract
    }

    fn dim(&self) -> Dim {
        Dim::D2
    }

    fn state_len(&self) -> usize {
        self.params.bars.len() * 2
    }

    fn initial_state(&self) -> Vec<Real> {
        vec![0.0; self.state_len()]
    }

    /// Integrates the fibers at total e without mutating the section.
    ///
    /// `ENG-E-0010` for an e or history length other than the one declared
    /// here. `ENG-E-0008` for a nonfinite deformation, history entry, or
    /// overflowed sum. Concrete and steel errors propagate.
    fn respond(&self, e: &[Real], committed: &[Real]) -> EResult<SectionResponse> {
        vector(e, 2, "Generalized deformations")?;
        vector(committed, self.state_len(), "Accepted history")?;
        let axial_strain = e[0];
        let curvature = e[1];
        let mut sums = Sums::new();
        let mut trial_state = Vec::with_capacity(self.state_len());
        for strip in &self.strips {
            let strain = fiber_strain(axial_strain, curvature, strip.y)?;
            let response = self.concrete.respond(strain, &[])?;
            sums.add(share(
                strip.area,
                strip.y,
                response.stress,
                response.tangent,
            )?)?;
        }
        for (index, bar) in self.params.bars.iter().enumerate() {
            let strain = fiber_strain(axial_strain, curvature, bar.y)?;
            let start = index * 2;
            let response = self.steel.respond(strain, &committed[start..start + 2])?;
            if response.trial_state.len() != 2 {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Steel history length is not 2.",
                ));
            }
            sums.add(share(bar.area, bar.y, response.stress, response.tangent)?)?;
            trial_state.extend(response.trial_state);
        }
        let tangent = vec![sums.axial, sums.couple, sums.couple, sums.bending];
        Ok(SectionResponse {
            resultants: vec![sums.force, sums.moment],
            tangent: DMat::new(2, 2, tangent)?,
            trial_state,
        })
    }
}
