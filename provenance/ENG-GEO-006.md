# Bitácora clean-room · ENG-GEO-006

- Ticket: MAM-48 · ENG-GEO-006, «Consolidación 1D multicapa».
- Autor: Codex (GPT-6), 2026-10-15, America/Guayaquil.
- Rol: especificador e implementador en sesión limpia; sin rol de operador de
  oráculo ni auditor; sin delegación a otros agentes.
- Contrato propio: `consolidation/1`, API Rust de engine-geo, no element/1.
- Fuentes utilizadas: derivaciones propias reproducidas a continuación; código
  interno de engine-geo (perfil y procedencia), engine-core (catálogo y math),
  engine-linalg (CSR y factorización existente) y patrón de fixtures/WASM de
  engine-validation; docs/CLEAN_ROOM.md y provenance/TEMPLATE.md.
- Referencia publicada conceptual: Karl Terzaghi (1943), *Theoretical Soil
  Mechanics*, primera edición, John Wiley & Sons, New York, teoría de
  consolidación unidimensional. No se consultó una copia del libro en esta
  sesión sin red ni se atribuyen páginas/números de ecuación no verificados.
  La separación de variables, convolución y todos los datos de fixtures son
  propios, derivados aquí; no se transcribe ningún ejemplo ni listado del libro.
- Código de terceros copiado: ninguno. Dependencia nueva exclusivamente path a
  engine-linalg; sin paquetes externos nuevos ni cambios de versiones externas.
- Validación: soluciones analíticas propias, invariantes y refinamiento; los
  mismos siete fixtures y tablas de convergencia en nativo y WASM/Node
  tras la revisión r32 (seis en la entrega inicial).
- Comparación con oráculo externo: no; sin red ni consulta a otros motores.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room independiente: pendiente de asignación; no se simula una
  revisión humana en esta entrega.

## Modelo, unidades y datos obligatorios

Profundidad z positiva hacia abajo, origen en la cara superior, t en segundos,
Δσ y exceso u en kPa (compresión positiva). s en m, positivo hacia abajo.
Capas saturadas, deformaciones pequeñas, esqueleto lineal, agua incompresible,
flujo Darcy 1D y sobrecarga espacialmente uniforme. No se calcula presión
hidrostática: puede sumarse aparte usando el perfil geostático existente.
Los espesores se pueden trasladar de Layer, pero no se exigen γ, γsat ni zw
que no intervienen en la ecuación incremental. No se modificó StressProfile.

Cada capa exige espesor >0 en m, cv>0 en m²/s, mv>0 en 1/kPa y k>0 en m/s;
γw>0 en kN/m³ es global y requerido. Todos estos datos, incluso espesor y cada
ordenada de carga, usan Parameter con Provenance Measured/Assumed. No hay
valores físicos predeterminados; cv, mv, k y γw ausentes dan ENG-E-0003.
Se comprueba cv=(k/γw)/mv con tolerancia relativa 1e-10, una tolerancia de
consistencia de datos redundantes, no una calibración física. Un conjunto
incompatible da ENG-E-0010: nunca se elige silenciosamente uno de los tres.
Se permite descarga y tensión negativa; no se recortan resultados.

Assumed se reporta con rutas inequívocas de todos los parámetros: primero γw,
luego espesor/cv/mv/k por capa y finalmente las ordenadas de la historia entera,
incluso aquellas posteriores a la última salida. La tabla completa se valida
antes de integrar. θ y controles de malla/tiempo son opciones numéricas, sin
Provenance física. Crank–Nicolson θ=0.5 es el único valor predeterminado; el
paso máximo siempre se aporta explícitamente.

## Derivación conservativa y elementos finitos

Sea ε=mv(Δσ−u). La continuidad de volumen y Darcy q=−(k/γw)u_z dan

```text
mv u_t − ∂z[(k/γw)u_z] = mv Δσ_t
cv = k/(γw mv)
```

Para una capa homogénea esto es u_t=cv u_zz+Δσ_t. En una interfaz sin
almacenamiento superficial, u es continuo y (k/γw)u_z es continuo.
Integrando por partes con test v continuo, nulo en caras drenantes:

