# Bitácora clean-room · engine-linalg: seguimiento J4 (RHS vacío, límites documentados, orden de errores)

- Ticket / PR: seguimiento de #36/#38 (J0/J4); rama `j2/linalg-empty-rhs`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: backend de álgebra lineal interno (engine-linalg)
- Fuentes permitidas usadas: ninguna externa.
- Derivaciones propias: (a) `solve_in_place` con n=0 retorna sin recorrer columnas (antes no terminaba con B 0×usize::MAX); (b) documentado el límite de rango ~1e300 por columna de b (subdesborde del componente pequeño); (c) `MemoryBudget::solve_dense` valida la forma antes de contabilizar: el orden de errores (forma → desborde → presupuesto ENG-E-0013) ya no depende del presupuesto.
- Código de terceros copiado: ninguno
- Validación: pruebas `empty_factorization_solve_with_huge_rhs_columns_terminates`, `rhs_spanning_more_than_1e300_...`, `error_order_does_not_depend_on_the_budget`.
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)

## ENG-LA-009-FU · pruebas anti-revert de lowest_modes (MAM-70, #75)

- Seguimiento: J4 r19 de #58; rama `j2/eng-la-009-fu`, basada en `main`.
- Autor: Codex · 2026-10-15 (America/Guayaquil). Sin delegación.
- Alcance: cuatro pruebas de integración, una por punto, en
  `crates/engine-linalg/tests/lowest_modes_anti_revert.rs`.
- Fuentes: exclusivamente contratos y código propios de `engine-linalg` y
  derivaciones analíticas propias. Sin red, sin oráculos externos, sin consultar
  ni ejecutar Stabileo/OpenSees ni otras bibliotecas de álgebra de terceros.
- Código de terceros copiado: ninguno. Dependencias nuevas: ninguna.
- Biblioteca: no se encontró un bug; las cuatro defensas ya estaban presentes.
  Las mutaciones fueron temporales y se restauraron byte por byte. No se deja
  diff en `crates/engine-linalg/src/`.
- No se modifican `docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` ni `vendor/**`.
  Los fixtures nuevos usan solo aritmética elemental; no necesitan trascendentales.

### Oráculos analíticos y aislamiento de las regresiones

Para K diagonal y M=I, det(K−λM)=∏(Kᵢᵢ−λ), de modo que las raíces son
las entradas diagonales y los vectores propios son los ejes canónicos. La inercia
de K−σM se obtiene contando los signos de Kᵢᵢ−σ. No se usa el backend como
oráculo de los valores esperados.

1. **Orden ascendente** — `anti_revert_rejects_unsorted_eigenpairs`.
   K=diag(1,2,4), M=I; el caso sano devuelve [1,2,4]. Un backend de prueba
   intercambia las dos primeras raíces **y sus columnas**, entregando [2,1,4].
   Los pares siguen siendo válidos y, al solicitar los tres, Sturm cuenta tres
   porque la última raíz sigue siendo 4. Solo la guarda de orden debe rechazar
   el resultado, con ENG-E-0007 y el diagnóstico «sin ordenar».
2. **Suma de inercia igual al orden** —
   `anti_revert_rejects_inertia_whose_sum_differs_from_order`.
   Para la misma K, σ=2 da diag(−1,0,2), inercia (1,1,1) y conteo 1.
   En `lowest_modes(count=1)`, el desplazamiento queda entre 1 y 2 y la
   inercia correcta es (2,1,0). Se inyectan (0,1,0), (2,1,1),
   (usize::MAX,1,0) y (2,1,usize::MAX): todos conservan el conteo negativo 1,
   pero incumplen la suma. Se exige ENG-E-0007 y «inercia incompatible» tanto
   por `lowest_modes` como por `sturm_count`; se cubren déficit, exceso y
   desborde en cada una de las dos sumas comprobadas.
