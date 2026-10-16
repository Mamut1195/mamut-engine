# ENG-BENCH-002 · Benchmarks publicados de P-Delta (MAM-582)

Pruebas: `crates/engine-analysis/tests/bench_pdelta.rs`. Oráculo: solo valores publicados y
soluciones cerradas de la ecuación diferencial de la viga-columna. Clean-room: no se abrió ni usó
código ni salidas de OpenSees, Stabileo, CalculiX o LAPACK.

Revisión como autor: Codex (GPT-6), 2026-10-15 (America/Guayaquil), rol implementador.
Fuentes de esta revisión: commit HEAD, código propio y derivaciones de esta nota; sin red,
sin consultar los libros originales ni código de terceros. Las referencias publicadas se
conservan del commit; las atribuciones pendientes siguen pendientes. Revisor clean-room: PENDIENTE.

## Fuentes

1. AISC, *ANSI/AISC 360-16 Specification for Structural Steel Buildings*, 7 jul. 2016,
   Commentary C2.1 "General Analysis Requirements", párrafo "Analysis Benchmark Problems",
   pp. 16.1-290 a 16.1-291; Fig. C-C2.2 (Case 1) y Fig. C-C2.3 (Case 2), p. 16.1-291.
   Tolerancias (p. 16.1-291): momentos dentro del 3 % y flechas dentro del 5 %.
   Se usan los valores entre corchetes ("exclude shear deformations"), porque `EulerFrame2D` no
   tiene deformación por cortante.
   - Datos geométricos leídos de las figuras: W14x48 eje fuerte (A = 14.1 in², I = 484 in⁴),
     E = 29 000 ksi, L = 28 ft; caso 1 w = 0.200 kip/ft; caso 2 H = 1.0 kip. Comprobados con
     P = 0: wL²/8 = 235.2 kip·in, 5wL⁴/384EI = 0.197 in, HL = 336 kip·in, HL³/3EI = 0.901 in.
   - AISC 360-22 no se verificó en una fuente accesible: PENDIENTE confirmar que las tablas no cambian.
