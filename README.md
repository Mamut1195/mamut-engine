# MAMUT Engine (provisional name)

Final name and brand: **PENDING**.

Status: foundational decisions proposed, pending Jonathan's approval of the remaining items. Jonathan authorized **PUBLIC publication** of `Mamut1195/mamut-engine` on **2026-11-03** under the existing **AGPL-3.0-only** license.

Parent-verified snapshot on **2026-11-03**: the original repository is now the **PRIVATE** `Mamut1195/mamut-engine-private-archive-20261103`, preserving its PRs, issues, logs and branches. A different repository at the original `Mamut1195/mamut-engine` URL holds sanitized history and is now published **PUBLIC**. GitHub Actions is configured by two workflow files under `.github/workflows`. `Rust` (`.github/workflows/rust.yml`) runs on pull requests of types `opened`, `synchronize`, `reopened`, `labeled` and `unlabeled`, and on pushes to `main`. Its job `rust` uses `ubuntu-24.04` with a 20 minute limit, installs Rust 1.99.0 with rustfmt, clippy and the target `wasm32-unknown-unknown`, runs `scripts/changelog-check.sh` on pull requests (setting `CHANGELOG_RELEASE` when the pull request carries the label `release`) and then `scripts/check-offline-linux.sh`. `Clean-room` (`.github/workflows/clean-room.yml`) runs on `pull_request`. Its job `clean-room` uses `ubuntu-24.04` with a 10 minute limit, takes the validator from the base branch when that branch already has `clean-room/check.sh` and `clean-room/tests/run.sh`, and runs those tests plus `clean-room/check.sh`. Its job `protected-review` uses the same runner and limit, reads pull-request reviews, and fails when the pull request changes the protected paths named in that job without a current approval from `Mamut1195` on the latest commit. There is no third workflow file named `protected-review`: that name is a job of `Clean-room`. This paragraph describes those files. It does not record a hosted run result. Historical PRs, issues and commit SHAs from before the migration on 2026-11-03 must be consulted in the private archive, despite reuse of the original URL. See [repository identities and historical lookup](docs/REPOSITORY_HISTORY.md).

License: **AGPL-3.0-only** (text in [LICENSE](LICENSE)), with a **commercial license available from MAMPRO SAS** for those unable to comply with the AGPL. Recommended on 2026-10-08 and confirmed by Jonathan on 2026-10-15 (the Engine is not proprietary). Notice in [NOTICE](NOTICE). Copyright © MAMPRO SAS. Commercial license text: **PENDING** (ENG-GOV-004). Commercial licensing contact: MAMPRO SAS, <mampro.proyectos@gmail.com>.

When the commercial license is needed: [docs/COMMERCIAL_LICENSE.md](docs/COMMERCIAL_LICENSE.md).

- [ADR-001: foundational decisions](docs/adr/ADR-001-fundacionales.md)
- [ADR-002: linear algebra library selection](docs/adr/ADR-002-algebra-lineal.md)
- [ADR-003: native Python V1](docs/adr/ADR-003-python-v1.md) · [Ticket↔PR/issue reconciliation](docs/TICKETS_PRS.md)
- [Clean-room protocol](docs/CLEAN_ROOM.md)
- [Contribution guide](CONTRIBUTING.md)

Portable facade [engine-api](crates/engine-api/README.md): `run()` executes
linearStatic, pDelta, modal and responseSpectrum with canonical results/1 output,
input SHA-256 and cooperative cancellation. The limitations tracked in
[#78](https://github.com/Mamut1195/mamut-engine-private-archive-20261103/issues/78),
[#84](https://github.com/Mamut1195/mamut-engine-private-archive-20261103/issues/84),
[#85](https://github.com/Mamut1195/mamut-engine-private-archive-20261103/issues/85) and
[#86](https://github.com/Mamut1195/mamut-engine-private-archive-20261103/issues/86)
in the private archive are documented in the crate README.
