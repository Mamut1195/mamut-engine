# Política de determinismo · V1 · ENG-CORE-010 (MAM-23)

Misma entrada, versión del motor, backend, opciones y entorno de compilación
deben producir los mismos bytes canónicos de `results/1`, incluido su SHA-256
externo. Esto incluye `git_commit` y `target`: al cambiar de target cambia
la procedencia y puede cambiar el hash del documento completo. El campo
`input_sha256` identifica la entrada canónica: no es el hash del resultado. No hay relojes, rutas ni estado aleatorio en `run`.

## Auditoría inicial de main y #88

Auditoría local sin red, 2026-10-14: `origin/main` = `eed45dd`;
`origin/j2/eng-api-001` / base de esta rama = `b270626`, PR #88.
No se consultó el estado remoto del PR. La base incorpora main.

| Aspecto | main | Base #88 |
| --- | --- | --- |
| Colecciones numéricas | `BTreeMap/BTreeSet`, `Vec`; ninguna colección hash | Igual; engine-api también usa BTreeMap y ordena claves JSON explícitamente |
| Hilos | Ningún `std::thread` ni rayon en bibliotecas | Igual; cancelación cooperativa, solves indivisibles |
| Backend ENG-LA-006 | `ReferenceBackend` operativo; feature faer vacía, adaptador Unsupported | Igual; `run` selecciona ReferenceBackend |
| Reducciones | Bucles/iteradores secuenciales; CSR suma duplicados en orden de entrada | Igual |
| SIMD/FMA | Sin SIMD explícito ni despacho numérico por CPU; un `mul_add` explícito en Sturm | Igual |
| Semillas | Ningún RNG ni semilla en bibliotecas | Igual |
| Escalares | `sqrt`, `hypot`, `sin_cos`, `ln`, `ln_1p`, `exp` de std | Igual; sin llamadas directas a sin/cos/powf en bibliotecas |
| Fachada canónica | engine-api aún no está en main | `run`, results/1, input_sha256 y pruebas de dos corridas |

Los hilos del test `engine-linalg/tests/heap_peak.rs` son instrumentación del
test, fuera de `src`; no son ejecución numérica del backend.

## Configuración V1 y orden de operaciones

- Un hilo de ejecución por análisis. Prohibidos hilos internos, pools, rayon,
  atomics usados para acumular resultados y reducciones paralelas. `Arc` y el
  token de cancelación no alteran el orden numérico; para repetibilidad se usa
  un token sin cancelar y callbacks sin efectos sobre el modelo.
- `ReferenceBackend` sin features opcionales es la configuración V1. `faer`
  sigue siendo un stub, no un proveedor certificado. Cualquier backend nuevo
  debe declarar orden de reducción, desempates, SIMD/FMA y pasar esta suite.
- Prohibidos `HashMap/HashSet` en caminos numéricos: índices y recorridos
  ordenados con `BTreeMap/BTreeSet` o vectores. No se reordenan arrays de input:
  permutar cargas o miembros puede cambiar el redondeo y no es «misma entrada».
- No hay aleatoriedad hoy. Si se incorpora un algoritmo con RNG, su algoritmo,
  versión y semilla deben fijarse explícitamente (semilla V1 `0_u64`), sin
  entropía del sistema, tiempo ni semillas implícitas; requiere ampliar pruebas.
- binary64 IEEE-754, redondeo habitual al más cercano con empate a par; sin
  fast-math, reassociation, flush-to-zero ni opciones dependientes de CPU como
  `target-cpu=native`. No se promete el contrato bajo flags que cambien esto.
- No hay SIMD explícito ni selección de kernel según target. Nativo usa sus
  instrucciones escalares f64 (p. ej. SSE2); wasm usa f64 de WebAssembly, sin
  habilitar simd128. La autovectorización que preserve semántica puede operar
  sobre componentes independientes; no puede reagrupar una reducción f64.
  Las compilaciones verificadas usan los flags Cargo habituales, sin RUSTFLAGS
  especiales, tanto debug como release.
