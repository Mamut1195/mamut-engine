# Bitácora clean-room · ENG-ELM-002

- Ticket / PR: ENG-ELM-002, «Viga-columna elástica 2D con liberaciones»;
  rama existente `j2/eng-elm-002-frame2d`, creada desde main con ENG-ELM-001
  fusionado. Entrega de archivos, sin commit, push ni PR.
- Autor(es): Codex, agente basado en GPT-6 · 2026-10-11
  (America/Guayaquil). No se delegó a otros agentes.
- Rol: implementador; especificación, derivaciones y fixtures analíticos
  propios. Sin actividad de operador de oráculo ni auditor de similitud.
- Contrato implementado: `element/1` versión 1.0, kind
  `frame2d.euler_elastic`, y `engine_contracts::ElementFactory`.
  Crate/método 0.1.0; versiones de método y contrato independientes.
- Fuentes permitidas usadas: contratos y documentos internos enumerados
  abajo; derivaciones propias de energía, equilibrio e integración polinómica.
  Bibliografía exacta de resistencia de materiales/análisis estructural
  clásico (autor, título, edición, capítulo/sección/ecuación): **PENDIENTE**.
  No se consultó una fuente externa precisa: no se atribuyen páginas,
  ecuaciones ni ejemplos a un libro sin haberlos verificado.
