//! EQ-ALL: independent physical input sums and recovered support reactions.
#[path = "support/equilibrium_dynamic.rs"]
mod dynamic;
#[path = "support/harness.rs"]
mod harness;
#[path = "support/table.rs"]
mod table;

use engine_analysis::{combination::combine, results::CaseResults};
use engine_contracts::MemberLoad;
use engine_core::math;
use engine_core::{Dof, NodeId};
use engine_model::{CaseFactor, Combination, Model};
use engine_validation::fixtures::{NativeTarget, Target, REPOSITORY_FIXTURES};
use serde_json::Value;
use std::collections::BTreeSet;
use table::{Fixture, Kind, FIXTURES};

const RTOL: f64 = 1e-10;
const ROW_COUNT: usize = 60;
const MATRIX: &str = include_str!("../ORACLES.md");
const PROVENANCE: &str = include_str!("../../../provenance/ENG-VAL-006-EQUILIBRIO.md");

#[derive(Clone, Default)]
struct Actions {
    // Forces and TOTAL moments about the fixed global origin O=(0,0,0).
    sum: [f64; 6],
    force_magnitude: f64,
    moment_magnitude: f64,
}

impl Actions {
    fn add(&mut self, position: [f64; 3], values: [f64; 6]) {
        assert!(position.into_iter().chain(values).all(f64::is_finite));
        let [x, y, z] = position;
        let [fx, fy, fz, mx, my, mz] = values;
        let arms = [y * fz, -z * fy, z * fx, -x * fz, x * fy, -y * fx];
        let total = [
            fx,
            fy,
            fz,
            mx + arms[0] + arms[1],
            my + arms[2] + arms[3],
            mz + arms[4] + arms[5],
        ];
        for (sum, value) in self.sum.iter_mut().zip(total) {
            *sum += value;
        }
        self.force_magnitude += fx.abs() + fy.abs() + fz.abs();
        self.moment_magnitude += mx.abs() + my.abs() + mz.abs();
        self.moment_magnitude += arms.into_iter().map(f64::abs).sum::<f64>();
    }

    fn scaled_add(&mut self, other: &Self, factor: f64) {
        assert!(factor.is_finite());
        for (sum, value) in self.sum.iter_mut().zip(other.sum) {
            *sum += factor * value;
        }
        self.force_magnitude += factor.abs() * other.force_magnitude;
        self.moment_magnitude += factor.abs() * other.moment_magnitude;
    }
}

fn position(model: &Model, id: NodeId) -> [f64; 3] {
    model
        .nodes
        .iter()
        .find(|n| n.id == id)
        .expect("Input node must exist")
        .coordinates
}

fn component(dof: Dof, value: f64) -> [f64; 6] {
    let mut values = [0.0; 6];
    values[dof.index()] = value;
    values
}

fn nodal_input(model: &Model, case: &str) -> Actions {
    let mut applied = Actions::default();
    let case = model
        .cases
        .iter()
        .find(|c| c.name == case)
        .expect("Input load case must exist");
    for load in &case.loads {
        for v in &load.values {
            applied.add(position(model, load.node), component(v.dof, v.value));
        }
    }
    applied
}

fn uniform_input(applied: &mut Actions, a: [f64; 3], b: [f64; 3], q: &MemberLoad) {
    let delta: [f64; 3] = std::array::from_fn(|i| b[i] - a[i]);
    let length = math::sqrt(delta.into_iter().map(|v| v * v).sum());
    assert!(length.is_finite() && length > 0.0);
    let midpoint = std::array::from_fn(|i| (a[i] + b[i]) / 2.0);
    let values = std::array::from_fn(|i| {
        length
            * if i < 3 {
                q.force_per_length[i]
            } else {
                q.moment_per_length[i - 3]
            }
    });
    // Integral q dx = q L; integral r cross q dx = r_mid cross q L.
    // This never calls load_vector or reads the assembled solver force.
    applied.add(midpoint, values);
}

