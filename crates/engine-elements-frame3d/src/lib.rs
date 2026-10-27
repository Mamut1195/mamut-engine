#![forbid(unsafe_code)]
//! Spatial beam-column `frame3d.euler_elastic`, contract element/1, version 0.1.0.
//! [`WinklerFrame3D`] adds a consistent transverse foundation attached to ground;
//! [`WinklerFactory`] constructs `frame3d.winkler` with SI properties.
//!
//! Homogeneous, infinitesimal Euler–Bernoulli, with Saint-Venant torsion.
//! Right-handed global triad, horizontal X-Y and Z upward. Order per node
//! `[Ux,Uy,Uz,Rx,Ry,Rz]`; local x points from i to j. Orientation is a
//! reference for local z: projected normal to x, then y=z×x, z=x×y.
//! Rotations follow the right-hand rule; v′=rz, w′=−ry; positive N in tension.
//! E,G in Pa; A in m²; J,Iy,Iz in m⁴; rho in kg/m³; length in m.
//! J is the torsion constant, distinct from polar inertia Iy+Iz.
//! Forces in N, moments in N·m, rotations in rad. Properties in self,
//! coordinates in ctx; ctx.params is unused, as in frame2d.
//!
//! `respond` returns elastic K and K u, without loads or Kg. Kg is obtained
//! separately from axial N=EA(uj−ui)/L and fixed reference geometry.
//! Releases in local rx,ry,rz: analytical elastic Schur complement; mass and Kg
//! projected with HᵀMH and HᵀKgH (Guyan and Schur derivative at N=0).
//! This is not condensation of K+Kg at finite N. Released nodal rotations remain
//! disconnected; the assembler must remove/restrain mechanisms.
//! Doubly released torsion: common internal rotation set to zero (an
//! energy-free reference choice); a net distributed torque requires impossible
//! equilibrium and produces ENG-E-0006. No artificial stiffness is added.
//!
//! Transverse Hermite mass without bending rotary inertia; linear torsional
//! mass with rho(Iy+Iz). Lumped mass: rho A L/2 per translation and
//! rho(Iy+Iz)L/2 in rx. rho=0 returns None after validating geometry.
//! MemberLoad is global; [`EulerFrame3D::global_load`] converts local
//! intensities to the contract. Fully restrained ends = minus equivalent load.
//! Equilibrium recovery: `[N,Vy,Vz,T,My,Mz]` on the local positive face.
//!
//! ENG-E-0005 for L≤1e-12; ENG-E-0008 for nonfinite values/overflow;
//! ENG-E-0010 for ranges/sizes/degenerate orientation. No I/O, threads,
//! clock, global state, or history; deterministic per target. Numerical wasm
//! execution pending. Derivations in provenance/ENG-ELM-003.md.
//!
//! ```
//! use engine_elements_frame3d::{EulerFrame3D, MomentReleases};
//! use engine_contracts::{Element, ElementCtx, ElementParams};
//! use engine_core::Dim;
//! let beam = EulerFrame3D::new(200.0,80.0,2.0,1.0,3.0,4.0,0.0,
//!     [0.0,0.0,1.0],MomentReleases::default())?;
//! let params = ElementParams::default();
//! let coords = [[0.0;3],[2.0,0.0,0.0]];
//! let ctx = ElementCtx { dim: Dim::D3, coords: &coords, params: &params };
//! let mut u = [0.0;12]; u[6] = 0.06;
//! assert!((beam.respond(&ctx,&u,&[])?.internal_force[6]-12.0).abs()<1e-12);
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

mod factory;
mod geometric;
mod geometry;
mod loads;
mod mass;
mod producer;
mod recover;
mod release;
mod stiffness;
mod winkler;

use engine_contracts::{Element, ElementCtx, ElementOutput, ElementResponse, MassKind, MemberLoad};
use engine_core::{ContractId, Dof, EResult, EngineError, Real};
use engine_laws::uniaxial::Elastic;
use engine_linalg::DMat;
pub use factory::{FrameFactory, WinklerFactory};
pub use producer::geometric_producer;
pub use release::MomentReleases;
pub use winkler::WinklerFrame3D;
type Matrix = [[Real; 12]; 12];
// Hermite pairs: v′=rz and w′=−ry.
const PLANES: [([usize; 4], Real); 2] = [([1, 5, 7, 11], 1.0), ([2, 4, 8, 10], -1.0)];

