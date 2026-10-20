# Bitácora clean-room · ENG-PY-003

- Ticket / PR: MAM-79 · ENG-PY-003; cambios locales sin commit ni publicación.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Rol: implementador.
- Contrato implementado: fachada Python tipada de `engine-input/1`, `results/1`,
  `capabilities/1` y `method-descriptor/1` sobre engine-api/PyO3 (ENG-SFC-007).
- Fuentes permitidas usadas: exclusivamente contratos, schemas, documentación,
  fixtures y código propios del checkout actual: `crates/engine-api`,
  `bindings/python`, `docs/CONVENCION_EJES.md`, `docs/DETERMINISMO.md`,
  `docs/adr/ADR-003-python-v1.md`, `docs/CLEAN_ROOM.md` y `provenance/TEMPLATE.md`.
  No se consultaron fuentes externas ni código de dependencias numéricas.
- Derivaciones propias: ninguna física nueva. El test del voladizo conserva
  el oráculo propio de ENG-SFC-007: integrar EI·y'' = −P(L−x) con base empotrada
  da y(L)=−PL³/(3EI); equilibrio da R=P y M=PL. P=12, L=4, E=200, I=3;
  tolerancias absoluta/relativa 1e−12 para redondeo f64 del sistema pequeño.
- Código de terceros copiado: ninguno. Sin dependencias nuevas ni cambios
  de Cargo.toml/Cargo.lock o del SDK vendorizado.
- Validación: referencias directas Rust para los cinco fixtures válidos de
  engine-api y sus once fixtures de determinismo; comparación de bytes completos
  y SHA-256 con la wheel instalada, tolerancia cero. Son ejemplos analíticos
  propios existentes, no resultados de otros motores.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código ni resultados de CalculiX o LAPACK,
  ni informes/salidas de motores externos. Trabajo exclusivamente offline.
- Revisor clean-room: **PENDIENTE (J4)**.

## Revisión crítica del WIP frente al main actual

Base local: `2adc337` (MAM-582, P-Delta Stable por defecto). Se revisó el WIP
preexistente sin asumir que estuviese validado. La fachada conserva únicamente
las seis operaciones existentes en Rust; `Engine` solo las agrupa, sin estado.
No calcula física, convierte unidades ni suministra constantes físicas.

Se corrigió la huella del WIP: SHA-256 del JSON compacto antiguo no coincide
con RFC 8785/JCS de ENG-CORE-005. El nuevo helper Rust `engine_api::input_sha256`
reutiliza el mismo parseo acotado y la misma llamada a mamut-canon que `run`.
El adaptador privado lo transporta a Python para `Model.input_sha256` y stale;
no se reimplementa JCS ni se ejecuta un análisis para obtener una huella.
`run` comparte el helper interno sobre Value sin cambiar números ni documentos.

Se completaron los DTO de procedencia (`git_commit`, `target`), el criterio
opcional `residual_convergence_ratio` y los descriptores de capacidades actuales.
La regresión P-Delta ejecuta `options: {}` y comprueba `Stable`; la opción
`allow_experimental` se conserva únicamente según el contrato de Rust.

Errores públicos son clases Python que conservan código, entidades, severidad,
causa, hints, diagnósticos y documento originales. NotConverged, Unstable,
Cancelled y Unsupported siempre lanzan excepción. La interpretación secundaria
de un resultado fallido no puede ocultar el error original. Los resultados
importados conservan extensiones, campos opcionales y los bytes originales;
los límites de entrada no se aplican arbitrariamente a arrays de salida.

Las pruebas detectaron y corrigieron dos supuestos adicionales del WIP: una
orientación 3D añadida a una viga 2D es rechazada por Rust, y la resolución de
anotaciones recursivas necesita el namespace JSONValue en los módulos usuarios.
La prueba de orientación usa el fixture frame3d real. `Results.version` conserva
los contratos del documento productor, que incluyen element/1 y system/1;
no se fuerza a coincidir con la lista más breve de `version()`.

## Evidencia de ejecución offline

Toolchain stable Rust 1.99.0; `CARGO_TARGET_DIR=/workspace/.cargo-target/me-py3`.
Construcción mediante `MATURIN=/workspace/venv-maturin/bin/maturin` (1.15.0)
y `scripts/build-python-wheel.sh`, con dos builds release independientes,
instalación `PIP_NO_INDEX=1` en venv temporal dentro de CARGO_TARGET_DIR y tests
contra el paquete instalado. No hay publicación en PyPI.

