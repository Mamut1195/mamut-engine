# Bitácora clean-room · ENG-VAL-006

- Ticket / PR: MAM-855 · ENG-VAL-006; solo documentación.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Rol: especificador del protocolo documental; sin implementación del Engine.
- Contrato implementado: no aplica; enmienda de `docs/CLEAN_ROOM.md`, §2/§3/§5/§5.1/§8/§9/§10.
- Fuentes permitidas usadas: petición del ticket; `docs/CLEAN_ROOM.md`;
  `provenance/TEMPLATE.md`; declaraciones clean-room de
  `provenance/ENG-BENCH-001-SMOKE-VPS.md`; decisión de Jonathan
  (BENCH-001, MAM-253, 2026-10-20), aportada en la petición.
- Derivaciones propias: no aplica; no se introducen métodos ni cálculos.
- Código de terceros copiado: ninguno.
- Validación: revisión documental y `bash clean-room/check.sh origin/main`:
  `Clean-room: OK` (salida 0). `git diff --check`: OK.
  Cargo y pruebas Rust omitidos por la excepción explícita de esta tarea.
- Comparación con oráculo (repo hermano, informativa): no; no se ejecutaron
  oráculos ni se incorporaron sus valores, informes o salidas.
- Declaración: "No consulté código fuente de OpenSees, Frame3DD, Stabileo
  ni derivados para este ticket. Tampoco consulté código de CalculiX o LAPACK.
  No usé resultados de estos programas para especificar, probar o implementar
  métodos del Engine. Se trabajó sin red ni fuentes externas."
- Revisor clean-room: J4 r1 sobre 5974c9c (APTO con seguimiento).

Se incorpora Frame3DD (release 20140514 o posterior) como ejecutable externo
exclusivo de `mamut-engine-oracles`, con versión y hash del binario fijados,
sin leer fuentes ni enlazar, empaquetar o distribuir con el Engine o MAMPRO.
La opción A reserva la operación de OpenSees y Frame3DD a Q2 en sesión
aislada, bajo auditoría de J4; J2 y J0 solo reciben «difiere en caso X», sin valores.
El seguimiento de revisión alinea §2 y §3 con Frame3DD y la opción A; precisa
que el operador fija versión y hash por campaña y que un cambio de versión
requiere un nuevo registro y auditoría de J4.

Seguimiento J4 r1 del PR #178 (2026-10-20): H1 separa la aprobación directa
por la fusión personal de Jonathan de los antecedentes relayados por J0 y cita
el registro de humo solo como origen de la opción A. H2 extiende la opción A
a los oráculos de §5.1, con Stabileo pendiente de ENG-GOV-004, y reserva
`results/` a Q2 y J4 para auditar. H4 añade Frame3DD a la declaración de la
plantilla. N1 registra la revisión anterior. N2 renombra la bitácora a
`provenance/ENG-VAL-006-CLEAN-ROOM.md` (sin referencias textuales al nombre anterior).
H3, H5 y N3 quedan pendientes para un issue.

Incidente de lectura en este seguimiento: al abrir el registro de humo citado
se incluyó por error su sección con una salida de OpenSees. No se reproducen
ni se usan esos valores; esta sesión queda vetada permanentemente para
implementar o corregir los métodos afectados según §5.1.
Auditoría J4 (2026-10-20): el log de la sesión Codex vetada
01a15b2a-feb4-7742-9b48-e970bcb01e49 se conserva fuera del repo, en acceso
restringido, con sha256
259be28dee01d6e7afe356f43569b09c96cbbdae83ca46b10541d24266ec0fe5. Fue una
lectura accidental y no se usó después. El veto cubre solo la barra axial y
la celosía elástica. El log de OpenCode r4 no leyó el archivo.

No se encontró contraparte en inglés del protocolo. Se añade una entrada breve
al changelog, que ya registra cambios documentales. La modificación de
`docs/CLEAN_ROOM.md` está expresamente autorizada por MAM-855; no se modifican
`clean-room/**`, workflows, código, fixtures ni `docs/DETERMINISMO.md`.
