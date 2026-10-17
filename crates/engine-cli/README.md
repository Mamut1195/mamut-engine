# ENG-SFC-002 · MAM-91 · mamut-engine

CLI nativo del Engine, paquete `engine-cli`, binario `mamut-engine`, miembro y
miembro por defecto del workspace. Rust 1.99 según `rust-toolchain.toml`;
`license.workspace = true` hereda **AGPL-3.0-only**. Sin dependencias externas
nuevas, red, telemetría ni procesos auxiliares.

El bloqueo anterior queda resuelto por SDK-WF-012 / MAM-493. Se usan las
bibliotecas vendorizadas `mamut-workflow-cli` y `mamut-workflow-execution` del
commit `1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`. `EngineCli::run` llama al
`parse_args` público y a un único `dispatch` de la base con `CliConfig`,
`MethodProvider`, `CliIdentity` y `ExitCodes::engine_v1()`. No hay parser,
compilador, executor ni despacho de comandos propios. No se modifica vendor.

## Uso y entrada

Desde la raíz del repositorio, con Cargo offline:

```sh
cargo build -p engine-cli --offline
cargo run -p engine-cli --offline -- describe
cargo run -p engine-cli --offline -- methods --catalog crates/engine-cli/catalog.json
MAMUT_ENGINE_INPUT=crates/engine-api/tests/fixtures/determinism/cantilever.json \
  cargo run -p engine-cli --offline -- run \
  --catalog crates/engine-cli/catalog.json \
  --workflow crates/engine-cli/run.workflow.json --progress ndjson
```

`catalog.json` es el catálogo propio, también embebido como `engine_cli::CATALOG`.
Registra `engine.run` v1 y `engine.validate` v1. El primero llama a
`engine_api::run`; soporta los análisis que publique esa fachada (actualmente
linearStatic, pDelta, modal y responseSpectrum). El segundo llama a
`engine_api::validate_with`. Para usarlo, ejecutar el mismo comando `run` con
`validate.workflow.json`. Ambos métodos reciben parámetros `{}`; el proveedor
rechaza parámetros numéricos y métodos/versiones ajenos, sin usar IdentityProvider.

**Decisión abierta de transporte:** `engine-input/1` es un documento estructurado
y no cabe en `NumericParameters`. La vía mínima actual es
`MAMUT_ENGINE_INPUT=<ruta-local>` (en Rust, `EngineCli::new(Some(path))`). El
método abre esa ruta al ejecutarse, con el límite `engine_api::MAX_INPUT_BYTES`;
una variable ausente, error de lectura, UTF-8 inválido o exceso de tamaño da 2.
No se añade un flag, comando ni parser de entrada. Cada paso lee la misma ruta;
el archivo debe permanecer estable durante la invocación. Queda pendiente un
transporte de documentos aprobado por el SDK, con entradas por paso y stdin
directo de engine-input/1. Aquí `-` se reserva al documento de catálogo o workflow
que lee la base; en la variable de entorno se trata como ruta literal.

`--workflow -` **o** `--catalog -` consume stdin hasta EOF, con el límite de 16 MiB
de la base. Ambos a la vez son uso inválido (2). Por ejemplo:

```sh
MAMUT_ENGINE_INPUT=modelo.json mamut-engine run \
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
conservan datos y manifiesto con `outcome: "warning"` y código 1.

Ante fallos de entrada, método o cancelación stdout queda vacío; stderr termina
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

El status de `Outcome` determina el mapeo de run. En validate, errores de contrato
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
No se añaden comandos, flags, hilos ni manejadores de señales de SO. Ctrl-C
conserva el comportamiento del SO y no garantiza el código cooperativo 5.

## Correspondencia real con engine-api

| Operación engine-api | Comando estándar / cobertura real | Pendiente |
| --- | --- | --- |
| version | `describe.version` es la versión de engine-api; run incluye engineVersion | No publica `version().contracts`; no hay comando version ni flag --version |
| capabilities | `describe` publica identidad, comandos y códigos engine_v1; `methods --catalog …` lista engine.run y engine.validate | No publica el documento completo capabilities (elementos, métodos de análisis, esquemas) |
| schema | Los descriptores de methods referencian contratos y parámetros; no son los JSON Schema de engine-api | Sin comando estándar para `schema("engine-input/1")` / `schema("results/1")` |
| explain | Los resultados conservan diagnósticos del motor | Sin comando estándar para `engine_api::explain(code)` |
| validate | `validate --catalog … --workflow …` valida/compila **el workflow**; `run` seleccionando engine.validate valida engine-input/1 | No existe comando estándar de validación directa de entrada Engine |
| run | `run --catalog … --workflow …` ejecuta engine.run con engine-api y entrega su resultado | Aprobar transporte/extension de salida anteriores |

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
stdin de catálogo/workflow, NDJSON, igualdad del resultado con engine-api,
conservación del manifiesto, orden de resultados de varios pasos y distinción
de las dos validaciones. El 4 incluye
inestabilidad y P-Delta sin convergencia; el 70 del binario usa un pipe stdout
cerrado antes de entregar el workflow por stdin, sin depender de una carrera.
El 5 usa un marcador antes y después de lanzar el binario, mientras stdin
retiene el workflow; no depende de sleeps ni de señales. El 6 queda reservado.

La verificación offline y el recuento de pruebas del 2026-10-17 se registran en
`provenance/ENG-SFC-002.md`, con Rust 1.99.0 y
`CARGO_TARGET_DIR=/workspace/.cargo-target/me-sfc2`.

El check `wasm32-unknown-unknown` comprueba compilación de todo el workspace,
incluido este crate; no requiere exclusiones. No implica ejecución del CLI:
el binario necesita stdio, archivos y entorno de un anfitrión nativo.

**Pendiente:** binarios Windows/Linux/macOS en CI (**lo decide Jonathan**),
aprobación de decisiones de contratos y revisión
clean-room **PENDIENTE (J4)**. No se modifica `.github/`, `clean-room/` ni vendor.
