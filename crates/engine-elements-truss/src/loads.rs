//! Global uniform axial forces; explicit rejection of moments/transverse loads.
//!
//! Uniform temperature on a bar fixed at both ends is a separate axial resultant.
//! It is not a `MemberLoad`: a force per length cannot keep N constant and
//! independent of L. A thermal gradient is not represented.
use crate::{finite, geometry::Geometry, invalid, vector as check_vector};
use engine_contracts::MemberLoad;
use engine_core::{EResult, Real};

pub(crate) fn axial(g: &Geometry, load: &MemberLoad) -> EResult<Real> {
    check_vector(&load.force_per_length, 3, "q (N/m)")?;
    check_vector(&load.moment_per_length, 3, "Moment per length (N)")?;
    if load.moment_per_length != [0.0; 3] || (g.axes == 2 && load.force_per_length[2] != 0.0) {
        return Err(invalid(
            "Truss: moments or out-of-plane loads are not allowed.",
        ));
    }
    let mut q = 0.0;
    for axis in 0..g.axes {
        q = finite(q + g.d[axis] * load.force_per_length[axis], "q axial (N/m)")?;
    }
    // Relative projection tolerance: rounding only, without an absolute threshold
    // that would allow a small transverse load to be omitted.
    let magnitude = load
        .force_per_length
        .iter()
        .fold(0.0_f64, |m, v| m.max(v.abs()));
    for axis in 0..g.axes {
        let residual = finite(
            load.force_per_length[axis] - q * g.d[axis],
            "Transverse q (N/m)",
        )?;
        if residual.abs() > 64.0 * f64::EPSILON * magnitude {
            return Err(invalid(
                "Truss: only uniform load parallel to the axis, with no transverse component.",
            ));
        }
    }
    Ok(q)
}
pub(crate) fn vector(g: &Geometry, load: &MemberLoad) -> EResult<Vec<Real>> {
    let half = finite(axial(g, load)? * (g.length / 2.0), "Equivalent load (N)")?;
    (0..2 * g.axes)
        .map(|i| finite(half * g.d[i % g.axes], "Equivalent F (N)"))
        .collect()
}

/// Tension-positive axial force when both ends are fixed and ΔT is uniform.
///
/// N = -(E A α ΔT). The product E A α ΔT is the magnitude of the prevented
/// expansion. Length and direction do not enter. No gradient term is accepted.
pub(crate) fn fixed_end_uniform_temperature(
    ea: Real,
    alpha_per_kelvin: Real,
    delta_t_kelvin: Real,
) -> EResult<Real> {
    finite(alpha_per_kelvin, "α (1/K)")?;
    finite(delta_t_kelvin, "ΔT (K)")?;
    let expansion = finite(alpha_per_kelvin * delta_t_kelvin, "αΔT")?;
    let prevented = finite(ea * expansion, "EA αΔT (N)")?;
    // Tension positive: prevented expansion is compression.
    finite(-prevented, "N (N)")
}

/// Global joint forces of that restrained state, in `load_vector` order.
///
/// f = -N b with b = [-d, d]. Heating (N < 0) pushes the joints outward.
pub(crate) fn fixed_end_uniform_temperature_forces(
    g: &Geometry,
    axial_force: Real,
) -> EResult<Vec<Real>> {
    g.b()
        .into_iter()
        .map(|component| finite(-axial_force * component, "Equivalent F (N)"))
        .collect()
}
