# Changelog

## Unreleased

## 0.2.0 - por fijar al etiquetar

### Añadido

- MAM-4: Issue 4: uniaxial law `concrete.kent_park`, the monotonic Kent–Park
  compression envelope from the published equations (A.9)–(A.15).
  This fragment does not close MAM-107 and does not approve a time or
  memory budget.

- MAM-5: Issue 5: rectangular fiber section `rectangle.fiber_bending` and a
  moment-curvature point at constant axial force. Concrete is
  `concrete.kent_park` and steel is `steel.bilinear_kinematic`.
  Refs #5. This fragment does not close MAM-107 and does not approve a
  time or memory budget.

- MAM-6: barra experimental `frame2d.fiber_stations`. Integra
  `rectangle.fiber_bending` en tres estaciones a lo largo de la luz para
  un análisis estático no lineal de material. No sustituye a
  `frame2d.euler_elastic`. No está validada por comparación externa.
  Refs #6. Este fragmento no cierra MAM-107 y no aprueba un presupuesto
  de tiempo o memoria.

- MAM-11: add library-level linear buckling of K + λ Kg and name -Kg in its diagnostics.

- MAM-49: add library-level pile-axis discretization with tributary lengths.
  - MAM-49: add a semi-infinite linear Winkler lateral analytical solution with
  free and fixed rotational head conditions and analytical tests.
  - MAM-49: add uniform pile-axis meshes and explicit dimensional conversion of
  p-y, t-z, and q-z tables to nodal spring resistance tables.
  - MAM-49: add a finite planar pile assembly and Newton solve candidate with a
  linear frame, independent nonlinear monotonic nodal soil springs, accepted
  load-settlement points, and depth profiles. Softening is rejected, and
  absolute load factors scale the head loads together.

- MAM-60: especificación ENG-INT-001 del adaptador para `StructuralSolverPort`
  de Structures (`docs/integration/ENG-INT-001-structural-solver-port.md`):
  mapeo contra `engine-input/1`, `results/1`, capacidades, estados y códigos de
  salida de la CLI. Solo documentación; la implementación va al backlog de
  Structures.

- MAM-84: Add a Spanish Python user guide, a community method procedure, and reproducible cantilever notebooks with an offline runner.

- MAM-86: add a shared analytical conformance matrix for Rust API, CLI, WASM facade, and Python bindings with numerical, unit, sign, SHA-256, and typed error parity checks.

- MAM-92: Contrato CLI experimental con Harness: ficha Windows, schema de referencia y pruebas de stdin, códigos de salida, hash RFC 8785 y determinismo; kit SDK-WF-009 pendiente.

- MAM-96: añade el paquete ES privado experimental `mamut-engine-wasm`
  (ENG-SFC-005), una prueba Node de metadatos y artefactos sin build y el aviso
  AGPL-3.0-only con enlace al código fuente correspondiente en la página mínima.

- MAM-107: add candidate Windows native and Node/WASM performance and
  repeatability campaign executors for the indexed Engine models.

- MAM-836: Add a shared English, Spanish and Portuguese catalog for stable codes, explicit language selection in CLI (`--lang`), Python and WASM (`lang`), and workflow fingerprints that exclude diagnostic message text.

- MAM-843: traduce al inglés los comentarios, docstrings y mensajes de uso y
  diagnóstico de `scripts/` y de las pruebas de `bindings/python/tests`;
  conserva los literales comparados por pruebas y contratos (tramo 3).

- MAM-861 / ENG-REL-003: fragmentos de changelog en `changelog.d/`,
  ensamblado determinista en release (`scripts/changelog-assemble.sh`) y
  validación de PR (`scripts/changelog-check.sh`); flujo en CONTRIBUTING.md.

- MAM-863 / ENG-SFC-012: `validate --input <ruta|->` valida el modelo sin
  resolver y emite directamente validate-report/1 determinista (0/1/2),
  sustituyendo el sobre cli-run/1 de la abreviatura de #197. Esquema publicado
  y registrado, diagnósticos ENG-* para rechazos de contrato/límites,
  pruebas del binario y del esquema, y benchmark E1 con umbral ≥10×.
  Se conservan run, transporte y validación explícita de workflows.
  Publicación local de validate-report/1 mediante SCHEMAS, con bytes e índice SHA-256 en schemas/.

- MAM-864: añade autodescripción determinista `cli-describe/1` mediante
  `mamut-engine describe --json` y `--help --json`, consultas de versión,
  capacidades, esquemas y explicaciones de la API, con contratos publicados,
  inventario SHA-256 y pruebas del binario. Conserva el describe histórico y
  los bytes de ejecución existentes.

