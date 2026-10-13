# Bitácora clean-room · ENG-MASS-001

- Ticket / PR: ENG-MASS-001 «Masas»; rama apilada sobre ENG-LOAD-001, PR #51 no fusionado. Sin commit ni push en esta sesión.
- Autor(es): J2 (agente, Grok Bot), 2026-10-11 (America/Guayaquil), etiqueta solicitada por el usuario. Modelo de ejecución: Codex basado en GPT-6.
- Rol: especificador e implementador por derivaciones propias; sin actividad de auditor ni operador de oráculo.
- Contrato implementado: módulo público `engine-loads::mass`; reutiliza `engine-model::NodalMass`, `DofMap`, `Element::mass`, `engine-contracts::MassKind` y `LinearSystem::global_mass`. Sin nueva numeración de GDL ni formulación elemental paralela.
- Fuentes permitidas usadas: exclusivamente especificación del usuario, documentación, contratos y código interno pertinente de este repositorio. Ninguna fuente externa, consulta por red ni oráculo.
- Derivaciones propias: ecuaciones (1)-(4) de esta nota, elaboradas para este ticket.
- Código de terceros copiado: ninguno. Sin nuevas dependencias externas; solamente dependencias por ruta a crates existentes del workspace.
- Validación: analítica, mediante energía cinética, tablas cerradas de masa y conversión de fuerza/peso a masa. Fixtures con `SOURCE` y `CITATION` en `crates/engine-loads/tests/mass.rs`.
- Comparación con oráculo (repo hermano, informativa): no. No se consultó ni ejecutó ningún motor externo.
- Declaración: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'.
- Revisor clean-room: PENDIENTE (J4).

## Ubicación y dependencias

Se elige `crates/engine-loads/src/mass.rs`: engine-loads ya depende de
engine-assembly, y engine-assembly no depende de engine-loads. Se añade
engine-linalg por ruta para nombrar las matrices CSR existentes y su contrato
de backend; no introduce ciclos ni dependencias externas. engine-elements-truss
se añade únicamente como dependencia de pruebas, junto al frame ya existente.
Los productores `Element::mass` se leyeron antes de implementar el módulo:
ambas matrices son globales y LinearSystem ya suma masas nodales a las dos
MassKind. No se cambia esa ruta ni se vuelve a sumar masa de elemento.

## Derivaciones propias

### (1) Energía cinética y masa total por dirección

Para una velocidad nodal v, T=½vᵀMv. Una traslación rígida con velocidad
unitaria d̂ tiene el mismo vector d̂ en las traslaciones de cada nodo y giros
nulos: v=r_d. Una barra o viga de sección y densidad constantes tiene
T=½∫rho·A·|d̂|²dx=½rho·A·L. Por tanto r_dᵀM_elemento r_d=rho·A·L.

Una masa nodal diagonal anisótropa aporta Σ_a m_n,a·d̂_a²; una inercia nodal
no contribuye porque su velocidad angular es cero. Al sumar aportes:

masa(d)=r_dᵀM_global r_d=Σ_n Σ_a m_n,a·d̂_a²+Σ_e rho_e·A_e·L_e.

Para d=X,Y,Z, se seleccionan solamente las masas de esa componente.
Una masa nodal isotrópica aporta su escalar una vez por dirección, nunca
tres veces. Se usa la matriz global completa, incluidos los GDL restringidos:
la matriz reducida por apoyos no representa una traslación rígida completa.
Se exige que todos los nodos declaren las traslaciones no nulas de d̂;
eliminar una componente ausente daría una masa aparente menor y se rechaza.

### (2) Tablas analíticas de barra y viga

Para dos nodos con interpolación axial t=1−x/L y r=x/L,
∫t²dx=∫r²dx=L/3 y ∫tr dx=L/6. Cada dirección de una barra tiene
M_consistente=(rho·A·L/6)[[2,1],[1,2]]. La suma contra [1,1] es rho·A·L.
La masa concentrada reparte por simetría la mitad en cada extremo:
M_concentrada=(rho·A·L/2)I, que cumple la misma identidad.

Para la viga, la interpolación transversal Hermite es
H=[t²(1+2r), Lrt², r²(1+2t), −Lr²t]. Integrar rho·A·HᵀH dx mediante
∫₀¹r^k dr=1/(k+1) da la matriz, en orden [v_i,theta_i,v_j,theta_j]:

```text
(rho A L/420) *
[ 156      22L      54     -13L   ]
[  22L      4L²     13L     -3L²  ]
[  54      13L     156     -22L   ]
[ -13L     -3L²    -22L      4L²  ]
```

Con giros cero y v_i=v_j=1, su suma es (156+54+54+156)/420=1.
La traza traslacional transversal es 312/420 veces la masa y no sirve para
computar la masa total. Las pruebas usan rho=3, A=2, L=4: masa=24 kg,
diagonal axial consistente 8 kg, acoplamiento axial 4 kg y concentrada 12 kg
por extremo/dirección. La tabla Hermite se comprueba con literales independientes.
Para liberaciones, la transformación elástica H_release conserva una
traslación rígida, porque v_i=v_j y theta_i=theta_j=0 dan giros liberados cero;
por ello rᵀH_releaseᵀ M H_release r=rᵀMr. Se prueba sin liberaciones,
con un extremo liberado y con ambos liberados.

### (3) Cambio de orientación

