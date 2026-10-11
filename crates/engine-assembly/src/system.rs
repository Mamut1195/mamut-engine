//! Sistema lineal poseído; ensamblaje global, reducción y estado atómico.
use crate::{
    assemble::{assemble_model, missing_case, multiply, triplets, GlobalAssembly},
    error, finite, reactions, size, Constraints, DofMap, ElementContribution, PrescribedDof,
    Reaction,
};
use engine_contracts::{CaseRef, MassKind, SystemView};
use engine_core::{Dof, DofIndex, EResult, EngineError, EntityRef, NodeId, Real};
use engine_linalg::{DMat, LinAlgBackend, SparseMatrix, SparsityPattern, SymMatrix, Triplet};
use engine_model::Model;
use std::collections::BTreeMap;

/// Sistema de matrices constantes con desplazamientos impuestos constantes.
///
/// No evalúa leyes/elementos ni convierte unidades. Las operaciones SystemView
/// usan vectores reducidos totales en m/rad, escriben out completamente y nunca
/// aceptan implícitamente. Un fallo conserva salidas y estados. El factor de
/// carga escala solo F, no u_p. El estado inicial aceptado tiene u_f=0 y u_p
/// prescrito; no implica equilibrio. Para escalados de normas mixtas, el método
/// de análisis debe escoger escalas SI de longitud/fuerza explícitas.
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
    accepted: Vec<Real>,
    trial: Vec<Real>,
}

fn reduced<B: LinAlgBackend>(
    backend: &mut B,
    matrix: &SparseMatrix,
    constraints: &Constraints,
) -> EResult<SymMatrix> {
    let mut entries = Vec::new();
    for row in 0..matrix.rows() {
        if let Some(r) = constraints.reduced_index(DofIndex(row as u32))? {
            for p in matrix.row_offsets()[row]..matrix.row_offsets()[row + 1] {
                if let Some(c) =
                    constraints.reduced_index(DofIndex(matrix.column_indices()[p] as u32))?
                {
                    entries.push(Triplet {
                        row: r.0 as usize,
                        col: c.0 as usize,
                        value: matrix.values()[p],
                    });
                }
            }
        }
    }
    let n = constraints.free_dofs().len();
    SymMatrix::new(backend.assemble(n, n, &entries)?.to_dense()?)
}

