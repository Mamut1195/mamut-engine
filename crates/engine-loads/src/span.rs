//! Exact integration of linear axial and cubic transverse interpolations.
use crate::{error, finite, length};
use engine_contracts::MemberLoad;
use engine_core::math;
use engine_core::{EResult, Real};

/// Axes for the two force (or force per unit length) components.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadAxes {
    /// Local `[x,y]` components of the member oriented i → j.
    Local,
    /// Global `[X,Y]` components in the XY plane.
    Global,
}

/// Immutable 2D beam geometry, independent of stiffness and section.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beam2D {
    length: Real,
    c: Real,
    s: Real,
}

impl Beam2D {
    /// Validates length > 0 and finite angle (rad from +X toward +Y).
    /// No dimensional tolerance is imposed: L=1e-300 m is also accepted.
    pub fn new(member_length: Real, angle: Real) -> EResult<Self> {
        length(member_length)?;
        finite(&[angle])?;
        let (s, c) = math::sin_cos(angle);
        finite(&[s, c])?;
        Ok(Self {
            length: member_length,
            c,
            s,
        })
    }

    /// Member length in m.
    pub fn length(&self) -> Real {
        self.length
    }

    /// Transforms local nodal forces to global axes; moments retain Z.
    /// Does not apply releases. ENG-E-0008 for nonfinite inputs or results.
    pub fn forces_to_global(&self, local: [Real; 6]) -> EResult<[Real; 6]> {
        finite(&local)?;
        let mut global = local;
        for i in [0, 3] {
            global[i] = self.c * local[i] - self.s * local[i + 1];
            global[i + 1] = self.s * local[i] + self.c * local[i + 1];
        }
        finite(&global)?;
        Ok(global)
    }

    fn local(&self, q: [Real; 2], axes: LoadAxes) -> EResult<[Real; 2]> {
        finite(&q)?;
        let result = match axes {
            LoadAxes::Local => q,
            LoadAxes::Global => [
                self.c * q[0] + self.s * q[1],
                -self.s * q[0] + self.c * q[1],
            ],
        };
        finite(&result)?;
        Ok(result)
    }
}

/// Span forces, without distributed moments; axial and transverse components.
/// Distances are measured along the actual length, even in global axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpanLoad {
    /// Constant load over the entire length.
    Uniform {
        /// `[qx,qy]` components in N/m according to axes.
        q: [Real; 2],
        /// Axis system for q.
        axes: LoadAxes,
    },
    /// Linear load between a and b; zero outside that interval.
    Trapezoidal {
        /// Intensity at a (N/m), components according to axes.
        q_i: [Real; 2],
        /// Intensity at b (N/m), components according to axes.
        q_j: [Real; 2],
        /// Start measured from i (m); None is equivalent to 0.
        a: Option<Real>,
        /// End measured from i (m); None is equivalent to L. Requires 0 ≤ a < b ≤ L.
        b: Option<Real>,
        /// Axis system for the intensities.
        axes: LoadAxes,
    },
    /// Point force, including the ends x=0 and x=L.
    Point {
        /// Distance from i in m, 0 ≤ x ≤ L.
        x: Real,
        /// Components of P in N according to axes.
        p: [Real; 2],
        /// Axis system for P.
        axes: LoadAxes,
    },
}

enum LocalLoad {
    Distributed {
        qi: [Real; 2],
        qj: [Real; 2],
        a: Real,
        b: Real,
    },
    Point {
        x: Real,
        p: [Real; 2],
    },
}

fn local_load(beam: &Beam2D, load: &SpanLoad) -> EResult<LocalLoad> {
    let l = beam.length;
    match *load {
        SpanLoad::Uniform { q, axes } => {
            let q = beam.local(q, axes)?;
            Ok(LocalLoad::Distributed {
                qi: q,
                qj: q,
                a: 0.0,
                b: l,
            })
        }
        SpanLoad::Trapezoidal {
            q_i,
            q_j,
            a,
            b,
            axes,
        } => {
            let a = a.unwrap_or(0.0);
            let b = b.unwrap_or(l);
            finite(&[a, b])?;
            let qi = beam.local(q_i, axes)?;
            let qj = beam.local(q_j, axes)?;
            if a < 0.0 || a >= b || b > l {
                return Err(error("ENG-E-0010", "Load interval: 0 ≤ a < b ≤ L."));
            }
            Ok(LocalLoad::Distributed { qi, qj, a, b })
        }
        SpanLoad::Point { x, p, axes } => {
            finite(&[x])?;
            let p = beam.local(p, axes)?;
            if x < 0.0 || x > l {
                return Err(error("ENG-E-0010", "Point position is outside [0,L]."));
            }
            Ok(LocalLoad::Point { x, p })
        }
    }
}

