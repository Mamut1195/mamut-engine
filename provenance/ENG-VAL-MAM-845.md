# Clean-room log · ENG-VAL-MAM-845

- Ticket / PR: MAM-845 · ENG-VAL-006. Rama `j2/mam-845-preregistro-tolerancias`. PR \#24.
- Author(s): agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO. Fecha: 2026-11-09 23:43 (America/Guayaquil).
- Role: implementer. Solo documentación: tabla, marco, integración en el informe y esta bitácora.
- Contract implemented: ninguno de producción. El informe sigue siendo ENG-VAL-006.
- Permitted sources used: ticket MAM-845; tolerancias ya escritas en el repositorio y citadas fila a fila en `docs/validation-report/preregistro/tolerancias.csv`. ASME V&V 10-2019 (R2025) se nombra como encuadre. Sección y página de la norma: cita desconocida. No se inventan apartados. El plan de las bandas preregistradas no está en el repositorio. Su aprobación por Jonathan es DESCONOCIDO.
- Own derivations: ninguna ecuación nueva y ninguna campaña. La métrica relativa, y la normalización por el máximo de la serie cuando la referencia es aproximadamente cero, se declaran como regla del plan. El umbral numérico de «aproximadamente cero» es DESCONOCIDO.
- Third-party code copied: none.
- Validation: no se ejecutó campaña de oráculos en este ticket. Treinta y cuatro filas consolidan una tolerancia ya escrita (VAL-001 a VAL-030, VAL-037, VAL-038, VAL-039 y VAL-040). Cinco filas son preregistro (VAL-033, VAL-034, VAL-035, VAL-036 y VAL-041). VAL-031 queda en DESCONOCIDO. VAL-032 queda pendiente de congelación (Q2). En revisión: 0. VAL-037 se fijó después de la revisión r1 de \#23.
- Comparison with oracle (sibling repo, informative): no.
- Session: no consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK.
- Declaration: «No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK ni derivados para este ticket.»
- Clean-room reviewer: PENDING.

## Commit de la tabla

- SHA: `c0109829dcc54af0c781c5287e64726275552aa7`.
- Fecha de committer: `2026-11-10T04:43:09+00:00`.
- Ese commit se empujó a `origin` antes del commit que integra el informe. La hora del push, según la actividad de la rama, es `2026-11-10T04:43:13Z` (23:43:13 ECT).

## Orden

El orden de ejecución de los casos consolidados es DESCONOCIDO. Git solo muestra la introducción de los archivos citados en la columna `introduccion_git`. Los CSV de datos del informe, de VAL-001 a VAL-030, seguían en el marcador de dos líneas. No se leyó ninguna serie numérica para llenar la tabla.

## Qué no se abrió

No se abrieron `results/` ni archivos de salida de un programa de contraste. Para VAL-032 la tabla cita solo el nombre del campo `tolerances_preregistered` y no transcribe sus valores numéricos. No hay código de producción en este cambio. `docs/contracts/error-codes-1.json` no se modifica.

## Ajuste tras el merge de MAM-850

`origin/main` tras MAM-850 es el squash `9aeebcc000ca42c0f94b5aa41507b1da1e378743`. Entró con un commit de merge, sin rebase. VAL-037 es consolidación. La cota ya escrita es `|B2_motor − 50/47| ≤ 1e-6`, en `docs/validation-report/casos/deriv/VAL-037.tex` y en `B2_TOLERANCE` de `crates/engine-api/tests/val037_val038.rs`. La ficha `docs/validation-report/casos/VAL-037.tex` repite la misma cota. Se fijó después de la revisión r1 de \#23. El preregistro está en `5696f82a806b05ab9745e2557dfac2d2b0e04cd4`, commit de la rama de ese PR, fecha de committer `2026-11-10T04:34:24+00:00`. Por el squash ese commit no es accesible desde main. Esta tabla no lo preregistra otra vez. La banda anterior queda retirada en la derivación; no se copia aquí. No se transcriben series de un tercer programa: la columna correspondiente de VAL-037 es de Q2, en MAM-1040.

La prueba también escribe `|Delta1 − 0.001| ≤ 1e-8`. No es la cota de aceptación nombrada en la derivación.

VAL-038 en main es `|\lambda − P_cr| ≤ 0 + 10^{-3}|P_cr|`, en `docs/validation-report/casos/VAL-038.tex` y en `EULER_RELATIVE` de la misma prueba. La fila anterior decía absoluta `1e-8`; no coincidía con esa ficha y se transcribe el valor de main. `provenance/ENG-ANL-006.md` sigue con la absoluta `1e-8` y la forma cuadrática a `1e-6`; la página de VAL-038 dice que no las reajusta.

`herramientas/gen_casos.py` ya conserva `\input{casos/VAL-037}` y `\input{casos/VAL-038}` al rehacer `secciones/casos.tex`. Esos `\input` están en el informe después del merge. `informe.tex` sigue incluyendo `preregistro/marco`. No hizo falta reinsertar ninguno.

Conteos: 34 consolidados, 5 preregistrados, 1 DESCONOCIDO, 1 pendiente de congelación (Q2) y 0 en revisión.

## Revisión r1 de \#24

VAL-032 no es consolidación. El README de `docs/validation-report/fixtures/val032/README.md`, sección Congelación, no declara un SHA de congelación: Q2 debe verificar antes de congelar. Una parte del caso es solo diagnóstico. La fila cita el campo `tolerances_preregistered` y no copia valores.

Las fichas VAL-001 a VAL-030 salen de `docs/validation-report/herramientas/gen_casos.py`. La frase «fijada a priori» pasó a la clase consolidada, con orden DESCONOCIDO, y remite a las otras clases del marco. Los números de \(A\) y \(R\) no se tocaron. Los `\input` de VAL-037 y VAL-038 siguen en `secciones/casos.tex`.

## Seguimiento MAM-846

Las filas de preregistro cambiaron el texto pero no los números. La cita de VAL-033, VAL-034, VAL-036 y VAL-041 gana el punto que faltaba antes de «El plan». VAL-035 recupera la absoluta 0: un preregistro no pierde un valor ya escrito. No es una tolerancia nueva. Ningún otro número de esas filas cambia.
