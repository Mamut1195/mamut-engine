//! Owned linear system; global assembly, reduction, and atomic state.
use crate::{
    assemble::{
        assemble_matrix, assemble_model, location, missing_case, multiply, triplets, GlobalAssembly,
    },
    error, finite, reactions, size, Constraints, DiaphragmForces, DiaphragmNodalForce, DofMap,
    ElementContribution, PrescribedDof, Reaction, RigidDiaphragm,
};
use engine_contracts::{CaseRef, MassKind, SystemView};
use engine_core::{Dof, DofIndex, EResult, EngineError, EntityRef, NodeId, Real};
use engine_linalg::{
    DMat, LinAlgBackend, SparseMatrix, SparsityPattern, SymFactorization, SymMatrix, Triplet,
};
use engine_model::Model;
use std::collections::{BTreeMap, BTreeSet};

/// Deterministic global element Kg evaluator from global u_e (m/rad).
/// Does not require Send/Sync; errors propagate without modifying output or trial.
pub type GeometricEval = dyn Fn(&[Real]) -> EResult<DMat>;

/// State-dependent geometric stiffness producer, for buckling/P-Delta.
///
/// u_e contains total global displacements, including prescribed ones, in
/// location order. The producer computes axial force N from u_e and returns
/// square, finite, exactly symmetric global Kg in that same order.
/// Units: N/m, N, or N·m according to row and column translations/rotations.
/// The closure must be deterministic and not modify element history.
pub struct GeometricProducer {
    /// Distinct global indices, in element order (not reduced indices).
    pub location: Vec<DofIndex>,
    /// Global element evaluation, without Send/Sync requirements.
    pub eval: Box<GeometricEval>,
}

type GeometricBlock = (Vec<DofIndex>, Option<DMat>);

fn produced_geometric<B: LinAlgBackend>(
    backend: &mut B,
    order: usize,
    elements: &[GeometricBlock],
    producers: &[GeometricProducer],
    global: &[Real],
) -> EResult<SparseMatrix> {
    let matrices = producers
        .iter()
        .map(|p| {
            let u_e: Vec<_> = p.location.iter().map(|d| global[d.0 as usize]).collect();
            (p.eval)(&u_e)
        })
        .collect::<EResult<Vec<_>>>()?;
    let mut blocks = Vec::new();
    for (indices, matrix) in elements {
        if let Some(i) = producers.iter().position(|p| p.location == *indices) {
            blocks.push((indices.as_slice(), &matrices[i]));
        } else if let Some(matrix) = matrix {
            blocks.push((indices.as_slice(), matrix));
        }
    }
    for (p, matrix) in producers.iter().zip(&matrices) {
        if !elements.iter().any(|(indices, _)| *indices == p.location) {
            blocks.push((p.location.as_slice(), matrix));
        }
    }
    assemble_matrix(backend, order, &blocks)
}

/// System with constant K/M, optionally produced Kg, and constant supports.
///
/// Only evaluates registered Kg producers; does not convert units. Its
/// SystemView operations use total reduced vectors in m/rad, completely write out, and never
/// commit implicitly. A failure preserves outputs and states. The load
/// factor scales only F, not u_p. The initial committed state has u_f=0 and prescribed
/// u_p; this does not imply equilibrium. For scaling mixed norms, the analysis
/// method must choose explicit SI length/force scales.
pub struct LinearSystem<B> {
    backend: B,
    map: DofMap,
    constraints: Constraints,
    k: SparseMatrix,
    lumped: SparseMatrix,
    consistent: SparseMatrix,
    kg: SparseMatrix,
    k_free: SymMatrix,
    lumped_free: SymMatrix,
    consistent_free: SymMatrix,
    kg_free: SymMatrix,
    pattern: SparsityPattern,
    forces: BTreeMap<String, Vec<Real>>,
    nodal_forces: BTreeMap<String, Vec<Real>>,
    accepted: Vec<Real>,
    trial: Vec<Real>,
    kg_reference: Vec<Real>,
    /// Is static Kg left without a producer? Independent of whether Kg(ref) is 0.
    has_geometric: bool,
    geometric_elements: Vec<GeometricBlock>,
    geometric_producers: Vec<GeometricProducer>,
}

