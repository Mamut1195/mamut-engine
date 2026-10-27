//! Energy ½kδ²: generalized force kδ and Hessian kbbᵀ.
use crate::{finite, vector as validate_vector, LinearSpring, SpringConnection};
use engine_contracts::Element;
use engine_core::{EResult, Real};
use engine_linalg::DMat;

pub(crate) fn force(s: &LinearSpring, u: &[Real], committed: &[Real]) -> EResult<Real> {
    validate_vector(u, s.size(), "u_e")?;
    validate_vector(committed, 0, "Historia")?;
    // k=0 disconnects exactly; avoids products/sums that do not affect output.
    if s.k == 0.0 {
        return Ok(0.0);
    }
    let width = s.dofs_per_node().len();
    let mut delta = 0.0;
    for (i, dof) in s.dofs_per_node().iter().enumerate() {
        let c = s.global[dof.index()];
        if c == 0.0 {
            continue;
        }
        let relative = match s.connection {
            SpringConnection::BetweenNodes => finite(u[width + i] - u[i], "Relative displacement")?,
            SpringConnection::Ground => u[i],
        };
        delta = finite(
            delta + finite(c * relative, "Projection")?,
            "Generalized deformation",
        )?;
    }
    finite(s.k * delta, "Generalized force kδ")
}

pub(crate) fn vector(s: &LinearSpring, force: Real, local: bool) -> EResult<Vec<Real>> {
    let coefficients = if local { &s.local } else { &s.global };
    let mut values = Vec::with_capacity(s.size());
    for node in 0..s.n_nodes() {
        let sign = if s.n_nodes() == 2 && node == 0 {
            -1.0
        } else {
            1.0
        };
        for dof in s.dofs_per_node() {
            values.push(finite(
                sign * force * coefficients[dof.index()],
                "Nodal action",
            )?);
        }
    }
    Ok(values)
}

pub(crate) fn matrix(s: &LinearSpring) -> EResult<DMat> {
    matrix_with_tangent(s, s.k)
}

pub(crate) fn matrix_with_tangent(s: &LinearSpring, tangent: Real) -> EResult<DMat> {
    let b = vector(s, 1.0, false)?;
    let n = b.len();
    let mut values = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..=i {
            let v = finite(
                tangent * finite(b[i] * b[j], "Coupling bbᵀ")?,
                "Tangent stiffness bbᵀ",
            )?;
            values[i * n + j] = v;
            values[j * n + i] = v;
        }
    }
    DMat::new(n, n, values)
}
