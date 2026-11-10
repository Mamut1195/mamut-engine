# Bitácora clean-room · ENG-PY-009

- Ticket / PR: MAM-925 · generador Python de dataclasses v1. Rama `j2/mam-925-generador-gen-v1`.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-10 (América/Guayaquil, ECT, UTC−5).
- Session: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO. No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni salidas de esos programas.
- Rol: implementador. Generador y tipos de transporte. Sin cálculo numérico.
- Contrato implementado: ninguno nuevo. Se leen los esquemas ya publicados `engine-input/1` y `results/1`.
- Fuentes permitidas: `schemas/engine-input/1.json`, `schemas/results/1.json`, `docs/tickets/README.md`, `docs/tickets/01-hormigon-kent-park.md` a `08-toml-linea-columna.md`, `docs/CLEAN_ROOM.md`, `docs/METHODS_GUIDE.md`, `provenance/TEMPLATE.md`, `changelog.d/README.md`, `bindings/python/pyproject.toml` (solo lectura) y el empaquetado ya existente del módulo `mamut_engine`. No entró ningún libro ni PDF. No se leyó datamodel-code-generator ni otro generador.
- Derivaciones propias: el generador `bindings/python/tools/gen_v1.py` y el módulo que escribe.
- Código de terceros copiado: ninguno.
- Validación: estructural. El módulo generado se compara byte a byte con una ejecución nueva. No hay oráculo numérico.
- Comparación con oráculo (repositorio hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni salidas de esos programas, para este ticket."
- Revisor clean-room: PENDING.

## Alcance

Jonathan confirmó la opción C el 2026-11-10: «Confirmo la opción C para MAM-925». La opción B queda fuera.

El generador usa solo la biblioteca estándar. Escribe dataclasses `frozen`, `slots` y `kw_only`, `StrEnum` cerrados y la unión `Analysis` discriminada por `method`. Todo queda en `mamut_engine.v1`. No hay metaclases, post-proceso manual ni dependencia de runtime.

Denominador: cada clave de `$defs`, más el documento raíz. Las ramas anónimas de un `oneOf` y los objetos o enums escritos dentro de una propiedad no suman: son la expansión de la definición que los contiene.

| Esquema | Claves de `$defs` | Documento raíz | Total cubierto |
| --- | --- | --- | --- |
| engine-input/1 | 37 | 1 | 38 |
| results/1 | 29 | 1 | 30 |

Unidades: no se supone un sistema en campos ni en docstrings. Las unidades tipadas son MAM-1046 y no entran aquí. La constante `units` del esquema engine-input/1 se copia tal cual, sin añadir valores.

En Python 3.10 la biblioteca estándar todavía no tiene `enum.StrEnum`. El módulo declara una clase `StrEnum` sobre `str` y `Enum` solo en ese intérprete. El piso `requires-python` sigue en `>=3.10`. Cambiarlo exigiría `pyproject.toml`, y este PR no lo toca.

Los límites numéricos, los patrones y el `allOf` condicional de la coordenada quedan en el esquema. El módulo no los reimplementa. Un campo opcional ausente y un null explícito se representan los dos con `None`. Distinguirlos es MAM-1062 (centinela UNSET).

## Verificación

| Comando | Código de salida |
| --- | --- |
| `cargo test --offline --locked --workspace` | 0 |
| `mypy --strict --follow-imports=silent --python-version 3.12` sobre el generador y `mamut_engine/v1.py` | 0 |
| `mypy --strict --follow-imports=silent --python-version 3.10` sobre los mismos archivos | 0 |
| pyright en modo strict, Python 3.12, sobre los mismos archivos | 0 |
| pyright en modo strict, Python 3.10, sobre los mismos archivos | 0 |
| `python3 -m unittest discover -s bindings/python/tests -p test_gen_v1.py` sin wheel instalada | 1. Nueve pruebas pasan. La décima importa el paquete instalado y no lo encuentra. |
| `bash scripts/build-python-wheel.sh` | 1. La wheel se construye, se instala en un venv limpio y las pruebas de `test_gen_v1` y `test_typing` pasan contra ella. Falla `test_parity_matrix`: pide `MAMUT_PARITY_REFERENCES`, y el script no exporta esa variable. El fallo es anterior a este cambio. |

`mypy --strict` sobre el paquete completo sigue informando errores en módulos que este PR no toca. Por eso la corrida de arriba limita el seguimiento de importaciones a silencio y nombra solo el generador y el módulo generado.

## Confirmación en main, sin esta rama

Commit: `6ace1bca93c3e566b1f9af8c56644ed521e627a8`. Árbol desprendido en ese commit de main. `git branch --show-current` no devolvió rama. `git rev-parse HEAD` devolvió ese SHA. No estaba la rama de este PR.

`MAMUT_PARITY_REFERENCES` no estaba definida. `CARGO_TARGET_DIR` apuntaba a un directorio de compilación fuera del repositorio, como exige el script. `MATURIN` era maturin 1.15.0. El directorio de salida era nuevo.

Comando, desde la raíz de ese árbol:

```
unset MAMUT_PARITY_REFERENCES
export CARGO_NET_OFFLINE=true
bash scripts/build-python-wheel.sh <directorio-nuevo>
```

Código de salida: 1.

Extracto, sin la ruta del archivo de la prueba:

```
test_same_matrix_matches_closed_forms_rust_bytes_and_typed_errors ... FAIL
AssertionError: None is not true : Set MAMUT_PARITY_REFERENCES using the Rust parity_matrix test
FAILED (failures=1, skipped=7)
```

98 pruebas, 1 fallo, 7 omitidas. Este PR no modifica `scripts/build-python-wheel.sh`.
