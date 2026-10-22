# Publicación de esquemas versionados · MAM-869 / ENG-REL-002

Fecha: 2026-10-21. Este ticket prepara archivos locales y propuestas de release.
La base por defecto es **https://schemas.mamut.systems/**, **PENDIENTE de
confirmar por Jonathan**. Ningún comando de generación o empaquetado publica.

## Contrato del árbol

`schemas/` se versiona en Git. Cada kind `id/major` produce `id/major.json`.
Los documentos API se transportan desde `engine_api::schema(kind)` mediante el
ejemplo `export_schemas`: no se parsean ni reformatean al copiarlos. Se conservan
espacios, finales de línea y salto final. `engine_api::schema_kinds()` enumera el
registro único usado por `schema()`. `capabilities()` conserva sus bytes y sus
contratos actuales; sus claves de `schemas` deben estar incluidas en el registro.

El índice `schemas/index.json` contiene `base_url` y `schemas`, ordenados por
kind. Cada entrada contiene:

| Campo | Significado |
| --- | --- |
| `id` | Nombre sin la versión, por ejemplo `engine-input` |
| `version` | Versión mayor como entero positivo |
| `$id` | Identidad literal del documento original |
| `path` | Ruta relativa a `schemas/`, por ejemplo `engine-input/1.json` |
| `url` | `base_url` concatenada con `path` |
| `sha256` | SHA-256 hexadecimal de los bytes del documento, sin canonizarlos |
| `bytes` | Número exacto de bytes UTF-8 |

El índice es JSON compacto ASCII, con claves ordenadas recursivamente y un LF
final; solo usa strings, enteros y colecciones. No incluye fechas, rutas de la
máquina ni versión del ejecutable. Este formato canónico es determinista.

## Decisión sobre `$id`

Los `$id` actuales se conservan. La correspondencia URN → URL se publica en cada
entrada del índice; por ejemplo:

| URN conservado | URL de distribución propuesta |
| --- | --- |
| `urn:mamut:engine-input:1` | `https://schemas.mamut.systems/engine-input/1.json` |
| `urn:mamut:results:1` | `https://schemas.mamut.systems/results/1.json` |
| `urn:mamut:validate-report:1` | `https://schemas.mamut.systems/validate-report/1.json` |
| `urn:mamut:cli-error:1` | `https://schemas.mamut.systems/cli-error/1.json` |
| `urn:mamut:capabilities:1` | `https://schemas.mamut.systems/capabilities/1.json` |
| `urn:mamut:method-descriptor:1` | `https://schemas.mamut.systems/method-descriptor/1.json` |
| `urn:mamut:progress:1` | `https://schemas.mamut.systems/progress/1.json` |

La URL sirve los bytes cuya identidad sigue siendo el URN. Un consumidor puede
usar el índice como catálogo para resolver esa identidad. Cambiar la base de
distribución solo modifica el índice. La eventual migración de `$id` a URL es
decisión de Jonathan: cambiaría los documentos, sus huellas y la evidencia de
`docs/DETERMINISMO.md`, y debe planificarse junto con la compatibilidad del
contrato. Este ticket no modifica esos datos ni capabilities.

## Generación y extensión

Requisitos locales: Python 3 con biblioteca estándar y el toolchain Cargo del
repositorio con sus dependencias disponibles offline. Desde la raíz:

```sh
export PATH=$HOME/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/workspace/.cargo-target/rel002
python3 scripts/generate-schemas.py --write
python3 scripts/generate-schemas.py --check
cargo test -p engine-api --test schema_publication --offline
```

`--write` está bloqueado si existe `CI` (incluso vacía) o si `GITHUB_ACTIONS`
tiene un valor no vacío.
`--check` nunca escribe y, como `--write`, invoca Cargo con `--offline`.
El test normal de Cargo suministra directamente los bytes desde `schema()` por
stdin; no ejecuta un Cargo anidado. Comprueba todos los kinds, los contratos de
capabilities, el inventario de fuentes API, el árbol completo, su índice y hashes.
Python 3 debe estar instalado también en el runner de CI existente; no hace
falta modificar sus workflows para ejecutar estas pruebas.

