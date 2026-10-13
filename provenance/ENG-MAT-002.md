# Bitácora clean-room · ENG-MAT-002

- Ticket / PR: ENG-MAT-002, ley uniaxial `multilinear_elastic`; entrega local sin commit ni push.
- Autor(es): J2 (agente, Grok Bot), atribución solicitada por el usuario; implementación realizada por Codex, modelo GPT-6; fecha 2026-10-11 (America/Guayaquil). Sin delegación a otros agentes.
- Rol: implementador, con derivaciones y fixtures analíticos propios.
- Contrato implementado: `uniaxial/1` (versión 1.0), kind `multilinear_elastic`, método 0.1.0, dentro de `engine-laws`.
- Fuentes permitidas usadas: solicitud ENG-MAT-002 del usuario, código y documentación internos enumerados abajo; sin fuentes externas, libros, manuales ni acceso a la red.
- Derivaciones propias: [nota de esta bitácora](#derivaciones-propias-y-fixtures) y [pruebas analíticas](../crates/engine-laws/tests/multilinear.rs).
- Código de terceros copiado: ninguno. Solo biblioteca estándar y las dependencias locales existentes `engine-core` y `engine-contracts`; sin crates ni dependencias nuevos.
- Validación: analítico; valores exactos por tramos y extrapolación, tangentes por diferencias finitas, continuidad, simetría/asimetría, gaps, extremos numéricos, rechazo de entradas inválidas y conformidad uniaxial.
- Comparación con oráculo (repo hermano, informativa): no. No se ejecutó ni consultó ningún motor externo.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket." Tampoco se consultaron fuentes externas ni manuales de otros motores. No hubo acceso a la red.
- Revisor clean-room: PENDIENTE (J4).

## Documentos internos consultados

`provenance/TEMPLATE.md`, `provenance/ENG-MAT-001.md`, `docs/CLEAN_ROOM.md`,
`CONTRIBUTING.md`, `clean-room/README.md` y `clean-room/check.sh`; implementación
y pruebas existentes de `engine-laws`; trait `UniaxialLaw` y kit
`conformance::check_uniaxial_law` de `engine-contracts`; catálogo y errores de
`engine-core`; manifests del workspace. No se encontró un `AGENTS.md` aplicable
en el repo ni en sus ascendientes. No se abrieron fuentes de motores prohibidos.

## Decisiones de implementación

La API se exporta desde `engine_laws::uniaxial`, con implementación privada en
`src/uniaxial/multilinear.rs`. Incluye ley, parámetros tipados, descriptor y
fábrica que implementa el `UniaxialFactory` existente. Igual que ENG-MAT-001,
no hay registro global, parser ni cambios en `engine-contracts`. La tabla y
las pendientes se validan una vez y se conservan inmutables; el estado inicial,
aceptado y de prueba tiene longitud cero. Ley y fábrica son `Send + Sync`.

Se reciben puntos explícitos ordenados `(ε, σ)` en SI; ε adimensional positiva
en tracción y σ en Pa. Debe haber al menos dos puntos y un origen exacto
`(0, 0)`, sin tolerancia ni corrección automática. Puntos reflejados definen una
curva simétrica impar; puntos independientes definen una curva asimétrica.
También se permiten tablas unilaterales con el origen en un extremo: el tramo
extremo se prolonga al otro lado, salvo cuando el modo lo inhabilita.

Los modos son ambos lados, solo-compresión y solo-tracción. El lado inactivo
da exactamente σ = 0 y tangente = 0. El origen pertenece al lado activo;
solo-compresión elige allí la pendiente izquierda si existe, y solo-tracción
la derecha si existe. En los demás vértices se elige el tramo derecho, salvo
el último punto, que usa el último tramo. `initial_tangent()` coincide siempre
con `respond(0).tangent`, incluidas las tablas unilaterales.

En un vértice de pendientes distintas o en la apertura del gap no existe
derivada bilateral. El contrato exige declarar estos puntos; el kit existente
acepta la elección con muestras cuyo intervalo ε ± h sea diferenciable,
y comprueba por separado la tangente inicial contra la respuesta en el origen.
No se afirma que la diferencia central coincida con una derivada unilateral
en un quiebre. Los vértices se prueban directamente sin alterar el kit.

Se admiten mesetas exactas y se rechazan pendientes negativas con
`ENG-E-0010` y contexto del tramo. El mismo código cubre longitudes, orden,
origen e historia inválidos, siguiendo el rango del trait existente. Una
pendiente positiva reducida a cero por subdesbordamiento también se rechaza
con `ENG-E-0010`, para no confundirla con una meseta válida. `ENG-E-0008` cubre
entradas no finitas y desbordamientos en diferencias, cociente, producto y
suma. El mensaje identifica la magnitud; no se añaden códigos al catálogo.
Se valida toda la tabla, aunque un modo deshabilite uno de los lados, y se
validan ε e historia antes de responder en el gap.

La búsqueda del tramo es binaria. Se usa el extremo más cercano como ancla
para reducir cancelación; los puntos tabulados se devuelven exactamente.
Una meseta evita calcular desplazamientos innecesarios y permite evaluar
deformaciones extremas finitas. La implementación de la librería no contiene
`panic!`, `unwrap` ni `expect`. Los índices privados quedan acotados por las
invariantes del constructor; el usuario no puede mutar puntos ni pendientes.

## Derivaciones propias y fixtures

Para dos puntos consecutivos, Δε = εᵢ₊₁ − εᵢ > 0,
Δσ = σᵢ₊₁ − σᵢ ≥ 0 y mᵢ = Δσ / Δε. La única función afín que pasa
por ambos puntos es σ(ε) = σᵢ + mᵢ(ε − εᵢ), con dσ/dε = mᵢ.
Usar el extremo derecho como ancla da la expresión equivalente
σᵢ₊₁ + mᵢ(ε − εᵢ₊₁). Ambas llegan a la misma tensión en cada vértice,
por lo que la curva es continua. Las prolongaciones extremas conservan esa
continuidad y la pendiente del tramo exterior correspondiente. En el gap se
define σ = 0; como el origen tiene σ = 0, la transición también es continua.

El fixture asimétrico propio usa puntos
`(-3, -10), (-1, -6), (0, 0), (1, 4), (3, 6)`; las pendientes son
`2, 6, 4, 1` Pa. Evaluaciones exactas, con datos representables en binario:

| ε | σ (Pa) | Tangente elegida (Pa) |
|---|---|---|
| −4 | −12 | 2 |
| −3 | −10 | 2 |
| −2 | −8 | 2 |
| −1 | −6 | 6 |
| −0.5 | −3 | 6 |
| 0 | 0 | 4 |
| 0.5 | 2 | 4 |
| 1 | 4 | 1 |
| 2 | 5 | 1 |
| 3 | 6 | 1 |
| 4 | 7 | 1 |

Para simetría se usan `(-3, -6), (-1, -4), (0, 0), (1, 4), (3, 6)`.
Para mesetas: `(-2, -4), (-1, -4), (0, 0), (1, 4), (2, 4)`.
Tablas mínimas `(0, 0), (1, 2)` y `(-1, -2), (0, 0)` reproducen σ = 2ε
en ambos signos antes de aplicar el gap. La recta σ = ε se prueba con
extremos `±f64::MAX` y el menor subnormal positivo; las mesetas se extrapolan
hasta esos extremos finitos sin producto ni resta innecesarios.

Las diferencias centrales lejos de vértices usan h = 10⁻⁶ y error relativo
menor que 10⁻⁶. La continuidad en todos los vértices y modos se verifica
exactamente con h = 1/1024 y las pendientes de los lados respectivos.
Los valores de los fixtures llevan `source` y `citation` a esta nota; son
elecciones y derivaciones propias, sin resultados de otro motor.

## Verificación local

Verificaciones ejecutadas sin acceso a red, con `CARGO_NET_OFFLINE=true`.
Las corridas finales usan el toolchain instalado estable (`rustc 1.99.0`,
2026-09-28), anteponiendo `/home/box/.cargo/bin` al PATH; el target wasm ya
estaba instalado y no se descargó nada.

- `cargo fmt` y `cargo fmt --check`: satisfactorios.
- `cargo clippy --all-targets -- -D warnings`: satisfactorio.
- `cargo test --workspace`: satisfactorio; 161 pruebas unitarias/de integración
  y 35 doctests. La nueva ley aporta 12 pruebas de integración y un doctest;
  el crate completo pasa 24 pruebas de integración y 3 doctests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: satisfactorio.
- `cargo check --workspace --target wasm32-unknown-unknown`: satisfactorio.
  Es comprobación de compilación; no se ejecutaron pruebas numéricas en wasm.
- `bash clean-room/check.sh origin/main`: satisfactorio (`Clean-room: OK`),
  incluyendo los tres archivos nuevos mediante `git add -N` en un índice y
  directorio de objetos temporales bajo `/tmp`, sin tocar el índice ni los
  objetos Git reales.
- `git diff --check` con el mismo índice temporal: satisfactorio. El diff solo
  contiene los cuatro archivos de `engine-laws` y esta bitácora.

No se modifican rutas protegidas, manifests ni dependencias. No se crea commit
ni se hace push. La revisión clean-room humana permanece PENDIENTE (J4).
