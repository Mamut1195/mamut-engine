//! Assembly through public APIs. No engine solution is an oracle.
use engine_analysis::pdelta::geometric_producer;
use engine_assembly::{DofMap, ElementContribution, GeometricProducer, LinearSystem};
use engine_contracts::{Element, ElementCtx, ElementParams, MassKind};
use engine_core::math;
use engine_core::{Dim, Dof, EResult, EngineError, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_elements_truss::LinearTruss;
use engine_linalg::{DMat, ReferenceBackend, SparseMatrix};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, NodalMass, Node, Support};
use std::sync::Arc;

use super::table::{Fixture, FrameData, Tolerance};

pub fn missing() -> EngineError {
    EngineError::new(
        "ENG-E-0012",
        "Fixture dinámico: resultado ausente o incoherente.",
    )
}
pub struct Check(pub Tolerance);
impl Check {
    pub fn close(&self, field: &str, actual: f64, expected: f64) -> EResult<()> {
        let limit = self.0.absolute + self.0.relative * expected.abs();
        if !actual.is_finite() || !expected.is_finite() || (actual - expected).abs() > limit {
            return Err(EngineError::new("ENG-E-0012", format!(
                "{field}: obtenido={actual:.15e}, analítico={expected:.15e}, tolerancia={limit:.3e}; {}",
                self.0.justification
            )));
        }
        Ok(())
    }
}
pub fn require(condition: bool, message: &str) -> EResult<()> {
    if condition {
        Ok(())
    } else {
        Err(EngineError::new("ENG-E-0012", message))
    }
}
pub fn run_group(fixtures: &[Fixture], validate: fn(&Fixture) -> EResult<()>) {
    let mut names = std::collections::BTreeSet::new();
    for f in fixtures {
        assert!(names.insert(f.name), "Duplicate fixture: {}", f.name);
        assert!(
            !f.source.is_empty()
                && f.citation.starts_with("provenance/ENG-VAL-001.md#")
                && !f.tolerance.justification.is_empty(),
            "Metadata: {}",
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
pub fn support(model: &mut Model, node: u32, dofs: &[Dof]) {
    model.supports.push(Support {
        id: format!("s{node}"),
        node: NodeId(node),
        restrained: dofs.to_vec(),
    });
}
pub fn load(model: &mut Model, name: &str, values: &[(u32, Dof, f64)]) {
    model.cases.push(LoadCase {
        name: name.into(),
        loads: values
            .iter()
            .map(|&(node, dof, value)| NodalLoad {
                node: NodeId(node),
                values: vec![DofValue { dof, value }],
            })
            .collect(),
    });
}
pub fn dot(matrix: &SparseMatrix, a: &[f64], b: &[f64]) -> EResult<f64> {
    let dense = matrix.to_dense()?;
    let mut value = 0.0;
    for (i, &x) in a.iter().enumerate() {
        for (j, &y) in b.iter().enumerate() {
            value += x * dense.get(i, j).ok_or_else(missing)? * y;
        }
    }
    Ok(value)
}
pub fn contribution(
    model: &Model,
    element: Arc<dyn Element>,
    nodes: [u32; 2],
) -> EResult<(ElementContribution, GeometricProducer)> {
    let map = DofMap::new(&model.nodes)?;
    let ids = nodes.map(NodeId);
    let coords = [map.coordinates(ids[0])?, map.coordinates(ids[1])?];
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let mut location = Vec::new();
    for id in ids {
        for &d in element.dofs_per_node() {
            location.push(map.global_index(id, d)?);
        }
    }
    let entry = ElementContribution {
        stiffness: element
            .respond(&ctx, &vec![0.0; location.len()], &[])?
            .tangent,
        mass_lumped: element.mass(&ctx, MassKind::Lumped)?,
        mass_consistent: element.mass(&ctx, MassKind::Consistent)?,
        location: location.clone(),
        geometric: None,
        loads: vec![],
    };
    let producer = geometric_producer(element, &ctx, location, vec![])?;
    Ok((entry, producer))
}
pub fn shear(
    stiffness: &[f64],
    masses: &[f64],
    chain: bool,
) -> EResult<LinearSystem<ReferenceBackend>> {
    require(
        stiffness.len() == masses.len(),
        "Datos de cadena incompatibles",
    )?;
    let mut model = Model::new();
    for j in 0..=masses.len() {
        let id = j as u32 + 1;
        let mut node = Node::new(NodeId(id), Dim::D2, [j as f64, 0.0, 0.0]);
        node.dofs = vec![Dof::Ux, Dof::Uy];
        model.nodes.push(node);
        support(
            &mut model,
            id,
            if j == 0 {
                &[Dof::Ux, Dof::Uy]
            } else {
                &[Dof::Uy]
            },
        );
        if j > 0 {
            model.masses.push(NodalMass {
                id: format!("m{j}"),
                node: NodeId(id),
                values: vec![DofValue {
                    dof: Dof::Ux,
                    value: masses[j - 1],
                }],
            });
        }
    }
    let map = DofMap::new(&model.nodes)?;
    let mut entries = Vec::new();
    for (i, &k) in stiffness.iter().enumerate() {
        let ids = [i as u32 + 1, i as u32 + 2];
        if chain {
            // Axial springs EA/ℓ=k with massless truss members and point masses.
            let e: Arc<dyn Element> = Arc::new(LinearTruss::new(Dim::D2, k, 1.0, 0.0)?);
            entries.push(contribution(&model, e, ids)?.0);
        } else {
            entries.push(ElementContribution {
                location: vec![
                    map.global_index(NodeId(ids[0]), Dof::Ux)?,
                    map.global_index(NodeId(ids[1]), Dof::Ux)?,
                ],
                stiffness: DMat::new(2, 2, vec![k, -k, -k, k])?,
                mass_lumped: None,
                mass_consistent: None,
                geometric: None,
                loads: vec![],
            });
        }
    }
    LinearSystem::from_model(ReferenceBackend, &model, &entries, &[])
}
pub fn beam(
    mesh: usize,
    cantilever: bool,
    angle: f64,
    data: FrameData,
) -> EResult<LinearSystem<ReferenceBackend>> {
    let mut model = Model::new();
    for j in 0..=mesh {
        let x = data.length * j as f64 / mesh as f64;
        model.nodes.push(Node::new(
            NodeId(j as u32 + 1),
            Dim::D2,
            [x * math::cos(angle), x * math::sin(angle), 0.0],
        ));
    }
    support(
        &mut model,
        1,
        if cantilever {
            Dim::D2.dofs()
        } else {
            &[Dof::Ux, Dof::Uy]
        },
    );
    if !cantilever {
        support(&mut model, mesh as u32 + 1, &[Dof::Ux, Dof::Uy]);
    }
    let e: Arc<dyn Element> = Arc::new(EulerFrame2D::new(
        data.e,
        data.area,
        data.inertia,
        data.density,
        MomentReleases::default(),
    )?);
    let mut entries = Vec::new();
    for j in 1..=mesh {
        entries.push(contribution(&model, e.clone(), [j as u32, j as u32 + 1])?.0);
    }
    LinearSystem::from_model(ReferenceBackend, &model, &entries, &[])
}
pub fn column(mesh: usize, axial: f64, data: FrameData) -> EResult<LinearSystem<ReferenceBackend>> {
    let mut model = Model::new();
    for i in 0..=mesh {
        model.nodes.push(Node::new(
            NodeId(i as u32 + 1),
            Dim::D2,
            [data.length * i as f64 / mesh as f64, 0.0, 0.0],
        ));
    }
    support(&mut model, 1, Dim::D2.dofs());
    load(&mut model, "G", &[(mesh as u32 + 1, Dof::Ux, -axial)]);
    load(&mut model, "H", &[(mesh as u32 + 1, Dof::Uy, data.force)]);
    let e: Arc<dyn Element> = Arc::new(EulerFrame2D::new(
        data.e,
        data.area,
        data.inertia,
        data.density,
        MomentReleases::default(),
    )?);
    let mut entries = Vec::new();
    let mut producers = Vec::new();
    for j in 1..=mesh {
        let (entry, producer) = contribution(&model, e.clone(), [j as u32, j as u32 + 1])?;
        entries.push(entry);
        producers.push(producer);
    }
    let mut system = LinearSystem::from_model(ReferenceBackend, &model, &entries, &[])?;
    system.set_geometric_producers(producers)?;
    Ok(system)
}
pub fn portal(
    mesh: usize,
    axial: f64,
    rigid: f64,
    data: FrameData,
) -> EResult<LinearSystem<ReferenceBackend>> {
    let mut model = Model::new();
    for col in 0..2 {
        for j in 0..=mesh {
            model.nodes.push(Node::new(
                NodeId((col * (mesh + 1) + j + 1) as u32),
                Dim::D2,
                [
                    data.width * col as f64,
                    data.length * j as f64 / mesh as f64,
                    0.0,
                ],
            ));
        }
        support(&mut model, (col * (mesh + 1) + 1) as u32, Dim::D2.dofs());
    }
    let tips = [(mesh + 1) as u32, (2 * (mesh + 1)) as u32];
    load(
        &mut model,
        "G",
        &[
            (tips[0], Dof::Uy, -axial / 2.0),
            (tips[1], Dof::Uy, -axial / 2.0),
        ],
    );
    load(
        &mut model,
        "H",
        &[
            (tips[0], Dof::Ux, data.force / 2.0),
            (tips[1], Dof::Ux, data.force / 2.0),
        ],
    );
    let column: Arc<dyn Element> = Arc::new(EulerFrame2D::new(
        data.e,
        data.area * rigid,
        data.inertia,
        data.density,
        MomentReleases::default(),
    )?);
    let beam: Arc<dyn Element> = Arc::new(EulerFrame2D::new(
        data.e,
        data.area * rigid,
        data.inertia * rigid,
        data.density,
        MomentReleases::default(),
    )?);
    let mut entries = Vec::new();
    let mut producers = Vec::new();
    for col in 0..2 {
        for j in 1..=mesh {
            let id = (col * (mesh + 1) + j) as u32;
            let (entry, producer) = contribution(&model, column.clone(), [id, id + 1])?;
            entries.push(entry);
            producers.push(producer);
        }
    }
    let (entry, producer) = contribution(&model, beam, tips)?;
    entries.push(entry);
    producers.push(producer);
    let mut system = LinearSystem::from_model(ReferenceBackend, &model, &entries, &[])?;
    system.set_geometric_producers(producers)?;
    Ok(system)
}
