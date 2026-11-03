# Clean-room log - ENG-CI-PUBLIC-20261103

- Work item: PublicCI; no ticket ID supplied.
- Repository: `Mamut1195/mamut-engine`.
- Source base supplied by the user: `c64d9977d62f539a3a61e69a078bf4d8bc5526df`.
- Intended isolated branch supplied by the user: `t3/public-ci-runner-migration`.
- Author: Codex; requested model `gpt-6.1-sol`, reasoning effort `high`.
- Date: 2026-11-03, America/Guayaquil.
- Role: implementer of CI, synthetic validator regression and documentation changes;
  no numerical methods.
- Status: coordinator reports literal application through corrected round four;
  final evidence-only update drafted from supplied provenance and coordinator evidence.
  No patch was applied or tested by this author. Independent root review is PENDING.

## Author command and mode

The user explicitly authorized read-only patch drafting as an alternative to
the normal workspace-write author mode in `docs/CLEAN_ROOM.md` section 10.
The coordinator may apply the literal patch and validate it; implementation
decisions and corrections remain with this clean author.

For round three, the user explicitly authorized replacing only the historical
dependency block in `clean-room/tests/run.sh` with three self-contained synthetic
regression cases, updating only the historical-test statements in
`clean-room/README.md`, and recording the scope and evidence in this log.
The historical commit and report blob will not exist in the public history.
The checker script, lists, other tests and vendor content are outside this
round's authorized changes.

The coordinator reports that the malformed third-round proposal was not applied:
README removal contexts contained damaged accents from ASCII stdin transport,
and a provenance hunk omitted the beginning of an existing line. The corrected
round-four block, README and provenance patch was applied literally.
This final patch updates evidence only.

Launch attestation supplied by the coordinator, not independently observed by
this author:

```text
codex exec --ignore-user-config -c web_search=disabled -c sandbox_workspace_write.network_access=false --disable apps -s read-only -m gpt-6.1-sol -c model_reasoning_effort=high -C C:\Users\Jonnathan\.t3\worktrees\mamut-engine\t3-8f03b855 --json --output-last-message <launcher-managed-temp-author-round3.patch> -
```

The coordinator reports literal local checkout sources supplied through prompt
stdin; the launcher writes the response file. The original clean author run
was thread `01a1a1cb-48f7-78e2-b4f2-566175eb72a2`, exit 0, with skill-file reads
blocked by policy and no external calls or checkout writes. The coordinator's
first workspace-write run was rejected and stopped untouched; the user then
explicitly authorized read-only patch drafting. No bypass occurred.

This author used the supplied local source text without checkout writes,
permission elevation, installs, downloads, network calls, web tools or apps.
In the original author run, attempts to read local planning and verification
skill instructions were rejected by process-creation policy; neither file was
read. Subsequent patch drafting, including this final evidence update, used only
the supplied text and made no tool invocations. No validation command executed
by this author.

## Permitted sources used

The user supplied literal checkout contents for:

- `.github/workflows/rust.yml`
- `.github/workflows/clean-room.yml`
- `.gitattributes`
- `README.md`
- `docs/adr/ADR-001-fundacionales.md`
- `docs/CLEAN_ROOM.md`
- `provenance/ENG-CI-PUBLIC-20261103.md`
- `provenance/TEMPLATE.md`
- `changelog.d/README.md`
- `scripts/changelog-check.sh`
- `rust-toolchain.toml`
- `.github/CODEOWNERS`
- `NOTICE`
- `COPYRIGHT`
- `clean-room/tests/run.sh` (round three)
- `clean-room/README.md` (round three)

The user's task instructions supply the publication and sanitization
authorization, allowed paths, requested runner/toolchain behavior, validation
constraints, and coordinator/parent attestations recorded below.
No other repository, online documentation, historical report, oracle output
or third-party source code was consulted. The supplied base and branch metadata
were not independently checked with Git.

Own derivations: three original synthetic validator regression cases, derived
only from the supplied test script and its explicitly synthetic examples.
All new fixture text and figures are invented; no Engine fixtures or numerical
algorithms changed.
Third-party code copied: none.
Comparison with oracle: no.

