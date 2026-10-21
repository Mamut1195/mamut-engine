# ENG-SFC-002 · MAM-91 · mamut-engine

CLI nativo del Engine, paquete `engine-cli`, binario `mamut-engine`, miembro y
miembro por defecto del workspace. Rust 1.99 según `rust-toolchain.toml`;
`license.workspace = true` hereda **AGPL-3.0-only**. Sin dependencias externas
nuevas, red, telemetría ni procesos auxiliares.

El bloqueo anterior queda resuelto por SDK-WF-012 / MAM-493. Se usan las
bibliotecas vendorizadas `mamut-workflow-cli` y `mamut-workflow-execution` del
commit `1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`. `EngineCli::run` llama al
`parse_args` público y a un único `dispatch` de la base para workflows con `CliConfig`,
`MethodProvider`, `CliIdentity` y `ExitCodes::engine_v1()`. El adaptador hace un
pre-parseo de `--input` y los documentos predeterminados; la compilación y la
ejecución de workflows siguen en la base. `validate --input` sin documentos de
workflow llama directamente a `engine_api::validate_with`. No se modifica vendor.

## Uso y entrada

Desde la raíz del repositorio, con Cargo offline:

```sh
cargo build -p engine-cli --offline
cargo run -p engine-cli --offline -- describe
cargo run -p engine-cli --offline -- methods --catalog crates/engine-cli/catalog.json
cargo run -p engine-cli --offline -- run \
  --input crates/engine-api/tests/fixtures/determinism/cantilever.json --progress ndjson
cargo run -p engine-cli --offline -- run --input - < modelo.json
cargo run -p engine-cli --offline -- validate --input modelo.json
cargo run -p engine-cli --offline -- validate --input - < modelo.json
```

`catalog.json` es el catálogo propio, también embebido como `engine_cli::CATALOG`.
Registra `engine.run` v1 y `engine.validate` v1. El primero llama a
`engine_api::run`; soporta los análisis que publique esa fachada (actualmente
linearStatic, pDelta, modal y responseSpectrum). El segundo llama a
`engine_api::validate_with`. También puede seleccionarse desde `run` con
`validate.workflow.json`. Ambos métodos reciben parámetros `{}`; el proveedor
rechaza parámetros numéricos y métodos/versiones ajenos, sin usar IdentityProvider.

Si se omiten tanto `--catalog` como `--workflow`, `run` usa `engine_cli::CATALOG`
y `engine_cli::RUN_WORKFLOW`. `validate --input` valida directamente el modelo,
sin workflow ni temporales. Por compatibilidad, `validate` sin `--input` sigue
usando el catálogo y `engine_cli::VALIDATE_WORKFLOW`. Los documentos están embebidos con
`include_str!`. El adaptador los materializa en un directorio temporal privado
para el despacho de la base y lo elimina al terminar; sus rutas no aparecen en
la salida. Con ambos argumentos explícitos se conserva el comportamiento
anterior: `run` ejecuta el workflow elegido y `validate` solo valida/compila ese
workflow, sin ejecutar engine-api. Especificar solo uno de ellos sigue siendo
uso inválido (2).

**Decisión abierta de transporte (adopción en el SDK):** `--input <ruta|->` es
ahora la vía principal de engine-input/1 en el adaptador. `-` lee stdin hasta
EOF. Stdin se lee entero una sola vez antes del despacho y todos los pasos
reutilizan el documento en memoria; no hay streaming por paso. La lectura se
detiene al detectar exceso del límite `engine_api::MAX_INPUT_BYTES`
(8 MiB, 8 388 608 bytes); exactamente ese tamaño se admite. Exceso, ruta
inexistente, error de lectura o UTF-8 inválido dan código 2, stdout vacío y
sobre de error sanitizado en stderr. Repetir `--input` también da 2.

Se requiere un espacio entre flag y valor: `--input=valor` se rechaza con 2,
igual que `--catalog=valor` y `--workflow=valor` en el parser de la base. Se
conserva una única sintaxis de opciones. El token `--` no actúa como separador:
se rechaza con 2, también antes o después de `--input`. `--input` sin valor,
con valor vacío o seguido de otro flag (incluido `-h`) da 2; `-` es el único
valor que puede empezar con guion. Para una ruta cuyo nombre empieza con guion,
use un prefijo como `./-modelo.json`.

