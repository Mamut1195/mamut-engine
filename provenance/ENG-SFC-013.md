# MAM-864 / ENG-SFC-013 · Autodescripción del CLI para agentes

Fecha: **2026-10-22**. Autor: Codex, modelo **gpt-6.1-sol high**, implementador.
Rama de trabajo basada en `origin/main`, con las incorporaciones de transporte
de entrada, validación directa, timeout, señales y publicación de esquemas
ya presentes.

## Fuentes y alcance

Se consultaron exclusivamente código y documentación locales: la fachada pública
`engine-api` (`version`, `capabilities`, `schema_kinds`, `schema`, `explain` y sus
constantes públicas), el adaptador `engine-cli`, el parser y despacho de la base
vendorizada, `ExitCodes::engine_v1`, los esquemas y su publicador, las pruebas
existentes y las convenciones de changelog y determinismo.

Sin red ni dependencias nuevas. No se modifican vendor, Cargo, configuración de
CI, archivos de protección, las reglas clean-room, barra axial ni truss. No se
accede a procedencia restringida, directorios de auditoría, fuentes externas de
referencia ni logs de otras sesiones. No se cambia cálculo ni serialización de
run. No se ejecutan `git add` ni commit. Los temporales de Cargo, Rust y Python
y los registros de verificación se sitúan en el directorio de compilación
configurado, con el directorio temporal del entorno apuntando a una subcarpeta
del mismo.

## Decisiones y contratos

La lógica se concentra en `crates/engine-cli/src/describe.rs`; `lib.rs` solo
registra el módulo y lo invoca después de validar el timeout y antes de preparar
entrada/workflows. Las consultas nuevas no abren archivos, leen stdin, crean
workflows temporales, instalan señales ni inician temporizadores. Conservan la
validación y precedencia flag/entorno del timeout existente.

- `describe --json`: documento `cli-describe/1` con identidad real, comandos,
  aliases, argumentos, flags, subcomandos vacíos porque no existen comandos
  anidados, entorno, entradas/salidas, tabla efectiva de salida y límites públicos.
- `--help --json`, `help --json`, `-h --json`: bytes idénticos a la descripción.
- `--version`: texto `mamut-engine <engine_version>` y newline.
  `--version --json` serializa `engine_api::version()` sin campos añadidos;
  su contrato externo es `version/1`. No se añade un subcomando `version`.
- `capabilities`: serializa directamente el documento de la API y reutiliza
  capabilities/1. No acepta `--json` porque su única salida es JSON.
- `schema <id>`: `write_all` de los bytes originales de `engine_api::schema`,
  sin reserializar ni añadir newline. Admite exactamente los ids de la API.
- `explain <code>`: explicación original más newline; con `--json`, exactamente
  el objeto `{code, explanation}`, contrato explain/1.

Éxito: 0. Id/código desconocido, argumento ausente o sobrante: 2, stdout vacío y
un sobre `cli-error/1` en stderr con `CLI-E-USAGE`, registrado por main.
Fallos de escritura: categoría interna, 70 y `CLI-E-INTERNAL`, también registrado,
con posibilidad de stdout incompleto como en los comandos existentes.

La base ya ofrece `describe` con un documento distinto: **se conserva sin cambios
cuando no lleva `--json`**, incluidos sus metadatos históricos de timeout
reservado. La versión nueva anuncia timeout efectivo/6 y conserva las categorías
que comparten código mediante `exitCodeCategories`. La base no ofrece las otras
consultas nuevas. La ayuda textual ya toma `mamut-engine` de
`CliConfig::with_identity`; se verifica también la ayuda anidada y no se requiere
sustitución ni edición del vendor. Estas decisiones constan en el README inglés,
siguiendo el idioma de su sección en main.

El inventario de esquemas usa `schema_kinds()` y `schema()` de la fachada y el
índice publicado embebido mediante `include_str!`. Cada entrada anuncia id de
contrato, `$id`, SHA-256 y `availableViaSchema`. Las huellas se toman del índice,
que se verifica contra los bytes originales mediante pruebas y el publicador;
no se añade una implementación ni dependencia criptográfica al adaptador.
El inventario incluye `cli-error/1` de main y la lista de salidas lo anuncia
como contrato terminal de stderr. Los esquemas adicionales del CLI se anuncian
como publicación estática y no amplían los ids admitidos por la API. Cambiar la
publicación requiere regenerar el índice y recompilar el binario.

Los tres esquemas nuevos se guardan en `crates/engine-cli/schemas/`, se registran
en `scripts/schema-sources.json` y se publican con
`python3 scripts/generate-schemas.py --write`. No se cambia ningún byte de los
esquemas previos de la API ni de su publicación.

Merge: se integran los cambios de `origin/main` (#205 y #183), conservando el
emisor `cli-error/1`, su registro de códigos y su publicación junto a los tres
esquemas de MAM-864; el merge queda sin commit.

## Pruebas y verificación

Siete pruebas nuevas con el binario comprueban contratos, determinismo y snapshot
SHA-256 versionado del stdout completo de describe, equivalencia de la ayuda JSON,
igualdad con version/capabilities/explain de la API, inventario y huellas contra
cada archivo publicado, todos los esquemas API byte por byte, ayuda textual,
códigos 0/2, argumentos inválidos y precedencia/validación del timeout. El
snapshot evita copiar salidas completas en archivos fuente. El validador es el
existente en `engine-api/tests/support/schema.rs`; para capabilities se resuelve
su referencia externa al esquema real de method-descriptor en memoria, sin red.
SHA-256 de las pruebas usa la biblioteca estándar de Python, ya requerida por la
publicación. Las mutaciones negativas comprueban campos requeridos, adicionales
y tipos anidados. Se retiran de un test anterior solo las dos expectativas de
rechazo de comandos que ahora están implementados.

Una octava prueba comprueba los fallos de escritura de todas las consultas,
incluidas las variantes JSON, contra el esquema `cli-error/1` y el registro de
códigos de main. Los rechazos de argumentos también usan ese validador y
comprueban `CLI-E-USAGE` y la operación sanitizada. Se actualiza el snapshot
de describe para incluir `cli-error/1` como esquema publicado y salida terminal.

Entorno: Cargo offline, herramientas Rust disponibles localmente y temporales
en el directorio de compilación configurado.

Rust/Cargo **1.99.0**, plataforma nativa Linux. Resultados finales:

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 821 passed / 0 failed / 0 ignored |
| `python3 scripts/generate-schemas.py --check` | OK; bytes, índice e inventario |
| `python3 -m unittest scripts/tests/test_schema_publication.py` | 7 tests, OK |
| Pruebas de engine-cli | 12 unitarias + 40 CLI + 8 consultas + 7 registro + 1 esquema de error, OK |
| Identificadores prohibidos en texto nuevo | Sin coincidencias; se conserva el texto heredado de main |
| `git diff --check` | OK |

Las pruebas existentes de run conservan comparación de bytes, manifiestos y
resultados, incluidos stdin/ruta, workflows embebidos/explícitos, timeout y señales.
Revisión humana clean-room externa: **PENDIENTE (J4)**.
