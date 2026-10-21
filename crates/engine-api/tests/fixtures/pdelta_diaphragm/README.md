# E3-pdelta: referencia independiente (MAM-860)

La referencia canónica y sus datos están junto al fixture VAL-032:
[`e3_pdelta_reference.py`](../../../../../docs/validation-report/fixtures/val032/_ref/e3_pdelta_reference.py)
y [`e3_pdelta_reference.json`](../../../../../docs/validation-report/fixtures/val032/_ref/e3_pdelta_reference.json).
`reference.py` conserva el comando de generación anterior y ejecuta ese script.
Se elimina la copia local del JSON para mantener una sola fuente de referencia.

El script propio lee `E3-v1.1.3.json`, sin importar ni ejecutar el Engine.
Ensambla Euler 3D y diafragmas XY exactos desde la energía elástica y
`N ∫ H′_a H′_b dx` con Hermite cúbico. `N=EA/L (u_g,b,x−u_g,a,x)` es positivo
en tracción y queda congelado en la solución lineal de gravedad. La solución
lateral directa es `u_PD=T[Tᵀ(K+Kg)T]⁻¹TᵀF_EX`; no suma desplazamientos de G.
Las columnas usan 1, 8 y 16 elementos por entrepiso; se condensan los nudos
interiores sin carga, conservando exactamente el sistema refinado. Las vigas
mantienen un elemento y también incluyen su Kg. La solución lineal es
invariante al refinamiento (se comprueba la condensación elástica a 1e-10).

La matriz geométrica es la estándar `N/(30L)` de segundo orden (McGuire,
Gallagher y Ziemian, *Matrix Structural Analysis*, 2.ª ed., 2000, cap. 9).
El script documenta su derivación energética propia; no copia código del libro.
NumPy preinstalado se usa como dependencia numérica BSD-3-Clause, sin consultar
su implementación. La referencia no consume respuestas del Engine ni de
otros programas de análisis.

Desde la raíz del repositorio, offline:

```sh
OPENBLAS_NUM_THREADS=1 PIP_NO_INDEX=1 python3 crates/engine-api/tests/fixtures/pdelta_diaphragm/reference.py
PIP_NO_INDEX=1 python3 crates/engine-api/tests/fixtures/pdelta_diaphragm/reference.py --selftest
export PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine-e3crit
cargo test -p engine-api --test pdelta_diaphragm --offline -- --nocapture
```

`gen.py` de VAL-032 también regenera los datos y registra ambos hashes en
`index.json`. Rust consume solo el JSON versionado, verifica los hashes del
fixture y script y ejecuta E3-lineal/EX y E3-pdelta. Para cada piso reconstruye
Ux del CM desde el maestro y Rz y resta los CM de pisos consecutivos (base cero).
Calcula `amp=Δ_PD/Δ_lin` y `θ=P_i Δ_lin/(V_i h_i)` con su respuesta lineal.

Los criterios relativos por piso son **1e-3 para amplificación y deriva P-Δ
frente a ambas mallas de 8/16 elementos**, y **1e-9 frente a un elemento**.
La convergencia 8/16 se comprueba a **2e-6** (máximo observado ≈1.20e-6).
Se conservan y refuerzan los controles de θ lineal/P-Δ a 1e-9.
El diagnóstico unilateral comprueba **0 ≤ amp·(1−θ)−1 ≤ 5e-2**;
`1/(1−θ)` solo estima P-Δ de cuerda y omite P-δ. La tolerancia externa
**5e-3 frente a Frame3DD permanece intacta**; no se consultan sus resultados.
El test imprime por piso los seis errores y el exceso unilateral.

Ver la [bitácora MAM-860](../../../../../provenance/ENG-VAL-032-MAM-860.md).
