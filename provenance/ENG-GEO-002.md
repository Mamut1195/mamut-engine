# Bitácora clean-room · ENG-GEO-002

- Ticket: MAM-44 · ENG-GEO-002, «Viga sobre lecho elástico (Winkler) 2D/3D».
- Autor: Codex gpt-6.1-sol high bajo J2; 2026-10-14 (America/Guayaquil).
- Rol: especificador e implementador, sesión limpia; sin rol de operador ni auditor.
- Contrato: element/1; kinds `frame2d.winkler` y `frame3d.winkler`.
- Fuentes usadas: derivaciones propias desarrolladas abajo; contratos y código
  interno de frame2d/frame3d, engine-assembly, engine-linalg y engine-validation;
  docs/CLEAN_ROOM.md, docs/CONVENCION_EJES.md y provenance/TEMPLATE.md.
- Referencias conceptuales, sin consultar sus textos ni código: E. Winkler (1867),
  *Die Lehre von der Elasticitaet und Festigkeit*, H. Dominicus, Praga;
  M. Hetényi (1946), *Beams on Elastic Foundation*, University of Michigan Press,
  Ann Arbor. Se citan por el modelo físico y el problema clásico; no se atribuye
  una página ni ecuación no verificada. Las ecuaciones de fixtures son propias.
- Código de terceros copiado: ninguno. Sin fuentes externas, red, manuales de
  otros motores, libros con listados ni salidas de oráculos.
- Validación: integración polinómica, energía, equilibrio y soluciones cerradas
  derivadas aquí; mismos fixtures en nativo y wasm. Sin mamut-workflows.
- Comparación con oráculo externo: no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Modelo y unidades

El lecho es bilateral, lineal, homogéneo y fijo a tierra: la fuerza distribuida
sobre la viga es q_b=−k v. k tiene unidades N/m²: k·v es N/m, por longitud
axial de viga. No es presión por área ni requiere ancho implícito. La energía
es Π_b=½∫₀ᴸ k v² dx. No hay masa del suelo, amortiguamiento, contacto unilateral,
acoplamiento de cortante, movimiento prescrito del terreno ni ajuste de k.

Se mantienen los ejes, SI, orientación y GDL de los frames existentes:
D2 en XY, orden [ux,uy,rz] por nodo; D3 [ux,uy,uz,rx,ry,rz].
En 3D orientation es la referencia de z local, y=z×x y x×y=z.
ky resiste v en y local (EI=EIz); kz resiste w en z local (EI=EIy).
Las pendientes son v′=rz y w′=−ry. No hay acoplamiento entre planos,
resistencia axial ni torsional del lecho.

## Derivación propia de la matriz consistente

Sea t=x/L. Interpolo v=N d con d=[v_i,θ_i,v_j,θ_j], θ=v′:

```text
N1=1−3t²+2t³       N2=L(t−2t²+t³)
N3=3t²−2t³         N4=L(−t²+t³)
δΠ_b = δdᵀ ∫ k NᵀN dx d
K_b = k L ∫₀¹ NᵀN dt
```

La regla ∫₀¹ tⁿ dt=1/(n+1) permite integrar los diez productos distintos.
Muestro explícitamente dos expansiones:

```text
N1²=1−6t²+4t³+9t⁴−12t⁵+4t⁶
∫N1²dt=1−2+1+9/5−2+4/7=13/35=156/420
N1 N2/L=t−2t²−2t³+8t⁴−7t⁵+2t⁶
∫N1 N2/L dt=1/2−2/3−1/2+8/5−7/6+2/7=11/210=22/420
```

Los demás productos, incluyendo los signos de N4, dan:

```text
∫ N1 N3 dt=9/70=54/420
∫ N1 N4 dt=−13L/420
∫ N2² dt=L²/105=4L²/420
∫ N2 N3 dt=13L/420
∫ N2 N4 dt=−L²/140=−3L²/420
∫ N3² dt=13/35=156/420
∫ N3 N4 dt=−11L/210=−22L/420
∫ N4² dt=L²/105=4L²/420

           [ 156    22L     54    −13L  ]
K_b=kL/420 [ 22L    4L²    13L    −3L² ]
           [ 54     13L    156    −22L  ]
           [−13L   −3L²   −22L     4L² ]
```

La implementación calcula a=kL/420, b=aL y c=bL para evitar potencias
innecesarias y valida cada coeficiente y entrada. No integra numéricamente
en producción ni usa un conjunto de resortes nodales concentrados.

En 2D inserto K_b en índices [1,2,4,5] de la matriz local 6×6.
En 3D inserto K_b(ky) en [1,5,7,11] y D K_b(kz) D en [2,4,8,10],
D=diag(1,−1,1,−1), por w′=−ry. Con la transformación T de cada frame:

```text
K_global = K_frame,global + Tᵀ K_b,local T
f_int,global = f_frame,global + Tᵀ K_b,local T u_global
```

El lecho no altera masa, Kg ni cargas externas del frame. Es historia vacía,
lineal y determinista. k=0 produce exactamente el frame sin lecho en todas
las operaciones admitidas. Se valida la geometría incluso con k=0.

### Simetría, positividad y modos rígidos

K_b es simétrica por ser Gram. Si k>0 y d≠0, N d no es el polinomio cero
(la interpolación valor/pendiente en extremos es unisolvente); por tanto
 dᵀK_b d=k∫(N d)² dx>0. El bloque transversal 4×4 es definido positivo.
La matriz completa es semidefinida positiva: sus GDL axiales y torsionales
son nulos; con k=0 todo el bloque se anula. No se regularizan esos modos.

Un desplazamiento rígido transversal mueve la viga respecto al suelo fijo y
sí genera reacción. El helper `check_element` exige todos los modos rígidos
sin fuerza y no puede aplicarse literalmente a un elemento apoyado en tierra.
Se precisa esa hipótesis en la documentación de element/1 y se prueban
directamente identidad, estado, tamaños, simetría, FD, repetición, covarianza,
masa y recuperación. No se cambian traits, versiones ni helpers de conformidad.

## Recuperación por equilibrio con reacción distribuida

Las acciones locales de extremo incluyen el lecho consistente:
p=(K_frame+K_b)u−f_ext,equiv. No se reporta K_frame u−f como si el lecho
fuera una carga nodal física. Para v(t)=Σ₀³ a_j tʲ:

```text
a0=vi; a1=Lθi
a2=−3vi−2Lθi+3vj−Lθj; a3=2vi+Lθi−2vj+Lθj
F_b(x)=∫₀ˣ k v(s)ds = kL Σ a_j t^(j+1)/(j+1)
J_b(x)=∫₀ˣ (x−s) k v(s)ds = kL² Σ a_j t^(j+2)/[(j+1)(j+2)]
Vy=−p_vi−qy x+F_b(x)
Mz=−p_rzi+p_vi x+qy x²/2−mz x−J_b(x)
```

En z local se usa d=[wi,−ryi,wj,−ryj],
Vz=−p_wi−qz x+F_b y My=−p_ryi−p_wi x−qz x²/2−my x+J_b.
Así las caras en x=0 y x=L coinciden con los extremos, y V′=−q_ext+k v.
Son esfuerzos equilibrados del campo discreto, no una afirmación de solución
continua exacta dentro de cada elemento. La curvatura EI v_h″ puede diferir
localmente de este esfuerzo equilibrado; la diferencia converge al refinar.

## Oráculos analíticos y cotas

### Viga infinita y semiinfinita

Fuera de la carga EI v''''+k v=0. Con λ=(k/(4EI))^(1/4), las raíces son
±λ±iλ. En x≥0 la solución que decae es
v=e^(−λx)[A cos(λx)+B sin(λx)]. Sus valores iniciales son:

```text
v(0)=A; v′(0)=λ(B−A); v″(0)=−2λ² B; v'''(0)=2λ³(A+B)
```

En viga infinita y carga central P, la simetría impone v′(0)=0, luego B=A.
El salto EI[v''']=P implica 2 EI v'''(0+)=P, de donde
A=P/(8EIλ³)=Pλ/(2k). |M0|=2EIλ²A=P/(4λ).

En semiinfinita cargada en el extremo libre, momento cero impone B=0 y
EI v'''(0)=P (signo de la cara negativa). Entonces
A=P/(2EIλ³)=2Pλ/k. Se usan extremos alejados libres, sin apoyos de flexión.

Los fixtures de viga infinita tienen λL=20, con centro a distancia 10/λ
de cada extremo; los semiinfinitos λL=12. Para cuantificar la truncación,
en una distancia adimensional a, sean c=cos a, s=sin a, d=[A,B] los
coeficientes del campo e^(−xλ)(A cos xλ+B sin xλ), y r=[C,D] los del
campo creciente e^(xλ)(C cos xλ+D sin xλ). Las dos condiciones libres dan:

```text
       [ s     −c   ]          [ −s       c   ]