fn reduced<B: LinAlgBackend>(
    backend: &mut B,
    matrix: &SparseMatrix,
    constraints: &Constraints,
) -> EResult<SymMatrix> {
    let mut entries = Vec::new();
    for t in triplets(matrix) {
        for &(r, a) in constraints.free_transformation_row(DofIndex(t.row as u32))? {
            for &(c, b) in constraints.free_transformation_row(DofIndex(t.col as u32))? {
                // Accumulating a single half and mirroring the result prevents the
                // summation order from producing rounding asymmetry in Tᵀ K T.
                if r >= c {
                    let value = a * t.value * b;
                    finite(&[value])?;
                    entries.push(Triplet {
                        row: r.0 as usize,
                        col: c.0 as usize,
                        value,
                    });
                }
            }
        }
    }
    let n = constraints.free_dofs().len();
    let dense = backend.assemble(n, n, &entries)?.to_dense()?;
    let mut values = dense.values().to_vec();
    for r in 0..n {
        for c in 0..r {
            values[c * n + r] = values[r * n + c];
        }
    }
    SymMatrix::new(DMat::new(n, n, values)?)
}

impl<B: LinAlgBackend> LinearSystem<B> {
    /// Assembles model nodes, supports, cases, and masses plus element contributions.
    ///
    /// location indices are obtained with DofMap::new(&model.nodes).
    /// Elements are summed in input order, then nodal actions/masses
    /// in model order. Nodal masses are added to both MassKind variants.
    /// Empty cases exist and produce zero; an element load must reference
    /// a declared case. The entire boundary used here is validated; members,
    /// sections, laws, and combinations require engine-model::validate in the
    /// preceding layer and are not interpreted in this interface.
    /// Propagates ENG-E-0002/0003/0008/0010/0011; ENG-E-0012 if the backend provides
    /// a pattern inconsistent with the reduced matrices.
    pub fn from_model(
        backend: B,
        model: &Model,
        elements: &[ElementContribution],
        prescribed: &[PrescribedDof],
    ) -> EResult<Self> {
        Self::from_model_with_diaphragms(backend, model, elements, prescribed, &[])
    }

    /// Assembles and reduces through D3 XY diaphragms, without modifying Model.
    ///
    /// Preserves from_model when the list is empty. Uses u = T_f q_f + u_0,
    /// K_red = T_fᵀ K T_f and F_red = T_fᵀ F; the RHS subtracts T_fᵀ K u_0.
    /// M and Kg, including producers, receive the same transformation.
    /// Propagates diagnostics from from_model and Constraints::with_diaphragms.
    pub fn from_model_with_diaphragms(
        mut backend: B,
        model: &Model,
        elements: &[ElementContribution],
        prescribed: &[PrescribedDof],
        diaphragms: &[RigidDiaphragm],
    ) -> EResult<Self> {
        let map = DofMap::new(&model.nodes)?;
        let constraints =
            Constraints::with_diaphragms(&map, &model.supports, prescribed, diaphragms)?;
        let GlobalAssembly {
            k,
            lumped,
            consistent,
            kg,
            forces,
            nodal_forces,
        } = assemble_model(&mut backend, &map, model, elements)?;
        let k_free = reduced(&mut backend, &k, &constraints)?;
        let lumped_free = reduced(&mut backend, &lumped, &constraints)?;
        let consistent_free = reduced(&mut backend, &consistent, &constraints)?;
        let kg_free = reduced(&mut backend, &kg, &constraints)?;
        // Structural union of K, M, and Kg, including their explicit zeros.
        let mut coordinates = Vec::new();
        for matrix in [&k, &lumped, &consistent, &kg] {
            for t in triplets(matrix) {
                for &(r, _) in constraints.free_transformation_row(DofIndex(t.row as u32))? {
                    for &(c, _) in constraints.free_transformation_row(DofIndex(t.col as u32))? {
                        coordinates.push((r.0 as usize, c.0 as usize));
                    }
                }
            }
        }
        let nf = constraints.free_dofs().len();
        let pattern = SparsityPattern::from_coordinates(nf, coordinates)?;
        if pattern.order() != k_free.order() {
            return Err(error(
                "ENG-E-0012",
                "Patrón incompatible con matriz reducida.",
            ));
        }
        // Also validates the initial state's internal force with imposed u_p.
        multiply(&k, &constraints.expand(&vec![0.0; nf])?)?;
        Ok(Self {
            backend,
            map,
            constraints,
            k,
            lumped,
            consistent,
            kg,
            k_free,
            lumped_free,
            consistent_free,
            kg_free,
            pattern,
            forces,
            nodal_forces,
            accepted: vec![0.0; nf],
            trial: vec![0.0; nf],
            kg_reference: vec![0.0; nf],
            has_geometric: elements.iter().any(|e| e.geometric.is_some()),
            geometric_elements: elements
                .iter()
                .map(|e| (e.location.clone(), e.geometric.clone()))
                .collect(),
            geometric_producers: Vec::new(),
        })
    }

