//! A single test traverses the table; each failure identifies the fixture, field, and citation.
#[path = "support/harness.rs"]
mod harness;
#[path = "support/table.rs"]
mod table;

use engine_analysis::{
    combination::combine,
    results::{CaseResults, Field, MemberResult, NodeResult},
};
use engine_core::math;
use engine_core::{EResult, MemberId, NodeId};
use engine_model::{CaseFactor, Combination};
use harness::{build, compare, missing, solve, Check};
use table::{Fixture, Kind, Tolerance, FIXTURES};

fn node(r: &CaseResults, id: u32) -> EResult<&NodeResult> {
    r.nodes
        .iter()
        .find(|n| n.id == NodeId(id))
        .ok_or_else(missing)
}
fn member(r: &CaseResults, id: u32) -> EResult<&MemberResult> {
    r.members
        .iter()
        .find(|m| m.id == MemberId(id))
        .ok_or_else(missing)
}
fn moment_range(r: &CaseResults, c: &Check, min: f64, max: f64) -> EResult<()> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for m in &r.members {
        let e = m
            .extremes
            .iter()
            .find(|e| e.field == Field::Mz)
            .ok_or_else(missing)?;
        lo = lo.min(e.min.value);
        hi = hi.max(e.max.value);
    }
    c.close("M mínimo exacto", lo, min)?;
    c.close("M máximo exacto", hi, max)
}
fn axial(r: &CaseResults, c: &Check, id: u32, n: f64) -> EResult<()> {
    let m = member(r, id)?;
    for s in &m.stations {
        c.close("N por compatibilidad/equilibrio", s.forces_local[0], n)?;
        for &value in &s.forces_local[1..] {
            c.close("resultante no axial", value, 0.0)?;
        }
    }
    Ok(())
}

