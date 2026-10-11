# ADR-002: evidencia de verificación (V1, V2, V4, V5, V6 parcial)

- **Relacionado con:** [ADR-002](ADR-002-algebra-lineal.md) (provisional: faer principal, nalgebra opcional, sprs-ldl excluido)
- **Fecha de las mediciones:** 2026-10-10/11 (America/Guayaquil)
- **Estado:** evidencia medida en la caja de desarrollo; **no** sustituye la aprobación de Jonathan ni la CI.
- **Código reproducible:** [`spikes/adr-002-la/`](../../spikes/adr-002-la/) (paquete aparte con su propio `[workspace]`; no es miembro del workspace del motor ni añade dependencias a este).
- **Propuesta de `cargo-deny`:** [`deny.toml`](../../deny.toml) en la raíz.

Las validaciones son solo autoconsistentes (residual relativo y comparación entre implementaciones); no se usó OpenSees, CalculiX ni otro motor como oráculo.

## Entorno

| Elemento | Valor |
|---|---|
| SO / CPU | Linux 6.12.94, Intel Xeon (virtualizado), 8 hilos disponibles |
| Rust | 1.85.1 (MSRV del workspace) y stable 1.99.0 (cargo 1.99.0), vía rustup 1.29.1 en `~/.rustup` del usuario de la caja |
| Target wasm | `wasm32-unknown-unknown` (`rustup target add` en ambas toolchains) |
| Node | v22.23.2 (runner wasm: `WebAssembly.instantiate`, sin imports; no hay `wasm-bindgen-test`/wasmtime instalados) |
| cargo-deny | 0.20.2 |
| Candidatos | faer 0.24.4 (`default-features = false`, `features = ["std", "sparse-linalg"]`), nalgebra 0.35.0 (`default-features = false`, `features = ["std"]`) |
| Perfil release | `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`; sin `-C target-cpu`, sin `target-feature=+fma`, sin `wasm-opt` |

## 1. Compilación wasm

| Comprobación | Comando | Resultado |
|---|---|---|
| Crates actuales del motor (`engine-core`, `engine-model`, `engine-contracts`) | `cargo build --workspace --target wasm32-unknown-unknown` (1.85.1) | **OK** |
| Spike con faer | `cargo +stable build --release --lib --target wasm32-unknown-unknown --no-default-features --features with-faer` | **OK** |
| Spike con nalgebra | idem con `--features with-na` | **OK** |
| faer en MSRV 1.85.1 (nativo) | `cargo +1.85.1 build --release --lib --no-default-features --features with-faer` | **OK** |
| nalgebra en 1.85.1 | `cargo +1.85.1 build` con nalgebra | **Falla** (coherente con el MSRV 1.89 del ADR) |
| Imports del `.wasm` | `WebAssembly.Module.imports` | **0** imports en ambos; se instancia en Node sin glue |
| Dependencias `rayon`, `*-sys`, `getrandom`, BLAS/LAPACK/ARPACK, `sprs-ldl` en el árbol wasm de faer | `cargo tree --target wasm32-unknown-unknown -e normal --no-default-features --features with-faer` | **Ninguna** (111 líneas de árbol) |

Con la feature opcional `rayon` de faer el árbol sí incorpora `rayon`/`rayon-core`; el perfil propuesto no la activa.

## 2. Ejecución wasm en Node

Problema: factorizar A = B·Bᵀ/n + n·I (SPD, B generada con un LCG fijo, RHS `1 + (i mod 7)`), n = 50, 200, 1000. Residual relativo ‖Ax−b‖/‖b‖ (norma 2).

| n | faer residual (Cholesky) | nalgebra residual (Cholesky) |
|---|---|---|
| 50 | 4.31e-16 | 3.70e-16 |
| 200 | 6.20e-16 | 7.10e-16 |
| 1000 | 1.46e-15 | 1.58e-15 |

Todos < 1e-12 (umbral V2). Ejecuta en Node: **cumple** para Cholesky. LU también se ejecuta en Node, pero allí solo se comparó su hash con el nativo (§4); sus residuales (≤ 1.80e-15) son los del benchmark nativo (§3, `bench`), no medidos en wasm.

Tamaño del `.wasm` (release, sin `wasm-opt`, incluye el generador de datos del spike, 24 068 bytes en la base sin backend):

| Artefacto | Bytes | gzip -9 |
|---|---|---|
| base (sin backend) | 24 068 | no medido |
| faer 0.24.4 (Cholesky+LU+residual) | 217 750 | 63 824 |
| nalgebra 0.35.0 (Cholesky+LU+residual) | 34 638 | 13 247 |

