# Bitácora clean-room · ENG-SFC-002

- Ticket / PR: ENG-SFC-002 · MAM-91; PR PENDIENTE.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Rol: implementador del adaptador CLI; sin implementación numérica nueva.
- Contrato implementado: CLI nativo `mamut-engine` sobre SDK-WF-012;
  cli-run/1 + extensión `engineResults`, engine-input/1, results/1;
  catálogo de operaciones Engine v1 y códigos `ExitCodes::engine_v1()`.
- Fuentes permitidas usadas: contratos, implementación y fixtures propios de
  `crates/engine-api/src`, `crates/engine-api/tests/fixtures/determinism/`;
  configuración del workspace y convenciones de `bindings/python`;
  `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md` y estado previo de este ticket;
  `vendor/mamut-workflows/{README.md,mamut-workflow-cli/README.md}` y fuentes
  locales de mamut-workflow, mamut-workflow-cli y mamut-workflow-execution,
  commit `1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d` (SDK-WF-012 / MAM-493).
  No se encontró AGENTS.md aplicable en el repo ni sus ancestros.
- Derivaciones propias: ninguna ecuación nueva; decisiones de integración
  documentadas en `crates/engine-cli/README.md`.
- Código de terceros copiado: ninguno nuevo. Se enlazan por path las bibliotecas
  propias de MAMPRO ya vendorizadas bajo Apache-2.0, con sus LICENSE/NOTICE
  existentes. No se modifica ningún archivo de vendor.
- Validación: pruebas de integración del propio binario, contraste directo con
  engine-api y manifiesto SDK, fixtures propios existentes; controles offline
  del workspace y guardias de integridad/clean-room.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni código de derivados o rutas prohibidas para este ticket."
- Revisor clean-room: **PENDIENTE (J4)**.

## Estado y cambios

El bloqueo histórico documentado aquí para SDK-WF-007 queda resuelto por la
base ya fijada en el repo, SDK-WF-012 / MAM-493. No se re-vendoriza ni se altera
su código. Se entrega `crates/engine-cli` como paquete `engine-cli`, binario
`mamut-engine`, biblioteca `engine_cli`, miembro y default-member del workspace.
La licencia se hereda con `license.workspace = true`: AGPL-3.0-only tras #120.
Cargo.lock solo incorpora el paquete local, sin dependencias externas nuevas.

Parser y despacho son exclusivamente `mamut_workflow_cli::parse_args` y
`dispatch`. CliConfig inyecta identidad mamut-engine, versión de
`engine_api::version()`, operación run, tabla engine_v1, cancelación y un
MethodProvider propio. El proveedor construye métodos reales engine.run y
engine.validate, que llaman a engine-api con la entrada local. No se implementa
un executor, parser de argumentos, validador numérico ni compilador paralelo.
El JSON de entrada se valida por engine-api.

Se incluyen catálogo propio embebido y dos workflows de ejemplo. Como los
parámetros SDK son numéricos, la entrada estructurada se obtiene de la ruta
`MAMUT_ENGINE_INPUT` (o PathBuf suministrado por biblioteca); no se añade un flag.
La lectura aplica MAX_INPUT_BYTES del motor y clasifica fallos como Document/2.
Stdin `-` sigue siendo el transporte estándar de catálogo o workflow de la base.
El README distingue expresamente esa ruta de un futuro stdin de engine-input/1.

Method devuelve solo Result<Value,String> y el manifiesto solo guarda huellas.
El proveedor conserva el estado semántico en memoria compartida con el
anfitrión; el adaptador transforma el sobre terminal de run, sin cambiar el
manifiesto sellado, para añadir `engineResults`. Los Outcome/diagnósticos se
mapean explícitamente a FailureKind; Warning usa ExitCodes.warning porque no
existe esa variante en FailureKind. No se clasifican mensajes arbitrarios.
Se comprueban 1/3/4 con ejecuciones reales de engine-api, además de 0/2/5/70.

