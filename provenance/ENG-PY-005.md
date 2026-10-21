# Bitácora clean-room · ENG-PY-005

- Ticket / PR: ENG-PY-005 · MAM-80 · experimentación y workflows Python externos al núcleo.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Rol: implementador.
- Contrato implementado: módulo Python puro `mamut_engine.experiments`, perfil
  `mamut.experiment/1`; reutiliza engine-input/1, results/1 y los contratos SDK
  de workflow/plan/manifiesto. **No es un método numérico.**
- Fuentes permitidas usadas: contratos y código propios del checkout:
  `bindings/python` (fachada ENG-PY-003 y arrays ENG-PY-004),
  `crates/engine-api` (registro ENG-CORE-006), `crates/engine-cli`
  (ENG-SFC-002) y las interfaces vendorizadas de mamut-workflows
  (SDK-WF-003/SDK-WF-012). No hay fuentes externas ni bibliografía numérica nueva.
- Derivaciones propias: ninguna; selección de campos, copias y transporte de
  documentos. No hay ecuaciones, solver ni cálculo numérico nuevo.
- Código de terceros copiado: ninguno.
- Validación: contratos e integración sobre los fixtures propios existentes
  `cantilever-linear-static.json` y `p-delta.json`, sin nuevos valores de
  referencia ni tolerancias. Los tests comparan documentos/huellas y estados
  de la misma fachada Rust; no validan otro método numérico.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código ni resultados de CalculiX o
  LAPACK, ni utilicé red, tablas o resultados de motores externos.
- Revisor clean-room: **PENDIENTE (J4)**.

## Decisiones y alcance

- Sin cambios Rust, Cargo, pyproject, vendor, clean-room ni CI. El empaquetado
  mixto existente incluye automáticamente los módulos Python nuevos.
- El producto cartesiano y la comparación viven en el proceso Python del
  usuario. Cada ejecución recibe una copia independiente del modelo. La
  fachada existente conserva validación, errores tipados y huella JCS de Rust;
  los fallos Engine quedan identificados por variante y no detienen el lote.
  Se ofrece `raise_for_error` para volver a lanzar la instancia original.
- Perfil, seed opcional (solo metadato), unidades declaradas, análisis originales
  y seleccionados, métodos efectivos, versión, commit y target acompañan al
  resultado. La huella de salida cubre los bytes exactos de results/1; no se
  confunde con la huella JCS de entrada. Un commit ausente queda null.
- Las métricas son selecciones escalares por ruta; no calculan, agregan ni
  convierten unidades. La etiqueta de unidad la declara el usuario según SI.
- No se ofrecen callbacks ni funciones Python registrables. No se carga Python
  dentro de Rust, solver, WASM, MCP o CLI. Los errores Python ajenos al contrato
  Engine se propagan.
- Los workflows usan el CLI Rust instalado por el usuario. Se conserva el plan
  compilado y los snapshots de catálogo/definición; el CLI recompila esos mismos
  documentos al ejecutar porque su contrato actual no importa planes
  serializados. Python no interpreta ni ejecuta pasos. El sobre SDK íntegro
  conserva las huellas oficiales y las sustituciones de métodos.
- El binario CLI se instala aparte de la wheel. La representación JSON anidada
  del CLI conserva el documento semántico y las huellas SDK, no los bytes de
  results/1 originales. Los fallos sin documento Engine conservan el sobre y
  un código `SDK-EXIT-*`; no se inventa un diagnóstico numérico.

## Verificación offline

