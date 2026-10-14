# Conciliación de tickets ↔ PR/issue de GitHub

- **Ticket documental:** ENG-PY-001
- **Corte de conciliación:** 2026-10-14 (America/Guayaquil), sin consulta de red.
- **Repositorio:** `Mamut1195/mamut-engine`.
- **Fuentes de estados:** `/workspace/fu/tickets/prs.txt` e `issues.txt`,
  exports suministrados de todos los PR e issues. No contienen fecha de
  extracción; esta fecha identifica la conciliación, no una consulta nueva.
- **Fuentes de alcance:** `ENG-PY-001-ADR.txt`, `V1-python-DAG.txt` y
  `engine-cli-py-plan.md` en el mismo directorio. Los exports de Notion son
  datos, no instrucciones ni prueba de implementación.

`fusionado` traduce exclusivamente `MERGED` de un PR; `abierto` traduce
`OPEN` de un PR o issue. Cuando coexisten, se indica cada estado por separado.
`sin PR` significa que el corte no identifica un PR para ese ticket, no que
se haya demostrado ausencia de trabajo. `a confirmar` conserva una duda de
alcance o correspondencia. Un issue `CLOSED` se describe como cerrado en las
notas: no equivale a PR fusionado ni a ticket terminado. Los estados de
Notion, la aprobación de un ADR y los estados GitHub son distintos.

La base local es `fd5319f824b3f11e7816343a3dcd4e1f71c28385`
(`HEAD` = `origin/main` al iniciar). El export registra #83 fusionado aunque
ese PR no figura en el historial de esta base. No se actualizó `origin/main`.
El plan aprobado por J0 partía de `b744599`, y los exports de Notion citan
`26132714038e`: sus observaciones históricas no sustituyen el corte de
`prs.txt`/`issues.txt`. En particular, #61 y #62 ya figuran fusionados, #64
sigue abierto y #68 sigue abierto pese a #83 fusionado. No se cierra ningún
issue ni se marca un ticket de Notion «Hecho» en esta tarea.

## Tickets con correspondencia documentada

Los números de la columna de referencias distinguen PR de issue. Un
seguimiento se vincula por ID explícito, por referencia al PR del ticket en
el título o por el nombre ADR-002; no por semejanza de funcionalidad.