- MAM-865 / ENG-SFC-014: timeout cooperativo del CLI mediante `--timeout <s>`
  o `MAMUT_ENGINE_TIMEOUT` (el flag gana), validación de segundos finitos >0
  y ≤31536000 (1 año, decisión #204); valores superiores dan uso inválido (2).
  Se respetan los valores de los flags de la base al extraer `--timeout` (B3).
  Al vencer el plazo, código 6 y sobre distinguible en stderr con stdout vacío.
  Temporizador std aislado del parser SDK; controles dentro de Newton, modal,
  P-Delta, estático,
  recuperación, combinaciones y consolidación, sin cambiar el orden numérico.
  Pruebas del binario con consolidación pesada, límite timeout + 1 s,
  cancel file y salida byte-idéntica.

- MAM-866: `--progress ndjson` añade eventos de etapa `progress/1` en stderr,
  con índice del paso y los campos reales de engine-api, esquema publicado,
  líneas completas y salida determinista; conserva los eventos del SDK y stdout.
  Integrado con entrada `--input`, validate-report/1, timeout y señales de main;
  reutiliza EventWriter y publica progress/1 mediante additional_schemas e índice
  versionado, sin exponer el transporte CLI desde engine-api.

- MAM-867 / ENG-CONT-001: publish the terminal `cli-error/1` stderr contract,
  its schema and a versioned registry of 32 codes with frozen definitions,
  stable message keys and English base messages. Add compatibility and emitter
  inventory guards to `cargo test`, validating binary error envelopes. Integrate
  input transport, usage, cancellation and timeout failures under the same
  contract while preserving exit codes, stdout/results/1 and direct validation's
  validate-report/1 on stdout with empty stderr for ok/warning/invalid (0/1/2).
  Publish the CLI schema in the versioned schemas tree and
  document the vendored SDK's MCP frame exception.
  Integrate Unix SIGINT/SIGTERM cancellation with `cli-error/1` and
  `CLI-E-CANCELLED` (exit 5, empty stdout), preserving external cancellation's
  failed-document policy. Guard all ENG/API/CLI/WF severity families and reject
  unregistered literal fragments used in macro construction and fragment joins.

- MAM-868: publica el contrato de éxito/aviso cli-run/1 con engineResults,
  esquema JSON y política semver V1. Añade validación del stdout real y golden
  mediante manifiesto versionado de longitud y SHA-256 de bytes con LF para
  entradas válidas, determinismo, SFC-010 y engine_input de E1, con regeneración
  local explícita prohibida en CI. Las huellas del SDK son objetos opacos.

- MAM-869: añade esquemas versionados con índice de URN, URLs, SHA-256 y tamaños,
  generación local verificable y TAR reproducible para preparar el release.

- MAM-870: guía «CLI para agentes» (`docs/GUIA_CLI_AGENTES.md`) con recetas de
  tuberías, tabla de códigos de salida, manejo de errores y límites, más un
  modelo de ejemplo de pórtico de dos pisos. `cargo test -p engine-cli --test
  agent_guide` ejecuta los ejemplos con el binario real y contrasta las tablas
  con `describe --json` y el registro de códigos de error.

- MAM-938: añade la dependencia TOML fijada para engine-cli y la configuración de dependencias vendorizadas, con inventario verificable, pruebas de alteración en copias temporales y una puerta de validación sin red para Linux. La generación de dependencias y las evidencias de ejecución quedan pendientes de integración.

- MAM-940: convert the mamut-toml/1 structural core to engine-input/1 and back.

- MAM-941: convert masses, loads, cases, combinations, prescribed displacements, diaphragms, and linear springs.

- MAM-943: convert the specified analysis branches of mamut-toml/1, including experimental methods only when allow_experimental is set.

- MAM-972: the existing linear bar reports the fixed-end axial force for a
  uniform temperature change, tension-positive N = -(E·A·α·ΔT), and the
  equivalent joint forces. Element::recover includes that same axial force,
  including at zero displacement. A thermal gradient is not implemented.

- MAM-993: accept an optional name on nodes, members and springs. The integer id stays required. The name stays out of engine JSON and export writes it back only with an idmap/1 document.

- MAM-1032: casos analíticos de `rectangle.fiber_bending` con historia de
  acero distinta de cero y con flexión en la que el hormigón traccionado
  queda sin tensión. La sección no cambia. Este fragmento no cierra
  MAM-107 y no aprueba un presupuesto de tiempo o memoria.

### Cambiado

- MAM-6: la curvatura de `frame2d.fiber_stations` es κz = v″, el
  convenio de `ElasticFrame`. La recuperación admite como máximo 64
  estaciones. Sigue experimental y no está validada por comparación
  externa. Refs #6. Este fragmento no cierra MAM-107 y no aprueba un
  presupuesto de tiempo o memoria.

- MAM-11: publish buckling.linear in run, capabilities, the Python facade, and the mamut-toml/1 converter. The factor remains the lowest finite eigenvalue of K + λ Kg = 0. The Timoshenko–Gere page stays unknown.

- MAM-50: add COPYRIGHT naming MAMPRO SAS as holder and align holder and commercial license contact in the Python bindings NOTICE and README, CLA, CONTRIBUTING and ADR-001.

- Documentación de Python (MAM-84) y del puerto StructuralSolverPort (MAM-60): pDelta figura como «Stable; ENG-BENCH-002; Validated: PENDIENTE (sin ticket asignado)», sin ligarlo a ENG-VAL-006, que solo cubre nonlinearStatic, soilProfile y consolidation1d; se documenta que MAMUT_ENGINE_TIMEOUT aplica aunque se omita --timeout.

- MAM-86: Harden the parity matrix with pointer-derived SI units, a tolerance factor cap of 64, and exact input and check counts.
  Require unique check pointers and the exact ordered pointer list for each parity case in Rust and Python, deriving check counts from those lists.

- MAM-107: una llamada repetida de linearStatic puede devolver los desplazamientos
  nodales de un modelo ya preparado, sin reconstruir el documento results/1.
  No aprueba presupuestos de tiempo o memoria y no cierra MAM-107.

- MAM-480: Freeze the 15 legacy schema definition exceptions and detect Unicode format characters and Cyrillic letters in string values, while allowing Greek mathematical notation.

- MAM-837: translate rustdoc, comments, and existing README text in engine-core,
  engine-model, engine-linalg, engine-contracts, and engine-assembly into English,
  preserving axial-bar and elastic-truss documentation and all executable code.

- MAM-838: traduce al inglés la documentación Rust y los comentarios de las fuentes
  de análisis, cargas, leyes, geotecnia y elementos truss, frame, frame3d y spring;
  conserva literales, contratos, esquemas y resultados para siguientes entregas.

- MAM-839: rename public test/helper identifiers and four documents to English
  (DETERMINISM.md, METHODS_GUIDE.md, ORACLES.md, AXES_CONVENTION.md), updating live references.
  Translate schema titles/descriptions to English with new SHA-256 digests while
  preserving the same $id values and schema structure; regenerate published schemas
  and update the CLI description digest snapshot.

- MAM-840: translate technical guides, contribution documentation and method templates into English, preserving code, link destinations, hashes and the existing method-section anchor.

- MAM-841: Translate user-facing engine diagnostics, causes, hints, and usage messages to consistent English defaults for the future i18n catalog. Update covered text expectations while preserving stable codes and data contracts.

- MAM-842: traduce al inglés la documentación Rust y los comentarios de las fuentes
  de API, CLI y bindings Python; verifica las fuentes WASM ya en inglés y conserva
  los literales, contratos y resultados para las siguientes entregas.

- MAM-843: traduce al inglés la documentación Rust, los comentarios y los mensajes
  de aserción y uso autorizados de engine-validation en el tramo 1; conserva
  los literales de datos, contratos y salidas para siguientes entregas.

- MAM-844: traduce al inglés los ADR de `docs/adr/` (ADR-001 a ADR-004 y la
  evidencia de ADR-002); conserva decisiones, estados, números, tablas, código,
  enlaces y las anclas citadas por otras bitácoras.
  - MAM-844: traduce al inglés el protocolo clean-room (`docs/CLEAN_ROOM.md`) y el
  proceso de CLA (`docs/CLA.md`); conserva el sentido, obligaciones, plazos,
  cifras, estructura, código y enlaces.
  - Provenance notes (`provenance/`) translated into English (ENG-LANG-008, provenance group).

- MAM-865 / ENG-SFC-014: SIGINT/SIGTERM ahora cancelan; manejo cooperativo de SIGINT y SIGTERM en el CLI Unix
  mediante la API segura de flags de signal-hook =0.4.5 (decisión #203), sin
  código unsafe. Cancela el token existente y termina con código 5, stdout vacío
  y sobre cli-run/1 con outcome cancelled en stderr, distinguible del timeout (6).
  Recoge el hilo y los registros al salir. Pruebas del binario con consolidación
  pesada, kill -INT/-TERM, timeout amplio y limpieza de directorios temporales.
  Sin manejadores en plataformas no Unix.

- MAM-918: a text `units` label other than `"SI"` now yields `ENG-E-0001`
  (exit 2, no results) in `run` and `validate` instead of `API-E-INPUT` /
  `ENG-E-0009`. Incompatible: new public variant `ApiError::UnknownUnit` breaks
  exhaustive `match` on `ApiError`, and the serialized diagnostic code and
  error kind change for wasm and Python consumers.
  The cause omits the received label for security, and the unit check precedes structural validation.

- MAM-919: los números JSON no finitos (por ejemplo 1e999) y los tokens NaN,
  Infinity, -Infinity y -NaN se rechazan antes del parseo con `ENG-E-0008`
  (exit 2) en vez de `API-E-INPUT`; nueva variante `ApiError::NonFiniteNumber`.

- MAM-921: un `ENG-E-0006` cuyo DOF singular tiene una rigidez de elemento
  absorbida por redondeo al ensamblar (contraste de rigidez fuera de la
  resolución de f64) se informa ahora como `ENG-E-0010` «Ill-conditioned
  stiffness» (invalid, exit 2) con nudo y DOF; los mecanismos reales siguen en
  `ENG-E-0006` (hallazgo H4 de VAL-039).

- MAM-939: export mamut-toml/1 as one UTF-8 byte form, with LF, one trailing newline, two-space indentation, and specification key order, keeping a value the author wrote.

- MAM-940: a TOML document error keeps its code and reports a 1-based line and column when the parser provides a position. The root table remains nesting level 1. An error without a position does not invent a line.

- MAM-972: engine-input, run, Python, and the TOML converter accept a uniform
  temperature load on a linear truss. Each record carries alpha_per_kelvin
  and delta_t_kelvin, with no gradient. A bar held at both ends recovers
  N = -(E·A·α·ΔT). Elastic beam-column kinds reject the record; no frame
  formula was added.

- MAM-991: type the Python Winkler branches and the experimental result methods.

- MAM-1033: `validate` anticipa `ENG-E-0013` si `buckling.linear` trae
  `temperature_loads` en el caso de referencia o pDelta lo trae en la gravedad.
  La temperatura en el caso lateral, o en un caso que no es la referencia del
  pandeo, sigue permitida. En la cercha alineada ese caso lateral alarga α·ΔT·L.

- MAM-1036: `validate` anticipa los rechazos que `run` ya emitía (temperatura o `member_loads` en la referencia geométrica, y un documento que no trae exactamente un análisis), con el mismo código y el mismo texto. Si `run` se detiene antes, también al construir el sistema con prescritos o diafragmas, `validate` devuelve esa causa y no el rechazo de `member_loads`. `run` no cambia: un modelo inválido con temperatura en la referencia sigue en `invalid`.

- MAM-1037: las fichas fechan la corrección de κz el 2026-11-09 (ECT)
  y anclan el voladizo axial de `StaticNewton` con Ux y εp en forma cerrada.
  El voladizo con momento de punta queda en equilibrio, signo y simetría.
  La ficha de `rectangle.fiber_bending` cuenta los tests reales de la sección,
  y la tracción de cada franja se lee en la ley uniaxial pública.

- MAM-1038: el acuerdo de `validate` con `run` cubre un pórtico D3
  con diafragma válido y `member_loads` en la referencia, y el aviso
  `API-W-DIAPHRAGM-RECOVERY` queda delante del error.
  El voladizo con momento de punta ancla Ux y θ con la cuadrática de
  `docs/fichas/frame2d.fiber_stations.md`, y esa ficha escribe la cota
  256/9375000 < fy/E. `run` y `validate` no cambian de comportamiento.

### Corregido

- MAM-11: walk the buckling residual by component so the test loop satisfies clippy.
  - MAM-11: move the buckling wording tests below `SystemView` so clippy accepts the module order.

- MAM-943: clear the TOML converter clippy warnings without changing the mapping.

- MAM-972: walk the uniform-temperature checks by component and use `expect_err`.

### Validación

- MAM-838: traduce al inglés el rustdoc y los comentarios de `pdelta.rs` y `modal.rs`
  y actualiza sus hashes en el índice de val032; sin cambios de código ni de resultados.

- MAM-843: traduce al inglés la documentación Rust, los comentarios y los mensajes
  de aserción y pánico autorizados de las pruebas de los demás crates; conserva
  los literales de datos, contratos y salidas (tramo 2).

- MAM-850: añade VAL-037 y VAL-038 al informe de validación.
  VAL-037 compara el B2 de un pórtico de tres pisos con el operador
  cúbico de Hermite 1/(1−6r/5), con Δ1 = H h³/(12 E ΣIc) y banda 1e-6.
  Esa banda queda escrita después de la revisión r1.
  VAL-038 publica los cuatro Euler ya ensayados, en el capítulo de pandeo.
  buckling.linear sigue Experimental. pDelta sigue Stable en el registro.

- MAM-851: añade pruebas de robustez VAL-039 de API y CLI para mecanismos,
  rigideces extremas, longitudes casi nulas, pDelta supercrítico, modos repetidos,
  masa nula, entradas mal formadas y escalado SI; exige códigos del registro,
  exit codes, finitud y equilibrio. Documenta diagnósticos pendientes de unidades,
  no finitos, recuperación con algunas cargas transversales a partir de
  ~1e6 N (umbral no monótono) y falso
  singular en barras en serie con contraste extremo de rigideces.

- MAM-852: add 2D/3D linear static invariance, Maxwell-Betti reciprocity, symmetry, global equilibrium, and analytical cantilever integration tests using internal references only.
  - MAM-854/ENG-VAL-006: clarify EQ-ALL detection limits and the pending filename collision decision (J4 engine#245 r2).
  - MAM-96/ENG-SFC-005: record private AGPL source access, notice visibility limits, and unverified Linear criteria (J4 engine#247 r2).
  - MAM-92/ENG-SFC-004: record engine#248 merged as 4603d3d, J4 r1 verification (893 passed / 0 failed, green CI), contract test limits, and pending Windows/SDK integration.
  - ENG-PY-008: clarify the separate nbclient environment and wheel package via PYTHONPATH; offline notebook execution remains unknown (J4 engine#242 r1).

- MAM-854: añade EQ-ALL con 60 filas de equilibrio global de fuerzas y momentos,
  cortante basal y tolerancia relativa 1e-10; documenta inventario y pendientes.

- MAM-972: `prepare` aplica `temperature_loads` con las mismas fuerzas de nudo
  equivalentes que `run`. La barra libre axialmente alarga α·ΔT·L = 0.5·3·2 = 3.0 m,
  y el caso con fuerza mecánica más temperatura coincide entre `prepare` y `run`
  con tolerancia absoluta 1e-12 m. Las combinaciones siguen en `run`.
  `buckling.linear` rechaza un caso de referencia con `temperature_loads` no vacío,
  y pDelta rechaza el caso de gravedad en la misma situación, con `ENG-E-0013`
  y sin factor publicado. Kg no incluye el axil térmico αΔT.

### Otros (legado)

- MAM-862 / ENG-SFC-011: entrada engine-input/1 mediante `--input <ruta|->`
  en run/validate, con límite de 8 MiB y lectura única compartida entre pasos;
  catálogo/workflows embebidos por defecto, precedencia sobre MAMUT_ENGINE_INPUT
  y conservación de los bytes de la vía explícita anterior. Pruebas de binario
  para transporte, límites, validación, errores y compatibilidad.
  Ronda 2: rechazo documentado de `--input=valor`, `--` y valores ausentes/flags;
  stdin sin streaming por paso, límites MAX-1/MAX/MAX+1 y temporales con PID,
  contador atómico y pruebas de colisión/concurrencia sin reiniciar el global.
- MAM-860 · VAL-032 E3 v1.1.2: amplificación y deriva P-Δ por piso frente a
  referencia propia P-Δ+P-δ convergida (8/16 elementos por columna) a 1e-3
  relativo, y verificación independiente con un elemento a 1e-9. Script/JSON
  reproducibles con axial de gravedad lineal congelada y Kg Hermite consistente.
  `1/(1−θ)` pasa a diagnóstico unilateral `0 ≤ amp·(1−θ)−1 ≤ 5e-2`.
  Tests del Engine, hashes y documentación actualizados; 5e-3 frente a Frame3DD
  y las demás tolerancias se conservan. Sin cambios del solver ni de entradas.
  Ronda 2: estados VAL-032 sincronizados con #188 (Guyan) y #185/#186:
  ocho runs modales y E3-pdelta ejecutables con warning #84. E1/E2/E4 v1.1.2,
  E2b/E5b v1.0.2 y E3 v1.1.3; generador, índice, hashes y README coherentes.
  Prosa heredada de requisitos de formato aclarada para la guardia clean-room,
  conservando las diez cifras exigidas y todos los criterios numéricos.
  `val032_runs` contrasta metadatos con estados/avisos reales y verifica
  las huellas actuales de fuentes Rust; modelos, runs y tolerancias intactos.

- MAM-80 / ENG-PY-005 · PR #170, ronda 6 (J4 r3, S1): sincroniza la
  preparación de los tests de timeout con PID publicado después de exec/setsid;
  conserva el tope de terminación de 2 s y añade subcaso, PID y tiempo a los fallos.

- MAM-80 / ENG-PY-005 · PR #170, ronda 5 (J4 r2, H6/H7/N4/N5): Ctrl-C y
  otras excepciones durante la espera del CLI terminan el grupo POSIX antes
  de propagarse; una sonda de grupo denegada conserva el timeout tipado.
  Regresiones de interrupción y tiempo de terminación inferior a 2 s;
  README aclara los descendientes con `setsid` y la espera de limpieza de ~5 s.

- MAM-80 / ENG-PY-005 · PR #170, ronda 4: limpieza de timeout acotada
  incluso si un descendiente fuera del grupo retiene stdout/stderr; cierre
  explícito de tuberías y recolección acotada, conservando el error tipado.
  Tests con sondeo de PID y ejecutables independientes del PATH.

- MAM-80 / ENG-PY-005 · PR #170, ronda 3 (H1): el timeout de workflows
  Python termina todo el grupo POSIX (SIGTERM y SIGKILL tras espera acotada),
  incluidos nietos de envoltorios; conserva error tipado, causa y limpieza.
  Sin `killpg` (Windows) se mantiene la terminación del proceso directo.

- MAM-80 / ENG-PY-005 · PR #170, ronda 2: timeout configurable y validado
  en workflows Python (300 s por defecto), error tipado y limpieza al expirar;
  estado de variantes sin Results derivado del documento/categoría del error.
  Tests de CLI simulado que duerme y de copias independientes del catálogo.

- MAM-80 / ENG-PY-005: experimentos Python externos sobre la fachada Rust:
  barridos por rutas, comparación de escenarios con métricas y huellas,
  perfiles versionados y errores tipados por variante. Transporte al CLI SDK
  existente para workflows compilados/manifiestos, sin registro, planificador,
  callbacks ni cálculo numérico Python. Ejemplos de voladizo y pruebas offline;
  sin cambios Rust, Cargo ni pyproject.
- MAM-835 / ENG-SFC-010, r3 (J4 H1'): presupuesto público denso de
  consolidation1d `Σ intervalos n³ + Σ subpasos n² ≤ 500000000`, 256 intervalos
  y 100000 subpasos totales; rechazo temprano API-E-CONSOLIDATION-COST,
  conservando la cota r2. Calibración release con objetivo ≤5 s.
  Reutilización del factor de referencia con dt idéntico por bits y cancelación
  antes de combine/factorize y entre bloques de hasta 16 columnas; aritmética
  y hashes conservados. [Provenance r3](provenance/ENG-SFC-010.md#r3--revisión-j4-h1).

- MAM-835 / ENG-SFC-010, r2 H2: nonlinearStatic, soilProfile y
  consolidation1d se registran como `Experimental` hasta la cobertura de
  ENG-VAL-006 (decisión de Jonathan vía J0 2026-10-20). Catálogo, esquemas,
  explain y guía actualizados; fixtures SFC-010 habilitan explícitamente
  `options.allow_experimental`. Los demás métodos conservan su estado.

- MAM-835 / ENG-SFC-010, r2 (J4): cota pública de trabajo total de
  consolidation1d (50000000 subpasos por elementos), rechazo temprano
  API-E-CONSOLIDATION-WORK y cancelación en cada subpaso con el diagnóstico
  existente. Pruebas de fronteras M1/M4/M8/M9/M10 y longitud de la historia
  del acero; sin cambios de aritmética ni estados de métodos.
  [Provenance r2](provenance/ENG-SFC-010.md#r2--revisión-j4).

- MAM-835 / ENG-SFC-010: frontera engine-input/1 para Winkler 2D/3D,
  liberaciones, resortes y leyes uniaxiales; nonlinearStatic por pasos,
  soilProfile y consolidation1d. Esquemas y DTO Python compatibles,
  diagnósticos y fixtures analíticos compartidos por CLI/Python/WASM;
  núcleo numérico intacto. [Provenance](provenance/ENG-SFC-010.md).
- #184 · ENG-LA-009: condensación estática de Guyan para GDL libres sin masa
  en modal y espectro, tras apoyos/diafragmas; recuperación de modos completos
  y un solo factor de Kss. ENG-E-0014 identifica mecanismos sin masa.
  Ocho runs modales VAL-032 habilitados con períodos contra referencia NumPy
  propia y tolerancias originales; fixtures y camino sin condensación intactos.
  Ronda 2: pruebas bit a bit del camino directo, masa consistente acoplada,
  M-ortogonalidad completa tras recover, GDL exactos y cinemática de esclavos;
  decisión de compatibilidad de diafragmas #84 y cita p. 380 documentadas.
- #185 · ENG-E-0007: P-Δ proyecta la rigidez geométrica, sus fuerzas y el
  residuo mediante la transformación de diafragmas rígidos; la inercia LDLᵀ
  se evalúa en la rigidez total proyectada. Regresiones de restricciones
  exactas y E3-pdelta con referencia NumPy propia reproducible que lee el
  fixture de main, sin duplicarlo ni cambiar los JSON/hashes de VAL-032.
  VAL-032 acepta E3-pdelta ejecutable con aviso #84 justificado y conserva
  los ocho bloqueos modales exactos de #184 / ENG-E-0006.

- MAM-46 · ENG-GEO-003: generadores estáticos `py.matlock_soft_clay`,
  `py.reese_sand` y tabla simétrica `py.user` en engine-geo, puntos (m, N/m),
  parámetros obligatorios y errores tipados; adaptador de tensión efectiva
  del perfil a Pa. Tres fixtures analíticos compartidos nativo/wasm,
  integración dimensional con multilinear_elastic y bitácora clean-room.
  Sin cambios de los análisis existentes ni exposición JSON del futuro pilote.
- MAM-45 · ENG-GEO-004: tablas axiales inmutables `tz.api_clay`,
  `tz.api_sand`, `qz.api`, `tz.user` y `qz.user` en m/Pa/N; parámetros
  obligatorios, tracción explícita, mesetas y errores tipados. Adaptadores
  a la ley multilineal con conversión de unidades/signos y rechazo claro
  del ablandamiento que esa ley no admite. Fixtures API RP 2GEO y analíticos
  compartidos en nativo/wasm, sin nuevas dependencias. Coyle–Reese queda
  solo vía tablas del usuario, pendiente de la fuente original.
  Seguimiento J4 r1: `Explicit` valida el signo de resistencia en ambas ramas.
- MAM-855 / ENG-VAL-006: incorpora Frame3DD a la comparación externa de
  §5.1 del protocolo clean-room y formaliza la opción A: Q2 opera en sesión
  aislada, J4 audita y J2/J0 solo reciben discrepancias sin valores.
  Seguimiento de revisión: alinea §2/§3 y exige versión y hash por campaña,
  con nuevo registro y auditoría de J4 al cambiar de versión.
  Seguimiento J4 r1 (#178): precisa aprobación directa y antecedentes,
  alcance de la opción A, acceso a `results/` y declaración de la plantilla.

- MAM-82 · #162: quitar rutas locales de provenance ENG-PY-004 y anotar licencia BSD-3-Clause de NumPy en ADR-004

- MAM-82 / ENG-PY-004 · seguimiento #156: conserva la causa al faltar NumPy,
  explicita el rechazo de bool numérico y comprueba la independencia de las
  exportaciones. Pruebas NumPy migradas a unittest con skip opcional para el
  descubrimiento oficial de la wheel.

- MAM-82 / ENG-PY-004: exportación tabular de resultados con listas o copias
  NumPy float64 C, IDs/GDL/unidades/estado y extra opcional de importación
  perezosa. ADR-004 y benchmark offline Rust/CLI/FFI/fachada/copias, sin nuevas
  dependencias Cargo, física Python ni cambios del GIL o resultados del motor.

- MAM-105 · ENG-VAL-005, seguimiento PR #155: fuentes primarias propias con
  anclas verificadas, bibliografía por cotejar separada de la aceptación y
  cobertura calculada desde las tablas/fixtures, con diagnóstico de diferencias.
  Pruebas negativas de citas pendientes y justificaciones vacías; justificación
  específica de los cinco defaults fixture/1, sin cambios numéricos.
  [Provenance](provenance/ENG-VAL-005.md#seguimiento-de-revisión-opencode--pr-155).

- MAM-105 · ENG-VAL-005: matriz y guardia de oráculos libres para los 91 casos
  V1 de engine-validation, incluidos Winkler, consolidación y fixture/1.
  Tolerancias existentes explícitas, sin cambios de valores numéricos;
  ENG-REL-001 depende de ENG-VAL-005 y no exige ENG-VAL-003. OpenSees solo
  como contraste externo adicional, sin incorporar archivos ni resultados.
  [Provenance](provenance/ENG-VAL-005.md).

- MAM-25 · ENG-CORE-011: guía en español para contribuir métodos Rust,
  plantillas de ficha, módulo, conformidad, fixtures, prompt IA restringido,
  provenance y revisión. Piloto `elastic.cubic` con derivación energética propia,
  fábrica/descriptor uniaxial/1 y benchmark analítico nativo; sin modificar
  métodos existentes ni los 16 hashes de determinismo.
- MAM-91 / MAM-79 · #152 (2026-10-20): rutas locales de ENG-SFC-002 y ENG-PY-003 sustituidas por marcadores genéricos; solo documentación.
- MAM-79 / #144: PyO3 y sus cuatro crates auxiliares pasan de 0.25.1 a
  0.29.3 para corregir RUSTSEC-2026-0176 y RUSTSEC-2026-0177. El adaptador
  usa `Python::detach` y conserva la API Python y la feature `abi3-py310`;
  actualización y validación de wheels reproducibles exclusivamente offline.
- MAM-91: helpers de tests del CLI toleran solo BrokenPipe al escribir stdin de hijos que terminan antes de leerlo; se mantienen los asserts de salida y stderr, sin cambios del CLI.
- MAM-79 / #143: engine-api rechaza literales enteros fuera de ±(2^53−1)
  antes de JCS con `InvalidInput` (`API-E-INPUT`, salida 2, huella null).
  Python aplica el mismo rango simétrico antes del FFI y detecta como stale
  mutaciones hacia enteros inseguros. Regresiones de fronteras y de bool como
  entero; sin cambios de vendor, cálculos ni los 16 hashes de determinismo.
- MAM-91 / #146: pruebas del CLI con EOF para aceptar exactamente MAX_INPUT_BYTES y rechazar MAX+1 como Document/2; mutación del límite verificada y bitácora sin ruta local de compilación.
- MAM-22 / #142: exige exactamente cinco fixtures JSON, elimina rutas locales de la documentación de engine-validation y explicita el alcance de los agentes dentro de su worktree.
- MAM-91 / #141: regresiones del CLI para lectura acotada a MAX+1 y mapeo de ENG-E-0013 en validate a salida 3, verificadas mediante mutaciones.
- MAM-79 / ENG-PY-003: fachada Python pública tipada y documentada sobre
  engine-api, con modelos/resultados versionados, errores completos, controles
  SI/ejes y stale mediante la huella JCS de Rust. Completa procedencia,
  descriptores y convergencia P-Delta del main actual; prueba P-Delta Stable sin
  consentimiento experimental. Incluye regresiones de compatibilidad, stubs y
  comparación de la wheel offline con 16 entradas Rust. Sin física en Python.
- MAM-22 · ENG-CORE-009: contrato JSON fixture/1 con source obligatorio y veto
  recursivo de procedencias prohibidas; cinco casos analíticos propios y runner
  en engine-validation con targets engine-api nativo y wasm32/Node sin npm.
  Informes legibles y JSON determinista, tolerancias justificables e interfaz
  Target para añadir el CLI tras ENG-SFC-002. Véase
  [provenance](provenance/ENG-CORE-009.md).

- MAM-91 / ENG-SFC-002: CLI nativo `mamut-engine` sobre `parse_args`/`dispatch`
  de SDK-WF-012, con métodos `engine.run` y `engine.validate`, resultados completos
  y códigos Engine v1 0/1/2/3/4/5/70 probados ejecutando el binario. Entrada y
  cancelación cooperativa por rutas locales configuradas mediante entorno;
  sin lógica numérica ni dependencias externas nuevas. Se documenta la
  correspondencia de operaciones con describe/methods/validate/run y sus límites.
  Binarios Windows/Linux/macOS en CI pendientes de Jonathan; workflows intactos.
- MAM-582: pDelta pasa a `Stable` y se habilita por defecto, según
  [ENG-BENCH-002 (PR #132)](provenance/ENG-BENCH-002.md): AISC 360-16
  Comm. C2, casos 1 y 2, error ≤0.27% con tolerancias 3% M / 5% Δ. Se conserva `options.allow_experimental` para futuros métodos
  Experimental y su gate tiene cobertura unitaria.
- Se añade Stable al esquema method-descriptor/1; se actualizan pruebas,
  documentación y tres fixtures P-Delta sin consentimiento experimental.
  Se regeneran los dos hashes de cruce afectados por input_sha256 y se verifica
  igualdad nativo/wasm en debug y release. Los cálculos mecánicos no cambian;
  véase la tabla antes/después en [provenance](provenance/ENG-CORE-006.md#mam-582--pdelta-estable--2026-10-15).
- Cerca o por encima del pandeo, P-Delta puede terminar `unstable` o
  `notConverged`: salida 4, sin resultados mecánicos; se conserva el reporte
  de convergencia. El test
  [`pdelta_nonconvergence_and_instability_return_four_with_iteration_report`](crates/engine-api/tests/dynamic.rs)
  comprueba no convergencia al limitar a una iteración (P/Pcr=0,2) e
  inestabilidad por encima del pandeo (P/Pcr=1,2), ambos con salida 4.

