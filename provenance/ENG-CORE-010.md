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
