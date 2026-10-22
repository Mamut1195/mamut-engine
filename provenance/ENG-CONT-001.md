# Bitácora clean-room · ENG-CONT-001

- Ticket: MAM-867 (ENG-CONT-001).
- Autor: agente Codex (GPT-6), 2026-10-21 (America/Guayaquil).
- Rol: implementador.
- Base local: `origin/main` = `7375fa25a34a3cb004bcaec84f645743b864ff89`.
- Worktree: `/workspace/mamut-engine-cont1`.
- Contratos: `cli-error/1`, `error-codes/1`, `error-codes-lock/1`.
- Fuentes permitidas usadas: ticket y restricciones del usuario; código propio de
  engine-cli, engine-api y catálogo engine-core; README del CLI,
  `docs/DETERMINISMO.md`, `changelog.d/README.md`; API pública y rutas de emisión
  del SDK ya vendorizado en el repositorio (solo lectura, sin cambios).
- Derivaciones propias: diseño del sobre terminal, registro y guardias de
  compatibilidad descritos en `crates/engine-cli/README.md`.
- Código de terceros copiado: ninguno nuevo. El validador de tests se adapta del
  propio `crates/engine-api/tests/support/mod.rs`, sin dependencias nuevas.
- Comparación con oráculo: no.
- Declaración: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni
  derivados para este ticket."
- Sin red; no se consultaron fuentes restringidas ni logs de otras sesiones.
- Sin cambios en vendor, Cargo.toml/Cargo.lock, .github, clean-room,
  CLEAN_ROOM.md, CODEOWNERS, deny.toml, CHANGELOG.md, barra axial ni truss.
- Revisor clean-room: pendiente de revisión humana.

## Decisiones y alcance

`src/error.rs` centraliza todos los fallos terminales observables del adaptador.
El esquema se publica junto al CLI (`schemas/cli-error-1.json`,
`urn:mamut:cli-error:1`) para mantener separado el contrato de proceso de los
contratos numéricos de engine-api. No cambia la API de esquemas del motor.

Se elige cli-error/1 como único sobre terminal en stderr, conservando los campos
compatibles del fallo de cli-run (`type`, identidad, `exitCode`, `outcome`,
`error` y documento fallido opcional). `error.code` coincide con el nuevo `code`;
se sustituye WF-E-* por ENG/API cuando hay diagnóstico principal o por CLI-E-*.
En otros comandos se sustituye el antiguo `{exit_code,message}`. El comando
real identifica `operation`; argumentos desconocidos y bytes no UTF-8 no se
reflejan. `args_os` evita que std::env::args aborte ante argumentos no UTF-8.

Los diagnósticos y documentos del motor se conservan íntegros. El primer error
identifica el código principal; entidades no implican una ubicación JSON única,
por lo que pointer es null. Hint procede del diagnóstico o es null. Los textos
base ingleses están en messages/en.json y las claves estables en el registro;
no se implementa i18n ni se traducen diagnósticos existentes. ENG-E-0014 se mapea
a inestabilidad / 4 también en el clasificador validate, como hace engine-api.

El registro inicial congela 32 códigos (9 CLI, 14 ENG, 9 API). La versión de alta
es la del registro, 1.0.0. El lock contiene definiciones completas inmutables,
excepto status; se permiten deprecaciones y altas por adición. Los tests detectan
ids duplicados, desaparición, cambios de definición y literales emitidos sin
registro. El inventario léxico recorre src de todos los crates, omite comentarios,
chars y elementos cfg(test); no analiza vendor ni códigos dinámicos. Todos los
emisores actuales usan literales.

MCP: el `serve` de mamut-workflow-cli construye errores de tools mediante
CliFailure::envelope, y McpServer del SDK construye los errores JSON-RPC. El
CliConfig que recibe dispatch no expone reemplazo de estos sobres. Cambiar esos
frames exigiría vendor o reemplazar/interceptar el transporte; se dejan intactos
y se documenta la excepción autorizada. Los errores terminales de catálogo y
transporte MCP sí adoptan cli-error/1. Se prueba el error RPC de parseo y la
terminación por frame truncado para distinguir ambos canales.

Timeout (6) sigue reservado en origin/main; no se añade temporizador ni se toca
el trabajo del PR #202. Se prueba el emisor con FailureKind::Timeout. Los
códigos alcanzables 2/3/4/5/70 se prueban con el binario, incluida cancelación
por archivo y pipe stdout cerrado. stderr roto impide entregar un sobre; se
retorna 70 sin reintentar. Una E/S fallida después de escribir puede dejar bytes
parciales; los fallos anteriores a la salida dejan stdout vacío.

## Validación local

Entorno: PATH con `$HOME/.cargo/bin`,
`CARGO_TARGET_DIR=/workspace/.cargo-target/cont1`, Cargo siempre `--offline`.

