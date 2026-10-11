# Bitácora clean-room · ENG-ELM-001

- Ticket / PR: ENG-ELM-001, «Barra 2D/3D lineal»; rama existente apilada sobre
  ENG-ASM-001 (PR #20). Entrega de archivos, sin commit, push ni PR.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-11 (America/Guayaquil),
  identificación solicitada para este ticket. No se delegó a otros agentes.
- Rol: implementador; derivaciones y fixtures analíticos propios, sin actividad
  de operador de oráculo ni auditor de similitud.
- Contrato implementado: `element/1` versión 1.0, kinds `truss2d.linear` y
  `truss3d.linear`, y `engine_contracts::ElementFactory`. Material `elastic`
  de `engine-laws` (`uniaxial/1`), sin modificar su implementación. Crate/método
  0.1.0; las versiones del contrato y del método son independientes.
- Fuentes permitidas usadas: documentos y contratos internos enumerados abajo;
  derivaciones propias de elasticidad, energía y equilibrio. McGuire et al.,
  referencia sugerida para Kg por el ticket: **PENDIENTE: cita completa,
  edición, capítulo/sección y ecuación**. No se dispone de una fuente precisa
  consultada y no se atribuye la derivación a una página o ecuación inventada.
  Bibliografía exacta para estática/resistencia de materiales: **PENDIENTE**.
- Derivaciones propias: [nota en esta bitácora](#derivaciones-propias),
  [fixtures](../crates/engine-elements-truss/tests/fixtures/mod.rs) y
  [pruebas de invariantes](../crates/engine-elements-truss/tests/invariants.rs).
- Código de terceros copiado: ninguno. Código propio y dependencias internas
  por path; sin dependencias externas nuevas ni listados de libros copiados.
- Validación: exclusivamente analítica; soluciones cerradas de barra axial,
  triángulo isostático de tres barras y trípode espacial, masa por integración,
  segunda variación de longitud, equilibrio de cargas uniformes y conformidad.
  Todos los fixtures declaran `source` y `citation`, con bibliografía exacta
  **PENDIENTE** y referencia a las derivaciones propias de esta nota.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: J4, **APTO** (2026-10-11, comunicado vía J0). La fusión la decide J0.

## Documentos internos consultados

Antes de implementar se leyeron `/workspace/motor-rust/04_TICKETS.md`
(ENG-ELM-001, ENG-MAT-001, ENG-ASM-001 y límites de ENG-ELM-002/003),
`/workspace/motor-rust/00_PROYECTO.md`,
`/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md` (§3 y §4.4),
`docs/CLEAN_ROOM.md`, `docs/CONVENCION_EJES.md`, `provenance/TEMPLATE.md`,
`provenance/ENG-ASM-001.md` y `provenance/ENG-MAT-001.md`.
Se consultaron los traits, tipos, ejemplo de barra y kit de conformidad de
`engine-contracts`, `engine-laws::uniaxial::Elastic`, errores de `engine-core`,
`DMat` y backend de `engine-linalg`, y `ElementContribution`, `LinearSystem`,
mapa, restricciones y pruebas de `engine-assembly`. No se encontró AGENTS.md
aplicable en el repo ni sus directorios ascendientes. No se consultaron fuentes
externas, código de otros motores, listados de libros ni números de oráculo;
no se utilizó la red.

La convención aprobada prevalece sobre la descripción provisional de
arquitectura: D2 está en X-Y horizontal, Z=0; D3 es diestro con Z vertical.
La propuesta de elevación D2 sigue pendiente. Solo importa el eje local
x de i a j: no se inventa una orientación transversal D3.

## Derivaciones propias

### Geometría, material y rigidez

Sea r=x_j−x_i, L=||r|| y d=r/L, todos en geometría de referencia. Para a=2
en D2 o a=3 en D3, se ordena u_e=[u_i,u_j] con a traslaciones por nodo.
La elongación infinitesimal es Δ=d·(u_j−u_i)=bᵀu_e, b=[−d,d]. Entonces
ε=Δ/L, σ=Eε (evaluada por `Elastic::respond`) y N=Aσ, positivo en tracción.
La energía elástica U=EA Δ²/(2L) da f_int=N b y K=(EA/L)bbᵀ.
K es simétrica, positiva semidefinida de rango uno para EA>0, sin pretensión.
Se construyen las mitades copiando cada entrada para conservar simetría exacta.

Una traslación uniforme tiene Δ=0. Un giro infinitesimal ω tiene
u_j−u_i=ω×r y d·(ω×r)=0, de modo que los tres modos rígidos D2 y seis D3
no producen fuerza y K u_rígido=0. No se exige invariancia bajo giros finitos
de los desplazamientos en una teoría infinitesimal. Al rotar coordenadas y
componentes con Q ortogonal, d'=Qd, f'=Tf y K'=TKTᵀ, T=diag(Q,Q).

### Masa

Para rho constante se interpola la velocidad global con
H=[(1−s)I,sI], s=x/L. La energía cinética es
½rho A∫₀ᴸ v_eᵀHᵀH v_e dx. Las integrales de (1−s)², s² y s(1−s)
son 1/3, 1/3 y 1/6 respectivamente. Con m=rho A L:

```text
M_consistente = (m/6) [2I  I]
                       [ I 2I]
M_concentrada = (m/2) diag(I,I)
```

La masa física total se verifica con vᵀMv=m para una traslación unitaria
uniforme en CADA eje; sumar la traza o todos los ejes no define la masa física.
Ambas matrices son isotrópicas y covariantes. rho=0 devuelve None; nunca se
fabrica masa ni inercia de giro. Con rho=3 kg/m³, A=2 m², L=2 m, m=12 kg:
concentrada diagonal 6 kg, consistente diagonal 4 y acoplamiento 2 kg por eje.

### Rigidez geométrica

Considerando una variación relativa h, la expansión de ||r+h|| hasta orden
dos es L+d·h+(hᵀ(I−ddᵀ)h)/(2L). El trabajo de un axial de referencia N
produce la segunda variación N/L (I−ddᵀ) en h. Sustituyendo h=δu_j−δu_i:

```text
P = I−ddᵀ
Kg = (N/L) [ P −P]
             [−P  P]
```

Aquí N se obtiene del propio u_e total mediante la ley elástica; no se acepta
un axial externo ni se guarda historia. P proyecta al subespacio transversal.
Kg es simétrica, anula traslaciones y movimientos relativos axiales; tiene
rango a−1 cuando N≠0, rango cero cuando N=0. Sus a−1 autovalores no nulos
son 2N/L, positivos en tracción y negativos en compresión. La covarianza usa
el mismo T que K. Para barra de 2 m con EA=400 N y elongación ±0.06 m,
N=±12 N, N/L=±6 N/m y autovalores transversales ±12 N/m.

Esta es una derivación propia por segunda variación. La cita precisa sugerida
de McGuire et al. queda **PENDIENTE**. `respond` sigue siendo lineal con K
elástica consistente; Kg se ofrece por separado y no se suma ocultamente a K.
No se implementa equilibrio P-Delta, actualización de geometría ni pandeo.

### Cargas y recuperación

`MemberLoad` admite fuerza uniforme q_global N/m y momento uniforme N.
Para esta idealización se exige q_global=q d y momento cero. La integración
de las formas lineales da f_eq=[q L d/2,q L d/2], en GDL globales activos.
La carga transversal no se puede equilibrar dentro de una barra articulada
recta de teoría lineal sin flexión: se rechaza, también en `recover`, con
ENG-E-0010. No se proyecta silenciosamente una carga física transversal.
En D2 se rechaza siempre q_z≠0. El residuo de proyección admitido es solo
redondeo: ≤64ε máquina·max|q_global|, sin umbral absoluto; una carga transversal
pura arbitrariamente pequeña se rechaza. Los momentos deben ser cero exacto.

El peso propio requiere que el llamador entregue q_global=rho A g en N/m y
que esa fuerza sea axial. No se infiere g de los ejes o la densidad. Ejemplo
D3 vertical ascendente de 2 m, rho=3, A=2 y g=(0,0,−10) m/s²:
q_z=−60 N/m y cada nodo recibe −60 N en Uz. Una barra inclinada sometida a
gravedad transversal se debe cargar nodalmente mediante una idealización
explícita fuera de este método; el load_vector de este ticket la rechaza.
D2 horizontal no admite peso propio en Z.

Sea N_e=EAΔ/L y q la suma de las cargas uniformes axiales. Del equilibrio
dN/dx=−q y del promedio compatible N_e se obtiene
N(s)=N_e+qL(1/2−s). Las acciones de extremo conjugadas son
[−N_e−qL/2,0,…,N_e−qL/2,0,…]; signo de cara negativa en i y positiva en j.
Los vectores locales tienen a componentes por extremo y las estaciones
[N,0,0,0,0,0], sin ejes transversales arbitrarios. Se conservan orden y
duplicados de las estaciones, incluyendo extremos; todas deben estar en [0,1].
Invertir conectividad invierte d y q, permuta los bloques globales y conserva
N de la misma sección física cuando s pasa a 1−s.

Con N_e=12 N, q=8 N/m y L=2 m, estaciones solicitadas [1,0,0.5] dan
[4,20,12] N; extremos [−20,4] N cumplen equilibrio con la carga total 16 N.
En la barra vertical con u=0 y q=−60 N/m, N en [0,0.5,1] es [−60,0,60] N.

### Fixtures de estática y resistencia de materiales

Todos son elecciones analíticas propias en SI con E=200 Pa y A=2 m²;
no representan calibración de un material real ni un ejemplo atribuido a un
libro. Cada fixture tiene source y citation a esta bitácora; la referencia
externa exacta queda **PENDIENTE**.

1. Barra axial L=2 m, extremo i fijo y j restringido transversalmente,
   P=12 N: k=EA/L=200 N/m, δ=PL/EA=0.06 m, N=12 N y R_i=−12 N.
   Se resuelve mediante ASM y backend en ambas dimensiones.
2. Triángulo isostático D2: A=(0,0), B=(2,0), C=(1,1); barras AB, AC y BC.
   A restringe Ux,Uy y B restringe Uy. C recibe (6,−12) N. Por momentos en A,
   2R_By−12−6=0, así R_B=(0,9) N y R_A=(−6,3) N.
   En C: (−N_AC+N_BC)/√2+6=0 y −(N_AC+N_BC)/√2−12=0.
   Resultan N_AC=−3√2, N_BC=−9√2 N; por equilibrio B, N_AB=9 N.
   Compatibilidad: u_Bx=9·2/400=0.045 m,
   u_Cx=0.0225+0.015√2 m, u_Cy=−0.0225−0.03√2 m.
   Se verifica también rotación de 90° de todo el modelo, cargas y rodillo.
3. Trípode D3: bases fijas A=(1,0,0), B=(0,1,0), C=(−1,−1,0), ápice
   D=(0,0,1), barras AD, BD y CD, carga D=(6,−3,−12) N.
   Sean a=N_AD/√2, b=N_BD/√2, c=N_CD/√3. En D:
   a−c+6=0, b−c−3=0, −a−b−c−12=0; a=−9, b=0, c=−3 N.
   Axiales [−9√2,0,−3√3] N; reacciones A=(−9,0,9), B=(0,0,0), C=(3,3,3) N.
   Compatibilidad: u_Dz=−3√3/400−3√2/200, u_Dy=u_Dz,
   u_Dx=u_Dz+9√2/200 m. Se verifica también rotación de 0.73 rad en Y.
4. Barra axial con P=12 N terminal y q=4 N/m, L=2 m:
   f_eq=(4,4) N, F_j=16 N, δ=0.08 m, N_e=16 N, R_i=−20 N.
   N en [0,0.5,1]=[20,16,12] N. Se ensamblan K, ambas M y f_eq mediante
   ElementContribution. No se consulta Kg a través de SystemView (ver
   decisión 8).

Se verifican fuerzas y momentos de equilibrio global por dirección; no se
suman fuerzas de distintos ejes en una sola condición escalar.

## Decisiones y límites

1. Nuevo crate `engine-elements-truss` 0.1.0 y miembro del workspace.
   Dependencias de ejecución únicamente path a engine-core/contracts/laws/linalg;
   dependencias de prueba únicamente path a engine-assembly/model. Cargo.lock
   solo registra el paquete nuevo y esos enlaces; no añade paquetes externos.
   Fuera del crate nuevo cambian Cargo.toml, Cargo.lock, esta bitácora y el
   ajuste aditivo mínimo de portabilidad descrito abajo en engine-linalg.
2. Módulos privados pequeños geometry, stiffness, mass, geometric, loads,
   recover y factory; `lib.rs` reúne API y validadores. API pública documentada
   con `#![deny(missing_docs)]`; no hay E/S, hilos, reloj ni mutable global.
3. `LinearTruss::new(dim,E,A,rho)` conserva `Elastic`, A, rho y EA validados.
   Como permite el contrato y el ejemplo interno, propiedades en self y
   coordenadas en ctx. `ctx.params` no participa en estos kinds: no hay
   precedencia implícita entre propiedades propietarias y contexto. No se
   aceptan leyes arbitrarias/no lineales ni referencias JSON a materiales;
   la fábrica construye Elastic desde E. Parámetros resueltos genéricos para
   otros kinds siguen disponibles en contracts, sin modificar el trait.
4. La fábrica implementa el trait existente sin serde ni parser: objeto JSON
   en memoria con exactamente E, A, rho numéricos explícitos, rangos y unidades
   en el esquema. Valida dimensión, ids nodales únicos y coordenadas finitas
   de ModelView, y Z=0 en D2. No hay campos de referencias ni conexión en este
   esquema; ninguna geometría de barra se presume de ModelView. La geometría
   concreta se valida al evaluar ElementCtx.
5. Longitud acumulada con hypot, sin cuadrar coordenadas; se verifica finitud
   de diferencias y L. L≤1e-12 m usa ENG-E-0005, igual a la convención aprobada.
   D2 exige exactamente Z=0 y dos coordenadas; dimensión incompatible usa
   ENG-E-0010. Se validan coordenadas incluso cuando rho=0.
6. E,A>0; rho≥0, todos finitos. EA, EA/L y coeficientes positivos de masa
   reducidos a cero por subdesbordamiento se rechazan con ENG-E-0010 cuando
   se necesitan. NaN, infinitos y desbordamientos de geometría, material,
   fuerzas, rigideces, masas y cargas producen ENG-E-0008. Tamaños de u_e,
   historia no vacía, estaciones y cargas incompatibles usan ENG-E-0010.
   El método no garantiza todo el rango extremo de f64; errores numéricos
   explícitos son preferibles a matrices físicas nulas por subdesbordamiento.
7. Masa nula devuelve None; Kg siempre devuelve Some, incluso matriz cero
   para N=0. Historia inicial, aceptada y de prueba vacías; no hay estado
   escondido, aceptación implícita, deformación inicial ni pretensión.
8. La integración usa ElementContribution con matrices globales evaluadas.
   AVISO (J0, 2026-10-11): `SystemView::geometric(u)` de engine-assembly
   (system.rs) ignora u y devuelve el Kg precalculado, contra el contrato
   SystemView; este ticket no depende de ello (solo K, M, F) y otro PR lo
   corrige. La Kg del elemento se valida solo a nivel elemental (matriz y
   espectro). No se añade un adaptador
   dinámico ni un solver en este ticket. La solución se delega al backend
   existente, sin implementar eliminación, factorización o eigen nuevos.
9. ENG-ELM-002+ conserva flexión, corte, torsión, liberaciones, cargas
   transversales/momentos distribuidos y orientación transversal de vigas 3D.
   No linealidad material, geometría corrotacional, deformaciones grandes,
   estabilidad y solvers de segundo orden quedan fuera de este método.
10. Evolución semver desde 0.1.0: correcciones compatibles en parche;
    cambios en API, hipótesis físicas, parámetros/esquema o signos requieren
    una versión apropiada y revisión de fixtures. No se cambia element/1 ni
    se registran nuevos errores del núcleo. Determinismo binario cruzado
    nativo/wasm y ejecución de pruebas en runtime wasm siguen **PENDIENTES**.
11. La comprobación wasm descubrió E0080 previo en
    `engine-linalg/src/lib.rs`: `MAX_ALLOC_BYTES: usize = 1 << 33` no cabe
    en wasm32. Se añadió cfg para conservar exactamente la constante de
    64 bits y una constante alternativa para destinos menores con
    `isize::MAX as usize`, el límite de bytes de una reserva Vec. Se documentó
    el límite específico (2 GiB−1 en 32 bits). Es una corrección aditiva mínima
    necesaria para compilar las dependencias del elemento en wasm; sin cambios
    de API, dependencias, algoritmos ni comportamiento de 64 bits. No se cambia
    el resto de engine-linalg ni ningún otro crate existente. Las pruebas de
    dimensiones extremas existentes verifican rechazo sin reserva/pánico.

## Pruebas y tolerancias

Catorce pruebas de integración y un doctest nuevo. Cuatro pruebas analíticas
ensamblan elementos reales y resuelven con `ReferenceBackend` de engine-linalg.
Diez pruebas de invariantes comprueban ambos elementos y ambas fábricas.

- Fuerzas/reacciones de fixtures y ΣF: absoluta 1e-9 N por eje. También ΣM
  absoluta 1e-9 N·m; desplazamientos esperados absoluta 1e-12 m.
- Invariantes/matrices/espectros/cargas/recuperación: absoluta 1e-10 en la
  unidad SI de la magnitud en los fixtures pequeños.
- Kits `check_element` y `check_element_factory`: incremento FD 1e-7 m,
  criterio del kit tangente FD relativo <1e-6, estados puros, tamaños,
  simetría, modos rígidos, rotaciones no triviales y masa covariante.
- Simetría exacta verificada con SymMatrix; K u_rígido=0 explícito para todos
  los modos infinitesimales; masa total por eje y entries analíticas completas.
- Kg: espectro obtenido con backend existente y M=I; autovalores 0 y 2N/L,
  rango con umbral absoluto 1e-9 N/m, tracción/compresión/axial nulo,
  núcleo axial y traslaciones, covarianza de matriz rotada no alineada.
- Covarianza de cargas y N recuperado, conectividad invertida, estaciones
  fuera de orden, cargas axiales múltiples y peso propio axial vertical D3.
- Repetibilidad: diez evaluaciones idénticas por muestras en cada dimensión;
  respuesta con historia vacía y propiedades privadas inmutables.
- Negativos: E/A cero o negativos, rho negativa, NaN/±inf en propiedades,
  cada coordenada y cada desplazamiento, geometría corta/nula y fuera del
  plano, dimensiones/tamaños/historia incompatibles, estaciones inválidas,
  componentes transversales (incluidas 1e-30 N/m), momentos, cargas no finitas,
  desbordamientos de geometría, rigidez, fuerza, masa y suma de cargas,
  EA subdesbordado; esquema con campos ausentes, extras, tipos erróneos,
  modelo de fábrica con ids duplicados, coordenadas no finitas o dimensión/plano
  incompatibles. Ningún resultado externo se usa como referencia.

## Verificación local

Comandos ejecutados con `/home/box/.cargo/bin` en PATH y
`RUSTUP_TOOLCHAIN=stable`, rustc 1.99.0. Resultados finales después de corregir
la constante de engine-linalg para wasm32:

- `cargo fmt --check`: satisfactorio.
- `cargo clippy --all-targets -- -D warnings`: satisfactorio.
- `cargo test --workspace`: satisfactorio; 106 pruebas unitarias/de integración
  y 33 doctests del workspace, incluidos 14 y 1 del nuevo crate.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`: satisfactorio.
- Target wasm32-unknown-unknown instalado: `cargo check --target
  wasm32-unknown-unknown -p engine-elements-truss`: satisfactorio.
- `cargo check --target wasm32-unknown-unknown -p engine-elements-truss
  --tests`: satisfactorio. Solo compilación; ejecución en runtime wasm y
  validación numérica cruzada nativo/wasm **PENDIENTES**, no afirmadas.
- `git diff --check` y revisión de espacios finales en archivos nuevos:
  satisfactorios. Diff restringido al crate nuevo, Cargo.toml, Cargo.lock,
  esta bitácora y el ajuste por destino de engine-linalg.

La primera compilación wasm falló con E0080 en una dependencia existente;
la decisión 11 registra causa y corrección. No se instalaron dependencias,
targets ni herramientas, ni se hicieron operaciones remotas.

No se modifican rutas protegidas ni el índice Git del usuario; el único cambio
en otro crate es la constante por destino de engine-linalg explicada arriba.
No se añaden campos license, archivos LICENSE ni texto legal. Revisión
clean-room humana: J4 APTO (2026-10-11, vía J0).

Seguimiento (2026-10-11): MAX_ALLOC_BYTES por destino (8 GiB en 64 bits, 1 GiB en 32 bits) con test; reserva fallible de estaciones en recover.rs; documentada la suposición de stiffness.rs.
