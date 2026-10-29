# ENG-SFC-004 · MAM-92 · Contrato CLI con Harness

Decisión de Jonathan del **2026-10-29**: V1 integra el motor con Harness
exclusivamente mediante CLI. ENG-SFC-003 (MCP) queda fuera de V1; este trabajo
no modifica ese transporte.

La [ficha del programa](../../crates/engine-cli/harness/mamut-engine.cli-program.json)
declara `program_id: mamut-engine`, versión `0.1.0` y el ejecutable Windows
`bin/mamut-engine.exe`, relativo a la raíz del paquete distribuido, como
suposición declarada. La ficha es **experimental**: Harness real todavía no
la ha probado en Windows.
No constituye evidencia de empaquetado, registro ni ejecución en ese sistema.

Harness consume la ficha conforme a `cli-program.schema.json`, versión `1.0.0`,
del repositorio `harness-determinista` en el commit
`85f912ea3225fe72c9f0f475a43dce8d6f5e9ec4`. Resuelve el ejecutable dentro del
paquete, selecciona `capability_id` y lanza exclusivamente su vector
`fixed_arguments`, sin shell ni argumentos derivados del documento. Para
`input: json` escribe un documento UTF-8 por stdin y cierra el canal; para
`input: none` no envía documento. Recoge stdout/stderr y el código de salida,
con entorno mínimo, plazo y límite de salida de la operación.

| Capacidad | Argumentos | Entrada | Salida | Plazo externo | Límite de salida |
| --- | --- | --- | --- | --- | --- |
| `mamut-engine.run` | `run --input -` | JSON | cli-run/1 con `engineResults` results/1 | 300 s | 64 MiB |
| `mamut-engine.validate` | `validate --input -` | JSON | validate-report/1 | 30 s | 64 MiB |
| `mamut-engine.describe` | `describe --json` | Ninguna | cli-describe/1 | 5 s | 1 MiB |

`MAX_INPUT_BYTES` admite hasta **8 MiB (8 388 608 bytes)**, inclusive, para
ambas operaciones de modelo. La ficha no tiene campo para ese límite de entrada;
el consumidor debe respetarlo. Los 64 MiB son ocho veces ese límite y el máximo
permitido por el registro. Son un presupuesto de transporte: no garantizan que
todos los modelos admitidos por los límites del motor produzcan una respuesta
menor. Harness debe rechazar una salida que lo exceda. El plazo también pertenece
al host; no añade `--timeout` a los argumentos ni equivale al código cooperativo 6
si el host termina el proceso.

Las tres operaciones declaran `mutates_external_state: false`: no publican ni
persisten resultados fuera del proceso. `run` materializa documentos embebidos
en un directorio temporal privado y lo elimina al finalizar normalmente;
esa implementación requiere un directorio temporal escribible. Una terminación
forzada puede dejar temporales. La prueba conserva únicamente las variables de
temporales y, en Windows, `SystemRoot`; no hereda `PATH` ni variables `MAMUT_*`.

Los códigos publicados son 0 éxito, 1 aviso, 2 entrada/uso inválido,
3 no soportado, 4 inestabilidad/no convergencia, 5 cancelación,
6 timeout cooperativo y 70 error interno. En `run`, éxito y aviso entregan
stdout JSON; los errores terminales dejan stdout vacío y entregan cli-error/1
por stderr. En validación directa, los estados `ok`, `warning` e `invalid`
entregan validate-report/1 por stdout con códigos 0, 1 y 2; los fallos de
transporte usan stderr. `describe --json` devuelve 0 con su descripción JSON.

[harness_contract.rs](../../crates/engine-cli/tests/harness_contract.rs) valida
la ficha mediante el validador compartido de engine-api y una copia exacta del
schema de Harness. Complementa manualmente sus patrones y longitudes de cadenas;
`const`, referencias, propiedades cerradas, tipos y límites numéricos siguen en
el validador compartido. Incluye mutaciones negativas de esas reglas.

Cada operación ejecuta el binario real dos veces con exactamente los argumentos
registrados, stdin según la ficha y `env_clear`. Se comprueban códigos, schemas,
JSON parseable, lectura acotada y plazo externo. El fixture propio `cantilever`
se reutiliza; para `run`, cada results/1 debe contener el SHA-256 esperado de
la entrada según RFC 8785, calculado con `engine_api::input_sha256`. El proceso
de prueba no invoca la ejecución del motor mediante una librería. La versión
se compara con `describe` y con la del crate.

La repetición exige stdout idéntico byte a byte, sin retirar campos. Una entrada
con claves reordenadas debe conservar hash y stdout. También se repiten aviso
de ejecución (1), validación del mismo modelo (0) y entrada inválida (2),
comparando ambos canales. validate-report/1 y cli-describe/1 no contienen
`input_sha256`; no se les añade ese campo.

Limitaciones conocidas (H2/H3, J4 engine#248 r1): el nuevo test de contrato solo
ejercita códigos 0/1/2; el resto tiene cobertura en `crates/engine-cli/tests/cli.rs`.
El reorden solo invierte claves de primer nivel. El esquema cli-program 1.0.0
acepta cualquier subcomando: la correspondencia ficha-binario solo la garantiza
el test, con argumentos fijados a mano.

El kit **SDK-WF-009 / `mamut-tool-conformance`** está excluido del vendor,
según su [README](../../vendor/mamut-workflows/README.md). No se incorpora ni
se reimplementa. Las familias de comprobaciones solicitadas ya tienen esta
cobertura local, que no certifica equivalencia con todas las aserciones del kit:

| Familia | Evidencia existente en engine-cli |
| --- | --- |
| Descripción e identidad | `tests/describe.rs`: schema cli-describe/1, versión real, tabla de códigos, repetición y snapshot; `tests/cli.rs::describe_identifies_engine_and_publishes_engine_v1_table`: descripción histórica |
| Códigos de salida | `tests/cli.rs`: casos reales 0/1/2/3/4/5/6/70, cancelación, timeout, errores y validación directa; `tests/support/schema.rs`: errores registrados |
| cli-run/1 | `tests/contract.rs`: schemas, resultados completos, manifiestos, huellas, salidas golden, repetición y mutaciones; reutiliza `engine-api/tests/support/schema.rs` |

**DESCONOCIDOS / PENDIENTES:** SDK-ADO-014 no está en Linear, según el contexto
de esta tarea; no se verificó en línea. ENG-SFC-003 corresponde a MCP y queda
fuera de V1. El kit real SDK-WF-009 **NO EJECUTADO: PENDIENTE / DESCONOCIDO**;
su resultado completo no se infiere de los tests locales. Los tests Rust en
`crates/engine-cli/tests/harness_contract.rs` fueron ejecutados y verificados
por J4 en r1 de engine#248 (workspace **893 passed / 0 failed**, CI verde),
fusionado en main como `4603d3d`. La integración real de Harness en Windows
queda pendiente.
