#![forbid(unsafe_code)]
//! Homogeneous pin-jointed trusses `truss2d.linear` and `truss3d.linear`, element/1.
//!
//! Linear elasticity, infinitesimal displacements and rotations, without prestress,
//! history, bending, or torsion. Axis d points from node i to j; positive N is
//! tension. D2 uses horizontal X-Y, Z=0; D3 uses the right-handed triad, Z upward.
//! Only Ux,Uy (D2) or Ux,Uy,Uz (D3), ordered by node. Coordinates/u in m,
//! E in Pa, A in m², rho in kg/m³, force in N, K/Kg in N/m, and mass in kg.
//!
//! The element owns the validated properties; `ctx.coords` supplies
//! reference geometry. `ctx.params` is unused: two property sources are not
//! mixed. The factory uses explicit SI numbers E, A, and rho.
//! Isotropic translational mass with linear interpolation. Kg is provided separately:
//! it does not modify the tangent or turn respond into second-order analysis.
//! [`geometric_producer`] adapts this Kg to engine-assembly: captures geometry and
//! properties and evaluates N with total u_e, including prescribed displacements.
//! Only uniform axial loads without moments; self-weight must be converted
//! explicitly to q=rho A g and be parallel to the axis. Gravity is not assumed.
//! A bar fixed at both ends with a uniform temperature change is
//! [`LinearTruss::fixed_end_uniform_temperature`]: tension-positive
//! N = -(E A α ΔT), plus the equivalent joint forces of this same bar.
//! A thermal gradient is not represented. That resultant is not a `MemberLoad`.
//! Recovery returns local ends [-N_i,0,…,N_j,0,…] and stations
//! with N on the positive face; no D3 transverse orientation is needed.
//!
//! Nonfinite inputs/results: ENG-E-0008; invalid sizes, parameters, loads, or
//! stations: ENG-E-0010; L≤1e-12 m: ENG-E-0005. No I/O, threads,
//! clock, or mutable state. Deterministic on each target; binary equality
//! across targets pending ENG-CORE-010.
//!
//! ```
//! use engine_elements_truss::LinearTruss;
//! use engine_contracts::{Element, ElementCtx, ElementParams};
//! use engine_core::Dim;
//! let bar = LinearTruss::new(Dim::D2, 200.0, 2.0, 3.0)?;
//! let params = ElementParams::default();
//! let coords = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
//! let ctx = ElementCtx { dim: Dim::D2, coords: &coords, params: &params };
//! let r = bar.respond(&ctx, &[0.0, 0.0, 0.06, 0.0], &[])?;
//! assert!((r.internal_force[2] - 12.0).abs() < 1e-12);
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
mod stiffness;

use engine_contracts::{Element, ElementCtx, ElementOutput, ElementResponse, MassKind, MemberLoad};
use engine_core::{ContractId, Dim, Dof, EResult, EngineError, Real};
use engine_laws::uniaxial::Elastic;
use engine_linalg::DMat;
pub use factory::TrussFactory;
pub use producer::geometric_producer;

/// Uniform temperature on one existing bar fixed at both ends.
///
/// Tension is positive. The thermal gradient is absent: this record is only
/// the uniform change. [`Element::recover`] is unchanged and does not infer
/// this force from a `MemberLoad`.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedEndUniformTemperature {
    /// Axial force in N, constant along the bar. Negative when αΔT > 0.
    pub axial_force: Real,
    /// Global equivalent joint forces in N, in [`Element::load_vector`] order.
    ///
    /// Assemble them through the existing element-load channel. Their sum is
    /// zero. Heating pushes the two joints outward.
    pub joint_forces: Vec<Real>,
    /// Local end actions and stations for this constant N.
    ///
    /// The axial slot at end i is -N and the axial slot at end j is +N.
    /// Every requested station repeats `axial_force`. Other components are zero.
    pub output: ElementOutput,
}

/// Two-node linear truss with immutable SI properties and no history.
///
/// Both kinds share the formulation; only dimension and active DOFs differ.
/// See the crate documentation for assumptions, units, and supported loads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearTruss {
    dim: Dim,
    material: Elastic,
    area: Real,
    density: Real,
    ea: Real,
}

