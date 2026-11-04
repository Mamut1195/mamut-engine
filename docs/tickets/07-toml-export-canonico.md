# 07 · Export TOML byte a byte

- Estado: abierto. Puede empezar ya. No depende del ticket 06, pero no debe borrar el trabajo de nombres si ese ticket ya entró.
- Decisión cerrada: U9, en `docs/V1.md`, sección 13. No se rediscute.
- Reglas comunes: [README.md](README.md).

## Qué construir

Dos exports del mismo modelo producen los mismos bytes.

- UTF-8.
- Saltos LF.
- Un solo salto al final.
- Sangría de dos espacios.
- Claves en el orden de la especificación `docs/formats/mamut-toml-1.md`.
- No se borra un valor que la persona escribió.

El export actual ya es repetible dentro de una misma corrida. Este
ticket fija esa forma canónica y la cubre con un test que compara
bytes, no solo el valor parseado.

## Hecho cuando

- El test escribe dos veces el mismo modelo y los bytes coinciden.
- Un valor escrito por la persona sigue presente.
- Hay fragmento de changelog. El revisor clean-room queda en PENDING.
