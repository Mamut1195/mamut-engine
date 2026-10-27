//! Owned adapter from a truss P-Delta Kg to engine-assembly.
use crate::{invalid, LinearTruss};
use engine_assembly::GeometricProducer;
use engine_contracts::{Element, ElementCtx};
use engine_core::{DofIndex, EResult};
use std::collections::BTreeSet;

/// Constructs an owned producer for a D2/D3 truss and its global location.
///
/// Captures the element, coordinates, and a copy of params; reconstructs
/// ElementCtx and calls geometric_stiffness with total global u_e and empty state.
/// Retains no context borrows. location follows node i, node j, with
/// Ux,Uy in D2 or Ux,Uy,Uz in D3; the system validates its global range on registration.
///
/// N=(EA/L) d·(u_j−u_i), positive in tension, Kg=(N/L)[P,−P;−P,P],
/// P=I−ddᵀ. u/coords in m, EA in N, Kg in N/m. This is a P-Delta contribution with
/// fixed reference geometry; it does not change respond or update the axis/length.
/// Validates geometry and size here; propagates element errors on each evaluation.
/// ENG-E-0010 for size, ENG-E-0002 for repeated DOFs; range checked on registration.
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