impl LinearTruss {
    /// Constructs with E>0 Pa, A>0 m², and rho≥0 kg/m³, all finite.
    ///
    /// ENG-E-0008 for nonfinite values/overflowed products; ENG-E-0010 for
    /// invalid range or positive EA reduced to zero by underflow.
    pub fn new(dim: Dim, e: Real, area: Real, density: Real) -> EResult<Self> {
        let material = Elastic::new(e)?;
        positive(area, "A (m²)")?;
        finite(density, "rho (kg/m³)")?;
        if density < 0.0 {
            return Err(invalid("rho must be nonnegative (kg/m³)."));
        }
        let ea = positive(e * area, "EA (N)")?;
        Ok(Self {
            dim,
            material,
            area,
            density,
            ea,
        })
    }

    /// Truss dimension; determines two or three translations per node.
    pub fn dim(&self) -> Dim {
        self.dim
    }

    /// Validated area A, in m².
    pub fn area(&self) -> Real {
        self.area
    }

    /// Validated density rho, in kg/m³; zero means no mass.
    pub fn density(&self) -> Real {
        self.density
    }

    /// Uniaxial elastic material from engine-laws, with modulus E in Pa.
    pub fn material(&self) -> &Elastic {
        &self.material
    }

    fn geometry(&self, ctx: &ElementCtx<'_>) -> EResult<geometry::Geometry> {
        geometry::Geometry::new(self.dim, ctx)
    }

    /// Fixed-end axial response to one uniform temperature change.
    ///
    /// `alpha_per_kelvin` is the linear expansion coefficient in 1/K and
    /// `delta_t_kelvin` is the uniform change in K. Both must be finite;
    /// either sign is kept. With the tension-positive convention of this
    /// element, `axial_force = -(E · A · α · ΔT)` in N. The magnitude
    /// `E · A · α · ΔT` does not depend on the length. `stations` are
    /// dimensionless x/L in [0, 1], as in [`Element::recover`].
    ///
    /// No second temperature and no gradient are accepted. A nonfinite
    /// product is ENG-E-0008. Geometry errors are the same codes as the
    /// other bar operations.
    ///
    /// ```
    /// use engine_contracts::{ElementCtx, ElementParams};
    /// use engine_core::Dim;
    /// use engine_elements_truss::LinearTruss;
    /// let bar = LinearTruss::new(Dim::D2, 8.0, 4.0, 0.0)?;
    /// let params = ElementParams::default();
    /// let coords = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
    /// let ctx = ElementCtx { dim: Dim::D2, coords: &coords, params: &params };
    /// let thermal = bar.fixed_end_uniform_temperature(&ctx, 0.5, 3.0, &[0.5])?;
    /// assert_eq!(thermal.axial_force, -48.0);
    /// # Ok::<(), engine_core::EngineError>(())
    /// ```
    pub fn fixed_end_uniform_temperature(
        &self,
        ctx: &ElementCtx<'_>,
        alpha_per_kelvin: Real,
        delta_t_kelvin: Real,
        stations: &[Real],
    ) -> EResult<FixedEndUniformTemperature> {
        let axial_force =
            loads::fixed_end_uniform_temperature(self.ea, alpha_per_kelvin, delta_t_kelvin)?;
        let g = self.geometry(ctx)?;
        let joint_forces = loads::fixed_end_uniform_temperature_forces(&g, axial_force)?;
        let output = recover::fixed_end_uniform_temperature(&g, axial_force, stations)?;
        Ok(FixedEndUniformTemperature {
            axial_force,
            joint_forces,
            output,
        })
    }
}

impl Element for LinearTruss {
    fn kind(&self) -> &'static str {
        match self.dim {
            Dim::D2 => "truss2d.linear",
            Dim::D3 => "truss3d.linear",
        }
    }
    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    }
    fn n_nodes(&self) -> usize {
        2
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        match self.dim {
            Dim::D2 => &[Dof::Ux, Dof::Uy],
            Dim::D3 => &[Dof::Ux, Dof::Uy, Dof::Uz],
        }
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        let g = self.geometry(ctx)?;
        let n = stiffness::axial(self, &g, u_e, committed)?;
        let internal_force = g
            .b()
            .iter()
            .map(|b| finite(n * b, "Internal F (N)"))
            .collect::<EResult<_>>()?;
        Ok(ElementResponse {
            internal_force,
            tangent: stiffness::matrix(self, &g)?,
            trial_state: vec![],
        })
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        mass::matrix(self, &self.geometry(ctx)?, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        let g = self.geometry(ctx)?;
        let n = stiffness::axial(self, &g, u_e, committed)?;
        Ok(Some(geometric::matrix(&g, n)?))
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        loads::vector(&self.geometry(ctx)?, load)
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput> {
        recover::output(self, &self.geometry(ctx)?, u_e, committed, loads, stations)
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
