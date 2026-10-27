//! Original static lateral curves: `py.matlock_soft_clay`, `py.reese_sand`,
//! `py.user`. Unprefixed SI: y, x, d in m; p in N/m; c, σ'v in Pa;
//! γ' and k in N/m³. p has the sign of y; soil reaction is −p.
//!
//! Sources: Matlock (1970), OTC 1204, Eqs. 1–5 and Fig. 5a; Reese, Cox, and Koop
//! (1974), OTC 2080, Eqs. 4–9 and 11–16, steps 2 and 4–8. J2 specification and
//! original derivations: `provenance/ENG-GEO-003.md`. Cyclic loading is not included.
//!
//! The profile supplies only σ'v through [`effective_stress_pa`]; it does not store c,
//! εc, φ, k, A, or B. These data and the geometry are explicit and required.
//! Reese assumes constant γ' and φ to determine Xt; γ' is not replaced by
//! σ'v/x in a variable profile. Matlock accepts σ'v from a variable profile.
//!
//! [`PyCurve::points`] contains tuples that can be consumed numerically by
//! `engine-laws::uniaxial::MultilinearElastic`. Its physical contract is ε/σ:
//! to preserve units, the future pile adapter must normalize
//! ε=y/L and σ=p·L/area, with explicit L and area, and recover p=σ·area/L.
//! Passing (y,p) directly only reuses interpolation, not its units.
//! The law extends the end segments: generated curves end in
//! plateaus; `py.user` retains its ends and [`PyCurve::at`] rejects queries
//! outside the table. The consumer must explicitly choose its extrapolation.
//!
//! ```
//! use engine_geo::py::{matlock_soft_clay, MatlockParams, Sampling};
//! let soil = MatlockParams {
//!     depth_m: Some(2.0), diameter_m: Some(1.0),
//!     undrained_strength_pa: Some(20_000.0),
//!     effective_vertical_stress_pa: Some(20_000.0),
//!     j: Some(0.5), epsilon_50: Some(0.01),
//! };
//! let sampling = Sampling {
//!     positive_y_m: vec![0.003125, 0.025], plateau_y_m: Some(0.4),
//! };
//! let result = matlock_soft_clay(&soil, &sampling)?;
//! assert_eq!(result.curve.at(0.025)?, 50_000.0);
//! # Ok::<(), engine_core::EngineError>(())
//! ```

use crate::StressProfile;
use engine_core::math;
use engine_core::{EResult, EngineError, Real};

/// Internal identity of the static soft-clay generator.
pub const MATLOCK_SOFT_CLAY: &str = "py.matlock_soft_clay";
/// Internal identity of the static sand generator.
pub const REESE_SAND: &str = "py.reese_sand";
/// Internal identity of the user-supplied table.
pub const USER: &str = "py.user";

fn invalid(field: &str) -> EngineError {
    EngineError::new(
        "ENG-E-0010",
        format!("p-y: {field}: outside the domain or numerically unrepresentable geometry."),
    )
}
fn finite(value: Real, field: &str) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            format!("p-y: {field}: non-finite value or overflow."),
        ))
    }
}
fn positive(value: Real, field: &str) -> EResult<Real> {
    finite(value, field)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(invalid(field))
    }
}
fn nonnegative(value: Real, field: &str) -> EResult<Real> {
    finite(value, field)?;
    if value >= 0.0 {
        Ok(value)
    } else {
        Err(invalid(field))
    }
}
fn required(value: Option<Real>, field: &str) -> EResult<Real> {
    value.ok_or_else(|| {
        EngineError::new(
            "ENG-E-0003",
            format!("p-y: missing {field}; provide the parameter explicitly."),
        )
    })
}
fn pos(value: Option<Real>, field: &str) -> EResult<Real> {
    positive(required(value, field)?, field)
}

/// Converts σ'v from `stress-profile/1` (kPa) to Pa, without clipping or extrapolation.
/// Propagates the code, entities, and context of the profile diagnostic. The caller
/// retains its provenance report through `StressProfile::stresses_at`.
/// Rejects negative or nonfinite stresses and conversion overflow.
pub fn effective_stress_pa(profile: &dyn StressProfile, depth_m: Real) -> EResult<Real> {
    nonnegative(depth_m, "x")?;
    let stress = profile.stresses_at(depth_m).map_err(|d| EngineError {
        code: d.code,
        message: format!("{}; {:?}", d.message, d.hint),
        entities: d.entities,
    })?;
    nonnegative(stress.effective_vertical_kpa * 1000.0, "σ'v (Pa)")
}