e^(−a) [ c−s   s+c  ] d + e^a [ −s−c    c−s ] r = 0
```

La segunda matriz tiene determinante 1. Ambas matrices y su inversa tienen
norma infinito ≤2√2; por tanto ||r||∞≤8 e^(−2a)||d||∞. En la infinita,
las condiciones de simetría/salto en el centro dan
A=A0+C, B=A0−D (A0=Pλ/(2k)). En la semiinfinita,
A=A0+C−2D, B=D (A0=2Pλ/k). La norma de la dependencia de d en r es
respectivamente 1 y 3; resolver esa realimentación da denominadores
1−8e^(−2a) y 1−24e^(−2a). La flecha central o terminal difiere de A0
por 2C o 2(C−D), y el momento central por un término proporcional a 2D.
Una cota relativa uniforme conservadora para estas observaciones es
64 e^(−2a)/(1−24 e^(−2a)), menor que 1.4e-7 para a≥10.
No se presenta el truncamiento como error de discretización nulo.

La solución es suave por cada lado de la carga, situada siempre en un nodo.
En subespacios Hermite anidados, la aproximación de Galerkin minimiza la
energía y el error de energía no aumenta. Para la carga puntual,
P[v(0)−v_h(0)]=a(v−v_h,v−v_h)≥0, de modo que la flecha cargada converge
monótonamente al dominio finito. Interpolar una solución suave con Hermite
da error de energía O(h²) y error de esa flecha O(h⁴). No se deduce de esta
identidad una monotonía universal del momento; se comprueba en estos fixtures.

Se refinan 16/32/64 elementos (infinita; λh=1.25/.625/.3125) y
12/24/48 (semi; λh=1/.5/.25). La cota de aceptación relativa 2e-4
incluye truncamiento <1e-6 y discretización Hermite; se verifica además error
estrictamente decreciente en las tres mallas para flecha y, en la infinita,
para momento. La cota es una cota de regresión de estos datos adimensionales,
no una estimación a priori universal para cualquier EI, k, dominio o malla.

### Viga corta rígida

Con extremos libres y carga centrada, integrar la ecuación da ∫k v dx=P,
y la simetría anula la rotación rígida. Cuando EI→∞, v es constante:
v=P/(kL). Para EI finito escribo v=v̄+z, ∫z=0 y v̄=P/(kL).
En el subespacio simétrico v′(0)=0, Cauchy–Schwarz aplicado dos veces a
z(x)−z(0)=∫₀ˣ(x−s)z″(s)ds y a su promedio da
|z(0)|≤C||z″||, C²=L³/320. En efecto, en 0≤x≤a=L/2,
z(0)=−(1/(2a))∫₀ᵃ(a−s)²z″(s)ds y la simetría da
||z″||²=2∫₀ᵃz″²; integrar (a−s)⁴ da ese valor de C².
La ecuación débil probada con z da EI||z″||²+k||z||²=P z(0).
Por tanto |z(0)|≤P L³/(320EI), y el error relativo de flecha central se acota
por ε/320, ε=kL⁴/EI. En los fixtures L=2, EI=[1e5,2e5], k=[3,6],
ε=4.8e-4 y ε/320=1.5e-6. La cota de aceptación 2e-5 añade margen f64.
Se usan 2/4/8 elementos y se verifica el límite en cada última malla.

La suma de reacciones del suelo no se obtiene restando fuerzas nodales
externas: se integra k v_h con Gauss de tres puntos (exacto para el campo
cúbico) y su primer momento (polinomio cuártico). Se exige ΣR=P y
ΣxR=P a con error relativo ≤1e-7 y absoluto 1e-8 en todas las mallas.
El residual libre numérico tiene umbral 1e-8, o 1e-6 en el límite rígido:
EI grande y malla fina amplifican la cancelación de fuerzas flexionales.
Las caras de extremo usan tolerancia absoluta 1e-8 (1e-6 en el límite
rígido) y relativa 1e-9 por cancelación de términos flexionales grandes.
Las sumas aplicadas/reacciones de apoyos del informe genérico no incluyen
un lecho que forma parte de K; se comprueba explícitamente el balance físico
con sus integrales, sin modificar el ensamblador.

## API, fábrica y diagnósticos

Cada crate de frame exporta `WinklerFrame2D`/`WinklerFrame3D` y su
`WinklerFactory`, igual al patrón de registro mediante `ElementFactory`
de `FrameFactory`; no existe registro global central de elementos.
Las fábricas poseen las propiedades y reutilizan la construcción validada
del frame base. Esquemas propios con additionalProperties=false, k o ky/kz
requeridos, mínimo cero y unidades explícitas. No hay nuevas dependencias
externas: solo dependencia path de ambos frames a engine-geo.

Esta versión no admite liberaciones de extremo: se rechazan tanto en fábrica
como en el constructor compuesto. Condensar con la transformación del frame
sin suelo sería incorrecto; soportarlo exigiría condensar K_frame+K_b y
recuperar giros con esa matriz. Se mantiene esa extensión fuera de este ticket.
No se modifica el comportamiento de los frames Euler existentes.

ENG-E-0010 por k negativo, campos/tipos/rangos inválidos, liberaciones y
coeficientes positivos subdesbordados; ENG-E-0008 por NaN/infinito/overflow;
ENG-E-0005 por longitud ≤1e-12; ENG-E-0002 por ids repetidos en ModelView.
Los módulos de biblioteca nuevos no contienen panic!, unwrap ni expect.
No se promete representar todo el rango extremo de f64.

## Fixtures y verificación

`engine-validation/tests/support/winkler_table.rs` contiene seis problemas
con source, citation, datos SI y tolerancias: infinita, semiinfinita y rígida,
cada uno en 2D y 3D. Los espaciales cargan ambos planos con signos opuestos,
rigideces distintas y soluciones analíticas independientes. El harness usa
las fábricas reales, DofMap, LinearSystem y StaticLinear, sin apoyos
transversales artificiales; solo restringe axial/torsión desacoplados sin carga.

Pruebas de elementos: integral de cada producto polinómico independiente,
Cholesky positivo 4×4, inserción local y signo en z, ausencia de acoplamiento,
k=0 en todas las operaciones, FD de todos los GDL, repetibilidad, rotación
completa de geometría/orientación, K/M, modos transversales resistidos,
validaciones de fábrica, historia, geometría y resultados extremos.

El ejemplo `engine-validation/examples/winkler_wasm.rs` reutiliza exactamente
el harness y tabla nativos. Exporta `winkler_benchmarks()` que devuelve seis
si todo pasa, cero ante un error, ejecutable desde Node sin imports externos.

Resultados finales (todos sin red, CARGO_NET_OFFLINE=true o --offline y
CARGO_TARGET_DIR=/workspace/.cargo-target):

- `cargo fmt --all -- --check`: aprobado.
- `cargo clippy --workspace --all-targets -- -D warnings`: aprobado.
- `cargo test --workspace`: 428 pruebas aprobadas incluyendo doctests,
  cero fallidas y cero ignoradas. La ampliación añade siete tests Rust:
  seis de invariantes y uno que recorre seis problemas con tres mallas cada uno.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: aprobado.
- `cargo check --workspace --target wasm32-unknown-unknown`: aprobado.
- Adicional: `cargo check --workspace --all-targets --target
  wasm32-unknown-unknown`: aprobado, incluidos fixtures y pruebas.
- Build release del ejemplo para wasm y ejecución mediante
  `node crates/engine-validation/scripts/run-winkler-wasm.cjs
  /workspace/.cargo-target/wasm32-unknown-unknown/release/examples/winkler_wasm.wasm`:
  seis fixtures aprobados; el script verifica que no existan imports externos.
- `bash clean-room/check.sh origin/main`: aprobado con todos los archivos
  nuevos incorporados al índice, para que el script revise el diff completo.
- `git diff --check` y `git diff --cached --check`: aprobados.
- Inspección de biblioteca: sin panic!, unwrap ni expect añadidos;
  sin cambios a docs/CLEAN_ROOM.md, .github/** ni clean-room/**.

Nativo (tests y rustdoc) con Rust/Cargo 1.85.1 del sistema, para respetar el
MSRV del workspace. Clippy y wasm con stable 1.99.0 instalado, desde
/home/box/.cargo/bin, con target wasm previamente instalado; sin descargas.
La ejecución wasm cubre los seis problemas Winkler, no afirma haber ejecutado
las otras suites en ese runtime ni igualdad binaria entre destinos.

Se hicieron dos mutaciones temporales y se restauró el contenido original
antes de las verificaciones finales:

1. Sustituir K_b por cero manteniendo los tipos/fábricas: falla el benchmark
   por singularidad sin soporte transversal; fallan además cuatro pruebas de
   invariantes (integración/positividad, planos, bloque resoluble y movimiento
   rígido resistido). Ambos targets de prueba terminan con exit 101.
2. Invertir el signo de ∫k v en recuperación: falla la prueba independiente
   por cuadratura con cargas distribuidas (exit 101); las otras cinco
   pruebas de invariantes siguen pasando. Así se detecta retirar/corromper
   la recuperación del suelo aunque se conserve la tangente.

La primera ejecución del fixture rígido usó EI=[1e6,2e6], cuya cancelación
f64 con ocho elementos excedió el umbral de equilibrio relativo 1e-7.
Se eligió EI=[1e5,2e5], todavía 1/ε>2000 respecto al lecho, y se derivó
la cota ε/320 anterior para justificar la comparación sin relajar el balance
físico. Las tolerancias de recuperación de caras separan este redondeo del
error de la flecha. Rustdoc detectó inicialmente `[0,1]` como enlace roto;
se corrigió la documentación y se repitió la verificación satisfactoriamente.

Revisión clean-room independiente: PENDIENTE (J4).