```text
∫ mv v u_t dz + ∫ (k/γw) v_z u_z dz = ∫ mv v Δσ_t dz
```

Los términos internos de frontera se cancelan por continuidad del flujo.
Las caras no drenantes tienen flujo cero natural, las drenantes u=0 esencial.
Se admiten dren superior, inferior o ambos; no se impone simetría artificial.
Cada capa tiene su propio número de elementos, sin elementos que crucen una
interfaz. Los nodos de interfaz son únicos, no se duplican ni se promedian k.

Con ξ en [0,1], N=[1−ξ,ξ], B=[−1/h,1/h], integrar monomios exactamente da

```text
Me = mv h/6 [2 1; 1 2]
Ke = (k/γw)/h [1 −1; −1 1]
fe = mv h/2 [1;1]
```

Se ensamblan M y K reducidas como CSR con Triplet/SparseMatrix de
engine-linalg, en orden capa/elemento/fila/columna. El vector f se integra
antes de eliminar los drenes: f_f NO es M_ff·1 cuando hay nodos prescritos.
La presión del dren siempre es cero. No hay HashMap, hilos ni regularización.

```text
(M + θ Δt K) u[n+1] = (M − (1−θ) Δt K) u[n] + f (Δσ[n+1]−Δσ[n])
```

La historia empieza en t=0 y tiene al menos dos abscisas estrictamente
crecientes. Δσ(t) interpola linealmente sin extrapolar. Δσ(0) representa una
carga instantánea sobre el estado inicial en reposo: nodos libres u=Δσ(0),
drenes u=0. No se añade un parámetro de presión inicial independiente.
Por esta interpolación, s(0) tiene un error de borde O(h); no se presenta
como un asiento inicial exacto. Para tiempos positivos fijos la convergencia
espacial observada es O(h²). CN puede oscilar al inicio si Δt/h² es grande;
se conserva la solución del esquema, sin amortiguación ni recorte oculto.
θ=1 permite al llamador elegir amortiguación de altas frecuencias.

El integrador llega exactamente a todos los quiebres de carga y salidas.
Cada intervalo se divide en ceil(longitud/max_step_s) subpasos iguales;
se factoriza una vez por intervalo con LinAlgBackend::factorize y se reutiliza
el factor para todas sus RHS. solve usa ReferenceBackend; solve_with_backend
admite el contrato existente. La revisión inicial confirmó que engine-linalg
NO contiene un solver de banda/disperso operativo: tiene CSR y factorización
simétrica densa de referencia; faer es un stub. Se usa esa factorización, sin
escribir otro solver ni afirmar escalabilidad dispersa. Coste de referencia:
O(n³) por factor, O(n²) por solve y memoria O(n²).

Para θ<0.5, un modo λ del par (K,M) exige Δt λ≤2/(1−2θ).
Gershgorin y el cociente de Rayleigh dan las cotas suficientes

```text
a = min_i (Mii − Σj≠i |Mij|) > 0
b = max_i Σj |Kij| > 0
λmax ≤ b/a; por tanto Δt ≤ 2a/[(1−2θ)b]
```

La masa lineal consistente es estrictamente diagonal dominante incluso con
capas distintas y tras eliminar drenes. Se comprueba esta cota para cada
intervalo con θ<0.5; es conservadora, no una búsqueda de λmax exacto.
θ≥0.5 no tiene esta restricción de estabilidad lineal.

## Salidas e integración

Se devuelven todas las profundidades nodales y u(z_i,t_j). pressure_at
interpola el campo FE a cualquier profundidad del perfil, sin extrapolar;
el tiempo se selecciona por índice de salida. Para cada elemento:

```text
s_e = mv h [Δσ − (u_i+u_j)/2]
s = Σ s_e
s∞(t) = Δσ(t) Σ mv h
U(t) = s(t)/s∞(t)
```

Éste es el grado medio por asiento, ponderado por compresibilidad. En suelo
homogéneo coincide con 1−∫u dz/(H Δσ). En multicapa no se sustituye por una
media aritmética sin mv. Para Δσ=0, U=None aunque haya presión residual;
s sigue definido. Durante descarga U puede quedar fuera de [0,1].

