# Proceso de CLA para contribuyentes externos

Este documento corresponde a ENG-GOV-007 y describe el proceso de gestión del acuerdo de contribución (CLA). No contiene el texto contractual ni sustituye su redacción o revisión jurídica.

## Por qué existe

El CLA forma parte del modelo de licencia dual de MAMUT Engine: AGPL-3.0 y licencia comercial. Su finalidad es gestionar las autorizaciones necesarias sobre las contribuciones externas para sostener ese modelo. El contenido y los efectos jurídicos del acuerdo quedan pendientes de redacción y revisión por un abogado.

## Alcance

- **Individual:** contribuciones de una persona externa que aporta a título individual.
- **Corporativo:** contribuciones realizadas por cuenta de una empresa u otra entidad. La forma de acreditar la representación y vincular a los autores con la firma corporativa está **PENDIENTE** de definición y revisión jurídica.

La identificación de autores externos y los detalles de aplicación de cada modalidad están **PENDIENTES** de definición.

## Regla previa al merge

**Debe existir una firma registrada antes del primer merge de un autor externo.** Esta regla aplica tanto al alcance individual como al corporativo; en este último caso, deberá verificarse que la firma registrada cubra al autor.

La implementación del control y su verificación automática están **PENDIENTES**. La regla documentada no implica que el control ya esté activo.

## Flujo previsto de firma automática en un PR

1. Al abrir un PR, el flujo automático identifica a los autores externos y consulta si cuentan con una firma registrada que los cubra.
2. Si falta una firma, solicita completar la modalidad individual o corporativa que corresponda y remite al texto contractual, que vivirá solo en Notion.
3. Una vez completada la firma, el flujo registra su resultado y comprueba que cubra al autor.
4. Antes del primer merge de cada autor externo, se verifica la firma registrada. Si falta, el PR no debe integrarse.

Este flujo es una descripción del proceso previsto; su puesta en funcionamiento está **PENDIENTE**.

- **Herramienta de automatización en PR:** **PENDIENTE** de selección y configuración.
- **Lugar de registro de las firmas:** **PENDIENTE** de definición. No se ha establecido un sistema ni una ubicación de registro.
- **Mecanismo de verificación y bloqueo del merge:** **PENDIENTE** de implementación.

## Estado del texto contractual y de la titularidad

- **Texto del CLA:** **PENDIENTE** de redacción y revisión por un abogado.
- **Ubicación del texto contractual:** solo en **Notion**, fuera del repositorio. La página o enlace concreto está **PENDIENTE** de definición.
- **Titular del copyright:** **PENDIENTE** de definición.

La ubicación del texto contractual no determina dónde se registrarán las firmas; ese registro sigue pendiente.

## Criterios de aceptación

Las casillas marcadas corresponden únicamente a aspectos documentados, no a una implementación operativa ni a una aprobación jurídica.

- [x] **DOCUMENTADO:** explicar la finalidad del CLA en el modelo dual AGPL-3.0 y comercial.
- [x] **DOCUMENTADO:** describir el alcance individual y corporativo.
- [x] **DOCUMENTADO:** establecer la regla de firma registrada antes del primer merge de un autor externo.
- [x] **DOCUMENTADO:** describir el flujo previsto de firma automática en PR y señalar que la herramienta está pendiente.
- [x] **DOCUMENTADO:** indicar que el texto contractual vivirá solo en Notion y no en el repositorio.
- [x] **DOCUMENTADO:** declarar pendientes la redacción y revisión jurídica, el registro de firmas y el titular del copyright.
- [ ] **PENDIENTE:** definir cómo se identifican los autores externos y cómo se aplica cada modalidad.
- [ ] **PENDIENTE:** definir y revisar la acreditación de representación y cobertura de autores en la modalidad corporativa.
- [ ] **PENDIENTE:** redactar el texto contractual y completar su revisión por un abogado.
- [ ] **PENDIENTE:** disponer del texto contractual en Notion y definir su enlace.
- [ ] **PENDIENTE:** definir el titular del copyright.
- [ ] **PENDIENTE:** seleccionar y configurar la herramienta de firma automática en PR.
- [ ] **PENDIENTE:** definir y habilitar el lugar de registro de las firmas.
- [ ] **PENDIENTE:** implementar y verificar el control que impide el primer merge de un autor externo sin firma registrada.
