# Clean-room log · ENG-VND-002 · Python import isolation

- Ticket: MAM-938, independent R2 P2 import-shadowing fix.
- Author: offline clean-room author Sol6.1 high; 2026-11-03 (America/Guayaquil).
- Role: implementer; literal patch supplied for coordinator application only in a disposable external copy, never the parent candidate.
- Exact input base: public candidate `97c9d9058fb5720380fb42c665164df17454451c`.
- Permitted sources used: coordinator-supplied current own-code excerpts and implementation contract only. No older original or private B worktree code was used.
- Third-party code copied: none. Attack fixtures contain only synthetic own-code metadata/security test content.
- Author boundary: no tools, filesystem reads or writes, shell, network, browser, Cargo, vendor sources, foreign-engine sources, or oracle outputs were accessed. Source inspection was limited to the supplied text.
- Comparison with oracle: no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING coordinator review.

## Reported bypass and preserved failure record

The coordinator reports that the trusted-base checker enters the PR checkout and invokes its embedded inventory helper with `python3 -`. A PR-root `json.py` containing `raise SystemExit(0)` is imported before inventory validation. Its successful process exit skips the complete vendor inventory check, allowing an invalid root file under `vendor-crates/` to be accepted.

For the exact input base, the coordinator reports that the invalid control without a shadow exited 1 with 235 diagnostic bytes; the corresponding root-module shadow exited 0 with the 15-byte `Clean-room: OK` output. These are coordinator-reported observations, not author-executed results. Preserve the original failed security test as incident evidence; do not relabel it PASS.

Reported evidence locations:

- `EXEC/review-final-composition/shadow-control_no_shadow.log`
- `EXEC/review-final-composition/shadow-cwd_json_shadow.log`

## Bounded trust-boundary findings and change

The supplied checker has one unisolated Python stdin invocation in `verify_vendor_crates`. The supplied test runner has three unisolated Python fixture-generator invocations. This patch adds `-I` to those four invocations. Isolated mode excludes the checkout working directory, `PYTHONPATH`, and user site packages from these imports. The embedded helper body and fixture-generator bodies remain byte-unchanged.

The coordinator's bounded scan reports that all three Python runtime calls in `scripts/check-offline-linux.sh` already use `-I`, checker subprocesses in `scripts/test-vendor-crates.py` already use `-I`, and no Python calls were found in `.github/workflows/clean-room.yml`. Those files are outside this patch. This records the supplied bounded findings, not a global Python security audit or successful execution of those paths.

All existing integrity checks, identifier scans, route checks, source guards, allowlists, and historical/synthetic test blocks are preserved. Scope is limited to `clean-room/check.sh`, `clean-room/tests/run.sh`, and this log.

## Regression contract

Two Linux cases use the existing repository, valid-crate, and expectation helpers. Each builds the valid synthetic crate and checksum before writing and tracking PR-root `json.py` with the payload `raise SystemExit(0)`; each also tracks `vendor-crates/`.

- `vc_cwd_json_valid`: valid inventory with the shadow must exit 0.
- `vc_cwd_json_invalid`: the same shadow plus benign synthetic `vendor-crates/unowned.rs` must exit 1, emit both inventory-guard diagnostics, and omit `Clean-room: OK`. An unrelated exception does not satisfy the diagnostic assertions.

The coordinator reports an independently reviewed baseline of 182 tests and zero failures. This patch adds two cases without replacing existing tests; the expected suite count is 184, pending execution.

## Validation status

No validation was executed by this offline author. Coordinator results and new artifact locations remain PENDING:

- Exact-base patch-forward application and unique replacement matching: PENDING.
- Diff and three-file scope review, including unchanged Python bodies and existing test blocks: PENDING.
- Baseline bypass/control replay in a disposable external copy: PENDING; the earlier reported observations remain recorded above.
- New regression cases and full current public clean-room suite on Ubuntu: PENDING.
- Sensitivity run removing only the production `-I`, while retaining fixture isolation and new tests: PENDING. Required observation: `FAIL vc_cwd_json_invalid: expected 1, got 0`; the valid-shadow case must still exit 0. Record the actual output and suite result before claiming sensitivity success.

Full Rust checks, CI, and CF: NOT RUN. No remote push or PR operation was performed. Test artifacts and disposable copies belong outside the worktree. No unexecuted path is claimed PASS.