Límites explícitos del backend de referencia: 2048 elementos, hasta 100000
puntos de historia/salida, 8000000 valores nodales de salida y 1000000
subpasos por intervalo. Se rechazan tamaños y acumulaciones no representables.
ENG-E-0010 por rangos, geometría, incompatibilidad, estabilidad y límites;
ENG-E-0008 por NaN/infinito/desbordamiento; ENG-E-0003 por datos ausentes.
Los errores del solver conservan código y entidades dentro del Diagnostic
con mensaje de catálogo y contexto en hint. Sin panic!, unwrap ni expect en
el módulo nuevo de biblioteca; forbid(unsafe_code) se hereda del crate.

## Serie de Terzaghi: derivación y fixtures

Con dren en x=0 y cara impermeable x=Hdr, separar u=X(x)T(t) da
X=sin(Mx/Hdr), M=π(2m+1)/2, T=exp(−M²Tv), Tv=cv t/Hdr².
La proyección de la condición inicial uniforme Δσ en esos senos tiene
coeficiente 2Δσ/M, puesto que ∫sin(Mx/Hdr)dx=Hdr/M y
∫sin²(Mx/Hdr)dx=Hdr/2. Integrar el campo sobre el espesor da

```text
u(x,t)/Δσ = Σ (2/M) sin(M x/Hdr) exp(−M² Tv)
ū(t)/Δσ = Σ (2/M²) exp(−M² Tv)
U(Tv) = 1 − Σ (2/M²) exp(−M² Tv)
```

En doble drenaje Hdr=H/2 y se refleja cada semiespesor. En dren inferior
se usa x=H−z. No cambia Tv por un factor cuatro sin cambiar Hdr.
Para una rampa 0→Δσ en R (r=cv R/Hdr²), se convoluciona la respuesta
al escalón. Cada exponencial de la suma se reemplaza por

```text
A(M,Tv,r) = exp[−M²(Tv−min(Tv,r))] · (1−exp[−M² min(Tv,r)])/(M² r)
```

Se evalúa 1−exp(−a) con −engine_core::math::exp_m1(−a). Descarga lineal
posterior es la resta de una rampa desplazada; no se crea un salto oculto.
Los oráculos usan 2048 términos, math::sin/cos/exp/exp_m1 del núcleo.
Para escalones y Tv≥.01, la cola exponencial es despreciable en f64.
En rampa, la cola de la media está acotada por
2/r Σm≥N 1/M⁴ ≤ 2/[3r π⁴(N−1/2)³], <5e-12 con r=.2,N=2048;
para la presión, |sin|≤1 da cota 1/[r π³(N−1/2)²]<4e-8.
Estos límites están por debajo de las tolerancias discretas.

Tabla versionada: engine-validation/tests/support/consolidation_table.rs.
Todos los fixtures contienen source y citation con edición. H=2 m,
cv=.25 m²/s, mv=.002 /kPa, γw=10 kN/m³, k=.005 m/s, Δσ=100 kPa.
Son datos analíticos sencillos, sin pretensión de representar un suelo real.
Malla de referencia: 64 elementos para dren simple, 128 para doble;
ΔTv máximo=.0005. Salidas Tv=.01,.05,.1,.2,.4,.6,1.

| Fixture | Historia y solución |
| --- | --- |
| Superior | Escalón constante; serie con Hdr=2 |
| Inferior | Reflejo del superior, misma U y s |
| Doble | Escalón, Hdr=1, idéntica U(Tv) |
| Rampa | Rampa hasta Tv=.2 y meseta; convolución analítica |
| Rampa/descarga | Rampa hasta .2, meseta hasta .4, descarga lineal hasta .6, cero hasta 1 |
| Multicapa | Dos capas transformadas, misma serie y flujo interfacial no nulo |

La aceptación exige error RELATIVO de U <.005, error absoluto de presión
normalizado por 100 kPa <.004 y error de asiento normalizado por .4 m <.0015.
En Δσ=0 se comprueba U=None y presión/asiento residual contra convolución.
Además se comparan todos los nodos u(z,t), no sólo U.

### Multicapa con flujo no nulo: derivación propia

