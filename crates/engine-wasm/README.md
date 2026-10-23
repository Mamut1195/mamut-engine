# engine-wasm · ENG-SFC-001 (MAM-90)

Fachada fina de `engine-api`, con wasm-bindgen **0.2.129**. No implementa
contratos, validadores ni matemática propios. Compila también como `rlib`
nativa para las comprobaciones del workspace; la ejecución JS requiere wasm.
`#![forbid(unsafe_code)]` permanece activo, sin excepciones de la guardia.

## Build y suite V1 offline

```bash
export CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine
# En este entorno /usr/bin/cargo es 1.85.1 y no tiene std wasm32 instalado.
export PATH=/home/box/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin:$PATH
bash scripts/build-wasm.sh
```

El script usa `cargo --offline --locked`, release sin perfil especial ni
wasm-opt, CLI `/workspace/tools-bin/wbg/bin/wasm-bindgen` y Node
`/home/box/.local/bin/node`. Se pueden sustituir los dos últimos mediante
`WASM_BINDGEN` y `NODE`; la versión de CLI se comprueba antes del build.

Genera en `crates/engine-wasm/pkg/` (ignorado por Git):

- `web/engine_wasm.js`, `.wasm` y declaraciones TypeScript: módulo ES.
- `node/engine_wasm.js`, `.wasm` y declaraciones: CommonJS de prueba.
- `v1.json`: oráculo generado por `examples/v1_native.rs` directamente con
  `engine-api` nativo, con entradas, resultados, validación y progreso.
- `size-report.md`: bytes wasm originales y de ambos targets wasm-bindgen,
  crudos y gzip `-9 -n`, frente a **1.000.000 bytes gzip propuestos,
  PENDIENTES de confirmación**. Incluye herramienta y procedimiento.

La suite descubre todos los JSON válidos e inválidos publicados en
`../engine-api/fixtures/input/` (5 + 9 actualmente). Esa es la ruta existente;
no hay directorio `engine-api/src/fixtures/input`. Node verifica inventario,
`results/1` byte a byte, validación, progreso, repetibilidad, errores,
metadatos y cancelación directa. También ejecuta el target ES y el
`worker.js` real con un adaptador de `worker_threads` exclusivo de pruebas,
incluida cancelación activa desde el hilo principal. Esto no sustituye la
verificación en motores de navegador.

## API JS: strings JSON

```js
import init, { run, CancelToken } from './pkg/web/engine_wasm.js';
await init();
const token = new CancelToken();
try {
  const resultsJson = run(inputJson, token, progressJson => {
    console.log(JSON.parse(progressJson));
    // token.cancel() solicita cancelación en este punto de control.
  });
} finally {
  token.free();
}
```

| Export | Entrada | Salida |
| --- | --- | --- |
| `version()` | — | JSON canónico de engine-api |
| `capabilities()` | — | JSON canónico de engine-api |
| `schema(kind)` | nombre `engine-input/1` o `results/1` | documento JSON Schema publicado, sin reformatear |
| `explain(code, lang?)` | código del catálogo | literal string JSON con la explicación |
| `validate(input, token, progress?, lang?)` | texto `engine-input/1` | array JSON canónico de diagnósticos |
| `run(input, token, progress?, lang?)` | texto `engine-input/1` | bytes canónicos `results/1` de engine-api, sin envoltura |

Los argumentos `kind` y `code` son texto plano. Los demás documentos y
callbacks atraviesan la frontera como strings JSON, sin convertir números
ni IDs mediante objetos JS. Los errores de API en validate/schema/explain
lanzan un string JSON de `ApiError` (parseable con `JSON.parse(String(e))`).
`run` conserva los estados, diagnósticos y resultados canónicos de
engine-api, incluso con entrada inválida o cancelación. Una excepción del
callback cancela el trabajo y se vuelve a lanzar con su identidad original.
Los callbacks son síncronos: una Promise devuelta no se espera. Cada
request debe usar su propio token; no se puede restablecer un token cancelado.

## Web Worker y cancelación

