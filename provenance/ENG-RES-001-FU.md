# Bitácora clean-room · ENG-RES-001-FU

- Ticket / PR: issue #69, seguimientos J4 r14 de #53, ENG-RES-001.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-14 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: recuperación `element/1` en `CaseResults::recover`;
  coherencia de cargas y miembros previa a CLI/API y `results/1`.
- Fuentes permitidas usadas: petición del ticket y APIs/código propios de
  engine-analysis, engine-assembly, engine-contracts y elementos del repositorio;
  `provenance/TEMPLATE.md` y `provenance/ENG-RES-001.md`. Sin red ni fuentes externas.
- Derivaciones propias: fórmulas y regla de equilibrio de esta bitácora;
  [pruebas analíticas](../crates/engine-analysis/tests/results.rs).
- Código de terceros copiado: ninguno. Sin dependencias nuevas.
- Validación: analítica propia, equilibrio nodal y viga bajo carga uniforme.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Regla de coherencia y tolerancia

El ensamblado conserva vectores de fuerzas equivalentes, no las distribuciones
originales de tramo ni los elementos. No puede reconstruir genéricamente las
cargas originales de `MemberRecovery`. Se conservan las cargas nodales explícitas
por caso en el mismo recorrido de ensamblaje, sin mezclarlas con las equivalentes
elementales. `global_nodal_force` expone esa fuente, escalada una vez por el factor.

Para cada miembro se obtienen las acciones de extremo **sobre el miembro** en
global como `p_e = Element::respond(u_e, committed).internal_force − Σ load_vector`.
Es la recuperación global de esas acciones usando `element/1`: no se infieren
transformaciones de los vectores locales, cuyo sistema de ejes depende del kind.
Las cargas suministradas se escalan una vez, incluyendo factores negativos o cero;
los desplazamientos prescritos conservan sus valores físicos.

Las acciones del miembro sobre el nodo tienen signo opuesto. Por cada GDL global:

```text
r_d = −Σ p_e,d + F_nodal,d + R_d + D_d
s_d = Σ |f_int,e,d| + Σ |f_carga,e,d| + |F_nodal,d| + |R_d| + Σ |D_d|
|r_d| / max(1, s_d) ≤ 1e-9
```

`R` se recalcula con K y las cargas físicas del sistema. `D` son las acciones
de restricciones de diafragma, si existen, recuperadas del mismo sistema.
La escala se calcula antes de cancelar los términos elementales; así los
momentos nulos en una viga simplemente apoyada admiten el redondeo al restar
fuerzas internas y cargas equivalentes. Cada GDL usa exclusivamente sus unidades
N o N·m: no se mezclan normas de fuerzas y momentos. El piso de 1 N o 1 N·m
sigue la convención SI de los informes de análisis; implica una tolerancia
absoluta de 1e-9 en componentes de escala menor que uno. El umbral relativo
1e-9 deja margen sobre el redondeo de doble precisión en ensamblaje, solución,
proyección y recuperación, y está muy por debajo de la discrepancia de la sonda.
Esta es una verificación numérica, no una tolerancia física de aceptación.

La lista de recuperación debe incluir todos los miembros del sistema; las
consultas parciales se hacen después mediante `query`. Un desequilibrio genera
ENG-E-0010 con caso, nodo, GDL global y miembros incidentes suministrados, más el
residual relativo en el mensaje. Un miembro omitido no puede identificarse por
id porque el ensamblado no conserva ese registro. Aportes erróneos que se
cancelen nodalmente, o miembros sin acciones en el estado concreto, pueden ser
indistinguibles por equilibrio; no se promete validar identidad completa del
modelo. No se modifica historia ni estado aceptado/de prueba.

Se verifican tamaños y finitud de `respond.internal_force` y `load_vector`
antes de indexarlos para el equilibrio. Tamaños incoherentes generan ENG-E-0012
con el miembro. Se mantienen los controles de estaciones y acciones locales.

## Estaciones y alcance de extremos

Se normaliza −0.0 a +0.0 antes de ordenar/deduplicar estaciones y también al
recibir la salida elemental. La igualdad numérica admite ambas representaciones,
pero las estaciones publicadas, consultas y ubicaciones de extremos usan +0.0.
En empates algebraicos exactos se conserva la menor estación x/L.

La afirmación de extremos exactos bajo cargas uniformes queda acotada a frame2d:
se añaden raíces interiores de cortante y de la derivada del momento. Frame3d
no dispone de esa búsqueda y sus extremos pertenecen a las estaciones recuperadas.
La serialización `results/1` sigue fuera de alcance.

