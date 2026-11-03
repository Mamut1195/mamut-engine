# ADR-001: foundational decisions

- **Task:** ENG-GOV-001
- **Status:** Proposed, pending Jonathan's approval

## Context

The project needs to record its foundational decisions on identity, repository visibility, license, ownership and external contributions. The provisional name is **MAMUT Engine** and the repository is `Mamut1195/mamut-engine`.

On **2026-10-08**, a dual license **AGPL-3.0 + commercial license** was recommended and adopted as the initial decision; Jonathan **confirmed it on 2026-10-15** as AGPL-3.0-only + a commercial license from MAMPRO SAS (the Engine is not proprietary; copyright holder: MAMPRO SAS). This ADR records that decision already made; its proposed status does not supersede or revoke that decision. The other outstanding items require approval or definition.

The [clean-room protocol](../CLEAN_ROOM.md) governs work on the engine. The AGPL-3.0 text is in `LICENSE` and the notice in `NOTICE`; the commercial license text remains pending in **ENG-GOV-004**. This document contains no legal text.

On **2026-11-03**, Jonathan authorized **PUBLIC publication** of `Mamut1195/mamut-engine` under the existing **AGPL-3.0-only** license. The confirmed sanitization plan preserves the entire old repository privately, including PRs, issues and logs, and publishes filtered history at the original repository URL. That decision is confirmed despite this ADR's proposed status. Completed publication and current remote visibility have not been verified here; this authorization does not establish a release or integration.

## Decision

1. Use **MAMUT Engine** as the provisional name. Final name and brand: **PENDING**.
2. Preserve the entire old repository privately, including PRs, issues and logs, and publish filtered history at the original `Mamut1195/mamut-engine` URL under the existing **AGPL-3.0-only** license, as authorized by Jonathan on **2026-11-03**. This records the plan; its execution and current remote visibility remain unverified here.
3. Adopt the dual license **AGPL-3.0 + commercial license**, recommended on **2026-10-08** and confirmed by Jonathan on **2026-10-15** (AGPL-3.0-only; holder MAMPRO SAS). AGPL text in `LICENSE` and notice in `NOTICE`; commercial license text: **PENDING**, in ENG-GOV-004.
4. Copyright holder: **MAMPRO SAS** (confirmed by Jonathan on 2026-10-15).
5. Require a CLA from external contributors, according to **ENG-GOV-007** and the [contribution guide](../../CONTRIBUTING.md). The holder must be able to distribute contributions under AGPL and under a commercial license. The CLA text remains **PENDING** legal review.
6. Apply the [clean-room protocol](../CLEAN_ROOM.md) to contributions.

## Consequences

- The documentation will use the provisional name until the final name and brand are defined.
- Public publication with private preservation of the entire old repository and filtered history at the original URL is authorized by Jonathan (2026-11-03); execution and remote visibility have not been verified here.
- The parent manages backup, history and secrets auditing, actual publication, and runner and public repository configuration.
- Dual licensing requires external contributions to be covered by a CLA allowing the holder to distribute them under both arrangements.
- The copyright holder is MAMPRO SAS (confirmed on 2026-10-15).
- The AGPL-3.0 text is in `LICENSE` and the notice in `NOTICE`. The MAMPRO SAS commercial license contact is mampro.proyectos@gmail.com; only the commercial license text remains pending (ENG-GOV-004).
- Contributions must comply with the clean-room protocol and the CLA requirement for external contributors.

## Outstanding items

- [ ] Jonathan's approval of this ADR.
- [ ] Define the final name and brand.
- [x] Public publication and sanitization plan authorized by Jonathan under the existing AGPL-3.0-only license (2026-11-03): preserve the entire old repository privately, including PRs, issues and logs, and publish filtered history at the original URL.
- [x] Copyright holder: MAMPRO SAS (2026-10-15).
- [x] MAMPRO SAS commercial license contact: mampro.proyectos@gmail.com.
- [ ] Commercial license text: **PENDING**, ENG-GOV-004.
- [ ] Complete ENG-GOV-007: legal review of the CLA text (`docs/CLA.md` already exists).
