# MAM-863 / ENG-SFC-012 · validate --input directo

Fecha: **2026-10-21**. Autor: Codex (GPT-6), implementador.
Worktree: `/workspace/mamut-engine-sfc12`, rama apilada sobre MAM-862 (#197).

## Fuentes y alcance

- Código local de `engine-cli`, `engine-api::validate/validate_with`, límites,
  decodificación de DTO, `run::validation`, validador del modelo y catálogo de
  diagnósticos; documentación local de determinismo y contratos JSON existentes.
- Para rendimiento, exclusivamente `runs[0].engine_input` de
  `docs/validation-report/fixtures/val032/E1-v1.1.2.json`. El test deserializa
  structs con solo `runs[].engine_input`, ignorando los demás campos. No consulta
  ni imprime valores de referencia/oráculo; mide la API propia en proceso.
- Sin red, dependencias nuevas ni cambios en Cargo, vendor, `.github`,
  `clean-room`, barra axial o truss. No se accede al provenance restringido,
  directorios de auditoría ni logs de otras sesiones. No se cambia aritmética,
  resolución, fixtures ni bytes de los otros esquemas. Sin commit.

## Decisiones de contrato

`validate --input <ruta|->`, sin catálogo/workflow explícitos, llama directamente
a `validate_with`. No crea workflow, manifiesto, huellas ni archivos temporales.
Se sustituye deliberadamente el sobre cli-run/1 de esa abreviatura de #197, aún
sin fusionar, por un único JSON determinista `validate-report/1` en stdout:
`schema`, `operation: "validate"`, `status`, `diagnostics`, con newline final.
Identidad mínima mediante esquema y operación, sin timestamps ni rutas locales.

Estados y códigos: `ok`/0, `warning`/1, `invalid`/2. Cualquier diagnóstico de
severidad error invalida el modelo, sin usar el mapeo de estados de resolución.
Los fallos `InvalidInput`/`UnsupportedSchema` se traducen a `ENG-E-0009`
(contrato incompatible), con causa original y entidades vacías. Los diagnósticos
de dominio/extensiones se conservan íntegros y en su orden. Es validación del
contrato, convenciones, límites y topología; no certifica estabilidad ni
convergencia y no resuelve el sistema. Se prueba un modelo sin apoyos que valida.

Los errores de transporte conservan el comportamiento de MAM-862: exceso del
tope inclusivo de 8 MiB, UTF-8 inválido, rutas inexistentes o lectura fallida dan
2, stdout vacío y el mismo sobre cli-run/1 sanitizado en stderr que en #197;
las pruebas comparan sus bytes con la vía run. Los errores de uso también dan 2.
La cancelación cooperativa conserva 5 y la E/S de salida fallida, 70.
No se implementa el alias opcional `run --dry-run`.

`validate --catalog … --workflow …` conserva exactamente el despacho de la base,
también con `--input`. El método engine.validate dentro de un workflow conserva
su formato anterior. `run` y results/1 conservan sus bytes y manifiestos; no se
modifican `version().contracts`, los métodos ni la serialización de resolución.

Esquema nuevo: `crates/engine-api/schemas/validate-report-1.json`, Draft 2020-12,
`$id: urn:mamut:validate-report:1`. Registrado en `SCHEMAS` para
`engine_api::schema` y anunciado en `capabilities().schemas`, además del índice
del README de la API. La generación local con `scripts/generate-schemas.py --write`
publica sus bytes en `schemas/validate-report/1.json` y su entrada con SHA-256 en
`schemas/index.json`. Se extrae sin cambios funcionales el validador de
pruebas existente de results/1 a `tests/support/schema.rs` para reutilizarlo
desde los tests del binario, sin añadir otro validador ni dependencias.

## Evidencia y límites

Pruebas de binario: válido/0 y errores de esquema, JSON, límites internos
MAX_NODES y topología/2 con códigos ENG-*, stdout conforme al esquema y stderr
vacío. Comparación exacta stdin/ruta para éxito y errores; conservación exacta
de validación explícita de workflows. Las pruebas previas de run conservan
igualdad de bytes y manifiestos. Cobertura adicional de cancelación antes y
entre cada etapa, flags inválidos y mutaciones negativas del esquema en todos
sus niveles. El mismo validador sigue comprobando results/1.

No existe fixture que produzca aviso durante `validate_with`: el validador
actual genera solo errores y E1 recibe el aviso de diafragmas al ejecutar run.
No se añade un aviso artificial al binario. La rama `warning`/1 se prueba en el
clasificador con diagnóstico sintético, y la forma warning del esquema también
se comprueba. Esta es una limitación explícita de la cobertura del código 1
del binario para validación de modelo.

Rendimiento debug final con Rust **1.99.0**, x86_64-unknown-linux-gnu, mediana de
**11** repeticiones alternadas tras un
calentamiento de ambos caminos, extracción/serialización del input fuera del
cronómetro: **validate 5,440681 ms**, **run 151,456720 ms**, **27,84×**.
El test exige ratio ≥10 y permanece activo (sin `ignore`); no es una garantía
de tiempo absoluto en otras máquinas. Reproducción offline:

```sh
export PATH=/home/box/.cargo/bin:$PATH
CARGO_TARGET_DIR=/workspace/.cargo-target/sfc12 cargo test -p engine-api \
  --test validate_performance --offline -- --nocapture
```

Verificación final en este worktree con Rust/Cargo **1.99.0** (PATH de rustup),
`CARGO_TARGET_DIR=/workspace/.cargo-target/sfc12` y Cargo offline. El PATH inicial
seleccionaba Rust/Cargo 1.85.1 del sistema; se repitieron los controles con el
toolchain 1.99.0 fijado en el repositorio.

| Comprobación | Resultado |
| --- | --- |
| Tests dirigidos de CLI, fachada y results/1 | OK |
| `cargo fmt --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 764 passed / 0 failed / 0 ignored; CLI: 6 unitarios + 27 de integración |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --check` | OK |

Revisión humana clean-room externa: **PENDIENTE (J4)**.
