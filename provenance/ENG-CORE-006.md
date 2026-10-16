# Bitácora clean-room · ENG-CORE-006

- Ticket: MAM-20 · ENG-CORE-006 «Registro de métodos y capacidades (sobre mamut-workflows)».
- Rama: j2/eng-core-006, apilada sobre j2/vnd-mamut-workflows / PR #100.
- Autor: agente Codex, GPT-6, 2026-10-14 (America/Guayaquil).
- Rol: implementador.
- Contratos: method-descriptor/1, descubrimiento capabilities/1 y opción del job
  engine-input/1 `options.allow_experimental`; results/1 permanece compatible.
- Fuentes permitidas: solicitud del ticket; APIs propias de engine-api y
  engine-analysis, sus pruebas y bitácoras; interfaces vendorizadas autorizadas
  de mamut-workflow v0.1.0 (registry.rs, descriptor.rs, catalog.rs, compiler.rs,
  contract.rs, workflow.rs y lib.rs). Sin red.
- Derivaciones propias: adaptación de documentos completos a puertos SDK
  adimensionales `document`; URN engine-input/results con compatibilidad de
  versión mayor. No se añaden fórmulas mecánicas ni se cambian los solvers.
- Código de terceros copiado: ninguno en este ticket. Se usa por dependencia
  path el SDK Apache-2.0 vendorizado previamente en #100, sin modificar vendor.
- Validación: resolución id/contrato mediante el compilador SDK, incompatibilidad
  de dominio/nombre/mayor, compatibilidad minor/patch, política Experimental
  antes del ensamblaje, determinismo de capacidades y evidencia existente.
- Comparación con oráculo externo: no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Comparación y alcance

El SDK ya proporciona MethodRegistry, fábricas, MethodDescriptor, CatalogDocument
para procesos que solo tienen JSON y WorkflowCompiler con resolución de contratos.
Se usa MethodRegistry<EngineMethod> porque el motor necesita fábricas ejecutables;
no se crea otro registro, índice, mapa ni implementación de MethodCatalog.

La fachada ya ejecutaba linearStatic, combinaciones, pDelta, modal y
responseSpectrum, con pruebas analíticas, cancelación, progreso, SHA-256 y
salida canónica. Se conservan esos cálculos y los avisos #78/#84/#85/#86.
Faltaban los descriptores, los estados, la evidencia en capabilities y el
consentimiento del job. Se separan los adaptadores del orquestador: `run`
invoca la instancia compilada por SDK, sin ramas por método. Las combinaciones
se resuelven en el mismo registro como postproceso de casos lineales recuperados.

Cada MethodDescriptor contiene el descriptor genérico SDK bajo `workflow`,
con versión de método 1, más estado, esquemas JSON, fixtures, citas a derivaciones
propias y rutas de provenance. No se inventan citas bibliográficas externas:
las ecuaciones y su validación están en ENG-API-001 (incrementos 2a/2c) y las
bitácoras originales de cada método. Las URN solo adaptan engine-input/1 y
results/1 a la sintaxis de contrato SDK; no publican otro formato de modelo/job.
Las opciones mecánicas se validan en el DTO del documento, no se duplican como
NumericParameters del workflow. No se implementa un plan de workflows general.

P-Delta es Experimental por las limitaciones documentadas #78/#84; se requiere
`options.allow_experimental: true` y se publica su fixture con consentimiento
explícito. Ausente o false produce invalid / API-E-INPUT / salida 2 con nombre
del método y opción requerida, sin ensamblar. Los otros cuatro métodos son
Validated dentro del alcance analítico publicado. Deprecated se define para
compatibilidad futura; no se depreca ningún método actual. Validated no certifica
modelos arbitrarios ni cierra los seguimientos abiertos.

capabilities conserva los campos anteriores y añade `method_descriptors` en
orden del registro. Se publican schemas para method-descriptor/1 y capabilities/1;
engine-input/1 admite la opción booleana del job. Todas las claves nuevas y
nuevas definiciones usan snake_case ASCII. LEGACY_DEFS permanece congelada.

## Cargo

engine-api añade únicamente una dependencia path a mamut-workflow =0.1.0.
Cargo.lock añade esa arista; no cambian versiones, miembros ni dependencias
externas. CARGO_TARGET_DIR conserva el valor exportado /workspace/.cargo-target/me45.

## Verificaciones anteriores al merge

