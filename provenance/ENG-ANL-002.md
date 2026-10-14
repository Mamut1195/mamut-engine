# Bitácora clean-room · ENG-ANL-002

## Registro de ampliación ENG-VAL-001 (2026-10-11)

- Ticket / PR: ENG-ANL-002 (#61), validación modal desde ENG-VAL-001;
  ampliación de tests, sin modificar el análisis modal.
- Autor(es): J2 (agente, Grok Bot), etiqueta solicitada; ejecutor Codex basado
  en GPT-6 · 2026-10-11 (America/Guayaquil).
- Rol: implementador de fixtures y autor de derivaciones propias, sin delegación.
- Contrato implementado: validación de `Modal::run` con APIs públicas.
- Fuentes permitidas usadas: petición, APIs/pruebas propias del workspace,
  `provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md`; sin red ni fuentes externas.
- Derivaciones propias: [ENG-VAL-001, D1–D4](ENG-VAL-001.md#ampliación-dinámica-y-2º-orden).
- Código de terceros copiado: ninguno; sin dependencias nuevas.
- Validación: analítica, 12 fixtures nuevos; ver recuento y verificaciones en
  ENG-VAL-001. Sin ejemplos publicados ni citas de libro inventadas.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: PENDIENTE (J4).

## Registro previo del ticket

- Ticket / PR: ENG-ANL-002 «Análisis modal», rama `j2/eng-anl-002` apilada
  sobre ENG-MASS-001 (#55 → #51, sin fusionar), ENG-ANL-001 (#52, sin
  fusionar) y la parte implementable de ENG-LA-009 (#58, sin fusionar).
  Sin commit ni push.
- Autor(es): J2 (agente, Grok Bot) · etiqueta solicitada por el usuario;
  ejecutor Codex basado en GPT-6 · 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias, sin delegación.
- Contrato implementado: `modal`, API `engine_analysis::modal::Modal::run`
  sobre `LinearSystem`; resultados modales y participación direccional.
  No adapta el trait genérico `AnalysisMethod` (sink/control/settings).
- Fuentes permitidas usadas: exclusivamente la petición del usuario y las APIs
  y código propios pertinentes del workspace: engine-analysis, engine-assembly,
  engine-loads, engine-linalg, engine-core, engine-contracts, engine-model y
  engine-elements-frame; `provenance/TEMPLATE.md` y `docs/CLEAN_ROOM.md`.
  Sin red, bibliografía externa, resultados publicados ni oráculos.
- Derivaciones propias: [ecuaciones y límites de esta nota](#derivaciones-propias)
  y [fixtures analíticos](../crates/engine-analysis/tests/modal.rs), marcados
  `SOURCE` / `CITATION`.
- Código de terceros copiado: ninguno. Una dependencia interna nueva por path:
  engine-loads; sin dependencias externas nuevas.
- Validación: exclusivamente analítica: edificio de cortante, vigas
  Euler–Bernoulli, identidades de energía, completitud y rotación.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'.
- Revisor clean-room: PENDIENTE (J4).

## Apilado y declaraciones anteriores

Los tres tickets previos del contexto son ENG-MASS-001, ENG-ANL-001 y
ENG-LA-009. Sus bitácoras existentes contienen la declaración completa arriba,
la autoría J2 (agente, Grok Bot) y «Revisor clean-room: PENDIENTE (J4)».
Esta sesión no sustituye sus derivaciones ni afirma que sus PR estén fusionados.
ENG-MASS-001 (#55) depende a su vez de ENG-LOAD-001 (#51). El módulo modal
consume esas interfaces apiladas y la entrega parcial #58, sin implementar
la reducción pendiente de GDL sin masa ni un solver nuevo.

## Interfaz y decisiones

`Modal::run(system, count, kind, directions)` recibe count, MassKind y una
lista no vacía de direcciones explícitas. Cada dirección se normaliza con
`engine_loads::mass::rigid_translation`. Los giros de r son cero. Se conserva
el orden de las direcciones; se pueden repetir o invertir sin error.
No hay factores normativos, gravedad implícita ni umbral de masa exigible.

`LinearSystem::lowest_modes(kind, count)` usa el backend del sistema con las
matrices K_ff y M_ff que este ya redujo de K global y `global_mass(kind)`.
No se duplican ni reensamblan masas. No cambia estados aceptados/de prueba.
El módulo normaliza de nuevo la forma con la masa elegida antes de evaluar
participación. No usa `Constraints::expand`, pues incluiría asentamientos:
las perturbaciones modales prescritas son siempre cero.

Los resultados incluyen λ, ω, f, T, formas libres y globales, Γ, M*, sumas
acumuladas y fracciones individuales/acumuladas. Un λ numéricamente nulo
se representa con ω=f=0 y T=None (periodo infinito), sin generar infinito
IEEE-754. Se conserva el conteo de modos rígidos o posibles mecanismos.
Una dirección con masa global cero tiene fracción None; no se divide por cero.
Count=0 conserva la semántica de lowest_modes: no exige M definida positiva
ni factoriza, pero valida direcciones y devuelve sus totales con sumas cero.

## Derivaciones propias

### Problema modal, participación y completitud

De M_ff q̈+K_ff q=0 y q=φ exp(iωt) resulta K_ffφ=ω²M_ffφ.
Para M_ff definida positiva, se toma φ/√(φᵀM_ffφ), de modo que φᵀM_ffφ=1.
Entonces ω=√λ, f=ω/(2π) y T=2π/ω si λ>0.

La proyección de la excitación traslacional r_f en cada forma da
Γ_n=φ_nᵀM_ff r_f/(φ_nᵀM_ffφ_n). Con normalización a masa,
M*_n=(φ_nᵀM_ff r_f)². Una base modal completa Φ satisface ΦᵀM_ffΦ=I;
como Φ es cuadrada e invertible, ΦΦᵀ=M_ff⁻¹. Así:

Σ_n M*_n = r_fᵀ M_ff ΦΦᵀ M_ff r_f = r_fᵀ M_ff r_f.

Esta identidad es para todos los n GDL libres con M_ff positiva, sin eliminar
GDL de masa nula. Se prueba con todos los modos del edificio de 1, 4 y 9 pisos
(ambos tipos de masa) y todos los modos axiales/flexionales de las dos vigas.
No se afirma completitud de un espectro truncado.

La masa global de la dirección se calcula mediante `engine_loads::mass::total_mass`,
equivalente a `system_total_mass`, sobre r completo y M global: rᵀMr.
Es el denominador de las fracciones solicitadas. Los apoyos pueden tener masa
o acoplamientos consistentes: en ese caso rᵀMr no tiene por qué coincidir con
r_fᵀM_ff r_f. Se exponen `total_mass` y `free_mass` para hacer esta diferencia
explícita. Los tests de completitud fijan a cero las componentes prescritas
de r y evalúan el producto independiente con M global; así obtienen exactamente
r_fᵀM_ff r_f. El edificio usa masa solo en pisos libres y alcanza fracción 1.
Las vigas y una barra axial restringida comprueban una fracción final menor
que 1 sin modificar artificialmente la masa del denominador.

### Edificio de cortante de N pisos iguales, derivación propia

Sea q_0=0 en la base, masa m por piso y rigidez k por entrepiso.
La energía V=½k Σ_{j=1}^N(q_j−q_{j−1})² produce K con diagonales 2k
hasta el piso N−1, última diagonal k y vecinas −k. M=mI.
Para φ_j=sin(jθ), la ecuación interior es:

−φ_{j−1}+2φ_j−φ_{j+1}=2(1−cosθ)φ_j=4sin²(θ/2)φ_j.

La última ecuación es φ_N−φ_{N−1}=(λm/k)φ_N; se puede escribir como
la interior introduciendo φ_{N+1}=φ_N. La condición superior exige:

sin((N+1)θ)−sin(Nθ)=2cos((2N+1)θ/2)sin(θ/2)=0.

Se excluye θ=0 (forma nula con base fija). Las N raíces distintas son
θ_n=(2n−1)π/(2N+1), n=1,…,N; por tanto:

ω_n=2√(k/m) sin((2n−1)π/[2(2N+1)]),
φ_{j,n}=sin(j(2n−1)π/(2N+1)).

Se divide cada forma por √(m Σ_j φ_{j,n}²). La participación esperada es
m Σ_j φ_{j,n}/√(m Σ_j φ_{j,n}²), con el signo elegido por el backend;
la masa efectiva es su cuadrado. No se usan cifras publicadas. Los tests
usan k=17, m=3, N=1,4,9; tolerancias ~3e−12 relativas para frecuencias,
3e−12 absolutas para formas/proyecciones y 3e−11 para masas.

### Vigas Euler–Bernoulli, derivación propia

EI w''''+ρA ẅ=0 y w=W exp(iωt) dan W''''=β⁴W,
β⁴=ρAω²/EI. Su solución es combinación de sin, cos, sinh y cosh.

Simplemente apoyada: W=W''=0 en x=0,L elimina cos/cosh en el primero
y obliga sin(βL)=0 en el segundo: β_nL=nπ.

Voladizo: W=W'=0 en el origen da
W=a(cosh βx−cos βx)+b(sinh βx−sin βx).
W''=W'''=0 en L deja el sistema con filas
[cosh βL+cos βL, sinh βL+sin βL] y
[sinh βL−sin βL, cosh βL+cos βL].
Su determinante es 2(1+cos βL cosh βL), luego
cos βL cosh βL+1=0. Los tests obtienen las primeras tres raíces por bisección
propia en (0.1,π), (π+0.1,2π), (2π+0.1,3π), 55 iteraciones; no copian
aproximaciones de raíces de otra fuente.

En ambos casos ω_n=(β_nL)²√(EI/(ρA))/L². Se usa el frame2d existente,
masa consistente, E=ρ=A=L=1, I=10⁻⁴. Esto separa los tres primeros modos
flexionales de los axiales sin suprimir los GDL axiales ni introducir inercia
rotatoria/deformación de corte. Se refinan 4 → 12 elementos: los errores
relativos de las tres primeras ω deben caer por un factor >10 y quedar <3e−4.
No se promete exactitud continua con malla finita.

### Ortogonalidad, rotación y casos límite

La simetría de K y M da (λ_i−λ_j)φ_iᵀMφ_j=0; una base M-ortonormal
satisface ΦᵀKΦ=diag(λ). Se verifican ambas identidades, también para todos
los modos de las vigas, con tolerancia 2e−9 escalada por λ para K.

Una rotación ortogonal Q transforma traslaciones de φ y r y deja Rz igual;
K'=QKQᵀ, M'=QMQᵀ conservan λ, φᵀMφ, φᵀMr y M*. Se rota conjuntamente
cada viga, sus apoyos (ambas traslaciones restringidas) y la dirección en
0.7 rad. Se comparan frecuencias, masas, |Γ| y formas transformadas, admitiendo
el signo arbitrario de cada modo; tolerancias entre 2e−10 y 3e−9.

Un único GDL axial libre con E=ρ=A=L=1 tiene K_ff=1,
M_ff=1/2 concentrada o 1/3 consistente: λ=2 o 3 y M*=M_ff.
El mismo test aplica un asentamiento de 0.2 a la base y acepta previamente
u_f=0.7: φ en el apoyo sigue cero y los estados permanecen intactos.

## Errores y límites

- ENG-E-0010: direcciones vacías/nulas, count>n y restricciones numéricas de
  lowest_modes (masa indefinida, K indefinida). No se regularizan matrices.
- ENG-E-0008: entradas o resultados no finitos/desbordados, incluidos Γ,
  productos de masa, sumas, cocientes y periodos.
- ENG-E-0011: componentes de traslación ausentes para la dirección pedida.
- ENG-E-0006: M_ff singular; se conserva código/mensaje del solver y se añade
  nodo/GDL global si existe entidad con índice reducido. Sin índice se conserva
  el diagnóstico original; índice inexistente da ENG-E-0012. En el frame con
  masa concentrada la rotación libre sin masa señala nodo 2, Rz, global 5.
  La condensación/reducción de esos GDL sigue pendiente en ENG-LA-009/ASM.
- ENG-E-0007: falta de convergencia, espectro incompleto o corte de multiplicidad;
  se propaga sin devolver menos modos. Se prueba un corte de dos modos nulos.
- ENG-E-0012: espectro incompatible, negativo/desordenado o forma sin norma
  positiva devuelta por un backend que incumple el contrato.
- No hay `panic!`, `unwrap` ni `expect` en la nueva biblioteca. No se cambia el
  `expect` preexistente del adaptador `SystemView::dof_label` de engine-assembly;
  el análisis modal no llama a esa ruta.
- Se heredan el camino denso O(n²) de memoria y las tolerancias numéricas de
  Cholesky/Jacobi/Sturm de ENG-LA-009. Un corte dentro de modos múltiples se
  rechaza y exige pedir el grupo completo. No se garantiza precisión de
  proyecciones subnormales o problemas extremadamente mal condicionados.

## Verificación de la sesión

Todas las comprobaciones Cargo se ejecutaron con `CARGO_NET_OFFLINE=true`.

- `cargo fmt`: OK, salida 0.
- `cargo clippy --all-targets -- -D warnings`: OK, salida 0. Se corrigió una
  iteración por índice del nuevo test señalada por Clippy antes de repetirlo.
- `cargo test --workspace`: OK, salida 0; 245 pruebas y doctests, incluidos
  los 6 tests modales nuevos. Se repitió después del ajuste del test.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK, salida 0.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, salida 0,
  con `PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable`.
- `bash clean-room/check.sh origin/main`: OK, salida 0. Incluye los tres
  archivos nuevos mediante `git add -N` en un índice y objetos temporales
  bajo `/tmp`, sin modificar índice ni objetos reales del repositorio.
  `git diff --check` sobre ese índice: OK.

Compilación nativa, pruebas y rustdoc usaron rustc 1.85.1 (MSRV); Clippy
instalado es 0.1.99. El intento wasm con el compilador del sistema falló por
falta de std para ese destino. Se utilizó el toolchain `stable` ya instalado,
rustc 1.99.0, que sí tiene el destino wasm. No se instaló ni descargó nada.
El chequeo wasm verifica compilación, no ejecución de pruebas ni igualdad
numérica entre destinos. Una primera preparación del índice temporal usó por
error el directorio de objetos del worktree; se corrigió al directorio común
de Git y se ejecutó el chequeo correctamente, siempre con escritura en `/tmp`.

Registros: `/tmp/eng-anl-002-{fmt,clippy,test,doc,wasm-stable,clean-room}.log`;
el intento wasm inicial queda en `/tmp/eng-anl-002-wasm.log`.
Sin commit ni push. Revisor clean-room: PENDIENTE (J4).