fn linear_input(f: &Fixture, model: &Model, case: &str) -> Actions {
    let mut applied = nodal_input(model, case);
    let p = f.parameters;
    // Entry loads are private in the reused harness. This is their physical
    // input manifest, transcribed from build(), not inferred from reactions.
    // Every Kind is explicit so that a new family requires a decision here.
    let (pairs, intensity): (&[(u32, u32)], f64) = match f.kind {
        Kind::CantileverUniform | Kind::SimpleUniform | Kind::FixedUniform | Kind::Propped => {
            (&[(1, 2), (2, 3)], p.w)
        }
        Kind::Continuous => (&[(1, 2), (2, 3), (3, 4), (4, 5)], p.w),
        Kind::Release => (&[(1, 2)], p.w),
        Kind::PortalVertical => (&[(3, 5), (5, 4)], p.w),
        Kind::Combination => match case {
            "Q" => (&[], 0.0),
            "W" => (&[(1, 2), (2, 3)], p.w),
            "D" => (&[(1, 2), (2, 3)], p.w * p.factors[1]),
            _ => panic!("Unexpected combination input case: {case}"),
        },
        Kind::CantileverPoint
        | Kind::CantileverMoment
        | Kind::SimplePoint
        | Kind::FixedPoint
        | Kind::TwoBars
        | Kind::Triangle
        | Kind::Concurrent
        | Kind::Settlement
        | Kind::PortalLateral => (&[], 0.0),
    };
    let q = MemberLoad {
        force_per_length: [
            intensity * math::sin(p.angle),
            -intensity * math::cos(p.angle),
            0.0,
        ],
        moment_per_length: [0.0; 3],
    };
    for &(a, b) in pairs {
        uniform_input(
            &mut applied,
            position(model, NodeId(a)),
            position(model, NodeId(b)),
            &q,
        );
    }
    applied
}

fn recovered(model: &Model, result: &CaseResults) -> Actions {
    let expected: BTreeSet<_> = model.nodes.iter().map(|n| n.id).collect();
    let observed: BTreeSet<_> = result.nodes.iter().map(|n| n.id).collect();
    assert_eq!(observed, expected, "Missing/extra recovered nodes");
    assert_eq!(
        result.nodes.len(),
        expected.len(),
        "Repeated recovered node"
    );
    let mut reactions = Actions::default();
    for node in &result.nodes {
        reactions.add(position(model, node.id), node.reactions);
    }
    reactions
}

fn radius(points: impl Iterator<Item = [f64; 3]>) -> f64 {
    points
        .map(|p| math::sqrt(p.into_iter().map(|x| x * x).sum()))
        .fold(1.0, f64::max)
}

fn vector(values: &[f64]) -> String {
    let components: Vec<_> = values.iter().map(|v| format!("{v:.12e}")).collect();
    format!("[{}]", components.join(", "))
}

struct Row {
    id: String,
    family: &'static str,
    applied: Actions,
    reactions: Actions,
    scales: [f64; 2],
    lateral_axis: Option<usize>,
}

impl Row {
    fn new(
        id: String,
        family: &'static str,
        applied: Actions,
        reactions: Actions,
        length: f64,
        imposed_force: f64,
    ) -> Self {
        // Scales depend only on inputs, including EA*|settlement|/L.
        let force = applied.force_magnitude.max(imposed_force).max(1.0);
        let moment = applied.moment_magnitude.max(force * length).max(1.0);
        assert!(force.is_finite() && moment.is_finite());
        assert!(
            applied.force_magnitude + applied.moment_magnitude + imposed_force > 0.0,
            "Empty physical problem: {id}"
        );
        if imposed_force > 0.0 {
            assert!(
                reactions.force_magnitude >= imposed_force,
                "Settlement must produce nonzero opposing reactions: {id}"
            );
        }
        let lateral_axis = (applied.sum[0] != 0.0).then_some(0);
        Self {
            id,
            family,
            applied,
            reactions,
            scales: [force, moment],
            lateral_axis,
        }
    }

    fn residuals(&self) -> [f64; 6] {
        std::array::from_fn(|i| {
            (self.applied.sum[i] + self.reactions.sum[i]).abs() / self.scales[usize::from(i >= 3)]
        })
    }

    fn basal(&self) -> String {
        self.lateral_axis.map_or_else(
            || "N/A".into(),
            |i| {
                format!(
                    "{:.12e}/{:.12e}",
                    -self.reactions.sum[i], self.applied.sum[i]
                )
            },
        )
    }

    fn verify(&self) {
        for (i, relative) in self.residuals().into_iter().enumerate() {
            assert!(
                relative.is_finite() && relative <= RTOL,
                "{} component {i}: F={:.16e}, R={:.16e}, relative={relative:.16e}, scale={:.16e}",
                self.id,
                self.applied.sum[i],
                self.reactions.sum[i],
                self.scales[usize::from(i >= 3)]
            );
        }
        if let Some(i) = self.lateral_axis {
            let basal = -self.reactions.sum[i];
            let lateral = self.applied.sum[i];
            assert!(lateral.abs() > 0.0, "Vacuous basal check: {}", self.id);
            assert!(
                (basal - lateral).abs() / self.scales[0] <= RTOL,
                "{}: basal={basal:e}, lateral={lateral:e}",
                self.id
            );
        }
    }
}

