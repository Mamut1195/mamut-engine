# Especificación normativa de mamut-toml/1

- Ticket: MAM-939 (T1).
- Fecha: 2026-10-31 (America/Guayaquil).
- Estado: decisión documental; implementación y verificación funcional posteriores.
- Decisión arquitectónica: [ADR-006](../adr/ADR-006-mamut-toml.md).
- Procedencia: [ENG-TOML-001](../../provenance/ENG-TOML-001.md).

## 1. Alcance y autoridad

Las palabras DEBE, NO DEBE y PUEDE expresan requisitos, prohibiciones y
opciones del formato. Este documento define una representación TOML legible
de [engine-input/1](../../crates/engine-api/schemas/engine-input-1.json).
La sintaxis de referencia es TOML 1.1.0. Las decisiones vigentes de Jonathan
para MAM-939 prevalecen sobre el plan de entrada TOML, revisiones v4, v4.1,
v4.2 y v4.3. El resumen de tickets se considera desactualizado en alias,
salidas y tratamiento de identificadores equivalentes.

La conversión pertenece exclusivamente a engine-cli y ocurre antes de
engine-api. NO DEBE modificar engine-api, engine-wasm, engine-input/1 ni
results/1. La Opción 0 conserva cli-error/1 y cli-describe/1: en este último,
inputs.format sigue siendo const "json". No se añaden campos ni códigos.
Esta especificación no declara que el CLI actual implemente TOML.

La dependencia prevista es toml =1.1.8+spec-1.1.0, con
default-features=false y std, serde, parse, display; sin unbounded.
Solo se incorpora en engine-cli, mediante MAM-938. Jonathan edita Cargo;
esta entrega documental no cambia dependencias.

El versionado de mamut-toml se rige por MAM-990. Los asuntos de la sección 10
permanecen sin resolver; este documento no los convierte en decisiones.

### Decisiones del consejo · ronda 2

Cada fila queda registrada como **decidido por el consejo (delegación de Jonathan), 2026-10-31, pendiente de revisión de Jonathan**.
Estos puntos salen del registro DESCONOCIDO; se conservan los identificadores
de los demás puntos abiertos.

| Punto cerrado | Decisión |
| --- | --- |
| (b), U2 | Recuento: 32 códigos en total, 14 ENG-E, 9 CLI-E y 9 API (4 API-E, 4 API-W, 1 API-I); 27 son `*-E-*`. Comprobado en error-codes/1 (docs/contracts/error-codes-1.json) en la base actual 4fcde46d45ccff13c2a89f23123720d361d9fe96, con el mismo resultado. |
| (a), U1 | NaN/±inf y demás números no finitos → en `run`, ENG-E-0008, salida 2, como en JSON; en `validate`, diagnóstico ENG-E-0009 en validate-report/1, salida 2, como en JSON (engine-cli, lib.rs, validate_input). Datetime/fechas y horas TOML → CLI-E-VALIDATION, salida 2. |
| (d), U4 | vendor/ existe hoy; vendor-crates/ es futuro. No se crea ni se cambia ninguno en esta entrega. |
| (f), U6 | MAM-993 usa [ADR-005](../adr/ADR-005-alias-name-idmap.md); este documento arquitectónico es ADR-006. No hay choque. |

### Decisiones del consejo · ronda 3

Atribución de la decisión: **Decisión del consejo (delegada por Jonathan, relatada por J0, 2026-10-31), pendiente de revisión de Jonathan**.

| Punto | Decisión |
| --- | --- |
| U3 | Dos clases de hash distintas: {4, 4.0, 4e0} y {0, 0.0, -0.0}. Las representaciones de cada clase comparten hash; las dos clases tienen hashes distintos. |

U3 sale del registro de puntos abiertos.

### Decisión de Jonathan · U8

Atribución: **decisión de Jonathan del 2026-10-31, relayada por J0, opción A**.

| Punto | Decisión |
| --- | --- |
| U8 | En TOML, un archivo de entrada que falta, un error de lectura, UTF-8 inválido o un tamaño excedido producen CLI-E-DOCUMENT, salida 2, invalid_input, igual que el código actual y JSON. CLI-E-INTERNAL, salida 70, internal, queda solo para error de E/S interna o de salida (por ejemplo, fallo al escribir la salida) y fallo interno. |

U8 sale del registro de puntos abiertos. No hay cambio de código: la regla
coincide con el comportamiento actual de main para JSON.

## 2. Documento e identidades

La raíz DEBE contener format = "mamut-toml/1" y units = "SI".
El emisor DEBE escribirlos primero, en ese orden; el lector no exige orden
textual de claves. Una versión de formato no soportada se rechaza.
El bloque [engine] está prohibido: no existe vía para introducir JSON crudo.
Tampoco se aceptan schema, model, axes ni analyses como claves de la raíz.

La conversión deriva schema = "engine-input/1", model.axes = "Z-up" y
analyses = [analysis]. [analysis] es único y su method es obligatorio.
Las opciones de cada método se anidan en analysis.options del JSON,
excepto linearStatic, que solo contiene method. [options] de la raíz es
distinto de esas opciones de análisis.

Las colecciones omitidas se convierten en colecciones vacías: nodes,
members, sections, laws, supports, masses, cases y combinations satisfacen
así los campos obligatorios del JSON. Las extensiones prescribed,
diaphragms y springs también admiten vacío. [options] omitido produce {};
allow_experimental ausente mantiene false. El export DEBE omitir
colecciones vacías. No se inventan propiedades materiales ni opciones
numéricas de análisis.