No hay mypy ni pyright provisionados offline. La alternativa obligatoria
`test_typing.py` contrasta exports/reexports y firmas de los stubs instalados,
así como campos requeridos/opcionales de los TypedDict con los schemas Rust;
`get_type_hints` resuelve la superficie pública. No equivale a un checker completo.

Resultados finales:

| Comprobación | Resultado |
| --- | --- |
| Wheel reproducible, dos compilaciones release independientes | OK, bytes idénticos |
| Tests Python contra wheel instalada en CPython 3.13.5 | 29 passed / 0 failed |
| Tests de artefactos scripts/tests | 25 passed / 0 failed |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline (incluidos doctests) | 631 passed / 0 failed / 0 ignored |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | OK |
| Cruce nativo/wasm debug y release, diez corridas por caso | 16/16 hashes idénticos |
| Tabla publicada docs/DETERMINISMO.md y cmp debug/release | 16/16 sin cambios |
| git diff HEAD --check | OK |

Artefactos: `$CARGO_TARGET_DIR/eng-py-003-dist-verified/`, incluidos
REPRODUCIBILITY.json, SMOKE.json, COMPATIBILITY.json, SBOM y SHA256SUMS.
Wheel abi3 CPython ≥3.10, manylinux_2_34_x86_64, validada aquí con CPython 3.13.5;
no se afirma verificación de otros intérpretes o plataformas.
SHA-256 de la wheel:
`7e741382348caf24b7a7d46a206e80bfec98a66340caa8e27d21272b511e1d3e`.

Logs dentro de CARGO_TARGET_DIR: `eng-py-003-wheel-verified.log`,
`eng-py-003-{fmt,clippy,test,doc,wasm-check,clean-room}.log`,
`eng-py-003-determinism-{debug,release}.log` y
`eng-py-003-native-{debug,release}.txt`. Los primeros intentos de wheel
detectaron las regresiones de pruebas descritas arriba; el intento final
termina con las 29 pruebas en verde, sin omitirlas.

El directorio Git común es de solo lectura: no se modificó el índice original.
Para comprobar también los archivos nuevos todavía sin registrar se copió el
índice a `eng-py-003-review.index`, usando objetos temporales en
`eng-py-003-review-objects` y el almacén original como alternativo de lectura.
Con `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY` y
`GIT_ALTERNATE_OBJECT_DIRECTORIES` así configurados se registraron únicamente
intenciones de añadir la bitácora nueva y test_typing.py en esa copia, y se
ejecutó el mismo `bash clean-room/check.sh origin/main`: OK, log
`eng-py-003-clean-room-complete.log`. También se ejecutó en el índice original;
la bitácora de ENG-SFC-007 enlaza esta ampliación del adaptador PyO3.

## Seguimiento #143

- Ticket: MAM-79 · issue #143.
- Autor: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Rol: implementador; contratos `engine-input/1`, `results/1.input_sha256`
  y fachada Python. Revisor clean-room: **PENDIENTE (J4)**.
- Fuentes: solo el checkout propio, en particular `engine-api/src/{input,
  analysis_input,limits,json,lib}.rs`, el schema `engine-input-1.json`, los
  fixtures propios, la fachada Python y esta documentación. La premisa JCS /
  binary64 viene del ticket y del contrato ya documentado; sin consultas
  externas, código copiado ni dependencias nuevas; sin consultar ni modificar
  fuentes de vendor.

**Decisión: opción A, rechazo simétrico en el parseo.** Se revisaron todos los
campos enteros del schema y DTOs: ids de nodos/miembros y referencias (`nodes`,
`node`, `member`, `master`, `slaves`) son u32, máximo 4 294 967 295;
`max_iterations` y `mode_count` son usize limitados a 16 384, también el
producto modos por direcciones. Los demás ids (secciones, apoyos, masas,
combinaciones) y nombres son cadenas; los GDL son etiquetas. No se publican
seeds ni índices enteros adicionales en esta entrada. La política futura de
semilla `0_u64` en DETERMINISMO.md no es un campo de engine-input/1.
Ninguno de estos contratos necesita enteros superiores al rango seguro.

Derivación propia de la decisión: binary64 tiene 53 bits de precisión entera;
JCS convierte los números a esa representación. Restringir los tokens enteros
a `|n| <= 2^53-1` elimina la pérdida de identidad de enteros admitidos antes
de JCS. Documentar únicamente la colisión mantendría un fallo silencioso en
stale. Los campos físicos f64 también pueden escribirse como enteros: se
endurece deliberadamente su representación léxica fuera del rango seguro,
sin cambiar el dominio de los floats finitos. Los tokens con punto/exponente
siguen siendo binary64; distintas escrituras que redondeen al mismo f64
conservan la misma identidad canónica, no una identidad léxica.

