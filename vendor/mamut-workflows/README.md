# Mamut Workflows vendorizado

Origen: <https://github.com/Mamut1195/mamut-workflows>, repositorio propio de
MAMPRO. Origen fijado al commit de `main`
`4cb9ecd0e1117d86444149b17a6b34d1283d9455`.
La versión declarada sigue siendo `0.1.0`, pero el tag `v0.1.0` **no corresponde**
a este commit: apunta a `5c4e75af21564edc1a3b900d565a2ab90447ac1c`.
Ese tag escribe `jcs-rfc8785` en vez del `rfc8785-jcs` exigido por el contrato
y su manifiesto no cumple `execution-manifest/1`; el commit fijado corrige ambos.
Fuente local de solo lectura usada: `/workspace/mamut-workflows`.

Historia: primero se vendorizó `5c4e75af21564edc1a3b900d565a2ab90447ac1c`;
por decisión de J0/Jonathan del 2026-10-14 se sustituyó por
`585f32dc3455208566a0ffb894ad03ce3816b888` por esas correcciones de contrato.
En los dos crates incluidos solo cambió `mamut-canon/src/lib.rs`, copiado con
`git show 585f32d:crates/mamut-canon/src/lib.rs`; LICENSE no cambió. Después se re-fijó a `4cb9ecd0e1117d86444149b17a6b34d1283d9455` (main, PR #5: CI en VPS y
corrección de MAM-490), con ficheros de librería y LICENSE idénticos byte a
byte a 585f32d (comparado con `cmp` contra `git show 4cb9ecd:<ruta>`).
Las correcciones del emisor de manifiestos están en upstream y fuera de estos
dos crates; este vendorizado no incluye el ejecutor.

Se copian íntegramente `crates/mamut-canon` (SDK-WF-002: RFC 8785 y SHA-256)
y `crates/mamut-workflow` (SDK-WF-003: registro y compilación de workflows)
a los directorios homónimos. Su código Rust y sus pruebas permanecen verbatim.
Ambos crates son miembros y miembros predeterminados del workspace del Engine.
No se conectan todavía a la fachada del Engine.

Licencia: Apache-2.0. `LICENSE` es copia exacta del upstream; `NOTICE` es nuevo
en esta distribución y detalla atribución y ajustes. Código propio de MAMPRO,
sin OpenSees/Stabileo.

## Ajustes permitidos

No editar a mano salvo los ajustes listados en los dos `Cargo.toml`:

- Metadatos literales del commit fijado: `version = "0.1.0"`, `edition = "2021"`,
  `license = "Apache-2.0"`, `rust-version = "1.83"`,
  `repository = "https://github.com/Mamut1195/mamut-workflows"`.
- `readme = "../README.md"`, `publish = false` y comentario de modificación.
- Dependencias entre estos crates por `path` si upstream las introduce;
  en el commit fijado no existen dependencias entre ellos.

No se cambia la lógica, no se añaden `panic`/`unwrap`/`expect` en biblioteca.
No se necesitan excepciones de Clippy ni exclusiones de formato para esta copia.

## Integridad y actualización

Ejecutar desde cualquier directorio:

```sh
bash /ruta/a/mamut-engine/scripts/check-vendor.sh
```

`MANIFEST.sha256` fija el SHA-256 de **todos** los archivos de este directorio,
incluidos los manifiestos adaptados, LICENSE, NOTICE y este README; excluye
únicamente el propio `MANIFEST.sha256` para evitar autorreferencia. El script
verifica tanto los hashes como el inventario (archivos añadidos o eliminados).
El manifiesto detecta cambios accidentales; los cambios deliberados del propio
manifiesto deben revisarse en Git junto con la procedencia.

Decisión de Jonathan del 2026-10-14, MAM-481 · ENG-VND-001, opción (a):
`clean-room/check.sh` excluye únicamente `vendor/mamut-workflows/` del escaneo
de identificadores; las rutas prohibidas siguen comprobándose. Si el diff toca
esta distribución, el validador confiable verifica inline el inventario completo,
los hashes, la declaración del commit exacto y la presencia de LICENSE, sin
ejecutar scripts del PR. Rechaza symlinks y rutas no canónicas del manifiesto
para evitar escapes fuera del directorio. Se conserva `scripts/check-vendor.sh`.
`CODEOWNERS` asigna esta distribución al mismo dueño que `clean-room/**`
(@Mamut1195, Jonathan), para revisar también cambios de manifiesto y contenido.
Jonathan fusiona el cambio autorizado de `clean-room/**`.

Se regenera `MANIFEST.sha256` por el cambio de origen: cambian los hashes de
este README, NOTICE y `mamut-canon/src/lib.rs`.

MAM-490: si un test upstream presupone `../../target` y falla con
`CARGO_TARGET_DIR` exportado, conservarlo verbatim y registrar el fallo en
`provenance/ENG-VND-001.md`; no parchear el test vendorizado.
En la verificación de este commit del 2026-10-14 no se reprodujo ese problema:
`cargo test --offline --workspace` pasó con
`CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine` exportado;
los dos crates incluidos no presuponen `../../target`.

Si upstream cambia, re-vendorizar; no aplicar parches manuales de lógica:

1. Verificar el nuevo tag/commit y la licencia en un checkout upstream limpio.
   No modificar `/workspace/mamut-workflows` durante esta tarea.
2. Sustituir ambos directorios completos por copias de `crates/` de ese commit
   y copiar exactamente su LICENSE. Reaplicar solo los ajustes permitidos.
3. Actualizar este README, NOTICE y `provenance/ENG-VND-001.md` con el commit,
   el tag, los archivos y todos los ajustes realmente aplicados.
4. Actualizar Cargo.lock con Cargo offline, revisar su diff y ejecutar las
   verificaciones del workspace, incluido wasm32. Si formato o Clippy requieren
   excepciones, mantener la lógica intacta y documentar la configuración mínima.
   Ejecutar también `bash clean-room/check.sh origin/main`; cualquier cambio
   del commit permitido en el validador requiere revisión de Jonathan.
5. Regenerar el manifiesto desde la raíz del repositorio y revisar su diff:

   ```sh
   (cd vendor/mamut-workflows &&
     find . -type f ! -path './MANIFEST.sha256' -print0 |
     LC_ALL=C sort -z | xargs -0 sha256sum > MANIFEST.sha256)
   bash scripts/check-vendor.sh
   ```

6. Registrar las verificaciones y solicitar revisión J4 del cambio completo.