/// Immutable, bilateral, odd table. Includes (0,0), with nonnegative slopes.
#[derive(Clone, Debug)]
pub struct PyCurve {
    kind: &'static str,
    points: Vec<(Real, Real)>,
}
impl PyCurve {
    /// Generator identity.
    pub fn kind(&self) -> &'static str {
        self.kind
    }
    /// Ordered points (y (m), p (N/m)); includes both branches and the origin.
    pub fn points(&self) -> &[(Real, Real)] {
        &self.points
    }
    /// Transfers the table for law adapters, without changing units.
    pub fn into_points(self) -> Vec<(Real, Real)> {
        self.points
    }
    /// Secant interpolation bounded by the tabulated interval; never extrapolates.
    /// ENG-E-0010 outside the table; ENG-E-0008 for nonfinite query/result.
    pub fn at(&self, y_m: Real) -> EResult<Real> {
        finite(y_m, "y")?;
        let first = self.points.first().ok_or_else(|| invalid("empty table"))?;
        let last = self.points.last().ok_or_else(|| invalid("empty table"))?;
        if y_m < first.0 || y_m > last.0 {
            return Err(invalid("y is outside the table"));
        }
        let i = self.points.partition_point(|&(y, _)| y < y_m);
        let &(y1, p1) = self
            .points
            .get(i)
            .ok_or_else(|| invalid("y is outside the table"))?;
        if y1 == y_m {
            return Ok(p1);
        }
        let &(y0, p0) = self
            .points
            .get(i.saturating_sub(1))
            .ok_or_else(|| invalid("tramo"))?;
        finite(p0 + (p1 - p0) * ((y_m - y0) / (y1 - y0)), "interpolated p")
    }
}

/// Validates the positive branch, then reflects without duplicating the origin.
fn curve(kind: &'static str, points: Vec<(Real, Real)>) -> EResult<PyCurve> {
    if points.len() < 2 || points.first() != Some(&(0.0, 0.0)) {
        return Err(invalid(
            "origin and at least one positive point are required",
        ));
    }
    for &(y, p) in &points {
        nonnegative(y, "tabulated y")?;
        nonnegative(p, "tabulated p")?;
    }
    for pair in points.windows(2) {
        let dy = positive(pair[1].0 - pair[0].0, "strictly positive Δy")?;
        let dp = nonnegative(pair[1].1 - pair[0].1, "nondecreasing Δp")?;
        let slope = finite(dp / dy, "secant slope")?;
        if dp > 0.0 && slope == 0.0 {
            return Err(invalid("slope underflow"));
        }
    }
    // Differences in the bilateral table must also be representable.
    if let Some(&(y, p)) = points.last() {
        finite(2.0 * y, "bilateral y range")?;
        finite(2.0 * p, "bilateral p range")?;
    }
    let mut both: Vec<_> = points
        .iter()
        .skip(1)
        .rev()
        .map(|&(y, p)| (-y, -p))
        .collect();
    both.extend(points);
    Ok(PyCurve { kind, points: both })
}

/// Constructs `py.user` from the y≥0 branch. Requires the origin, at least two finite
/// points, strictly increasing y, and nondecreasing p≥0. Does not sort, add
/// plateaus, or extrapolate; reflects the user's table exactly.
pub fn user(points: Vec<(Real, Real)>) -> EResult<PyCurve> {
    curve(USER, points)
}