Todas las comprobaciones Cargo usan `--offline` y el CARGO_TARGET_DIR exportado
`/workspace/.cargo-target/me45`; no se accedió a la red ni se instalaron paquetes.

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 521 pruebas y doctests, sin fallos ni
  ignoradas; incluye tres pruebas nuevas de resolución/política/descubrimiento.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --workspace --no-deps`: OK.
- `PATH=/home/box/.cargo/bin:$PATH cargo +stable check --offline --workspace
  --target wasm32-unknown-unknown`: OK, con el target ya instalado. Comprueba
  compilación; no ejecución en un runtime wasm.
- `git diff --check` y revisión de archivos modificados: OK. Sin cambios a
  docs/CLEAN_ROOM.md, .github/**, clean-room/** ni vendor/**; LEGACY_DEFS intacta.
- Sin panic/unwrap/expect en crates/engine-api/src.
- `bash clean-room/check.sh origin/main`: salida 1 únicamente por las cuatro
  apariciones preexistentes de `algorithm` en
  vendor/mamut-workflows/mamut-canon/src/lib.rs, bloqueo conocido de #100.
  No hay otros diagnósticos. Se exceptúa exclusivamente ese fallo autorizado;
  no se cambian el filtro, las listas ni el SDK vendorizado.

Revisión humana clean-room pendiente (J4). Commit local, sin push.

## Reconciliación del merge y determinismo · 2026-10-14

Base revisada: merge e53cf0d de j2/vnd-mamut-workflows, con #98 (libm y
determinismo), #99 (Winkler) y mamut-canon 585f32d. No quedan entradas sin
resolver en el índice ni marcadores de conflicto. La resolución de
crates/engine-api/Cargo.toml conserva mamut-workflow y el ejemplo cdylib
determinism_wasm; el registro y los adaptadores pasan sus pruebas existentes.
No se encontraron otras roturas del merge dentro de las verificaciones abajo.

Se reprodujeron los fallos shared_native_wasm_suite_hashes_are_repeatable y
v1_results_are_identical_in_ten_runs: los fixtures pdelta y pdelta-inclined
de #98 aún tenían options vacío. Ambos declaran ahora
`options.allow_experimental: true` en el job. Por decisión de Jonathan,
P-Delta conserva el estado Experimental hasta BENCH-001; no se cambia el
registro ni se omite su política para hacer pasar las pruebas.

Hashes SHA-256 del results/1 completo, referencia de #98 → regeneración:

| Caso | Anterior (#98) | Actual (ENG-CORE-006) |
| --- | --- | --- |
| pdelta | `5bcea296b5fd2652f75aac339b80bffa82e2076b06279f4020db817644a31101` | `61462a610d4b66c1ac13f0e5e3a37a4194052ca519cb0d3c2cd76bc2d3970ec4` |
| pdelta-inclined | `efd49f53009742c13ff5dbc66fcbf43e063cbfb707cc61dbe23ee043aaf4614f` | `01510cb66b56a52eeecdfa275d0c8035a775522c2f81dc59456497cbb0b5a31d` |

El consentimiento cambia el input canónico y su input_sha256, incluido en
results/1. Auditoría adicional offline: retirar allow_experimental del input
solo para calcular su hash anterior, sustituir únicamente input_sha256 en
el resultado actual y canonizar recupera exactamente ambos SHA-256 de #98.
Así se comprueba que ningún otro campo del resultado ni cálculo mecánico
cambió en estos dos casos. Los otros catorce hashes permanecen iguales.
method_descriptors sí amplía capabilities/1, pero esa salida no está incluida
en results/1 ni en los casos escalares: no causa cambios en estos hashes.

Regeneración reproducible mediante los comandos de docs/DETERMINISMO.md:
ejemplos determinism_native y determinism_wasm, más
scripts/check-determinism.mjs, sin npm, WASI ni wasm-bindgen. Rust 1.99.0
(LLVM 23.1.1), x86_64-unknown-linux-gnu y wasm32-unknown-unknown, Node
v22.19.0, flags habituales. Debug y release: dieciséis casos con diez corridas
por caso/target, hashes completos iguales; cmp de las salidas nativas de los
dos perfiles pasa. La tabla actualizada de docs/DETERMINISMO.md coincide con
los dieciséis hashes de ambas salidas. No había goldens congelados en las
pruebas Rust; se conserva su comparación de hashes y bytes entre ejecuciones.
Se corrige además la nota obsoleta sobre ausencia de Winkler en ese documento,
sin atribuirle cobertura en esta suite.

Verificaciones posteriores al merge, todas sin red y con CARGO_TARGET_DIR
exportado `/workspace/.cargo-target/me45`:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace --no-fail-fast`: 535 pruebas y doctests
  pasan, una falla, ninguna ignorada. Único fallo:
  engine_crates_use_only_the_central_scalar_module, por sqrt directo en
  crates/engine-validation/tests/support/winkler_harness.rs. El archivo es
  idéntico al de origin/main (abe7453); se deja intacto según la solicitud,
  pues su corrección corresponde al PR #106. Las tres pruebas de determinismo
  de engine-api pasan.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --workspace --no-deps`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- Cruce real con Node debug y release y cmp entre perfiles: OK.
- `bash clean-room/check.sh origin/main`: OK; la excepción de vendoring
  integrada valida el manifiesto de 585f32d y ya no produce el bloqueo
  documentado en las verificaciones anteriores.
- `git diff --check`: OK. Solo se modifican los dos fixtures P-Delta,
  docs/DETERMINISMO.md y esta bitácora. Sin cambios a vendor/**,
  clean-room/**, .github/** ni docs/CLEAN_ROOM.md.

Commit local, sin push. Revisión humana clean-room pendiente (J4).


## Reconciliación de #103 sobre #104 · 2026-10-15

Rama local `wip/core-006-sobre-005`: HEAD `67c54d3`, merge en curso de
`origin/j2/eng-core-005` (`6a4b58d`), con `origin/main` = `19c3ae5`.
Esta sección registra la nueva evidencia; las secciones anteriores describen
las comprobaciones históricas, incluidos sus bloqueos ya corregidos en la base.
No se aborta ni se hace rebase; no se ejecutan git add/commit y el índice se
conserva intacto. J2 debe marcar la resolución y crear el commit del merge.

Se resuelven Cargo.lock, crates/engine-api/README.md, src/lib.rs, src/run.rs y
docs/DETERMINISMO.md. Cargo.toml conserva las dos dependencias path
mamut-canon y mamut-workflow; Cargo.lock se regenera con `cargo metadata
--offline --format-version 1` y se comprueba con `cargo check --offline
--workspace`. La dependencia directa sha2 de engine-api se elimina: run usa
`mamut_canon::fingerprint_value` (RFC 8785/JCS), y las pruebas y ejemplos usan
`mamut_canon::sha256_hex`. sha2 permanece únicamente como dependencia
transitiva de mamut-canon; eliminarla requeriría modificar el vendor prohibido.
No cambian versiones de paquetes ni se modifica vendor.

lib.rs incluye methods y query, con sus exportaciones. run conserva despacho
y presupuesto de progreso por método de #103, sin duplicar el checkpoint
parse; emite git_commit y target también en el sobre de error interno de
#104. El README conserva descubrimiento/registro y huella/consultas.

### Tabla anterior/nueva

Hashes de #103: results/1 completo, antes de incorporar los metadatos y JCS
de #104. Hashes nuevos: results/1 con git_commit null, retirando únicamente
target tras comprobarlo; los cinco casos escalares siguen hasheando los mismos
bits big-endian sin JSON. No se compara el hash del input con el del resultado.

| Caso | Anterior (#103) | Nuevo (#103 + #104) |
| --- | --- | --- |
| cantilever | `d993e74910a942cdd27f727e635e3f109d7eac65ad8c0da69cec63a37afd1861` | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` |
| inclined-cantilever | `3378d324bd96923bab07640a17c9afd1fff18f69842a1f21bbc06152c8d76eed` | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` |
| modal-1dof | `2ba08c590e219f982b9b33f0955d9c790b2183b5d1af6309f09b579d43c92a71` | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` |
| pdelta | `61462a610d4b66c1ac13f0e5e3a37a4194052ca519cb0d3c2cd76bc2d3970ec4` | `980bf21b60a5b931a0f0c137ebc782a4a878abb95f7e8bce880c91c4374f56ca` |
| modal-coupled | `59499893099017bbb13ffcea8bed513537a059d929cddebc5511bee9a2de2f4e` | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` |
| spectrum-log-cqc | `a403028d3dc23b3d1e3312492866c7eacd9dd0431c2b50f5563bdefd83c25eb1` | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` |
| pdelta-inclined | `01510cb66b56a52eeecdfa275d0c8035a775522c2f81dc59456497cbb0b5a31d` | `9b5a4584e2151bb4d72c34933f564e8642e85600d44662e78ea376a61541a16b` |
| modal-inclined | `b3c4b92d66bc3894b2400354bb3518d78a8c224a6870cfc7aadd2861e2c0e480` | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` |
| spectrum-inclined | `a03dea7b4d9dd115cbef6b74cdf68e739e0d4655197957064d03a58a11bd0ac8` | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` |
| frame3d-inclined-static | `152dc9c3a40210634053378061c25668b93bf2eb018223e90b8b18712d9e754f` | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` |
| frame3d-inclined-modal | `58e6c5f1733a576a969d323164bf7dbee19155974688a33c73036b2462d4f060` | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` |
| scalar-trig | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` |
| scalar-exp-log | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` |
| scalar-powers-norm | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` |
| load-rotation-bits | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` |
| scalar-sin-cos | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` |

Cambian los once análisis frente a #103; los cinco casos escalares permanecen
idénticos. Auditoría offline adicional: retirar git_commit y target del
resultado nuevo y sustituir input_sha256 por el SHA-256 de canonical_json del
mismo input reproduce exactamente los once hashes anteriores de #103. Así se
atribuye el cambio a esos campos, sin diferencias mecánicas.

Nueve análisis y los cinco escalares coinciden con la tabla de #104. Para
pdelta y pdelta-inclined se comprueba que los inputs difieren exclusivamente
en `options.allow_experimental: true`, necesario por la política Experimental
de #103. Sustituir únicamente input_sha256 en el resultado nuevo por la huella
JCS del fixture de #104 reproduce exactamente sus hashes:

| Caso | Hash #104 | Hash nuevo |
| --- | --- | --- |
| pdelta | `426db50197873a7a8cafb586233ac7af70d8953c22d3a718fe6dbbb51b4637f0` | `980bf21b60a5b931a0f0c137ebc782a4a878abb95f7e8bce880c91c4374f56ca` |
| pdelta-inclined | `f08ff9c7264cb64c8209956d97da0b2e99343479854f11d39d75773b6ec7c71c` | `9b5a4584e2151bb4d72c34933f564e8642e85600d44662e78ea376a61541a16b` |

El único campo de results/1 que explica esa diferencia frente a #104 es:

| Caso | input_sha256 #104 | input_sha256 nuevo |
| --- | --- | --- |
| pdelta | `93887960a5cc262e58eefbe32a1942dc3910570c389c950d74472dc5108765de` | `06203cc969a0da17c9289014b81de32aabab113d3e5d98796571c2efcedec88e` |
| pdelta-inclined | `ea61ee74c64f263cda128ad360208d3c1ce846f1380b0b3bf46a8626329ad422` | `c10e197c32b64a35b9a9a4880f7564d3480ea95ebf6d5d7abf698794b6ed86e8` |

method_descriptors amplía capabilities/1 y no interviene en results/1 ni en
los casos escalares. No se cambia la política Experimental para obtener hashes.

### Ejecución y verificaciones

Sin red, sin instalar dependencias, con PATH=/home/box/.cargo/bin:$PATH,
CARGO_NET_OFFLINE=true, CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6
exportado y MAMUT_ENGINE_GIT_COMMIT sin definir. Rust 1.99.0
(LLVM 23.1.1), Node v22.19.0, x86_64-unknown-linux-gnu y
wasm32-unknown-unknown; flags Cargo habituales.

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace --no-fail-fast`: OK, 621 pruebas y
  doctests, cero fallidas y cero ignoradas. Incluye guardia escalar (36 pruebas),
  registro de métodos, huellas JCS, contrato results/1 y consultas.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --workspace --no-deps`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --offline -p engine-api --tests --target wasm32-unknown-unknown`:
  OK (compilación de tests, sin ejecución wasm).
- Comandos de cruce de docs/DETERMINISMO.md: build/run de los ejemplos
  determinism_wasm y determinism_native en debug y release, script Node en ambos
  perfiles y cmp de las salidas nativas: OK. Dieciséis casos, diez corridas por
  caso/target/perfil, todos los hashes iguales nativo/wasm y debug/release.
- `bash clean-room/check.sh origin/main`: OK.
- `git diff --check`: OK; sin marcadores de conflicto en el árbol de trabajo.

Salidas y logs locales: `/tmp/me-geo6-merge-evidence/`, incluidos
native-debug.txt, native-release.txt, node-debug.log, node-release.log,
status.txt y fingerprint-audit.json; auditoría temporal fuera del repo.
No se modifican docs/CLEAN_ROOM.md, .github/**, clean-room/** ni vendor/**.
El índice conserva sus entradas unmerged hasta que J2 marque la resolución.
Revisión humana clean-room pendiente (J4).
