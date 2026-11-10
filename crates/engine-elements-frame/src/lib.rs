#![forbid(unsafe_code)]
//! Beam-column `frame2d.euler_elastic`, contract element/1, method 0.1.0.
//! [`WinklerFrame2D`] adds a consistent transverse foundation attached to ground;
//! [`WinklerFactory`] constructs `frame2d.winkler` with SI properties.
//! [`FiberStationBar`] is a separate bar: it samples `rectangle.fiber_bending`
//! at three stations and does not replace [`EulerFrame2D`]. The notes below
//! describe [`EulerFrame2D`].
//!
//! Homogeneous, infinitesimal Euler–Bernoulli, without shear deformation, section
//! rotary inertia, or history. D2 is in horizontal X-Y, Z=0; local x
//! from i to j, y=(-sin,cos), z=+Z. Order `[ux,uy,rz]` per node; positive N in
//! tension and counterclockwise rotations. E in Pa, A in m², I=Iz in m⁴, rho in kg/m³;
//! coordinates/translations in m, rotations in rad, forces in N, moments in N·m. K and Kg
//! have entries in N/m, N, and N·m; M has entries in kg, kg·m, and kg·m².
//!
//! Immutable properties in self; ctx supplies coordinates, not properties:
//! ctx.params is unused, following the internal element template.
//! `respond` is elastic: Kg is returned separately with N obtained from u_e.
//! Global uniform XY loads and distributed moment about Z;
//! self-weight requires explicit gravity; Y is never assumed vertical.
//! A uniform temperature change is not implemented: this element has no
//! αΔT formula, and the public temperature load is rejected for this kind.
//! Recovered ends are K u minus equivalent loads (actions on
//! the member); stations `[N,Vy,0,0,0,Mz]` on the local positive face.
//!
//! Releases replace internal rotations through elastic equilibrium:
//! Kc=HᵀKH, Mc=HᵀMH (Guyan static reduction), Kgc=HᵀKgH,
//! fc=Hᵀf. Kg is the first derivative of condensation of K+Kg at N=0;
//! not an exact N-dependent condensation at finite compression.
//! With two releases, Kflexión=0 exactly: nodal rotations are
//! disconnected. The assembler must restrain/remove DOFs without other
//! contributions; no artificial stiffness is added. Member rotations are recovered
//! through [`EulerFrame2D::local_displacements`], including loads.
//!
//! ENG-E-0005: L≤1e-12 m; ENG-E-0008: nonfinite values/overflow;
//! ENG-E-0010: invalid ranges, sizes, plane, or stations;
//! ENG-E-0013: out-of-plane loads. No I/O, threads, clock, or global state;
//! deterministic per target; wasm execution and numerical equivalence pending.
//! Derivations and limits in provenance/ENG-ELM-002.md.
//!
//! ```
//! use engine_elements_frame::{EulerFrame2D, MomentReleases};
//! use engine_contracts::{Element, ElementCtx, ElementParams};
//! use engine_core::Dim;
//! let beam = EulerFrame2D::new(200.0, 2.0, 3.0, 0.0, MomentReleases::default())?;
//! let params = ElementParams::default();
//! let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
//! let ctx = ElementCtx { dim: Dim::D2, coords: &coords, params: &params };
//! let response = beam.respond(&ctx, &[0.0,0.0,0.0,0.06,0.0,0.0], &[])?;
//! assert!((response.internal_force[3] - 12.0).abs() < 1e-12);
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

mod factory;
mod fiber_stations;
mod geometric;
mod geometry;
mod loads;
mod mass;
mod recover;
mod release;
mod stiffness;
mod winkler;

use engine_contracts::{Element, ElementCtx, ElementOutput, ElementResponse, MassKind, MemberLoad};
use engine_core::{ContractId, Dof, EResult, EngineError, Real};
use engine_laws::uniaxial::Elastic;
use engine_linalg::DMat;
pub use factory::{FrameFactory, WinklerFactory};
pub use fiber_stations::{
    FiberStationBar, FIBER_STATIONS_EXPERIMENTAL, FIBER_STATION_COUNT, MAX_RECOVER_STATIONS,
};
pub use release::MomentReleases;
pub use winkler::WinklerFrame2D;
type Matrix = [[Real; 6]; 6];