Para configurar otra base HTTPS terminada en `/`:

```sh
python3 scripts/generate-schemas.py --write --base-url https://example.test/contracts/
python3 scripts/generate-schemas.py --check --base-url https://example.test/contracts/
```

La comprobación sin `--base-url`, incluida la de Cargo, usa la base registrada en
el índice versionado y verifica todas las URLs contra ella. La regeneración sin
esa opción vuelve al valor por defecto. Revisar cualquier cambio de base en el PR.

Para añadir un esquema API, agregar su archivo fuente y **una entrada** al
registro `SCHEMAS` de `crates/engine-api/src/lib.rs`; no hay otra lista de kinds
API en el generador. Así se publica también `validate-report/1`. Para añadir uno de
CLI, agregar el archivo y una entrada a `scripts/schema-sources.json`:

```json
{
  "additional_schemas": [
    {"kind": "cli-error/1", "source": "crates/engine-cli/schemas/cli-error-1.json"},
    {"kind": "cli-run/1", "source": "crates/engine-cli/schemas/cli-run-1.json"},
    {"kind": "progress/1", "source": "crates/engine-cli/schemas/progress-1.json"}
  ]
}
```

Actualmente se publica `progress/1` desde el CLI mediante `additional_schemas`.
Es un contrato de transporte del CLI: no se registra en `SCHEMAS` ni se expone
mediante `engine_api::schema()`. La API no depende del CLI; el generador reúne
ambos registros para la distribución pública.

Registrar cada entrada cuando su archivo exista. Después regenerar y revisar
`schemas/`. Los archivos adicionales se leen como bytes; las pruebas rechazan
fuentes CLI sin registrar, kinds duplicados, enlaces, archivos especiales,
archivos publicados huérfanos y directorios vacíos sobrantes. La regeneración
rechaza huérfanos existentes para que su eliminación sea explícita y revisable.

## Preparación local del release

Con el árbol revisado y versionado:

```sh
python3 scripts/generate-schemas.py --check
cargo fmt --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
bash scripts/package-schemas.sh /tmp/mamut-schemas-a
bash scripts/package-schemas.sh /tmp/mamut-schemas-b
cmp /tmp/mamut-schemas-a/mamut-schemas.tar /tmp/mamut-schemas-b/mamut-schemas.tar
sha256sum /tmp/mamut-schemas-a/mamut-schemas.tar /tmp/mamut-schemas-b/mamut-schemas.tar
```

El script acepta un directorio de salida, incluso fuera del repositorio, y
comprueba inventario, índice canónico, identidades, URLs, tamaños y hashes antes
de crear `mamut-schemas.tar`. La comprobación previa del generador enlaza ese
árbol con la API actual; el empaquetador por sí solo comprueba el árbol publicado.
Rechaza una salida dentro de `schemas/`. Puede sustituir un TAR local anterior.

Se usa USTAR de la biblioteca estándar Python, sin compresión: entradas de
archivos ordenadas por ruta, prefijo `schemas/`, mtime cero, modo 0644, uid/gid
cero y nombres de usuario/grupo vacíos. No hay entradas de directorios ni
metadatos PAX. `package-schemas.sh` ignora `SOURCE_DATE_EPOCH`: el mtime de
todas las entradas del TAR se fija siempre en 0. No depende de la disponibilidad
de tar/gzip deterministas del sistema. Las pruebas de Cargo ejecutan dos
empaquetados, comparan SHA-256, inspeccionan metadatos y contenido del TAR, y
comprueban que mtime y permisos de las fuentes no alteran el resultado.

## Pasos para Jonathan

Estos pasos son una propuesta para ejecutar posteriormente por Jonathan.

1. Confirmar la base pública y el alojamiento. Si cambia, regenerar localmente
   con `--base-url`, revisar el diff del índice y versionarlo junto con el árbol.
   Mantener los URN. Decidir por separado si habrá una migración futura de `$id`.
