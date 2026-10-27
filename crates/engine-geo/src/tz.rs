//! Local axial t-z and q-z curves, in unprefixed SI (m, Pa, N).
//!
//! `z > 0` means the pile moves downward relative to the soil (compression).
//! Ordinates are mobilized resistances with that sign; soil action on
//! the pile has the opposite sign. These are not geostatic stresses.
//!
//! # Source and scope
//!
//! API RP 2GEO, *Geotechnical and Foundation Design Considerations*, first
//! edition, April 2011, Addendum 1, October 2014, reaffirmed in 2021
//! (ISO 19901-4:2003 modified): §8.4.2, Figure 2 and Table 2 for t-z;
//! §8.4.3, Figure 3 for q-z. Only the numerical vertices supplied in
//! the J2 specification are implemented; text and figures are not reproduced.
//! Linear interpolation between vertices and shape reflection in tension
//! with an independent scale are original derivations, not rules attributed
//! to the source. See `provenance/ENG-GEO-004.md`.
//!
//! # Usage and errors
//!
//! Tables are immutable, include (0, 0), and have no default mesh or
//! parameters. The evaluator extends end segments, like the
//! multilinear elastic law in engine-laws (ENG-MAT-002); the APIs close both
//! ends with plateaus. User tables control their extrapolation.
//! Adapters convert to (ε, σ), positive in tension, for
//! `MultilinearElastic::new(points, MultilinearMode::Both)`, without depending
//! on engine-laws. This law rejects negative slopes: the adapter returns
//! `ENG-E-0010` for softening, although the generator preserves it.
//! No history, axial bar, pile, or structural equilibrium is implemented.
//!
//! `ENG-E-0003`: missing required parameter; `ENG-E-0010`: domain, order,
//! origin, underflow, or incompatibility with the law;
//! `ENG-E-0008`: nonfinite input or calculation. Messages identify the datum.
//!
//! # Extensions
//!
//! Identity `tz.coyle_reese_clay` remains **unimplemented — pending the
//! original source**, Coyle & Reese (1966), ASCE JSMFD. In this tranche, only
//! user points can be entered through [`user`]; there is no function or
//! constants for that method. Reese/Isenhower/Wang (2006) was not used.

use engine_core::{EResult, EngineError, Real};

/// Required data for `tz.api_sand` and `tz.api_clay`, without substitutions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApiTzParams {
    /// Maximum unit friction in compression, Pa, strictly positive.
    /// The user must compute it according to §8.1; equations 17–21 pending.
    pub tmax_compression_pa: Option<Real>,
    /// Maximum unit friction in tension ft(z), Pa, strictly positive.
    /// May match the compression value, only by user choice.
    pub tmax_tension_pa: Option<Real>,
    /// Peak displacement, m, strictly positive, supplied by the user.
    /// §8.4.2 recommends zpeak/D = 0.01 for routine design and mentions
    /// 0.0025–0.02; none of those values is adopted or imposed here.
    pub zpeak_m: Option<Real>,
    /// Positive plateau end, m; also closes the table at −z.
    /// Must exceed zpeak for sand and 2·zpeak for clay.
    pub plateau_z_m: Option<Real>,
}

/// Required choice of the negative branch in user tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserTraction {
    /// Negative and positive points are supplied, along with (0, 0).
    /// Requires resistance ≤ 0 if z < 0 and resistance ≥ 0 if z > 0.
    /// Allows independent scales and negative branches with zero resistance.
    Explicit,
    /// Only (0, 0) and positive points are supplied; adds (−z, −resistance).
    Symmetric,
}

/// Immutable table (z in m, t in Pa); allows post-peak softening.
#[derive(Clone, Debug, PartialEq)]
pub struct TzCurve(Table);

impl TzCurve {
    /// Ordered vertices; read-only, including origin and declared branches.
    pub fn points(&self) -> &[(Real, Real)] {
        &self.0.points
    }

    /// Interpolated unit resistance, Pa; extends end segments.
    /// Numerical errors as documented in the module.
    pub fn at(&self, z_m: Real) -> EResult<Real> {
        self.0.at(z_m)
    }