    /// Global map; combine global_index with constraints().reduced_index.
    pub fn dof_map(&self) -> &DofMap {
        &self.map
    }
    /// Constant transformation and free/prescribed independent partition.
    pub fn constraints(&self) -> &Constraints {
        &self.constraints
    }
    /// Looks up node/DOF → reduced index; None means prescribed or dependent.
    /// For slaves consult constraints().free_transformation_row().
    /// ENG-E-0003 for a nonexistent node, ENG-E-0011 for an undeclared component.
    pub fn reduced_index(&self, node: NodeId, dof: Dof) -> EResult<Option<DofIndex>> {
        self.constraints
            .reduced_index(self.map.global_index(node, dof)?)
    }
    /// Global CSR stiffness in SI, without eliminating supported DOFs.
    pub fn stiffness(&self) -> &SparseMatrix {
        &self.k
    }
    /// Global CSR mass (kg, kg·m, kg·m²); includes nodal masses.
    pub fn global_mass(&self, kind: MassKind) -> &SparseMatrix {
        match kind {
            MassKind::Lumped => &self.lumped,
            MassKind::Consistent => &self.consistent,
        }
    }

    /// Indicates whether M_ff contains exactly zero rows/columns after supports and MPC.
    /// Uses no tolerances and does not classify a small positive mass as zero.
    pub fn has_massless_dofs(&self, kind: MassKind) -> bool {
        let mass = match kind {
            MassKind::Lumped => &self.lumped_free,
            MassKind::Consistent => &self.consistent_free,
        };
        mass.dense()
            .values()
            .chunks_exact(mass.order().max(1))
            .any(|row| row.iter().all(|&v| v == 0.0))
    }

    /// Lowest modes of K_ff and M_ff, with the system's own backend.
    /// Both matrices come from global assembly and the partition of free
    /// DOFs. Condenses zero-mass rows with Guyan and recovers complete modes.
    /// Does not modify states; without massless DOFs preserves the original solver.
    /// Propagates diagnostics from `engine_linalg::lowest_modes`, mapping
    /// reduced ENG-E-0006/0014 indices to global node/DOF when provided.
    /// A nonexistent diagnostic index returns ENG-E-0012.
    pub fn lowest_modes(
        &mut self,
        kind: MassKind,
        count: usize,
    ) -> EResult<engine_linalg::LowestModes> {
        let mass = match kind {
            MassKind::Lumped => &self.lumped_free,
            MassKind::Consistent => &self.consistent_free,
        };
        let mut solve = || -> EResult<engine_linalg::LowestModes> {
            if let Some(reduction) =
                engine_linalg::guyan_condense(&mut self.backend, &self.k_free, mass)?
            {
                let mut spectrum = self
                    .backend
                    .lowest_modes(&reduction.stiffness, &reduction.mass, count)
                    .map_err(|mut e| {
                        for entity in &mut e.entities {
                            if let EntityRef::Dof(d) = entity {
                                if let Some(&original) = reduction.massive().get(d.0 as usize) {
                                    *d = DofIndex(original as u32);
                                }
                            }
                        }
                        e
                    })?;
                spectrum.vectors = reduction.recover(&spectrum.vectors)?;
                Ok(spectrum)
            } else {
                self.backend.lowest_modes(&self.k_free, mass, count)
            }
        };
        solve().map_err(|e| {
            if e.code == "ENG-E-0006" && !e.entities.iter().any(|v| matches!(v, EntityRef::Dof(_)))
            {
                e
            } else {
                self.translate_solver_error(e)
            }
        })
    }
    /// Static global CSR snapshot received in from_model.
    /// Does not include replacements or evaluations of registered producers;
    /// consult [`SystemView::geometric`] for the current reduced Kg(u).
    pub fn global_geometric(&self) -> &SparseMatrix {
        &self.kg
    }

