# Oráculos libres V1 · ENG-VAL-005 (MAM-105)

La validación primaria usa soluciones analíticas propias y fórmulas publicadas
permitidas por [CLEAN_ROOM](../../docs/CLEAN_ROOM.md), §4 y §8. OpenSees solo
puede ser un contraste adicional externo: nunca el único oráculo, dependencia
de CI o condición de aceptación. No se incorporan sus archivos ni resultados.

**ENG-REL-001 depende de ENG-VAL-005 y no exige ENG-VAL-003.**

Inventario: **91 casos** = 42 estáticos + 12 modales + 10 P-Delta +
9 espectrales de ENG-VAL-001 + 6 Winkler (ENG-GEO-002) + 7 de consolidación
(ENG-GEO-006) + 5 documentos `fixture/1` (ENG-CORE-009). No hay archivos ni
casos identificados como ENG-VAL-002 en este árbol; no se declara una suite
adicional inexistente. Toda futura incorporación debe tener fuente libre,
valor esperado y tolerancia, además de entrar en esta matriz y su guardia.

Cada fila enlaza la derivación independiente del solver. Los valores esperados
se evalúan desde las ecuaciones y los parámetros SI de las tablas, en
`tests/analytical.rs`, `tests/dynamic.rs`, `tests/support/winkler_harness.rs` y
`tests/support/consolidation_harness.rs` (más `consolidation_multilayer.rs`).
No se usan resultados del Engine como valores esperados. Los JSON declaran
cada número esperado en `expected` y su tolerancia explícita en `tolerance`.
Los tests numéricos recorren todos los casos; `tests/oracles.rs` comprueba
fuentes y límites finitos, unicidad, cobertura de la matriz mediante la suma de
longitudes de las tablas/fixtures y coincidencia de sus tolerancias. Comprueba
también que cada enlace resuelva un encabezado del archivo de provenance.
Los tests negativos comprueban fuentes ausentes/prohibidas o con la palabra
`PENDIENTE`, límites inválidos, justificaciones vacías/en blanco y enlaces rotos.

En las tablas históricas, `A` es el piso absoluto SI y `R` el error relativo:
`|calculado−esperado| ≤ A + R·|esperado|`. En `fixture/1` se conserva
`|calculado−esperado| ≤ max(A, R·|esperado|)`. Explicitar sus defaults no cambia
la aceptación. En consolidación las desigualdades son estrictas: error de
asiento dividido por s∞ (0.4 m, o 0.55 m en multicapa no transformable), error
relativo de U, error de presión dividido por 100 kPa y error relativo de flujo
solo donde hay interfaz con flujo no nulo. U es indefinido cuando Δσ=0.

## Matriz por caso

Las ecuaciones indicadas son el oráculo principal; las notas y pruebas incluyen
además giros, reacciones, modos, participación, equilibrio y refinamiento.
Las tolerancias existentes se conservan, incluidas las comprobaciones
adicionales descritas después de la matriz.

