# Bitácora clean-room · ENG-GEO-003

- Ticket / PR: MAM-46 · ENG-GEO-003; PR Mamut1195/mamut-engine#168 (commit base 97d34a2a4c88bda0070157d8a38ce099e6153dd8).
- Autor(es): Codex gpt-6.1-sol high, 2026-10-20 (America/Guayaquil).
- Rol: implementador. Especificador: J2, resumen de ecuaciones aportado en el ticket.
- Contrato implementado: generadores de tablas SI propios `py.matlock_soft_clay`,
  `py.reese_sand`, `py.user`; consumo por interpolación ENG-MAT-002 y futuro ENG-GEO-005.
- Fuentes permitidas usadas: resumen J2 de Matlock, H. (1970), «Correlations
  for Design of Laterally Loaded Piles in Soft Clay», Offshore Technology
  Conference, Houston, **OTC 1204, edición 1970**, Ec. 1–5 y curva estática
  de Fig. 5a descrita en la ficha; Reese, L. C., Cox, W. R., Koop, F. D.
  (1974), «Analysis of Laterally Loaded Piles in Sand», Offshore Technology
  Conference, **OTC 2080, edición 1974**, Ec. 4–9 y 11–16, pasos 2 y 4–8.
  No se consultaron los papers por red; se implementó la ficha del especificador,
  sin reproducir prosa, tablas ni figuras de las publicaciones.
- Derivaciones propias: apartados de esta bitácora, reducción afín,
  intersección determinista, evaluación normalizada y fixtures SI.
- Código de terceros copiado: ninguno. Sin nuevas dependencias externas.
- Validación: analítica propia a partir de las ecuaciones citadas; tablas Rust
  compartidas por nativo y wasm, con `source.kind = analytical` y cita explícita.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
  Tampoco consulté código ni resultados de CalculiX, LAPACK ni otros motores
  prohibidos. Sesión sin consultas externas ni uso de red; no hay datos de E3S.
- Revisor clean-room: **PENDIENTE (J4)**.

## API y unidades

Se añade `engine_geo::py`, sin modificar las ecuaciones ni parámetros de otros
métodos. Los parámetros usan `Option<Real>` para representar ausencia y producen
`EngineError`: ENG-E-0003 ausente, ENG-E-0010 dominio/degeneración/rango inválido,
ENG-E-0008 no finito/desbordamiento. No hay defaults ni unwrap/expect/panic/unsafe
nuevos en biblioteca. Las tablas se validan también por finitud de pendientes,
orden estricto y monotonía, rechazando pérdida de pendientes positivas por
subdesbordamiento. Se limita además el rango bilateral a diferencias finitas.

Las entradas usan m, N, Pa, N/m³; salida p en N/m, y en m. x es positiva hacia
abajo y d=b es el diámetro. `effective_stress_pa(&dyn StressProfile, x)` consulta
el perfil (incluido `StratifiedProfile`) sin extrapolación, convierte kPa a Pa
por 1000 y rechaza σ'v negativa. El perfil puede representar tensión efectiva
negativa, pero el dominio de estos generadores exige σ'v≥0. La procedencia
Measured/Assumed se conserva consultando el reporte del propio perfil; el
adaptador no declara como medidos los datos externos. c, ε50, J, geometría y los
parámetros de arena siempre se aportan explícitamente: el perfil actual carece
de dichos campos. Matlock acepta σ'v variable, sin imponer γ'x a un perfil.
Reese requiere γ' constante explícita: no se sustituye por σ'v/x ni se inventa
una extensión estratificada de la expresión homogénea para Xt.

`Sampling` exige un extremo de meseta y permite una lista explícita (incluso
vacía) de y positivos, finitos, estrictamente crecientes y dentro del extremo.
Se añaden origen, vértices obligatorios, meseta y reflejos impares; solo estos
vértices internos se ordenan y deduplican. El extremo debe superar estrictamente
8yc o yu; ningún punto del usuario se descarta. No se ofrece un n predeterminado.
La aproximación entre puntos es secante; el usuario controla su refinamiento.
Matlock tiene derivada infinita en cero; la tabla usa la secante al primer y>0.
La rama de Matlock incluye 8yc; la de Reese incluye yk, ym y yu (en x=0, sin yk).

`py.user` recibe una rama y≥0 que ya debe incluir (0,0), exige al menos dos
puntos, p≥0 no decreciente y refleja sin modificar sus extremos. `PyCurve::at`
interpola solo en el intervalo tabulado, devolviendo error fuera. Las tablas
Matlock/Reese tienen mesetas explícitas. `MultilinearElastic` prolonga los
tramos extremos por su propio contrato: para `py.user` el consumidor debe
aceptar explícitamente esa política o conservar el evaluador acotado. No se
inyecta una meseta ni se extrapola silenciosamente desde el generador.