    /// Evaluates current global Kg at total reduced u, without altering any state.
    /// Includes producers and prescribed values; applies the same reference and
    /// validation rule as [`SystemView::geometric`]. Allows forming Kg_fp u_p and
    /// second-order reactions without losing constrained blocks.
    pub fn geometric_at(&mut self, u: &[Real]) -> EResult<SparseMatrix> {
        let global = self.constraints.expand(u)?;
        if self.has_geometric && u != self.kg_reference.as_slice() {
            return Err(error(
                "ENG-E-0013",
                "Un aporte Kg estático sin productor exige su estado de referencia.",
            ));
        }
        if self.geometric_producers.is_empty() {
            Ok(self.kg.clone())
        } else {
            produced_geometric(
                &mut self.backend,
                self.map.len(),
                &self.geometric_elements,
                &self.geometric_producers,
                &global,
            )
        }
    }

    /// Constant K_ff; neither evaluates nor modifies trial states.
    pub fn reduced_stiffness(&self) -> &SymMatrix {
        &self.k_free
    }

    /// Factorizes a reduced stiffness of the same order through the internal backend.
    /// Maps mechanisms to node/DOF as [`Self::solve`]; ENG-E-0010 for incorrect
    /// order. Does not regularize or commit states; preserves factor inertia.
    pub fn factorize_reduced(&mut self, matrix: &SymMatrix) -> EResult<Box<dyn SymFactorization>> {
        size(matrix.order(), self.n_free_dofs())?;
        self.backend
            .factorize(matrix)
            .map_err(|e| self.translate_solver_error(e))
    }
    /// Declares the reduced state u (m/rad) at which static Kg was evaluated.
    ///
    /// Default is u = 0. If any Kg contribution remains without a producer,
    /// [`SystemView::geometric`] only accepts that exact reference, even if
    /// Kg(ref) is zero; another u yields ENG-E-0013. Producers do not depend on
    /// this restriction. Without any contribution, Kg=0 for any valid u.
    /// ENG-E-0010 for size; ENG-E-0008 for nonfinite values.
    pub fn set_geometric_reference(&mut self, u: &[Real]) -> EResult<()> {
        size(u.len(), self.n_free_dofs())?;
        finite(u)?;
        self.kg_reference = u.to_vec();
        Ok(())
    }

