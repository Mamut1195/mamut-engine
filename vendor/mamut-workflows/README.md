# Mamut Workflows vendorizado

Origen: <https://github.com/Mamut1195/mamut-workflows>, repositorio propio de
MAMPRO. Commit fijado de `main`:
`1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d` (PR #6 y #7, SDK-WF-012 /
MAM-493: extracción de las librerías y puntos de extensión de la CLI).
Re-vendorizado el 2026-10-15 para MAM-481 / ENG-VND-001.
Fuente local de solo lectura: `/workspace/mamut-workflows`; cada archivo se
obtiene con `git -C /workspace/mamut-workflows show 1eeb91a:<ruta>`, nunca
del árbol de trabajo.

La versión declarada continúa siendo `0.1.0`. El tag `v0.1.0` apunta a
`5c4e75af21564edc1a3b900d565a2ab90447ac1c`, no al commit fijado.
Historia de la distribución:

- `5c4e75af21564edc1a3b900d565a2ab90447ac1c`: copia inicial.
- `585f32dc3455208566a0ffb894ad03ce3816b888`: decisión J0/Jonathan del
  2026-10-14; correcciones `rfc8785-jcs` y `execution-manifest/1`.
- `4cb9ecd0e1117d86444149b17a6b34d1283d9455`: PR #5, CI y MAM-490;
  los dos crates entonces incluidos y LICENSE eran idénticos a 585f32d.
- `1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`: incorpora las librerías
  de ejecución y CLI reutilizable que necesita SFC-002.

## Contenido y ajustes

Se copian íntegramente los cuatro directorios upstream `crates/<nombre>` a
los directorios homónimos de esta distribución, incluidos tests y fixtures:

- `mamut-canon`: canonicalización RFC 8785 y SHA-256.
- `mamut-workflow`: registro y compilación de workflows.
- `mamut-workflow-execution`: ejecución compartida, manifiestos y observadores.
- `mamut-workflow-cli`: librería CLI con proveedor, identidad, progreso,
  cancelación y códigos de salida configurables.

Los cuatro son `members` y `default-members` del workspace del Engine.
No se modifica `crates/` del Engine ni se conecta todavía su fachada.
No se incluyen `mamut-workflow-run` (binario/fachada de compatibilidad) ni
`mamut-tool-conformance` (herramienta de conformidad): quedan fuera del alcance
de las librerías requeridas por SFC-002 y no son dependencias de estos crates.

Únicos archivos modificados respecto a upstream: los cuatro `Cargo.toml`.
Se sustituyen los metadatos heredados por literales del commit fijado:
version `0.1.0`, edition `2021`, license `Apache-2.0`, rust-version `1.83`,
repository `https://github.com/Mamut1195/mamut-workflows`. Se fija
`readme = "../README.md"`, `publish = false` y un comentario de modificación.
Las dependencias internas ya usan rutas relativas entre hermanos y se conservan.
No hay cambios de lógica ni exclusiones de tests, formato o Clippy.
Todo el código Rust, las pruebas, los snapshots y el README propio de la CLI
son verbatim; `cmp` contra cada `git show` consta en la bitácora de procedencia.

Los tests `mamut-workflow-cli/tests/extensions.rs` requieren los dos ejemplos
upstream `examples/open-local/{catalog,workflow}.json`. Se copian verbatim a
`examples/open-local/` de esta distribución, cubiertos por su manifiesto.
Sus rutas literales también exigen copias en `vendor/examples/open-local/`:
se conservan ambas para ejecutar todos los tests sin parchear Rust. Ambos
verificadores exigen igualdad byte a byte con los originales del manifiesto
y rechazan enlaces en las rutas de esas copias. No requieren el binario
`mamut-workflow` ni otros archivos externos. Los archivos temporales de tests
usan `CARGO_TARGET_TMPDIR`; MAM-490 no requiere parche local.

Licencia Apache-2.0: LICENSE es una copia exacta del upstream. NOTICE es un
aviso añadido a esta distribución. Código propio de MAMPRO, sin OpenSees/Stabileo.

## Integridad y actualización

```sh
bash scripts/check-vendor.sh
bash clean-room/tests/run.sh
bash clean-room/check.sh origin/main
```

`MANIFEST.sha256` enumera **todos** los archivos de esta distribución salvo él
mismo, con formato `<sha>  ./ruta` y orden de rutas `LC_ALL=C`. El script
comprueba inventario, hashes y el commit completo en README y NOTICE.
`clean-room/check.sh` conserva las comprobaciones inline de inventario físico,
rutas canónicas, ausencia de enlaces, SHA-256 estricto, commit y LICENSE.
Además fija el hash del propio manifiesto en la constante
`EXPECTED_WORKFLOWS_MANIFEST_SHA256`: regenerar hashes coherentes dentro de un
PR ya no permite cambiar el contenido aprobado por la base (J4 r31).
Los cambios en `vendor/examples/` también activan esta verificación.

Se mantiene la excepción autorizada por Jonathan el 2026-10-14 únicamente para
el escaneo de identificadores bajo rutas canónicas `vendor/mamut-workflows/`;
las rutas prohibidas se comprueban siempre. En CI el verificador y sus anclas
proceden de la base confiable; nunca se ejecutan scripts del PR. Actualizar
el ancla junto con el vendor requiere revisión de Jonathan, dueño de
`clean-room/**` y de esta distribución según CODEOWNERS. No se modifica `.github/`.

Para actualizar: elegir un commit y copiar íntegramente sus cuatro crates,
LICENSE y los dos ejemplos con `git show`, reaplicar únicamente los ajustes
anteriores, actualizar README/NOTICE/procedencia y Cargo.lock offline.
Verificar con `cmp` cada archivo verbatim, regenerar el manifiesto:

```sh
(cd vendor/mamut-workflows &&
  find . -type f ! -path './MANIFEST.sha256' -print0 |
  LC_ALL=C sort -z | xargs -0 sha256sum > MANIFEST.sha256)
sha256sum vendor/mamut-workflows/MANIFEST.sha256
```

Actualizar la constante del validador confiable al resultado y ejecutar formato,
Clippy, tests, rustdoc, controles vendor/clean-room, determinismo y wasm32 offline.
Si upstream tiene problemas, conservar Rust verbatim y documentarlos;
no aplicar parches locales de lógica. Registrar resultados y revisión J4 en
`provenance/ENG-VND-001.md`.
