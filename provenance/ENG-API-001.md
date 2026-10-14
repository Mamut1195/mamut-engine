# Bitácora clean-room · ENG-API-001

- Ticket / PR: ENG-API-001 · Fachada engine-api · incremento 1 · rama j2/eng-api-001.
- Autor(es): Codex gpt-6.1-sol high bajo dirección de J2, 2026-10-14 America/Guayaquil.
- Rol: implementador.
- Contrato implementado: engine-input/1 (parseo y validación topológica); results/1 (esquema estático reservado); metadatos, catálogo, JSON canónico, progreso y cancelación.
- Fuentes permitidas usadas: ticket local /workspace/fu/tickets/ENG-API-001.txt y engine-cli-py-plan.md (contexto de datos); docs/CONVENCION_EJES.md, secciones «Ejes globales y dimensión» y «Acciones, desplazamientos y giros»; APIs propias de engine-core, engine-model y engine-analysis::results; JSON Schema Draft 2020-12 (estructura declarativa, sin consulta de red).
- Derivaciones propias: mapeo de DTO a dominio y canonización descritos en crates/engine-api/README.md; no hay derivaciones mecánicas ni cambios al solver.
- Código de terceros copiado: ninguno. Dependencias serde 1.0.229 (derive) y serde_json 1.0.154 (float_roundtrip) del registro local, únicamente en engine-api; Cargo offline.
- Validación: oráculos de contrato y representación binary64; diagnósticos del catálogo/dominio propio; determinismo, -0.0, cancelación antes de empezar y entre todas las etapas. Sin oráculo numérico externo.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

Alcance: no se implementa run en este incremento; README detalla recuperación
linearStatic → results/1, SHA-256, combinaciones y demás métodos pendientes.

Verificaciones offline con `CARGO_TARGET_DIR=/workspace/.cargo-target` y el
toolchain stable instalado (`PATH=/home/box/.cargo/bin:$PATH`, Rust 1.99.0):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 431 pruebas (15 nuevas de engine-api), 0 fallos/ignoradas.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- Metadatos Cargo: engine-api en members/default-members, único consumidor de serde/serde_json; versiones del engine alineadas en 0.1.0.
- Documentos JSON Schema: estructura JSON y referencias locales verificadas, sin red.
- `bash clean-room/check.sh origin/main`: OK sobre el diff preparado para commit (incluidos archivos nuevos); revisión humana clean-room pendiente de J4.

## Incremento 2a · 2026-10-14

- Ticket / PR: ENG-API-001 · PR #88 borrador · rama j2/eng-api-001.
- Autor: Codex gpt-6.1-sol high bajo J2, 2026-10-14 America/Guayaquil.
- Rol: implementador. Revisor clean-room: PENDIENTE (J4).
- Contrato: run(engine-input/1, cancel, progress) → results/1 canónico;
  linearStatic, recuperación nodal/de todos los miembros y combinaciones;
  Outcome/Status y códigos 0/1/2/3/4/5/6 reservado/70. SHA-256 del input canónico.
- Fuentes permitidas: ticket ENG-API-001.txt y sección Interfaz CLI de
  engine-cli-py-plan.md como datos; contrato y README previos de engine-api;
  APIs propias de engine-model, engine-analysis (StaticLinear, CaseResults::recover,
  combine), engine-assembly y fábricas propias de frames/truss. Sin red.
- Derivaciones propias: equilibrio del voladizo R=P, M=PL; integración de
  EI y''=−P(L−x) con y(0)=y'(0)=0 → y(L)=−PL³/(3EI). Viga apoyada uniforme:
  simetría y equilibrio → R=wL/2; M(x)=wx(L−x)/2, raíz M'=0 en L/2 → wL²/8.
  Barra: EAδ/L=P → δ=PL/EA. Combinación: suma componente a componente
  1,2D+1,6L; prescritos de combinación advertidos según #85, repetidos rechazados.
- Coherencia #69: una colección de MemberLoad por caso/miembro alimenta tanto
  load_vector como MemberRecovery; se recuperan todos los miembros. Unión de
  estaciones críticas para compatibilidad de combinaciones, sin cambiar solver.
- Seguimientos abiertos: #85 emite API-W-COMBINED-PRESCRIBED y warning siempre
  que haya prescritos en salida combinada; #84 emite API-W-DIAPHRAGM-RECOVERY
  y warning. La prueba smoke de fachada no declara cerrada la cobertura del dominio.
