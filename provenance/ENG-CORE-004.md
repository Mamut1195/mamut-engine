# Bitácora clean-room · ENG-CORE-004

- Ticket: MAM-17 · ENG-CORE-004, «Esquema de entrada mamut.engine.model/1 y job/1».
- Rama: j2/eng-core-004, apilada sobre j2/eng-api-001 (PR #88, sin fusionar).
- Autor: Codex gpt-6.1-sol high bajo J2, 2026-10-14 America/Guayaquil.
- Rol: implementador. Revisor clean-room: PENDIENTE (J4).
- Contrato: conserva engine-input/1, serde y schemas publicados en #88;
  añade fixtures, pruebas de carpetas/nombres y documentación model + job.
  Nombre definitivo PENDIENTE de Jonathan, Notion «02 · Contratos».
- Fuentes permitidas usadas: requisitos de J2 recibidos en esta conversación;
  schemas, API, README y pruebas propias de crates/engine-api; catálogo propio
  de engine-core; clean-room/README.md, check.sh y forbidden_identifiers.txt
  (este último incorporado mediante include_str! relativo en la prueba).
- Derivaciones propias: ejemplos SI declarados a mano sobre las ecuaciones
  analíticas ya comprobadas en #88. Voladizo: equilibrio R=P, M=PL e integración
  de EI y''=−P(L−x) → y(L)=−PL³/(3EI). Viga apoyada uniforme:
  R=wL/2, Mmax=wL²/8; combinación explícita D+0,5L por superposición,
  sin coeficientes normativos. Barra osciladora: EA/L=100 N/m y m=5 kg,
  ω²=k/m=20 s⁻². Espectro constante Sa=3 m/s²: u=Sa/ω²=0,15 m,
  V=m Sa=15 N. P-Delta: compresión axial explícita de 10 N y carga lateral
  de 1 N en un voladizo de L=4 m, EI=600 N·m²; la ecuación propia
  EI y''''+P y''=0 tiene primer polo Pcr=π²EI/(4L²)>10 N. Las pruebas
  analíticas existentes siguen siendo los oráculos; no se duplican solvers.
- Rechazos: esquema desconocido, versión mayor desconocida, 1e400 (sustituye
  NaN, que no es representable en JSON), referencia ausente, unidades/ejes,
  entero fuera de u64, id fuera de u32 y campo desconocido. Cada .expect fija
  código real de #88, salida y resultado de validate; sin cambiar biblioteca.
- Nombres: se inspeccionan schemas y fixtures, incluso .expect, campos públicos
  opcionales y claves JSON escapadas. La lista adicional de claves complementa
  los identificadores prohibidos del repositorio. `member.element` se conserva
  como selector legítimo de la fábrica propia, sin estructura de comandos ni
  argumentos posicionales; excepción documentada en README y en la prueba.
  `pattern` con valor string en schemas es la palabra clave estándar para regex
  (SHA-256/códigos de results/1); se prohíbe en fixtures y campos públicos.
- Se conserva #85: la combinación se ejecuta con aviso y salida 1 debido a los
  prescritos cero de los apoyos; el ticket no modifica ni cierra ese seguimiento.
- Código de terceros copiado: ninguno. Sin consultas de red, sin dependencias
  nuevas, sin cambios a docs/CLEAN_ROOM.md, .github/** ni clean-room/**.
- Comparación con oráculo externo o repo hermano: no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

Verificaciones sobre el árbol final, con Rust 1.99.0,
`CARGO_TARGET_DIR=/workspace/.cargo-target`, `CARGO_NET_OFFLINE=true`
y `PATH=/home/box/.cargo/bin:$PATH`:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 479 pruebas, 0 fallos/ignoradas;
  incluye cinco pruebas nuevas (dos de fixtures y tres de nombres).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main`: OK, incluidos archivos nuevos.
- `git diff --check`: OK. Solo fixtures, pruebas, README y esta bitácora;
  sin cambios en biblioteca, schemas, dependencias ni archivos protegidos.
- Revisión humana clean-room: PENDIENTE (J4).

## Correcciones J4 r23

- Fecha: 2026-10-14 (America/Guayaquil).
- Autor: Codex gpt-6.1-sol high bajo J2. Rol: implementador.
- Revisor: PENDIENTE (J4).
- Ticket / PR: MAM-17 · ENG-CORE-004 / #95.
- Fuentes permitidas: petición de J2 y revisión local
  `/workspace/j4rev/comments/engine-95-r23.md`; lista propia
  `clean-room/forbidden_identifiers.txt`; schemas, fixtures y pruebas existentes
  de engine-api. Sin consultas de red ni código de otros motores.
- Base de la corrección: rama j2/eng-core-004 ya integrada con `origin/main`
  en 9a27e9f, incluido #88; se conserva el contrato publicado.
- Guardia: comparación ASCII sin distinguir mayúsculas y eliminando `_`/`-`
  para la lista clean-room y COMMAND_KEYS. Recorrido estructural con rutas,
  strings decodificados y claves duplicadas, sin convertir números fuera de
  rango de los fixtures negativos. No cambia el parser de biblioteca.
- Datos: regex ASCII `^[a-z][a-z0-9_]*$` en todas las claves de fixtures y
  nombres bajo `properties`/`$defs`. Palabras clave de JSON Schema separadas
  mediante lista blanca explícita; `pattern` solo como keyword de valor string.
  Los objetos de `const`, `enum`, `default` y `examples` se inspeccionan como
  datos, sin heredar permisos de palabras clave.
- Excepciones contractuales explícitas, PENDIENTES de decisión de Jonathan/J4:
  se pide decidir si se conservan o se planifica una migración posterior. No
  se renombra ningún contrato, propiedad ni destino `$ref` publicado en #88.
  Todos los nombres de propiedades publicados cumplen la regex; estos 15
  nombres internos de `$defs` usan camelCase y quedan exceptuados solo en la
  raíz y en el contrato indicado:
  - engine-input/1: `/$defs/uniformLoad`, `/$defs/dofValue`,
    `/$defs/massSource`, `/$defs/pDeltaOptions`, `/$defs/modalOptions`,
    `/$defs/spectrumTable`, `/$defs/spectrumDirection`, `/$defs/spectrumOptions`.
  - results/1: `/$defs/nodeIdentity`, `/$defs/memberIdentity`,
    `/$defs/fieldExtremes`, `/$defs/spectrumResponse`, `/$defs/caseFactor`,
    `/$defs/pDeltaIteration`, `/$defs/pDeltaConvergence`.
- `element` se incorpora a COMMAND_KEYS. Solo se admite en
  `model.members[].element` de fixtures y `/$defs/member/properties/element`
  del schema engine-input/1: selector declarativo de la fábrica propia.
  Se detectó además el campo REAL ya publicado
  `/$defs/memberIdentity/properties/element` en results/1, identidad del kind
  usado por cada miembro. Se conserva con una excepción contractual exacta,
  también PENDIENTE de decisión, para respetar el contrato de #88. No se
  admite `element` en otras rutas, ni variantes normalizadas de su nombre.
- Pruebas: un negativo independiente para cada caso pedido por r23, además
  de guion, camelCase nuevo, variantes de toda la lista, escapes Unicode,
  duplicados, palabras clave desconocidas y excepciones fuera de ruta/contrato.
- Oráculos: solo los analíticos propios ya derivados a mano; esta corrección
  no añade solvers, dependencias, fuentes externas ni cambios a biblioteca.
- Código de terceros copiado: ninguno. Comparación con oráculo externo o repo
  hermano: no. Sin cambios a docs/CLEAN_ROOM.md, .github/** ni clean-room/**.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

Verificaciones de las correcciones con Rust 1.99.0, sin red,
`CARGO_TARGET_DIR=/workspace/.cargo-target` y `CARGO_NET_OFFLINE=true`:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 501 pruebas, 0 fallos/ignoradas;
  incluye 25 pruebas de nombres (22 añadidas respecto a r23).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main`: OK.
- `git diff --check`: OK. Cambios limitados a tests, README y provenance.
- Revisión humana clean-room: PENDIENTE (J4).
