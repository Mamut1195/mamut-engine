//! Ensamblaje público y oráculos analíticos propios, sin dependencias de runtime.
use super::table::{Fixture, Kind, FIXTURES};
use engine_analysis::StaticLinear;
use engine_assembly::{DofMap, ElementContribution, LinearSystem};
use engine_contracts::{
    CaseRef, Element, ElementCtx, ElementFactory, ElementParams, JsonValue, ModelView,
};
use engine_core::math;
use engine_core::{Dim, Dof, EResult, EngineError, NodeId};
use engine_linalg::ReferenceBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};
use std::collections::BTreeMap;

fn error(message: impl Into<String>) -> EngineError {
    EngineError::new("ENG-E-0012", message)
}
fn close(label: &str, a: f64, b: f64, relative: f64) -> EResult<()> {
    close_with_absolute(label, a, b, 1e-8, relative)
}
fn close_with_absolute(label: &str, a: f64, b: f64, absolute: f64, relative: f64) -> EResult<()> {
    if !a.is_finite() || (a - b).abs() > absolute + relative * b.abs() {
        return Err(error(format!(
            "{label}: obtenido={a:.12e}, analítico={b:.12e}, tolerancia relativa={relative:.3e}"
        )));
    }
    Ok(())
}
fn element(f: &Fixture) -> EResult<Box<dyn Element>> {
    let mut fields = BTreeMap::from([
        ("E".into(), JsonValue::Number(1.0)),
        ("A".into(), JsonValue::Number(10.0)),
        ("rho".into(), JsonValue::Number(0.0)),
    ]);
    let mut number = |key: &str, value| {
        fields.insert(key.into(), JsonValue::Number(value));
    };
    if f.spatial {
        number("G", 4.0);
        number("J", 3.0);
        number("Iy", f.ei[1]);
        number("Iz", f.ei[0]);
        number("ky", f.k[0]);
        number("kz", f.k[1]);
        fields.insert(
            "orientation".into(),
            JsonValue::Array(vec![
                JsonValue::Number(0.0),
                JsonValue::Number(0.0),
                JsonValue::Number(1.0),
            ]),
        );
        engine_elements_frame3d::WinklerFactory.build(
            &JsonValue::Object(fields),
            &ModelView {
                dim: Dim::D3,
                nodes: &[],
            },
        )
    } else {
        number("I", f.ei[0]);
        number("k", f.k[0]);
        engine_elements_frame::WinklerFactory.build(
            &JsonValue::Object(fields),
            &ModelView {
                dim: Dim::D2,
                nodes: &[],
            },
        )
    }
}
struct Observation {
    w: [f64; 2],
    m: [f64; 2],
}
fn solve(f: &Fixture, n: usize, length: f64) -> EResult<Observation> {
    let dim = if f.spatial { Dim::D3 } else { Dim::D2 };
    let beam = element(f)?;
    let stride = beam.dofs_per_node().len();
    let h = length / n as f64;
    let loaded = if f.kind == Kind::SemiInfinite {
        0
    } else {
        n / 2
    };
    let mut model = Model::new();
    for i in 0..=n {
        let id = NodeId(i as u32 + 1);
        model
            .nodes
            .push(Node::new(id, dim, [i as f64 * h, 0.0, 0.0]));
        // Solo GDL desacoplados sin carga; ningún apoyo transversal ni de flexión.
        model.supports.push(Support {
            id: format!("axial-torsion-{i}"),
            node: id,
            restrained: if f.spatial {
                vec![Dof::Ux, Dof::Rx]
            } else {
                vec![Dof::Ux]
            },
        });
    }
    let mut values = vec![DofValue {
        dof: Dof::Uy,
        value: f.p[0],
    }];
    if f.spatial {
        values.push(DofValue {
            dof: Dof::Uz,
            value: f.p[1],
        });
    }
    model.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![NodalLoad {
            node: NodeId(loaded as u32 + 1),
            values,
        }],
    });
    let map = DofMap::new(&model.nodes)?;
    let params = ElementParams::default();
    let mut contributions = Vec::new();
    for i in 0..n {
        let coords = [model.nodes[i].coordinates, model.nodes[i + 1].coordinates];
        let ctx = ElementCtx {
            dim,
            coords: &coords,
            params: &params,
        };
        let mut location = Vec::new();
        for j in [i, i + 1] {
            for &dof in beam.dofs_per_node() {
                location.push(map.global_index(NodeId(j as u32 + 1), dof)?);
            }
        }
        contributions.push(ElementContribution {
            location,
            stiffness: beam.respond(&ctx, &vec![0.0; stride * 2], &[])?.tangent,
            mass_lumped: None,
            mass_consistent: None,
            geometric: None,
            loads: vec![],
        });
    }
    let mut system = LinearSystem::from_model(ReferenceBackend, &model, &contributions, &[])?;
    let result = StaticLinear::run(&mut system, &[(CaseRef("P".into()), 1.0)])?;
    let result = result.cases.first().ok_or_else(|| error("Caso ausente"))?;
    // EI grande eleva el condicionamiento: cota numérica separada de la física.
    let residual_bound = if f.kind == Kind::Rigid { 1e-6 } else { 1e-8 };
    if result.relative_free_residual > residual_bound {
        return Err(error(format!(
            "Residual libre {} > {residual_bound}",
            result.relative_free_residual
        )));
    }
    let global = &result.global_displacements;
    let mut reaction = [0.0; 2];
    let mut reaction_moment = [0.0; 2];
    let planes = if f.spatial { 2 } else { 1 };
    for (i, contribution) in contributions.iter().enumerate() {
        let coords = [model.nodes[i].coordinates, model.nodes[i + 1].coordinates];
        let ctx = ElementCtx {
            dim,
            coords: &coords,
            params: &params,
        };
        let u: Vec<_> = contribution
            .location
            .iter()
            .map(|d| global[d.0 as usize])
            .collect();
        let out = beam.recover(&ctx, &u, &[], &[], &[0.0, 1.0])?;
        for plane in 0..planes {
            let shear = plane + 1;
            let rotation = if plane == 0 { stride - 1 } else { 4 };
            let moment = if plane == 0 { 5 } else { 4 };
            let sign = if plane == 0 { 1.0 } else { -1.0 };
            // Gauss de 3 puntos: exacto para v cúbica y x·v cuártica.
            for (t, weight) in [
                (0.5 - math::sqrt(3.0_f64 / 5.0) / 2.0, 5.0 / 18.0),
                (0.5, 4.0 / 9.0),
                (0.5 + math::sqrt(3.0_f64 / 5.0) / 2.0, 5.0 / 18.0),
            ] {
                let v = (1.0 - 3.0 * t * t + 2.0 * t * t * t) * u[shear]
                    + h * (t - 2.0 * t * t + t * t * t) * sign * u[rotation]
                    + (3.0 * t * t - 2.0 * t * t * t) * u[stride + shear]
                    + h * (-t * t + t * t * t) * sign * u[stride + rotation];
                let dr = f.k[plane] * h * weight * v;
                reaction[plane] += dr;
                reaction_moment[plane] += dr * (i as f64 + t) * h;
            }
            close_with_absolute(
                "cara i cortante",
                out.stations[0].forces_local[shear],
                -out.end_forces_local[shear],
                if f.kind == Kind::Rigid { 1e-6 } else { 1e-8 },
                1e-9,
            )?;
            close_with_absolute(
                "cara j cortante",
                out.stations[1].forces_local[shear],
                out.end_forces_local[stride + shear],
                if f.kind == Kind::Rigid { 1e-6 } else { 1e-8 },
                1e-9,
            )?;
            close_with_absolute(
                "cara i momento",
                out.stations[0].forces_local[moment],
                -out.end_forces_local[rotation],
                if f.kind == Kind::Rigid { 1e-6 } else { 1e-8 },
                1e-9,
            )?;
            close_with_absolute(
                "cara j momento",
                out.stations[1].forces_local[moment],
                out.end_forces_local[stride + rotation],
                if f.kind == Kind::Rigid { 1e-6 } else { 1e-8 },
                1e-9,
            )?;
        }
    }
    for plane in 0..planes {
        close("Σ lecho=P", reaction[plane], f.p[plane], 1e-7)?;
        close(
            "Σ momento lecho=P·a",
            reaction_moment[plane],
            f.p[plane] * loaded as f64 * h,
            1e-7,
        )?;
    }
    let mut observation = Observation {
        w: [0.0; 2],
        m: [0.0; 2],
    };
    for plane in 0..planes {
        observation.w[plane] = global[loaded * stride + plane + 1];
    }
    if loaded > 0 {
        let i = loaded - 1;
        let coords = [model.nodes[i].coordinates, model.nodes[i + 1].coordinates];
        let ctx = ElementCtx {
            dim,
            coords: &coords,
            params: &params,
        };
        let u: Vec<_> = contributions[i]
            .location
            .iter()
            .map(|d| global[d.0 as usize])
            .collect();
        let out = beam.recover(&ctx, &u, &[], &[], &[1.0])?;
        observation.m = [
            out.stations[0].forces_local[5],
            out.stations[0].forces_local[4],
        ];
    }
    Ok(observation)
}
pub fn validate(f: &Fixture) -> EResult<()> {
    let lambda = math::sqrt(math::sqrt(f.k[0] / (4.0 * f.ei[0])));
    let length = match f.kind {
        Kind::Infinite => 20.0 / lambda,
        Kind::SemiInfinite => 12.0 / lambda,
        Kind::Rigid => 2.0,
    };
    let planes = if f.spatial { 2 } else { 1 };
    let mut previous_w = [f64::INFINITY; 2];
    let mut previous_m = [f64::INFINITY; 2];
    for (level, &n) in f.meshes.iter().enumerate() {
        let result = solve(f, n, length)?;
        for plane in 0..planes {
            let lambda = math::sqrt(math::sqrt(f.k[plane] / (4.0 * f.ei[plane])));
            let exact_w = match f.kind {
                Kind::Infinite => f.p[plane] * lambda / (2.0 * f.k[plane]),
                Kind::SemiInfinite => 2.0 * f.p[plane] * lambda / f.k[plane],
                Kind::Rigid => f.p[plane] / (f.k[plane] * length),
            };
            let ew = ((result.w[plane] - exact_w) / exact_w).abs();
            if f.kind != Kind::Rigid && ew >= previous_w[plane] {
                return Err(error(format!(
                    "Flecha no converge: {previous_w:?} -> {ew:e}"
                )));
            }
            previous_w[plane] = ew;
            if level == 2 {
                close("flecha analítica", result.w[plane], exact_w, f.bound)?;
            }
            if f.kind == Kind::Infinite {
                let exact_m = (if plane == 0 { -1.0 } else { 1.0 }) * f.p[plane] / (4.0 * lambda);
                let em = ((result.m[plane] - exact_m) / exact_m).abs();
                if em >= previous_m[plane] {
                    return Err(error(format!(
                        "Momento no converge: {previous_m:?} -> {em:e}"
                    )));
                }
                previous_m[plane] = em;
                if level == 2 {
                    close("momento Hetenyi", result.m[plane], exact_m, f.bound)?;
                }
            }
        }
    }
    Ok(())
}
pub fn run_all() -> EResult<usize> {
    for f in FIXTURES {
        validate(f).map_err(|e| {
            error(format!(
                "{}: {}; source={}; citation={}; cota={}",
                f.name, e, f.source, f.citation, f.justification
            ))
        })?;
    }
    Ok(FIXTURES.len())
}