NDJSON se escribe y vacía en stderr por la base; solo se retiene el sobre de
stdout hasta adjuntar resultados. Control conecta CancelFlag y CancelToken
entre pasos y etapas, con callback síncrono Progress de engine-api; también
acepta cancelación del token desde un hilo del anfitrión, sin crear hilos.
El binario conecta además MAMUT_ENGINE_CANCEL_FILE a Control::with_cancel_file:
la presencia de la ruta cancela antes de un paso o tras un callback del motor.
No se instala un manejador de señales ni un hilo; Ctrl-C conserva la semántica
del SO. El marcador es optativo, no se crea ni elimina y, una vez observado,
la cancelación queda enclavada. Se comprueba el código 5 ejecutando el binario,
tanto con marcador previo como creado tras spawn mientras espera el workflow.
En biblioteca se comprueba también la cancelación entre etapas run/validate.
MCP no retiene los documentos completos de llamadas sucesivas en State; solo
el comando run necesita capturarlos para enriquecer su sobre terminal.
No se añade unsafe ni unwrap/expect/panic en biblioteca/binario propios.

## Correspondencia y decisiones abiertas

La tabla completa y ejemplos de uso están en `crates/engine-cli/README.md`.
Describe publica versión/identidad/comandos/códigos; methods lista las dos
operaciones registradas. No equivalen al documento entero capabilities ni a
los esquemas JSON. Validate estándar valida workflows; el método engine.validate
valida la entrada del motor. No existen comandos estándar schema, explain,
version o capabilities: siguen rechazados como uso inválido (2).

Quedan abiertas, sin inventar comandos propios:

- Transporte de documentos por paso, sustitución del puente por entorno y
  entrada directa engine-input/1 por stdin.
- Publicación/aprobación de la extensión engineResults y del informe de
  engine.validate, o canal de resultados completos y errores tipados en el SDK.
- Exposición estándar de capabilities completo, contracts de version, schema
  y explain. Los descriptores solo referencian contratos.
- Cancelación cooperativa por señales del proceso y un posible NDJSON de etapas
  internas del motor; hoy NDJSON es el contrato started/finished del workflow.
- Paridad MCP: el despacho estándar conserva sus manifiestos de huellas y
  semántica genérica de errores de método (70), sin extensión terminal
  engineResults ni código 1 para warnings. Se inyectan proveedor/identidad/tabla,
  pero no se declara MCP equivalente al run enriquecido de este ticket.
- **Binarios Windows/Linux/macOS en CI: PENDIENTE; lo decide Jonathan**.
  No se modifica `.github/workflows`, `clean-room/` ni vendor.
- Revisión humana y clean-room **PENDIENTE (J4)**.

## Verificación offline · 2026-10-17

Toolchain seleccionada con
`export PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH`
(rustc 1.99.0) y
`CARGO_TARGET_DIR` propio del worktree.
`CARGO_NET_OFFLINE=true`, `PIP_NO_INDEX=1`; sin acceso a fuentes externas.

Resultados de la revisión actual, todos en verde:

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **643 passed / 0 failed / 0 ignored**, incluidos doctests; **13 tests propios del CLI** |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK, sin exclusiones |
| `bash clean-room/check.sh origin/main` | OK |
| `bash scripts/check-vendor.sh` | OK, inventario y hashes íntegros |
| `bash clean-room/tests/run.sh` | **73 pruebas / 0 fallos** |
| `git diff --check` | OK |

Determinismo: se ejecutaron los ejemplos existentes determinism_native y
determinism_wasm de engine-api, en debug y release, con
`MAMUT_ENGINE_GIT_COMMIT` sin definir. `scripts/check-determinism.mjs` confirmó
**16 casos nativo=wasm, diez corridas por caso/target/perfil**. `cmp` confirmó
igualdad debug/release y una comparación por nombre de caso contra la tabla
final de `docs/DETERMINISMO.md` confirmó **16/16 hashes sin cambios**.
No se modificaron fixtures, implementación numérica ni la tabla de referencia.