Todas las referencias son por id o nombre; nunca por posición de array.
En engine-input/1, los ids de nudos, miembros y springs son enteros entre
0 y 4294967295, ambos incluidos. Los ids de apoyos, masas, secciones y
combinaciones, los nombres de leyes y los nombres de casos son strings.
Para alias de texto de nudos/barras/springs: definido en MAM-993, [ADR-005](../adr/ADR-005-alias-name-idmap.md).

Los identificadores textuales admiten "" y tienen un máximo de 256 bytes
UTF-8, también al exportar. La longitud se mide antes de cualquier
truncado para diagnósticos. Superar el límite produce error sin eco.
No hay normalización NFC/NFD; las identidades se comparan por bytes.
Los ámbitos son los del esquema; springs comparte el de members.

Los GDL son Ux, Uy, Uz, Rx, Ry, Rz, con esas mayúsculas. Las cargas
Fx, Fy, Fz, Mx, My, Mz corresponden respectivamente a esos seis GDL.
Las unidades son SI: m, kg, s, N, Pa, rad y momentos N·m.
Los campos geotécnicos conservan los sufijos del esquema, incluidos _m,
_kn_m3, _kpa, _s, _m2_s, _per_kpa y _m_s; no se renombran ni se
convierten implícitamente a otra unidad.

## 3. Mapeo completo del esquema

D significa forma directa en su nivel: estructura y claves conservadas.
R significa derivación, renombre, aplanado o valor por defecto. La raíz,
descrita en la sección 2, no es una definición de $defs y no se cuenta.
La tabla contiene exactamente las 35 definiciones del esquema:
15 D y 20 R. Los hijos siguen su propio mapeo aunque el contenedor sea D.

| Definición de $defs | Forma TOML | Clase | Renombres, derivaciones, valores por defecto y rechazos |
| --- | --- | --- | --- |
| id | id o referencia entera | D | Entero 0..4294967295. No se convierte un float en identidad. |
| dof | string de GDL | D | Solo Ux, Uy, Uz, Rx, Ry, Rz. |
| method | analysis.method | D | Siete valores literales del esquema; no se traducen. |
| model | Hijos en la raíz | R | Se reconstruye model; axes se deriva como Z-up. Sin tabla model ni axes escrito. Colecciones omitidas: vacías. |
| node | [[nodes]] | R | id, x, y; coordinates = [x,y,z], z = 0 si falta. dim y dofs locales prevalecen sobre [defaults]; ambos deben quedar definidos. D2 exige z = 0. |
| member | [[members]] | R | from y to forman nodes = [from,to]; section y law son referencias textuales. element, orientation, release_i, release_j y winkler se conservan. element ausente deja al motor sus valores frame2d.euler_elastic o frame3d.euler_elastic según dimensión. |
| parameter | Clave numérica en sección o ley | R | Cada par clave = valor forma {name,value}; no hay lista TOML de parámetros ni sobrescritura de duplicados. Reservas P1 en leyes. |
| section | `[sections.<nombre>]` | R | El nombre de tabla forma id; sus entradas numéricas forman properties, incluso si la tabla está vacía. |
| law | `[laws.<nombre>]` | R | El nombre forma name; las entradas numéricas forman parameters. kind, points y mode son metadatos conservados y están prohibidos como nombres de parámetro (P1). Sin kind se conserva la semántica histórica del motor. |
| support | [[supports]] | R | node y exactamente uno de restrained o type. id omitido: `N<nodo>`. type se expande según la dimensión del nudo; no se añaden tipos (P2). Colisión con id escrito: error sin eco. |
| dofValue | Clave de componente = número | R | Forma {dof,value}; emisión en orden Ux,Uy,Uz,Rx,Ry,Rz. Sin GDL repetidos. |
| mass | [[masses]] | R | id, node y claves Ux..Rz forman values; componentes ausentes se omiten, sin masa implícita. |
| load | [[cases.loads]] | R | node y claves Fx..Mz forman values con GDL Ux..Rz; componentes ausentes se omiten, sin rellenarlas con cero. |
| uniformLoad | [[cases.member_loads]] | R | member se conserva. force_per_length y moment_per_length son vectores de tres números; vector omitido se reconstruye como [0,0,0]. El export omite vectores completamente nulos. |
| case | [[cases]] | D | name y colecciones de cargas; loads omitido produce []; member_loads conserva su carácter opcional y admite vacío. Orden del documento conservado. |
| combination | `[combinations.<id>]` | R | Nombre de tabla → id; cada clave de caso y factor → terms. Sin casos inexistentes ni términos repetidos. |
| term | Clave de caso = factor; tablas gravity/lateral | R | En combinación, forma corta → {case,factor}. En pDelta se conserva explícitamente {case,factor}. No se inventa factor. |
| prescribed | [[prescribed]] | D | node, dof, value obligatorios; sin valor por defecto. |
| diaphragm | [[diaphragms]] | D | master, slaves obligatorios; identidades de nudos, sin renumeración por posición. |
| spring | [[springs]] | D | Mismas claves del esquema. id, nodes, kind, dof obligatorios; condiciones de k/law y direction/rotation en sección 4. |
| winkler | winkler en [[members]] | D | Exactamente {k} para D2 o {ky,kz} para D3, valores ≥ 0. Sin mezclar ramas; conserva restricciones del motor sobre liberaciones. |
| analysis | [analysis] | R | analyses, de longitud uno, se representa por una tabla. method selecciona rama; options se aplana. Opciones de otra rama: error. |
| massSource | [analysis.mass_source] | D | method, case, factor, direction, gravity obligatorios; opcional solo en modal y responseSpectrum. Sin suma implícita con masas nodales. |
| pDeltaOptions | [analysis] y tablas gravity/lateral | R | Se aplana options; seis campos obligatorios, sin tolerancias ni escalas por defecto. |
| modalOptions | [analysis] | R | Se aplana options; mode_count, mass_kind, directions obligatorios. directions es array de vectores, no de tablas espectrales. |
| spectrumTable | [[analysis.directions.spectra]] | D | pairs, units, damping, interpolation, extrapolation obligatorios. Espectros bajo su dirección; no existe analysis.spectra global. |
| spectrumDirection | [[analysis.directions]] | D | direction y spectra obligatorios; spectra tiene al menos una tabla. |
| spectrumOptions | [analysis] | R | Se aplana options; seis campos obligatorios y mass_source opcional. Conserva orden de direcciones y espectros. |
| newton_options | [analysis], nonlinearStatic | R | Se aplana options; cinco campos obligatorios; criterios adicionales conservan sus escalas positivas. Sin combinaciones ni diafragmas. |
| geo_parameter | Tabla en línea {value, provenance} | D | Ambos obligatorios; provenance solo measured o assumed. No se sustituye por número desnudo. |
| soil_layer | [[analysis.layers]], soilProfile | D | thickness_m obligatorio; gamma_kn_m3 y gamma_sat_kn_m3 son geo_parameter, con requisitos físicos adicionales del motor. |
| soil_options | [analysis], soilProfile | R | Se aplana options; layers y depths_m obligatorios. Sin valores geotécnicos implícitos; requisitos adicionales en sección 4. |
| consolidation_layer | [[analysis.layers]], consolidation1d | D | thickness_m y elements obligatorios en el esquema; cv_m2_s, mv_per_kpa y k_m_s conservan geo_parameter y exigencias del motor. |
| consolidation_load | [[analysis.history]] | D | time_s obligatorio en esquema; delta_stress_kpa conserva geo_parameter. Sin incremento de tensión inventado. |
| consolidation_options | [analysis], consolidation1d | R | Se aplana options; layers, drainage, history, output_times_s, theta, max_step_s obligatorios. water_unit_weight_kn_m3 mantiene geo_parameter. |