    /// Adapts to the uniaxial law: ε = −z/L, σ = −t (Pa); reverses the order.
    /// L (m) is an explicit, positive, finite reference length;
    /// D is not used and no element length is assigned implicitly.
    /// This represents the unit interface: the future pile must
    /// integrate t over its surface to obtain force (N).
    /// Rejects softening with `ENG-E-0010`, without modifying the generator.
    pub fn multilinear_points(
        &self,
        reference_length_m: Option<Real>,
    ) -> EResult<Vec<(Real, Real)>> {
        self.0.multilinear_points(reference_length_m, 1.0)
    }
}

/// Sand `tz.api_sand`: API RP 2GEO §8.4.2, Figure 2 and Table 2.
///
/// Tabulated rise and plateau t/tmax = 1 from zpeak. Linear interpolation
/// and negative branch with equal shape and scale tmax_tension: original derivation.
/// Requires all fields of [`ApiTzParams`]; errors as in the module.
pub fn api_sand(params: ApiTzParams) -> EResult<TzCurve> {
    api_tz(params, None)
}

/// Clay `tz.api_clay`: API RP 2GEO §8.4.2, Figure 2 and Table 2.
///
/// `residual_ratio` is required, 0 < tres/tmax ≤ 1. The source specifies
/// 0.70–0.90, but that interval is not imposed here. Sets zres = 2·zpeak
/// according to Table 2; other zres allowed by the source are out of scope.
/// Linear interpolation between peak and residual (original derivation);
/// followed by a plateau. The negative branch reflects the shape, with scale ft(z).
/// tmax and zpeak are supplied by the user, see [`ApiTzParams`].
pub fn api_clay(params: ApiTzParams, residual_ratio: Option<Real>) -> EResult<TzCurve> {
    let ratio = positive(residual_ratio, "tres/tmax")?;
    if ratio > 1.0 {
        return Err(domain("tres/tmax must be ≤ 1."));
    }
    api_tz(params, Some(ratio))
}

/// `tz.user` table, without sorting, correcting, or imposing monotonic t.
///
/// Includes (0, 0), strictly increasing z, and finite values. Tension is
/// chosen explicitly; see [`UserTraction`]. Negative slopes are allowed:
/// current engine-laws does not support them and the adapter diagnoses them.
/// `None` gives `ENG-E-0003`; other errors as documented in the module.
pub fn user(points: Option<Vec<(Real, Real)>>, traction: UserTraction) -> EResult<TzCurve> {
    Ok(TzCurve(user_table(points, traction)?))
}

// Numerical vertices of §8.4.2 Table 2, supplied by J2.
const ASCENDING: [(Real, Real); 6] = [
    (0.0, 0.0),
    (0.16, 0.30),
    (0.31, 0.50),
    (0.57, 0.75),
    (0.80, 0.90),
    (1.0, 1.0),
];

fn api_tz(params: ApiTzParams, residual: Option<Real>) -> EResult<TzCurve> {
    let compression = positive(params.tmax_compression_pa, "tmax_compression_pa")?;
    let tension = positive(params.tmax_tension_pa, "tmax_tension_pa")?;
    let peak = positive(params.zpeak_m, "zpeak_m")?;
    let plateau = positive(params.plateau_z_m, "plateau_z_m")?;
    let last_z = finite(peak * if residual.is_some() { 2.0 } else { 1.0 }, "zres_m")?;
    if plateau <= last_z {
        return Err(domain("plateau_z_m must exceed the last standard vertex."));
    }
    let mut shape: Vec<_> = ASCENDING
        .iter()
        .map(|&(z, t)| Ok((product(z, peak, "vertex z")?, t)))
        .collect::<EResult<_>>()?;
    let end_ratio = match residual {
        Some(ratio) => {
            shape.push((last_z, ratio));
            ratio
        }
        None => 1.0, // Sand plateau, not an omitted parameter.
    };
    shape.push((plateau, end_ratio));
    let mut points: Vec<_> = shape
        .iter()
        .skip(1)
        .rev()
        .map(|&(z, t)| Ok((-z, -product(t, tension, "t in tension")?)))
        .collect::<EResult<_>>()?;
    for &(z, t) in &shape {
        points.push((z, product(t, compression, "t in compression")?));
    }
    Ok(TzCurve(Table::new(points)?))
}

/// Tip q-z curves: API RP 2GEO §8.4.3, Figure 3, or explicit table.
/// Also exported as `engine_geo::qz`.
pub mod qz {
    use super::*;