Evidencia local: `$CARGO_TARGET_DIR/mam91-verificacion/` (logs de cada comando,
salidas nativas y cruce wasm). Incidencia operativa corregida: los logs se
escribieron inicialmente en `$TMPDIR/mam91-verificacion`, fuera de la restricción
de rutas del ticket; se trasladaron al target autorizado y se fijó `TMPDIR`
bajo ese target para los controles finales. No se modificaron vendor/,
clean-room/, docs/CLEAN_ROOM.md, .github/workflows ni CODEOWNERS. Cargo.toml y
Cargo.lock conservan la integración de engine-cli que ya traía la rama.

La cobertura del binario incluye 0/1/2/3/4/5/70: combinación real con avisos,
elemento no soportado, mecanismo, iteración P-Delta agotada, cancelación por
marcador y pipe stdout cerrado antes de entregar el workflow. Se comparan los
resultados con engine-api y el manifiesto con la base. Los tests de biblioteca
cubren además cancelación por flag/token/archivo entre etapas, fallos de E/S
y aislamiento del estado entre invocaciones. No se añade lógica numérica.

Revisión clean-room **PENDIENTE (J4)**.

## Seguimiento #141

- Ticket: MAM-91 · GitHub #141, seguimiento J4 de #138.
- Autor: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Rol: implementador de pruebas; sin lógica numérica nueva.
- Fuentes: contratos e implementación propios de `crates/engine-cli/src/lib.rs`,
  `crates/engine-api/src/{lib.rs,limits.rs}`, `crates/engine-model/src/lib.rs`,
  `docs/CLEAN_ROOM.md` y `provenance/TEMPLATE.md`. Sin fuentes externas,
  derivaciones numéricas, código de terceros copiado ni comparación con oráculos.
- Declaración: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni derivados o rutas prohibidas para este ticket."
- Revisor clean-room: **PENDIENTE (J4)**.

Se añaden dos tests unitarios. La lectura se extrae sin cambios de comportamiento
a `read_engine_input`, privada (sin ampliar API ni visibilidad). Un generador de
espacios sin EOF contabiliza los bytes consumidos; un fusible devuelve error de
E/S después de MAX+2 para acotar memoria y tiempo incluso al quitar el límite.
Se exige consumir exactamente MAX+1 y devolver Document/2 (`invalid_input`).

El segundo test inyecta un diagnóstico de error ENG-E-0013 en `diagnostic_status`,
el clasificador privado usado por el método validate; recorre `failure_kind` y
`report_failure` y exige Unsupported/3 y el sobre `unsupported`. La API actual
`validate_with` solo comprueba topología y no genera ENG-E-0013: una prueba con
elemento desconocido retorna 0 en validate (run sí lo rechaza). Por ello esta
regresión se prueba en la biblioteca con el diagnóstico explícito, sin extender
la validación ni inventar una entrada JSON que active una ruta hoy inalcanzable.
No se detectó un bug en los dos comportamientos objeto de este seguimiento;
no se introduce un fix funcional.

Mutaciones temporales, compiladas y ejecutadas por separado:

```bash
cargo test -p engine-cli --lib --offline tests::input_read_stops_at_max_plus_one_and_returns_document_two -- --exact
cargo test -p engine-cli --lib --offline tests::validate_eng_e_0013_maps_to_exit_three -- --exact
```

- Sin `.take((engine_api::MAX_INPUT_BYTES + 1) as u64)` (añadiendo `mut` al
  parámetro solo para compilar la mutación): **0 passed / 1 failed**, retorno
  Cargo 101; aserción 8388610 bytes consumidos frente a 8388609 esperados.
- Con `"ENG-E-0013" => Status::Invalid`: **0 passed / 1 failed**, retorno Cargo
  101; aserción salida 2 frente a 3.