2. S. P. Timoshenko y J. M. Gere, *Theory of Elastic Stability*, 2.ª ed., McGraw-Hill, 1961,
   cap. 1 (viga-columna simplemente apoyada con carga concentrada; χ(u) = 3(tan u − u)/u³ y
   amplificación aproximada 1/(1 − P/Pe)). Sección y página: PENDIENTE de verificar en un ejemplar;
   las fórmulas se re-derivan de EI v'''' + P v'' = q y son el oráculo efectivo.
3. W. F. Chen y E. M. Lui, *Structural Stability: Theory and Implementation*, Elsevier, 1987
   (citado por AISC C2): PENDIENTE; no hubo fuente accesible y no se usaron valores.

## Modelo y discretización

- `EulerFrame2D` + `geometric_producer` (Kg consistente), `PDelta::run`, tolerancia 1e-10,
  max 5000 pasos. Fuerzas en kip, longitudes en in, momentos en kip·in.
  Escalas de convergencia: 1 in para traslaciones y 0.01 rad para giros.
  Malla declarada: 8 elementos por miembro. P > 0 denota compresión y se aplica como −P en Ux;
  las cargas laterales y las flechas tabuladas son positivas en Uy.
- Caso 1: apoyos Ux,Uy / Uy; w con cargas nodales consistentes (wΔx en nodos interiores,
  wΔx/2 en cada apoyo y ±wΔx²/12 en los giros extremos; los momentos interiores se cancelan).
  M_mid por estática de media barra en la configuración deformada: wL²/8 + P·Δ_mid.
- Caso 2: empotrado; M_base = −Rz_base del motor, positivo para H > 0.
- Solución cerrada: caso 1 M = w/k²(sec u − 1), Δ = w/(EIk⁴)(sec u − 1) − wL²/(8P), u = kL/2;
  caso 2 M = H tan(kL)/k, Δ = H(tan kL − kL)/(Pk); k = √(P/EI).
- Carga puntual central Q (derivación propia): en el semivano izquierdo, 0 ≤ x ≤ L/2,
  EI v'' + Pv = −Qx/2, con v(0) = 0 y v'(L/2) = 0. Así, con u = kL/2,
  M_mid = Q tan(u)/(2k) y Δ_mid = [QL³/(48EI)]·3(tan u − u)/u³.
  El límite P = 0 es M_mid = QL/4, Δ_mid = QL³/(48EI).
  La aproximación Δ/Δ₀ ≈ 1/(1 − P/Pe) se comprueba al 2 % solo en los cocientes ensayados;
  esta comprobación no se atribuye al libro pendiente de verificar.

## Resultados (malla 8)

Los errores se calculan antes de redondear los resultados mostrados.

| Caso | P | Magnitud | Engine | Referencia | Error % | Tol | Estado |
|---|---|---|---|---|---|---|---|
| AISC 1 | 0 | M_mid kip·in | 235.20 | 235 | 0.09 | 3 % | pasa |
| AISC 1 | 150 | M_mid | 268.89 | 269 | 0.04 | 3 % | pasa |
| AISC 1 | 300 | M_mid | 313.52 | 313 | 0.17 | 3 % | pasa |
| AISC 1 | 450 | M_mid | 375.41 | 375 | 0.11 | 3 % | pasa |
| AISC 1 | 0 | Δ_mid in | 0.1971 | 0.197 | 0.03 | 5 % | pasa |
| AISC 1 | 150 | Δ_mid | 0.2246 | 0.224 | 0.27 | 5 % | pasa |
| AISC 1 | 300 | Δ_mid | 0.2611 | 0.261 | 0.02 | 5 % | pasa |
| AISC 1 | 450 | Δ_mid | 0.3116 | 0.311 | 0.19 | 5 % | pasa |
| AISC 2 | 0 | M_base kip·in | 336.00 | 336 | 0.00 | 3 % | pasa |
| AISC 2 | 100 | M_base | 469.07 | 469 | 0.01 | 3 % | pasa |
| AISC 2 | 150 | M_base | 598.65 | 598 | 0.11 | 3 % | pasa |
| AISC 2 | 200 | M_base | 848.98 | 848 | 0.12 | 3 % | pasa |
| AISC 2 | 0 | Δ_tip in | 0.9009 | 0.901 | 0.02 | 5 % | pasa |
| AISC 2 | 100 | Δ_tip | 1.3307 | 1.33 | 0.05 | 5 % | pasa |
| AISC 2 | 150 | Δ_tip | 1.7510 | 1.75 | 0.06 | 5 % | pasa |
| AISC 2 | 200 | Δ_tip | 2.5649 | 2.56 | 0.19 | 5 % | pasa |
| T&G | P/Pe 0.2–0.7 | M, Δ vs cerrada | — | — | ≤ 0.0053 | 0.1 % | pasa |

Caso 2 con P = 200 tiene P/Pe ≈ 0.65, con Pe = π²EI/(4L²) para el voladizo
(AISC pide incluir 0.6–0.7). En los casos simplemente apoyados, Pe = π²EI/L².
Todos los valores del motor coinciden con la solución cerrada a ≤ 0.1 %;
el mayor error frente a los valores AISC conservados es 0.27 %.

## Sensibilidad de malla (error vs solución cerrada, P máximo)

| n | C1 M | C1 Δ | C2 M | C2 Δ |
|---|---|---|---|---|
| 2 | 0.172 % | 0.461 % | 0.037 % | 0.062 % |
| 4 | 0.011 % | 0.030 % | 0.002 % | 0.004 % |
| 8 | 0.0007 % | 0.0019 % | 0.0002 % | 0.0003 % |
| 16 | <0.0001 % | 0.0001 % | <0.0001 % | <0.0001 % |

La prueba exige que los cuatro errores disminuyan en cada refinamiento.
La tolerancia 1e-3 frente a la solución cerrada se exige en la malla declarada de 8 elementos;
las mallas del estudio mantienen las tolerancias publicadas de 3 % en M y 5 % en Δ.

## Diagnóstico: piso del residual relativo

Con tolerancia 1e-12 el caso 2 (P = 200, malla 8) termina `NotConverged` tras 5000 pasos, y con
1e-10 lo mismo ocurre con malla 32: el cambio relativo llega a ~3e-15, pero el residual relativo
se estanca en ~4.8e-10 (piso de redondeo; el residual combina componentes de fuerza en kip y
de momento en kip·in sin escalarlas por GDL; la rigidez traslacional crece como 12EI/Δx³).
La comparación con las soluciones cerradas cuantifica la precisión de las mallas tabuladas;
el estancamiento por sí solo no garantiza precisión para una ejecución `NotConverged`.
Una tolerancia pedida por el usuario puede ser inalcanzable y el motor solo lo reporta como
`NotConverged`. Se recomienda un seguimiento antes o
junto con declarar P-Delta estable (p. ej. documentar el piso o normalizar el residual por GDL).

## Verificación de esta revisión

Sin red, con `CARGO_NET_OFFLINE=true` y `CARGO_TARGET_DIR=/tmp/me-bench-review-target`:

- `cargo fmt`: OK.
- `cargo clippy -p engine-analysis --all-targets -- -D warnings`: OK.
- `cargo test -p engine-analysis --test bench_pdelta -- --nocapture`: OK, 4 pruebas;
  resultados coherentes con ambas tablas. Referencias y tolerancias sin cambios.

Sin commit ni push.
