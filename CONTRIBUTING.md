# Contributing to MAMUT Engine (provisional name)

Every contribution must follow the [clean-room protocol](docs/CLEAN_ROOM.md).

To implement a method in Rust, follow the [method guide](docs/METHODS_GUIDE.md):
it includes templates, review and a reproducible uniaxial pilot.

External contributors must sign a **mandatory CLA** before their contribution is accepted. This requirement follows from the dual **AGPL-3.0 + commercial license**: the holder must be able to distribute contributions under both AGPL and a commercial license.

The CLA will be documented in [docs/CLA.md](docs/CLA.md), to be delivered with **ENG-GOV-007**. The CLA text is **PENDING legal review**.

Foundational decisions are recorded in [ADR-001](docs/adr/ADR-001-fundacionales.md). The copyright holder remains **PENDING**. The license text and legal notice reside only in Notion, under **ENG-GOV-004**: **PENDING (see Notion)**.

## Changelog

Each PR adds a `changelog.d/MAM-ID.tipo.md` fragment following the
[fragment format](changelog.d/README.md), without editing `CHANGELOG.md`.
PRs opened before this workflow keep their existing `CHANGELOG.md` entry;
they do not need to migrate it to a fragment.

In CI, `bash scripts/changelog-check.sh BASE_REF` compares `BASE_REF...HEAD`:
if `crates/` or `bindings/` changes, it requires a valid new fragment and rejects
changes to `CHANGELOG.md`. CI sets `CHANGELOG_RELEASE=1` when the PR has
the `release` label, exempting the release PR.

The release PR runs `bash scripts/changelog-assemble.sh VERSION DATE`
(preview with `--dry-run`), reviews the changelog and records the removal
of the fragments. `DATE` uses `AAAA-MM-DD`; it may be omitted if
`SOURCE_DATE_EPOCH` is set. Old Unreleased entries move to “Otros
(legado)”. `merge=union` was rejected because it can duplicate and reorder
entries, hiding semantic conflicts; per-PR files avoid
sharing the same edit location.

## Tests

The toolchain is pinned to Rust **1.99.0** in [rust-toolchain.toml](rust-toolchain.toml), with rustfmt, Clippy and the `wasm32-unknown-unknown` target. Use rustup from the root to run the same checks as CI:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace
cargo check --workspace --target wasm32-unknown-unknown
```