- `cargo fmt --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: OK, 761 pruebas, 0 fallos, 0 ignoradas
  (117 suites, incluidos doctests y SDK vendorizado que ya forma parte del workspace).
- CLI: 31 pruebas propias (7 unitarias, 18 del binario/adaptador, 5 del registro,
  1 de mutaciones del esquema). Validación de todos los FailureKind, timeout
  reservado incluido, y de cada fallo observado en los tests del binario.
- Comparación exacta de stdout de run exitoso con la reconstrucción del sobre
  anterior a partir del SDK, y conservación de bytes canónicos del motor en los
  fixtures existentes. Pruebas negativas de esquema y de congelación del registro.
- `bash clean-room/check.sh origin/main`: OK, con todos los archivos nuevos
  incluidos mediante `git add -N` en un índice temporal.
- `git diff --check`: OK, también con el índice temporal.

El índice real está en `/workspace/mamut-engine-r/.git/worktrees/mamut-engine-cont1`,
fuera de las raíces escribibles; el intento de `git add -N` fue rechazado por el
sistema de archivos de solo lectura. Para verificar el diff completo se usaron
`GIT_INDEX_FILE=/tmp/mam867-review.index`,
`GIT_OBJECT_DIRECTORY=/tmp/mam867-review-objects` y como
`GIT_ALTERNATE_OBJECT_DIRECTORIES` el directorio de objetos original obtenido con
`git rev-parse --git-path objects`. Tras `git read-tree HEAD`, se aplicó `git add -N`
a los diez archivos nuevos y se ejecutó la guardia con ese entorno. Tanto el
índice real como los objetos originales quedan intactos; los archivos nuevos
siguen apareciendo como untracked en el `git status` normal.
- Sin commit. `scripts/changelog-check.sh origin/main` no aplica sin commit,
  conforme a las instrucciones; se añade `changelog.d/MAM-867.added.md`.

## Revisión MAM-867 · ronda 2

- Se validan los archivos reales de registro y lock mediante `include_str!`.
  La guardia exige ids únicos en ambos y compara el lock completo, en orden,
  con 32 definiciones completas incrustadas en `tests/error_registry.rs`.
  Comparar valores exactos evita introducir un hash propio o dependencias.
- Una edición coordinada de registro y lock ya no permite borrar ni modificar
  entradas congeladas. Los tests negativos mutan cada entrada y cada campo,
  eliminan entradas en ambos archivos y alteran el orden del lock. También
  prueban que deprecated conserve su definición y reserve el id frente a altas
  duplicadas, tanto active como deprecated.
- Cada alta exige extender registro, lock y ancla del test. Límite explícito:
  quien modifique también el ancla puede eludir la guardia. Sin una referencia
  externa inmutable no hay garantía automática frente a esa tercera edición;
  la revisión debe exigir que el lock y el ancla solo crezcan por adición.
- Se revisaron definiciones de códigos en `crates/*/src`, incluido el catálogo
  `engine-core::catalog::CODES`, referencias indirectas y códigos locales API.
  El registro ya contiene los 14 ENG y los 9 API actuales, incluido ENG-E-0014.
  La cobertura recorre todos esos src y la regresión léxica incluye definiciones
  const, static y CodeInfo usadas indirectamente, además de llamadas directas.
- `experiments.py` reexporta el transporte de `_experiment_workflows.py`.
  Se adapta ese transporte para cli-error/1 sin engineResults: conserva code,
  error.message, diagnostics y el sobre completo en details. results/document
  permanecen None. Sigue admitiendo el formato anterior SDK-EXIT-* y la ruta
  con documento results/1 conserva sus excepciones y diagnósticos originales.
  Tests nuevos ejercitan compile_workflow y run_workflow sin documento, las
  categorías de salida, copias independientes y compatibilidad anterior; el
  test con CLI real verifica también el fallo compile sin engineResults.

Validación offline de ronda 2:

- `cargo fmt --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: OK, 762 pruebas, 117 suites,
  0 fallos y 0 ignoradas; 6 pruebas del registro.
- Wheel debug construida con maturin 1.15.0 local, `build --offline --locked`,
  sin dependencias nuevas; instalada con pip --no-index --no-deps en venv
  temporal con NumPy ya instalado. Se generaron 27 referencias independientes
  con facade_reference offline y se construyó engine-cli offline.
- `python -I -B -m unittest discover -s bindings/python/tests -v`: OK,
  77 pruebas, sin omisiones, con MAMUT_EXPERIMENT_CLI y referencias Rust.
  Log local: `/tmp/mam867-r2-python-test.log`; log de Cargo:
  `/tmp/mam867-r2-cargo-test.log`.
- `bash clean-room/check.sh origin/main`: OK, incluyendo los diez archivos
  nuevos mediante índice y objetos temporales bajo /tmp, sin alterar el índice
  real. `git diff --check`: OK con el mismo inventario completo.
- Sin red ni commit; se mantienen todas las restricciones de archivos del ticket.