`MAMUT_ENGINE_INPUT=<ruta-local>` se mantiene por compatibilidad (en Rust,
`EngineCli::new(Some(path))`), con los mismos bytes de salida para los comandos
con catálogo/workflow explícitos. **Si se indican ambos, gana `--input`** y no
se abre la ruta de la variable. Sin `--input`, la ruta se lee una vez al inicio
de `run`, pero sus errores se notifican al ejecutar el método, conservando la
prioridad de cancelación y el progreso existentes. En la variable, `-` sigue
siendo una ruta literal. MCP conserva la lectura por llamada de la vía antigua.
Sigue abierta la adopción del transporte de documentos en el SDK y la entrada
independiente por paso; engine-input/1 no cabe en `NumericParameters`.

`--workflow -` **o** `--catalog -` consume stdin hasta EOF, con el límite de 16 MiB
de la base. Ambos a la vez son uso inválido (2), al igual que `--input -` junto
con cualquiera de ellos. Una entrada por ruta permite stdin de workflow o
catálogo. Por ejemplo:

```sh
mamut-engine run --input modelo.json \
  --catalog crates/engine-cli/catalog.json --workflow - < crates/engine-cli/run.workflow.json
```

Los descriptores usan cantidades adimensionales para las referencias opacas a
entrada/salida, no para representar magnitudes del modelo. La referencia local
`urn:mamut:engine:engine-input:1.0.0` remite a engine-input/1;
`urn:mamut:engine:operation-result:1.0.0` designa el resultado del método:
results/1 para run, `{operation:"validate",status,diagnostics}` para validate.
Las unidades y magnitudes físicas se validan dentro de engine-api (SI).

## Salida, códigos y cancelación

La identidad configurada es `engineId: "mamut-engine"`, `engineVersion` obtenido
de `engine_api::version().engine_version`, `operation: "run"`. La operación del
sobre representa la ejecución del workflow; el método de validación informa
`operation: "validate"` en su propio resultado.

La base entrega un sobre cli-run/1 cuyo `data` es el manifiesto de ejecución.
Este almacena **huellas**, no los valores calculados. El adaptador conserva ese
manifiesto íntegro y añade `engineResults` al sobre terminal: array de documentos
completos de engine-api, en orden de pasos. Así stdout de `run` contiene el
results/1 real, incluidos desplazamientos, reacciones y diagnósticos. Los avisos
de run conservan datos y manifiesto con `outcome: "warning"` y código 1.

Desde MAM-863 / ENG-SFC-012, `validate --input <ruta|->` sin catálogo/workflow
emite directamente un único JSON compacto, con claves ordenadas y newline final:

```json
{"diagnostics":[],"operation":"validate","schema":"validate-report/1","status":"ok"}
```

Esta decisión sustituye el sobre cli-run/1 de la abreviatura introducida en
#197 (MAM-862). Su contrato publicado es
[`validate-report/1`](../engine-api/schemas/validate-report-1.json), disponible
también por `engine_api::schema("validate-report/1")` y registrado en capabilities.
`status` es `ok` (0), `warning` (1) o `invalid` (2). Es un dry-run del **modelo**:
comprueba parseo JSON estricto, DTO/esquema, convenciones, límites y topología
mediante `validate_with`. No ensambla/resuelve el sistema ni certifica estabilidad,
convergencia o resultados. No se añade el alias opcional `run --dry-run`.

En los tres estados el informe va a stdout y stderr queda vacío. Errores de
parseo, DTO, contrato o límites internos se normalizan a `ENG-E-0009`
(contrato incompatible), con causa de la API y entidades vacías. Los diagnósticos
del modelo se conservan íntegros y en su orden original; cualquier severidad
`error` produce `invalid`/2. Los fallos de transporte (incluido exceso de 8 MiB),
uso y cancelación conservan stdout vacío y error sanitizado en stderr como antes;
la cancelación da 5 y un fallo de escritura da 70.

Actualmente `validate_with` no genera avisos y no hay fixture de validación que
dé 1. El mapeo warning/1 se prueba con diagnóstico sintético; no se inventan
avisos para el binario. El aviso de recuperación de diafragmas de E1 se genera
solo durante run, por lo que E1 da 0 al validar. `validate --catalog … --workflow …`
mantiene exactamente su validación/compilación de workflow, incluso con `--input`.
La forma directa rechaza flags de workflow como `--progress ndjson` con uso/2.

