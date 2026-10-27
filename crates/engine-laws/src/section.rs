//! Elastic frame section `elastic_frame`, contract `section/1`.
//!
//! Approved axes: right-handed triad, horizontal X-Y, vertical Z upward. D2 is
//! represented in X-Y, Z = 0. Local x runs from i to j; in D2 local z
//! points toward +Z. In D3 the consumer sets the transverse orientation.
//! See `docs/AXES_CONVENTION.md`; the D2 elevation plane is not redefined.

use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{DMat, SectionLaw, SectionResponse};
use engine_core::{ContractId, Dim, EResult, Real};

/// Elastic frame section descriptor, method 0.1.0.
pub const ELASTIC_FRAME: LawDescriptor = LawDescriptor {
    kind: "elastic_frame",
    contract: ContractId::new("section", 1, 0),
    version: (0, 1, 0),
};

/// SI properties supplied by the consumer; no geometric calculation.
///
/// All required properties must be finite and positive, including
/// Iy, J, and G in D2. Effective shear areas are validated only when present.
/// G is supplied explicitly; it is not inferred from E or a Poisson
/// ratio. The y/z axes must be centroidal principal axes (without coupling).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElasticFrameParams {
    /// Young's modulus E, Pa.
    pub e: Real,
    /// Shear modulus G, Pa.
    pub g: Real,
    /// Area A, m².
    pub area: Real,
    /// Second moment Iy = integral of z² over the area, m⁴.
    pub iy: Real,
    /// Second moment Iz = integral of y² over the area, m⁴.
    pub iz: Real,
    /// Saint-Venant torsion constant J, m⁴; polar inertia is not assumed.
    pub j: Real,
    /// Local y effective shear area, Ay in m²; `None` indicates absence.
    pub ay: Option<Real>,
    /// Local z effective shear area, Az in m²; `None` indicates absence.
    pub az: Option<Real>,
}

/// Interchangeable section factory, without assembly or a global registry.
///
/// Implementations with the same `Params` can be interchanged as trait
/// objects; products are consumed through [`SectionLaw`].
pub trait SectionFactory: Send + Sync {
    /// Typed SI parameters of the implementation.
    type Params;
    /// Kind, contract, and semantic version of the constructed method.
    fn descriptor(&self) -> LawDescriptor;
    /// Constructs for the requested dimension; errors depend on the implementation.
    fn build(&self, dim: Dim, params: Self::Params) -> EResult<Box<dyn SectionLaw>>;
}

/// Factory for [`ElasticFrame`] in D2 and D3.
#[derive(Clone, Copy, Debug, Default)]
pub struct ElasticFrameFactory;

impl SectionFactory for ElasticFrameFactory {
    type Params = ElasticFrameParams;

    fn descriptor(&self) -> LawDescriptor {
        ELASTIC_FRAME
    }

    fn build(&self, dim: Dim, params: ElasticFrameParams) -> EResult<Box<dyn SectionLaw>> {
        Ok(Box::new(ElasticFrame::new(dim, params)?))
    }
}

/// Homogeneous linear section, without coupling, preload, warping, or history.
///
/// D2: `e = [ε0, κz]`, `s = [N, Mz]`, diagonal tangent `(EA, EIz)`.
/// D3: `e = [ε0, κy, κz, θ′]`, `s = [N, My, Mz, T]`, diagonal tangent
/// `(EA, EIy, EIz, GJ)`. ε0 is dimensionless; κy, κz, and θ′ in m⁻¹
/// (rad/m for rotations). N in N; My, Mz, and T in N·m. EA in N; EI and GJ in N·m².
///
/// Positive N is tension. On the +x face, positive moments follow the
/// right-hand rule about +y, +z, and +x, respectively; on the −x face,
/// their actions are opposite. Uses εx(y,z) = ε0 + κy z − κz y,
/// conjugate to My = integral of z σx and Mz = integral of −y σx. Positive
/// curvatures produce positive moments; θ′ is the derivative of local rotation
/// about +x. This section does not transform end or global actions.
///
/// Ay/Az only enable the GAy/GAz accessors in N. They add no shear strains
/// or resultants to `section/1`; a shear consumer must use them
/// explicitly. The numerical range requires finite inputs and resultants.
///
/// ```
/// use engine_contracts::SectionLaw;
/// use engine_core::Dim;
/// use engine_laws::section::{ElasticFrame, ElasticFrameParams};
/// let params = ElasticFrameParams {
///     e: 200.0, g: 80.0, area: 3.0, iy: 2.0, iz: 5.0, j: 4.0,
///     ay: Some(1.5), az: None,
/// };
/// let section = ElasticFrame::new(Dim::D3, params)?;
/// let r = section.respond(&[0.02, -0.03, 0.04, -0.05], &[])?;
/// assert_eq!(r.resultants, vec![12.0, -12.0, 40.0, -16.0]);
/// assert_eq!(section.shear_rigidity_y(), Some(120.0));
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ElasticFrame {
    dim: Dim,
    params: ElasticFrameParams,
    rigidities: [Real; 4],
    shear_y: Option<Real>,
    shear_z: Option<Real>,
}