Toolchain estable Rust 1.99.0; CARGO_TARGET_DIR dedicado al worktree,
CARGO_NET_OFFLINE=true y PIP_NO_INDEX=1. CPython 3.13/Linux x86_64,
maturin 1.15.0, pytest 9.1.1 y NumPy 2.5.3 disponible localmente para la
segunda ejecución. Los cambios se integran en commits de la rama del PR;
sin publicación ni acceso a red.

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **672 passed / 0 failed / 0 ignored**, incluidos doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| pytest sobre wheel final, sin NumPy | **58 passed / 0 failed / 3 skipped**, 266 subtests; skips solo del extra NumPy |
| pytest sobre wheel final, con NumPy | **61 passed / 0 failed**, 389 subtests |
| Tests nuevos ENG-PY-005 | **18 passed / 0 failed**, incluidos cuatro tests SDK con el CLI Rust real |
| `scripts/build-python-wheel.sh` | OK: 25 tests de herramientas; wheel instalada con unittest: 58 passed / 0 failed / 3 skipped |
| Contenido wheel | Ambos módulos nuevos presentes y byte a byte iguales al árbol de trabajo; sin editar pyproject |
| Reproducibilidad wheel | Dos construcciones independientes idénticas; SHA-256 `ff94fd973048917574455043f1a5f78641a55454ba534412056b3dc5a42ea23a` |
| Ejemplos Python sobre wheel instalada | OK: tres variantes de carga y dos escenarios de sección del voladizo |
| Determinismo nativo/wasm, debug/release | **16/16 hashes idénticos a docs/DETERMINISMO.md**, diez ejecuciones por caso/target; debug=release |
| `git diff --check` | OK |

La prueba de opciones de análisis comprueba que `base_analyses` conserva la
lista original incluso al barrer su tolerancia; `requested_analyses` registra
la opción aplicada. Los perfiles de la vía SDK rechazan campos de barrido,
métricas y sustitución de análisis para evitar ignorarlos silenciosamente.
Los errores de transporte conservan la causa de E/S y los fallos del motor
conservan su clase, código, entidades, documento y diagnósticos.

## Ronda 2 · PR #170 · 2026-10-20

- Autor: Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Rol: implementador. Alcance limitado a los comentarios #5, #4 y #6.
- Fuentes: contratos y código propio Python del checkout; ninguna fuente
  externa, código de terceros copiado ni derivación numérica nueva.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código o resultados de CalculiX/LAPACK,
  ni usé red, oráculos externos, tablas o figuras de terceros.
- Revisor clean-room: **PENDIENTE (J4)**.

### Correcciones y evidencia

- #5: `compile_workflow` y `run_workflow` aceptan `timeout` solo por keyword,
  predeterminado 300 s; se rechazan cero, negativos, NaN/Inf, bool, tipos
  incompatibles y enteros no representables como tiempo finito. Se reutiliza
  `AnalysisError` con código `PY-E-WORKFLOW-TIMEOUT`, conservando
  `subprocess.TimeoutExpired` como causa. `subprocess.run` mata y espera al
  CLI al expirar; el contexto temporal elimina sus entradas antes de lanzar
  el error público. Un ejecutable Python propio que duerme 60 s verifica
  ambas operaciones con timeout de 0,2 s, proceso terminado y directorio
  eliminado. Otros tests comprueban validación previa y propagación del valor
  predeterminado/personalizado al transporte.
- #4: sin `Results`, se prioriza el estado de fallo de `error.document` si es
  JSON legible; después se usa la categoría pública: `InvalidInputError` →
  `invalid`, `Unstable` → `unstable`, `NotConverged` → `notConverged`,
  `Unsupported` → `unsupported`, `Cancelled` → `cancelled` e
  `InternalEngineError` → `internalError`. `AnalysisError` genérico →
  `analysisError`; otro `EngineError` → `error` (estados de variante Python).
  Se conservan instancia, código, documento y huella del error original.
  Los tests cubren las ocho categorías, continuidad del lote, precedencia
  del documento y respaldo ante documentos ilegibles o sin estado de fallo.
  El README documenta el mapeo y el timeout.
- #6: se capturan copias profundas de `methods()` y `capabilities()`, se mutan
  los descriptores y contenedores devueltos y se comparan las llamadas
  posteriores por igualdad profunda con las capturas completas originales.

### Verificación offline de la ronda 2