- Tras cada mutación se restaura el archivo byte a byte y se repite exactamente
  su comando: **1 passed / 0 failed**, retorno 0 en ambos casos. No quedan
  mutaciones en el árbol.

Verificación offline con Rust 1.99.0, PATH de stable indicado en el ticket,
`CARGO_TARGET_DIR` apuntando a un directorio de compilación externo al worktree,
`CARGO_NET_OFFLINE=true` y `PIP_NO_INDEX=1`; temporales y logs en
`$CARGO_TARGET_DIR/mam91-141/`.

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **654 passed / 0 failed / 0 ignored**, incluidos doctests; CLI: 2 unitarios nuevos + 13 de integración |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --check` | OK |

Determinismo: ejemplos `determinism_native` y `determinism_wasm`, en debug y
release, con `MAMUT_ENGINE_GIT_COMMIT` sin definir. El script existente
`scripts/check-determinism.mjs` confirma **16 casos nativo=wasm, diez corridas
por caso/target/perfil**. Las salidas nativas debug/release son idénticas byte
a byte y sus 16 hashes coinciden por nombre con la tabla final de
`docs/DETERMINISMO.md`. No se modifican fixtures ni resultados numéricos.

Evidencia: `mutation-{reading-limit,validate-0013}.log`,
`restored-{reading-limit,validate-0013}.log`, logs individuales de los controles,
`native-{debug,release}.txt` y `determinism-cross-{debug,release}.log`, todos en
`$CARGO_TARGET_DIR/mam91-141/`. Solo cambian tests/extracción privada de lectura,
esta bitácora y CHANGELOG.md. No se tocan rutas protegidas ni Cargo.toml/lock.
Pendiente únicamente la revisión clean-room **PENDIENTE (J4)** de este seguimiento.

## Seguimiento #146

- Ticket: MAM-91 · GitHub #146, seguimiento de ENG-SFC-002 y #141.
- Autor: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Rol: implementador de pruebas; contrato: lectura CLI con límite inclusivo
  `engine_api::MAX_INPUT_BYTES` y rechazo Document/2 al superarlo.
- Fuentes permitidas: implementación y contratos propios de
  `crates/engine-cli/src/lib.rs`, `docs/CLEAN_ROOM.md` y `provenance/TEMPLATE.md`.
  Validación mediante entradas sintéticas propias de espacios con EOF;
  sin derivaciones numéricas, código de terceros copiado ni comparación con oráculos.
- Declaración: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni derivados o rutas prohibidas para este ticket."
- Revisor clean-room: **PENDIENTE (J4)**.

Se añaden dos tests: exactamente MAX bytes con EOF devuelve Ok y longitud MAX;
MAX+1 con EOF real devuelve Document y se traduce a salida 2. No se detecta
un bug funcional: solo cambian tests y documentación. Se sustituye la ruta
local de compilación de #141 por una descripción genérica.

Mutación temporal `input.len() > engine_api::MAX_INPUT_BYTES` → `>=`, ejecutada con
`cargo test -p engine-cli --lib --offline tests::input_read_accepts_exact_max_with_eof -- --exact`:
**0 passed / 1 failed**, error Document, salida Cargo 101. Tras restaurar el archivo
byte a byte (`cmp`), el mismo comando da **1 passed / 0 failed**, salida 0.
Logs: `mutation-inclusive-limit.log` y `restored-inclusive-limit.log`, en
`$CARGO_TARGET_DIR/mam91-146/`.

Verificación offline con Rust 1.99.0, PATH de stable indicado en el ticket,
`CARGO_TARGET_DIR` externo al worktree, `CARGO_NET_OFFLINE=true` y `PIP_NO_INDEX=1`:

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **657 passed / 0 failed / 0 ignored**, incluidos doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --check` | OK |

