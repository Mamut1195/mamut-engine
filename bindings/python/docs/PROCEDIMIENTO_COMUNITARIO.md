# Procedimiento comunitario para proponer un método

> **validación humana requerida**. Un prototipo o un notebook que pasa no
> convierte una propuesta en método oficial ni acredita su uso profesional.

El recorrido es: **prototipo Python + fuentes/fixtures + test reproducible →
ticket de método → implementación Rust clean-room → revisión humana**.

1. **Preparar el prototipo en Python.** Expresar entradas completas, hipótesis,
   unidades, ejes y límites como documentos versionados. Usar la API de
   `mamut_engine` para validar y ejecutar; `experiments.sweep` permite preparar
   variantes y conservar errores/procedencia. Python organiza experimentos y
   presenta resultados: **no duplica el cálculo** del motor, ni añade un solver,
   ensamblador o método registrable mediante funciones Python. Para una propuesta
   todavía no implementada, documentar la especificación y los casos esperados.
2. **Adjuntar fuentes, fixtures propios y un test reproducible.** Identificar
   autoría y permiso de las fuentes, derivar las expectativas analíticas y
   declarar tolerancias con justificación. Incluir los documentos de entrada,
   comando de ejecución offline, versión, contratos, estado, diagnósticos y
   huellas. Un resultado emitido por el mismo motor no basta como expectativa
   independiente. Los notebooks de esta entrega evalúan una fórmula cerrada
   solo como comprobación didáctica; no implementan un método de producción.
   No incluir secretos, rutas de máquina ni salidas con datos personales.
3. **Abrir el ticket de método.** Describir el problema concreto, las hipótesis,
   contrato propuesto, evidencia y contraejemplos; distinguir PENDIENTE y
   DESCONOCIDO de lo demostrado. Adjuntar el prototipo y el test, y proponer
   criterios de aceptación medibles, unidades, errores esperados y límites.
   La discusión de una propuesta no cambia `capabilities()` ni registra métodos.
4. **Implementar en Rust mediante clean-room.** Una vez acordado el alcance,
   desarrollar el cálculo en el núcleo Rust a partir de la especificación y
   fuentes permitidas, con trazabilidad de las derivaciones. Integrar contratos,
   diagnósticos y registro de capacidades donde corresponda. El binding Python
   transporta esos contratos y conserva la implementación numérica única.
5. **Someter a revisión humana.** Revisar fuentes y procedencia, hipótesis,
   resultados analíticos, tolerancias, errores y reproducibilidad. Mantener como
   `Experimental` lo que así declaran `schemas/engine-input/1.json` (línea 5) y
   `crates/engine-api/src/methods.rs` (hoy `nonlinearStatic`, `soilProfile` y
   `consolidation1d`), distinguir `Stable` de `Validated` (pDelta: Stable;
   ENG-BENCH-002; Validated pendiente de ENG-VAL-006) y dejar
   DESCONOCIDO lo que no tenga evidencia. La revisión técnica, la aceptación del
   método y su publicación son decisiones distintas. Documentar quién revisó y
   qué casos cubrió antes de declarar una validación.

Los tres notebooks de la [guía](GUIA_USUARIO.md) ilustran la parte reproducible
con un voladizo ya disponible. No autorizan incorporar nuevos métodos ni
sustituyen la prueba con un usuario real.

Antes de aceptar contribuciones externas, consultar
[CONTRIBUTING.md](../../../CONTRIBUTING.md) y el proceso
[CLA.md](../../../docs/CLA.md): se exige una firma registrada antes del primer
merge externo; texto contractual, registro y automatización siguen PENDIENTES.
La licencia dual y el contacto del titular están en la
[guía de usuario](GUIA_USUARIO.md#licencia-y-contribuciones).
