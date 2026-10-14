# Bitácora clean-room · ENG-RES-001

- Ticket / PR: ENG-RES-001 «Resultados estructurales y postproceso». Apila sobre
  ENG-ANL-001 (`engine-analysis`, `StaticLinear`/`StaticResults`, PR #52 aún sin
  fusionar). Sin commit ni push.
- Autor(es): J2 (agente, Grok Bot) · identificación solicitada por el usuario;
  agente ejecutor Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: módulo `engine_analysis::results` dentro de
  `engine-analysis`, con recuperación mediante `element/1`. Modelo de datos
  propio con campos públicos documentados, preparado para serialización futura.
  El esquema `results/1` y su serialización quedan pendientes de ENG-CORE-005:
  no se inventan JSON, hashes, metadatos serializados ni adaptadores ResultSink.
- Fuentes permitidas usadas: exclusivamente la petición del ticket, las APIs
  propias de engine-core, engine-contracts, engine-assembly, engine-model,
  engine-analysis, engine-elements-frame y engine-elements-truss, y la frontera
  pública/backend de referencia existente de engine-linalg para pruebas.
  Plantilla: `provenance/TEMPLATE.md`. Sin red ni fuentes externas.
- Derivaciones propias: [nota de esta bitácora](#derivaciones-propias) y
  [pruebas analíticas embebidas](../crates/engine-analysis/tests/results.rs).
- Código de terceros copiado: ninguno. Sin crate ni dependencias nuevas.
- Validación: exclusivamente analítica, por equilibrio y formas cerradas propias;
  sin comparación con otros motores.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: PENDIENTE (J4).

## Diseño y convenciones

`CaseResults::recover(solution, system, members)` usa únicamente el vector
`global_displacements` de `StaticCaseResult`, que incluye los desplazamientos
prescritos. Comprueba tamaño, finitud y coherencia con los prescritos del sistema.
Obtiene cada `u_e` por el DofMap, nodo y luego GDL según el elemento, y llama a
`Element::recover(ctx, u_e, committed, loads, stations)` sin aceptar ni modificar
historia. No utiliza las copias de desplazamientos libres o reacciones del caso:
recalcula las reacciones mediante K y las cargas físicas del sistema.

El sistema ensamblado no conserva los elementos ni sus cargas originales de
tramo. `MemberRecovery` aporta id, conectividad, elemento, dimensión, parámetros,
historia aceptada y cargas **sin escalar** del caso concreto. La lista debe incluir
todos los miembros del sistema; el seguimiento #69 verifica su equilibrio nodal
contra las cargas nodales y reacciones del mismo ensamblado, según la regla en
[ENG-RES-001-FU](ENG-RES-001-FU.md). No hay registro implícito de miembros
en LinearSystem: una referencia ausente se diagnostica al consultar los ids de
CaseResults o al resolver nodos/GDL en DofMap; el identificador numérico no es
un índice de vector. Los ids repetidos se rechazan. Las cargas de tramo se
escalan una vez por el factor del caso; los asentamientos nunca se escalan.

Nodos y miembros se devuelven por id ascendente. Los vectores nodales tienen
seis componentes globales: `[Ux,Uy,Uz,Rx,Ry,Rz]` (m/rad) y
`[Fx,Fy,Fz,Mx,My,Mz]` (N/N·m). Las componentes no declaradas y las reacciones de
GDL libres valen cero. En D2, Rz ocupa la sexta componente, no la tercera.
Las acciones de extremo mantienen el orden local declarado por el elemento;
frame2d usa `[Ni,Vi,Mi,Nj,Vj,Mj]`. Los diagramas usan la cara positiva local
`[N,Vy,Vz,T,My,Mz]`, con N positivo en tracción. En frame2d, V=Vy y M=Mz;
Vz, T y My son cero. En armaduras solo N es distinto de cero.

Las estaciones predeterminadas son `[0, 0.25, 0.5, 0.75, 1]`. Una lista explícita
las sustituye; siempre se añaden 0 y 1, se ordena y se deduplica. Una lista vacía
obtiene los extremos y las raíces críticas aplicables. Para frame2d con carga
uniforme transversal no nula se añaden las estaciones interiores de cortante
nulo y de derivada de momento nula. Toda resultante se obtiene otra vez del
elemento, que incluye las cargas exactamente; no se interpolan las acciones de
extremo. La recuperación valida tamaños, estaciones y finitud de la salida.

Cada campo local tiene máximo y mínimo **algebraicos**, valor y estación x/L.
En empates se conserva la menor estación. Para **frame2d** las
cargas uniformes dan N/V afines y M cuadrático, por lo que extremos de tramo
más las raíces añadidas dan extremos exactos. **Frame3d no añade estaciones
críticas interiores**: sus extremos se limitan a las estaciones recuperadas.
Para otros kinds se documenta
que los extremos se limitan a las estaciones recuperadas; no se promete buscar
raíces interiores de campos arbitrarios ni se aproxima su interpolación.

`query(nodes, members, fields)` devuelve una proyección propia con solo los
campos seleccionados. `None` significa todos; `Some(&[])`, ninguno. La lista de
ids conserva el orden del resultado y no duplica entidades; los campos locales
mantienen el orden N/Vy/Vz/T/My/Mz. Desplazamientos, reacciones y acciones de
extremo excluidos son `None`; estaciones y extremos contienen solo los campos
locales elegidos. Un campo que no aplica al bloque no añade valores. Los ids
ausentes generan ENG-E-0003, con su entidad; duplicados en recuperación,
ENG-E-0002; tamaños, prescritos o estaciones fuera de `[0,1]`, ENG-E-0010;
no finitos/desbordamiento, ENG-E-0008; salidas elementales incoherentes,
ENG-E-0012. Se propagan errores elementales con el miembro en entidades.

## Derivaciones propias

Datos artificiales SI: E=200 Pa, A=2 m², I=3 m⁴, L=4 m, w=6 N/m, P=12 N.
Las pruebas comprueban valores con tolerancia absoluta 1e-9 SI y estaciones
con 1e-12; el informe de equilibrio del análisis debe tener residual relativo
menor que 1e-12. No hay archivos, procesos, red ni oráculos en los fixtures.

### Raíces interiores exactas

En los ejes locales de frame2d, bajo cargas uniformes qy y mz:

```text
V(x) = V(0) − qy x
M(x) = M(0) − (V(0)+mz)x + qy x²/2
V(x*)=0       => x*/L = V(0)/(qy L)
M′(x**)=0     => x**/L = (V(0)+mz)/(qy L)
```

Sin par distribuido, ambas raíces coinciden. Se añaden solo si caen
estrictamente en `(0,1)`. qy se proyecta de global a local con
`qy=−sin(alpha) qX + cos(alpha) qY`, sumando todas las cargas escaladas.
La comprobación de interior precede a la división para evitar desbordar raíces
exteriores. Se usa la recuperación elemental exacta para evaluar cada raíz.

### Viga simplemente apoyada

Con qy=−w, Ry_i=Ry_j=wL/2; V(x)=−wL/2+wx y
M(x)=wx(L−x)/2. La raíz está en x=L/2 y Mmax=wL²/8=12 N·m.
La prueba pide únicamente x/L=0.2 y 0.8 (desordenadas y repetidas) y comprueba
que se añade 0.5, además de los extremos. Los giros son
theta_i=−wL³/(24EI), theta_j=+wL³/(24EI). El momento a 0.2 se comprueba contra
la parábola. Con un par terminal +8 N·m, Ry_i=14 N, Ry_j=10 N y
M(x)=14x−3x²: el máximo se desplaza a x/L=7/12 y vale 14²/(2w).

Una variante gira la viga 90° y suma dos cargas uniformes globales +X. Se
comprueban factores 2, −1.5 y 0; el factor negativo cambia el máximo positivo
por un mínimo negativo y el factor nulo anula cargas y resultados.

### Voladizo con cargas nodales

Una fuerza terminal axial H=5 N da N=H y u_x=HL/(EA). La fuerza transversal
−P da V=−P y M(x)=−P(L−x), v(L)=−PL³/(3EI), theta(L)=−PL²/(2EI).
Acciones de extremo: `[−H,P,PL,H,−P,0]`. Una carga adicional −7 N en el apoyo
da Ry=P+7, Rx=−H y Mz=PL; no modifica los diagramas del miembro. Se comprueban
los brazos usando coordenadas no situadas en el origen, el equilibrio y los
extremos de campos constantes. Se vacían las copias reducidas/reacciones del
StaticCaseResult para verificar que se usa el vector global.

### Viga biempotrada en flexión

Con giros y desplazamientos transversales nulos en ambos extremos, las acciones
locales son Mi=+wL²/12, Mj=−wL²/12 y Vi=Vj=wL/2. Por cara positiva,
M(x)=−wL²/12+wx(L−x)/2: mínimos en los extremos, −wL²/12, y máximo en el
centro, +wL²/24. La prueba pide una lista vacía y encuentra el centro.

LinearSystem rechaza modelos con todos los GDL prescritos. Se deja Ux_j libre
sin carga axial, conservando el empotramiento de TODOS los GDL de flexión.
EA/L>0 fuerza Ux_j=0; al no haber acoplamiento axial-flexión en esta viga recta,
su solución es exactamente la biempotrada con carga transversal. No se cambia
el ensamblador para este ticket. Una variante aplica mz=3 N de par distribuido:
V=0 ocurre en x/L=0.625, pero M′=0 sigue en 0.5; se recuperan ambas estaciones.

### Armadura y asentamiento

Dos barras entre bases (−4,0) y (4,0) y ápice (0,3), L_barra=5 m.
De equilibrio vertical, 2N(3/5)=−P: N=−5P/6=−10 N en cada barra.
Reacciones verticales P/2 y horizontales opuestas ±4P/6. Los diagramas son
constantes, todas las otras resultantes locales nulas. Se comprueban consultas
de nodos/miembros/campos, listas vacías, ids repetidos y referencias ausentes.

Dos barras axiales en serie, longitud total 4 m, con u_i=0 y u_j=0.03 m, sin
carga, dan N=EA(0.03)/L=3 N, desplazamiento medio 0.015 m y reacciones ±3 N.
Con factor 2 los asentamientos y estas acciones se conservan. Un GDL interior
libre permite resolver el sistema sin modificar la restricción existente.

### Entradas inválidas

Se prueban estaciones negativas, mayores que uno, NaN y ±infinito;
miembros/conectividades repetidos; nodos ausentes; tamaños de conectividad,
historia y desplazamientos globales incoherentes; carga/factor no finitos;
carga fuera del plano; caso ausente y desplazamiento prescrito incoherente.
Los diagnósticos deben pertenecer al catálogo. Los errores no modifican los
estados aceptado y de prueba. Sin `panic!`, `unwrap` ni `expect` en el módulo.

## Verificación local

Verificación del 2026-10-11, sin acceso a red:

- `cargo fmt` y `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK; 215 pruebas, incluidas las nueve pruebas de
  integración nuevas del postproceso y los doctests existentes.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, usando
  `PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable`, con el destino y
  toolchain ya instalados. No se descargó nada.
- `bash clean-room/check.sh origin/main`: OK. También se ejecutó con un índice
  y directorio de objetos temporales en `/tmp`, usando los objetos originales
  en lectura y `git add --intent-to-add` en el índice temporal, para incluir los
  tres archivos nuevos en el diff comprobado. No se modificó el índice real,
  no se crearon commits y no se hizo push.
- `git diff --check`: OK, incluidos los archivos nuevos mediante el mismo
  índice temporal. Sin `panic!`, `.unwrap()` ni `.expect()` en el módulo ni
  en sus pruebas nuevas.

Pruebas nativas y rustdoc: Cargo/rustc 1.85.1 del sistema. Clippy instalado:
0.1.99; compilación wasm: toolchain stable existente, rustc 1.99.0.
El chequeo wasm valida compilación, no ejecución en un runtime wasm.
La serialización `results/1` de ENG-CORE-005 y la revisión clean-room J4
siguen pendientes según el alcance del ticket.