Rust 1.99.0, CPython 3.13.5, `CARGO_NET_OFFLINE=true`, `PIP_NO_INDEX=1` y
`CARGO_TARGET_DIR=$CARGO_TARGET_DIR`. Los artefactos,
temporales de pruebas y venv están dentro de ese directorio autorizado.
No se dispone de maturin en esta sesión: la extensión se compiló mediante
Cargo y el paquete se instaló manualmente en el venv de pruebas, con fuentes
copiadas y metadatos obtenidos de los manifests existentes. Esta ronda verifica
el código y su integración; no reconstruye ni acredita una wheel nueva.
Las referencias de regresión se generaron directamente desde engine-api,
mediante el ejemplo propio `facade_reference`.

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **672 passed / 0 failed / 0 ignored**, incluidos doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| unittest completo Python, con NumPy y CLI Rust real | **66 passed / 0 failed / 0 skipped** |
| Subsuite ENG-PY-005 (incluida en los 66) | **23 passed / 0 failed**, cinco tests nuevos y uno reforzado |
| Determinismo nativo/wasm, debug/release | **16/16 hashes iguales a docs/DETERMINISMO.md**, diez corridas por caso/target; debug=release |
| `git diff --check` | OK |

Sin cambios Rust, Cargo, pyproject, workflows, vendor, clean-room ni documentos
de determinismo. No se modifican resultados numéricos. Los cambios se integran
en commits de la rama del PR.

## Ronda 3 · PR #170 · H1 de J4 · 2026-10-20

- Autor: Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Rol: implementador. Alcance exclusivo: H1 (nietos del CLI tras timeout).
  H2–H5 y N1–N3 se difieren al issue #177.
- Fuentes permitidas: contratos y código propio Python del checkout; prueba
  propia de procesos POSIX, sin oráculo numérico ni fuentes externas.
- Derivaciones propias: ninguna. Código de terceros copiado al producto: ninguno.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código o resultados de CalculiX/LAPACK,
  ni usé red, tablas, figuras u oráculos externos.
- Revisor clean-room: **PENDIENTE (J4)**.

### Corrección y evidencia

- El transporte usa `Popen` con `start_new_session=True` en POSIX con
  `os.killpg`. El PID inicial identifica su grupo incluso si el envoltorio
  termina antes que sus descendientes. Al expirar, SIGTERM alcanza al grupo;
  se comprueba su existencia durante hasta 0,2 s con reloj monotónico y
  sondeos de hasta 0,01 s. Si persiste, recibe SIGKILL. Se ignora
  `ProcessLookupError` en las señales; `poll`/`communicate` recogen al hijo y
  cierran las tuberías. La comprobación del grupo no depende de que el hijo
  siga vivo ni de que sus descendientes mantengan abiertas las tuberías.
- Se conserva `AnalysisError` / `PY-E-WORKFLOW-TIMEOUT`, la instancia original
  de `TimeoutExpired` como causa y la limpieza de las entradas temporales.
  La vía normal conserva stdout/stderr de texto UTF-8, normalización de
  saltos de línea y tratamiento de returncode. Sin `killpg` (Windows) se
  mantiene el kill del proceso directo; esta limitación figura en el README.
- La regresión POSIX, omitida en otras plataformas, usa un script `sh` con
  un `sleep` en segundo plano, PID escrito a un archivo y espera larga
  interrumpida por timeout de 0,2 s. Cubre compile y run, salida cooperativa
  y un envoltorio que persiste tras SIGTERM con stdout/stderr cerrados
  (terminación por SIGKILL comprobada). Un trap recoge los hijos del shell
  para no depender de que init recoja zombis; `os.kill(pid, 0)` debe lanzar
  `ProcessLookupError`, con sondeo de 0,01 s y límite de 1 s. Se comprueba
  también el proceso directo recogido, el error tipado y el temporal eliminado.
  No hay esperas largas efectivas en la prueba. Otra prueba cubre UTF-8,
  CRLF y códigos de salida 0, 1 y 3.

### Verificación offline de la ronda 3

Rust 1.99.0, CPython 3.13.5, pytest 9.1.1, `CARGO_NET_OFFLINE=true`,
`PIP_NO_INDEX=1` y `CARGO_TARGET_DIR=$CARGO_TARGET_DIR`.
Venv, paquetes de prueba copiados desde la caché local y artefactos en ese
directorio autorizado. No hay maturin: se compila la extensión con Cargo y
se instala manualmente el paquete en el venv con los metadatos de los
manifests existentes; no se acredita una wheel nueva. Referencias propias
generadas directamente por Rust con `facade_reference`; CLI Rust real y
NumPy disponibles en la suite completa.

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **672 passed / 0 failed / 0 ignored**, incluidos doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| pytest completo Python, con NumPy y CLI Rust real | **68 passed / 0 failed / 0 skipped**, 433 subtests |
| Subsuite timeout/transporte (incluida en los 68) | **5 passed / 0 failed**, 31 subtests |
| Regresión POSIX repetida | **5/5 ejecuciones passed**, cuatro escenarios por ejecución; sin fallos |
| Determinismo nativo/wasm, debug/release | **16/16 hashes iguales a docs/DETERMINISMO.md**, diez corridas por caso/target; debug=release |
| `git diff --check` | OK |