2. Integrar el ticket y usar una versión de release de MAM-95 que contenga los
   archivos versionados y pase las verificaciones anteriores. Conservar rutas
   ya distribuidas y revisar cualquier cambio de bytes bajo la misma versión
   mayor antes de su publicación.
3. Para la opción GitHub Pages, en Settings → Pages elegir **GitHub Actions**,
   configurar `schemas.mamut.systems` como dominio personalizado, apuntar su
   registro DNS CNAME a `<propietario>.github.io` y habilitar HTTPS cuando el
   certificado esté disponible. Confirmar la disponibilidad de Pages para la
   visibilidad y plan de este repositorio. Crear el entorno `github-pages` y
   permitir los tags del release. Si se elige otro alojamiento, sustituir solo
   los pasos de Pages por la carga del contenido de `schemas/` a la raíz pública.
4. Añadir posteriormente `.github/workflows/schema-release.yml` con el YAML
   sugerido abajo, o trasladar sus jobs al workflow de MAM-95. El evento
   `release.published` enlaza esta propuesta con el release creado por MAM-95;
   el checkout usa exactamente su tag. También se incluye ejecución manual por
   tag para releases cuyo evento no inicia workflows. Preparar el runner de
   `check` con Python 3, Rust 1.99.0 en `/opt/rust/bin` y una caché Cargo completa para trabajar con
   `--offline`. Crear la variable de repositorio `SCHEMA_CARGO_CACHE` con la ruta
   de esa caché local del runner (solo dependencias y configuración pública, sin
   credenciales); el job copia su contenido a un CARGO_HOME aislado.
   Antes de aplicar el YAML, sustituir cada placeholder `<SHA-40-de-...>` por
   el SHA completo de 40 caracteres obtenido del release oficial de la acción
   y versión indicadas en el comentario `# vX`. Los placeholders están pendientes
   de verificación online; el ejemplo no es ejecutable hasta completarlos.
5. Mantener `permissions: {}` en el nivel superior y conceder permisos por job:
   `check` solo `contents: read`, `release` solo `contents: write`, y `deploy`
   solo `pages: write`/`id-token: write`. `check` puede usar un runner propio;
   `release` y `deploy` deben usar `ubuntu-latest` alojado por GitHub, nunca
   self-hosted. `deploy` debe usar el entorno `github-pages` revisado.
   El comando `gh release upload` siguiente falla si el asset ya existe;
   comparar su SHA-256 y decidir
   explícitamente una sustitución si se vuelve a ejecutar.
6. Tras el despliegue, comprobar `/index.json` y cada `url` del índice contra
   sus `bytes` y `sha256`, y comparar el TAR adjunto con el preparado. Publicar
   el contenido de `schemas/` en la raíz del dominio: `engine-input/1.json`,
   sin añadir un segmento `/schemas/` a las URLs. Si se sirven versiones
   históricas, mantenerlas también en el árbol y en el índice de su release.
7. Cargo: este ticket no requiere cambios de manifiesto, lockfile ni un paso de
   `cargo publish`. El ejemplo se descubre automáticamente por Cargo, usa
   dependencias existentes y los JSON fuente siguen incluidos en engine-api.
   El árbol público se distribuye como TAR y sitio estático. Cualquier futura
   publicación de crates o Python corresponde a la decisión de release MAM-95.

YAML sugerido, **no aplicado** (las operaciones remotas aquí descritas solo
deben ejecutarse posteriormente como parte del release autorizado):