    /// Atomically replaces the entire Kg(u) producer registry.
    ///
    /// Identity is location with exact equality, including its order. A
    /// producer replaces the static Kg of the single contribution with that location,
    /// even if it was None. Without a match it adds an independent block.
    /// Two producers with the same location or a match with several
    /// contributions are ambiguous and yield ENG-E-0002. Permutations are not matched.
    /// An empty registry restores all original static contributions.
    ///
    /// Validates locations and evaluates at the expanded reference with u_p to
    /// validate sizes, finiteness, and exact symmetry through assemble_matrix. Revalidates
    /// matrices at each geometric(u). Propagates evaluator errors;
    /// ENG-E-0010 for location/size/asymmetry, ENG-E-0008 for nonfinite values.
    /// Extends the pattern with all pairs of free DOFs of each producer,
    /// including zeros. Preserves output, registry, and trial on any error.
    pub fn set_geometric_producers(&mut self, producers: Vec<GeometricProducer>) -> EResult<()> {
        let mut seen = BTreeSet::new();
        for p in &producers {
            location(self.map.len(), &p.location)?;
            if !seen.insert(p.location.clone())
                || self
                    .geometric_elements
                    .iter()
                    .filter(|(indices, _)| *indices == p.location)
                    .count()
                    > 1
            {
                return Err(error(
                    "ENG-E-0002",
                    "Ubicación de productor geométrico ambigua.",
                ));
            }
        }
        let global = self.constraints.expand(&self.kg_reference)?;
        let kg = produced_geometric(
            &mut self.backend,
            self.map.len(),
            &self.geometric_elements,
            &producers,
            &global,
        )?;
        reduced(&mut self.backend, &kg, &self.constraints)?;
        let mut coordinates = Vec::new();
        for matrix in [&self.k, &self.lumped, &self.consistent, &self.kg, &kg] {
            for t in triplets(matrix) {
                for &(r, _) in self
                    .constraints
                    .free_transformation_row(DofIndex(t.row as u32))?
                {
                    for &(c, _) in self
                        .constraints
                        .free_transformation_row(DofIndex(t.col as u32))?
                    {
                        coordinates.push((r.0 as usize, c.0 as usize));
                    }
                }
            }
        }
        let pattern = SparsityPattern::from_coordinates(self.n_free_dofs(), coordinates)?;
        self.has_geometric = self.geometric_elements.iter().any(|(indices, matrix)| {
            matrix.is_some() && !producers.iter().any(|p| p.location == *indices)
        });
        self.geometric_producers = producers;
        self.pattern = pattern;
        Ok(())
    }
    /// Node/DOF label for a reduced index, without panic: ENG-E-0010 if d ≥ n.
    pub fn try_dof_label(&self, d: DofIndex) -> EResult<(NodeId, Dof)> {
        match self.constraints.free_dofs().get(d.0 as usize) {
            Some(&global) => self.map.label(global),
            None => {
                let mut e = error("ENG-E-0010", "Índice reducido fuera de rango.");
                e.entities.push(EntityRef::Dof(d));
                Err(e)
            }
        }
    }
    /// Last committed reduced displacement, in m/rad.
    pub fn accepted_displacement(&self) -> &[Real] {
        &self.accepted
    }
    /// Last successfully evaluated reduced displacement, in m/rad.
    pub fn trial_displacement(&self) -> &[Real] {
        &self.trial
    }

    /// Global linear internal force K u, including constant prescribed values.
    /// Does not modify committed or trial displacements.
    pub fn global_internal_force(&self, free: &[Real]) -> EResult<Vec<Real>> {
        multiply(&self.k, &self.constraints.expand(free)?)
    }

    /// Adds variable element tangents to K and extracts the ff partition.
    /// Blocks are additional contributions: they must not be duplicated in from_model.
    /// Validates location, size, finiteness, and symmetry as assemble_matrix does.
    /// Prescribed values enter internal force by evaluating TOTAL u;
    /// their correction is zero and no K_fp Δu_p term is added to the RHS.
    pub fn tangent_with_blocks(&mut self, blocks: &[(&[DofIndex], &DMat)]) -> EResult<SymMatrix> {
        let extra = assemble_matrix(&mut self.backend, self.map.len(), blocks)?;
        let mut entries = triplets(&self.k);
        entries.extend(triplets(&extra));
        let global = self
            .backend
            .assemble(self.map.len(), self.map.len(), &entries)?;
        reduced(&mut self.backend, &global, &self.constraints)
    }