La ley existente interpreta ε adimensional y σ [Pa]. El ensayo de integración
usa L=2 m y área S=0.5 m² explícitos: ε=y/L, σ=pL/S, p=σS/L. Su tangente se
recupera como dp/dy=(dσ/dε)S/L². Las tuplas pueden pasar directamente como
interpolador numérico, pero el futuro pilote debe conservar las unidades y
aplicar su propia longitud tributaria. No se incluye pilote, barra axial ni
truss. No se exponen en engine-api/engine-input: falta el contrato físico de
ENG-GEO-005 y añadir campos a modelos sin consumidor no aporta integración.
La extensión JSON queda pendiente junto al pilote, sin bloquear este crate.

## Parámetros a cargo del usuario

- Matlock: x≥0, d>0, c>0, σ'v≥0, J>0 y ε50>0, todos finitos y obligatorios.
  Los J=0.5/0.25 y εc=0.005–0.020 mencionados en OTC 1204 son orientación,
  nunca defaults ni límites universales de calibración.
- Reese: x≥0, diámetro>0, 0<φ<π/2, γ'>0, 0<α<π/2, K0>0, A>B>0, k>0.
  α=φ/2 y K0=0.4 se documentan como propuestas del paso 2 del método;
  se exigen explícitos incluso en x=0. β y Ka sí se calculan por Ec. 6/7.
  A y B solo están en Fig. 8/9 y k se proporciona en tabla del paper:
  no se copian ni digitalizan; el usuario aporta estos parámetros.
  Se exige además n>1, yk<ym y una intersección Xt positiva única bajo los
  criterios descritos debajo. Se devuelven errores para otros dominios.
- También son obligatorios los puntos/extremo de muestreo; no se supone una
  deformación final, tolerancia empírica ni factor de suelo ausente.

## Matlock: derivación analítica

Caso inventado en SI, no ejemplo publicado: d=1 m, c=20000 Pa, γ'=10000 N/m³,
J=0.5, ε50=0.01, σ'v=γ'x. Ec. 3 da Np=3+x (x expresado en m para este caso);
Ec. 2 limita a 9. Para x=2 m: Np=5 y Ec. 1 da pu=100000 N/m.
Ec. 5 da yc=2.5·0.01·1=0.025 m. La estática de Fig. 5a permite:

- y=yc/8=0.003125 m: p=0.5pu·(1/8)^(1/3)=25000 N/m.
- y=yc=0.025 m: p=50000 N/m.
- y=8yc=0.2 m: p=pu=100000 N/m, continuo con la meseta.
- Extremo explícito y=0.4 m: p=pu. Se reflejan los signos para y negativo.

Ec. 4: xr=6·1/(10000·1/20000+0.5)=6 m. Ec. 3 en x=6 da Np=9,
en x=5 da 8 y en x=7 da 10 antes del límite (9 después). El fixture compara
los tres puntos y el control independiente xr=6. En x=0, Np=3: no se impone
una curva nula a la arcilla, porque Ec. 1/3 conservan resistencia superficial.

## Reese: derivación analítica

Caso inventado para simplificar radicales: diámetro=1 m, γ'=10000 N/m³,
φ=π/6, **α=π/6 explícito** (distinto de la propuesta φ/2), K0=0.4, A=1,
B=0.5. No es un ejemplo del paper ni calibración recomendada. Se ejerce la
libertad de α parametrizado; la conformidad también acepta α=φ/2 explícito.
β=π/3, tanβ=√3, tanφ=tanα=1/√3, sinβ=cosα=√3/2,
tan(β−φ)=1/√3, Ka=1/3 (Ec. 6/7).

Reduciendo Ec. 5 por γ'H para H>0:

q(H)=q1 H+q0,
q1=K0·tanφ·sinβ/[tan(β−φ)cosα] + tan²β·tanα/tan(β−φ)
   +K0·tanβ·(tanφ·sinβ−tanα)=3+0.2√3,
q0=diámetro·[tanβ/tan(β−φ)−Ka]=8/3 m.

El corchete de Ec. 5 es afín en H: cada término interior tiene unidades de m;
pct es cuadrático en H, con unidades N/m. Ec. 8 por γ'H da
D=(80/3)+(6/5)√3=28.74512763574932 m. La ecuación para la raíz no trivial
(paso 4) es (3+√3/5)Xt−[24+(6/5)√3]=0. Su raíz cerrada independiente es
Xt=[24+(6/5)√3]/[3+√3/5]=7.792966107085287 m.

En general la implementación exige q1>0, q0>0 y D−q0>0. Entonces
f(H)=q1 H−(D−q0) crece para H>0, f(0)<0 y tiene una raíz positiva única.
Se busca una cota superior empezando en diámetro y duplicando hasta
128 veces; se hacen 96 bisecciones en orden fijo. Se exige ancho final
≤8·EPSILON·cota superior (precisión relativa binary64); se rechaza una raíz
que no se resuelva con ese presupuesto. Si no se encuentra cota o
se desborda un intermedio, se devuelve error tipado. No se usa una tolerancia
física supuesta ni la raíz trivial H=0; toda rama no soportada es explícita.
Para H≤Xt se toma pct y para H>Xt pcd. Son continuas en la intersección.

