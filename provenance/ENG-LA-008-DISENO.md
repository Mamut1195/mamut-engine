# Bitácora clean-room · ENG-LA-008-DISENO

- Ticket / PR: MAM-67, ENG-LA-008. Diseño del solver directo disperso simétrico. Rama `j2/mam-67-diseno-solver-disperso`. Pull request en borrador contra `main`. Este corte no implementa el solver, no añade dependencias y no cierra MAM-107.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-10 01:09 (America/Guayaquil, ECT, UTC-5).
- Rol: especificador. No hay código de producción.
- Session: sesión del 2026-11-10 (America/Guayaquil, ECT, UTC-5). Esta sesión no vio código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados. No se abrió el código de SuiteSparse, CSparse, CHOLMOD, AMD de Davis, MUMPS, PARDISO ni de un derivado, ni el de las crates candidatas.
- Contrato implementado: ninguno. Se describe el uso futuro de `LinAlgBackend`, `FactorInfo`, `SymFactorization`, `MemoryBudget` y `LinearSystem::translate_solver_error`, ya presentes en el commit `9aeebcc000ca42c0f94b5aa41507b1da1e378743`.
- Fuentes permitidas: las de la sección siguiente. Páginas no abiertas: cita desconocida. No se copió un listado.
- Código de terceros copiado: ninguno.
- Validación: no hay ensayo numérico nuevo. Los esperados del diseño son derivación propia, escritos en `docs/design/MAM-67-solver-disperso.md`, apartado 8.1. El modelo de 10 pisos no se ejecutó.
- Comparación con oráculo (repositorio hermano, informativa): no.
- Declaración: "No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados, para este ticket. Esta sesión no abrió ese código ni esas salidas."
- Revisor clean-room: PENDING

## Fuentes consultadas

Código y documentos del propio repositorio, en el commit de base:

- `crates/engine-linalg/src/faer_backend.rs`
- `crates/engine-linalg/src/lib.rs`
- `crates/engine-linalg/src/matrix.rs`
- `crates/engine-linalg/src/sparse.rs`
- `crates/engine-linalg/src/budget.rs`
- `crates/engine-linalg/src/modes.rs`
- `crates/engine-linalg/Cargo.toml`
- `crates/engine-assembly/src/system.rs`
- `crates/engine-assembly/src/dofmap.rs`
- `crates/engine-assembly/src/constraints.rs`
- `crates/engine-contracts/src/traits.rs`
- `crates/engine-core/src/units.rs`
- `docs/contracts/error-codes-1.json` (32 entradas; no se modifica)
- `docs/DETERMINISM.md`
- `docs/adr/ADR-002-algebra-lineal.md`
- `docs/adr/ADR-002-evidencia.md`, solo para no repetir un grafo ya marcado allí; no se rehízo ese ensayo
- `docs/adr/ADR-003-python-v1.md`
- `docs/tickets/README.md` y las fichas 01, 04, 08 y 09, para el estilo
- `provenance/ENG-LA-007.md` y `provenance/ENG-LA-008.md`, como antecedente del marcador y del presupuesto de memoria
- `provenance/TEMPLATE.md`
- `changelog.d/README.md`

No se abrió `provenance/ENG-BENCH-001-SMOKE-VPS.md`.

Medición del propio motor citada en el diseño (250 nudos 3D): dato entregado en el encargo. El informe de origen no se abrió. Cita desconocida.

Citas bibliográficas. El artículo o el libro no se abrió. Volumen, número y páginas: cita desconocida, salvo el año cuando forma parte del título recordado:

- Patrick R. Amestoy, Timothy A. Davis e Iain S. Duff, «An Approximate Minimum Degree Ordering Algorithm», SIAM Journal on Matrix Analysis and Applications, 1996.
- Joseph W. H. Liu, «The Role of Elimination Trees in Sparse Factorization», SIAM Journal on Matrix Analysis and Applications, 1990.
- E. Cuthill y J. McKee, «Reducing the bandwidth of sparse symmetric matrices», proceedings de la ACM National Conference, 1969.
- J. R. Bunch y L. Kaufman, Mathematics of Computation, 1977. Título de artículo: cita desconocida.
- T. A. Davis, *Direct Methods for Sparse Linear Systems*, SIAM, 2006. Solo como mención de la ficha pública de una crate. No se abrió el libro.

Fichas y documentación de API, consultadas el 2026-11-10. No se abrió el código fuente enlazado desde esas páginas:

- ficha de crates.io de `faer`, versión 0.24.4, SPDX MIT en el manifiesto
- endpoint de dependencias de `faer` 0.24.4: 12 dependencias normales obligatorias
- `https://docs.rs/faer/0.24.4/faer/sparse/linalg/cholesky/index.html`
- `https://docs.rs/faer/0.24.4/faer/linalg/cholesky/ldlt/factor/struct.LdltRegularization.html`
- página de `SimplicialLdltRef` en docs.rs para `faer`, canal `latest`; la ficha de crates.io ese día marcaba 0.24.4 como última versión
- ficha y endpoint de dependencias de `sparse-ldlt` 0.3.3, SPDX MIT, MSRV 1.75, sin dependencias normales
- ficha de `sprs-ldl` 0.10.0, SPDX LGPL-2.1
- ficha de `nalgebra-sparse` 0.12.0, SPDX Apache-2.0
- ficha de `scirs2-sparse` 0.6.5, SPDX Apache-2.0
- ficha de `amd` 0.2.2, SPDX BSD-3-Clause

El recorrido manual del grafo de `faer` por la API de crates.io, sin `cargo tree`, no se usa como conteo verificado. Quedó escrito así en el diseño.

`Cargo.lock` se consultó solo para confirmar que el nombre `faer` no está. No se resolvió ninguna crate nueva. Cargo de este repositorio no se usó para descargar.

## Qué no se hizo

- No hay implementación, ni reserva de matrices, ni ensayo de tiempo o de memoria.
- `docs/contracts/error-codes-1.json` no gana entradas. ENG-E-0015 queda como propuesta en el diseño.
- El revisor clean-room no está firmado.
