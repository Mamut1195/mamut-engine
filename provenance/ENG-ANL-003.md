# Bitácora clean-room · ENG-ANL-003

- Ticket / PR: ENG-ANL-003 «P-Delta con detección de inestabilidad» · PR #59.
  Implementación original: commit `5e54ea9` del 2026-10-11. Bitácora añadida
  el 2026-10-13 para su incorporación en un commit aparte; sin git commit ni
  push en esta sesión.
- Autor(es): Codex (gpt-6.1-sol, esfuerzo high) bajo dirección de J2 (agente,
  Grok Bot). Fecha de incorporación de la bitácora: 2026-10-13
  (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: `pdelta`, API `PDelta::run` sobre `LinearSystem`,
  con `PDeltaOptions`, `PDeltaOutcome`, `PDeltaIteration`, `PDeltaResult` y
  adaptador `geometric_producer`. Usa la frontera pública `Element` para Kg
  y `LinAlgBackend` / `FactorInfo` para resolución e inercia. No implementa
  la adaptación al trait genérico `AnalysisMethod` (`analysis/1`).
- Fuentes permitidas usadas: exclusivamente la petición del ticket y las APIs
  y código propios del repositorio: engine-analysis, engine-assembly,
  engine-linalg, engine-core, engine-contracts, engine-model,
  engine-elements-frame y engine-elements-truss. Plantilla:
  `provenance/TEMPLATE.md`; formato: `provenance/ENG-ANL-001.md`.
  Sandbox `workspace-write` sin red; sin fuentes externas, libros, búsquedas
  en red, OpenSees, resultados de otros motores ni oráculos.
- Derivaciones propias: [nota de esta bitácora](#derivaciones-propias) y
  [pruebas analíticas embebidas](../crates/engine-analysis/tests/pdelta.rs).
- Código de terceros copiado: ninguno. Sin dependencias externas nuevas.
- Validación: analítica, soluciones cerradas, refinamiento de malla, inercia
  LDLᵀ, contador de factorizaciones/resoluciones y comprobaciones de errores
  y conservación de estados.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: J4 APTO con seguimiento (2026-10-13, abe0690); procedencia autodeclarada, escrita después del commit.

## Integración con main (#58), 2026-10-13

Se integran los módulos `combination`, `pdelta` y `results`. LDLᵀ conserva
`FactorInfo.inertia: Option<(usize, usize, usize)>` (positivos, negativos,
nulos), la consulta `symmetric_inertia` de main para Sturm/espectro y el
rechazo ENG-E-0006 de factores singulares usados por P-Delta. Se mantienen
los mensajes de error de main y el contador validado de la rama; los bloques
2×2 usan el autovalor de mayor módulo y `det/λ_grande` para evitar cancelación.
Una regresión combina bloques 1×1/2×2 y modos nulos con permutaciones y escalas
extremas, comprobando ambas APIs. Integración local sin red ni git add,
commit o push.

Verificación con `CARGO_NET_OFFLINE=true`: `cargo fmt --all` y su `--check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`
(373 pruebas, incluidas las 13 de P-Delta) y documentación del workspace
con `RUSTDOCFLAGS='-D warnings'`: OK, Rust 1.85.1. `cargo check --workspace
--target wasm32-unknown-unknown`: OK con `stable` 1.99.0 y el destino ya
instalado. La primera pasada focalizada falló en el contador global de heap;
la prueba aislada con/sin captura y la suite completa posterior pasaron.

## Incorporación tardía

Según los hechos aportados para este registro, la implementación original del
commit `5e54ea9` se realizó con el método clean-room descrito arriba. Aquella
sesión terminó con `No space left on device` antes de escribir la bitácora.
Por eso se incorpora ahora, el 2026-10-13, como documentación separada del
commit de implementación. Este registro describe el código y las pruebas
existentes, leídos localmente, y distingue las verificaciones ejecutadas hoy
de la sesión original. No atribuye a aquella sesión los resultados actuales.

## Diseño, convención y límites

El alcance principal es
[pdelta.rs](../crates/engine-analysis/src/pdelta.rs) y sus pruebas. El soporte
en [system.rs](../crates/engine-assembly/src/system.rs) expone
`geometric_at`, `reduced_stiffness` y `factorize_reduced`: conserva los bloques
globales de Kg, incluidos los restringidos, y traduce los mecanismos del
backend a nodo/GDL. `SystemView::geometric` comparte la evaluación global.
En [reference.rs](../crates/engine-linalg/src/reference.rs), el contador de
inercia diferencia signos positivos, negativos y nulos; valida finitud y la
presencia del acumulador, y cuenta correctamente los signos de bloques 2×2.
Los accesos a bloques/permutaciones vacíos devuelven errores explícitos.

Primero se valida el caso lateral y se resuelve la gravedad con `StaticLinear`.
Cada productor evalúa Kg en el desplazamiento de gravedad expandido, incluidos
los prescritos. El adaptador posee coordenadas originales, parámetros e historia
aceptada y delega en `Element::geometric_stiffness`; no estima el axial de un
miembro a partir de la carga total del modelo. Los axiales, ejes y materiales
quedan congelados durante el análisis lateral. Tracción tiene signo positivo;
compresión reduce la rigidez transversal. No hay factores normativos,
actualización de geometría ni análisis de gran deformación.

Con `F` ya escalada por el factor lateral, se forma `Kt = K + Kg` y se comprueba
la inercia de `Kt_ff` antes de iterar. La gravedad y la lateral son casos
independientes: el desplazamiento lateral no incluye el de gravedad y las
reacciones laterales no son las de la combinación gravedad+lateral. Los valores
prescritos `u_p` no se escalan. La iteración comienza con `u_f = 0` y resuelve:

```text
K_ff u_f^(i+1) = F_f − K_fp u_p − (Kg u^(i))_f
u^(i) = expandir(u_f^(i), u_p)
```

En el límite, `(K+Kg)_ff u_f = F_f − (K+Kg)_fp u_p`.
Las reacciones se calculan como `(Kt u − F)_p`. El factor de `Kt_ff` se usa
solo para diagnosticar estabilidad; el punto fijo reutiliza el factor de K_ff.
No se llama a `generalized_eigen`, no se regulariza y no hay reintentos ni
cambio automático de estrategia.

La inercia declarada por el backend debe sumar el orden reducido. Una inercia
con nulos devuelve ENG-E-0006; con negativos y sin nulos devuelve `Unstable`
con el conteo negativo, sin iteraciones, desplazamiento lateral ni reacciones.
El backend de referencia ya rechaza pivotes numéricamente singulares durante
la factorización: su contador de nulos no significa que entregue un factor
singular utilizable. Una diagonal nula en un bloque 2×2 invertible no implica
un modo nulo.

En cada paso se reporta el desplazamiento libre, el cambio relativo
`‖Δu/s‖∞ / max(1, ‖u_nuevo/s‖∞, ‖u_previo/s‖∞)` y el residual libre absoluto
`‖(Kt u − F)_f‖∞`. El diagnóstico histórico `relative_free_residual` divide por
`max(1, ‖(Kt u)_f‖∞, ‖F_f‖∞)`. Las escalas `s` distinguen traslaciones en m
y giros en rad y no cambian las ecuaciones. El residual mezcla componentes
SI N/N·m; es un indicador numérico, no una norma física invariante de unidades.
Desde la corrección de redondeo del 2026-10-15, ese diagnóstico no determina
la parada: `Completed` exige cambio relativo `<= tolerance` y
`residual_convergence_ratio <= 1`, con referencias separadas por tipo de GDL
y piso por fila según la derivación siguiente.
`NotConverged` agota exactamente `max_iterations` y conserva la última prueba.
Ningún resultado acepta ni modifica el estado aceptado/de prueba del sistema.
Una matriz estable puede no converger por punto fijo, especialmente con
tracción grande. La reducción y el backend de referencia son densos para
modelos pequeños; no se promete escalabilidad ni ejecución wasm en estas pruebas.

## Derivaciones propias

### Residual relativo por tipo de GDL y piso de redondeo (2026-10-15)

Para la matriz global almacenada `Kt = K + Kg`, el producto escalar de una
fila se evalúa como suma secuencial de `m_i` productos. Si
`q_i = Σ_j |Kt_ij| |u_j|`, el error absoluto del producto está acotado,
en el modelo usual de redondeo sin overflow/underflow, por
`γ_m q_i`, donde `γ_m = m_i·eps / (1 − m_i·eps)`; usamos
`eps = Real::EPSILON = 2^-52`, conservador frente al redondeo unitario
`eps/2`. La resta de `F_i` añade un término del orden de
`eps·(|(Kt u)_i| + |F_i|)`. Cerca del equilibrio, estas magnitudes son del
orden de `q_i`; la cancelación de los términos con signo reduce el producto
final pero no su error de evaluación. Por eso normalizar solo con el
producto con signo o con la carga externa puede exigir un residual inferior
al redondeo disponible. Una constante absoluta 1 mezcla además fuerza y
momento y depende de las unidades elegidas.

Se definen dos grupos entre los GDL libres: traslaciones (ecuaciones de fuerza)
y giros (ecuaciones de momento). En cada iteración:

```text
q_i       = (|Kt| |u|)_i
ref_g     = max_{j libre del grupo g} (|F_j|, q_j)
piso_i    = 64 · max(1, m_i) · eps · q_i
límite_i  = tolerance · ref_grupo(i) + piso_i
indicador = max_{i libre} |(Kt u − F)_i| / límite_i
```

`m_i` cuenta las entradas CSR de la fila **global**, incluidos los ceros
almacenados. `q_i` incluye todos los desplazamientos, también los prescritos;
solo las filas comprobadas y las referencias de grupo se limitan a GDL
libres. El factor 64 es un margen numérico explícito, no un factor físico ni
normativo: para `m_i·eps << 1` deja varias decenas de veces la escala de
error de la suma, permitiendo el redondeo previo de ensamblaje, recuperación
de Kg y resolución elástica. No constituye una cota universal del error de
cualquier backend ni garantiza error hacia delante en sistemas mal
condicionados. No se estima el piso usando el residual observado ni se
aumenta con el número de iteraciones.

El residual aceptado tampoco equivale directamente al error de desplazamiento:
si `r = Kt u − F` y `u*` es la solución exacta del sistema discreto,
`u_f − u*_f = Kt_ff^-1 r_f`. Cerca del primer modo de pandeo, la rigidez de ese
modo decrece aproximadamente como `1−P/Pcr`; por tanto, el residuo aceptado
equivale a un error de desplazamiento amplificado por `~1/(1−P/Pcr)`.
La tolerancia efectiva en `u` se afloja en ese factor. Es una estimación
modal de condicionamiento, no una cota universal ni una garantía de error
hacia delante; el criterio independiente de cambio de desplazamiento sigue
siendo obligatorio.

En la práctica, el piso de redondeo solo es activo con desplazamientos
prescritos: puede quedar un residual no nulo aun cuando el incremento se
estanca en cero. El estancamiento del W14x48 se resuelve por las referencias
separadas de fuerza y momento basadas en `q_i`, no por el piso. Este último
es proporcional a `q_i`, igual que la referencia; con las tolerancias y
mallas del W14x48, su contribución queda por debajo de `tolerance·ref_g`.

Cada límite conserva las unidades de su ecuación. Escalar coherentemente
fuerzas o momentos escala por igual residual, referencia y piso de ese grupo;
no hay constante absoluta de fuerza ni comparación de fuerzas con momentos.
Un grupo sin carga ni producto absoluto tiene referencia y residual nulos:
`0/0` se define como 0. Si el límite subdesborda a cero con residual no nulo,
el indicador vale `Real::MAX` y no converge. Productos, sumas e indicadores
no finitos siguen siendo ENG-E-0008.

Se publica el indicador en `PDeltaIteration.residual_convergence_ratio`
y en el reporte JSON P-Delta. `free_residual_norm` y
`relative_free_residual` conservan su significado anterior, únicamente como
diagnósticos. El cambio relativo de desplazamiento no cambia: **ambos**
criterios deben pasar, sin actualizar el estado aceptado ni reintentar.
Las comprobaciones antiguas que identificaban el diagnóstico mixto con la
condición de parada se actualizan al nuevo indicador (análisis, validación y
API); se mantienen las comparaciones físicas y el conteo escalar de diez pasos.

La regresión usa el voladizo W14x48 del ticket: E=29000 kip/in²,
A=14.1 in², I=484 in⁴, L=336 in, H=1 kip y P=200 kip de compresión,
EulerFrame2D, escalas 1 in y 0.01 rad. Se consultó como referencia de
construcción del modelo `/tmp/bench_pdelta_ref.rs`, aportado por el usuario;
las pruebas son autocontenidas y no lo incluyen ni ejecutan. Las mallas
32 con tolerancia `1e-10` y 8 con `1e-12` deben completar; ambas deben
devolver `NotConverged` con solo dos pasos. La solución continua de referencia
es `H(tan(kL) − kL)/(Pk)`, `k²=P/(EI)`.

La flecha continua vale 2.5648953977433653 in. La malla de 8 da
2.5648889686672063 in, error relativo `2.5066e-6` por discretización:
no satisface el `1e-6` originalmente solicitado aunque se cierre la iteración.
Se conserva el alcance del criterio de convergencia: la regresión exige
`3e-6` con 8 elementos y `1e-6` con 32, y además contrasta ambas respuestas
iterativas con una resolución directa de la misma matriz reducida `Kt_ff`
(error relativo `< 1e-8`). Este contraste distingue el error iterativo del
error espacial; no se cambia la formulación de EulerFrame2D ni se corrigen
artificialmente las flechas. La diferencia de tolerancia se comunicó al usuario.

Una prueba escalar con prescrito 0.03 y tolerancia `1e-20` verifica que el
incremento llega a cero pero queda residual no nulo: se contrasta el indicador
con el piso calculado explícitamente usando ambos desplazamientos de la fila.
También se verifica referencia cero con carga nula. La prueba de invariancia
resuelve el W14x48 de 8 elementos en kip·in y N·m, con las conversiones exactas
`1 kip = 4448.2216152605 N` y `1 in = 0.0254 m`. Convierte también E, A, I
y la escala de traslación; conserva la escala angular. Con P=200 kip
(`~0.65 Pcr`), comprueba el mismo outcome y número de iteraciones, todos los
desplazamientos globales equivalentes y el indicador adimensional inicial.
Repite con la carga lateral multiplicada por `1e-6` en ambos sistemas,
conservando P y, por tanto, la proximidad al pandeo. Este caso comprueba la
invariancia también con referencias pequeñas, sensibles a introducir un
`max(1.0, ref_g)` absoluto.

Dos resortes Ux/Rz
independientes, alternando una carga de 1 y otra de `1e12`, comprueban que
un grupo equilibrado no oculta el residual pendiente del otro: el cambio y el
diagnóstico mixto pasan, pero el nuevo indicador impide completar.

Los fixtures están embebidos en Rust, sin archivos externos, red ni oráculos.
El voladizo usa E=200 Pa, A=2 m², I=3 m⁴, L=4 m y H=0.1 N.
Los ajustes comunes son tolerancia `1e-10`, máximo 2000 pasos y escalas
`0.01` m y `0.01` rad. Las tolerancias de comparación siguientes son las
declaradas por las pruebas; las absolutas usan las unidades de cada magnitud.

### Voladizo comprimido, carga crítica y amplificación

Para compresión P>0, la ecuación es `EI v'''' + P v'' = 0`, con
`v(0)=v'(0)=0`, `v''(L)=0` y `EI v'''(L)+P v'(L)=−H`.
Sea `k²=P/EI`: integrar y aplicar las condiciones da

```text
v(L) = H/P · [tan(kL)/k − L]
v_lineal(L) = H L³ / (3EI)
Pcr = π²EI / (4L²)
r = P/Pcr, z = kL = π√r/2
v(L)/v_lineal(L) = 3(tan z − z)/z³
```

La primera singularidad aparece en `cos(kL)=0`, es decir `kL=π/2`.
La amplificación `1/(1−P/Pcr)` es aproximada: cerca de `r=1`, la solución
exacta tiene coeficiente del polo `96/π⁴`, no uno.

La prueba usa `r=0.9` y `0.95` y mallas de 1, 2, 4, 8, 16 y 32 elementos.
Exige descenso estricto del error relativo con cada refinamiento. Con 32
elementos exige error relativo `< 5e-7` frente a la solución exacta y `< 0.02`
frente a `v_lineal/(1−r)`. El cambio relativo final debe ser `<= 1e-10`
y el indicador de residual con piso `<= 1`. El axial recuperado de gravedad en cada miembro,
`EA/Δx · (u_x,j−u_x,i)=−P`, se verifica con tolerancia absoluta `1e-10` N.

### Unstable por inercia y bloques 2×2

La congruencia de LDLᵀ conserva la inercia de la matriz simétrica. Para un
bloque `[[d,e],[e,g]]`, los signos se obtienen de
`h=(d+g)/2`, `ρ=hypot((d−g)/2,e)` y `det=dg−e²`.
El código calcula primero `λ_grande=h+copysign(ρ,h)` y después
`λ_pequeño=det/λ_grande`, evitando cancelación al obtener el menor.
El fixture `K_ff=I`, `Kg_ff=[[-1,2],[2,-1]]` produce
`Kt_ff=[[0,2],[2,0]]`: sus signos son positivo y negativo, con inercia exacta
`(1,1,0)` y `Unstable { negative_eigenvalues: 1 }` aunque ambas diagonales
sean cero. Esta comprobación usa igualdad de conteos, sin tolerancia numérica.

Con 16 elementos, `P/Pcr=1.01` y `1.2` dan un negativo; `10` da dos.
Se comprueba que el resultado no contiene iteraciones ni respuesta lateral.
La malla afecta el cruce: a `1.001 Pcr`, un único elemento devuelve
`NotConverged`, mientras ocho dan `Unstable` con un negativo. No se identifica
el umbral continuo de Euler con el de cualquier discretización.

### P=0 equivale al análisis lineal; tracción rigidiza

Con axial nulo, Kg=0 y el punto fijo resuelve las mismas ecuaciones que
`StaticLinear`. La prueba usa cuatro elementos, desplazamiento prescrito
de base `Uy=0.003` m y factor lateral 2. Compara todos los desplazamientos
con tolerancia absoluta `1e-12` m/rad y las reacciones con `1e-10` N/N·m.
El asentamiento no se escala.

Para tracción T>0, `EI v'''' − T v''=0` da, con `k²=T/EI`,
`v(L)=H/T · [L−tanh(kL)/k]`. La prueba toma `T=0.5 Pcr` y 16 elementos,
exige `v(L)<v_lineal(L)` y error relativo `< 2e-7` frente a esa forma cerrada.
Comprueba además `uᵀ Kg u>0`: la contribución geométrica rigidiza.

### Armadura y controles del punto fijo

Armadura simétrica: bases (−4,0), (4,0), ápice (0,3), barras de 5 m,
E=200 Pa, A=2 m², gravedad P=40 N y lateral H=10 N.
Con `c=4/5`, `s=3/5`, equilibrio vertical da `N=−P/(2s)` por barra.
Así `Kxx=2(EA/5)c²=102.4 N/m` y
`Kgxx=2(N/5)s²=−P·0.6/5=−4.8 N/m`.
Se verifica `ux=H/(Kxx+Kgxx)` con tolerancia absoluta `1e-9` m y
`uy=0` con `1e-12` m usando productores existentes de armaduras.

El fixture escalar `K=100`, `Kg=−25`, `F=10` tiene recurrencia
`u_(i+1)=0.1+0.25u_i`, desde cero. Por suma geométrica,
`u_i=(10/75)(1−0.25^i)`, residual relativo `0.25^i` y cambio relativo
`0.1·0.25^(i−1)` con escalas unitarias. Las tres expresiones se comparan
con tolerancia absoluta `1e-15`. Con tolerancia de parada `1e-6` y máximo
20 se exigen exactamente diez pasos, tres factorizaciones (gravedad,
inercia total, K elástica) y once resoluciones (una de gravedad y diez laterales).
Con límite de tres pasos se exige `NotConverged`, sin reintento ni aceptación.
Los estados previamente fijados en 0.7 y 0.8 permanecen iguales.

Con prescrito `s=0.03`, `(100−25)(u2−s)=10` implica
`u2=s+10/75`, `R1=−10`. Las tolerancias absolutas son `1e-15` m para
el prescrito, `1e-10` m para u2 y `1e-8` N para la reacción.
Con `Kg=200`, la matriz total vale 300>0 pero la recurrencia
`u_nuevo=0.1−2u` tiene radio espectral 2: se exige `NotConverged`
tras cuatro pasos y estado aceptado intacto.

## Códigos de error y validación de entradas

- ENG-E-0002: GDL elemental repetido en el adaptador.
- ENG-E-0003: caso de gravedad o lateral ausente.
- ENG-E-0006: mecanismo elástico o rigidez total singular. El backend de
  referencia aporta índice reducido, que el sistema traduce a nodo/GDL;
  la prueba escalar singular verifica nodo 2 y Ux. La rama que recibe nulos
  en `FactorInfo` devuelve directamente el código, sin esa traducción.
- ENG-E-0008: entradas, evaluación de Kg, soluciones, productos, sumas o
  indicadores no finitos, incluidos NaN, ±∞ y desbordamientos.
- ENG-E-0010: tolerancia fuera de `(0,1)`, límite de pasos cero, escalas no
  positivas, tamaños elementales/historia incompatibles o dimensiones inválidas.
- ENG-E-0012: diagnóstico incoherente del backend, conteos de inercia cuya
  suma no coincide con el orden (incluido desbordamiento del conteo), tamaño
  inválido de solución o acumulador LDLᵀ ausente.
- ENG-E-0013: backend sin inercia, elemento sin Kg o reutilización de una
  Kg estática sin productor fuera del estado de referencia declarado.

Las pruebas cubren parámetros inválidos, no finitos, casos ausentes,
desbordamiento del factor lateral y de la normalización del desplazamiento,
Kg no finita y conservación de estados. También comprueban ausencia e
incoherencia de inercia mediante backend instrumentado, tamaños/GDL del
adaptador, consistencia de Kg global/reducida y la protección de referencia
de una Kg estática. `Unstable` y `NotConverged` son resultados del método,
no códigos de error de entrada.

## Verificación local

Ejecutada el 2026-10-13, sin red. Todas las invocaciones de Cargo usaron
`CARGO_NET_OFFLINE=true`. No se descargaron toolchains ni dependencias.

- `cargo fmt --all --check`: OK, salida 0. Se usó comprobación de formato
  para respetar la restricción de no modificar otros archivos.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK, salida 0.
- `cargo test --workspace`: OK, salida 0; 286 pruebas aprobadas en total,
  incluidas las pruebas de documentación y las 13 de integración P-Delta;
  cero fallidas y cero ignoradas.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK, salida 0.
- `cargo check --workspace --target wasm32-unknown-unknown` con Cargo/rustc
  1.85.1 del sistema: FALLÓ, salida 101, E0463 (`can't find crate for std`);
  falta la biblioteca estándar de ese destino para dicho compilador.
- `RUSTUP_TOOLCHAIN=stable PATH=/home/box/.cargo/bin:$PATH cargo check --workspace --target wasm32-unknown-unknown`:
  OK, salida 0, con Cargo/rustc 1.99.0 y el destino wasm ya instalado.
- `bash clean-room/check.sh origin/main`: OK, salida 0 (`Clean-room: OK`).
  El script compara archivos conocidos por Git contra el merge-base local;
  no incluye archivos nuevos sin seguimiento, como esta bitácora al cierre.

### Corrección del piso de redondeo, 2026-10-15

Verificación local sin red, commit ni push, con Cargo/rustc 1.99.0 y
`CARGO_NET_OFFLINE=true`. Se antepuso `/home/box/.cargo/bin` al PATH para
usar el toolchain fijado por el repositorio. Se usó
`CARGO_TARGET_DIR=/tmp/me-conv-target` porque el destino configurado fuera
del workspace era de solo lectura.

- Regresión antes de la corrección: `NotConverged` a los 2000 pasos con
  32 elementos; cambio `4.33e-15`, residual mixto `5.68e-10` (la reproducción
  inicial se ejecutó con el Cargo/rustc 1.85.1 del PATH del sistema).
- `cargo fmt --all`: OK, salida 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK, salida 0.
- `cargo test --workspace`: OK, salida 0; 621 pruebas aprobadas,
  cero fallidas e ignoradas, incluidas las 16 de P-Delta y la guardia de
  determinismo escalar. Registro local: `/tmp/me-conv-workspace-tests.log`.
- El fixture escalar mantiene diez iteraciones, tres factorizaciones y once
  resoluciones. Las comprobaciones del indicador final en análisis,
  validación y API se actualizan porque la nueva parada puede ocurrir antes
  de que el diagnóstico mixto cruce su antigua tolerancia.

Formato, Clippy, pruebas nativas y documentación se ejecutaron con
Cargo/rustc 1.85.1. El chequeo wasm con `stable` verifica compilación, no
ejecución de pruebas en un runtime wasm. La revisión clean-room de J4 sigue
pendiente. El único archivo de trabajo añadido en esta sesión es esta bitácora;
no se modificó código, ni se ejecutaron git commit, push o acceso a red.

### Seguimiento de revisión J4 (2026-10-15)

Codex (GPT-6), rol implementador, usando exclusivamente la petición de revisión
y las APIs, pruebas y derivaciones propias de este checkout. Sin fuentes
externas, red, código de terceros copiado ni dependencias nuevas. Las funciones
escalares añadidas pasan por `engine_core::math`, con
`use engine_core::math;`. No consulté código fuente de OpenSees, Stabileo,
CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer,
nalgebra ni derivados.

- La prueba de invariancia pasa con el criterio actual. Mutación temporal:
  `tolerance * references[group].max(1.0) + floor`. Falla como se esperaba
  con carga lateral `1e-6`: N·m requiere 32 iteraciones y kip·in solo 23.
  Se restauró el criterio sin referencia absoluta antes de las validaciones.
  Registro: `/tmp/me-conv-j4-mutation.log`.
- `cargo fmt --all`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK.
  Registro: `/tmp/me-conv-j4-clippy.log`.
- `cargo test --workspace`: OK; 625 pruebas aprobadas, cero fallidas e
  ignoradas, incluidas las 17 de integración P-Delta y la guardia escalar.
  Registro: `/tmp/me-conv-j4-workspace-tests.log`.

Invocaciones con `CARGO_NET_OFFLINE=true`, Cargo/rustc 1.85.1 y
`CARGO_TARGET_DIR=/workspace/me-conv/target`. `advanced.rs` y
`docs/DETERMINISMO.md` permanecen intactos; no se ejecutó commit ni push.
La tabla siguiente transcribe la evidencia existente; esta revisión no
reejecuta el cruce nativo/wasm.

### Hashes de determinismo tras merge de main (#103)

Al incorporar main (52f0613, 5d0139c), el campo
`residual_convergence_ratio` entra en results/1 de P-Delta y cambia su criterio
de parada. La tabla recoge los 16 casos antes y después; los otros 14
permanecen iguales a [docs/DETERMINISMO.md](../docs/DETERMINISMO.md).
La evidencia de cruce ya registrada es nativo = wasm (Node), en debug y
release, diez corridas por caso.

| Entrada / caso de bits | Antes | Después |
| --- | --- | --- |
| cantilever | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` |
| inclined-cantilever | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` |
| modal-1dof | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` |
| pdelta | `980bf21b60a5b931a0f0c137ebc782a4a878abb95f7e8bce880c91c4374f56ca` | `f2cc535ed32fb2ed071a3ed8048ebf53c2cd81083efe8ba528e5c95d8cf46740` |
| modal-coupled | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` |
| spectrum-log-cqc | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` |
| pdelta-inclined | `9b5a4584e2151bb4d72c34933f564e8642e85600d44662e78ea376a61541a16b` | `70825cf724a03f9b82f1fd21c76175645a78bc4fcaea648504a7bfc6fd3b978b` |
| modal-inclined | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` |
| spectrum-inclined | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` |
| frame3d-inclined-static | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` |
| frame3d-inclined-modal | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` |
| scalar-trig | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` |
| scalar-exp-log | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` |
| scalar-powers-norm | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` |
| load-rotation-bits | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` |
| scalar-sin-cos | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` |