En la ejecución de workflows, ante fallos de entrada, método o cancelación stdout queda vacío; stderr termina
con el sobre de error. Un fallo de E/S puede dejar una escritura incompleta.
Si engine-api produjo un documento fallido, `engineResults` contiene solo ese
documento, sin resultados de pasos previos ni manifiesto parcial. Errores de
transporte o cancelación previa no tienen documentos. Las causas de E/S y rutas
no se incluyen en los mensajes del adaptador. Los diagnósticos del motor se
conservan como los entrega su API.

**Decisión abierta de salida:** aprobar la extensión `engineResults` de cli-run/1
y el informe del método validate como contratos publicados, o incorporar en el
SDK un canal estándar de resultados completos y fallos tipados. El adaptador
retiene resultados/semántica en estado compartido del proveedor porque el
`Method::run` vendorizado solo devuelve `Result<Value, String>`; no infiere estados
analizando mensajes. El callback de progreso y el manifiesto siguen siendo del SDK.

| Estado/categoría | Código | Mapeo |
| --- | --- | --- |
| OK | 0 | `ExitCodes.ok` |
| Warning | 1 | `ExitCodes.warning`; la base no tiene FailureKind de aviso |
| Uso, documento, workflow o entrada Engine inválidos | 2 | Usage / Document / Validation |
| Unsupported | 3 | FailureKind::Unsupported |
| NotConverged o Unstable | 4 | FailureKind::NotConverged |
| Cancelled | 5 | FailureKind::Cancelled |
| Timeout | 6 | Reservado; no se instala temporizador |
| InternalError, fallo sin clasificación o E/S de salida/progreso | 70 | Internal / Method |

El status de `Outcome` determina el mapeo de run. En el método engine.validate
de un workflow, errores de contrato
son Invalid; los diagnósticos de error del catálogo 0006/0007/0012/0013 se mapean
como en engine-api, y los restantes son Invalid. Un diagnóstico warning sin errores
da Warning. Un contrato de entrada desconocido es inválido (2), como en engine-api.

`--progress ndjson` usa los eventos de la base en stderr: `started` y `finished`
por paso, sin timestamps, escritos y vaciados inmediatamente. El fallo de método
emite started y el sobre final, sin finished. stdout sigue siendo un único JSON.
Las etapas internas `engine_api::Progress` están disponibles en biblioteca mediante
`with_engine_progress`; aún no forman otro dialecto NDJSON.

`Control` conecta `CancelFlag` con un `Arc<engine_api::CancelToken>`. Se sincronizan
antes de ejecutar y después de cada callback del motor; el token también es
consultado por engine-api entre etapas/iteraciones. `Control::cancel()` cancela
ambos; `flag()` y `token()` permiten conectar observadores del anfitrión.
La API de biblioteca puede cancelar antes de empezar o durante una etapa completada.

El binario admite `MAMUT_ENGINE_CANCEL_FILE=<ruta-local>`: la presencia de esa
ruta solicita cancelación cooperativa (5). Puede existir antes de iniciar o
crearse durante la ejecución; se consulta antes de cada paso y después de los
callbacks del motor. En biblioteca se configura con `Control::with_cancel_file`.
No se crea ni elimina el marcador; use una ruta propia por ejecución. Una ruta
ausente o cuya existencia no pueda comprobarse no solicita cancelación. Una vez
observada, la cancelación permanece activa aunque se retire el marcador.
La cancelación no añade hilos ni manejadores de señales de SO. Ctrl-C
conserva el comportamiento del SO y no garantiza el código cooperativo 5.

## Correspondencia real con engine-api

