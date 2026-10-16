# Bitácora clean-room · ENG-CORE-005 · MAM-14

- Ticket: ENG-CORE-005 «Esquema de resultados results/1 y procedencia».
- Rama: j2/eng-core-005, apilada sobre j2/vnd-mamut-workflows / PR #100.
- Autor: Codex (GPT-6), bajo dirección de J2, 2026-10-14 America/Guayaquil.
- Rol: implementador. Revisor clean-room: PENDIENTE (J4).
- Fuentes permitidas: requisitos del usuario; contratos, documentación, fixtures
  y APIs propios de crates/engine-api y engine-core; API pública y documentación
  de mamut-canon v0.1.0 vendorizado en #100 (SDK-WF-002); estructura declarativa
  JSON Schema Draft 2020-12 y RFC 8785 tal como los describe el SDK local.
  Sin red, repositorios hermanos, otros motores ni oráculos numéricos externos.
- Código de terceros copiado: ninguno. El SDK ya está vendorizado por #100 y
  se consume por dependencia path; vendor/** permanece verbatim y sin cambios.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

## Comparación previa y alcance incremental

El `main` local (b744599) está atrasado y aún no contiene engine-api. Se comparó
con origin/main (d0031f8, que incluye #88/#95), sin fetch ni acceso a red.
Antes del trabajo, crates/engine-api no tenía diferencias con origin/main.
La base de la rama es 49f4609, merge de origin/main en #100.

Ya existían schema results/1, bloques nodes/members/modes/cases/envelopes/
convergence/diagnostics y combinations, metadatos engine_version/contracts/
methods/input_sha256, salida canónica, manejo de errores/cancelación y todos los
métodos estático/P-Delta/modal/espectro. No se reimplementa el análisis.

Cambios de este ticket:

1. La huella usa `mamut_canon::fingerprint_value` sobre el Value del parseo
   estricto previo (límites y rechazo de duplicados/overflow conservados).
   Sin canonicalización propia para input_sha256.
2. Metadatos git_commit (revisión hexadecimal de 40/64 caracteres, env opcional
   MAMUT_ENGINE_GIT_COMMIT, null si ausente/inválida) y target (TARGET de Cargo).
   build.rs solo lee env, sin git/red/relojes. Se emiten también en errores y
   en el sobre constante de error interno. El schema admite ambos como campos
   opcionales para aceptar resultados previos; sin cambiar required existentes.
3. API Rust `Outcome::query(&ResultQuery)` con IDs tipados, conjuntos ordenados
   para búsqueda y rutas de campos. None = todo, Some vacío = nada. Filtra
   identidades y respuestas de casos/combinaciones/modos/espectro/envelopes;
   conserva orden, geometría referenciada, diagnósticos y procedencia, sin
   recalcular agregados. Proyección por puntos a través de arrays, conservando
   id; fragmentos proyectados no prometen satisfacer el schema completo.
4. Pruebas nativas validan salidas contra el schema local, incluidos avisos,
   fallos, convergencia y cancelación. El helper solo implementa el subconjunto
   Draft 2020-12 usado, con referencias exclusivamente locales y los tres
   patrones publicados; falla ante keywords/patrones desconocidos. Pruebas de
   mutación negativa verifican su capacidad de detectar incumplimientos. Se
   compila el mismo código de pruebas para wasm32; no se ejecuta wasm.

Los campos nuevos del contrato/API usan ^[a-z][a-z0-9_]*$. No se añadieron
definiciones ni excepciones a LEGACY_DEFS (congelada por MAM-480).

## Cambio de huella y compatibilidad de bytes

RFC 8785/JCS usa representación ECMAScript/binary64 y orden de claves UTF-16.
El hash puede cambiar para la misma entrada respecto de #88: `1.0` pasa a `1`,
`-0.0` a `0`, algunos exponentes cambian de forma, enteros mayores que 2⁵³ se
redondean a binary64 y claves suplementarias cambian de orden.
fixtures/results/fingerprints.json fija bytes JCS y hashes anteriores/nuevos:
para {"v":1.0}, el hash propio
f3c8be97307d38261061fa08e74365e54a239cddb1d189f3bea18022adbe2f35 pasa a
afbf9d0f3560b0fd7795e81c42a0a79ee6b6fc67e064f77826aee642cad28d91.
Los bytes explícitos de estos vectores son oráculos declarativos propios;
SHA-256 de esos bytes se calculó localmente con hashlib, sin red.
Se actualizaron las expectativas SHA de los tests de run y dinámica al SDK;
se comprueba igualdad entre entradas mecánicas con enteros/decimales,
permutación de claves, espacios y ceros negativos.

results/1 y la función pública canonical_json conservan su serializador propio
publicado: claves por orden escalar, floats round-trip, enteros exactos y
-0.0 → 0.0. Usar JCS ahí cambiaría bytes/tipos numéricos previamente publicados;
el SDK se aplica a la huella, sin migrar ese contrato de salida. Se conservan
los tests de determinismo/canonical_json sobre results/1.

## Cargo y pendientes

- engine-api sustituye la dependencia directa sha2 =0.10.9 por mamut-canon
  path ../../vendor/mamut-workflows/mamut-canon, versión =0.1.0.
- Cargo.lock cambia solo las dependencias de engine-api; no agrega paquetes ni
  versiones del registro. SHA-256 sigue disponible transitivamente por el SDK.
- Sin cambios adicionales a Cargo.toml del workspace ni a vendor/**.
- CLI/schema por CLI: PENDIENTE de ENG-SFC-002; todavía no existe CLI.
- El bloqueo inicial de #100 en clean-room/check.sh quedó resuelto en la
  dependencia antes del merge 3d10b33; se vuelve a verificar abajo sin modificar
  script/listas ni el contenido vendorizado.

## Verificación inicial (antes del merge 3d10b33)

Comandos offline con CARGO_TARGET_DIR exportado en
/workspace/.cargo-target/mamut-engine y toolchain de /home/box/.cargo/bin.

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 525 pruebas, 0 fallos/ignoradas.
  engine-api: 87 pruebas, incluidas 7 nuevas en tests/results.rs. Las 30 pruebas
  existentes de run/dynamic ahora validan cada Outcome contra el schema real.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --offline -p engine-api --tests --target wasm32-unknown-unknown`:
  OK; valida compilación portable de los tests, sin ejecución de wasm ni CLI.
- Prueba de metadatos con MAMUT_ENGINE_GIT_COMMIT=0123456789abcdef0123456789abcdef01234567:
  OK; comprobaciones completas posteriores con la env opcional ausente.
- `bash scripts/check-vendor.sh`: OK, manifiesto SHA intacto.
- `git diff --check`: OK; sin panic/unwrap/expect en biblioteca/build.rs.
  Sin cambios a docs/CLEAN_ROOM.md, .github/**, clean-room/** ni vendor/**.
- `bash clean-room/check.sh origin/main`: exit 1 únicamente por cuatro
  apariciones de `algorithm` en vendor/mamut-workflows/mamut-canon/src/lib.rs
  (bloqueo previo de #100). Sin otros hallazgos; no se modifica ninguna regla.
- Revisión humana clean-room: PENDIENTE (J4).

## Integración semántica de #98/#99 tras 3d10b33

Reparación local el 2026-10-14 por Codex (GPT-6), rol implementador, sin red ni
consulta de repositorios hermanos. Base de esta reparación: merge 3d10b33 de
j2/vnd-mamut-workflows; origin/main local = abe7453, SDK fijado a 585f32d.
Fuentes adicionales: tests, ejemplos, script de cruce y documentación propios de
#98; API pública `mamut_canon::sha256_hex` del SDK local. Se mantiene la
declaración clean-room inicial. No se modifican vendor/**, clean-room/**,
.github/**, docs/CLEAN_ROOM.md ni el harness de Winkler.

Los helpers y el test/ejemplo wasm de #98 aún importaban sha2 directamente.
Se sustituyen por `mamut_canon::sha256_hex(bytes)`, que calcula SHA-256 sobre
los bytes recibidos sin pasar por JCS. Para el ABI wasm se decodifican los 64
caracteres hexadecimales a los mismos 32 bytes. Los casos escalares concatenan
`to_bits().to_be_bytes()` en idéntico orden antes de hashear. No se reintroduce
sha2, no cambia Cargo.lock ni se agregan dependencias. El script Node conserva
su verificación independiente de los hashes de nombres con node:crypto.

`target` ahora identifica el target real de Cargo y necesariamente difiere
entre x86_64 y wasm32. El helper de cruce comprueba su valor y retira solo esa
clave antes de serializar otra vez con `engine_api::canonical_json` y hashear.
La salida pública de run no se modifica; el test de diez corridas sobre el
documento completo sigue comprobando también target. Una prueba adicional
comprueba que la proyección conserva todos los demás campos y rechaza target
incorrecto, nulo o ausente. `git_commit` sigue incluido en el hash de cruce:
la regeneración usa la variable opcional ausente y, por tanto, null.

Los once hashes de análisis cambian: input_sha256 adopta JCS y se incorpora
`git_commit: null` a los bytes comparados. La tabla anterior correspondía al
documento completo de #98, aún sin git_commit/target; la nueva corresponde al
documento actual tras validar/retirar target. Los cinco hashes escalares no
cambian, confirmando la equivalencia del digest y la conservación de bits.
Los valores nuevos están también en docs/DETERMINISMO.md; esa tabla es evidencia
reproducible, no una lista de goldens codificada en el test/script.

| Caso | SHA-256 anterior (#98) | SHA-256 nuevo (cruce ENG-CORE-005) |
| --- | --- | --- |
| cantilever | `d993e74910a942cdd27f727e635e3f109d7eac65ad8c0da69cec63a37afd1861` | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` |
| inclined-cantilever | `3378d324bd96923bab07640a17c9afd1fff18f69842a1f21bbc06152c8d76eed` | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` |
| modal-1dof | `2ba08c590e219f982b9b33f0955d9c790b2183b5d1af6309f09b579d43c92a71` | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` |
| pdelta | `5bcea296b5fd2652f75aac339b80bffa82e2076b06279f4020db817644a31101` | `426db50197873a7a8cafb586233ac7af70d8953c22d3a718fe6dbbb51b4637f0` |
| modal-coupled | `59499893099017bbb13ffcea8bed513537a059d929cddebc5511bee9a2de2f4e` | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` |
| spectrum-log-cqc | `a403028d3dc23b3d1e3312492866c7eacd9dd0431c2b50f5563bdefd83c25eb1` | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` |
| pdelta-inclined | `efd49f53009742c13ff5dbc66fcbf43e063cbfb707cc61dbe23ee043aaf4614f` | `f08ff9c7264cb64c8209956d97da0b2e99343479854f11d39d75773b6ec7c71c` |
| modal-inclined | `b3c4b92d66bc3894b2400354bb3518d78a8c224a6870cfc7aadd2861e2c0e480` | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` |
| spectrum-inclined | `a03dea7b4d9dd115cbef6b74cdf68e739e0d4655197957064d03a58a11bd0ac8` | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` |
| frame3d-inclined-static | `152dc9c3a40210634053378061c25668b93bf2eb018223e90b8b18712d9e754f` | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` |
| frame3d-inclined-modal | `58e6c5f1733a576a969d323164bf7dbee19155974688a33c73036b2462d4f060` | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` |

Comprobación adicional de la causa: en un ejemplo temporal propio se ejecutaron
los once fixtures actuales, se retiraron git_commit/target y se sustituyó
input_sha256 por SHA-256 de `canonical_json(input)` (la huella previa). Tras
serializar con canonical_json se recuperaron **exactamente los once hashes
anteriores** de la tabla de #98. El ejemplo se eliminó después de la auditoría.
Esto verifica que el resto de los resultados, incluidos los números del solver,
conserva sus bytes; no se ajustaron mecánica, tolerancias ni fixtures.

Reproducción del cruce: comandos de docs/DETERMINISMO.md con Rust 1.99.0
(LLVM 23.1.1), Node v22.19.0 y CARGO_TARGET_DIR exportado en
/workspace/.cargo-target/mamut-engine, sin RUSTFLAGS especiales. Debug y release:
16 casos por perfil, diez corridas por caso/target, hashes nativo/wasm iguales.
`cmp /tmp/eng-core-005-native-debug.txt /tmp/eng-core-005-native-release.txt`:
OK. Los cinco hashes escalares son idénticos a #98.

Verificación tras esta reparación (todos los comandos Cargo sin red y con el
CARGO_TARGET_DIR exportado indicado):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace --no-fail-fast`: **540 aprobadas, 1 fallo,
  0 ignoradas**, incluidas pruebas de documentación. Único fallo:
  `engine_crates_use_only_the_central_scalar_module`, por llamadas escalares
  directas en `crates/engine-validation/tests/support/winkler_harness.rs`.
  Se confirmó que también están en origin/main. Según la instrucción del
  usuario, corresponde a PR #106 y no se corrige aquí. engine-api/tests/determinism:
  cuatro pruebas aprobadas, incluida la comprobación de proyección.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --offline -p engine-api --tests --target wasm32-unknown-unknown`:
  OK.
- `bash clean-room/check.sh origin/main`: OK; el bloqueo anterior del vendor ya
  no se reproduce con la integración de #100.
- `bash scripts/check-vendor.sh`: OK, inventario y manifiesto SHA intactos.
- `git diff --check`: OK. Rutas protegidas y Cargo.lock sin cambios en esta
  reparación. Commit local, sin push. Revisión humana clean-room: PENDIENTE (J4).