El control reside en `engine-api/src/limits.rs`, inspección léxica común del
parseo estricto anterior a serde (`json::parse`), incluidos enteros gigantes
que serde podría redondear a float. `canonical_json`, `input_sha256`,
`validate`, `validate_with` y `run` comparten el control. Se reutiliza el error
tipado `ApiError::InvalidInput`, sin ampliar el enum público: en `run` produce
`API-E-INPUT`, salida 2, huella null y ningún callback de parseo. No se añade
unsafe, panic, unwrap ni expect al código de biblioteca.

Python aplica el mismo intervalo simétrico a ints / tokens JSON de entrada
antes del FFI y conserva la ruta en `InvalidInputError.details`. `bool` sigue
siendo JSON booleano y se rechaza para ids/contadores mediante tipos exactos.
Una mutación del modelo hacia enteros inseguros marca `Results` como stale.
`bounded=False` sigue reservado a resultados importados: sus extensiones no
reciben arbitrariamente los límites de entrada, incluida esta restricción.

El vector histórico JCS `9007199254740993` conserva sus hashes de referencia
y se marca `rejected_by_engine`; los tests ahora exigen su rechazo en la
fachada antes de huella. No cambian mamut-canon, vendor, los algoritmos de
canonización, las ecuaciones, los fixtures mecánicos ni los 16 hashes de
DETERMINISMO.md. Las pruebas numéricas usan exclusivamente los oráculos
analíticos propios existentes; no se usó ningún oráculo externo.

Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
para este ticket." Tampoco código ni resultados de CalculiX o LAPACK, ni
informes de motores externos. Trabajo offline, sin fuentes de terceros copiadas.

### Evidencia offline del seguimiento

Rust stable 1.99.0, `CARGO_TARGET_DIR=/workspace/.cargo-target/me-h53`;
`CARGO_NET_OFFLINE=true`, `PIP_NO_INDEX=1`. La wheel se construyó con
`MATURIN=/workspace/venv-maturin/bin/maturin bash scripts/build-python-wheel.sh
"$CARGO_TARGET_DIR/mam-79-143-dist"`, con salida nueva y dos builds release
independientes. Instalación y tests contra la wheel en CPython 3.13.5, sin
Rust en PATH durante el consumo. No se publicó ningún artefacto.

| Comprobación | Resultado final |
| --- | --- |
| Wheel offline, dos builds independientes | OK, bytes idénticos |
| Tests Python contra wheel instalada | 32 passed / 0 failed |
| Tests de artefactos scripts/tests | 25 passed / 0 failed |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline, incluidos doctests | 656 passed / 0 failed / 0 ignored |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | OK |
| Cruce nativo/wasm debug y release, diez corridas por caso | 16/16 idénticos |
| Comparación automática con tabla DETERMINISMO.md y bytes debug/release | 16/16 sin cambios |
| git diff --check | OK |

Las primeras pasadas detectaron una expectativa antigua de aceptación de
u64::MAX en canonical_json (actualizada al límite seguro) y cancelación
numérica de recuperación al probar el límite con una carga enorme en el
extremo libre. La prueba aplica ahora esa carga al apoyo: verifica parseo,
run y huella sin grandes desplazamientos; no cambia física ni tolerancias.
La suite completa se repitió después del ajuste y terminó en verde.

Artefactos: `$CARGO_TARGET_DIR/mam-79-143-dist/`, incluidos reproducibilidad,
compatibilidad, smoke, SBOM y SHA256SUMS. SHA-256 de la wheel:
`cc46e866bf838dd4c90e991492389cb1478e96d853940f34181f8a3f8c239dac`.
Logs: `$CARGO_TARGET_DIR/mam-79-143-{wheel,fmt,clippy,test,doc,wasm-check,
clean-room,determinism}.log`; resumen `mam-79-143-checks.json` y
`mam-79-143-native-{debug,release}.txt` con 160 filas cada uno. La comparación
con los hashes de la tabla se hizo por igualdad exacta, sin regenerarla.

## Pendientes

- Revisión clean-room independiente de J4.
- Tipado con mypy/pyright y matriz adicional de CPython/plataformas cuando esas
  herramientas estén provisionadas; la entrega incluye la alternativa offline.
- Sin cambios en rutas protegidas, política clean-room, workflows ni vendor.