`js/main.js` exporta `EngineWorker`. `request(operation, input,
{onProgress})` devuelve `{promise, cancel, cancellation}`. Un cliente admite
una operación pendiente; crear varios clientes permite trabajos independientes.
`promise` resuelve con el string JSON, y `onProgress` recibe strings JSON.
`close()` libera el worker y rechaza cualquier operación pendiente.

```js
import { EngineWorker } from './js/main.js';
const client = new EngineWorker();
const request = client.request('run', inputJson, {
  onProgress: json => console.log(JSON.parse(json)),
});
// Un botón puede ejecutar request.cancel().
try { console.log(await request.promise); }
finally { client.close(); }
```

Con `crossOriginIsolated` y `SharedArrayBuffer`, el cliente escribe un flag
con `Atomics.store`. El worker lo lee antes de iniciar y en cada callback
de progreso, y cancela el `CancelToken` de engine-api. La memoria de Rust
no se comparte: solo el flag JS de 4 bytes. También se manda `cancel` por
postMessage, pero el mensaje por sí solo queda en cola durante la llamada
síncrona del motor. La señal compartida permite observar cancelación activa
sin esperar que se atienda esa cola.

La granularidad es la de engine-api: parseo, mapeo/topología, ensamblado,
cada caso, cada iteración P-Delta/modo/dirección, combinaciones y
serialización son etapas indivisibles. No se interrumpe una factorización;
no hay plazo máximo de respuesta. El resultado de cancelación depende del
punto de control alcanzado y descarta resultados parciales. Si el trabajo
ya terminó cuando llega la señal, se conserva su resultado.

Sin memoria compartida (o con `{sharedCancellation: false}`),
`cancellation` es `terminate`: cancel() termina el worker y rechaza la
Promise con `AbortError`. No produce `results/1` cancelado. El siguiente
request crea un worker nuevo. Con memoria compartida, `cancellation` es
`cooperative` y run devuelve `results/1` con estado `cancelled`.

## Ejemplo y arnés de navegador

Después del build, ejecutar `python3 scripts/serve-wasm.py` y abrir:

- `http://127.0.0.1:8000/crates/engine-wasm/js/example.html`: ejemplo con Run/Cancel.
- `http://127.0.0.1:8000/crates/engine-wasm/js/test-browser.html`: suite compartida,
  paridad nativa directa y vía Web Worker, progreso, cancelación y fallback.

El servidor solo escucha en loopback y envía COOP `same-origin` y COEP
`require-corp`, necesarios para probar cancelación cooperativa. Servir
`.wasm` como `application/wasm`. No se requiere CDN, npm ni red externa.
El arnés publica PASS/FAIL en pantalla y en `documentElement.dataset.status`.

**Chrome/Firefox/Safari: PENDIENTE** en este entorno. Chrome está instalado,
pero el sandbox prohíbe sockets y su arranque; véase
[provenance](../../provenance/ENG-SFC-001.md). No se declara paridad de
navegadores a partir del adaptador Node.


## Explicit language (MAM-836, phase 1)

`explain(code, lang?)`, `validate(input, token, progress?, lang?)` and
`run(input, token, progress?, lang?)` accept a final optional `"en"`, `"es"` or
`"pt"`. Omitted/undefined language preserves existing English bytes. An invalid
identifier throws the existing serialized `ApiError::InvalidInput` string.
No locale or environment settings select a language. For example:

```js
const report = validate(inputJson, token, undefined, "es");
const results = run(inputJson, token, undefined, "pt");
const explanation = explain("ENG-E-0004", "es");
```

All localization delegates to engine-api: stable-code explanations and exact
English default ENG diagnostic causes are translated. Specific/parameterized
causes, hints and ApiError/parse messages remain unchanged for phase 2, alongside
Python-owned PY-E-* messages, mcp/compile/methods and describe/help language.
The CLI workflow fingerprint excludes diagnostic cause/hint text and therefore
does not attest to message wording; this JSON binding does not create manifests.
Translations await native-speaker review, especially Portuguese.
