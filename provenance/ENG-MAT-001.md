# Bitácora clean-room · ENG-MAT-001

- Ticket / PR: ENG-MAT-001, «Ley uniaxial elástica y sección elástica»; rama `j2/eng-mat-001-elastic`. Sin commit, push ni PR en esta sesión.
- Autor(es): Codex gpt-6.1-sol high, orquestado por J2; fecha 2026-10-10/11 (America/Guayaquil), según la identificación solicitada para esta entrega. No se delegó a otros agentes.
- Rol: implementador; sin actividad de operador de oráculo ni auditor de similitud.
- Contrato implementado: `uniaxial/1`, kind `elastic`, y `section/1`, kind `elastic_frame`, versiones de contrato 1.0. Crate y métodos versión semántica 0.1.0.
- Fuentes permitidas usadas: contratos y especificaciones internos enumerados abajo, convención aprobada de ejes y derivaciones analíticas propias. No se consultaron libros ni fuentes externas para esta implementación. Para una futura atribución bibliográfica de las relaciones elementales: **PENDIENTE: cita exacta**; no se inventan autores, páginas, capítulos ni números de ecuación.
- Derivaciones propias: [nota de esta bitácora](#derivaciones-propias-y-fixtures); [fixtures y pruebas](../crates/engine-laws/tests/elastic.rs). Los valores de referencia se calcularon a mano, sin extraer resultados de otro motor.
- Código de terceros copiado: ninguno. Código propio, biblioteca estándar y dependencias locales `engine-core` y `engine-contracts`; sin dependencias externas ni cambios en esos crates.
- Validación: analítico; σ=Eε, N=EAε0, My=EIyκy, Mz=EIzκz y T=GJθ′, tangentes consistentes, conformidad, entradas negativas y repetibilidad.
- Comparación con oráculo (repo hermano, informativa): no (**PENDIENTE ENG-GOV-004**). No se usó ningún motor externo como oráculo.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket." Tampoco consulté código ni manuales de CalculiX u otros motores, ni manuales de OpenSees/Stabileo. No hubo acceso a la red en esta sesión.
- Revisor clean-room: **PENDIENTE** de revisión humana.

## Documentos internos consultados

Antes de escribir la implementación se leyeron los traits `UniaxialLaw` y
`SectionLaw`, los ejemplos `LinearElastic` y `ElasticSection2D`, y el kit
`conformance::check_uniaxial_law` / `check_section_law` de `engine-contracts`.
También se leyeron `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md`,
`provenance/ENG-CORE-007.md` y `docs/CONVENCION_EJES.md`.

La especificación autorizada se consultó directamente en
`/workspace/motor-rust/04_TICKETS.md`, sección ENG-MAT-001, y
`/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`, §§4.2 y 4.3.
Se consultaron los tipos de intercambio, almacenamiento matricial y diagnósticos
del núcleo, los manifests del workspace y `clean-room/check.sh`. No se encontró
un `AGENTS.md` aplicable en el workspace ni en sus directorios ascendientes.
El comprobador consume las listas prohibidas existentes; no se alteraron esas
listas ni ninguna ruta protegida. No se abrieron fuentes ni rutas de motores
prohibidos.

## Decisiones de arquitectura

1. `engine-laws` 0.1.0 se añade al workspace. Depende exclusivamente de los dos
   crates locales autorizados, con versión 0.1.0 y path. `Cargo.lock` registra
   solamente el nuevo paquete y esas dependencias existentes.
2. `lib.rs` contiene documentación general, descriptor común y validadores
   privados. `uniaxial.rs` y `section.rs` mantienen implementaciones, parámetros,
   descriptores y fábricas separados. No se modifica `engine-contracts`.
3. `UniaxialFactory` y `SectionFactory` son interfaces locales `Send + Sync`
   con parámetros asociados tipados. Fábricas con el mismo tipo de parámetros
   son intercambiables mediante objetos del trait; sus productos son
   `Box<dyn UniaxialLaw>` / `Box<dyn SectionLaw>`. Las fábricas concretas delegan
   toda validación al constructor. No se añade registro global, parser JSON ni
   un nuevo contrato compartido. El despacho de modelos queda para el registro
   de métodos futuro.
4. `LawDescriptor` informa kind, contrato y versión semántica del método.
   Los contratos y la versión del método son independientes; el API inicial
   es 0.1.0 y su evolución debe respetar semver. No hay estado global mutable.
5. `Elastic` guarda E validado. `ElasticFrame` conserva propiedades inmutables
   y rigideces validadas; admite D2 y D3 con el orden exacto de `section/1`.
   Las leyes son puras, sin historia ni precarga; estado inicial/de prueba vacío.
6. Se exige finitud y positividad de E, G, A, Iy, Iz y J incluso en D2, y de
   Ay/Az cuando existen. Se validan todos sus productos: un desbordamiento
   produce `ENG-E-0008`; un producto positivo reducido a cero por
   subdesbordamiento produce `ENG-E-0010`. Entradas no finitas usan
   `ENG-E-0008`; rangos o longitudes inválidos usan `ENG-E-0010`, como los
   contratos existentes. Los mensajes identifican la magnitud SI implicada.
7. G es explícito: no se ofrece constructor desde ν ni se supone isotropía para
   deducirlo. J es la constante de torsión de Saint-Venant suministrada, no una
   inercia polar calculada. No hay calculadora geométrica, alabeo, plasticidad,
   daño, masa ni transformación a fuerzas de extremo.
8. Ay/Az son áreas efectivas de corte que ya incorporan cualquier corrección
   elegida por el consumidor. GAy/GAz se exponen como accesores `Option<Real>`
   en N, también en D2; no se amplían deformaciones ni resultantes del contrato.
9. La sección supone ejes centroidales principales y material homogéneo:
   la tangente es diagonal, sin acoplamiento axial-flexión o flexión cruzada.
   X-Y es horizontal, Z vertical, terna diestra; D2 sigue en X-Y con Z=0.
   No se adopta la propuesta pendiente de elevación D2.
10. `#![deny(missing_docs)]` exige rustdoc en todo el API público. Implementación
    y pruebas solo requieren memoria; no usan sistema de archivos, hilos,
    reloj, servicios de red ni dependencias específicas del host. Los benchmarks
    son fixtures analíticos en código, no mediciones de rendimiento.

## Derivaciones propias y fixtures

Se adopta la idealización de elasticidad lineal σ=Eε, tensión y módulo en Pa,
deformación total adimensional. Diferenciando se obtiene dσ/dε=E constante.
La energía por unidad de volumen es ½Eε²; no hay historia ni tensión inicial.

En los ejes locales centroidales principales, se define
εx(y,z)=ε0+κy z−κz y. Entonces:

- N=∫σx dA=EAε0, porque ∫y dA=∫z dA=0.
- My=∫zσx dA=EIyκy, con Iy=∫z² dA y ∫yz dA=0.
- Mz=−∫yσx dA=EIzκz, con Iz=∫y² dA y ∫yz dA=0.

En una cara de normal +x, estos momentos siguen la mano derecha alrededor de
+y y +z. En la cara opuesta sus acciones cambian de dirección. N positivo
significa tracción. κy y κz son derivadas de giros alrededor de sus ejes locales;
el signo de la distribución axial hace que sean conjugadas a My y Mz positivos.
La sección no decide el signo de acciones sobre los nodos.

Para torsión uniforme de Saint-Venant se adopta T=GJθ′, donde θ′ es la derivada
del giro alrededor de +x. G en Pa, J en m⁴ y θ′ en rad/m dan T en N·m.
La energía por unidad de longitud es
½(EAε0²+EIyκy²+EIzκz²+GJθ′²). Su gradiente da las resultantes y su Hessiana
da la tangente constante diagonal (EA, EIy, EIz, GJ). En D2 se retienen
solo ε0 y κz, con diagonal (EA, EIz). A en m² implica EA en N; Iy, Iz y J
en m⁴ implican EI y GJ en N·m². Las curvaturas están en m⁻¹.

Si el consumidor introduce deformaciones de corte fuera de este contrato,
las relaciones lineales correspondientes son Vy=GAyγy y Vz=GAzγz. Por eso
GAy/GAz tienen unidades N y se exponen únicamente como datos opcionales.

Fixtures propios (cada uno incluye `source` y `citation` a esta nota):

- Uniaxial: E=200 Pa, ε=0.02 da σ=4 Pa; E=200×10⁹ Pa, ε=−0.001
  da σ=−200×10⁶ Pa; ε=0 da σ=0 sin precarga.
- Sección pequeña: E=200 Pa, G=80 Pa, A=3 m², Iy=2 m⁴, Iz=5 m⁴,
  J=4 m⁴. Así EA=600 N, EIy=400 N·m², EIz=1000 N·m², GJ=320 N·m².
  D2 con e=(0.02,−0.03) da (N,Mz)=(12 N,−30 N·m).
  D3 con e=(0.02,−0.03,0.04,−0.05) da
  (N,My,Mz,T)=(12 N,−12 N·m,40 N·m,−16 N·m).
  Ay=1.5 m² y Az=1.2 m² dan GAy=120 N y GAz=96 N.
- Sección de escala estructural: E=200×10⁹ Pa, G=80×10⁹ Pa, A=0.03 m²,
  Iy=2×10⁻⁵ m⁴, Iz=5×10⁻⁵ m⁴, J=4×10⁻⁵ m⁴. Multiplicando a mano:
  EA=6×10⁹ N, EIy=4×10⁶ N·m², EIz=10⁷ N·m², GJ=3.2×10⁶ N·m².
  Con e=(0.002,−0.003,0.004,−0.005), resultan
  (12×10⁶ N,−12000 N·m,40000 N·m,−16000 N·m).

Los datos son elecciones propias, no ejemplos publicados ni resultados de
oráculo. No se atribuyen a una página o ecuación externa. Para cualquier futura
referencia bibliográfica externa: **PENDIENTE: cita exacta**.

## Cobertura y límites de validación

Doce pruebas de integración y dos doctests del crate. Se usan los comprobadores
públicos de conformidad con trayectorias que incluyen cero, tensión/compresión,
signos opuestos y retorno a entradas aceptadas. Las diferencias centrales usan
incremento 1e-7 en las unidades de cada componente; error relativo <1e-6.
Además se verifican todas las entradas de la tangente por separado con
incremento 1e-6 y error relativo <1e-6, evitando que EA oculte errores en EI/GJ.
Los fixtures se comparan con tolerancia 1e-12 absoluta + 1e-12 relativa.

Se comprueban matrices diagonales del tamaño requerido, simetría, ausencia de
historia, repetición pura e inmutabilidad, y `Send + Sync` como restricciones de
compilación sin crear hilos. Se ejercen fábricas mediante objetos de sus traits,
descriptores, errores propagados y accesores. Las áreas de corte se prueban
ausentes, presentes y por separado; no alteran ninguna respuesta generalizada.

Casos negativos: cero, negativo, NaN e infinito en cada propiedad (también
inactivas en D2); longitudes incorrectas de deformaciones e historia; NaN,
infinito y desbordamiento en cada componente; productos de rigidez desbordados
o reducidos a cero. No se verifica una geometría física ni se infiere una
orientación D3. La conformidad solo verifica las muestras, no demuestra
exactitud para todo el rango de números flotantes.

## Verificación local

Resultados finales de la verificación nativa local:

- `~/.cargo/bin/cargo fmt --check`: satisfactorio.
- `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: satisfactorio.
- `~/.cargo/bin/cargo test`: satisfactorio; workspace completo con 55 pruebas
  unitarias/de integración y 27 doctests, incluidos 12 y 2 del nuevo crate.
- `RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps`: satisfactorio.
- `bash clean-room/check.sh origin/main`: satisfactorio (`Clean-room: OK`).
  También se ejecutó con un índice y directorio de objetos temporales en `/tmp`,
  incluyendo mediante intent-to-add todos los archivos nuevos de este ticket,
  sin alterar el índice ni los objetos Git reales. Resultado satisfactorio.
- `git diff --check` sobre ese índice temporal: satisfactorio. El diff completo
  contiene únicamente `Cargo.toml`, `Cargo.lock`, los cinco archivos de
  `crates/engine-laws` y esta bitácora. Ninguna ruta protegida aparece en él.

Se ejecutó primero `cargo test -p engine-laws --offline` para verificar los
fixtures y después la suite completa. No se necesitaron correcciones de
formulación tras esas corridas. No hubo descargas ni CI remota. El índice real
conserva los archivos nuevos sin añadir, disponibles para revisión del usuario.

- Corrida wasm32: **PENDIENTE**, no ejecutada (intento `cargo build --target wasm32-unknown-unknown` en la caja falla con E0463: la std del target no está disponible para el toolchain activo; sin runner wasm). Las pruebas están escritas para
  compilar a wasm32 sin `std::fs`, hilos ni tiempo del sistema; no se afirma
  validación numérica wasm ni equivalencia nativo/wasm en esta entrega.
- Comparación con oráculo: no, **PENDIENTE ENG-GOV-004**.
- Revisor clean-room: **PENDIENTE**.

No se modificaron `clean-room/**`, `.github/workflows`, `CODEOWNERS`,
`docs/CLEAN_ROOM.md` ni archivos legales; no se creó `LICENSE`.
