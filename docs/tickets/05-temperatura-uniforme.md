# 05 · Temperatura uniforme en la entrada pública

- Estado: abierto. Puede empezar ya, en paralelo con el ticket 01.
- Relación: MAM-972. No se cierra con este ticket.
- Reglas comunes: [README.md](README.md).

## Qué ya existe

La cercha tiene la forma cerrada de temperatura uniforme en
`crates/engine-elements-truss`. El test
`crates/engine-elements-truss/tests/temperature.rs` fija el signo:
tracción positiva, y una barra impedida en ambos extremos queda
comprimida cuando el coeficiente y el incremento de temperatura son
positivos. La fuerza axial no depende de la longitud.

Esa carga no entra por `engine-input/1`, por Python ni por TOML.

## Qué construir

Un campo de carga de temperatura uniforme sobre la barra, con el
coeficiente por kelvin y el incremento de temperatura en kelvin.
Sin gradiente. El gradiente queda fuera de este ticket.

La recuperación en desplazamiento nulo de la barra impedida debe
coincidir con el test que ya existe. No se cambia esa fórmula.

El mismo campo, si el elemento elástico de pórtico aún no lo tiene,
se deja documentado como límite y no se inventa una fórmula nueva
en este ticket. Primero la cercha, que ya está derivada.

## Hecho cuando

- Un modelo de cercha impedida, leído por `run` y por el convertidor TOML, recupera la axial del test existente.
- Python acepta el mismo campo.
- Hay provenance y fragmento de changelog.
- MAM-972 y MAM-107 no se cierran. El revisor clean-room queda en PENDING.
