# Bitácora clean-room · ENG-VAL-MAM-850

- Ticket / PR: MAM-850 · VAL-037 y VAL-038
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-09 (América/Guayaquil, ECT, UTC−5)
- Session: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO. No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni salidas de esos programas.
- Rol: especificador. Sin código de producción.
- Contrato implementado: ninguno. Páginas de validación y un ensayo de API.
- Fuentes permitidas: docs/CLEAN_ROOM.md; docs/METHODS_GUIDE.md; docs/tickets/README.md; provenance/TEMPLATE.md; provenance/ENG-ANL-003.md; provenance/ENG-ANL-006.md; provenance/ENG-ELM-002.md; la rigidez geométrica de la barra y las fachadas pdelta y buckling.linear que ya están en el repositorio. No entró ningún libro ni PDF. La clave aisc16 del informe documenta el Comentario C2. El Apéndice 8 sin R_M, su sección y su ecuación: cita desconocida. Timoshenko y Gere, página, sección, tabla y ecuación: cita desconocida.
- Derivaciones propias: docs/validation-report/casos/deriv/VAL-037.tex y el resumen de Euler en docs/validation-report/casos/deriv/VAL-038.tex, que apunta a ENG-ANL-006.
- Código de terceros copiado: ninguno
- Validación: analítica. VAL-038 registra los cuatro autovalores de Euler ya cubiertos por los ensayos existentes. VAL-037 acepta el operador de Hermite 50/47.
- Comparación con oráculo (repositorio hermano, informativa): no
- Declaración: "No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni salidas de esos programas, para este ticket."
- Revisor clean-room: PENDING

## Primer preregistro, retirado

La banda `|B2_motor − B2_piso| ≤ 0.35 (B2_piso − 1) + 0.005` se escribió en `396759950c446499102756793b8fa45409ce7d8f` antes de la primera corrida. La r1 de J4 la declaró demasiado ancha: sobreviven las variantes 6/5→1, Kg×0.8 y Kg×1.1. Además `Pe = H h / Δ1` usaba la Δ1 del propio motor, así que aquella columna no era una referencia analítica. Era una comparación interna. El hueco no venía de los giros de nudo: con `EA = 4e7 N` el acortamiento elástico del piso bajo era del orden de `6e-3 h`, y la r1 lo señaló como la causa principal.

Esa banda y esa serie quedan retiradas.

## Segundo preregistro

Se fijó después de conocer el resultado de la r1, y antes de ejecutar el modelo nuevo.

Modelo: tres pisos, un vano, `h = 4 m`, vano `6 m`, `E = 2e8 Pa`, `Ic = 0.02 m⁴`, `Iv = 3e5 m⁴`, `A = 1e5 m²` (`EA = 2e13 N`). Cargas solo en la cubierta: `H = 1500 N` y `ΣP = 3e5 N`. Con vigas rígidas y columnas axialmente rígidas,

`Δ1 = H h³ / (12 E ΣIc) = 0.001 m`,

`Pe = 6e6 N`, `r = 1/20` y `B2 = 1 / (1 − 6r/5) = 50/47`.

La forma `1/(1−r)` queda aparte: es la cuerda sin arqueo, no el criterio.

Tolerancia: `|B2_motor − 50/47| ≤ 1e-6`. Cubre redondeo, `G ≤ 2e-7` y acortamiento `7.5e-9`. La suma de esas cotas está por debajo de `2e-7`.

Huecos de las variantes, calculados con la aritmética de la forma cerrada y sin ejecutar el motor: 6/5→1 da `1.12e-2`; Kg×0.8 da `1.34e-2`; Kg×1.1 da `6.83e-3`; sin término geométrico da `6.38e-2`; el signo de P cambiado da `1.20e-1`; la altura `2h` en la forma cerrada da `2.52e-1`. Todas superan `1e-6`.

La serie del motor de este modelo no está en este commit.

## VAL-038

Sigue la tolerancia relativa `1e-3` ya escrita en ENG-ANL-006. No se reajustó. `buckling.linear` es Experimental. `pdelta`, fachada `pDelta`, está publicado como Stable. Este ticket no cambia ese registro.

## Comandos de la primera entrega

Siguen en el historial del commit `0f60da3a2f6f9a5cc3defdebca764b1762e392b4`: fmt, clippy, test, doc y wasm32 en verde. LaTeX no se compiló: `lualatex` y `latexmk` no están instalados y no se descargó una distribución. `docs/contracts/error-codes-1.json` no se editó y sigue con 32 entradas.
