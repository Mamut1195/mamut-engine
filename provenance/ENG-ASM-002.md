# Bitácora clean-room · ENG-ASM-002

## Integración con main (2026-10-13)

- Ticket / PR: ENG-ASM-002, rama `j2/eng-asm-002`, PR #60; resolución del merge
  local en curso con `origin/main`, incluida la ampliación `solve_cases` de #52.
- Autor: Codex (agente basado en GPT-6), rol implementador. Revisión independiente
  de esta integración: PENDIENTE; no se atribuye a revisiones anteriores.
- Fuentes: las versiones locales de ambas ramas en las etapas del índice del
  merge; contratos y código propio de `engine-assembly`, `engine-analysis`,
  `engine-model`, `engine-core` y `engine-linalg`; pruebas y bitácoras propias
  `ENG-ASM-001.md` y `ENG-ANL-001.md`; `docs/CLEAN_ROOM.md`. Sin red, consultas
  externas, oráculos, código de terceros ni dependencias nuevas.
- Declaración de esta sesión: no consulté código fuente de OpenSees, Stabileo
  ni los motores o rutas prohibidos por `docs/CLEAN_ROOM.md`.

### Decisiones de integración

`DofMap` conserva una única representación de geometría:
`BTreeMap<NodeId, (Dim, [Real; 3])>`. `geometry()` obtiene dimensión y posición
para validar diafragmas; `coordinates()` obtiene la posición de esa misma
entrada para los brazos de fuerzas y la recuperación de `engine-analysis`.
Se conservan las coordenadas originales y ENG-E-0003 con entidad nodo ausente.
No se añade una copia de las posiciones.

`solve_cases` valida el lote completo antes de factorizar, conserva el orden
y factoriza una sola vez `K_red = T_fᵀ K T_f`. Cada caso usa exactamente el
`reduced_rhs` de `solve`: `T_fᵀ(factor F − K u_0)`. Ambos devuelven independientes
libres; `Constraints::expand` recupera `u = T_f q_f + u_0`, incluidos los
esclavos y los maestros prescritos. El factor escala solo F, nunca u_0.
Sin diafragmas se conserva la partición de main:
`K_ff u_f = factor F_f − K_fp u_p`.

Se mantiene `R_p = (Tᵀ(Ku − factor F))_p`: la reacción de un maestro prescrito
incluye las fuerzas de sus esclavos. Sin restricciones maestro-esclavo se
reduce a `K_pf u_f + K_pp u_p − factor F_p`. Un GDL dependiente no se informa
como reacción de apoyo.

Todos los métodos de resolución usan la misma traducción de ENG-E-0006 a
nodo, GDL y numeración global. Un diagnóstico incoherente del backend da
ENG-E-0012, incluido «El solver informó un GDL global inexistente.» si falla
la etiqueta global. Una consulta pública inválida a `DofMap::label` mantiene
ENG-E-0010; no se confunde con un incumplimiento interno del solver.

`StaticLinear` sigue usando `solve_cases` y expandiendo los resultados. Su
residual libre ahora es `T_fᵀ(Ku − factor F)`; la escala relativa usa
`max(1, ‖T_fᵀ Ku‖∞, ‖T_fᵀ factor F‖∞)`. Seleccionar solo las filas físicas de
los maestros omitiría las acciones transferidas por los esclavos. El equilibrio
global conserva todas las fuerzas físicas, sus brazos y las reacciones
proyectadas de los apoyos. Los métodos no modifican estados aceptados o de prueba.

### Derivación y pruebas analíticas propias

Fixture compartido `crates/engine-assembly/tests/fixtures/rigid_cases.rs`:
dos esclavos en (1,1,3) y (3,1,3) m, maestro en (0,0,3) m, y dos resortes
Ux/Uy de k=100 N/m por esclavo. Cada anclaje fijo coincide con su esclavo;
todos los resortes están en la misma planta, por lo que no se introduce un
par fuera del plano sin apoyo que lo equilibre. Cada par aporta la energía
`½k[(Ux − yθ)² + (Uy + xθ)²]`, con θ=Rz y los brazos respecto al maestro.
Por suma y derivación respecto a (Ux,Uy,θ):

```text
K_ind = [ 200     0   -200
            0   200    400
         -200   400   1200 ]
```

La carga A=(10,20,5) en el primer esclavo se transporta a (10,20,15) en el
maestro; B=(-20,10,-5) en el segundo se transporta a (-20,10,45). Las fuerzas
están en N y los momentos en N·m. Con Ux=a=0.03 m prescrito:

```text
θ  = [factor (Mz − 2 Fy) + 200 a] / 400
Uy = factor Fy / 200 − 2 θ
R_maestro,Ux = 200 a − 200 θ − factor Fx
```

Con maestro libre: `θ = factor (Mz + Fx − 2 Fy) / 200`,
`Ux = factor Fx / 200 + θ` y la misma expresión de Uy. Los esclavos cumplen
`Ux_s = Ux − y θ`, `Uy_s = Uy + x θ`, `Rz_s = θ`; los anclajes tienen
desplazamiento nulo y reacciones `−100 Ux_s`, `−100 Uy_s`.

Los lotes A×2, B×(−1) y un caso vacío ×0 se comparan con estas expresiones,
con `solve` caso a caso (igualdad exacta de los vectores reducidos), y con
las restricciones de cada esclavo. El contador confirma una factorización
por lote y una resolución por caso. Se prueban maestro libre y prescrito,
orden de resultados y conservación de estados aceptados y de prueba distintos.
Tolerancia analítica: `1e-12 max(1, |esperado|)`.

En `engine-analysis/tests/diaphragm.rs` se comprueban además recuperación
global, reacciones, fuerzas y momentos respecto a (4,-2,1) m y residual
proyectado nulo. Un solver de prueba devuelve la mitad de la solución:
las filas físicas de los maestros tienen rigidez cero, pero el residual
proyectado detecta el error (normas 20 y 22.5, relativos 0.5).

Se verifica que un caso tardío inválido (duplicado, ausente, NaN, infinito
o desbordamiento al proyectar fuerzas finitas con brazo x=3) impide cualquier
factorización/resolución del lote y conserva ambos estados. Las pruebas
anteriores de main y de la rama se conservan, incluidos los errores del solver.
Las fórmulas son derivaciones propias; no se usaron salidas de otros motores
ni se atribuyen a un libro no consultado.

### Verificación local de la integración

Comandos ejecutados con `/home/box/.cargo/bin` en PATH y
`CARGO_NET_OFFLINE=true`, sin acceso a red:

- `cargo fmt` y comprobación final `cargo fmt --check`: satisfactorios.
- `cargo clippy --all-targets -- -D warnings`: satisfactorio.
- `cargo test --workspace`: satisfactorio; incluye cuatro pruebas nuevas de
  integración (dos de ensamblaje y dos de análisis), más las pruebas existentes
  de ambas ramas y los doctests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: satisfactorio.
- `cargo check --workspace --target wasm32-unknown-unknown`: satisfactorio.
  Se verificó compilación; no se afirma ejecución en un runtime wasm.
- `git diff --check`: satisfactorio; archivos de trabajo sin marcadores de merge.
- Bibliotecas `engine-assembly` y `engine-analysis` sin `panic!`, `unwrap` ni
  `expect`; sin dependencias nuevas.

Sin `git add`, commit ni push. El índice conserva las etapas del merge en
`dofmap.rs` y `system.rs` (estado `UU` hasta que el usuario añada los archivos);
los contenidos de trabajo están resueltos. El merge se mantiene abierto.