/// Two-node elastic 3D beam, SI properties and local releases.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EulerFrame3D {
    material: Elastic,
    shear_modulus: Real,
    area: Real,
    torsion_constant: Real,
    iy: Real,
    iz: Real,
    density: Real,
    orientation: [Real; 3],
    ea: Real,
    gj: Real,
    eiy: Real,
    eiz: Real,
    releases: MomentReleases,
}
impl EulerFrame3D {
    /// Validates E,G,A,J,Iy,Iz>0, rho≥0, and finite nonzero orientation.
    /// The nonparallel-to-x condition is validated once coordinates are known.
    /// Positive products; ENG-E-0010 for range and ENG-E-0008 for nonfinite values.
    #[allow(clippy::too_many_arguments)] // Explicit SI properties, following the frame2d pattern.
    pub fn new(
        e: Real,
        g: Real,
        area: Real,
        j: Real,
        iy: Real,
        iz: Real,
        density: Real,
        orientation: [Real; 3],
        releases: MomentReleases,
    ) -> EResult<Self> {
        let material = Elastic::new(e)?;
        positive(g, "G (Pa)")?;
        positive(area, "A (m²)")?;
        positive(j, "J (m⁴)")?;
        positive(iy, "Iy (m⁴)")?;
        positive(iz, "Iz (m⁴)")?;
        finite(density, "rho (kg/m³)")?;
        if density < 0.0 {
            return Err(invalid("rho must be nonnegative."));
        }
        geometry::unit(orientation)?;
        Ok(Self {
            material,
            shear_modulus: g,
            area,
            torsion_constant: j,
            iy,
            iz,
            density,
            orientation,
            ea: positive(e * area, "EA")?,
            gj: positive(g * j, "GJ")?,
            eiy: positive(e * iy, "EIy")?,
            eiz: positive(e * iz, "EIz")?,
            releases,
        })
    }
    /// Uniaxial elastic material, modulus E in Pa.
    pub fn material(&self) -> &Elastic {
        &self.material
    }
    /// Shear modulus G in Pa, used only in torsion.
    pub fn shear_modulus(&self) -> Real {
        self.shear_modulus
    }
    /// Area A in m².
    pub fn area(&self) -> Real {
        self.area
    }
    /// Saint-Venant constant J in m⁴.
    pub fn torsion_constant(&self) -> Real {
        self.torsion_constant
    }
    /// Area moments `[Iy,Iz]` in m⁴.
    pub fn inertias(&self) -> [Real; 2] {
        [self.iy, self.iz]
    }
    /// Density rho in kg/m³.
    pub fn density(&self) -> Real {
        self.density
    }
    /// Global reference for local z, unnormalized.
    pub fn orientation(&self) -> [Real; 3] {
        self.orientation
    }
    /// Releases of rx,ry,rz at each local end.
    pub fn releases(&self) -> MomentReleases {
        self.releases
    }
    /// Constructs q=rho A g (N/m) with explicit global acceleration (m/s²).
    pub fn self_weight(&self, acceleration: [Real; 3]) -> EResult<MemberLoad> {
        vector(&acceleration, 3, "g (m/s²)")?;
        let line_mass = if self.density == 0.0 {
            0.0
        } else {
            positive(self.density * self.area, "rho A")?
        };
        let mut force_per_length = [0.0; 3];
        for (q, g) in force_per_length.iter_mut().zip(acceleration) {
            *q = finite(line_mass * g, "Self-weight")?;
        }
        Ok(MemberLoad {
            force_per_length,
            moment_per_length: [0.0; 3],
        })
    }
    /// Converts local force and moment intensities to a global MemberLoad.
    /// Uses the same triad as K; rejects invalid geometry and loads.
    pub fn global_load(&self, ctx: &ElementCtx<'_>, local: &MemberLoad) -> EResult<MemberLoad> {
        let g = geometry::Geometry::new(ctx, self.orientation)?;
        Ok(MemberLoad {
            force_per_length: g.global_triplet(local.force_per_length)?,
            moment_per_length: g.global_triplet(local.moment_per_length)?,
        })
    }
    /// Member displacements in local order of 12 DOFs, m/rad.
    /// Recovers released rotations with global loads; ignores their nodal rotations.
    pub fn local_displacements(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        loads: &[MemberLoad],
    ) -> EResult<[Real; 12]> {
        let g = geometry::Geometry::new(ctx, self.orientation)?;
        let u = g.local_vector(u_e)?;
        let q = loads::sum(&g, loads)?;
        release::displacements(self, &g, &u, &loads::local_vector(g.length, q)?)
    }
}
impl Element for EulerFrame3D {
    fn kind(&self) -> &'static str {
        "frame3d.euler_elastic"
    }
    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    }
    fn n_nodes(&self) -> usize {
        2
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        &[Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz]
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        vector(committed, 0, "Historia")?;
        let g = geometry::Geometry::new(ctx, self.orientation)?;
        let u = g.local_vector(u_e)?;
        let k = stiffness::condensed(self, &g)?;
        Ok(ElementResponse {
            internal_force: g.global_vector(&geometry::multiply(&k, &u)?)?.to_vec(),
            tangent: g.global_matrix(&k)?,
            trial_state: vec![],
        })
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        mass::matrix(self, &geometry::Geometry::new(ctx, self.orientation)?, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        vector(committed, 0, "Historia")?;
        let g = geometry::Geometry::new(ctx, self.orientation)?;
        let u = g.local_vector(u_e)?;
        let n = stiffness::axial(self, &g, &u)?;
        Ok(Some(g.global_matrix(&release::project(
            &geometric::local_matrix(g.length, n)?,
            &self.releases.transform(g.length)?,
        )?)?))
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        let g = geometry::Geometry::new(ctx, self.orientation)?;
        let f = loads::local_vector(g.length, loads::components(&g, load)?)?;
        let f = release::load(self, &g, &f)?;
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
        recover::output(
            self,
            &geometry::Geometry::new(ctx, self.orientation)?,
            u_e,
            loads,
            stations,
        )
    }
}
fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
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