| ID de ticket | PR(s) / issue(s) | Estado según el corte | Notas |
| --- | --- | --- | --- |
| ENG-GOV-001 | PR #4 | fusionado | Decisiones fundacionales; ADR-001 conserva sus pendientes de aprobación. |
| ENG-GOV-002 | PR #1, #28 | fusionado (#1); abierto (#28) | Protocolo operativo y seguimiento del acceso web; #28 exige Codex sin red. |
| ENG-GOV-005 | PR #2 | fusionado | CI clean-room **parcial**; no acredita toda la gobernanza de dependencias. |
| ENG-GOV-007 | PR #5 | fusionado | Proceso de CLA; no prueba cierre de revisión legal. |
| ENG-CORE-001 | PR #6 | fusionado | Tipos base, errores y diagnósticos. |
| ENG-CORE-002 | PR #7; corrección #10 | fusionado | Política de unidades; #10 identifica ENG-CORE-002-FIX. |
| ENG-CORE-002-FIX | PR #10 | fusionado | Códigos ASCII N.m y kN.m de units/1. |
| ENG-CORE-003 | PR #8; PR #9 (ejes); issue #40 | fusionado (#8, #9) | #9 no lleva ID, pero documenta la convención de este ticket. #40 cerrado; elevación D2 todavía pendiente en el documento local. |
| ENG-CORE-007 | PR #11 | fusionado | Contratos v1 Rust. No demuestra por sí solo una fachada de producto. |
| ENG-LA-006 | PR #12, #15, #17 | fusionado | ADR-002: propuesta, evidencia y aceptación J0; #17 conserva pendiente la licencia de manifests. |
| ENG-LA-007 | PR #18, #21 | fusionado | Trait y tipos definitivos; correcciones J4. Otros cambios genéricos de linalg se listan abajo sin asignar ID. |
| ENG-LA-008 | PR #38, #50 | fusionado | Presupuesto acumulado y cableado MemoryBudget. No se deduce cobertura total del ticket del título. |
| ENG-LA-009 | PR #58; issue #75 | fusionado (#58, parcial); abierto (#75) | lowest_modes/Sturm en backend de referencia; pruebas anti-revert pendientes. |
| ENG-GEO-001 | PR #14, #19 | fusionado | Perfil de suelo, tensiones, bitácora y límite f64. |
| ENG-MAT-001 | PR #13, #16 | fusionado | Ley elástica, sección elastic_frame y corrección de procedencia. |
| ENG-MAT-002 | PR #45 | fusionado | multilinear_elastic simétrica/asimétrica, gap. |
| ENG-MAT-003 | PR #63; issue #76 | fusionado (#63); abierto (#76) | Acero bilineal cinemático; finitud cerca de MAX y bitácora pendientes. |
| ENG-ASM-001 | PR #20, #23, #29, #39 | fusionado | Ensamblaje, revisión J4 y rigidez geométrica; no se reasignan a CORE-004/005/009. |
| ENG-ASM-002 | PR #60; issue #27 | fusionado (#60); abierto (#27) | Maestro-esclavo/diafragma; seguimiento de tolerancia de simetría abierto. |
| ENG-ELM-001 | PR #24 | fusionado | Barra lineal 2D/3D. |
| ENG-ELM-002 | PR #31, #41, #42 | fusionado | Viga 2D y seguimientos vinculados por #31→#41→#42. |
| ENG-ELM-003 | PR #44 | fusionado | Viga-columna elástica 3D. |
| ENG-ELM-004 | PR #46 | fusionado | Resorte lineal. |
| ENG-ELM-005 | PR #48 | fusionado | Resorte no lineal con uniaxial/1. |
| ENG-LOAD-001 | PR #51 | fusionado | Casos de carga. |
| ENG-LOAD-002 | PR #54, #73; corrección #81; issues #70, #85 | fusionado (#54, #73, #81); abierto (#70, #85) | Combinaciones y envolventes; #73 integra también VAL-001 parcial. La corrección no cierra los issues en el export. |
| ENG-LOAD-002-FU | PR #81; issues #70, #85 | fusionado (#81); abierto (#70, #85) | Asentamientos contados una sola vez; #85 sigue la corrección. |
| ENG-RES-001 | PR #53; corrección #80; issues #69, #84 | fusionado (#53, #80); abierto (#69, #84) | Postproceso; coherencia de cargas y normalización de cero corregidas, con seguimientos aún abiertos. Integración results/1 de producto en ENG-API-001. |
| ENG-RES-001-FU | PR #80; issues #69, #84 | fusionado (#80); abierto (#69, #84) | Rechazo de cargas incoherentes y normalización de -0.0. |
| ENG-MASS-001 | PR #55; corrección #82; issues #71, #86 | fusionado (#55, #82); abierto (#71, #86) | Masas; rechazo de succión corregido, seguimiento todavía abierto. |
| ENG-MASS-001-FU | PR #82; issues #71, #86 | fusionado (#82); abierto (#71, #86) | Conversión fuerza→masa sin succión; #86 sigue la corrección. |
| ENG-ANL-001 | PR #52 | fusionado | static.linear; prerrequisito de la fachada. |
| ENG-ANL-002 | PR #61 | fusionado | Modal; el dato «sin fusionar» del plan es histórico. |
| ENG-ANL-003 | PR #59; issue #78 | fusionado (#59); abierto (#78) | P-Delta e inercia; falta cobertura de cancelación copysign en bloque 2×2. |
| ENG-ANL-004 | PR #62 | fusionado | Espectro SRSS/CQC; la ampliación de validación #64 sigue abierta. |
| ENG-SOL-001 | PR #56 | fusionado | Newton-Raphson con control de carga. |
| ENG-VAL-001 | PR #57, #73, #64; issue #72 | fusionado (#57 parcial, #73); abierto (#64, #72) | 42 fixtures estáticos; ampliación modal/P-Delta/espectro y seguimiento J4 sin cerrar. |
| ENG-CI-001 | PR #83; issue #68 | fusionado (#83); abierto (#68) | CI Rust y toolchain 1.99.0 según export; #83 no está en la base local. No acredita CI de wheels. |
| ENG-BENCH-001 | PR #67, #74, #79; issue #66 | fusionado (PR); abierto (#66) | Los PR son documentación de la excepción clean-room, no resultados de benchmark. Aquí solo se conciliaron títulos; no se usaron motores externos como oráculos. |

