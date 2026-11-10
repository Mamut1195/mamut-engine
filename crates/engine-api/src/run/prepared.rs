//! Prepared linearStatic solve that returns nodal displacements only.
use super::{assemble_members, build_members, diagnostic, validation, Failure};
use crate::{input, ApiError, CancelToken, Diagnostic, Status};
use engine_assembly::LinearSystem;
use engine_contracts::CaseRef;
use engine_core::{DofIndex, EngineError, NodeId};
use engine_linalg::{DMat, ReferenceBackend, SymFactorization};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// One node, in the same component order as `results/1` (`Ux`…`Rz`).
#[derive(Clone, Debug, PartialEq)]
pub struct NodeDisplacement {
    /// Node identifier.
    pub id: u32,
    /// Global displacements. Unused components stay zero. −0.0 is stored as +0.0.
    pub displacements: [f64; 6],
}

/// Displacements of one load case.
#[derive(Clone, Debug, PartialEq)]
pub struct CaseDisplacements {
    /// Case name.
    pub case: String,
    /// Nodes in ascending identifier order, the same order as `results/1`.
    pub nodes: Vec<NodeDisplacement>,
}

/// Failure of [`PreparedLinear::prepare`] or [`PreparedLinear::displacements`].
///
/// `status` matches the status `run` would report for the same defect.
#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisFault {
    /// Execution status.
    pub status: Status,
    /// Domain diagnostics, in the order `run` would emit them.
    pub diagnostics: Vec<Diagnostic>,
}

impl fmt::Display for AnalysisFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.diagnostics.first() {
            Some(diagnostic) => write!(f, "{}: {}", diagnostic.code, diagnostic.cause),
            None => write!(f, "{:?}", self.status),
        }
    }
}

impl std::error::Error for AnalysisFault {}

impl From<Failure> for AnalysisFault {
    fn from(failure: Failure) -> Self {
        Self {
            status: failure.0,
            diagnostics: failure.1,
        }
    }
}

impl From<EngineError> for AnalysisFault {
    fn from(error: EngineError) -> Self {
        Failure::from(error).into()
    }
}

impl From<ApiError> for AnalysisFault {
    fn from(error: ApiError) -> Self {
        Failure::from(error).into()
    }
}

struct PreparedCase {
    name: String,
    rhs: Vec<f64>,
}

/// One linearStatic model, factored once.
///
/// `displacements` solves the stored factor again. It does not parse JSON,
/// fingerprint the input, recover member forces, or build `results/1`.
/// `run` is unchanged. Uniform temperature loads use the same joint forces
/// as `run`. Combinations, other analyses, and nonlinear springs
/// are rejected: those callers use `run`.
pub struct PreparedLinear {
    system: LinearSystem<ReferenceBackend>,
    factor: Box<dyn SymFactorization>,
    cases: Vec<PreparedCase>,
    nodes: Vec<u32>,
    /// Global DOF index → (node position, component index).
    slots: Vec<(usize, usize)>,
}

impl fmt::Debug for PreparedLinear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedLinear")
            .field("cases", &self.cases.len())
            .field("nodes", &self.nodes.len())
            .finish()
    }
}

