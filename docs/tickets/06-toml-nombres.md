# 06 · Nombres opcionales e idmap/1

- Estado: abierto. Puede empezar ya.
- Decisión cerrada: U5, en `docs/V1.md`, sección 13. No se rediscute.
- Reglas comunes: [README.md](README.md).

## Qué construir

En el convertidor TOML de `crates/engine-cli`:

- `name` es opcional.
- El id entero es obligatorio. El convertidor no inventa ids.
- El mismo nombre puede repetirse entre nudo, barra y resorte.
- El nombre no entra al JSON del motor.
- Al exportar, los nombres vuelven solo si el documento va acompañado
  de `idmap/1`.

U10 ya está hecho: el apoyo se escribe `restrained` y `type` se rechaza.
No volver a abrirlo.

## Hecho cuando

- Un archivo con ids y sin nombres convierte igual que hoy.
- Un archivo con nombres repetidos entre tipos distintos convierte, y el JSON del motor no contiene esos nombres.
- Sin `idmap/1`, el export no reintroduce nombres.
- Con `idmap/1`, el export devuelve los nombres escritos.
- Hay tests en `engine-cli` y fragmento de changelog. El revisor clean-room queda en PENDING.
