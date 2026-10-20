# Bitácora clean-room · ENG-GEO-004

- Ticket / PR: ENG-GEO-004 · MAM-45; PR #172.
- Autor(es): Codex gpt-6.1-sol high, 2026-10-20 (America/Guayaquil).
- Rol: implementador; especificador: J2 (ficha suministrada en el ticket).
- Contrato implementado: generadores `tz.api_clay`, `tz.api_sand`, `qz.api`,
  `tz.user`, `qz.user`; tablas inmutables de resistencia axial local,
  consumibles por el futuro pilote ENG-GEO-005 y, cuando no hay ablandamiento,
  por la ley multilineal elástica ENG-MAT-002 mediante adaptadores explícitos.
- Fuentes permitidas usadas: American Petroleum Institute, *Recommended Practice
  2GEO — Geotechnical and Foundation Design Considerations*, primera edición,
  abril de 2011, con Addendum 1, octubre de 2014, reafirmada 2021 (ISO
  19901-4:2003 modificada), según la ficha de J2; no verificado por esta sesión.
  §8.4.2, figura 2 y tabla 2 (fuste); §8.4.3, figura 3 (punta).
  Los vértices y el alcance fueron suministrados por J2 en el ticket;
  esta sesión no abrió el PDF ni consultó fuentes externas.
- Derivaciones propias: interpolación, ramas negativas y adaptación de unidades
  descritas abajo. Código propio; no se copió texto ni figuras de la fuente,
  solo los vértices numéricos que definen las curvas.
- Código de terceros copiado: ninguno.
- Validación: vértices publicados citados y oráculos analíticos propios para
  interpolación, tracción y tablas artificiales; mismo harness nativo/wasm.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código ni resultados de CalculiX o LAPACK,
  ni resultados de otros motores, ni código de fuentes prohibidas.
- Revisor clean-room: J4 r1 sobre 99fc5f9bf5820e7dd3d7e1c09f38a8cc9b5e74f4
  (APTO con seguimiento, 2026-10-20); r2 sobre
  47ef2f93aee5bfac3d7152d0084647face2d174f; r3 sobre
  90c6534a36bdaa7e1c9a69216c54b938b821e289 (APTO).

## Autorización y límites de la fuente

Jonathan confirmó el 2026-10-20 que dispone de copia legal de API RP 2GEO y
concedió la dispensa ENG-GOV-004 **limitada a API RP 2GEO en ENG-GEO-004**:
implementar estas curvas con código propio y citas por sección/figura/tabla,
sin copiar texto ni figuras. La dispensa consta en `docs/CLEAN_ROOM.md` §4
para este PR, editado por J2; el seguimiento H1 de J4 r1 añade únicamente
la aclaración autorizada sobre los vértices numéricos como constantes del código.
El PDF no se incorpora al repositorio. La aprobación directa de la fusión
corresponde a Jonathan; no se sustituye por esta bitácora.

No se usó el libro de Reese/Isenhower/Wang (2006), ni sus datos, curvas,
tablas o valores. Coyle–Reese (1966) solo puede representarse mediante puntos
aportados por el usuario a `tz.user`: no hay datos incorporados de ese método.
Se reserva documentalmente `tz.coyle_reese_clay` como «no implementada —
pendiente de la fuente original», Coyle & Reese (1966), ASCE JSMFD.
No hay función que fabrique datos de ese método ni constantes inventadas.
ENG-GEO-003 / PR #168 no se consultó ni se usa como dependencia.

## Parámetros a cargo del usuario

Todos los escalares se reciben como `Option<Real>` para diagnosticar ausencias
con ENG-E-0003. No hay valores por defecto ni inferencias de datos.

- Fuste: `tmax_compression_pa`, `tmax_tension_pa` y `zpeak_m`, todos positivos
  y finitos; tracción y compresión pueden tener igual escala por decisión
  explícita del usuario. La fuente refiere f(z) y ft(z) a §8.1; su cálculo,
  incluidas ecuaciones 17–21, queda **pendiente y fuera de alcance**.
