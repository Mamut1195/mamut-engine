# 02 · Sección de fibras y momento-curvatura

- Estado: abierto. Empieza cuando el ticket 01 está en `main` y su test pasa.
- Issue: https://github.com/Mamut1195/mamut-engine/issues/5
- Sesión: limpia, distinta de la que implementó el 01 y distinta de la del 2026-11-04.
- Reglas comunes: [README.md](README.md).

## Qué construir

Una sección rectangular discretizada en fibras. El hormigón es
`concrete.kent_park`. El acero es el que ya existe,
`steel.bilinear_kinematic`. No añadir otra ley de acero.

El análisis de la sección es momento-curvatura bajo axil constante.
El `kind` es un nombre propio del motor. La discretización, el número
de fibras y la posición del acero se escriben en la ficha antes del código.

## Punto de control

Elegir una curvatura y un axil. Integrar las tensiones de las fibras
con la cuadratura escrita en la ficha. El axil y el momento de esa suma
son los esperados del test. No se ajusta la malla para parecerse a otro programa.

## Hecho cuando

- El test compara axil y momento con esa suma.
- Hay provenance, fragmento de changelog y revisor clean-room en PENDING.
- El ticket 03 puede empezar.
