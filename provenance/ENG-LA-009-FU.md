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