- No hay FMA implícito: multiplicación y suma escritas separadamente conservan
  sus redondeos. **Excepción explícita existente:** `modes.rs`, formación de
  `K−σM` para Sturm con `(-sigma).mul_add(mi, ki)`. Es un único redondeo por
  contrato, hardware o software, también en wasm; no se sustituye por `a*b+c`.

Orden del backend, fijado por el código propio:

| Operación | Orden y desempates |
| --- | --- |
| Producto denso | `i` fila, `j` columna, `k` creciente; suma izquierda a derecha desde +0 |
| CSR | Tripletas en orden de entrada; duplicados sumados secuencialmente; salida `(fila,columna)` creciente por BTreeMap |
| Solve general | Pivote `k` creciente; máximo absoluto en `k..n`, empate toma último índice (`max_by`); eliminación `i,j` crecientes; sustitución filas decrecientes, términos `k` crecientes |
| Equilibrado | Barridos y máximos de filas en índices crecientes; hasta 40 pasos deterministas |
| LDLᵀ | Bloque activo creciente; diagonal máxima, empate último; off-diagonal por `i` y `j<i` crecientes, primer máximo estricto; actualización triángulo inferior `i,j` creciente |
| Solve LDLᵀ | RHS crecientes; sustitución directa fila y términos crecientes; bloques D en orden de factorización; sustitución inversa filas decrecientes y términos crecientes |
| Cholesky | `i` creciente, `j≤i` creciente, resta de productos `k<j` creciente |
| Jacobi | Búsqueda `i` creciente, `j<i` creciente, primer máximo estricto; rotaciones en ese orden; autovalores con sort estable ascendente (`total_cmp`), empate conserva índice |
| Normalización modal | Columnas crecientes; norma por `i,j` crecientes; componente dominante con max_by (última en empate), signo positivo |

Ensamblaje recorre contribuciones en su orden recibido, con GDL ordenados por
NodeId y componente. Las combinaciones recorren términos en el orden dado;
SRSS/CQC recorren modos e índices de pares crecientes. Ninguna suma se distribuye
entre trabajadores. La política no cambia tolerancias, pivotes ni el solver.

## Funciones escalares: libm aprobada por Jonathan 2026-10-14

La política V1 es **libm 0.2.16, licencia MIT**, aprobada por Jonathan el
2026-10-14. La decisión anterior de conservar std por ausencia de libm en el
registro offline queda sustituida. El registro local ya contiene esta versión;
se incorporó sin red ni código de otros motores.

`Cargo.toml` fija `libm = { version = "=0.2.16", default-features = false }` en
workspace.dependencies; engine-core la hereda. `Cargo.lock` añade exclusivamente
libm 0.2.16 (checksum `b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981`)
y su arista desde engine-core. Deshabilitar la feature predeterminada `arch`
selecciona las implementaciones Rust genéricas, sin kernels por arquitectura.
No se activan features inestables ni intrinsics opcionales de libm.

`engine_core::math` es el único punto de acceso a funciones escalares. Centraliza
sin/cos/sin_cos, tan y trigonométricas inversas, hiperbólicas e inversas, exp,
exp2, exp_m1, ln, ln_1p, log/log2/log10, powf/powi, atan2, hypot y cbrt. Los usos
de bibliotecas, ejemplos y tests de los crates del motor pasan por ese módulo.
`sin_cos` usa libm::sincos y conserva el par (seno, coseno). `powi(x,n)` usa
libm::pow(x,f64::from(n)): todo i32 se convierte exactamente, incluidos MIN/MAX;
puede redondear distinto de las multiplicaciones de std::powi. `log(x,base)`
fija dos logaritmos naturales y una división. No se cambian dominios, validación
de finitud, tolerancias, orden de reducciones ni el FMA explícito de Sturm.

**Excepción documentada:** math::sqrt conserva `f64::sqrt` de std. IEEE-754
exige raíz correctamente redondeada (al más cercano, empate a par) para
operandos finitos no negativos. Esto no significa que una raíz irracional sea
representable exactamente. Se incluyen subnormales y ceros firmados en el
cruce de bits. libm no garantiza redondeo correcto de todas sus trascendentes;
se fija su implementación para repetibilidad. No se promete identidad de los
payloads de NaN; los caminos de análisis rechazan resultados no finitos.

