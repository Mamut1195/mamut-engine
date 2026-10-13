# Bitácora clean-room · ENG-ELM-004

- Ticket / PR: ENG-ELM-004; implementación local, sin commit ni push.
- Autor(es): J2 (agente, Grok Bot), sesión ejecutada por Codex (GPT-6),
  2026-10-11 (America/Guayaquil). Se conserva la atribución solicitada.
- Rol: implementador; derivaciones analíticas propias.
- Contrato implementado: element/1 · kind = `spring.linear`.
- Fuentes permitidas usadas: ninguna fuente externa. Solo contratos, patrones
  y documentación internos del workspace enumerados abajo.
- Derivaciones propias: [esta nota](ENG-ELM-004.md), sección siguiente.
- Código de terceros copiado: ninguno.
- Validación: analítica, equilibrio y energía; fixtures propios en
  `crates/engine-elements-spring/tests/fixtures/mod.rs`.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Documentos internos consultados

`provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md`, `docs/CONVENCION_EJES.md`,
`provenance/ENG-ELM-002.md`; estructura, implementación y fixtures de
`engine-elements-truss` y `engine-elements-frame`; contratos, tipos y kit de
conformidad de `engine-contracts`; tipos/GDL de `engine-core`; modelo nodal de
`engine-model`; ensamblaje, restricciones, contribuciones, sistema y backend
de `engine-assembly` y `engine-linalg`; `clean-room/check.sh`.
No hay AGENTS.md aplicable. No se usaron red, fuentes externas, código de
otros motores, ejecución de oráculos ni resultados de oráculos.

## Derivaciones propias

### Energía, rigidez y fuerza interna

Sea c el vector de acoplamiento por nodo en ejes globales, fijo. Para un GDL
global elegido c es su vector canónico. Para varios GDL locales seleccionados,
c_local tiene un uno por componente seleccionada; todos comparten **una**
deformación generalizada, no rigideces independientes. Solo se suman componentes
de una misma familia para conservar las unidades (traslaciones o giros).
Para resortes independientes se crean varios elementos, que el ensamblador suma.

Entre dos nodos, δ=c·(u_j−u_i) y b=[−c,c]. A tierra, δ=c·u_i y b=c.
El único nodo del apoyo representa el extremo móvil positivo; el otro extremo
está inmóvil y no forma parte del contexto ni del mapa de GDL.

```text
W = ½ k δ²,       k ≥ 0
p = k δ
f_int = b p
K = k b bᵀ
```

Para un GDL entre nodos K=k[[1,−1],[−1,1]], y a tierra K=[k].
Para dos GDL globales seleccionados, el bloque por nodo es k[[1,1],[1,1]]:
hay acoplamiento y rango uno. Las componentes fuera del modo seleccionado
quedan libres, sin regularización; otros elementos o restricciones deben
eliminar mecanismos cuando se necesita resolver un sistema invertible.

K es simétrica, semidefinida positiva y constante. Entre nodos toda traslación
uniforme da δ=0 y K·u=0; los giros uniformes también para resortes rotacionales.
Para nodos separados un giro rígido puede cambiar la proyección sobre un eje
fijo y deformar un resorte traslacional: no es un elemento corrotacional.
El apoyo a tierra elimina el modo rígido asociado, por construcción.
No hay geometría actualizada, esfuerzos iniciales ni término de segundo orden:
Kg es nula. El resorte ideal no tiene energía cinética: masa nula para ambas
MassKind. Se usa None para ambos, como permite element/1, validando entradas.

### Orientación y recuperación

Global conserva los GDL elegidos, en orden canónico de engine-core. Rotation
recibe Q global→local, con ejes locales como filas; Q Qᵀ=I y det(Q)=+1,
tolerancia absoluta 1e-12. D2 conserva XY y +Z exactamente. Entonces
c_global=Qᵀc_local, tanto para traslaciones como para giros. Se activan todos
los GDL globales de esa familia, incluso aquellos con coeficiente nulo, para
no perder componentes al rotar. La tabla estática de subconjuntos cumple la
firma &'static [Dof] sin fugas de memoria ni registros globales mutables.

Direction solo admite Ux local y calcula c_global=d/||d||. No define una
orientación transversal arbitraria: las otras resultantes locales son cero.
Se normaliza d dividiendo primero por max|d_i|, y luego por la norma del vector
escalado obtenida con hypot. Así se admiten componentes finitas MAX o
subnormales sin elevarlas al cuadrado ni desbordar la norma original. Dirección
nula/no finita y dirección D2 fuera de XY se diagnostican.