## Oráculos analíticos propios

Con L=4 m, w=6 N/m y carga transversal −w, el equilibrio de una viga
simplemente apoyada da Ry_i=Ry_j=wL/2=12 N. Integrando el cortante:

```text
M(x) = wx(L−x)/2
M′(x) = w(L−2x)/2 = 0 => x=L/2
Mz_max = wL²/8 = 12 N·m
```

La sonda sin cargas de recuperación utilizaba solo los giros de la solución
cargada, produciendo Mz máximo 8 N·m. Ahora se rechaza por equilibrio nodal;
la misma solución recuperada con sus cargas da 12 N·m, incluso solicitando solo
los extremos, porque se añade la estación crítica.

En la viga biempotrada en flexión, la simetría y los giros nulos dan acciones
de extremo Mi=+wL²/12 y Mj=−wL²/12, de módulo 8 N·m. Por cara positiva:
`M(x)=−wL²/12+wx(L−x)/2`, mínimos −8 N·m en ambos extremos y máximo
wL²/24=4 N·m en el centro. Se conserva este oráculo existente.

Con par terminal +8 N·m, equilibrio: Ry_i=14 N y Ry_j=10 N;
`M(x)=14x−3x²`, pico en x=14/6 m con valor 14²/(2w). Una carga −7 N
directamente sobre el apoyo aumenta su reacción a 21 N sin cambiar el diagrama.
La prueba nueva verifica esa combinación, además de rechazar cargas cambiadas,
miembros omitidos y un elemento con rigidez distinta a la ensamblada.

Un elemento falso propio delega la física al frame del repositorio y altera
longitudes de salida, estaciones o el signo del cero para probar la frontera.
N, Vz, T y My constantes y nulos permiten probar empates exactos sin depender
de redondeos de Mz. No se usan archivos, procesos ni fuentes externas en fixtures.

## Verificación local

Verificación del 2026-10-14, sin red, con `CARGO_NET_OFFLINE=true` y
`CARGO_TARGET_DIR=/workspace/.cargo-target` en todos los comandos Cargo.
Se usó `RUSTFLAGS='--cfg eng_res001_fu_69'`, una marca sin usos en el código,
para aislar los artefactos de otras compilaciones del directorio compartido:
antes de aislarlos, se observó una dependencia cacheada que carecía del nuevo
método pese a estar presente en el archivo fuente.

- `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, 388 pruebas (incluidos doctests), cero fallos;
  13 pruebas de integración de resultados, cuatro nuevas.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, usando
  `PATH=/home/box/.cargo/bin:$PATH` y el toolchain stable y destino ya instalados.
  Valida compilación, no ejecución en un runtime wasm. No se descargó nada.
- `bash clean-room/check.sh origin/main`: OK, con los seis archivos del cambio
  en el índice para incluir la bitácora nueva.
- `git diff --check` y `git diff --cached --check`: OK. No se añadieron
  `panic!`, `unwrap` ni `expect` al código de biblioteca. Sin cambios a
  `docs/CLEAN_ROOM.md`, `.github/**` ni `clean-room/**`.

### Reversión temporal comprobada

Se guardó el `results.rs` corregido y se sustituyó temporalmente por
`git show HEAD:crates/engine-analysis/src/results.rs`, manteniendo todas las
pruebas nuevas intactas. `cargo test -p engine-analysis --test results`
devolvió exit 101: **9 pruebas existentes pasaron y las 4 nuevas fallaron**.

- Sonda `loads=[]`: se aceptó la recuperación; falló la exigencia ENG-E-0010.
- Cargas cambiadas/miembros incoherentes: se aceptó la primera recuperación
  con cargas omitidas; falló la exigencia de diagnóstico con entidades.
- Estación −0.0: bits obtenidos 9223372036854775808, esperados 0 para +0.0.
- Elemento falso: las salidas malformadas de `respond`/`load_vector` no eran
  comprobadas; falló la exigencia ENG-E-0012. Los controles preexistentes de
  tamaños locales y estaciones se conservan y también se ejercitan.

El archivo corregido se restauró mediante `finally` antes de las verificaciones
del workspace. Las 13 pruebas de resultados pasaron con la corrección, y el
workspace final pasó completo. No se revirtió ni modificó el historial Git
para esta comprobación.

Herramientas nativas de test/rustdoc: Cargo/rustc 1.85.1; Clippy 0.1.99 y
compilación wasm con stable/rustc 1.99.0 ya instalado. Sin push.