fn validate(f: &Fixture) -> EResult<()> {
    let p = f.parameters;
    let c = Check(f.tolerance);
    let results = solve(&build(f)?, &c)?;
    let r = results.first().ok_or_else(missing)?;
    let l = p.length;
    let ei = p.e * p.inertia;
    let ea = p.e * p.area;
    let w = p.w;
    let force = p.p;
    match f.kind {
        Kind::CantileverPoint | Kind::CantileverUniform | Kind::CantileverMoment => {
            let (v, theta, ry, rz, mi, mj) = match f.kind {
                Kind::CantileverPoint => (
                    -force * math::powi(l, 3) / (3.0 * ei),
                    -force * l * l / (2.0 * ei),
                    force,
                    force * l,
                    -force * l,
                    0.0,
                ),
                Kind::CantileverUniform => (
                    -w * math::powi(l, 4) / (8.0 * ei),
                    -w * math::powi(l, 3) / (6.0 * ei),
                    w * l,
                    w * l * l / 2.0,
                    -w * l * l / 2.0,
                    0.0,
                ),
                _ => (
                    force * l * l / (2.0 * ei),
                    force * l / ei,
                    0.0,
                    -force,
                    force,
                    force,
                ),
            };
            let tip = node(r, 3)?;
            // Explicit inverse transformation: compare global and local components.
            c.close("Ux rotado", tip.displacements[0], -v * math::sin(p.angle))?;
            c.close("Uy rotado", tip.displacements[1], v * math::cos(p.angle))?;
            c.close(
                "flecha transversal invariante",
                -math::sin(p.angle) * tip.displacements[0]
                    + math::cos(p.angle) * tip.displacements[1],
                v,
            )?;
            c.close("giro punta", tip.displacements[5], theta)?;
            let base = node(r, 1)?;
            c.close("Rx rotado", base.reactions[0], -ry * math::sin(p.angle))?;
            c.close("Ry rotado", base.reactions[1], ry * math::cos(p.angle))?;
            c.close("momento reacción", base.reactions[5], rz)?;
            moment_range(r, &c, mi.min(mj), mi.max(mj))?;
        }
        Kind::SimpleUniform | Kind::SimplePoint => {
            let (v, ta, tb, ra, rb, m) = if matches!(f.kind, Kind::SimpleUniform) {
                (
                    -5.0 * w * math::powi(l, 4) / (384.0 * ei),
                    -w * math::powi(l, 3) / (24.0 * ei),
                    w * math::powi(l, 3) / (24.0 * ei),
                    w * l / 2.0,
                    w * l / 2.0,
                    w * l * l / 8.0,
                )
            } else {
                let a = p.fraction * l;
                let b = l - a;
                (
                    -force * a * a * b * b / (3.0 * ei * l),
                    -force * b * (l * l - b * b) / (6.0 * ei * l),
                    force * a * (l * l - a * a) / (6.0 * ei * l),
                    force * b / l,
                    force * a / l,
                    force * a * b / l,
                )
            };
            c.close(
                "flecha en nodo de observación",
                node(r, 2)?.displacements[1],
                v,
            )?;
            c.close("θ izquierdo", node(r, 1)?.displacements[5], ta)?;
            c.close("θ derecho", node(r, 3)?.displacements[5], tb)?;
            c.close("RA", node(r, 1)?.reactions[1], ra)?;
            c.close("RB", node(r, 3)?.reactions[1], rb)?;
            moment_range(r, &c, 0.0, m)?;
        }
        Kind::FixedUniform | Kind::FixedPoint => {
            let (ma, mb, ra, rb, v, peak) = if matches!(f.kind, Kind::FixedUniform) {
                (
                    -w * l * l / 12.0,
                    -w * l * l / 12.0,
                    w * l / 2.0,
                    w * l / 2.0,
                    -w * math::powi(l, 4) / (384.0 * ei),
                    w * l * l / 24.0,
                )
            } else {
                let a = p.fraction * l;
                let b = l - a;
                (
                    -force * a * b * b / (l * l),
                    -force * a * a * b / (l * l),
                    force * b * b * (3.0 * a + b) / math::powi(l, 3),
                    force * a * a * (a + 3.0 * b) / math::powi(l, 3),
                    -force * math::powi(a, 3) * math::powi(b, 3) / (3.0 * ei * math::powi(l, 3)),
                    2.0 * force * a * a * b * b / math::powi(l, 3),
                )
            };
            c.close("δ biempotrada", node(r, 2)?.displacements[1], v)?;
            c.close("θ extremo i", node(r, 1)?.displacements[5], 0.0)?;
            c.close("θ extremo j", node(r, 3)?.displacements[5], 0.0)?;
            c.close("RA", node(r, 1)?.reactions[1], ra)?;
            c.close("RB", node(r, 3)?.reactions[1], rb)?;
            c.close("par apoyo i", node(r, 1)?.reactions[5], -ma)?;
            c.close("par apoyo j", node(r, 3)?.reactions[5], mb)?;
            moment_range(r, &c, ma.min(mb), peak)?;
        }
        Kind::Propped | Kind::Release => {
            let end = if matches!(f.kind, Kind::Release) {
                2
            } else {
                3
            };
            c.close("RA=5wL/8", node(r, 1)?.reactions[1], 5.0 * w * l / 8.0)?;
            c.close("RB=3wL/8", node(r, end)?.reactions[1], 3.0 * w * l / 8.0)?;
            c.close(
                "par empotramiento",
                node(r, 1)?.reactions[5],
                w * l * l / 8.0,
            )?;
            if matches!(f.kind, Kind::Propped) {
                c.close(
                    "flecha medio",
                    node(r, 2)?.displacements[1],
                    -w * math::powi(l, 4) / (192.0 * ei),
                )?;
                c.close(
                    "θ apuntalado",
                    node(r, 3)?.displacements[5],
                    w * math::powi(l, 3) / (48.0 * ei),
                )?;
            } else {
                c.close(
                    "momento articulación extremo",
                    member(r, 1)?
                        .end_forces_local
                        .get(5)
                        .copied()
                        .ok_or_else(missing)?,
                    0.0,
                )?;
                c.close(
                    "momento articulación sección",
                    member(r, 1)?
                        .stations
                        .last()
                        .ok_or_else(missing)?
                        .forces_local[5],
                    0.0,
                )?;
                c.close(
                    "par reacción giro desconectado",
                    node(r, 2)?.reactions[5],
                    0.0,
                )?;
            }
            // Root x=5L/8 lies outside the explicit grid [0,.2,.8,1].
            moment_range(r, &c, -w * l * l / 8.0, 9.0 * w * l * l / 128.0)?;
            let peak_member = if matches!(f.kind, Kind::Release) {
                1
            } else {
                2
            };
            let peak_position = if matches!(f.kind, Kind::Release) {
                0.625
            } else {
                0.25
            };
            let peak = member(r, peak_member)?
                .extremes
                .iter()
                .find(|e| e.field == Field::Mz)
                .ok_or_else(missing)?;
            c.close(
                "estación crítica exacta Mmax",
                peak.max.at_station,
                peak_position,
            )?;
        }
        Kind::Continuous => {
            for id in [1, 5] {
                c.close("R exterior", node(r, id)?.reactions[1], 3.0 * w * l / 8.0)?;
            }
            c.close(
                "R central=5wL/4",
                node(r, 3)?.reactions[1],
                5.0 * w * l / 4.0,
            )?;
            c.close("θ centro simétrico", node(r, 3)?.displacements[5], 0.0)?;
            c.close(
                "θ izquierdo",
                node(r, 1)?.displacements[5],
                -w * math::powi(l, 3) / (48.0 * ei),
            )?;
            c.close(
                "θ derecho",
                node(r, 5)?.displacements[5],
                w * math::powi(l, 3) / (48.0 * ei),
            )?;
            for id in [2, 4] {
                c.close(
                    "δ medio vano",
                    node(r, id)?.displacements[1],
                    -w * math::powi(l, 4) / (192.0 * ei),
                )?;
            }
            moment_range(r, &c, -w * l * l / 8.0, 9.0 * w * l * l / 128.0)?;
        }
        Kind::TwoBars | Kind::Triangle | Kind::Concurrent => {
            let h = p.height;
            let ell = math::hypot(l, h);
            let horizontal = p.horizontal;
            let triangle = matches!(f.kind, Kind::Triangle);
            let concurrent = matches!(f.kind, Kind::Concurrent);
            let nb = horizontal / 2.0 + force * l / (2.0 * h);
            let ux = horizontal * math::powi(ell, 3) / (2.0 * ea * l * l)
                + if triangle { nb * l / ea } else { 0.0 };
            let uy = if concurrent {
                -force / (2.0 * ea * h * h / math::powi(ell, 3) + ea / h)
            } else {
                -force * math::powi(ell, 3) / (2.0 * ea * h * h)
                    - if triangle { nb * l * l / (ea * h) } else { 0.0 }
            };
            let top = if concurrent { 4 } else { 3 };
            c.close("ux ápice", node(r, top)?.displacements[0], ux)?;
            c.close("uy ápice", node(r, top)?.displacements[1], uy)?;
            let n1 = if concurrent {
                ea / ell * (l / ell * ux + h / ell * uy)
            } else {
                horizontal * ell / (2.0 * l) - force * ell / (2.0 * h)
            };
            let n2 = if concurrent {
                ea / ell * (-l / ell * ux + h / ell * uy)
            } else {
                -horizontal * ell / (2.0 * l) - force * ell / (2.0 * h)
            };
            axial(r, &c, 1, n1)?;
            axial(r, &c, 2, n2)?;
            if triangle {
                axial(r, &c, 3, nb)?;
                c.close(
                    "ux rodillo",
                    node(r, 2)?.displacements[0],
                    2.0 * l * nb / ea,
                )?;
                c.close("Rx isostático", node(r, 1)?.reactions[0], -horizontal)?;
                c.close(
                    "Ry izquierdo",
                    node(r, 1)?.reactions[1],
                    force / 2.0 - horizontal * h / (2.0 * l),
                )?;
                c.close(
                    "Ry derecho",
                    node(r, 2)?.reactions[1],
                    force / 2.0 + horizontal * h / (2.0 * l),
                )?;
            } else {
                c.close("Rx izquierdo", node(r, 1)?.reactions[0], -n1 * l / ell)?;
                c.close("Ry izquierdo", node(r, 1)?.reactions[1], -n1 * h / ell)?;
                c.close("Rx derecho", node(r, 2)?.reactions[0], n2 * l / ell)?;
                c.close("Ry derecho", node(r, 2)?.reactions[1], -n2 * h / ell)?;
                if concurrent {
                    axial(r, &c, 3, ea * uy / h)?;
                    c.close("Ry barra central", node(r, 3)?.reactions[1], -ea * uy / h)?;
                }
            }
        }
        Kind::Settlement => {
            let n = ea * p.settlement / l;
            axial(r, &c, 1, n)?;
            axial(r, &c, 2, n)?;
            c.close(
                "u medio asentamiento",
                node(r, 2)?.displacements[0],
                p.settlement / 2.0,
            )?;
            c.close("u prescrito", node(r, 3)?.displacements[0], p.settlement)?;
            c.close("Rx inicial", node(r, 1)?.reactions[0], -n)?;
            c.close("Rx final", node(r, 3)?.reactions[0], n)?;
        }
        Kind::PortalLateral => {
            let h = p.height;
            let delta = force * math::powi(h, 3) / (24.0 * ei);
            // Finite closed-form solution independent of the rigid limit.
            let beam_ei = ei * p.rigid_ratio;
            let column_ea = ea * p.rigid_ratio;
            let beta = (12.0 * beam_ei / math::powi(l, 2))
                / (column_ea / h + 24.0 * beam_ei / math::powi(l, 3));
            let d = 4.0 * ei / h + 6.0 * beam_ei / l - 12.0 * beam_ei * beta / math::powi(l, 2);
            let finite_delta = force
                / (24.0 * ei / math::powi(h, 3)
                    - 72.0 * math::powi(ei, 2) / (math::powi(h, 4) * d));
            let finite_theta = -6.0 * ei * finite_delta / (h * h * d);
            let finite_v = -beta * finite_theta;
            let numerical = Check(Tolerance {
                absolute: 1e-8,
                relative: 1e-10,
                justification:
                    "Portal finito: margen f64 por contraste de rigideces 1e5; §portal-lateral.",
            });
            for (id, sign) in [(3, 1.0), (4, -1.0)] {
                numerical.close(
                    "Δ solución finita",
                    node(r, id)?.displacements[0],
                    finite_delta,
                )?;
                numerical.close(
                    "θ solución finita",
                    node(r, id)?.displacements[5],
                    finite_theta,
                )?;
                numerical.close(
                    "v solución finita",
                    node(r, id)?.displacements[1],
                    sign * finite_v,
                )?;
            }
            for id in [3, 4, 5] {
                c.close("Δ límite rígido", node(r, id)?.displacements[0], delta)?;
            }
            for id in [1, 2] {
                c.close("Rx portal lateral", node(r, id)?.reactions[0], -force / 2.0)?;
            }
            c.close(
                "ΣH portal",
                node(r, 1)?.reactions[0] + node(r, 2)?.reactions[0],
                -force,
            )?;
            // θ and uy are small but NOT exactly zero with finite stiffnesses.
        }
        Kind::PortalVertical => {
            let h = p.height;
            let gamma =
                (6.0 * ei / math::powi(h, 2)) / (12.0 * ei / math::powi(h, 3) + 2.0 * ea / l);
            let d = 4.0 * ei / h + 2.0 * ei / l - 6.0 * ei * gamma / math::powi(h, 2);
            let theta = -(w * l * l / 12.0) / d;
            let t = -gamma * theta;
            let v = -w * l * h / (2.0 * ea);
            for (id, sign) in [(3, 1.0), (4, -1.0)] {
                c.close("ux simétrico", node(r, id)?.displacements[0], sign * t)?;
                c.close("uy simétrico", node(r, id)?.displacements[1], v)?;
                c.close(
                    "giro simétrico",
                    node(r, id)?.displacements[5],
                    sign * theta,
                )?;
            }
            c.close(
                "δ dintel medio",
                node(r, 5)?.displacements[1],
                v + l * theta / 4.0 - w * math::powi(l, 4) / (384.0 * ei),
            )?;
            c.close("θ dintel medio", node(r, 5)?.displacements[5], 0.0)?;
            for id in [1, 2] {
                c.close("Ry portal vertical", node(r, id)?.reactions[1], w * l / 2.0)?;
            }
            c.close(
                "Rx antisimétrico",
                node(r, 1)?.reactions[0] + node(r, 2)?.reactions[0],
                0.0,
            )?;
        }
        Kind::Combination => {
            let combination = Combination {
                id: f.name.into(),
                terms: vec![
                    CaseFactor {
                        case: "Q".into(),
                        factor: p.factors[0],
                    },
                    CaseFactor {
                        case: "W".into(),
                        factor: p.factors[1],
                    },
                ],
            };
            let sum = combine(&combination, &results)?.results;
            let direct = results
                .iter()
                .find(|r| r.case.0 == "D")
                .ok_or_else(missing)?;
            compare(&sum, direct, &c)?;
            let [alpha, beta] = p.factors;
            c.close(
                "δ combinación cerrada",
                node(&sum, 3)?.displacements[1],
                -alpha * force * math::powi(l, 3) / (3.0 * ei)
                    - beta * w * math::powi(l, 4) / (8.0 * ei),
            )?;
            c.close(
                "θ combinación cerrada",
                node(&sum, 3)?.displacements[5],
                -alpha * force * l * l / (2.0 * ei) - beta * w * math::powi(l, 3) / (6.0 * ei),
            )?;
            c.close(
                "Ry combinación cerrada",
                node(&sum, 1)?.reactions[1],
                alpha * force + beta * w * l,
            )?;
            c.close(
                "Rz combinación cerrada",
                node(&sum, 1)?.reactions[5],
                alpha * force * l + beta * w * l * l / 2.0,
            )?;
        }
    }
    Ok(())
}

#[test]
fn suite_estatica_lineal_analitica() {
    assert_eq!(
        FIXTURES.len(),
        42,
        "Update the documented count if the table changes"
    );
    let mut names = std::collections::BTreeSet::new();
    for f in FIXTURES {
        assert!(names.insert(f.name), "Duplicate fixture name: {}", f.name);
        assert!(
            !f.source.is_empty()
                && f.citation.starts_with("provenance/ENG-VAL-001.md#")
                && !f.tolerance.justification.is_empty(),
            "Incomplete metadata: {}",
            f.name
        );
        if let Err(error) = validate(f) {
            panic!(
                "Fixture {}: {error}\nsource: {}\ncitation: {}",
                f.name, f.source, f.citation
            );
        }
    }
}