Los rechazos adicionales de esta representación incluyen el límite de id,
las reservas P1 y los duplicados que una tabla no puede representar.
La cobertura de las 35 formas no promete exportabilidad de cualquier JSON
válido sin atender a esas restricciones.

## 4. Claves permitidas: ALLOWED

Estas listas son exhaustivas. Se comparan claves literales ya leídas por
TOML, nunca rutas construidas concatenando puntos. Por ejemplo, "a.b" es
una clave y a.b expresa anidación TOML; no son intercambiables.
Una clave desconocida se rechaza. Los nombres dinámicos descritos en la
tabla son la única excepción a las listas cerradas.

En las tablas siguientes, «obligatorias» no implica que las demás claves
puedan tener cualquier tipo. Se conservan los tipos, rangos, cardinalidades,
enum, const y condiciones del esquema y los controles del motor.
Un array de tres componentes se escribe como [x,y,z]; una matriz de
rotación tiene tres filas de tres números.

### 4.1. Raíz, modelo y cargas

| Tabla o ubicación | ALLOWED | Obligatorias y condiciones |
| --- | --- | --- |
| Raíz | format, units, defaults, nodes, sections, laws, members, supports, masses, springs, prescribed, diaphragms, cases, combinations, analysis, options | format, units, analysis. Las demás colecciones admiten omisión según sección 2. |
| [defaults] | dim, dofs | Ninguna; solo aporta valores a nudos que los omiten. dim: D2 o D3. |
| [[nodes]] | id, x, y, z, dim, dofs | id, x, y; dim y dofs deben resultar de la tabla o de defaults. z ausente: 0. |
| [sections] | Nombres de sección dinámicos | Cada valor es tabla de sección. "" permitido. |
| `[sections.<nombre>]` | Nombres de propiedades dinámicos | Valores numéricos; forman properties. No existe clave envolvente properties. |
| [laws] | Nombres de ley dinámicos | Cada valor es tabla de ley. "" permitido. |
| `[laws.<nombre>]` | kind, points, mode y nombres dinámicos de parámetros numéricos | kind, points y mode nunca son parámetros. No existe subtabla params ni parameters. |
| [[members]] | id, from, to, section, law, element, orientation, release_i, release_j, winkler | id, from, to, section, law. orientation: tres números. release_i/j: booleano o tres booleanos, según esquema y dimensión. |
| members.winkler | k; o ky, kz | Exactamente una rama: k para D2; ky y kz para D3. Todos ≥ 0. |
| [[supports]] | id, node, restrained, type | node y exactamente una forma de restricción. id autoasignado si falta. |
| [[masses]] | id, node, Ux, Uy, Uz, Rx, Ry, Rz | id, node; values se construye incluso si queda vacío. |
| [[springs]] | id, nodes, kind, dof, k, law, direction, rotation | id, nodes, kind, dof. Condiciones debajo de la tabla. |
| [[prescribed]] | node, dof, value | Todas. |
| [[diaphragms]] | master, slaves | Todas. |
| [[cases]] | name, loads, member_loads | name; loads omitido: []. |
| [[cases.loads]] | node, Fx, Fy, Fz, Mx, My, Mz | node; values se construye con las componentes presentes. |
| [[cases.member_loads]] | member, force_per_length, moment_per_length | member; vectores omitidos: [0,0,0]. |
| [combinations] | Ids de combinación dinámicos | Cada valor es tabla. "" permitido. |
| `[combinations.<id>]` | Nombres de caso dinámicos | Cada valor es factor numérico de un caso existente. |
| [options] | allow_experimental | Opcional, booleano; ausente: false. |

