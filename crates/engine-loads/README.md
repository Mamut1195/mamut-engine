# engine-loads · ENG-LOAD-001

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
