# 04 · Exponer buckling.linear en run, Python y TOML

- Estado: abierto. Puede empezar ya, en paralelo con el ticket 01.
- Relación: MAM-11, bitácora `provenance/ENG-ANL-006.md`. Ese archivo sigue con el revisor en PENDING. Este ticket no lo firma y no marca MAM-11 como terminado.
- Reglas comunes: [README.md](README.md).

## Qué ya existe

`engine_analysis::buckling::BucklingLinear` calcula los factores finitos
más bajos de `K + λ Kg = 0`. El kind de biblioteca es `buckling.linear`.
Los tests analíticos están en `crates/engine-analysis/tests/buckling.rs`.
La cita de página de Timoshenko y Gere está escrita como desconocida.
No se inventa.

## Qué construir

Hacer que un documento de análisis pueda pedir ese método y que `run`
devuelva los factores y las formas ya calculadas por la biblioteca.
Publicarlo en `capabilities().methods`, en la fachada Python y en el
convertidor TOML de `crates/engine-cli`.

No cambiar la ecuación ni el solver. No añadir un campo de pandeo con
otro significado. Los tests de `run` comparan contra los casos
analíticos que esa biblioteca ya fija.

Si se toca el esquema público, el cambio va con provenance y con el
mismo contrato de errores que los métodos ya publicados. No se copia
otro esquema de pandeo.

## Hecho cuando

- Un modelo pequeño del repositorio pasa por `run`, por Python y por TOML y devuelve los factores del test de biblioteca.
- La cita de página que siga desconocida permanece escrita como desconocida.
- Hay fragmento de changelog. MAM-11 y MAM-107 no se cierran.
- El revisor clean-room queda en PENDING.
