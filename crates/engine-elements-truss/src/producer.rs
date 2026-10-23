//! Adaptador poseído de la Kg P-Delta de una barra a engine-assembly.
use crate::{invalid, LinearTruss};
use engine_assembly::GeometricProducer;
use engine_contracts::{Element, ElementCtx};
use engine_core::{DofIndex, EResult};
use std::collections::BTreeSet;

/// Construye un productor poseído para una barra D2/D3 y su location global.
///
/// Captura el elemento, las coordenadas y una copia de params; reconstruye
/// ElementCtx y llama geometric_stiffness con u_e global total y estado vacío.
/// No retiene préstamos del contexto. location sigue nodo i, nodo j, con
/// Ux,Uy en D2 o Ux,Uy,Uz en D3; el sistema valida su rango global al registrar.
///
/// N=(EA/L) d·(u_j−u_i), positivo en tracción, Kg=(N/L)[P,−P;−P,P],
/// P=I−ddᵀ. u/coords en m, EA en N, Kg en N/m. Es un aporte P-Delta con
/// geometría de referencia fija; no cambia respond ni actualiza el eje/longitud.
/// Valida geometría y tamaño aquí; propaga errores del elemento en cada eval.
/// ENG-E-0010 por tamaño, ENG-E-0002 por GDL repetidos; rango en el registro.
pub fn geometric_producer(
    element: LinearTruss,
    ctx: &ElementCtx<'_>,
    location: Vec<DofIndex>,
) -> EResult<GeometricProducer> {
    element.geometry(ctx)?;
    if location.len() != 2 * element.dofs_per_node().len() {
        return Err(invalid("Truss producer: incompatible location size."));
    }
    if location.iter().collect::<BTreeSet<_>>().len() != location.len() {
        return Err(engine_core::EngineError::new(
            "ENG-E-0002",
            "Truss producer: duplicate DOF.",
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
                engine_core::EngineError::new("ENG-E-0013", "Truss did not provide Kg.")
            })
        }),
    })
}
