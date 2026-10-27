//! ε=d·(u_j−u_i)/L; N=Aσ; elastic Hessian (EA/L)bbᵀ.
use crate::{finite, geometry::Geometry, positive, vector, LinearTruss};
use engine_contracts::UniaxialLaw;
use engine_core::{EResult, Real};
use engine_linalg::DMat;

pub(crate) fn axial(
    bar: &LinearTruss,
    g: &Geometry,
    u: &[Real],
    committed: &[Real],
) -> EResult<Real> {
    vector(u, 2 * g.axes, "u_e (m)")?;
    vector(committed, 0, "Accepted history")?;
    let mut extension = 0.0;
    for axis in 0..g.axes {
        let relative = finite(u[g.axes + axis] - u[axis], "Relative displacement (m)")?;
        extension = finite(extension + g.d[axis] * relative, "Elongation (m)")?;
    }
    let strain = finite(extension / g.length, "ε")?;
    // Assumption: stress is evaluated with the uniaxial law at strain ε,
    // but only `response.stress` is used; the law tangent is unused.
    // Stiffness `matrix` uses constant EA/L (linear truss), so N and K are
    // consistent only if the law is linear elastic with modulus E = ea/area. A
    // nonlinear law would require using the tangent (outside the scope of ENG-ELM-001).
    // `committed` is the accepted history and is not modified here.
    let response = bar.material.respond(strain, committed)?;
    finite(bar.area * response.stress, "N (N)")
}
pub(crate) fn matrix(bar: &LinearTruss, g: &Geometry) -> EResult<DMat> {
    let k = positive(bar.ea / g.length, "EA/L (N/m)")?;
    let b = g.b();
    let n = b.len();
    let mut values = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..=i {
            let value = k * (b[i] * b[j]);
            values[i * n + j] = value;
            values[j * n + i] = value;
        }
    }
    DMat::new(n, n, values)
}
