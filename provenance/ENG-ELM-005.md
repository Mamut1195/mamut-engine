# Bitácora clean-room · ENG-ELM-005

- Ticket / PR: ENG-ELM-005; entrega local sin commit ni push. Apila sobre
  ENG-ELM-004 (`spring.linear`) y ENG-MAT-002 (`multilinear_elastic`).
- Autor(es): J2 (agente, Grok Bot), atribución solicitada; sesión ejecutada por
  Codex (GPT-6), 2026-10-11 (America/Guayaquil). Sin delegación a otros agentes.
- Rol: implementador; derivaciones y fixtures analíticos propios.
- Contrato implementado: element/1 · kind = `spring.nonlinear`.
- Fuentes permitidas usadas: solicitud del usuario y código/documentación internos
  enumerados abajo. Ninguna fuente externa; sin red, libros ni manuales externos.
- Derivaciones propias: [esta nota](#derivaciones-propias-y-fixtures) y
  [pruebas analíticas](../crates/engine-elements-spring/tests/nonlinear.rs).
- Código de terceros copiado: ninguno. Sin crates ni dependencias nuevas,
  incluidas dependencias de desarrollo; manifests y lockfile intactos.
- Validación: analítico; forma cerrada, Newton manual, trayectoria reversible,
  covarianza, diferencias finitas del kit con historia y entradas inválidas.
- Comparación con oráculo (repo hermano, informativa): no. No se ejecutó ningún
  motor externo ni se usaron resultados de oráculos.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Documentos internos consultados

`provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md`, `provenance/ENG-ELM-004.md`,
`provenance/ENG-MAT-002.md`; manifests del workspace; implementación y pruebas
existentes de `engine-elements-spring` y `engine-elements-truss`; tipos, traits,
ejemplos y kit de conformidad de `engine-contracts`; implementación y pruebas de
`engine-laws::uniaxial`, incluido `multilinear_elastic`; tipos de `engine-core`;
referencias de estado en `engine-assembly`; `clean-room/check.sh`.
No hay AGENTS.md aplicable en el repo ni en sus ascendientes.
No se abrieron fuentes ni rutas de motores prohibidos.

## Derivaciones propias y fixtures

Sea c el acoplamiento global fijo, con la misma selección, normalización,
rotación y orden canónico de GDL que en ELM-004. Entre nodos:
δ=c·(u_j−u_i), b=[−c,c]. A tierra: δ=c·u_i, b=c.
La ley se suministra como **fuerza-desplazamiento**, con longitud y área unitarias:

```text
ε = δ                 σ = F(δ, historia_aceptada)
f_int = b F           K = (dF/dδ) b bᵀ
historia_elemento = historia_ley
```

Para traslaciones: δ en m, F en N, tangente en N/m. Para giros: δ en rad,
F representa momento en N·m y la tangente es N·m/rad. No hay conversión
implícita de una tabla de material en Pa/adimensional a fuerza/desplazamiento.
Seleccionar varios GDL de una familia suma sus proyecciones y produce un único
modo acoplado; no son resortes independientes. La tangente se evalúa en cada
prueba, sin reemplazarla por la tangente inicial ni por una secante.

K es simétrica por construcción, incluso para leyes con tangente nula o
negativa; el elemento no restringe el rango constitutivo de una ley compatible.
Entre nodos una traslación uniforme no modifica δ. Un apoyo a tierra elimina
ese modo rígido. Un eje fijo entre nodos separados no es corrotacional y puede
deformarse bajo giro rígido; masa y Kg son ausentes. Se heredan estos límites
de ELM-004. Se admiten coordenadas coincidentes, sin longitud física.

Al rotar modelo, u y eje propio con R, δ se conserva, f'=R f y K'=R K Rᵀ
con bloques nodales. Una inversión de eje cambia el lado activo de una ley
asimétrica: no se promete invariancia frente a esa inversión. Recuperación:
acciones locales ±c_local F y resultantes constantes en las estaciones [0,1],
que son etiquetas sin integración espacial. Solo se admiten cargas de tramo
nulas; las cargas aplicadas al resorte son nodales.

### Bilineal reversible

Fixture propio: puntos (δ,F) en m/N
`(-0.5,-18), (-0.1,-10), (0,0), (0.1,10), (0.5,18)`.
Así F=100δ para |δ|<0.1 y
F=sign(δ)[10+20(|δ|−0.1)] fuera de ese intervalo.
K generalizada es 100 N/m en el tramo central y 20 N/m en los exteriores.
Se prueban ambos signos, extrapolación, nodos coincidentes, desplazamientos
uniformes, recuperación y conexión a tierra/entre nodos.
Newton para carga +16 N a tierra comienza en u=0: u₁=0.16 m,
u₂=0.4 m; F(0.4)=16 N. La trayectoria de carga y descarga reproduce
la misma fuerza y tangente al repetir δ, sin energía disipada ni historia.

### Gap solo-compresión

Fixture propio: puntos `(-1.25,-100), (-0.25,0), (0,0)` en modo
`CompressionOnly`. F=100(δ+0.25) para δ<−0.25 y F=0 para δ≥−0.25.
La tangente es 100 N/m con contacto y cero en el gap abierto.
Nunca da fuerza positiva: el apoyo no tira. En −0.25 se respeta la convención
de MAT-002 (tramo derecho con tangente cero). No existe derivada bilateral allí.
Newton para carga −20 N comienza con contacto en u=−1 m y converge a
u=−0.45 m. En el gap la tangente nula requiere una estrategia de contacto del
análisis; este elemento no añade regularización ni modifica la ley.

### Historia no vacía y conformidad

Ley plástica analítica **solo de pruebas**, con k=100 N/m y límite |F|≤10 N:
historia p=desplazamiento plástico; predictor F_trial=100(δ−p).
Si |F_trial|≤10: F=F_trial, tangente 100 y p_trial=p.
En otro caso: F=sign(F_trial)10, tangente cero y p_trial=δ−F/100.
En δ=0.2 m desde p=0 se obtiene F=10 N y p_trial=0.1 m.
Descartar la prueba deja F(0)=0; aceptarla reproduce F(0.2)=10 y después
F(0.15)=5 N, mientras que sin aceptación F(0.15)=10 N.
Se verifica determinismo, copias independientes de historia inicial y ausencia
de aceptación en respond, recover, Kg y fallos.

`check_element` se ejecuta en D2/D3 con historia plástica no vacía y ambas
leyes multilineales. GDL global Ux y coordenadas coincidentes permiten aplicar
los modos rígidos del kit sin excepciones. Las muestras con aceptación explícita
son δ=0.02, 0.15, 0.12, −0.03, −0.4 m. Se evita −0.05 porque, después de
p=0.05, coincide con el quiebre plástico de descarga: una diferencia central
allí no representa la tangente unilateral. No se filtran diagnósticos ni se
modifica el kit. Covarianza de ejes propios y apoyos a tierra se comprueba con
fixtures separados, conforme a las limitaciones documentadas del kit existente.

## Arquitectura, API y diagnósticos

`NonlinearSpring::new` conserva un `Arc<dyn UniaxialLaw>` ya resuelto;
`from_params` lo obtiene de `ElementParams::law`. Como las barras existentes,
la propiedad constitutiva vive en el elemento, sin cambiarla desde ctx.params.
Se valida contrato uniaxial/1 (versiones menores compatibles), historia inicial
y tangente inicial. `state_len` e `initial_state` describen exactamente la
historia constitutiva validada, sin desplazamientos o fuerzas añadidos.

No hay nuevos métodos de commit/revert en element/1: la historia es propiedad
del llamador. `respond` devuelve trial_state; tras converger el llamador sustituye
su estado aceptado por esa prueba. Revert descarta la prueba y conserva el
último aceptado. El elemento no contiene buffers mutables de pruebas.

La composición privada con `LinearSpring` de rigidez unitaria reutiliza la
proyección (su fuerza generalizada vale δ), contexto, GDL, masa y cargas.
Los helpers de matriz y recuperación reciben ahora la tangente y fuerza
constitutivas; el comportamiento público de `spring.linear` se conserva.
La respuesta valida además tamaño y finitud de la historia de prueba, fuerza
y tangente, incluso si la ley suministrada incumple su contrato.

ENG-E-0010: ley ausente/incompatible, tamaños, geometría, GDL, ejes, estaciones
u otras entradas no admitidas. ENG-E-0008: entradas, historia inicial/aceptada/
de prueba o resultados no finitos y desbordamientos de proyección/fuerza/rigidez.
Los errores de la ley se propagan sin cambiar su código. Sin panic/unwrap/expect
en biblioteca; los índices privados provienen de geometría previamente validada.

El crate no dependía de engine-laws y se respeta la prohibición de añadir
cualquier dependencia. El test integra **el archivo real existente**
`engine-laws/src/uniaxial/multilinear.rs` mediante `#[path]`, sin copiar ni
reescribir su interpolación. Un adaptador privado al binario de prueba aporta
los validadores finitos/de tamaño y las interfaces auxiliares de descriptor y
fábrica que ese módulo importa. Esto no cambia la biblioteca ni la API productiva
ni añade dependencias. La conformidad de MAT-002 sigue probándose también en
su crate original durante `cargo test --workspace`.

## Pruebas y verificación local

Diez pruebas de integración nuevas y un doctest en español. Tolerancia absoluta
1e−9 en fuerzas, desplazamientos y matrices de estos fixtures SI; Newton exige
residuo <1e−10 N; el kit usa h=1e−7 m y exige error relativo FD <1e−6.
Los negativos incluyen ley incompatible, historia inicial inválida, historia
aceptada de longitud errónea/no finita, salidas constitutivas defectuosas,
propagación de error de ley, desplazamientos/contexto/ejes/cargas/estaciones
inválidos, diferencias relativas y fuerzas desbordadas.

Verificaciones finales, todas satisfactorias, con `CARGO_NET_OFFLINE=true`
y el toolchain instalado en `/home/box/.cargo/bin` (sin instalaciones):

- `cargo fmt` y `cargo fmt --check`.
- `cargo clippy --all-targets -- -D warnings`.
- `cargo test --workspace`: todas las pruebas unitarias, de integración y
  doctests; las diez pruebas nuevas y el doctest pasan.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`.
- `cargo check --workspace --target wasm32-unknown-unknown`: compilación
  satisfactoria; no se ejecutaron pruebas numéricas en un runtime wasm.
- `bash clean-room/check.sh origin/main`: `Clean-room: OK`, también incluyendo
  los tres archivos nuevos mediante un índice y directorio de objetos temporales
  en `/tmp`; índice y objetos Git reales del usuario intactos.
- `git diff --check`, incluidos los nuevos archivos en ese índice temporal.
- Búsqueda de `panic!`, `unwrap` y `expect` en la biblioteca sin coincidencias.

Solo se modifican seis archivos: lib.rs, stiffness.rs y recover.rs del crate,
el módulo nonlinear.rs, sus pruebas y esta bitácora. Sin cambios en contratos,
leyes, manifests, lockfile o reglas clean-room. Sin fuentes externas, red,
oráculos, commit ni push. Revisor clean-room: PENDIENTE (J4).

## Ronda J4 r7

- Las pruebas usan la ley real `engine_laws::uniaxial::MultilinearElastic` como dev-dependency interna (path) en lugar de incluir su fuente por `#[path]`; se elimina el código de soporte sin usar. Cargo.lock regenerado (solo dependencia interna).
