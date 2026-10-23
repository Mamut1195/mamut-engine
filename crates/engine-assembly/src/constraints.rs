//! Master-slave transformation and elimination of constant prescribed values in SI.
use crate::{
    diaphragm::{relations, Relations},
    error, finite, size, DofMap, RigidDiaphragm,
};
use engine_core::{Dof, DofIndex, EResult, NodeId, Real};
use engine_linalg::DMat;
use engine_model::Support;
use std::collections::BTreeSet;

/// Displacement or rotation imposed on a global nodal DOF.
#[derive(Clone, Copy, Debug)]
pub struct PrescribedDof {
    /// Receiving node.
    pub node: NodeId,
    /// Component in global axes.
    pub dof: Dof,
    /// Translation in m or rotation in rad, constant and independent of the load factor.
    pub value: Real,
}

/// Immutable transformation and partition of free/prescribed independent DOFs.
///
/// Domain supports constrain to zero. A PrescribedDof declaration may
/// replace that zero or constrain a previously free DOF. Repeating a DOF
/// among supports, or among prescribed declarations, is an error even if the
/// value matches. The support/prescribed intersection is deliberately allowed.
#[derive(Clone, Debug)]
pub struct Constraints {
    values: Vec<Option<Real>>,
    free: Vec<DofIndex>,
    reduced: Vec<Option<DofIndex>>,
    relations: Relations,
    independent: Vec<DofIndex>,
    rows: Relations,
    diaphragms: Vec<RigidDiaphragm>,
}

impl Constraints {
    /// Constructs the partition on map; all DOFs constrained yields ENG-E-0010.
    ///
    /// Propagates invalid map references/indices; ENG-E-0002 for duplicate support or
    /// DOF; ENG-E-0008 for a nonfinite value. Equivalent to with_diaphragms without diaphragms.
    pub fn new(map: &DofMap, supports: &[Support], prescribed: &[PrescribedDof]) -> EResult<Self> {
        Self::with_diaphragms(map, supports, prescribed, &[])
    }

    /// Constructs u = T_f q_f + u_0 with XY diaphragms in D3.
    ///
    /// All independent DOFs are transformed first (including prescribed
    /// ones), then prescribed ones are eliminated. A master may have a
    /// support or imposed value. Any prescribed or supported slave DOF
    /// is rejected with ENG-E-0013, including out of plane, to avoid ambiguity.
    /// Uz/Rx/Ry remain independent and receive no new constraints.
    /// Propagates diagnostics from new and RigidDiaphragm; all independent
    /// DOFs prescribed is rejected with ENG-E-0010.
    pub fn with_diaphragms(
        map: &DofMap,
        supports: &[Support],
        prescribed: &[PrescribedDof],
        diaphragms: &[RigidDiaphragm],
    ) -> EResult<Self> {
        let relations = relations(map, diaphragms)?;
        let slaves: BTreeSet<_> = diaphragms
            .iter()
            .flat_map(|d| d.slaves.iter().copied())
            .collect();
        let mut values = vec![None; map.len()];
        let mut ids = BTreeSet::new();
        for support in supports {
            // Even an empty support must point to an existing node.
            if !map.contains_node(support.node) {
                let mut e = error(
                    "ENG-E-0003",
                    format!("Support {} is on a nonexistent node.", support.id),
                );
                e.entities.push(engine_core::EntityRef::Node(support.node));
                return Err(e);
            }
            if !ids.insert(&support.id) {
                return Err(error(
                    "ENG-E-0002",
                    format!("Duplicate support {}.", support.id),
                ));
            }
            for &dof in &support.restrained {
                let i = map.global_index(support.node, dof)?.0 as usize;
                if values[i].replace(0.0).is_some() {
                    return Err(error(
                        "ENG-E-0002",
                        format!("Duplicate support: node {}, DOF {dof}.", support.node.0),
                    ));
                }
            }
        }
        let mut seen = BTreeSet::new();
        for p in prescribed {
            finite(&[p.value])?;
            let i = map.global_index(p.node, p.dof)?.0 as usize;
            if !seen.insert(i) {
                return Err(error("ENG-E-0002", "Duplicate imposed displacement."));
            }
            values[i] = Some(p.value);
        }
        for (i, value) in values.iter().enumerate() {
            if value.is_some() {
                let (node, _) = map.label(DofIndex(i as u32))?;
                if slaves.contains(&node) {
                    let mut e = error(
                        "ENG-E-0013",
                        "A supported or prescribed DOF is not allowed on a slave.",
                    );
                    e.entities.push(engine_core::EntityRef::Node(node));
                    return Err(e);
                }
            }
        }
        let independent: Vec<_> = relations
            .iter()
            .enumerate()
            .filter_map(|(i, row)| {
                (row.as_slice() == [(DofIndex(i as u32), 1.0)]).then_some(DofIndex(i as u32))
            })
            .collect();
        let mut free = Vec::new();
        let mut reduced = vec![None; map.len()];
        for index in &independent {
            let i = index.0 as usize;
            if values[i].is_none() {
                reduced[i] = Some(DofIndex(free.len() as u32));
                free.push(DofIndex(i as u32));
            }
        }
        if free.is_empty() {
            return Err(error("ENG-E-0010", "All DOFs are constrained."));
        }
        let rows = relations
            .iter()
            .map(|row| {
                row.iter()
                    .filter_map(|&(i, a)| reduced[i.0 as usize].map(|r| (r, a)))
                    .collect()
            })
            .collect();
        Ok(Self {
            relations,
            independent,
            rows,
            diaphragms: diaphragms.to_vec(),
            values,
            free,
            reduced,
        })
    }

