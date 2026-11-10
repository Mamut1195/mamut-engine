# Clean-room log · ENG-VAL-MAM-846

- Ticket / PR: MAM-846 · ENG-VAL-006. Rama `j2/mam-846-estructura-informe`. El número del pull request queda DESCONOCIDO al redactar esta bitácora: se abre en borrador después del push. No se marca listo y no se fusiona.
- Author(s): agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO. Fecha: 2026-11-10 01:30 (America/Guayaquil, ECT, UTC-5).
- Role: implementer. Solo documentación del informe. Sin código de crates.
- Contract implemented: ninguno de producción. El informe sigue siendo ENG-VAL-006.
- Permitted sources used: ticket MAM-846, aprobado por Jonathan vía J0 el 2026-10-20; `docs/validation-report/`; `docs/V1.md`; `capabilities()` y el nombre `reference.dense` en el código ya versionado; `provenance/ENG-VAL-MAM-845.md`; `provenance/TEMPLATE.md`. No entró ningún libro ni PDF. ASME V&V 10 no se reabrió: sección y página siguen en cita desconocida.
- Own derivations: ninguna ecuación nueva. La tabla semáforo lee tolerancias y series ya versionadas. No recalcula una tolerancia.
- Third-party code copied: none.
- Validation: no hubo campaña de oráculos. La comprobación numérica está más abajo. `herramientas/gen_casos.py` no se ejecutó. Los `\input` de VAL-037 y VAL-038 siguen en `secciones/casos.tex`.
- Comparison with oracle (sibling repo, informative): no.
- Session: no consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK.
- Declaration: «No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK ni derivados para este ticket.»
- Clean-room reviewer: PENDING.

## Base

La rama sale de main `18928b1b60516ea90bba56d9f0ab39c3185790f6`. El pull request 26 de la release sigue en borrador y no está fusionado. No hubo merge de main. `limitaciones.tex` se reescribió en esta rama según el código de ese commit: `buckling.linear` es un método Experimental de `run`, no una ausencia.

## Notas bajas

- De #24, VAL-035: `tolerancia_absoluta` vuelve a 0. Estaba vacía. Un preregistro no pierde un valor. La relativa sigue vacía: las cotas de 3 por ciento y 5 por ciento no entran en las columnas numéricas.
- De #24, cita: VAL-033, VAL-034, VAL-036 y VAL-041 ganan el punto que faltaba antes de «El plan».
- De #24, `metodologia.tex`: VAL-037 queda marcada «(consolidada)», dentro de las 34 del marco.
- De #24, bitácora: las filas de preregistro cambiaron el texto pero no los números. Quedó en `provenance/ENG-VAL-MAM-845.md` y se repite aquí. El único ajuste numérico de esas filas es devolver el 0 de VAL-035. No es una tolerancia nueva.
- De #25: el presupuesto de VAL-037 ya no dice que el acortamiento \(7{,}5\cdot 10^{-9}\) entre en \(B_2\). En `pDelta` no actúa sobre \(B_2\) y no entra en la suma. La cota \(10^{-6}\) no cambia. Las magnitudes de esa página conservan el mismo multiconjunto.

## Comprobación numérica, celda a celda

Contra main `18928b1b60516ea90bba56d9f0ab39c3185790f6`, `preregistro/tolerancias.csv` tiene 42 filas y 16 columnas. Hay 5 celdas distintas:

| Caso | Columna | Antes | Después |
| --- | --- | --- | --- |
| VAL-033 | cita | «publicado El plan» | «publicado. El plan» |
| VAL-034 | cita | «publicado El plan» | «publicado. El plan» |
| VAL-035 | tolerancia_absoluta | vacía | 0 |
| VAL-036 | cita | «cerrada El plan» | «cerrada. El plan» |
| VAL-041 | cita | «libro El plan» | «libro. El plan» |

La única celda numérica que cambia es la absoluta de VAL-035, de vacía a 0. Las otras cuatro celdas solo ganan un punto. `error_maximo` y `n_magnitudes` siguen en DESCONOCIDO en las 41 filas: el semáforo no reescribe el CSV.

Los CSV `datos/*/referencia.csv` y `datos/*/engine.csv` son idénticos a main. No se abrió ningún `opensees.csv`. `gen_casos.py` no se corrió.

En `casos/deriv/VAL-037.tex` el multiconjunto de magnitudes (`10^{n}`, `10^{-n}`, `7{,}5`, decimales) es el mismo. Hay dos apariciones más del subíndice \(B_2\): es el nombre del factor, no una tolerancia ni un valor esperado.

`limitaciones.tex` conserva 256 nudos, 1024 barras y 16\,384 registros. Sale la mención vieja al PR \#168 y a MAM-45, porque el código ya tiene las curvas y el pilote no es un método de `run`. No se añadió ninguna tolerancia.

La tabla semáforo, leída con Lua 5.4.6 sobre esos CSV y sin compilar LaTeX, da pendiente en 39 casos. VAL-037: PASA, error máximo 5.478e-08, \(N=3\), absoluta ya escrita. VAL-038: PASA, error máximo 5.121e-04, \(N=4\), \(A=0\), \(R=1\mathrm{e}{-3}\). Esos dos errores salen de la serie del motor contra la referencia analítica ya versionada. Si la relativa está vacía y la absoluta es un número, la comparación usa solo la absoluta y no escribe una relativa en el CSV.

## Comandos

| Comando | Resultado |
| --- | --- |
| `cargo test --offline --locked --workspace` | exit 0 |
| Lua 5.4.6, `mamut.semaforo` sobre los CSV versionados | exit 0. No es la compilación del informe. |
| `lualatex` y `latexmk` | DESCONOCIDO: no están en el PATH. El informe no se compiló. |
| `bash clean-room/check.sh origin/main` | exit 0. Clean-room: OK |
| `bash scripts/changelog-check.sh origin/main` | exit 0. No hay cambios en crates ni en bindings; el fragmento igual está en `changelog.d` |

## Qué no se abrió

No se abrió el repositorio hermano, ni un CSV de contraste con datos, ni la bitácora de humo del benchmark, ni el directorio restringido de auditoría, ni un registro de otra sesión. No se copió un libro. No hay rutas de esta máquina en los archivos.
