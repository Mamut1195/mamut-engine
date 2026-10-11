# ADR-002: selección de librerías de álgebra lineal

- **Tarea:** ENG-LA-006
- **Estado:** Aceptado (2026-10-11): aprobado por J0 (jefe de programa); revocable por Jonathan
- **Fecha:** 2026-10-10 (America/Guayaquil)

## Contexto

El motor necesita álgebra densa para matrices de elemento, almacenamiento
disperso para ensamblaje, factorización simétrica para sistemas definidos e
indefinidos y autovalores generalizados para análisis modal. La selección debe
servir en nativo y en `wasm32-unknown-unknown`, sin hilos en V1 ni dependencias
C/Fortran en el build wasm. El [protocolo clean-room](../CLEAN_ROOM.md) exige
dependencias permisivas y procedencia documentada.

La especificación de referencia es
`/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`, §§4.5–4.6, 9 y 10,
junto con `00_PROYECTO.md` y los tickets ENG-LA-006/007/008/009 de
`04_TICKETS.md` en ese mismo directorio. Esos documentos son externos a este
repositorio. El formato sigue [ADR-001](ADR-001-fundacionales.md).

Este ticket es documental: propone la selección y las pruebas para aprobarla.
No incorpora crates, código Rust, configuración de licencias ni workflows.

## Evidencia y comparación

Los metadatos de crates.io siguientes corresponden al corte **2026-10-10
verificado y proporcionado por el solicitante**. Las descargas son totales
aproximadas del crate, no de esa versión ni una medida de calidad. En esta
sesión no fue posible repetir la consulta: el acceso web a crates.io falló y
la consulta directa de su API falló por resolución DNS. Los enlaces identifican
las fuentes del corte; no se afirma una verificación remota nueva.

La evidencia de compilación procede exclusivamente de la prueba desechable del
**2026-10-08 registrada en arquitectura §10**. No se repitió aquí. Esa sección
registra también revisión de archivos LICENSE de faer, nalgebra y sprs, pero
no aporta en este ADR una revisión reproducible del grafo completo por versión.