Determinismo: **16/16 hashes sin cambios** frente a `docs/DETERMINISMO.md`,
con cruce nativo/wasm en debug y release, diez corridas por caso/target/perfil
mediante `scripts/check-determinism.mjs`. Salidas nativas de ambos perfiles
idénticas byte a byte; `MAMUT_ENGINE_GIT_COMMIT` sin definir.
Logs de los controles, `native-{debug,release}.txt`,
`determinism-cross-{debug,release}.log` y `determinism-table.log` en
`$CARGO_TARGET_DIR/mam91-146/`. Código de producción idéntico a HEAD;
sin cambios en rutas protegidas ni Cargo.toml/lock.
Pendiente únicamente la revisión clean-room **PENDIENTE (J4)**.

## Arreglo BrokenPipe en tests

- Ticket: MAM-91; autor: Codex gpt-6.1-sol high · 2026-10-17
  (America/Guayaquil); rol: implementador de pruebas.
- Contrato: tests del CLI conservan códigos de salida y diagnósticos cuando
  el hijo termina antes de leer stdin. Fuentes: tests propios de
  `crates/engine-cli/tests/cli.rs`, `docs/CLEAN_ROOM.md` y `provenance/TEMPLATE.md`.
- Derivación propia del manejo de E/S; sin fuentes externas, código de terceros
  copiado, derivaciones numéricas ni comparación con oráculos.
- Declaración: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni derivados o rutas prohibidas para este ticket."
- Revisor clean-room: **PENDIENTE (J4)**.

Se elige (a): `write_child_stdin` tolera exclusivamente `io::ErrorKind::BrokenPipe`;
cualquier otro error de escritura falla el test. Es la opción más simple:
no añade hilos ni sincronización para una entrada pequeña cuyo lector puede
terminar legítimamente con salida 2. El handle tomado se cierra al salir del
helper, antes de `wait_with_output`. Se aplica a las cuatro escrituras a stdin
de hijos: `Files::run`, catálogo, cancelación y stdout cerrado. Todos los asserts
existentes de código de salida, stdout y stderr se mantienen sin cambios.
Solo se modifican tests y documentación; sin cambios en producción ni cálculos.

