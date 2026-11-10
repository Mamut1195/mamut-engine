# Capacidades de main en la release 0.2.0

Fecha de esta lectura: 2026-11-09 (America/Guayaquil, ECT, UTC-5).

Este texto dice lo que el árbol de esta release hace, leído en el código y
en los documentos actuales. El inventario del 2026-11-04 sigue en
[V1.md](V1.md) y no se reescribe. `engine_api::version()` publica
`env!("CARGO_PKG_VERSION")` del crate `engine-api`. En esta release ese
número es `0.2.0`. Los contratos que acompaña son `engine-input/1` y
`results/1`.

## Qué hace `run`

`crates/engine-api/src/methods.rs` registra nueve fábricas:
`linearStatic`, `combinations`, `pDelta`, `modal`, `responseSpectrum`,
`nonlinearStatic`, `soilProfile`, `consolidation1d` y `buckling.linear`.

`capabilities()` marca con `engine: true` y `run: true` a
`linearStatic`, `nonlinearStatic`, `soilProfile`, `consolidation1d`,
`pDelta`, `modal`, `responseSpectrum` y `buckling.linear`. `combinations`
va aparte, con `run: true`: superpone casos ya resueltos y no es un
análisis nuevo del modelo. El README de `engine-api` publica
`linearStatic`, `modal`, `responseSpectrum` y `combinations` como
`Validated`, `pDelta` como `Stable`, y `nonlinearStatic`, `soilProfile`,
`consolidation1d` y `buckling.linear` como `Experimental`. Los
experimentales exigen `options.allow_experimental = true`.

La fábrica de barras de `run` acepta `frame2d.euler_elastic`,
`frame3d.euler_elastic`, `frame2d.winkler`, `frame3d.winkler`,
`truss2d.linear` y `truss3d.linear`. Otro nombre es `ENG-E-0013`.
`capabilities().elements` añade `spring.linear` y `spring.nonlinear`.
`spring.nonlinear` exige el método `nonlinearStatic` y una ley, sin `k`.

`capabilities().laws` nombra `elastic`, `steel.bilinear_kinematic`,
`multilinear_elastic`, `elastic.cubic` y `concrete.kent_park`. El esquema
`schemas/engine-input/1.json` enumera solo las cuatro primeras. Su texto
dice que las leyes no lineales se conectan solo a `spring.nonlinear`.
`concrete.kent_park` no aparece en ese esquema.

Los límites de `crates/engine-api/src/limits.rs` siguen siendo 8 388 608
bytes, 256 nudos, 1 024 barras, 64 casos, 16 384 cargas, 128
combinaciones, profundidad 64 y 16 384 entradas por arreglo.

El camino de `run` construye el sistema con `ReferenceBackend`. El README
de `engine-linalg` describe ese backend como eliminación densa: trabajo
cúbico en el orden y memoria cuadrática. La feature `faer` solo reserva
un stub. El solver disperso directo sigue pendiente en ENG-LA-008.
`guyan_condense` existe para grados libres sin masa en modal y espectro.

## Los seis puntos pedidos, contra este árbol

| Punto pedido | Qué se ve |
| --- | --- |
| Las fibras no están en la entrada | Coincide. `frame2d.fiber_stations` y `rectangle.fiber_bending` están en la biblioteca y en sus fichas. La fábrica de `run` no los nombra. La ficha dice que no hay fábrica JSON. |
| Lo no lineal es solo en resortes | Coincide en la entrada y en `nonlinearStatic`. Ese método arma el Newton solo con `spring.nonlinear`. No coincide como frase sobre toda la biblioteca: ahí también están la barra de fibras, la sección de fibras y `concrete.kent_park`, sin kind en `engine-input/1`. |
| No hay pushover | Coincide. No hay un método con ese nombre en el registro ni en el esquema. |
| No hay dinámica | No coincide si se lee como ausencia de todo análisis dinámico. Sí hay `modal` y `responseSpectrum`. No hay integración en el tiempo. |
| No hay láminas | Coincide. No hay un kind de lámina en la fábrica ni en `capabilities().elements`. |
| La temperatura es solo en cercha | Coincide. El esquema dice que solo `truss2d.linear` y `truss3d.linear` aceptan `temperature_loads`. El crate de pórtico no tiene ese registro. |
| El solver es denso | Coincide para el backend que usa `run`: `ReferenceBackend`. Hay almacenamiento disperso y condensación de Guyan. No hay un solver disperso directo activo. |

## Qué falta

- Fibras, Kent–Park y el pilote de `engine_analysis::pile` como análisis de `run`.
- No lineal de material en barras de la entrada. El Newton publicado es de resortes.
- Pushover, historia en el tiempo, láminas y elementos finitos de continuo.
- Grandes desplazamientos. P-Delta sigue elástico, con el axil del caso de gravedad.
- Temperatura en el pórtico, gradiente térmico y una segunda temperatura.
- Solver disperso directo y un backend `faer` funcional.
- Firma J4 de `provenance/ENG-ANL-006.md` y la página que el README de `engine-api` deja desconocida. Mientras eso siga así, ese README mantiene `buckling.linear` en Experimental.
- Texto de la licencia comercial (ENG-GOV-004).