El objetivo propuesto de ≤ 1 MB gzip del proyecto queda holgado en esta configuración mínima; el presupuesto sigue **PENDIENTE** de aprobación y no incluye ensamblaje disperso, LDLᵀ ni autovalores (**PENDIENTE**: tamaño con `sparse-linalg` realmente usado y con `wasm-opt`).

Tiempos en Node (mediana de 3–20 repeticiones tras calentamiento; el tiempo incluye generar la matriz, que se resta medido aparte, así que **los valores son solo orientativos** y a n ≤ 50 quedan bajo la resolución): n = 200: Cholesky faer ≈ 0.9 ms, nalgebra ≈ 0.4 ms; n = 1000: Cholesky faer ≈ 105 ms, nalgebra ≈ 53 ms; LU n = 1000: faer ≈ 232 ms, nalgebra ≈ 313 ms (ruido ±50 ms por la resta). Sin SIMD wasm (no se activó `simd128`), faer pierde su ventaja nativa.

## 3. Rendimiento nativo (V4 parcial)

Comando: `cargo +stable build --release && ./target/release/bench` (un solo hilo; sin `rayon`). Mediana de 500/50/5 repeticiones (n = 50/200/1000) tras una de calentamiento; incluye copiar a la estructura de cada librería y resolver con un RHS.

| n | operación | faer (ms) | nalgebra (ms) | razón nalgebra/faer |
|---|---|---|---|---|
| 50 | Cholesky | 0.0109 | 0.0088 | 0.81 |
| 50 | LU | 0.0133 | 0.0117 | 0.88 |
| 200 | Cholesky | 0.1850 | 0.3446 | 1.86 |
| 200 | LU | 0.2276 | 0.6303 | 2.77 |
| 1000 | Cholesky | 10.36 | 47.75 | 4.61 |
| 1000 | LU | 14.54 | 106.63 | 7.33 |

La dispersión entre repeticiones no se publica en detalle (**PENDIENTE**: desviación, memoria máxima, matrices dispersas/ensamblaje, tiempos separados simbólico/numérico). Una sola máquina virtualizada: indicativo, no presupuesto.

## 4. Determinismo (V6 parcial)

Se compara el hash FNV-1a de los bits de la solución (f64 little-endian) de las 4 combinaciones (faer/nalgebra × Cholesky/LU) en n = 50, 200, 1000.

| Prueba | Resultado |
|---|---|
| 5 ejecuciones nativas, sin `rayon`, mismo binario | **Idénticas bit a bit** (12/12 hashes) |
| Anclaje a 1 CPU (`taskset -c 0`) vs. 8 CPU, sin `rayon` | **Idénticas** (faer sin `rayon` es secuencial) |
| Build con feature `rayon` de faer y `Par::Seq` | **Idéntico** al build sin `rayon` |
| `Par::rayon(k)`, k = 1, 2, 4, 8, n = 50 y 200 | Idénticas entre sí |
| `Par::rayon(k)`, n = 1000 (faer, Cholesky y LU) | **Difieren con k y respecto a `Par::Seq`**; hash distinto para cada k ∈ {1, 2, 4, 8} |
| Mismo `k` repetido 10 veces (k = 4 y k = 8) | Repetible: 10/10 iguales |
| nalgebra (n = 50, 200, 1000), cualquier configuración | Idéntico siempre (no usa hilos) |
| Nativo vs. wasm (Node) | nalgebra **idéntico bit a bit** en los 6 casos. faer idéntico en n = 50 (ambos) y LU n = 200; **difiere** en Cholesky n = 200 y en ambos n = 1000 (SIMD/orden de reducción nativo vs. wasm sin SIMD) |

Conclusiones para el ADR: (a) `rayon` debe seguir desactivado en V1 (confirma la decisión 1); con él, el resultado depende del número de hilos para n = 1000; (b) la igualdad bit a bit nativo/wasm **no** se puede prometer con faer, aunque los residuales coinciden en orden de magnitud; (c) nalgebra es el único que, aquí, da igualdad entre destinos, a costa de rendimiento. El criterio de ENG-CORE-010 debería ser tolerancia justificada para faer, no bits. **PENDIENTE:** `simd128` en wasm, FMA, autovectores, LDLᵀ/AMD, matrices dispersas.

## 5. Licencias, fuentes y avisos (V1 parcial, `cargo-deny`)

Comandos (desde `spikes/adr-002-la/`, con `../../deny.toml` copiado o `--config`):

