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

El commit de este preregistro es `5696f82a806b05ab9745e2557dfac2d2b0e04cd4`, fecha real 2026-11-10 04:34:24 +0000 (2026-11-09 23:34:24 ECT). Se empujó a la rama antes de ejecutar el modelo. No se asignó fecha de autor ni de committer a mano.

## Corrida del modelo nuevo

`cargo test --offline --locked -p engine-api --test val037_val038` pasó después de ese empuje. Tres ensayos, cero fallos. `Δ1` cerrada `0.001 m`. Referencia `50/47 = 1.0638297872340425`.

| Piso | Δ1 del motor (m) | B2 del motor | |B2 − 50/47| |
| --- | ---: | ---: | ---: |
| 1 | 0.0010000004348738709 | 1.0638298070043912 | 1.98e-8 |
| 2 | 0.0010000010678617261 | 1.0638298357231972 | 4.85e-8 |
| 3 | 0.0010000012838284962 | 1.0638298455085473 | 5.83e-8 |

Los tres quedan dentro de `1e-6`.

Variantes ejecutadas o evaluadas con la forma cerrada. El hueco es la distancia a `50/47`. Todas superan `1e-6`, y el motor nominal también queda fuera de cada variante:

| Variante | Valor | Hueco |
| --- | ---: | ---: |
| 6/5→1, gravedad por 5/6 | 1.0526315950709941 | 1.12e-2 |
| Kg×0.8, gravedad por 0.8 | 1.0504201834796654 | 1.34e-2 |
| Kg×1.1, gravedad por 1.1 | 1.0706638335963594 | 6.83e-3 |
| sin término geométrico | 1 | 6.38e-2 |
| signo de P cambiado | 0.9433962109448136 | 1.20e-1 |
| altura 2h en la forma cerrada | 1.3157894736842106 | 2.52e-1 |

`Kg` entra lineal en el axil, así que escalar la gravedad escala `Kg`. El factor 5/6 convierte el 6/5 del corrimiento en 1.

## Comandos de esta entrega

- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --offline --locked --workspace --all-targets -- -D warnings`: exit 0
- `cargo test --offline --locked --workspace`: exit 0
- `RUSTDOCFLAGS=-D warnings cargo doc --offline --locked --no-deps --workspace`: exit 0
- `cargo check --offline --locked --workspace --target wasm32-unknown-unknown`: exit 0
- `cargo build --offline --locked --release --target wasm32-unknown-unknown -p engine-wasm`: exit 0

LaTeX no se compiló: `lualatex` y `latexmk` no están instalados y no se descargó una distribución. `docs/contracts/error-codes-1.json` no se editó y sigue con 32 entradas.

## VAL-038

Sigue la tolerancia relativa `1e-3` ya escrita en ENG-ANL-006. No se reajustó. `buckling.linear` es Experimental. `pdelta`, fachada `pDelta`, está publicado como Stable. Este ticket no cambia ese registro.

## Comandos de la primera entrega

Siguen en el historial del commit `0f60da3a2f6f9a5cc3defdebca764b1762e392b4`: fmt, clippy, test, doc y wasm32 en verde. LaTeX no se compiló: `lualatex` y `latexmk` no están instalados y no se descargó una distribución. `docs/contracts/error-codes-1.json` no se editó y sigue con 32 entradas.
