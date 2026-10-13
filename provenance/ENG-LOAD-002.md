# Bitácora clean-room · ENG-LOAD-002

- Ticket / PR: ENG-LOAD-002 «Combinaciones lineales y envolventes». Rama apilada
  sobre ENG-RES-001 (#53, `crates/engine-analysis::results`) y ENG-LOAD-001
  (#51, `crates/engine-loads`), ambos sin fusionar. Sin commit ni push.
- Autor(es): J2 (agente, Grok Bot) · identificación solicitada por el usuario;
  agente ejecutor Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: módulo `engine_analysis::combination`, superposición
  sobre `CaseResults` usando `engine_model::Combination`/`CaseFactor`, y
  envolventes algebraicas con gobernante por componente y extremo. Sin formatos
  de serialización nuevos, crates nuevos ni dependencias externas nuevas.
  `engine-model`, ya presente como dependencia de pruebas, pasa a dependencia
  normal para reutilizar su declaración de combinaciones.
- Fuentes permitidas usadas: petición del ticket y APIs propias de `results`,
  `engine-model`, `engine-core`, `engine-contracts`, `engine-assembly`,
  `engine-elements-frame`, `StaticLinear` y backend de referencia existente.
  Plantilla: `provenance/TEMPLATE.md`. Sin red ni fuentes externas.
- Derivaciones propias: [nota de esta bitácora](#derivaciones-propias) y
  [pruebas analíticas](../crates/engine-analysis/tests/combination.rs).
- Código de terceros copiado: ninguno.
- Validación: exclusivamente analítica; superposición, equilibrio y solución
  cerrada de viga en voladizo. Sin fuentes ni oráculos externos.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: PENDIENTE (J4).

## Contrato y decisiones

`combine(&Combination, &[CaseResults])` entrega `CombinationResults`, con id
propio y `CaseResults` reutilizado (`case` = id de combinación, `factor` = 1).
No reaplica el factor de carga almacenado en cada caso: esos valores ya están
escalados. Los términos del usuario son los únicos coeficientes. No se
introducen factores de ninguna norma. Cada valor se suma en el orden de los
`CaseFactor`, con producto y suma f64 separados, sin FMA ni interpolación.
Cada producto y suma parcial se comprueba para detectar desborde; no se permite
que un desborde intermedio se oculte mediante cancelación posterior.

Se emparejan nodos y miembros por id, independientemente de su orden en la
entrada; la salida se ordena por id. Las acciones de extremo conservan su orden
local. Los diagramas conservan exactamente las posiciones originales. Tamaño
y posición de estaciones deben coincidir, sin tolerancia; no se interpola ni
se unen rejillas distintas. Se comprueban rango [0,1], orden estricto y extremos
0/1. Se permiten resultados exclusivamente nodales, sin miembros; cada entrada
requiere nodos. Los términos de factor cero se validan igual que los demás.

`envelope(&[EnvelopeInput])` admite casos y combinaciones, con `ResultId::Case`
y `ResultId::Combination` para distinguir nombres iguales. Cada GDL nodal,
acción de extremo y resultante de estación tiene máximo y mínimo algebraicos,
con valor e identidad gobernante. Los extremos globales de miembro tienen
además posición x/L. Se vuelven a calcular los extremos de cada entrada desde
sus estaciones, sin confiar en el caché público `MemberResult::extremes`.

Los empates exactos mantienen la **primera entrada en el orden del usuario**,
sin ordenar por nombre ni preferir casos sobre combinaciones. En un mismo
miembro se mantiene la primera estación (menor x/L). Un máximo/mínimo de una
envolvente puede pertenecer a otra entrada en cada componente: no representa
una solución física simultánea ni un caso de equilibrio.

Diagnósticos: ENG-E-0003 por caso ausente, también con factor cero;
ENG-E-0008 por factor/dato no finito o producto/suma desbordados;
ENG-E-0010 por lista vacía, duplicados, tamaños/topología/estaciones incoherentes.
La biblioteca no añade `panic!`, `unwrap` ni `expect`. Se validan todos los casos
referenciados; los casos ajenos a los términos solo se revisan por duplicidad.

## Derivaciones propias

Para una rigidez constante K y apoyos homogéneos, K uᵢ = Fᵢ.
Multiplicando cada ecuación por el factor del usuario αᵢ y sumando:

K (Σ αᵢ uᵢ) = Σ αᵢ Fᵢ.

Las reacciones R = K u − F, las acciones de extremo y las resultantes recuperadas
son operadores lineales de desplazamientos y cargas; por ello se aplica la
misma suma a cada campo, posición y componente. El extremo de la suma es
`max_s Σ αᵢ fᵢ(s)`/`min_s Σ αᵢ fᵢ(s)`, que en general difiere de sumar los
extremos individuales. Un ejemplo de prueba usa dos diagramas opuestos:
su combinación es cero en todas las estaciones aunque cada diagrama aislado
tenga extremos no nulos.

Con desplazamientos prescritos no nulos el problema de cargas es afín:
u(F) = u(0) + L(F). Sumar resultados completos aporta (Σ αᵢ) u(0).
El módulo cumple la suma ponderada de resultados solicitada, incluidos los
prescritos; no sustrae automáticamente el caso sin carga. La equivalencia con
resolver cargas combinadas manteniendo los mismos asentamientos necesita
Σ αᵢ = 1, asentamientos nulos o un tratamiento explícito del aporte u(0).

La envolvente puntual de un campo v es maxᵢ vᵢ/minᵢ vᵢ, almacenando el
argmáx/argmín. Comparaciones estrictas `>`/`<` conservan el primer gobernante
si hay empate, incluso entre +0 y −0. Los extremos globales del miembro son
maxᵢ max_s fᵢ(s) y minᵢ min_s fᵢ(s), conservando también la estación.

Los extremos se limitan a las estaciones disponibles. La combinación de
resultados muestreados no permite deducir nuevas raíces críticas interiores de
campos arbitrarios. Aunque dos casos tengan estaciones críticas individuales
coincidentes, una combinación puede necesitar otras estaciones para el extremo
continuo; no se promete ese cálculo sin recuperación adicional. Se conserva
exactamente el alcance de los diagramas discretos de entrada.

## Fixture analítico y tolerancias

Viga en voladizo horizontal, L=2 m, E=16 Pa, A=2 m², I=2 m⁴, sin liberaciones,
apoyos Ux=Uy=Rz=0. Caso A: Pₓ=48 N en el extremo libre. Caso B: Pᵧ=−48 N
en el extremo libre y carga uniforme qᵧ=−24 N/m. Para cada combinación se
ensambla y resuelve un caso D con cargas αA+βB mediante `StaticLinear` y
`EulerFrame2D`; se recuperan todos los campos con `CaseResults::recover`.

Integrar N/(EA) da uₓ(L)=PₓL/(EA)=3 m. Para flexión, integrar M/(EI) con
uᵧ(0)=θ(0)=0 da:

uᵧ(L)=PᵧL³/(3EI)+qᵧL⁴/(8EI)=−5.5 m;
θ(L)=PᵧL²/(2EI)+qᵧL³/(6EI)=−4 rad.

Estas magnitudes sirven como fixture algebraico del modelo lineal y no como
hipótesis de un diseño físico con pequeños giros. Para A: N=Pₓ constante.
Para B: Vy(x)=Pᵧ+qᵧ(L−x), Mz(x)=Pᵧ(L−x)+qᵧ(L−x)²/2.
Reacciones: Rₓ=−Pₓ, Rᵧ=−Pᵧ−qᵧL y Rz=−PᵧL−qᵧL²/2.
La carga de tramo usa las acciones consistentes ya implementadas por el
frame2d propio. No hay soluciones numéricas ni fuentes de otro motor.

Se comprueban factores [2,0.5], [−0.5,2], [0,−0.5] y [1.3,−0.7].
La prueba binaria usa una variante con qᵧ=0: B tiene uᵧ=−4 m y θ=−3 rad.
Para [2,0.5] y [−0.5,2] se exige igualdad de bits entre la combinación y el caso
D en todos los desplazamientos, reacciones, acciones y estaciones. La misma
comparación con factor axial cero usa igualdad algebraica dentro de tolerancia:
la recuperación puede producir −0 al calcular N=−Ni, frente al +0 de una suma
iniciada en cero. La suma explícita ordenada sí se verifica bit a bit, incluidos
los ceros, factores negativos, cero y factores no binarios.

La variante con carga de tramo qᵧ=−24 usa
128·EPSILON·max(1,|esperado|,escala) para comparar con D y las fórmulas
cerradas. Los GDL nodales usan escala=1; las acciones de extremo usan el máximo
módulo de ambos vectores; las estaciones y sus extremos usan el máximo módulo
del mismo campo en ambos diagramas. En la comparación analítica se usan
48|α|, 96|β| y 144|β| para N, Vy y Mz respectivamente. Así, los residuos en
campos cuyo valor teórico es cero se juzgan respecto a las acciones que los
producen. La variante nodal usa 128·EPSILON·max(1,|esperado|) para factores
no binarios y extremos. Al mezclar cargas nodales y de tramo, el escalado numérico y la
recuperación del solucionador pueden variar unos ULP entre resolver D y
superponer, incluso con factores potencia de dos; la combinación sigue
coincidiendo bit a bit con la suma ordenada de los resultados de entrada.

Las pruebas de envolvente usan distintos gobernantes por GDL, campo y signo,
un pico interior, casos/combinaciones con nombres iguales, combinación que
gobierna, inversión del orden de empates, y empate de estaciones. Se comprueban
estaciones distintas por un ULP, listas vacías, duplicados, topología, tamaños,
NaN/inf, términos nulos inválidos, valores ±MAX, suma/producto desbordados,
cancelación de extremos finitos y subnormales sin panic.

## Verificación de la sesión

Resultados finales de esta sesión, todos con salida 0 y sin red
(`CARGO_NET_OFFLINE=true`):

- `cargo fmt`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, incluidas las 11 pruebas nuevas de combinación
  y envolvente, las 9 de results y las pruebas existentes de engine-loads.
  Las pruebas nativas se ejecutaron con rustc 1.85.1.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK. Se usaron
  cargo y rustc 1.99.0 ya instalados en `/home/box/.cargo/bin`, mediante
  `RUSTC=/home/box/.cargo/bin/rustc /home/box/.cargo/bin/cargo`; la biblioteca
  estándar wasm ya estaba instalada. No se descargó ningún componente.
  Se verifica compilación, no ejecución ni equivalencia numérica en wasm.
- `bash clean-room/check.sh origin/main`: OK, incluyendo los tres archivos
  nuevos mediante `git add -N` sobre una copia temporal del índice en `/tmp`.
  Los objetos temporales también se alojaron en `/tmp`; el índice real quedó
  intacto. `git diff --check` sobre esa copia: OK.

Sin commit ni push. Revisor clean-room: PENDIENTE (J4).
