# Bitácora clean-room · ENG-SFC-001

- Ticket / PR: ENG-SFC-001 (MAM-90), «Bindings WebAssembly»; rama
  `j2/eng-sfc-001`, basada en `origin/main`. Commit local solicitado; sin push.
- Autor(es): agente Codex bajo instrucciones del usuario, 2026-10-15
  (America/Guayaquil).
- Rol: implementador.
- Contrato implementado: adaptación de engine-api (`validate`, `run`,
  `capabilities`, `schema`, `version`, `explain`), `engine-input/1` y
  `results/1`, usando strings JSON; sin redefinir contratos ni matemática.
- Fuentes permitidas usadas: instrucciones del ticket en esta sesión;
  `crates/engine-api/src/{lib,run,json,limits}.rs`, manifest, ejemplos y tests;
  fixtures publicados `crates/engine-api/fixtures/input/{valid,invalid}`;
  `crates/engine-linalg/tests/determinism_guard.rs`, manifests y scripts
  locales de determinismo/clean-room; metadata del registro local de
  wasm-bindgen 0.2.129. No se consultaron repositorios hermanos ni fuentes
  de otros motores. Sin red, fetch ni descargas.
- Derivaciones propias: ninguna matemática. Fachada, protocolo de worker y
  arneses de paridad escritos para este ticket. No se añaden trascendentes;
  toda la matemática sigue en engine-api y sus dependencias, incluida
  engine_core::math.
- Código de terceros copiado: ninguno. Dependencias del registro local
  referenciadas por Cargo; glue de wasm-bindgen generado únicamente en pkg
  ignorado. No se incorporan binarios ni artefactos generados al commit.
- Validación: comparación byte a byte con engine-api nativo de todos los
  fixtures V1 publicados (5 válidos y 9 inválidos), no un oráculo externo.
  El path real es `engine-api/fixtures/input`; no existe `src/fixtures/input`.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Implementación y límites

`crates/engine-wasm` contiene una fachada wasm-bindgen fina, compilable
nativamente. `run` devuelve el documento canónico de engine-api sin
reescritura. Validación y callbacks usan serialización de tipos de
engine-api y su normalizador canónico. Los errores conservan `ApiError`
como string JSON; errores lanzados por callbacks se propagan después de
cancelar. `schema` conserva el documento publicado y `explain` devuelve un
literal string JSON. No se usa serde-wasm-bindgen porque el transporte es
texto JSON, sin mapear los contratos a objetos JS.

La biblioteca mantiene `#![forbid(unsafe_code)]`, sin panic, unwrap ni
expect en su código fuente. wasm-bindgen 0.2.129 compila con esa política;
no fue necesario modificar ni usar la lista
`UNSAFE_LIBRARY_EXCEPTIONS` (actualmente vacía) de la guardia de
determinismo. La suite del workspace incluye esa guardia y pasa.

`js/worker.js` es módulo ES; `js/main.js` envía requests y recibe strings de
progreso/resultados mediante postMessage. Con aislamiento de origen, el
cliente comparte un flag JS de 4 bytes, consultado en callbacks, y llama al
CancelToken original de engine-api. El mensaje cancel también se publica,
pero queda en cola mientras Rust ejecuta de forma síncrona: **no se atribuye
cancelación activa a ese mensaje**. Los pasos de engine-api son
indivisibles, incluida la factorización; cancelación entre pasos/modos/
iteraciones, sin promesa de latencia máxima. Si la señal llega después de
terminar, puede conservarse el resultado completo. Sin SharedArrayBuffer,
se termina el worker y se rechaza con AbortError; no se inventa un
results/1 cancelado. El cliente puede crear otro worker en el próximo
request. Ejemplo HTML, servidor loopback con COOP/COEP y arnés de navegador
incluidos; sin dependencias npm.

## Cambios de Cargo

- Nuevo miembro `crates/engine-wasm` en `members` y `default-members` porque
  compila nativo. Tipos `cdylib` y `rlib`; no se modifica el perfil release.
- Dependencias directas: engine-api local; serde `=1.0.229` y serde_json
  `=1.0.154` ya presentes; wasm-bindgen **`=0.2.129`** y js-sys **`=0.3.106`**.
