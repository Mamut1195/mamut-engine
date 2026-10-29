# Clean-room log · ENG-SFC-005

- Ticket / PR: ENG-SFC-005 · MAM-96; PR #247.
- Author(s): Codex gpt-6.1-sol high (package.json, example.html, README,
  test-package.mjs, changelog) · J2 helper (this provenance, README note on the
  commercial licence text) · 2026-10-29 (America/Guayaquil).
- Role: packaging of the existing ES/wasm facade; no new numerical implementation.
- Implemented contract: private ES package `mamut-engine-wasm` 0.1.0 at the root of
  `crates/engine-wasm` (exports `.` → `js/main.js`, `./worker.js` →
  `js/worker.js`, `./wasm` → `pkg/web/engine_wasm.js` + `.d.ts`) and the
  minimal test page `js/example.html`, which loads a repository fixture, runs it
  and shows JSON, with a visible link to the corresponding source
  (AGPL-3.0 §13) and an "experimental, no es producto" notice.
- Permitted sources used: own files of this repo only: `crates/engine-wasm/`
  (README, js/, Cargo.toml version), `scripts/build-wasm.sh` (output names),
  `LICENSE`, `bindings/python/docs/GUIA_USUARIO.md` (wording of the PENDING
  commercial-licence note), `provenance/TEMPLATE.md` and other provenance files
  for the format. Ticket MAM-96 in Linear (acceptance criteria).
- Own derivations: none (no equations).
- Third-party code copied: none. No dependencies are added; the package declares
  no `dependencies*` and no install/lifecycle scripts.
- Licences: AGPL-3.0-only, holder MAMPRO SAS (ADR-001). Commercial licence
  offered via mampro.proyectos@gmail.com; its text is PENDING (ENG-GOV-004).
- Validation (measured 2026-10-29, Node v22.19.0, offline, without `pkg/` built):
  `node crates/engine-wasm/js/test-package.mjs` → 44 checks PASS, 0 FAIL, exit 0.
  Mutations, each restored afterwards, all detected (exit 1): removing the
  "experimental" notice; adding `postinstall`; adding `dependencies`; adding
  `js/test-node.mjs` to `files`; adding `.env` to `files`; duplicated entry in
  `files`; licence MIT; removing the source URL from the page; export to a
  missing file; removing the `.wasm` from `files`.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni código de derivados o rutas prohibidas para este ticket."
- Clean-room reviewer: **PENDING (J4)**.

## DESCONOCIDO / notes

- npm registry (public or organisation) and final package name: DESCONOCIDO.
  The package stays `private: true` and is not published.
- Package built from this head: DESCONOCIDO. No wasm was compiled for this
  change; `js/test-node.mjs` and the browser test were not run against it.
- Browser behaviour of the page: DESCONOCIDO (not run).
- The crate has no LICENSE file of its own and `files` does not include one; to
  distribute the package under AGPL the licence text should travel with it.
  DESCONOCIDO / decision for Jonathan before any publication.
- The crate README (included in `files`) already carried local machine paths in
  its build section before this change; not modified here.
- CI does not run `test:package`; adding it touches `.github` (Jonathan only).
- Harness and Windows: not applicable.
