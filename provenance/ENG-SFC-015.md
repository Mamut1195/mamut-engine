# Bitácora clean-room · ENG-SFC-015 · MAM-866

- Fecha: 2026-10-21 (America/Guayaquil).
- Autor: agente Codex, GPT-6; rol: implementador.
- Base local: origin/main, revisión 7375fa25a34a3cb004bcaec84f645743b864ff89.
- Contrato implementado: progress/1, transporte NDJSON de etapas del motor.
- Fuentes permitidas: ticket MAM-866; código propio de engine-api y engine-cli;
  docs/DETERMINISMO.md; changelog.d/README.md; interfaces locales del SDK
  vendorizado (parser, configuración, observadores, proveedor y ejecución).
- Derivación propia: adaptación del callback de la API a un evento con índice
  del paso, sin cálculos de dominio ni resultados mecánicos nuevos.
- Código de terceros copiado: ninguno.
- Declaración: no consulté implementaciones externas de análisis estructural,
  sus entradas/salidas de referencia, rutas restringidas ni registros de otras
  sesiones. No se consultó la bitácora restringida indicada en el ticket.
- Revisor clean-room: pendiente de revisión humana.

## Contrato y decisiones

El tipo real engine_api::Progress contiene operation, stage, completed y total;
no contiene iteración ni residuo. Se transportan esos cuatro campos y el índice
del SDK desde cero. No se infiere iteración a partir del contador de etapas.
El esquema no admite campos adicionales ni números no finitos. Para una futura
extensión con residuo, el valor no finito se representará como null y se deberá
revisar el esquema; hoy el campo no existe.

El flag existente --progress ndjson incluye progress/1 por defecto, conservando
los eventos originales del SDK. La README documenta la selección por schema o
phase para consumidores. No se añade sintaxis al parser vendorizado.
Se usa el mismo canonizador público que results/1, sin tiempos, para mantener
el orden de claves y el formato numérico determinista.

La lógica de transporte se concentra en crates/engine-cli/src/progress.rs.
El escritor compartido agrupa las escrituras del SDK hasta el salto de línea y
envía cada evento completo mediante write_all; cada etapa se escribe y vacía
durante el callback, antes del observador del anfitrión. Los errores de E/S se
recuerdan para devolver 70 y evitar una salida de éxito en stdout.

El SDK exige que Method no contenga préstamos, pero invoca la fábrica dentro
del intervalo started/finished. La fábrica ejecuta el método local con el
escritor prestado y devuelve un método que entrega ese resultado una sola vez.
Así se conserva la secuencia visible, incluidos fallos y cancelación, sin
añadir hilos, estado global, requisitos Send ni acumulación de etapas por paso.
La API pública del anfitrión y su callback conservan su firma.

## Validación

Las pruebas del binario recorren los fixtures propios lineal, pDelta y modal
de engine-api/tests/fixtures/determinism y los once fixtures compartidos sfc010,
incluidos nonlinearStatic/Newton y consolidación. Cada entrada usa tres pasos
(validación y dos ejecuciones) para comprobar índices y reinicio del contexto.
Se compara cada evento con los callbacks reales de la API y se valida contra
el esquema con el validador de pruebas existente, ampliado con una entrada
genérica de esquema; no se añaden dependencias.

Se exige igualdad de stderr entre dos ejecuciones, igualdad de stdout frente a
la ejecución sin progreso, JSON válido en todas las líneas y orden
started/etapas/finished por paso. También se prueban el fallo de convergencia
sin finished, mutaciones rechazadas por el esquema, escritura de líneas
completas, entrega inmediata al anfitrión y errores de escritura y vaciado.
No se guardan documentos del SDK como fixtures ni capturas de referencia.

Comprobaciones finales offline, con CARGO_TARGET_DIR=/workspace/.cargo-target/sfc15:

| Comprobación | Resultado |
| --- | --- |
| cargo fmt --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline | 753 correctas, 0 fallidas, 0 ignoradas; 115 suites, incluidas pruebas de documentación |
| bash clean-room/check.sh origin/main | OK, incluidos archivos nuevos mediante índice y objetos temporales en /tmp; índice real intacto |
| Revisión de identificadores en el contenido completo de cada archivo nuevo o modificado | OK |
| git diff --check | OK |

Los 19 tests de integración del CLI y sus 4 tests unitarios pasan. La cobertura
de progreso comprende 14 entradas propias, tres pasos por entrada y dos
ejecuciones con progreso frente a una sin él. Los bytes de los eventos del SDK
también se comparan con su formato original; se rechazan líneas vacías.

No se modifican vendor, manifiestos Cargo, configuración de CI, reglas
clean-room, métodos de dominio, barra axial ni elementos de celosía.
Sin red ni commits. No se modifica CHANGELOG.md: se añade el fragmento por ticket.

## Merge local de origin/main · 2026-10-22

Integración sin commit de origin/main e4002e16b13720f297b1f3c10b46efd0d2233496
sobre la rama de #210, HEAD 884eef2bc3a8d0b3bca19dddc1c693b62dc02db0.
Se mantiene el merge en curso; no se ejecutan rebase ni abort.

Resolución de los tres conflictos:

- `crates/engine-cli/src/lib.rs`: se conserva la lectura única y precedencia de
  `--input` de #197, la vía directa validate-report/1 de #200, el módulo timeout
  de #202 y la instalación opcional de señales con signal-hook de #214. Encima
  se conecta el observador de etapas de #210 al contexto del paso del SDK.