No se sustituye hypot por sqrt(x*x+y*y): math::hypot mantiene libm::hypot y sus
protecciones frente a overflow/underflow.

| Ruta migrada | Funciones |
| --- | --- |
| Geometría de model, frame/frame3d/truss/spring, resultados, masas/direcciones | hypot |
| ReferenceBackend: LDLᵀ, Jacobi, Cholesky, equilibrado y normalización | hypot y sqrt (excepción IEEE) |
| Modal y SRSS/CQC | sqrt, hypot, powi |
| Interpolación de espectro log-log y distancia entre periodos | ln, ln_1p, exp |
| Rotación/integración de cargas y conformidad de contratos | sin_cos, sqrt |
| Oráculos analíticos y ejemplos/tests | También sin/cos/tan, cosh/tanh, powf/powi; misma política escalar |

## Pruebas y cruce reproducible sin red

`engine-linalg/tests/determinism_guard.rs` mantiene la guardia de colecciones e
hilos sobre `crates/*/src/**/*.rs`: rechaza HashMap, HashSet, thread, rayon y
rayon_*. Añade una guardia escalar sobre **todos los archivos Rust en crates/**,
incluidos tests y ejemplos y crates futuros. Solo exceptúa
engine-core/src/math.rs para trascendentes y libm. Rechaza métodos como
`x.sin()` y `x.r#sin()`, UFCS/referencias como `f64::sin` o `Real::sin`,
y acceso directo/imports de libm fuera del módulo. También exige centralizar
sqrt. Autoriza `engine_core::math::función` y la abreviatura `math::función`
solo con `use engine_core::math;` sin alias: rechaza `as math`, alias de
primitivos, imports sueltos de funciones y redefiniciones del nombre reservado
math. Comprueba la ruta completa, sin aceptar `other::math::función`.
Dentro de macros rechaza identificadores escalares sin esa ruta, por ejemplo
`call!(x, exp)`; conserva receptores de datos como `log.acceleration(...)`.
`mul_add` solo se admite en engine-linalg/src/modes.rs, también frente a
identificadores crudos o argumentos de macros.

`include!` y `#[path = "..."]` solo admiten rutas literales terminadas en .rs,
cuyo destino, canonicalizado respecto al directorio de la fuente, debe estar
en el inventario Rust inspeccionado de crates/**. Rechaza otras extensiones,
rutas calculadas (concat!/env!, etc.) y destinos inexistentes o fuera del
inventario. Comprueba también `path` dentro de `cfg_attr`, incluso anidados,
con espacios y cadenas raw, sin exceptuar math.rs ni examples/.
`include_str!` e `include_bytes!` pueden incluir datos de otras extensiones.

FU2 #107 (J4 r27) añadió la prohibición de `#[link_name = ...]` y bloques
`extern`, con una excepción general para examples/. FU3 (J4 r29/r30) elimina
esa excepción general: en **todos los Rust de crates/** rechaza `link_name`,
`export_name`, bloques `extern "..." { ... }` / `extern { ... }` y las macros
`asm!`, `global_asm!`, `naked_asm!`, incluidas rutas `core::arch::` y
`std::arch::`, espacios e identificadores crudos. Visita atributos `cfg_attr`
anidados y envoltorios `#[unsafe(...)]` con independencia de la condición.
También rechaza atributos con metavariables (`#[$attr]`, `#[$($a)*]`, etc.)
y `extern` con ABI/tokens variables (`extern $abi {}`, `extern $($t)*`,
`extern "C" $body`), incluso dentro de `macro_rules!`. No necesita expandir
macros para cerrar esas vías de código no inspeccionado.

Hasta este cambio, la excepción de unsafe para tests/examples era convencional:
FU2 exigía `forbid(unsafe_code)` en bibliotecas, sin codificar los permisos de
esos directorios, y exceptuaba examples/ en bloque de su comprobación FFI.
Ahora la guardia prohíbe unsafe, ensamblador, export_name, no_mangle, link_name
y bloques o funciones/tipos extern en crates/*/tests/** y crates/*/examples/**,
salvo la lista explícita `SOURCE_POLICY_EXCEPTIONS`, por ruta exacta y por
construcción:

- `engine-linalg/tests/heap_peak.rs`: solo unsafe para instrumentar GlobalAlloc.
- `engine-api/examples/determinism_wasm.rs`: solo no_mangle y funciones públicas
  extern "C" para su ABI wasm; sin permiso de unsafe.
- `engine-validation/examples/winkler_wasm.rs`: los mismos dos permisos para
  su ABI wasm; sin permiso de unsafe.

La guardia falla si desaparece un archivo de la lista o si deja de usar
**cualquiera** de las construcciones autorizadas, aunque conserve otra.
Ninguna excepción permite ensamblador, export_name, link_name, bloques extern,
atributos/extern variables ni relaja las reglas de include!/path o escalares.
`extern crate` conserva las prohibiciones previas. En src/, las funciones y
tipos con ABI extern siguen distinguiéndose de los bloques de importación.

Cada biblioteca crates/*/src/lib.rs declara `#![forbid(unsafe_code)]` en la
raíz, de forma incondicional, y la guardia comprueba su presencia. La lista
explícita `UNSAFE_LIBRARY_EXCEPTIONS` está vacía: las quince bibliotecas
compilan sin unsafe. El binding Python también compila con la prohibición
pese a sus macros PyO3; se añade allí y se comprueba por separado. No se
inspecciona vendor/**. El `unsafe impl GlobalAlloc` y sus llamadas a System
en engine-linalg/tests/heap_peak.rs instrumentan un test de integración;
no son código de biblioteca ni requieren exceptuar engine-linalg.

La guardia ignora comentarios, cadenas normales/raw y literales char. Cinco
pares de fixtures de texto positivos/negativos cubren FU #105; FU2 añade
cinco tests negativos específicos (path no .rs, destino fuera del inventario,
link_name, bloque extern y ausencia de forbid), comprobados retirando cada
regla por separado, y positivos para comentarios, cadenas,
atributos anidados y destinos Rust inspeccionados. FU3 añade negativos por
construcción, directamente y dentro de macros con tres delimitadores, en src/,
tests/ y examples/; también comprueba permisos mínimos, rutas exactas y
excepciones obsoletas por permiso o por archivo. Se comprueban retirando sus
reglas individualmente; se conservan positivos sin falsos positivos para
comentarios, strings normales/raw y atributos no relacionados. Conserva las
pruebas de aliases, espacios, macros y todas las funciones escalares de la lista.
Es una guardia léxica conservadora, no resolución de tipos; no inspecciona
dependencias transitivas ni código generado por build scripts. Cambiar proveedor
requiere revisión.

`engine-api/tests/determinism.rs` ejecuta diez veces once entradas válidas:
Status::Ok, canonización idempotente, SHA-256 del documento completo y también
igualdad directa de bytes. Se mantienen voladizo, voladizo inclinado, modal de
un GDL, P-Delta, modal de dos GDL acoplados y espectro log-log/CQC; se añaden
P-Delta, modal y espectro con geometría inclinada (coordenadas 0,8x/0,6x),
incluyendo las cargas rotadas de P-Delta. Ambos fixtures P-Delta declaran
`options.allow_experimental: true`: el método permanece Experimental hasta
BENCH-001 por decisión de Jonathan; incluirlo en esta suite no lo certifica
como Validated. Se añaden dos entradas frame3d del
mismo pórtico espacial: `frame3d-inclined-static` (linearStatic) y
`frame3d-inclined-modal` (seis modos, masas nodales en los seis GDL de cada nodo
libre y participación en X/Y/Z). La API exige un análisis por entrada.
El pórtico tiene tres nodos (0,0,0), (2.4,1.8,1.3), (4.1,0.7,3.2), dos miembros
inclinados en los tres ejes y referencias de z local (0.3,−0.4,1), (−0.6,1,0.2).
Iy=3 e Iz=5 evitan simetría de flexión. El test nativo comprueba recuperación
de los doce esfuerzos locales por miembro, seis modos positivos y que cambiar
las orientaciones cambia los resultados mecánicos de ambos análisis.
Son variantes de fixtures propios, sin datos de otros motores.
Winkler se integró en #99 y está presente tras el merge; estos dieciséis casos
no cubren esa ruta. Tiene su propia suite en engine-validation. El harness usa
el módulo escalar central y la guardia pasa en la reejecución del 2026-10-15.

Los ejemplos nativo y cdylib wasm comparten esas once entradas y cinco casos
adicionales: scalar-trig, scalar-exp-log, scalar-powers-norm, load-rotation-bits
y scalar-sin-cos.
Estos últimos calculan SHA-256 directamente de `f64::to_bits().to_be_bytes()`:
no hay JSON, tolerancia ni normalización que oculte ceros firmados o redondeos.
Ejercitan todas las funciones del módulo, subnormales, valores pequeños/grandes,
argumentos trigonométricos de hasta 1e300, atan2 con ceros firmados, exponentes
enteros extremos, y sin_cos a través de Beam2D::forces_to_global. El caso
independiente scalar-sin-cos usa directamente engine_core::math::sin_cos y
hashea el par (seno, coseno) para dieciséis ángulos: ceros firmados,
subnormales, vecinos binary64 de π/2 y argumentos de magnitud hasta 1e300.
El envoltorio sobre libm::sincos y su uso al calcular seno/coseno del mismo
ángulo en el motor ya estaban presentes en fccda4d; se conservan.
Solo incluyen resultados finitos y verifican que las listas no estén vacías.

Desde ENG-CORE-005, los hashes de cruce se calculan sobre `results/1` después
de comprobar que `target` coincide con el target de Cargo y retirar únicamente
ese campo. Se vuelve a serializar con `engine_api::canonical_json`, conservando
el contrato de salida; no se aplica JCS a los resultados. `git_commit`,
`input_sha256` y todos los campos numéricos permanecen en la comparación.
La prueba nativa de diez corridas sigue comparando los bytes y el hash del
**documento completo**, incluido `target`. La prueba de proyección rechaza target
ausente/incorrecto y comprueba que todos los demás campos se conservan.
`mamut_canon::sha256_hex` hashea los bytes recibidos sin canonicalización;
los helpers decodifican su hex a los mismos 32 bytes del ABI anterior. Los casos
escalares concatenan exactamente los mismos bytes big-endian, sin JSON.

ABI C: `fixture_count()`, `fixture_name_hash(index,word)`, `run_status(index)` y
`run_hash(index,word)`. Cuatro u64 big-endian forman cada SHA-256; cada palabra
calcula de nuevo el caso, sin estado mutable global. Node recibe u64 como BigInt
y normaliza su signo. El script verifica nombres/orden, cobertura obligatoria,
ABI fuera de rango, cero imports, diez hashes estables e igualdad entre targets;
falla ante la primera diferencia. Sin wasm-bindgen, WASI, npm ni red.

Reproducción local con Node (conservar el CARGO_TARGET_DIR ya exportado):

```bash
export PATH=/home/box/.cargo/bin:$PATH
export CARGO_NET_OFFLINE=true
unset MAMUT_ENGINE_GIT_COMMIT # tabla reproducida con git_commit: null
cargo test --offline -p engine-linalg --test determinism_guard
cargo test --offline -p engine-api --test determinism
cargo build --offline -p engine-api --example determinism_wasm --target wasm32-unknown-unknown
cargo run --offline -p engine-api --example determinism_native > /tmp/eng-core-006-native-debug.txt
node scripts/check-determinism.mjs \
  "$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/examples/determinism_wasm.wasm" \
  /tmp/eng-core-006-native-debug.txt
cargo build --offline --release -p engine-api --example determinism_wasm --target wasm32-unknown-unknown
cargo run --offline --release -p engine-api --example determinism_native > /tmp/eng-core-006-native-release.txt
node scripts/check-determinism.mjs \
  "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/examples/determinism_wasm.wasm" \
  /tmp/eng-core-006-native-release.txt
cmp /tmp/eng-core-006-native-debug.txt /tmp/eng-core-006-native-release.txt
```

La guardia escalar y las cuatro pruebas de determinismo de engine-api pasan
en el árbol reconciliado; no se omiten comprobaciones para ejecutar el cruce.

Se requiere toolchain con wasm32-unknown-unknown, dependencias provisionadas y
Node con WebAssembly/BigInt. El cruce es adicional a cargo test/check:
`cargo check` para wasm solo compila, no ejecuta los análisis.

## Evidencia y límites

Cruce real offline el 2026-10-15 al reconciliar #103 (HEAD 67c54d3) con
#104 (MERGE_HEAD 6a4b58d; origin/main 19c3ae5), sin finalizar el merge:
Rust 1.99.0 (LLVM 23.1.1), x86_64-unknown-linux-gnu frente a
wasm32-unknown-unknown en Node v22.19.0,
CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6, flags Cargo habituales
y MAMUT_ENGINE_GIT_COMMIT sin definir (`git_commit: null`).
**Debug y release: 16 hashes iguales nativo/wasm, diez corridas por caso/target**;
para análisis se valida y excluye únicamente `target`. `cmp` de las salidas
nativas confirma los mismos 16 hashes entre perfiles.

Frente a #103 cambian los once hashes de análisis por la huella JCS y la
incorporación de `git_commit: null` (`target` no participa en el cruce).
Los cinco hashes escalares permanecen idénticos. Nueve análisis coinciden
con la tabla de #104. Solo `pdelta` y `pdelta-inclined` difieren de #104:
los fixtures de #103 añaden `options.allow_experimental: true` y cambia
`input_sha256`. Sustituir únicamente ese campo por la huella JCS del fixture
de #104 reproduce exactamente ambos hashes de #104; no cambian otros campos
ni resultados mecánicos. El registro publica `method_descriptors` en
capabilities/1, fuera de esta suite. La tabla anterior/nueva y las huellas
P-Delta están en [ENG-CORE-006](../provenance/ENG-CORE-006.md).
La evidencia anterior de #98/#103 y #104 se conserva en las bitácoras
ENG-CORE-006 y ENG-CORE-005, respectivamente.

La evidencia cubre once análisis V1 y cinco casos de bits escalares; no
certifica todos los inputs/arquitecturas ni los bits intermedios del solver.
libm fijada elimina la alternativa std de precisión no especificada para las
trascendentes, pero cualquier backend, versión de libm o flags nuevos requiere
volver a ejecutar el cruce. Ante diferencias registrar entrada exacta,
versiones/flags, hashes y campo/valores afectados; una tolerancia numérica no
sustituye igualdad bit a bit.

MAM-581 (#134, tras #103 en main 5d0139c): los resultados P-Delta añaden
`residual_convergence_ratio` por iteración y la parada usa el criterio por GDL con
piso de redondeo. Cambian solo `pdelta` (980bf21b…56ca → f2cc535e…6740) y
`pdelta-inclined` (9b5a4584…a16b → 70825cf7…978b); los otros 14 hashes siguen iguales.
Cruce offline 2026-10-15 con stable, Node: 16 casos nativo=wasm en debug y release,
diez corridas por caso; `cmp` debug/release OK.

SHA-256 de cruce (análisis sin `target`; bits escalares íntegros), iguales en
nativo/wasm y debug/release:

| Entrada / caso de bits | Hash |
| --- | --- |
| cantilever | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` |
| inclined-cantilever | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` |
| modal-1dof | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` |
| pdelta | `f2cc535ed32fb2ed071a3ed8048ebf53c2cd81083efe8ba528e5c95d8cf46740` |
| modal-coupled | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` |
| spectrum-log-cqc | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` |
| pdelta-inclined | `70825cf724a03f9b82f1fd21c76175645a78bc4fcaea648504a7bfc6fd3b978b` |
| modal-inclined | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` |
| spectrum-inclined | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` |
| frame3d-inclined-static | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` |
| frame3d-inclined-modal | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` |
| scalar-trig | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` |
| scalar-exp-log | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` |
| scalar-powers-norm | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` |
| load-rotation-bits | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` |
| scalar-sin-cos | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` |

Los hashes registran esta versión del contrato, motor y casos; cambios legítimos
pueden alterarlos. La tabla es la referencia reproducible; las pruebas comparan
ejecuciones, sin congelar hashes como goldens en código.
