# MAMUT Engine (nombre provisional)

Nombre y marca definitivos: **PENDIENTE**.

Estado: decisiones fundacionales propuestas, pendientes de aprobación de Jonathan. El repositorio `Mamut1195/mamut-engine` es **PRIVADO por ahora**; mantenerlo privado es la decisión recomendada. Publicarlo desde el día 1 o al release queda **PENDIENTE** de Jonathan.

Licencia dual **AGPL-3.0 + licencia comercial**, decidida por Jonathan el **2026-10-08**. Texto de licencia y aviso legal: **PENDIENTE (ver Notion)**, en ENG-GOV-004. Titular del copyright: **PENDIENTE**.

- [ADR-001: decisiones fundacionales](docs/adr/ADR-001-fundacionales.md)
- [ADR-002: selección de librerías de álgebra lineal](docs/adr/ADR-002-algebra-lineal.md)
- [ADR-003: Python nativo V1](docs/adr/ADR-003-python-v1.md) · [Conciliación ticket↔PR/issue](docs/TICKETS_PRS.md)
- [Protocolo clean-room](docs/CLEAN_ROOM.md)
- [Guía de contribución](CONTRIBUTING.md)

Fachada portable [engine-api](crates/engine-api/README.md): `run()` ejecuta
linearStatic, pDelta, modal y responseSpectrum con salida canónica results/1,
SHA-256 de entrada y cancelación cooperativa. Las limitaciones de los
seguimientos #78, #84, #85 y #86 están documentadas en el README del crate.