Tomar x=∫dz/√cv y a=mv√cv constante. Entonces
mv dz=a dx y (k/γw)u_z=mv√cv u_x=a u_x; la forma débil se convierte
en a∫v u_t dx+a∫v_x u_x dx=a∫v Δσ_t dx. Es difusión homogénea con
coeficiente 1 en x, conservando presión, flujo y asiento, incluso en interfaz.

Capa 1: H1=1 m, cv1=.25, mv1=.002, k1=.005.
Capa 2: H2=2 m, cv2=1, mv2=.001, k2=.01.
γw=10; a=.001 en ambas, longitudes transformadas 2+2=4.
Dren superior, Tv=t/16, 32 elementos por capa; el dominio transformado
es exactamente una malla uniforme de 64 elementos. ∫mv dz=.004.
La interfaz está en x/Hx=.5 y tiene u_x no nulo: no se valida sólo un plano
de simetría con flujo cero. A Tv=.2 se comparan ambos flujos elementales
−(k/γw)Δu/h con la derivada de la serie; cada uno debe diferir <3%.
Los flujos de derivada elemental convergen a ambos lados; la continuidad
exacta es débil/nodal, no se promete igualdad puntual de pendientes P1.

### Revisión J4 r32: multicapa no transformable por autofunciones

Referencia general publicada: H. S. Carslaw y J. C. Jaeger (1959),
*Conduction of Heat in Solids*, segunda edición, Oxford University Press:
difusión en medios compuestos y separación de variables. La ecuación de calor
con capacidad y conductividad por tramos es el mismo problema de difusión.
No se consultó una copia en esta sesión sin red; no se atribuyen páginas ni
fórmulas concretas al libro. Todo el desarrollo siguiente y su código son
propios; no se copian listados, valores ni ejemplos de terceros.

Fixture versionado en `engine-validation/tests/support/consolidation_table.rs`
(`MULTILAYER`, `MULTILAYER_SOURCE`, `MULTILAYER_CITATION`), con referencia
espectral en `support/consolidation_multilayer.rs`. Dren superior u(0,t)=0,
base impermeable u_z(3,t)=0, escalón uniforme de 100 kPa, γw=10 kN/m³:

| Capa | H (m) | cv (m²/s) | mv (1/kPa) | k (m/s) | a=mv√cv |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | .5 | .25 | .002 | .005 | .001 |
| 2 | 1.5 | 2.25 | .001 | .0225 | .0015 |
| 3 | 1 | 1 | .003 | .03 | .003 |

Las tres razones k/mv son 2.5, 22.5 y 10. Las longitudes H/√cv son 1,
pero las impedancias a son distintas. Un cambio de coordenada x=f(z) que
homogeneice el coeficiente de difusión exige f' proporcional a 1/√cv;
el coeficiente de flujo resultante es proporcional a mv√cv, discontinuo.
Por tanto no se elimina la heterogeneidad mediante un cambio de escala de z:
las condiciones de transmisión siguen siendo no homogéneas en x.

Separar u=X(z)exp(−λt) en mv u_t=(D u_z)_z, D=k/γw, da en cada capa
X''+(λ/cv)X=0. Sea ω=√λ y ξ=(z−z_superior)/√cv en [0,1]. Escribir
X_i=A_i cos(ωξ)+B_i sin(ωξ). Entonces p=D X_z=a_i X_ξ. Integrar la
conservación en una interfaz sin almacenamiento exige continuidad de p;
la presión también es continua. Como γw es común, esto equivale a
continuidad de k X_z. Con c=cos ω, s=sin ω, las dos condiciones dan

```text
A_1=0, B_1=1                    (dren superior y normalización)
A_{i+1}=A_i c+B_i s
B_{i+1}=(a_i/a_{i+1})(−A_i s+B_i c)
```

Equivalentemente, el estado (X,q), q=p/ω, se propaga en cada capa mediante
la matriz [[c,s/a_i],[−a_i s,c]]. Empezando en (0,a_1), el flujo en la base
es q_base=a_1 c[c²−B s²], donde
B=a_2/a_1+a_3/a_1+a_3/a_2=6.5. La base impermeable exige q_base=0.
No hay modo λ=0: la solución lineal estacionaria con dren y base impermeable
es nula. El espectro positivo completo es, para n=0,1,...,

```text
α=atan(1/√6.5)
ω = nπ+α, nπ+π/2, nπ+π−α
λ = ω²
```

