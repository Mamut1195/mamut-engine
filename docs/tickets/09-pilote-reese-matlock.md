# 09 · Ejemplo publicado de Reese y Matlock

- Estado: abierto, detenido hasta que la sesión tenga la fuente publicada.
- Relación: MAM-49. La bitácora `provenance/ENG-GEO-005.md` es solo la cuadratura del eje del pilote. No es el ejemplo publicado y no se marca MAM-49 como terminado.
- Nombres propios ya reservados en `docs/CLEAN_ROOM.md`: `py.matlock_soft_clay` y `py.reese_sand`.
- Reglas comunes: [README.md](README.md).

## Qué construir

Reproducir un ejemplo publicado de pilote, con figura, tabla y números
de esa publicación. La ficha cita autor, título, edición, figura, tabla
y página. Los esperados del test son los de esa tabla, copiados como
datos citados, no recalculados para que el programa cierre.

Si la sesión no tiene esa publicación a la vista, se detiene. No se
inventa la tabla y no se pide a Jonathan que la derive. La exención ya
escrita para unas curvas de API RP 2GEO, en `provenance/ENG-GEO-004.md`,
no se extiende sola a este ejemplo.

La entrada pública, cuando el ejemplo exista, usa
`engine_analysis::pile::solve` o el módulo que esa cita obligue, y se
publica en `run`, Python y TOML. Hasta tener la cita, no se publica un
método nuevo.

## Hecho cuando

- El test nombra la figura y la tabla y compara contra esos valores citados.
- Hay provenance y fragmento de changelog.
- MAM-49 no se cierra si la tabla publicada no está en el test.
- El revisor clean-room queda en PENDING.