```yaml
name: Schemas del release
on:
  release:
    types: [published]
  workflow_dispatch:
    inputs:
      release_tag:
        description: Tag del release existente de MAM-95
        required: true
        type: string
permissions: {}
env:
  RELEASE_TAG: ${{ inputs.release_tag || github.event.release.tag_name }}
concurrency:
  group: schema-release
  cancel-in-progress: false
jobs:
  check:
    if: ${{ github.event_name == 'workflow_dispatch' || !github.event.release.prerelease }}
    runs-on: [self-hosted, contabo, engine]
    timeout-minutes: 20
    permissions:
      contents: read
    env:
      CARGO_NET_OFFLINE: "true"
      SCHEMA_CARGO_CACHE: ${{ vars.SCHEMA_CARGO_CACHE }}
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
        with:
          ref: ${{ env.RELEASE_TAG }}
          persist-credentials: false
      - name: Preparar entorno aislado y caché offline
        run: |
          test -d "$SCHEMA_CARGO_CACHE/registry"
          mkdir -p "$RUNNER_TEMP/schema-home" "$RUNNER_TEMP/schema-cargo-home"
          cp -R "$SCHEMA_CARGO_CACHE/." "$RUNNER_TEMP/schema-cargo-home/"
          {
            echo "HOME=$RUNNER_TEMP/schema-home"
            echo "CARGO_HOME=$RUNNER_TEMP/schema-cargo-home"
            echo "CARGO_TARGET_DIR=$RUNNER_TEMP/schema-target"
            echo "PATH=/opt/rust/bin:/usr/local/bin:/usr/bin:/bin"
            echo "RUSTUP_TOOLCHAIN="
          } >> "$GITHUB_ENV"
          /opt/rust/bin/rustc -V
      - name: Comprobar el árbol del tag
        run: |
          cargo build --workspace --offline
          python3 scripts/generate-schemas.py --check
          cargo test -p engine-api --test schema_publication --offline
      - name: Preparar el asset dos veces
        run: |
          bash scripts/package-schemas.sh "$RUNNER_TEMP/schemas-a"
          bash scripts/package-schemas.sh "$RUNNER_TEMP/schemas-b"
          cmp "$RUNNER_TEMP/schemas-a/mamut-schemas.tar" "$RUNNER_TEMP/schemas-b/mamut-schemas.tar"
          sha256sum "$RUNNER_TEMP/schemas-a/mamut-schemas.tar"
      - uses: actions/upload-artifact@<SHA-40-de-upload-artifact-v4> # v4
        with:
          name: schema-archive
          path: ${{ runner.temp }}/schemas-a/mamut-schemas.tar
          if-no-files-found: error
      - name: Preparar la raíz pública
        run: |
          mkdir -p "$RUNNER_TEMP/schema-site"
          cp -R schemas/. "$RUNNER_TEMP/schema-site/"
          touch "$RUNNER_TEMP/schema-site/.nojekyll"
      - uses: actions/upload-pages-artifact@<SHA-40-de-upload-pages-artifact-v3> # v3
        with:
          path: ${{ runner.temp }}/schema-site
  release:
    needs: check
    runs-on: ubuntu-latest
    permissions:
      contents: write
    env:
      GH_TOKEN: ${{ github.token }}
      GH_REPO: ${{ github.repository }}
    steps:
      - uses: actions/download-artifact@<SHA-40-de-download-artifact-v4> # v4
        with:
          name: schema-archive
          path: ${{ runner.temp }}/schema-archive
      - name: Adjuntar al release de MAM-95
        run: gh release upload "$RELEASE_TAG" "$RUNNER_TEMP/schema-archive/mamut-schemas.tar" --repo "$GH_REPO"
  deploy:
    needs: check
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/configure-pages@<SHA-40-de-configure-pages-v5> # v5
      - uses: actions/deploy-pages@<SHA-40-de-deploy-pages-v4> # v4
        id: deployment
```

Si MAM-95 crea el release con su `GITHUB_TOKEN`, GitHub no dispara otro workflow
por ese evento. En ese caso ir a Actions → Schemas del release → Run workflow,
e introducir en `release_tag` el tag del release existente de MAM-95. El workflow
debe estar previamente integrado en la rama por defecto. La ejecución manual
debe apuntar al release estable que se quiere servir, pues también actualiza
Pages. Para automatizarlo, incorporar estos pasos directamente **después de
crear el release** en MAM-95, suministrando su tag mediante `RELEASE_TAG`, o
convertir los jobs en un workflow reutilizable con input de tag y llamarlo desde
MAM-95. No basar la integración en un evento que no llegará.
