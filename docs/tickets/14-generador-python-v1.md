# 14 · Generador Python de dataclasses v1

- Estado: abierto. Jonathan confirmó la opción C el 2026-11-10 con estas palabras: «Confirmo la opción C para MAM-925».
- Ticket: MAM-925.
- La opción B no se usa. No se instala ni se evalúa datamodel-code-generator.
- Reglas comunes: [README.md](README.md).

## Qué construir

Un generador propio, `bindings/python/tools/gen_v1.py`, que usa solo la biblioteca estándar. Lee `schemas/engine-input/1.json` y `schemas/results/1.json` y escribe el módulo `mamut_engine.v1`. No hay post-proceso manual.

El módulo declara:

- dataclasses con `frozen`, `slots` y `kw_only`;
- `StrEnum` cerrados, uno por cada `enum` de cadenas del esquema, sin miembros de más;
- la unión `Analysis`, discriminada por `method`.

Dos ejecuciones del generador escriben los mismos bytes. El módulo no usa metaclases ni dependencias de runtime. En Python 3.10 la biblioteca estándar todavía no tiene `enum.StrEnum` (llega en 3.11). El módulo declara una clase `StrEnum` sobre `str` y `Enum` solo en ese intérprete, sin metaclase propia. El piso `requires-python` sigue en `>=3.10`; cambiarlo exigiría `pyproject.toml`, y este ticket no lo toca.

## Denominador

Una definición contada es una clave de `$defs`, o el documento raíz. El raíz cuenta porque es un objeto del esquema y no vive dentro de `$defs`. Una rama anónima de `oneOf`, un objeto anónimo de una propiedad y un `enum` escrito dentro de una propiedad no suman: son la expansión de la definición que los contiene.

Con ese denominador, hoy:

| Esquema | Claves de `$defs` | Documento raíz | Total |
| --- | --- | --- | --- |
| engine-input/1 | 37 | 1 | 38 |
| results/1 | 29 | 1 | 30 |

El generador cubre las 38 y las 30. Si un esquema gana una clave o el raíz deja de poder expresarse, el generador falla o el test de bytes falla.

## Unidades

No se supone un sistema de unidades en un campo ni en un docstring. Las unidades tipadas son MAM-1046 y no entran en este ticket. Si `engine-input/1` escribe `units` con la constante `"SI"`, el campo generado es exactamente esa constante y no añade otros valores.

Los límites numéricos, los patrones y el `allOf` condicional de la coordenada D2 siguen en el esquema. El módulo no los reimplementa.

## Hecho cuando

- El archivo generado coincide, byte a byte, con una ejecución nueva. Otro test comprueba que dos ejecuciones son idénticas.
- Si el esquema cambia y el módulo no se vuelve a generar, el test falla.
- `mypy --strict` y pyright en modo strict no informan errores en el generador ni en el módulo generado.
- La wheel se construye, se instala en un venv limpio y los tests corren contra ella. La wheel incluye `py.typed` y los `.pyi`. `test_typing` pasa.
- Hay fragmento `changelog.d/` y bitácora `provenance/ENG-PY-*.md`. El revisor clean-room queda en PENDING.
- No se editan los esquemas, `CHANGELOG.md` ni los manifiestos de empaquetado.
