# ENG-SFC-014 · MAM-865 · Timeout y señales del CLI

Fecha: **2026-10-21**. Base: `origin/main`; worktree
`/workspace/mamut-engine-sfc14`. Complementa #174 H1. Sin red ni fuentes de
otros motores; no se consultaron los materiales restringidos del ticket.

## Implementación y decisiones

- `timeout::parse_timeout_args`, en un módulo propio de `engine-cli`, retira
  exclusivamente `--timeout <s>` y deja el resto al parser SDK. La llamada en
  `lib.rs` es pequeña para minimizar conflictos con el preparseo de #197.
  Segundos finitos >0, fraccionarios/científicos; valor inválido, omitido o flag
  duplicado: 2. El flag prevalece sobre `MAMUT_ENGINE_TIMEOUT`, incluso inválida.
  Positivos menores que la resolución del reloj vencen inmediatamente;
  valores finitos fuera del rango de `Instant` se esperan por tramos acotados.
- Un hilo std espera un deadline monotónico y marca el `Arc<CancelToken>`
  existente. `CancelFlag` del SDK no es `Send`; permanece en el hilo llamador
  y `Control::sync` sincroniza ambas fronteras. Un canal despierta el temporizador
  al terminar y `Drop` recoge el hilo. Sin timeout no se crea hilo ni cambia la
  salida. El reloj nunca interviene en aritmética, progreso ni manifiestos.
- El SDK vendorizado ya publica `FailureKind::Timeout` y código 6. El adaptador
  traduce la cancelación causada por el vencimiento a ese fallo: stdout vacío;
  stderr termina en el sobre
  cli-run/1 con `exitCode: 6`, `outcome: "timeout"` y mensaje sanitizado.
  Se omiten documentos parciales; una cancelación previamente observada
  conserva el código 5. No se modifica vendor. Se conserva el metadato histórico
  de categorías reservadas de `describe` para no alterar bytes sin timeout.
- Ronda 2, precedencia terminal: `timeout::resolve` convierte únicamente
  `Cancelled` en `Timeout` cuando venció el temporizador. Un despacho ya
  completado con éxito/warning o un error distinto de cancelación conserva su
  resultado, incluso si el plazo expira antes de la comprobación posterior.
  El plazo sigue siendo cooperativo durante E/S: su mera expiración no invalida
  un resultado terminal. Una prueba unitaria determinista inyecta el estado
  vencido y comprueba éxito, errores terminales y ambas clases de cancelación.
- Ronda 2, NDJSON: `EventWriter::write_fmt` materializa cada evento SDK emitido
  con `writeln!`, incluido el salto de línea, antes de pasarlo a stderr con
  `write_all`. El sobre de error usa el mismo adaptador. La atomicidad es del
  buffer de cada línea; los fallos reales de E/S conservan el código 70.
  El temporizador solo marca el token y no escribe en stderr.
- Revisión de bucles: Newton consulta en validación de factores, pasos,
  iteraciones, evaluaciones de elementos y antes de aceptar estado, con rollback
  del estado de prueba ante cancelación. Estático, P-Delta y modal pasan el
  control a factorización/solve y eigensolver; espectro lo pasa a modal y CQC.
  Recuperación y combinaciones consultan entre miembros/nodos/términos.
  Consolidación conserva la consulta por subpaso y bloques LDLᵀ de SFC-010;
  añade consultas en validación, ensamblaje, salidas, integración y sustitución.
  La referencia consulta en productos/solve densos, Cholesky, Jacobi,
  normalización, inercia Sturm y condensación/recuperación Guyan. Las variantes
  cooperativas son aditivas; los callbacks de progreso conservan sus eventos.
  Ninguna expresión numérica, reducción, pivote, tolerancia o resultado de una
  ejecución completada se modifica. Véase `docs/DETERMINISMO.md`.
- La guarda de determinismo conserva la prohibición de concurrencia en fuentes
  numéricas y permite únicamente el token `thread` en la ruta exacta
  `crates/engine-cli/src/timeout.rs`. Una regresión comprueba el alcance mínimo:
  otras rutas, colecciones hash y Rayon siguen prohibidos. La excepción solo
  permite el temporizador expresamente solicitado, sin cálculo paralelo.
- La guarda `val032_runs` exige hashes de fuentes actuales: se sincronizan solo
  los dos SHA-256 de `pdelta.rs` y `modal.rs` en el índice de validación. No se
  regeneran fixtures, referencias, modelos ni tolerancias.

## C: señales bloqueadas por la política

**SIGINT/SIGTERM no implementadas.** `std` no proporciona una API de registro
de manejadores para estas señales. Están prohibidos cambios en Cargo.toml/lock
y los crates mantienen `forbid(unsafe_code)`; instalar handlers mediante FFI
propia incumpliría la política. Ctrl-C y SIGTERM conservan el comportamiento
del SO, sin garantía de salida cooperativa 5.

**Decisión pendiente de Jonathan:** autorizar una dependencia `signal-hook`
en `engine-cli`, con registro y puente de señales hacia el token fuera del
manejador; alternativamente `libc`, que requeriría resolver también la política
de unsafe/FFI. No se solicitan excepciones implícitas ni se añade dependencia
en este ticket. El mecanismo disponible de cancelación externa sigue siendo
`MAMUT_ENGINE_CANCEL_FILE` (5).

El timeout es cooperativo, no un terminador forzoso: las lecturas bloqueantes
de stdin/archivos y la E/S del SDK no se interrumpen. Los backends ajenos que
no implementen los nuevos hooks conservan sus kernels indivisibles. El sobre
terminal corresponde a `run`; MCP conserva sus frames y semántica SDK.