Las leyes conservan kind ∈ {elastic, steel.bilinear_kinematic,
multilinear_elastic, elastic.cubic}, mode ∈ {both, compression_only,
tension_only} y points con entre 2 y 16384 pares numéricos.
El conversor no añade parámetros para completar una ley.

P2 mantiene solo pin, roller-x, roller-y y el empotramiento, cuyo literal
TOML puede escribirse "\u0066\u0069\u0078\u0065\u0064".
Esta escritura escapada conserva el valor existente. No se incorporan
otros type, incluido roller-z. Las listas exactas de la expansión por
dimensión quedan registradas como U10; restrained permite expresar
explícitamente los GDL. type y restrained juntos son error.

En springs, nodes contiene uno o dos ids; con dos, son nudos distintos.
kind = "spring.linear" exige k ≥ 0 y prohíbe law.
kind = "spring.nonlinear" exige law, prohíbe k y requiere nonlinearStatic.
direction y rotation son mutuamente excluyentes; direction tiene tres
componentes y rotation es 3×3. La ley se interpreta directamente como
fuerza/desplazamiento o momento/giro, sin transformación material.

### 4.2. [analysis] según method

method es obligatorio en todas las ramas. Ninguna admite una subtabla
options: las opciones del método están aplanadas en esta representación.
El carácter * marca los campos adicionales obligatorios del esquema.
No se permite ninguna clave de otra rama, aunque tenga un valor inocuo.

| method | ALLOWED, incluido method | Obligatorias adicionales |
| --- | --- | --- |
| linearStatic | method | Ninguna. |
| pDelta | method, gravity, lateral, tolerance, max_iterations, displacement_scale, rotation_scale | gravity*, lateral*, tolerance*, max_iterations*, displacement_scale*, rotation_scale*. |
| modal | method, mode_count, mass_kind, directions, mass_source | mode_count*, mass_kind*, directions*. |
| responseSpectrum | method, mode_count, mass_kind, directions, modal_combination, directional_combination, directional_factors, mass_source | mode_count*, mass_kind*, directions*, modal_combination*, directional_combination*, directional_factors*. |
| nonlinearStatic | method, case, load_factors, tolerance, max_iterations, force_scale, displacement_scale, displacement_tolerance, energy_scale, energy_tolerance | case*, load_factors*, tolerance*, max_iterations*, force_scale*. |
| soilProfile | method, layers, depths_m, groundwater_depth_m, water_unit_weight_kn_m3 | layers*, depths_m*. |
| consolidation1d | method, layers, drainage, history, output_times_s, theta, max_step_s, water_unit_weight_kn_m3 | layers*, drainage*, history*, output_times_s*, theta*, max_step_s*. |

linearStatic, pDelta, modal y responseSpectrum son métodos estables.
nonlinearStatic, soilProfile y consolidation1d requieren
[options] allow_experimental = true; se conserva el control de engine-api.

### 4.3. Tablas internas y restricciones de análisis

| Tabla o ubicación | ALLOWED | Obligatorias, tipos y condiciones |
| --- | --- | --- |
| [analysis.gravity], [analysis.lateral], pDelta | case, factor | Ambas en cada tabla. case referencia un caso existente. |
| [analysis.mass_source], modal o responseSpectrum | method, case, factor, direction, gravity | Todas. method: masses_from_case, system_case o masses_from_system_case; case existente; factor ≥ 0; direction de tres números; gravity > 0. |
| [[analysis.directions]], responseSpectrum | direction, spectra | Ambas. direction de tres números; spectra con al menos un espectro. |
| [[analysis.directions.spectra]] | pairs, units, damping, interpolation, extrapolation | Todas. pairs: al menos dos pares numéricos. 0 ≤ damping < 1; interpolation: linear o logLog; extrapolation: error o plateau. |
| [[analysis.layers]], soilProfile | thickness_m, gamma_kn_m3, gamma_sat_kn_m3 | thickness_m > 0 obligatorio en esquema. Los otros dos son geo_parameter. |
| [[analysis.layers]], consolidation1d | thickness_m, cv_m2_s, mv_per_kpa, k_m_s, elements | thickness_m y elements obligatorios en esquema; thickness_m y coeficientes son geo_parameter. elements entero 1..2048. |
| [[analysis.history]], consolidation1d | time_s, delta_stress_kpa | time_s ≥ 0 obligatorio en esquema; delta_stress_kpa es geo_parameter. |
| Cada geo_parameter, también en línea | value, provenance | Ambas. value numérico finito; provenance: measured o assumed. |

No hay tabla interna para directions de modal: es un array no vacío
de vectores de tres números. En ambos métodos dinámicos, mode_count es
entero 1..16384 y mass_kind es lumped o consistent. mass_source no
autoriza suma implícita de dos fuentes de masa.

En responseSpectrum, modal_combination es srss o cqc;
directional_combination es srss o weightedSum; directional_factors es
array no vacío de números ≥ 0. units de cada espectro es exactamente
periodSecondsAccelerationMetersPerSecondSquared o
frequencyHertzAccelerationMetersPerSecondSquared. No se confunde con
units de la raíz ni se reordena pairs.

En pDelta, 0 < tolerance < 1; max_iterations es entero 1..16384;
displacement_scale y rotation_scale son positivos.
En nonlinearStatic, tolerance y force_scale son positivos,
max_iterations es entero 1..16384 y load_factors contiene entre 1 y
16384 valores absolutos, admitiendo descarga y repeticiones. Cada
criterio displacement_tolerance o energy_tolerance requiere su escala
positiva correspondiente; todos los criterios activos deben cumplirse.
Se mantienen las incompatibilidades del motor con combinaciones y diafragmas.

