# engine-loads · ENG-LOAD-001 / ENG-MASS-001

Funciones puras en SI, con dependencias internas por ruta. No aplican factores
normativos ni suponen una dirección de gravedad.

`Beam2D::new(L, ángulo)` define una viga sin liberaciones en el plano global XY;
el ángulo se mide en radianes desde +X hacia +Y. Su eje local x sigue i → j y
su eje local y es (-sin, cos). `SpanLoad` contiene fuerzas axiales/transversales
en ejes `Local` o `Global`:

- `Uniform { q, axes }`: intensidad constante [qx,qy], N/m, sobre todo el tramo.
- `Trapezoidal { q_i, q_j, a, b, axes }`: intensidades en los límites de un
  intervalo; `a` y `b` son distancias desde i, en m. `None` equivale a 0 y L,
  respectivamente; exige 0 ≤ a < b ≤ L y carga cero fuera del intervalo.
- `Point { x, p, axes }`: fuerza P=[px,py] en N, a distancia x desde i.

`equivalent_nodal_forces_local` devuelve las cargas nodales equivalentes en orden
`[N_i,V_i,M_i,N_j,V_j,M_j]`, con signos de fuerzas nodales locales y momentos
positivos alrededor de +Z. `fixed_end_forces_local` devuelve su opuesto: las
reacciones sobre el miembro cuando los seis desplazamientos son cero. Para una
carga positiva qy uniforme, las magnitudes son qyL/2 y qyL²/12. Para P en el
centro, los cortantes son P/2 (N) y los momentos PL/8 (N·m).

La interpolación axial es lineal; la transversal es Hermite cúbica.
Gauss–Legendre de tres puntos integra exactamente el producto con una carga
lineal, cuyo grado máximo es cuatro. No hay integración aproximada ni discretización
del tramo. La carga uniforme completa usa directamente su fórmula cerrada.
Las derivaciones y tablas independientes están en
[la bitácora](../../provenance/ENG-LOAD-001.md).

`to_member_load` convierte cargas uniformes completas a `engine_contracts::MemberLoad`
en ejes globales, con momento por longitud cero. También admite un trapecio
completo con intensidades iguales. Rechaza cargas parciales, variables o puntuales
con ENG-E-0013; no altera su distribución para adaptarlas al contrato.
`Beam2D::forces_to_global` permite transformar el vector equivalente para construir
un `engine_assembly::ElementLoad` en la ubicación del elemento correspondiente.
Las funciones de este crate describen el miembro sin liberaciones; si el elemento
tiene liberaciones, debe aplicar su propia condensación.

`NodalLoad`, `LoadCase` y `DofValue` son reexportaciones de `engine-model`.
`ImposedDisplacement` es un alias de `engine_assembly::PrescribedDof`.
`assemble_nodal_loads` emplea el `DofMap` existente y `assemble_vector`;
`assemble_imposed_displacements` produce `Vec<Option<Real>>`, donde `None`
es libre y `Some(0.0)` es un desplazamiento fijado a cero. La lista original puede
pasarse directamente a `Constraints::new`. `validate_case` valida ambas listas
y añade el nombre del caso al error. Cada nodo/GDL debe aparecer una sola vez
en cada lista del caso; una fuerza y un desplazamiento sí pueden compartir GDL.
Pueden existir varias entradas del mismo nodo con componentes distintas.
No se validan aquí rigideces, apoyos o conectividad de los miembros.

`self_weight(rho,A,L,g)` devuelve la fuerza total global rho·A·L·g;
`self_weight_per_length(rho,A,g)` devuelve rho·A·g. Se requiere rho≥0, A>0,
L>0; admite gravedad global tridimensional. La suma de miembros es
Σ rho·A·L·g, sin factor adicional. D2 exige que el consumidor rechace Z no nulo.

Todas las entradas y salidas se comprueban como finitas. Los desbordamientos,
incluidos productos intermedios, son ENG-E-0008; los rangos y duplicados de
acciones del caso son ENG-E-0010. Las referencias/GDL propagan ENG-E-0003 y
ENG-E-0011 del mapa. No se usa una tolerancia dimensional para L: se admiten
longitudes positivas muy pequeñas. El elemento frame existente mantiene su
propia tolerancia geométrica. El redondeo y subdesbordamiento siguen IEEE-754;
no se garantiza precisión relativa para resultados subnormales.

El módulo público `mass` reutiliza `engine_model::NodalMass`. Valida masas en
kg e inercias en kg·m², referencias y GDL; `assemble_nodal_masses` devuelve la
diagonal global y `assemble_nodal_mass_matrix` la matriz diagonal CSR. Se suman
asignaciones con ids distintos y componentes repetidas, igual que en
`LinearSystem`; los ids repetidos se rechazan. Ninguna operación modifica el modelo.

Las masas elementales siguen procediendo de `Element::mass(ctx, MassKind)` y
se pasan en `ElementContribution` a `LinearSystem::from_model`. Las masas
nodales se añaden a ambas clases. `mass::system_total_mass(system, kind, d)`
consulta `global_mass(kind)` y calcula `r_dᵀ M r_d`, incluidos los nodos apoyados:
para cada dirección cartesiana, el resultado esperado es la suma de masas
nodales de esa componente más Σrho·A·L de los elementos. En una dirección
oblicua, una masa nodal diagonal aporta Σm_a·d̂_a². Los giros del vector rígido
son cero. No se usa la traza de M, que no conserva masa consistente.
La dirección se normaliza y cada nodo debe declarar todas sus componentes de
traslación no nulas; una dirección Z en un modelo XY produce ENG-E-0011.

`mass::masses_from_case(map, case, factor, d, g_usuario)` convierte cargas
nodales en asignaciones `NodalMass`; `masses_from_system_case` incluye también
las fuerzas equivalentes de los elementos del caso. Por nodo se calcula
`m = |F·d̂|·factor/g_usuario` con fuerzas globales netas, antes de sumar las
masas entre nodos. Los momentos se validan y no contribuyen. La masa escalar
se asigna por igual a cada traslación declarada del nodo; no se crean inercias.
La dirección selecciona el peso de origen. Para Fz=−120 N, factor=0.25,
d=−Z y g_usuario=10 m/s², la masa es 3 kg en cada traslación.

El factor debe ser finito y no negativo, y g_usuario finita y positiva; ambos
son parámetros obligatorios. Se admiten factor cero y fuerzas de cualquier
signo. `mass_from_force` permite la conversión escalar, y
`masses_from_load_vector` recibe un vector global ya ensamblado y un prefijo
de ids explícito. Los adaptadores de caso usan `mass:case:<nombre>:node:<id>`;
al combinar fuentes, el llamador debe asegurar ids únicos y agregarlas
explícitamente a `Model::masses` antes de construir el sistema. No se vuelven
a añadir masas de elemento durante la conversión. Las masas negativas son
ENG-E-0010; los no finitos y desbordamientos, incluidos intermedios, ENG-E-0008.
Una masa positiva que se reduce a cero en el producto por factor o división
por g se rechaza con ENG-E-0010. La normalización admite direcciones grandes
y subnormales; no se garantiza precisión relativa de cantidades subnormales.
Derivaciones y evidencia en [ENG-MASS-001](../../provenance/ENG-MASS-001.md).
