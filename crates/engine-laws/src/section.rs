//! Sección de pórtico elástico `elastic_frame`, contrato `section/1`.
//!
//! Ejes aprobados: terna diestra, X-Y horizontal, Z vertical arriba. D2 se
//! representa en X-Y, Z = 0. El eje local x va de i a j; en D2 el local z
//! apunta a +Z. En D3 la orientación transversal la establece el consumidor.
//! Véase `docs/AXES_CONVENTION.md`; no se redefine el plano de elevación D2.

use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{DMat, SectionLaw, SectionResponse};
use engine_core::{ContractId, Dim, EResult, Real};

/// Descriptor de la sección elástica de pórtico, método 0.1.0.
pub const ELASTIC_FRAME: LawDescriptor = LawDescriptor {
    kind: "elastic_frame",
    contract: ContractId::new("section", 1, 0),
    version: (0, 1, 0),
};

/// Propiedades SI suministradas por el consumidor; sin cálculo geométrico.
///
/// Todas las propiedades requeridas deben ser finitas y positivas, incluso
/// Iy, J y G en D2. Las áreas efectivas de corte se validan solo si existen.
/// G se proporciona explícitamente; no se deduce de E ni de un coeficiente
/// de Poisson. Los ejes y/z deben ser centroidales principales (sin acoplamiento).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElasticFrameParams {
    /// Módulo de Young E, Pa.
    pub e: Real,
    /// Módulo de corte G, Pa.
    pub g: Real,
    /// Área A, m².
    pub area: Real,
    /// Segundo momento Iy = integral de z² sobre el área, m⁴.
    pub iy: Real,
    /// Segundo momento Iz = integral de y² sobre el área, m⁴.
    pub iz: Real,
    /// Constante de torsión de Saint-Venant J, m⁴; no se supone inercia polar.
    pub j: Real,
    /// Área efectiva de corte local y, Ay en m²; `None` indica ausencia.
    pub ay: Option<Real>,
    /// Área efectiva de corte local z, Az en m²; `None` indica ausencia.
    pub az: Option<Real>,
}

/// Fábrica sustituible de secciones, sin ensamblaje ni registro global.
///
/// Implementaciones con el mismo `Params` pueden intercambiarse como objetos
/// del trait; los productos se consumen a través de [`SectionLaw`].
pub trait SectionFactory: Send + Sync {
    /// Parámetros tipados SI de la implementación.
    type Params;
    /// Kind, contrato y versión semántica del método construido.
    fn descriptor(&self) -> LawDescriptor;
    /// Construye para la dimensión pedida; errores según la implementación.
    fn build(&self, dim: Dim, params: Self::Params) -> EResult<Box<dyn SectionLaw>>;
}

/// Fábrica de [`ElasticFrame`] para D2 y D3.
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

/// Sección homogénea lineal, sin acoplamiento, precarga, alabeo ni historia.
///
/// D2: `e = [ε0, κz]`, `s = [N, Mz]`, tangente diagonal `(EA, EIz)`.
/// D3: `e = [ε0, κy, κz, θ′]`, `s = [N, My, Mz, T]`, tangente diagonal
/// `(EA, EIy, EIz, GJ)`. ε0 es adimensional; κy, κz y θ′ en m⁻¹
/// (rad/m para giros). N en N; My, Mz y T en N·m. EA en N; EI y GJ en N·m².
///
/// N positivo es tracción. En la cara +x los momentos positivos siguen la
/// mano derecha alrededor de +y, +z y +x, respectivamente; en la cara −x
/// sus acciones son opuestas. Se adopta εx(y,z) = ε0 + κy z − κz y,
/// conjugada a My = integral de z σx y Mz = integral de −y σx. Curvaturas
/// positivas producen momentos positivos; θ′ es la derivada del giro local
/// alrededor de +x. Esta sección no transforma acciones de extremo o globales.
///
/// Ay/Az solo habilitan los accesores GAy/GAz en N. No añaden deformaciones
/// ni resultantes de corte a `section/1`; un consumidor de corte debe usarlos
/// explícitamente. El rango numérico exige entradas y resultantes finitas.
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
    /// Valida propiedades y productos EA, EIy, EIz, GJ y GAy/GAz existentes.
    ///
    /// # Errores
    /// `ENG-E-0008` por no finitos/desbordamiento; `ENG-E-0010` por valores
    /// no positivos, incluidos productos reducidos a cero por subdesbordamiento.
    /// Se validan todos los productos también en D2, aunque Iy y J no entren
    /// en su tangente. No se aplica tolerancia física ni corrección de corte.
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

    /// Propiedades SI originales validadas, inmutables.
    pub fn params(&self) -> &ElasticFrameParams {
        &self.params
    }

    /// Rigidez axial EA, N.
    pub fn axial_rigidity(&self) -> Real {
        self.rigidities[0]
    }

    /// Rigidez de flexión alrededor del eje local y, EIy en N·m².
    pub fn bending_rigidity_y(&self) -> Real {
        self.rigidities[1]
    }

    /// Rigidez de flexión alrededor del eje local z, EIz en N·m².
    pub fn bending_rigidity_z(&self) -> Real {
        self.rigidities[2]
    }

    /// Rigidez de torsión de Saint-Venant GJ, N·m².
    pub fn torsional_rigidity(&self) -> Real {
        self.rigidities[3]
    }

    /// Rigidez opcional de corte local y, GAy en N; también disponible en D2.
    pub fn shear_rigidity_y(&self) -> Option<Real> {
        self.shear_y
    }

    /// Rigidez opcional de corte local z, GAz en N; también disponible en D2.
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

    /// Evalúa e total sin mutar entradas ni aceptar historia.
    ///
    /// `ENG-E-0010` por longitud de e o historia incorrecta; `ENG-E-0008`
    /// por e no finita o resultantes desbordadas. Tangente simétrica constante,
    /// historia de prueba vacía; unidades y orden según [`ElasticFrame`].
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
