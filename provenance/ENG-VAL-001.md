# Bitácora clean-room · ENG-VAL-001 (parcial, apilado)

- Ticket / PR: ENG-VAL-001 «Suite de benchmarks estructurales V1», parcial:
  estática lineal, modal, P-Delta y espectro. Rama apilada sobre ENG-ANL-004
  (#62), ENG-ANL-002 (#61), ENG-MASS-001 (#55), ENG-LOAD-001 (#51), con
  ENG-ANL-003 (#59) y la entrega lineal (#57/#54/#53/#52). Sin commit ni push.
- Autor(es): J2 (agente, Grok Bot) · identificación solicitada por el usuario;
  agente ejecutor Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: crate de desarrollo `engine-validation`, un test lineal
  tabulado sobre `StaticLinear`, `element/1` (truss/frame),
  `engine_analysis::results` y `engine_analysis::combination` de LOAD-002;
  tres tests tabulados adicionales sobre `Modal`, `PDelta` y `Spectrum`.
- Fuentes permitidas usadas: exclusivamente la petición, APIs y pruebas propias
  del workspace y `provenance/TEMPLATE.md`. Sin red, fuentes externas, libros,
  páginas, ejemplos numéricos publicados, fuentes de otros motores ni oráculos.
- Derivaciones propias: las notas cerradas de esta bitácora; datos en
  [tabla de fixtures](../crates/engine-validation/tests/support/table.rs),
  [comprobaciones](../crates/engine-validation/tests/analytical.rs) y
  [montaje](../crates/engine-validation/tests/support/harness.rs).
- Código de terceros copiado: ninguno. Sin dependencias externas nuevas;
  exclusivamente `dev-dependencies` internas por `path`. Biblioteca documental
  sin dependencias de producción, `publish = false`.
- Validación: exclusivamente analítica, por integración, energía, compatibilidad,
  equilibrio y superposición. Los números SI son datos artificiales propios.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: PENDIENTE (J4).

Las bitácoras de los tres tickets apilados
[ANL-001](ENG-ANL-001.md), [RES-001](ENG-RES-001.md) y
[LOAD-002](ENG-LOAD-002.md) ya contienen esa declaración completa, el autor
solicitado y el revisor pendiente. Esta entrega conserva esas declaraciones;
no declara revisión J4 ni fusión de los PR anteriores.

## Alcance y conteo

Se entregan **42 fixtures lineales**; cada fila es un modelo con parámetros y
acciones, no una sola aserción. Un test recorre todas las filas, verificando
nombres únicos, metadatos y resultados. Las variantes cambian geometría,
posición de carga, dirección, asentamiento o factores.

| Familia | Filas | Datos que cambian |
|---|---:|---|
| Voladizos: punta, uniforme y par | 5 | L=4/7; P, w o par terminal |
| Simplemente apoyadas | 7 | L=4/7; uniforme; a/L=.5, .25, .4, .7 |
| Biempotradas | 5 | L=4/7; uniforme; a/L=.5, .25, .7 |
| Voladizo apuntalado | 2 | L=4/7 |
| Continua, dos vanos iguales | 2 | L de cada vano=4/7 |
| Dos barras, triángulo, tres concurrentes | 6 | H=0/5 |
| Barra con asentamiento | 2 | s=.03/−.02 |
| Pórtico lateral y vertical | 4 | h=3/5 |
| Liberación de momento | 2 | L=4/7 |
| Rotación de todo el modelo | 3 | α=30°, 90°, 137° |
| Superposición y combinación | 4 | (α,β)=(1,1),(1.3,−.7),(−.5,2),(0,.5) |
| **Total lineal de esta entrega** | **42** | |

**ENG-VAL-001 sigue parcial.** La ampliación al final de esta nota añade 31
fixtures: **73 en total**, supera ≥30 y cubre modal, P-Delta y espectro. Las
42 filas anteriores no cambian. Siguen pendientes las referencias publicadas
con cita exacta, revisión J4 y ejecución en runtime wasm; se detallan abajo.

## Datos y convenciones

Valores predeterminados de `Parameters`: E=200 Pa, A=2 m², I=3 m⁴, L=4 m,
h=3 m, P=12 N, w=6 N/m, a/L=.5, H=0, s=.03 m, ángulo=0, factores=(1,1),
r=100000. En el fixture de par, el valor P representa 12 N·m.
Las sobreescrituras de cada fila están en la tabla Rust. Para armaduras L
representa el semiancho a, no la longitud inclinada ℓ=√(a²+h²). Para pórticos
L es la luz b; para la viga continua es la longitud de cada vano.

D2 en XY; desplazamientos y reacciones recuperados en seis componentes globales:
`[Ux,Uy,Uz,Rx,Ry,Rz]` y `[Fx,Fy,Fz,Mx,My,Mz]`. θ=v′ es positivo antihorario.
En vigas horizontales, v positivo hacia arriba y M=EI v″ positivo en sagado.
Cargas hacia abajo −P, −w. Los pares de reacción en extremos i/j son −M(0)
y +M(L), respectivamente. N positivo en tracción. No se usan fórmulas numéricas
de un tercero ni se ajustan esperados a una salida del motor.

Los nodos de observación se colocan en x=L/2 o x=a. Bajo carga uniforme, el
polinomio interno es cuártico pero el elemento con cargas consistentes obtiene
los desplazamientos y giros **nodales exactos**: la solución particular con
v=θ=0 en extremos no aporta valores nodales. La deformada cúbica interpolada
entre nodos no se utiliza como oráculo de flecha interior. Las estaciones
solicitadas son [0,.2,.8,1]; RES-001 añade raíces críticas cuando corresponden.
Los máximos de M se comprueban contra formas cerradas, incluso el del
apuntalado fuera de las estaciones solicitadas, con su ubicación exacta.

## Voladizos

Con EI constante y v(0)=θ(0)=0, integrar EI v″=M dos veces:

```text
Punta: M(x)=−P(L−x)
θ(x)=−P(Lx−x²/2)/EI
v(x)=−P(Lx²/2−x³/6)/EI
v(L)=−PL³/(3EI), θ(L)=−PL²/(2EI)
Ry=P, Rz=PL, Mmin=−PL, Mmax=0.

Uniforme: M(x)=−w(L−x)²/2
θ(x)=−w(L²x−Lx²+x³/3)/(2EI)
v(x)=−w(L²x²/2−Lx³/3+x⁴/12)/(2EI)
v(L)=−wL⁴/(8EI), θ(L)=−wL³/(6EI)
Ry=wL, Rz=wL²/2, Mmin=−wL²/2, Mmax=0.

Par terminal C: M(x)=C
θ(L)=CL/EI, v(L)=CL²/(2EI), Ry=0, Rz=−C.
```

Se comprueban flecha, giro, reacciones y rango exacto de momento. Las vigas se
sitúan con origen (2,3) para que el balance de momentos incluya brazos reales.

## Simplemente apoyadas

Con uniforme, equilibrio da RA=RB=wL/2 y M=wx(L−x)/2. Integrar con v(0)=v(L)=0:

```text
v(x)=w(Lx³/12−x⁴/24−L³x/24)/EI
θA=−wL³/(24EI), θB=+wL³/(24EI)
v(L/2)=−5wL⁴/(384EI)
M′=w(L−2x)/2=0 => x=L/2, Mmax=wL²/8, Mmin=0.
```

Con puntual en a, b=L−a, RA=Pb/L, RB=Pa/L. Escribir
`<x−a>=max(0,x−a)` permite integrar ambos tramos a la vez:

```text
EI v(x)=RA x³/6−P<x−a>³/6+EI θA x
θA=−Pb(L²−b²)/(6EIL)      (de v(L)=0)
θB=+Pa(L²−a²)/(6EIL)
v(a)=−Pa²b²/(3EIL)
M(x)=RA x−P<x−a>
Mmax=M(a)=Pab/L, Mmin=0.
```

La representación por tramos asegura continuidad de v, θ y M y salto −P de
M′. Para a=b=L/2 resulta v(a)=−PL³/(48EI), θA=−PL²/(16EI), θB=+PL²/(16EI),
Mmax=PL/4. En casos asimétricos se comprueba v(a); no se afirma que sea la
flecha máxima, cuya ubicación es distinta. Se comprueban ambos giros y el
momento máximo exacto en la posición de la carga.

## Biempotradas

Con uniforme, imponer v=θ=0 en 0,L al polinomio que satisface EI v⁽⁴⁾=−w da:

```text
v(x)=−w x²(L−x)²/(24EI)
M(x)=−wL²/12+wx(L−x)/2
MA=MB=−wL²/12, RA=RB=wL/2
v(L/2)=−wL⁴/(384EI), Mmax=wL²/24.
```

Con puntual, v(0)=θ(0)=0 elimina las constantes libres:

```text
EI v(x)=MA x²/2+RA x³/6−P<x−a>³/6
θ(L)=0 => MA L+RA L²/2−Pb²/2=0
v(L)=0 => MA L²/2+RA L³/6−Pb³/6=0
MA=−Pab²/L², MB=−Pa²b/L²
RA=Pb²(3a+b)/L³, RB=Pa²(a+3b)/L³
v(a)=−Pa³b³/(3EIL³)
Mmax=M(a)=2Pa²b²/L³, Mmin=min(MA,MB).
```

En centro MA=MB=−PL/8 y v(a)=−PL³/(192EI). Se comprueban ambos momentos,
reacciones verticales, giros cero, flecha en la carga y extremos de M.
Se deja Ux derecho libre y sin carga axial; queda exactamente cero por EA>0.
Las restricciones flexionales siguen siendo las de una biempotrada.

## Voladizo apuntalado

Superponer al voladizo uniforme una fuerza RB hacia arriba en la punta:

```text
v(L)=−wL⁴/(8EI)+RB L³/(3EI)=0 => RB=3wL/8
RA=wL−RB=5wL/8
Rz=wL²/2−RB L=wL²/8
M(x)=−w(L−x)²/2+RB(L−x)
v(x)=−w x²(6L²−4Lx+x²)/(24EI)+RB x²(3L−x)/(6EI)
v(L/2)=−wL⁴/(192EI), θ(L)=wL³/(48EI)
M(0)=−wL²/8, M(L)=0
M′=0 => x=5L/8, Mmax=9wL²/128.
```

Las filas `articulacion-extremo` usan un elemento con liberación real j=true,
extremo derecho apoyado en Uy y giro externo desconectado fijado a cero para
eliminar el mecanismo sin restringir su giro interno condensado. La solución
flexional es la del apuntalado: se comprueban reacciones, Mmax y M(L)=0 tanto
en acción de extremo como en sección, y par de reacción externo cero. El giro
externo fijado no se interpreta como el giro físico interno de la articulación.

## Dos vanos

Dos vanos iguales de longitud L, EI idéntico, cargas uniformes idénticas y
apoyos verticales en 0,L,2L. Simetría exige θ(L)=0; cada mitad, vista desde el
centro, tiene las condiciones del voladizo apuntalado. Por compatibilidad de la
sección anterior, la reacción central aportada por cada vano es 5wL/8:

```text
R0=R2L=3wL/8, RL=2·5wL/8=5wL/4
M(L)=−wL²/8
Primer vano: M(x)=3wLx/8−wx²/2
θ(0)=−wL³/(48EI), θ(2L)=+wL³/(48EI)
v(L/2)=v(3L/2)=−wL⁴/(192EI)
M′=0 => x=3L/8 y x=13L/8, Mmax=9wL²/128.
```

El giro inicial se obtiene integrando M e imponiendo v(0)=v(L)=0. La suma de
reacciones es 2wL; no se usa el resultado como un número publicado.

## Armaduras

Bases (−a,0),(a,0), ápice (0,h); ℓ=√(a²+h²), c=a/ℓ, s=h/ℓ.
Para bases fijas, la energía de dos barras es
`U=EA[(c ux+s uy)²+(−c ux+s uy)²]/(2ℓ)`.
Derivar `U−Hux+Puy` da:

```text
Kx=2EAa²/ℓ³, Ky=2EAh²/ℓ³
ux=Hℓ³/(2EAa²), uy=−Pℓ³/(2EAh²)
N1=Hℓ/(2a)−Pℓ/(2h), N2=−Hℓ/(2a)−Pℓ/(2h)
R1=(−N1c,−N1s), R2=(+N2c,−N2s).
```

Triángulo isostático: añadir barra horizontal de longitud 2a y liberar Ux de
la base derecha. Equilibrio del ápice conserva N1,N2; equilibrio de la base
derecha da `Nb=−N2c=H/2+Pa/(2h)`. Reacciones:
`Rx1=−H`, `Ry1=P/2−Hh/(2a)`, `Ry2=P/2+Hh/(2a)`.
Para obtener desplazamientos, derivar la energía complementaria
`U*=Σ Ni² ℓi/(2EA)` respecto de H y respecto de la carga vertical +Y=−P
(equivalentemente trabajo virtual unitario):

```text
ux=Hℓ³/(2EAa²)+Nb a/(EA)
uy=−Pℓ³/(2EAh²)−Nb a²/(EAh)
ux_base_derecha=Nb·2a/(EA).
```

Armadura hiperestática simple: conservar las dos bases fijas y añadir una
tercera barra desde (0,0) al ápice con base fija. Su energía `EA uy²/(2h)` se
suma a U anterior, asegurando compatibilidad de las tres barras:

```text
Kx=2EAa²/ℓ³, Ky=2EAh²/ℓ³+EA/h
ux=H/Kx, uy=−P/Ky
N1=(EA/ℓ)(c ux+s uy), N2=(EA/ℓ)(−c ux+s uy), N3=EA uy/h
R3y=−N3.
```

Las fuerzas no se deducen solo por equilibrio en la hiperestática. Se comprueban
ambos desplazamientos, N de todas las barras en todas las estaciones, reacciones
y las cinco resultantes no axiales nulas.

## Asentamiento

Barra axial con u(0)=0, u(L)=s y sin carga distribuida. De N′=0 y u′=N/(EA),
`N=EA s/L`, `u(L/2)=s/2`, reacciones `R0=−N`, `RL=+N`.
Se usan dos elementos en serie para conservar un GDL axial interior libre;
los transversales se restringen. Se comprueban asentamientos de ambos signos,
las fuerzas y el desplazamiento prescrito. La combinación identidad usa factor
1; no se suma el mismo asentamiento varias veces como si fuera carga homogénea.

## Portal lateral

Luz b, altura h, dos columnas empotradas. H/2 en cada esquina superior.
En el límite de dintel rígido y columnas inextensibles, las columnas tienen
traslación Δ y giro superior cero. La energía de cada columna es
`6EI Δ²/h³`; derivar la energía total menos HΔ da
`K=2·12EI/h³`, `Δ=Hh³/(24EI)`, `Rx1=Rx2=−H/2`.

Solo aumentar EI del dintel no impediría su rotación por deformación axial de
las columnas. Por eso las filas aumentan simultáneamente EI del dintel y EA
de columnas/dintel por r=100000, dejando EI de las columnas intacto.
La forma cerrada **finita** se deriva antes de tomar el límite. Sean J=EI_col,
B=EI_dintel y C=EA_col; simetría impone traslaciones Δ, giros θ iguales y
verticales v,−v. El dintel no se alarga. Sumando energías:

```text
Π=12JΔ²/h³+12JΔθ/h²+(4J/h+6B/b)θ²
 +(C/h+24B/b³)v²+24Bvθ/b²−HΔ
β=(12B/b²)/(C/h+24B/b³), v=−βθ
D=4J/h+6B/b−12Bβ/b²
θ=−6JΔ/(h²D)
Δ=H/(24J/h³−72J²/(h⁴D)).
```

La tabla usa B=rJ, C=rEA. Definiendo q=3J/(hD), el error relativo respecto al
límite es `q/(1−q)`: <1.80e-5 para h=3 y <1.53e-5 para h=5, b=4 y los datos
SI anteriores. Se comprueba Δ límite con margen relativo 5e-5 y se comprueban
también Δ, θ, v de la solución finita con tolerancia numérica independiente.

## Portal vertical

Columnas y dintel con E,A,I iguales y carga uniforme −w solo en el dintel.
Simetría: esquinas superiores `(t,v,θ)` y `(−t,v,−θ)`. La energía reducida,
incluido el trabajo de las cargas consistentes del dintel, es (constante omitida):

```text
Π=(12EI/h³+2EA/b)t²+12EI tθ/h²+(4EI/h+2EI/b)θ²
  +EA v²/h+wb v+wb²θ/6
γ=(6EI/h²)/(12EI/h³+2EA/b)
t=−γθ
D=4EI/h+2EI/b−6EIγ/h²
θ=−(wb²/12)/D
v=−wbh/(2EA), Ry1=Ry2=wb/2.
```

Son las tres ecuaciones ∂Π/∂t=∂Π/∂θ=∂Π/∂v=0. En el centro del dintel, sumar
la interpolación cúbica de las rotaciones y la particular biempotrada:
`vmedio=v+bθ/4−wb⁴/(384EI)` y `θmedio=0`. Se comprueban desplazamientos,
giros, reacciones iguales y horizontales opuestas sin aproximación de rigidez.

## Rotacion

Rotar todos los puntos del voladizo alrededor del empotramiento (2,3) y su
fuerza terminal por Q(α), con α=30°,90°,137°. Como las restricciones del
empotramiento son completas, permanecen idénticas al rotarlas. La energía
elástica y el trabajo externo son invariantes bajo cambio ortogonal de ejes:

```text
Q=[[cosα,−sinα],[sinα,cosα]]
u_global=Q(0,−PL³/(3EI))
R_global=Q(0,P)
θ=−PL²/(2EI), Rz=PL, M(x)=−P(L−x).
```

Se compara cada componente global con la solución rotada, la proyección
transversal con la misma flecha original, el giro y los momentos invariantes.
Se comprueba equilibrio respecto del origen global con las posiciones rotadas.

## Superposicion

Apoyos homogéneos: `K uP=FP`, `K uw=Fw` implica
`K(αuP+βuw)=αFP+βFw`. Reacciones `R=Ku−F` y recuperación de acciones son
lineales en u y F. En el voladizo:

```text
v(L)=−αPL³/(3EI)−βwL⁴/(8EI)
θ(L)=−αPL²/(2EI)−βwL³/(6EI)
Ry=αP+βwL, Rz=αPL+βwL²/2.
```

Cada una de cuatro filas resuelve P, w y D=αP+βw por `StaticLinear`, recupera
todos los resultados y usa `combine` de LOAD-002. Compara suma contra D en
desplazamientos, reacciones, acciones y las estaciones comunes; además compara
flecha, giro y reacciones contra las formas cerradas anteriores. La combinación
con factores opuestos puede tener nuevas raíces en D: no se exige que sus
extremos continuos coincidan con los extremos discretos de `combine`, ni se
inventa una búsqueda de raíces para LOAD-002. Cada fixture también pasa sus
resultados por combinación identidad, verificando los campos recuperados.

## Equilibrio

Cada caso resuelto comprueba residual libre/global del análisis y calcula un
balance independiente con los resultados recuperados:

```text
ΣFx=ΣRx+ΣFx_nodal+Σqx ℓ=0
ΣFy=ΣRy+ΣFy_nodal+Σqy ℓ=0
ΣMz=Σ[Rz+x Ry−y Rx]+Σ[Mz_nodal+x Fy−y Fx]
    +Σ[mz ℓ+xcentro qy ℓ−ycentro qx ℓ]=0.
```

Se usan cargas físicas originales; no se reaprovecha el vector equivalente
ensamblado como oráculo de equilibrio. El balance incluye los brazos de fuerzas
y no exige que la suma de pares sin brazos sea cero. Se aplica a los 42
fixtures, incluidos los casos P,w,D y los asentamientos.

## Tolerancias

Cada fila almacena explícitamente `Tolerance { absolute, relative,
justification }`. Regla: `|obtenido−esperado| ≤ absoluto+relativo·|esperado|`;
ambos valores deben ser finitos. No se utiliza la magnitud obtenida para aumentar
el umbral. Las magnitudes son SI según el campo (m, rad, N, N·m); las posiciones
son adimensionales. Para los modelos ordinarios, absoluto=1e-9 y relativo=1e-10:
unos pocos nodos, parámetros moderados y soluciones nodales cerradas justifican
un margen conservador frente a eliminación y recuperación f64. Los ceros,
residuos y equilibrio se comparan con 1e-9 absoluto.

Para portal lateral: absoluto=1e-8 por contraste de rigidez 1e5 y cancelación en
la recuperación; relativo=5e-5 solo para aproximar el límite rígido. El error
finito está acotado analíticamente arriba. La comprobación adicional de la
solución finita usa absoluto=1e-8 y relativo=1e-10, justificados por ese
condicionamiento. En nativo el residuo de momento del portal h=5 fue 2.45e-9;
queda dentro del margen absoluto documentado. Los residuos/ceros no reciben un
margen relativo por aproximación, porque su esperado es cero.

## Verificación de la sesión

Verificación local del 2026-10-11, sin red (`CARGO_NET_OFFLINE=true`):

- `cargo fmt` y `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK; 241 pruebas nativas, incluidos doctests.
  El único test nuevo recorre los 42 fixtures y señala el nombre ante un fallo.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --workspace --all-targets --target wasm32-unknown-unknown`: OK;
  incluye el ejecutor, tabla y oráculos de pruebas, sin E/S ni hilos.
- `bash clean-room/check.sh origin/main`: OK. Comprobación adicional incluyendo
  todos los archivos nuevos en un índice y objetos temporales en `/tmp`, con
  objetos del repositorio original en lectura: OK. El índice real quedó intacto.
- `git diff --check`: OK para los cambios ya seguidos; revisión con archivos
  nuevos mediante el mismo índice temporal: OK.

Pruebas nativas y documentación: rustc/cargo 1.85.1 del sistema. Clippy disponible
0.1.99. Para wasm se usó `PATH=/home/box/.cargo/bin:$PATH
RUSTUP_TOOLCHAIN=stable`, toolchain y destino ya instalados (rustc 1.99.0), sin
descargar componentes. Compilar para wasm no significa ejecutar un runtime wasm.
No se modificó el índice Git real; sin commit ni push. Esta verificación
corresponde a la entrega lineal anterior; la ampliación y su verificación se
registran a continuación. Revisión J4 y ejecución en runtime wasm pendientes.

## Ampliación dinámica y 2.º orden

### Bitácora de la ampliación

- Ticket / PR: ENG-VAL-001, ampliación sobre ENG-ANL-002 (#61, modal),
  ENG-ANL-003 (#59, P-Delta) y ENG-ANL-004 (#62, espectro), cadena abierta.
- Autor(es): J2 (agente, Grok Bot), etiqueta solicitada; ejecutor Codex
  basado en GPT-6 · 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de las derivaciones propias, sin delegación.
- Contrato implementado: fixtures de validación por APIs públicas `Modal::run`,
  `PDelta::run`, `Spectrum::run`, tablas y combinaciones espectrales, elementos
  `EulerFrame2D` y `LinearTruss`; sin cambiar implementación de los análisis.
- Fuentes permitidas usadas: petición del usuario, APIs/pruebas propias del
  workspace, `provenance/TEMPLATE.md` y `docs/CLEAN_ROOM.md`. Sin red,
  bibliografía externa, libros, páginas, ejemplos numéricos publicados u oráculos.
- Derivaciones propias: ecuaciones D1–D12 a continuación;
  [datos](../crates/engine-validation/tests/support/dynamic_table.rs),
  [montaje](../crates/engine-validation/tests/support/dynamic_harness.rs) y
  [oráculos](../crates/engine-validation/tests/dynamic.rs).
- Código de terceros copiado: ninguno; sin dependencias nuevas.
- Validación: exclusivamente analítica; datos SI artificiales propios.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración para ENG-ANL-002, ENG-ANL-003 y ENG-ANL-004 en esta sesión:
  "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: PENDIENTE (J4).

Se conservan las declaraciones anteriores; se añade registro de esta sesión a
ANL-002/004 y se crea registro de la validación de ANL-003, ausente en este
checkout. Esto no atribuye a esta sesión autoría retroactiva de su implementación.

### Recuento y alcance

| Grupo nuevo | Filas | Problemas completos |
|---|---:|---|
| Modal | 12 | Cortante N=1..5; 2 pisos variables; cadena axial libre-fija; viga apoyada y voladizo con mallas 4/12; voladizo rotado |
| P-Delta | 10 | Columna r=0,.2,.5,.8,.9,−.5,1.05; marco r=.05,.1; serie de iteración y límite |
| Espectro | 9 | SRSS 2 pisos; desplazamiento modal con otra Sa; interpolaciones lineal/log-log; CQC separado/cercano/coincidente; dos combinaciones direccionales |
| **Nuevos** | **31** | Un test recorre cada grupo y nombra fixture, campo, source/citation y tolerancia ante fallo |
| Lineales existentes | 42 | Tabla y test originales intactos |
| **Total ENG-VAL-001** | **73** | ≥30 totales y ≥24 nuevos |

No se cuentan aserciones internas como fixtures. Ortogonalidad, acumulación de
masa y participación se comprueban dentro de cada problema de cortante/cadena.
Todos los nuevos `source/citation` remiten a esta derivación, sin citas a libros.
No se convierten los resultados calculados por el motor en valores esperados.

### Modal: derivación D1–D4

**D1. Cortante uniforme.** De la energía
`U=k/2 Σ(q_j−q_{j−1})²`, `T=m/2 Σq̇_j²`, `q_0=0`, obtenemos
`k(2φ_j−φ_{j−1}−φ_{j+1})=mω²φ_j`. El techo libre equivale a
`φ_{N+1}=φ_N`. Proponer `φ_j=sin(jθ)` da `ω²=4k/m sin²(θ/2)` y
`sin((N+1)θ)=sin(Nθ)`, es decir

```text
θ_n=(2n−1)π/(2N+1), n=1..N
ω_n=2√(k/m) sin((2n−1)π/[2(2N+1)])                    (D1)
φ_j=sin(jθ_n)/√[m Σ_l sin²(lθ_n)].
```

Las cinco filas usan k=23 N/m, m=4 kg, N=1..5. La cadena libre-fija usa
tres barras axiales de longitud 1, EA=31, densidad cero y masas puntuales
de 6 kg. Su energía axial es exactamente D1 con k=EA/ℓ; un extremo fijo,
el otro libre y desplazamientos transversales restringidos. Así verifica
elementos reales y masas puntuales con el mismo oráculo físico.

**D2. Dos pisos variables y Γ a mano.** Con k1=19, k2=7 N/m y
m1=2, m2=5 kg:

```text
K=[[k1+k2,−k2],[−k2,k2]], M=diag(m1,m2)
det(K−λM)=m1m2 λ²−[m2(k1+k2)+m1k2]λ+k1k2=0
10λ²−144λ+133=0 ⇒ λ±=(144±√15416)/20
v=(1,t), t=(k1+k2−m1λ)/k2
s=√(m1+m2t²), φ=v/s, Γ=(m1+m2t)/s, M*=Γ².             (D2)
```

Para los dos pisos iguales de D1, `λ±=(k/m)(3±√5)/2`,
`t=(1±√5)/2`, `s=√[m(1+t²)]`, `Γ=m(1+t)/s` y
`M*±=m(1±2/√5)`. Su suma es `2m`. El test adapta únicamente el signo
arbitrario del autovector al comparar Γ; el oráculo de λ, φ y M* es manual.

**D3. Ortogonalidad y completitud.** Restar `φᵢᵀKφⱼ=λⱼφᵢᵀMφⱼ` y su
traspuesta da `(λᵢ−λⱼ)φᵢᵀMφⱼ=0`; con modos distintos y normalización a masa,
`ΦᵀMΦ=I` y `ΦᵀKΦ=diag(λ)`. Expandir la traslación rígida libre `r` en
la base completa, `r=ΣΓ_n φ_n`, da

```text
Γ_n=φ_nᵀMr, ΣM*_n=ΣΓ_n²=rᵀMr=Σm_j.                 (D3)
```

Se calculan productos con M/K ensambladas, incluyendo todas las parejas;
los resultados esperados son I y λ, derivados arriba. Se comparan las
acumulaciones por modo y el informe de masa libre/total con Σm_j, sin usar
la masa reportada como oráculo. En vigas con masa en apoyos, la igualdad de
completitud corresponde a masa libre, no necesariamente a la masa física total;
por ello la igualdad exacta al total se exige a los modelos de masas puntuales.

**D4. Vigas y rotación.** Separar variables en `ρA v̈+EI v''''=0` da
`v''''=β⁴v` y `ω=β²√(EI/ρA)/L²` (β adimensional). La solución espacial
es combinación de cos, sin, cosh y sinh. Para apoyada, `v=v''=0` en ambos
extremos ⇒ `β=nπ`. Para voladizo, `v=v'=0` en la base y `v''=v'''=0` en
punta ⇒ determinante `cosβ coshβ+1=0`. El test obtiene las tres primeras
raíces por 55 bisecciones en `(jπ+.1,(j+1)π)`; no usa números publicados.
Aquí E=ρ=A=L=1, I=1e−4 ⇒ `ω=.01β²`, primeros tres modos flexionales.
Se emplea masa consistente y elementos Hermite `EulerFrame2D` con 4/12
elementos: márgenes relativos 1.9% / 0.03%, verificados para estas filas,
y disminución del error >10 veces al refinar. Son márgenes de estas mallas,
no una cota universal para cualquier viga o modo.

Para α=.83 rad, `Q=[[cosα,−sinα],[sinα,cosα]]` rota coordenadas, formas y
dirección de excitación. La congruencia ortogonal simultánea de K/M preserva
λ y `φᵀMr`; se comparan frecuencias, |Γ|, M* y cada componente `Qφ` con
margen numérico independiente del margen de discretización.

### P-Delta: derivación D5–D8

**D5. Voladizo comprimido.** El funcional
`Π=1/2 ∫[EI(v'')²−P(v')²] dx−Hv(L)` da
`EI v''''+P v''=0`, `v(0)=v'(0)=0`, `v''(L)=0` y
`EI v'''(L)+P v'(L)=−H`. Integrar la solución trigonométrica da

```text
a=√(P/EI), z=aL, Pcr=π²EI/(4L²), r=P/Pcr
v(L)=H/P [tan(aL)/a−L]
v0=HL³/(3EI), v/v0=3(tan z−z)/z³, z=π√r/2.          (D5)
```

El polo de tan en z=π/2 corresponde a la inestabilidad. Su coeficiente
respecto a `1/(1−r)` es `96/π⁴≈.9855`; por tanto esa amplificación es una
aproximación incluso con malla infinita. Las filas usan EI=600 N·m²,
L=4 m, H=.1 N y r=.2,.5,.8,.9. Se contrasta la amplificación aproximada
con 1.6% relativo (error de modelo), y la solución D5 con 2e−6 relativo
para 16 elementos (error de malla). Una resolución adicional con 4 elementos
debe tener error >10 veces mayor. r=1.05 devuelve `Unstable`, una dirección
negativa por inercia y ningún desplazamiento ni historial lateral.

**D6. Tracción y cero.** Sustituir P=−T en D5 da
`v=H/T [L−tanh(aL)/a]`, `a=√(T/EI)`; para r=−.5,
`v/v0=3(z−tanh z)/z³<1`. Se comprueba rigidez mayor y la solución exacta
con el mismo margen de malla. Para P=0, el desarrollo
`tan z=z+z³/3+…` recupera `v0`; además se compara todo el vector lateral
con `StaticLinear(H)` y se comprueba la convergencia final de cada análisis.

**D7. Marco de un piso.** n=2 columnas empotradas, h=4, dintel rígido,
giro superior cero. Para cada columna, la forma cúbica lineal
`v=Δ(3s²−2s³)`, s=x/h, tiene energía `6EIΔ²/h³`.
La rigidez de piso es `Kpiso=12nEI/h³`. En la idealización de cuerda,
el trabajo de gravedad total P produce `−PΔ²/(2h)`:

```text
(Kpiso−P/h)Δ=H
Pcr_piso=Kpiso h=12EI_c n/h²
Δ/Δ0≈1/(1−P/Pcr_piso), Δ0=Hh³/(12nEI_c).             (D7)
```

El símbolo ≈ es esencial: la energía geométrica del campo cúbico real es
`−(P/n)/2 ∫(v')²dx=−(P/n)·3Δ²/(5h)`, factor 6/5 frente a la cuerda.
El análisis con Kg distribuida no puede tener exactamente el Pcr de esa
idealización. Resolver D5 con `v'(0)=v'(h)=0` y desplazamiento terminal Δ
da el contraste continuo de cada columna guiada:

```text
K_col(P/n)=EI a³/[2tan(z/2)−z], a=√[(P/n)/EI], z=ah
Δ/Δ0=12[2tan(z/2)−z]/z³, z=√(12r).
```

El polo físico continuo es `Pcr=π²nEI/h²`, distinto de `12nEI/h²`.
Las filas r=.05,.1 comparan D7 con margen 2.6%, suficiente para la diferencia
de idealización en esos dos puntos. También comparan con la solución guiada
continua a 1e−4 relativo. Se usan 12 elementos por columna, EA de columnas y
EA/EI del dintel multiplicadas por 1e5; el margen continuo incluye esa
flexibilidad finita y error de malla. Se comprueba la deriva de ambos nudos
superiores y su simetría. No se presenta el Pcr de cuerda como el Euler exacto.

**D8. Convergencia de la iteración.** Modelo escalar con k=80 N/m,
gravedad −rk, r=.3, carga lateral F=8 N y productor `kg=k(u_g2−u_g1)=−rk`.
El punto fijo desde u0=0 satisface `u_i=F/k+r u_{i−1}`:

```text
u_i=F/[k(1−r)](1−r^i), residual_relativo=r^i
cambio_relativo=(F/k)r^(i−1), escala=1 y |u_i|<1.     (D8)
```

La fila comprueba cada desplazamiento, residual, cambio y el primer paso
que cumple ambos criterios a 1e−7, con límite 32. Con límite 3 exige
`NotConverged` sin reintento. Las columnas y marcos también exigen que
el último residual y cambio estén dentro de la tolerancia declarada 1e−10.

### Espectro: derivación D9–D12

**D9. SRSS y coordenada modal máxima.** Multiplicar la ecuación dinámica
por un modo normalizado a masa da `q̈_n+ω_n²q_n=−Γ_n a_g` (se omite el
signo convencional en la amplitud firmada según API). La pseudoaceleración
de la tabla produce `qmax_n=Γ_n Sa/ω_n²`; la fuerza equivalente es
`Mφ_nΓ_nSa`. Su suma proyectada es `Vb,n=Γ_n²Sa=M*_nSa`. Por independencia:

```text
Vb²=Σ_n(M*_n Sa)², u_n=φ_nΓ_nSa/ω_n².               (D9)
```

Para los dos pisos de D2, k=23, m=4 y Sa=2.7 / 5.1 m/s²:
`M*±=m(1±2/√5)`, `Vb²=(18/5)(mSa)²`. Cada fila comprueba Sa, coordenada
modal máxima, cortante de cada modo, cada desplazamiento físico firmado y
envolventes SRSS contra D2/D9, sin obtener Γ esperado del resultado modal.

**D10. Tablas.** Lineal entre `(1,3),(5,11)`:
`Sa(T)=3+(T−1)(11−3)/(5−1)` ⇒ `Sa(2.5)=6`.
Log-log entre `(1,3),(4,48)`:
`ln Sa=ln3+ln(T/1) ln(48/3)/ln(4/1)` ⇒ `Sa=3T²`,
`Sa(2)=12`. También se verifican ambos extremos. No se da a la interpolación
log-log el valor esperado de la interpolación lineal.

**D11. CQC.** La ecuación WDKB 1981 es la fórmula suministrada por la
petición; la atribución identifica esa ecuación, sin afirmar acceso al paper,
su código o un ejemplo publicado. En el test se expande directamente:

```text
r=ω_j/ω_i
ρ_ij=8√(ζ_iζ_j)(ζ_i+rζ_j)r^(3/2) /
 [(1−r²)²+4ζ_iζ_jr(1+r²)+4(ζ_i²+ζ_j²)r²]
R_CQC=√(A²+B²+2ρ_ijAB).                              (D11)
```

La fila cercana usa ω=(10,10.2), ζ=(.05,.07), A=3, B=−4; el signo de
AB discrimina la combinación de magnitudes prematura. Se compara ρ con la
expansión manual, su simetría y la combinación. Para modos iguales y ζ igual,
r=1 ⇒ numerador=denominador=16ζ² ⇒ `ρ_ii=1`, `R=|A+B|`.
Para ω=(1,100), ζ=.05, `ρ<2.1e−5`: CQC≈SRSS con margen relativo 2e−5;
no es identidad exacta con amortiguamiento positivo y separación finita.
Con esas frecuencias distintas y ζ=0, el numerador es cero, por tanto
**CQC=SRSS exactamente** (dentro de margen f64). Se comprueban ambos casos.

**D12. Factores direccionales.** Masa puntual isotrópica de 2 kg,
resortes independientes kx=18, ky=50 N/m unidos a base fija: ωx=3, ωy=5.
Excitar x/y por separado da `Vx=2Sa,x`, `Vy=2Sa,y`, `ux=Sa,x/9`,
`uy=Sa,y/25`. Para factores de usuario ax,ay:

```text
V_ponderado=axVx+ayVy
V_SRSS=√[(axVx)²+(ayVy)²].                            (D12)
```

Dos filas ejecutan `Spectrum::run` completo con Sa=(2,5), factores=(1.4,.35)
y Sa=(3,4), factores=(.65,1.2), respectivamente; verifican ambos basales y
las dos componentes de desplazamiento combinado. No hay factores normativos.

### Tolerancias, portabilidad y criterios pendientes

La regla sigue siendo `|a−e|≤absoluto+relativo·|e|`, con finitos obligatorios;
cada fila tiene justificación. Las comparaciones algebraicas usan 1e−9 absoluto
y relativo; ortogonalidad, rotación y comparación lineal usan 2e−9 absoluto
y 2e−8 relativo para acumulación/condicionamiento f64. Los márgenes de malla,
de idealización y del límite CQC están separados y declarados arriba. No se
amplía una tolerancia en función del valor obtenido para hacer pasar un fallo.

Los fixtures son tablas constantes y los tests solo operan en memoria. Sin
E/S, red, hilos, procesos ni reloj. `Arc` es un requisito de la API de
productores, no introduce hilos. La biblioteca documental no añade código que
pueda entrar en panic/unwrap/expect; los auxiliares devuelven errores del
catálogo `ENG-E-0012` y propagan errores públicos. Solo el límite del test
convierte un fallo en panic con nombre, campo y provenance.

Estado de los criterios de VAL-001:

- **CUBIERTO:** ≥30 fixtures totales, ≥24 nuevos y cobertura analítica modal,
  P-Delta y espectral solicitada; 73 totales / 31 nuevos.
- **CUBIERTO:** datos tabulados, source/citation propia, tolerancias declaradas,
  fallo con nombre y compilación de pruebas para wasm sin dependencias nuevas.
- **PENDIENTE:** ejemplos de libro/publicados con edición, página y cita exacta:
  no verificables sin acceso; fuera del alcance de J2. No se inventan citas.
- **PENDIENTE:** ejecución de los tests en un runtime wasm; compilar no equivale
  a ejecutarlos. Tampoco se declara habilitada cobertura de gran deformación,
  plasticidad o historias temporales a partir del P-Delta elástico/espectro.
- **Revisor clean-room: PENDIENTE (J4)**. No se declara cerrado VAL-001 completo.

### Verificación de la ampliación (2026-10-11)

Todos los comandos Cargo se ejecutaron con `CARGO_NET_OFFLINE=true`, sin
descargar dependencias ni componentes:

- `cargo fmt` y `cargo fmt --check`: **OK**.
- `cargo clippy --all-targets -- -D warnings`: **OK**.
- `cargo test --workspace`: **OK**, **294 pruebas nativas**, incluidos
  doctests; el test lineal original recorre 42 fixtures y los tres tests nuevos
  recorren 12/10/9 filas. Ningún fallo ni fixture existente modificado.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: **OK**.
- `cargo check --workspace --target wasm32-unknown-unknown`: **OK**.
- `cargo check --workspace --all-targets --target wasm32-unknown-unknown`:
  **OK**, incluye explícitamente las tablas, auxiliares y oráculos nuevos.
- `bash clean-room/check.sh origin/main`: **OK**. Comprobación adicional con
  todos los archivos nuevos reconocidos mediante índice y objetos temporales
  en `/tmp`: **OK**, también `git diff --cached --check` en ese índice.
- `git diff --check`: **OK**. Cargo.toml, Cargo.lock, tabla/harness/test lineales
  originales y bibliotecas de análisis/elementos permanecen intactos.

Pruebas nativas/documentación: rustc/cargo 1.85.1 del sistema; Clippy 0.1.99.
Wasm: `PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable`, rustc 1.99.0
con destino ya instalado. No se ejecutó un runtime wasm. El índice Git real
permanece intacto; **sin commit ni push**, sin red ni dependencias nuevas.
Revisor clean-room: PENDIENTE (J4).