Son las tres raíces simples por cada intervalo de longitud π. El operador
es autoadjunto con peso mv: al restar las ecuaciones de dos modos, los
extremos y términos de interfaz se cancelan precisamente por continuidad
de X y p. Los modos son ortogonales en ∫mv X_n X_m dz. La proyección de
u(z,0)=100 sobre este sistema da b_n=100 I_n/N_n, con
I_n=∫mv X_n dz, N_n=∫mv X_n² dz. Como mv dz=a_i dξ, ambas integrales
se evalúan sin malla ni cuadratura, sumando por capa:

```text
I_i = a_i [A_i sin ω+B_i(1−cos ω)]/ω
N_i = a_i { A_i²[1/2+sin(2ω)/(4ω)]
          + B_i²[1/2−sin(2ω)/(4ω)] + A_i B_i sin²ω/ω }
u(z,t) = Σ 100 I_n/N_n X_n(z) exp(−ω_n²t)
s(t) = .55 − Σ 100 I_n²/N_n exp(−ω_n²t)
U(t) = s(t)/.55
k u_z = Σ 100 I_n/N_n · 10 a_i ω_n
        [−A_i sin(ω_n ξ)+B_i cos(ω_n ξ)] exp(−ω_n²t)
```

∫mv dz=.0055 m/kPa y s∞=.55 m. El oráculo no usa matrices FE, backend
lineal, paso temporal ni integración espacial de la implementación. Sólo
evalúa estas expresiones cerradas con `engine_core::math`.

Se incluyen 16 grupos de tres raíces (48 modos). Para los omitidos,
ω≥16π, |X|≤1 porque cada transmisión rota (A,B) y contrae su segunda
componente (a_i/a_{i+1}≤1). Así |I|≤.0055 y
N≥.001[1/2−1/(4ω)]>.000495. Tanto cada coeficiente de u/100 como cada
coeficiente de presión media ponderada están acotados por .0055/.000495.
Para t≥.1, la cola de cualquiera de esas dos magnitudes es menor que

```text
3(.0055/.000495) exp[−.1(16π)²] / {1−exp[−.1·33π²]} < 1e−107.
```

Esta cota es de truncamiento en aritmética exacta; no afirma error de
redondeo de ese orden. Se comprueba para los 48 modos la norma positiva,
ambas condiciones de frontera y continuidad de X y k X_z en las dos
interfaces (tolerancia absoluta 1e−12 en las unidades del modo).

FE: 32 elementos por capa (96 en total), Crank–Nicolson y Δt máximo .002 s;
salidas t=.1,.5,1,3,10,30 s. Se comparan todos los nodos, incluidas ambas
interfaces. Tolerancias fijadas como en los fixtures anteriores:
|u_FE−u_serie|/100<.004, |s_FE−s_serie|/.55<.0015 y error relativo U<.005.
A t=1 s ambas interfaces tienen |k u_z|>.01 kPa·m/s; cada pendiente
elemental de ambos lados se compara con el flujo analítico común con error
relativo <.03. No se exige igualdad puntual entre pendientes P1.

Máximos medidos en nativo: presión/100=.0004286818;
asiento/.55=.0000667471; relativo U=.0010288205;
relativo flujo=.0181946330. Estas medidas verifican las tolerancias;
la referencia se obtiene de las ecuaciones, sin ajustar al solver.
El harness compartido ejecuta ahora siete fixtures en nativo y WASM.

### Revisión r32: discrepancia de cv intermedio y prueba de mutación

`engine-geo/tests/consolidation.rs` añade
`middle_layer_cv_discrepancy_is_rejected_without_substitution`. Primero
acepta las tres capas compatibles de la tabla anterior; después cambia
únicamente cv de `layers[1]` por factores 1e−1 y 10, manteniendo k y mv.
Exige ENG-E-0010 y contexto `layers[1]`/`cv incompatible`. Esto comprueba
que el dato redundante no se sustituye silenciosamente por k/(γw mv).

Comprobación efectiva, con `CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6`
exportado, usando este comando antes, durante y después de la mutación:

```sh
cargo test --offline -p engine-geo --test consolidation \
  middle_layer_cv_discrepancy_is_rejected_without_substitution -- --exact
```

