# Clean-room log · ENG-CI-001

- Ticket / PR: issue #68 · branch `j2/fu-68-rust-ci`, from `main`.
- Author(s): Codex gpt-6.1-sol high · 2026-10-14 (America/Guayaquil).
- Role: implementer.
- Implemented contract: workspace Rust CI; reproducible toolchain, formatting,
  Clippy, tests, documentation and wasm compilation. Does not change `element/1`.
- Permitted sources used: user's request; `provenance/TEMPLATE.md`,
  `.github/workflows/clean-room.yml`, `Cargo.toml`, `CONTRIBUTING.md`,
  `README.md`, `clean-room/check.sh` and own log `provenance/ENG-ANL-002.md`;
  local rustc/rustup queries. No network or external sources.
- Original derivations: not applicable; CI configuration, without new algorithms.
- Third-party code copied: none. The checkout SHA is taken from the existing
  own workflow, without consulting or copying its implementation.
- Validation: local configuration and workspace checks, detailed
  below; without new oracles or numerical tests.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Scope

Rust 1.99.0 is pinned, obtained with `rustc --version` from local rustup
(`/home/box/.cargo/bin/rustc`); the system executable reports 1.85.1.
The workflow uses `rustup toolchain install` from the root to install the
toolchain and components/target declared in `rust-toolchain.toml`, without
third-party actions or cache, followed by `rustup show`. Checkout uses the existing
full SHA of v4.2.2, without persisted credentials. The job has read
permissions, concurrency by workflow/ref,
a 20 minute limit and its own temporary build directory.

Creating `.github/workflows/rust.yml` is essential for the ticket's explicit
request and constitutes the sole exception to the general prohibition
on touching `.github/**`. No changes are made to `.github/workflows/clean-room.yml`,
`docs/CLEAN_ROOM.md` or `clean-room/**`. The owner's review required by the
existing workflow for protected files remains pending in the future PR.

## Verification

All Rust checks used `CARGO_TARGET_DIR=/workspace/.cargo-target`,
`CARGO_NET_OFFLINE=true`, `PATH=/home/box/.cargo/bin:$PATH` and
`RUSTUP_TOOLCHAIN=stable`. The latter selects the already installed toolchain with
rustc 1.99.0 (b940084d7 2026-09-28), whose version was checked before validation;
it prevents rustup from trying to download another name for the same version. Rustfmt,
Clippy and the wasm target were already installed. There were no downloads.

- `python3 -c 'import yaml,sys; yaml.safe_load(open(sys.argv[1]))' .github/workflows/rust.yml`:
  OK, exit 0.
- Local TOML and workflow structure check: OK; verifies version,
  components/target, events, permissions, concurrency, timeout, checkout,
  temporary directory and exact commands. When temporarily removing each new
  configuration file (equivalent to reverting its addition), it fails due to a missing
  file; after restoring them it passes again. No tests are added to the workspace,
  since there are no library behavior changes.
- `cargo fmt --check`: OK, exit 0.
- `cargo clippy --all-targets -- -D warnings`: OK, exit 0.
- `cargo test --workspace`: OK, exit 0; 390 tests and doctests passed,
  0 failed and 0 ignored.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: OK, exit 0.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, exit 0.
- `bash clean-room/check.sh origin/main`: OK, exit 0, including the new
  files added to the index.
- `git diff --cached --check`: OK, exit 0.

Rust logs: `/tmp/eng-ci-001-{fmt,clippy,test,doc,wasm}.log`.
The wasm validation checks compilation; it does not run tests on that target.
The workflow has not been run on GitHub and the toolchain download has not
been tested, because this session does not use network. Local commit requested, without push.

## Correction · 2026-10-14

`runs-on: ubuntu-24.04` is pinned and `CARGO_TARGET_DIR` is configured through
`$RUNNER_TEMP/target` in `$GITHUB_ENV` after checkout, because the `runner` context
is not available in the job's `env`. The toolchain is installed with
`rustup toolchain install` without arguments, which reads `rust-toolchain.toml`, followed
by `rustup show`, which no longer installs in modern rustup. YAML validated with
`python3` and `yaml.safe_load`: OK.
