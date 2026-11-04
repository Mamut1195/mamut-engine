# 08 · Línea y columna en los errores de TOML

- Estado: abierto. Puede empezar ya.
- Decisión cerrada: U11, en `docs/V1.md`, sección 13. U7 no se reabre: un id de más de 256 bytes ya se rechaza.
- Reglas comunes: [README.md](README.md).

## Qué construir

Cuando el convertidor informe un error de documento:

- La raíz cuenta como nivel 1. La profundidad 64 ya cuenta así y se mantiene.
- Si el error tiene posición, la línea y la columna empiezan en 1.
- El diagnóstico existente no cambia de código. Gana la posición.
- Un error que de verdad no tiene posición no inventa una línea.

## Hecho cuando

- Un TOML inválido de prueba produce línea y columna empezando en 1.
- Hay fragmento de changelog. El revisor clean-room queda en PENDING.