Sin cambios Rust, Cargo, pyproject, workflows, vendor, clean-room ni resultados
numéricos. Los cambios se integran en commits de la rama del PR; sin acceso a red.

## Ronda 4 · PR #170 · 2026-10-20

- Alcance: transporte y tests Python de timeout, esta bitácora y CHANGELOG.
  Sin cambios Rust, Cargo, pyproject, workflows ni vendor; sin red ni commits
  creados en esta sesión. La entrega del PR se integra en commits de la rama.
- Fuentes: código y contratos propios del checkout; sin fuentes externas,
  código de terceros copiado ni derivaciones numéricas nuevas.
- Revisor clean-room: **PENDIENTE (J4)**.
- El test del shell sondea el archivo de PID hasta 5 s, en pasos de hasta
  0,01 s, y exige un entero positivo terminado en salto de línea. Si falta
  el archivo o queda incompleto, falla con un mensaje explícito.
- El `communicate` final de limpieza tiene un límite de 5 s. Si expira,
  se reintenta SIGKILL al grupo cuando aplica y se mata el proceso directo,
  se cierran explícitamente stdout/stderr y se recoge el hijo con `wait`
  limitado a 5 s. Los errores de esa limpieza no sustituyen el timeout
  original: se conserva `AnalysisError` / `PY-E-WORKFLOW-TIMEOUT` y su causa.
- Nueva regresión POSIX para compile y run: un descendiente crea una sesión
  mediante `setsid` y conserva las tuberías mientras ejecuta `sleep`.
  Ambos ejecutables se resuelven en rutas absolutas con `os.defpath`,
  independiente del PATH limitado del consumidor de la wheel; si faltan,
  hay skip explícito. Se comprueban tiempo acotado, error y causa originales,
  proceso directo recogido, tuberías cerradas y entradas temporales eliminadas.
  El descendiente fuera del grupo se mata siempre en `finally`.

### Verificación offline de la ronda 4

Rust 1.99.0, CPython 3.13.5 y maturin 1.15.0 local;
`CARGO_NET_OFFLINE=true`, `PIP_NO_INDEX=1` y
`CARGO_TARGET_DIR=$CARGO_TARGET_DIR`.
Se usó `MATURIN=/workspace/venv-maturin/bin/maturin` y la toolchain estable
local al inicio del PATH, sin descargar herramientas ni dependencias.

| Comprobación | Resultado |
| --- | --- |
| `bash scripts/build-python-wheel.sh $CARGO_TARGET_DIR/pyv-check` | **exit 0**; dos construcciones independientes idénticas y consumo offline con PATH limitado al venv |
| unittest de herramientas del script | **25 passed / 0 failed** |
| unittest de la wheel instalada | **62 passed / 0 failed / 7 skipped** (69 tests; cuatro requieren CLI Rust y tres NumPy opcional) |
| Regresiones POSIX de shell/nieto y descendiente fuera del grupo | **Ambas passed**, compile y run; ejecutadas dentro del script, sin skip |
| `cargo fmt --all -- --check` | **exit 0** |
| `git diff --check` | OK |

## Ronda 5 · PR #170 · revisión J4 r2 · 2026-10-20

- Rol: implementador. Alcance: H6, H7, N4 y N5, limitado al transporte,
  `WorkflowTimeoutTest`, README, CHANGELOG y esta bitácora.
- Fuentes: código y contratos propios del checkout y artefactos locales de
  prueba; sin red, fuentes externas, código de terceros copiado ni derivaciones
  numéricas nuevas. No se crearon commits ni se consultaron las rutas excluidas.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código ni resultados de CalculiX/LAPACK.
