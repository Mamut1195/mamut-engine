# Bitácora clean-room · ENG-ASM-001-KG

- Ticket / PR: productor real Kg(u), rama `j2/geometric-producer`, apilada sobre
  el cambio `has_geometric` de PR #29. Sin commit, push ni PR en este trabajo.
- Autor(es): agente Codex, GPT-6, 2026-10-11 (America/Guayaquil).
- Rol: implementador.
- Contrato implementado: system/1, `LinearSystem::geometric`; adaptador de
  element/1, kinds `truss2d.linear` y `truss3d.linear`.
- Fuentes permitidas usadas: ninguna externa. Solo código y contratos del árbol
  local; sin red ni consulta bibliográfica.
- Derivaciones propias: las secciones de esta misma nota.
- Código de terceros copiado: ninguno.
- Validación: analítica propia por energía, segunda variación de longitud,
  equilibrio y diferencias centradas de fuerza analítica incremental.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: no consulté código ni soluciones de motores externos para este ticket.
- Revisor clean-room: J4, **PENDIENTE**.
- Cita bibliográfica precisa para P-Delta de barras: **PENDIENTE**; no se
  atribuyen páginas ni ecuaciones a publicaciones no consultadas.

## Frontera, identidad y atomicidad

El tipo `GeometricProducer` posee `location: Vec<DofIndex>` y una clausura
`Box<dyn Fn(&[Real]) -> EResult<DMat>>`, sin Send/Sync. Su resultado debe ser
determinista para cada u_e; no modifica historia. El registro valida índices
distintos, no vacíos y globales. Evalúa en la referencia reducida expandida,
incluidos los prescritos, y valida cada matriz por la misma ruta de ensamblaje
que los aportes estáticos: cuadratura, tamaño, finitud y simetría exacta.

La API conserva los literales `ElementContribution { .. }`. Por no disponer de
id elemental en esa API, la identidad del productor es la secuencia `location`
exacta, incluido el orden. Sustituye la Kg estática de su único aporte coincidente,
incluso si era None. Sin coincidencia agrega un bloque independiente. Dos
productores con la misma secuencia o varios aportes coincidentes se rechazan
con ENG-E-0002; no se elige silenciosamente entre elementos coincidentes.
Una permutación distinta no identifica el mismo aporte. Vaciar el registro
restaura las Kg estáticas recibidas originalmente.

Cada `geometric(u,out)` valida tamaño de out y expande u reducido finito con
`constraints.expand`, extrae u_e, evalúa productores en orden del registro,
ensambla en orden de aportes originales seguido de productores independientes,
y reduce a libres. Solo después sobrescribe out y actualiza trial. Un error
de evaluación, simetría, reducción o acumulación conserva out, trial y accepted.
Los errores del productor conservan su código, mensaje y entidades. Registrar
no modifica trial/accepted; un fallo conserva el registro y el patrón anterior.
El estado privado interno del backend sigue su propio contrato.

Si queda cualquier aporte Kg estático sin productor, se exige u exactamente
igual a la referencia (ENG-E-0013), incluso cuando Kg(ref)=0. Con todos los
aportes sustituidos se acepta cualquier u finito del tamaño correcto, sujeto a
los errores de evaluación de Kg; no se calcula Ku para validar Kg, evitando
rechazos por desbordamientos ajenos a esta operación. Tamaño: ENG-E-0010;
desplazamiento no finito: ENG-E-0008.

El patrón une K, masas, Kg original y todos los pares libres de cada productor,
incluidos sus ceros. Cambiar el registro reconstruye esta unión. La API
`global_geometric()` mantiene su instantánea estática original; la Kg vigente
reducida se consulta mediante `SystemView::geometric`. La referencia declarada
solo etiqueta las instantáneas estáticas, no almacena una nueva matriz.