1. Original: 1 passed, 0 failed.
2. Se insertó temporalmente en `ConsolidationProfile::new`, inmediatamente
   después de leer cv y antes de leer mv:
   `let cv = if i == 1 { cv * 1e-1 } else { cv };`.
   El test falló (exit 101, 0 passed, 1 failed) en la aceptación del perfil
   compatible: ENG-E-0010, `layers[1]: cv incompatible con k/(γw mv)`.
3. Se restauró el archivo original byte a byte mediante `finally` de un
   script local Python; el mismo test volvió a pasar.
4. Control adicional: se excluyó temporalmente `i==1` de la comprobación de
   discrepancia (`if i != 1 && ...`). El test falló (exit 101) en
   `expect_err` al aceptar el cv intermedio incompatible. Se restauró otra
   vez el archivo byte a byte y el test pasó.

Ambas mutaciones se eliminaron; `src/consolidation.rs` no tiene diff.
No se cambió la biblioteca ni su política de errores. Sin fuentes externas,
sin red, sin nuevas dependencias y sin delegación; se mantiene la declaración
clean-room inicial.

### Revisión r34: frontera de tolerancia relativa de cv

`engine-geo/tests/consolidation.rs` añade
`cv_consistency_relative_tolerance_boundary`: acepta la razón `1+5e-11` y
rechaza `1+2e-10` con ENG-E-0010 y contexto `layers[0]`/`cv incompatible`.

Construcción exacta dentro de f64: k=.0078125=2⁻⁷ m/s,
γw=8=2³ kN/m³ y mv=.0009765625=2⁻¹⁰ /kPa. Las divisiones
`k/γw=2⁻¹⁰` y `(k/γw)/mv=1` son exactas. Se asigna `cv=1.0+δ`, por
lo que `cv/cv_derivado` conserva exactamente los bits del resultado f64 de
`1.0+δ`, sin redondeo adicional de los parámetros. El test comprueba esos
bits, el cv derivado igual a uno y el lado de la tolerancia de cada caso.
Los decimales matemáticos no son representables exactamente en binario;
estos son sus valores redondeados a f64 y sus discrepancias efectivas:

| δ | Bits de la razón f64 `1.0+δ` | `razón−1.0` en f64 | Resultado |
| --- | --- | --- | --- |
| 5e-11 | `0x3ff0000000036f9c` | 5.000000413701855e-11 | Aceptado |
| 2e-10 | `0x3ff00000000dbe70` | 2.000000165480742e-10 | Rechazado |

Prueba efectiva de las cuatro mutaciones, sin red y con
`export CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6`:

```sh
cargo test --offline -p engine-geo --test consolidation \
  cv_consistency_relative_tolerance_boundary -- --exact
```

Solo se cambió temporalmente el literal de comparación `> 1e-10` en
`ConsolidationProfile::new`; el mensaje de diagnóstico quedó intacto.

| Tolerancia de biblioteca | Exit | Resultado del test |
| --- | ---: | --- |
| 1e-10 original | 0 | 1 passed, 0 failed |
| 1e-1 mutada | 101 | 0 passed, 1 failed |
| 1e-3 mutada | 101 | 0 passed, 1 failed |
| 1e-6 mutada | 101 | 0 passed, 1 failed |
| 1e-9 mutada | 101 | 0 passed, 1 failed |
| 1e-10 restaurada | 0 | 1 passed, 0 failed |

Cada mutación falla en `expect_err("razón 1+2e-10 debe rechazarse")`:
el constructor acepta indebidamente el perfil. El caso `1+5e-11`, ejecutado
primero, sigue pasando. Un script Python restauró los bytes originales en
`finally` después de cada mutación y al salir; se comprobó igualdad byte a
byte antes de repetir el test original. No queda ninguna mutación ni diff
en la biblioteca. Sin fuentes externas, dependencias nuevas ni delegación.

Verificación de r34 (2026-10-15), con el mismo `CARGO_TARGET_DIR` exportado
y `CARGO_NET_OFFLINE=true`, toda aprobada:

