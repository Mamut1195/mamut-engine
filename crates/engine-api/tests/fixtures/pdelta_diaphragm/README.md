# E3-pdelta: referencia independiente

El test y `reference.py` leen directamente `runs[id=E3-pdelta].engine_input`
de `docs/validation-report/fixtures/val032/E3-v1.1.1.json`, incorporado a
`origin/main` en `8a1f21f` por #180. Se eliminó la copia local `e3.json`:
su entrada es numéricamente idéntica a la de main. No se modifican los JSON
ni los hashes de VAL-032.

`reference.py` deriva la función `assemble` del modelo propio NumPy
`docs/validation-report/fixtures/val032/_ref/val032_calibrate.py` de esa revisión
(SHA-256 del script original:
`7068d3a69fde879c68dc10c0215735e0dfbbc8988f55e411fb76f12186377f84`).
La extensión P-Δ es una derivación propia para #185; no consume respuestas del
Engine ni datos de otros programas de análisis. NumPy se usa como dependencia
numérica (BSD-3-Clause); no se consultó ni copió su código fuente.

## Generación offline

Desde la raíz del repositorio, con Python 3 y NumPy instalado (esta sesión:
Python 3.13.5, NumPy 2.5.4):

```sh
OPENBLAS_NUM_THREADS=1 python3 crates/engine-api/tests/fixtures/pdelta_diaphragm/reference.py
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/pdelta-main-target \
  cargo test -p engine-api --test pdelta_diaphragm -- --nocapture
```

El primer comando regenera `reference.json` y comprueba estabilidad positiva y
residuo proyectado relativo <1e-10. El test Rust consume los valores guardados;
no necesita Python/NumPy en CI. `fixture_sha256` identifica los bytes completos
del contenedor E3 y `run_id` identifica `E3-pdelta`; el test verifica ambos.
Se regeneró `reference.json` tras cambiar la fuente: solo cambian esos metadatos,
con los cinco pisos y toda la evidencia numérica idénticos a la referencia previa.
Distintas versiones de BLAS pueden variar las últimas cifras; el criterio Rust
es 1e-4 relativo por piso, sin piso absoluto.

## Ecuaciones y definición de θ

El ensamblaje elástico es Euler 3D, sin liberaciones ni cortante. T impone
exactamente los diafragmas XY y elimina apoyos homogéneos. Se calcula
`u_g = T (Tᵀ K T)⁻¹ Tᵀ F_G`. Para cada barra, el axial positivo a tracción es
`N = EA/L (u_g,b,x − u_g,a,x)` en ejes locales originales.

Para las dos flexiones se integra `Kg_ab = N ∫ H′_a H′_b dx`, con Hermite
cúbico. Tres puntos Gauss integran exactamente el producto de derivadas de
grado cuatro, dando los coeficientes consistentes N/(30L) de pretensión
inicial. Se ensamblan también los axiales de las vigas. No se añaden términos
de torsión geométrica ni se actualizan axiales con el desplazamiento lateral.
La solución de referencia se obtiene directamente, sin punto fijo:

`u_PD = T [Tᵀ(K + Kg(u_g))T]⁻¹ Tᵀ F_EX`.

`Delta_i` es la diferencia de Ux entre centros de masa de pisos consecutivos
(base cero), transportada desde cada maestro mediante su Rz.
`P_i` acumula las cargas verticales G desde el piso i hasta cubierta;
`V_i` acumula las fuerzas EX del mismo conjunto y `h_i` es la altura de
entrepiso. Se calcula `theta_i = P_i Delta_i / (V_i h_i)`.

`theta_linear` usa la deriva de `K u = F_EX`, como `val032_calibrate.py`;
`theta_pdelta` usa la deriva lateral independiente de `(K+Kg)u = F_EX`, sin
sumarle los desplazamientos de gravedad. El test compara **theta_pdelta y la
deriva P-Δ** en los cinco pisos a 1e-4 relativo. No se compara la deriva P-Δ
con θ lineal ni se usa la aproximación de amplificación `1/(1-theta)`.

Los θ lineales reproducen los del modelo propio VAL-032 (máxima diferencia
relativa ~7e-15). La referencia P-Δ da θ aproximadamente
0.19266315, 0.16906781, 0.12229877, 0.07876087 y 0.03881422.
Ver también [la bitácora](../../../../../provenance/ENG-ANL-pdelta-diafragma.md).
