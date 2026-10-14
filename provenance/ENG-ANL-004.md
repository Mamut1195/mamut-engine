# Bitácora clean-room · ENG-ANL-004

- Ticket / PR: ENG-ANL-004 «Espectro de respuesta», rama `j2/eng-anl-004`
  apilada sobre ENG-ANL-002 (#61, sin fusionar), su cadena ENG-MASS-001
  (#55 → ENG-LOAD-001 #51), ENG-ANL-001 (#52) y ENG-LA-009 (#58), sin
  fusionar. Sin commit ni push.
- Autor(es): J2 (agente, Grok Bot) · etiqueta solicitada por el usuario;
  ejecutor Codex basado en GPT-6 · 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias; sin delegación,
  actividad de auditor ni operador de oráculo.
- Contrato implementado: `spectrum`, API `engine_analysis::spectrum::Spectrum::run`
  sobre `LinearSystem` y ENG-ANL-002; tablas SI, respuesta modal máxima,
  reacciones, cortante basal, SRSS/CQC y combinación direccional explícita.
  No adapta el trait genérico `AnalysisMethod` (sink/control/settings).
- Fuentes permitidas usadas: exclusivamente la petición del usuario y las APIs
  y código propios pertinentes del workspace: engine-analysis, engine-assembly,
  engine-loads, engine-linalg, engine-core, engine-contracts, engine-model y
  engine-elements-frame; `provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md` y
  las notas analíticas propias de ENG-ANL-001/002. La fórmula CQC y su
  atribución Wilson–Der Kiureghian–Bayo (1981) proceden de la petición;
  no se consultó ese artículo ni otra publicación. Sin red ni fuentes externas.
- Derivaciones propias: [ecuaciones de esta nota](#derivaciones-propias) y
  [fixtures analíticos](../crates/engine-analysis/tests/spectrum.rs), marcados
  `SOURCE` / `CITATION`.
- Código de terceros copiado: ninguno. Sin dependencias nuevas, internas o externas.
- Validación: exclusivamente analítica, sin valores publicados: edificio de
  cortante de N pisos, correlaciones, identidades de combinación, osciladores
  independientes y barra axial con masa consistente y recuperación de miembro.
- Comparación con oráculo (repo hermano, informativa): no. No se consultó
  ni ejecutó ningún motor externo.
- Declaración: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'.
- Revisor clean-room: PENDIENTE (J4).

## Apilado y declaraciones de los tres tickets

Las bitácoras de ENG-ANL-001, ENG-ANL-002 y ENG-ANL-004 contienen la
misma declaración completa, autor J2 (agente, Grok Bot) y «Revisor clean-room:
PENDIENTE (J4)». Las tres dependencias anteriores ENG-MASS-001, ENG-ANL-001
y ENG-LA-009 también conservan esos campos. No se sustituyen sus derivaciones
ni se afirma que sus PR estén fusionados. Se consume el módulo modal de #61
sin cambiar el solver, la reducción de masa o los contratos de las ramas base.

## Interfaz, unidades y decisiones explícitas

`TabulatedSpectrum::new(pairs, units, damping, interpolation, extrapolation)`
requiere al menos dos pares y una decisión para cada opción. La tabla es
inmutable tras validarse. `SpectrumUnits` declara periodos en segundos o
frecuencias en Hz, siempre con Sa en m/s². Sa es pseudoaceleración espectral
para el amortiguamiento ζ de esa tabla. No se interpreta Sa como múltiplo de g,
no hay conversión por gravedad ni ajuste automático por ζ. Se permiten ζ=0 y
Sa=0 en interpolación lineal; ζ debe ser menor que 1.

Las abscisas originales deben ser estrictamente crecientes. Los periodos
pueden empezar en cero; las frecuencias deben ser positivas. Una tabla en
frecuencia se transforma mediante T=1/f y se invierte su orden: las dos
interpolaciones son SIEMPRE en periodo. No se ordenan entradas inválidas.

- Lineal: a=(T−T₀)/(T₁−T₀), Sa=(1−a)Sa₀+aSa₁.
- Log-log: a=ln(T/T₀)/ln(T₁/T₀), ln Sa=(1−a)ln Sa₀+a ln Sa₁;
  exige T y Sa estrictamente positivos en cada punto. Se usa `ln_1p` en
  intervalos cercanos y diferencias de ln cuando el cociente desbordaría.
- Los valores en abscisas exactas se conservan. `Extrapolation::Error`
  rechaza fuera del intervalo cerrado. `Extrapolation::Plateau`, elegida
  explícitamente por el usuario, conserva Sa del extremo correspondiente;
  incluye T=0 fuera de una tabla log-log. Nunca se extrapola silenciosamente.

`Spectrum::run(system, directions, options)` recibe direcciones no nulas,
normalizadas sin desbordar su norma. Cada `SpectrumDirection` declara una
tabla común a todos los modos o exactamente una tabla por modo, en orden
ascendente de frecuencia. Así se permite ζ común o ζ_n y Sa(T,ζ_n) aportados
por el usuario. No se interpola entre amortiguamientos ni se modifica Sa.
`SpectrumOptions` declara número de modos, MassKind, combinación modal,
combinación direccional y un factor no negativo por dirección; no tiene Default.
No hay mínimos normativos de masa modal ni porcentajes incorporados.

Ejemplo de opciones proporcionadas por un llamador (valores ilustrativos del
usuario, sin significado normativo):

```rust
use engine_analysis::spectrum::{
    DirectionalCombination, Extrapolation, Interpolation, ModalCombination,
    SpectrumOptions, SpectrumUnits, TabulatedSpectrum,
};
use engine_contracts::MassKind;
let table = TabulatedSpectrum::new(
    &[(0.0, 1.0), (10.0, 3.0)],
    SpectrumUnits::PeriodSecondsAccelerationMetersPerSecondSquared,
    0.05,
    Interpolation::Linear,
    Extrapolation::Error,
)?;
let options = SpectrumOptions {
    mode_count: 4,
    mass_kind: MassKind::Consistent,
    modal_combination: ModalCombination::Cqc,
    directional_combination: DirectionalCombination::WeightedSum,
    directional_factors: vec![1.0, 0.3], // Dos direcciones declaradas por el usuario.
};
# Ok::<(), engine_core::EngineError>(())
```

Los resultados conservan tablas, opciones, direcciones unitarias, formas y
participaciones de ENG-ANL-002, ζ y Sa por modo, coordenada modal, fuerzas
equivalentes, desplazamientos libres/globales, reacciones y basal. Las respuestas
modales son firmadas; las combinadas son envolventes por escalar. El sistema
no retiene objetos `Element` ni sus contextos; para miembros, el llamador extrae
u_e de cada desplazamiento global, usa `Element::recover` y combina cada
acción firmada mediante `combine_modal`. Un test demuestra esta ruta con el
axial de un frame existente. No se recuperan miembros sobre el desplazamiento
combinado, que perdería signos modales y correlaciones. No se añade una fachada
results/1 ni se combinan estados no lineales.

## Derivaciones propias

### Respuesta modal, fuerzas y reacciones

Para formas normalizadas a masa, φ_nᵀM_ffφ_n=1, la excitación direccional usa
Γ_n=φ_nᵀM_ff r_f. Del sistema desacoplado y la definición de pseudoaceleración
Sa=ω_n² S_d se obtiene la coordenada máxima firmada:

```text
q_n,max = Γ_n Sa(T_n,ζ_n)/ω_n²
u_n = φ_n q_n,max
F_n = M_global φ_n Γ_n Sa(T_n,ζ_n)
R_n,p = (K_global u_n − F_n)_p
V_n = −Σ_apoyos R_n,traslacional · d̂
```

Las formas y perturbaciones prescritas son cero, incluso con asentamientos.
No se añaden cargas estáticas ni se llama a `Constraints::expand`, que añadiría
los prescritos estáticos. Tampoco se usa M_fp r_p en Γ. M_global φ puede tener
componentes en apoyos con masa consistente: F_p se resta al recuperar R_p.
El cortante basal físico no se reemplaza en general por M*_n Sa, porque ese
último solo proyecta las fuerzas libres en la definición modal utilizada.

Para la barra axial del test, E=12, A=L=1, ρ=6, M_axial=[[2,1],[1,2]],
K_ff=12, M_ff=2, ω²=6, φ_f=1/√2 y Γ=√2. Con Sa=2:
u_f=1/3, F_p=2, F_f=4, R_p=−12/3−2=−6 y V=6 N.
La acción axial recuperada es EA u_f/L=4 N. Un asentamiento de 0.2 m y el
estado previo aceptado u_f=0.7 no intervienen ni cambian durante el análisis.

### Edificio de cortante: sustituto analítico propio, sin ejemplo publicado

**No se reproduce ningún ejemplo publicado de Chopra**: no se dispone de sus
valores ni se consultó su libro. El requisito se verifica con el edificio de
cortante derivado en ENG-ANL-002. Para N pisos iguales, k por entrepiso y m por
piso, q₀=0 y techo libre:

```text
θ_n = (2n−1)π/(2N+1), n=1,…,N
ω_n² = 4(k/m) sin²(θ_n/2)
s_jn = sin(jθ_n)
c_n = √(m Σ_j s_jn²)
φ_jn = s_jn/c_n
Γ_n = m Σ_j s_jn/c_n
M*_n = Γ_n²
u_jn = φ_jn Γ_n Sa_n/ω_n²
F_jn = m φ_jn Γ_n Sa_n
V_n = Σ_j F_jn = Γ_n² Sa_n = M*_n Sa_n
R_base,n = −k u_1n = −V_n
```

Un cambio de signo simultáneo de φ y Γ deja u, F y V iguales. Los tests
calculan estas expresiones directamente con senos, sumas finitas y los parámetros
N=1,4,9; k=17 N/m; m=3 kg. Se usa Sa constante 2.5 m/s² y la tabla lineal
(T,Sa)=(0,1),(100,21), que da Sa(T)=1+0.2T en el intervalo. Γ, q, u, F,
reacción y basal se verifican con tolerancia 3e−11·max(1,|valor esperado|).
La completitud satisface Σ M*_n=Nm, para los N modos de este fixture.

Por DEFINICIÓN de SRSS aplicado al escalar basal:

```text
V_b,SRSS² = Σ_n V_n² = Σ_n (M*_n Sa_n)²
Para Sa_n=Sa constante: V_b,SRSS² = Sa² Σ_n (M*_n)².
```

Esto se deriva aquí y se verifica con todos los modos; no es
Σ_n M*_n Sa_n = V_b,SRSS², expresión dimensionalmente incompatible. La suma
lineal de fuerzas, Σ_n M*_n Sa, para este fixture completo es Nm Sa, pero
no es su combinación SRSS. No se atribuye esta derivación a una publicación.

### CQC y combinación direccional

La fórmula general proporcionada por el usuario es:

```text
r = ω_j/ω_i
ρ_ij = 8√(ζ_iζ_j)(ζ_i+rζ_j)r^(3/2) /
       [(1−r²)² + 4ζ_iζ_j r(1+r²) + 4(ζ_i²+ζ_j²)r²]
```

Con ζ_i=ζ_j=ζ, el denominador se reduce a
(1−r²)²+4ζ²r(1+r)² y el numerador a 8ζ²(1+r)r^(3/2), la expresión
Wilson–Der Kiureghian–Bayo (1981) dada en la petición. Permutar i,j equivale
a r→1/r y dividir ambos polinomios por r⁴; así ρ_ji=ρ_ij. Se evalúa con
r≤1 y productos normalizados por la raíz del denominador, evitando cocientes
grandes y cuadrados subnormales. Se limita al intervalo [0,1] el redondeo final.
Para r=1 y ζ común positivo, numerador=denominador=16ζ² ⇒ ρ=1.
Con ζ_i=ζ_j=0 y r=1 la expresión es 0/0: se declara la convención ρ=1.
Con ambos ζ=0 y r≠1, ρ=0. Con r=1 y ζ distintos:
ρ=2√(ζ_iζ_j)/(ζ_i+ζ_j), cero si solo un modo está amortiguado.

Los tests comparan ambas fórmulas, simetría, límites [0,1], diagonal unitaria,
ζ subnormal y frecuencias extremas. Para ω=(1,1000) rad/s, ζ=0.05 y
R=(3,4), se exige ρ<1e−6 y |CQC/SRSS−1|<1e−6. Para frecuencias próximas
se comprueba correlación >0.9 y los términos cruzados con respuestas opuestas.

```text
SRSS: √Σ_n R_n²
CQC:  √Σ_iΣ_j ρ_ij R_i R_j
Direcciones SRSS: √Σ_d (a_d R_d)²
Direcciones suma ponderada: Σ_d a_d R_d
```

Se escala R por su máximo módulo antes de la forma cuadrática. Solo se admite
un negativo de redondeo de hasta 64ε n² antes de tomar la raíz; una forma más
negativa devuelve ENG-E-0012. Se combinan componentes libres/globales y cada
reacción por separado. Para V se suma primero cada conjunto de reacciones
firmadas por modo y después se combina el escalar. Dos osciladores apoyados
por separado con m=(2,3), k=(18,48), Sa=2 dan V=(4,6): V_SRSS=√52,
mientras la suma de envolventes de apoyos es 10. Un test verifica esta diferencia.
La combinación entre direcciones reúne magnitudes proyectadas en direcciones
distintas; no se interpreta como una nueva dirección ni como equilibrio.

## Errores y límites

- ENG-E-0010: tablas cortas, abscisas sin orden estricto/negativas, Sa negativa,
  f=0, log-log con ceros, ζ fuera de [0,1), consultas T negativas/fuera de
  rango con política Error, listas vacías, modos rígidos, dirección nula,
  cardinalidad de tablas/factores incompatible o factores negativos.
- ENG-E-0008: entrada o resultado no finito, inversión de f que desborda,
  desplazamientos/fuerzas/productos/sumas no representables.
- Se propagan diagnósticos de Modal/LinearSystem, incluidos ENG-E-0006,
  ENG-E-0007, ENG-E-0011 y ENG-E-0012; no se regularizan ni condensan matrices.
- No hay `panic!`, `unwrap` ni `expect` en engine-analysis/src. La indexación
  interna usa dimensiones garantizadas por constructores privados o resultados
  validados de Modal. No se llama al `SystemView::dof_label` con `expect`
  preexistente en engine-assembly.
- Las formas son de vibración libre no amortiguada; CQC usa amortiguamiento
  modal clásico, no matrices de amortiguamiento no proporcionales. Las respuestas
  son máximos estimados, no historias, ni estados simultáneos, ni signos de diseño.
- Se heredan límites y tolerancias del solver modal apilado. No se garantiza
  completitud con modos truncados. El camino modal sigue usando el solver denso.

## Verificación local

Todas las comprobaciones Cargo se ejecutaron con `CARGO_NET_OFFLINE=true`,
sin red, descarga ni instalación de dependencias.

- `cargo fmt` y `cargo fmt --check`: OK, salida 0.
- `cargo clippy --all-targets -- -D warnings`: OK, salida 0.
- `cargo test --workspace`: OK, salida 0; 257 pruebas y doctests, incluidas
  12 pruebas espectrales nuevas. Las pruebas espectrales se ejecutaron primero
  por separado; se corrigió el acceso del test a `StationResult::forces_local`
  antes de la verificación completa.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK, salida 0.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, salida 0,
  con `PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable`, toolchain
  ya instalado que contiene std para ese destino. No se descargó nada.
- `bash clean-room/check.sh origin/main`: OK, salida 0. Los archivos nuevos
  quedaron incluidos mediante `git add -N` exclusivamente en un índice y un
  directorio de objetos temporales bajo `/tmp`, usando los objetos existentes
  como alternativos de solo lectura. No se modificó el índice real ni se
  crearon commits. `git diff --check` con ese índice: OK.
- Búsqueda de `panic!`, `.unwrap(` y `.expect(` en engine-analysis/src:
  sin coincidencias. Las pruebas usan aserciones y unwrap en el fixture,
  siguiendo la convención de los tests analíticos existentes.

Compilación nativa, pruebas y rustdoc: Cargo/rustc 1.85.1 del sistema
(MSRV); Clippy instalado: 0.1.99. El chequeo wasm usó rustc 1.99.0
del toolchain `stable` ya instalado.

El chequeo wasm verifica compilación, no ejecución en un runtime wasm ni
igualdad numérica entre destinos. La revisión clean-room de J4 sigue pendiente.
Registros: `/tmp/eng-anl-004-{fmt,clippy,test,doc,wasm,clean-room,diff-check}.log`.
Sin commit ni push. Revisor clean-room: PENDIENTE (J4).