| Caso | Fuente libre y valor esperado (ecuación evaluada con los datos de la tabla) | Tolerancia |
| --- | --- | --- |
| `voladizo-punta-L4` | [Propia](../../provenance/ENG-VAL-001.md#voladizos) — v=−PL³/(3EI) | A=1e-9; R=1e-10 |
| `voladizo-punta-L7` | [Propia](../../provenance/ENG-VAL-001.md#voladizos) — v=−PL³/(3EI) | A=1e-9; R=1e-10 |
| `voladizo-uniforme-L4` | [Propia](../../provenance/ENG-VAL-001.md#voladizos) — v=−wL⁴/(8EI) | A=1e-9; R=1e-10 |
| `voladizo-uniforme-L7` | [Propia](../../provenance/ENG-VAL-001.md#voladizos) — v=−wL⁴/(8EI) | A=1e-9; R=1e-10 |
| `voladizo-par-terminal` | [Propia](../../provenance/ENG-VAL-001.md#voladizos) — v=PL²/(2EI) | A=1e-9; R=1e-10 |
| `simple-uniforme-L4` | [Propia](../../provenance/ENG-VAL-001.md#simplemente-apoyadas) — v=−5wL⁴/(384EI) | A=1e-9; R=1e-10 |
| `simple-uniforme-L7` | [Propia](../../provenance/ENG-VAL-001.md#simplemente-apoyadas) — v=−5wL⁴/(384EI) | A=1e-9; R=1e-10 |
| `simple-punto-centro-L4` | [Propia](../../provenance/ENG-VAL-001.md#simplemente-apoyadas) — v(a)=−Pa²b²/(3EIL) | A=1e-9; R=1e-10 |
| `simple-punto-centro-L7` | [Propia](../../provenance/ENG-VAL-001.md#simplemente-apoyadas) — v(a)=−Pa²b²/(3EIL) | A=1e-9; R=1e-10 |
| `simple-punto-cuarto` | [Propia](../../provenance/ENG-VAL-001.md#simplemente-apoyadas) — v(a)=−Pa²b²/(3EIL) | A=1e-9; R=1e-10 |
| `simple-punto-40porciento` | [Propia](../../provenance/ENG-VAL-001.md#simplemente-apoyadas) — v(a)=−Pa²b²/(3EIL) | A=1e-9; R=1e-10 |
| `simple-punto-70porciento` | [Propia](../../provenance/ENG-VAL-001.md#simplemente-apoyadas) — v(a)=−Pa²b²/(3EIL) | A=1e-9; R=1e-10 |
| `biempotrada-uniforme-L4` | [Propia](../../provenance/ENG-VAL-001.md#biempotradas) — v=−wL⁴/(384EI) | A=1e-9; R=1e-10 |
| `biempotrada-uniforme-L7` | [Propia](../../provenance/ENG-VAL-001.md#biempotradas) — v=−wL⁴/(384EI) | A=1e-9; R=1e-10 |
| `biempotrada-punto-centro` | [Propia](../../provenance/ENG-VAL-001.md#biempotradas) — v(a)=−Pa³b³/(3EIL³) | A=1e-9; R=1e-10 |
| `biempotrada-punto-cuarto` | [Propia](../../provenance/ENG-VAL-001.md#biempotradas) — v(a)=−Pa³b³/(3EIL³) | A=1e-9; R=1e-10 |
| `biempotrada-punto-70porciento` | [Propia](../../provenance/ENG-VAL-001.md#biempotradas) — v(a)=−Pa³b³/(3EIL³) | A=1e-9; R=1e-10 |
| `apuntalado-uniforme-L4` | [Propia](../../provenance/ENG-VAL-001.md#voladizo-apuntalado) — RA=5wL/8, RB=3wL/8 | A=1e-9; R=1e-10 |
| `apuntalado-uniforme-L7` | [Propia](../../provenance/ENG-VAL-001.md#voladizo-apuntalado) — RA=5wL/8, RB=3wL/8 | A=1e-9; R=1e-10 |
| `continua-dos-vanos-L4` | [Propia](../../provenance/ENG-VAL-001.md#dos-vanos) — Rext=3wL/8, Rcentro=5wL/4 | A=1e-9; R=1e-10 |
| `continua-dos-vanos-L7` | [Propia](../../provenance/ENG-VAL-001.md#dos-vanos) — Rext=3wL/8, Rcentro=5wL/4 | A=1e-9; R=1e-10 |
| `dos-barras-vertical` | [Propia](../../provenance/ENG-VAL-001.md#armaduras) — ux=Hℓ³/(2EAa²), uy=−Pℓ³/(2EAh²) | A=1e-9; R=1e-10 |
| `dos-barras-mixta` | [Propia](../../provenance/ENG-VAL-001.md#armaduras) — ux=Hℓ³/(2EAa²), uy=−Pℓ³/(2EAh²) | A=1e-9; R=1e-10 |
| `triangulo-vertical` | [Propia](../../provenance/ENG-VAL-001.md#armaduras) — Ni por equilibrio; u=Σ Ni ni ℓi/(EA) | A=1e-9; R=1e-10 |
| `triangulo-mixta` | [Propia](../../provenance/ENG-VAL-001.md#armaduras) — Ni por equilibrio; u=Σ Ni ni ℓi/(EA) | A=1e-9; R=1e-10 |
| `tres-concurrentes-vertical` | [Propia](../../provenance/ENG-VAL-001.md#armaduras) — uy=−P/(2EAh²/ℓ³+EA/h) | A=1e-9; R=1e-10 |
| `tres-concurrentes-mixta` | [Propia](../../provenance/ENG-VAL-001.md#armaduras) — uy=−P/(2EAh²/ℓ³+EA/h) | A=1e-9; R=1e-10 |
| `barra-asentamiento-positivo` | [Propia](../../provenance/ENG-VAL-001.md#asentamiento) — N=EA s/L, u(L/2)=s/2 | A=1e-9; R=1e-10 |
| `barra-asentamiento-negativo` | [Propia](../../provenance/ENG-VAL-001.md#asentamiento) — N=EA s/L, u(L/2)=s/2 | A=1e-9; R=1e-10 |
| `portal-lateral-h3` | [Propia](../../provenance/ENG-VAL-001.md#portal-lateral) — Δ=Hh³/(24EI), más solución finita | A=1e-8; R=5e-5 |
| `portal-lateral-h5` | [Propia](../../provenance/ENG-VAL-001.md#portal-lateral) — Δ=Hh³/(24EI), más solución finita | A=1e-8; R=5e-5 |
| `portal-vertical-h3` | [Propia](../../provenance/ENG-VAL-001.md#portal-vertical) — v=−wbh/(2EA), θ=−(wb²/12)/D | A=1e-9; R=1e-10 |
| `portal-vertical-h5` | [Propia](../../provenance/ENG-VAL-001.md#portal-vertical) — v=−wbh/(2EA), θ=−(wb²/12)/D | A=1e-9; R=1e-10 |
| `articulacion-extremo-L4` | [Propia](../../provenance/ENG-VAL-001.md#voladizo-apuntalado) — RA=5wL/8, RB=3wL/8, Mj=0 | A=1e-9; R=1e-10 |
| `articulacion-extremo-L7` | [Propia](../../provenance/ENG-VAL-001.md#voladizo-apuntalado) — RA=5wL/8, RB=3wL/8, Mj=0 | A=1e-9; R=1e-10 |
| `rotacion-modelo-30grados` | [Propia](../../provenance/ENG-VAL-001.md#rotacion) — uglobal=Q(α)(0,−PL³/(3EI)) | A=1e-9; R=1e-10 |
| `rotacion-modelo-90grados` | [Propia](../../provenance/ENG-VAL-001.md#rotacion) — uglobal=Q(α)(0,−PL³/(3EI)) | A=1e-9; R=1e-10 |
| `rotacion-modelo-137grados` | [Propia](../../provenance/ENG-VAL-001.md#rotacion) — uglobal=Q(α)(0,−PL³/(3EI)) | A=1e-9; R=1e-10 |
| `superposicion-P-mas-w` | [Propia](../../provenance/ENG-VAL-001.md#superposicion) — v=−αPL³/(3EI)−βwL⁴/(8EI) | A=1e-9; R=1e-10 |
| `combinacion-1_3-menos-0_7` | [Propia](../../provenance/ENG-VAL-001.md#superposicion) — v=−αPL³/(3EI)−βwL⁴/(8EI) | A=1e-9; R=1e-10 |
| `combinacion-menos-0_5-mas-2` | [Propia](../../provenance/ENG-VAL-001.md#superposicion) — v=−αPL³/(3EI)−βwL⁴/(8EI) | A=1e-9; R=1e-10 |
| `combinacion-cero-mas-0_5` | [Propia](../../provenance/ENG-VAL-001.md#superposicion) — v=−αPL³/(3EI)−βwL⁴/(8EI) | A=1e-9; R=1e-10 |
| `modal-cortante-1` | [Propia D1/D3](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=2√(k/m)sin((2j−1)π/(4n+2)) | A=1e-9; R=1e-9 |
| `modal-cortante-2` | [Propia D1/D3](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=2√(k/m)sin((2j−1)π/(4n+2)) | A=1e-9; R=1e-9 |
| `modal-cortante-3` | [Propia D1/D3](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=2√(k/m)sin((2j−1)π/(4n+2)) | A=1e-9; R=1e-9 |
| `modal-cortante-4` | [Propia D1/D3](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=2√(k/m)sin((2j−1)π/(4n+2)) | A=1e-9; R=1e-9 |
| `modal-cortante-5` | [Propia D1/D3](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=2√(k/m)sin((2j−1)π/(4n+2)) | A=1e-9; R=1e-9 |
| `modal-variable-2x2` | [Propia D2/D3](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — λ±=(144±√15416)/20, ω=√λ | A=1e-9; R=1e-9 |
| `modal-cadena-libre-fija` | [Propia D1/D3](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=2√(k/m)sin((2j−1)π/(4n+2)) | A=1e-9; R=1e-9 |
| `modal-apoyada-malla4` | [Propia D4](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=βj²√(EI/ρA)/L²; βj=jπ o cosβj coshβj=−1 | A=1e-9; R=1.9e-2 |
| `modal-apoyada-malla12` | [Propia D4](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=βj²√(EI/ρA)/L²; βj=jπ o cosβj coshβj=−1 | A=1e-9; R=3e-4 |
| `modal-voladizo-malla4` | [Propia D4](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=βj²√(EI/ρA)/L²; βj=jπ o cosβj coshβj=−1 | A=1e-9; R=1.9e-2 |
| `modal-voladizo-malla12` | [Propia D4](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=βj²√(EI/ρA)/L²; βj=jπ o cosβj coshβj=−1 | A=1e-9; R=3e-4 |
| `modal-voladizo-rotado` | [Propia D4](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ωj=βj²√(EI/ρA)/L²; βj=jπ o cosβj coshβj=−1 | A=1e-9; R=3e-4 |
| `pdelta-cero` | [Propia D5/D6](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — r=0: v=v0=HL³/(3EI) | A=1e-9; R=1.6e-2 |
| `pdelta-compresion-020` | [Propia D5/D6](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Pcr=π²EI/(4L²); v/v0=3(tan z−z)/z³, tracción: tanh | A=1e-9; R=1.6e-2 |
| `pdelta-compresion-050` | [Propia D5/D6](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Pcr=π²EI/(4L²); v/v0=3(tan z−z)/z³, tracción: tanh | A=1e-9; R=1.6e-2 |
| `pdelta-compresion-080` | [Propia D5/D6](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Pcr=π²EI/(4L²); v/v0=3(tan z−z)/z³, tracción: tanh | A=1e-9; R=1.6e-2 |
| `pdelta-compresion-090` | [Propia D5/D6](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Pcr=π²EI/(4L²); v/v0=3(tan z−z)/z³, tracción: tanh | A=1e-9; R=1.6e-2 |
| `pdelta-traccion` | [Propia D5/D6](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — r=−0.5: v/v0=3(z−tanh z)/z³ | A=1e-9; R=1.6e-2 |
| `pdelta-inestable` | [Propia D5/D6](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — r=1.05>1: Unstable, inercia negativa=1, sin desplazamientos | A=1e-9; R=1.6e-2 |
| `pdelta-piso-005` | [Propia D7](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Δ/Δ0=12(2tan(z/2)−z)/z³, z=√(12r) | A=1e-9; R=2.6e-2 |
| `pdelta-piso-010` | [Propia D7](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Δ/Δ0=12(2tan(z/2)−z)/z³, z=√(12r) | A=1e-9; R=2.6e-2 |
| `pdelta-iteracion-geometrica` | [Propia D8](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — ui=F(1−r^i)/(k(1−r)); residual=r^i | A=1e-9; R=1e-9 |
| `espectro-srss-2pisos` | [Propia D9](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — q=ΓSa/ω², Vb²=Σ(M*Sa)² | A=1e-9; R=1e-9 |
| `espectro-desplazamiento-modal` | [Propia D9](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — q=ΓSa/ω², Vb²=Σ(M*Sa)² | A=1e-9; R=1e-9 |
| `espectro-interpolacion-lineal` | [Propia D10](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Sa(2.5)=6 (lineal) o Sa(2)=12 (log-log) | A=1e-9; R=1e-9 |
| `espectro-interpolacion-loglog` | [Propia D10](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Sa(2.5)=6 (lineal) o Sa(2)=12 (log-log) | A=1e-9; R=1e-9 |
| `espectro-cqc-separados` | [Propia D11](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — R²=A²+B²+2ρAB; ρ por expansión algebraica suministrada | A=1e-9; R=1e-9 |
| `espectro-cqc-cercanos` | [Propia D11](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — R²=A²+B²+2ρAB; ρ por expansión algebraica suministrada | A=1e-9; R=1e-9 |
| `espectro-cqc-diagonal` | [Propia D11](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — R²=A²+B²+2ρAB; ρ por expansión algebraica suministrada | A=1e-9; R=1e-9 |
| `espectro-direccional-ponderado` | [Propia D12](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Vx=mSa,x, Vy=mSa,y; ponderada o SRSS | A=1e-9; R=1e-9 |
| `espectro-direccional-srss` | [Propia D12](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden) — Vx=mSa,x, Vy=mSa,y; ponderada o SRSS | A=1e-9; R=1e-9 |
| `Hetenyi-centro-2d` | [Propia](../../provenance/ENG-GEO-002.md#oráculos-analíticos-y-cotas) — v0=Pλ/(2k), M0=∓P/(4λ) | A=1e-8; R=2e-4 |
| `Hetenyi-extremo-2d` | [Propia](../../provenance/ENG-GEO-002.md#oráculos-analíticos-y-cotas) — v0=2Pλ/k | A=1e-8; R=2e-4 |
| `rigida-centro-2d` | [Propia](../../provenance/ENG-GEO-002.md#oráculos-analíticos-y-cotas) — v0=P/(kL) | A=1e-8; R=2e-5 |
| `Hetenyi-centro-3d-dos-planos` | [Propia](../../provenance/ENG-GEO-002.md#oráculos-analíticos-y-cotas) — v0=Pλ/(2k), M0=∓P/(4λ) | A=1e-8; R=2e-4 |
| `Hetenyi-extremo-3d-dos-planos` | [Propia](../../provenance/ENG-GEO-002.md#oráculos-analíticos-y-cotas) — v0=2Pλ/k | A=1e-8; R=2e-4 |
| `rigida-centro-3d-dos-planos` | [Propia](../../provenance/ENG-GEO-002.md#oráculos-analíticos-y-cotas) — v0=P/(kL) | A=1e-8; R=2e-5 |
| `Terzaghi-dren-superior` | [Propia](../../provenance/ENG-GEO-006.md#serie-de-terzaghi-derivación-y-fixtures) — U=1−Σ2 exp(−M²Tv)/M²; u/Δσ=Σ2 sin(Mx)exp(−M²Tv)/M | s/s∞<1.5e-3; Urel<5e-3; u/100<4e-3; qrel<3e-2 |
| `Terzaghi-dren-inferior` | [Propia](../../provenance/ENG-GEO-006.md#serie-de-terzaghi-derivación-y-fixtures) — U=1−Σ2 exp(−M²Tv)/M²; u/Δσ=Σ2 sin(Mx)exp(−M²Tv)/M | s/s∞<1.5e-3; Urel<5e-3; u/100<4e-3; qrel<3e-2 |
| `Terzaghi-doble-drenaje` | [Propia](../../provenance/ENG-GEO-006.md#serie-de-terzaghi-derivación-y-fixtures) — U=1−Σ2 exp(−M²Tv)/M²; u/Δσ=Σ2 sin(Mx)exp(−M²Tv)/M | s/s∞<1.5e-3; Urel<5e-3; u/100<4e-3; qrel<3e-2 |
| `Terzaghi-rampa` | [Propia](../../provenance/ENG-GEO-006.md#serie-de-terzaghi-derivación-y-fixtures) — Convolución de la serie con rampa r=0.2; descarga como rampa desplazada restada | s/s∞<1.5e-3; Urel<5e-3; u/100<4e-3; qrel<3e-2 |
| `Terzaghi-rampa-descarga` | [Propia](../../provenance/ENG-GEO-006.md#serie-de-terzaghi-derivación-y-fixtures) — Convolución de la serie con rampa r=0.2; descarga como rampa desplazada restada | s/s∞<1.5e-3; Urel<5e-3; u/100<4e-3; qrel<3e-2 |
| `multicapa-flujo-interfaz` | [Propia](../../provenance/ENG-GEO-006.md#multicapa-con-flujo-no-nulo-derivación-propia) — Serie en x=∫dz/√cv con mv√cv constante; flujo por derivada | s/s∞<1.5e-3; Urel<5e-3; u/100<4e-3; qrel<3e-2 |
| `multicapa-no-transformable` | [Propia r32](../../provenance/ENG-GEO-006.md#revisión-j4-r32-multicapa-no-transformable-por-autofunciones) — u=Σ100 Im Xm exp(−ωm²t)/Nm; s=0.55−Σ100 Im² exp(−ωm²t)/Nm | s/s∞<1.5e-3; Urel<5e-3; u/100<4e-3; qrel<3e-2 |
| `axial-bar` | [Propia](../../provenance/ENG-CORE-009.md#derivaciones-unidades-y-signos) — /cases/0/nodes/1/displacements/0 = 0.12 (resto en expected) | A=1e-10; R=1e-10 |
| `axial-series` | [Propia](../../provenance/ENG-CORE-009.md#derivaciones-unidades-y-signos) — /cases/0/nodes/1/displacements/0 = 0.06 (resto en expected) | A=1e-10; R=1e-10 |
| `cantilever-point` | [Propia](../../provenance/ENG-CORE-009.md#derivaciones-unidades-y-signos) — /cases/0/nodes/1/displacements/1 = -0.4266666666666667 (resto en expected) | A=1e-10; R=1e-10 |
| `simply-supported-uniform` | [Propia](../../provenance/ENG-CORE-009.md#derivaciones-unidades-y-signos) — /cases/0/nodes/1/displacements/1 = -0.03333333333333333 (resto en expected) | A=1e-10; R=1e-10 |
| `spatial-cantilever` | [Propia](../../provenance/ENG-CORE-009.md#derivaciones-unidades-y-signos) — /cases/0/nodes/1/displacements/2 = -0.4266666666666667 (resto en expected) | A=1e-10; R=1e-10 |

## Comprobaciones adicionales y referencias publicadas

- P-Delta, columna: solución continua a A=1e-10, R=2e-6; el R=1.6e-2
  tabulado corresponde a la aproximación de polo 1/(1−r). En r>1 se exige
  el estado de inestabilidad y no se compara una flecha inexistente.
- Pórtico P-Delta: solución continua a A=1e-10, R=1e-4; R=2.6e-2 para
  la aproximación de cuerda. Pórtico lineal lateral: solución finita adicional
  a A=1e-8, R=1e-10. Se conservan los límites de las pruebas originales.
- Ortogonalidad/rotación modal: A=2e-9, R=2e-8. CQC muy separado frente
  a SRSS: A=1e-10, R=2e-5; la fórmula CQC completa usa el límite de su fila.
- Winkler: λ=(k/(4EI))^(1/4); además equilibrio del lecho a A=1e-8,
  R=1e-7, refinamiento decreciente y límites de residual/caras indicados en
  [ENG-GEO-002](../../provenance/ENG-GEO-002.md#oráculos-analíticos-y-cotas).
- Consolidación: 2048 términos de la serie propia, cola acotada en
  [ENG-GEO-006](../../provenance/ENG-GEO-006.md#serie-de-terzaghi-derivación-y-fixtures).
  Multicapa no transformable usa autofunciones y continuidad de presión/flujo,
  independientes de la discretización FE; también se verifica convergencia.

Los benchmarks publicados adicionales existentes están en
[`bench_pdelta.rs`](../engine-analysis/tests/bench_pdelta.rs). Los dos casos
AISC conservan sus datos versionados y se contrastan también con la solución
cerrada propia de `EI v''''+P v''=q`: caso 1 a P=0/150/300/450 y caso 2 a
P=0/100/150/200 (kip). La aceptación publicada es 3% en M y 5% en Δ;
la cerrada es 0.1% en ambas. La carga puntual central usa P/Pe=.2/.4/.6/.7,
0.1% frente a la solución cerrada y 2% frente a la aproximación de polo.
El cuarto test exige convergencia monótona en mallas 2/4/8/16. Fuentes y
derivaciones existentes: [ENG-BENCH-002](../../provenance/ENG-BENCH-002.md).
No se copian aquí sus tablas ni figuras; sus valores y tolerancias no cambian.

## Referencia bibliográfica opcional (por cotejar)

Estas atribuciones no cotejadas no son el oráculo de aceptación ni condicionan
sus tolerancias. La fuente primaria verificable de cada caso es la derivación
propia enlazada en la matriz y en `citation`. No se consultaron fuentes externas
en esta sesión.

- Winkler (1867): atribución conceptual heredada; obra y localizador por cotejar.
  La ecuación de lecho y su solución se derivan en ENG-GEO-002.
- M. Hetényi, *Beams on Elastic Foundation*, 1946. Edición y localizador de
  ecuación: **PENDIENTES**. La derivación efectiva resuelve EI v''''+kv=Pδ.
- K. Terzaghi, *Theoretical Soil Mechanics*, primera edición, Wiley, 1943.
  Localizador de ecuación: **PENDIENTE**. Se deriva la serie por separación y
  proyección de una presión inicial uniforme, con convolución de rampas.
- H. S. Carslaw y J. C. Jaeger, *Conduction of Heat in Solids*, segunda
  edición, Oxford University Press, 1959. Localizador de ecuación:
  **PENDIENTE**. La referencia efectiva es la derivación multicapa propia r32.
- S. P. Timoshenko y J. M. Gere, *Theory of Elastic Stability*, segunda
  edición, McGraw-Hill, 1961. Sección/página/ecuación: **PENDIENTES**;
  las soluciones se derivan en ENG-BENCH-002, sin depender de esa atribución.
- AISC, *ANSI/AISC 360-16*, Commentary C2.1, pp. 16.1-290–291:
  referencia heredada de ENG-BENCH-002; su cotejo con el original en esta
  sesión offline es **PENDIENTE**. Se mantiene el contraste analítico propio.
- CQC/WDKB 1981: fórmula suministrada y expandida algebraicamente en D11;
  cita bibliográfica y número de ecuación exactos **PENDIENTES**.

Estos pendientes bibliográficos no se rellenan de memoria ni convierten un
resultado de otro motor en referencia. Revisión clean-room: **PENDIENTE (J4)**.