- Código de terceros copiado: ninguno. Dependencias serde/serde_json existentes
  y sha2 0.10.9 (registro local); Cargo offline. Sin código de otros motores,
  sin oráculos externos ni acceso a red.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Validación: 16 pruebas nuevas de fachada, basadas en las derivaciones propias
  anteriores y contratos; determinismo y SHA con claves permutadas/-0.0,
  cancelación antes de empezar y en las siete fronteras de dos casos, métodos
  Unsupported, entrada inválida, mecanismo, avisos, asentamientos repetidos,
  varios miembros y compatibilidad de estaciones de combinaciones.

## Finalización del incremento 2a · J4 r22 · 2026-10-14

- Ticket / PR: ENG-API-001 · PR #88 borrador · rama j2/eng-api-001;
  revisión del WIP 19e297f después del merge eed09df de origin/main.
- Autor: Codex gpt-6.1-sol high bajo J2, 2026-10-14 America/Guayaquil.
- Rol: implementador. Revisor clean-room: PENDIENTE (J4).
- Fuentes permitidas: /workspace/fu/papi2a.txt, ENG-API-001.txt y
  engine-cli-py-plan.md (datos de alcance); implementación y contratos propios
  del workspace, README y esquemas de engine-api. Sin consultas de red.
- J4 r22: constantes públicas documentadas para bytes UTF-8, nodos, miembros,
  casos, total de cargas nodales/de miembro, combinaciones, profundidad y
  entradas de arrays auxiliares. Inspección léxica propia sin recursión ni árbol
  de entrada, anterior al parseo serde y al modelo/solver; los errores nombran
  el límite y máximo, con InvalidInput / API-E-INPUT / salida 2.
- Números: comprobación del token antes de serde; enteros negativos limitados
  a i64 y no negativos a u64, sin degradación silenciosa a float. Overflow f64,
  incluido -1e400, rechazado. Canonización de salida sobre Value, conservando
  normalización de -0.0 y determinismo, sin aplicar límites de input a results/1.
- Licencia: AGPL-3.0-only literal en engine-api, como bindings/python.
- Capacidades: run true para linearStatic y combinaciones; pDelta, modal y
  responseSpectrum siguen Unsupported (3). Se conservan los avisos de #84/#85
  y el rechazo de asentamientos repetidos; esos seguimientos no se cierran.
- Derivaciones propias adicionales: voladizo D3 sobre X con fuerza en Z,
  EI_y z''=−P(L−x) e integración con z(0)=z'(0)=0 → z(L)=−PL³/(3EI_y);
  r×F=+PL e_y → reacción −PL e_y. Barra D3 axial: EAδ/L=P → δ=PL/EA,
  equilibrio R=−P. Sin modificación del solver ni oráculo numérico externo.
- Pruebas adicionales: ocho de fronteras inclusivas y exceso de límites antes
  del parseo, claves escapadas, cadenas, UTF-8, cargas acumuladas, enteros
  gigantes y overflow f64, cancelación previa; dos analíticas para frame3d y
  truss3d. Se amplía el rechazo numérico en entradas completas de run.
- Código de terceros copiado: ninguno. Dependencias del registro local ya
  disponibles (serde, serde_json y sha2); todas las comprobaciones Cargo offline.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

