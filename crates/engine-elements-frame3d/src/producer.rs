//! Adaptador poseído de la Kg P-Delta de una viga espacial a engine-assembly.
use crate::{invalid, EulerFrame3D};
use engine_assembly::GeometricProducer;
use engine_contracts::{Element, ElementCtx};
use engine_core::{DofIndex, EResult};
use std::collections::BTreeSet;

/// Construye un productor poseído para una viga espacial D3 y su location global.
///
/// Captura el elemento, las coordenadas y una copia de params; reconstruye
/// ElementCtx y llama geometric_stiffness con u_e global total y estado vacío.
/// No retiene préstamos del contexto. location sigue nodo i, nodo j, con
/// Ux,Uy,Uz,Rx,Ry,Rz en D3; el sistema valida su rango global al registrar.
///
/// N=EA(uj−ui)/L en el eje local, positivo en tracción. Kg se integra
/// con pendientes Hermite biaxiales y se condensa con liberaciones elásticas.
/// Geometría de referencia fija; no modifica respond ni actualiza el eje.
/// Valida geometría y tamaño aquí; propaga errores del elemento en cada eval.
/// ENG-E-0010 por tamaño, ENG-E-0002 por GDL repetidos; rango en el registro.
pub fn geometric_producer(
    element: EulerFrame3D,
    ctx: &ElementCtx<'_>,
    location: Vec<DofIndex>,
) -> EResult<GeometricProducer> {
    crate::geometry::Geometry::new(ctx, element.orientation())?;
    if location.len() != 2 * element.dofs_per_node().len() {
        return Err(invalid(
            "Spatial beam producer: incompatible location size.",
        ));
    }
    if location.iter().collect::<BTreeSet<_>>().len() != location.len() {
        return Err(engine_core::EngineError::new(
            "ENG-E-0002",
            "Spatial beam producer: duplicate DOF.",
        ));
    }
    let dim = ctx.dim;
    let coords = ctx.coords.to_vec();
    let params = ctx.params.clone();
    Ok(GeometricProducer {
        location,
        eval: Box::new(move |u_e| {
            let ctx = ElementCtx {
                dim,
                coords: &coords,
                params: &params,
            };
            element.geometric_stiffness(&ctx, u_e, &[])?.ok_or_else(|| {
                engine_core::EngineError::new("ENG-E-0013", "Spatial beam did not provide Kg.")
            })
        }),
    })
}