impl ElasticFrame {
    /// Validates properties and products EA, EIy, EIz, GJ, and any GAy/GAz present.
    ///
    /// # Errors
    /// `ENG-E-0008` for nonfinite values/overflow; `ENG-E-0010` for nonpositive
    /// values, including products reduced to zero by underflow.
    /// All products are also validated in D2, although Iy and J do not enter
    /// its tangent. No physical tolerance or shear correction is applied.
    pub fn new(dim: Dim, params: ElasticFrameParams) -> EResult<Self> {
        for (value, name) in [
            (params.e, "E (Pa)"),
            (params.g, "G (Pa)"),
            (params.area, "A (m²)"),
            (params.iy, "Iy (m⁴)"),
            (params.iz, "Iz (m⁴)"),
            (params.j, "J (m⁴)"),
        ] {
            positive(value, name)?;
        }
        let shear = |area: Option<Real>, name: &str| -> EResult<Option<Real>> {
            area.map(|a| {
                positive(a, name)?;
                positive(params.g * a, &format!("G × {name}"))
            })
            .transpose()
        };
        Ok(Self {
            dim,
            params,
            rigidities: [
                positive(params.e * params.area, "EA (N)")?,
                positive(params.e * params.iy, "EIy (N·m²)")?,
                positive(params.e * params.iz, "EIz (N·m²)")?,
                positive(params.g * params.j, "GJ (N·m²)")?,
            ],
            shear_y: shear(params.ay, "Ay (m²)")?,
            shear_z: shear(params.az, "Az (m²)")?,
        })
    }

    /// Original validated SI properties, immutable.
    pub fn params(&self) -> &ElasticFrameParams {
        &self.params
    }

    /// Axial stiffness EA, N.
    pub fn axial_rigidity(&self) -> Real {
        self.rigidities[0]
    }

    /// Bending stiffness about local y, EIy in N·m².
    pub fn bending_rigidity_y(&self) -> Real {
        self.rigidities[1]
    }

    /// Bending stiffness about local z, EIz in N·m².
    pub fn bending_rigidity_z(&self) -> Real {
        self.rigidities[2]
    }

    /// Saint-Venant torsional stiffness GJ, N·m².
    pub fn torsional_rigidity(&self) -> Real {
        self.rigidities[3]
    }

    /// Optional local y shear stiffness, GAy in N; also available in D2.
    pub fn shear_rigidity_y(&self) -> Option<Real> {
        self.shear_y
    }

    /// Optional local z shear stiffness, GAz in N; also available in D2.
    pub fn shear_rigidity_z(&self) -> Option<Real> {
        self.shear_z
    }
}

impl SectionLaw for ElasticFrame {
    fn kind(&self) -> &'static str {
        ELASTIC_FRAME.kind
    }

    fn contract(&self) -> ContractId {
        ELASTIC_FRAME.contract
    }

    fn dim(&self) -> Dim {
        self.dim
    }

    fn state_len(&self) -> usize {
        0
    }

    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }

    /// Evaluates total e without mutating inputs or accepting history.
    ///
    /// `ENG-E-0010` for incorrect e or history length; `ENG-E-0008`
    /// for nonfinite e or overflowed resultants. Constant symmetric tangent,
    /// empty trial history; units and order as in [`ElasticFrame`].
    fn respond(&self, e: &[Real], committed: &[Real]) -> EResult<SectionResponse> {
        let d2 = [self.axial_rigidity(), self.bending_rigidity_z()];
        let diagonal: &[Real] = match self.dim {
            Dim::D2 => &d2,
            Dim::D3 => &self.rigidities,
        };
        let n = diagonal.len();
        vector(e, n, "Generalized deformations")?;
        vector(committed, 0, "Accepted history")?;
        let mut resultants = Vec::with_capacity(n);
        let mut tangent = vec![0.0; n * n];
        for (i, (&stiffness, &strain)) in diagonal.iter().zip(e).enumerate() {
            resultants.push(finite(stiffness * strain, "Section resultant")?);
            tangent[i * n + i] = stiffness;
        }
        Ok(SectionResponse {
            resultants,
            tangent: DMat::new(n, n, tangent)?,
            trial_state: Vec::new(),
        })
    }
}