## Pruebas y reproducción

Pruebas de binario en `crates/engine-cli/tests/cli.rs`:

- Consolidación derivada del fixture `consolidation` de
  `crates/engine-api/tests/support/sfc010.rs`: cuatro capas, 80 elementos,
  64 intervalos alternos de 1/1,25 s, 72000 subpasos y dos salidas. Coste
  declarado `64·80³ + 72000·80² = 493568000`, inferior a 500000000; también
  cumple cotas de intervalos, subpasos y trabajo por elementos. La corrida
  completa sin timeout verifica entrada válida y código 0; medición local
  inicial: **8,48 s** en debug antes de añadir controles a la sustitución.
- Flag y variable con timeout **0,15 s**: código 6, stdout vacío y sobre timeout;
  el proceso debe finalizar en **<1,15 s** desde spawn. Un watchdog mata y recoge
  el hijo si hay regresión, para no bloquear la suite.
- Valores inválidos, flag duplicado/sin valor, precedencia sobre variable
  inválida o diminuta, valores finitos enormes y timeout amplio de 60 s.
  Comparación directa de bytes stdout/stderr y salida rápida sin esperar 60 s.
- Se conserva la prueba existente del cancel file antes/después de spawn: 5.
- NDJSON + timeout por flag y variable: plazos de 1 ns, 0,1 ms, 1 ms y 150 ms
  sobre la consolidación pesada. Todas las líneas de stderr se parsean como
  JSON válido, terminan en salto de línea y la última es el sobre timeout 6
  sin documentos parciales; se exige observar también progreso `started`.
  Una prueba unitaria con escritor estricto comprueba determinísticamente que
  cada evento y el sobre llegan como una línea JSON completa, sin fragmentos.

Regresiones de dominio: cancelación dentro de una corrección Newton sin perder
el último paso aceptado; interrupción en cada checkpoint del camino modal de
referencia, igualdad de bits de modos/vectores; consolidación con control sin
cancelar frente al backend sin caché conserva todos los bits.

Entorno de verificación, toolchain del repositorio Rust 1.99.0:

```sh
export PATH=$HOME/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/workspace/.cargo-target/sfc14
cargo fmt --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
bash clean-room/check.sh origin/main
```

Resultados finales, repetidos en ronda 2: **las cuatro verificaciones pasan**.
`cargo fmt --check`
sin diferencias; Clippy de todo el workspace/all-targets sin advertencias;
`cargo test --workspace --offline`: **758 pruebas pasan, 0 fallos, 0 ignoradas**
en 115 suites (incluidas suites vacías y doctests). La suite de integración CLI
tiene 19 pruebas y la unitaria, 6. La ronda 2 añade tres regresiones: precedencia
terminal determinista, escritura de líneas completas y binario NDJSON + timeout.
`git diff --check` también pasa.

`clean-room/check.sh` solo inspecciona archivos conocidos por Git. Para incluir
los dos archivos nuevos sin modificar el índice real ni escribir en `.git`,
se ejecutó el mismo comando con un índice y almacén de objetos temporales:

```sh
mam865_git_objects=$(git rev-parse --path-format=absolute --git-path objects)
export GIT_INDEX_FILE=/tmp/mam865-verification.index
export GIT_OBJECT_DIRECTORY=/tmp/mam865-verification.objects
export GIT_ALTERNATE_OBJECT_DIRECTORIES="$mam865_git_objects"
mkdir -p "$GIT_OBJECT_DIRECTORY"
git read-tree HEAD
git -c core.hooksPath=/dev/null add -N -- \
  crates/engine-cli/src/timeout.rs provenance/ENG-SFC-014.md
bash clean-room/check.sh origin/main
```

Resultado: **Clean-room: OK**, incluyendo ambos archivos nuevos. El entorno
temporal se limita a ese comando; el índice real permanece sin cambios.

No se modifican Cargo.toml/lock, vendor, CI, políticas clean-room, barra axial
ni truss. No se crea commit.

## Señales SIGINT/SIGTERM (decisión #203, PR #214) — 2026-10-21

Implementación propia sobre la API pública y segura de `signal-hook =0.4.5`
(`flag::register`, `low_level::unregister`; MIT OR Apache-2.0, crates.io).
Dependencia solo `cfg(unix)` con `default-features = false`; transitivas nuevas en
Cargo.lock: signal-hook-registry 1.4.8, errno 0.3.14, windows-sys 0.61.2 y
windows-link 0.2.1 (las dos últimas solo en Windows vía errno). Sin `unsafe`
propio: `#![forbid(unsafe_code)]` se mantiene en lib.rs y main.rs.

`crates/engine-cli/src/signals.rs` registra un `Arc<AtomicBool>` por señal; un
hilo `std` lo consulta cada 5 ms y cancela el mismo `CancelToken` del plazo, de
modo que `run` termina con código 5, stdout vacío y sobre `outcome: "cancelled"`.
La guardia de determinismo admite `thread` solo en las rutas exactas
`timeout.rs` y `signals.rs` del CLI. No se consultó código de terceros ajeno a la
documentación pública de signal-hook.

Limitaciones documentadas en el README: señal previa a la instalación de los
manejadores → disposición por defecto (sin código 5); una SIGINT ignorada por el
padre queda sustituida por el manejador.

Pruebas: `sigint_cancels_heavy_consolidation_with_five`,
`sigterm_cancels_heavy_consolidation_with_five` y
`cli_cancellation_exceptions_only_allow_threads_in_their_exact_sources`.