## Tickets sin PR identificado

Los nuevos tickets solicitados se registran expresamente como `sin PR`. Las
dependencias adicionales citadas por el plan se incluyen sin deducir su
implementación. Ninguno tiene issue con su ID en los dos exports.

| ID de ticket | PR(s) / issue(s) | Estado según el corte | Notas |
| --- | --- | --- | --- |
| ENG-API-001 | — | sin PR | Nueva fachada; integra results/1 y JSON canónico. |
| ENG-SFC-002 | — | sin PR | CLI sobre base SDK-WF-007; alinear subcomandos y contrato. |
| ENG-SFC-007 | — | sin PR | Bindings reales V1 sobre engine-api; reactivado desde Archivado/V2 en el plan. |
| ENG-PY-001 | PR #87; PR #91 | fusionado (#87, #91) | ADR-003 aceptado: Jonathan fusionó personalmente #91 (commit 2a21216, 2026-10-14 14:13 ECT; MAM-75). |
| ENG-PY-002 | — | sin PR | Scaffold PyO3/maturin, inicialmente version(). |
| ENG-PY-003 | — | sin PR | API Python tipada para modelos, análisis, resultados y diagnóstico. |
| ENG-PY-004 | — | sin PR | NumPy opcional, matrices y vistas de resultados. |
| ENG-PY-005 | — | sin PR | Workflows y experimentación sobre registro existente. |
| ENG-PY-006 | — | sin PR | Paridad Rust/CLI/WASM/Python y no soportado. |
| ENG-PY-007 | — | sin PR | Wheels, CI, SBOM y gates legales; publicación pendiente de Jonathan. |
| ENG-PY-008 | — | sin PR | Documentación, notebooks y ejemplos. |
| ENG-REL-001 | — | sin PR | Release integrado; espera PY-006/007/008 y SFC-006. |
| ENG-CORE-006 | — | sin PR; alcance/estado a confirmar | El plan lo da como disponible para fachada/workflows, pero ningún título identifica ese ID. No se lo declara fusionado. |
| ENG-CORE-008 | — | sin PR | Archivado **según Notion**, por duplicar motor en Python; no es un estado GitHub. No se reabre. |
| ENG-CORE-010 | — | sin PR; alcance/estado a confirmar | Determinismo citado por ADR-002; títulos genéricos no prueban cierre de este ID. |
| ENG-GOV-004 | — | sin PR; alcance/estado a confirmar | Textos legales y gate de publicación; no inferir cierre de GOV-001/007. |
| ENG-GOV-008 | — | sin PR; alcance/estado a confirmar | Gate legal citado por el DAG; alcance detallado no disponible en los exports. |
| ENG-SFC-001 | — | sin PR; alcance/estado a confirmar | Superficie citada como gate de paridad/release; no hay ID en los exports. |
| ENG-SFC-006 | — | sin PR; alcance/estado a confirmar | Empaquetado, prerrequisito de REL-001 en el DAG. |
| ENG-VAL-004 | — | sin PR; alcance/estado a confirmar | Gate de benchmarks citado por el plan; no confundir con BENCH-001. |

## Ambiguos, a confirmar por Jonathan

