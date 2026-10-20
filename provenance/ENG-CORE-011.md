# Bitácora clean-room · ENG-CORE-011

- Ticket / PR: MAM-25 · ENG-CORE-011, guía de métodos Rust y piloto cúbico.
- Autor(es): Codex gpt-6.1-sol high, **2026-10-20** (America/Guayaquil).
- Rol: especificador e implementador; sin operador de oráculo ni auditoría de similitud.
- Contrato implementado: `uniaxial/1`, kind `elastic.cubic`, método 0.1.0.
- Fuentes permitidas usadas: docs/CLEAN_ROOM.md §§3–8 y 10; provenance/TEMPLATE.md;
  engine-core (tipos, catálogo, math); engine-contracts (traits, tipos y
  conformance, ENG-CORE-007); engine-laws (leyes, fábricas y pruebas locales);
  engine-validation (README, schema y fixture axial-bar, ENG-CORE-009);
  provenance/ENG-CORE-007.md, provenance/ENG-CORE-009.md y docs/DETERMINISMO.md.
  Se leyó clean-room/check.sh como herramienta local de comprobación, sin cambios.
- Derivaciones propias: Codex gpt-6.1-sol high. *Elasticidad cúbica conservativa*,
  2026-10-20, [guía §5](../docs/GUIA_METODOS.md#5-piloto-reproducible-elasticidad-cúbica),
  ecuaciones (1)–(3) y tabla analítica propia. Sin referencias bibliográficas externas.
- Código de terceros copiado: ninguno. Sin dependencias nuevas ni cambios de manifests/lock.
- Validación: analítica propia, kit `check_uniaxial_law`, fixture tipado nativo
  con source/cita y diez filas de tracción, descarga y compresión.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
  Tampoco consulté código, resultados o informes de CalculiX, code_aster,
  LAPACK o derivados; no se abrieron repos hermanos ni fuentes externas.
  Trabajo sin red, Cargo offline y PIP_NO_INDEX=1.
- Revisor clean-room: **PENDIENTE (J4)**.

## Decisiones y ejecución del piloto

Se inspeccionaron primero las leyes existentes: elasticidad lineal,
multilineal (incluye curvas bilineales) y plasticidad bilineal cinemática
(incluye b = 0). Se eligió una curva suave cúbica que ninguna representa
exactamente con C > 0. El límite lineal C = 0 sirve como comprobación.

La guía y su ficha con esperados se escribieron antes del código del piloto.
Después se siguieron sus pasos: módulo en engine-laws, parámetros privados
validados, trait puro sin historia, descriptor, fábrica, exportaciones, pruebas
de conformidad y fixture analítico nativo. El piloto ejercita la guía en la misma
sesión autora; no constituye una prueba independiente con otro implementador.

Derivación: W = Eε²/2 + Cε⁴/4, σ = dW/dε y K = d²W/dε²; E > 0, C ≥ 0
en Pa. Se evalúan q = (Cε)ε, σ = (E+q)ε y K = E+3q, exigiendo finitud
en cada operación. El fixture usa E = 8 Pa, C = 16 Pa y fracciones binarias
con esperados evaluados directamente en la ficha. Límite de comparación
max(1e-12 Pa, 1e-12·|esperado|); FD h = 1e-6, error central C h².
Sin tablas, figuras ni salidas copiadas; sin calibración con otros programas.

El registro replica el mecanismo actual de las otras leyes: `LawDescriptor`,
`UniaxialFactory` y `pub use`, sin registro global ni cambios de engine-api.
El fixture es tipado y nativo porque `fixture/1` de ENG-CORE-009 requiere
engine-input/1 → results/1 y la API no expone estas leyes no lineales. La
plantilla JSON se reserva para métodos accesibles por esa frontera; no se
alteran los cinco JSON de ENG-CORE-009 ni su inventario cerrado.

Sin cambios en clean-room/**, docs/CLEAN_ROOM.md, CODEOWNERS,
.github/workflows, vendor/ o docs/DETERMINISMO.md. No se añadieron unsafe,
unwrap/expect/panic de biblioteca, E/S, hilos ni estado global.

## Evidencia local

Ejecución offline el **2026-10-20**, Rust 1.99.0
(`b940084d7 2026-09-28`), Node v22.19.0, flags Cargo habituales y
`MAMUT_ENGINE_GIT_COMMIT` sin definir para reproducir los hashes de referencia.
Todos los comandos terminaron con salida 0:

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK, sin warnings. |
| `cargo test --workspace --offline` | **665 passed / 0 failed / 0 ignored**, incluyendo doctests. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK. |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK; compilación, sin ejecución del piloto en wasm. |
| `bash clean-room/check.sh origin/main` | OK, incluidos los archivos nuevos. |

El piloto tiene **6 tests nativos passed / 0 failed** y un doctest nuevo
incluido en el total del workspace. Las pruebas cubren el fixture propio de
diez filas, conformidad para C = 0/16/1600 Pa, descriptor/fábrica/Send + Sync,
propiedad del estado, simetría impar de σ y par de K, límite lineal, escalas
extremas representables y rechazo de parámetros/entradas/historias inválidos.
Se ejercitan los seis caminos de desbordamiento de productos y sumas.

Cruce adicional con los ejemplos `determinism_native` y `determinism_wasm`
y `scripts/check-determinism.mjs`, en **debug y release**, con diez corridas
por caso/target/perfil: **16 hashes nativo = wasm**, todos idénticos a la
tabla vigente de docs/DETERMINISMO.md. Se compararon además las 160 filas
nativas de cada perfil con esa tabla y las salidas debug/release entre sí:
idénticas byte a byte. No se actualizó ningún hash de referencia.

El control clean-room empleó `GIT_INDEX_FILE` y `GIT_OBJECT_DIRECTORY`
temporales bajo `$CARGO_TARGET_DIR`, con objetos originales solo como alternos:
el `.git` del entorno es de solo lectura. Se incluyeron las rutas nuevas con
ese índice, sin modificar el índice real ni crear commits.

Logs retenidos bajo `$CARGO_TARGET_DIR/eng-core-011/`:
`fmt.log`, `clippy.log`, `tests.log`, `doc.log`, `wasm-check.log`,
`clean-room.log`, `wasm-build-{debug,release}.log`,
`native-{debug,release}.{log,txt}` y `determinism-{debug,release}.log`.
Se comprobaron también enlaces locales de los documentos nuevos, ausencia
de rutas locales absolutas en la documentación del entregable, diff sin
errores de espacios y ausencia de cambios en las rutas protegidas.

## Pendientes externos

Revisión clean-room **PENDIENTE (J4)**. La ejecución wasm del piloto no forma
parte de la validación nativa exigida; compilar wasm no la acredita.
No quedan comprobaciones técnicas obligatorias pendientes. Cambios sin commit.