Invertir d cambia δ y p de signo, pero deja K y fuerza global invariantes.
Rotar modelo, desplazamientos y eje propio por R da f'=R f y K'=R K Rᵀ,
con R repetida en bloques nodales. La matriz explícita se transforma como
Q'=Q Rᵀ. No se espera invariancia si se rota el modelo dejando su eje fijo.

Recuperación: acciones locales [−c_local p,c_local p] entre nodos y
[c_local p] a tierra, según los GDL activos. Cada estación entrega las seis
componentes canónicas [N,Vy,Vz,T,My,Mz]=c_local p. Las posiciones [0,1] son
etiquetas del contrato, no una integración sobre longitud; se conservan orden
y duplicados. No se admite carga distribuida no nula: se aplican cargas nodales.
No se infieren longitud, eje ni gravedad desde las coordenadas, y se permiten
coordenadas coincidentes. D2 exige Z=0 y todo contexto debe ser finito.

### Soluciones cerradas ensambladas

Fixtures SI propios: k1=120 N/m, k2=80 N/m y F=12 N.

1. Serie: equilibrio exige p1=p2=F; u_intermedio=F/k1 y
   u_final=F(1/k1+1/k2). Por ello k_eq=1/(1/k1+1/k2)=48 N/m,
   u_intermedio=0.1 m, u_final=0.25 m, reacción de base −12 N.
2. Paralelo: ambos tienen la misma δ, F=(k1+k2)δ; k_eq=200 N/m,
   u=0.06 m, fuerzas 7.2 N y 4.8 N, reacción de base −12 N.
3. Apoyo a tierra: ku=F, u=0.1 m, fuerza interna en el nodo +12 N.
4. Apoyo oblicuo: un resorte k1 en d y otro k2 en su perpendicular eliminan
   el mecanismo transversal. Bajo Fd la solución exacta es u=(F/k1)d.
   Se comprueba en tres ángulos sin añadir rigidez artificial.

Todos se ensamblan con DofMap, ElementContribution y LinearSystem; el solve
usa ReferenceBackend. Las fuerzas de miembro se recuperan después de expandir
los desplazamientos restringidos. No hay un solver particular en este crate.

## Arquitectura, API y diagnósticos

- Crate 0.1.0, miembro del workspace, dependencias path a core/contracts/linalg,
  y dev a assembly/model. Sin dependencias externas nuevas. Módulos privados
  factory, geometry, stiffness, recover, mass, geometric y loads como las
  plantillas internas; API y validadores en lib.rs; deny(missing_docs).
- LinearSpring::new(dim, connection, k, dofs, axes) admite uno o varios GDL
  distintos de la misma familia y valida propiedades inmutables;
  ctx.params no interviene. No se usa la ley Elastic de barras: requiere módulo
  positivo, mientras spring.linear admite k=0 y no representa una tensión Pa.
- SpringFactory::new(dim): k y dofs requeridos; ground booleano opcional false;
  dofs contiene exactamente un GDL local de la dimensión seleccionada,
  como exige el esquema JSON (minItems=maxItems=1). Las listas vacías o con
  más GDL que la dimensión se rechazan; varios GDL distintos válidos producen
  ENG-E-0013 para evitar acoplamientos no pedidos.
  direction y rotation opcionales, mutuamente excluyentes; sin ellos usa ejes
  globales. JsonValue propio en memoria, sin parser. Esquema JSON y validación
  de tipo, rango, campos desconocidos, dimensión y ModelView como truss/frame.
- ENG-E-0008: entradas o resultados no finitos, incluyendo productos/sumas y
  rigidez transformada desbordados; ENG-E-0010: rangos, tamaños, GDL duplicados,
  vacíos, ajenos a dimensión o familias mezcladas, dirección nula, matriz no
  ortonormal/diestra, plano D2, cargas y estaciones no admitidas;
  ENG-E-0002: ids nodales duplicados en ModelView;
  ENG-E-0013: varios GDL distintos válidos en la fábrica. Sin códigos nuevos.
- Sin panic/unwrap/expect en biblioteca. La cantidad de GDL es como máximo seis
  por nodo y todos los índices internos dependen de entradas ya validadas.
  k=0 desconecta exactamente después de validar u e historia; evita diferencias
  desbordadas que no afectan la salida. k diminuta no se sustituye por un umbral;
  subdesbordamiento IEEE de proyecciones/productos puede producir ceros finitos.
  No se garantiza representabilidad de todo cálculo extremo: overflow da
  diagnóstico. Sin estado mutable, E/S, reloj ni hilos.

## Pruebas y tolerancias

- Cuatro pruebas analíticas ensambladas y doce de invariantes/conformidad,
  negativos, fábrica, energía, recuperación, masa/Kg y extremos numéricos;
  un doctest en español. Todos los fixtures citan esta derivación propia.