- `cargo fmt --all -- --check`.
- `/home/box/.cargo/bin/cargo clippy --offline --workspace --all-targets -- -D warnings`.
- `cargo test --offline --workspace --no-fail-fast`: 557 pruebas aprobadas,
  0 fallidas, 0 ignoradas, en 87 targets de prueba/doctest.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --workspace --no-deps`.
- `/home/box/.cargo/bin/cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown`.
- `bash clean-room/check.sh origin/main`: `Clean-room: OK` sobre archivos
  ya conocidos por Git; no requiere índice temporal para este cambio.
- `git diff --check`: aprobado; biblioteca sin panic!, unwrap ni expect.

Nativo y rustdoc con Cargo/Rust 1.85.1; clippy y WASM con 1.99.0 ya instalado.
Solo cambian el test de integración y este provenance; no se tocaron
docs/CLEAN_ROOM.md, .github/**, clean-room/** ni vendor/**. Sin red y sin
`git add` ni commit; el índice permanece intacto para J2.

## Convergencia y resultados medidos

Serie a Tv=.2: U=0.50408782020255 aproximadamente. Se fija ΔTv=.00025
para refinamiento espacial, dren superior y mallas uniformes anidadas.

| Elementos | ΔTv máx. | U FE | Error absoluto frente a serie |
| ---: | ---: | ---: | ---: |
| 8 | .00025 | .508227050481 | .004139230278 |
| 16 | .00025 | .505126887757 | .001039067555 |
| 32 | .00025 | .504347873894 | .000260053691 |
| 64 | .00125 | .504153341943 | .000065521741 |
| .00025 | .504152865634 | .000065045431 |

Cocientes cercanos a cuatro: convergencia espacial de segundo orden para
este observable a tiempo positivo. El harness exige disminución estricta.

Para tiempo se fija n=64. La referencia temporal es la misma discretización
con ΔTv=.00025, no se confunde con la solución continua. CN y el escalón
inicial pueden retener altas frecuencias en pasos grandes; el error total
frente a la serie puede cambiar de signo al compensarse con error espacial.
El harness exige que |U(ΔTv)−U(.00025)| disminuya estrictamente.

| ΔTv máx. | U FE | Error absoluto frente a serie |
| ---: | ---: | ---: |
| .02 | .497450256297 | .006637563906 |
| .01 | .502321203644 | .001766616559 |
| .005 | .503898123046 | .000189697157 |
| .0025 | .504148482059 | .000060661856 |
| .00125 | .504153341943 | .000065521741 |
| .00025 | .504152865634 | .000065045431 |

En malla de referencia, sobre todos los tiempos de los seis fixtures:

| Fixture | Máximo error relativo U (%) | Máximo error absoluto u/100 kPa (%) |
| --- | ---: | ---: |
| Dren superior | .25941147 | .14919098 |
| Dren inferior | .25941147 | .14919098 |
| Doble drenaje | .25941147 | .14919098 |
| Rampa | .14477007 | .00368148 |
| Rampa y descarga | .14477007 | .00407868 |
| Multicapa con flujo | .25941147 | .14919098 |

## Verificación de entrega inicial (anterior a r32)

Las pruebas de engine-geo incluyen ecuaciones de un elemento derivadas a mano
para θ=0,.25,.5,.75,1 (masa consistente y vector de carga completo), ambos
drenajes simples, quiebres de carga sin salidas en ellos, carga cero, presión
interpolada, toda la procedencia Assumed, datos faltantes en capas profundas,
materiales incompatibles, geometría y dimensiones extremas, θ inestable,
historias inválidas, ausencia de extrapolación, repetición bit a bit y asiento
asintótico con doble drenaje.

El ejemplo examples/consolidation_wasm.rs comparte tabla y harness completos
con el test nativo: exporta consolidation_benchmarks()=6 si pasan también las
tablas de convergencia. scripts/run-consolidation-wasm.cjs verifica ausencia
de imports externos y la devolución de seis. No requiere paquetes ni red.

Verificaciones finales, todas aprobadas, sin red (`--offline`) y con el
`CARGO_TARGET_DIR` exportado `/workspace/.cargo-target/mamut-engine`:

- `cargo fmt --all -- --check`.
- `/home/box/.cargo/bin/cargo clippy --offline --workspace --all-targets -- -D warnings`.
- `cargo test --offline --workspace --no-fail-fast`: 555 pruebas aprobadas,
  0 fallidas, 0 ignoradas, en 87 targets de prueba/doctest. Incluye las nueve
  pruebas nuevas de API y el harness de seis fixtures con convergencia.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --workspace --no-deps`.