- `zpeak_m` se aporta directamente: §8.4.2 recomienda zpeak/D = 0.01 para
  diseño rutinario y señala 0.0025–0.02; no se adopta ningún valor ni se
  impone ese rango. D no es necesario para generar t-z si ya se conoce zpeak.
- Arcilla: `tres/tmax` obligatorio, 0 < ratio ≤ 1. Se documenta el intervalo
  0.70–0.90 de la fuente sin imponerlo. El residual se fija en 2·zpeak de la
  tabla 2; otros zres admitidos por la norma están fuera de esta entrega.
- Punta: `qp_n` (capacidad Qp en N, cálculo de §8.1 fuera de alcance) y
  `diameter_m` (m), positivos y finitos.
- `plateau_z_m` es obligatorio: mayor que zpeak en arena, 2·zpeak en arcilla,
  0.10·D en punta. Cierra la tabla; no se crea una malla por defecto.
- Tablas del usuario: puntos obligatorios, finitos, origen incluido y z
  estrictamente creciente. `UserTraction::Explicit` requiere ambas ramas
  y resistencia ≤ 0 si z < 0, ≥ 0 si z > 0;
  `Symmetric` solo recibe origen y rama positiva y refleja por elección
  explícita. `qz.user` permite una rama negativa cero aportada por el usuario;
  no impone el contacto unilateral de `qz.api` a una tabla libre.

## Derivaciones propias y decisiones de integración

1. Entre dos vértices (zi, ri), (zj, rj), se usa
   r(z) = ri + (rj − ri)·(z − zi)/(zj − zi). Es una interpretación propia
   de unión lineal de los vértices numéricos, no texto ni figura reproducidos.
   En la evaluación se elige el extremo más cercano para reducir cancelación;
   en un vértice se devuelve exactamente su ordenada. Fuera de la tabla se
   prolonga el tramo extremo, como ENG-MAT-002. Las tablas API terminan
   con mesetas; las tablas de usuario determinan sus propias prolongaciones.
2. Fuste negativo: reflexión de forma respecto del origen, con escala
   `tmax_tension_pa` independiente; se conserva el mismo zpeak y ratio
   residual. Es derivación propia, no se atribuye la reflexión a API.
3. Punta API: se añade (−zmeseta, 0) antes del origen; la prolongación del
   primer tramo produce Q = 0 para cualquier z < 0. Idealización propia
   de punta sin tracción. La meseta positiva empieza en 0.10·D.
4. Las unidades son z y D en m, t en Pa, Q en N, sin prefijos. z > 0 indica
   que el pilote baja respecto al suelo. Las ordenadas representan resistencia
   movilizada; la fuerza del suelo sobre el pilote tiene signo opuesto.
5. Adaptación a ENG-MAT-002: ε = −z/L, σ = −t para fuste unitario, o
   σ = −Q/A para punta. L (m) y A (m²) son referencias obligatorias del
   llamador, sin inferir geometría; se invierte el orden de puntos y se
   consume con `MultilinearMode::Both`. La punta API conserva su gap con ε > 0;
   se puede usar `CompressionOnly` para elegir en el origen la tangente del
   lado de compresión. Con `Both`, la ley elige la tangente derecha (cero en
   ese gap) en el origen; las resistencias coinciden en ambos modos.
   Las tablas de usuario bilaterales requieren `Both`.
   El futuro pilote debe integrar t sobre la superficie para obtener N.
6. La ley multilineal actual exige pendientes no negativas. El generador
   conserva el ablandamiento de arcilla/tablas de usuario; el adaptador lo
   rechaza con ENG-E-0010 y mensaje que identifica ENG-MAT-002. No se recorta
   ni se endurece artificialmente una curva. Ratio residual 1 es compatible.
7. No se añaden dependencias, barras, trusses ni ley axial estructural.
   La única ampliación a la guardia de determinismo permite `no_mangle` y
   función pública `extern "C"` en la ruta exacta del nuevo ejemplo wasm;
   no permite unsafe ni otras construcciones. Los hashes no se modifican.

## Fixtures y comprobaciones

`crates/engine-geo/tests/support/axial_benchmarks.rs` contiene cinco fixtures
compartidos con engine-validation nativo y el ejemplo `axial_wasm`:

- Arena: todos los vértices; zpeak = 0.01 m, tmax = 50000 Pa;
  t(0.0016) = 15000 Pa, t(0.01) = 50000 Pa.
- Arcilla: ratio 0.8; t(0.015) = (50000 + 40000)/2 = 45000 Pa;
  t(0.02) = t(0.03) = 40000 Pa; tracción escalada por 30000/50000.
- Punta: Qp = 1000000 N, D = 1 m; Q(0.013) = 500000 N;
  Q(0.1) = 1000000 N; Q(0.0075) = (250000 + 500000)/2 = 375000 N;
  Q(z < 0) = 0.
- Tabla t-z artificial: ramas independientes y ablandamiento.
- Tabla q-z artificial: reflexión solicitada explícitamente.

Los vértices usan comparación exacta de ordenadas; la interpolación admite
2e-14 de error relativo con escala mínima 1 SI, solo por redondeo binary64.
Las pruebas de contrato adicionales cubren cada ausencia obligatoria,
no finitos, dominios, orden, origen, simetría, escala, mesetas, rechazo del
ablandamiento en el adaptador, desbordamiento y subdesbordamiento.

Resultados finales (offline, Rust 1.99.0, target nativo x86_64 y
wasm32-unknown-unknown, Node v22.19.0):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: **685 passed, 0 failed, 0 ignored**
  (incluye 12 tests nuevos en engine-geo y 1 en engine-validation).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `cargo check --workspace --offline --target wasm32-unknown-unknown`: OK.
- Build release del ejemplo `axial_wasm` y ejecución Node: **5 fixtures
  aprobados**, sin imports externos.
- `bash clean-room/check.sh origin/main`: OK. Se utilizó `GIT_INDEX_FILE`
  temporal dentro de CARGO_TARGET_DIR para que el script inspeccionase también
  los archivos nuevos aún sin seguimiento; el índice real no se modificó.
- Los **16 hashes de docs/DETERMINISMO.md son idénticos**, comprobados con
  diez corridas nativas por caso del ejemplo `determinism_native` y cotejo
  exacto con la tabla publicada; ese documento no se modificó.
- `git diff --check`: OK. Sin commits ni cambios de Cargo.toml/Cargo.lock,
  workflows, CODEOWNERS, vendor o clean-room/** por el implementador.

Pendientes fuera del alcance implementado:
cálculos de capacidad de §8.1; fuente original para añadir Coyle–Reese;
y futuro pilote ENG-GEO-005. La ley ENG-MAT-002 actual no puede consumir
ablandamiento: el rechazo tipado del adaptador es el comportamiento previsto.

Seguimiento J4 r1 de MAM-45: validación de signo de `Explicit` en `tz.user`
y `qz.user`, con rechazo de signos opuestos y aceptación de ceros y signos
correctos; regresión analítica del subdesbordamiento de resistencia al adaptar
unidades (N4).

Comprobaciones de este seguimiento (offline, Rust 1.99.0):
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --offline -- -D warnings`,
`RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`,
`cargo check --workspace --offline --target wasm32-unknown-unknown`,
`bash clean-room/check.sh origin/main` y `git diff --check`: OK.
`cargo test --workspace --offline`: **688 passed, 0 failed, 0 ignored**,
incluidos los tres tests añadidos. Mutación N4 detectada: al retirar
el chequeo «Resistencia reducida a cero», el test
`adapter_rejects_resistance_reduced_to_zero_by_area` falla
(**0 passed, 1 failed**); pasa antes y después de restaurar `tz.rs`
exactamente, byte por byte (**1 passed, 0 failed** en cada ejecución).
Los **16 hashes de docs/DETERMINISMO.md permanecen idénticos** a la tabla
publicada, con diez corridas por caso en nativo y wasm, tanto en debug como
en release; las salidas nativas de ambos perfiles también son idénticas.
Build release de `axial_wasm` y ejecución Node v22.19.0: **5 fixtures
aprobados**, sin imports externos. Evidencia local en
`$CARGO_TARGET_DIR/j4-r1-evidence/`. Sin comprobaciones técnicas pendientes
de este seguimiento; su cierre corresponde a J4.
