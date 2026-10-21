# engine-assembly · ENG-ASM-001

Linear assembler of global element matrices in SI. Depends only on
internal crates. Theory and analytical fixtures are documented in
[the work log](../../provenance/ENG-ASM-001.md).

`DofMap::new(&model.nodes)` numbers nodes by increasing id and components by
`Dim::dofs()`: Ux, Uy, Rz in D2 and six components in D3. Supports subsets
of declared DOFs, for example Ux for an axial chain. Global indices
do not change when supports are eliminated; `LinearSystem::reduced_index` returns the
reduced index or `None` for a prescribed or dependent DOF. `coordinates()` queries
original positions from the same geometry map used by diaphragms.

`ElementContribution` takes `location`, K, two optional types of M, optional
Kg, and load vectors per case. Matrices and vectors are already in global
axes and their order follows `location`. Geometric connectivity is not interpreted,
no element is created, and no local matrix is transformed. A future
adapter may obtain these data from `engine_contracts::Element`.

`LinearSystem::from_model(backend, model, elements, prescribed)` sums element
contributions, then nodal loads and masses. Only uses domain nodes, supports, cases, and
masses; complete topological validation belongs to the preceding
layer. Does not interpret combinations. Each case must be declared even if
it only contains equivalent element forces. `None` in M/Kg contributes zero;
nodal masses are added to both mass types without converting a consistent
matrix to a lumped one.

Support constraints are zero. A `PrescribedDof` may replace that
zero or constrain another DOF; duplicates within either list are not allowed. Imposed
values are constant and do not depend on the case factor. The system
rejects a model without free DOFs with ENG-E-0010.

`from_model_with_diaphragms` adds XY master-slave constraints in D3:
`u = T_f q_f + u_0`. Slaves satisfy `Ux_s = Ux_m − ry Rz_m`,
`Uy_s = Uy_m + rx Rz_m` and `Rz_s = Rz_m`; Uz/Rx/Ry remain independent.
K/M/Kg reduction uses `T_fᵀ A T_f`.

The `SystemView` contract returns physical internal force `T_fᵀ K u`, including
prescribed values, and physical external force `factor T_fᵀ F`. To solve
`K_red q_f = T_fᵀ(factor F − K u_0)` directly, use `reduced_rhs` or `solve`.
`solve_cases` preserves batch order and reuses a single factorization;
it returns the same free independent DOFs as `solve` case by case.
`Constraints::expand` recovers dependent and prescribed DOFs without scaling the latter.
The `reactions` recovery returns `R_p = (Tᵀ(Ku − factor F))_p`, including
force transferred by slaves to a prescribed master. Without diaphragms,
it yields `K_ff u_f = factor F_f − K_fp u_p` and
`R_p = K_pf u_f + K_pp u_p − factor F_p`. Includes forces on supports.

K and M are constant. Without registered producers, Kg is a snapshot of the
state declared with `set_geometric_reference` (zero reduced u by default).
If any static Kg contribution exists, even zero, `geometric(u)` rejects
another state with ENG-E-0013. Without contributions, it returns zero for any finite u.

`GeometricProducer { location, eval }` allows evaluating P-Delta-type Kg with
axial forces N from state u. `set_geometric_producers` replaces the entire registry:
each producer replaces the static Kg of the single `ElementContribution` whose
`location` matches exactly, including order, and if it does not match adds an
independent block. Repeated locations among producers or matches
with several contributions yield ENG-E-0002 for ambiguity. A static Kg contribution without
a producer remains subject to its reference; emptying the registry restores the
original snapshots. `global_geometric()` preserves the original snapshot;
the current reduced matrix is obtained with `geometric(u, out)`.

The registry validates locations and evaluates at the expanded reference. Each query
expands reduced u with `Constraints::expand`, including prescribed
displacements, extracts u_e in element order, evaluates, assembles through
`assemble_matrix` with exact symmetry, and reduces to free DOFs. Overwrites `out`.
Propagates producer errors; ENG-E-0010 for size/asymmetry/indices and
ENG-E-0008 for nonfinite data or sums. The pattern includes all free
pairs of registered locations, even if Kg is zero at the reference.
Closures are deterministic and do not require Send/Sync. Assembly follows
the order of original contributions, then that of independent producers;
evaluations follow registry order.

El adaptador `engine_elements_truss::geometric_producer` captura elemento y
contexto para calcular N=(EA/L)d·Δu y Kg=(N/L)(I−ddᵀ), con geometría fija.
La dependencia interna de assembly, ya usada por truss en pruebas, pasa a ser
de producción; no se incorpora ningún crate adicional.
Las [derivaciones y pruebas P-Delta](../../provenance/ENG-ASM-001-KG.md) explican
el alcance incremental de K+Kg y el multiplicador crítico del arco de dos barras.
Esto proporciona Kg al método de análisis; `tangent` e `internal_force` siguen
siendo las operaciones lineales K y Ku. Cada evaluación
exitosa de desplazamiento actualiza la prueba; `accept(u)` reevalúa exactamente
u y lo acepta, y `rollback` recupera el último aceptado. Los errores no dejan
salidas ni estados parcialmente modificados.

Global matrices use CSR and reduced matrices use dense `SymMatrix`, under the
ENG-LA-007 API. `solve` delegates factorization and solution to the supplied backend and
maps singular pivots to node/DOF; it does not implement its own solver. The reduced
pattern joins K, both M types, and Kg, and preserves structural zeros. CSR contraction
with u traverses sorted rows and columns, without expanding the global matrix.
Sums in contribution order, `BTreeMap`/vectors, without threads; bitwise reproducibility
across architectures PENDING ENG-CORE-010.

Translations: m; rotations: rad; forces: N; moments: N·m. K and Kg: N/m, N, or
N·m according to components; M: kg, kg·m, or kg·m². D2 uses the horizontal X-Y plane
according to `docs/CONVENCION_EJES.md`; Y is not interpreted as vertical. Norms
with forces/moments or translations/rotations require scales from the analysis method.

Fixtures are compiled from `tests/fixtures/mod.rs` and contain `source`,
citation, and hand-calculated values. Tests use no files, clock, or threads
and also compile for wasm. Execution in a wasm runtime remains PENDING.
ENG-ASM-002 retains master-slave constraints; ENG-ELM-* retains element physics,
transformation, evaluation, and history.