| Operación engine-api | Comando estándar / cobertura real | Pendiente |
| --- | --- | --- |
| version | `describe.version` es la versión de engine-api; run incluye engineVersion | No publica `version().contracts`; no hay comando version ni flag --version |
| capabilities | `describe` publica identidad, comandos y códigos engine_v1; `methods --catalog …` lista engine.run y engine.validate | No publica el documento completo capabilities (elementos, métodos de análisis, esquemas) |
| schema | Los descriptores de methods referencian contratos y parámetros; no son los JSON Schema de engine-api | Sin comando estándar para `schema("engine-input/1")` / `schema("results/1")` |
| explain | Los resultados conservan diagnósticos del motor | Sin comando estándar para `engine_api::explain(code)` |
| validate | `validate --input …` valida directamente el modelo y emite validate-report/1; `validate --catalog … --workflow …` conserva la validación/compilación **del workflow** | Adopción del transporte en el SDK |
| run | `run --input …` ejecuta engine.run con documentos embebidos; admite catálogo/workflow explícitos | Aprobar transporte/extensión de salida anteriores |

Los comandos/flags `capabilities`, `schema`, `explain`, `version` y `--version`
no se añaden: el parser de la base los rechaza con 2. `compile` conserva el plan
estándar de workflow. La tabla no declara cubiertas operaciones sin equivalente.

La base también conserva `mcp --catalog …`: inyecta proveedor, identidad,
cancelación y tabla engine_v1, pero sus tools siguen entregando manifiestos de
huellas y errores de método sin clasificación (70). La extensión terminal de run
no se aplica a los frames MCP; warnings allí conservan el comportamiento genérico.
La paridad MCP de datos completos y códigos semánticos es una decisión abierta
fuera de la correspondencia describe/methods/validate/run de este ticket.

## Verificación y plataforma

Los tests propios solo crean archivos locales y ejecutan el propio binario mediante
`env!("CARGO_BIN_EXE_mamut-engine")`; la cancelación entre etapas y fallos de
escritores se prueban además en biblioteca, sin señales, shells ni red.
Cubren **0/1/2/3/4/5/70 con ejecuciones reales del binario**, identidad/tabla en describe,
stdin de entrada/catálogo/workflow, límites de 8 MiB, UTF-8 inválido, conflictos
de stdin, precedencia sobre la variable y bytes idénticos entre transportes,
NDJSON, igualdad del resultado con engine-api,
conservación del manifiesto, orden de resultados de varios pasos y distinción
de las dos validaciones. El 4 incluye
inestabilidad y P-Delta sin convergencia; el 70 del binario usa un pipe stdout
cerrado antes de entregar el workflow por stdin, sin depender de una carrera.
El 5 usa un marcador antes y después de lanzar el binario, mientras stdin
retiene el workflow; no depende de sleeps ni de señales. El 6 queda reservado.

MAM-863 añade pruebas del binario para informes válidos/invalid por esquema,
límites internos y topología, determinismo stdin/ruta, cancelación y ausencia de
resolución. Reutiliza el validador de esquemas de pruebas de engine-api, con
mutaciones negativas en cada nivel, sin dependencias nuevas ni cambios en los
otros esquemas. La rama de aviso se prueba en el clasificador del informe.

Rendimiento reproducible (en proceso, mediana de 11 repeticiones alternadas tras
calentar ambos caminos; solo `runs[0].engine_input` de E1):

```sh
CARGO_TARGET_DIR=/workspace/.cargo-target/sfc12 cargo test -p engine-api \
  --test validate_performance --offline -- --nocapture
```

Medición debug con Rust 1.99.0 del 2026-10-21: validate **5,44 ms**, run
**151,46 ms**, ratio **27,84×**. El test exige ≥10× y permanece activo en CI. La deserialización y la
extracción del input quedan fuera de la medición; no se consultan valores de
referencia del fixture. Véase [provenance ENG-SFC-012](../../provenance/ENG-SFC-012.md).

La verificación offline y el recuento de pruebas del 2026-10-17 se registran en
`provenance/ENG-SFC-002.md`, con Rust 1.99.0 y
`CARGO_TARGET_DIR=/workspace/.cargo-target/me-sfc2`.

El check `wasm32-unknown-unknown` comprueba compilación de todo el workspace,
incluido este crate; no requiere exclusiones. No implica ejecución del CLI:
el binario necesita stdio, archivos y entorno de un anfitrión nativo.

**Pendiente:** binarios Windows/Linux/macOS en CI (**lo decide Jonathan**),
aprobación de decisiones de contratos y revisión
clean-room **PENDIENTE (J4)**. No se modifica `.github/`, `clean-room/` ni vendor.
