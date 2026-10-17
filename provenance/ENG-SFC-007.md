# Bitácora clean-room · ENG-SFC-007 · MAM-94

- Autor: Codex, 2026-10-15, America/Guayaquil.
- Rol: implementador, bajo la solicitud del usuario.
- Rama: `j2/eng-sfc-007`, basada en main; HEAD inicial
  `800ef3f` (ENG-CORE-010-FU2). Dependencias PY-002/API-001 presentes.
- Fuentes: solicitud del usuario; código, contratos, fixtures y pruebas locales
  de engine-api; scaffold bindings/python; manifiestos y provenance existentes;
  documentación de tipos y conversiones de PyO3 0.25.1 en el registro Cargo
  local. Herramientas Rust, Python y maturin ya instaladas. Sin red.
- Derivaciones propias: adaptación de la fachada Rust a Python, conversión de
  valores JSON, excepciones con contexto, token atómico y progreso diferido.
  Sin implementación numérica ni acceso directo a solvers desde el binding.
- Código de terceros copiado: ninguno. Sin dependencias nuevas del registro;
  se reutilizan engine-api, serde_json 1.0.154 y PyO3 0.25.1 disponibles offline.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Sin cambios de licencia declarada ni textos legales añadidos; sin cambios a
  docs/CLEAN_ROOM.md, .github/**, clean-room/** ni vendor/**. Sin publicación.
- Revisor clean-room externo: PENDIENTE (J4).

## Contrato y decisiones

`_native` expone validate/run/capabilities/schema/explain/version; __init__.py,
_native.pyi y py.typed componen la API tipada instalada. validate devuelve un
objeto con diagnostics; los defectos topológicos siguen siendo diagnósticos,
como en engine-api. run devuelve exactamente el documento canónico results/1
para ok/warning. version devuelve el objeto engine_version/contracts de la
fachada, sustituyendo la cadena del scaffold; __version__ conserva la cadena.
No se duplica la implementación del motor.

Los fallos de frontera y de análisis producen EngineError o sus subclases
InvalidInputError, AnalysisError, CancelledError e InternalEngineError. Se
preservan code, entities y cause del primer error; los fallos de run conservan
además document y diagnostics completos. Los errores de frontera usan la misma
traducción que engine-api::run (API-E-INPUT o ENG-E-0009). Los nombres/códigos
no publicados también se tratan como API-E-INPUT, con su causa original.

run y validate usan py.allow_threads. CancelToken contiene un Arc del token
atómico de engine-api; otro hilo Python puede solicitar cancelación. El motor
consulta el token existente entre etapas/iteraciones/modos; no se ejecuta código
Python en iteraciones. La cancelación es cooperativa: no interrumpe un solve,
eigensolver, ensamblado o parseo indivisible. El token no se restablece.

El callback Rust de progreso acumula eventos solo si se solicitó progress.
Después del run se readquiere el GIL y se entregan en orden a Python, incluidos
los eventos previos a un fallo. Es progreso diferido; cancel() desde ese callback
llega después del análisis y solo afecta futuros usos del token. Se documenta
usar otro hilo para cancelar el análisis en curso. Los errores del callback se
propagan, salvo que exista un error del motor, que tiene prioridad.

Las operaciones usan una barrera catch_unwind/AssertUnwindSafe común que
traduce fallos a InternalEngineError (ENG-E-0012, entities vacías, cause
explícita). PyO3 mantiene además su barrera en el código generado. La prueba
Rust inyecta resume_unwind desde un callback de engine-api y comprueba que la
barrera lo contiene y que el siguiente análisis funciona. No se introduce
panic!, unwrap() ni expect() en la biblioteca.

El arnés unitario del cdylib sigue desactivado: en este entorno hay Python pero
no libpython3.13 para enlazar un intérprete embebido (el intento inicial lo
confirmó). La prueba Rust de integración comparte directamente el módulo de
contención sin enlazar PyO3, y las pruebas Python ejercitan la extensión
instalada. No se añaden hooks de fallo a la API pública.

Python permanece fuera de default-members, y todas sus dependencias son solo
nativas. En wasm32 el crate está vacío, sin engine-api ni PyO3. La comprobación
del DAG confirma que ningún crate del motor depende del binding o de PyO3;
no se cambia el workspace ni los crates del motor.

## Caso real y tolerancias

Fixture propia de engine-api:
`crates/engine-api/fixtures/input/valid/cantilever-linear-static.json`.
El ejemplo Rust `facade_reference` llama directamente a engine_api::run y emite
el documento sin añadir saltos de línea. unittest compara los bytes y SHA-256
de la wheel instalada contra esa referencia en tres ejecuciones. Tolerancia de
transporte: cero; no se reserializan floats en Python.

SHA-256 results/1 (Rust y Python):
`9f0674dd8a18a094b99e16c803eefa89b992085632c379432c221105e1282bb2`.

Comprobación analítica independiente del transporte: voladizo D2 con P=12,
L=4, E=200, I=3. Integrar EI y''=-P(L-x) con desplazamiento y giro nulos en
el empotramiento da y(L)=-PL³/(3EI); equilibrio da R=P y M=PL. Las tolerancias
relativa y absoluta 1e-12 admiten redondeo f64 del pequeño sistema bien
condicionado; no compensan divergencias entre Python y Rust.

Pruebas Python: contrato público y distribución, capacidades/esquemas/catálogo,
todas las fixtures de entrada inválida, diagnósticos con entidades, mecanismo,
avisos y métodos avanzados, entrega diferida/reentrante del callback, errores
de callback y de tipos, token precancelado permanente, cancelación desde otro
hilo con GIL liberado. Esta última usa espacios JSON válidos bajo el límite de
8 MiB para mantener trabajo en el motor mientras el hilo adquiere el GIL;
no altera límites ni introduce sleeps de sincronización.

## Verificación y reproducción

Entorno exportado: PATH=/home/box/.cargo/bin:$PATH,
RUSTUP_TOOLCHAIN=stable (rustc 1.99.0),
CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine,
CARGO_NET_OFFLINE=true y PIP_NO_INDEX=1. CPython 3.13.5/Linux;
maturin 1.15.0. Sin red en construcción, instalación ni pruebas.

- cargo fmt --check: OK.
- cargo clippy --offline --workspace --all-targets -- -D warnings: OK.
- cargo clippy --offline -p mamut-engine-py --all-targets -- -D warnings: OK.
- cargo test --offline --workspace: OK, 546 pruebas, 0 fallos/ignoradas;
  incluye la prueba Rust de contención del binding.
- RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace: OK.
- cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown: OK.
- cargo metadata y cargo tree para WASM: aislamiento verificado.
- maturin build --offline: OK, wheel cp310-abi3-manylinux_2_34_x86_64.
- Instalación pip --no-index en venv nuevo y unittest: OK, 12 pruebas.
- git diff --check y bash clean-room/check.sh origin/main: OK.

Reproducción completa en `bindings/python/README.md`. Comandos principales,
desde la raíz con el entorno anterior:

```bash
mkdir -p /tmp/eng-sfc-007/wheels
cargo run --offline --quiet -p mamut-engine-py --example facade_reference -- \
  crates/engine-api/fixtures/input/valid/cantilever-linear-static.json \
  > /tmp/eng-sfc-007/cantilever-rust.json
/workspace/venv-maturin/bin/maturin build --offline \
  --manifest-path bindings/python/Cargo.toml --out /tmp/eng-sfc-007/wheels \
  --interpreter /workspace/venv-maturin/bin/python
python3 -m venv /tmp/eng-sfc-007/venv-clean
/tmp/eng-sfc-007/venv-clean/bin/python -m pip install --no-index --no-cache-dir \
  /tmp/eng-sfc-007/wheels/*.whl
MAMUT_RUST_REFERENCE=/tmp/eng-sfc-007/cantilever-rust.json \
  /tmp/eng-sfc-007/venv-clean/bin/python -m unittest discover \
  -s bindings/python/tests -v
```

La ejecución de validación se hizo desde /tmp usando la ruta absoluta de tests;
la importación procede de site-packages del venv nuevo. Artefactos, referencia
y logs locales en `/tmp/eng-sfc-007/`. No se verificaron Windows/macOS ni otros
intérpretes. El ABI estable no constituye una matriz de plataformas probada.

## r41 · Integración de main (#117) y smoke de la fachada

- Solicitud J4 r41, MAM-94, 2026-10-15. Merge ya iniciado en
  `j2/eng-sfc-007`: HEAD `9557d8c`, MERGE_HEAD `b0c39f4` (origin/main).
  Se conserva el merge, sin abortarlo ni hacer rebase.
- Fuentes: ambos lados del conflicto del README, scripts y pruebas locales de
  PY-007 (#117), implementación y pruebas de la fachada (#119), manifiestos y
  ejemplo Rust `facade_reference`. Sin red ni código de terceros copiado.
- README integrado: API, EngineError, CancelToken, GIL y progreso al terminar,
  pruebas Python, construcción reproducible, artefactos, matriz y rollback.
  El README usa variables para las rutas locales. El rollback admite la cadena
  de wheels antiguas y el dict de la fachada actual.
- Smoke adaptado a `version()["engine_version"]`; compara también el dict
  nativo completo y los contratos con COMPATIBILITY. `SMOKE.json` conserva la
  versión escalar y registra los contratos. COMPATIBILITY y la propiedad
  `mamut:contract_status` del SBOM declaran la fachada completa
  validate/run/capabilities/schema/explain/version.
- El script genera primero una referencia independiente mediante engine-api y
  exporta `MAMUT_RUST_REFERENCE` al ejecutar las pruebas de la wheel en el venv
  nuevo, con PATH limitado a sus binarios. Las regresiones comprueban el dict,
  versiones/contratos incompatibles, Rust visible y estado de contrato de ambos
  artefactos. Sin cambios numéricos propios de esta integración.

Entorno: Rust 1.99.0, maturin 1.15.0, CPython 3.13.5, Linux x86_64.
`CARGO_TARGET_DIR` exportado en todas las verificaciones; Cargo offline/locked,
`CARGO_NET_OFFLINE=true`, `RUSTUP_AUTO_INSTALL=0` y `PIP_NO_INDEX=1`.
Las herramientas ya estaban instaladas; no se descargaron dependencias.

Reproducción desde la raíz, seleccionando binarios locales y una salida nueva:

```bash
export PATH="$RUST_TOOLCHAIN_BIN:$PATH"
export RUSTUP_TOOLCHAIN=1.99.0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 PIP_NO_INDEX=1
export MATURIN="$MATURIN_BIN"
bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/python-dist-r41"
(cd "$CARGO_TARGET_DIR/python-dist-r41" && sha256sum --check SHA256SUMS)
python3 -m unittest discover -s scripts/tests
cargo fmt --check
cargo clippy --offline --locked --workspace --all-targets -- -D warnings
cargo clippy --offline --locked -p mamut-engine-py --all-targets -- -D warnings
cargo test --offline --locked --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace
cargo check --offline --locked --workspace --all-targets --target wasm32-unknown-unknown
bash clean-room/check.sh origin/main
```

Resultados r41:

- Script completo: OK. Dos builds release con targets independientes, wheel
  idéntica `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`.
  SHA-256: `0841090a11bc0030644bdd95188bcbe19b1b6089f1f96d25ee4095e6a80bc0e2`.
- Portabilidad de todos los miembros ZIP y metadatos, LICENSE/NOTICE sin
  alteraciones, artefactos y comprobación de SHA256SUMS: OK.
- Smoke en venv nuevo, sin rustc/cargo/rustup en PATH: OK; engine/binding 0.1.0,
  contratos engine-input/1 y results/1. Pruebas Python sobre esa wheel: 12 OK,
  incluida igualdad exacta con la referencia Rust, errores y cancelación/GIL.
- `python3 -m unittest discover -s scripts/tests`: 13 OK.
- fmt, ambos clippy con warnings denegados, test del workspace (570 pruebas,
  0 fallos/ignoradas), doc con warnings denegados y check wasm32 con todos los
  targets: OK.
- `git diff --check` y `bash clean-room/check.sh origin/main`: OK.

Evidencia local: `$CARGO_TARGET_DIR/python-dist-r41/` y logs
`$CARGO_TARGET_DIR/{python-wheel,fmt,clippy-workspace,clippy-python,test-workspace,doc,wasm,clean-room}-r41.log`.
No se versionan los artefactos. Sin cambios a licencias, docs/CLEAN_ROOM.md,
.github/**, clean-room/** ni vendor/**. Sin publicación. Otras plataformas e
intérpretes siguen pendientes; revisión clean-room externa J4 pendiente.
Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

## Seguimiento ENG-PY-003 · 2026-10-17

Codex gpt-6.1-sol high añade el transporte privado de canonización y huella
JCS de engine-api para la fachada Python tipada, sin cálculo físico en Python.
La revisión frente al main actual, pruebas de la wheel, compatibilidad de
contratos y declaración clean-room de esta ampliación se registran en
[ENG-PY-003](ENG-PY-003.md). Revisor clean-room: **PENDIENTE (J4)**.