Verificaciones finales sobre el incremento completo, incluido bindings/python
como miembro del workspace, con `CARGO_TARGET_DIR=/workspace/.cargo-target`,
`CARGO_NET_OFFLINE=true` y Rust 1.99.0 (`PATH=/home/box/.cargo/bin:$PATH`):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 462 pruebas, 0 fallos/ignoradas;
  engine-api: 41 (15 facade, 8 limits, 18 run).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `git diff --check`: OK. Sin panic/unwrap/expect en la biblioteca engine-api;
  sin cambios a docs/CLEAN_ROOM.md, .github/** ni clean-room/**.
- `bash clean-room/check.sh origin/main`: OK sobre el diff preparado para
  commit, incluidos los archivos nuevos. Revisión humana PENDIENTE (J4).

## Incremento 2c · 2026-10-14

- Ticket / PR: ENG-API-001 · PR #88 borrador · rama j2/eng-api-001.
- Autor: Codex gpt-6.1-sol high bajo J2, 2026-10-14 America/Guayaquil.
- Rol: implementador. Revisor clean-room: PENDIENTE (J4).
- Contrato: run de linearStatic, pDelta, modal y responseSpectrum → results/1
  canónico y SHA-256; capabilities publica run:true en los cuatro métodos.
  Un análisis por entrada, opciones mecánicas explícitas, sin factores de norma.
- Fuentes permitidas: APIs y documentación propias de engine-analysis (pdelta,
  modal, spectrum), engine-assembly, engine-loads::mass y engine-linalg;
  contrato previo de engine-api y requisitos del incremento recibidos de J2.
  No se consultaron repositorios hermanos ni oráculos numéricos externos.
- Derivación modal propia: equilibrio de barra EAδ/L=F → k=EA/L;
  m u''+ku=0 → ω²=k/m, f=ω/(2π), T=2π/ω. Para 1 GDL, φ=1/√m,
  Γ=√m, M*=m y participación=1. Dos barras independientes: autovalores
  20/80 s⁻² con m=5 kg por GDL; participación de 50% por modo.
- Derivación espectral propia: q=Γ Sa/ω², u=φq=Sa/ω² y V=m Sa;
  constantes de prueba Sa=3 m/s², k=100 N/m, m=5 kg → u=0,15 m, V=15 N.
  En dos osciladores independientes, cada GDL responde a su único modo;
  suma direccional ponderada con factores 1/2 multiplica la respuesta por 3.
- Derivación P-Delta propia: EI y''''+P y''=0, y(0)=y'(0)=0,
  y''(L)=0 y EI y'''(L)+P y'(L)=−H. Integrando y resolviendo constantes:
  y(L)=(H L/P)(tan α/α−1), α=L√(P/EI);
  A=y(L)/(HL³/(3EI))=3(tan α−α)/α³. Primer polo α=π/2 →
  Pcr=π²EI/(4L²). P/Pcr=0,2: comparación con 1/(1−P/Pcr) al 0,4%
  por aproximación del primer polo; error de malla decrece entre 4/8 elementos
  y con 8 es <1e−5 frente a la solución exacta, tolerancia iterativa 1e−10.
- Masas: usa masas nodales explícitas o masses_from_case/system_case (incluidas
  fuerzas equivalentes de miembro), fuentes simultáneas rechazadas; factor y
  gravedad del usuario. No incorpora rho automáticamente ni condensa GDL sin
  masa. Succión conserva ENG-E-0010 / invalid / 2, sin tolerancia adicional.
- Seguimientos abiertos: #86 siempre emite API-W-MASS-PROJECTION cuando se
  convierte carga a masa, también al fallar; advierte rechazo por redondeo con
  gravedad no alineada a ejes. #78 emite API-W-PDELTA-INERTIA al informar
  bloques 2×2 en la inercia (o metadato desconocido), también en unstable.
  Se publica metadato de pivotes sin cambiar copysign ni certificar su prueba.
  #84(c): no usa CaseResults::recover con P-Delta; solo desplazamientos y
  reacciones laterales independientes, sin esfuerzos de miembros. Conserva
  avisos previos #84/#85. No se declara cerrado ningún seguimiento.
- P-Delta: registra productores geométricos por miembro; gravedad con cargas
  de miembro se rechaza como Unsupported porque el productor recupera N desde
  deformaciones sin corrección de cargas de miembro. Convergence conserva
  opciones, inercia y métricas por iteración; NotConverged/Unstable → código 4.
- Progreso/cancelación: callbacks cooperativos en el dominio tras gravedad y
  cada iteración P-Delta, tras el eigensolver y cada modo modal, y por cada
  modo/dirección espectral. Cada solve/eigensolver es indivisible. Cancelación
  elimina todas las respuestas parciales; sin cambios de estado aceptado.
- Código de terceros copiado: ninguno. serde/serde_json/sha2 del registro local
  existentes; dependencia propia engine-loads añadida a engine-api. Sin red.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Pruebas: oráculos analíticos anteriores; SHA y bytes deterministas; todas las
  fronteras de cancelación, incluidos dos modos/direcciones; entradas inválidas,
  succión, masas desde cargas de miembro, aviso con gravedad oblicua y aviso
  2×2 incluso en fallo. La prueba de aviso #78 no prueba ni cierra copysign.

Verificación final del incremento 2c, con `CARGO_TARGET_DIR=/workspace/.cargo-target`,
`CARGO_NET_OFFLINE=true` y Rust 1.99.0 (`PATH=/home/box/.cargo/bin:$PATH`):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 474 pruebas, 0 fallos/ignoradas;
  engine-api: 53 (12 dynamic, 15 facade, 8 limits, 18 run).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- JSON Schema: documentos parseados y todas las referencias locales comprobadas,
  sin red ni dependencias nuevas de validación de esquemas.
- `git diff --check`: OK; sin panic/unwrap/expect en la biblioteca engine-api.
- `bash clean-room/check.sh origin/main`: OK sobre el diff preparado para commit,
  incluidos archivos nuevos. Sin cambios a docs/CLEAN_ROOM.md, .github/** ni
  clean-room/**. Revisión humana clean-room PENDIENTE (J4).