## Candidate behavior

All three existing jobs use standard GitHub-hosted `ubuntu-24.04`.
Job IDs, triggers, concurrency, timeouts, read-only permissions, pinned checkout
SHA and `persist-credentials: false` are retained. No action, cache, release,
deployment or publishing automation is added.

Each job creates its own randomly named directory directly beneath the resolved
`RUNNER_TEMP`, with isolated `HOME`, `CARGO_HOME` and `TMPDIR`. The Rust job also
places build output there. Its setup captures the preinstalled rustup binary
directory and `RUSTUP_HOME` before isolating `HOME`, preserving proxy and toolchain
resolution without copying credentials or configuration files.

The Rust job installs exactly `1.99.0` with the minimal profile, rustfmt, clippy
and `wasm32-unknown-unknown`, then checks the exact rustc and cargo release
versions. Existing fmt, clippy, workspace tests, documentation and WebAssembly
checks remain. Clean-room and protected-review do not install Rust.

Final cleanup runs with `always()`. It resolves the target, requires its parent
to equal the resolved `RUNNER_TEMP`, checks the job-specific directory prefix
and removes only that directory. It neither kills processes nor wipes the
runner temporary directory or runner command files. Hosted disposal handles
the remaining runner lifetime and preinstalled rustup state.

Trusted BASE validator/list selection, including the existing bootstrap path,
and validator invocations are unchanged. Checker-script changes belong to the parent
and are outside this patch.

The executable protected-review gate is unchanged: protected changes require
a real `APPROVED` review by `Mamut1195` on the latest `PR_HEAD`. GitHub does not
allow authors to approve their own PRs. A PR authored by `Mamut1195` therefore
cannot satisfy this gate; it must remain failing until a legitimate PR from a
distinct authorized contributor receives the owner's real latest-head approval.
Owner merge or chat approval is not a substitute for that review.

The second round adds `* text=auto eol=lf` to `.gitattributes`, preserving both
existing engine-bench `linguist-language=JSON` rules exactly. This extends LF
checkout policy to files Git detects as text, making fixture bytes reproducible
across future clean checkouts. It does not rewrite existing CRLF worktrees,
renormalize files, regenerate references or change fixture logic.

Parent-reported root cause: with `core.autocrlf=true`, a tracked engine-api
pdelta_diaphragm E3 JSON fixture had `i/lf w/crlf`; CRLF normalization matches
the Git blob bytes. This author did not run hash tests. The parent manages LF
restoration in its main checkout; unrelated files and edits in this worktree
are untouched.

Jonathan confirmed PUBLIC publication and the sanitization plan under the
existing AGPL-3.0-only license on 2026-11-03: preserve the entire old repository
privately, including PRs, issues and logs, and publish filtered history at the
original repository URL. README and ADR record authorization and plan without
claiming completed remote operations, observed visibility, release or
integration. The parent manages backup, history/secrets auditing, publication,
and runner/public configuration.

The explicitly authorized removal of the visibility pending item from
clean-room section 12 is retained; section 10 is unchanged. Remaining brand,
legal, CLA and ADR approvals stay pending. License files and Cargo metadata
are unchanged. Second-round changes were limited to `.gitattributes`, README,
ADR-001 and this log; the initial workflow and CLEAN_ROOM changes are retained.

The third round replaces the test block that formerly read a historical report
with `git cat-file`/`git show`, extracted an obtained output, and failed CI when
the historical object was absent. That dependency originally blocked running
the full suite in clean author and coordinator sessions.

Its replacement always runs three synthetic rejection cases: a complete
invented report with metadata, inputs, expected output and obtained output;
then the isolated obtained output with and without a trailing period.
This preserves three regression cases. The coordinator's valid Ubuntu run
completed 144 tests with zero failures on the c64d997 candidate.
No real report text, values, outputs or history were accessed or used.
Expected rejection and metadata allowance policies, `check.sh`, lists and
every test outside that block are unchanged. The coordinator compared the exact
prefix and suffix of `tests/run.sh` against c64d997 after LF normalization;
both matched. The README removes the obsolete history requirement.
The parent's separate MAM938 work adds five vendor tests; those tests are
not included or integrated in this candidate.