```
cargo-deny check
cargo-deny --all-features check
```

Resultado: `advisories ok, bans ok, licenses ok, sources ok`, con ambos candidatos y con la feature `rayon`. Licencias en el grafo de ambos (por crate, expresiones SPDX): MIT (55), Apache-2.0 (29), Unicode-3.0 (1, `unicode-ident`), Zlib (3: `bytemuck`, `safe_arch`, `wide`); ninguna LGPL/GPL.

Observaciones:

- El `Cargo.toml` del spike declara `license = "MIT"` solo para que `cargo-deny` evalúe el grafo; no es una decisión de licenciamiento del proyecto.
- **RUSTSEC-2024-0436 (`paste` sin mantenimiento):** `paste 1.0.15` entra por `gemm` ← faer; es una macro procedural de compilación, sin vulnerabilidad registrada. En `deny.toml` se ignora con motivo explícito; revisar en cada actualización de faer.
- Con el `deny.toml` propuesto en el workspace del motor, `cargo deny check licenses` marca como **sin licencia** a `engine-core`, `engine-model` y `engine-contracts` (sus manifests no declaran `license`) y avisa de comodines de rutas (resuelto con `allow-wildcard-paths = true`). Declarar `license`/`publish = false` es decisión de gobernanza (ENG-GOV-004; no hay texto legal en el repositorio) y no se toca aquí; hasta entonces el paso de CI debe ejecutarse sobre el spike o esperar esa decisión. Hoy el workspace no tiene dependencias externas.
- Los identificadores permitidos en `deny.toml` que no aparecen en el grafo actual (BSD-2/3, ISC, CC0, 0BSD, Apache-2.0 WITH LLVM-exception, Unicode-DFS-2016) generan solo avisos `license-not-encountered`; se mantienen como lista permitida de la política.
- La base de avisos RustSec se descargó de red en la ejecución; en CI debe permitirse acceso o fijarse `db-path`.

### Paso de CI propuesto (no aplicado)

`.github/workflows` es archivo protegido y la facturación de Actions está bloqueada; este PR no lo modifica. Propuesta para cuando se reactive:

```yaml
- uses: EmbarkStudios/cargo-deny-action@v2   # fijar a un SHA verificado
  with:
    command: check
    arguments: --all-features
```

más una matriz `cargo build --workspace --target wasm32-unknown-unknown` con Rust 1.85.1 y la prueba Node de `run.mjs`.

## 6. Estado de los pendientes del ADR-002

| Pendiente | Estado tras esta evidencia |
|---|---|
| V1 licencias del grafo y `cargo-deny` | **Cerrado para faer y nalgebra** (spike); MSRV efectivo: faer compila con 1.85.1, nalgebra no; scirs2-sparse/sprs/nalgebra-sparse **PENDIENTE** |
| V2 ejecución nativa/wasm en Node | **Cerrado para Cholesky/LU densos**; ensamblaje por tripletas, SPD/indefinido disperso, singularidad, varios RHS **PENDIENTE**; CI **PENDIENTE** |
| V3 APIs, inercia, autovalores | **PENDIENTE** (no abordado) |
| V4 rendimiento | **Parcial** (denso, un hilo, nativo + wasm orientativo); disperso/memoria **PENDIENTE** |
| V5 tamaño wasm | **Parcial** (denso mínimo); con `wasm-opt`, disperso y LDLᵀ **PENDIENTE** |
| V6 determinismo | **Parcial** (ver §4); autovectores y `simd128`/FMA **PENDIENTE** |
| V7 mantenimiento | **PENDIENTE** |
| Aprobación de Jonathan | **PENDIENTE** |

## Cómo reproducir

```
rustup toolchain install 1.85.1 stable && rustup target add wasm32-unknown-unknown --toolchain 1.85.1
rustup target add wasm32-unknown-unknown --toolchain stable
cargo install cargo-deny --locked
cd spikes/adr-002-la
cargo +stable build --release && ./target/release/bench && ./target/release/det
cargo +stable build --release --features rayon --bin det --target-dir target-rayon && ./target-rayon/release/det 4
for f in with-faer with-na; do cargo +stable build --release --lib --target wasm32-unknown-unknown --no-default-features --features $f --target-dir target-w-$f; done
node run.mjs target-w-with-faer/wasm32-unknown-unknown/release/spike_la.wasm faer
node run.mjs target-w-with-na/wasm32-unknown-unknown/release/spike_la.wasm na
cargo-deny check
```