- Derivaciones propias: [nota en esta bitácora](#derivaciones-propias),
  [fixtures](../crates/engine-elements-frame/tests/fixtures/mod.rs),
  [estática y pandeo](../crates/engine-elements-frame/tests/analytical.rs),
  [invariantes](../crates/engine-elements-frame/tests/invariants.rs).
- Código de terceros copiado: ninguno. Código propio y dependencias
  internas por path; sin paquetes externos nuevos ni listados de libros.
- Validación: exclusivamente analítica, por soluciones cerradas e integrales
  de energía. Cada fixture declara `source`, `citation` y tolerancias;
  bibliografía exacta **PENDIENTE**, con enlace a las derivaciones propias.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: J4, **PENDIENTE**.

## Documentos internos consultados

Antes de implementar se leyeron `/workspace/motor-rust/04_TICKETS.md`
(ENG-ELM-002), `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`
(§4.4 element/1), `docs/CLEAN_ROOM.md`, `docs/CONVENCION_EJES.md`,
`provenance/TEMPLATE.md`, `provenance/ENG-ELM-001.md`, y estructura,
implementación y pruebas de `engine-elements-truss`.
Se consultaron traits, tipos, ejemplos y kit de conformidad de
`engine-contracts`; ley uniaxial y sección elástica de `engine-laws`;
catálogo de errores de `engine-core`; matrices/backend de `engine-linalg`;
mapa, contribuciones, restricciones, sistema y reacciones de `engine-assembly`.
No se encontró AGENTS.md aplicable. No se consultaron fuentes externas,
código de otros motores ni resultados externos; no se utilizó la red.

La convención aprobada prevalece: D2 está en X-Y horizontal con Z=0,
local x de i a j, local y=(-s,c), local z=+Z. En el fixture de pórtico,
«altura» es una dimensión en Y dentro de este plano matemático; no redefine
Y como vertical. La elevación 2D y su gravedad siguen pendientes.

## Derivaciones propias

### Geometría, interpolación y rigidez

Con r=x_j−x_i, L=hypot(r_x,r_y), c=r_x/L, s=r_y/L, cada bloque
global→local de desplazamientos es [[c,s,0],[-s,c,0],[0,0,1]].
Denotamos T al bloque diagonal de ambos nodos. Se calculan f_global=Tᵀf_local
y K_global=TᵀK_local T; la misma congruencia se aplica a masa y Kg.

Orden local d=[u_i,v_i,theta_i,u_j,v_j,theta_j], theta=v′. Sea t=x/L.
u=(1−t)u_i+t u_j. Para v se usan las formas cúbicas que interpolan
valor y derivada en los extremos:

```text
h1 = 1−3t²+2t³       h2 = L(t−2t²+t³)
h3 = 3t²−2t³         h4 = L(−t²+t³)
v  = h1 v_i + h2 theta_i + h3 v_j + h4 theta_j
```

La energía es ½∫[EA u′²+EI v″²]dx. N=EA(u_j−u_i)/L, positivo en
tracción. El axial se obtiene mediante `engine-laws::uniaxial::Elastic`,
sin sustituir una ley por otra implícitamente. I es Iz, la inercia alrededor
de z local; EI coincide con la rigidez de flexión D2 de la sección elástica
del repo. No se fabrican G, Iy o J para construir una sección 3D innecesaria.

La integral de las segundas derivadas de h da, en [v_i,theta_i,v_j,theta_j]:

```text
Kb = (EI/L³) [[12,   6L, −12,   6L],
              [6L, 4L², −6L,  2L²],
              [−12,−6L,  12,  −6L],
              [6L, 2L², −6L,  4L²]]
Ka = (EA/L) [[1,−1], [−1,1]] en [u_i,u_j].
```

La tangente es constante, f_int=Kc u, sin cargas ni Kg añadidas ocultamente.
La rigidez no incluye deformación de corte, grandes giros, actualización
de geometría ni historia. Las tres formas de cuerpo rígido D2 (traslación
X, traslación Y y u=(-Y,X), theta=1) tienen u′=v″=0; K u_rígido=0.

### Liberaciones y recuperación de giros

Dividiendo d en componentes a conservadas y r liberadas, el equilibrio
del giro interno exige K_ra d_a+K_rr d_r=f_r. Por ello:

```text
d_r = −K_rr⁻¹ K_ra d_a + K_rr⁻¹ f_r
d = H d_nodal + d_carga
Kc = Hᵀ K H = Kaa − Kar Krr⁻¹ Kra (embebido en 6×6)
fc = Hᵀ f    = fa − Kar Krr⁻¹ fr
```

Las columnas correspondientes a giros nodales liberados son cero; los
giros del miembro son variables internas distintas de esos giros nodales.
La inversa se deriva analíticamente, no se implementa un solver adicional:
Krr=4EI/L para una liberación; con dos es (EI/L)[[4,2],[2,4]],
cuyo inverso es (L/EI)[[1/3,−1/6],[−1/6,1/3]].

- Solo i: theta_i=3(v_j−v_i)/(2L)−theta_j/2+f_theta_i L/(4EI).
- Solo j: theta_j=3(v_j−v_i)/(2L)−theta_i/2+f_theta_j L/(4EI).
- Ambos: cada giro sin carga es (v_j−v_i)/L, más las componentes
  Krr⁻¹[f_theta_i,f_theta_j].

Una sola liberación deja energía de flexión
3EI/(2L³)(v_j−v_i−L theta_fijo)², de rango uno.
Con ambos extremos liberados la curvatura de la interpolación sin cargas es
cero y **Kflexión=0 exactamente**. Se construye este Schur analítico para
evitar rigidez artificial debida a cancelación numérica. Este miembro es válido
como elemento articulado; las rotaciones nodales desconectadas deben eliminarse,
restringirse o recibir aportes de otros miembros. Si quedan libres sin aportes,
el backend informa mecanismo ENG-E-0006. No se regularizan matrices.

`local_displacements` devuelve los seis desplazamientos locales del miembro,
incluyendo giros liberados y todas las cargas suministradas. `recover` usa
esos giros y K completa para obtener K d−f. Los momentos liberados se fijan
a cero después de resolver su equilibrio, eliminando solo redondeo.
Se verifica contra una condensación independiente con `ReferenceBackend`:
Kc, fc, masas, Kg y acciones recuperadas para las tres liberaciones.

### Masa

La energía cinética es ½rho A∫[u̇²+v̇²]dx, con u lineal y v Hermite.
No se añade rho I: las inercias de giros siguientes provienen exclusivamente
de la velocidad transversal interpolada, no de una inercia rotatoria de sección.
Con m=rho A L:

```text
Ma = (m/6) [[2,1],[1,2]]
Mb = (m/420) [[156,   22L,  54,  −13L],
               [22L,  4L², 13L,  −3L²],
               [54,   13L, 156,  −22L],
               [−13L,−3L²,−22L,   4L²]]
M_lumped = (m/2) diag(1,1,0,1,1,0)
Mc = Hᵀ M H
```

La reducción estática de masa conserva energía cinética en el subespacio
elástico condensado (reducción de Guyan); no se calcula un Schur de M,
que implicaría equilibrio inercial independiente y eliminaría otra física.
Es una aproximación dinámica: no resuelve giros internos con inercia a frecuencia
finita. Se excluye la parte afín dependiente de la carga de la transformación
de velocidades, apropiado para la masa de pequeñas oscilaciones a carga fija.
Para dos liberaciones, las formas transversales se vuelven lineales y Mb
condensada en [v_i,v_j] es (m/6)[[2,1],[1,2]].
La masa física se verifica por vᵀMc v=m para una traslación unitaria
uniforme en **cada** dirección global; sumar traza o todos los ejes no es
la masa física. rho=0 devuelve None, validando antes la geometría.

### Rigidez geométrica de viga-columna

Una pendiente transversal infinitesimal produce elongación adicional
½∫v′²dx. El trabajo del axial N de referencia tiene segunda variación
N∫δv′²dx. Integrando las primeras derivadas de las cuatro formas h:

```text
Kgb = (N/(30L)) [[36,   3L, −36,   3L],
                 [3L, 4L², −3L,  −L²],
                 [−36,−3L,  36,  −3L],
                 [3L, −L², −3L,  4L²]]
```

El resto de Kg local es cero. Es simétrica; tracción estabiliza, compresión
desestabiliza. N procede de u_e del elemento, no de un parámetro externo ni
de las cargas de tramo (el contrato geometric_stiffness no recibe cargas).
No se infiere un campo axial variable por q_x: se usa N constante compatible
con el desplazamiento axial total. El orden de acumulación es fijo.

Para liberaciones se utiliza **Kgc=HᵀKgH**, con la transformación de la
rigidez elástica. En efecto, para A(t)=K+t Kg, la energía estacionaria respecto
de d_r tiene derivada en t=0 igual a ½d_aᵀHᵀKgH d_a: la variación de
los giros internos se cancela por estacionariedad. Es la primera variación del
Schur de K+tKg, consistente con esta formulación inicial de tensiones.
**No** es el Schur de Kg separado (singular en N=0) ni la condensación exacta
dependiente de N de K+Kg a carga finita. Para esta última haría falta una
evaluación conjunta de tangente y cargas, fuera del elemento lineal actual.
Con dos liberaciones Kgc transversal es (N/L)[[1,−1],[-1,1]], pues v es lineal.

La prueba de matrices integra polinomios independientemente, verificando
∫h″ᵀh″, ∫hᵀh y ∫h′ᵀh′ contra K, M y Kg. La bibliografía precisa
de viga-columna para esta derivación propia queda **PENDIENTE**.

### Cargas uniformes y esfuerzos en estaciones

MemberLoad entrega fuerzas globales uniformes N/m y momentos uniformes N·m/m.
Se rotan fuerzas XY a qx,qy locales; mz conserva signo. qz,mx,my no nulos,
incluso arbitrariamente pequeños, se rechazan con ENG-E-0013. No se omiten
cargas. Para theta=v′, el trabajo externo es ∫(qx u+qy v+mz v′)dx:

```text
f = [qxL/2, qyL/2−mz, qyL²/12, qxL/2, qyL/2+mz, −qyL²/12]
f_empotramiento = −f; con liberaciones se usa −fc.
```

`self_weight(g)` construye q_global=rho A g con g explícita en m/s²,
rechazando g_z no nula. No presupone gravedad desde densidad ni eje Y.
En D2 horizontal el peso real hacia Z no tiene una representación en este
elemento; se informa la capacidad no soportada. No se cambia la convención.
Cargas puntuales interiores no están en MemberLoad: se modelan subdividiendo
el tramo y aplicando una carga nodal, como en el fixture biempotrado.

Sumando las intensidades de todas las cargas, sean p_i las tres acciones de
extremo local i obtenidas de K d−f. El equilibrio del segmento [0,x] da:

```text
N(x) = −p_i,x − qx x
V(x) = −p_i,y − qy x
M(x) = −p_i,theta + p_i,y x + qy x²/2 − mz x
N′=−qx, V′=−qy, M′=−V−mz.
```

En x=0 son los opuestos de las acciones en cara negativa i; en x=L coinciden
con las acciones por cara positiva j. Se entrega [N,V,0,0,0,M] en estaciones
x/L conservando orden y duplicados. Esta recuperación incluye el campo
parabólico debido a qy aun cuando todos los desplazamientos nodales son cero.
No se publica una flecha interior cúbica como solución exacta bajo carga
uniforme: para comprobar flechas se subdivide y se evalúan nodos interiores.

### Soluciones cerradas de estática y pandeo

Fixtures propios SI: E=200 Pa, A=2 m², I=3 m⁴, rho=3 kg/m³,
L=4 m, w=6 N/m hacia −Y, P=12 N. Son escalas de prueba, sin pretensión
de representar un material real. Las soluciones provienen de integrar
EI v''''=q, con continuidad en cargas puntuales y las condiciones de apoyo.

1. Voladizo: empotramiento i, fuerza −P en j.
   v_j=−PL³/(3EI), theta_j=−PL²/(2EI), R_y=P, R_z=PL;
   V=−P, M(x)=−P(L−x). Un elemento es exacto para esta deformada cúbica.
2. Biempotrada uniforme: v(x)=−w x²(L−x)²/(24EI),
   v(L/2)=−wL⁴/(384EI). Reacciones transversales wL/2 y momentos
   nodales (+wL²/12,−wL²/12). M de sección en extremos −wL²/12,
   en centro +wL²/24. Dos elementos permiten verificar la flecha central.
3. Biempotrada con fuerza central −P mediante subdivisión en dos:
   v(L/2)=−PL³/(192EI); reacciones w se sustituyen por P/2,
   momentos nodales (+PL/8,−PL/8).
4. Articulada en ambos extremos: v(L/2)=−5wL⁴/(384EI),
   R_y=wL/2 en cada apoyo, Mextremos=0, Mcentro=wL²/8,
   theta_i=−wL³/(24EI). Dos elementos, liberación i del primero y j del
   segundo. Los giros exteriores nodales desconectados están restringidos
   y el giro físico se recupera mediante equilibrio.
5. Apoyada-empotrada y su inversión: para i empotrado y j articulado,
   v(x)=−w x²(3L²−5Lx+2x²)/(48EI),
   v(L/2)=−wL⁴/(192EI), R_i=5wL/8, R_j=3wL/8,
   momento nodal empotrado +wL²/8. La inversión permuta reacciones y
   cambia el signo del momento nodal. Ambas liberaciones se prueban.
6. Pórtico simétrico: dos columnas de longitud h=3 m y viga de luz b=4 m,
   bases empotradas, P/2 en Ux de cada nodo superior, mismas propiedades.
   Se incluyen deformaciones axiales finitas de columnas. Por simetría,
   desplazamientos superiores (delta,+v,theta), (delta,−v,theta).
   El equilibrio vertical izquierdo y el equilibrio de momentos dan:

   ```text
   beta = (12EI/b²)/(EA/h+24EI/b³), v=−beta theta
   D = 4EI/h+6EI/b−12EI beta/b²
   theta = −6EI delta/(h² D)
   P = 24EI delta/h³+12EI theta/h²
   delta = P/[24EI/h³−72(EI)²/(h⁴ D)]
   ```

   Se verifican delta,v,theta y reacciones con ensamblaje real y backend,
   sin reemplazar la deformación axial por una rigidez infinita.
7. Euler biarticulada: EI v″+P v=0, v(0)=v(L)=0,
   Pcr=pi²EI/L². Rotaciones nodales libres en los apoyos son la condición
   natural M=0. Se ensamblan K y −Kg con axial unitario N=−1 N obtenido
   de u_j−u_i=−L_elemento/(EA). Se eliminan traslaciones axiales y
   transversales apoyadas; el backend resuelve K phi=P(−Kg) phi.
   Un elemento cúbico da Pcr=12EI/L² (error de discretización +21.59%);
   dos elementos dan aproximadamente 9.94384680 EI/L² (+0.7522%).
   Tolerancias relativas explícitas: 22% y 0.8%, respectivamente.
   No se pretende exactitud de Euler con un único campo cúbico.
   Una liberación doble en un solo elemento con traslaciones fijas elimina
   todos los GDL de flexión y no puede representar un modo de pandeo interior;
   no se usa esa idealización para atribuir una carga crítica finita.

## Decisiones y límites

1. Nuevo crate `engine-elements-frame` 0.1.0, miembro del workspace.
   Dependencias runtime path a core/contracts/laws/linalg y dev path a
   assembly/model. Sin dependencias externas nuevas ni campo license ni LICENSE.
   Cargo.lock únicamente incorpora el paquete nuevo y sus enlaces internos.
2. Módulos pequeños privados geometry, stiffness, mass, geometric, loads,
   release, recover, factory; API y validadores en lib.rs.
   `#![deny(missing_docs)]`, SI documentado, sin hilos/fs/reloj ni estado global.
3. Propiedades en self, geometría en ctx: ctx.params no interviene, como en
   la plantilla de barra. Fábrica con E,A,I,rho SI explícitos y release_i/j
   booleanos opcionales (false por defecto); rechaza campos desconocidos,
   tipos/rangos inválidos y ModelView fuera de D2, ids duplicados o coordenadas
   no finitas/fuera del plano. Sin ids de sección ni referencias JSON.
4. L por hypot; diferencias y productos validados. L≤1e-12 m usa ENG-E-0005;
   no finitos/desbordamientos ENG-E-0008; tamaños de u_e, historia no vacía,
   estaciones fuera de [0,1], plano y parámetros ENG-E-0010. Propiedades
   E,A,I y EA/EI positivos; rho no negativa. Coeficientes positivos elásticos
   y de masa subdesbordados a cero se rechazan cuando se necesitan.
   No se garantiza todo el rango extremo de f64 ni igualdad binaria entre
   destinos. La geometría se valida incluso con rho=0.
5. Kg se entrega separada de respond; derivación propia de viga-columna y
   condensación inicial de tensiones documentadas arriba. No se implementa
   solver P-Delta, estabilidad exacta a axial finito ni masa dinámica exacta
   con liberaciones. N no incluye cargas de tramo en geometric_stiffness.
6. Ensamblaje estático real usa ElementContribution, DofMap, LinearSystem y
   ReferenceBackend. Pandeo ensambla K/Kg con assemble_matrix y resuelve con
   generalized_eigen del backend, sin implementar solvers/eigen adicionales.
   No se supone que SystemView actual reevaluará elementos o axiales: recibe
   contribuciones precalculadas y su Kg tiene estado de referencia explícito.
7. ENG-ELM-003 (3D), orientación espacial, torsión, flexión biaxial, materiales
   no lineales y deformaciones de corte: **fuera de alcance**.
8. Compilación wasm se comprueba si el target está instalado; ejecución en
   runtime wasm y validación numérica nativo/wasm: **PENDIENTES**.
   Semver inicial 0.1.0; cambios físicos/API/signos requieren revisión de fixtures
   y versión apropiada. No se cambia element/1 ni el catálogo de errores.

## Pruebas y tolerancias

- Cinco pruebas analíticas ensambladas: voladizo; biempotrada uniforme/puntual;
  articulada y propped en ambos sentidos; pórtico con axial finito; Euler 1/2
  elementos. source/citation en fixtures, bibliografía exacta PENDIENTE.
- Diez pruebas de invariantes/conformidad/negativos; una prueba rustdoc.
- Desplazamientos/giros: absoluto 1e-12 m/rad; reacciones/esfuerzos/equilibrio:
  absoluto 1e-9 N/N·m. Matrices, masa, rotaciones, Schur y recuperación:
  absoluto 1e-10 en las unidades SI de cada entrada de estos casos pequeños.
- Kit element/1 y factory: FD central h=1e-7 m/rad, error relativo tangente
  <1e-6 del kit; cuatro combinaciones de liberaciones, repetibilidad 10 veces,
  tamaños, pureza, simetría, modos rígidos y rotaciones globales no triviales.
- Integrales polinómicas de energía independientes para K, M y Kg;
  condensación de bloques con backend y comparación de energías/cargas/giros.
- Masa total rho A L por dirección para ambas masas y todas las liberaciones;
  doble liberación con Kflexión exactamente cero, masa transversal lineal,
  Kg transversal N/L, momentos nulos y giros cargados recuperados.
- Kg/cargas/esfuerzos covariantes bajo rotación no trivial; recuperación N/V/M
  en extremos y estaciones desordenadas/duplicadas; cargas múltiples, uniformes
  axiales, transversales, momentos distribuidos y peso propio explícito.
- Euler: tolerancia relativa 22% (uno), 0.8% (dos); residual de autovector
  absoluto 1e-8 en unidades de la ecuación; valor de un elemento 12EI/L²
  absoluto 1e-9. Error de discretización documentado, no una tolerancia del solver.
- Negativos: E/A/I≤0, rho<0, propiedades no finitas, EA/EI subdesbordados,
  geometría nula/corta/no finita/fuera del plano, estados y tamaños inválidos,
  estaciones inválidas, cargas no finitas y fuera del plano incluso 1e-30,
  desbordamientos de productos y sumas, campos JSON ausentes/extra/tipos
  erróneos, liberaciones no booleanas, ids nodales duplicados y mecanismo de
  giros desconectados. Códigos del catálogo core sin añadir códigos nuevos.

## Verificación local

Resultados finales:

- `cargo fmt --check`: satisfactorio.
- `cargo clippy --all-targets -- -D warnings`: satisfactorio.
- `cargo test --workspace`: satisfactorio; 129 pruebas unitarias/de integración
  y 34 doctests del workspace, incluidas 15 pruebas y 1 doctest del crate nuevo.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`: satisfactorio.
- Target wasm32-unknown-unknown instalado: `cargo check --workspace --target
  wasm32-unknown-unknown`: satisfactorio con `/home/box/.cargo/bin` en PATH
  (toolchain stable, rustc 1.99.0). `cargo check -p engine-elements-frame
  --tests --target wasm32-unknown-unknown`: satisfactorio. No se ejecutaron
  fixtures en runtime wasm: ejecución numérica cruzada **PENDIENTE**.
- Las pruebas y rustdoc nativas se ejecutaron también con cargo/rustc 1.85.1
  del sistema, compatible con rust-version=1.85. Clippy usa el componente
  instalado del toolchain stable. No se instalaron herramientas ni targets.
- `bash clean-room/check.sh origin/main`: satisfactorio.
  El script solo examina rutas conocidas por Git; adicionalmente se ejecuta
  con `GIT_INDEX_FILE` y `GIT_OBJECT_DIRECTORY` temporales en /tmp, y objetos
  existentes como alternativos de solo lectura, incorporando los archivos
  nuevos para revisar el diff completo. El índice real del usuario no se
  modifica. La primera preparación con solo índice temporal encontró el
  directorio de objetos del repo de solo lectura; se repitió aislando también
  los objetos, sin solicitar permisos ni modificar .git.
- `git diff --check`: satisfactorio, también para el diff con archivos nuevos
  mediante el índice temporal. Rutas cambiadas: Cargo.toml, Cargo.lock,
  crate nuevo y esta bitácora; ningún otro crate ni ruta protegida.

En la iteración inicial rustdoc detectó tres vectores escritos con corchetes
sin formato de código y los interpretó como enlaces; se corrigieron sus
delimitadores. Clippy señaló una aserción constante redundante de una prueba,
que se eliminó. Los resultados arriba corresponden a los archivos corregidos.

No se modifican clean-room/**, .github, CODEOWNERS ni docs/CLEAN_ROOM.md,
ni otros crates existentes. No se hace commit, push, PR ni se cambia el índice
Git del usuario. Revisión humana clean-room J4: **PENDIENTE**.
