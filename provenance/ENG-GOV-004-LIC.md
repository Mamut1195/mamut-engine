# ENG-GOV-004 · Metadatos de licencia AGPL-3.0-only (MAM-56)

- Autor: J2 (cambio mecánico de metadatos; sin código).
- Decisión: Jonathan, 2026-10-15 (vía J0): AGPL-3.0 + licencia comercial de MAMPRO SAS; el Engine no es propietario.
- Cambios: `LICENSE` raíz = copia literal del texto AGPL-3.0 ya presente en `bindings/python/LICENSE` (SHA-256 0d96a4ff…abcb0, idéntico a https://www.gnu.org/licenses/agpl-3.0.txt); `[workspace.package] license = "AGPL-3.0-only"` y `license.workspace = true` en los 14 crates `crates/engine-*` que no declaraban licencia (engine-api y bindings/python ya lo hacían); README con aviso breve de licencia comercial disponible de MAMPRO SAS.
- Sin cambios: vendor/mamut-workflows (Apache-2.0), pyproject/NOTICE de bindings/python (ya conformes).
- Titular del copyright: MAMPRO SAS (Jonathan, 15-oct). spikes/adr-002-la pasa de MIT a AGPL-3.0-only. Pendiente (ENG-GOV-004): texto de la licencia comercial y contacto (marcador explícito; no hay contacto de MAMPRO SAS en el repo ni en Notion). No se añade texto legal.
- Clean-room: sin fuentes externas de código.