    /// Solves a reduced correction with ONE tangent factorization.
    /// Without regularization, acceptance, or trial modification. Maps
    /// ENG-E-0006 to node/DOF as solve does; rejects nonfinite values and sizes.
    pub fn solve_correction(
        &mut self,
        tangent: &SymMatrix,
        residual: &[Real],
    ) -> EResult<Vec<Real>> {
        size(tangent.order(), self.n_free_dofs())?;
        size(residual.len(), self.n_free_dofs())?;
        finite(residual)?;
        let rhs = DMat::new(self.n_free_dofs(), 1, residual.to_vec())?;
        let factorization = self
            .backend
            .factorize(tangent)
            .map_err(|e| self.translate_solver_error(e))?;
        let solution = factorization
            .solve(&rhs)
            .map_err(|e| self.translate_solver_error(e))?;
        size(solution.rows(), self.n_free_dofs())?;
        size(solution.cols(), 1)?;
        finite(solution.values())?;
        Ok(solution.values().to_vec())
    }

    /// Physical global case forces factor F (N/N·m), including supports.
    /// ENG-E-0003 for a missing case; ENG-E-0008 for a nonfinite factor/result.
    pub fn global_force(&self, case: &CaseRef, factor: Real) -> EResult<Vec<Real>> {
        finite(&[factor])?;
        let force = self.forces.get(&case.0).ok_or_else(|| missing_case(case))?;
        let result: Vec<_> = force.iter().map(|v| v * factor).collect();
        finite(&result)?;
        Ok(result)
    }

    /// Explicit nodal loads from the model, without equivalent span loads.
    /// Same global order and N/N·m units as [`Self::global_force`]. They are
    /// preserved during assembly to check postprocessing equilibrium.
    /// ENG-E-0003 for a missing case; ENG-E-0008 for a nonfinite factor/result.
    pub fn global_nodal_force(&self, case: &CaseRef, factor: Real) -> EResult<Vec<Real>> {
        finite(&[factor])?;
        let force = self
            .nodal_forces
            .get(&case.0)
            .ok_or_else(|| missing_case(case))?;
        let result: Vec<_> = force.iter().map(|v| v * factor).collect();
        finite(&result)?;
        Ok(result)
    }

    /// Direct RHS: T_fᵀ(factor F − K u_0) (N/N·m).
    /// u_p is not scaled by factor; errors preserve the system.
    pub fn reduced_rhs(&self, case: &CaseRef, factor: Real) -> EResult<Vec<Real>> {
        let force = self.global_force(case, factor)?;
        let shift = multiply(
            &self.k,
            &self.constraints.expand(&vec![0.0; self.n_free_dofs()])?,
        )?;
        let residual: Vec<_> = force.iter().zip(shift).map(|(f, k)| f - k).collect();
        self.constraints.project_force(&residual)
    }

    /// Solves (T_fᵀ K T_f) q_f = RHS through the backend, without committing the result.
    ///
    /// Convenience for linear problems; this is not an analysis/1 method.
    /// Maps reduced ENG-E-0006 indices to a global index, node entity,
    /// and DOF label in message. The pivot identifies part of the mechanism,
    /// not all its motions. ENG-E-0012 if the backend reports an invalid index.
    pub fn solve(&mut self, case: &CaseRef, factor: Real) -> EResult<Vec<Real>> {
        let rhs = DMat::new(self.n_free_dofs(), 1, self.reduced_rhs(case, factor)?)?;
        let factorization = self
            .backend
            .factorize(&self.k_free)
            .map_err(|e| self.translate_solver_error(e))?;
        let solution = factorization
            .solve(&rhs)
            .map_err(|e| self.translate_solver_error(e))?;
        size(solution.rows(), self.n_free_dofs())?;
        size(solution.cols(), 1)?;
        finite(solution.values())?;
        Ok(solution.values().to_vec())
    }

