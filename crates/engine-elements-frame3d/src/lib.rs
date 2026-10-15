#![forbid(unsafe_code)]
//! Viga-columna espacial `frame3d.euler_elastic`, contrato element/1, versión 0.1.0.
//! [`WinklerFrame3D`] añade un lecho transversal consistente fijo a tierra;
//! [`WinklerFactory`] construye `frame3d.winkler` con propiedades SI.
//!
//! Euler–Bernoulli homogénea, infinitesimal, con torsión de Saint-Venant.
//! Terna global diestra X-Y horizontal y Z arriba. Orden por nodo
//! `[Ux,Uy,Uz,Rx,Ry,Rz]`; x local apunta de i a j. La orientación es una
//! referencia de z local: se proyecta normal a x y se toma y=z×x, z=x×y.
//! Giros según mano derecha; v′=rz, w′=−ry; N positivo en tracción.
//! E,G en Pa; A en m²; J,Iy,Iz en m⁴; rho en kg/m³; longitud en m.
//! J es la constante torsional, distinta de la inercia polar Iy+Iz.
//! Fuerzas en N, momentos en N·m, giros en rad. Propiedades en self,
//! coordenadas en ctx; ctx.params no interviene, como en frame2d.
//!
//! `respond` entrega K elástica y K u, sin cargas ni Kg. Kg se obtiene
//! separadamente del axial N=EA(uj−ui)/L y geometría de referencia fija.
//! Liberaciones en rx,ry,rz locales: Schur elástico analítico; masa y Kg
//! proyectadas con HᵀMH y HᵀKgH (Guyan y derivada del Schur en N=0).
//! No es la condensación de K+Kg a N finito. Giros nodales liberados quedan
//! desconectados; el ensamblador debe eliminar/restringir mecanismos.
//! Torsión doblemente liberada: giro común interno fijado a cero (elección
//! de referencia sin energía); un torsor distribuido neto requiere equilibrio
//! imposible y produce ENG-E-0006. No se agrega rigidez artificial.
//!
//! Masa transversal Hermite sin inercia rotatoria de flexión; masa torsional
//! lineal con rho(Iy+Iz). Masa concentrada: rho A L/2 por traslación y
//! rho(Iy+Iz)L/2 en rx. rho=0 devuelve None tras validar la geometría.
//! MemberLoad es global; [`EulerFrame3D::global_load`] convierte intensidades
//! locales al contrato. Empotramiento perfecto = menos la carga equivalente.
//! Recuperación por equilibrio: `[N,Vy,Vz,T,My,Mz]` en cara positiva local.
//!
//! ENG-E-0005 por L≤1e-12; ENG-E-0008 por no finitos/desbordamientos;
//! ENG-E-0010 por rangos/tamaños/orientación degenerada. Sin E/S, hilos,
//! reloj, estado global ni historia; determinista por destino. Ejecución
//! numérica wasm pendiente. Derivaciones en provenance/ENG-ELM-003.md.
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
// Pares Hermite: v′=rz y w′=−ry.
const PLANES: [([usize; 4], Real); 2] = [([1, 5, 7, 11], 1.0), ([2, 4, 8, 10], -1.0)];

/// Viga 3D elástica de dos nodos, propiedades SI y liberaciones locales.
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
    /// Valida E,G,A,J,Iy,Iz>0, rho≥0 y orientación finita no nula.
    /// La condición no paralela a x se valida cuando se conocen coordenadas.
    /// Productos positivos; ENG-E-0010 por rango y ENG-E-0008 por no finitos.
    #[allow(clippy::too_many_arguments)] // Propiedades SI explícitas, patrón de frame2d.
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
            return Err(invalid("rho debe ser no negativa."));
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
    /// Material elástico uniaxial, módulo E en Pa.
    pub fn material(&self) -> &Elastic {
        &self.material
    }
    /// Módulo de corte G en Pa, usado solo en torsión.
    pub fn shear_modulus(&self) -> Real {
        self.shear_modulus
    }
    /// Área A en m².
    pub fn area(&self) -> Real {
        self.area
    }
    /// Constante J de Saint-Venant en m⁴.
    pub fn torsion_constant(&self) -> Real {
        self.torsion_constant
    }
    /// Momentos de área `[Iy,Iz]` en m⁴.
    pub fn inertias(&self) -> [Real; 2] {
        [self.iy, self.iz]
    }
    /// Densidad rho en kg/m³.
    pub fn density(&self) -> Real {
        self.density
    }
    /// Referencia global de z local, sin normalizar.
    pub fn orientation(&self) -> [Real; 3] {
        self.orientation
    }
    /// Liberaciones de rx,ry,rz en cada extremo local.
    pub fn releases(&self) -> MomentReleases {
        self.releases
    }
    /// Construye q=rho A g (N/m) con aceleración global explícita (m/s²).
    pub fn self_weight(&self, acceleration: [Real; 3]) -> EResult<MemberLoad> {
        vector(&acceleration, 3, "g (m/s²)")?;
        let line_mass = if self.density == 0.0 {
            0.0
        } else {
            positive(self.density * self.area, "rho A")?
        };
        let mut force_per_length = [0.0; 3];
        for (q, g) in force_per_length.iter_mut().zip(acceleration) {
            *q = finite(line_mass * g, "Peso propio")?;
        }
        Ok(MemberLoad {
            force_per_length,
            moment_per_length: [0.0; 3],
        })
    }
    /// Convierte intensidades locales de fuerza y momento a un MemberLoad global.
    /// Usa la misma terna que K; rechaza geometría y cargas inválidas.
    pub fn global_load(&self, ctx: &ElementCtx<'_>, local: &MemberLoad) -> EResult<MemberLoad> {
        let g = geometry::Geometry::new(ctx, self.orientation)?;
        Ok(MemberLoad {
            force_per_length: g.global_triplet(local.force_per_length)?,
            moment_per_length: g.global_triplet(local.moment_per_length)?,
        })
    }
    /// Desplazamientos del miembro en orden local de 12 GDL, m/rad.
    /// Recupera giros liberados con cargas globales; ignora sus giros nodales.
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
            format!("{name}: entrada o resultado no finito."),
        ));
    }
    Ok(value)
}
fn positive(value: Real, name: &str) -> EResult<Real> {
    finite(value, name)?;
    if value <= 0.0 {
        return Err(invalid(&format!("{name}: debe ser positivo.")));
    }
    Ok(value)
}
fn vector(values: &[Real], size: usize, name: &str) -> EResult<()> {
    if values.len() != size {
        return Err(invalid(&format!(
            "{name}: tamaño {}, esperado {size}.",
            values.len()
        )));
    }
    for &v in values {
        finite(v, name)?;
    }
    Ok(())
}
