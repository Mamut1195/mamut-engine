# Bitácora clean-room · ENG-SOL-001

- Ticket / PR: ENG-SOL-001 «Newton-Raphson estático con control de carga».
  Rama `j2/eng-sol-001`, apilada sobre ENG-ELM-005 (#48: `spring.nonlinear`,
  incluye #46/#45) y ENG-ANL-001 (#52: `crates/engine-analysis`), sin fusionar.
  Entrega local sin commit ni push.
- Autor(es): J2 (agente, Grok Bot), atribución solicitada por el usuario;
  agente ejecutor Codex (GPT-6), 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de derivaciones y fixtures analíticos propios.
- Contrato implementado: método `static.newton`, API tipada
  `newton::StaticNewton::run` sobre `NewtonSystem<B>`; respuestas `element/1`
  e historia de leyes `uniaxial/1`. Como StaticLinear, esta entrega no adapta
  el método al trait AnalysisMethod (JSON, sink y control cooperativo).
- Fuentes permitidas usadas: únicamente solicitud del usuario, código y
  documentación internos de engine-core, engine-contracts, engine-assembly,
  engine-elements-spring, engine-laws, engine-analysis, engine-model y frontera
  de engine-linalg, incluida la política de errores del backend propio.
  `provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md` y bitácoras ENG-ELM-005 y
  ENG-ANL-001. Sin red, fuentes externas, manuales ni listados de terceros.
- Derivaciones propias: [ecuaciones y soluciones de esta nota](#derivaciones-propias)
  y [pruebas analíticas](../crates/engine-analysis/tests/newton.rs).
- Código de terceros copiado: ninguno. Se reutilizan las leyes y el backend
  propios existentes. Solo dos dev-dependencies internas por path para probar
  el resorte y la ley reales; sin dependencias externas nuevas.
- Validación: exclusivamente analítica, ecuaciones cerradas por tramos y
  backend contador de factorizaciones/resoluciones; ejemplo Rustdoc ejecutable.
- Comparación con oráculo (repo hermano, informativa): no. No se ejecutó
  ningún motor externo ni se consultó ningún oráculo.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: PENDIENTE (J4).

## Diseño y aceptación

NewtonElement posee un Element puro, contexto SI e índices globales. Valida
contrato element/1 con versión menor compatible, tamaños, coordenadas y orden
nodo/GDL frente al DofMap. NewtonSystem conserva el LinearSystem existente y
las historias aceptadas independientes, en orden de elementos variables.
El desplazamiento inicial es el ya aceptado de LinearSystem; las historias
iniciales proceden de Element::initial_state. La evaluación inicial valida la
respuesta pero no acepta su historia de prueba ni factor de carga alguno.

Los elementos variables son **adicionales** a K_lineal: no deben duplicarse
como rigidez de ElementContribution. Esta regla permite combinar elementos
lineales constantes con cualquier Element no lineal de tangente simétrica.
Las cargas equivalentes constantes se declaran en ElementContribution::loads
(puede tener rigidez cero si solo aporta cargas); las nodales, en el modelo.
No se interpreta Kg ni se altera la geometría de referencia desde el análisis.

En cada evaluación, expand(u_f) incorpora u_p constante y gather obtiene u_e
TOTAL. respond(ctx, u_e, historia_aceptada) devuelve fuerza, tangente consistente
e historia de prueba. assemble_vector suma las fuerzas y assemble_matrix suma
las tangentes; tamaños, finitud y simetría exacta se revalidan en cada evaluación.

```text
u = expand(u_f, u_p)
F_int(u) = K_lineal u + Σ scatter(F_e(u_e, historia_aceptada))
K_t(u)   = K_lineal   + Σ scatter(K_e(u_e, historia_aceptada))
R_f      = (λ_k F − F_int(u))_f
K_t,ff Δu_f = R_f
u_f ← u_f + Δu_f
```

La condensación selecciona la partición ff como LinearSystem. Δu_p=0: no se
resta K_fp u_p en el RHS de corrección, porque u_p ya aparece en F_int(u).
Cada corrección requiere exactamente una factorización y una resolución por
backend de engine-linalg. Ensamblar o evaluar la respuesta no factoriza.

Después de actualizar u se reevalúa la respuesta usando **la misma historia
aceptada del paso anterior**, nunca una historia de una iteración intermedia.
Si todas las tolerancias activas se satisfacen, se acepta exactamente u y se
mueven los trial_state validados a las historias aceptadas. Si falla el máximo,
se devuelve Status::NotConverged con el paso fallido reportado, y se detiene la
lista; resultados y sistema conservan u, factor e historias del último paso
aceptado. Si no hubo aceptación previa, conservan u/historias iniciales y factor
None. También los errores descartan pruebas; rollback de LinearSystem deja su
trial_displacement igual a accepted_displacement. No hay reintentos ocultos,
bisección, reducción de paso, búsqueda de línea ni regularización.

Los factores son absolutos sobre UN caso: [0.25, 0.5, 1] termina en F_total.
Se permiten factores repetidos, cero, negativos y descarga explícita. Toda la
lista, sus productos λF y las opciones se validan antes del primer commit.
No se impone λ_final=1: el usuario puede pedir otro objetivo de carga.

## Criterios y diagnósticos

R se mide tras actualizar u. Δu es la última corrección realmente aplicada:

```text
‖R_f‖∞ ≤ tol_f · force_scale
‖Δu_f‖∞ ≤ tol_u · displacement_scale
|Δu_fᵀR_f| ≤ tol_e · energy_scale
```

Cada tolerancia es Option<Real>; None desactiva ese criterio. Al menos uno
debe estar activo, todos los activos deben cumplirse, con comparación inclusiva.
Tolerancias y escalas son finitas y estrictamente positivas; max_iterations>0.
El reporte conserva normas absolutas y relativas, opciones/escalas usadas,
iteraciones realizadas, factor y convergencia. Las escalas SI son explícitas,
constantes, por defecto 1; no hay normalización oculta ni cambio entre iteraciones.
Las normas mixtas SI son indicadores numéricos; no se afirma invariancia física
al mezclar fuerzas/momentos ni traslaciones/giros.

Por defecto solo se activa fuerza (1e−10), máximo 25. Un problema lineal
converge en una corrección con fuerza sola (también puede satisfacer energía).
Si se exige además desplazamiento pequeño, puede requerir otra corrección:
no se hace una resolución oculta para estimar una corrección futura.
Solo fuerza/solo desplazamiento/solo energía pueden autorizar aceptación;
no se añaden comprobaciones de fuerza cuando el usuario las ha desactivado.

ENG-E-0006: tangente singular, con traducción a nodo, GDL e índice global como
LinearSystem; no se estabiliza un gap abierto sin otra rigidez física.
ENG-E-0008: no finitos de entradas, fuerzas, tangentes, historias, correcciones,
productos/sumas de trabajo y normas relativas. ENG-E-0010: parámetros,
escalas, tamaños/contexto/contrato inválidos. Se propagan casos ausentes
ENG-E-0003, duplicados ENG-E-0002 y diagnóstico incoherente de backend
ENG-E-0012 como las APIs existentes. No se añaden panic!, unwrap ni expect
al código de biblioteca. No cambia ningún contrato existente.

## Derivaciones propias

Todos los fixtures usan SI artificial, datos embebidos, sin red ni E/S durante
las pruebas. Comparaciones analíticas absolutas <1e−9; residual por defecto
≤1e−10. Las pruebas de igualdad de tolerancia usan números binarios exactos.

1. **Bilineal a tierra:** puntos (δ,F)=(-.5,-18),(-.1,-10),(0,0),(.1,10),(.5,18).
   F=100δ para |δ|≤.1 y F=sign(δ)[10+20(|δ|−.1)] fuera.
   Para P=16, desde cero: u1=.16, F1=11.2, R1=4.8;
   Δu2=4.8/20=.24, u2=.4 y F2=16. Exactamente dos factorizaciones y solves.
   Para factores .25,.5,1 se aceptan u=.04,.08,.4 en 1,1,2 correcciones.
2. **Serie:** primer resorte k=100/20 con vértice (.1,10), segundo k=200/40
   con vértice (.1,20). Equilibrio F1=F2=P. δ1=P/100 si P≤10, o
   .1+(P−10)/20; δ2=P/200 si P≤20, o .1+(P−20)/40. u1=δ1 y u2=δ1+δ2.
   P=4,12,24 da (δ1,δ2)=(.04,.02),(.2,.06),(.8,.2).
3. **Gap solo-compresión que se cierra:** puntos (-1.25,-100),(-.25,0),(0,0),
   modo CompressionOnly. F_gap=100(u+.25) para u<−.25, cero fuera.
   Resorte lineal físico k=10 en paralelo; carga −20. Desde abierto u=0,
   u1=−2; cerrado, 10u+100(u+.25)=−20 → u=−45/110 en dos correcciones.
   El mismo gap sin resorte paralelo es singular y produce ENG-E-0006.
4. **Lineal y prescritos:** resorte entre nodos k=100, u1=.03, nodo 2 libre,
   carga nodal 12 y equivalente 3, factor 2. u2=.03+2(12+3)/100=.33.
   Converge en una corrección e iguala StaticLinear, sin escalar u1.
   El resorte no lineal entre nodos con u1=.03 y P=16 da u2=.43.
5. **Historia y rollback:** ley propia de prueba con k=100, límite |F−Hp|=10,
   H=25. Predictor ξ=100(u−p)−25p; si |ξ|>10, p_trial=p+sign(ξ)(|ξ|−10)/125.
   F=100(u−p_trial), tangente kH/(k+H)=20; rama elástica k=100.
   Con max_iter=1 y factores .25,1,2: acepta u=.04,p=0; prueba u=.16,p=.048
   y R=4.8, Δu=.12, trabajo=.576; devuelve el primer aceptado, dos
   factorizaciones/solves totales y no intenta factor 2. Otra ejecución
   acepta P=16,u=.4,p=.24; fallar después conserva esa historia no vacía.
6. **Criterios independientes:** primera corrección del bilineal desde cero
   da (‖R‖∞,‖Δu‖∞,|ΔuᵀR|)=(4.8,.16,.768). Se activan por separado con
   escala 2 y tolerancias que pasan o fallan; cada criterio activo puede
   impedir aceptar aunque los otros dos pasen. Los tres estrictos requieren
   tres correcciones (la última es numéricamente cero). Backend a media
   solución con P=8,k=2 produce Δu=2,R=4,trabajo=8: igualdad inclusiva exacta.
7. **Sin estrategia oculta:** desde u=.4 descargar directamente a P=0 oscila
   entre ±.4 con tangente 20. El ensayo exige cinco correcciones y cinco
   factorizaciones, un único paso fallido λ=0 y el aceptado u=.4 intacto.
   Una descarga con factores intermedios EXPLÍCITOS sí converge.
8. **Errores:** se prueban opciones/factores/casos inválidos, overflow λF,
   backend no finito y ley que entrega historia no finita después de aceptar
   u=.04. El error conserva ese desplazamiento, su historia y factor .25.
   Ubicaciones fuera de rango, tamaños y coordenadas inconsistentes se
   rechazan antes de resolver.

## Verificación local

Verificación del 2026-10-11, sin red, con CARGO_NET_OFFLINE=true:

- `cargo fmt` y `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, incluidas las 16 pruebas nuevas y el doctest
  de uso de StaticNewton con la ley y el resorte reales.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK usando el
  toolchain stable ya instalado de rustup, con `/home/box/.cargo/bin` al inicio
  de PATH; sin descargas. Compilación wasm, no ejecución en runtime wasm.
- `bash clean-room/check.sh origin/main`: OK; también se verifica el diff
  incluyendo archivos nuevos con índice y objetos Git temporales en `/tmp`,
  sin modificar el índice ni los objetos reales del usuario.
- `git diff --check`: OK, también incluyendo archivos nuevos en el índice
  temporal. Sin panic!, unwrap ni expect en engine-analysis/src ni en las
  tres operaciones nuevas de engine-assembly.

Se modifica engine-analysis (módulo, exportación, ejemplo y pruebas), se añaden
las tres operaciones compatibles de LinearSystem y dos dependencias internas
de prueba; Cargo.lock solo registra esas dos relaciones entre crates existentes.
Se amplía la declaración de ENG-ELM-005; ENG-ANL-001 ya contiene la declaración,
autor y revisor solicitados. No se modifican leyes, elementos, contratos ni
reglas clean-room. Sin dependencias externas nuevas, commit ni push.
La revisión clean-room de J4 sigue PENDIENTE.