    /// Solves several cases with ONE factorization of K_red = T_fᵀ K T_f.
    ///
    /// Preserves input order and neither commits nor modifies trials. Each RHS
    /// is `T_fᵀ(factor F − K u_0)`: prescribed values are not scaled. Validates all
    /// cases before factorization. ENG-E-0010 for an empty list or duplicate
    /// names; ENG-E-0003 for a missing case; ENG-E-0008 for nonfinite values.
    /// Maps backend errors as [`Self::solve`] does, including
    /// ENG-E-0006 mechanisms with node/DOF and ENG-E-0012 invalid diagnostics.
    /// Returns free independent displacements, as [`Self::solve`] does.
    /// [`Constraints::expand`] recovers dependent and prescribed DOFs through
    /// `u = T_f q_f + u_0`. Without diaphragms this equals `factor F_f − K_fp u_p`.
    pub fn solve_cases(&mut self, cases: &[(CaseRef, Real)]) -> EResult<Vec<Vec<Real>>> {
        if cases.is_empty() {
            return Err(error("ENG-E-0010", "Lista de casos vacía."));
        }
        let mut names = BTreeSet::new();
        for (case, _) in cases {
            if !names.insert(&case.0) {
                return Err(error("ENG-E-0010", "Caso duplicado en análisis."));
            }
        }
        let rhs = cases
            .iter()
            .map(|(case, factor)| {
                DMat::new(self.n_free_dofs(), 1, self.reduced_rhs(case, *factor)?)
            })
            .collect::<EResult<Vec<_>>>()?;
        let factorization = self
            .backend
            .factorize(&self.k_free)
            .map_err(|e| self.translate_solver_error(e))?;
        rhs.iter()
            .map(|rhs| {
                let solution = factorization
                    .solve(rhs)
                    .map_err(|e| self.translate_solver_error(e))?;
                size(solution.rows(), self.n_free_dofs())?;
                size(solution.cols(), 1)?;
                finite(solution.values())?;
                Ok(solution.values().to_vec())
            })
            .collect()
    }

    /// Recovers R = (Tᵀ(Ku − factor F))_p at every prescribed independent DOF:
    /// those constrained by supports and those with imposed displacement (settlement).
    /// Includes force transferred by slaves to a prescribed master.
    /// Without diaphragms this equals K_pf u_f + K_pp u_p − factor F_p.
    /// There is no support reaction at a free or dependent DOF. Stable global order; units
    /// N/N·m. Neither commits nor modifies trials. Includes loads applied
    /// directly to prescribed DOFs.
    pub fn reactions(
        &self,
        u_free: &[Real],
        case: &CaseRef,
        factor: Real,
    ) -> EResult<Vec<Reaction>> {
        reactions::recover(
            &self.map,
            &self.constraints,
            &self.k,
            u_free,
            &self.global_force(case, factor)?,
        )
    }

    /// Recovers actions of each diaphragm on slaves and master.
    /// Signs and moment transfer documented in DiaphragmForces.
    /// Propagates expansion, force, and product errors (0003/0008/0010).
    pub fn diaphragm_forces(
        &self,
        u_free: &[Real],
        case: &CaseRef,
        factor: Real,
    ) -> EResult<Vec<DiaphragmForces>> {
        let u = self.constraints.expand(u_free)?;
        let internal = multiply(&self.k, &u)?;
        let external = self.global_force(case, factor)?;
        let residual: Vec<_> = internal.iter().zip(external).map(|(k, f)| k - f).collect();
        finite(&residual)?;
        let mut result = Vec::new();
        for d in self.constraints.diaphragms() {
            let (_, origin) = self.map.geometry(d.master)?;
            let mut master = [0.0; 3];
            let mut slaves = Vec::new();
            for &node in &d.slaves {
                let (_, coords) = self.map.geometry(node)?;
                let mut values = [0.0; 3];
                for (i, dof) in [Dof::Ux, Dof::Uy, Dof::Rz].into_iter().enumerate() {
                    values[i] = residual[self.map.global_index(node, dof)?.0 as usize];
                }
                master[0] -= values[0];
                master[1] -= values[1];
                master[2] -= values[2] + (coords[0] - origin[0]) * values[1]
                    - (coords[1] - origin[1]) * values[0];
                slaves.push(DiaphragmNodalForce { node, values });
            }
            finite(&master)?;
            result.push(DiaphragmForces {
                master: DiaphragmNodalForce {
                    node: d.master,
                    values: master,
                },
                slaves,
            });
        }
        Ok(result)
    }