| Candidato y función evaluada | Licencia | `wasm32-unknown-unknown`: compila / ejecuta | MSRV | Mantenimiento: evidencia disponible | Rendimiento | Tamaño wasm | Determinismo |
|---|---|---|---|---|---|---|---|
| [faer 0.24.4](https://crates.io/crates/faer/0.24.4): denso, disperso, solver simétrico con AMD y autovalores densos según §10 | MIT | **Compila**, según §10, con `default-features = false`, `features = ["std", "sparse-linalg"]`; por defecto falla por `getrandom`. **Ejecuta: PENDIENTE**, V2 | 1.84; compatible con el 1.85 declarado, sujeto al grafo resuelto (V1) | ~5.0 M descargas; actualización 2026-06-24. Salud del mantenimiento: **PENDIENTE**, V7 | **PENDIENTE**, V4 | **PENDIENTE**, V5 | Sin `rayon` propuesto; repetibilidad y equivalencia entre destinos **PENDIENTE**, V6 |
| [nalgebra 0.35.0](https://crates.io/crates/nalgebra/0.35.0): segundo backend denso mínimo | Apache-2.0 | **Compila**, según §10; configuración exacta a reproducir. **Ejecuta: PENDIENTE**, V2 | 1.89; incompatible con 1.85 cuando se habilita | ~94.5 M descargas; actualización 2026-05-24. Salud del mantenimiento: **PENDIENTE**, V7 | **PENDIENTE**, V4 | **PENDIENTE**, V5 | **PENDIENTE**, V6 |
| [nalgebra-sparse 0.12.0](https://crates.io/crates/nalgebra-sparse/0.12.0): almacenamiento disperso alternativo | Apache-2.0 | **Compila**, según §10. **Ejecuta: PENDIENTE**, V2 si se reconsidera | Sin MSRV declarado; mínimo efectivo **PENDIENTE**, V1 | ~669 k descargas; fecha de actualización y salud del mantenimiento **PENDIENTE**, V7 | **PENDIENTE**, V4 si se reconsidera | **PENDIENTE**, V5 si se reconsidera | **PENDIENTE**, V6 si se reconsidera |
| [sprs 0.11.5](https://crates.io/crates/sprs/0.11.5): almacenamiento disperso alternativo | MIT OR Apache-2.0; su solver `sprs-ldl` LGPL-2.1 queda **excluido** según §10 | **Compila**, según §10. **Ejecuta: PENDIENTE**, V2 si se reconsidera | 1.88; incompatible con 1.85 cuando se habilita | ~8.5 M descargas; fecha de actualización y salud del mantenimiento **PENDIENTE**, V7 | **PENDIENTE**, V4 si se reconsidera | **PENDIENTE**, V5 si se reconsidera | **PENDIENTE**, V6 si se reconsidera |
| [scirs2-sparse 0.6.5](https://crates.io/crates/scirs2-sparse/0.6.5): candidato a autovalores dispersos generalizados | Apache-2.0 | **Compila**, según §10. **Ejecuta: PENDIENTE**, V2/V3 | Sin MSRV declarado; mínimo efectivo **PENDIENTE**, V1 | ~42 k descargas; §10 lo considera joven. Fecha de actualización y madurez **PENDIENTE**, V7 | **PENDIENTE**, V4 | **PENDIENTE**, V5 | **PENDIENTE**, V6 |
| arpack-ng (bindings): alternativa para autovalores dispersos | BSD-3-Clause, según §10; versión de bindings y licencia de su grafo **PENDIENTE**, V1 solo si se reconsidera | **Descartado** para wasm por dependencia Fortran/C, según §10; ejecución no probada (**PENDIENTE**, V2 solo si se reconsidera) | No verificado: **PENDIENTE**, V1 solo si se reconsidera | No evaluado: **PENDIENTE**, V7 solo si se reconsidera | **PENDIENTE**, sin medición; V4 solo si se reconsidera | **PENDIENTE**, sin medición; V5 solo si se reconsidera | **PENDIENTE**, V6 solo si se reconsidera |

La comparación permite recomendar una configuración por cobertura funcional y
compatibilidad declarada. No permite afirmar qué librería es más rápida, qué
artefacto es menor ni que un resultado numérico sea idéntico entre destinos.
V1–V7 identifican pruebas concretas del plan siguiente.

## Decisión

1. **Recomendar faer 0.24.4 como backend principal**, con
   `default-features = false` y `features = ["std", "sparse-linalg"]`, sin
   `rayon`. Usar su almacenamiento disperso y su factorización simétrica con
   AMD: Cholesky LLᵀ para matrices definidas y LDLᵀ/LBLᵀ para indefinidas,
   conforme a §10 y ENG-LA-008. La cobertura exacta de APIs con estas features,
   incluidas las operaciones densas y autovalores, queda **PENDIENTE** de V3.
   Si alguna API requiere otra feature, documentar y revisar el ajuste antes
   de incorporarlo; no activar las features por defecto como solución implícita.
2. **Recomendar nalgebra 0.35.0 como segundo backend denso mínimo** en
   ENG-LA-007, detrás de una feature opcional y deshabilitada por defecto.
   Pasará la misma suite de operaciones densas y resolución de sistemas
   pequeños que faer para demostrar intercambiabilidad. No se le atribuye
   cobertura del solver disperso ni del análisis modal completo.
3. **Mantener el MSRV 1.85 del workspace** para la configuración principal.
   [Cargo.toml](../../Cargo.toml) declara actualmente `rust-version = "1.85"`;
   nalgebra exige **1.89** y sprs **1.88**. La alternativa sería subir todo el
   workspace a 1.89; se descarta por ahora para conservar la compatibilidad del
   camino principal con faer. La feature de nalgebra requerirá Rust **al menos
   1.89**, con mínimo efectivo confirmado sobre dependencias transitivas en V1.
   ENG-LA-007 deberá documentar ambos perfiles y probar el principal con 1.85
   y el segundo con su toolchain compatible. La selección opcional no convierte
   nalgebra en compatible con 1.85; `--all-features` no tendrá esa garantía.
   Este ADR no modifica el manifest.
4. **Definir un trait propio `LinAlgBackend` en ENG-LA-007**, con tipos del
   motor para matrices densas, tripletas dispersas, factorización con `solve`
   e información de inercia y autovalores generalizados. Solo `engine-linalg`
   importará las librerías. Los contratos `linsolve/1` y `eigen/1` de §§4.5–4.6
   seguirán visibles mediante tipos propios; los errores se traducirán a
   `ENG-E` con contexto de nodo/GDL cuando esté disponible.
5. **Mantener PENDIENTE la inercia expuesta por faer**: confirmar acceso a los
   signos de D, incluidos bloques 2×2, y su interpretación en V3. El campo
   `FactorInfo.inertia` admite ausencia de información; no se anunciará control
   de Sturm ni detección de modos faltantes sin evidencia. No se implementará
   una factorización propia para cubrir esta incertidumbre.
6. **Usar faer como candidato denso para ENG-LA-009**, conforme a
   `self_adjoint_eigen`/`gevd` citados en §10, sujeto a la verificación de V3.
   **Solver disperso de autovalores: PENDIENTE.** Evaluar scirs2-sparse y la
   interoperabilidad de `eigsh_generalized`/shift-invert con la factorización
   de ENG-LA-008. Compilar ese crate no demuestra madurez, corrección ni que
   permita reutilizar nuestra factorización. No queda seleccionado todavía.

## Alternativas descartadas

- **nalgebra como principal:** no ofrece en esta evaluación la cobertura
  integrada propuesta para el solver disperso y exige subir el MSRV. Se conserva
  como segundo backend denso opcional.
- **nalgebra-sparse y sprs como almacenamiento principal:** añadirían otra
  representación y conversiones al solver de faer sin beneficio medido. Pueden
  reconsiderarse con V2/V4/V5/V6; su compilación histórica no decide la selección.
- **sprs-ldl:** excluido por LGPL-2.1 frente a la política de dependencias
  permisivas. La exclusión del solver no afecta la licencia permisiva de sprs.
- **scirs2-sparse como elección inmediata para autovalores dispersos:** se
  aplaza hasta resolver V1/V2/V3/V7. No hay evidencia suficiente de madurez ni
  de integración con los contratos exigidos.
- **arpack-ng y builds con BLAS/LAPACK/ARPACK C/Fortran:** descartados para el
  build wasm y el requisito de backend Rust. Su licencia permisiva no resuelve
  la dependencia de toolchains y bibliotecas externas.
- **Features por defecto de faer o paralelismo V1:** incompatibles con la
  configuración wasm registrada o con la política sin hilos. No se propone
  álgebra lineal propia para sustituir librerías.

## Consecuencias

- El resto del motor dependerá del contrato propio, con pruebas comunes para
  las capacidades compartidas y capacidades dispersas verificadas por separado.
  La integración y los cambios de tipos provisionales de ENG-CORE-007
  corresponden a ENG-LA-007, no a este ADR.
- La feature de nalgebra tendrá un requisito de toolchain superior al perfil
  principal. La resolución del grafo y la unificación de features deberán
  comprobar que no activa nalgebra ni `rayon` en el build principal.
- La selección principal es provisional hasta validar ejecución wasm, APIs,
  licencias transitivas y MSRV efectivo. No hay estimaciones de rendimiento ni
  tamaño wasm verificadas; los presupuestos siguen **PENDIENTE**.
- ENG-LA-008 conserva pendientes los diagnósticos de singularidad y la inercia;
  ENG-LA-009 conserva pendiente el solver disperso y el control de modos
  faltantes. No se declara completo el análisis modal por seleccionar faer.
- Sin hilos, orden estable de ensamblaje y semillas fijas reducen fuentes de
  variación, pero SIMD y FMA pueden diferir entre destinos. La igualdad bit a
  bit sigue **PENDIENTE** en ENG-CORE-010.

## Plan de verificación

Estas acciones se ejecutarán en los tickets de integración y gobernanza; no
se realizaron pruebas numéricas ni builds de candidatos en este ticket.
Cada informe deberá registrar versiones, features, toolchain, target, entradas,
comandos y resultados, con bitácora clean-room. Solo se usarán fuentes permitidas
y soluciones analíticas, sin motores externos como oráculos.

1. **V1 — Licencias y MSRV (ENG-GOV-005 / ENG-LA-007).** Repetir la consulta
   a crates.io de las versiones exactas; contrastar su expresión SPDX con los
   archivos LICENSE oficiales de esas versiones y los del grafo transitivo,
   sin abrir código de motores. Preparar en ENG-GOV-005 un `deny.toml` con
   allow-list de MIT, Apache-2.0, BSD, ISC, Zlib y Unicode, usando identificadores
   SPDX concretos para las variantes BSD/Unicode presentes; rechazar licencias
   desconocidas o fuera de esa lista. Ejecutar `cargo-deny check licenses`
   para ambos perfiles y targets, con nalgebra activado y desactivado, incluyendo
   scirs2-sparse si llega a proponerse. Revisar `cargo tree -e features` y
   `cargo tree --target wasm32-unknown-unknown`: ausencia de `sprs-ldl`,
   `rayon` y dependencias C/Fortran/BLAS/LAPACK/ARPACK en el perfil wasm.
   Construir el perfil principal con Rust 1.85 y el de nalgebra con 1.89;
   registrar cualquier exigencia transitiva mayor. Para crates sin MSRV
   declarado, determinar el mínimo efectivo mediante builds con toolchains
   fijadas. **No se crea `deny.toml` en esta entrega.**
2. **V2 — Compilación y ejecución nativa/wasm (ENG-LA-007/008/009).** Crear
   la suite de integración al implementar los adaptadores y compilar los
   perfiles por separado para `wasm32-unknown-unknown`, reproduciendo las
   features de faer de §10. Ejecutar la misma suite en nativo y en Node mediante
   un runner de pruebas wasm (`wasm-bindgen-test`, por ejemplo con
   `wasm-pack test --node`), registrando versión de Node y toolchain. Cubrir
   operaciones densas, ensamblaje por tripletas, solución conocida SPD y
   simétrica indefinida, singularidad y varios RHS con una factorización.
   Exigir residual relativo < 1e-12 en sistemas bien condicionados según
   ENG-LA-008; declarar norma y tratamiento de RHS cero. Ejecutar en CI al
   integrar esos tickets; **este ADR no crea workflows**. Las alternativas
   de almacenamiento solo recibirán esta suite si se reconsideran.
3. **V3 — Cobertura funcional e inercia (ENG-LA-007/008/009).** Comprobar en
   documentación/API y en pruebas de la versión fijada las operaciones densas,
   la separación simbólica/numérica con AMD y la reutilización de factores.
   Verificar que las features mínimas alcanzan las APIs densas y dispersas
   requeridas. Probar inercia con matrices de signos conocidos, permutaciones,
   caso singular y matriz indefinida que requiera pivote de bloque 2×2; verificar
   que un eventual desplazamiento/regularización no confunda la inercia original.
   Si no hay API verificable, conservar ausencia de inercia y documentar la
   limitación del control de Sturm. Para autovalores, usar K y M diagonales
   con solución cerrada, masa singular y benchmarks analíticos de viga apoyada
   y voladizo; comprobar residual `Kφ − λMφ`, normalización `φᵀMφ = 1` y
   multiplicidades mediante subespacios. Evaluar scirs2-sparse con shift-invert,
   shifts próximos a autovalores, no convergencia y modos faltantes; confirmar
   si puede consumir nuestra operación de solución sin duplicar factorizaciones.
   Fijar tolerancias de los fixtures antes de evaluar resultados. Registrar
   selección o rechazo del solver disperso en una revisión de este ADR.
4. **V4 — Rendimiento (ENG-LA-007/008/009 / ENG-VAL-004).** Medir familias
   reproducibles de matrices de elemento, bandas y dispersas estructurales con
   tamaños, `nnz`, condición y semillas registrados. Separar ensamblaje,
   análisis simbólico, factorización, solución por RHS y autovalores; medir tiempo
   y memoria en nativo y Node, tras calentamiento y con repeticiones, en la misma
   máquina. Comparar faer/nalgebra solo en capacidades comunes, y otros candidatos
   si se reconsideran. Publicar dispersión y entorno; acordar presupuestos
   antes de aceptar o rechazar por rendimiento. No extrapolar de descargas.
5. **V5 — Tamaño wasm (ENG-LA-007 / ENG-VAL-004).** Construir en release
   artefactos equivalentes con el mismo código consumidor, LTO, optimización y
   herramientas fijadas; comparar base sin backend, faer mínimo y perfil con
   nalgebra, separando también cualquier candidato modal. Medir bytes de `.wasm`
   y gzip con opciones fijas, conservando comandos y hashes. El objetivo de
   **1 MB gzip o menos** de `00_PROYECTO.md` es una propuesta **PENDIENTE**,
   no un resultado ni un presupuesto aprobado. Determinar también límites de
   modelo/memoria; §10 limita V1 a wasm32 sin Memory64.
6. **V6 — Determinismo (ENG-CORE-010).** Ejecutar repetidamente los mismos
   fixtures en cada destino y cruzar nativo/Node. Registrar orden de tripletas
   y sumas, backend, features, SIMD, flags y semillas; sin hilos ni activación
   de `-C target-feature=+fma`, con estructuras ordenadas y la política `libm`
   de §9 para funciones escalares. Comparar bits de salidas escalares; en
   autovectores fijar convención de signo y comparar subespacios para valores
   repetidos. Si difieren los destinos, publicar diferencias y tolerancias
   justificadas. No prometer igualdad bit a bit por deshabilitar `rayon`.
7. **V7 — Mantenimiento.** Para cada candidato aún considerado, revisar
   historial de releases y cambios incompatibles, política de MSRV, documentación,
   pruebas nativas/wasm publicadas, issues y respuesta de mantenedores. Registrar
   fechas y enlaces de documentación oficial sin abrir código prohibido. Para
   scirs2-sparse, exigir evidencia específica del solver generalizado y casos
   de convergencia, masa singular e interoperabilidad. Las cifras de descargas
   por sí solas no satisfacen esta revisión.

## Aceptación (2026-10-11)

Este ADR pasa de provisional a **aceptado** el 2026-10-11: **aprobado por J0
(jefe de programa); revocable por Jonathan**. Se acepta faer como backend
principal y nalgebra como segundo opcional. La evidencia medida está en
[ADR-002-evidencia.md](ADR-002-evidencia.md).

Queda **PENDIENTE** (no cubierto por esta aceptación): matrices dispersas
(solver disperso), LDLt, autovalores, `simd128`, `wasm-opt`, memoria y CI.

Garantías de determinismo: con faer **no se promete igualdad bit a bit entre
nativo y wasm**, ni con `rayon` entre distinto número de hilos.

Licencia de manifests: ADR-001 registra la licencia dual AGPL-3.0 + comercial,
que no es una única expresión SPDX; la expresión `license` de los `Cargo.toml`
es **decisión pendiente de Jonathan** y esta aceptación no la fija.

## Criterios de aceptación de ENG-LA-006

| Criterio del ticket | Estado | Evidencia o trabajo restante |
|---|---|---|
| Evaluación y decisión documentada de álgebra densa, almacenamiento disperso, solver simétrico y autovalores generalizados | **Cumplido** para la comparación y recomendación documental; solver disperso **PENDIENTE** | Tabla, decisión y alternativas; V3 debe cerrar el solver disperso |
| Licencia de cada crate verificada en crates.io y LICENSE oficial, y aceptada por `cargo-deny` | **PENDIENTE** | Corte crates.io proporcionado; §10 registra LICENSE de faer/nalgebra/sprs. Falta revisión reproducible por versión y grafo completo, y `cargo-deny` (V1) |
| Cada crate elegido compila y corre pruebas en wasm (Node) y nativo en CI | **PENDIENTE** | Compilación wasm histórica registrada en §10; ejecución y CI de integración no verificadas (V2) |
| Sin dependencias C/Fortran (BLAS, LAPACK, ARPACK) en el build wasm | **PENDIENTE** de verificación del build | Restricción y descarte documental de arpack-ng **cumplidos**; falta grafo y build de los perfiles elegidos (V1/V2) |
| Decisión aprobada | **Cumplido** (2026-10-11) | Aprobada por J0 (jefe de programa); revocable por Jonathan |

## Pendientes

- [x] Aprobación (2026-10-11) por J0; revocable por Jonathan. La recomendación de MSRV por perfil queda incluida.
- [ ] V1: licencias del grafo, `cargo-deny`, MSRV efectivo y features resueltas.
- [ ] V2: ejecución real de pruebas en nativo y wasm en Node/CI.
- [ ] V3: APIs con features mínimas, inercia de faer y solver disperso de autovalores.
- [ ] V4/V5: rendimiento, memoria, tamaño wasm y presupuestos aprobados.
- [ ] V6/V7: determinismo comprobado y revisión de mantenimiento/madurez.

La [bitácora ENG-LA-006](../../provenance/ENG-LA-006.md) registra las fuentes,
el alcance y los límites de verificación de esta sesión.
