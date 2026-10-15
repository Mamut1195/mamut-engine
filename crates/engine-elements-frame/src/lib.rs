#![forbid(unsafe_code)]
//! Viga-columna `frame2d.euler_elastic`, contrato element/1, método 0.1.0.
//! [`WinklerFrame2D`] añade un lecho transversal consistente fijo a tierra;
//! [`WinklerFactory`] construye `frame2d.winkler` con propiedades SI.
//!
//! Euler–Bernoulli homogénea, infinitesimal, sin deformación de corte, inercia
//! rotatoria de sección ni historia. D2 está en X-Y horizontal, Z=0; local x
//! de i a j, y=(-sin,cos), z=+Z. Orden `[ux,uy,rz]` por nodo; N positivo en
//! tracción y giros antihorarios. E en Pa, A en m², I=Iz en m⁴, rho en kg/m³;
//! coordenadas/traslaciones m, giros rad, fuerzas N, momentos N·m. K y Kg
//! tienen entradas N/m, N y N·m; M tiene entradas kg, kg·m y kg·m².
//!
//! Propiedades inmutables en self; ctx aporta coordenadas, no propiedades:
//! ctx.params no interviene, siguiendo la plantilla interna de elementos.
//! `respond` es elástico: Kg se entrega aparte con N obtenido de u_e.
//! Cargas uniformes globales XY y momento distribuido alrededor de Z;
//! peso propio requiere gravedad explícita, nunca se supone Y vertical.
//! Extremos recuperados son K u menos cargas equivalentes (acciones sobre
//! el miembro); estaciones `[N,Vy,0,0,0,Mz]` por cara positiva local.
//!
//! Liberaciones sustituyen giros internos mediante equilibrio elástico:
//! Kc=HᵀKH, Mc=HᵀMH (reducción estática de Guyan), Kgc=HᵀKgH,
//! fc=Hᵀf. Kg es la derivada primera de la condensación de K+Kg en N=0;
//! no una condensación exacta dependiente de N a compresión finita.
//! Con dos liberaciones Kflexión=0 exactamente: los giros nodales quedan
//! desconectados. El ensamblador debe restringir/eliminar GDL sin otros
//! aportes; no se añade rigidez artificial. Se recuperan los giros del miembro
//! mediante [`EulerFrame2D::local_displacements`], incluyendo cargas.
//!
//! ENG-E-0005: L≤1e-12 m; ENG-E-0008: no finitos/desbordamientos;
//! ENG-E-0010: rangos, tamaños, plano o estaciones inválidos;
//! ENG-E-0013: cargas fuera del plano. Sin E/S, hilos, reloj ni estado global;
//! determinista por destino; ejecución y equivalencia numérica wasm pendientes.
//! Derivaciones y límites en provenance/ENG-ELM-002.md.
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
pub use release::MomentReleases;
pub use winkler::WinklerFrame2D;
type Matrix = [[Real; 6]; 6];

/// Viga 2D elástica de dos nodos, con propiedades SI y liberaciones propietarias.
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
    /// Valida E,A,I>0 y rho≥0 finitos; I es Iz. Productos EA/EI positivos.
    /// ENG-E-0010 por rango/subdesbordamiento, ENG-E-0008 por no finitos.
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
            return Err(invalid("rho debe ser no negativa."));
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
    /// Material elástico uniaxial, módulo E en Pa.
    pub fn material(&self) -> &Elastic {
        &self.material
    }
    /// Área A en m².
    pub fn area(&self) -> Real {
        self.area
    }
    /// Segundo momento I=Iz en m⁴.
    pub fn inertia(&self) -> Real {
        self.inertia
    }
    /// Densidad rho en kg/m³; cero representa ausencia de masa.
    pub fn density(&self) -> Real {
        self.density
    }
    /// Liberaciones de momento; nunca libera traslaciones.
    pub fn releases(&self) -> MomentReleases {
        self.releases
    }
    /// Construye q=rho A g (N/m) desde aceleración global explícita (m/s²).
    /// D2 admite solo g en XY; ENG-E-0013 por componente Z no nula.
    pub fn self_weight(&self, acceleration: [Real; 3]) -> EResult<MemberLoad> {
        vector(&acceleration, 3, "g (m/s²)")?;
        if acceleration[2] != 0.0 {
            return Err(unsupported("Peso propio fuera del plano XY."));
        }
        let line_mass = if self.density == 0.0 {
            0.0
        } else {
            positive(self.density * self.area, "rho A (kg/m)")?
        };
        let mut force_per_length = [0.0; 3];
        for (q, g) in force_per_length.iter_mut().zip(acceleration) {
            *q = finite(line_mass * g, "Peso propio (N/m)")?;
        }
        Ok(MemberLoad {
            force_per_length,
            moment_per_length: [0.0; 3],
        })
    }
    /// `[u_i,v_i,theta_i,u_j,v_j,theta_j]` locales, m/rad; recupera giros
    /// liberados por equilibrio de K, incluyendo todas las cargas uniformes.
    /// Los giros nodales suministrados en extremos liberados no intervienen.
    /// Errores de geometría, tamaños, cargas y finitud según el crate.
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