/// Two-node elastic 2D beam, with owned SI properties and releases.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EulerFrame2D {
    material: Elastic,
    area: Real,
    inertia: Real,
    density: Real,
    ea: Real,
    ei: Real,
    releases: MomentReleases,
}
impl EulerFrame2D {
    /// Validates finite E,A,I>0 and rho≥0; I is Iz. Positive products EA/EI.
    /// ENG-E-0010 for range/underflow, ENG-E-0008 for nonfinite values.
    pub fn new(
        e: Real,
        area: Real,
        inertia: Real,
        density: Real,
        releases: MomentReleases,
    ) -> EResult<Self> {
        let material = Elastic::new(e)?;
        positive(area, "A (m²)")?;
        positive(inertia, "I (m⁴)")?;
        finite(density, "rho (kg/m³)")?;
        if density < 0.0 {
            return Err(invalid("rho must be nonnegative."));
        }
        Ok(Self {
            material,
            area,
            inertia,
            density,
            ea: positive(e * area, "EA (N)")?,
            ei: positive(e * inertia, "EI (N·m²)")?,
            releases,
        })
    }
    /// Uniaxial elastic material, modulus E in Pa.
    pub fn material(&self) -> &Elastic {
        &self.material
    }
    /// Area A in m².
    pub fn area(&self) -> Real {
        self.area
    }
    /// Second moment I=Iz in m⁴.
    pub fn inertia(&self) -> Real {
        self.inertia
    }
    /// Density rho in kg/m³; zero represents no mass.
    pub fn density(&self) -> Real {
        self.density
    }
    /// Moment releases; never releases translations.
    pub fn releases(&self) -> MomentReleases {
        self.releases
    }
    /// Constructs q=rho A g (N/m) from explicit global acceleration (m/s²).
    /// D2 allows g only in XY; ENG-E-0013 for a nonzero Z component.
    pub fn self_weight(&self, acceleration: [Real; 3]) -> EResult<MemberLoad> {
        vector(&acceleration, 3, "g (m/s²)")?;
        if acceleration[2] != 0.0 {
            return Err(unsupported("Self-weight is outside the XY plane."));
        }
        let line_mass = if self.density == 0.0 {
            0.0
        } else {
            positive(self.density * self.area, "rho A (kg/m)")?
        };
        let mut force_per_length = [0.0; 3];
        for (q, g) in force_per_length.iter_mut().zip(acceleration) {
            *q = finite(line_mass * g, "Self-weight (N/m)")?;
        }
        Ok(MemberLoad {
            force_per_length,
            moment_per_length: [0.0; 3],
        })
    }
    /// Local `[u_i,v_i,theta_i,u_j,v_j,theta_j]`, m/rad; recovers released
    /// rotations through K equilibrium, including all uniform loads.
    /// Supplied nodal rotations at released ends are unused.
    /// Errors for geometry, sizes, loads, and finiteness as documented for the crate.
    pub fn local_displacements(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        loads: &[MemberLoad],
    ) -> EResult<[Real; 6]> {
        let g = geometry::Geometry::new(ctx)?;
        let u = g.local_vector(u_e)?;
        let q = loads::sum(&g, loads)?;
        release::displacements(self, &g, &u, &loads::local_vector(g.length, q)?)
    }
}
impl Element for EulerFrame2D {
    fn kind(&self) -> &'static str {
        "frame2d.euler_elastic"
    }
    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    }
    fn n_nodes(&self) -> usize {
        2
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        &[Dof::Ux, Dof::Uy, Dof::Rz]
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        vector(committed, 0, "Historia")?;
        let g = geometry::Geometry::new(ctx)?;
        let u = g.local_vector(u_e)?;
        let k = stiffness::condensed(self, &g)?;
        let force = geometry::multiply(&k, &u)?;
        Ok(ElementResponse {
            internal_force: g.global_vector(&force)?.to_vec(),
            tangent: g.global_matrix(&k)?,
            trial_state: vec![],
        })
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        let g = geometry::Geometry::new(ctx)?;
        mass::matrix(self, &g, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        vector(committed, 0, "Historia")?;
        let g = geometry::Geometry::new(ctx)?;
        let u = g.local_vector(u_e)?;
        let n = stiffness::axial(self, &g, &u)?;
        let kg = geometric::local_matrix(g.length, n)?;
        Ok(Some(g.global_matrix(&release::project(
            &kg,
            &self.releases.transform(g.length)?,
        )?)?))
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        let g = geometry::Geometry::new(ctx)?;
        let f = loads::local_vector(g.length, loads::components(&g, load)?)?;
        let f = release::force(&f, &self.releases.transform(g.length)?)?;
        Ok(g.global_vector(&f)?.to_vec())
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput> {
        vector(committed, 0, "Historia")?;
        recover::output(self, &geometry::Geometry::new(ctx)?, u_e, loads, stations)
    }
}
fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
fn unsupported(message: &str) -> EngineError {
    EngineError::new("ENG-E-0013", message)
}
fn finite(value: Real, name: &str) -> EResult<Real> {
    if !value.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            format!("{name}: non-finite input or result."),
        ));
    }
    Ok(value)
}
fn positive(value: Real, name: &str) -> EResult<Real> {
    finite(value, name)?;
    if value <= 0.0 {
        return Err(invalid(&format!("{name}: must be positive.")));
    }
    Ok(value)
}
fn vector(values: &[Real], size: usize, name: &str) -> EResult<()> {
    if values.len() != size {
        return Err(invalid(&format!(
            "{name}: size {}, expected {size}.",
            values.len()
        )));
    }
    for &v in values {
        finite(v, name)?;
    }
    Ok(())
}