- Revisor clean-room: **PENDIENTE (J4)**.
- H6: después de la captura de `TimeoutExpired`, `_invoke` captura
  `BaseException`, termina el proceso/grupo y relanza la excepción original.
  `KeyboardInterrupt` conserva su clase e instancia; el timeout sigue siendo
  `AnalysisError` / `PY-E-WORKFLOW-TIMEOUT`, con `TimeoutExpired` como causa.
  La nueva regresión POSIX usa un envoltorio `sh` con nieto `sleep`, PID escrito
  a un archivo y trap para recogerlo. Un mock de `communicate` espera al PID,
  comprueba que pertenece al grupo del CLI y lanza `KeyboardInterrupt` una
  sola vez; la limpieza usa el `communicate` real. Cubre compile y run,
  exige desaparición de ambos PID con sondeo acotado a 1 s, entradas eliminadas
  y ausencia de directorios `mamut-experiment-*` nuevos. Hay limpieza en
  `finally` también si falla una aserción. Se omite sin grupos POSIX/`killpg`.
- H7: la regresión de timeout del shell mide con reloj monotónico y exige
  `elapsed < 2` en ambos subcasos (cooperativo y persistente), para compile y
  run. Así detecta el recurso a la espera final de 5 s si falta el killpg
  principal.
- N4: un `OSError` distinto de `ProcessLookupError` en la sonda
  `os.killpg(pid, 0)` hace pasar inmediatamente a SIGKILL del grupo. La nueva
  prueba deniega solo esa sonda con `PermissionError`, deja pasar las señales
  reales y verifica SIGKILL y el timeout público original en compile y run.
- N5: README documenta la supervivencia de descendientes que salgan del grupo
  con `setsid`, el retraso adicional de hasta unos 5 s si retienen tuberías y
  la terminación del grupo también ante Ctrl-C u otras excepciones.

### Verificación offline de la ronda 5

CPython 3.13.5 y pytest 9.1.1. Se reutiliza la wheel local de PR #170 y se
extrae a un directorio autorizado de pruebas, sustituyendo únicamente
`_experiment_workflows.py` por el archivo modificado. Los demás módulos `.py`
de la wheel se cotejan byte a byte con el checkout y son idénticos. No se
reconstruye ni acredita una wheel nueva. CLI, referencias Rust y NumPy son
artefactos locales existentes; sin descargas.

La invocación con `/workspace/venv-j2/bin/python -m pytest -q
bindings/python/tests` y `PYTHONPATH` obtiene inicialmente 70 passed y un fallo
de entorno: el subproceso aislado `python -I` de la prueba de importación no
ve `PYTHONPATH`, y ese venv no tiene instalado `mamut_engine`. Se prepara un
venv de pruebas dentro del directorio autorizado, usando el mismo intérprete
y los paquetes locales mediante `.pth`, para que también lo vea `python -I`.

Comando de la suite completa en ese entorno, con `MAMUT_EXPERIMENT_CLI` y las
variables de referencias Rust apuntando a los artefactos locales:

```sh
$CARGO_TARGET_DIR/r5-venv/bin/python -m pytest -q bindings/python/tests
```

| Comprobación | Resultado |
| --- | --- |
| Suite completa Python con CLI Rust y NumPy locales | **71 passed / 0 failed / 0 skipped**, 433 subtests |
| `WorkflowTimeoutTest`, clase completa en 20 procesos pytest sucesivos | **20/20 ejecuciones passed**; 8 tests y 31 subtests por ejecución, sin skips ni fallos |
| `git diff --check` | OK |

Cada repetición tarda entre 12,61 y 12,65 s, incluyendo las esperas de limpieza
del descendiente con `setsid`. Los logs individuales quedan en
`$CARGO_TARGET_DIR/r5-repetitions/01.log` a `20.log`.
La regresión nueva de `KeyboardInterrupt` cubre ambas operaciones en cada
ejecución: 40 escenarios de interrupción en estas 20 repeticiones. El módulo
de transporte probado sigue siendo idéntico al archivo del checkout.

## Ronda 6 · PR #170 · revisión J4 r3 · S1 · 2026-10-20

- Rol: implementador. Alcance: `WorkflowTimeoutTest`, CHANGELOG y esta
  bitácora; sin cambios del transporte de producción, Rust, Cargo, pyproject,
  workflows ni vendor. Sin red ni commits; no se consultaron las rutas excluidas.
