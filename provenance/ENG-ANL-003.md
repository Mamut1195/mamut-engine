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
`‖(Kt u − F)_f‖∞`. El residual relativo divide por
`max(1, ‖(Kt u)_f‖∞, ‖F_f‖∞)`. Las escalas `s` distinguen traslaciones en m
y giros en rad y no cambian las ecuaciones. El residual mezcla componentes
SI N/N·m; es un indicador numérico, no una norma física invariante de unidades.
`Completed` exige que cambio y residual relativos cumplan ambos la tolerancia;
`NotConverged` agota exactamente `max_iterations` y conserva la última prueba.
Ningún resultado acepta ni modifica el estado aceptado/de prueba del sistema.
Una matriz estable puede no converger por punto fijo, especialmente con
tracción grande. La reducción y el backend de referencia son densos para
modelos pequeños; no se promete escalabilidad ni ejecución wasm en estas pruebas.

## Derivaciones propias

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
frente a `v_lineal/(1−r)`. Ambos indicadores de convergencia finales deben
ser `<= 1e-10`. El axial recuperado de gravedad en cada miembro,
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

Formato, Clippy, pruebas nativas y documentación se ejecutaron con
Cargo/rustc 1.85.1. El chequeo wasm con `stable` verifica compilación, no
ejecución de pruebas en un runtime wasm. La revisión clean-room de J4 sigue
pendiente. El único archivo de trabajo añadido en esta sesión es esta bitácora;
no se modificó código, ni se ejecutaron git commit, push o acceso a red.