/// Explicit sampling: additional positive positions and plateau end.
/// Adds the origin, method vertices, and their reflections. No default mesh
/// is used. Interpolation between samples approximates the analytical curve.
#[derive(Clone, Debug)]
pub struct Sampling {
    /// Finite, strictly increasing y>0; may be empty. Must not exceed the plateau.
    pub positive_y_m: Vec<Real>,
    /// Required, strictly greater than 8yc (Matlock) or yu (Reese).
    pub plateau_y_m: Option<Real>,
}
fn sample(
    sampling: &Sampling,
    end: Real,
    landmarks: &[Real],
    kind: &'static str,
    evaluate: impl Fn(Real) -> EResult<Real>,
) -> EResult<PyCurve> {
    let plateau = pos(sampling.plateau_y_m, "plateau y")?;
    if plateau <= end {
        return Err(invalid("plateau must exceed the onset of yielding"));
    }
    let mut prev = 0.0;
    for &y in &sampling.positive_y_m {
        positive(y, "sampling y")?;
        if y <= prev || y > plateau {
            return Err(invalid("sampling order/range"));
        }
        prev = y;
    }
    let mut ys = vec![0.0, plateau];
    ys.extend_from_slice(&sampling.positive_y_m);
    for &y in landmarks {
        positive(y, "sampling vertex")?;
        if y > plateau {
            return Err(invalid("vertex is outside the sampling range"));
        }
        ys.push(y);
    }
    ys.sort_by(Real::total_cmp);
    ys.dedup();
    let points = ys
        .into_iter()
        .map(|y| evaluate(y).map(|p| (y, p)))
        .collect::<EResult<Vec<_>>>()?;
    curve(kind, points)
}

/// Required static Matlock parameters, without assumed values.
#[derive(Clone, Debug)]
pub struct MatlockParams {
    /// Depth x≥0 (m).
    pub depth_m: Option<Real>,
    /// Diameter d>0 (m).
    pub diameter_m: Option<Real>,
    /// Undrained strength c>0 (Pa).
    pub undrained_strength_pa: Option<Real>,
    /// Effective overburden σ'v≥0 (Pa), explicit or from [`effective_stress_pa`].
    pub effective_vertical_stress_pa: Option<Real>,
    /// Dimensionless J>0, required. OTC 1204 mentions 0.5 and 0.25 depending on site.
    pub j: Option<Real>,
    /// Dimensionless εc>0 at half the maximum stress (ε50); without imposing the
    /// indicative OTC 1204 interval 0.005–0.020 as a universal limit.
    pub epsilon_50: Option<Real>,
}
/// Matlock curve and control quantities (OTC 1204).
#[derive(Clone, Debug)]
pub struct MatlockCurve {
    /// Bilateral table in SI.
    pub curve: PyCurve,
    /// Np from Eq. 2/3, limited to 9.
    pub np: Real,
    /// pu from Eq. 1 (N/m).
    pub ultimate_n_m: Real,
    /// yc from Eq. 5 (m).
    pub yc_m: Real,
}
/// Matlock (1970), OTC 1204, Eqs. 1–3 and 5, static curve Fig. 5a.
/// Applies min(Np,9) pointwise, even with a variable profile. Eq. 4 is
/// informative for constant soil: xr=6d/(γ'd/c+J), not needed here.
/// Always includes 8yc and an explicit subsequent plateau. At the origin, the
/// analytical derivative is infinite: the first segment is secant to the first y>0.
/// ENG-E-0003 for missing data; ENG-E-0010 for invalid domain/sampling/precision;
/// ENG-E-0008 for nonfinite inputs or results. Does not accept cyclic loading.
pub fn matlock_soft_clay(params: &MatlockParams, sampling: &Sampling) -> EResult<MatlockCurve> {
    let x = nonnegative(required(params.depth_m, "x")?, "x")?;
    let d = pos(params.diameter_m, "d")?;
    let c = pos(params.undrained_strength_pa, "c")?;
    let stress = nonnegative(required(params.effective_vertical_stress_pa, "σ'v")?, "σ'v")?;
    let j = pos(params.j, "J")?;
    let epsilon = pos(params.epsilon_50, "ε50")?;
    let np = finite(3.0 + stress / c + j * (x / d), "uncapped Np")?.min(9.0);
    let pu = positive(np * c * d, "pu")?;
    let yc = positive(2.5 * epsilon * d, "yc")?;
    let end = positive(8.0 * yc, "8yc")?;
    let curve = sample(sampling, end, &[end], MATLOCK_SOFT_CLAY, |y| {
        if y >= end {
            Ok(pu)
        } else {
            finite(pu * 0.5 * math::cbrt(y / yc), "p Matlock")
        }
    })?;
    Ok(MatlockCurve {
        curve,
        np,
        ultimate_n_m: pu,
        yc_m: yc,
    })
}