- Fuentes: código y tests propios del checkout, biblioteca estándar Python
  instalada y artefactos locales. Sin fuentes externas, código de terceros
  copiado ni derivaciones numéricas nuevas.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código ni resultados de CalculiX/LAPACK.
- Revisor clean-room: **PENDIENTE (J4)**.

### Causa raíz reproducida y límites del diagnóstico

- Los escenarios del test iniciaban el timeout de 0,2 s sin confirmar que el
  envoltorio hubiera creado el nieto y escrito su PID. Bajo carga puede morir
  primero el envoltorio: esperar después el archivo no puede recuperar un PID
  que nunca se publicó. En el caso `setsid`, el padre publicaba `Popen.pid`:
  eso tampoco confirmaba que el nieto hubiera ejecutado `setsid` o `sleep`.
  En el shell, `$!` confirmaba fork, pero no exec ni el restablecimiento de las
  señales del nieto. La prueba de interrupción tenía esta última carrera incluso
  aunque esperase al archivo.
- Evidencia: con la instrumentación y sin corregir la preparación, la primera
  ronda con 16 `yes` y pytest restringidos a una misma CPU falló en
  `escaped/compile`. `_invoke` ya había vuelto en unos 0,301 s; el archivo
  `escaped.pid` seguía vacío/ausente. La primera espera falló a los 5,301 s
  totales y el `finally` volvió a esperar otros 5 s (10,304 s totales), ocultando
  el primer fallo. La clase dio **1 failed / 7 passed**, 31 subtests, en 15,19 s;
  el proceso pytest completo tardó 23,264 s bajo esa carga. Los 16 procesos
  de carga se mataron y recogieron en `finally`.
- Esto demuestra una causa de inestabilidad del test, pero no identifica con
  certeza el fallo histórico de 1/62 rondas sin diagnóstico. Un exceso de unos
  5 s también encaja con la espera por PID del shell o con el límite final de
  `communicate`. No se observó un fallo de `_terminate` ni se cambió ese código.
- Lectura del transporte: `start_new_session=True` establece la sesión/grupo
  antes de exec del hijo directo; `Popen` espera el resultado de exec, por lo
  que enviar `killpg` antes de crear ese grupo no explica este caso. Las sondas
  pueden ver grupos que contienen zombis, pero la espera de gracia termina
  a los 0,2 s; los zombis no conservan las tuberías. Un descendiente que salga
  del grupo con `setsid` y retenga stdout/stderr sí provoca legítimamente el
  `communicate(timeout=5)` de reserva, dos veces en la clase (compile/run).
  Por eso la duración normal sigue rondando 13 s. La falta de planificación
  del shell durante la gracia o un proceso que tarde en responder a SIGKILL
  siguen siendo hipótesis residuales; los nuevos mensajes permiten distinguirlas.

### Arreglo y conservación de las comprobaciones

- Cada operación tiene nombre `compile`/`run`; todos los asserts de tiempo,
  PID/PGID, señales y recolección identifican el escenario y el tiempo
  monotónico transcurrido mediante `msg=f"..."`. Las esperas fallidas de PID
  indican el subcaso, PID del CLI conocido, archivo, contenido parcial, tiempo
  de espera y tiempo total. Si un PID desaparece antes de consultar su PGID,
  el fallo también lo identifica.
- Un pequeño programa Python de prueba publica su propio PID después de exec
  y duerme con SIGTERM por defecto. El shell instala el trap, crea ese nieto
  y publica además su PID cuando ya ha cerrado sus tuberías en el caso
  persistente. En el escenario externo se ejecuta `setsid` en ruta absoluta
  y después el mismo programa; se verifica PID = PGID antes del timeout.
- La primera llamada a `communicate` espera esas señales de preparación
  (hasta 5 s, con diagnóstico), y luego llama al `communicate` real con el
  timeout original de **0,2 s**. La medición de timeout + terminación comienza
  después de la preparación; los fallos incluyen también su duración separada.
  Se conserva **elapsed < 2 s** para shell cooperativo y persistente, en ambas
  operaciones. Se siguen exigiendo SIGKILL para el persistente, desaparición
  real del nieto (no se aceptan zombis), recolección del hijo directo, error y
  causa originales, cierre de tuberías y eliminación de entradas temporales.
