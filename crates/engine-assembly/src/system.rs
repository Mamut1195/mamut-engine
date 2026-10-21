//! Sistema lineal poseído; ensamblaje global, reducción y estado atómico.
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

/// Evaluador determinista de Kg elemental global a partir de u_e global (m/rad).
/// No requiere Send/Sync; los errores se propagan sin modificar salida ni prueba.
pub type GeometricEval = dyn Fn(&[Real]) -> EResult<DMat>;

/// Productor de rigidez geométrica dependiente del estado, para pandeo/P-Delta.
///
/// u_e contiene desplazamientos totales globales, incluidos los prescritos, en
/// el orden de location. El productor calcula el esfuerzo N de u_e y devuelve
/// Kg global cuadrada, finita y exactamente simétrica en ese mismo orden.
/// Unidades: N/m, N o N·m según traslaciones/giros de fila y columna.
/// La clausura debe ser determinista y no modificar historia del elemento.
pub struct GeometricProducer {
    /// Índices globales distintos, en el orden elemental (no índices reducidos).
    pub location: Vec<DofIndex>,
    /// Evaluación elemental global, sin requisitos Send/Sync.
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

/// Sistema de K/M constantes, Kg opcionalmente producida y apoyos constantes.
///
/// Solo evalúa los productores Kg registrados; no convierte unidades. Sus
/// operaciones SystemView usan vectores reducidos totales en m/rad, escriben out completamente y nunca
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
    nodal_forces: BTreeMap<String, Vec<Real>>,
    accepted: Vec<Real>,
    trial: Vec<Real>,
    kg_reference: Vec<Real>,
    /// ¿Queda Kg estática sin productor? Independiente de que Kg(ref) sea 0.
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
                // Acumular una sola mitad y reflejar el resultado evita que el
                // orden de sumas produzca asimetría de redondeo en Tᵀ K T.
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
        backend: B,
        model: &Model,
        elements: &[ElementContribution],
        prescribed: &[PrescribedDof],
    ) -> EResult<Self> {
        Self::from_model_with_diaphragms(backend, model, elements, prescribed, &[])
    }

    /// Ensambla y reduce por diafragmas XY D3, sin modificar Model.
    ///
    /// Conserva from_model cuando la lista está vacía. Usa u = T_f q_f + u_0,
    /// K_red = T_fᵀ K T_f y F_red = T_fᵀ F; el RHS resta T_fᵀ K u_0.
    /// M y Kg, incluidos productores, reciben la misma transformación.
    /// Propaga diagnósticos de from_model y Constraints::with_diaphragms.
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
        // Unión estructural de K, M y Kg, incluidos sus ceros explícitos.
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

    /// Mapa global; combine global_index con constraints().reduced_index.
    pub fn dof_map(&self) -> &DofMap {
        &self.map
    }
    /// Transformación y partición independiente libre/prescrita constantes.
    pub fn constraints(&self) -> &Constraints {
        &self.constraints
    }
    /// Busca nodo/GDL → índice reducido; None significa prescrito o dependiente.
    /// Para esclavos consulte constraints().free_transformation_row().
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

    /// Indica si M_ff contiene filas/columnas exactamente nulas tras apoyos y MPC.
    /// No emplea tolerancias ni clasifica como nula una masa pequeña positiva.
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

    /// Modos inferiores de K_ff y M_ff, con el backend propio del sistema.
    /// Ambas matrices proceden del ensamblaje global y la partición de GDL
    /// libres. Condensa filas de masa nulas con Guyan y recupera los modos completos.
    /// No modifica estados; sin GDL sin masa conserva el solver original.
    /// Propaga los diagnósticos de `engine_linalg::lowest_modes`, traduciendo
    /// índices reducidos de ENG-E-0006/0014 a nodo/GDL global cuando se proporcionan.
    /// Un índice diagnóstico inexistente devuelve ENG-E-0012.
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
    /// Instantánea global CSR estática recibida en from_model.
    /// No incluye sustituciones ni evaluaciones de productores registrados;
    /// consulte [`SystemView::geometric`] para Kg(u) reducida vigente.
    pub fn global_geometric(&self) -> &SparseMatrix {
        &self.kg
    }

    /// Evalúa Kg global vigente en u reducido total, sin alterar ningún estado.
    /// Incluye productores y prescritos; aplica la misma regla de referencia y
    /// validación que [`SystemView::geometric`]. Permite formar Kg_fp u_p y
    /// reacciones de segundo orden sin perder los bloques restringidos.
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

    /// K_ff constante; no evalúa ni modifica estados de prueba.
    pub fn reduced_stiffness(&self) -> &SymMatrix {
        &self.k_free
    }

    /// Factoriza una rigidez reducida del mismo orden mediante el backend propio.
    /// Traduce mecanismos a nodo/GDL como [`Self::solve`]; ENG-E-0010 por orden
    /// incorrecto. No regulariza ni acepta estados; conserva la inercia del factor.
    pub fn factorize_reduced(&mut self, matrix: &SymMatrix) -> EResult<Box<dyn SymFactorization>> {
        size(matrix.order(), self.n_free_dofs())?;
        self.backend
            .factorize(matrix)
            .map_err(|e| self.translate_solver_error(e))
    }
    /// Declara el estado reducido u (m/rad) en el que se evaluó la Kg estática.
    ///
    /// Por defecto es u = 0. Si queda algún aporte Kg sin productor,
    /// [`SystemView::geometric`] solo acepta esa referencia exacta, aunque
    /// Kg(ref) sea cero; otro u da ENG-E-0013. Los productores no dependen de
    /// esta restricción. Sin ningún aporte, Kg=0 para cualquier u válido.
    /// ENG-E-0010 por tamaño; ENG-E-0008 por valores no finitos.
    pub fn set_geometric_reference(&mut self, u: &[Real]) -> EResult<()> {
        size(u.len(), self.n_free_dofs())?;
        finite(u)?;
        self.kg_reference = u.to_vec();
        Ok(())
    }

    /// Sustituye el registro completo de productores Kg(u), de forma atómica.
    ///
    /// La identidad es location con igualdad exacta, incluido su orden. Un
    /// productor sustituye la Kg estática del único aporte con esa location,
    /// incluso si era None. Sin coincidencia añade un bloque independiente.
    /// Dos productores con la misma location o una coincidencia con varios
    /// aportes son ambiguos y dan ENG-E-0002. No se emparejan permutaciones.
    /// Un registro vacío restaura todos los aportes estáticos originales.
    ///
    /// Valida ubicaciones y evalúa en la referencia expandida con u_p para
    /// validar tamaños, finitud y simetría exacta por assemble_matrix. Revalida
    /// las matrices en cada geometric(u). Propaga errores del evaluador;
    /// ENG-E-0010 por ubicación/tamaño/asimetría, ENG-E-0008 por no finitos.
    /// Amplía el patrón con todos los pares de GDL libres de cada productor,
    /// incluidos ceros. Conserva salida, registro y prueba ante cualquier error.
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
    /// Etiqueta nodo/GDL de un índice reducido, sin pánico: ENG-E-0010 si d ≥ n.
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
    /// Último desplazamiento reducido aceptado, en m/rad.
    pub fn accepted_displacement(&self) -> &[Real] {
        &self.accepted
    }
    /// Último desplazamiento reducido evaluado con éxito, en m/rad.
    pub fn trial_displacement(&self) -> &[Real] {
        &self.trial
    }

    /// Fuerza interna lineal global K u, incluidos los prescritos constantes.
    /// No modifica desplazamientos aceptados ni de prueba.
    pub fn global_internal_force(&self, free: &[Real]) -> EResult<Vec<Real>> {
        multiply(&self.k, &self.constraints.expand(free)?)
    }

    /// Suma tangentes elementales variables a K y extrae la partición ff.
    /// Los bloques son aportes adicionales: no deben duplicarse en from_model.
    /// Valida ubicación, tamaño, finitud y simetría como assemble_matrix.
    /// Los prescritos se incorporan a la fuerza interna evaluando u TOTAL;
    /// su corrección es cero y no se añade ningún término K_fp Δu_p al RHS.
    pub fn tangent_with_blocks(&mut self, blocks: &[(&[DofIndex], &DMat)]) -> EResult<SymMatrix> {
        let extra = assemble_matrix(&mut self.backend, self.map.len(), blocks)?;
        let mut entries = triplets(&self.k);
        entries.extend(triplets(&extra));
        let global = self
            .backend
            .assemble(self.map.len(), self.map.len(), &entries)?;
        reduced(&mut self.backend, &global, &self.constraints)
    }

    /// Resuelve una corrección reducida con UNA factorización de la tangente.
    /// Sin regularización, aceptación ni modificación de pruebas. Traduce
    /// ENG-E-0006 a nodo/GDL igual que solve; rechaza no finitos y tamaños.
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

    /// Fuerzas globales físicas factor F del caso (N/N·m), incluidos apoyos.
    /// ENG-E-0003 por caso ausente; ENG-E-0008 por factor/resultado no finito.
    pub fn global_force(&self, case: &CaseRef, factor: Real) -> EResult<Vec<Real>> {
        finite(&[factor])?;
        let force = self.forces.get(&case.0).ok_or_else(|| missing_case(case))?;
        let result: Vec<_> = force.iter().map(|v| v * factor).collect();
        finite(&result)?;
        Ok(result)
    }

    /// Cargas nodales explícitas del modelo, sin cargas equivalentes de tramo.
    /// Mismo orden global y unidades N/N·m que [`Self::global_force`]. Se
    /// conservan al ensamblar para verificar el equilibrio del postproceso.
    /// ENG-E-0003 por caso ausente; ENG-E-0008 por factor/resultado no finito.
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

    /// RHS directo: T_fᵀ(factor F − K u_0) (N/N·m).
    /// u_p no se escala por factor; los errores conservan el sistema.
    pub fn reduced_rhs(&self, case: &CaseRef, factor: Real) -> EResult<Vec<Real>> {
        let force = self.global_force(case, factor)?;
        let shift = multiply(
            &self.k,
            &self.constraints.expand(&vec![0.0; self.n_free_dofs()])?,
        )?;
        let residual: Vec<_> = force.iter().zip(shift).map(|(f, k)| f - k).collect();
        self.constraints.project_force(&residual)
    }

    /// Resuelve (T_fᵀ K T_f) q_f = RHS mediante el backend, sin aceptar el resultado.
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

    /// Resuelve varios casos con UNA factorización de K_red = T_fᵀ K T_f.
    ///
    /// Conserva el orden de entrada y no acepta ni modifica pruebas. Cada RHS
    /// es `T_fᵀ(factor F − K u_0)`: los prescritos no se escalan. Valida todos
    /// los casos antes de factorizar. ENG-E-0010 por lista vacía o nombres
    /// duplicados; ENG-E-0003 por caso ausente; ENG-E-0008 por no finitos.
    /// Traduce errores del backend igual que [`Self::solve`], incluidos
    /// mecanismos ENG-E-0006 con nodo/GDL y diagnósticos inválidos ENG-E-0012.
    /// Devuelve desplazamientos independientes libres, igual que [`Self::solve`].
    /// [`Constraints::expand`] recupera los dependientes y prescritos mediante
    /// `u = T_f q_f + u_0`. Sin diafragmas equivale a `factor F_f − K_fp u_p`.
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

    /// Recupera R = (Tᵀ(Ku − factor F))_p sobre todo independiente prescrito:
    /// los restringidos por apoyos y los de desplazamiento impuesto (asentamiento).
    /// Incluye la fuerza transferida por esclavos a un maestro prescrito.
    /// Sin diafragmas equivale a K_pf u_f + K_pp u_p − factor F_p.
    /// No existe reacción de apoyo en un GDL libre o dependiente. Orden global estable; unidades
    /// N/N·m. No acepta ni modifica pruebas. Incluye las cargas aplicadas
    /// directamente sobre GDL prescritos.
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

    /// Recupera las acciones de cada diafragma sobre esclavos y maestro.
    /// Signos y transporte de momentos documentados en DiaphragmForces.
    /// Propaga errores de expansión, fuerzas y productos (0003/0008/0010).
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
    /// Precondición del contrato: d < n. El trait no puede devolver error; use
    /// [`LinearSystem::try_dof_label`] para entradas no confiables. Si se viola
    /// la precondición devuelve el centinela (NodeId(0), Ux), sin pánico.
    fn dof_label(&self, d: DofIndex) -> (NodeId, Dof) {
        match self.try_dof_label(d) {
            Ok(label) => label,
            // El trait no tiene canal de error. Centinela para una llamada
            // fuera de contrato; try_dof_label devuelve ENG-E-0010.
            Err(_) => (NodeId(0), Dof::Ux),
        }
    }
}