    /// Free independent global indices; their position is the reduced index.
    pub fn free_dofs(&self) -> &[DofIndex] {
        &self.free
    }

    /// Prescribed values in global order; None includes free and dependent DOFs.
    pub fn prescribed_values(&self) -> &[Option<Real>] {
        &self.values
    }

    /// Reduced index of an independent DOF; None if prescribed or slave.
    /// For a slave use free_transformation_row: it may depend on several DOFs.
    /// ENG-E-0010 if the global index is out of range.
    pub fn reduced_index(&self, global: DofIndex) -> EResult<Option<DofIndex>> {
        self.reduced
            .get(global.0 as usize)
            .copied()
            .ok_or_else(|| error("ENG-E-0010", "Global index is out of range in constraints."))
    }

    /// Reconstructs global u with u_f (m/rad) and constant u_p values.
    /// ENG-E-0010 for size, ENG-E-0008 for nonfinite displacements.
    pub fn expand(&self, free: &[Real]) -> EResult<Vec<Real>> {
        size(free.len(), self.free.len())?;
        finite(free)?;
        let mut result: Vec<_> = self
            .values
            .iter()
            .map(|v| match v {
                Some(x) => *x,
                None => 0.0,
            })
            .collect();
        for (index, value) in self.free.iter().zip(free) {
            result[index.0 as usize] = *value;
        }
        let expanded: Vec<_> = self
            .relations
            .iter()
            .map(|row| row.iter().map(|&(i, a)| a * result[i.0 as usize]).sum())
            .collect();
        finite(&expanded)?;
        Ok(expanded)
    }

    /// Global independent DOFs, including prescribed ones, in stable order.
    pub fn independent_dofs(&self) -> &[DofIndex] {
        &self.independent
    }

    /// Matrix T: full independent DOFs → full global DOFs.
    /// Columns follow independent_dofs; includes prescribed masters.
    /// Rows are dimensionless except for arms coupling m with rad.
    /// ENG-E-0010 for size/allocation unrepresentable according to DMat.
    pub fn transformation(&self) -> EResult<DMat> {
        let mut matrix = DMat::zeros(self.relations.len(), self.independent.len())?
            .values()
            .to_vec();
        for (r, row) in self.relations.iter().enumerate() {
            for &(i, a) in row {
                let c = self
                    .independent
                    .binary_search(&i)
                    .map_err(|_| error("ENG-E-0012", "Inconsistent transformation."))?;
                matrix[r * self.independent.len() + c] = a;
            }
        }
        DMat::new(self.relations.len(), self.independent.len(), matrix)
    }

    /// Sparse row of T_f: (free reduced index, coefficient) pairs.
    /// A prescribed row may be empty; u_0 is obtained with expand(q_f=0).
    /// ENG-E-0010 for an invalid global index.
    pub fn free_transformation_row(&self, global: DofIndex) -> EResult<&[(DofIndex, Real)]> {
        self.rows
            .get(global.0 as usize)
            .map(Vec::as_slice)
            .ok_or_else(|| error("ENG-E-0010", "Transformation row is out of range."))
    }

    /// Validated diaphragms in declaration order.
    pub fn diaphragms(&self) -> &[RigidDiaphragm] {
        &self.diaphragms
    }

    /// Projects global forces through T_fᵀ, in N/N·m.
    /// ENG-E-0010 for size; ENG-E-0008 for nonfinite data or accumulations.
    pub fn project_force(&self, force: &[Real]) -> EResult<Vec<Real>> {
        let projected = self.project_independent(force)?;
        Ok(self.free.iter().map(|i| projected[i.0 as usize]).collect())
    }

    pub(crate) fn project_independent(&self, force: &[Real]) -> EResult<Vec<Real>> {
        size(force.len(), self.relations.len())?;
        finite(force)?;
        let mut result = vec![0.0; force.len()];
        for (row, &f) in self.relations.iter().zip(force) {
            for &(i, a) in row {
                result[i.0 as usize] += a * f;
            }
        }
        finite(&result)?;
        Ok(result)
    }
}