fn linear_rows(rows: &mut Vec<Row>) {
    assert_eq!(FIXTURES.len(), 42);
    for f in FIXTURES {
        assert!(!f.source.is_empty() && !f.citation.is_empty());
        let problem = harness::build(f).expect(f.name);
        let results = harness::solve(&problem, &harness::Check(f.tolerance)).expect(f.name);
        let combo = matches!(f.kind, Kind::Combination);
        let cases: &[&str] = if combo { &["Q", "W", "D"] } else { &["Q"] };
        assert_eq!(results.len(), cases.len(), "{}", f.name);
        let length = radius(problem.model.nodes.iter().map(|n| n.coordinates));
        let imposed_force = if matches!(f.kind, Kind::Settlement) {
            let p = f.parameters;
            p.e * p.area * p.settlement.abs() / p.length
        } else {
            0.0
        };
        for &case in cases {
            let result = results.iter().find(|r| r.case.0 == case).expect(f.name);
            rows.push(Row::new(
                format!("{}/{case}", f.name),
                "lineal",
                linear_input(f, &problem.model, case),
                recovered(&problem.model, result),
                length,
                imposed_force,
            ));
        }
        if combo {
            let terms: Vec<_> = ["Q", "W"]
                .into_iter()
                .zip(f.parameters.factors)
                .map(|(case, factor)| CaseFactor {
                    case: case.into(),
                    factor,
                })
                .collect();
            let mut applied = Actions::default();
            for term in &terms {
                applied.scaled_add(&linear_input(f, &problem.model, &term.case), term.factor);
            }
            let combination = Combination {
                id: f.name.into(),
                terms,
            };
            let combined = combine(&combination, &results).expect(f.name).results;
            rows.push(Row::new(
                format!("{}/C", f.name),
                "combinación",
                applied,
                recovered(&problem.model, &combined),
                length,
                0.0,
            ));
        }
    }
}

fn array(value: &Value) -> [f64; 3] {
    let values = value.as_array().expect("Three-component input vector");
    assert_eq!(values.len(), 3);
    std::array::from_fn(|i| values[i].as_f64().expect("Finite input component"))
}

fn json_position(nodes: &[Value], id: &Value) -> [f64; 3] {
    array(
        &nodes
            .iter()
            .find(|n| n["id"] == *id)
            .expect("JSON input node must exist")["coordinates"],
    )
}

fn dof_index(value: &Value) -> usize {
    match value.as_str().expect("Input DOF") {
        "Ux" => 0,
        "Uy" => 1,
        "Uz" => 2,
        "Rx" => 3,
        "Ry" => 4,
        "Rz" => 5,
        other => panic!("Unexpected input DOF: {other}"),
    }
}

fn json_rows(rows: &mut Vec<Row>) {
    assert_eq!(REPOSITORY_FIXTURES.len(), 5);
    for &(id, text) in REPOSITORY_FIXTURES {
        let document: Value = serde_json::from_str(text).expect(id);
        let input = &document["input"];
        let nodes = input["model"]["nodes"].as_array().expect(id);
        let cases = input["cases"].as_array().expect(id);
        assert_eq!(cases.len(), 1, "Review the fixed row inventory: {id}");
        assert!(input["combinations"].as_array().expect(id).is_empty());
        let output: Value =
            serde_json::from_str(&NativeTarget.execute(&input.to_string()).expect(id)).expect(id);
        let solved = output["cases"].as_array().expect(id);
        assert_eq!(solved.len(), cases.len(), "{id}");
        for case in cases {
            let name = case["name"].as_str().expect(id);
            let mut applied = Actions::default();
            for load in case["loads"].as_array().expect(id) {
                for v in load["values"].as_array().expect(id) {
                    let mut values = [0.0; 6];
                    values[dof_index(&v["dof"])] = v["value"].as_f64().expect(id);
                    applied.add(json_position(nodes, &load["node"]), values);
                }
            }
            if let Some(loads) = case.get("member_loads") {
                for load in loads.as_array().expect(id) {
                    let member = input["model"]["members"]
                        .as_array()
                        .expect(id)
                        .iter()
                        .find(|m| m["id"] == load["member"])
                        .expect(id);
                    let ends = member["nodes"].as_array().expect(id);
                    assert_eq!(ends.len(), 2);
                    uniform_input(
                        &mut applied,
                        json_position(nodes, &ends[0]),
                        json_position(nodes, &ends[1]),
                        &MemberLoad {
                            force_per_length: array(&load["force_per_length"]),
                            moment_per_length: array(&load["moment_per_length"]),
                        },
                    );
                }
            }
            let result = solved.iter().find(|r| r["case"] == name).expect(id);
            let output_nodes = result["nodes"].as_array().expect(id);
            let expected: BTreeSet<_> = nodes.iter().map(|n| n["id"].as_u64().unwrap()).collect();
            let actual: BTreeSet<_> = output_nodes
                .iter()
                .map(|n| n["id"].as_u64().unwrap())
                .collect();
            assert_eq!(actual, expected, "{id}: missing/extra output node");
            assert_eq!(output_nodes.len(), expected.len());
            let mut reactions = Actions::default();
            for node in output_nodes {
                let values = node["reactions"].as_array().expect(id);
                assert_eq!(values.len(), 6);
                reactions.add(
                    json_position(nodes, &node["id"]),
                    std::array::from_fn(|i| values[i].as_f64().expect(id)),
                );
            }
            rows.push(Row::new(
                format!("{id}/{name}"),
                "fixture/1",
                applied,
                reactions,
                radius(nodes.iter().map(|n| array(&n["coordinates"]))),
                0.0,
            ));
        }
    }
}

