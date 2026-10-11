# ADR-001: decisiones fundacionales

- **Tarea:** ENG-GOV-001
- **Estado:** Propuesto, pendiente de aprobación de Jonathan

## Contexto

El proyecto necesita registrar sus decisiones fundacionales sobre identidad, visibilidad del repositorio, licencia, titularidad y contribuciones externas. El nombre provisional es **MAMUT Engine** y el repositorio es `Mamut1195/mamut-engine`.

Jonathan decidió el **2026-10-08** adoptar una licencia dual **AGPL-3.0 + licencia comercial**. Este ADR registra esa decisión ya tomada; su estado propuesto no sustituye ni revoca esa decisión. Los demás puntos pendientes requieren aprobación o definición.

El [protocolo clean-room](../CLEAN_ROOM.md) rige el trabajo del motor. El texto de licencia y el aviso legal viven solo en Notion, en **ENG-GOV-004**; este documento no contiene texto legal.

## Decisión

1. Usar **MAMUT Engine** como nombre provisional. Nombre y marca definitivos: **PENDIENTE**.
2. Mantener `Mamut1195/mamut-engine` **PRIVADO por ahora**, como decisión recomendada. Publicarlo desde el día 1 o al release: **PENDIENTE** de Jonathan.
3. Adoptar la licencia dual **AGPL-3.0 + licencia comercial**, según la decisión de Jonathan del **2026-10-08**. Texto de licencia y aviso legal: **PENDIENTE (ver Notion)**, en ENG-GOV-004.
4. Titular del copyright: **PENDIENTE**; debe definirse si será Jonathan como persona o una empresa.
5. Exigir CLA a los contribuyentes externos, según **ENG-GOV-007** y la [guía de contribución](../../CONTRIBUTING.md). El titular debe poder distribuir las contribuciones bajo AGPL y bajo licencia comercial. El texto del CLA queda **PENDIENTE** de revisión legal.
6. Aplicar el [protocolo clean-room](../CLEAN_ROOM.md) a las contribuciones.

## Consecuencias

- La documentación usará el nombre provisional hasta que se definan el nombre y la marca definitivos.
- El repositorio permanece privado por ahora; la decisión de cuándo publicarlo sigue pendiente de Jonathan.
- La licencia dual requiere que las contribuciones externas estén cubiertas por un CLA que permita al titular distribuirlas bajo ambas modalidades.
- La titularidad del copyright debe definirse para completar la documentación correspondiente.
- El texto de licencia y el aviso legal se mantienen solo en Notion (ENG-GOV-004), con el marcador **PENDIENTE (ver Notion)** en el repositorio. Esta tarea no crea `LICENSE` ni copia texto de la AGPL o texto legal.
- Las contribuciones deben cumplir el protocolo clean-room y el requisito de CLA para externos.

## Pendientes

- [ ] Aprobación de este ADR por Jonathan.
- [ ] Definir nombre y marca definitivos.
- [ ] Decidir si publicar el repositorio desde el día 1 o al release.
- [ ] Definir el titular del copyright: Jonathan como persona o una empresa.
- [ ] Texto de licencia y aviso legal: **PENDIENTE (ver Notion)**, ENG-GOV-004.
- [ ] Completar ENG-GOV-007: revisión legal del texto del CLA y creación de `docs/CLA.md`.
