# 01 · Ley uniaxial concrete.kent_park

- Estado: abierto.
- Issue: https://github.com/Mamut1195/mamut-engine/issues/4
- Sesión: limpia. La sesión del 2026-11-04 no implementa este método.
- Reglas comunes: [README.md](README.md).

## Qué construir

Una ley uniaxial nueva, `concrete.kent_park`, en `crates/engine-laws`,
al lado de `steel.bilinear_kinematic`. No modificar esa ley de acero.

La curva es la publicada por Kent y Park: parábola hasta la deformación
de pico y rama posterior hasta la deformación última. La ficha cita el
libro o el artículo, la edición, la sección y el número de ecuación.

El signo de compresión es el que ya usa el motor. Se lee en las leyes
uniaxiales existentes y se escribe en la ficha antes del código.

## Punto de control

Elegir una deformación dentro de la parábola y otra en la rama posterior.
La tensión de cada una se calcula con la ecuación citada, a mano, y ese
valor es el esperado del test. El fixture lleva `source` y la cita.

Cubrir también un parámetro fuera de rango, un valor no finito y el
estado inicial. La tangente se deriva a historia fija y se comprueba en
los dos tramos y en el pico.

## Hecho cuando

- El test del repositorio compara contra esos dos puntos derivados.
- Hay `provenance/ENG-*.md` y fragmento `changelog.d/`.
- `capabilities().laws` incluye `concrete.kent_park`.
- El fragmento no cierra MAM-107 ni aprueba presupuestos.
- La bitácora deja el revisor clean-room en PENDING.