- `/home/box/.cargo/bin/cargo check --offline --workspace --all-targets
  --target wasm32-unknown-unknown`: incluye biblioteca, tests y ejemplos;
  satisface también el check de biblioteca del workspace solicitado.
- `/home/box/.cargo/bin/cargo build --offline --release -p engine-validation
  --example consolidation_wasm --target wasm32-unknown-unknown`.
- `node crates/engine-validation/scripts/run-consolidation-wasm.cjs
  /workspace/.cargo-target/mamut-engine/wasm32-unknown-unknown/release/examples/consolidation_wasm.wasm`:
  seis fixtures analíticos y convergencia aprobados, sin imports externos.
- `bash clean-room/check.sh origin/main`: aprobado con todos los archivos
  nuevos en el índice, para revisar el diff completo.
- `git diff --check` y `git diff --cached --check`: aprobados.
- Inspección del nuevo módulo: sin panic!, unwrap, expect, HashMap ni hilos.
  No se modificaron docs/CLEAN_ROOM.md, .github/**, clean-room/** ni vendor/**.

Nativo y rustdoc con Cargo/Rust 1.85.1 del sistema (MSRV del workspace);
clippy y wasm con Cargo/Rust 1.99.0 previamente instalado en
/home/box/.cargo/bin. No hubo descargas ni dependencias externas nuevas.
La ejecución WASM cubre estos seis fixtures y ambas tablas de convergencia;
no se afirma ejecución de las otras suites en Node ni igualdad binaria entre
arquitecturas. El backend denso de referencia y la interpolación inicial
siguen teniendo los límites numéricos y de tamaño descritos arriba.

Durante la verificación se corrigieron dos incidencias locales: clippy pidió
simplificar una multiplicación por −1 en el harness; la guardia de determinismo
exigió `use engine_core::math;` separado, rechazando el import agrupado aunque
las llamadas ya usaban el módulo central. No se modificó ninguna guardia;
se repitieron fmt, clippy, toda la suite, doc y los checks/build/runtime WASM
con resultado satisfactorio tras esas correcciones.

## Verificación final de r32 (2026-10-15)

Todos los comandos Cargo se ejecutaron sin red y con
`export CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6`:

- `cargo fmt --all -- --check`: aprobado.
- `/home/box/.cargo/bin/cargo clippy --offline --workspace --all-targets -- -D warnings`:
  aprobado, sin warnings.
- `cargo test --offline --workspace --no-fail-fast`: 556 pruebas aprobadas,
  0 fallidas, 0 ignoradas, en 87 targets de prueba/doctest. Incluye los siete
  fixtures analíticos y el nuevo test de discrepancia intermedia.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --workspace --no-deps`:
  aprobado.
- `/home/box/.cargo/bin/cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown`:
  aprobado.
- `/home/box/.cargo/bin/cargo build --offline --release -p engine-validation --example consolidation_wasm --target wasm32-unknown-unknown`:
  aprobado.
- `node crates/engine-validation/scripts/run-consolidation-wasm.cjs /workspace/.cargo-target/me-geo6/wasm32-unknown-unknown/release/examples/consolidation_wasm.wasm`:
  siete fixtures analíticos y convergencia aprobados, sin imports externos.
- `bash clean-room/check.sh origin/main`: aprobado sobre el diff completo,
  incluyendo el archivo nuevo mediante índice temporal en `/tmp` (el índice
  original del worktree está montado como solo lectura en esta sesión).
- `git diff --check` y `git diff --cached --check` con ese índice: aprobados.
- El módulo de biblioteca sigue sin panic!, unwrap ni expect y sin cambios.
  No se tocaron docs/CLEAN_ROOM.md, .github/**, clean-room/** ni vendor/**.

Nativo y rustdoc: Cargo/Rust 1.85.1; clippy y WASM: Cargo/Rust 1.99.0 ya
instalado, igual que en la entrega inicial. No se descargaron dependencias.