- Cargo.lock añade engine-wasm y 11 paquetes externos resueltos offline:
  wasm-bindgen y sus tres crates macro/shared, js-sys, bumpalo,
  futures-core/task/util, pin-project-lite y slab. No se actualizan versiones
  existentes. CLI 0.2.129 comprobada por el script antes de compilar.

## Tamaños medidos

`scripts/build-wasm.sh` genera `pkg/size-report.md`. Mediciones release con
rustc **1.99.0 (b940084d7 2026-09-28)**, wasm-bindgen CLI **0.2.129**, target
`wasm32-unknown-unknown`; gzip **`-9 -n`** (sin nombre ni timestamp). Sin
wasm-opt, LTO adicional ni cambios al perfil del workspace.

| Artefacto | Bytes crudos | Bytes gzip -9 | Frente a ≤ 1.000.000 bytes gzip propuestos |
| --- | ---: | ---: | --- |
| Cargo release, antes de wasm-bindgen | 1.908.323 | 496.172 | PASS |
| wasm-bindgen `--target web`, engine_wasm_bg.wasm | 1.455.396 | 383.805 | PASS |
| wasm-bindgen `--target nodejs`, engine_wasm_bg.wasm | 1.455.396 | 383.805 | PASS |

**Presupuesto de ≤ 1 MB gzip: PROPUESTO, PENDIENTE de confirmación.**
Se usa la interpretación decimal más estricta (1.000.000 bytes); estas
mediciones quedan también por debajo de 1 MiB. Son tamaños del .wasm;
el glue JS web suma 13.533 bytes crudos, y el cliente/worker son adicionales.
No se presentan como tamaño total de aplicación. SHA-256 del wasm web:
`b255bdbc4b6328a89fc5fd937d1c316ad489c360bf525ac1f913507ce15f6d36`.

## Verificación local

Todos los comandos Cargo usan
`export CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine` y
`PATH=/home/box/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin:$PATH`.
Compilación, pruebas y docs con `--offline --locked`; no hubo red.
Node **v22.19.0**, binario `/home/box/.local/bin/node`; CLI
`/workspace/tools-bin/wbg/bin/wasm-bindgen`.

| Comando / evidencia | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --offline --locked --workspace --no-fail-fast` | PASS: 545 tests/doctests; cero fallos |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace` | PASS |
| `cargo check --offline --locked --workspace --target wasm32-unknown-unknown` | PASS |
| `bash clean-room/check.sh origin/main` | PASS sobre cambios añadidos al índice |
| `bash scripts/build-wasm.sh` | PASS: build release, CLI web+nodejs, oráculo nativo, suite Node y reporte |
| Node, target nodejs | PASS: 14 fixtures, results/1 idéntico byte a byte, progreso, validación, metadatos, errores y cancelación |
| Node, target web ES module | PASS: los mismos 14 fixtures y pruebas directas |
| worker_threads, adaptador exclusivo de pruebas | PASS: worker.js y main.js reales, 14 fixtures, todas las operaciones, progreso, cancelación activa con flag compartido, fallback terminate y reutilización |
| `git diff --check` y sintaxis JS/Bash/Python | PASS |
| Chrome headless | **PENDIENTE**: instalado, pero el sandbox impide sockets y arranque |
| Firefox | **PENDIENTE**: no se ejecutó motor de navegador |
| Safari | **PENDIENTE**: no se ejecutó motor de navegador |

Se intentó arrancar el servidor offline con `python3 scripts/serve-wasm.py`:
`PermissionError: [Errno 1] Operation not permitted` al crear el socket.
Chrome con perfil en `/tmp/eng-sfc-chrome`, `--headless --no-sandbox
--disable-gpu --disable-dev-shm-usage --dump-dom about:blank` terminó con
salida 133 y `setsockopt: Operation not permitted`. No se pidieron permisos
ni se intentó ejecutar fuera del sandbox. El arnés
`crates/engine-wasm/js/test-browser.html` queda listo para ejecutar en
Chrome/Firefox/Safari con servidor local autorizado; publica PASS/FAIL.
La ejecución del adaptador Node **no certifica navegadores**. La aceptación
multinavegador permanece pendiente, junto con la confirmación del presupuesto.

No se modifican `docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` ni
`vendor/**`. Sin subagentes, push ni artefactos .wasm/pkg comiteados.
