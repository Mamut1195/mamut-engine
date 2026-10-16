# Bitácora clean-room · ENG-VND-001

- Ticket: MAM-481 · ENG-VND-001; rama `j2/vnd-mamut-workflows` (PR #100).
- Fecha: 2026-10-14 (America/Guayaquil).
- Autor: Codex gpt-6.1-sol high bajo J2. Rol: implementador.
- Revisor clean-room: PENDIENTE (Jonathan, @Mamut1195).
- Origen: <https://github.com/Mamut1195/mamut-workflows>, repositorio propio
  de MAMPRO; fuente local `/workspace/mamut-workflows`, consultada en solo lectura.
- Origen fijado al commit de `main`:
  `585f32dc3455208566a0ffb894ad03ce3816b888`, resuelto mediante
  `git -C /workspace/mamut-workflows rev-parse 585f32d`.
  Versión declarada: `0.1.0`. El tag `v0.1.0` **no corresponde** a ese commit;
  apunta a `5c4e75af21564edc1a3b900d565a2ab90447ac1c`.
  El checkout fuente permanece limpio en el commit anterior; se consulta el
  nuevo commit con `git show`, sin modificar el repositorio ni su checkout.
- Licencia upstream y de ambos crates: Apache-2.0. LICENSE copiado exactamente.
  NOTICE redactado para esta distribución (upstream no trae NOTICE), copyright
  Mamut1195 / MAMPRO SAS; se conserva también el aviso original MAMUT de LICENSE.
- Declaración clean-room literal: "código propio de MAMPRO, sin OpenSees/Stabileo".
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

## Archivos y ajustes

Se copian `crates/mamut-canon` a `vendor/mamut-workflows/mamut-canon`
(SDK-WF-002, RFC 8785 + SHA-256), y `crates/mamut-workflow` a
`vendor/mamut-workflows/mamut-workflow` (SDK-WF-003, registro y compilación).
Inventario exacto con hashes: `vendor/mamut-workflows/MANIFEST.sha256`.
Todo `src/**/*.rs` y `tests/**/*.rs` permanece byte por byte igual al upstream.
No se añaden llamadas panic/unwrap/expect en biblioteca ni cambios de lógica.

Únicos archivos upstream modificados: los `Cargo.toml` de ambos crates.
Los campos `.workspace` se sustituyen por version 0.1.0, edition 2021,
license Apache-2.0, rust-version 1.83 y repository
https://github.com/Mamut1195/mamut-workflows. Se fija `readme = "../README.md"`,
`publish = false` y un comentario que señala la modificación y NOTICE.
En este commit no existen dependencias entre ambos crates, por lo que no hay
dependencias internas que convertir a path. Dependencias externas sin cambios.
No se necesitan allow de Clippy ni exclusiones de rustfmt.

Cargo.lock incorpora ambos paquetes locales 0.1.0, `json-canon 0.1.3` y su
dependencia transitiva `ryu-js 0.2.2`, disponibles en el registro local.
Se reutilizan `sha2 0.10.9`, `serde 1.0.229` y `serde_json 1.0.154` existentes;
no se actualiza ninguna versión previamente fijada.

Archivos nuevos de distribución: `vendor/mamut-workflows/NOTICE`, `README.md`
y `MANIFEST.sha256`; `LICENSE` proviene exactamente de la raíz upstream.
Integración propia: `Cargo.toml` raíz (ambos crates en members/default-members),
`Cargo.lock` actualizado offline, `scripts/check-vendor.sh` y esta bitácora.
El script verifica inventario completo y SHA-256, excluyendo solo el manifiesto
de sí mismo.

## Decisión de Jonathan (2026-10-14)

Jonathan autoriza y fusiona la opción (a): excluir exclusivamente las rutas
canónicas bajo `vendor/mamut-workflows/` del escaneo de identificadores en
`clean-room/check.sh`. La comprobación de rutas prohibidas sigue aplicándose.
Si el diff toca vendor, el código confiable del validador comprueba inline
manifiesto presente y no vacío, inventario completo (también extras ignorados),
SHA-256 estricto, commit `585f32dc3455208566a0ffb894ad03ce3816b888` en README
o NOTICE y LICENSE presente. Rechaza symlinks y rutas no canónicas, sin ejecutar
scripts del PR. Se conserva `scripts/check-vendor.sh` como control adicional.

Se añaden pruebas de la excepción y sus límites en `clean-room/tests/run.sh`.
`/vendor/mamut-workflows/` se asigna a @Mamut1195 en `.github/CODEOWNERS`,
los mismos propietarios de `clean-room/**`, para revisión de cambios conjuntos
de archivos y manifiesto. La aprobación y fusión corresponden a Jonathan.
No se modifican workflows ni `docs/CLEAN_ROOM.md`.

En la primera actualización se modificó `vendor/mamut-workflows/README.md`,
listado en el manifiesto, y se regeneró `MANIFEST.sha256`: solo cambió el hash
del README. Los otros archivos listados, incluido todo el código vendorizado,
permanecieron intactos en esa actualización.

## Cambio de origen por decisión de J0/Jonathan (2026-10-14)

Historia: el origen inicial fue `5c4e75af21564edc1a3b900d565a2ab90447ac1c`
(tag `v0.1.0`); J0/Jonathan ordenó fijarlo después a
`585f32dc3455208566a0ffb894ad03ce3816b888` (`main`), aunque la versión declarada
continúa siendo `0.1.0`. El tag anterior emite `jcs-rfc8785` en vez de
`rfc8785-jcs`, exigido por el contrato, y su manifiesto no cumple
`execution-manifest/1`. El nuevo commit corrige ambos problemas upstream.
Las correcciones del emisor del manifiesto están fuera de los dos crates
vendorizados: no se incorpora el ejecutor ni se amplía el alcance.

Se compararon todos los archivos de `crates/mamut-canon`,
`crates/mamut-workflow` y LICENSE mediante `git show 585f32d:<ruta>`.
Solo difiere `crates/mamut-canon/src/lib.rs`: se copia verbatim, incluida la
nueva prueba del contrato de fingerprint. LICENSE y los otros archivos Rust
permanecen idénticos. Los Cargo.toml upstream no cambiaron y se conservan
sus ajustes de distribución documentados arriba. No cambia Cargo.lock.

Se actualizan README, NOTICE y esta bitácora; se regenera MANIFEST.sha256
(cambian los hashes de README, NOTICE y mamut-canon/src/lib.rs).
La comprobación inline exige ahora el hash completo nuevo. Sus pruebas
incluyen el rechazo del commit anterior y del hash abreviado, aun con
inventario y SHA-256 válidos.

MAM-490: no parchear tests vendorizados que presupongan `../../target` y fallen
con `CARGO_TARGET_DIR` exportado. En estos dos crates no se encontró ese supuesto;
el resultado de ejecutar las pruebas con el directorio exportado se registra
en la verificación del nuevo origen al final de esta bitácora.

## Verificación inicial del vendorizado (histórica, 5c4e75a)

Se utiliza el CARGO_TARGET_DIR heredado
`/workspace/.cargo-target/mamut-engine`, `CARGO_NET_OFFLINE=true`
y el toolchain local Rust 1.99.0 (`PATH=/home/box/.cargo/bin:$PATH`).
Sin consultas de red y sin modificar el repositorio fuente.

- `cargo fmt --all -- --check`: OK, sin excepciones.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 491 pruebas, 0 fallos/ignoradas;
  17 de los crates vendorizados (canon: 6, workflow: 6, two_step: 5).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK,
  incluidos ambos crates vendorizados.
- `cargo metadata --offline --locked --no-deps --format-version 1`: OK;
  ambos crates en members/default-members, publish desactivado y README local.
- Comparación byte por byte de los 10 archivos Rust y LICENSE con upstream: OK;
  checkout fuente permanece limpio en el commit indicado.
- `bash scripts/check-vendor.sh` y `bash -n scripts/check-vendor.sh`: OK.
  En una copia temporal, el control rechaza un archivo alterado, uno añadido
  y uno eliminado; la copia intacta pasa antes y después de esas simulaciones.
- `git diff --cached --check`: señala únicamente una línea vacía final en
  `vendor/mamut-workflows/LICENSE:202`, heredada de upstream; se conserva
  para cumplir la copia exacta de la licencia.
- El bloqueo previo por las cuatro apariciones de `algorithm` en el código
  upstream de `mamut-canon` se resuelve con la opción (a) autorizada arriba.
  Revisión humana PENDIENTE (Jonathan).

## Verificación anterior de MAM-481 (2026-10-14, histórica, 5c4e75a)

Con `CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine` exportado,
`CARGO_NET_OFFLINE=true` y el mismo toolchain local, sin red:

- `bash clean-room/tests/run.sh`: OK, 35 pruebas, 0 fallos. Cubre integridad,
  metadatos, symlinks, eliminación completa, manifiestos inválidos, escapes,
  scripts no confiables y los límites de la exclusión de identificadores.
- `bash clean-room/check.sh origin/main`: OK, incluidos los hashes inline.
- `bash scripts/check-vendor.sh`: OK; solo cambia el hash del README.
- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash -n clean-room/check.sh clean-room/tests/run.sh` y `git diff --check`: OK.

## Verificación del nuevo origen 585f32d (2026-10-14)

Origen: `585f32dc3455208566a0ffb894ad03ce3816b888`.
Se exportan `CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine`,
`CARGO_NET_OFFLINE=true` y `PATH=/home/box/.cargo/bin:$PATH`.
Toolchain local: cargo 1.99.0 / rustc 1.99.0. Todas las comprobaciones sin red:

- `bash clean-room/tests/run.sh`: OK, 37 pruebas, 0 fallos; incluye el rechazo
  del commit anterior y del hash abreviado.
- `bash clean-room/check.sh origin/main`: OK, incluido el control inline.
- `bash scripts/check-vendor.sh`: OK, inventario y hashes completos.
- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 526 pruebas incluidas las de
  documentación, 0 fallos/ignoradas; 18 de los crates vendorizados
  (canon: 7, workflow: 6, two_step: 5), incluida la nueva prueba del fingerprint.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash -n clean-room/check.sh clean-room/tests/run.sh scripts/check-vendor.sh`
  y `git diff --check`: OK.
- Comparación de los 13 archivos upstream (10 Rust, 2 Cargo.toml, LICENSE):
  OK contra `git show 585f32d:<ruta>`, salvo los ajustes de distribución
  documentados de los dos Cargo.toml. LICENSE idéntico; Cargo.lock sin cambios.
- Checkout upstream limpio y sin cambios, todavía en
  `5c4e75af21564edc1a3b900d565a2ab90447ac1c`; `main` resuelve al nuevo origen.
- MAM-490 no reproducido con `CARGO_TARGET_DIR` exportado. Los dos crates
  incluidos no presuponen `../../target`; no se parcheó ningún test vendorizado.

## Re-fijado a 4cb9ecd (2026-10-14, J0)

Origen vigente: `4cb9ecd0e1117d86444149b17a6b34d1283d9455` (main de
mamut-workflows, PR #5: CI en VPS y corrección de MAM-490). J0 indicó que el
código de las librerías es el mismo que en 585f32d; se comprobó:
`git diff --stat 585f32d 4cb9ecd -- crates/mamut-canon crates/mamut-workflow LICENSE`
vacío, y `cmp` byte a byte de cada fichero vendorizado (salvo los `Cargo.toml`
adaptados) y de LICENSE contra `git show 4cb9ecd:<ruta>`: idénticos. Solo
cambian README.md, NOTICE (procedencia), MANIFEST.sha256 (por esos dos) y el
commit exigido en `clean-room/check.sh` y sus pruebas. Edición documental hecha
por J2 (sin Codex: no hay código nuevo).

## FU J4 r28 (2026-10-14)

MAM-481 · ENG-VND-001-FU, rama `j2/vnd-001-fu`, seguimiento de PR #100.
Cambio en `clean-room/**` autorizado; Jonathan revisa y fusiona.
Esta sección sustituye la exclusión amplia documentada en las decisiones
históricas anteriores. No se modifican `.github/**` ni `docs/CLEAN_ROOM.md`.

- NOTICE propio: se sustituye el identificador prohibido `fix` por
  `correction`. Se revisa y actualiza README propio sin introducir
  identificadores prohibidos, y se regenera MANIFEST.sha256: solo cambian
  los hashes de NOTICE y README. Ningún archivo upstream se modifica.
  **Nota (2026-10-15):** estas ediciones de NOTICE/README y la regeneración
  del MANIFEST se descartaron al integrar #127 y nunca entraron en `main`;
  `vendor/` queda exactamente como en #127 y la política de excepción se
  documenta solo en esta bitácora (ver «Integración de #110 y #127»).
- `clean-room/check.sh` mantiene una lista explícita con una sola pareja:
  `vendor/mamut-workflows/mamut-canon/src/lib.rs:algorithm`.
  No se permiten otros identificadores en ese archivo ni ese identificador
  en otras rutas. Se conserva el control de rutas prohibidas.
- Si el diff toca la distribución, se verifica inline su integridad y después
  se escanean todos sus archivos físicos y todas sus líneas, incluidas las
  heredadas sin cambios, las últimas sin salto final y los archivos ignorados
  o no rastreados presentes en el manifiesto. Se mantienen inventario,
  SHA-256 estricto, rechazo de symlinks y rutas no canónicas, LICENSE y commit
  completo `4cb9ecd0e1117d86444149b17a6b34d1283d9455`, sin ejecutar scripts del PR.
- Escaneo independiente completo con la lista vigente: cinco apariciones de
  `algorithm`, todas en la ruta permitida; cero hallazgos adicionales tras
  corregir NOTICE. No se amplía la excepción por otros nombres upstream.
- Comparación contra `git show 4cb9ecd:<ruta>` en la fuente local de solo
  lectura: los 10 archivos Rust y LICENSE siguen idénticos byte por byte.
  Los dos Cargo.toml adaptados y Cargo.lock tampoco cambian.

Verificación sin red con `CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine`
exportado, `CARGO_NET_OFFLINE=true`, `PATH=/home/box/.cargo/bin:$PATH`
y cargo/rustc 1.99.0:

- `bash clean-room/tests/run.sh`: OK, 55 pruebas, 0 fallos; incluye todas
  las anteriores, la pareja permitida, rechazos en otras rutas del vendor,
  otro identificador en la ruta permitida, NOTICE/README y contenido fuera
  del diff (también ignorado o no rastreado con hashes válidos).
- `bash clean-room/check.sh origin/main`: OK, escaneo completo del vendor
  y verificación inline del manifiesto y commit.
- `bash scripts/check-vendor.sh`: OK, inventario y hashes completos.
- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace --no-fail-fast`: OK, 538 pruebas,
  incluidas las de documentación, 0 fallos/ignoradas.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash -n clean-room/check.sh clean-room/tests/run.sh scripts/check-vendor.sh`
  y `git diff --check`: OK.


## Re-vendorizado a 1eeb91a · MAM-481 / MAM-493 (2026-10-15)

- Fecha: 2026-10-15 (America/Guayaquil).
- Autor: Codex gpt-6.1-sol high. Rol: implementador.
- Rama: `j2/eng-vnd-001-wf-1eeb91a`, ya creada y limpia al comenzar,
  desde `c290b10984b9f7d929e4bca4c17e6481fe4a670a` (origin/main).
- Origen: `1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`, commit de main de
  mamut-workflows, PR #6 y #7; SDK-WF-012 / MAM-493, puntos de extensión CLI
  requeridos por SFC-002. Este es el origen vigente de la distribución.
- Historia: `5c4e75a → 585f32d → 4cb9ecd → 1eeb91a`; los SHA completos y
  decisiones previas se conservan en las entradas históricas y README/NOTICE.
- Fuente local `/workspace/mamut-workflows`, solo lectura. Cada copia se obtiene
  **siempre** de `git -C /workspace/mamut-workflows show 1eeb91a:<ruta>`;
  no se usa el árbol de trabajo. Estado upstream limpio antes y después.
- Licencia: Apache-2.0; LICENSE idéntico al blob del commit fijado.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Sin red, sin commit ni push en el Engine. No se modifican `crates/`, bindings
  ni `.github/`. Revisión del dueño clean-room/vendor: PENDIENTE (Jonathan).

Se incluyen íntegramente `mamut-canon`, `mamut-workflow`,
`mamut-workflow-execution` y `mamut-workflow-cli`, con todos sus archivos,
README propio, tests y snapshots. Los cuatro son members/default-members.
Se excluyen deliberadamente `mamut-workflow-run` (binario/fachada) y
`mamut-tool-conformance` (herramienta de conformidad), ajenos a este alcance
bibliotecario y sin ser dependencias de los cuatro incluidos.

Únicas modificaciones respecto a upstream: los cuatro Cargo.toml. Se usan
literales version 0.1.0, edition 2021, license Apache-2.0, rust-version 1.83 y
repository https://github.com/Mamut1195/mamut-workflows; se fija
readme="../README.md", publish=false y un comentario de modificación.
Las dependencias internas ya son path entre hermanos, sin cambios.
Se comparó el TOML parseado de cada manifiesto contra upstream sustituyendo
únicamente esos campos: todos los demás campos y dependencias son idénticos.
Cargo.lock se actualizó mediante `cargo check --workspace --offline`:
solo añade los dos paquetes locales, sin cambiar dependencias externas.
Toda la caché necesaria estaba disponible.

No se excluyen tests ni se parchea Rust. `tests/extensions.rs` de la CLI usa
rutas literales hacia `examples/open-local/{catalog,workflow}.json` de la raíz
upstream. Se copian ambos blobs a `vendor/mamut-workflows/examples/open-local/`,
cubiertos por el manifiesto, y a `vendor/examples/open-local/`, donde resuelven
las rutas originales. Ambos verificadores comparan esas copias con los blobs
cubiertos por el ancla y rechazan enlaces en sus rutas. Los cambios solo en
`vendor/examples/` también activan verify_workflows; no se amplía la excepción
al escaneo de identificadores. No se necesita el binario mamut-workflow.
MAM-490 no reproducido: los tests nuevos usan CARGO_TARGET_TMPDIR, y se
verifican con CARGO_TARGET_DIR fuera del árbol.

Se actualizan README/NOTICE y el inventario completo (46 archivos), además de
scripts/check-vendor.sh, que exige PINNED_COMMIT completo en ambos documentos.
J4 r31: clean-room/check.sh fija el SHA-256 del propio MANIFEST.sha256 en
EXPECTED_WORKFLOWS_MANIFEST_SHA256:
`214b71529137733aa14babaefa3eac67365f94bd986407ea7e7218bf68174c68`.
Se conservan inventario físico, ausencia de enlaces, rutas canónicas, hashes
estrictos, commit completo y LICENSE. El ancla debe revisarse en el validador
confiable de la base; el resultado local del script del árbol no constituye
aprobación del cambio por el dueño.

Las pruebas clean-room usan distribuciones sintéticas con un verificador de
base que fija su manifiesto inicial, sin actualizarlo tras mutaciones. Se
comprueba que un payload alterado con manifiesto regenerado y hashes coherentes
falla. Los casos de commit anterior/abreviado/ausente fijan expresamente el
manifiesto sintético correspondiente para comprobar también el rechazo por
metadatos de forma independiente. Se añaden casos de fixture externo alterado,
cambio solo externo y directorio enlazado. Continúan pasando los controles de
inventario, rutas, enlaces, licencia y scripts no confiables.

### Comparación verbatim, archivo por archivo

Se obtiene cada blob con `git -C /workspace/mamut-workflows show 1eeb91a:<ruta>`
a un archivo temporal bajo /tmp y se ejecuta `cmp -- <temporal> <destino>`.
La tabla registra el código de salida real de cada comparación: **42 archivos
idénticos**, incluidos los dos fixtures duplicados. También se comparó el
inventario de cada crate contra `git ls-tree -r --name-only 1eeb91a`: sin
archivos omitidos ni extras. Los cuatro Cargo.toml son las únicas excepciones
verificadas arriba; ningún archivo Rust se reformateó.

| Ruta upstream en 1eeb91a | Destino en el Engine | cmp |
| --- | --- | --- |
| `crates/mamut-canon/src/lib.rs` | `vendor/mamut-workflows/mamut-canon/src/lib.rs` | 0 |
| `crates/mamut-workflow/src/catalog.rs` | `vendor/mamut-workflows/mamut-workflow/src/catalog.rs` | 0 |
| `crates/mamut-workflow/src/compiler.rs` | `vendor/mamut-workflows/mamut-workflow/src/compiler.rs` | 0 |
| `crates/mamut-workflow/src/contract.rs` | `vendor/mamut-workflows/mamut-workflow/src/contract.rs` | 0 |
| `crates/mamut-workflow/src/descriptor.rs` | `vendor/mamut-workflows/mamut-workflow/src/descriptor.rs` | 0 |
| `crates/mamut-workflow/src/error.rs` | `vendor/mamut-workflows/mamut-workflow/src/error.rs` | 0 |
| `crates/mamut-workflow/src/lib.rs` | `vendor/mamut-workflows/mamut-workflow/src/lib.rs` | 0 |
| `crates/mamut-workflow/src/registry.rs` | `vendor/mamut-workflows/mamut-workflow/src/registry.rs` | 0 |
| `crates/mamut-workflow/src/workflow.rs` | `vendor/mamut-workflows/mamut-workflow/src/workflow.rs` | 0 |
| `crates/mamut-workflow/tests/two_step.rs` | `vendor/mamut-workflows/mamut-workflow/tests/two_step.rs` | 0 |
| `crates/mamut-workflow-execution/src/cli_run.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/cli_run.rs` | 0 |
| `crates/mamut-workflow-execution/src/executor.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/executor.rs` | 0 |
| `crates/mamut-workflow-execution/src/exit_codes.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/exit_codes.rs` | 0 |
| `crates/mamut-workflow-execution/src/lib.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/lib.rs` | 0 |
| `crates/mamut-workflow-execution/src/manifest.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/manifest.rs` | 0 |
| `crates/mamut-workflow-execution/src/mcp.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/mcp.rs` | 0 |
| `crates/mamut-workflow-execution/src/method.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/method.rs` | 0 |
| `crates/mamut-workflow-execution/src/observe.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/observe.rs` | 0 |
| `crates/mamut-workflow-execution/src/registered.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/registered.rs` | 0 |
| `crates/mamut-workflow-execution/src/tests.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/tests.rs` | 0 |
| `crates/mamut-workflow-cli/README.md` | `vendor/mamut-workflows/mamut-workflow-cli/README.md` | 0 |
| `crates/mamut-workflow-cli/src/config.rs` | `vendor/mamut-workflows/mamut-workflow-cli/src/config.rs` | 0 |
| `crates/mamut-workflow-cli/src/lib.rs` | `vendor/mamut-workflows/mamut-workflow-cli/src/lib.rs` | 0 |
| `crates/mamut-workflow-cli/src/parser.rs` | `vendor/mamut-workflows/mamut-workflow-cli/src/parser.rs` | 0 |
| `crates/mamut-workflow-cli/tests/extensions.rs` | `vendor/mamut-workflows/mamut-workflow-cli/tests/extensions.rs` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/compile.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/compile.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/describe.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/describe.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/help.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/help.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/methods.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/methods.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/missing.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/missing.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run-document.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run-document.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run-method.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run-method.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run-usage.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run-usage.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run-validation.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run-validation.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/unknown.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/unknown.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/validate.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/validate.txt` | 0 |
| `LICENSE` | `vendor/mamut-workflows/LICENSE` | 0 |
| `examples/open-local/catalog.json` | `vendor/mamut-workflows/examples/open-local/catalog.json` | 0 |
| `examples/open-local/catalog.json` | `vendor/examples/open-local/catalog.json` | 0 |
| `examples/open-local/workflow.json` | `vendor/mamut-workflows/examples/open-local/workflow.json` | 0 |
| `examples/open-local/workflow.json` | `vendor/examples/open-local/workflow.json` | 0 |

### Resultados de verificación offline

Toolchain local Rust/cargo 1.99.0; PATH=/home/box/.cargo/bin:$PATH,
CARGO_NET_OFFLINE=true, CARGO_TARGET_DIR=/tmp/eng-vnd2-target.
El target wasm32-unknown-unknown está instalado. Todos los comandos Cargo que
resuelven dependencias se ejecutan con --offline; sin descargas ni dependencias
faltantes. Logs de esta sesión: /tmp/eng-vnd2-*.log (no se incorporan al repo).

- `cargo fmt --check` y `cargo fmt --all -- --check`: OK. Sin exclusiones.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: OK, 597 tests, 0 fallos/ignorados,
  incluidos doctests. Vendor: canon 7, workflow 6 + two_step 5, execution 10,
  CLI 1 + extensions 16 + doctest 1 (46 en total).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `bash scripts/check-vendor.sh`: OK, inventario de 46 archivos, hashes,
  commit y copias externas de fixtures.
- `bash clean-room/tests/run.sh`: OK, 41 pruebas, 0 fallos.
- `bash clean-room/check.sh origin/main`: OK. Ejecutado expresamente con
  el check.sh del árbol modificado, contra origin/main=c290b10, como solicitado;
  no con la versión previa del validador obtenida de la base.
- `cargo check --workspace --target wasm32-unknown-unknown --offline`: OK.
- `cargo metadata --offline --locked --no-deps --format-version 1`: OK;
  cuatro members/default-members, publish=false, metadatos y readme local.
- Determinismo: `cargo build --offline -p engine-api --example determinism_wasm
  --target wasm32-unknown-unknown`, `cargo run --offline -p engine-api
  --example determinism_native` y `node scripts/check-determinism.mjs
  /tmp/eng-vnd2-target/wasm32-unknown-unknown/debug/examples/determinism_wasm.wasm
  /tmp/eng-vnd2-native.txt`: OK, 16 casos, diez corridas por caso,
  SHA-256 idénticos entre nativo y wasm (perfil debug).
- `bash -n clean-room/check.sh clean-room/tests/run.sh scripts/check-vendor.sh`
  y `git diff --check`: OK.


## Integración de #110 y #127 · J4 r28 / r31 (2026-10-15)

La distribución vigente es la de #127, fijada a
`1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`. En esta integración no se modifica
ningún archivo de `vendor/`, incluidos NOTICE, README y MANIFEST.sha256.
Las referencias a una sola pareja y a cambios de textos propios en la entrada
FU J4 r28 anterior describen la distribución histórica de 2026-10-14.

Se traslada aquí la política de excepción que #110 documentaba en NOTICE/README:
`clean-room/check.sh` escanea todos los archivos físicos y todas las líneas de
la distribución cuando el diff la toca, incluidas las líneas heredadas sin
cambios, las últimas sin salto final y los archivos ignorados o no rastreados.
Antes del escaneo se verifican inline inventario completo, SHA-256 estricto,
ausencia de enlaces, rutas canónicas, LICENSE, commit completo y el ancla
`EXPECTED_WORKFLOWS_MANIFEST_SHA256` de #127:
`214b71529137733aa14babaefa3eac67365f94bd986407ea7e7218bf68174c68`.
También se mantienen la comprobación de los examples externos y
`scripts/check-vendor.sh`; nunca se ejecutan scripts del PR desde el validador
confiable. Las rutas prohibidas siguen comprobándose.

La única excepción son estas **4 parejas literales ruta:identificador**:

- `vendor/mamut-workflows/mamut-canon/src/lib.rs:algorithm`
- `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run.txt:algorithm`
- `vendor/mamut-workflows/mamut-workflow-execution/src/mcp.rs:algorithm`
- `vendor/mamut-workflows/mamut-workflow-execution/src/tests.rs:algorithm`

El nombre corresponde al algoritmo SHA-256 de los fingerprints, sus pruebas y
su snapshot CLI. No se permiten otros identificadores en esas rutas ni ese
identificador en otras rutas; no se aceptan prefijos ni globs. Un escaneo con
`rg --hidden --no-ignore --text` y límites de identificador ASCII, usando toda
la lista `clean-room/forbidden_identifiers.txt` sobre `vendor/`, confirma que
solo aparece `algorithm`, exactamente en esas cuatro rutas. No hay hallazgos
adicionales ni se amplía la lista por otros nombres upstream.

La atribución y la historia del origen permanecen en los NOTICE/README de #127.
`CODEOWNERS` mantiene al mismo dueño para la distribución y `clean-room/**`
(@Mamut1195, Jonathan); la revisión humana y la fusión siguen pendientes.

Verificación de esta integración, sin red, con cargo/rustc 1.99.0,
`PATH=/home/box/.cargo/bin:$PATH`, `CARGO_NET_OFFLINE=true` y
`CARGO_TARGET_DIR=/tmp/eng-vndfu-target`:

- `bash clean-room/tests/run.sh`: OK, 73 pruebas, 0 fallos. Conserva los casos
  de #127 (examples, commit y ancla), e incorpora las cuatro parejas exactas,
  rechazos por otros archivos, prefijos y globs, otros identificadores en las
  cuatro rutas y contenido sin cambios, sin salto final, ignorado o no rastreado.
  Los negativos de identificadores exigen su diagnóstico y una integridad válida;
  no se actualiza el ancla después de las mutaciones del caso de ataque r31.
- `bash clean-room/check.sh origin/main`: OK, con el script del árbol.
- `bash scripts/check-vendor.sh`: OK.
- `cargo fmt --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: OK, 610 pruebas incluidos doctests,
  0 fallos/ignoradas.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `git diff MERGE_HEAD -- vendor`: vacío; distribución íntegra de #127.
- `verify_workflows` y el ancla se compararon contra #127: idénticos.
  Ambas entradas históricas de procedencia se conservaron completas.

Logs de Cargo y verificadores en `/tmp/eng-vndfu-*.log`, fuera del repositorio.
El merge quedó commiteado en `9c985da` (push sin force a `j2/vnd-001-fu`).