3. **Margen relativo de σ** —
   `anti_revert_relative_shift_includes_nearby_distinct_root`.
   K=diag(s,s(1+5·10⁻⁸)), M=I, s∈{10⁻¹²,1,10¹²}. El punto medio tiene
   conteo 1: son raíces distintas. Para count=1, σ=s(1+10⁻⁷) incluye las dos
   raíces, por lo que se exige ENG-E-0007 y Sturm=2. El piso
   16·tol_cero≈9.1·10⁻¹³·s es mucho menor que la separación 5·10⁻⁸·s.
   Solicitar ambas raíces sí debe funcionar y conservarlas (error relativo
   <10⁻¹⁴). Las tres escalas comprueban que el margen es relativo.
4. **Puesta a cero de modos rígidos** —
   `anti_revert_rigid_modes_are_exactly_zero_for_both_signs`.
   K=diag(−10⁻¹⁵,+10⁻¹⁵,1), M=I. Los dos primeros valores son no nulos,
   de ambos signos, pero |λ|<128·ε·3≈8.53·10⁻¹⁴. Se comprueba que la salida
   densa cruda conserva esos signos y que `lowest_modes`, para count=2 y 3,
   devuelve exactamente [0,0], cuenta dos rígidos y conserva los vectores
   canónicos y la raíz elástica 1. Un fixture con ceros exactos no detectaría
   la regresión de esta asignación.

### Mutaciones temporales y resultados observados

Se ejecutó primero el fichero nuevo completo: **4 passed, 0 failed**. Para cada
mutación se cambió únicamente `crates/engine-linalg/src/modes.rs`, se ejecutó
la prueba indicada, se restauró el contenido original con `finally` y se volvió
a ejecutar la misma prueba. Comando de cada ejecución aislada:

```bash
cargo test -p engine-linalg --test lowest_modes_anti_revert NOMBRE -- --exact
```

| Mutación | Cambio temporal en biblioteca | Fallo observado | Tras restaurar |
| --- | --- | --- | --- |
| M1 / orden | Eliminar el bloque `if all.eigenvalues.windows(2)...` y su error. | La prueba recibe `Ok` con [2,1,4] en lugar del rechazo; falla `unwrap_err`. Salida 101, 1 failed. | Salida 0, 1 passed. |
| M2 / inercia | Eliminar la validación de `positive.checked_add(negative).and_then(...null) != Some(n)`; conservar solo el retorno de negativos (y `let _ = (positive, null)` para evitar avisos). | Se acepta la inercia (0,1,0) de orden 3 y se devuelve `Ok` con [1]; falla `unwrap_err`. Salida 101, 1 failed. | Salida 0, 1 passed. |
| M3 / margen | Sustituir `(last * (1.0 + 1e-7)).max(last + 16.0 * zero_tol)` por `last + 16.0 * zero_tol`. | En s=10⁻¹² se acepta un solo modo y se devuelve `Ok` con [10⁻¹²]; falla `unwrap_err`. Salida 101, 1 failed. | Salida 0, 1 passed. |
| M4 / ceros | Añadir `*eigenvalue = all.eigenvalues[col]` en la rama rígida, conservando el incremento de `rigid_mode_count`. | La comparación exacta recibe [−10⁻¹⁵,+10⁻¹⁵] frente a [0,0]. Salida 101, 1 failed. | Salida 0, 1 passed. |

Los cuatro fallos fueron aserciones de las pruebas nuevas, sin errores de
compilación. SHA-256 de `src/modes.rs` antes y después de las cuatro mutaciones:
`9451d04d90ba44b5e22b8d93d311596a58bf166b871dfeb7f25117649eafd43e`.
`git diff --exit-code -- crates/engine-linalg/src` terminó con salida 0.
Los `unwrap`/`unwrap_err` pertenecen exclusivamente al código de pruebas;
no se introducen `panic`, `unwrap` ni `expect` en la biblioteca.

### Verificación final de esta sesión

Todas las ejecuciones Cargo usan `CARGO_NET_OFFLINE=true` y el `CARGO_TARGET_DIR` exportado
`/workspace/.cargo-target/mamut-engine`; toolchain instalado Rust/Cargo 1.99.0
(se antepone `/home/box/.cargo/bin` al PATH, pues `/usr/bin/cargo` es 1.85.1).