impl PreparedLinear {
    /// Parses, validates, assembles, and factorizes one linearStatic model.
    ///
    /// Requires exactly one analysis, method `linearStatic`, at least one case,
    /// and no combinations. Uniform `temperature_loads` are applied with the
    /// same equivalent joint forces as `run`. Checks `cancel` before parsing
    /// and before factorization.
    pub fn prepare(input_json: &str, cancel: &CancelToken) -> Result<Self, AnalysisFault> {
        cancel.check()?;
        let value = crate::json::parse(input_json)?;
        let schema = value
            .get("schema")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ApiError::InvalidInput("schema is required".into()))?;
        if schema != "engine-input/1" {
            return Err(ApiError::UnsupportedSchema(schema.into()).into());
        }
        let execution = input::decode(value)?;
        if execution.methods.len() != 1 || execution.methods[0] != "linearStatic" {
            return Err(rejected(
                "prepare accepts exactly one linearStatic analysis; use run for other analyses",
            ));
        }
        if !execution.model.combinations.is_empty() {
            return Err(rejected("prepare does not accept combinations; use run"));
        }
        if execution.model.cases.is_empty() {
            return Err(
                ApiError::InvalidInput("linearStatic requires at least one case".into()).into(),
            );
        }
        crate::methods::resolve(
            execution.methods[0],
            crate::methods::INPUT_CONTRACT,
            execution.allow_experimental,
        )?;
        let diagnostics = validation(&execution);
        if diagnostics.iter().any(|d| d.severity == "error") {
            let status = if diagnostics.iter().any(|d| d.code == "ENG-E-0013") {
                Status::Unsupported
            } else {
                Status::Invalid
            };
            return Err(AnalysisFault {
                status,
                diagnostics,
            });
        }
        let members = build_members(&execution)?;
        if members.iter().any(|member| member.variable) {
            return Err(rejected(
                "prepare does not accept nonlinear springs; use run",
            ));
        }
        let model_for_system = super::temperature::model_for_system(
            &execution.model,
            &members,
            &execution.temperature,
        )?;
        let contributions = assemble_members(&execution.model, &members)?;
        let mut system = LinearSystem::from_model_with_diaphragms_static(
            ReferenceBackend,
            model_for_system.as_ref(),
            &contributions,
            &execution.prescribed,
            &execution.diaphragms,
        )?;
        let NodeLayout { nodes, slots } = layout(system.dof_map())?;
        let mut cases = Vec::with_capacity(execution.model.cases.len());
        for case in &execution.model.cases {
            let name = CaseRef(case.name.clone());
            let rhs = system.reduced_rhs(&name, 1.0)?;
            cases.push(PreparedCase {
                name: case.name.clone(),
                rhs,
            });
        }
        cancel.check()?;
        let factor = system.factorize_free()?;
        Ok(Self {
            system,
            factor,
            cases,
            nodes,
            slots,
        })
    }

    /// Solves every case again and returns nodal displacements.
    ///
    /// Node order and component order match `results/1`. Member forces,
    /// reactions, and the results document are not computed.
    pub fn displacements(&self) -> Result<Vec<CaseDisplacements>, AnalysisFault> {
        let n = self.cases.first().map_or(0, |case| case.rhs.len());
        let cols = self.cases.len();
        let mut values = vec![0.0; n.saturating_mul(cols)];
        for (column, case) in self.cases.iter().enumerate() {
            if case.rhs.len() != n {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Prepared case has an unexpected number of unknowns",
                )
                .into());
            }
            for (row, &value) in case.rhs.iter().enumerate() {
                values[row * cols + column] = value;
            }
        }
        let load = DMat::new(n, cols, values)?;
        let solved = self.factor.solve(&load)?;
        if solved.rows() != n || solved.cols() != cols {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Prepared solve returned an unexpected size",
            )
            .into());
        }
        let mut cases = Vec::with_capacity(cols);
        for (column, case) in self.cases.iter().enumerate() {
            let free: Vec<f64> = (0..n)
                .map(|row| solved.values()[row * cols + column])
                .collect();
            let global = self.system.constraints().expand(&free)?;
            if global.len() != self.slots.len() {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Prepared displacement vector does not match the DOF map",
                )
                .into());
            }
            let mut nodes = vec![[0.0; 6]; self.nodes.len()];
            for (value, &(position, component)) in global.iter().zip(&self.slots) {
                let value = if *value == 0.0 { 0.0 } else { *value };
                nodes[position][component] = value;
            }
            cases.push(CaseDisplacements {
                case: case.name.clone(),
                nodes: self
                    .nodes
                    .iter()
                    .zip(nodes)
                    .map(|(&id, displacements)| NodeDisplacement { id, displacements })
                    .collect(),
            });
        }
        Ok(cases)
    }
}

fn rejected(cause: &str) -> AnalysisFault {
    AnalysisFault {
        status: Status::Invalid,
        diagnostics: vec![diagnostic("API-E-INPUT", "error", cause.to_owned())],
    }
}

struct NodeLayout {
    nodes: Vec<u32>,
    slots: Vec<(usize, usize)>,
}

fn layout(map: &engine_assembly::DofMap) -> Result<NodeLayout, EngineError> {
    let mut labeled = Vec::with_capacity(map.len());
    let mut ids = BTreeSet::new();
    for i in 0..map.len() {
        let index = u32::try_from(i)
            .map_err(|_| EngineError::new("ENG-E-0010", "Global index exceeds u32."))?;
        let (id, dof) = map.label(DofIndex(index))?;
        labeled.push((id, dof.index()));
        ids.insert(id);
    }
    // BTreeSet iteration is sorted by node id, matching results/1.
    let nodes: Vec<u32> = ids.iter().map(|id| id.0).collect();
    let rank: BTreeMap<NodeId, usize> = ids
        .into_iter()
        .enumerate()
        .map(|(position, id)| (id, position))
        .collect();
    let mut slots = Vec::with_capacity(labeled.len());
    for (id, component) in labeled {
        let position = rank.get(&id).copied().ok_or_else(|| {
            EngineError::new("ENG-E-0012", "Node missing from the displacement layout")
        })?;
        slots.push((position, component));
    }
    Ok(NodeLayout { nodes, slots })
}