#[test]
fn eq_all_global_equilibrium() {
    let mut rows = Vec::new();
    linear_rows(&mut rows);
    json_rows(&mut rows);
    let (model, reactions) = dynamic::zero_prestress().expect("pdelta-cero");
    let mut support = Actions::default();
    for r in reactions {
        support.add(position(&model, r.node), component(r.dof, r.value));
    }
    let mut zero = Row::new(
        "pdelta-cero/H".into(),
        "preesfuerzo cero",
        nodal_input(&model, "H"),
        support,
        radius(model.nodes.iter().map(|n| n.coordinates)),
        0.0,
    );
    zero.lateral_axis = Some(1); // Transverse load of the column in dynamic_harness.
    rows.push(zero);

    assert_eq!(rows.len(), ROW_COUNT, "An EQ-ALL case is missing");
    let ids: BTreeSet<_> = rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids.len(), ROW_COUNT, "Duplicate EQ-ALL case");
    let documented: BTreeSet<_> = PROVENANCE
        .lines()
        .filter_map(|line| line.strip_prefix("| `")?.split_once("` |"))
        .map(|(id, _)| id)
        .collect();
    assert_eq!(ids, documented, "EQ-ALL table/provenance mismatch");
    let included: BTreeSet<_> = rows
        .iter()
        .map(|r| r.id.split_once('/').unwrap().0)
        .collect();
    assert_eq!(included.len(), 48);
    let inventory: BTreeSet<_> = MATRIX
        .lines()
        .filter_map(|line| line.strip_prefix("| `")?.split_once("` |"))
        .map(|(id, _)| id)
        .collect();
    assert_eq!(inventory.len(), 94, "Review the VAL-005 inventory");
    assert!(included.is_subset(&inventory));
    for pending in inventory.difference(&included) {
        assert!(
            PROVENANCE.contains(&format!("- `{pending}`: PENDIENTE")),
            "Missing pending reason: {pending}"
        );
    }
    assert_eq!(inventory.difference(&included).count(), 46);
    assert!(rows.iter().filter(|r| r.lateral_axis.is_some()).count() >= 8);
    assert!(rows.iter().any(|r| r.applied.sum[2].abs() > 0.0));
    assert!(rows.iter().any(|r| r.applied.sum[4].abs() > 0.0));
    assert!(rows.iter().filter(|r| r.applied.sum[5].abs() > 0.0).count() >= 30);

    println!("EQ-ALL; O=(0,0,0) m; forces=N; moments=N m; rtol={RTOL:e}");
    println!("| id | familia | F aplicada [Fx,Fy,Fz,Mx,My,Mz] | R [Fx,Fy,Fz,Mx,My,Mz] | S_F / S_M | residuo relativo | basal / lateral | eje lateral |");
    println!("| --- | --- | --- | --- | --- | --- | --- | --- |");
    for row in &rows {
        let residual = row.residuals().into_iter().fold(0.0, f64::max);
        println!(
            "| {} | {} | {} | {} | {} | {:.12e} | {} | {:?} |",
            row.id,
            row.family,
            vector(&row.applied.sum),
            vector(&row.reactions.sum),
            vector(&row.scales),
            residual,
            row.basal(),
            row.lateral_axis
        );
    }
    // Print the complete single table before asserting the per-case balances.
    for row in &rows {
        row.verify();
    }
}
