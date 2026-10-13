# Bitácora clean-room · ENG-MAT-003

- Ticket / PR: ENG-MAT-003, «Acero bilineal con endurecimiento cinemático»; cambios locales, sin commit ni push.
- Autor(es): J2 (agente, Grok Bot); fecha 2026-10-11 (America/Guayaquil). Identificación solicitada para la entrega; sesión de implementación ejecutada por Codex. Sin delegación a otros agentes.
- Rol: implementador y autor de la derivación analítica propia; sin actividad de operador de oráculo ni auditor de similitud.
- Contrato implementado: `uniaxial/1`, kind `steel.bilinear_kinematic`, contrato 1.0 y método 0.1.0.
- Fuentes permitidas usadas: especificación del usuario, contratos internos y código propio del repositorio enumerados abajo. Cita: **derivación propia de plasticidad 1D con endurecimiento cinemático lineal**, documentada en esta nota. Sin libros, páginas inventadas, red ni fuentes externas.
- Derivaciones propias: [retorno 1D](#derivación-propia-del-retorno-1d), [ciclo y energía](#tabla-analítica-del-ciclo) y [fixtures](../crates/engine-laws/tests/steel_bilinear.rs).
- Código de terceros copiado: ninguno. Solo biblioteca estándar y las dependencias locales existentes `engine-core` y `engine-contracts`; sin crate ni dependencias nuevas.
- Validación: analítico; tabla de histéresis, efecto Bauschinger, área de ciclo cerrado, plasticidad perfecta, tangentes por diferencias finitas, conformidad con historia y diagnósticos de entradas inválidas.
- Comparación con oráculo (repo hermano, informativa): no. No se usó ningún motor externo ni se consultó el repo de oráculos.
- Declaración: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'.
- Revisor clean-room: PENDIENTE (J4).

## Documentos internos consultados

Se leyeron `provenance/TEMPLATE.md`, `provenance/ENG-MAT-001.md`,
`docs/CLEAN_ROOM.md`, `UniaxialLaw`, `UniaxialResponse`,
`conformance::check_uniaxial_law`, la ley `Elastic`, sus validadores,
fábrica y pruebas, los manifests y `clean-room/check.sh`.

Este checkout no contiene la ley multilineal ni la bitácora ENG-MAT-002.
Se consultaron mediante `git show` exclusivamente los archivos propios
`crates/engine-laws/src/uniaxial/multilinear.rs` y `uniaxial.rs` de la rama
local `j2/eng-mat-002-multilinear`, para seguir el patrón de
`MultilinearElasticFactory`: módulo privado re-exportado en `uniaxial`,
parámetros tipados, descriptor estático y construcción mediante
`UniaxialFactory`. No se fusionó esa rama ni se alteró el índice Git real.
No se encontró ningún `AGENTS.md` aplicable en este checkout o sus ascendientes.

La declaración solicitada para los tickets ENG-MAT-001, ENG-MAT-002 y
ENG-MAT-003 es la transcrita arriba. Esta bitácora registra la actividad de
la sesión de ENG-MAT-003; no certifica retrospectivamente otras sesiones
ni sustituye las revisiones históricas de los otros tickets.

## Derivación propia del retorno 1D

Se adopta ε = ε_e + ε_p, σ = Eε_e y una superficie trasladable
|σ−α| ≤ fy. E, fy y α se expresan en Pa; ε y ε_p son adimensionales.
No hay endurecimiento isotrópico: fy permanece constante.
La regla de flujo es Δε_p = sΔγ y la evolución cinemática
Δα = HsΔγ, donde s = sign(σ_tr−α) y H ≥ 0.
Para estado inicial nulo, α = Hε_p a lo largo de cualquier trayectoria.

La predicción desde el estado aceptado es σ_tr = E(ε−ε_p),
ξ = σ_tr−α y f = |ξ|−fy. Si f ≤ 0, Δγ = 0 y σ = σ_tr.
Si f > 0, la consistencia en la superficie final exige
s(σ−α') = fy. Sustituyendo σ = σ_tr−EsΔγ y
α' = α+HsΔγ, se obtiene |ξ|−(E+H)Δγ = fy y por tanto
Δγ = f/(E+H), no negativo. Es el retorno implícito exacto en 1D.

Lejos de un vértice, ds/dε = 0 y df/dε = sE. Entonces
dσ/dε = E−E²/(E+H) = EH/(E+H). Definimos el parámetro
b = (dσ/dε)_plástico/E, de modo que H = bE/(1−b) y 0 ≤ b < 1.
La rama elástica tiene tangente E y la plástica bE.
En f = 0 se selecciona la rama elástica; la derivada bilateral no es única.

Al invertir desde σ = α+fy hacia compresión, α y ε_p permanecen
constantes hasta σ = α−fy. El recorrido elástico es exactamente
2fy en tensión y 2fy/E en deformación. Desde compresión ocurre lo
mismo con el signo opuesto. El centro α cambia con la deformación
plástica, produciendo el efecto Bauschinger sin cambiar el ancho de fluencia.

La energía almacenada es ψ = σ²/(2E) + α²/(2H) para H > 0.
Usando α = Hε_p, el segundo término equivale a Hε_p²/2.
El trabajo plástico cumple σ dε_p = d(Hε_p²/2) + fy |dε_p|.
En un ciclo que retorna a la misma ε, σ, ε_p y α, el cambio de ψ es cero,
de modo que ∮σ dε = fy∫|dε_p|. Para H = 0 no hay término cinemático
almacenado; se conserva la misma fórmula de disipación.

## Tabla analítica del ciclo

Fuente y cita de todos los fixtures: derivación propia de esta nota,
sin ejemplos publicados ni resultados de otro motor.
E = 200 Pa, fy = 1 Pa, b = 0,1; H = 200/9 Pa.
En carga plástica positiva desde el origen:
σ = 0,9 + 20ε, ε_p = 0,9(ε−0,005), α = 20(ε−0,005).
El pico ε = 0,02 deja σ = 1,3, ε_p = 0,0135 y α = 0,3.
La descarga elástica termina en ε = 0,01 y σ = −0,7.
Después, σ = 20ε−0,9 hasta ε = −0,02, donde
σ = −1,3, ε_p = −0,0135 y α = −0,3. La recarga elástica
termina en ε = −0,01 y σ = 0,7; después σ = 20ε+0,9.

| ε objetivo | σ (Pa) | α (Pa) | ε_p | Tangente (Pa) |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 0 | 0 | 0 | 200 |
| 0,0025 | 0,5 | 0 | 0 | 200 |
| 0,005 | 1 | 0 | 0 | 200 (f = 0) |
| 0,02 | 1,3 | 0,3 | 0,0135 | 20 |
| 0,0175 | 0,8 | 0,3 | 0,0135 | 200 |
| 0,015 | 0,3 | 0,3 | 0,0135 | 200 |
| 0,0125 | −0,2 | 0,3 | 0,0135 | 200 |
| 0,005 | −0,8 | 0,2 | 0,009 | 20 |
| −0,02 | −1,3 | −0,3 | −0,0135 | 20 |
| −0,0175 | −0,8 | −0,3 | −0,0135 | 200 |
| −0,015 | −0,3 | −0,3 | −0,0135 | 200 |
| −0,0125 | 0,2 | −0,3 | −0,0135 | 200 |
| −0,005 | 0,8 | −0,2 | −0,009 | 20 |
| 0,02 | 1,3 | 0,3 | 0,0135 | 20 |

El ciclo cerrado parte del pico A = (0,02; 1,3), sin incluir la carga
inicial, y recorre B = (0,01; −0,7), C = (−0,02; −1,3),
D = (−0,01; 0,7) y A. Los vectores AB = (−0,01; −2) y
AD = (−0,03; −0,6) dan área |det(AB, AD)| = 0,054 Pa.
También se obtiene 4fy(1−b)(ε_max−fy/E) = 4·1·0,9·0,015.
La integral exacta por lados da −0,003+0,03−0,003+0,03 = 0,054 Pa.
La variación plástica acumulada es 2·(0,0135−(−0,0135)) = 0,054;
al multiplicar por fy coincide con el área disipada (J/m³).

Para b = 0, la carga inicial alcanza σ = fy en ε = fy/E = 0,005
y continúa sobre la meseta σ = 1; ε_p = ε−0,005, α = 0 y tangente 0.
La rama negativa tiene σ = −1 y ε_p = ε+0,005; la descarga/recarga
dentro de la superficie conserva tangente 200 y la historia aceptada.

## Estabilidad numérica, errores e historia

La implementación no forma H, E+H ni fy/E. Utiliza las identidades
EΔγ = (1−b)f, HΔγ = bf, Δγ = ((1−b)f)/E y tangente bE.
Evalúa σ = α'+s fy para evitar cancelar σ_tr−EsΔγ cuando b = 0
y la predicción es mucho mayor que fy. E = 1e300, ε = 1e−300 y fy
enorme dan σ ≈ 1 sin desbordar. También se verifica un b próximo a 1
con H no representable, y E pequeño con fy/E no representable.

`ENG-E-0010` diagnostica E ≤ 0, fy ≤ 0, b fuera de [0,1) e historia
de longitud distinta de dos. `ENG-E-0008` diagnostica entradas no finitas
y desbordamientos en ε−ε_p, σ_tr, ξ, Δγ o los resultados.
f = |ξ|−fy es finito cuando ξ y fy son finitos y fy > 0;
(1−b)f, bf y bE tampoco pueden desbordar en sus rangos validados.
El código de biblioteca no usa `panic`, `unwrap` ni `expect`.

El API no comprueba coherencia física entre dos componentes arbitrarios
de una historia finita suministrada por el llamador; garantiza su longitud
y finitud. Las historias producidas desde el origen satisfacen α = Hε_p
dentro del redondeo. En escalas extremas f64 puede perder incrementos
o no resolver ε−ε_p con precisión suficiente; no se afirma precisión
relativa uniforme ni representación exacta de cualquier historia real.

Toda llamada es pura. Las pruebas repiten respuestas, descartan pruebas,
modifican una copia de la historia de prueba y verifican que la ley y el
estado aceptado conservan sus valores. La aceptación consiste únicamente
en que el llamador asigne `committed = trial.trial_state`.

## Verificación local

Las 11 pruebas nuevas y el doctest del material pasan en nativo, junto con
las pruebas elásticas existentes. Las diferencias finitas centrales usan
h = 1e−7 y muestras lejos de vértices, con historia aceptada fija y error
relativo ≤ 1e−6. El kit existente recorre ciclos aceptando historia
explícitamente para b = 0, 0,1 y 0,8. Los tests no necesitan archivos,
red, hilos ni reloj. La compilación wasm verifica portabilidad, no ejecución
numérica de pruebas en un runtime wasm.

Resultados finales:

- `cargo fmt` y `cargo fmt --check`: satisfactorios.
- `cargo clippy --all-targets -- -D warnings`: satisfactorio.
- `cargo test --workspace`: satisfactorio; incluye las 11 pruebas nuevas y
  el doctest del material.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: satisfactorio.
- `cargo check --workspace --target wasm32-unknown-unknown`: satisfactorio
  con `PATH=/home/box/.cargo/bin:$PATH` y `CARGO_NET_OFFLINE=true`, usando
  el toolchain local Rust 1.99.0 que ya tiene instalado el target.
  El intento inicial con el Rust 1.85.1 predeterminado falló con E0463 por
  ausencia de la biblioteca estándar wasm en ese toolchain. Las demás
  verificaciones anteriores pasaron con Rust 1.85.1. No hubo descargas.
- `bash clean-room/check.sh origin/main`: `Clean-room: OK`, sobre un índice
  y directorio de objetos temporales en `/tmp`, con los tres archivos nuevos
  incluidos mediante intent-to-add. El intento sobre el índice real no veía
  la bitácora nueva sin seguimiento y reclamó provenance; el índice temporal
  permite comprobar también todos los archivos nuevos sin alterar el real.
- `git diff --check`: satisfactorio, también sobre el diff completo del
  índice temporal. El diff contiene únicamente cinco archivos: documentación
  general y re-exportaciones de `engine-laws`, módulo nuevo, tests y esta nota.

No se modificaron manifests, dependencias, contratos ni rutas protegidas.
No hubo commit, push, red ni ejecución de oráculos. Revisión clean-room
independiente: PENDIENTE (J4).