- Se conservan los dos descendientes del shell: ambos publican ahora su PID
  después de exec y se exige la desaparición de ambos. En el caso persistente,
  el primero conserva las tuberías originales y el segundo nace después de
  cerrarlas en el shell. El trap recoge a los dos y ejecuta `sleep` después de
  SIGTERM; el envoltorio debe terminar por SIGKILL aun con las tuberías cerradas.
- Los `finally` usan los PID ya registrados, no vuelven a esperar un archivo
  ausente, y limpian también si falla una aserción anterior. El caso de
  interrupción usa igualmente el PID publicado por el propio nieto.

### Verificación offline de la ronda 6

CPython 3.13.5 y pytest 9.1.1 del venv local `v-con`; módulo de transporte
instalado cotejado con el checkout mediante `cmp` (idéntico). No se reconstruye
ni acredita una wheel. CLI Rust y referencias son artefactos locales existentes.

```sh
MAMUT_EXPERIMENT_CLI=$CARGO_TARGET_DIR/release/mamut-engine \
$CARGO_TARGET_DIR/v-con/bin/python -I -B -m pytest -q -p no:cacheprovider bindings/python/tests/test_experiments.py -k WorkflowTimeoutTest
```

| Comprobación | Resultado |
| --- | --- |
| Instrumentación sobre el test original, sin carga | **8 passed**, 31 subtests, 12,64 s |
| Test original instrumentado con 16 `yes` en una CPU | **1 failed / 7 passed** en la primera ronda: PID no publicado; diagnóstico reproducido arriba |
| Serie de desarrollo bajo carga | **10/10 rondas passed**; se completó antes de la serie final |
| Archivo final, clase completa, sin carga | **10/10 rondas passed**, 8 tests y 31 subtests por ronda, sin skips; 12,80–13,43 s por clase |
| Archivo final, clase completa, 16 `yes` y pytest en una CPU | **10/10 rondas passed**, 8 tests y 31 subtests por ronda, sin skips; 17,11–17,73 s por clase; los 16 `yes` matados y recogidos |
| Suite completa Python, CLI Rust y NumPy locales, referencias Rust existentes | **71 passed / 0 failed / 0 skipped**, 433 subtests, 14,04 s |
| Comprobación del tope mediante retraso añadido solo en memoria | Añadir 2,05 s antes de `_terminate` real produce **4/4 fallos esperados** de `elapsed < 2`, con operación, persistencia, PID, elapsed y setup en el mensaje |
| Revisión AST de los asserts de tiempo/PID | Todos contienen `msg` con f-string |
| `git diff --check` | OK |

Las **20 rondas finales** usaron el mismo test (SHA-256
`3628b5a6984ea18e303eee32d3d0724229817747da0b9a2a7504c100839752ae`,
comprobado en cada ronda); suman 160 ejecuciones de test y 620 subtests. No se
mezclan con las diez rondas de desarrollo. El transporte instalado y el del
checkout coinciden (SHA-256
`c0b58f41eacbc65589a4cf9268ea5d0ff51b804039cfe65c62865be047873b3c`).

Logs y ejecutores locales en
`$CARGO_TARGET_DIR/r6-repetitions/`:
`instrumented-load-01.log` conserva el fallo; `final-load-01.log` a `10.log`
y `final-normal-01.log` a `10.log` conservan las rondas finales. Los archivos
`final-load-summary.log` y `final-normal-summary.log` registran SHA, duración
del proceso entero y limpieza de la carga; `full-suite-summary.log` registra
la suite completa. `time-bound.log` contiene los cuatro fallos deliberados y
`time-bound-summary.log` confirma que fueron exactamente los esperados. El
retraso se inyectó mediante mock en un proceso aparte, sin editar el transporte.

## Pendientes

Revisión clean-room **PENDIENTE (J4)**. El CLI debe estar instalado aparte para
usar workflows; la reimportación directa de planes serializados no forma parte
de su contrato actual. No se modificó ninguna ruta protegida ni resultado
numérico existente. Los cambios del PR se integran en commits de la rama.