- Desplazamientos: absoluto 1e-12 m; fuerzas y reacciones ensambladas: 1e-9 N.
  Matrices/fuerzas/energía de invariantes: absoluto 1e-10 en su unidad SI.
  Diferencias finitas centrales: h=1e-7 m/rad, error absoluto <1e-6 en entradas
  de K; el kit exige su error relativo <1e-6.
- Kit check_element y check_element_factory: resortes globales Ux/Rz en D2/D3,
  con modos rígidos compatibles; repetibilidad, estado vacío, simetría y FD.
  Un apoyo k=0 también recorre el kit completo. Los casos a tierra k>0 y ejes
  propios oblicuos usan FD/simetría/covarianza/masa/Kg/recuperación explícitas,
  pues el kit presupone ausencia de apoyo a tierra y rota parámetros escalares
  sin rotar orientaciones globales propias. No se filtran diagnósticos del kit
  ni se altera engine-contracts para hacer pasar excepciones físicas.
- Dirección oblicua D2/D3, inversiones y escalas 1e±200; covarianza espacial
  de fuerza y congruencia Q K Qᵀ independiente; equivalencia matriz/dirección,
  GDL traslacionales/rotacionales acoplados, energía y modos nulos entre nodos.
- k=0, MAX, 1e-300 y mínimo subnormal; dirección MAX/mínimo subnormal;
  overflow de kδ, diferencias y rigidez rotada; parámetros, coordenadas,
  desplazamientos, cargas/estaciones y JSON inválidos, todos con diagnóstico.

## Verificación local

Resultados de la implementación inicial, todos satisfactorios:

- `cargo fmt` y `cargo fmt --check`.
- `cargo clippy --all-targets -- -D warnings`.
- `cargo test --workspace`: todas las pruebas unitarias, de integración y
  doctests; el crate nuevo aporta 14 pruebas de integración y un doctest.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`.
- `cargo check --workspace --target wasm32-unknown-unknown`: toolchain instalado
  mediante `/home/box/.cargo/bin` en PATH; no se instalaron herramientas/targets.
  También `cargo check -p engine-elements-spring --tests --target
  wasm32-unknown-unknown` pasa. Ejecución de fixtures en runtime wasm y
  comparación numérica entre destinos: PENDIENTES; no se atribuyen a un check.
- `bash clean-room/check.sh origin/main`: pasa en el árbol de trabajo y con
  todos los archivos nuevos incorporados en un índice temporal. Se usaron
  GIT_INDEX_FILE y GIT_OBJECT_DIRECTORY en /tmp, con los objetos existentes
  como alternativos de solo lectura. El índice real del usuario no se cambió.
- `git diff --check`, también con archivos nuevos en el índice temporal.
- Comprobación local de sintaxis JSON de los esquemas D2/D3 con la biblioteca
  estándar de Python; búsqueda de panic/unwrap/expect en la biblioteca nueva
  sin coincidencias. Sin dependencias externas ni acceso a red.

Solo se modificaron Cargo.toml, Cargo.lock, el crate nuevo y esta bitácora.
No se modificaron crates existentes, reglas clean-room ni rutas protegidas.
No se hizo commit ni push. Revisor clean-room: PENDIENTE (J4).

## Ronda J4 r7

- La fábrica `spring.linear` admite exactamente un GDL por elemento y rechaza
  varios GDL distintos válidos con ENG-E-0013: con varios GDL el resorte es de
  rango uno (K=k·bbᵀ) y acopla los GDL; para resortes independientes se usa un
  elemento por GDL. `LinearSpring::new` conserva el acoplamiento documentado.
  Test `factory_rejects_multi_dof_springs_that_would_couple_dofs` para 2..3 GDL
  en D2 y 2..6 en D3, sin cambios en el flujo ni en los códigos de error.

## Corrección del diagnóstico · 2026-10-13

- El diagnóstico de la fábrica para listas vacías o de más de seis entradas
  indica que requiere exactamente un GDL de la dimensión seleccionada.
  `factory_empty_dofs_reports_exactly_one_dof_required` fija el mensaje y
  ENG-E-0010 para la lista vacía en D2 y D3.
- Documentación en español de la fábrica, lib.rs y esta bitácora: exactamente
  un GDL en la fábrica; uno o varios de la misma familia en la API directa,
  sin modificar `LinearSpring::new` ni las validaciones de la fábrica.
- Verificación sin red (`CARGO_NET_OFFLINE=true`): `cargo fmt`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace` y
  `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace` pasan.
  El crate aporta ahora 16 pruebas de integración y un doctest.
- `cargo check --workspace --target wasm32-unknown-unknown` pasa (con el
  target instalado en el entorno local) y `bash clean-room/check.sh origin/main` da OK.
- Sin commit ni push.
