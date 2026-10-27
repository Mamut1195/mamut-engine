//! Euler–Bernoulli + consistent Hermite subgrade composition.
use crate::{finite, geometry, invalid, EulerFrame2D, Matrix};
use engine_contracts::{Element, ElementCtx, ElementOutput, ElementResponse, MassKind, MemberLoad};
use engine_core::{ContractId, Dof, EResult, Real};
use engine_geo::winkler;
use engine_linalg::DMat;

/// `frame2d.winkler` beam, bilateral linear foundation attached to ground, with no mass of its own.
/// k in N/m² per beam length: local reaction −k·displacement (N/m).
/// Local transverse y direction.
/// Does not support end releases. Retains frame mass, loads, and Kg.
/// The foundation removes transverse rigid modes; K of the complete foundation
/// is semidefinite due to its axial zeros.
/// Original derivation: provenance/ENG-GEO-002.md (Winkler 1867; Hetényi 1946).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WinklerFrame2D {
    beam: EulerFrame2D,
    k: Real,
}
impl WinklerFrame2D {
    /// Adds a foundation to the frame without releases; ENG-E-0010 for releases or k<0,
    /// ENG-E-0008 for nonfinite k. Frame properties are already validated.
    pub fn new(beam: EulerFrame2D, k: Real) -> EResult<Self> {
        winkler::validate(k)?;
        if beam.releases != crate::MomentReleases::default() {
            return Err(invalid("Winkler: end releases are not allowed."));
        }
        Ok(Self { beam, k })
    }
    /// Foundation matrix in local axes and frame nodal order, without EI.
    pub fn foundation_matrix(&self, ctx: &ElementCtx<'_>) -> EResult<DMat> {
        let g = geometry::Geometry::new(ctx)?;
        DMat::new(
            6,
            6,
            self.local_matrix(g.length)?.into_iter().flatten().collect(),
        )
    }
    fn local_matrix(&self, l: Real) -> EResult<Matrix> {
        let mut out = [[0.0; 6]; 6];
        for (ids, sign, k) in [([1, 2, 4, 5], 1.0, self.k)] {
            let block = winkler::matrix(k, l)?;
            let signs = [1.0, sign, 1.0, sign];
            for (i, &r) in ids.iter().enumerate() {
                for (j, &s) in ids.iter().enumerate() {
                    out[r][s] = signs[i] * signs[j] * block[i][j];
                }
            }
        }
        Ok(out)
    }
}
impl Element for WinklerFrame2D {
    fn kind(&self) -> &'static str {
        "frame2d.winkler"
    }
    fn contract(&self) -> ContractId {
        self.beam.contract()
    }
    fn n_nodes(&self) -> usize {
        self.beam.n_nodes()
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        self.beam.dofs_per_node()
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        // History, geometry, and vector validation identical to the base frame.
        let mut response = self.beam.respond(ctx, u_e, committed)?;
        let g = geometry::Geometry::new(ctx)?;
        let bed = self.local_matrix(g.length)?;
        let force = g.global_vector(&geometry::multiply(&bed, &g.local_vector(u_e)?)?)?;
        let global = g.global_matrix(&bed)?;
        let values = response
            .tangent
            .values()
            .iter()
            .zip(global.values())
            .map(|(a, b)| finite(a + b, "K Euler + Winkler"))
            .collect::<EResult<Vec<_>>>()?;
        response.tangent = DMat::new(6, 6, values)?;
        for (f, b) in response.internal_force.iter_mut().zip(force) {
            *f = finite(*f + b, "F Euler + Winkler")?;
        }
        Ok(response)
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        self.beam.mass(ctx, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        self.beam.geometric_stiffness(ctx, u, committed)
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        self.beam.load_vector(ctx, load)
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput> {
        let mut out = self.beam.recover(ctx, u, committed, loads, stations)?;
        let g = geometry::Geometry::new(ctx)?;
        let u = g.local_vector(u)?;
        let bed = geometry::multiply(&self.local_matrix(g.length)?, &u)?;
        for (f, b) in out.end_forces_local.iter_mut().zip(bed) {
            *f = finite(*f + b, "Winkler end")?;
        }
        for (ids, sign, k) in [([1, 2, 4, 5], 1.0, self.k)] {
            let d = [u[ids[0]], sign * u[ids[1]], u[ids[2]], sign * u[ids[3]]];
            let shear = if ids[0] == 1 { 1 } else { 2 };
            let moment = if ids[0] == 1 { 5 } else { 4 };
            for station in &mut out.stations {
                let x = station.position * g.length;
                let [force, torque] = winkler::integrals(k, g.length, d, station.position)?;
                station.forces_local[shear] = finite(
                    station.forces_local[shear] - bed[ids[0]] + force,
                    "Winkler shear",
                )?;
                station.forces_local[moment] = finite(
                    station.forces_local[moment] - bed[ids[1]] + sign * (bed[ids[0]] * x - torque),
                    "Winkler moment",
                )?;
            }
        }
        Ok(out)
    }
}
