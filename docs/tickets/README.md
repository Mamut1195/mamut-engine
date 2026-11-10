# Tickets de continuación · 2026-11-04

Estos archivos son el trabajo que falta para seguir el motor. No cierran
tickets viejos y no aprueban presupuestos. Una sesión ejecutora toma un
archivo, en el orden de abajo, y no vuelve a preguntar el alcance.

El índice histórico de pull requests está en
[TICKETS_PRS.md](../TICKETS_PRS.md). Ese corte no se reescribe aquí.

## Reglas de toda sesión ejecutora

- Leer `docs/CLEAN_ROOM.md` y `docs/METHODS_GUIDE.md` antes de escribir código.
- La ficha de `docs/plantillas/FICHA_METODO.md` se completa antes del código
  cuando el ticket añade una ley, una sección o un elemento nuevo.
- Citar libro o artículo con autor, título, edición, sección y ecuación.
  Si esa página no está en la sesión, escribir «cita desconocida» y no
  inventarla. No copiar listados de libros.
- Los esperados de los tests salen de una derivación propia o de un caso
  analítico ya escrito en este repositorio. Llevan `source` y la cita.
- Fragmento nuevo en `changelog.d/`. No editar `CHANGELOG.md`. No cerrar
  MAM-107 y no aprobar presupuestos de tiempo o memoria.
- Bitácora `provenance/ENG-*.md` con la declaración de fuentes y el revisor
  clean-room en PENDING. Una sesión de agente no firma esa línea.
- Cargo offline y locked. Sin dependencias nuevas.
- Prohibido abrir o traducir código de OpenSees, Frame3DD, Stabileo o derivados.
- Prohibido copiar a este repositorio scripts, informes o salidas de un
  oráculo, y prohibido calibrar constantes para igualarlos.
- Si un resultado difiere de otro programa, el comentario solo dice
  «difiere en el caso X», sin valores. La corrección sale de la derivación.
- Los nombres de `kind` no usan la lista prohibida de `docs/CLEAN_ROOM.md`.

La sesión que el 2026-11-04 vio salidas numéricas de un programa externo
no implementa los tickets 01, 02 ni 03.

## Orden

| Orden | Archivo | Puede empezar | Depende de |
| --- | --- | --- | --- |
| 1 | [01-hormigon-kent-park.md](01-hormigon-kent-park.md) | Sesión limpia | Ninguno. Issue 4. |
| 2 | [02-seccion-fibras.md](02-seccion-fibras.md) | Después del 01 | Issue 5. |
| 3 | [03-barra-integrada.md](03-barra-integrada.md) | Después del 02 | Issue 6. |
| 4 | [04-pandeo-en-run.md](04-pandeo-en-run.md) | Ya, en paralelo | Biblioteca ya existente. |
| 5 | [05-temperatura-uniforme.md](05-temperatura-uniforme.md) | Ya, en paralelo | Forma cerrada ya probada en la cercha. |
| 6 | [06-toml-nombres.md](06-toml-nombres.md) | Ya, en paralelo | Decisión U5 ya escrita en `docs/V1.md`. |
| 7 | [07-toml-export-canonico.md](07-toml-export-canonico.md) | Ya, en paralelo | Decisión U9 ya escrita en `docs/V1.md`. |
| 8 | [08-toml-linea-columna.md](08-toml-linea-columna.md) | Ya, en paralelo | Decisión U11 ya escrita en `docs/V1.md`. |
| 9 | [09-pilote-reese-matlock.md](09-pilote-reese-matlock.md) | Solo con la fuente publicada en la sesión | MAM-49. |
| — | [10-no-empezar.md](10-no-empezar.md) | No | Presupuestos, release y licencia comercial. |
| 11 | [11-solver-disperso.md](11-solver-disperso.md) | El diseño, ya. La implementación es la opción A | MAM-67, ENG-LA-008. |
| 14 | [14-generador-python-v1.md](14-generador-python-v1.md) | Ya. Jonathan confirmó la opción C el 2026-11-10. | MAM-925. No depende de una ley nueva. |
