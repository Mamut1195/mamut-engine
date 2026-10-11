# Bitácora clean-room · T2 baja «acotar h» (kit de conformidad)

- Ticket / PR: T2 (J4 retro), baja sobre h; rama `j2/conf-h-bounds`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: kit de conformidad de engine-contracts
- Fuentes permitidas usadas: ninguna externa; análisis elemental de error en diferencias centrales (redondeo ~ε/h, truncamiento ~h²).
- Derivaciones propias: el incremento FD debe estar en [1e-9, 1e-2] (`FD_STEP_MIN`, `FD_STEP_MAX`); fuera de ese rango el kit emite diagnóstico en vez de comparar con tolerancia 1e-6. Las muestras existentes (1e-7…1e-5) no cambian.
- Código de terceros copiado: ninguno
- Validación: analítico (h = 1e-12, 1e-10, 0.1, 1, 1e6 detectados; 1e-9, 1e-6, 1e-2 limpios)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)