| Comando | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK, salida 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK, salida 0. |
| `cargo test --workspace --no-fail-fast` | OK, salida 0; 549 pruebas aprobadas (incluidos doctests), 0 fallos, 0 ignoradas; incluye las cuatro pruebas nuevas. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace` | OK, salida 0. |
| `cargo check --workspace --target wasm32-unknown-unknown` | OK, salida 0; comprobación de compilación, sin ejecución wasm. |
| `bash clean-room/check.sh origin/main` | `Clean-room: OK`, salida 0; incluye el fichero nuevo en el diff mediante `git add -N`. |
| `git diff --check` | OK, salida 0. |

## J4 r40 · ENG-LA-009-FU2: desborde que envuelve al orden (MAM-70, #122)

- Rama: `j2/eng-la-009-fu2`, basada en `main`; seguimiento de las pruebas
  anti-revert incorporadas en #118.
- Autor: Codex · 2026-10-15 (America/Guayaquil). Sin delegación.
- Fuentes: únicamente código y contratos propios del repositorio y aritmética
  entera elemental. Sin red, sin oráculos externos ni código de terceros copiado.
  Dependencias nuevas: ninguna.
- Cambio: se añade `(usize::MAX, n + 1, 0)` al test
  `anti_revert_rejects_inertia_whose_sum_differs_from_order`, con `n = k.order()`
  (3). Se exigen ENG-E-0007 y «inercia incompatible» por `lowest_modes` y por
  la API pública `sturm_count`; no se necesita exponer funciones privadas.
- Oráculo: `usize::MAX = 2^w - 1`, por lo que la suma matemática con `n + 1`
  es `2^w + n`, fuera de rango. `checked_add` debe rechazarla; `wrapping_add`
  devuelve `n` y elude la guarda. Los casos de desborde anteriores envuelven
  a valores distintos de `n` y no detectan esta sustitución.

### Mutación temporal comprobada

Con `CARGO_NET_OFFLINE=true`, `PATH=/home/box/.cargo/bin:$PATH` y
`CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6` exportados:

```bash
cargo test -p engine-linalg --test lowest_modes_anti_revert anti_revert_rejects_inertia_whose_sum_differs_from_order -- --exact
```

1. Biblioteca original: salida 0, **1 passed**.
2. Se sustituye temporalmente
   `positive.checked_add(negative).and_then(|v| v.checked_add(null)) != Some(n)`
   por `positive.wrapping_add(negative).wrapping_add(null) != n` en `src/modes.rs`.
   La compilación termina correctamente y el test falla con salida **101**:
   `sturm_count` devuelve `Ok(4)`; falla el `unwrap_err` del test.
   `lowest_modes` alcanza el control posterior de cantidad de modos, cuyo
   diagnóstico tampoco satisface el rechazo por inercia incompatible exigido.
3. Se restaura la biblioteca en un bloque `finally` y se repite el comando:
   salida 0, **1 passed**.

SHA-256 de `src/modes.rs` original y restaurado:
`9451d04d90ba44b5e22b8d93d311596a58bf166b871dfeb7f25117649eafd43e`.
`git diff --exit-code -- crates/engine-linalg/src` termina con salida 0.
No se encontró un bug en la biblioteca ni se deja diff en ella; no se añaden
`panic`, `unwrap` ni `expect` en código de biblioteca. No se modifican
`docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` ni `vendor/**`.

### Verificación final de r40

Todos los comandos Cargo se ejecutan sin red, con las variables exportadas
indicadas arriba y el toolchain instalado 1.99.0.

| Comando | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK, salida 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK, salida 0. |
| `cargo test --workspace` | OK, salida 0; 569 pruebas aprobadas, incluidos doctests, 0 fallos, 0 ignoradas. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace` | OK, salida 0. |
| `cargo check --workspace --target wasm32-unknown-unknown` | OK, salida 0; compilación comprobada, sin ejecución wasm. |
| `bash clean-room/check.sh origin/main` | `Clean-room: OK`, salida 0. |
| `git diff --check` | OK, salida 0. |
