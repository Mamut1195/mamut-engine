# Bitácora clean-room · ENG-VAL-006

- Ticket / PR: ENG-VAL-006, lado Engine del contraste de desempeño. El traductor
  hacia el otro motor y cualquier archivo de ese motor quedan fuera de este
  repositorio, en el repo hermano, a cargo de Q2.
- Autor(es): Cursor Grok 4.7 · 2026-10-20 y 2026-10-21 (America/Guayaquil).
- Rol: especificador e implementador del esquema, del generador, del traductor
  a `engine-input/1`, del índice y del orquestador de medidas del Engine.
- Contrato implementado: `bench-model/1` y `bench-index/1` (JSON Schema
  draft 2020-12). La entrada del motor sigue siendo `engine-input/1`.
- Fuentes permitidas usadas:
  1. ACI 318-19, §19.2.2.1.a: E = 4700·√(f'c) MPa, con f'c en MPa. Aquí
     f'c = 28 MPa. ν = 0.20 y G = E / (2 (1 + ν)) son constantes de este
     modelo, no un artículo de la norma.
  2. NEC-SE-DS 2015, forma del espectro elástico en g: T0 = 0.10·Fs·Fd/Fa,
     Tc = 0.55·Fs·Fd/Fa, TL = 2.4·Fd; rama 0..T0 lineal hasta la meseta
     η·Z·Fa; T0..Tc meseta; Tc..TL meseta·(Tc/T)^r; T > TL
     meseta·(Tc/TL)^r·(TL/T)². Coeficientes guardados, no recalculados por
     el motor: zona V, Z = 0.40, suelo D, Fa = 1.20, Fd = 1.19, Fs = 1.02,
     η = 1.80 (Sierra), R = 8, r = 1, ζ = 0.05. Sa publicada =
     (ordenada elástica en g) / R · g, con g = 9.80665 m/s². No se aplican
     φP ni φE. Los dos motores deben leer la tabla versionada; si un
     coeficiente de una edición posterior difiere, la tabla de este commit
     sigue siendo la común.
  3. Aproximación de Saint-Venant para rectángulo:
     J = a·b³·(1/3 − 0.21·(b/a)·(1 − (b/a)⁴/12)), con a el lado mayor.
     Implementada en `saint_venant_j`. Página de monografía no reabierta
     en esta sesión.
  4. R. J. Hyndman y Y. Fan, «Sample quantiles in statistical packages»,
     The American Statistician 50(4), 1996: cuantil tipo 6, h = (n + 1)·p.
     Mediana, Q1, Q3, IQR, mínimo y máximo salen solo de las filas
     `phase=measure`.
- Derivaciones propias: reparto nodal de `{m, Izz}` documentado en el
  esquema (maestro Ux = Uy = m, Rz = Izz; Uz y las inercias Rx, Ry se
  reparten para que la matriz de masa sea definida positiva en los grados
  libres del diafragma). Izz de la losa uniforme respecto del maestro,
  no del centroide: m·(Lx² + Ly²)/12 + m·d². Carga lateral = 0.10·m·g
  en el eje del espectro, coeficiente fijo de este modelo.
- Código de terceros copiado: ninguno. Sin crates nuevos en la revisión
  de licencias: `engine-bench` usa `engine-api`, `engine-core`, `mamut-canon`,
  `serde` 1.0.229 y `serde_json` 1.0.154, ya presentes en el workspace.
  `sqrt`, `powi`, `powf` y `hypot` del generador pasan por `engine_core::math`.
- Validación: `cargo test -p engine-bench` (17 pruebas Rust; una de ellas
  ejecuta las 7 pruebas Python de `scripts/eng-val-006/test_orchestrator.py`).
  La prueba del pórtico de 3 pisos corre estático, modal y espectro y
  lee periodo, Ux estático de techo y Ux CQC por los punteros del índice.
  Los pórticos de 8 y 15 pisos se validan contra `engine-input/1` sin
  resolver el modal denso en la prueba. `cargo clippy -p engine-bench
  --all-targets -- -D warnings` y `cargo check -p engine-bench --target
  wasm32-unknown-unknown` en verde en esta sesión. El orquestador no se
  lanzó contra el VPS. Una pasada local mínima de `rc3d-3s` estático
  (un calentamiento y una medida, un hilo) escribió manifiesto, CSV,
  tabla y SVG bajo `target/`, que no se versiona. El código de salida
  fue 1, el aviso de diafragma que el protocolo acepta. Esta imagen no
  trae `/usr/bin/time`; la pasada usó GNU time 1.9 por `--time-bin` y
  el valor por defecto del orquestador sigue siendo `/usr/bin/time`.
  No se publican tiempos.
- Comparación con oráculo (repo hermano, informativa): no. No se abrió
  ese repositorio ni se generó ningún archivo de su lenguaje.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, Frame3DD, LAPACK ni derivados para este ticket."
- Revisor clean-room: J4 · 2026-10-21 (America/Guayaquil) · APTO sobre 6068f7c90c64f50877132481d34105ad980ffe74. Fuentes declaradas suficientes; sin código de oráculos (OpenSees, CalculiX, Frame3DD, LAPACK) ni de terceros; la descripción de la guarda escalar (opción A, math::powi = libm::pow, ±1 ulp en Iz y J de la viga, 12 sha256 nuevos) coincide con el diff 3567d8e..6068f7c. El merge requiere el sí explícito de Jonathan.

## Alcance

`crates/engine-bench` publica el JSON Schema `schemas/bench-model-1.json`
(`$id` `urn:mamut:bench-model:1`) y tres pórticos deterministas:

| id | nudos | barras | sha256 del bench-model |
| --- | ---: | ---: | --- |
| rc3d-3s | 64 | 120 | `199cfb84d248877d70caa6e1df641582386a0dabfe725d8ec659007d4552d7b3` |
| rc3d-8s | 144 | 320 | `33fa7da8e4af10edf89adf4d19737eab72391618c297a5952cfb059cf5efe960` |
| rc3d-15s | 256 | 600 | `b914e418b36a9ce8085ad31f7caecf1d512946215577ea961ccb0071e998a5b9` |

El límite de `engine-input/1` es 256 nudos y 1024 barras. El de 15 pisos
ocupa el máximo de nudos. Un piso más lo superaría y el traductor lo rechaza.

`orientation.kind` es `local_z_vector`: vector global de referencia del
eje local z. No hay ángulo. Columnas `[1, 0, 0]`; vigas `[0, 0, 1]`.
La dirección espectral es un eje global; en los oficiales, X = `[1, 0, 0]`.
La masa de piso es `mass {m, Izz}` en kg y kg·m². `Sa.unit` es `m/s^2`.

El índice `fixtures/index.json` (`bench-index/1`) lista `models[]` con
`id`, ruta y sha256 del bench-model, `engine_input.{static,modal,spectrum}`
con ruta y sha256 de la entrada ya traducida, `roof_node` y punteros
JSON de periodo, desplazamiento estático de techo y desplazamiento CQC
de techo. El sha256 es de los bytes UTF-8 del archivo con LF final y sin
BOM, en hexadecimal minúsculas. No es `results/1.input_sha256`. Las rutas
usan `/`. `.gitattributes` fuerza LF en esos JSON para que un checkout
en Windows no cambie el hash.

`scripts/eng-val-006/orchestrate.py` prepara la campaña del binario
release (3 calentamientos, 15 medidas, 1 hilo y N hilos, `taskset`,
variables BLAS, `/usr/bin/time -v`, runners ociosos antes y después de
cada bloque). Las tablas y los SVG salen solo de los CSV. No hay workflow
de esta campaña en `.github`.

## Ampliación aditiva

Tras el contraste de J2 en VAL-032 (PR #180, `8055c02`), `bench-model/1`
acepta, sin tocar los bytes de los `rc3d-*`:

- `nodal_masses`, que sustituye al reparto tributario cuando la clave está.
- `modal.n` en 1..=16384 y combinación modal `cqc` o `srss`.
- Varias direcciones espectrales, sus factores, y `spectrum.variants`.
- `p_delta` con las seis opciones de `engine-input/1`. El método existe y
  está estable; no se inventan tolerancia ni iteraciones.
- Patrones con nombre y `static_combinations` con factor con signo.
- Diafragma vacío, masa nula y barras `truss` (`truss3d.linear`, solo E, A y rho).

Los ids siguen siendo contiguos desde 1: un remapeo de VAL-032 necesita esa
biyección. Desde ENG-LA-009 el modal condensa con Guyan las filas de masa
exactamente nulas; el traductor sigue sin añadir masa. `ENG-E-0006` queda
para masa negativa o ausencia de masa libre, y `ENG-E-0014` para un `Kss`
singular. Un esclavo de diafragma no admite apoyo (`ENG-E-0013`).

## Guarda escalar

J4 aprobó la opción (A): `generate.rs` llama `engine_core::math::{sqrt, powi, powf, hypot}`
y `engine-bench` no queda fuera de la guarda. No se añadió otra implementación.
`math::powi` es `libm::pow`. En la viga, `0.30.powi(3)` pasa de
`0x3f9ba5e353f7ced9` (std) a `0x3f9ba5e353f7ced8` (libm), y `(0.30/0.70).powi(4)`
de `0x3fa145d66ad5e25e` a `0x3fa145d66ad5e25d`. El JSON de la viga cambia
`Iz` de `0.001575` a `0.0015749999999999998` y `J` de `0.004603782069970845`
a `0.004603782069970844`. El resto de los oficiales, incluido el maestro
`(1, 1)` elegido con `math::hypot`, queda igual. Los doce archivos y
`fixtures/index.json` se regeneraron con esos hashes. `ORIENTATION_KIND`
sigue siendo `pub(crate)`. El orquestador sigue reteniendo stdout en memoria.
