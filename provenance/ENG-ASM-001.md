# Bitácora clean-room · ENG-ASM-001

- Ticket / PR: ENG-ASM-001, Ensamblador, apoyos y desplazamientos impuestos;
  implementación en la rama existente sobre ENG-LA-007. Sin commit, push ni PR.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-11 (America/Guayaquil), identificación
  solicitada para este ticket. No se delegó a otros agentes.
- Rol: implementador; derivaciones y fixtures analíticos propios. Sin actividad
  de operador de oráculo ni auditor de similitud.
- Contrato implementado: `engine_contracts::SystemView` sobre tipos y backend
  `engine-linalg` de ENG-LA-007; numeración global/reducida, aportes K/M/Kg/F,
  eliminación de apoyos, reducción por desplazamientos impuestos y reacciones.
  Interfaz `ElementContribution`, sin implementar `Element` ni elementos finitos.
- Fuentes permitidas usadas: documentos y contratos propios enumerados abajo;
  derivaciones propias por energía, equilibrio y partición de ecuaciones. Cita
  bibliográfica con edición, página o ecuación de libro: **PENDIENTE**. No se
  consultó ni atribuyó un ejemplo a un libro no disponible.
- Derivaciones propias: [nota de esta bitácora](#derivaciones-propias) y
  [fixtures analíticos embebidos](../crates/engine-assembly/tests/fixtures/mod.rs).
- Código de terceros copiado: ninguno. No hay dependencias externas nuevas,
  fuentes de librerías copiadas ni implementación de un solver en este crate.
- Validación: exclusivamente analítica y conformidad de contratos propios.
  Matrices/soluciones de cadenas de dos y tres resortes, barra axial de dos
  segmentos y asentamiento; valores escritos a mano y derivados abajo. No se
  usaron números de otros motores. Citas bibliográficas precisas: **PENDIENTE**.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: **PENDIENTE (J4)**. No se realiza aprobación de merge en
  esta sesión.

## Documentos internos consultados

Se leyeron `/workspace/motor-rust/04_TICKETS.md` (ENG-ASM-001, ENG-ASM-002,
ENG-CORE-003, ENG-LA-007 y límites de ENG-LA-008/009);
`/workspace/motor-rust/00_PROYECTO.md`;
`/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md` (§3, §4, §4.7 y
determinismo); `docs/CLEAN_ROOM.md`; `docs/CONVENCION_EJES.md`;
`provenance/TEMPLATE.md`; `provenance/ENG-LA-007.md`; los tipos/errores de
`engine-core`, el dominio de `engine-model`, traits/tipos/conformidad de
`engine-contracts` y API propia de `engine-linalg` (DMat, SymMatrix,
SparsityPattern, SparseMatrix, Triplet y LinAlgBackend). No se encontró AGENTS.md
aplicable. No se consultaron código de motores externos, resultados de oráculo
ni listados de libros. Las fuentes son permitidas según docs/CLEAN_ROOM.md §4.

La convención aprobada de `docs/CONVENCION_EJES.md` prevalece sobre el texto
provisional del documento de arquitectura: D2 está en X-Y horizontal, con Rz;
D3 usa Z hacia arriba y una terna diestra. No se introduce otro plano 2D.

## Derivaciones propias

Para un aporte con ubicación L, cada entrada k_ij se suma en K[L_i,L_j].
Esto se obtiene al sustituir u_e[i]=u[L_i] en la energía
V_e=½u_eᵀk_e u_e. La fuerza equivalente se suma en F[L_i]. No hay reflejo
automático ni división de las mitades: se emiten ambas entradas simétricas.

En una cadena axial V=½Σ k_i(u_{i+1}−u_i)². Cada término aporta
[[k_i,−k_i],[−k_i,k_i]]. Por ejemplo, con k=[100,200] N/m:

```text
K = [ 100  -100     0
     -100   300  -200
        0  -200   200 ] N/m
```

Con k=[100,200,300], las filas son [100,−100,0,0], [−100,300,−200,0],
[0,−200,500,−300], [0,0,−300,300]. Fijando u_1=0 y aplicando P=12 N en el
último nodo, el equilibrio transmite P por cada resorte: Δu_i=P/k_i. Dos
resortes dan u_f=[0.12,0.18] m, tres dan [0.12,0.18,0.22] m, con R_1=−12 N.
Los fixtures contienen las matrices completas y estas soluciones como literales.

Para una barra homogénea en cada segmento, ε=Δu/L, σ=Eε y N=Aσ, por lo que
k=EA/L. Tomando E=200 Pa, A=2 m², L_1=2 m y L_2=4 m, k=[200,100] N/m.
La matriz tiene filas [200,−200,0], [−200,300,−100], [0,−100,100]. Con P=12 N,
u_f=[0.06,0.18] m. Es un ejemplo analítico con unidades consistentes, no una
calibración de material real. La prueba suministra los bloques calculados, sin
implementar física de elementos de barra.

Particionando los índices libres f y prescritos p en Ku=F+R, en las filas
libres R_f=0, así K_ff u_f=F_f−K_fp u_p. En las filas prescritas,
R_p=K_pf u_f+K_pp u_p−F_p. R es la acción del apoyo sobre el nodo; positiva
en el eje global de su componente. Para una cadena que tiene modo rígido,
la suma de filas de K es cero: ΣR+ΣF=0 en equilibrio. Solo se suman fuerzas
de la misma dirección; no se mezclan fuerzas con momentos.

Asentamiento: k_1=100, k_2=200 N/m, u_1=0, u_3=0.03 m. Sin fuerzas externas,
300u_2−200·0.03=0, así u_2=0.02 m, R_1=−2 N, R_3=2 N.
Con F_2=12 N y F_3=4 N, RHS=12+6=18 N, u_2=0.06 m, R_1=−6 N y
R_3=200(0.03−0.06)−4=−10 N. El equilibrio es −6−10+12+4=0.
El factor λ escala únicamente las fuerzas: RHS=12λ+6; en λ=2 da 30 N,
no 36 N. Estas pruebas también usan λ=0 y λ=−1 y verifican el equilibrio.

La prueba de ubicación no consecutiva usa A en [2,0] con matriz
[[30,4],[4,10]] y B en [1,2] con [[20,5],[5,6]]. La sustitución de índices da
K=[[10,0,4],[0,20,5],[4,5,36]]. Al restringir el índice global central,
K_ff=[[10,4],[4,36]]. Este fixture algebraico comprueba la ubicación sin
atribuir esas matrices a un elemento físico.

Las pruebas de M/Kg usan datos algebraicos de interfaz: en cada enlace,
M_l=diag(2,2), M_c=[[2,1],[1,2]], Kg=[[3,−3],[−3,3]] y una masa nodal de 5 kg
en el nodo central. La suma de los aportes reduce a M_l=diag(9,2),
M_c=[[9,1],[1,2]], Kg=[[6,−3],[−3,3]]. Son controles de ensamblaje; no se
anuncia un modelo de masa ni una fórmula geométrica de elementos.

## Decisiones de diseño y límites

1. Crate nuevo `engine-assembly` versión 0.1.0 con cuatro path-deps internas.
   Módulos `dofmap`, `constraints`, `assemble`, `system`, `reactions`; rustdoc
   público bajo `#![deny(missing_docs)]` y ejemplo compilable. No se modificó
   ningún crate existente. Solo Cargo.toml raíz añade el miembro y Cargo.lock
   registra el paquete interno, sin nuevas dependencias externas.
2. `DofMap` numera solo los GDL declarados; admite subconjuntos axiales. Orden
   por id de nodo y después Dim::dofs(), independientemente de la inserción.
   `Constraints` conserva mapas global→reducido y reducido→global, más los
   valores prescritos. `LinearSystem::reduced_index` enlaza nodo/GDL con el
   índice reducido. No se itera HashMap ni se usan hilos.
3. Un PrescribedDof puede restringir un GDL libre o sustituir el cero de un
   apoyo. La intersección entre esas dos listas es intencional; repeticiones
   dentro de cada lista fallan aunque coincidan los valores. Todos los GDL
   restringidos se rechaza con ENG-E-0010, como caso no admitido por SystemView.
4. Aportes K/M/Kg cuadrados y exactamente simétricos conforme a SymMatrix. Los
   GDL de location son distintos y válidos; su orden pertenece al productor.
   Se suman matrices/vectores en orden de entrada. Ausencia de M/Kg aporta cero;
   masas nodales diagonales se añaden a ambas MassKind, sin fabricar matrices
   elementales. Casos sin cargas existen; referencias a casos ausentes fallan.
5. Matrices globales CSR y patrón unión de K, ambas M y Kg, incluidos los ceros
   explícitos. Reducción densa porque el SymMatrix vigente es denso: memoria
   O(n_f²). La contracción CSR con u suma filas en orden estable sin expandir K
   global. Factorización y solución se delegan íntegramente al backend de
   ENG-LA-007; no se escribe eliminación ni regularización en este ticket.
6. SystemView usa fuerzas físicas `(Ku)_f` y `λF_f`. El asentamiento se incluye
   una vez en la fuerza interna. `reduced_rhs` lo resta una vez para solve
   directo. Tangente K_ff constante; u_p constante no depende del caso/factor.
   Reacciones incluyen cargas sobre apoyos. Traslaciones m, giros rad, fuerzas
   N, momentos N·m; K/Kg N/m, N o N·m, M kg, kg·m o kg·m². Las normas mixtas
   necesitan escalas explícitas del método de análisis.
7. Evaluaciones exitosas actualizan la prueba, sin aceptación implícita.
   `accept` reevalúa y acepta exactamente u atómicamente; fallos conservan
   aceptado, prueba y salidas; rollback es infalible/idempotente. No hay historia
   material en el sistema lineal. Estado inicial u_f=0 con u_p impuesto, sin
   afirmar equilibrio. Matrices y restricciones privadas e inmutables.
8. Catálogo del núcleo: ENG-E-0010 tamaños/índices; ENG-E-0008 entradas,
   productos o sumas no finitos; ENG-E-0003 casos/nodos ausentes; ENG-E-0002
   duplicados; ENG-E-0011 GDL incompatibles. ENG-E-0006 del solver se traduce
   de índice reducido a global y añade Node y componente en el mensaje.
   ENG-E-0012 para diagnóstico de mecanismo incoherente sin índice físico válido.
   El pivote señala parte del mecanismo, sin caracterizar todos sus movimientos.
9. `ElementContribution` es una interfaz de datos lineales evaluados, no un
   Element. Kg se recibe para un estado de referencia y permanece constante;
   `geometric(u)` no obtiene axiales ni ejecuta P-Delta. Evaluaciones de elementos,
   transformaciones, historia, cargas distribuidas y recuperación de acciones
   locales quedan para ENG-ELM-*. La validación completa del dominio corresponde
   a engine-model; este constructor valida los datos que usa, sin interpretar
   miembros/secciones/leyes/combinaciones. Un adaptador dinámico de elementos
   deberá conservar esta frontera y declarar su política de actualización de Kg.
10. ENG-ASM-002 conserva maestro-esclavo y transformaciones de restricciones.
    ENG-LA-008/009 conserva solver disperso productivo/modal. El backend de
    referencia usado en pruebas es para modelos pequeños. Determinismo bit a
    bit entre nativo/wasm PENDIENTE de ENG-CORE-010; no hay trascendentes nuevas
    ni dependencia directa de libm en este crate.

## Pruebas y tolerancias

Los fixtures Rust embebidos declaran `source` y `citation`; no leen archivos,
reloj ni hilos. Se valida analíticamente, sin oráculo y sin referencias de
validación a motores externos. Las citas bibliográficas no disponibles se
marcan PENDIENTE en el fixture y aquí, sin inventar edición/página/ecuación.

- Matrices esperadas literales, comparación exacta con enteros representables.
- Desplazamientos/fuerzas analíticos: error ≤ 1e-10·max(1,|esperado|).
- Equilibrio ΣR+ΣF: tolerancia absoluta 1e-9 N.
- Kit check_system_view sobre muestras lineales y con asentamiento: tangente
  por diferencias finitas, masas/Kg, escalado de casos y sobrescritura de salidas.
- Numeración D2/D3, nodo/GDL de índices libres, apoyo central, ubicación
  permutada/no consecutiva, fuerzas equivalentes y ambos MassKind.
- Negativas: GDL/nodo/caso/apoyo/prescrito duplicado, índices fuera de rango,
  componente incompatible, NaN/±inf y desbordamientos, caso ausente, apoyo sobre
  nodo inexistente, matrices/vectores incompatibles o asimétricos, todos los GDL
  restringidos, mecanismo libre y GDL aislado sin rigidez. Se prueba también
  ENG-E-0012 con un backend de prueba que informa un índice singular inexistente.
- Accept/rollback y atomicidad al fallar; diez repeticiones con matrices iguales
  y soluciones idénticas bit a bit en el mismo destino.

## Verificación local

Comandos con `/home/box/.cargo/bin` en PATH y `RUSTUP_TOOLCHAIN=stable`
(rustc 1.99.0). Resultados finales:

- `cargo fmt --check`: satisfactorio.
- `cargo clippy --all-targets -- -D warnings`: satisfactorio.
- `cargo test --workspace`: satisfactorio; 10 pruebas nuevas de integración y
  un doctest nuevo, además de las pruebas existentes.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`: satisfactorio.
- Target wasm32-unknown-unknown instalado; `cargo check --target
  wasm32-unknown-unknown -p engine-assembly`: satisfactorio.
- También `cargo check --target wasm32-unknown-unknown -p engine-assembly
  --tests`: satisfactorio. Compilación de las pruebas; ejecución en runtime wasm
  **PENDIENTE**, no se afirma haberlas ejecutado en wasm.
- `git diff --check`: satisfactorio. Ningún crate existente ni ruta protegida
  modificados; sin campo license, archivos LICENSE ni texto legal nuevos.

No se hicieron commit, push, PR ni cambios al índice del usuario. Revisión
clean-room humana **PENDIENTE (J4)**.