Una rotación ortogonal Q conserva |Qd̂|²=|d̂|² y L. La masa isotrópica de
elemento conserva rho·A·L en cada dirección, aunque cambien las entradas de
la matriz global de la viga. La barra se comprueba a lo largo de X, Y, Z y
del vector (4/3,8/3,8/3), todos de longitud 4, con origen trasladado.
La viga se comprueba con extremos (4,0), (0,4) y (2.4,3.2).
Una dirección oblicua se normaliza antes de evaluar rᵀMr. Para masas nodales
anisótropas, la expresión de (1) depende de la dirección y no se presume
isotropía. Un fixture mezcla masa elemental con masas nodales de 15 kg en X,
13 kg en Y e inercia 19 kg·m²: totales 39 y 37 kg; para d=(3,4,0),
24+15·9/25+13·16/25 kg.

### (4) Masa derivada de un caso

El usuario proporciona dirección d≠0, factor f≥0 y g_usuario>0, todos finitos.
Se normaliza d usando s=máx|d_a|, d̂=(d/s)/|d/s|, evitando el desbordamiento
de la norma para componentes grandes y su pérdida para direcciones subnormales.
La proyección del peso en cada nodo es |F_n·d̂|. De peso=m·g:

m_n=|F_n·d̂|·f/g_usuario.

F_n es la fuerza neta global por nodo del caso; se suman primero los aportes
elementales y nodales y después se toma el absoluto. No se convierte la
resultante global del modelo ni se toma el absoluto de cada aporte antes de
sumarlo. Las fuerzas opuestas en nodos diferentes aportan masas positivas.
Los momentos se validan como finitos pero no producen masa escalar. La masa
obtenida se asigna a cada traslación declarada del nodo, sin inercia de giro.
La dirección selecciona la fuente de peso; esta asignación es isotrópica en
las traslaciones disponibles. No hay factor normativo ni gravedad por defecto.

Fixture: Fz=−120 y +80 N en dos nodos, d=−Z, f=0.25, g_usuario=10 m/s²:
masas 3 y 2 kg, total 5 kg por dirección. Con fuerzas equivalentes elementales
de +20 y −40 N sobre esos GDL, los netos son −100 y +40 N: 2.5+1=3.5 kg.
Fuerza X=30 N y momento Z=999 N·m no aportan en la proyección −Z.
Se verifica que el factor se aplica una sola vez y que la conversión no
modifica las masas existentes. Se prueba rotación conjunta de F y d y cambio
explícito de gravedad. Casos sin cargas y factor cero dan masas cero.

## Errores y límites numéricos

- ENG-E-0010: masas/inercias o factores negativos; g_usuario≤0; dirección nula;
  tamaño de matriz/vector incompatible; masa positiva perdida a cero en el
  producto por factor o división por g; reserva fallida del vector de trabajo.
- ENG-E-0008: NaN, infinitos o desbordamiento en entradas y resultados, incluida
  suma de masas por GDL, proyección, producto por factor, división y rᵀMr.
- ENG-E-0003/0011: nodo/caso o GDL inexistentes. ENG-E-0002: ids repetidos.
- Componentes repetidas dentro de una masa y asignaciones distintas al mismo
  nodo/GDL se suman, según la semántica existente de LinearSystem.
- Ningún panic, unwrap ni expect en el nuevo código de biblioteca. Pruebas
  directas con MAX, NaN, infinitos, mínimo subnormal, dirección nula y listas
  vacías: cualquier pánico haría fallar el test.
- La medida direccional requiere una matriz física validada por su productor;
  comprueba tamaño, diagonales no negativas y finitud del resultado, pero no
  certifica simetría ni semidefinición positiva de una matriz CSR arbitraria.
- Se rechazan productos intermedios desbordados aunque una reordenación pudiera
  hacer representable el resultado. IEEE-754 no garantiza precisión relativa
  de proyecciones subnormales ni recupera componentes de dirección que se
  pierdan al dividir por su escala.

## Metadatos de los tres tickets

Se interpreta «los tres tickets» como ENG-GEO-001, ENG-LOAD-001 y ENG-MASS-001.
Se actualizan los metadatos pedidos; se conserva la evidencia histórica de
ENG-GEO-001, incluida su consulta bibliográfica anterior. Esta sesión no
consulta esa fuente ni ninguna otra fuente externa, y todas sus pruebas de
masa son derivaciones analíticas propias.

## Verificación de la sesión

Todos los comandos Cargo se ejecutaron sin red (`CARGO_NET_OFFLINE=true`).

- `cargo fmt`: OK, salida 0.
- `cargo clippy --all-targets -- -D warnings`: OK, salida 0.
- `cargo test --workspace`: OK, salida 0; 220 pruebas y doctests del workspace,
  incluidos 9 tests analíticos nuevos de masa y 1 doctest nuevo.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK, salida 0.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, salida 0,
  con `PATH=/home/box/.cargo/bin:$PATH`, toolchain ya instalado rustc 1.99.0.
  Los comandos nativos usaron rustc 1.85.1 (MSRV del workspace). No se
  descargaron componentes. Esta comprobación verifica compilación wasm,
  no ejecución ni igualdad numérica entre destinos.
- `bash clean-room/check.sh origin/main`: OK, salida 0; incluye los tres archivos
  nuevos mediante `git add -N` en un índice y objetos temporales en `/tmp`,
  sin modificar el índice real. `git diff --check` sobre ese índice: OK.
  Registro: `/tmp/eng-mass-001-clean-room.log`.

Los registros Cargo están en `/tmp/eng-mass-001-{fmt,clippy,test,doc}.log`.
Sin commit ni push. Revisión clean-room: PENDIENTE (J4).
