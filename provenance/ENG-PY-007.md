# Bitácora clean-room · ENG-PY-007

- Ticket / PR: MAM-85 · ENG-PY-007 «Wheels, CI, SBOM y distribución Python V1»; issue #94. Rama `j2/eng-py-007`, basada en `origin/main`.
- Autor(es): Codex (GPT-6), bajo dirección del usuario, 2026-10-15 (America/Guayaquil).
- Rol: implementador de empaquetado, sin cambios de métodos ni API del binding.
- Contrato implementado: construcción/consumo offline y artefactos de distribución local; referencias `engine-input/1` y `results/1` del engine, sin implementarlas en el scaffold Python (PY-003 PENDIENTE).
- Fuentes permitidas usadas: solicitud del usuario; archivos propios `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `bindings/python/**`, manifiesto de `engine-core`, contratos de `engine-api`/`engine-contracts`, `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md` y bitácora ENG-PY-002-FU; ayuda local de maturin 1.15.0 y metadatos locales de Cargo (incluidas expresiones de licencia). Sin consultar fuentes externas ni usar red.
- Derivaciones propias: scripts Bash/Python con biblioteca estándar; identidades de componentes independientes de las rutas de Cargo; pruebas de regresión y procedimiento de rollback. No hay derivaciones numéricas.
- Código de terceros copiado: ninguno. `LICENSE`/`NOTICE` existentes se verifican byte a byte, no se modifican. Los artefactos contienen metadatos declarativos de dependencias, no código copiado.
- Validación: reproducibilidad binaria, importación/llamada nativa, inspección ZIP/legal, regresiones de portabilidad del SBOM, controles Rust y rollback local; no es validación de un solver.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: **PENDIENTE (J4)**.
- Licencias, titularidad, aviso definitivo, licencia comercial y CLA: **PENDIENTE de revisión (ENG-GOV-004 / ENG-GOV-007)**. El inventario no constituye autorización para distribuir.

## Implementación

`scripts/build-python-wheel.sh` acepta `MATURIN` y `PYTHON`, exige
`CARGO_TARGET_DIR` exportado y una salida nueva. No depende de rutas de esta
máquina. Comprueba Rust contra `rust-toolchain.toml` (1.99.0), bloquea la
instalación automática de rustup, fija maturin 1.15.0 y
`SOURCE_DATE_EPOCH=1767225600` (2026-01-01 UTC). Ejecuta Cargo con
`--offline --locked`; los dos builds release/stripped no comparten target y
remapean rutas Rust. No instala herramientas ni publica.

El venv consumidor es nuevo; solo su `bin` aparece en PATH. pip usa
`--no-index --no-deps --no-cache-dir`. Se comprueba la ausencia de
`cargo`/`rustc`/`rustup`, la versión de distribución y las llamadas reales a
`mamut_engine.version()` y `_native.version()`. Esa es la única función actual;
no se añade análisis ni ninguna API nueva.

`scripts/python-wheel-artifacts.py` genera `SBOM.cdx.json` (CycloneDX 1.5),
`THIRD_PARTY_NOTICES`, `COMPATIBILITY.json`, `REPRODUCIBILITY.json`, `SMOKE.json`
y `SHA256SUMS` junto a la wheel. El SBOM usa el cierre transitivo desde
`mamut-engine-py`, no todo el workspace; incluye conservadoramente dependencias
Cargo de build y runtime para todos los destinos. No inventaría bibliotecas del
sistema ni herramientas. Se documenta esa limitación. No se utilizó un generador
SBOM externo: la generación propia solo requiere Cargo y biblioteca estándar.

**Closes #94:** el README Python ya no contiene rutas de workspace; las
identidades SBOM nunca reutilizan los package IDs absolutos de Cargo. Un chequeo
recursivo rechaza rutas del sistema y ocho pruebas cubren referencias absolutas,
reubicación y orden de metadata, colisiones de identidad, serialización,
generación de SBOM/notices, integridad de archivos legales y README empaquetado.
El inventario real tiene 22 componentes; `engine-core` carece de expresión de
licencia en su metadata y queda expresamente PENDIENTE, sin inferirla.

Para #93 se comprueba la presencia de `dist-info/licenses/LICENSE` y `NOTICE`,
su igualdad byte a byte con el binding y los campos `License-File` en METADATA.
Los avisos legales existentes permanecen sin cambios.

## Verificación local

Entorno real: Linux x86_64, CPython 3.13.5, rustc
`1.99.0 (b940084d7 2026-09-28)`, maturin 1.15.0. Para seleccionar la toolchain
ya instalada se antepuso su directorio `bin` local a PATH. Se conservó el
`CARGO_TARGET_DIR` exportado: `/workspace/.cargo-target/mamut-engine`.
Todas las operaciones Cargo usaron modo offline; ninguna requirió red.

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK, salida 0 |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | OK, salida 0 |
| `cargo test --offline --locked --workspace --no-fail-fast` | OK, salida 0; 545 pruebas/doctests, 0 fallidos, 0 ignorados |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace` | OK, salida 0 |
| `cargo check --offline --locked --workspace --target wasm32-unknown-unknown` | OK, salida 0 |
| `bash -n scripts/build-python-wheel.sh` | OK |
| Script completo, incluido unittest del binding y regresiones de artefactos | OK, salida 0 |
| `sha256sum --check SHA256SUMS` | OK para wheel y todos los artefactos |
| `bash clean-room/check.sh origin/main` | OK con archivos nuevos añadidos al índice |
| `git diff --cached --check` | OK |

Wheel: `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`.
Dos compilaciones desde cero produjeron exactamente el mismo SHA-256:
`92913abb592f3dfeeae8ceaef6f7611f58ea72318cbe74e41d4bf4bb24d263b6`.
La reproducibilidad acreditada es con las herramientas locales indicadas, no
entre diferentes SO/toolchains. El script falla e informa hashes y diferencias
ZIP si esa igualdad deja de cumplirse.

Salida final local: `/tmp/eng-py-007-dist-final/`. Registros locales:
`/tmp/eng-py-007-{fmt,clippy,test,doc,wasm}.log`,
`/tmp/eng-py-007-wheel-final.log` y `/tmp/eng-py-007-rollback.log`.
Wheels, artefactos y registros no se añaden al repositorio.

## Ensayo de rollback

Se archivaron fuentes propias de HEAD en `/tmp/eng-py-007-rollback-source` y
se cambió únicamente la versión del binding de esa copia a **0.1.1 sintética**;
`engine-core` siguió en 0.1.0. Se actualizó su lockfile con `cargo metadata
--offline` y se construyó con maturin `--offline --locked`, sin tocar las
versiones ni el lockfile del checkout de trabajo.

En otro venv limpio, con PATH limitado a su `bin`, se ejecutó:

1. Instalar wheel 0.1.0 y comprobar distribución 0.1.0 / engine 0.1.0.
2. Instalar wheel sintética 0.1.1 con `--force-reinstall`, comprobar distribución
   0.1.1 / engine 0.1.0.
3. Verificar `SHA256SUMS` anteriores y reinstalar wheel 0.1.0 con
   `--no-index --no-deps --no-cache-dir --force-reinstall`.
4. En un proceso Python nuevo, comprobar distribución 0.1.0 / engine 0.1.0,
   llamada nativa y ausencia de Rust en PATH.

Resultado: **OK**, `0.1.0 → 0.1.1 → 0.1.0`. La versión sintética solo ensaya el
mecanismo de sustitución; no prueba rollback de cambios de API/datos reales ni
representa una versión publicada. Procedimiento operativo en el README Python.

## Pendientes y límites

- Solo Linux x64 y CPython 3.13.5 acreditados aquí. Windows x64, macOS arm64 y
  CPython 3.11/3.12/3.14: PENDIENTE. Matriz candidata en README Python.
- `abi3-py310` ya estaba configurado; tag inspeccionado y carga en 3.13.5 probada.
  Compatibilidad abi3 entre intérpretes PENDIENTE de probar cada celda.
- CI de Actions PENDIENTE por presupuesto; runner VPS PENDIENTE de migración por J0.
- Revisión clean-room, licencias y CLA PENDIENTES. No se afirma distribución autorizada.
- Sin cambios en `docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` ni `vendor/**`.
  El directorio no rastreado `crates/engine-wasm/`, previo a este trabajo, queda intacto.
- Sin publicación, uploads ni push. Solo commit local solicitado por el usuario.

## J4 r37 — SBOM embebido y portabilidad de toda la wheel

Revisión del 2026-10-15, MAM-85 / PR #117. La wheel anterior incluía
`dist-info/sboms/mamut-engine-py.cyclonedx.json` generado automáticamente por
maturin, con referencias `path+file:///workspace/...` que el chequeo del SBOM
externo no alcanzaba.

Se consultaron offline `maturin --version`, `maturin --help` y
`maturin build --help` del ejecutable local 1.15.0. La ayuda ofrece
`--sbom-include`, pero no una opción CLI para desactivar la generación.
La configuración local se comprobó con un proyecto temporal: el parser de
maturin reconoce `sbom.rust` y `sbom.auditwheel` y rechaza valores de texto
porque exige booleanos. Se configuró en `[tool.maturin]`
`sbom = { rust = false, auditwheel = false }`; los builds reales confirman
que no hay miembros `dist-info/sboms/**`. No se reescribe el ZIP ni RECORD.
El SBOM portable propio sigue generándose junto a la wheel.

`check_wheel_portable` recorre `ZipFile.infolist()` y comprueba el nombre y
el contenido descomprimido de **todos** los miembros, incluidos RECORD,
todo `dist-info/**`, archivos legales y `_native.abi3.so`. Busca directamente
bytes de `/workspace`, `/home`, `/tmp`, `/root`, `file://`, `path+` y unidades
Windows con barra o contrabarra; no decodifica binarios ni descarta bytes.
El patrón Windows distingue una unidad de URLs como `https://`.
Una coincidencia aborta con miembro, posición y fragmento de la ruta.
El script ejecuta `check-wheel` para cada build antes de comparar hashes,
instalar o exponer la salida; `artifacts` también lo ejecuta al comenzar.

Se conserva y documenta el remapeo reproducible ya presente para rutas Rust
de panic/debuginfo: `--remap-path-prefix` transforma el checkout en `.`,
Cargo home en `cargo-home` y el directorio de cada build en `build`.
El script pasa estos RUSTFLAGS mediante `CARGO_ENCODED_RUSTFLAGS`, equivalente
de Cargo que preserva argumentos con espacios y evita depender de flags
heredados del entorno. El binario real pasa la búsqueda de bytes indicada.

Las 11 regresiones de artefactos pasan. La nueva wheel sintética comprueba
48 combinaciones: ocho rutas/patrones en METADATA, RECORD, SBOM anidado,
otro miembro anidado de dist-info, binario `.so` y datos del paquete.
Incluye NUL y bytes no UTF-8; cada combinación falla por la ruta inyectada.
También se prueba rechazo de nombres absolutos y aceptación de contenido
portable/URLs. La CLI devuelve salida 1 al inyectar una ruta en RECORD y
en `.so`. Ambas wheels reales pasan el mismo chequeo dentro del script.

Verificación r37, todo offline, con
`CARGO_TARGET_DIR=/workspace/.cargo-target/me45` exportado y PATH apuntando
a Rust 1.99.0 ya instalado:

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | OK |
| `cargo test --offline --locked --workspace --no-fail-fast` | OK; 545 pruebas/doctests, 0 fallidos |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace` | OK |
| `cargo check --offline --locked --workspace --target wasm32-unknown-unknown` | OK |
| `bash -n scripts/build-python-wheel.sh` | OK |
| Script completo: dos builds independientes, chequeos ZIP, venv y binding | OK |
| `sha256sum --check SHA256SUMS` | OK para todos los artefactos |
| Inspección de la wheel real | 9 miembros; sin SBOM embebido; `License-Expression: AGPL-3.0-only` |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --cached --check` | OK |

Wheel `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`;
SHA-256 idéntico en los dos builds independientes:
`3ab15573289adb3dd3109c9cbf0d1f2d51705b820bbd2fa5fe0db5cd6470841f`.
Salida local: `/tmp/eng-py-007-r37-dist/`; registros
`/tmp/eng-py-007-r37-{wheel,fmt,clippy,test,doc,wasm}.log`.
Se mantiene el alcance de reproducibilidad de la verificación previa:
herramientas y plataforma locales indicadas, sin afirmar igualdad entre
toolchains distintas. Sin cambios de licencia ni nuevos textos legales;
sin cambios en `docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` o `vendor/**`.
Sin red, publicación ni push.