    /// Required parameters of `qz.api`, in unprefixed SI.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ApiQzParams {
        /// Tip capacity Qp (N), positive; calculation from §8.1 is out of scope.
        pub qp_n: Option<Real>,
        /// Diameter D (m), strictly positive.
        pub diameter_m: Option<Real>,
        /// Plateau end z (m), strictly greater than 0.10·D.
        pub plateau_z_m: Option<Real>,
    }

    /// Immutable table (z in m, Q in N), dimensionally distinct from t-z.
    #[derive(Clone, Debug, PartialEq)]
    pub struct QzCurve(Table);

    impl QzCurve {
        /// Ordered vertices, including (0, 0) and the declared negative branch.
        pub fn points(&self) -> &[(Real, Real)] {
            &self.0.points
        }

        /// Interpolated tip resistance, N; extends end segments.
        /// For [`api`], Q = 0 for all z < 0 and Q = Qp for z ≥ 0.10·D.
        pub fn at(&self, z_m: Real) -> EResult<Real> {
            self.0.at(z_m)
        }

        /// Adapts to (ε, σ): ε = −z/L and σ = −Q/A, reversing the order.
        /// L (m) and A (m²) are required, positive, and finite; Q (N) is not
        /// identified with σ (Pa). The gap of [`api`] is included in
        /// the table, with compression ε < 0. For [`api`], `CompressionOnly`
        /// retains the compression-side tangent at the origin; `Both`
        /// gives the same resistances but zero tangent at the origin (right-hand
        /// gap). For bilateral user tables, uses `Both`.
        /// Rejects negative slopes with `ENG-E-0010`.
        pub fn multilinear_points(
            &self,
            reference_length_m: Option<Real>,
            reference_area_m2: Option<Real>,
        ) -> EResult<Vec<(Real, Real)>> {
            let area = positive(reference_area_m2, "reference_area_m2")?;
            self.0.multilinear_points(reference_length_m, area)
        }
    }

    /// `qz.api`: API RP 2GEO §8.4.3, Figure 3.
    ///
    /// Original linear interpolation between vertices Q/Qp versus z/D, plateau
    /// from 0.10·D and explicit subsequent closure. Adds (−zmeseta, 0):
    /// tip without tension for z < 0, original contact idealization.
    /// Qp and D are not computed; all parameters come from the user.
    pub fn api(params: ApiQzParams) -> EResult<QzCurve> {
        let qp = positive(params.qp_n, "qp_n")?;
        let diameter = positive(params.diameter_m, "diameter_m")?;
        let plateau = positive(params.plateau_z_m, "plateau_z_m")?;
        let last = finite(0.1 * diameter, "0.10·D")?;
        if plateau <= last {
            return Err(domain("plateau_z_m must exceed 0.10·D."));
        }
        // Numerical vertices of §8.4.3 Figure 3, supplied by J2.
        let normalized = [
            (0.0, 0.0),
            (0.002, 0.25),
            (0.013, 0.50),
            (0.042, 0.75),
            (0.073, 0.90),
            (0.100, 1.00),
        ];
        let mut points = vec![(-plateau, 0.0)];
        for (z, q) in normalized {
            points.push((
                product(z, diameter, "q-z vertex z")?,
                product(q, qp, "vertex Q")?,
            ));
        }
        points.push((plateau, qp));
        Ok(QzCurve(Table::new(points)?))
    }

    /// `qz.user`: explicit (z, Q), same requirements as [`super::user`].
    /// The negative branch is explicit or optionally symmetric; can express
    /// Q = 0 through negative points with zero ordinate. Unlike
    /// [`api`], no gap is imposed on user data.
    pub fn user(points: Option<Vec<(Real, Real)>>, traction: UserTraction) -> EResult<QzCurve> {
        Ok(QzCurve(user_table(points, traction)?))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Table {
    points: Vec<(Real, Real)>,
}

impl Table {
    fn new(points: Vec<(Real, Real)>) -> EResult<Self> {
        for &(z, value) in &points {
            finite(z, "vertex z (m)")?;
            finite(value, "vertex resistance")?;
        }
        if points.len() < 2 || !points.contains(&(0.0, 0.0)) {
            return Err(domain(
                "The table requires at least two points and the origin (0, 0).",
            ));
        }
        for pair in points.windows(2) {
            let ((z0, v0), (z1, v1)) = (pair[0], pair[1]);
            if z1 <= z0 {
                return Err(domain("z must be strictly increasing; data is not sorted."));
            }
            let dz = finite(z1 - z0, "span Δz")?;
            let dv = finite(v1 - v0, "span Δresistance")?;
            let slope = finite(dv / dz, "span slope")?;
            if dv != 0.0 && slope == 0.0 {
                return Err(domain("Nonzero slope reduced to zero by underflow."));
            }
        }
        Ok(Self { points })
    }

    fn at(&self, z: Real) -> EResult<Real> {
        finite(z, "evaluation z (m)")?;
        let upper = self.points.partition_point(|&(x, _)| x <= z);
        let i = upper.saturating_sub(1).min(self.points.len() - 2);
        let ((z0, v0), (z1, v1)) = (self.points[i], self.points[i + 1]);
        if z == z0 || v0 == v1 {
            return Ok(v0);
        }
        if z == z1 {
            return Ok(v1);
        }
        let slope = finite((v1 - v0) / (z1 - z0), "evaluation slope")?;
        let (anchor_z, anchor_v) = if z <= z0 * 0.5 + z1 * 0.5 {
            (z0, v0)
        } else {
            (z1, v1)
        };
        let dz = finite(z - anchor_z, "evaluation Δz")?;
        let dv = finite(slope * dz, "resistance increment")?;
        finite(anchor_v + dv, "interpolated/extrapolated resistance")
    }

    fn multilinear_points(&self, length: Option<Real>, area: Real) -> EResult<Vec<(Real, Real)>> {
        let length = positive(length, "reference_length_m")?;
        if self.points.windows(2).any(|p| p[1].1 < p[0].1) {
            return Err(domain("The ENG-MAT-002 elastic multilinear law does not support softening (negative slope)."));
        }
        let points: Vec<_> = self
            .points
            .iter()
            .rev()
            .map(|&(z, v)| (-z / length, -v / area))
            .collect();
        // Revalidates order, finiteness, and slopes after changing scale and sign.
        let table = Self::new(points)?;
        for (&(_, v), &(_, scaled)) in self.points.iter().rev().zip(&table.points) {
            if v != 0.0 && scaled == 0.0 {
                return Err(domain("Resistance reduced to zero when adapting units."));
            }
        }
        Ok(table.points)
    }
}

fn user_table(points: Option<Vec<(Real, Real)>>, traction: UserTraction) -> EResult<Table> {
    let mut table = Table::new(points.ok_or_else(|| missing("points"))?)?;
    match traction {
        UserTraction::Explicit => {
            if !table.points.iter().any(|&(z, _)| z < 0.0)
                || !table.points.iter().any(|&(z, _)| z > 0.0)
            {
                return Err(domain(
                    "The Explicit option requires negative and positive points.",
                ));
            }
            if table
                .points
                .iter()
                .any(|&(z, v)| (z < 0.0 && v > 0.0) || (z > 0.0 && v < 0.0))
            {
                return Err(domain(
                    "The Explicit option requires resistance ≤ 0 for z < 0 and ≥ 0 for z > 0.",
                ));
            }
        }
        UserTraction::Symmetric => {
            if table.points[0] != (0.0, 0.0) {
                return Err(domain(
                    "The Symmetric option requires only the origin and positive points.",
                ));
            }
            let mut reflected: Vec<_> = table
                .points
                .iter()
                .skip(1)
                .rev()
                .map(|&(z, v)| (-z, -v))
                .collect();
            reflected.append(&mut table.points);
            table = Table::new(reflected)?;
        }
    }
    Ok(table)
}

fn missing(name: &str) -> EngineError {
    EngineError::new("ENG-E-0003", format!("Missing required parameter {name}."))
}

fn domain(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}

fn finite(value: Real, name: &str) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            format!("{name}: non-finite input or result."),
        ))
    }
}

fn positive(value: Option<Real>, name: &str) -> EResult<Real> {
    let value = finite(value.ok_or_else(|| missing(name))?, name)?;
    if value <= 0.0 {
        return Err(domain(&format!("{name} must be strictly positive.")));
    }
    Ok(value)
}

fn product(a: Real, b: Real, name: &str) -> EResult<Real> {
    let result = finite(a * b, name)?;
    if a != 0.0 && b != 0.0 && result == 0.0 {
        return Err(domain(&format!("{name}: nonzero product reduced to zero.")));
    }
    Ok(result)
}
