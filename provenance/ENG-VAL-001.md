# Bitácora clean-room · ENG-VAL-001 (parcial, apilado)

- Ticket / PR: ENG-VAL-001 «Suite de benchmarks estructurales V1», únicamente
  estática lineal. Rama apilada sobre ENG-LOAD-002 (#54), ENG-RES-001 (#53),
  ENG-ANL-001 (#52) y ENG-LOAD-001 (#51), aún sin fusionar. Sin commit ni push.
- Autor(es): J2 (agente, Grok Bot) · identificación solicitada por el usuario;
  agente ejecutor Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: crate de desarrollo `engine-validation`, un único test
  tabulado sobre `StaticLinear`, `element/1` (truss/frame),
  `engine_analysis::results` y `engine_analysis::combination` de LOAD-002.
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

**ENG-VAL-001 sigue parcial.** La entrega de la suite total V1 de ≥30 fixtures,
con la cobertura requerida de modal, P-Delta y espectro, queda pendiente de
ANL-003/004 y ASM-002. Los 42 aquí entregados satisfacen el umbral numérico en
estática lineal, pero no sustituyen esa cobertura. No se inventan métodos,
contratos ni resultados de esos tickets. No se afirma cerrar VAL-001 completo.

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
No se modificó el índice Git real; sin commit ni push. La revisión J4, ejecución
en runtime wasm y cobertura no lineal/dinámica siguen pendientes.
