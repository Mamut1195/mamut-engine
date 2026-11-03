# Clean-room log · ENG-INTEGRATION-20261103

- Ticket / PR: documentation-only integration provenance followup; no PR created.
- Author: Codex gpt-6.1-sol/high · 2026-11-03 (America/Guayaquil).
- Role: documentation recorder; no implementation or numerical verification.
- Contract implemented: N/A; additive provenance only.
- Permitted sources used: parent-provided facts dated 2026-11-03 and the exact text of `provenance/TEMPLATE.md` and `docs/CLEAN_ROOM.md` supplied in the prompt. No independent checkout inspection.
- Own numerical derivations: N/A.
- Third-party code copied: none.
- Validation: parent-reported evidence below; no numerical validation performed by this author.
- Comparison with oracle: no; no oracle data consulted.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room document reviewer: **PENDING**. Parent-reported code reviews below do not constitute review of this document.

## Evidence boundary and authorship

All repository, preservation, execution, review and test facts below are attributed to the parent snapshot. This author and the coordinator have not independently verified its remote, execution-log, review or test claims. Paths are supplied pointers, not inspected sources. No tools, shell, network, web, apps, MCP, external sources, other repositories, history or oracle data were consulted by this author.

The parent reports a clean worktree on `t3/public-ci-runner-migration` at `09a6f74369c9f3afb1f14c37cb27f2af729cbb6d`, with previous local docs-public commit `d815b55bf6588598580e6aaffa03b7aed5f33f6c`. This document records a new snapshot; earlier documentation and provenance snapshots remain unchanged.

The user explicitly authorized read-only final-patch authorship as an alternative to the workspace-write author mode in `docs/CLEAN_ROOM.md` §10, retaining its offline configuration. The coordinator supplied this author launcher:

```text
codex exec --ignore-user-config -c web_search=disabled -c sandbox_workspace_write.network_access=false --disable apps -s read-only -m gpt-6.1-sol -c model_reasoning_effort=high -C <sameworktree> --json --output-last-message <executiondir-author.patch> -
```

The prompt is supplied through UTF-8 stdin. This record makes no legal or license interpretation and changes no implementation, numerical fixtures, Rust, CI or licensing.

## Publication and preservation reported by parent

- Current public repository: `Mamut1195/mamut-engine`, repository ID `1444444923`, **PUBLIC**, at the original URL, with the existing AGPL license unchanged.
- Original repository: ID `1412637886`, renamed `Mamut1195/mamut-engine-private-archive-20261103`, **PRIVATE**. Parent reports preservation of its PRs, issues, logs and 84 branches. Archive Actions are disabled.
- The public repository is a new repository identity at the same address. Historical PR numbering must not be treated as transferred.
- New repository Actions are also **DISABLED**, with 0 persistent runners, a read-only workflow token and `canapprove=false`. No hosted CI, Done or approval status is established here.
- Parent supplies the private bundle/mirror evidence directory as `EXEC=C:\Users\Jonnathan\Projects\mamut-engine-execution-20261103-091746` and reports all 260 original refs preserved. Specific bundle/mirror filenames were not supplied.
- Parent reports that the 14 historical excluded blobs have reachable count 0 and that `fsck` passed in a fresh public GitHub clone. This author/coordinator did not inspect that evidence. No prohibited blob contents or hashes are reproduced.

## Composed candidate and ports reported by parent

Identifiers below are reproduced as supplied. Except for the explicitly supplied full local HEAD above, these are abbreviated identifiers, including `748bca...f39`; none was independently resolved. Grouped originals and ports do not establish individual pairings or a new machine history map.

| Group, in supplied order | Reported original/local identifiers | Reported port or supplement identifiers |
| --- | --- | --- |
| Candidate `97c9d90`: reviewed 49 | `033` / `bb3` / `fa35` | `0d9` / `df724` / `ba57` |
| Candidate `97c9d90`: 107 | `bc48` | `6099953` |
| Candidate `97c9d90`: 938 | `2b` / `694` / `be0` | `616` / `5a172` / `be6063` |
| Candidate `97c9d90`: clean-room P2 supplement | `313466b`; supplied abbreviated hash `748bca...f39` | Reviewed R1, per parent; reviewer identity not supplied |
| Workflow | Local `09a6`, corresponding to the full local HEAD recorded above | `ebd005f` |

Parent reports that inventory revision `97c9d90` changed only its recorded lock SHA after comparison found 60 registry tuples/checksums unchanged. This does not establish an actual `Cargo.lock` edit or any additional inventory change. This documentation makes no numerical-engine changes.

## Validation states reported by parent

### Physical offline namespace result at `be6063`

Parent reports a full Cargo gate **PASS** at `be6063`: fmt, clippy, tests, docs, wasm, tamper and features, using Rust `1.99.0`. The reported physical conditions were initially empty HOME/CARGO_HOME/TMP, a nonexistent target, a new kernel network namespace, UID 1000, capabilities 0, NoNewPrivs 1, loopback DOWN and TCP `ENETUNREACH`.

Supplied log pointer: `EXEC\mam938-namespace-validation-fixed.log`, where EXEC is the directory above. Neither this author nor the coordinator opened or read that log.

This is local physical namespace evidence for that revision only. It establishes neither hosted CI nor a PASS for combined candidate `97c9d90` nor an end-to-end outer-launcher PASS.

The outer SUDO launcher was **NOT RUN** locally because passwordless sudo/NOPASSWD was missing. Parent reports no sudo, policy or privilege changes.

### Current combined candidate `97c9d90`

Parent reports **FAILED** `engine-linalg` `determinism_guard` because of the central math import policy in `pile/lateral.rs`. Earlier phases passed, but their detailed list was not supplied. Documentation and WASM were **NOT RUN** in this combined run.

Terra is fixing the issue; no corrective/final SHA or passing rerun was supplied. Independent R2 for 938 and the combined rerun remain ongoing. The prior physical namespace result does not supersede the observed combined failure.

## Bounded fronts and remaining acceptance

- **49 — OPEN:** parent reports earlier native 178 and WASI 12 PASS, and an independent review reporting no P1/P2 before the global guard failure. These are bounded earlier results. Permitted published Reese/Matlock citation/data and JSON CLI exposure remain missing; published acceptance is incomplete.
- **107 — PROVISIONAL:** parent reports R2 closed 3 P2, with 6 native, 31 wasm and 132 hashes PASS. Budgets, hardware and maximum support remain unapproved; external 60 s timeout receipts are missing. No verified maximum, campaign or approval is established.
- **938 — ongoing:** independent R2 and the combined rerun remain pending. No combined success, closure, maximum support or release claim follows from the earlier namespace result.

Parent will append actual final CI receipts later. Pending evidence includes the corrective SHA; combined rerun outcome including documentation/WASM; current review outcomes; outer-launcher and hosted CI receipts if run; 49 citation/data and JSON CLI acceptance; and 107 budget, hardware, maximum-support and timeout acceptance. None is asserted to have occurred.

## Documentation handoff and validation limits

At author handoff, this is a final-response patch only; coordinator validation and the local commit are **planned**, not completed. The planned checks are literal one-additive-path scope, unchanged previously tracked files, supplied identifiers/statuses/limits, Markdown local links and `git diff --check`, followed by exactly one local commit. Evidence pointers are plain paths.

No Cargo, namespace or hosted tests, remote checks, or local diff checks were run by this author for this documentation-only task. Parent-reported tests remain supplied evidence. Actual author/patch/check receipts are recorded outside the checkout; no second deliverable file or changelog fragment is required. No push, PR, merge, rebase, amend or remote operation is authorized by this handoff.
