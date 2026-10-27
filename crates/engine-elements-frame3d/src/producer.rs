//! Owned adapter from a spatial beam P-Delta Kg to engine-assembly.
use crate::{invalid, EulerFrame3D};
use engine_assembly::GeometricProducer;
use engine_contracts::{Element, ElementCtx};
use engine_core::{DofIndex, EResult};
use std::collections::BTreeSet;

/// Constructs an owned producer for a D3 spatial beam and its global location.
///
/// Captures the element, coordinates, and a copy of params; reconstructs
/// ElementCtx and calls geometric_stiffness with total global u_e and empty state.
/// Retains no context borrows. location follows node i, node j, with
/// Ux,Uy,Uz,Rx,Ry,Rz in D3; the system validates its global range on registration.
///
/// N=EA(uj−ui)/L along the local axis, positive in tension. Kg is integrated
/// with biaxial Hermite slopes and condensed with elastic releases.
/// Fixed reference geometry; does not modify respond or update the axis.
/// Validates geometry and size here; propagates element errors on each evaluation.
/// ENG-E-0010 for size, ENG-E-0002 for repeated DOFs; range checked on registration.
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