En soilProfile, layers tiene entre 1 y 16384 entradas; depths_m,
entre 1 y 16384 valores ≥ 0. groundwater_depth_m y
water_unit_weight_kn_m3 son geo_parameter. Aunque el esquema no los
enumera todos en required, el motor exige gamma_kn_m3 en cada capa,
gamma_sat_kn_m3 bajo el nivel freático, groundwater_depth_m y
water_unit_weight_kn_m3 incluso en seco. No se rellenan valores ausentes;
se conserva el diagnóstico de datos incompletos. Profundidad positiva
hacia abajo y tensiones en kPa, con compresión positiva.

En consolidation1d, layers tiene entre 1 y 2048 entradas; history,
entre 2 y 257; output_times_s, entre 1 y 16384 valores ≥ 0;
theta está entre 0 y 1 y max_step_s > 0. drainage es top, bottom
o double. El motor exige cv_m2_s, mv_per_kpa, k_m_s y
water_unit_weight_kn_m3 explícitos con procedencia y conserva
cv = k/(mv·γw). history es creciente y empieza en cero; los tiempos
de salida son crecientes, sin extrapolación. Se conservan todos los
presupuestos de elementos, memoria, trabajo y coste del esquema.
Para soilProfile y consolidation1d, modelo estructural, casos y
combinaciones deben estar vacíos.

## 5. Controles anteriores a engine-api

El conversor DEBE realizar estos controles sin entregar datos inválidos
al motor:

1. Leer como máximo 8 MiB (8388608 bytes), antes de parsear; el byte
   siguiente excede el límite. El documento debe ser UTF-8.
2. Parsear con TOML 1.1.0 y rechazar claves o tablas duplicadas. No se
   permite la política de conservar la última aparición.
3. Justo tras parsear, comprobar profundidad máxima 64 sobre el árbol,
   antes de mapear o resolver referencias. El límite propio del lector
   es una segunda barrera; unbounded está prohibido.
4. Recorrer el árbol y rechazar datetime/fechas y horas TOML en todas sus
   formas con CLI-E-VALIDATION, salida 2; NaN, +inf, -inf y cualquier otro
   número no finito con ENG-E-0008, salida 2, en `run`, y con ENG-E-0009,
   salida 2, en `validate`, como en JSON. No existe null;
   ni el conversor ni el export lo introducen como sustituto de ausencia.
   Todos los enteros deben estar en ±(2^53−1), es decir,
   entre -9007199254740991 y 9007199254740991.
5. Comprobar format y units obligatorios, las listas ALLOWED literales,
   tipos, ramas, campos obligatorios y reservas P1. No hay normalización
   Unicode ni reinterpretación de claves con punto.
6. Comprobar límite de 256 bytes por id, identidades exactas y referencias:
   from/to, node, member, section, law, case, master/slaves, nodes de
   springs y referencias de términos y mass_source deben existir.
   No se tratan S01 y S1, ni 0 y 00, como ids duplicados.
7. Aplicar valores de defaults, z = 0 y vectores de carga omitidos;
   reconstruir las listas del JSON. Autoasignar id de apoyo `N<nodo>`
   cuando falte; si coincide con un id escrito, rechazar sin eco y
   señalar el apoyo escrito. No se renombra silenciosamente. Tampoco
   se permite que dos autoasignaciones produzcan el mismo id exacto.
8. Para cada miembro, calcular la intersección entre las claves de
   properties de su sección y las claves de parameters de su ley.
   Si no está vacía, producir error propio sin eco antes de engine-api.
   Los metadatos kind, points y mode no son parámetros.
9. Entregar engine-input/1 a engine-api, conservando sus validaciones
   físicas y topológicas. Convertir representación no certifica
   estabilidad, convergencia ni resultados.

No se deben fusionar duplicados de properties, parameters ni values al
exportar JSON: una propiedad, un parámetro o un GDL repetido produce
CLI-E-VALIDATION sin eco. Los ids duplicados son exclusivamente los
idénticos byte a byte; ENG-E-0002 conserva ese significado cuando
procede del motor. Las reglas del conversor usan las salidas de la sección 9.

La convención exacta de conteo de profundidad se registra en U11: el
límite y el momento del control están decididos, su contabilidad no se
inventa aquí.

## 6. Orden canónico del export

El export DEBE emitir las secciones de primer nivel en este orden:

format, units, defaults, nodes, sections, laws, members, supports,
masses, springs, prescribed, diaphragms, cases, combinations, analysis,
options.

Las cargas de cada caso se escriben dentro de ese caso. Se omiten las
colecciones vacías y los vectores de carga de barra completamente nulos;
el lector los reconstruye según las reglas anteriores.

| Contenido | Orden obligatorio |
| --- | --- |
| nodes, members, supports, masses, springs | Por id, con orden natural de la sección 7. Para ids enteros no negativos coincide con el orden numérico. |
| sections, laws | Por nombre, por bytes UTF-8; no por orden natural. |
| combinations | Por id con orden natural; términos por nombre de caso, por bytes. |
| properties, parameters | Por nombre de clave, por bytes UTF-8. |
| Componentes de GDL | Ux, Uy, Uz, Rx, Ry, Rz; cargas en correspondencia Fx, Fy, Fz, Mx, My, Mz. |
| cases, loads, member_loads, prescribed, diaphragms | Conservan el orden del documento. |
| directions, spectra, pairs, layers, history, output_times_s, load_factors | Conservan el orden del documento. No se ordenan muestras ni historia física. |

Las claves con punto o que no admitan escritura simple DEBEN escribirse
entre comillas para conservar la clave literal. Se conservan los bytes
Unicode, sin NFC/NFD. El export repetido de la misma entrada DEBE
producir los mismos bytes.