// Hermite form factored in r=x/L avoids division by L²/L³ and reduces cancellation.
// Multiplication by L is deferred to avoid forming L² before combining the load.
fn point_vector(l: Real, r: Real, [px, py]: [Real; 2]) -> EResult<[Real; 6]> {
    let t = 1.0 - r;
    let result = [
        px * t,
        py * (t * t * (1.0 + 2.0 * r)),
        (py * (r * t * t)) * l,
        px * r,
        py * (r * r * (1.0 + 2.0 * t)),
        -(py * (r * r * t)) * l,
    ];
    finite(&result)?;
    Ok(result)
}

/// Local equivalent nodal forces ∫ Hᵀq dx or H(x)ᵀP.
///
/// Analytical integration using 3-point Gauss–Legendre, exact through
/// degree 5: H is cubic and q is linear, so Hᵀq has degree ≤4. There are no
/// subdivisions, quadrature tolerances, or approximate integration. The
/// points in `[0,1]` are (1±√(3/5))/2 and 1/2, with weights 5/18, 4/9, 5/18.
/// Exactness is algebraic, subject to Real=f64 rounding.
pub fn equivalent_nodal_forces_local(beam: &Beam2D, load: &SpanLoad) -> EResult<[Real; 6]> {
    match local_load(beam, load)? {
        LocalLoad::Point { x, p } => point_vector(beam.length, x / beam.length, p),
        LocalLoad::Distributed { qi, qj, a, b } if qi == qj && a == 0.0 && b == beam.length => {
            let axial = qi[0] * (beam.length / 2.0);
            let shear = qi[1] * (beam.length / 2.0);
            let moment = (qi[1] * (beam.length / 12.0)) * beam.length;
            let result = [axial, shear, moment, axial, shear, -moment];
            finite(&result)?;
            Ok(result)
        }
        LocalLoad::Distributed { qi, qj, a, b } => {
            let d = math::sqrt(3.0_f64 / 5.0);
            let points = [
                (0.5 * (1.0 - d), 5.0 / 18.0),
                (0.5, 4.0 / 9.0),
                (0.5 * (1.0 + d), 5.0 / 18.0),
            ];
            let mut result = [0.0; 6];
            for (t, weight) in points {
                let r = (a / beam.length) * (1.0 - t) + (b / beam.length) * t;
                let dx = (b - a) * weight;
                let p = [
                    (qi[0] * (1.0 - t) + qj[0] * t) * dx,
                    (qi[1] * (1.0 - t) + qj[1] * t) * dx,
                ];
                finite(&p)?;
                for (out, value) in result.iter_mut().zip(point_vector(beam.length, r, p)?) {
                    *out += value;
                }
                finite(&result)?;
            }
            Ok(result)
        }
    }
}

/// Local reactions on the member with restrained ends: −∫ Hᵀq dx or −H(x)ᵀP.
/// For uniform qy>0: `[0,−qL/2,−qL²/12,0,−qL/2,+qL²/12]`.
/// Shares validation and exact integration with [`equivalent_nodal_forces_local`].
pub fn fixed_end_forces_local(beam: &Beam2D, load: &SpanLoad) -> EResult<[Real; 6]> {
    Ok(equivalent_nodal_forces_local(beam, load)?.map(|f| -f))
}

/// Converts a full uniform load to [`MemberLoad`] in global axes.
/// Also accepts a full trapezoid with q_i=q_j; never averages a
/// variable or partial load (ENG-E-0013). Validates all inputs first.
pub fn to_member_load(beam: &Beam2D, load: &SpanLoad) -> EResult<MemberLoad> {
    local_load(beam, load)?;
    let (q, axes) = match *load {
        SpanLoad::Uniform { q, axes } => (q, axes),
        SpanLoad::Trapezoidal {
            q_i,
            q_j,
            a,
            b,
            axes,
        } if q_i == q_j && a.unwrap_or(0.0) == 0.0 && b.unwrap_or(beam.length) == beam.length => {
            (q_i, axes)
        }
        _ => {
            return Err(error(
                "ENG-E-0013",
                "MemberLoad requires a full uniform load.",
            ))
        }
    };
    let force_per_length = match axes {
        LoadAxes::Global => [q[0], q[1], 0.0],
        LoadAxes::Local => [
            beam.c * q[0] - beam.s * q[1],
            beam.s * q[0] + beam.c * q[1],
            0.0,
        ],
    };
    finite(&force_per_length)?;
    Ok(MemberLoad {
        force_per_length,
        moment_per_length: [0.0; 3],
    })
}
