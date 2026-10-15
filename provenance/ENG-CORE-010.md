# Bitácora clean-room · ENG-CORE-010

- Ticket / PR: MAM-23 · ENG-CORE-010 «Política de determinismo»; rama j2/eng-core-010 apilada sobre j2/eng-api-001 (#88), base b270626.
- Autor(es): Codex gpt-6.1-sol high bajo J2, 2026-10-14 America/Guayaquil.
- Rol: implementador.
- Contrato implementado: política de ejecución V1, run(engine-input/1) → results/1 canónico; guardia de biblioteca, repetibilidad y cruce nativo/wasm.
- Fuentes permitidas usadas: instrucciones del ticket recibidas de J2; fuentes y documentación propias de main eed45dd y #88 b270626; tests analíticos propios de engine-api; API std/Rust, Cargo offline y WebAssembly de Node instalados. Sin red ni consultas a motores externos.
- Derivaciones propias: voladizo EI y''=−P(L−x), y(0)=y'(0)=0 → y(L)=−PL³/(3EI); equilibrio R=P, M=PL. Oscilador EA/L=k=100 y m=5 → ω²=k/m=20. Columna Pcr=π²EI/(4L²), P=0,2Pcr fijado como decimal de entrada. Viga modal con GDL Uy/Rz y masas 5/2: rigidez de flexión propia del elemento; CQC y log-log usan APIs del repositorio. Las pruebas nuevas comparan resultados repetidos, sin un oráculo externo ni nuevos algoritmos mecánicos.
- Código de terceros copiado o vendorizado: ninguno. La primera entrega conservó std por ausencia de libm offline; esa decisión queda sustituida por la aprobación y dependencia registradas abajo. Se conservan serde/serde_json/sha2.
- Validación: guardia léxica sobre crates/*/src, con casos de aliases/macros/comentarios/cadenas; seis entradas válidas por diez corridas con hash de results/1; ejemplos nativo y cdylib C-ABI wasm ejecutados con Node sin wasm-bindgen ni red, en debug y release.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

La política escalar vigente y el inventario migrado figuran en docs/DETERMINISMO.md.
No se cambia el solver, sus tolerancias ni el FMA explícito de Sturm. Sin cambios
a docs/CLEAN_ROOM.md, .github/** ni clean-room/**. Igualdad nativo/wasm limitada
a la evidencia medida; no se certifica toda entrada/arquitectura.

## libm aprobada por Jonathan 2026-10-14

- Autorización: Jonathan aprueba libm para determinismo nativo/wasm; instrucción
  de J2 recibida el 2026-10-14. PR #98 / MAM-23 / ENG-CORE-010, rama
  j2/eng-core-010 integrada con origin/main d0031f8 (HEAD inicial 7fcc5c3).
- Implementación: engine_core::math centraliza las trascendentes con libm
  0.2.16; sqrt conserva std por el redondeo correcto IEEE-754. powi convierte
  exactamente i32 a f64 y usa libm::pow. Se migran bibliotecas, ejemplos y
  pruebas de todos los crates, sin modificar algoritmos mecánicos, validaciones,
  tolerancias, reducciones ni el FMA explícito de Sturm.
- Dependencia/licencia: metadata del Cargo.toml del paquete instalado en el
  registro local, version 0.2.16, license MIT. Cargo.toml del workspace añade
  version exacta =0.2.16 y default-features=false (sin kernels arch); el manifest
  engine-core hereda libm.workspace=true. Cargo.lock añade libm, la arista desde
  engine-core y checksum
  b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981.
  Ninguna descarga, dependencia adicional, código copiado ni fuente de motores
  externos; se usa la API pública de la dependencia permitida, sin consultar
  algoritmos de OpenSees, Stabileo ni derivados.
- Guardia: test léxico sobre todos los Rust de crates/**, incluidos tests y
  ejemplos; solo exceptúa engine-core/src/math.rs. Detecta métodos std,
  UFCS/referencias a funciones, aliases de f64 y accesos directos a libm;
  incluye pruebas de macros/espacios/comentarios/cadenas/chars. Conserva la
  guardia previa de colecciones hash y concurrencia en bibliotecas.
- Suite propia: nueve entradas results/1 (seis originales y variantes
  inclinadas modal, P-Delta y espectro) y cuatro hashes de bits f64 (trig,
  exp/log, potencias/normas y rotación de cargas mediante Beam2D). Las variantes
  usan coordenadas 0,8x/0,6x y fuerzas globales rotadas en P-Delta; no incorporan
  nuevos algoritmos mecánicos ni datos externos. Winkler no existe en main
  d0031f8 ni en esta rama; no se afirma cobertura de esa ruta.
- ABI/script: identidad y orden de casos por SHA-256 del nombre, estados y
  límites de índices, SHA-256 completos, cero imports y trece casos obligatorios.
  Los bits escalares se hashean como to_bits().to_be_bytes(), sin JSON ni
  tolerancias; diez corridas por caso/target, sin estado mutable global.

Verificaciones offline con CARGO_TARGET_DIR=/workspace/.cargo-target/me45,
Rust 1.99.0 (LLVM 23.1.1), sin RUSTFLAGS especiales:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK; 507 pruebas incluidos doctests,
  0 fallos/ignoradas. Se volvió a verificar la suite compartida tras añadir
  el caso explícito sqrt(-0.0): 2 tests OK.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- Build/ejecución offline con Node v22.19.0: OK en debug y release,
  trece hashes iguales entre targets, diez corridas por caso/target.
  `cmp` de las salidas nativas debug/release: OK, también iguales entre perfiles.
  Comandos, hashes y límites en docs/DETERMINISMO.md.
- Control negativo del script: un hash nativo alterado (igual en sus diez
  corridas) es rechazado por igualdad bit a bit en la primera comparación wasm.
- `git diff --cached --check`: OK.
- `bash clean-room/check.sh origin/main`: OK sobre cambios preparados,
  incluidos archivos nuevos. Revisión humana PENDIENTE (J4).
- Sin cambios a docs/CLEAN_ROOM.md, .github/** ni clean-room/**; sin red ni push.

## Ampliación J4 r25 · 2026-10-14

- Solicitud: MAM-23 / PR #98, sobre fccda4d; caso explícito sin_cos y
  cruce frame3d inclinado con rotación de ejes locales, linearStatic y modal.
- Fuentes: instrucciones del usuario, código y fixtures propios del checkout,
  toolchain Rust y Node instalados. Sin red ni consulta a fuentes de otros
  motores. Ningún código de terceros copiado o vendorizado; sin dependencias
  nuevas ni cambios en biblioteca.
- sin_cos: el envoltorio engine_core::math::sin_cos sobre libm::sincos ya
  existía y ya se usaba para calcular ambos valores del mismo ángulo en
  engine-loads::Beam2D y engine-contracts::conformance. Se conserva y se añade
  scalar-sin-cos: hash directo de los bits del par (seno, coseno) para dieciséis
  ángulos, incluidos ceros firmados, subnormales, vecinos de π/2 y ±1e300.
- Frame3d: dos entradas del mismo pórtico espacial de tres nodos y dos miembros
  inclinados en XYZ; orientaciones locales distintas y no alineadas con los
  ejes globales, Iy distinto de Iz, apoyo empotrado y dos nodos libres con masas
  traslacionales y rotacionales. Una entrada linearStatic con seis componentes
  de carga y otra modal con seis modos y participación X/Y/Z; la API solo
  admite un análisis por entrada. No se añaden algoritmos mecánicos.
- Test nativo: diez corridas de las once entradas results/1 y de los cinco
  casos de bits; exige los tres casos nuevos. Comprueba doce esfuerzos locales
  no nulos por miembro, seis modos positivos y tres participaciones por modo;
  cambiar las orientaciones debe alterar los resultados mecánicos, no solo
  input_sha256. Los ejemplos nativo/wasm incluyen los nuevos casos a través
  del soporte compartido; el script Node los exige como cobertura obligatoria.
- Cruce real offline: Rust 1.99.0 (LLVM 23.1.1), Node v22.19.0,
  x86_64-unknown-linux-gnu / wasm32-unknown-unknown, sin RUSTFLAGS especiales,
  conservando CARGO_TARGET_DIR=/workspace/.cargo-target/me45. Debug y release:
  16 hashes completos idénticos, diez corridas por caso/target; cmp de las
  160 filas nativas debug/release también OK. Los trece hashes de fccda4d
  permanecen iguales. Tabla completa y reproducción en docs/DETERMINISMO.md.
- SHA-256 nuevos (iguales en ambos targets y perfiles):
  - scalar-sin-cos: b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a.
  - frame3d-inclined-static: 152dc9c3a40210634053378061c25668b93bf2eb018223e90b8b18712d9e754f.
  - frame3d-inclined-modal: 58e6c5f1733a576a969d323164bf7dbee19155974688a33c73036b2462d4f060.
- Verificación con el CARGO_TARGET_DIR exportado y CARGO_NET_OFFLINE=true:
  cargo fmt --all -- --check; cargo clippy --offline --workspace --all-targets
  -- -D warnings; cargo test --offline --workspace (508 pruebas incluidos
  doctests, cero fallos/ignoradas); RUSTDOCFLAGS="-D warnings" cargo doc --offline
  --no-deps --workspace; cargo check --offline --workspace --target
  wasm32-unknown-unknown: todo OK.
- git diff --cached --check y bash clean-room/check.sh origin/main: OK sobre
  cambios preparados, incluidos los dos fixtures nuevos. Revisión humana
  PENDIENTE (J4). Sin cambios a docs/CLEAN_ROOM.md, .github/** ni clean-room/**;
  sin red ni push. No se añaden panic/unwrap/expect a biblioteca.


## FU #105 · MAM-23 · J4 r26 · 2026-10-14

- Solicitud: cerrar los cinco huecos de la guardia escalar de #98; rama
  j2/eng-core-010-fu sobre main abe7453, sin red ni push.
- Fuentes: instrucciones del usuario, guardia y fuentes propias del checkout,
  APIs std/Rust y toolchain local. Ningún código de terceros copiado o
  vendorizado, ninguna dependencia nueva ni consulta a otros motores.
- Guardia: distingue r#identificador de cadenas raw y normaliza el nombre;
  exige la ruta engine_core::math:: o el import exacto sin alias del módulo
  para la abreviatura math::. Rechaza aliases de primitivos, as math y
  redefiniciones/imports alternativos de math, además de rutas con otro prefijo.
  Los argumentos escalares sin ruta dentro de macros son rechazados;
  receptores de datos como log.acceleration(...) siguen permitidos.
- FMA: excepción limitada al archivo engine-linalg/src/modes.rs, sin exceptuar
  tests u otros archivos llamados modes.rs; ni math.rs permite mul_add.
- include!: solo rutas literales .rs con destino dentro del inventario Rust
  inspeccionado de crates/**, comprobado tras canonicalizar rutas. Rechaza
  ficheros de otras extensiones y rutas dinámicas; include_str!/include_bytes!
  de datos permanecen permitidos. La excepción escalar de math.rs no exceptúa
  la comprobación de includes.
- Fixtures propios: cinco pares de textos .txt, positivos y negativos,
  consumidos como datos por cinco tests específicos. Pruebas adicionales para
  todas las funciones de la lista con identificadores crudos y tres delimitadores
  de macros, rutas completas, aliases equivalentes, comentarios y cadenas raw.
- Integración: main contiene usos std de sqrt/sin_cos en los tests de Winkler
  de engine-validation que la guardia previa también rechazaba. Se centralizan
  en engine_core::math en winkler_elements.rs y support/winkler_harness.rs;
  sqrt conserva su excepción IEEE y sin_cos usa el proveedor libm ya fijado.
  Sin cambios en biblioteca, algoritmos mecánicos, tolerancias ni FMA de Sturm;
  no se añaden panic/unwrap/expect en biblioteca.
- Documentación: se explicita el alcance de la guardia y se corrige la atribución
  a «ampliación tras J4 r25, reproducida por J4 r26» en docs/DETERMINISMO.md.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket."
- Verificación offline con el CARGO_TARGET_DIR exportado
  (/workspace/.cargo-target/me45), CARGO_NET_OFFLINE=true:
  - cargo fmt --all -- --check: OK.
  - cargo clippy --offline --workspace --all-targets -- -D warnings: OK.
  - cargo test --offline --workspace: OK, 520 pruebas incluidos doctests,
    cero fallos/ignoradas; los nueve tests de la guardia pasan.
  - RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace: OK.
  - cargo check --offline --workspace --target wasm32-unknown-unknown: OK.
  - git diff --cached --check y bash clean-room/check.sh origin/main: OK,
    sobre cambios preparados, incluidos los diez fixtures nuevos.
- Revisión humana clean-room: PENDIENTE (J4). Sin cambios a docs/CLEAN_ROOM.md,
  .github/** ni clean-room/**.


## FU2 #107 · MAM-23 · J4 r27 · 2026-10-14

- Solicitud: cerrar las vías de código no inspeccionado y FFI; rama
  j2/eng-core-010-fu2 apilada sobre j2/eng-core-010-fu, PR #106,
  commit base 101c4d1; origin/main local abe7453. Sin red ni push.
- Fuentes: instrucciones del usuario, fuentes y documentación propias del
  checkout, APIs std/Rust y toolchain instalado. Ningún código de terceros
  copiado o vendorizado, ninguna dependencia nueva ni consulta a otros motores.
- Rutas: #[path = "..."] comparte la validación de include!: literal .rs sin
  escapes y destino canonicalizado dentro del inventario Rust de crates/**.
  Se inspeccionan espacios, identificadores crudos, cadenas raw y atributos
  cfg_attr con varios atributos o cfg_attr anidados, independientemente de su
  condición. Los predicados cfg(path = "...") no son atributos de módulo.
  Se rechazan destinos inexistentes y archivos reales fuera del inventario;
  math.rs y examples/ no están exceptuados de esta regla.
- FFI: se rechazan #[link_name = ...], también en cfg_attr, y los bloques
  extern "..." { ... } / extern { ... }, incluidos ABI raw y macros, en
  crates/** fuera de crates/*/examples/. No se confunden extern crate,
  funciones extern ni tipos de puntero a función con bloques de importación;
  extern crate conserva las prohibiciones anteriores de la guardia.
- Biblioteca: se añade #![forbid(unsafe_code)] a los quince crates/*/src/lib.rs
  y también a bindings/python/src/lib.rs: PyO3 0.25.1 compila con la prohibición,
  por lo que no se necesita exceptuar el binding. La guardia exige el atributo
  interno, incondicional y en la raíz de cada biblioteca, y comprueba el binding
  por separado. Lista explícita UNSAFE_LIBRARY_EXCEPTIONS vacía, sin excepciones
  de bibliotecas ni globs. Los atributos en comentarios, cadenas, submódulos,
  macros o cfg_attr, y deny(unsafe_code), no satisfacen la comprobación.
- Unsafe existente: engine-linalg/tests/heap_peak.rs implementa GlobalAlloc
  y delega en System mediante unsafe para medir memoria; es un test de
  integración, fuera de la biblioteca. Se conserva sin imponerle forbid y
  no justifica una excepción para engine-linalg. No se toca vendor/**.
- Pruebas: cinco negativos específicos para path no .rs, path fuera del
  inventario (incluye un .rs real del binding), link_name, bloque extern y
  lib.rs sin forbid. Positivos para rutas inspeccionadas, raw strings,
  atributos anidados, comentarios, strings, predicados cfg y examples/;
  la excepción examples/ no relaja rutas ni las prohibiciones anteriores.
  Se retiró temporalmente cada una de las cinco comprobaciones: su test
  específico falló por aserción en los cinco casos. Restaurado el código,
  los dieciséis tests de la guardia pasan.
- Documentación: docs/DETERMINISMO.md recoge el nuevo alcance, la lista sin
  excepciones y la distinción entre bibliotecas y el test GlobalAlloc.
  No se cambian algoritmos mecánicos, tolerancias ni FMA de Sturm; no se
  añaden panic/unwrap/expect en biblioteca.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket."
- Verificación offline con CARGO_TARGET_DIR=/workspace/.cargo-target/me45
  ya exportado, CARGO_NET_OFFLINE=true y Rust/Cargo 1.99.0:
  - cargo fmt --all -- --check: OK.
  - cargo clippy --offline --workspace --all-targets -- -D warnings: OK.
  - cargo test --offline --workspace --no-fail-fast: OK, 527 pruebas incluidos
    doctests, cero fallos/ignoradas; dieciséis tests de la guardia.
  - RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace: OK.
  - cargo check --offline --workspace --target wasm32-unknown-unknown: OK.
  - git diff --cached --check y bash clean-room/check.sh origin/main: OK,
    sobre todos los cambios preparados.
- Revisión humana clean-room: PENDIENTE (J4). Sin cambios a docs/CLEAN_ROOM.md,
  .github/**, clean-room/** ni vendor/**.

## FU3 · MAM-23 · J4 r29/r30 · 2026-10-15

- Solicitud: cerrar atributos/extern generados por macro_rules, export_name,
  ensamblador y la excepción implícita de tests/examples; rama
  j2/eng-core-010-fu3 basada en main 800ef3f, que incluye #106 y #108.
- Fuentes: instrucciones del usuario, guardia, fuentes y documentación propias
  del checkout, APIs std/Rust y toolchain local. Trabajo offline, sin consultar
  fuentes de otros motores, copiar código de terceros ni añadir dependencias.
- Guardia: rechaza atributos con metavariables, incluidas repeticiones y
  cfg_attr, y extern con ABI/cuerpo/tokens variables dentro de macro_rules.
  Rechaza export_name y asm!/global_asm!/naked_asm! en crates/**, incluidas
  rutas core::arch/std::arch, identificadores crudos y delimitadores de macro.
  Los atributos cfg_attr anidados y los envoltorios unsafe(...) se inspeccionan
  sin depender de condiciones de compilación; se mantiene path/link_name,
  inventario de includes y forbid(unsafe_code) en bibliotecas.
- Tests/examples: SOURCE_POLICY_EXCEPTIONS sustituye la convención de unsafe
  y la excepción FFI general de examples/ por tres rutas exactas, justificadas
  y con permisos mínimos: heap_peak.rs solo unsafe (GlobalAlloc delegado en
  System); determinism_wasm.rs y winkler_wasm.rs solo no_mangle y funciones
  públicas extern "C". Los ejemplos actuales no usan unsafe y no lo permiten.
  No se permiten bloques extern, link_name, export_name ni ensamblador en
  ninguna excepción. No se heredan permisos por nombre o subdirectorio.
  La guardia falla si desaparece un archivo exceptuado o cualquiera de sus
  construcciones autorizadas, aunque siga usando las demás.
- Pruebas: veinte tests nuevos, treinta y seis en la guardia. Negativos por
  construcción en src/tests/examples y dentro de macros con tres delimitadores;
  permisos extra, rutas fuera de lista, permisos obsoletos y archivo desaparecido.
  Positivos para las excepciones mínimas, comentarios anidados, strings
  normales/raw, caracteres, atributos inocuos y macros con atributos fijos.
  Mutación temporal: 61 ejecuciones negativas confirmadas por fallo de aserción
  del test seleccionado, sin contar errores de compilación ni tests vacíos.
  Se retiró cada detector; también se omitieron cuerpos de macros, se restituyó
  por separado una excepción general de tests/ y examples/, y se retiraron las
  comprobaciones de permisos mínimos/obsoletos e inventario. Código restaurado.
- Documentación: docs/DETERMINISMO.md distingue la convención anterior de la
  nueva política codificada y detalla la lista exacta y su control de vigencia.
  Sin cambios en bibliotecas, algoritmos mecánicos, tolerancias ni FMA de Sturm;
  no se añaden panic/unwrap/expect en biblioteca.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket."
- Verificación offline con CARGO_TARGET_DIR=/workspace/.cargo-target/me45
  conservado del entorno y CARGO_NET_OFFLINE=true:
  - Guardia inicial y 61 ejecuciones por mutación: Rust/Cargo 1.85.1 del PATH
    inicial; todos los negativos fallaron en su aserción seleccionada.
  - Verificación completa con Rust/Cargo 1.99.0 ya instalado localmente,
    incluido su target wasm32; sin descargar ni instalar componentes:
    cargo fmt --all -- --check: OK;
    cargo clippy --offline --workspace --all-targets -- -D warnings: OK;
    cargo test --offline --workspace --no-fail-fast: OK, 565 pruebas incluidos
    doctests, cero fallos/ignoradas, treinta y seis tests de la guardia;
    RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace: OK;
    cargo check --offline --workspace --target wasm32-unknown-unknown: OK.
  - git diff --cached --check y bash clean-room/check.sh origin/main: OK
    sobre los tres archivos preparados para el commit.
- Revisión humana clean-room: PENDIENTE (J4). Sin cambios a docs/CLEAN_ROOM.md,
  .github/**, clean-room/** ni vendor/**. Sin red ni push.
