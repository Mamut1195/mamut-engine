# Bitácora clean-room · ENG-CORE-009

- Ticket / PR: MAM-22 · ENG-CORE-009 · formato fixture/1 y runner de benchmarks.
- Autor(es): Codex gpt-6.1-sol high, 2026-10-17 (America/Guayaquil).
- Rol: especificador e implementador.
- Contrato implementado: fixture/1, informe fixture-report/1 y Target enchufable
  sobre engine-input/1 → results/1; sin modificar métodos numéricos del motor.
- Fuentes permitidas usadas: docs/CLEAN_ROOM.md §§4, 7, 8 y 10;
  contratos, tests y ejemplos locales de engine-api; patrones locales de ABI
  en engine-api/examples/determinism_wasm.rs y scripts/check-determinism.mjs.
  Formato propio y derivaciones analíticas de esta nota, sin fuentes externas.
- Cita de la nota propia: Codex. *Casos estructurales cerrados, derivación propia
  para ENG-CORE-009 (MAM-22)*, 17 de octubre de 2026, casos identificados a
  continuación y en crates/engine-validation/fixtures/*.json. Autor de sesión:
  Codex gpt-6.1-sol high.
- Derivaciones propias: esta nota y source.derivation de cada fixture.
- Código de terceros copiado: ninguno. Se usan dependencias ya presentes:
  engine-api, serde 1.0.229 y serde_json 1.0.154. Cargo.lock cambia únicamente
  las tres aristas de engine-validation; no incorpora paquetes o versiones.
- Validación: analítica propia; ningún resultado calculado por otro motor.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código o resultados de CalculiX o LAPACK,
  informes de oráculos, contenido de repos hermanos ni recursos externos. Trabajo sin red.
- Revisor clean-room: **PENDIENTE (J4)**.

## Derivaciones, unidades y signos

Magnitudes SI artificiales para comprobar el álgebra del modelo lineal elástico;
no representan un diseño físico ni una validación de grandes desplazamientos.
Datos originales, sin copiar tablas, figuras o listados publicados. Los números
JSON se obtienen evaluando estas expresiones, nunca copiando la salida del motor.

1. **axial-bar**. L=4 m, E=200 Pa, A=2 m², P=12 N en +X. Por equilibrio
   N=P. Como ε=u′ y σ=Eε=N/A, integrar u′=P/(EA), u(0)=0, da
   u(L)=PL/(EA)=0,12 m. El apoyo reacciona −12 N; Uy=0 por las guías.
2. **axial-series**. E=200 Pa, P=12 N, L₁=2 m, A₁=2 m², L₂=3 m,
   A₂=1 m². El nodo intermedio sin carga impone N₁=N₂=P. Cada tramo
   tiene kᵢ=EAᵢ/Lᵢ. Compatibilidad: u₂=P/k₁=0,06 m;
   u₃=P(1/k₁+1/k₂)=0,24 m. Reacción −12 N y N₂=12 N. Se realiza
   la serie de resortes con barras, soportadas por la API actual.
3. **cantilever-point**. L=4 m, E=200 Pa, I=3 m⁴, P=12 N hacia −Y.
   Momento de empotramiento PL y cortante P por equilibrio. La curvatura
   v″=−P(L−x)/(EI), integrada con v(0)=v′(0)=0, da
   v′(L)=−PL²/(2EI)=−0,16 rad y v(L)=−PL³/(3EI)=−32/75 m.
   Ry=12 N, Mz=48 N·m y desplazamiento axial nulo.
4. **simply-supported-uniform**. L=4 m, E=200 Pa, I=3 m⁴, w=6 N/m
   hacia −Y. Equilibrio y simetría dan R₁=R₃=wL/2=12 N.
   M(x)=wx(L−x)/2; M′=0 en L/2, Mmáx=wL²/8=12 N·m.
   Integrar EI v″=M con v(0)=v(L)=0 produce
   v(x)=−wx(L³−2Lx²+x³)/(24EI), y v(L/2)=−5wL⁴/(384EI)=−1/30 m;
   v′(L/2)=0 por simetría. Dos elementos de longitud 2 m con carga uniforme
   consistente y nodo central permiten contrastar la flecha nodal exacta;
   se contrastan también ambos máximos de momento recuperados.
5. **spatial-cantilever**. L=4 m, E=200 Pa, Iy=3 m⁴, Iz=5 m⁴, P=12 N
   hacia −Z. Integrar EI_y z″=−P(L−x) con empotramiento produce
   z(L)=−PL³/(3EI_y)=−32/75 m. r×F=+PL e_y exige My=−48 N·m en
   el apoyo, con Rz=12 N. Desplazamiento Y nulo por desacoplamiento.

## Tolerancias y procedencia

Para cada valor, |actual−esperado| ≤ max(absolute, relative·|esperado|).
Defaults relative=absolute=1e-10 para linearStatic; 1e-7 para métodos
iterativos (pDelta, modal, responseSpectrum). El piso absoluto está en las
unidades SI de la magnitud y protege ceros; no se divide por el valor esperado.
Overrides finitos requieren justificación propia explícita. Los cinco casos
cerrados usan 1e-10, sin calibrar tolerancias con resultados externos.

source es obligatorio y sus campos no pueden estar en blanco. El contrato
admite nota analítica, libro con autor/título/edición/página y ejemplo publicado
con cita y localizador. Se inspeccionan source y provenance recursivamente,
incluidas claves y capitalización, para rechazar menciones de OpenSees,
Stabileo y CalculiX antes de invocar targets. Las menciones en código y tests
negativos expresan únicamente esa regla; no contienen resultados externos.

## Diseño y alcance

La biblioteca compara contra el oráculo declarado; los targets solamente
ejecutan la entrada. Target nativo en proceso vía engine-api y adaptador
Node/WebAssembly sin npm, wasm-bindgen, WASI, imports o red. La ABI transporta
bytes mediante funciones escalares, almacenamiento seguro en Mutex y límites
de entrada de engine-api; sin unsafe, punteros o dependencias nuevas externas.
Se añade una excepción **exacta** para fixture_wasm.rs en la guardia de fuentes,
limitada a no_mangle y funciones extern C, igual que los ejemplos existentes.
No relaja permisos para unsafe ni afecta al cálculo de los hashes del motor.

Se conservan todos los modelos, métodos y resultados existentes. Sin cambios
en Cargo.toml raíz, docs/DETERMINISMO.md, clean-room/**, docs/CLEAN_ROOM.md,
.github/workflows, CODEOWNERS o vendor/. El CLI mamut-engine no es dependencia:
su adaptador se añadirá después de la fusión de ENG-SFC-002. El runner mide
exactitud; no introduce mediciones de rendimiento ni tiempos no deterministas.

## Evidencia local

Ejecución offline el 2026-10-17: rustc 1.99.0 (b940084d7 2026-09-28),
Node v22.19.0, CARGO_TARGET_DIR=/workspace/.cargo-target/me-fix9 y flags Cargo
habituales. Las comprobaciones terminaron con código 0:

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK, sin warnings. |
| `cargo test --workspace --offline` | **639 passed / 0 failed / 0 ignored**, incluidos los 9 tests nuevos. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK. |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK. |
| `bash clean-room/check.sh origin/main` | OK, incluidos los archivos nuevos mediante un índice temporal bajo CARGO_TARGET_DIR; índice real intacto. |

`cargo test -p engine-validation --test fixtures --offline`: 9 passed / 0 failed.
Se cubren el inventario completo de fixtures JSON, source ausente/vacío,
variantes incompletas, veto recursivo y previo a targets, defaults, piso absoluto,
override justificado, discrepancias/rutas ausentes/status incorrecto, formatos
inválidos, continuación tras fallos y repetibilidad/orden de informes.

Se compiló fixture_wasm como cdylib wasm32-unknown-unknown y se ejecutó el runner
con Node: **5 fixtures × 2 targets = 10 filas aprobadas**, con 23 comprobaciones
por target (46 en total). Se ejercitaron las salidas de texto y JSON; dos corridas
completas de JSON resultaron idénticas byte a byte (`cmp`). Node verificó cero
imports y los errores de rango de la ABI. Los tests Cargo no requieren Node.

Se ejecutaron los ejemplos determinism_native y determinism_wasm y
scripts/check-determinism.mjs en **debug y release**, con diez corridas por
caso/target/perfil: **16 hashes nativo=wasm**, salidas nativas debug=release por
`cmp` y los 16 hashes idénticos a la tabla actual de docs/DETERMINISMO.md.
MAMUT_ENGINE_GIT_COMMIT sin definir, como exige esa referencia reproducible.

Logs e informes retenidos bajo CARGO_TARGET_DIR:
`eng-core-009-{clippy,tests,doc,wasm-check}.log`,
`fixture-report-009-{a,b}.json`, `fixture-report-009.txt`,
`eng-core-009-native-{debug,release}.txt` y
`eng-core-009-determinism-{debug,release}.log`.

## Incidencia de alcance y pendientes

La búsqueda inicial de instrucciones ejecutó `find .. -name AGENTS.md -print`
y enumeró nombres de archivos en directorios hermanos, fuera del alcance de
directorio indicado. No se abrió ninguno de esos archivos ni código, datos,
informes o resultados externos. Se corrigió el alcance de las búsquedas
posteriores al worktree; no hubo acceso de red ni lectura de fuentes prohibidas.

Pendientes externos: revisión clean-room **J4** y futuro adaptador CLI después
de la fusión de **ENG-SFC-002**. No quedan comprobaciones técnicas pendientes
de este entregable.
