# ENG-CONT-002 · MAM-868 · Contrato de éxito/aviso cli-run/1

Fecha: **2026-10-21**. Trabajo local sin red en
el worktree local, rama `j2/MAM-868-cli-run-contrato`.
Base observada: HEAD y `origin/main` en
`7375fa25a34a3cb004bcaec84f645743b864ff89`. No se consulta el estado remoto.

## Contrato y decisiones

Se publica [cli-run-1.json](../crates/engine-cli/schemas/cli-run-1.json),
versión contractual **1.0.0**, en `crates/engine-cli/schemas/`, siguiendo la
convención indicada para MAM-867 / PR #205. La base local no contiene ese esquema
de error; no se modifica su implementación ni ningún sobre de fallo.
No existía `docs/contracts/`; la política queda en el
[README del CLI](../crates/engine-cli/README.md#contrato-publicado-cli-run1--100).

El sobre de stdout para `run` conserva exactamente los bytes que producía el
adaptador. No se añade un campo `schema` ni se modifica `src/`. `type:result`,
identidad, operación `run`, outcome/código (`ok`/0, `warning`/1), `data` y
`engineResults` son obligatorios. El sobre y el informe validate son estrictos.
`engineResults` publica un resultado completo por paso, en orden: results/1 o
`{operation:"validate",status,diagnostics}`. El esquema relaciona outcome y
código; `ok` exige resultados ok y `warning` al menos un resultado warning.

Se cierra la decisión de salida: **engineResults es contrato del adaptador**.
Un canal estándar de resultados completos queda como trabajo del SDK. No se
modifica el SDK vendorizado, el transporte de entrada ni la decisión MCP.
Durante V1 no se admiten cambios mayores. Las adiciones compatibles requieren
versión menor; las aclaraciones sin cambios observables, parche. `engineVersion`
sigue identificando al motor, sin duplicar la versión contractual en stdout.

`data` y sus estructuras anidadas pertenecen a `mamut-workflow-execution`.
Se leyó su productor vendorizado únicamente para comprobar campos, estados y
preimágenes: el adaptador preserva el manifiesto, que admite extensiones SDK.
`recordFingerprint` siempre está presente en este productor, cubre data sin
ese campo y no cubre el sobre. Las huellas de cada paso cubren su resultado.
No se crea un segundo contrato de manifiesto.
El esquema solo exige tipo objeto para `recordFingerprint` y las huellas
anidadas: sus claves pertenecen al SDK y no se enumeran. Las partes del
adaptador conservan sus restricciones estrictas.

Los `$ref` usan `urn:mamut:results:1` y su definición `diagnostic` existentes.
Se extrae el validador de pruebas de engine-api a
[tests/support/schema.rs](../crates/engine-api/tests/support/schema.rs), compartido
con el CLI. Conserva sus reglas y añade únicamente resolución local explícita
de results/1 y `contains` para la agregación de avisos. Falla ante keywords,
patrones o referencias desconocidos, sin red ni paquetes nuevos. Es un
validador del subconjunto empleado, no un motor JSON Schema general.

## Cobertura y bytes

[tests/contract.rs](../crates/engine-cli/tests/contract.rs) descubre los once
fixtures de determinismo y los dieciséis JSON válidos de engine-api (incluidos
los once SFC-010). Extrae únicamente `runs[*].engine_input` de E1 versionado;
no usa valores esperados ni referencias numéricas externas. Añade el aviso de
combinaciones ya presente en los tests CLI. Son **29 entradas × 2 workflows**
(engine.run y engine.validate): **58 golden**, **55 códigos 0 y 3 códigos 1**.
Cada caso ejecuta dos veces el binario real, valida stdout contra el esquema,
comprueba stderr vacío, código/outcome y cantidad de pasos/resultados, y compara
longitud y SHA-256 de todos los bytes. Los golden congelan también el orden y
la estructura actuales.

[tests/golden/stdout-sha256.json](../crates/engine-cli/tests/golden/stdout-sha256.json)
es el único golden versionado: manifiesto de versión 1 con nombre, longitud
en bytes y SHA-256 por caso, más la procedencia de compilación base. No contiene
stdout ni se conservan archivos `.stdout.json`. Los hashes cubren UTF-8 compacto
con exactamente un LF final, sin CR. El binario ya usa `writeln!` y emite LF
también en Windows; no se cambia ese comportamiento. El test rechaza CRLF y
ausencia de LF; no normaliza stdout ni depende del locale o de rutas de ejecución.
La base se generó en Linux x86_64 con `git_commit:null`. El hash de bytes crudos
usa `hashlib` de Python 3, disponible en el entorno de paridad, sin dependencias
nuevas de Cargo. Ante discrepancias se imprimen hashes y longitudes esperados y
obtenidos y se guardan stdout real y comparable bajo
`env!("CARGO_TARGET_TMPDIR")/cli-contract-failure-<pid>/`.

La política [DETERMINISMO.md](../docs/DETERMINISMO.md) incluye `target` y
`git_commit` en los resultados: no pueden compararse literalmente entre builds
distintos. Para otra compilación, se lleva stdout a la procedencia del manifiesto sustituyendo
literalmente esos dos metadatos y las huellas dependientes (resultado de paso y
manifiesto). Se verifican primero las preimágenes del stdout real y se recalculan
por JCS/SHA-256 mediante la API pública existente. Los metadatos se obtienen
de engine-api compilado, independientemente del stdout bajo prueba. No se
proyectan campos ni se reserializa el stdout real; números, claves, arrays,
espacios y diagnósticos permanecen congelados. Un test de ida/vuelta comprueba
la sustitución para triples macOS x86_64/arm64 y Windows MSVC, incluyendo una
revisión de compilación no nula. Es una prueba del materializador, no una
ejecución del binario en esos sistemas.

Los tests negativos cubren campos obligatorios, identidad, tipo de engineResults,
versión/estado results, vector numérico, tipos de huellas, diagnósticos, correspondencia
de avisos y rechazo de extensiones del adaptador; las extensiones SDK se aceptan.

Regeneración explícita local:

```sh
export PATH=$HOME/.cargo/bin:$PATH
export CARGO_TARGET_DIR=<directorio-target-del-worktree>
# Linux; MAMUT_ENGINE_GIT_COMMIT ausente al compilar para conservar la base.
MAMUT_UPDATE_CLI_GOLDENS=1 cargo test -p engine-cli --test contract --offline \
  all_accepted_fixtures_match_schema_and_stdout_golden
cargo test -p engine-cli --test contract --offline
```

Sin la variable no se escribe ningún golden. Solo admite `1` o ausencia;
regenerar con `CI` o `GITHUB_ACTIONS` definidos falla antes de escribir.
Los golden ausentes u obsoletos se rechazan. Siempre se revisa el diff.

Se buscó ENG-PY-006, MAM-86, paridad y conformance en crates y bindings/python.
La prueba existente de paridad de bytes Rust/Python está en
[test_run.py](../bindings/python/tests/test_run.py),
`RunTest.test_cantilever_matches_rust_bytes_and_sha256`. Se enlazan allí los
golden CLI y desde el [README Python](../bindings/python/README.md).
Se conserva la referencia directa Rust del binding. No se ejecuta la suite de
wheel Python por un cambio de comentarios y enlaces; sí el workspace Rust.

## Verificación local y límites

Entorno: Rust 1.99.0, Cargo offline,
`CARGO_TARGET_DIR=<directorio-target-del-worktree>`, Linux x86_64.

- `cargo fmt --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test -p engine-cli --test contract --offline`: **3 tests OK**.
- `cargo test --workspace --offline`: **OK, 752 tests aprobados en 116 suites,
  sin fallos ni ignorados**, incluidas las tres pruebas nuevas del contrato.
- Guardia de regeneración con `CI=1`: rechazo esperado y manifiesto con sus
  **58 casos sin cambios**, comprobado byte a byte antes/después.
- Discrepancia de hash inyectada temporalmente: se verifican el hash esperado y
  obtenido en el fallo y los bytes de diagnóstico bajo `CARGO_TARGET_TMPDIR`.
  El manifiesto se restaura byte a byte y el contrato vuelve a pasar.
- `git diff --check`: OK.
- `bash clean-room/check.sh origin/main`: **OK**, con los archivos nuevos
  incluidos mediante `git add --intent-to-add` en un índice temporal. El
  manifiesto de hashes y los objetos SDK opacos permiten conservar la guardia
  sin cambios ni excepciones nuevas. Se comprueban además los archivos nuevos
  y modificados completos contra la lista de identificadores prohibidos.

El índice real y la base de objetos Git son de solo lectura en este entorno.
La ejecución indicada arriba usa también un directorio de objetos temporal en
un directorio temporal local, con los objetos originales como alternativos de solo lectura.
No se escribe en el índice ni en los objetos originales.

La ejecución real en **macOS y Windows necesita una matriz en `rust.yml`**,
**decisión de Jonathan**. No se edita `.github/workflows`; esta evidencia solo
acredita Linux. No se declara CI multiplataforma ejecutada.

No se modifican vendor, manifiestos/lock de Cargo, workflows, clean-room,
CLEAN_ROOM.md, CODEOWNERS, deny.toml, barra axial ni truss. No se abre material
restringido ni se incorporan resultados de otros motores. El script clean-room
se ejecuta sin modificarlo; la revisión humana J4 continúa pendiente.
No se edita CHANGELOG.md: se añade únicamente
[changelog.d/MAM-868.added.md](../changelog.d/MAM-868.added.md). No se hace commit.


## Merge de origin/main · 2026-10-22

Autor: **Codex gpt-6.1-sol high**. Se conserva lo incorporado por main,
incluidos #205 y #183, y se integra #208. Los tres archivos en conflicto quedan
sin marcadores. El índice permanece intacto: no se hace commit, staging, rebase
ni abort del merge.

La única implementación del validador es la de main en
`crates/engine-api/tests/support/schema.rs`; los helpers del CLI delegan en ella.
Se amplía con `contains`, las referencias locales conocidas de results/1 y los
patrones de cli-error/1, conservando el patrón más estricto de diagnósticos API.
`cli-run/1` se registra junto a `cli-error/1` en `additional_schemas`; el generador
publica `schemas/cli-run/1.json` y actualiza `schemas/index.json`. Los errores de
run conservan cli-error/1 y su registro; no se añaden códigos. Los 58 golden
siguen pasando sin regeneración ni cambios: solo hashes, longitudes y metadatos.
Se conserva `changelog.d/MAM-868.added.md`; no se edita CHANGELOG.md.

Verificación offline con temporales exclusivamente bajo `CARGO_TARGET_DIR`:
`cargo fmt --check`, `cargo clippy --workspace --all-targets --offline -- -D warnings`
y `cargo test --workspace --offline` verdes (**816 tests, 125 suites, sin fallos
ni ignorados**), incluido el test de cobertura del registro de errores.
`cargo test -p engine-cli --offline --test contract`: **3 tests OK**, sin variable
de actualización; el nombre real del paquete Cargo es `engine-cli`.
`python3 scripts/generate-schemas.py --write` y `--check` verdes;
`python3 -m unittest scripts/tests/test_schema_publication.py`: **7 tests OK**.
Pytest sobre `bindings/python/tests/test_run.py`: **9 tests y 36 subtests OK**,
con pytest y dependencias de la caché local, copia del paquete Python y extensión
nativa compilada offline; la referencia de bytes se genera con `facade_reference`.
Esta comprobación no construye ni verifica una wheel. No se usa red.
