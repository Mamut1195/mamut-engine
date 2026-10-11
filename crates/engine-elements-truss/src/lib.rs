//! Barras homogéneas articuladas `truss2d.linear` y `truss3d.linear`, element/1.
//!
//! Elasticidad lineal, desplazamientos y giros infinitesimales, sin pretensión,
//! historia, flexión ni torsión. El eje d apunta del nodo i al j; N positivo es
//! tracción. D2 usa X-Y horizontal, Z=0; D3 usa la terna diestra, Z hacia arriba.
//! Solo Ux,Uy (D2) o Ux,Uy,Uz (D3), ordenados por nodo. Coordenadas/u en m,
//! E en Pa, A en m², rho en kg/m³, fuerza en N, K/Kg en N/m y masa en kg.
//!
//! Las propiedades validadas son propiedad del elemento; `ctx.coords` aporta
//! la geometría de referencia. `ctx.params` no interviene: no se mezclan dos
//! fuentes de propiedades. La fábrica usa números SI E, A y rho explícitos.
//! Masa isotrópica de traslación con interpolación lineal. Kg se ofrece aparte:
//! no modifica la tangente ni convierte respond en un análisis de segundo orden.
//! Solo cargas uniformes axiales sin momentos; peso propio debe convertirse
//! explícitamente en q=rho A g y ser paralelo al eje. No se presume gravedad.
//! La recuperación devuelve extremos locales [-N_i,0,…,N_j,0,…] y estaciones
//! con N por cara positiva; no necesita orientación transversal D3.
//!
//! Entradas/resultados no finitos: ENG-E-0008; tamaños, parámetros, cargas o
//! estaciones inválidos: ENG-E-0010; L≤1e-12 m: ENG-E-0005. Sin E/S, hilos,
//! reloj ni estado mutable. Determinismo en cada destino; igualdad binaria
//! entre destinos pendiente de ENG-CORE-010.
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
mod recover;
mod stiffness;

use engine_contracts::{Element, ElementCtx, ElementOutput, ElementResponse, MassKind, MemberLoad};
use engine_core::{ContractId, Dim, Dof, EResult, EngineError, Real};
use engine_laws::uniaxial::Elastic;
use engine_linalg::DMat;
pub use factory::TrussFactory;

/// Barra lineal de dos nodos con propiedades SI inmutables y sin historia.
///
/// Ambos kinds comparten la formulación; solo cambian dimensión y GDL activos.
/// Véase la documentación del crate para hipótesis, unidades y cargas admitidas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearTruss {
    dim: Dim,
    material: Elastic,
    area: Real,
    density: Real,
    ea: Real,
}

impl LinearTruss {
    /// Construye con E>0 Pa, A>0 m² y rho≥0 kg/m³, todos finitos.
    ///
    /// ENG-E-0008 por no finitos/productos desbordados; ENG-E-0010 por
    /// rango inválido o EA positivo reducido a cero por subdesbordamiento.
    pub fn new(dim: Dim, e: Real, area: Real, density: Real) -> EResult<Self> {
        let material = Elastic::new(e)?;
        positive(area, "A (m²)")?;
        finite(density, "rho (kg/m³)")?;
        if density < 0.0 {
            return Err(invalid("rho debe ser no negativa (kg/m³)."));
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

    /// Dimensión de la barra; determina dos o tres traslaciones por nodo.
    pub fn dim(&self) -> Dim {
        self.dim
    }

    /// Área A validada, en m².
    pub fn area(&self) -> Real {
        self.area
    }

    /// Densidad rho validada, en kg/m³; cero significa ausencia de masa.
    pub fn density(&self) -> Real {
        self.density
    }

    /// Material elástico uniaxial de engine-laws, con módulo E en Pa.
    pub fn material(&self) -> &Elastic {
        &self.material
    }

    fn geometry(&self, ctx: &ElementCtx<'_>) -> EResult<geometry::Geometry> {
        geometry::Geometry::new(self.dim, ctx)
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
            .map(|b| finite(n * b, "F interna (N)"))
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