No MAM ID was supplied. No changelog fragment is needed: the supplied
`scripts/changelog-check.sh` requires one only for changes under `crates/` or
`bindings/`, neither of which is modified.

## Validation evidence and pending work

This author ran no candidate checks. The coordinator reports an initially
CLEAN worktree at `c64d9977d62f539a3a61e69a078bf4d8bc5526df`, branch
`t3/public-ci-runner-migration`, then literal application of the initial
six-file patch (two workflows, README, ADR-001, CLEAN_ROOM section 12 and this
log). These reports do not constitute independent Git verification by this
author.

| Check | Supplied coordinator result for initial applied patch |
| --- | --- |
| Local actionlint v1.7.12 on both workflows | EXIT 0; no diagnostics |
| Python stdlib extraction of inline and multiline run blocks, parsed with `C:\Program Files\Git\bin\bash.exe -n` | PASS: all 17 blocks |
| Semantic harness | PASS; details below |
| Applied scope review and `git diff --check` | Reviewed; EXIT 0 |
| `cargo fmt --all -- --check` | EXIT 0 on Windows host Rust 1.99.0; formatter only |
| Clippy, workspace tests, documentation and WebAssembly checks | NOT RUN by coordinator |

The semantic harness reports all three jobs on `ubuntu-24.04`; unchanged
triggers, concurrency, permissions, checkout uses/SHA and persist-credentials;
exact text equality of the owner-review environment/script and trusted-base
selection/checks; all five Rust commands unchanged; no self-hosted, Contabo,
`/opt/rust`, `pgrep` or kill operations; exact 1.99.0 setup/components/target;
and job-prefix/realpath-bounded cleanup. License, Cargo, toolchain, checker and
CODEOWNERS text were unchanged. Its first byte comparison falsely flagged
LICENSE because worktree CRLF differed from blob LF; the coordinator verified
normalization matches the blob, corrected the text comparison and reran PASS.
No LICENSE change or candidate failure occurred.

The formatter check used `RUSTUP_TOOLCHAIN=1.99.0`, `CARGO_NET_OFFLINE=true`
and a unique `CARGO_TARGET_DIR` outside the checkout, after parent confirmation
of host standard-library readiness. It validates formatting only.

The coordinator additionally reports literal application and path review of
the second-round patch, limited to `.gitattributes`, README, ADR and this log,
followed by `git diff --check` EXIT 0. Workflow code remained unchanged from
round one; actionlint v1.7.12 on both workflows, Bash syntax checks on all
17 run blocks, static semantics and offline Rust 1.99.0 formatting remain
reported PASS. These are supplied coordinator results, not author executions.

Dynamic cleanup boundary tests in Git Bash used the literal extracted scripts
for all three jobs: 15 cases PASS. For each job, removal of its own direct
temporary directory succeeded; `RUNNER_TEMP` itself, an outside path and a
wrong prefix were rejected; unset `CI_JOB_TEMP` was a no-op. Only synthetic
temporary directories were used, and no runner processes were killed.
This verifies the guards locally, not execution on hosted runners.

LF policy validation is reported PASS by the coordinator. `git check-attr`
reported `text: auto` and `eol: lf` on workflow and engine-api fixture paths,
and `text: set`, `eol: lf`, `linguist-language: JSON` on the engine-bench
fixture path; both original linguist rules remained identical. In a synthetic
temporary repository with `core.autocrlf=true` and the current literal
`.gitattributes`, adding a CRLF text fixture and a binary fixture containing
NUL/CRLF, then running `git checkout-index -f`, produced LF text and preserved
the binary bytes. This did not rewrite the current checkout and does not prove
candidate workspace tests pass.