U9, decidido en [docs/V1.md](../V1.md) sección 13, fija esa escritura:
UTF-8, saltos LF y un solo salto al final. `format` y `units` se escriben
en la columna 0. Cada encabezado de tabla queda en la columna 0 y sus
claves llevan sangría de dos espacios, también en una tabla anidada.
No se extrae `[defaults]` quitando un valor que la persona escribió en
el nudo, la barra o una opción. Un booleano escrito se emite, incluido
`allow_experimental = false` y una liberación `false`. Siguen omitiéndose
las colecciones vacías y los vectores de carga de barra completamente
nulos, porque el lector los reconstruye.

Un float de valor entero solo PUEDE escribirse como Integer cuando
|v| ≤ 2^53−1. Por encima de ese límite DEBE conservar representación
float; por ejemplo, 1e20 puede emitirse como 1.0e20.
El export rechaza números no finitos con ENG-E-0008, salida 2.
U3 establece dos clases de hash distintas: {4, 4.0, 4e0} y
{0, 0.0, -0.0}. Las representaciones de cada clase comparten hash;
las dos clases tienen hashes distintos. Rige la atribución de la ronda 3
en la sección 1.

El hash de un documento TOML es el del JSON convertido, calculado mediante
la canonicalización de engine-api. No se calcula sobre los bytes TOML.
El export reordena arrays del JSON, y el orden de arrays participa en el
hash; no se promete igualdad con el hash del JSON fuente.

B2: el ensamblaje suma en orden de entrada. Al reordenar miembros
u otras colecciones que aportan contribuciones, puede variar el redondeo
en coma flotante.
Por tanto, el reordenamiento puede afectar resultados numéricos y sus
diagnósticos, además de input_sha256, recordFingerprint y huellas de pasos.
Las observaciones del prototipo sobre unos pocos modelos no demuestran
igualdad física universal. Las pruebas posteriores deben distinguir
estabilidad de bytes de una misma entrada, hash del JSON convertido y
comparación numérica entre entradas reordenadas.

## 7. Orden natural exacto: cmp_natural

El algoritmo opera sobre bytes UTF-8, sin locale ni normalización.
Solo los bytes ASCII 0..9 son dígitos, mediante u8::is_ascii_digit.
NO DEBE usarse char::is_numeric ni convertir una corrida numérica a entero.
Ⅻ, ٣, ９ y los superíndices no cuentan como dígitos.

1. Se parte cada id en corridas maximales alternas de dígitos ASCII y
   bytes no dígitos. Los dígitos ASCII no aparecen dentro de una secuencia
   UTF-8 multibyte; los cortes respetan sus fronteras.
2. Todo id con algún dígito precede a cualquier id sin dígitos.
   Los ids sin dígitos se comparan entre sí por bytes.
3. Entre ids con dígitos, se comparan las corridas correspondientes.
   Una corrida de dígitos precede a una corrida de texto.
   Dos corridas de texto se comparan por bytes.
4. Para comparar corridas de dígitos, se quitan los ceros iniciales.
   Si queda vacío se usa "0". Se compara primero la longitud restante
   y luego los bytes, sin conversión a entero de ningún tamaño.
5. Si todas las corridas comunes empatan, precede el id con menos
   corridas. Así, S1 precede a S1a y x10y a x10y2.
   S1 precede a S porque se aplica antes la agrupación por presencia
   de dígitos.
6. Si también empata el número de corridas, se comparan los bytes
   completos originales como desempate final. S01 precede a S1;
   0 precede a 00. Ambos pares se aceptan (H1).
7. Solo los bytes completos idénticos comparan igual. El comparador
   puede devolver igualdad; la colección rechaza el mismo id exacto
   repetido, sin confundirlo con equivalencia numérica.

La clave se precalcula una vez por id y se ordena mediante
sort_by_cached_key (H2). Debe contener la agrupación con/sin dígitos,
las corridas comparables y el desempate por bytes, sin retokenizar cada
comparación. La cota pública sigue siendo 256 bytes por id.

El ejemplo normativo exacto de orden es:

0, 1, 10, -1, S1, S2, S10, S-1, "", A, S.

### 7.1. Batería completa de v4.2, actualizada con H1 y H2

Cada fila cita el número original del caso en el plan v4.2 §3.
«Menor» y «mayor» son resultados del comparador; «error» se refiere
al control de una colección. El caso 32 excede la cota pública: se
conserva como requisito de comparación sin enteros y se registra la
tensión de su integración en U7. No se declara ejecutada esta batería.