La única modificación de dependencias mueve la dependencia interna existente
de truss hacia assembly desde dev-dependencies a dependencies. No se agrega
ningún paquete al grafo ni se cambia Cargo.lock. El crate
`engine-elements-frame` no está en este árbol (PR #31 no fusionado): no se toca.

## Unidades y segunda variación propia de longitud

Sean r=x_j−x_i, L=|r|, d=r/L, q=u_j−u_i y k=EA/L. Coordenadas y
desplazamientos en m; E en Pa; A en m²; EA y N en N. Para una perturbación
incremental v de r:

```text
l(v) = |r+v|
dl(0)[v] = d·v
d²l(0)[v,v] = (|v|²−(d·v)²)/L = vᵀ(I−ddᵀ)v/L.
```

El elemento lineal calcula N(u_e)=k d·q, positivo en tracción. El trabajo
incremental del axial aporta N d²l, de donde:

```text
P = I−ddᵀ
Kg(u_e) = (N(u_e)/L) [ P  −P ]
                         [−P   P ]
K        = k             [ ddᵀ  −ddᵀ ]
                         [−ddᵀ   ddᵀ ].
```

Kg es simétrica y transversal al eje; tiene unidades N/m para estas barras.
En compresión N<0 reduce la rigidez transversal; en tracción la aumenta.
El adaptador llama `LinearTruss::geometric_stiffness` con el u_e recibido y
estado comprometido vacío, reconstruyendo `ElementCtx` con geometría/params
poseídos. D2 sigue Ux,Uy por nodo; D3 Ux,Uy,Uz. No actualiza r,L,d con u.
K/Kg generales con giros tienen unidades N/m, N o N·m según los componentes.

## Barra axial de referencia nula

Con r=(2,0[,0]) m, E=200 Pa, A=2 m² y desplazamiento axial δ del nodo j:
k=200 N/m, N=200δ N, Kg transversal=100δ N/m. Para δ=0.06 m se obtiene
N=12 N y Kg_yy=6 N/m (también Kg_zz en D3); para δ=−0.04 m se obtiene
−4 N/m. El desplazamiento transversal no modifica el axial proyectado.
Kg(0)=0 coincide con la instantánea estática; sin productor, una consulta
δ≠0 conserva ENG-E-0013 aunque esa instantánea sea nula.

Con u_ix=−0.06 m prescrito y u_j reducido nulo, la extensión es +0.06 m:
Kg(0 reducido)_yy=6 N/m. Se compara también contra la instantánea estática
evaluada con ese mismo u global: coincidencia no nula.

## Arco plano simétrico de dos barras: rigidez incremental y singularidad

Las bases (−a,0) y (+a,0) están fijas; el vértice es (0,h). Este fixture usa
−Y como dirección de la carga vertical local; no cambia la convención global
del motor. Definimos L=sqrt(a²+h²), c=a/L, s=h/L, k=EA/L.
Los ejes hacia el vértice son (c,s) y (−c,s). Para el estado reducido
u=(0,w), N=k s w en ambas barras. Con w<0 son comprimidas y la carga que
equilibra el esfuerzo axial en la configuración fija es P=−2Ns sobre −Y.
Las matrices del vértice son:

```text
K  = 2k    diag(c²,s²)
Kg = 2N/L  diag(s²,c²)
Kt = K+Kg.
```

Para validar físicamente esta suma se considera una perturbación incremental
v del vértice, alrededor de la configuración en la que se evalúan esas matrices.
Por barra se usa una energía local con axial inicial N y módulo k constante:

```text
l(v) = sqrt((±a+v_x)²+(h+v_y)²)
V(v) = N(l(v)−L) + (k/2)(l(v)−L)²
f(v) = [N+k(l(v)−L)] (±a+v_x,h+v_y)/l(v).
```

Equivalentemente, l0=L−N/k y f(v)=k(l(v)−l0)r(v)/l(v).
Esta expresión analítica de fuerza usa longitudes exactas bajo la perturbación;
su derivada en v=0 es k ddᵀ+(N/L)(I−ddᵀ). Sumando las dos barras da Kt.
El test calcula f(v) independientemente de las rutinas de matrices y compara
diferencias centradas en ambas direcciones con K+Kg(u). Este es un análisis
local de barras preesforzadas cuyo N proviene del u_e lineal proyectado.
No es la derivada de `LinearTruss::respond` ni de `LinearSystem::internal_force`:
esas operaciones siguen siendo lineales, con derivada K. Tampoco representa
la trayectoria exacta de grandes desplazamientos desde una longitud natural
global inicial; tal trayectoria requiere actualizar geometría y ley axial.

Para el problema local de pandeo por multiplicador, K+λKg pierde rigidez en:

```text
λ_cr,y = −EA s²/(N c²)
λ_cr,x = −EA c²/(N s²).
```

Con a=2 m, h=1 m, EA=400 N, w=−0.1 m se tiene L=√5 m,
c²=4/5, s²=1/5, N=−8 N, P=16/√5 N. Resulta λ_cr,y=12.5,
λ_cr,x=200. El primer modo es vertical/transversal [0,1], con rigidez
horizontal positiva. A λ=12.5 la rigidez vertical se anula y cambia de signo
entre 0.99λ y 1.01λ. La carga crítica local es λ_cr P=200/√5 N.
Se verifica el residual de ese modo y el cambio de signo. No se presenta como
columna de Euler: las barras articuladas no tienen rigidez de flexión.

## Pruebas y tolerancias explícitas

- `engine-elements-truss/tests/geometric_producer.rs`: barra D2/D3 y proyector
  global D3 oblicuo con L=7 m, d=(2,3,6)/7 y N=4 N; cambio
  con N(u), instantánea nula y no nula con prescritos, contexto poseído,
  validación del adaptador, arco y conformance con muestras fuera de referencia.
  Fórmulas y residual crítico: tolerancia absoluta 1e-12 N/m (y 1e-12 en
  las magnitudes escalares comparadas). Fuerza incremental por diferencias
  centradas: paso 1e-5 m; error relativo <1e-6 tanto en norma máxima de la
  matriz completa como en el componente vertical separado.
- `engine-assembly/tests/geometric_producers.rs`: extracción ordenada de u_e,
  prescritos, sustitución sin doble conteo, simetría exacta, sobrescritura,
  mezcla de estática y productores, restauración al vaciar, patrón dinámico,
  Rc sin Send/Sync, entradas no finitas, tamaños/asimetrías, sumas desbordadas,
  orden de acumulación no asociativa y propagación de error tras evaluaciones
  parciales preservando out/trial. Valores binarios simples: igualdad exacta.
  `check_system_view` usa paso 1e-5 m y su umbral relativo contractual 1e-6.
- La regresión existente `zero_reference_geometric_still_rejects_other_states`
  conserva ENG-E-0013 sin productor aun con Kg(ref)=0.

## Verificación local

Ejecución final con toolchain `stable-x86_64-unknown-linux-gnu`, PATH solicitado
y `CARGO_NET_OFFLINE=true`, todos con salida 0:

```text
cargo fmt --all -- --check
cargo clippy --offline --workspace --all-targets -- -D warnings
cargo test --offline --workspace
RUSTDOCFLAGS='-D warnings' cargo doc --offline --workspace --no-deps
cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown
git diff --check
```

Pasaron 130 pruebas unitarias/de integración y 33 doctests en el workspace,
incluidas 13 pruebas nuevas (8 de ensamblaje y 5 de barras). La compilación de
tests para wasm también pasó; no se ejecutaron en un runtime wasm.

La ejecución del guard de clean-room contra origin/main se omite porque el
usuario exige no crear commits y ese guard se solicita tras commit. No se
modifican clean-room/**, .github, CODEOWNERS ni docs/CLEAN_ROOM.md.
