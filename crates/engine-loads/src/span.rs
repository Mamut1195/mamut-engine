//! Integración exacta de interpolaciones axial lineal y transversal cúbica.
use crate::{error, finite, length};
use engine_contracts::MemberLoad;
use engine_core::math;
use engine_core::{EResult, Real};

/// Ejes de las dos componentes de fuerza (o fuerza por longitud).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadAxes {
    /// Componentes `[x,y]` locales del miembro orientado i → j.
    Local,
    /// Componentes `[X,Y]` globales del plano XY.
    Global,
}

/// Geometría inmutable de la viga 2D, independiente de rigidez y sección.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beam2D {
    length: Real,
    c: Real,
    s: Real,
}

impl Beam2D {
    /// Valida longitud > 0 y ángulo finito (rad desde +X hacia +Y).
    /// No se impone una tolerancia dimensional: también admite L=1e-300 m.
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

    /// Longitud del miembro en m.
    pub fn length(&self) -> Real {
        self.length
    }

    /// Transforma fuerzas nodales locales a globales; los momentos conservan Z.
    /// No aplica liberaciones. ENG-E-0008 para entradas o resultados no finitos.
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

/// Fuerzas de tramo, sin momentos distribuidos; componentes axiales y transversales.
/// Las distancias se miden sobre la longitud real, incluso en ejes globales.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpanLoad {
    /// Carga constante sobre toda la longitud.
    Uniform {
        /// Componentes `[qx,qy]` en N/m según axes.
        q: [Real; 2],
        /// Sistema de ejes de q.
        axes: LoadAxes,
    },
    /// Carga lineal entre a y b; cero fuera de ese intervalo.
    Trapezoidal {
        /// Intensidad en a (N/m), componentes según axes.
        q_i: [Real; 2],
        /// Intensidad en b (N/m), componentes según axes.
        q_j: [Real; 2],
        /// Inicio desde i (m); None equivale a 0.
        a: Option<Real>,
        /// Fin desde i (m); None equivale a L. Exige 0 ≤ a < b ≤ L.
        b: Option<Real>,
        /// Sistema de ejes de las intensidades.
        axes: LoadAxes,
    },
    /// Fuerza puntual, incluidos los extremos x=0 y x=L.
    Point {
        /// Distancia desde i en m, 0 ≤ x ≤ L.
        x: Real,
        /// Componentes de P en N según axes.
        p: [Real; 2],
        /// Sistema de ejes de P.
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

// Hermite factorizado en r=x/L evita dividir por L²/L³ y reduce cancelaciones.
// Se multiplica por L al final para no formar L² antes de combinar la carga.
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

/// Fuerzas nodales equivalentes locales ∫ Hᵀq dx o H(x)ᵀP.
///
/// Integración analítica mediante Gauss–Legendre de 3 puntos, exacta hasta
/// grado 5: H es cúbica y q lineal, por tanto Hᵀq tiene grado ≤4. No hay
/// subdivisiones, tolerancias de cuadratura ni integración aproximada. Los
/// puntos en `[0,1]` son (1±√(3/5))/2 y 1/2, con pesos 5/18, 4/9, 5/18.
/// La exactitud es algebraica, sujeta al redondeo de Real=f64.
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

/// Reacciones locales sobre el miembro empotrado: −∫ Hᵀq dx o −H(x)ᵀP.
/// Para qy>0 uniforme: `[0,−qL/2,−qL²/12,0,−qL/2,+qL²/12]`.
/// Comparte validación e integración exacta con [`equivalent_nodal_forces_local`].
pub fn fixed_end_forces_local(beam: &Beam2D, load: &SpanLoad) -> EResult<[Real; 6]> {
    Ok(equivalent_nodal_forces_local(beam, load)?.map(|f| -f))
}

/// Convierte una carga uniforme completa a [`MemberLoad`] en ejes globales.
/// También acepta un trapecio completo con q_i=q_j; nunca promedia una carga
/// variable o parcial (ENG-E-0013). Valida primero todas las entradas.
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
