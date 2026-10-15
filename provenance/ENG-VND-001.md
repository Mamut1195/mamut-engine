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
