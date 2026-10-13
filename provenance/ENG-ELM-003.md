# Bitácora clean-room · ENG-ELM-003

- Ticket / PR: ENG-ELM-003 · viga-columna elástica Euler–Bernoulli 3D; PR #44.
- Autor(es): J2 (agente, Grok Bot) · modelo de ejecución de esta sesión:
  GPT-6 (Codex) · 2026-10-11 (America/Guayaquil).
- Rol: especificador e implementador de derivaciones analíticas propias.
- Contrato implementado: element/1 · kind = `frame3d.euler_elastic`.
- Fuentes permitidas usadas: ninguna externa. Solo documentos y patrones internos
  de este repo, enumerados abajo. No se consultó la red.
- Derivaciones propias: [sección de derivaciones de esta bitácora](#derivaciones-propias).
- Código de terceros copiado: ninguno. Se reutilizan patrones internos de frame2d
  y del productor de truss conforme a la solicitud.
- Validación: analítica, integrales polinómicas propias, soluciones cerradas,
  condensación independiente y kit de conformidad existente.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados para este ticket (docs/CLEAN_ROOM.md §5)."
- Revisor clean-room: PENDIENTE (J4).

## Documentos internos consultados

`docs/CLEAN_ROOM.md`, `docs/CONVENCION_EJES.md`, `provenance/TEMPLATE.md`,
`provenance/ENG-ELM-002.md`, todos los módulos, fábrica y pruebas de
`crates/engine-elements-frame`; el productor y pruebas de productor de
`engine-elements-truss`; traits, tipos y kit de `engine-contracts`; catálogo
`engine-core`, ley `engine-laws::uniaxial::Elastic`, matrices y backend de
`engine-linalg`, ensamblaje y sistema de `engine-assembly` y tipos de
`engine-model`. No existe AGENTS.md aplicable. No se consultaron otros motores,
fuentes externas, ejemplos externos ni resultados de oráculos.

## Derivaciones propias

### Terna, interpolación y energía

La convención aprobada es global X×Y=Z, X-Y horizontal y Z hacia arriba;
traslaciones y giros positivos por sus ejes y mano derecha. Se conserva la
convención de caras y N positivo en tracción de CONVENCION_EJES.md.

Sean d=p_j−p_i, L=hypot(d_x,d_y,d_z), x=d/L. El argumento `orientation`
es una referencia global de **z local**, no de y. Se normaliza escalando por
su componente máxima absoluta (evita overflow y admite escalas diminutas).
Con referencia unitaria a, se calcula y=unit(a×x), z=unit(x×y).
Esto equivale a proyectar a en el plano perpendicular a x, usar la proyección
como z y elegir y=z×x. Cumple x×y=z; la orientación transversal explícita
prevista por la convención se obtiene mediante y, sin eje automático.
El caso a=0 o |a×x|≤1e-12 se rechaza; el segundo umbral es angular,
adimensional y distinto de la tolerancia absoluta de longitud.

R tiene filas x,y,z. En desplazamientos global→local se usa
T=diag(R,R,R,R), uno por traslación y giro de ambos nodos:
K_global=TᵀK_local T y f_global=Tᵀf_local. Una rotación propia Q del
modelo completo rota **también a**; entonces K'=Q_e K Q_eᵀ, f'=Q_e f.

Orden local:
`[u_i,v_i,w_i,rx_i,ry_i,rz_i,u_j,v_j,w_j,rx_j,ry_j,rz_j]`.
Sin corte, la rotación de sección tiene rz=v′ y ry=−w′, obtenidas de
r×x=(0,rz,−ry). u y rx se interpolan linealmente. Con t=x/L,
las formas de Hermite de valor y derivada son:

```text
h1=1−3t²+2t³       h2=L(t−2t²+t³)
h3=3t²−2t³         h4=L(−t²+t³)
v=h1 vi+h2 rzi+h3 vj+h4 rzj
w=h1 wi−h2 ryi+h3 wj−h4 ryj
Π=½∫[EA u′²+GJ rx′²+EIz v″²+EIy w″²]dx
```

J es la constante torsional de Saint-Venant, **no** Iy+Iz. No se deduce G
mediante Poisson ni se agrega deformación por corte. El axial se evalúa con
la ley elástica interna: ε=(uj−ui)/L, N=A·Eε. No hay historia.

### Matriz Euler 12×12

La integral de h″ᵀh″ produce el bloque de rigidez de viga de Euler–Bernoulli (resultado clásico de texto; no se cita ningún manual concreto), derivado aquí por integración propia de h″ᵀh″:

```text
B(EI)=(EI/L³) [[12,6L,−12,6L],
               [6L,4L²,−6L,2L²],
               [−12,−6L,12,−6L],
               [6L,2L²,−6L,4L²]]
```

En v se inserta en índices (1,5,7,11); en w, en (2,4,8,10) con congruencia
D=diag(1,−1,1,−1). Los bloques de u y rx son respectivamente
(EA/L)[[1,−1],[-1,1]] en (0,6) y (GJ/L)[[1,−1],[-1,1]] en (3,9).
Definiendo a=EA/L, t=GJ/L, bα=12EIα/L³, cα=6EIα/L²,
dα=4EIα/L, eα=2EIα/L, se obtiene explícitamente:

```text
[ a   0    0   0    0    0  −a   0    0   0    0    0 ]
[ 0  bz    0   0    0   cz   0 −bz    0   0    0   cz ]
[ 0   0   by   0  −cy    0   0   0  −by   0  −cy    0 ]
[ 0   0    0   t    0    0   0   0    0  −t    0    0 ]
[ 0   0  −cy   0   dy    0   0   0   cy   0   ey    0 ]
[ 0  cz    0   0    0   dz   0 −cz    0   0    0   ez ]
[−a   0    0   0    0    0   a   0    0   0    0    0 ]
[ 0 −bz    0   0    0  −cz   0  bz    0   0    0  −cz ]
[ 0   0  −by   0   cy    0   0   0   by   0   cy    0 ]
[ 0   0    0  −t    0    0   0   0    0   t    0    0 ]
[ 0   0  −cy   0   ey    0   0   0   cy   0   dy    0 ]
[ 0  cz    0   0    0   ez   0 −cz    0   0    0   dz ]
```

No se integra numéricamente en producción. Las pruebas integran productos de
polinomios mediante ∫₀¹tⁿdt=1/(n+1), con coeficientes independientes de
la construcción de matrices. `respond` entrega Kc y Kc·u, sin cargas ni Kg.

### Liberaciones de T, My, Mz

`MomentReleases { i:[T,My,Mz], j:[T,My,Mz] }` contiene booleanos locales.
Dividiendo desplazamientos en a conservados y r internos liberados:

```text
ur=−Krr⁻¹Kra ua+Krr⁻¹fr
u=H unodal+ucarga
Kc=Kaa−Kar Krr⁻¹Kra = HᵀKH
fc=fa−Kar Krr⁻¹fr = Hᵀf
```

Los tres bloques rotacionales son independientes. Para cada plano, en la
variable de pendiente θ (rz para v, −ry para w):

- Solo i: θi=3(vj−vi)/(2L)−θj/2+fθi L/(4EI).
- Solo j: expresión permutada.
- Ambos: θi=θj=(vj−vi)/L sin carga. La parte afín usa
  Krr⁻¹=(L/EI)[[1/3,−1/6],[-1/6,1/3]].

Una liberación deja energía 3EI/(2L³)(vj−vi−Lθfijo)²; dos anulan
Kflexión **exactamente**, con columnas y filas de giros liberados nulas.
Se construye ese Schur analítico para evitar rigideces residuales con EI enorme.

En torsión, una liberación da rx_r=rx_a+fr/(GJ/L) y Ktorsión condensada=0:
no puede transmitirse torsión a través de un extremo libre. La carga equivalente
transfiere todo el par al extremo conservado. Con ambos libres,
Krr=(GJ/L)[[1,−1],[-1,1]] es singular en el giro común. Para carga compatible
fi+fj=0 se toma giro medio cero y rx_i=fi/(2GJ/L), rx_j=−rx_i;
H tiene ambas filas torsionales cero. Bajo carga uniforme mx≠0,
fi+fj=mx L≠0: no existe equilibrio, ENG-E-0006. Una pareja de cargas
uniformes opuestas sí puede sumar cero en `recover`/`local_displacements`.
No se oculta el mecanismo ni se añade rigidez artificial.

`local_displacements` recupera los doce GDL físicos locales incluyendo
las partes afines cargadas. `recover` obtiene los extremos mediante Kc u−fc,
algebraicamente igual a K u_rec−f en GDL conservados, y cero en los liberados.
Así evita cancelaciones de términos enormes también con cargas y dos
liberaciones. Las pruebas comparan con un Schur numérico independiente,
recuperación completa y proyección de matrices; para doble torsión el solver
independiente fija un gauge diferente, sin alterar Kc, fc ni esfuerzos.

### Masa consistente y concentrada

La energía cinética es
½∫[rho A(u̇²+v̇²+ẇ²)+rho(Iy+Iz)rẋ²]dx.
La inercia polar física Ip=Iy+Iz es independiente de J. Se omite inercia
rotatoria de flexión, como en frame2d. Las inercias en ry/rz provienen de la
interpolación transversal, y no de añadir rho Iy o rho Iz.

```text
m=rho A L, p=rho(Iy+Iz)L
M_axial=(m/6)[[2,1],[1,2]], M_torsión=(p/6)[[2,1],[1,2]]
M_Hermite=(m/420)[[156,22L,54,−13L],
                 [22L,4L²,13L,−3L²],
                 [54,13L,156,−22L],
                 [−13L,−3L²,−22L,4L²]]
```

El bloque Hermite se inserta en los mismos planos y con el mismo D que K.
Masa concentrada: m/2 en cada traslación, p/2 en rx de ambos nodos;
ry/rz sin masa propia concentrada. rho=0 devuelve None tras validar geometría.
Con liberaciones Mc=HᵀMH (Guyan), no Schur de M. Es una reducción estática
para pequeñas oscilaciones a carga fija, no dinámica exacta de giros internos.
En doble torsión el gauge de giro común cero excluye ese mecanismo de la masa
nodal; los giros desconectados deben eliminarse o restringirse en el sistema.
Las pruebas comprueban masa física rho A L en cada dirección global, simetría,
congruencia de rotación e integrales independientes.

### Kg y productor poseído

El incremento de longitud de referencia debido a pendientes es
½∫(v′²+w′²)dx. La segunda variación del trabajo de N constante da:

```text
Bg=N/(30L) [[36,3L,−36,3L],
           [3L,4L²,−3L,−L²],
           [−36,−3L,36,−3L],
           [3L,−L²,−3L,4L²]]
```

Se inserta Bg en ambos planos, usando D en w. Tracción estabiliza;
compresión desestabiliza. No hay Kg axial ni torsional en esta formulación.
N proviene de desplazamientos axiales totales, no de las cargas de tramo:
`geometric_stiffness` no recibe cargas y no infiere un N(x) variable.
Kgc=HᵀKgH es la derivada del Schur de K+λKg en λ=0 por estacionariedad,
no su condensación exacta a N finito. Geometría fija, sin corrotación.

`geometric_producer` sigue el adaptador poseído de truss: captura el elemento,
coords y params; valida 12 locations distintos; reconstruye ElementCtx y
llama Kg con estado vacío en cada evaluación del desplazamiento global total.
El sistema valida el rango de location al registrar; el productor valida tamaño
(ENG-E-0010), repetición (ENG-E-0002), geometría y finitud.
Se prueba tanto aislado como registrado en LinearSystem.

### Cargas uniformes y recuperación

El contrato MemberLoad expresa q y m globales. Se rotan por R; `global_load`
convierte intensidades **locales** a un MemberLoad global para usar todas las
operaciones del contrato sin nuevas variantes ni cambios en element/1.
Trabajo externo: ∫(qx u+qy v+qz w+mx rx−my w′+mz v′)dx:

```text
f_i=[qxL/2, qyL/2−mz, qzL/2+my, mxL/2, −qzL²/12, +qyL²/12]
f_j=[qxL/2, qyL/2+mz, qzL/2−my, mxL/2, +qzL²/12, −qyL²/12]
```

Se condensa fc=Hᵀf; las acciones de empotramiento perfecto son −fc.
Se soportan las seis intensidades, sin omitir momentos distribuidos.
`self_weight(g)` calcula q=rho A g global, exige g explícita y admite −Z.
Sean pi las seis acciones locales del extremo i; por equilibrio en [0,x]:

```text
N=−pi_u−qx x          Vy=−pi_v−qy x         Vz=−pi_w−qz x
T=−pi_rx−mx x
My=−pi_ry−pi_w x−qz x²/2−my x
Mz=−pi_rz+pi_v x+qy x²/2−mz x
```

Cumplen F′=−q y M′=−x×V−m. En x=0 son los opuestos de las acciones
de la cara negativa i; en x=L coinciden con las acciones de cara positiva j.
Se preservan orden y duplicados de estaciones x/L en [0,1]. No se entrega
la flecha Hermite interior como solución exacta de una carga uniforme cuártica.

## Pruebas y tolerancias

Fixtures propios SI: E=200, G=80, A=2, J=1.5, Iy=3, Iz=4, rho=3, L=4.
`source` y `citation` remiten a estas derivaciones, sin citas externas pendientes.

- Ménsula con Py=12, Pz=−15 y T=9: δy=PyL³/(3EIz),
  δz=PzL³/(3EIy), rx=TL/(GJ), ry=−PzL²/(2EIy), rz=PyL²/(2EIz).
  Ensamblaje con DofMap/LinearSystem, reacciones y seis esfuerzos en estaciones.
- Biempotrada uniforme en ambos planos: dos elementos y nodo interior libre;
  reacciones −qL/2 y momentos de empotramiento de magnitud qL²/12.
  Axial uniforme, torsor y pares transversales distribuidos; equilibrio de caras.
- Biarticulada uniforme: reacciones −qL/2, momentos extremos cero,
  flector central de magnitud |q|L²/8. Articulada-empotrada en ambos sentidos:
  reacciones 3|q|L/8 y 5|q|L/8, momento fijo |q|L²/8. Torsión liberada
  en cada extremo con carga uniforme: reacción −mxL en el conservado.
- Euler biarticulada en ambos planos: Kφ=P(−Kg)φ, N=−1 a partir de u.
  Pcr=π²EI/L². Un Hermite da 12EI/L² (+21.59%); dos dan error <0.8%;
  cuatro <0.06%. Rotaciones libres de apoyo conservadas, sin condensarlas.
  Se comprueban residual del autovector y valor cerrado de un elemento.
- Kit existente `check_element` y `check_element_factory`, sin modificar:
  16 combinaciones de liberaciones isotrópicas (Iy=Iz y My/Mz pareadas por
  extremo). El kit rota coordenadas y u pero **conserva self y ctx.params**:
  no puede representar una rotación completa de una orientación propietaria
  de una sección anisótropa. Exigir su covarianza parcial para Iy≠Iz cambiaría
  físicamente el roll de la sección; no es una propiedad del modelo original.
- Para Iy≠Iz: pruebas propias de las 64 combinaciones, rotación general de
  Rodrigues del modelo **incluida orientation**, K'=Qe K Qeᵀ, f, masa, Kg,
  extremos, estaciones y giros internos; también traslación rígida de coordenadas.
  Tangente por FD central en los 12 GDL, h=1e-7 y repetibilidad.
- K simétrica, seis modos rígidos K u≈0 y masa total por eje global para las
  64 combinaciones. Schur independiente con backend, cargas y giros recuperados;
  integrales polinómicas independientes de energía elástica/cinética/geométrica.
- E y G de 1e200: ninguna combinación entra en pánico, flexión doble liberada
  sin cortantes espurios; doble torsión con carga incompatible genera diagnóstico.
- Negativos: rangos, no finitos, productos subdesbordados/desbordados,
  coordenadas/longitud, dimensión, historia y tamaños de vectores, cargas,
  orientación cero/paralela/casi paralela; JSON, modelo y locations inválidos.
  Orientaciones de escala 1e-310 y f64::MAX producen la misma terna.

Tolerancia de comparaciones SI de estos fixtures:
|a−b|≤1e-9+1e-11|b|, aplicada a cada entrada con su unidad correspondiente.
FD: error absoluto <1e-7+1e-6|Kij|; kit h=1e-7 y tolerancias originales.
Los umbrales de discretización de Euler no son tolerancias del solver.

## Estructura, diagnósticos y límites

Nuevo crate 0.1.0, miembro del workspace; dependencias **path internas** a
core/contracts/laws/linalg/assembly, dev a model. Sin dependencias externas
nuevas. Módulos privados factory, geometry, stiffness, release, loads, mass,
geometric, recover y producer; API/validadores en lib.rs, `deny(missing_docs)`.
Doc comments en español. No se cambian contratos, catálogo, frame2d ni truss.

E,G,A,J,Iy,Iz y productos EA/GJ/EIy/EIz deben ser positivos finitos, rho≥0.
Se validan resultados, diferencias, sumas, proyecciones y coeficientes requeridos.
ENG-E-0008 para entradas/resultados no finitos y overflow; ENG-E-0010 para
rangos/tamaños, coeficientes positivos subdesbordados a cero y orientación
inválida; ENG-E-0005 para L≤1e-12; ENG-E-0006 para torsión doble liberada
sin equilibrio. No hay unwrap, expect ni pánicos explícitos en código de librería.
Las estaciones usan reserva fallible; no hay E/S, hilos, reloj ni estado global.
No se garantiza todo el rango extremo de f64 ni igualdad binaria entre destinos.

Modelo homogéneo lineal, Euler–Bernoulli sin deformación de corte ni alabeo,
geometría de referencia fija. Sin solver no lineal dentro del elemento.
Masa/Kg condensadas son aproximaciones declaradas; no se regularizan mecanismos.
La ejecución y comparación numérica en runtime wasm queda PENDIENTE;
se comprueba compilación al target instalado.

## Verificación local

Resultados finales:

- `cargo fmt` y `cargo fmt --check`: satisfactorios.
- `cargo clippy --all-targets -- -D warnings`: satisfactorio, con
  `/home/box/.cargo/bin` en PATH (componente instalado de stable).
- `cargo test --workspace`: satisfactorio; 164 pruebas unitarias/de integración
  y 35 doctests. El crate nuevo aporta 15 pruebas de integración y un doctest.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: satisfactorio.
- `cargo check --workspace --target wasm32-unknown-unknown`: satisfactorio,
  con `/home/box/.cargo/bin` en PATH y target ya instalado; no se descargó nada.
- Tests y rustdoc nativos con cargo/rustc 1.85.1 del sistema, compatibles con
  rust-version=1.85; Clippy y wasm con stable 1.99.0 instalado.
- `bash clean-room/check.sh origin/main`: satisfactorio. El script solo ve
  rutas conocidas por Git; también se ejecutó con GIT_INDEX_FILE y
  GIT_OBJECT_DIRECTORY temporales en /tmp y objetos originales como alternativos
  de solo lectura, incorporando todos los archivos nuevos al diff revisado.
  El índice y objetos reales del usuario se conservaron intactos.
- `git diff --check`: satisfactorio, también sobre el diff completo de nuevos
  archivos mediante el índice temporal.
- Inspección final: código de librería sin unwrap/expect/pánicos explícitos.
  Solo cambian Cargo.toml, Cargo.lock, el crate nuevo y esta bitácora;
  ninguna modificación a crates existentes, contratos o clean-room/**.

Durante la primera iteración, el fixture biempotrado dejó todos los GDL
restringidos, situación que LinearSystem rechaza explícitamente; se subdividió
analíticamente en dos elementos con nodo interior libre. También se detectó
la limitación de rotación parcial del kit descrita arriba y se separó la
conformidad isotrópica de la verificación propia del modelo anisótropo completo.
Los resultados finales corresponden a los fixtures corregidos.

Ver PR #44. Revisión clean-room independiente: PENDIENTE (J4).
Ejecución numérica en runtime wasm: PENDIENTE.

## Ronda J4 r6

- Convención (decidida por J0): el vector de orientación es la referencia del eje local z (referencia del eje z local, plano x-z); y = z×x; coincide con +Z en 2D. Fijado en docs/CONVENCION_EJES.md.
- geometry.rs: se rechaza sen(ángulo orientación–x) ≤ 1e-8 y se reortogonaliza la terna (Gram–Schmidt).
- release.rs: equilibrio de torsión con tolerancia relativa 1e-12·máx|f|.
- stiffness.rs: axial robusto a subdesbordamiento de E·ε.
- Pruebas: tests/r6.rs.
