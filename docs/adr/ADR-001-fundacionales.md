# ADR-001: decisiones fundacionales

- **Tarea:** ENG-GOV-001
- **Estado:** Propuesto, pendiente de aprobación de Jonathan

## Contexto

El proyecto necesita registrar sus decisiones fundacionales sobre identidad, visibilidad del repositorio, licencia, titularidad y contribuciones externas. El nombre provisional es **MAMUT Engine** y el repositorio es `Mamut1195/mamut-engine`.

El **2026-10-08** se recomendó y adoptó como decisión inicial una licencia dual **AGPL-3.0 + licencia comercial**; Jonathan la **confirmó el 2026-10-15** como AGPL-3.0-only + licencia comercial de MAMPRO SAS (el Engine no es propietario; titular del copyright: MAMPRO SAS). Este ADR registra esa decisión ya tomada; su estado propuesto no sustituye ni revoca esa decisión. Los demás puntos pendientes requieren aprobación o definición.

El [protocolo clean-room](../CLEAN_ROOM.md) rige el trabajo del motor. El texto AGPL-3.0 está en `LICENSE` y el aviso en `NOTICE`; el texto de la licencia comercial sigue pendiente en **ENG-GOV-004**. Este documento no contiene texto legal.

## Decisión

1. Usar **MAMUT Engine** como nombre provisional. Nombre y marca definitivos: **PENDIENTE**.
2. Mantener `Mamut1195/mamut-engine` **PRIVADO por ahora**, como decisión recomendada. Publicarlo desde el día 1 o al release: **PENDIENTE** de Jonathan.
3. Adoptar la licencia dual **AGPL-3.0 + licencia comercial**, recomendada el **2026-10-08** y confirmada por Jonathan el **2026-10-15** (AGPL-3.0-only; titular MAMPRO SAS). Texto AGPL en `LICENSE` y aviso en `NOTICE`; texto de la licencia comercial: **PENDIENTE**, en ENG-GOV-004.
4. Titular del copyright: **MAMPRO SAS** (confirmado por Jonathan el 2026-10-15).
5. Exigir CLA a los contribuyentes externos, según **ENG-GOV-007** y la [guía de contribución](../../CONTRIBUTING.md). El titular debe poder distribuir las contribuciones bajo AGPL y bajo licencia comercial. El texto del CLA queda **PENDIENTE** de revisión legal.
6. Aplicar el [protocolo clean-room](../CLEAN_ROOM.md) a las contribuciones.

## Consecuencias

- La documentación usará el nombre provisional hasta que se definan el nombre y la marca definitivos.
- El repositorio permanece privado por ahora; la decisión de cuándo publicarlo sigue pendiente de Jonathan.
- La licencia dual requiere que las contribuciones externas estén cubiertas por un CLA que permita al titular distribuirlas bajo ambas modalidades.
- El titular del copyright es MAMPRO SAS (confirmado el 2026-10-15).
- El texto AGPL-3.0 está en `LICENSE` y el aviso en `NOTICE`. Solo quedan pendientes el texto de la licencia comercial y el contacto de MAMPRO SAS (ENG-GOV-004).
- Las contribuciones deben cumplir el protocolo clean-room y el requisito de CLA para externos.

## Pendientes

- [ ] Aprobación de este ADR por Jonathan.
- [ ] Definir nombre y marca definitivos.
- [ ] Decidir si publicar el repositorio desde el día 1 o al release.
- [x] Titular del copyright: MAMPRO SAS (2026-10-15).
- [ ] Texto de la licencia comercial y contacto de MAMPRO SAS: **PENDIENTE**, ENG-GOV-004.
- [ ] Completar ENG-GOV-007: revisión legal del texto del CLA y creación de `docs/CLA.md`.