- `crates/engine-api/tests/support/mod.rs`: se conserva íntegra la versión de
  main, que delega en el validador compartido `support/schema.rs`. Los tests del
  CLI importan ese validador una sola vez para validate-report/1 y progress/1.
- `crates/engine-cli/README.md`: se conserva la traducción inglesa de #216 y
  se documenta progress/1 en inglés, junto con timeout, señales y validación.

`progress.rs` reutiliza el EventWriter de main para los eventos de etapas y
los del SDK; se elimina el buffer de líneas duplicado de #210. Se mantienen
write_all, flush, entrega durante el callback, claves canónicas y ausencia de
tiempos. Los errores de escritura/vaciado devuelven 70 y no publican éxito.
Las pruebas de timeout y señales aceptan y validan las etapas intermedias,
conservando el sobre terminal y los códigos 6/5 respectivamente. Una regresión
adicional compara los bytes de stdout/stderr entre la vía anterior y --input
por archivo/stdin, con y sin timeout amplio.

M1 de J4: se registra progress/1 en `scripts/schema-sources.json`, mediante
additional_schemas, y se regenera `schemas/progress/1.json` junto con el índice.
Se actualiza la tabla URN/URL y se explica la separación de registros en
`docs/PUBLICACION_ESQUEMAS.md`. No se añade a SCHEMAS: es un evento del CLI y
la convención de #212 publica esos contratos desde additional_schemas, sin
exigir que engine_api::schema() los exponga ni crear dependencia hacia el CLI.

La entrada MAM-866 permanece en `changelog.d/MAM-866.added.md`, actualizada
con la integración y publicación. CHANGELOG.md no contiene una entrada de
#210 que trasladar y no se edita. Los manifiestos, lockfile y demás rutas
prohibidas conservan los cambios que ya traía el merge, sin edición adicional.
La revisión de contenido completo del merge detecta un término vetado en prosa
heredada de `provenance/ENG-SFC-002.md`; se sustituye por «corrección» sin alterar
su significado. No se consultan bitácoras restringidas ni registros ajenos.

Entorno de verificación: sin red, Cargo offline, PATH con $HOME/.cargo/bin y
CARGO_TARGET_DIR=/workspace/.cargo-target/sfc15. TMPDIR, TMP y TEMP apuntan al
mismo directorio de destino durante las verificaciones; no se usan temporales
en /tmp. Los registros de esta ejecución también se guardan en ese destino.

Resultados de esta integración:

| Comprobación | Resultado |
| --- | --- |
| cargo fmt --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline | 786 correctas, 0 fallidas, 0 ignoradas; 117 suites, incluidas pruebas de documentación |
| python3 scripts/generate-schemas.py --write | OK, generación local |
| python3 scripts/generate-schemas.py --check | OK |
| python3 -m unittest scripts/tests/test_schema_publication.py | 7 correctas |
| Contenido completo de los 124 archivos nuevos/modificados del merge | Sin identificadores prohibidos ni marcadores |
| git diff --check | OK |

El CLI tiene 9 pruebas unitarias y 41 de integración correctas. Se conserva
EventWriter sin duplicar su lógica de líneas completas. La revisión detectó
una importación repetida del validador en los tests, corregida compartiendo
una sola declaración de módulo antes de ejecutar Clippy y la suite completos.

Limitación del entorno al registrar la resolución: `git add` no pudo crear
`/workspace/mamut-engine-r/.git/worktrees/mamut-engine-sfc15/index.lock`, porque
el directorio Git real está montado como solo lectura, fuera de los destinos
escribibles. Los tres archivos carecen de marcadores y pasan las pruebas, pero
el índice real conserva sus entradas sin fusionar (UU). No se altera el índice
mediante rutas alternativas ni se intenta escalar permisos. MERGE_HEAD sigue
en e4002e16b13720f297b1f3c10b46efd0d2233496 y HEAD no cambia.

Se deja el comando exacto de git add, sin commit, en
`/workspace/.cargo-target/sfc15/MAM-866-stage-resolutions.sh`, para ejecutarlo
desde un entorno que permita escribir el índice real. Todos los cambios de
contenido y la verificación quedan completados; solo falta registrar la
resolución en ese índice.

## Merge de main con cli-error/1 · 2026-10-22

- Autor: Codex gpt-6.1-sol high.
- Se conservan las incorporaciones de main (#205 y #183) y se integra
  progress/1 de #210. Los cinco archivos en conflicto quedan sin marcadores;
  el índice Git conserva su estado previo porque no se ejecuta git add.
- Los fallos de escritura y vaciado del progreso pasan por el emisor central
  cli-error/1 con CLI-E-INTERNAL, ya registrado, y salida 70. Las pruebas
  validan las etapas intermedias y el único sobre terminal contra sus esquemas,
  incluidos timeout, señales y fallos del método.
- Se unen cli-error/1 y progress/1 en scripts/schema-sources.json y se regenera
  schemas/index.json exclusivamente con scripts/generate-schemas.py --write.
- Verificación offline: cargo fmt --check, Clippy del workspace con todos los
  targets y advertencias como errores, cargo test --workspace --offline
  (818 correctas, 0 fallidas, 0 ignoradas; 124 suites), generación --check
  y las 7 pruebas de publicación Python: OK. Pasa la cobertura de todos los
  emisores de códigos del workspace.
- CHANGELOG.md y changelog.d/MAM-866.added.md se conservan. No se editan
  las rutas prohibidas; temporales y registros quedan bajo CARGO_TARGET_DIR.
  Sin red, commit, git add, rebase ni abort.
