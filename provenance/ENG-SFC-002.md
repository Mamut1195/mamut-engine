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
`export CARGO_TARGET_DIR=/workspace/.cargo-target/me-sfc2`.
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
escribieron inicialmente en `/tmp/mam91-verificacion`, fuera de la restricción
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
