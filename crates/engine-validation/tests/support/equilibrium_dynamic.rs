//! The zero-prestress member of the existing second-order table is linear statics.
#[allow(dead_code)]
#[path = "dynamic_harness.rs"]
mod harness;
#[allow(dead_code)]
#[path = "dynamic_table.rs"]
mod table;

use engine_analysis::pdelta::{PDelta, PDeltaOptions, PDeltaOutcome};
use engine_assembly::Reaction;
use engine_contracts::CaseRef;
use engine_core::{Dim, Dof, EResult, NodeId};
use engine_model::{Model, Node};

pub fn zero_prestress() -> EResult<(Model, Vec<Reaction>)> {
    let f = table::PDELTA
        .iter()
        .find(|f| f.name == "pdelta-cero")
        .ok_or_else(harness::missing)?;
    let table::Parameters::Column { mesh, ratio } = f.parameters else {
        return Err(harness::missing());
    };
    assert_eq!(
        ratio, 0.0,
        "Only zero prestress has a linear moment balance"
    );
    // Mirror the input geometry and loads of harness::column, using its table.
    // No force or coordinate is obtained from the solution.
    let mut model = Model::new();
    for i in 0..=mesh {
        model.nodes.push(Node::new(
            NodeId(i as u32 + 1),
            Dim::D2,
            [f.frame.length * i as f64 / mesh as f64, 0.0, 0.0],
        ));
    }
    harness::support(&mut model, 1, Dim::D2.dofs());
    harness::load(
        &mut model,
        "H",
        &[(mesh as u32 + 1, Dof::Uy, f.frame.force)],
    );
    let mut system = harness::column(mesh, 0.0, f.frame)?;
    let result = PDelta::run(
        &mut system,
        &(CaseRef("G".into()), 1.0),
        &(CaseRef("H".into()), 1.0),
        PDeltaOptions {
            tolerance: 1e-10,
            max_iterations: 2000,
            displacement_scale: 0.01,
            rotation_scale: 0.01,
        },
    )?;
    assert_eq!(result.outcome, PDeltaOutcome::Completed);
    assert!(!result.reactions.is_empty());
    Ok((model, result.reactions))
}