Validación offline con Rust 1.99.0, PATH de stable indicado,
`CARGO_TARGET_DIR` propio del worktree, `CARGO_NET_OFFLINE=true`
y `PIP_NO_INDEX=1`: el test afectado pasa **400/400 ejecuciones**, con ocho
procesos concurrentes (`--exact` sobre el binario compilado por Cargo).

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **658 passed / 0 failed / 0 ignored**, incluidos doctests; CLI: **13/13** de integración |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` y `git diff --check` | OK |

Los **16/16 hashes** de `docs/DETERMINISMO.md` coinciden con diez ejecuciones
nativas debug por caso de `determinism_native`, sin `MAMUT_ENGINE_GIT_COMMIT`.
Logs en `$CARGO_TARGET_DIR/mam91-brokenpipe/`. Pendiente: revisión clean-room
**PENDIENTE (J4)**; cambios sin commitear.

## MAM-862 / ENG-SFC-011 · transporte `--input` · 2026-10-21

- Autor: Codex (GPT-6), implementador, America/Guayaquil.
- Alcance: pre-parseo del adaptador CLI para `--input <ruta|->`, documentos
  embebidos por defecto y reutilización de la entrada entre pasos; sin cambios
  numéricos, de contratos del motor, dependencias ni fuentes vendorizadas.
- Fuentes usadas: código y documentación propios de `crates/engine-cli`, límite
  de `engine-api`, `docs/DETERMINISMO.md`, `crates/engine-api/tests/val032_runs.rs`,
  fixture determinista `cantilever.json` y fixture propio VAL-032
  `docs/validation-report/fixtures/val032/E1-v1.1.2.json`; parser y despacho
  locales del SDK vendorizado para conocer sus puntos de extensión públicos;
  `clean-room/check.sh` y `provenance/TEMPLATE.md` para registrar la verificación.
- Código externo copiado, derivaciones numéricas y comparación con oráculos:
  ninguno. No se consultaron fuentes externas, resultados de otros motores,
  rutas restringidas ni registros de otras sesiones. Trabajo sin red.
- Compatibilidad: se comparan bytes completos entre `--input`, documentos
  embebidos y MAMUT_ENGINE_INPUT con catálogo/workflow explícitos. La validación
  explícita de workflows conserva su comportamiento; la abreviada ejecuta
  `engine.validate`. La entrada por variable conserva su clasificación de
  errores y prioridad de cancelación. MCP mantiene la lectura por llamada.
- E1 conserva su aviso conocido de recuperación de diafragmas y el código 1;
  el test de éxito con código 0 usa el voladizo determinista existente. No se
  suprimen avisos para satisfacer una expectativa incompatible con el fixture.
- Verificación inicial: fmt, diff y clean-room OK; Clippy y tests quedaron
  bloqueados por la ausencia de `/workspace/.cargo-target/sfc11`. En la ronda 2
  el destino ya existe; se ejecutan allí los controles offline indicados abajo.
- Revisor clean-room: **PENDIENTE (J4)**.

### Ronda 2 · revisión OpenCode · 2026-10-21

- Autor: Codex (GPT-6), implementador; mismas fuentes y alcance de MAM-862.
  Se consulta el parser local `mamut-workflow-cli/src/parser.rs` del SDK
  vendorizado solo para verificar su sintaxis; no se modifica ni copia código.
  Sin red, fuentes externas, derivaciones numéricas ni comparación con oráculos.
- Decisión: la base reconoce tokens exactos `--catalog` y `--workflow`, sin
  `=valor` ni separador `--`. Se rechaza explícitamente `--input=valor` con
  Usage/2 para conservar una única sintaxis. `--` también es Usage/2 antes,
  después o como supuesto valor de `--input`. Un valor ausente, vacío o que
  comienza con guion es Usage/2, salvo `-` (stdin); las rutas con guion pueden
  escribirse con `./`. Tests del binario cubren ambos comandos, flags conocidos,
  desconocidos y ayuda, y contrastan las formas `=` con el parser de la base.
- Límites por stdin: documentos válidos rellenados con espacios hasta MAX-1 y
  MAX bytes dan 0, stdout con resultados y stderr vacío tanto en run como en
  validate; MAX+1 da 2 y stdout vacío. Se conserva la cobertura de UTF-8 y rutas.
- README aclara la lectura completa de stdin antes del despacho, con límite
  inclusivo de 8 MiB y sin streaming por paso.
- Temporales: se mantiene PID + contador atómico global, sin reinicios,
  creación exclusiva y hasta 128 reintentos ante AlreadyExists. Relaxed basta
  para reservar nombres únicos; la creación del directorio arbitra colisiones
  con residuos de procesos cuyo PID se haya reutilizado. Se extrae una función
  privada que permite probar con raíz y contador locales: ocho colisiones
  previas y 16 creaciones, rutas únicas con PID, contenido correcto,
  limpieza propia y conservación de los directorios que provocaron colisiones.
  El test no altera el contador global, evitando interferencias entre tests.
  Otro test de integración sincroniza con barrera 16 invocaciones concurrentes
  de EngineCli, usando el contador global real, y exige resultados idénticos.

Verificación con Rust 1.99.0, `CARGO_TARGET_DIR=/workspace/.cargo-target/sfc11`,
`CARGO_NET_OFFLINE=true` y `PIP_NO_INDEX=1`. Logs y temporales de los controles
en `$CARGO_TARGET_DIR/mam862-r2/`. Sin commits ni cambios en Cargo, vendor,
`.github` o `clean-room`; revisión humana clean-room **PENDIENTE (J4)**.

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 758 passed / 0 failed / 0 ignored; CLI: 5 unitarios y 23 de integración |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --check` | OK |
