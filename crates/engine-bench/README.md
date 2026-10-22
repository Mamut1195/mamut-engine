# engine-bench

Modelos `bench-model/1` y su traducción a `engine-input/1` para el lado Engine de ENG-VAL-006.

El esquema está en [`schemas/bench-model-1.json`](schemas/bench-model-1.json).
El índice que consume un kit externo está en [`fixtures/index.json`](fixtures/index.json)
y su esquema en [`schemas/bench-index-1.json`](schemas/bench-index-1.json).

`engine-input/1` admite como máximo 256 nudos y 1024 barras, y un solo análisis por
documento. Los tres pórticos tienen 64/120, 144/320 y 256/600 nudos/barras. Cada modelo
deja versionados tres archivos traducidos: estático, modal y espectro.

`orientation` es el vector global de referencia del eje local z (`local_z_vector`), no un
ángulo. La excitación espectral es un eje global; en los modelos oficiales es X, con
vector `[1, 0, 0]`. La masa de cada piso es `{m, Izz}` en kg y kg·m², con `Izz` medido
respecto del eje vertical que pasa por el nudo maestro. `Sa.unit` es `m/s^2`.

El orquestador de medidas está en `scripts/eng-val-006/orchestrate.py`. Usa el binario
release, no se ejecuta contra el VPS desde este repositorio y no publica tiempos por su cuenta.

Los tres `rc3d-*` no usan los campos opcionales. Esos campos, y un fixture mínimo de
cada uno, están para los edificios que no cabían en el primer borrador:

- `nodal_masses`: si la clave está, es la única masa nodal. Si falta, sigue el reparto
  tributario de `{m, Izz}`.
- `modal.n`: entero de 1 a 16384. Los oficiales siguen en 12.
- `combination.method`: `cqc` o `srss`. `spectrum.directions` admite varios ejes, con un
  factor por eje. `spectrum.variants` nombra otras corridas del mismo modelo.
- `p_delta`: opciones literales de `pDelta`. Si faltan, la traducción falla.
- `load_patterns.patterns` y `static_combinations`: casos con nombre y combinaciones
  con factor y signo, solo en `linearStatic`.
- Diafragma vacío, `m` e `Izz` en 0, y `kind: truss` sin `orientation`, `G`, `Iy`, `Iz` ni `J`.

Un grado libre sin masa sigue siendo singular para el modal del Engine (`ENG-E-0006`).
El traductor no le añade masa para evitarlo. Un esclavo de diafragma tampoco admite
apoyo: el Engine responde `ENG-E-0013`.
