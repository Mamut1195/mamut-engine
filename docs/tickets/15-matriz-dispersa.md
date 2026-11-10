# 15 · Matriz simétrica dispersa

- Estado: en curso. Fase 2 de MAM-67.
- Relación: MAM-1055. El diseño está en el PR #27, rama
  `j2/mam-67-diseno-solver-disperso`, archivo
  `docs/design/MAM-67-solver-disperso.md`. Esa rama no se copia a este corte.
- Bitácora: `provenance/ENG-LA-010.md`. ENG-LA-009 ya existe. El revisor
  queda en PENDING. Esta ficha no cierra MAM-67 ni MAM-107.
- Reglas comunes: [README.md](README.md).

## Numeración

En las ramas remotas `j2/*` consultadas el 2026-11-10, nadie tenía el 15.
El 11 es MAM-67 y vive en la rama de diseño. El 12 (MAM-1045), el 13
(MAM-1043) y el 14 (MAM-925) no tienen archivo en esas ramas; el encargo
los da por ocupados. Esta ficha usa el siguiente número libre, el 15.

## Decisiones ya tomadas

- Opción A: implementación propia, en `engine-linalg`. Sin crates nuevas
  y sin cambios de Cargo.
- El rechazo por tamaño o memoria usa ENG-E-0013. No se añade un código.
  `docs/contracts/error-codes-1.json` permanece en 32 entradas.
- El tipo no convierte unidades. Los valores ya llegan en un sistema
  coherente.
- El corte de orden entre la ruta densa y la dispersa no se fija aquí.
  Se mide en la fase 10. Esta ficha no elige 2048 ni 60000.

## Qué construir

Un tipo de matriz simétrica que guarda solo el triángulo superior en CSC,
diagonal incluida. En la columna `j` se almacenan filas `i ≤ j`, crecientes.
El diseño queda alineado con esta fase: triángulo superior y suma
canónica. La otra mitad no se materializa.

La construcción parte de tripletas.

- Un duplicado es otra tripleta con la misma posición y el mismo lado
  (superior o inferior). Esos valores se suman de menor a mayor según
  `f64::total_cmp`. El resultado no depende del orden de inserción.
  No se usa `HashMap`.
- Si llegan las dos mitades, cada una se suma por su lado y las dos sumas
  tienen que ser iguales. Se guarda un solo valor, el de la mitad superior.
  Si difieren, la simetría declarada se rechaza.
- Si llega una sola mitad, ese valor es el de la entrada. No se duplica.
- Un cero explícito se conserva. Si una diagonal no viene en las tripletas,
  se guarda un cero. Hace falta para nombrar el pivote en una fase posterior.
  Esta fase no factoriza.
- Se rechazan los índices fuera de orden y los valores no finitos. También
  se rechaza una suma que deje de ser finita.

Antes de reservar, la cota usa solo el orden `n` y el número de tripletas
`nnz`:

- punteros de columna: `(n + 1)` enteros;
- índices y valores: como máximo `nnz + n` entradas, diagonal incluida
  aunque ya venga entre las tripletas;
- borrador de ordenación: un registro por tripleta, ordenado en el sitio.

Si `n` supera el orden máximo que pasa el llamador, o si esa cota supera
el tope de bytes que pasa el llamador, o si la cota desborda `usize`,
se devuelve ENG-E-0013 y no se reserva la matriz. El tope lo elige el
llamador. No es una constante nueva del motor.

## Fuera de esta fase

No hay factorización, ni ordenamiento, ni integración con `LinearSystem`.
No cambia la ruta densa, ni `SparseMatrix`, ni los goldens, ni
`docs/DETERMINISM.md`.

## Hecho cuando

- Esta ficha está en el índice y el código llega después.
- Hay tests de construcción, duplicados, los mismos bits con dos órdenes
  de inserción, índices fuera de rango, valores no finitos, rechazo previo
  sin reserva y producto matriz-vector contra la versión densa en casos
  pequeños. Los esperados salen de una derivación propia.
- Hay fragmento `changelog.d/MAM-1055.added.md` y la bitácora queda con
  el revisor en PENDING.
