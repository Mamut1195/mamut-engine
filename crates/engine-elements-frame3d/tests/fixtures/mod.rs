//! Fixtures propios SI y herramientas del backend interno; fuentes externas: ninguna.
#![allow(dead_code)] // Helpers compartidos por dos binarios de integración.
use engine_contracts::{ElementCtx, ElementParams, JsonValue, MemberLoad};
use engine_core::Dim;
use engine_elements_frame3d::{EulerFrame3D, MomentReleases};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend};
use std::collections::BTreeMap;

pub const SOURCE: &str =
    "analítico: energía Euler–Bernoulli y equilibrio espacial derivados en esta sesión";
pub const CITATION: &str = "provenance/ENG-ELM-003.md, Derivaciones propias (sin fuentes externas)";
pub const E: f64 = 200.0;
pub const G: f64 = 80.0;
pub const A: f64 = 2.0;
pub const J: f64 = 1.5;
pub const IY: f64 = 3.0;
pub const IZ: f64 = 4.0;
pub const RHO: f64 = 3.0;
pub const L: f64 = 4.0;
pub fn cited() {
    assert!(!SOURCE.is_empty() && !CITATION.is_empty());
}
pub fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9 + 1e-11 * b.abs(), "{a} != {b}");
}
pub fn beam(r: MomentReleases) -> EulerFrame3D {
    oriented(r, [0.0, 0.0, 1.0])
}
pub fn oriented(r: MomentReleases, z: [f64; 3]) -> EulerFrame3D {
    EulerFrame3D::new(E, G, A, J, IY, IZ, RHO, z, r).unwrap()
}
pub fn ctx<'a>(coords: &'a [[f64; 3]], params: &'a ElementParams) -> ElementCtx<'a> {
    ElementCtx {
        dim: Dim::D3,
        coords,
        params,
    }
}
pub fn load(q: [f64; 3], m: [f64; 3]) -> MemberLoad {
    MemberLoad {
        force_per_length: q,
        moment_per_length: m,
    }
}
pub fn releases(mask: usize) -> MomentReleases {
    MomentReleases {
        i: std::array::from_fn(|a| mask & (1 << a) != 0),
        j: std::array::from_fn(|a| mask & (1 << (a + 3)) != 0),
    }
}
pub fn params(r: MomentReleases, z: [f64; 3]) -> JsonValue {
    let mut fields: BTreeMap<_, _> = [
        ("E", E),
        ("G", G),
        ("A", A),
        ("J", J),
        ("Iy", IY),
        ("Iz", IZ),
        ("rho", RHO),
    ]
    .map(|(k, v)| (k.into(), JsonValue::Number(v)))
    .into();
    fields.insert(
        "orientation".into(),
        JsonValue::Array(z.map(JsonValue::Number).to_vec()),
    );
    fields.insert(
        "release_i".into(),
        JsonValue::Array(r.i.map(JsonValue::Bool).to_vec()),
    );
    fields.insert(
        "release_j".into(),
        JsonValue::Array(r.j.map(JsonValue::Bool).to_vec()),
    );
    JsonValue::Object(fields)
}
pub fn multiply(m: &DMat, u: &[f64]) -> Vec<f64> {
    ReferenceBackend
        .matmul(m, &DMat::new(u.len(), 1, u.to_vec()).unwrap())
        .unwrap()
        .values()
        .to_vec()
}
pub fn submatrix(m: &DMat, rows: &[usize], cols: &[usize]) -> DMat {
    DMat::new(
        rows.len(),
        cols.len(),
        rows.iter()
            .flat_map(|&r| cols.iter().map(move |&c| m.get(r, c).unwrap()))
            .collect(),
    )
    .unwrap()
}
pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