    fn evaluate(&self, u: &[Real]) -> EResult<Vec<Real>> {
        let global = self.constraints.expand(u)?;
        let internal = multiply(&self.k, &global)?;
        self.constraints.project_force(&internal)
    }

    fn translate_solver_error(&self, mut e: EngineError) -> EngineError {
        if e.code != "ENG-E-0006" && e.code != "ENG-E-0014" {
            return e;
        }
        let mut entities = Vec::new();
        let mut translated = false;
        for entity in &e.entities {
            if let EntityRef::Dof(d) = entity {
                let Some(&global) = self.constraints.free_dofs().get(d.0 as usize) else {
                    return error(
                        "ENG-E-0012",
                        "El solver informó un GDL reducido inexistente.",
                    );
                };
                let Ok((node, dof)) = self.map.label(global) else {
                    return error("ENG-E-0012", "El solver informó un GDL global inexistente.");
                };
                e.message.push_str(&format!(
                    " Nodo {}, GDL {dof} (global {}).",
                    node.0, global.0
                ));
                entities.extend([EntityRef::Node(node), EntityRef::Dof(global)]);
                translated = true;
            } else {
                entities.push(entity.clone());
            }
        }
        if !translated {
            return error("ENG-E-0012", "El solver omitió el índice del mecanismo.");
        }
        e.entities = entities;
        e
    }
}

impl<B: LinAlgBackend> SystemView for LinearSystem<B> {
    fn n_free_dofs(&self) -> usize {
        self.constraints.free_dofs().len()
    }
    fn pattern(&self) -> &SparsityPattern {
        &self.pattern
    }
    fn tangent(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()> {
        size(out.order(), self.n_free_dofs())?;
        self.evaluate(u)?;
        *out = self.k_free.clone();
        self.trial = u.to_vec();
        Ok(())
    }
    fn internal_force(&mut self, u: &[Real], out: &mut [Real]) -> EResult<()> {
        size(out.len(), self.n_free_dofs())?;
        let force = self.evaluate(u)?;
        out.copy_from_slice(&force);
        self.trial = u.to_vec();
        Ok(())
    }
    fn mass(&mut self, kind: MassKind, out: &mut SymMatrix) -> EResult<()> {
        size(out.order(), self.n_free_dofs())?;
        *out = match kind {
            MassKind::Lumped => self.lumped_free.clone(),
            MassKind::Consistent => self.consistent_free.clone(),
        };
        Ok(())
    }
    fn geometric(&mut self, u: &[Real], out: &mut SymMatrix) -> EResult<()> {
        size(out.order(), self.n_free_dofs())?;
        let kg = self.geometric_at(u)?;
        let result = if self.geometric_producers.is_empty() {
            self.kg_free.clone()
        } else {
            reduced(&mut self.backend, &kg, &self.constraints)?
        };
        *out = result;
        self.trial = u.to_vec();
        Ok(())
    }
    fn external_force(&self, case: &CaseRef, factor: Real, out: &mut [Real]) -> EResult<()> {
        size(out.len(), self.n_free_dofs())?;
        let force = self.global_force(case, factor)?;
        let reduced = self.constraints.project_force(&force)?;
        out.copy_from_slice(&reduced);
        Ok(())
    }
    fn accept(&mut self, u: &[Real]) -> EResult<()> {
        self.evaluate(u)?;
        self.accepted = u.to_vec();
        self.trial = u.to_vec();
        Ok(())
    }
    fn rollback(&mut self) {
        self.trial.clone_from(&self.accepted);
    }
    /// Contract precondition: d < n. The trait cannot return an error; use
    /// [`LinearSystem::try_dof_label`] for untrusted inputs. If the
    /// precondition is violated, returns the sentinel (NodeId(0), Ux), without panic.
    fn dof_label(&self, d: DofIndex) -> (NodeId, Dof) {
        match self.try_dof_label(d) {
            Ok(label) => label,
            // The trait has no error channel. Sentinel for a call
            // outside the contract; try_dof_label returns ENG-E-0010.
            Err(_) => (NodeId(0), Dof::Ux),
        }
    }
}