The parent independently reports the earlier baseline CRLF fixture failure
resolved by LF checkout; further Windows schema and temporary cleanup baseline
issues remain tracked by the parent. These are not candidate engine-test results.

| Check | Supplied coordinator status at final evidence handoff |
| --- | --- |
| Effective `.gitattributes` values and synthetic LF checkout with `core.autocrlf=true` | Coordinator reports PASS; text normalized to LF and binary bytes preserved |
| Application through corrected round four and scope review | Applied literally; allowed nine-file scope checked; `git diff --check` EXIT 0 after round four |
| Protected diff: Cargo metadata/lockfile, toolchain, LICENSE, NOTICE, COPYRIGHT, checker, vendor, crates and bindings | EXIT 0; unchanged |
| Workflow checks | actionlint v1.7.12 on both workflows EXIT 0; all 17 Bash run blocks PASS; semantic harness rerun after last code changes PASS |
| Cleanup, LF policy and formatting | 15 actual cleanup boundary cases PASS; LF checks PASS; offline Rust 1.99.0 formatting PASS |
| Full clean-room suite in Ubuntu WSL | PASS: 144 tests, 0 failures; EXIT 0 |
| Supplementary Git Bash Windows suite | FAILED/incomplete; details below |
| Clippy, workspace cargo tests, documentation and WebAssembly checks | NOT RUN by coordinator |
| Hosted rustup/network installation and Actions execution | NOT RUN |
| Live owner approval and remote visibility | NOT TESTED |
| Independent clean-room review and reviewer signature | PENDING; parent/root will arrange independent review |

The coordinator ran the full suite locally in WSL distro Ubuntu, using installed
`/usr/bin/bash`, Git and sha256sum, from a fresh LF-only temporary snapshot of
the current `clean-room/check.sh` and `tests/run.sh`, without `.git` or original
history. With `CI=true`, `GIT_CONFIG_NOSYSTEM=1` and an empty
`GIT_CONFIG_GLOBAL`, `bash <snapshot>/tests/run.sh <snapshot>` completed
`144 tests, 0 failures`, EXIT 0. All three new cases passed:
`or_incident_report_synth`, `or_incident_obtained_dot_synth` and
`or_incident_obtained_nodot_synth`. This is the main local validation gate.
No forbidden historical blob was accessed.

An earlier WSL launcher piped the script through PowerShell and appended CR
to the snapshot argument, causing `cd` failure and 144 tests with 167 failures.
That launcher run FAILED and is invalid validation, not a candidate defect or
a pass. The coordinator changed only the launcher to execute the same LF script
file by path with `wsl -d Ubuntu -- bash /mnt/c/.../validate-cleanroom-ubuntu.sh`;
the valid 144/0 run above followed without a candidate code edit.
Logs were retained outside the checkout.

The supplementary Windows suite did not pass. Preexisting `vendor_ok` expected
0 but got 1; reproduction with the same TMPDIR showed Windows sha256sum emitting
`hash *./LICENSE` where the unchanged checker expects `hash  ./LICENSE`.
The guard was not weakened. The run later stalled at preexisting
`vendor_dir_symlink` (`ln -s ../..`), with `ln.exe` identified. No broad process
kills occurred; root permission to end only this owned supplementary test
remains pending. Its status is FAILED/incomplete.

All reported checks were offline and local, with no network access or dependency
installs; this author executed none. Local staging and commit follow this final
evidence patch and its diff check; the commit receipt and any subsequent
`changelog-check.sh` result will be handed off separately. No commit SHA or
completed commit is claimed here.

The parent reports Actions were disabled; no CI minutes were consumed.
This author did not inspect or change remote settings, visibility, branches,
PRs, reviews or Actions. Review, publication and configuration remain with the
parent. No hosted CI success, publication, release, merge or integration is
asserted.

## Declaration and review

I did not consult source code of OpenSees, Frame3DD, Stabileo, their derivatives
or any prohibited path for this work. I did not consult oracle reports or
outputs, copy third-party code, or implement numerical methods.

Clean-room reviewer: **PENDING**; no independent review or signature supplied.
