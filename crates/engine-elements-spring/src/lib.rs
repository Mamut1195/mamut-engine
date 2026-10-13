//! Resorte `spring.linear`, contrato element/1, sin historia ni pretensión.
//!
//! La API directa [`LinearSpring::new`] admite uno o varios GDL locales de la
//! misma familia (traslaciones o giros), que comparten una rigidez k≥0:
//! δ es la SUMA de sus desplazamientos relativos, energía
//! W=½kδ². Esto acopla los GDL; resortes independientes requieren elementos
//! separados. Entre nodos δ=c·(u_j−u_i); a tierra δ=c·u_i. Para un GDL,
//! `K=k[[1,−1],[−1,1]]` entre nodos, `K=[k]` a tierra. Fuerza interna = ∂W/∂u.
//! Ejes fijos globales o explícitos, nunca inferidos de las coordenadas.
//! Dirección normalizada para el GDL local Ux; una rotación usa filas como
//! ejes locales expresados en globales. D2 está en XY, Z=0; D3 es diestra.
//! La fábrica [`SpringFactory`] y su esquema JSON admiten exactamente un GDL
//! local de la dimensión seleccionada. Se rechazan listas vacías o con más GDL
//! que la dimensión; varios GDL distintos válidos producen ENG-E-0013.
//!
//! Traslaciones m, fuerzas N, k N/m; giros rad, momentos N·m, k N·m/rad.
//! No se mezclan traslaciones y giros sin una escala dimensional explícita.
//! Propiedades validadas en self; ctx.params no interviene. Se admiten nodos
//! coincidentes: no hay longitud física, carga de tramo, masa ni Kg.
//! Masa/Kg devuelven None tras validar las entradas; carga nula devuelve cero,
//! cualquier carga de tramo no nula se rechaza. La recuperación entrega acciones
//! locales por nodo y resultantes constantes `[N,Vy,Vz,T,My,Mz]` en estaciones.
//! A tierra, el único nodo representa el extremo móvil j; su acción es +kδ.
//!
//! El eje es fijo: un giro rígido que cambia su proyección puede deformar un
//! resorte entre nodos separados. Las traslaciones uniformes sí son modos nulos.
//! Un apoyo a tierra elimina el modo rígido correspondiente. La covarianza
//! exige rotar también el eje propio; límites del kit general en la bitácora.
//! ENG-E-0008: no finitos/desbordamientos; ENG-E-0010: rango, tamaño, dirección
//! nula, rotación no ortonormal, GDL o carga no admitidos. Sin E/S, hilos,
//! reloj ni estado mutable. Determinismo por destino, sin promesa bit a bit
//! entre destinos ni regularización de rigideces diminutas.
//!
//! ```
//! use engine_elements_spring::{LinearSpring, SpringAxes, SpringConnection};
//! use engine_contracts::{Element, ElementCtx, ElementParams};
//! use engine_core::{Dim, Dof};
//! let spring = LinearSpring::new(Dim::D2, SpringConnection::Ground,
//!     200.0, &[Dof::Ux], SpringAxes::Global)?;
//! let params = ElementParams::default();
//! let coords = [[0.0; 3]];
//! let ctx = ElementCtx { dim: Dim::D2, coords: &coords, params: &params };
//! assert_eq!(spring.respond(&ctx, &[0.06], &[])?.internal_force, vec![12.0]);
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
use engine_linalg::DMat;
pub use factory::SpringFactory;
pub use geometry::SpringAxes;

/// Conexión del resorte; el orden nodal entre extremos es i, j.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpringConnection {
    /// Dos nodos, acciones iguales y contrarias.
    BetweenNodes,
    /// Un nodo móvil conectado a tierra inmóvil.
    Ground,
}

/// Resorte lineal de rango uno con propiedades SI inmutables.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearSpring {
    dim: Dim,
    connection: SpringConnection,
    k: Real,
    local: [Real; 6],
    global: [Real; 6],
    mask: usize,
}

impl LinearSpring {
    /// Construye con k≥0 finita y GDL distintos, no vacíos, de una misma familia.
    /// Los GDL se suman con coeficiente uno en el sistema seleccionado.
    /// Direction exige únicamente Ux local. Rotation expande los GDL globales
    /// activos a toda la familia para conservar la covarianza; Global conserva
    /// el subconjunto seleccionado. Orden de salida: el canónico de engine-core.
    pub fn new(
        dim: Dim,
        connection: SpringConnection,
        k: Real,
        dofs: &[Dof],
        axes: SpringAxes,
    ) -> EResult<Self> {
        finite(k, "k")?;
        if k < 0.0 {
            return Err(invalid("Resorte: k debe ser no negativa."));
        }
        let (local, global, mask) = geometry::coupling(dim, dofs, axes)?;
        Ok(Self {
            dim,
            connection,
            k,
            local,
            global,
            mask,
        })
    }

    /// Dimensión validada del resorte.
    pub fn dim(&self) -> Dim {
        self.dim
    }
    /// Conexión entre nodos o a tierra.
    pub fn connection(&self) -> SpringConnection {
        self.connection
    }
    /// Rigidez k validada, N/m o N·m/rad según la familia.
    pub fn stiffness(&self) -> Real {
        self.k
    }

    fn size(&self) -> usize {
        self.n_nodes() * self.dofs_per_node().len()
    }
}

impl Element for LinearSpring {
    fn kind(&self) -> &'static str {
        "spring.linear"
    }
    fn contract(&self) -> ContractId {
        ContractId::new("element", 1, 0)
    }
    fn n_nodes(&self) -> usize {
        match self.connection {
            SpringConnection::BetweenNodes => 2,
            SpringConnection::Ground => 1,
        }
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        geometry::dofs(self.mask)
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        geometry::validate(self, ctx)?;
        let force = stiffness::force(self, u_e, committed)?;
        Ok(ElementResponse {
            internal_force: stiffness::vector(self, force, false)?,
            tangent: stiffness::matrix(self)?,
            trial_state: vec![],
        })
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        mass::matrix(self, ctx, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        geometric::matrix(self, ctx, u_e, committed)
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        geometry::validate(self, ctx)?;
        loads::validate(load)?;
        Ok(vec![0.0; self.size()])
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput> {
        recover::output(self, ctx, u_e, committed, loads, stations)
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
fn vector(values: &[Real], size: usize, name: &str) -> EResult<()> {
    if values.len() != size {
        return Err(invalid(&format!(
            "{name}: tamaño {}, esperado {size}.",
            values.len()
        )));
    }
    for &value in values {
        finite(value, name)?;
    }
    Ok(())
}