/// Required static Reese parameters for homogeneous sand, without tables.
#[derive(Clone, Debug)]
pub struct ReeseParams {
    /// Depth x≥0 (m); x=0 produces a zero curve after parameter validation.
    pub depth_m: Option<Real>,
    /// Diameter b>0 (m).
    pub diameter_m: Option<Real>,
    /// φ in radians, 0<φ<π/2.
    pub friction_angle_rad: Option<Real>,
    /// Constant γ'>0 (N/m³); not implicitly obtained from the profile.
    pub effective_unit_weight_n_m3: Option<Real>,
    /// α in radians, 0<α<π/2. Step 2 proposes α=φ/2; supplied explicitly.
    pub alpha_rad: Option<Real>,
    /// K0>0. Step 2 sets 0.4 as the method value; supplied explicitly.
    pub k0: Option<Real>,
    /// A>0, factor for pu (Eq. 9); Fig. 8, supplied by the user.
    pub a: Option<Real>,
    /// B>0, factor for pm (Eq. 11); Fig. 9, supplied by the user.
    pub b: Option<Real>,
    /// k>0 (N/m³) from Eq. 4; the paper's table is not copied.
    pub subgrade_modulus_n_m3: Option<Real>,
}
/// Reese curve and controls (OTC 2080), unprefixed SI.
#[derive(Clone, Debug)]
pub struct ReeseCurve {
    /// Bilateral table with k, m, u, and plateau (at the surface only m, u, and plateau).
    pub curve: PyCurve,
    /// Shallow pct from Eq. 5 (N/m).
    pub pct_n_m: Real,
    /// Deep pcd from Eq. 8 (N/m).
    pub pcd_n_m: Real,
    /// Xt>0 (m), nontrivial intersection of Eq. 5/8.
    pub transition_depth_m: Real,
    /// pu from Eq. 9 (N/m).
    pub ultimate_n_m: Real,
    /// pm from Eq. 11 (N/m).
    pub pm_n_m: Real,
    /// ym=b/60 (m), step 7.
    pub ym_m: Real,
    /// yu=3b/80 (m), step 6.
    pub yu_m: Real,
    /// yk from Eq. 16 (m); None only for x=0 (zero curve).
    pub yk_m: Option<Real>,
    /// n>1 from Eq. 14, also validated at x=0 through its pc-independent form.
    pub n: Real,
}
/// Reese, Cox, and Koop (1974), OTC 2080, steps 2 and 4–8, Eqs. 4–9 and 11–16.
/// β=π/4+φ/2 and Ka=tan²(π/4−φ/2) (Eq. 6/7). Xt is found by dividing
/// pct−pcd by γ'H: the remaining function is affine. Requires positive slope
/// and negative constant term;
/// hence there is a unique positive root. Brackets from b by doubling up to 128
/// times and bisects 96 times, in a set order. Requires final width
/// ≤8·EPSILON·upper bound; an unresolved root is a typed error.
/// Above Xt, uses pct; below, pcd (step 4).
///
/// Rejects A≤B, n≤1, yk≥ym, and unrepresentable scales; no alternative
/// branch is invented. For x=0, Eqs. 5/8 give p=0 throughout the table; yk is not
/// computed (division by zero). All data remain required.
/// The parabola is evaluated as pm·(y/ym)^(1/n), a form of Eq. 12/15 that avoids
/// dimensional powers. yk=ym·[pm/(k·x·ym)]^(n/(n−1)) derives from Eq. 16.
/// Errors: ENG-E-0003 missing data, ENG-E-0010 domain/degeneracy/sampling,
/// ENG-E-0008 nonfinite values/overflow. Static loading only.
pub fn reese_sand(params: &ReeseParams, sampling: &Sampling) -> EResult<ReeseCurve> {
    let x = nonnegative(required(params.depth_m, "x")?, "x")?;
    let diameter = pos(params.diameter_m, "b diameter")?;
    let phi = pos(params.friction_angle_rad, "φ")?;
    let alpha = pos(params.alpha_rad, "α")?;
    if phi >= std::f64::consts::FRAC_PI_2 || alpha >= std::f64::consts::FRAC_PI_2 {
        return Err(invalid("φ and α must be less than π/2"));
    }
    let gamma = pos(params.effective_unit_weight_n_m3, "γ'")?;
    let k0 = pos(params.k0, "K0")?;
    let a = pos(params.a, "A")?;
    let b = pos(params.b, "B")?;
    let k = pos(params.subgrade_modulus_n_m3, "k")?;
    if a <= b {
        return Err(invalid("A must exceed B"));
    }
    let ym = positive(diameter / 60.0, "ym")?;
    let yu = positive(3.0 * diameter / 80.0, "yu")?;
    let n = positive((b / (a - b)) * ((yu - ym) / ym), "n")?;
    if n <= 1.0 {
        return Err(invalid("n must exceed 1"));
    }
    let beta = std::f64::consts::FRAC_PI_4 + phi / 2.0;
    let tb = math::tan(beta);
    let tf = math::tan(phi);
    let ta = math::tan(alpha);
    let divisor = math::tan(beta - phi);
    let ka = math::powi(math::tan(std::f64::consts::FRAC_PI_4 - phi / 2.0), 2);
    // Eq. 5 / (γ'H) = q1 H + q0; Eq. 8 / (γ'H) = deep.
    let q1 = positive(
        k0 * tf * math::sin(beta) / (divisor * math::cos(alpha))
            + tb * tb * ta / divisor
            + k0 * tb * (tf * math::sin(beta) - ta),
        "linear pct coefficient",
    )?;
    let q0 = positive(diameter * (tb / divisor - ka), "constant pct coefficient")?;
    let deep = positive(
        ka * diameter * (math::powi(tb, 8) - 1.0) + k0 * diameter * tf * math::powi(tb, 4),
        "pcd coefficient",
    )?;
    let offset = positive(deep - q0, "positive pct/pcd intersection")?;
    let residual = |h: Real| finite(q1 * h - offset, "Xt residual");
    let mut hi = diameter;
    let mut bracketed = false;
    for _ in 0..128 {
        if residual(hi)? >= 0.0 {
            bracketed = true;
            break;
        }
        hi = positive(hi * 2.0, "Xt bound")?;
    }
    if !bracketed {
        return Err(invalid("Xt has no bound after 128 doublings"));
    }
    let mut lo = 0.0;
    for _ in 0..96 {
        let mid = lo + (hi - lo) * 0.5;
        if residual(mid)? >= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    if hi - lo > 8.0 * Real::EPSILON * hi {
        return Err(invalid(
            "Xt was not resolved to binary64 relative precision in 96 bisections",
        ));
    }
    let xt = positive(lo + (hi - lo) * 0.5, "Xt")?;
    let pct = nonnegative(gamma * x * (q1 * x + q0), "pct")?;
    let pcd = nonnegative(gamma * x * deep, "pcd")?;
    let pc = if x <= xt { pct } else { pcd };
    let pu = nonnegative(a * pc, "pu")?;
    let pm = nonnegative(b * pc, "pm")?;
    let mut landmarks = vec![ym, yu];
    let (yk, initial) = if x == 0.0 {
        (None, 0.0)
    } else {
        positive(pc, "pc")?;
        positive(pu, "pu")?;
        positive(pm, "pm")?;
        let initial = positive(k * x, "k·x")?;
        let ratio = positive(pm / positive(initial * ym, "k·x·ym")?, "pm/(k·x·ym)")?;
        let yk = positive(ym * math::powf(ratio, n / (n - 1.0)), "yk")?;
        if yk >= ym {
            return Err(invalid("yk must be less than ym; increase k or check A/B"));
        }
        landmarks.push(yk);
        (Some(yk), initial)
    };
    let curve = sample(sampling, yu, &landmarks, REESE_SAND, |y| {
        if x == 0.0 || y == 0.0 {
            return Ok(0.0);
        }
        if y >= yu {
            return Ok(pu);
        }
        if y >= ym {
            return finite(pm + (pu - pm) * ((y - ym) / (yu - ym)), "m-u line");
        }
        if yk.is_some_and(|yk| y <= yk) {
            return finite(initial * y, "initial line");
        }
        finite(pm * math::powf(y / ym, 1.0 / n), "Reese parabola")
    })?;
    Ok(ReeseCurve {
        curve,
        pct_n_m: pct,
        pcd_n_m: pcd,
        transition_depth_m: xt,
        ultimate_n_m: pu,
        pm_n_m: pm,
        ym_m: ym,
        yu_m: yu,
        yk_m: yk,
        n,
    })
}
