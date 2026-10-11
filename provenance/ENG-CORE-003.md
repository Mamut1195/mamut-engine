# Bitácora clean-room · ENG-CORE-003

- Ticket / PR: ENG-CORE-003 (modelo de dominio y validación acumulativa) · rama `j2/eng-core-003-model`; sin commit, push ni PR creado en esta sesión.
- Autor(es): Codex (gpt-6.1-sol, esfuerzo medium) como autor del código, documentación, bitácora y verificación, bajo la dirección de J2 · 2026-10-10 (America/Guayaquil).
- Rol: especificador e implementador del modelo de dominio propio; sin actividad de operador de oráculo ni auditor de similitud.
- Contrato implementado: crate `engine-model`, registrado en el workspace y dependiente únicamente de `engine-core` por path. `Model` conserva nodos con coordenadas, dimensión y GDL; miembros de dos nodos con referencias a sección y ley; secciones con propiedades genéricas; leyes con nombre y parámetros; apoyos; casos de cargas nodales; combinaciones de casos con factores; masas e inercias nodales. Constructor e inserciones sin validación implícita. `validate(&Model) -> Vec<Diagnostic>` acumula duplicados (`ENG-E-0002`), referencias rotas (`ENG-E-0003`), huérfanos (`ENG-E-0004`), longitud nula (`ENG-E-0005`), entradas no finitas (`ENG-E-0008`) y GDL inválidos por dimensión (`ENG-E-0011`). Reutiliza exclusivamente el catálogo existente sin modificar `engine-core`. No implementa ningún `kind` de elemento ni método de cálculo estructural.
- Fuentes permitidas usadas: Ninguna fuente externa; solo documentos internos del proyecto: `README.md`, `docs/CLEAN_ROOM.md`, `clean-room/forbidden_identifiers.txt`, `crates/engine-core/src/lib.rs`, `crates/engine-core/src/catalog.rs`, `crates/engine-core/Cargo.toml`, configuración del workspace, `provenance/TEMPLATE.md` y `provenance/ENG-CORE-001.md`. Mecánica estructural estándar y diseño propio (coordenadas cartesianas, terna diestra, GDL, conectividad, distancia euclídea, resultantes por cara, combinación lineal de acciones). Mensajes de contexto, diseño de datos, documentación y pruebas son propios; no se reutilizó código de structural-kernel ni motor-geotecnico.
- Derivaciones propias: [convención de ejes y signos](../docs/CONVENCION_EJES.md) y [rustdoc del módulo](../crates/engine-model/src/lib.rs). Eje local x de i a j; en D2, y = (-x_y, x_x, 0) y z = x × y. La convención X-Y horizontal con Z vertical arriba está aprobada (ver más abajo); la de elevación 2D sigue pendiente. Tabla propia de signos por cara y extremo, con tracción positiva. **APROBADA por Jonathan 2026-10-10 22:52: X-Y horizontal, Z vertical arriba (el Engine gobierna).** La distancia se obtiene mediante acumulación `hypot`; longitud menor o igual a `1e-12` en la unidad de coordenadas se considera nula, sin escalado relativo. Se omite ese control si faltan extremos o hay coordenadas no finitas. No se deriva una formulación de rigidez ni se evalúa ninguna ley.
- Código de terceros copiado: ninguno. Solo biblioteca estándar de Rust y el contrato interno de `engine-core`; sin otras dependencias.
- Validación: N/A (analítico: N/A · libro: N/A · ejemplo publicado: N/A)
- Justificación: este ticket no implementa método numérico, ley ni formulación de rigidez (ver «Derivaciones propias»), así que no hay resultado que contrastar con solución analítica, libro ni ejemplo publicado y no se cita fuente externa (`docs/CLEAN_ROOM.md` §4 y §7). Lo que se valida son los invariantes topológicos del modelo de dominio (unicidad de identificadores, referencias, huérfanos, longitud nula, valores finitos, GDL por dimensión) mediante pruebas unitarias y doctests propios. Detalle: pruebas unitarias y doctests propios, sin juegos de datos externos. Diez pruebas unitarias del nuevo crate: modelo vacío y modelos válidos D2/D3; duplicados conservados en las ocho colecciones; todas las referencias rotas (incluidas acciones nodales); huérfanos aun con apoyo/carga/masa; extremos idénticos y límites inclusivos de tolerancia; NaN y ambos infinitos en todos los campos numéricos; GDL inválidos en nodos, apoyos, cargas y masas; referencias nominales para entidades sin representante conectado; orden determinista y primera declaración ante duplicados; distancias finitas grandes y pequeñas. Cada error se comprueba con código, entidad culpable y contexto exacto; se verifica reutilización de mensajes y gravedad del catálogo. Siete doctests: modelo válido y uno para cada uno de los seis códigos exigidos, comprobando entidades. Verificación local con `~/.cargo/bin/cargo fmt --check`, `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`, `~/.cargo/bin/cargo test` y `RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps`: todos satisfactorios. Workspace completo: 17 pruebas unitarias y 15 doctests satisfactorios. `#![deny(missing_docs)]` obligatorio en el nuevo crate. Comprobación local adicional de ausencia de identificadores prohibidos en los archivos del ticket y diff sin modificaciones a rutas protegidas. Sin ejecución de CI remota.
- Comparación con oráculo (repo hermano, informativa): no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: pendiente; revisión humana y aprobación de Jonathan pendientes.

## Decisiones de alcance y trazabilidad

Los identificadores se comparan dentro de cada clase, con igualdad exacta y sin
normalizar nombres. Las colecciones son listas para no perder duplicados durante
la construcción. El orden de diagnósticos sigue clases y posiciones de entrada;
las consultas resuelven un nodo duplicado a su primera declaración. Las listas
de parámetros, GDL y términos no son entidades con identificador propio, de modo
que no se impone unicidad adicional en ellas.

Un nodo es huérfano si no aparece en ningún extremo de miembro, incluso cuando
ese miembro tenga otra referencia inválida. Los GDL se contrastan con `Dim` del
nodo correspondiente. No se controla estabilidad, positividad física, geometría
Z = 0 de D2 ni compatibilidad dimensional de miembros en este ticket.

Sin ampliar `EntityRef`, las secciones señalan al primer miembro que las usa;
las secciones sin miembro emplean `Law(id_sección)` como referencia nominal.
Apoyos y masas señalan su nodo, aunque esté ausente. Las combinaciones señalan
el caso del término culpable; duplicados señalan al primer caso y combinaciones
vacías emplean `Case(id_combinación)` como referencia nominal. Una referencia
nominal no declara la existencia de otra entidad. Todos los diagnósticos incluyen
la clase y el nombre/id exacto de la entidad original en `hint`. Estas reglas
están documentadas y probadas para que el consumidor pueda interpretar el contexto.
