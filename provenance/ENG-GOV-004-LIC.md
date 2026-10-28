# ENG-GOV-004 · AGPL-3.0-only license metadata (MAM-56)

- Author: J2 (mechanical metadata change; no code).
- Decision: Jonathan, 2026-10-15 (via J0): AGPL-3.0 + MAMPRO SAS commercial license; the Engine is not proprietary.
- Changes: root `LICENSE` = verbatim copy of the AGPL-3.0 text already present in `bindings/python/LICENSE` (SHA-256 0d96a4ff…abcb0, identical to https://www.gnu.org/licenses/agpl-3.0.txt); `[workspace.package] license = "AGPL-3.0-only"` and `license.workspace = true` in the 14 `crates/engine-*` crates that did not declare a license (engine-api and bindings/python already did); README with a brief notice of a commercial license available from MAMPRO SAS.
- Unchanged: vendor/mamut-workflows (Apache-2.0), pyproject/NOTICE in bindings/python (already compliant).
- Copyright holder: MAMPRO SAS (Jonathan, 15-oct). spikes/adr-002-la changes from MIT to AGPL-3.0-only. Pending (ENG-GOV-004): commercial license text and contact (explicit placeholder; there is no MAMPRO SAS contact in the repo or in Notion). No legal text is added.
- Clean-room: no external code sources.
