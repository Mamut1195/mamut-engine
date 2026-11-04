# 03 · Barra integrada en varias estaciones

- Estado: abierto. Empieza cuando el ticket 02 está en `main` y su test pasa.
- Issue: https://github.com/Mamut1195/mamut-engine/issues/6
- Sesión: limpia, distinta de las que implementaron 01 y 02, y distinta de la del 2026-11-04.
- Reglas comunes: [README.md](README.md).

## Qué construir

Una barra de pórtico cuya respuesta de sección es la sección de fibras
del ticket 02, integrada en varias estaciones a lo largo de la luz, para
un análisis estático no lineal.

El número de estaciones y los pesos de integración se escriben en la
ficha. No se mueven para igualar un programa externo. El nombre del
elemento es propio del motor.

El patrón de elemento existente está en `crates/engine-elements-frame`.
Esta barra no sustituye a `frame2d.euler_elastic`.

## Punto de control

Una barra simple, voladizo o biapoyada. En la estación de control, el
axil, el corte y el momento coinciden con el equilibrio de la sección
ya probada en el ticket 02. El esperado sale de esa sección, no de otro programa.

La comparación de un pórtico contra un banco externo, si se hace, queda
fuera de este repositorio. No entran scripts, salidas ni números.

## Hecho cuando

- El test del voladizo o de la viga biapoyada pasa contra la sección propia.
- Hay provenance y fragmento de changelog.
- El changelog no declara el método validado por comparación externa.
- El revisor clean-room queda en PENDING.
