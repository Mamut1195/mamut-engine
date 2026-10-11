# Bitácora clean-room · ENG-GEO-001

- Ticket / PR: ENG-GEO-001 (Perfil de suelo para análisis); sin commit, push ni PR creado en esta sesión.
- Autor(es): Codex gpt-6.1-sol, esfuerzo high, bajo dirección de J2 · 2026-10-11 (America/Guayaquil; fecha indicada para esta bitácora).
- Rol: especificador e implementador; sin actividad de operador de oráculo ni auditor de similitud.
- Contrato implementado: crate `engine-geo` 0.1.0, registrado en el workspace, dependiente únicamente de `engine-core` por path. Trait propio `StressProfile: Send + Sync`, utilizable como `dyn StressProfile`, contrato `stress-profile/1` (mayor 1, menor 0). `StratifiedProfile` valida capas contiguas por espesor, nivel freático y peso unitario del agua; calcula tensión vertical total, presión de poro hidrostática y tensión efectiva. `GeoResult<T> = Result<T, Diagnostic>`, con códigos del catálogo del núcleo y contexto en `hint`. No implementa `element/1` ni ningún `kind` estructural.
- Fuentes permitidas usadas:
  - Documentos internos leídos primero: `README.md`, `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md`, `provenance/ENG-CORE-003.md`, `crates/engine-core` (tipos, `Diagnostic`, catálogo `ENG-E-*`, unidades), `crates/engine-contracts` (contratos y tipos públicos) y `docs/CONVENCION_EJES.md`. También se revisó `CONTRIBUTING.md` y la configuración del workspace. No se modificaron documentos protegidos ni el núcleo.
  - Ecuaciones estándar de mecánica de suelos proporcionadas en la especificación de J2: `σv = Σ γi hi`, `u = γw (z − zw)` bajo el nivel freático, `u = 0` encima y `σ'v = σv − u`. Implementación y ejemplos numéricos derivados aquí desde cero.
  - Referencia conceptual general: principio de esfuerzos efectivos de Terzaghi, tratado por Braja M. Das, *Principles of Geotechnical Engineering*, Cengage, capítulo general de esfuerzos en una masa de suelo (*Stresses in a Soil Mass*). **PENDIENTE: verificar edición/página**; también PENDIENTE número de capítulo/sección, ecuación y ejemplo de libro. No se consultó el texto del libro ni se atribuye a él ningún ejemplo numérico de esta bitácora.
  - Comprobación bibliográfica limitada a la búsqueda de la ficha editorial de [Cengage, Principles of Geotechnical Engineering, SI Edition](https://au.cengage.com/c/principles-of-geotechnical-engineering-si-edition-10e-das/9780357420485/): los resultados de búsqueda identifican una ficha de 10.ª edición y el título general del capítulo. La apertura de la ficha falló por redirección; esto no verifica una edición/página de las ecuaciones, ni sustituye la consulta pendiente del libro. No se usaron datos ni listados de código de fuentes externas.
- Derivaciones propias: integración por tramos y cálculos manuales de las secciones siguientes; [rustdoc y API](../crates/engine-geo/src/lib.rs), [implementación](../crates/engine-geo/src/profile.rs) y [pruebas](../crates/engine-geo/src/tests.rs).
- Código de terceros copiado: ninguno. Implementación propia con biblioteca estándar y API interna de `engine-core`; sin dependencias adicionales, sin código ni datos de motor-geotecnico.
- Validación: analítica, con resultados calculados a mano abajo y literales independientes en las pruebas; 14 pruebas unitarias y 3 doctests en el nuevo crate. Libro (edición, ejemplo, página): **PENDIENTE: verificar edición/página**. Ejemplo publicado: ninguno; los casos A–H son propios. Workspace completo: 57 pruebas unitarias y 28 doctests satisfactorios. Verificación de formato, clippy, tests y rustdoc detallada abajo; sin ejecución de CI remota.
- Comparación con oráculo (repo hermano, informativa): no. No se ejecutó ni consultó OpenSees, CalculiX u otro motor.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket." Tampoco consulté CalculiX, motor-geotecnico ni código de otros motores de las rutas prohibidas. Código, diseño de API, documentación y fixtures escritos desde cero.
- Revisor clean-room: **PENDIENTE** de revisión humana; firma y aprobación de Jonathan pendientes.

## Derivación por integración de peso propio

Se usa `z` como profundidad positiva hacia abajo, con superficie en `z = 0`.
Para la convención global aprobada, Z apunta hacia arriba y
`z = Z_superficie − Z`. El perfil usa espesores; no importa cotas globales ni
interpreta el plano D2 horizontal como plano vertical. La compresión del suelo
se toma positiva; una futura integración con tensión estructural positiva en
tracción deberá invertir el signo y convertir las unidades explícitamente.

Para una sección horizontal de área A, el peso de un tramo de altura h es
`W = γ A h`. Dividir entre A da `Δσv = γ h`. Sumando los tramos sobre la
sección consultada se obtiene `σv(z) = Σ γi hi`.

En una capa entre profundidades a y b, para `z > a`, sea `c = min(z, b)`.
Las alturas propias que se integran son:

```text
h_sobre = max(min(c, zw) − a, 0)
h_bajo  = max(c − max(a, zw), 0)
Δσv     = γ h_sobre + γsat h_bajo
```

Para `z <= a` la capa no aporta peso. Las alturas son disjuntas y su suma
es `c − a`; la división funciona tanto en una interfaz como dentro de una
capa. γsat es el peso unitario total saturado: no se le suma γw otra vez.

El equilibrio hidrostático con presión manométrica cero en `zw` da
`u(z) = γw max(z − zw, 0)`. El principio de esfuerzos efectivos se aplica
como `σ'v(z) = σv(z) − u(z)`. No se calcula succión sobre zw.
Cada altura aporta cero al empezar, por lo que σv es continua en interfaces
y en zw, aunque cambia su pendiente. u y σ'v también son continuas.

Unidades de toda esta API: m, kN/m³ y kPa, con
`(kN/m³)·m = kN/m² = kPa`. El núcleo y los contratos estructurales emplean
Pa; se debe usar `engine_core::units::convert(kpa, Kilopascal, Pascal)` en
esa frontera. No se usa `UnitsOut` para etiquetar un número en kPa como Pa:
esa estructura declara unidades base y no convierte números. El núcleo no
incluye unidades de peso unitario, y no se modifica en este ticket.

## Validación analítica: casos propios calculados a mano

Salvo indicación contraria, los pesos son Measured y γw = 10 kN/m³,
**dato explícito de prueba**, no constante del módulo ni dato de un libro.
Los perfiles secos también declaran γw y zw explícitamente. Todas las
profundidades/alturas siguientes están en m y las tensiones en kPa.
Los números esperados son literales calculados mediante las fórmulas anteriores,
sin llamar a la implementación para producirlos.

### A. Homogéneo seco, nivel freático en la base

H = 10, γ = 18, zw = 10; γsat ausente, permitido porque no hay tramo sumergido.

| z | σv = γz | u | σ'v = σv − u |
| --- | --- | --- | --- |
| 0 | 18·0 = 0 | 0 | 0 |
| 4 | 18·4 = 72 | 0 | 72 |
| 10 | 18·10 = 180 | 10·(10−10) = 0 | 180 |

Comprueba los extremos inclusivos, σv = γz y γsat no requerido en la base.

### B. Homogéneo, nivel freático en superficie

H = 10, γ = 18, γsat = 20, zw = 0. Todo el peso integrado es saturado.

| z | σv | u | σ'v |
| --- | --- | --- | --- |
| 0 | 20·0 = 0 | 10·0 = 0 | 0 |
| 4 | 20·4 = 80 | 10·4 = 40 | 80−40 = 40 |
| 10 | 20·10 = 200 | 10·10 = 100 | 200−100 = 100 |

También se contrastan los tres atajos escalares del trait a z = 4.

### C. Nivel freático dentro de una capa

H = 10, γ = 18, γsat = 20, zw = 3.

| z | σv | u | σ'v |
| --- | --- | --- | --- |
| 2 | 18·2 = 36 | 0 | 36 |
| 3 | 18·3 = 54 | 10·0 = 0 | 54 |
| 5 | 18·3 + 20·2 = 54+40 = 94 | 10·2 = 20 | 94−20 = 74 |
| 10 | 18·3 + 20·7 = 54+140 = 194 | 10·7 = 70 | 194−70 = 124 |

Con ε = 0.000001, justo sobre zw:
`σv(3−ε) = 54−18ε = 53.999982`, `u = 0`, `σ'v = 53.999982`.
Justo debajo: `σv(3+ε) = 54+20ε = 54.000020`,
`u = 10ε = 0.000010`, `σ'v = 54.000010`.
Estas diferencias tienden a cero: no se introduce salto al dividir la capa.
El doctest principal usa z = 5 y γw Assumed: 94, 20, 74.

### D. Multicapa, nivel freático en la interfaz

Capa 0: H₀ = 2, γ₀ = 16, sin γsat. Capa 1: H₁ = 3, γ₁ = 18,
γsat₁ = 21. zw = 2, H total = 5.

| z | σv | u | σ'v |
| --- | --- | --- | --- |
| 1 | 16·1 = 16 | 0 | 16 |
| 2 | 16·2 = 32 | 0 | 32 |
| 4 | 16·2 + 21·2 = 32+42 = 74 | 10·2 = 20 | 74−20 = 54 |
| 5 | 16·2 + 21·3 = 32+63 = 95 | 10·3 = 30 | 95−30 = 65 |

Con ε = 0.000001:
`σv(2−ε) = 32−16ε = 31.999984`, `u = 0`, `σ'v = 31.999984`;
`σv(2+ε) = 32+21ε = 32.000021`, `u = 0.000010`,
`σ'v = 32.000011`. La interfaz no añade ningún peso ni salto.

### E. Nivel freático bajo el perfil

Las mismas dos capas secas: H₀ = 2, γ₀ = 16; H₁ = 3, γ₁ = 18;
zw = 8, γsat ausente en ambas. En z = 2: `σv = 16·2 = 32`,
`u = 0`, `σ'v = 32`. En z = 5:
`σv = 16·2 + 18·3 = 32+54 = 86`, `u = 0`, `σ'v = 86`.
No se extrapola hasta zw.

El doctest seco menor usa H = 2, γ = 18, zw = 3:
z = 0 produce (0, 0, 0), z = 2 produce (36, 0, 36),
y z = 2.01 se rechaza por estar bajo la base.

### F. Tres capas, interfaces seca y sumergida

Capas: (H₀ = 2, γ₀ = 16, γsat₀ ausente),
(H₁ = 3, γ₁ = 18, γsat₁ = 21),
(H₂ = 4, γ₂ = 19, γsat₂ = 22). zw = 3, H total = 9.

En z = 2: `σv = 16·2 = 32`, `u = 0`, `σ'v = 32`.
En z = 5: `σv = 16·2 + 18·1 + 21·2 = 32+18+42 = 92`,
`u = 10·2 = 20`, `σ'v = 92−20 = 72`.
En z = 9: `σv = 92 + 22·4 = 180`, `u = 10·6 = 60`,
`σ'v = 180−60 = 120`.

Con ε = 0.000001, valores manuales alrededor de ambas interfaces:

| z | σv | u | σ'v |
| --- | --- | --- | --- |
| 2−ε | 32−16ε = 31.999984 | 0 | 31.999984 |
| 2+ε | 32+18ε = 32.000018 | 0 | 32.000018 |
| 5−ε | 92−21ε = 91.999979 | 20−10ε = 19.999990 | 71.999989 |
| 5+ε | 92+22ε = 92.000022 | 20+10ε = 20.000010 | 72.000012 |

### G. Reporte de parámetros ASSUMED

Se reutiliza el caso D, aportando además γsat₀ = 20 aunque la capa 0 esté seca.
Se declaran Assumed: γ₀, γsat₀, γsat₁, zw y γw; γ₁ es Measured.
El reporte exacto, sin duplicados y con índices desde cero, es:
`LayerGamma(0), LayerGammaSat(0), LayerGammaSat(1), GroundwaterDepth, WaterUnitWeight`.
Se verifica el mismo reporte en z = 0, 1 y 5, incluyendo datos que no
intervienen en esas consultas. Los resultados numéricos no cambian:
z = 5 produce (95, 30, 65). Modificar el vector devuelto no altera el perfil.

También se prueba H = 2, γ = 18 Assumed, γsat = 20 Measured, zw = 0
y γw = 10 Measured: γ es Assumed y se reporta aunque no se use al estar
toda la capa sumergida. En z = 2, las fórmulas dan (40, 20, 20).
Con todos los datos Measured, el reporte queda vacío.

### H. No imponer relación entre pesos ni recortar tensión efectiva

Caso deliberadamente ajeno a valores usuales de suelo: H = 2, γ = 18,
γsat = 8, zw = 0, γw = 10. Todos son positivos.
En z = 2: `σv = 8·2 = 16`, `u = 10·2 = 20`,
`σ'v = 16−20 = −4`. Se comprueba que γsat < γ se admite y que el módulo
no introduce una tensión efectiva cero ni corrige el dato silenciosamente.
La valoración física de esos pesos queda a cargo del consumidor.

## Validación negativa y aritmética

- Perfil vacío, espesores 0, −0 y −1, pesos γ/γsat/γw no positivos: `ENG-E-0010`. Se valida un γsat aportado incluso si no se utiliza.
- z = −1, z = −MIN_POSITIVE y z = 10.000001 para H = 10: `ENG-E-0010`; también se comprueba propagación en los tres atajos escalares. z = 0 y z = H son válidos, sin tolerancia ni extrapolación.
- zw = −0.1: `ENG-E-0010`; el nivel freático sobre el terreno se rechaza explícitamente.
- NaN, +∞ y −∞ en espesor, γ, γsat, γw, zw y profundidad: `ENG-E-0008`; incluye γsat no utilizado en un perfil seco.
- γ, γw o zw ausente: `ENG-E-0003`. γ es obligatorio incluso en una capa totalmente saturada; γw es obligatorio incluso si no genera presión de poro.
- γsat ausente con zw = 0, dentro de una capa o en una capa profunda bajo una interfaz: `ENG-E-0003`, con posición/campo en `hint`; el constructor valida el perfil completo. El doctest negativo usa H = 2, γ = 18, zw = 0 sin γsat.
- Suma de dos espesores `f64::MAX`: total no finito, `ENG-E-0008`. Espesores 1e20 y 1: la segunda interfaz colapsa en f64, `ENG-E-0010`; no se elimina esa capa.
- H = 2, γ = MAX, zw = 3: z = 0 da (0, 0, 0), pero `σv(2) = 2·MAX` desborda, `ENG-E-0008`. H = 2, γsat = MAX, zw = 0 también desborda al integrar. Dos capas secas H = 1 y γ = 1e308 tienen productos individuales finitos, pero la suma 2e308 desborda y se rechaza.
- H = 2, γ = γsat = 1, zw = 0, γw = MAX: `σv(2) = 2` es finita, pero `u(2) = 2·MAX` desborda, `ENG-E-0008`. Se verifican tensiones y presión antes de devolver un resultado; no se devuelve inf/NaN ni un resultado parcial.
- Los diagnósticos verifican código, gravedad Error y mensaje del catálogo, entidades vacías y `hint` con campo preciso y limitación del catálogo. Las comparaciones analíticas de tensión usan tolerancia absoluta 1e-10 kPa para redondeo f64; los extremos, identidades y reportes se contrastan exactamente.

## Decisiones de alcance y límites del contrato

Se usan capas horizontales, contiguas y de peso unitario constante, declaradas
por espesor. No se implementan cotas alternativas, geometría inclinada,
cohesión, capilaridad, succión, sobrecargas ni agua embalsada sobre el terreno.
No se modelan flujo, presión de poro no hidrostática, consolidación,
deformación, resistencia, estabilidad ni tensiones horizontales.

`zw = 0` satura todo el perfil; `zw = H` y `zw > H` dejan todas las alturas
integradas sobre el agua. γsat puede faltar solo en capas sin tramo sumergido,
y cualquier γsat aportado sigue validándose. No hay valores por defecto,
ni supuesto implícito de γw = 9.81 o 10, ni reemplazo γsat = γ.
γ, zw y γw siempre se requieren. Cada valor lleva un origen declarado;
el módulo no verifica si una medición fue realizada ni su calidad.

El reporte incluye **todos** los parámetros presentes marcados Assumed,
incluidos los no utilizados, para que una consulta superficial no oculte
supuestos del perfil. Se ordena por capas (γ y γsat), luego zw y γw.
Los atajos escalares extraen un número de `stresses_at`; para obtener
procedencia, el consumidor debe conservar el `StressResult` completo.

El núcleo no tiene códigos de falta de datos geotécnicos, dominio físico,
rango de profundidad ni geometría de suelo. Se reutiliza `ENG-E-0003`
(Referencia rota) para el dato ausente, y `ENG-E-0010` (Dimensión inválida)
para rangos/geometría, siguiendo la práctica de validación de parámetros
de `engine-contracts`. `ENG-E-0008` también se usa para resultados no finitos.
No se cambia `engine-core`. Mensaje/gravedad se conservan; el `hint` informa
la limitación semántica y la acción necesaria. No se inventa una entidad
estructural para una capa; `Diagnostic.entities` queda vacío.

Complejidad por consulta: O(n) en capas alcanzadas y O(a) para copiar los
a identificadores Assumed del perfil. No se interpola ni discretiza la
integración; solo hay redondeo de f64. Se rechazan espesores que colapsan
una interfaz al acumular; las tensiones se validan al consultarlas.
El perfil queda inmutable después del constructor y la respuesta es pura.

Semver: paquete 0.1.0; un cambio incompatible antes de 1.0 incrementará
la versión menor del paquete. Cambiar unidades, signos o requisitos del
trait exige nueva versión mayor del contrato `stress-profile`; extensiones
compatibles cambian su versión menor. Se comprueban identidad y compatibilidad
con minor 7, e incompatibilidad con major 2, usando `ContractId` del núcleo.

## Verificación local

El toolchain predeterminado Rust 1.85.1 no tiene instalados rustfmt ni clippy;
los primeros intentos sin override informaron componentes ausentes.
Se usaron los componentes ya instalados en `stable` (Rust 1.99.0), sin
instalar herramientas ni modificar configuración global. Todos los comandos
usan los ejecutables de `~/.cargo/bin`:

```bash
RUSTUP_TOOLCHAIN=stable ~/.cargo/bin/cargo fmt
RUSTUP_TOOLCHAIN=stable ~/.cargo/bin/cargo clippy --all-targets -- -D warnings
RUSTUP_TOOLCHAIN=stable ~/.cargo/bin/cargo test
RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps
```

Todos satisfactorios. Además, `~/.cargo/bin/cargo test` pasó con Rust 1.85.1
(compatible con rust-version 1.85 del workspace). Ambas ejecuciones de tests:
57 unitarias y 28 doctests; engine-geo aporta 14 y 3, respectivamente.
`#![deny(missing_docs)]` está activo en engine-geo y rustdoc termina sin warnings.
La revisión final del diff limita los archivos a Cargo.toml, Cargo.lock,
el nuevo crate y esta bitácora; no hay modificaciones a rutas protegidas.