Para H=2 m, Ec. 5 y 8 dan, calculados desde los radicales:

pct=20000·[26/3+(2/5)√3]=187189.73979388434 N/m,
pcd=20000·[80/3+(6/5)√3]=574902.5527149864 N/m.

H<Xt, luego pc=pct. Ec. 9/11: pu=pc y pm=pc/2.
Pasos 6/7: yu=3/80=0.0375 m, ym=1/60 m y yu−ym=1/48 m.
Ec. 13 da m=24pc [N/m²]; Ec. 14 da n=(pc/2)/(24pc/60)=5/4.
Ec. 15 da C=(pc/2)/(1/60)^(4/5). Ec. 12 se evalúa como
pm·(y/ym)^(1/n), equivalente sin potencias de magnitudes dimensionales.

Para este caso se **elige** k=60pc por metro cuadrado =
11231384.38763306 N/m³ (valor sintético, sin usar la tabla del paper).
Así kH=120pc [N/m²]. De Ec. 16, normalizada:
yk=ym·[pm/(kHym)]^[n/(n−1)]=(1/60)·(1/4)^5=1/61440 m.
Es menor que ym. Los controles k/m/u son:

| Punto | y [m] | p [N/m] |
| --- | --- | --- |
| k | 0.000016276041666666666 | pc/512 = 365.60496053493034 |
| m | 0.016666666666666666 | pc/2 = 93594.86989694217 |
| u | 0.0375 | pc = 187189.73979388434 |
| meseta | 0.4 | pc = 187189.73979388434 |

Controles interiores independientes: p(yk/2)=pc/1024=182.80248026746517 N/m;
p(ym/2)=(pc/2)·2^(−4/5)=53756.13654338948 N/m;
p((ym+yu)/2)=3pc/4=140392.30484541325 N/m. Se evalúan como muestras,
para distinguir la parábola analítica de su aproximación multilineal.

Para H=10 m: pct=100000·[98/3+2√3]=3613076.828180442 N/m,
pcd=100000·[80/3+(6/5)√3]=2874512.763574932 N/m. H>Xt, por tanto
pu=pcd=2874512.763574932 N/m. Se conserva el mismo k explícito.
Un control intermedio H=4<Xt da pct=40000·[44/3+(4/5)√3]
=642092.2925088706 N/m, que también debe ser pu (A=1).

Control adicional del α propuesto: α=φ/2=π/12, H=2 y los otros datos iguales.
cosα=(√6+√2)/4 y tanα=2−√3. Los cuatro términos del corchete de Ec. 5 son,
respectivamente, 0.7172603777344427, 5.7846096908265245,
0.3215390309173471 y −1/3 m. Su suma por γ'H=20000 N/m² da
pct=129801.51532289962 N/m. La pendiente de q(H) es
1.9117045497391574 y Xt=13.64147036875595 m; son controles separados del
test de conformidad, derivados de la fórmula sin la reducción del generador.

Ramas degeneradas: A≤B o n≤1 son errores; yk≥ym también es error, sin
sustituir la curva por otra. Comparaciones exactas sobre f64, sin tolerancia
que acepte una degeneración. En x=0 se validan todos los parámetros y n por
la forma independiente de pc: n=[B/(A−B)]·[(yu−ym)/ym]. Ec. 5/8 producen
pct=pcd=0: se devuelve tabla nula, yk=None, sin dividir por kx. Las fórmulas
normalizadas de n y yk son derivaciones algebraicas propias de Ec. 13–16.

## Validación y alcance

Los tres fixtures en `crates/engine-validation/tests/support/py_table.rs`
tienen fuente analítica, cita y controles literales derivados arriba. Comparten
runner nativo/wasm, sin imports JS ni npm. Tolerancia A=1e-8 en la unidad SI de
cada control y R=1e-12: margen de redondeo de radicales/libm/bisección, sin error
de discretización en los vértices. No procede de un motor externo.
Se usa exclusivamente `engine_core::math` para trascendentes deterministas.

Se añade solo una dependencia de desarrollo local engine-laws a
engine-validation para verificar el consumo dimensional de las tablas; Cargo
actualiza la arista correspondiente en el lockfile. La guardia escalar admite
la ruta exacta del nuevo runner wasm con no_mangle y extern C, igual que los
runners existentes; no admite unsafe ni amplía permisos por patrón.
No se modifican clean-room/**, docs/CLEAN_ROOM.md, workflows, CODEOWNERS ni vendor.

La carga cíclica queda fuera (ENG-GEO-009). La revisión clean-room de J4 y la
integración física del pilote/JSON (ENG-GEO-005) quedan pendientes.
