# Bitácora clean-room · ENG-MASS-001-FU

- Ticket / PR: issue #71, seguimiento J4 r16 de #55 / ENG-MASS-001; rama `j2/fu-71-suction` desde `main`. Corrección previa a CLI/API.
- Autor(es): Codex gpt-6.1-sol high, 2026-10-14 (America/Guayaquil).
- Rol: especificador e implementador mediante derivaciones propias.
- Contrato implementado: `engine-loads::mass`, conversión fuerza→masa en `mass_from_force`, `masses_from_load_vector`, `masses_from_case` y `masses_from_system_case`. Sin cambios a contratos elementales ni al ensamblado de fuerzas.
- Fuentes permitidas usadas: exclusivamente el ticket del usuario y los contratos, catálogo, documentación y código interno pertinente de mamut-engine. Sin fuentes externas ni consultas de red.
- Derivaciones propias: ecuaciones (1)-(5) de esta bitácora.
- Código de terceros copiado: ninguno. Sin nuevas dependencias.
- Validación: analítica; pruebas en `crates/engine-loads/tests/mass.rs`, incluyendo reversión temporal de la corrección.
- Comparación con oráculo (repo hermano, informativa): no. Solo oráculos analíticos propios derivados a mano.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Contrato corregido

Esta nota sustituye el contrato de valor absoluto descrito históricamente en
la sección (4) de `provenance/ENG-MASS-001.md`. La dirección del usuario define
el sentido de gravedad. Se conserva la suma algebraica de todos los aportes
por nodo antes de proyectar; no se compensan proyecciones entre nodos.
Para la fuerza neta nodal F_n y la dirección normalizada d̂:

(1) q_n = F_n·d̂; si q_n ≥ 0, m_n = q_n·factor/g_usuario.

Si q_n < 0, se rechaza con ENG-E-0010, sin tolerancia, incluso con factor cero.
La comprobación precede al producto por factor para no perder el signo de
proyecciones subnormales. Se valida la finitud de la proyección antes del signo;
no finitos/desbordamientos conservan ENG-E-0008. El cero exacto, incluido -0.0,
es válido y genera masa cero. Una proyección positiva con factor cero da cero.
Se mantienen la validación de g y factor y el rechazo de subdesbordamiento
de masa positiva con factor positivo. Los momentos no generan masa.
Los errores de conversión nodal incluyen `Node`; los adaptadores de caso
conservan `Case`. La conversión escalar no recibe entidades.

## Oráculos analíticos propios

(2) Para d = -Z y g = 10 m/s², Fz = -120 N representa W = 120 N:
q = 120 N y m = W/g = 12 kg (3 kg con factor 0.25).
Fz = +120 N produce q = -120 N y debe rechazarse. También se rechaza el menor
subnormal negativo representable; no se aplica tolerancia ni se toma su módulo.
F = 0 y las dos representaciones de cero producen masa cero.
En nodos distintos, Fz = -120 y +80 N tienen resultante global -40 N,
pero el segundo nodo tiene q = -80 N y se rechaza.

(3) Para d̂ = (3/5, 4/5, 0), F = (3, -4, 0) N da
q = (9-16)/5 = -1.4 N: error. F = (4, -2, 0) N da
q = (12-8)/5 = 0.8 N, m = 0.08 kg con factor 1 y g = 10.
Una componente opuesta no basta para rechazar: decide la proyección neta.
El fixture perpendicular usa F = (4,4,0) y d = (-1,1,0), con productos
simétricos que cancelan exactamente en f64. El fixture anterior (3,4,0)
contra (-4,3,0) tenía un residuo negativo de redondeo tras normalizar; bajo
el contrato estricto ese residuo se rechaza. No se introduce tolerancia.
En un sistema, el nodo 2 tiene Fz nodal = -80 N; aportes elementales de
+40, +80 y +100 N dan netos -40, 0 y +20 N: respectivamente 4 kg, 0 kg y error.
El nodo 1 conserva -120 N, por lo que incluso en el último caso la resultante
global sigue siendo gravitatoria. Esto comprueba la resta antes de proyectar.

(4) M13: un bloque de masa X entre dos nodos [[1,-2],[-2,1]] kg tiene diagonales
positivas, pero r_X = [1,1] da r_Xᵀ M r_X = 1-2-2+1 = -2 kg.
`total_mass` debe rechazar con ENG-E-0010 por el total negativo.

(5) M24: un modelo XY que declara solo Ux y Uy no contiene Uz. La dirección
Z requiere Uz y produce ENG-E-0011 en la medida direccional y los tres
adaptadores nodales, incluso con fuerzas nulas.

El fixture histórico de conversión válida cambia el segundo peso de +80 a
-80 N y su aporte elemental de -40 a +40 N: conserva las masas esperadas de
3 y 2 kg inicialmente y 2.5 y 1 kg tras ensamblar. La succión se cubre mediante
pruebas explícitas de rechazo en las cuatro funciones.
M13 y M24 solo añaden cobertura de comportamientos existentes.

## Verificación de la sesión

Todos los comandos Cargo usan `CARGO_NET_OFFLINE=true` y
`CARGO_TARGET_DIR=/workspace/.cargo-target`. No se modifica `docs/CLEAN_ROOM.md`,
`.github/**` ni `clean-room/**`. Sin push; commit local en español.

- Reversión temporal: se repuso únicamente `crates/engine-loads/src/mass.rs`
  desde `HEAD`, manteniendo las pruebas nuevas, y se ejecutó
  `cargo test -p engine-loads --test mass suction_`. Las cuatro regresiones
  fallaron (0 aprobadas, 4 fallidas, salida Cargo 101) porque la implementación
  anterior devolvía `Ok` en lugar del error. La corrección se restauró en un
  bloque `finally`; con ella las 15 pruebas de masa pasan, también dentro de
  la ejecución completa del workspace. M13/M24 son cobertura adicional de
  contratos existentes y no regresiones del cambio de signo.
- `cargo fmt --check`: OK, salida 0.
- `cargo clippy --all-targets -- -D warnings`: OK, salida 0.
  El primer intento mezcló Cargo/Rust 1.85.1 con Clippy 1.99.0 y falló con
  E0463 por artefactos incompatibles. Se resolvió usando el toolchain 1.99.0
  ya instalado, con `PATH=/home/box/.cargo/bin:$PATH`, sin descargas.
- `cargo test --workspace`: OK, salida 0; 396 pruebas y doctests aprobados,
  0 fallos, 0 ignorados, con Rust 1.85.1. Incluye seis pruebas nuevas: cuatro
  regresiones de succión y las dos comprobaciones M13/M24.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: OK, salida 0,
  con Rust 1.85.1.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, salida 0,
  usando Rust 1.99.0 y el destino ya instalado mediante el mismo ajuste de PATH.
  Verifica compilación wasm, no ejecución en ese destino.
- `bash clean-room/check.sh origin/main`: OK, salida 0; incluye la bitácora
  nueva registrada en el índice. `git diff --check`: OK.

Registros locales: `/tmp/eng-mass-001-fu-{revert,fmt,clippy,test,doc,wasm,clean-room}.log`.
No se consultó ni copió código de motores externos. Sin `panic`, `unwrap` ni
`expect` en el código de biblioteca modificado. Revisión clean-room pendiente de J4.
