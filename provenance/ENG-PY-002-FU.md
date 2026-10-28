# Clean-room log · ENG-PY-002-FU

- Ticket / PR: MAM-83 · ENG-PY-002-FU; J4 r22 follow-up on #89. Branch `j2/py-wheel-license`, from `origin/main`.
- Author(s): Codex gpt-6.1-sol high under J2's direction, 2026-10-14 America/Guayaquil.
- Role: implementer.
- Implemented contract: include `LICENSE` and `NOTICE` in the `bindings/python` wheel; no API or analysis contract changes.
- Permitted sources used: user's request; GNU AGPL version 3 text, 19 November 2007, supplied at `/workspace/fu/AGPL-3.0.txt` (complete document); `bindings/python/pyproject.toml`, binding README and root README; `provenance/TEMPLATE.md`; local Cargo and maturin 1.15.0 tools. No network access.
- Own derivations: brief notice aligned with the READMEs; no algorithms or numerical derivations.
- Third-party code copied: GNU AGPL-3.0 legal document copied literally from `/workspace/fu/AGPL-3.0.txt` to `bindings/python/LICENSE`; no third-party code is copied. The document's header allows literal copying and distribution. `bindings/python/NOTICE` is an in-house notice, not the complete commercial contract.
- Validation: offline workspace checks, local wheel build and inspection of its contents and metadata; no new numerical validation.
- Oracle comparison (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Scope

`[project].license-files = ["LICENSE", "NOTICE"]` preserves the expression
`AGPL-3.0-only` and declares the two legal files for the wheel through
PEP 639. The binding README distinguishes the included AGPL text from
ownership, commercial legal text and definitive legal notice pending
in ENG-GOV-004. The commercial license available from MAMPRO SAS is preserved,
as is Jonathan's explicit approval as a requirement for publication on PyPI.

No Rust changes: library `panic!`/`unwrap()`/`expect()` rule N/A.
No changes to `docs/CLEAN_ROOM.md`, `.github/**` or `clean-room/**`.
No publication or push; commit with J2's configured identity.

## Verification

Commands from `/workspace/me45`, with the requested environment:

```bash
export PATH=/home/box/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
export RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu
export CARGO_TARGET_DIR=/workspace/.cargo-target
export CARGO_NET_OFFLINE=true
```

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK, exit 0 |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | OK, exit 0 |
| `cargo test --offline --workspace` | OK, exit 0; 421 tests/doctests, 0 failed, 0 ignored |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace` | OK, exit 0 |
| `cargo check --offline --workspace --target wasm32-unknown-unknown` | OK, exit 0 |
| `bash clean-room/check.sh origin/main` | OK, exit 0, with the new files added to the index |
| `cd bindings/python && /workspace/venv-maturin/bin/maturin build --offline --out /tmp/eng-py-wheel-license` | OK, exit 0 |
| `unzip -l /tmp/eng-py-wheel-license/*.whl` | OK, exit 0; LICENSE and NOTICE present |
| Comparison of the legal files inside the ZIP with those of the binding | OK, identical byte for byte; LICENSE also identical to the supplied source |
| METADATA inspection | OK; version 2.4, `License-Expression: AGPL-3.0-only`, `License-File: LICENSE` and `License-File: NOTICE` |
| `git diff --cached --check` | OK, exit 0 |

Wheel built locally with maturin 1.15.0:
`/tmp/eng-py-wheel-license/mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`.

The listing contains 10 files, including:

```text
34523  mamut_engine-0.1.0.dist-info/licenses/LICENSE
  479  mamut_engine-0.1.0.dist-info/licenses/NOTICE
```

Full listing saved at `/workspace/fu/wheel-license-list.txt` through
`unzip -l /tmp/eng-py-wheel-license/*.whl | tee /workspace/fu/wheel-license-list.txt`.
Command records: `/tmp/eng-py-wheel-license-{fmt,clippy,test,doc,wasm,clean-room,wheel}.log`.
Results: `/tmp/eng-py-wheel-license-results.json`.
No additional platform verification or PyPI publication.
Clean-room review PENDING (J4).