| Caso | Entrada | Resultado exigido |
| --- | --- | --- |
| 1 | S1, S2 | S1 < S2. |
| 2 | S2, S10 | S2 < S10. |
| 3 | S1, S10 | S1 < S10. |
| 4 | N2, N10 | N2 < N10. |
| 5 | a1b2, a1b10 | a1b2 < a1b10. |
| 6 | a1b10, a2b1 | a1b10 < a2b1. |
| 7 | S01, S1 | Orden por bytes: S01 < S1; sin error. |
| 8 | 0, 00 | Orden por bytes: 0 < 00; sin error. |
| 9 | 0, 1 | 0 < 1. |
| 10 | 007, 8 | 007 < 8. |
| 11 | 99999999999999999999999, 100000000000000000000000 | El primero es menor, sin desbordamiento. |
| 12 | S99999999999999999999999, S100000000000000000000000 | El primero es menor, sin desbordamiento. |
| 13 | S1, S | S1 < S; los ids sin dígitos van después. |
| 14 | S1, S1a | S1 < S1a. |
| 15 | S1, S1-2 | S1 < S1-2. |
| 16 | S1a, S1b | S1a < S1b. |
| 17 | A1, a1 | A1 < a1, por bytes. |
| 18 | base, top | base < top, por bytes. |
| 19 | Z, a | Z < a, por bytes. |
| 20 | z9, a | z9 < a; tener dígitos precede al grupo sin dígitos. |
| 21 | Ⅻ | No tiene dígitos ASCII; grupo sin dígitos. |
| 22 | ٣ | No tiene dígitos ASCII; grupo sin dígitos. |
| 23 | N1, N٣ | N1 < N٣. |
| 24 | n, ñ | n < ñ, por bytes UTF-8. |
| 25 | "\u00e9", "e\u0301" | Distintos y aceptados; la segunda secuencia precede a la primera por bytes. Sin normalización. |
| 26 | 🏗1, 🏗2, 🏗10 | 🏗1 < 🏗2 < 🏗10. |
| 27 | 1a, a1 | 1a < a1. |
| 28 | S1, Sa | S1 < Sa. |
| 29 | "", a | "" < a dentro del grupo sin dígitos. |
| 30 | 1, "" | 1 < "". |
| 31 | S1, S1 | Comparador: igualdad, resultado 0. Si son dos registros, mismo id exacto duplicado. |
| 32 | S seguido de 4096 nueves; S1 seguido de 4096 ceros | Comparador: primero menor, sin convertir a entero. Conversor público: longitud excedida, error sin eco; integración pendiente en U7. |
| 33 | S seguido de 100 ceros y 5; S5 | Orden por bytes: primero menor; sin error por equivalencia. |
| 34 | Colección ["a","a"] | Error por mismo id exacto repetido; nunca por equivalencia numérica. ENG-E-0002 solo conserva ese significado en el motor. |
| 35 | x10y, x10y2 | x10y < x10y2. |
| 36 | S2/S10 y S10/S2 | Menor y mayor, respectivamente; antisimetría. |

Las pruebas posteriores deben cubrir totalidad, antisimetría,
transitividad, idempotencia y permutaciones deterministas, incluidos
empates numéricos resueltos por bytes. Los casos 7, 8 y 33 sustituyen
los antiguos resultados de error por orden por bytes. También deben
probarse "" en apoyos y secciones, 256 y 257 bytes por id, y
colisión de `N<nodo>` con un id escrito.

## 8. Diagnósticos sin eco

Los errores DEBEN usar el sobre cli-error/1 existente, sin campos nuevos.
Solo se permite obtener message() y span() del error TOML; nunca
Display, to_string() ni Debug, ni interpolaciones que los invoquen.
message() también debe sanearse: si contiene contenido de la entrada,
se sustituye por texto propio constante. No se incluyen fragmentos
fuente, valores, ids completos ni rutas de ejecución.

span() expresa posiciones en bytes. Se convierte a línea y columna
en caracteres UTF-8 para presentación, con una prueba multibyte.
Los punteros se forman por segmentos, escapando ~ como ~0 y / como
~1 conforme a RFC 6901. Cada segmento de clave emitido se limita a
64 bytes sin romper UTF-8; si se añade …, cabe dentro de esos 64 bytes.
Una clave desconocida se señala por el padre y su ubicación, sin
reproducir la clave. La línea y la columna no requieren campos nuevos.
La convención de base de estas coordenadas queda en U11.

La colisión de propiedades de sección y parámetros de ley lleva
error propio sin eco y puntero a la ubicación del parámetro en la ley.
Si el nombre afectado es contenido sensible, se señala su tabla padre.
No se usa el truncado como permiso para revelar parte de un valor.
Como segunda barrera, con entrada TOML el CLI DEBE sustituir cause y
message de ENG-E-0002 por texto constante sin nombre.

Las pruebas de aceptación deben usar un valor centinela en duplicados,
clave desconocida, tipo incorrecto, id demasiado largo, colisión de
apoyo y colisión sección/ley. Ni convert ni run deben repetirlo,
ni siquiera en stderr. La ubicación puede ser parcial cuando el lector
no aporte span; no se inventa un fragmento para suplirlo.

## 9. Salidas aprobadas

Se reutiliza el [registro de códigos](../contracts/error-codes-1.json).
Las siguientes categorías corresponden a fallos anteriores al motor.
No crean códigos ni alteran las salidas de análisis de engine-api.

| Situación | Código | Salida de proceso | Categoría existente |
| --- | --- | --- | --- |
| Documento TOML inválido, sintaxis o duplicación de claves | CLI-E-DOCUMENT | 2 | invalid_input |
| NaN/±inf y cualquier otro número no finito en `run`, como en JSON | ENG-E-0008 | 2 | invalid_input |
| NaN/±inf y cualquier otro número no finito en `validate`, como en JSON: diagnóstico en validate-report/1 | ENG-E-0009 | 2 | invalid |
| Datetime/fechas y horas TOML en todas sus formas | CLI-E-VALIDATION | 2 | invalid_input |
| Incumplimiento de reglas del formato, límites o referencias; colisión de alias | CLI-E-VALIDATION | 2 | invalid_input |
| Argumentos o flags inválidos | CLI-E-USAGE | 2 | usage |
| Versión de formato no soportada | CLI-E-UNSUPPORTED | 3 | unsupported |
| Archivo de entrada TOML que falta, error de lectura, UTF-8 inválido o tamaño excedido | CLI-E-DOCUMENT | 2 | invalid_input |
| Error de E/S interna o de salida, o fallo interno | CLI-E-INTERNAL | 70 | internal |

