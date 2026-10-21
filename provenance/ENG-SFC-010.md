# Bitácora clean-room · ENG-SFC-010

- Ticket / PR: MAM-835 / ENG-SFC-010; cambios sin commit, revisión pendiente.
- Autor: Codex gpt-6.1-sol high, 2026-10-20 (America/Guayaquil; fecha específica del ticket).
- Rol: especificador e implementador de frontera.
- Contratos: engine-input/1 y results/1; adaptación de element/1 y uniaxial/1 existentes.
- Fuentes: inventario local `engine-v1-capacidades.md` aportado en el contexto; código propio de engine-api, engine-analysis/newton.rs, engine-geo, engine-laws/uniaxial, frame y spring; docs/GUIA_METODOS.md §5; provenance/ENG-GEO-001.md, ENG-GEO-002.md y ENG-GEO-006.md.
- Fuentes analíticas: derivaciones propias de equilibrio y energía descritas abajo. Contexto bibliográfico: Hetényi, M. (1946), *Beams on Elastic Foundation*, University of Michigan Press (ecuación EI v'''' + k v = q); Terzaghi, K. (1943), *Theoretical Soil Mechanics*, Wiley, capítulo XIII (consolidación unidimensional). No se consultaron fuentes externas ni se copiaron tablas, figuras o código.
- Código de terceros copiado: ninguno. Sin dependencias externas nuevas. Cargo.toml/Cargo.lock solo enlazan tres crates locales imprescindibles para el despacho: engine-elements-spring, engine-laws y engine-geo.
- Núcleo numérico: intacto. No se modifica engine-analysis, engine-geo, engine-laws, engine-elements-* ni engine-linalg. No hace falta API pública adicional. La adaptación de resultados Newton reúne respuestas de Element y LinearSystem; no implementa un solver ni una ley.
- Validación: fixtures analíticos propios `sfc010-*.json`, pruebas API, CLI, Python y WASM nativo. Resultados y comprobaciones finales registrados abajo.
- Comparación con oráculo externo: no.
- Declaración: «No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket». Tampoco consulté código o resultados de CalculiX, LAPACK ni directorios de oráculos. No implementé, modifiqué ni leí la implementación de barra/celosía.
- Revisor clean-room: **PENDIENTE (J4)**.

## Derivaciones y fuentes de cada fixture

Los JSON son documentos engine-input/1 estrictos, sin metadatos de prueba. Esta tabla aporta su `source` y cita sin añadir campos ajenos al contrato.

| Fixture (prefijo sfc010-) | Source / solución propia | Aceptación |
| --- | --- | --- |
| spring-series | Equilibrio de dos resortes en serie: F1=F2=P; u1=P/k1; u2=P/k1+P/k2 | P=12, k1=100, k2=200: u1=0.12, u2=0.18 |
| multilinear | Interpolación de la tabla de fuerza-desplazamiento explícita: F=10+20(δ−0.1) | F=16: δ=0.4 |
| cubic | docs/GUIA_METODOS.md §5: F=8δ+16δ³ | F=6: δ=0.5 |
| steel | Equilibrio F=P, retorno 1D ya documentado en la ley propia. Pendiente inicial 100, fluencia 10, pendiente posterior 20; H=25. Descarga elástica hasta fluencia inversa | Factores [0.5,1,0.5,0,−0.5,0] sobre P=16: δ=[0.08,0.4,0.32,0.24,0,0.08]; memorias [δp,α] |
| spring3d | Tres direcciones independientes; Fx=100Ux, Fy=20Uy, Fz=30Uz | P=16: Ux=0.16, Uy=Uz=0 |
| winkler2d / winkler3d | Ecuación de Winkler–Hetényi EI v''''+kv=q; solución particular exacta v=q/k con giros cero. Está en el espacio Hermite: sin error de malla. Apoyo axial y torsional donde corresponde | qy=−20, ky=10: v=−2; qz=−60, kz=20: w=−3; flexión interior nula |
| release2d / release3d | Equilibrio de viga articulada con q uniforme: Ri=Rj=−qL/2; M(x)=q x(L−x)/2 según signo local. Giros nodales desconectados restringidos para evitar mecanismo ajeno | L=4: reacciones 40 en y, 120 en z D3; momentos de extremos liberados cero |
| soil | Integración σv=∫γ dz, u=γw max(z−zw,0), σ'v=σv−u (provenance/ENG-GEO-001.md) | A z=5: σv=101 kPa, u=40 kPa, σ'v=61 kPa; γw supuesto preservado |
| consolidation | Terzaghi 1943, ecuación 1D homogénea, dos capas contiguas de iguales propiedades, drenaje superior. U=1−Σ[8/(π²(2m+1)²)] exp[−(2m+1)²π² Tv/4], Tv=cv t/H²; s∞=Δσ mv H | U(t) con error absoluto <0.03 (malla 40 elementos, Euler implícito dt≤0.05 s); s∞=0.02 m. Se explicita el error inicial de interpolación del dren, sin recortarlo |

La prueba adicional de consolidación con capas distintas verifica el límite drenado s∞=ΔσΣ mv_i H_i y continuidad del campo nodal compartido; no usa valores de otro programa.

## Decisiones de frontera

- Campos nuevos opcionales y ausentes de los documentos históricos; sin cambiar su serialización ni hashes.
- `model.springs` permite uno o dos nodos; su id comparte espacio con members. La salida usa members para recuperación de Element; section vacío, referencia law si existe y etiquetas locales end_dofs en la identidad de un resorte.
- Leyes explícitas en model.laws; las no lineales solo se conectan a spring.nonlinear. No se añade plasticidad de sección ni se toca truss.
- Winkler usa sus fábricas existentes y conserva su prohibición de liberaciones. Resortes se ofrecen en linearStatic/nonlinearStatic. Combinaciones lineales siguen disponibles; Newton rechaza combinaciones y diafragmas.
- Newton llama al solver existente por cada factor absoluto manteniendo el mismo NewtonSystem y sus historias. Publica normas, resultados e historia de cada paso aceptado en convergence.steps y el último caso si termina. Un paso sin convergencia no publica desplazamientos de prueba; preserva informes previos, se detiene y retorna salida 4. Cancelación entre pasos.
- Consolidación llama solve una vez para conservar su discretización temporal; cancelación antes/después de esa operación indivisible. No se altera el integrador para informar progreso.
- Los análisis geotécnicos requieren colecciones estructurales vacías; propiedades con unidades y origen explícitos. No se inventan constantes físicas.

## Comprobaciones finales offline

Entorno: toolchain stable del ticket, CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine-sfc10, sin red, PIP_NO_INDEX=1. No se hicieron commits ni cambios de rama. El árbol y el índice Git real se conservan sin staging.

| Comprobación | Resultado |
| --- | --- |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline | **686 passed, 0 failed, 0 ignored**, incluidas pruebas de documentación; 108 ejecutables/suites |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | **Clean-room: OK**, incluyendo todos los archivos nuevos mediante índice temporal externo |
| bindings/python/tests, unittest discover | **45 passed, 0 failed**, con NumPy disponible |
| Determinismo publicado | Los **16 hashes** de docs/DETERMINISMO.md idénticos; prueba automática contra esa tabla, sin modificarla |
| Paridad SFC-010 | **11 fixtures**: API compartida, sobre CLI, FFI/fachada Python y función exportada WASM ejecutada en nativo; bytes canónicos completos idénticos |

La CLI publica un sobre adicional: se extrae engineResults[0] y se compara su JSON canónico, sin quitar campos del resultado. WASM nativo y Python usan el mismo target que API/CLI. La comprobación wasm32 compila toda la frontera; no se afirma igualdad del campo target entre destinos diferentes ni ejecución JS de estos fixtures.

Python: maturin no estaba disponible offline. Se compiló la extensión real con `cargo build --offline -p mamut-engine-py --features pyo3/extension-module --example facade_reference --lib`. Se instaló en un venv temporal un paquete local de prueba con esa extensión, los archivos Python y metadatos derivados de pyproject/Cargo, sin descargas. Los 27 documentos de referencia (16 entradas válidas más 11 entradas estructurales de determinismo) se emitieron con facade_reference, independientemente del binding. No se verificó aquí la reproducibilidad del empaquetador maturin, que no forma parte de este cambio.

El guardia clean-room solo observa archivos rastreados. Para revisar también los nuevos sin escribir en .git se creó un índice Git temporal y un directorio temporal de objetos en CARGO_TARGET_DIR; `git add --intent-to-add` actuó exclusivamente sobre ese índice alternativo. El índice real no se modificó. Los logs y artefactos locales de esta sesión están bajo CARGO_TARGET_DIR/mam835-artifacts.

Incidencias corregidas: la primera suite Rust detectó el uso de exp de std en la prueba analítica; se sustituyó por engine_core::math::exp, sin tocar el guardia ni el núcleo. La primera ejecución Python desde PYTHONPATH carecía de metadatos instalados; la instalación local y los ajustes de las pruebas para resultados geotécnicos sin nodos permitieron ejecutar las 45 pruebas completas. Inicialmente se creó material temporal en /tmp y se buscó únicamente nombres AGENTS.md con alcance /workspace; ese alcance excedía la regla operativa del ticket. Los temporales propios se trasladaron a CARGO_TARGET_DIR, las búsquedas siguientes se limitaron al worktree y no se abrió contenido de otros repos ni contenido prohibido. La única lectura de un documento externo al worktree fue el inventario solicitado expresamente por el usuario.

Pendiente: revisión clean-room **J4**; no hay implementación ni comprobaciones solicitadas pendientes. No se modificaron clean-room/**, docs/CLEAN_ROOM.md, workflows, CODEOWNERS, vendor ni crates numéricos.

## r2 — revisión J4

- Ticket: MAM-835 / ENG-SFC-010, correcciones H1 y H3 sobre d72ee11.
- Autor: **Codex gpt-6.1-sol high**, fecha **2026-10-17** (America/Guayaquil).
- Rol: implementador. Contratos: engine-input/1, results/1 y consolidation/1.
- Fuentes permitidas: implementación propia local de geo_input, consolidation,
  extensions y run; docs/GUIA_METODOS.md y docs/CLEAN_ROOM.md. Sin fuentes
  externas, nuevas dependencias, tablas, figuras ni código de terceros.
- Derivación propia H1: unir los quiebres de carga y tiempos de salida; sumar
  max(1, ceil((fin−inicio)/max_step_s)) por intervalo en toda la historia,
  incluso después de la última salida, y multiplicar por elementos totales.
  Acumulación y producto u128 saturados, incluida la conversión del cociente
  no finito. `engine_api::MAX_CONSOLIDATION_WORK = 50_000_000` es pública y
  rechaza superar la cota antes de integrar con API-E-CONSOLIDATION-WORK
  (invalid / 2). La cota mide trabajo; no garantiza un plazo del backend denso.
- H1 cancelación: `solve_with_cancel` consulta el cierre antes de cada subpaso;
  `solve` y `solve_with_backend` delegan con cierre false. La aritmética,
  factorización y partición temporal se mantienen. La fachada conecta el
  CancelToken y conserva API-I-CANCELLED (cancelled / 5), sin salida parcial.
- H3: pruebas exactas de código/estado/salida para M1 (1024 miembros+resortes
  aceptados, 1025 rechazados), M4 (Winkler en viga Euler), M8 (ley no lineal en
  viga), M9 (Newton con combinaciones) y M10 (resortes en modal y espectro).
  Se exige longitud seis antes del zip de la historia del acero.
- Validación propia: rechazo rápido (<2 s) de trabajo agregado excesivo,
  cociente infinito, aceptación de la cota exacta y cómputo de redondeos por
  salidas. Cancelación desde otro hilo durante una corrida larga bajo la cota;
  prueba del dominio que cuenta cada consulta y compara bits con cierre false.
  Se capturaron antes del cambio los documentos completos de los 27 fixtures
  válidos (16 entradas API y 11 de determinismo) para comparación byte a byte.
- Sin cambios de estado Stable/Experimental, N1/N2, fixtures, hashes publicados,
  dependencias ni archivos protegidos. El único cambio del integrador es el
  punto de cancelación y las funciones delegadas; no se cambia física numérica.
- Código de terceros copiado: ninguno. Comparación con oráculo externo: no.
- Declaración clean-room: «No consulté código fuente de OpenSees, Stabileo ni
  derivados para este ticket». Tampoco consulté código ni resultados de
  CalculiX, LAPACK ni repositorios de oráculos. Trabajo offline.
- Incidencias: la búsqueda inicial de nombres AGENTS.md se extendió a /workspace
  por usar `find ..`; solo enumeró rutas, sin abrir contenido de otros repos.
  Se corrigió el alcance de las lecturas siguientes al worktree. La primera
  prueba M9 usó `name` en lugar del campo `id` de combinación; se corrigió la
  entrada de prueba al contrato local, sin modificar el parser.
- Revisor clean-room: **PENDIENTE (J4)**.

### Comprobaciones r2

Entorno offline: PATH antepone stable-x86_64-unknown-linux-gnu (rustc 1.99.0),
CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine-sfc10, PIP_NO_INDEX=1.
Logs, capturas y temporales: CARGO_TARGET_DIR/mam835-r2-artifacts; sin staging
ni commits. No se modificaron archivos protegidos ni manifiestos Cargo.

| Comprobación | Resultado |
| --- | --- |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline | **693 passed, 0 failed, 0 ignored**, 108 suites, incluidas pruebas de documentación; siete pruebas nuevas |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | **Clean-room: OK** |
| Comparación contra la captura previa de d72ee11 | **27/27 fixtures** con documentos completos idénticos byte a byte |
| Determinismo publicado | Los **16 hashes** de docs/DETERMINISMO.md idénticos, comprobados por la prueba automática existente |

Pendiente únicamente la revisión clean-room **J4**; no quedan cambios ni
comprobaciones solicitadas por ejecutar. N1/N2 y estados de métodos intactos.


## r2 — H2: métodos Experimental

- Ticket: MAM-835 / ENG-SFC-010, ronda r2, punto H2; se construye sobre los
  cambios H1/H3 sin deshacerlos.
- Autor: **Codex gpt-6.1-sol high**, fecha **2026-10-17** (America/Guayaquil).
- Rol: implementador de frontera. Contratos: engine-input/1,
  method-descriptor/1 y capabilities/1.
- Decisión: **decisión de Jonathan vía J0 2026-10-20**: `nonlinearStatic`,
  `soilProfile` y `consolidation1d` se registran como `Experimental` hasta que
  ENG-VAL-006 cubra su validación. Los otros cinco estados se conservan.
- Mecanismo: se mantiene el gate existente del registro, con
  `options.allow_experimental = true` en la raíz del job. Sin consentimiento,
  `run` devuelve invalid / API-E-INPUT / salida 2 antes del cálculo. Catálogo,
  descripciones de esquemas, explain y guía reflejan esta política.
- Fixtures: solo los seis válidos y tres inválidos SFC-010 que seleccionan
  esos métodos añaden la habilitación explícita. La prueba que adapta un
  voladizo a Newton la añade a su entrada derivada. Ningún fixture anterior
  a SFC-010 se modifica. El cambio legítimo de input_sha256 de los seis
  válidos se comprueba por separado del resto del documento.
- Fuentes permitidas: código propio local de methods, capabilities, run y
  pruebas de frontera; docs/GUIA_METODOS.md y docs/CLEAN_ROOM.md. No se añaden
  ecuaciones, oráculos, tablas ni figuras. Las soluciones analíticas propias
  de SFC-010 descritas arriba se conservan.
- Validación: pruebas de los ocho estados en API, gate sin opción/con opción
  false, paridad de fixtures habilitados y rechazo en CLI, Python y WASM
  nativo; comprobación de los 16 hashes publicados y capturas previas.
- Código de terceros copiado: ninguno. Comparación con oráculo externo: no.
- Declaración clean-room: «No consulté código fuente de OpenSees, Stabileo ni
  derivados para este ticket». Tampoco consulté código ni resultados de
  CalculiX, LAPACK ni repositorios de oráculos. Trabajo offline y limitado
  al worktree y CARGO_TARGET_DIR. Sin cambios de física, dependencias,
  manifiestos Cargo ni archivos protegidos.
- Revisor clean-room: **PENDIENTE (J4)**.

### Comprobaciones H2

Entorno offline: stable-x86_64-unknown-linux-gnu, rustc 1.99.0,
CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine-sfc10, PIP_NO_INDEX=1.
Logs y capturas en CARGO_TARGET_DIR/mam835-h2-artifacts; sin staging ni commits.

| Comprobación | Resultado |
| --- | --- |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline | **696 passed, 0 failed, 0 ignored**, 108 suites, incluidas pruebas de documentación; tres pruebas Rust nuevas en H2 |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | **Clean-room: OK** |
| Python unittest discover, extensión nativa local y NumPy disponible | **46 passed, 0 failed**, una prueba nueva en H2 |
| Captura previa H1/H3 frente a H2 | **21/21 documentos completos idénticos**, incluidos los 11 fixtures de determinismo; los otros **6/6 SFC-010** cambian exclusivamente input_sha256 por el consentimiento añadido |
| Determinismo publicado | **16/16 hashes** de docs/DETERMINISMO.md idénticos, verificados por all_sixteen_published_hashes_remain_unchanged |
| Paridad SFC-010 | **11/11 fixtures habilitados** con bytes iguales en API/CLI/Python/WASM nativo |

Se comprueba explain para API-E-INPUT y API-E-SFC-OPTIONS: ambos indican la
madurez Experimental, los tres métodos, el consentimiento y ENG-VAL-006.
La CLI emite los sobres de fallo por stderr con salida 2. Python conserva su
traducción existente a InvalidInputError en FFI y fachada. No se altera el
contrato de errores de ninguna superficie.

Python: maturin no está provisionado offline. Se compiló la extensión real con
`cargo build --offline -p mamut-engine-py --features pyo3/extension-module
--example facade_reference --lib`, se copió con el paquete Python a un venv
local en CARGO_TARGET_DIR y se añadieron metadatos de versión y extra NumPy
según Cargo.toml/pyproject.toml. Las 27 referencias se emitieron mediante
facade_reference independiente del binding. No se certifica el empaquetado
maturin. WASM: pruebas de la función exportada ejecutadas en nativo y
compilación wasm32; no se afirma ejecución en JS de SFC-010 en esta ronda.

Durante el ajuste de las pruebas se corrigieron supuestos de prueba sobre el
campo obligatorio options, convergence null, sobres de error CLI por stderr y
excepciones nativas Python. Se conservó el comportamiento del motor.

Pendiente: revisión clean-room **J4** y cobertura de validación **ENG-VAL-006**
antes de reconsiderar los tres estados. Las comprobaciones solicitadas de H2
están completas. H1/H3 se conservan; ningún manifiesto Cargo ni archivo
protegido fue modificado en esta ronda.

## r3 — revisión J4 H1'

- Ticket: **MAM-835 / ENG-SFC-010**, r3 sobre r1, r2 y el merge de main.
- Autor: **Codex gpt-6.1-sol high**, fecha **2026-10-17** (America/Guayaquil).
- Rol: implementador. Contratos: engine-input/1, consolidation/1 y control
  cooperativo opcional de LinAlgBackend; se conservan resultados y estados.
- Fuentes permitidas: código propio local de geo_input, consolidation,
  LinAlgBackend y ReferenceBackend, sus pruebas, docs/GUIA_METODOS.md,
  docs/DETERMINISMO.md y docs/CLEAN_ROOM.md. Derivación propia de coste y
  medición del motor local; sin nuevas ecuaciones físicas, fuentes externas,
  dependencias, tablas ni figuras ajenas.
- Código de terceros copiado: ninguno. Comparación con oráculo externo: no.
- Declaración clean-room: «No consulté código fuente de OpenSees, Stabileo ni
  derivados para este ticket». Tampoco consulté código ni resultados de
  CalculiX, LAPACK ni repositorios de oráculos. Todo el trabajo fue offline.
- Revisor clean-room: **PENDIENTE (J4)**.

### Derivación y decisiones r3

La combinación M+theta·dt·K se densifica y LDLᵀ factoriza un sistema de `n`
nodos libres por intervalo. Cada subpaso resuelve con ese factor denso.
De `free_range`, con E elementos hay E+1 nodos y se eliminan uno o dos drenes:
n=E con drenaje simple; n=E−1 con doble. El dominio exige al menos un nodo
libre. Se añade el coste **Σ_intervalos n³ + Σ_subpasos n²**, sin medir tiempo
durante la validación ni descontar la caché. Sumas, productos y potencias usan
`u128` saturante; el cociente infinito convertido a subpasos también satura.

Se recorren los quiebres de carga y los tiempos de salida en el mismo orden
que solve: cada intervalo positivo aporta max(1, ceil(span/max_step_s)). La
validación continúa hasta el final de la historia aunque solve termine en la
última salida, por lo que la estimación es conservadora. Las coincidencias
entre carga/salida se cuentan una sola vez; la salida inicial no crea tramo.

| Constante pública en engine_api | Valor | Código estable |
| --- | ---: | --- |
| MAX_CONSOLIDATION_WORK, r2 conservada | 50_000_000 | API-E-CONSOLIDATION-WORK |
| MAX_CONSOLIDATION_COST, propia de r3 | 500_000_000 | API-E-CONSOLIDATION-COST |
| MAX_CONSOLIDATION_SEGMENTS, intervalos de historia/salida | 256 | API-E-CONSOLIDATION-COST |
| MAX_CONSOLIDATION_SUBSTEPS, agregados | 100_000 | API-E-CONSOLIDATION-COST |

Se conserva la prioridad del rechazo r2. Los rechazos r3 también son
`invalid`, salida 2, sin integrar. MAX_CONSOLIDATION_SEGMENTS limita además
los tramos de historia a 256 (257 puntos); los tiempos de salida pueden reducir
lo disponible. El coste implica n≤793: no se necesita otra cota de elementos
(≤793 con drenaje simple, ≤794 con doble). Los topes del dominio permanecen.
Los topes de intervalos/subpasos son necesarios para el coste fijo por paso
cuando n es pequeño. Explain, esquema y guía explican topes y remedios.

Solo la ruta ReferenceBackend de solve/solve_with_cancel conserva el último
factor cuando dt.to_bits() coincide con el intervalo anterior. M, K y theta
son inmutables; el mismo dt reproduce el mismo producto theta·dt, cada entrada
de la matriz y el factor LDLᵀ secuencial determinista. El factor posee sus datos
y solve lo consulta inmutablemente. Se reutiliza sin reconstruir la matriz;
un dt distinto reemplaza la caché. solve_with_backend conserva cada llamada:
el contrato de un backend externo no promete ausencia de estado. No se cambia
ni un orden de suma/producto/pivoteo, tolerancia, partición temporal o solver.
Pruebas comparan todos los bits con ReferenceBackend explícito sin caché,
incluyendo drenajes, descarga, tensión cero, dt repetidos y dt distintos.

Cancelación: antes de estabilidad/combine en **cada** intervalo, antes del
equilibrado/factorize y entre bloques de hasta 16 columnas LDLᵀ, además de cada
subpaso r2. factorize_with_check es un método opcional del trait: por defecto
consulta el control antes de factorize; la referencia consulta entre bloques.
El umbral de 15 columnas permite un pivote 2×2 sin pasar de 16 entre consultas.
La ruta histórica factorize delega con control vacío. Se insertan únicamente
consultas y retornos, sin cambiar operaciones numéricas. Equilibrado, cada
actualización de pivote y solve siguen siendo indivisibles. El dominio y la
fachada conservan API-I-CANCELLED, cancelled / 5 y ausencia de salida parcial.

### Calibración release en esta máquina

Entorno: rustc 1.99.0 (b940084d7 2026-09-28), x86_64-unknown-linux-gnu,
Intel Xeon (frecuencia anunciada 2400 MHz); perfil release Cargo habitual,
sin flags numéricos adicionales ni red. PATH antepone stable; se exportan
CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine-sfc10 y PIP_NO_INDEX=1.

El ejemplo reproducible `crates/engine-api/examples/consolidation_cost.rs`
mide con Instant **todo engine_api::run**, incluida validación, integración y
serialización. No incluye compilación ni arranque del proceso. Parte del
fixture analítico propio de consolidación, con una capa, theta=1, propiedades
explícitas coherentes y cargas 20/21/22 kPa. Argumentos: elementos, intervalos,
máximo de subpasos por intervalo y opciones `same` / `outputs`. Sin same las
duraciones alternan 1.25/1 s, dt distintos: no se descuenta ninguna factorización.
Con outputs se emiten todos los quiebres, incluidos los dos extremos.

Mediciones iniciales antes de r3: n=793, un intervalo/un subpaso, **0.602436 s**;
n=128, 238 intervalos distintos/un subpaso, **0.686472 s**; n=64, un
intervalo/100000 subpasos, **0.965755 s**; n=1, 256 intervalos y hasta 390
subpasos por intervalo, **0.008684 s**. Se escogió 500 millones con margen
respecto al objetivo aproximado de 5 s, sin depender de la reutilización.
El caso n=128/238 intervalos inicial supera ligeramente el nuevo tope y ahora
se rechaza; la calibración final usa 236 intervalos.

Tres corridas por caso con el código final; mínimos/máximos del tiempo total:

| n | Intervalos | Subpasos totales | Coste estimado | Tiempo release (s) |
| ---: | ---: | ---: | ---: | ---: |
| 793 | 1 | 2 | 499_934_955 | 0.528–0.629 |
| 100 | 1 | 49_900 | **500_000_000** | 1.078–1.131 |
| 70 | 1 | 100_000 | 490_343_000 | 1.075–1.090 |
| 200 | 1 | 12_299 | 499_960_000 | 1.055–1.072 |
| 400 | 1 | 2_725 | **500_000_000** | 0.989–1.017 |
| 512 | 1 | 1_394 | 499_646_464 | 1.265–**1.702** |
| 128 | 236 | 236 | 498_794_496 | 0.765–1.190 |
| 124 | 256 | 256 | 492_032_000 | 0.448–0.581 |
| 124, dt idéntico | 256 | 256 | 492_032_000 | 0.015–0.017 |
| 1 | 1 | 100_000 | 100_001 | 0.008–0.009 |

Los dos casos n=124 incluyen **257 estados/32125 valores nodales**: máximo de
intervalos y serialización repetida. Los otros solo publican los extremos.
Parte del barrido coincidió con compilación clippy del workspace; se registra
el máximo observado con esa contención, sin descartar muestras. El peor caso
**medido** aceptado fue n=512/1394 subpasos, 1.702 s. Hay entradas aceptadas
con coste exactamente igual al tope (n=100/49900 y n=400/2725); el mayor orden
aceptado es n=793, dos subpasos, 0.629 s. Un tercer subpaso con n=793 ya se
rechaza. Estos extremos ejercitan factorización, solves, cambios de dt,
historia máxima y salida repetida; no se afirma enumeración de toda entrada
física ni garantía temporal en otra máquina, WASM o debug.

Reproducción (exportar PATH, CARGO_TARGET_DIR y PIP_NO_INDEX como arriba):

```bash
cargo build --release --offline -p engine-api --example consolidation_cost
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 793 1 2
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 100 1 49900
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 70 1 100000
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 200 1 12299
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 400 1 2725
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 512 1 1394
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 128 236 1
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 124 256 1 outputs
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 124 256 1 same outputs
"$CARGO_TARGET_DIR/release/examples/consolidation_cost" 1 1 100000
cargo test --release --offline -p engine-api --test sfc010 \
  -p engine-geo --test consolidation -p engine-linalg --test robustness
```

### Comprobaciones r3

Artefactos locales: CARGO_TARGET_DIR/mam835-r3-artifacts. Captura release del
estado inicial de los **27 fixtures** válidos API/determinismo y comparación
final: **27/27 documentos completos idénticos byte a byte**, sin excluir campos.
El capturador temporal se eliminó. Las 52 pruebas enfocadas release pasan.
Siete pruebas nuevas: cuatro de presupuesto API, dos de dominio y una de
control de factorización. Se conserva la prueba r2 del tope de trabajo y su
redondeo, ajustando su expectativa cuando r3 añade un rechazo independiente.
Las entradas 2048 elementos con 1 intervalo/1 subpaso, 24 intervalos/1 subpaso
y 1 intervalo/24000 subpasos se rechazan en menos de 2 s con el código estable,
aunque cumplen r2. Se prueba cancelación antes de factorizar 2048 elementos y
antes/durante bloques de columnas; el caso n=793 aceptado exige ≤5 s en release.

| Comprobación final offline | Resultado |
| --- | --- |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline | **733 passed, 0 failed, 0 ignored**, 112 suites, incluidas pruebas de documentación |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | **Clean-room: OK** |
| Guardia clean-room incluyendo el nuevo ejemplo | **OK**, índice y objetos temporales en CARGO_TARGET_DIR; índice real intacto |
| Pruebas enfocadas API/geo/linalg, release | **52 passed, 0 failed** |
| Captura anterior frente a r3 | **27/27 documentos completos idénticos** |
| all_sixteen_published_hashes_remain_unchanged | **16/16 hashes idénticos**, sin modificar docs/DETERMINISMO.md |

Logs: fmt.log, clippy.log, test-workspace.log, doc.log, wasm-check.log,
clean-room.log, clean-room-all-files.log y targeted-release.log. Medidas:
baseline-timings.log y final-timings.log. Las comprobaciones pedidas están
completas; no queda implementación pendiente de r3.

Incidencias de esta ronda: la búsqueda inicial `find .. -name AGENTS.md`
enumeró nombres fuera del worktree; no se abrió contenido de otros repos ni
rutas prohibidas y las siguientes búsquedas se limitaron al worktree. La
primera captura consideraba solo Ok; se repitió admitiendo también Warning,
que ya es válido para un fixture anterior. Las primeras pruebas nuevas
intentaban un elemento con drenaje doble; se corrigió a dos elementos según
el contrato existente, sin cambiar el dominio.

No se modifican clean-room/**, docs/CLEAN_ROOM.md, .github/workflows,
CODEOWNERS, vendor/**, Cargo.toml ni Cargo.lock. Sin unsafe nuevo ni
unwrap/expect/panic en bibliotecas; sin commits, staging, cambios de rama ni
red. Pendiente externo: revisión clean-room **J4**; ENG-VAL-006 conserva su
pendiente previo de validación de métodos Experimental.