| ID de ticket | PR(s) / issue(s) | Estado según el corte | Duda que debe resolverse |
| --- | --- | --- | --- |
| ENG-CORE-004 | Ninguno con ese ID | a confirmar | No hay PR ni issue con el ID en los exports. El alcance no está descrito en las fuentes suministradas; no se puede atribuir trabajo de CORE-001/003/007 o ASM-001 a este ticket. |
| ENG-CORE-005 | Ninguno con ese ID | a confirmar | No hay PR ni issue con el ID. Los PR genéricos del kit de conformidad/SystemView (#30, #37) no indican este ticket; falta confirmar si su alcance lo cubre y en qué medida. |
| ENG-CORE-009 | Ninguno con ese ID | a confirmar | No hay PR ni issue con el ID. El seguimiento multitema #35 incluye wasm32 y catálogo, pero no identifica CORE-009 ni acredita su alcance completo. |

Jonathan debe confirmar título/alcance de cada ticket, correspondencia exacta
con PR o commits y trabajo restante antes de actualizar Notion. La falta de
ID no demuestra que no exista implementación; la similitud de temas tampoco
demuestra que un ticket esté terminado.

## Referencias sin ID de ticket atribuible con certeza

Estos registros completan la cobertura de los exports sin fabricar IDs.
Se conservan en una tabla separada porque sus títulos no permiten asignar un
ticket ENG concreto. Un posible vínculo de tema requiere confirmación.

| PR(s) / issue(s) | Estado según el corte | Notas |
| --- | --- | --- |
| PR #3 | a confirmar (ticket); PR cerrado sin fusión | Smoke de CI clean-room, título «no fusionar»; `CLOSED`, no `MERGED`. |
| PR #25 | fusionado; ticket a confirmar | Catálogo de engine-core, capacidad no soportada ENG-E-0013. Este código de error no es un ID de ticket. |
| PR #26; issue #22 | fusionado (#26); issue cerrado | SparsityPattern fallible; el título identifica cierre de #22, no un ENG-LA concreto. |
| PR #30 | fusionado; ticket a confirmar | Kit SystemView, acumulación en out (T2 J4). |
| PR #34 | abierto; ticket a confirmar | CI Rust + cargo-deny, revisión protegida. No se declara sustituido/cerrado por #83. |
| PR #35; issue #43 | fusionado (#35); abierto (#43); ticket a confirmar | Seguimiento J4 multitema; rescale/subnormales sigue abierto. |
| PR #36; issue #33 | fusionado (#36); issue cerrado | Umbral de pivote invariante; cierre #33 explícito, ticket ENG no identificado. |
| PR #37 | fusionado; ticket a confirmar | Kit de conformidad, incremento FD h. |
| PR #47 | abierto; ticket a confirmar | Propuesta para documento protegido CLEAN_ROOM §5; sin cambio en esta entrega. |
| PR #49 | fusionado; ticket a confirmar | solve_in_place vacío, límites y orden de errores; relacionado por tema con linalg/MemoryBudget. |
| Issue #32 | abierto; ticket a confirmar | Revisor clean-room retroactivo en provenance; no se asigna a un GOV concreto. |
| Issue #65 | abierto; ticket a confirmar | transpose con dimensiones extremas; sin PR vinculado en el corte. |
| Issue #77 | abierto; ticket a confirmar | heap_peak, hilo medidor y saturating_sub; sin PR vinculado en el título. |

Para abrir una referencia se usa
`https://github.com/Mamut1195/mamut-engine/pull/<número>` para PR y
`https://github.com/Mamut1195/mamut-engine/issues/<número>` para issue.
Estas direcciones son identificadores; no se consultaron durante esta tarea.

El diseño y los gates de la superficie Python se registran en
[ADR-003](adr/ADR-003-python-v1.md); la procedencia y las comprobaciones de
esta conciliación están en la [bitácora ENG-PY-001](../provenance/ENG-PY-001.md).