Los fallos del conversor se emiten por stderr como cli-error/1; stdout
queda vacío si todavía no se comenzó a escribir una salida. Un fallo
durante escritura puede dejar bytes parciales. No se fabrican results/1
ni engineResults para un análisis que no se ejecutó. La validación
directa que sí llega a engine-api conserva su informe existente; no se
rediseña validate-report/1 en este ticket.

La clasificación de fechas y no finitos está decidida por el consejo,
según el registro de la sección 1. La regla de U8 está
decidida por Jonathan (opción A), según la sección 1: archivo de entrada
TOML que falta, error de lectura, UTF-8 inválido o tamaño excedido →
CLI-E-DOCUMENT, salida 2, igual que JSON, el código y el README de
engine-cli; CLI-E-INTERNAL, salida 70, solo para E/S interna o de salida
y fallo interno. No se alteran 0/1 de
éxito/aviso ni 4/5/6 de análisis, cancelación y tiempo límite.

## 10. Registro de puntos abiertos

U9 quedó decidido en [docs/V1.md](../V1.md), sección 13. Las demás filas
siguen en **DESCONOCIDO — pendiente de decisión de Jonathan**.
No se elige una alternativa por silencio, ni se presenta como verificada.

| Punto | Estado | Contradicción o precisión pendiente |
| --- | --- | --- |
| U5 | DESCONOCIDO — pendiente de decisión de Jonathan | definido en MAM-993, [ADR-005](../adr/ADR-005-alias-name-idmap.md). |
| U7 | DESCONOCIDO — pendiente de decisión de Jonathan | Caso 32 de v4.2, con 4096 dígitos, frente al máximo público de 256 bytes por id. La comparación matemática y el rechazo público están descritos; queda pendiente el alcance de la prueba que exigía aceptación del caso largo. |
| U9 | Decidido en docs/V1.md, sección 13 | Dos exports del mismo modelo son los mismos bytes: UTF-8, LF, un salto final, sangría de dos espacios y las claves en el orden de la sección 6. No se borra un valor que la persona escribió. |
| U10 | DESCONOCIDO — pendiente de decisión de Jonathan | El plan v4 exige expandir los cuatro type de apoyo según dimensión, pero no enumera las listas de GDL para todas las dimensiones y conjuntos activos. El esquema solo define restrained. No se deducen listas nuevas para cerrar ese vacío. |
| U11 | DESCONOCIDO — pendiente de decisión de Jonathan | Profundidad 64 y posiciones UTF-8 están decididas; las fuentes permitidas no precisan si la raíz cuenta como nivel ni la base de línea/columna. Deben concretarse antes de establecer expectativas de frontera. |

El rendimiento y el umbral de la prueba de 8 MiB con muchos tramos por id
son **DESCONOCIDO — pendiente de decisión de Jonathan** después de medir
en el entorno de integración. T1 no presenta mediciones nuevas.

## 11. Verificación exigida a la implementación posterior

La cobertura deberá recorrer las 35 definiciones y sus referencias,
ramas oneOf y condiciones allOf/if, relacionando campos del esquema
con este mapeo en ambos sentidos. Un campo nuevo sin forma documentada
debe hacer fallar la comprobación. Se cubrirá cada valor enum y const.

Se requieren negativos por tabla ALLOWED y por método, tipos inválidos,
reservas P1, referencias ausentes, duplicados exactos, intersección
sección/ley, fechas, no finitos, null de entrada al export, enteros fuera
de rango, ids fuera de u32, profundidad 65 y 81, tamaño 8 MiB más un
byte y claves extensas. Se cubrirán las fronteras 2^53−1 y 2^53,
1e20, claves con punto, Unicode y números que desborden a infinito.
Fechas y horas deben producir CLI-E-VALIDATION/2; NaN/±inf y los demás
no finitos, ENG-E-0008/2 en `run` y ENG-E-0009/2 en `validate`;
al exportar, ENG-E-0008/2. Se comprobarán las dos
clases de hash de U3: {4, 4.0, 4e0} y {0, 0.0, -0.0}, con igualdad dentro
de cada clase y diferencia entre clases. Para TOML, archivo de entrada
que falta, error de lectura, UTF-8 inválido o tamaño excedido deben
producir CLI-E-DOCUMENT/2; un fallo al escribir la salida, CLI-E-INTERNAL/70.
Las expectativas de U7 y U11 requieren decisión antes de
cerrar sus pruebas.

La sintaxis TOML 1.1 incluye tablas en línea multilínea, coma final
permitida por esa gramática y escapes \e y \xHH. Un diferencial con
un lector TOML 1.0 solo puede usar el subconjunto 1.0; no acredita
cobertura 1.1. Se emplearán casos y mutaciones deterministas.

Las pruebas de export comprobarán repetición de bytes, omisión de vacíos,
conservación de literales, orden natural, orden por bytes y hash del
JSON convertido. No confundirán ese hash con el del JSON original ni
tomarán las mediciones del plan como pruebas ejecutadas en MAM-939.
La comparación física debe tener en cuenta B2.

## 12. Fuentes documentales

- Esquema propio: [engine-input/1](../../crates/engine-api/schemas/engine-input-1.json).
- Registro propio: [error-codes/1](../contracts/error-codes-1.json).
- Contrato actual: [README del CLI](../../crates/engine-cli/README.md)
  y [guía de agentes](../GUIA_CLI_AGENTES.md).
- Plan de entrada TOML, secciones v4, v4.1, v4.2 y v4.3; resumen de
  tickets TOML, con las salvedades de la sección 1.
- [TOML 1.1.0](https://toml.io/en/v1.1.0), especificación normativa de sintaxis; sin consulta de red
  en esta tarea.
- Decisiones vigentes de Jonathan comunicadas para MAM-939.