impl<B: LinAlgBackend> LinearSystem<B> {
    /// Ensambla nodos, apoyos, casos y masas del modelo más aportes elementales.
    ///
    /// Los índices de location se obtienen con DofMap::new(&model.nodes).
    /// Se suman elementos en orden de entrada y después acciones/masas nodales
    /// en orden del modelo. Las masas nodales se añaden a ambos MassKind.
    /// Los casos vacíos existen y producen cero; una carga elemental debe citar
    /// un caso declarado. Se valida toda la frontera usada aquí; miembros,
    /// secciones, leyes y combinaciones requieren engine-model::validate en la
    /// capa anterior y no son interpretados en esta interfaz.
    /// Propaga ENG-E-0002/0003/0008/0010/0011; ENG-E-0012 si el backend entrega
    /// un patrón incoherente con las matrices reducidas.
    pub fn from_model(
        mut backend: B,
        model: &Model,
        elements: &[ElementContribution],
        prescribed: &[PrescribedDof],
    ) -> EResult<Self> {
        let map = DofMap::new(&model.nodes)?;
        let constraints = Constraints::new(&map, &model.supports, prescribed)?;
        let GlobalAssembly {
            k,
            lumped,
            consistent,
            kg,
            forces,
        } = assemble_model(&mut backend, &map, model, elements)?;
        let k_free = reduced(&mut backend, &k, &constraints)?;
        let lumped_free = reduced(&mut backend, &lumped, &constraints)?;
        let consistent_free = reduced(&mut backend, &consistent, &constraints)?;
        let kg_free = reduced(&mut backend, &kg, &constraints)?;
        // Unión estructural de K, M y Kg, incluidos sus ceros explícitos.
        let mut coordinates = Vec::new();
        for matrix in [&k, &lumped, &consistent, &kg] {
            for t in triplets(matrix) {
                if let (Some(r), Some(c)) = (
                    constraints.reduced_index(DofIndex(t.row as u32))?,
                    constraints.reduced_index(DofIndex(t.col as u32))?,
                ) {
                    coordinates.push((r.0 as usize, c.0 as usize));
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
        // Valida también la fuerza interna del estado inicial con u_p impuesto.
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
            accepted: vec![0.0; nf],
            trial: vec![0.0; nf],
        })
    }

    /// Mapa global; combine global_index con constraints().reduced_index.
    pub fn dof_map(&self) -> &DofMap {
        &self.map
    }
    /// Partición libre/prescrita constante del sistema.
    pub fn constraints(&self) -> &Constraints {
        &self.constraints
    }
    /// Busca nodo/GDL → índice reducido; None significa GDL restringido.
    /// ENG-E-0003 por nodo inexistente, ENG-E-0011 por componente no declarada.
    pub fn reduced_index(&self, node: NodeId, dof: Dof) -> EResult<Option<DofIndex>> {
        self.constraints
            .reduced_index(self.map.global_index(node, dof)?)
    }
    /// Rigidez global CSR en SI, sin eliminar GDL apoyados.
    pub fn stiffness(&self) -> &SparseMatrix {
        &self.k
    }
    /// Masa global CSR (kg, kg·m, kg·m²); incluye masas nodales.
    pub fn global_mass(&self, kind: MassKind) -> &SparseMatrix {
        match kind {
            MassKind::Lumped => &self.lumped,
            MassKind::Consistent => &self.consistent,
        }
    }
    /// Rigidez geométrica global CSR constante en el estado de referencia.
    pub fn global_geometric(&self) -> &SparseMatrix {
        &self.kg
    }
    /// Último desplazamiento reducido aceptado, en m/rad.
    pub fn accepted_displacement(&self) -> &[Real] {
        &self.accepted
    }
    /// Último desplazamiento reducido evaluado con éxito, en m/rad.
    pub fn trial_displacement(&self) -> &[Real] {
        &self.trial
    }

    /// Fuerzas globales físicas factor F del caso (N/N·m), incluidos apoyos.
    /// ENG-E-0003 por caso ausente; ENG-E-0008 por factor/resultado no finito.
    pub fn global_force(&self, case: &CaseRef, factor: Real) -> EResult<Vec<Real>> {
        finite(&[factor])?;
        let force = self.forces.get(&case.0).ok_or_else(|| missing_case(case))?;
        let result: Vec<_> = force.iter().map(|v| v * factor).collect();
        finite(&result)?;
        Ok(result)
    }

    /// RHS para solve directo: factor F_f − K_fp u_p (N/N·m).
    /// u_p no se escala por factor; los errores conservan el sistema.
    pub fn reduced_rhs(&self, case: &CaseRef, factor: Real) -> EResult<Vec<Real>> {
        let force = self.global_force(case, factor)?;
        let shift = multiply(
            &self.k,
            &self.constraints.expand(&vec![0.0; self.n_free_dofs()])?,
        )?;
        let result: Vec<_> = self
            .constraints
            .free_dofs()
            .iter()
            .map(|i| force[i.0 as usize] - shift[i.0 as usize])
            .collect();
        finite(&result)?;
        Ok(result)
    }

    /// Resuelve K_ff u_f = RHS mediante el backend, sin aceptar el resultado.
    ///
    /// Conveniencia para problemas lineales; no es un método analysis/1.
    /// Traduce los índices reducidos de ENG-E-0006 a índice global, entidad nodo
    /// y etiqueta GDL en message. El pivote identifica parte del mecanismo,
    /// no todos sus movimientos. ENG-E-0012 si el backend informa índice inválido.
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

    /// Recupera R = K_pf u_f + K_pp u_p − factor F_p sobre los nodos apoyados.
    /// Orden global estable; unidades N/N·m. No acepta ni modifica pruebas.
    /// Incluye las cargas aplicadas directamente sobre apoyos.
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

    fn evaluate(&self, u: &[Real]) -> EResult<Vec<Real>> {
        let global = self.constraints.expand(u)?;
        let internal = multiply(&self.k, &global)?;
        Ok(self
            .constraints
            .free_dofs()
            .iter()
            .map(|i| internal[i.0 as usize])
            .collect())
    }

    fn translate_solver_error(&self, mut e: EngineError) -> EngineError {
        if e.code != "ENG-E-0006" {
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
                let (node, dof) = self.map.label(global).expect("Mapa inmutable validado");
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
        self.evaluate(u)?;
        *out = self.kg_free.clone();
        self.trial = u.to_vec();
        Ok(())
    }
    fn external_force(&self, case: &CaseRef, factor: Real, out: &mut [Real]) -> EResult<()> {
        size(out.len(), self.n_free_dofs())?;
        let force = self.global_force(case, factor)?;
        let reduced: Vec<_> = self
            .constraints
            .free_dofs()
            .iter()
            .map(|i| force[i.0 as usize])
            .collect();
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
    fn dof_label(&self, d: DofIndex) -> (NodeId, Dof) {
        let global = self.constraints.free_dofs()[d.0 as usize];
        self.map
            .label(global)
            .expect("Índice reducido válido requerido por SystemView")
    }
}
