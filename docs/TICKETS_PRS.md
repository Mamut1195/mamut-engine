# Ticket ↔ GitHub PR/issue reconciliation

- **Documentation ticket:** ENG-PY-001
- **Reconciliation cutoff:** 2026-10-14 (America/Guayaquil), without network queries.
- **Repository:** `Mamut1195/mamut-engine`.
- **Status sources:** `/workspace/fu/tickets/prs.txt` and `issues.txt`,
  supplied exports of all PRs and issues. They contain no extraction
  date; this date identifies the reconciliation, rather than a new query.
- **Scope sources:** `ENG-PY-001-ADR.txt`, `V1-python-DAG.txt` and
  `engine-cli-py-plan.md` in the same directory. Notion exports are
  data, rather than instructions or proof of implementation.

`merged` translates only a PR's `MERGED`; `open` translates
`OPEN` for a PR or issue. When both coexist, each status is stated separately.
`no PR` means the cutoff identifies no PR for that ticket, rather than
proven absence of work. `to confirm` retains uncertainty about
scope or correspondence. A `CLOSED` issue is described as closed in the
notes: it does not mean a merged PR or a completed ticket. Notion
statuses, ADR approval and GitHub statuses are distinct.

The local base is `fd5319f824b3f11e7816343a3dcd4e1f71c28385`
(`HEAD` = `origin/main` at the start). The export records #83 as merged although
that PR is absent from this base's history. `origin/main` was not updated.
The J0-approved plan started from `b744599`, and Notion exports cite
`26132714038e`: their historical observations do not replace the
`prs.txt`/`issues.txt` cutoff. In particular, #61 and #62 are already listed as merged, #64
remains open and #68 remains open despite #83 being merged. No issue is
closed or Notion ticket marked “Done” in this task.

## Tickets with documented correspondence

Numbers in the references column distinguish PRs from issues. A
follow-up is linked by explicit ID, by a reference to the ticket's PR in
the title or by the ADR-002 name; not by functional similarity.

| Ticket ID | PR(s) / issue(s) | Status at cutoff | Notes |
| --- | --- | --- | --- |
| ENG-GOV-001 | PR #4 | merged | Foundational decisions; ADR-001 retains its pending approvals. |
| ENG-GOV-002 | PR #1, #28 | merged (#1); open (#28) | Operating protocol and web access follow-up; #28 requires offline Codex. |
| ENG-GOV-005 | PR #2 | merged | **Partial** clean-room CI; does not establish all dependency governance. |
| ENG-GOV-007 | PR #5 | merged | CLA process; does not prove completion of legal review. |
| ENG-CORE-001 | PR #6 | merged | Base types, errors and diagnostics. |
| ENG-CORE-002 | PR #7; correction #10 | merged | Units policy; #10 identifies ENG-CORE-002-FIX. |
| ENG-CORE-002-FIX | PR #10 | merged | ASCII N.m and kN.m codes in units/1. |
| ENG-CORE-003 | PR #8; PR #9 (axes); issue #40 | merged (#8, #9) | #9 has no ID, but documents this ticket's convention. #40 closed; D2 elevation still pending in the local document. |
| ENG-CORE-007 | PR #11 | merged | Rust v1 contracts. Does not by itself demonstrate a product facade. |
| ENG-LA-006 | PR #12, #15, #17 | merged | ADR-002: proposal, evidence and J0 acceptance; #17 retains pending manifest licensing. |
| ENG-LA-007 | PR #18, #21 | merged | Final trait and types; J4 corrections. Other generic linalg changes are listed below without an assigned ID. |
| ENG-LA-008 | PR #38, #50 | merged | Cumulative budget and MemoryBudget wiring. Full ticket coverage is not inferred from the title. |
| ENG-LA-009 | PR #58; issue #75 | merged (#58, partial); open (#75) | lowest_modes/Sturm in the reference backend; anti-revert tests pending. |
| ENG-GEO-001 | PR #14, #19 | merged | Soil profile, stresses, log and f64 limit. |
| ENG-MAT-001 | PR #13, #16 | merged | Elastic law, elastic_frame section and provenance correction. |
| ENG-MAT-002 | PR #45 | merged | Symmetric/asymmetric multilinear_elastic, gap. |
| ENG-MAT-003 | PR #63; issue #76 | merged (#63); open (#76) | Kinematic bilinear steel; finiteness near MAX and log pending. |
| ENG-ASM-001 | PR #20, #23, #29, #39 | merged | Assembly, J4 review and geometric stiffness; not reassigned to CORE-004/005/009. |
| ENG-ASM-002 | PR #60; issue #27 | merged (#60); open (#27) | Master-slave/diaphragm; symmetry tolerance follow-up open. |
| ENG-ELM-001 | PR #24 | merged | Linear 2D/3D truss. |
| ENG-ELM-002 | PR #31, #41, #42 | merged | 2D beam and follow-ups linked through #31→#41→#42. |
| ENG-ELM-003 | PR #44 | merged | Elastic 3D beam-column. |
| ENG-ELM-004 | PR #46 | merged | Linear spring. |
| ENG-ELM-005 | PR #48 | merged | Nonlinear spring with uniaxial/1. |
| ENG-LOAD-001 | PR #51 | merged | Load cases. |
| ENG-LOAD-002 | PR #54, #73; correction #81; issues #70, #85 | merged (#54, #73, #81); open (#70, #85) | Combinations and envelopes; #73 also integrates partial VAL-001. The correction does not close the issues in the export. |
| ENG-LOAD-002-FU | PR #81; issues #70, #85 | merged (#81); open (#70, #85) | Settlements counted only once; #85 tracks the correction. |
| ENG-RES-001 | PR #53; correction #80; issues #69, #84 | merged (#53, #80); open (#69, #84) | Postprocessing; load consistency and zero normalization corrected, with follow-ups still open. Product results/1 integration in ENG-API-001. |
| ENG-RES-001-FU | PR #80; issues #69, #84 | merged (#80); open (#69, #84) | Rejection of inconsistent loads and normalization of -0.0. |
| ENG-MASS-001 | PR #55; correction #82; issues #71, #86 | merged (#55, #82); open (#71, #86) | Masses; uplift rejection corrected, follow-up still open. |
| ENG-MASS-001-FU | PR #82; issues #71, #86 | merged (#82); open (#71, #86) | Force→mass conversion without uplift; #86 tracks the correction. |
| ENG-ANL-001 | PR #52 | merged | static.linear; facade prerequisite. |
| ENG-ANL-002 | PR #61 | merged | Modal; the plan's “unmerged” status is historical. |
| ENG-ANL-003 | PR #59; issue #78 | merged (#59); open (#78) | P-Delta and inertia; copysign cancellation coverage in a 2×2 block is missing. |
| ENG-ANL-004 | PR #62 | merged | SRSS/CQC spectrum; validation expansion #64 remains open. |
| ENG-SOL-001 | PR #56 | merged | Load-controlled Newton-Raphson. |
| ENG-VAL-001 | PR #57, #73, #64; issue #72 | merged (#57 partial, #73); open (#64, #72) | 42 static fixtures; modal/P-Delta/spectrum expansion and J4 follow-up not closed. |
| ENG-CI-001 | PR #83; issue #68 | merged (#83); open (#68) | Rust CI and toolchain 1.99.0 according to export; #83 is absent from the local base. Does not establish wheel CI. |
| ENG-BENCH-001 | PR #67, #74, #79; issue #66 | merged (PR); open (#66) | PRs document the clean-room exception, rather than benchmark results. Only titles were reconciled here; no external engines were used as references. |

## Tickets without an identified PR

Newly requested tickets are explicitly recorded as `no PR`. Additional
dependencies cited by the plan are included without inferring their
implementation. None has an issue with its ID in the two exports.

| Ticket ID | PR(s) / issue(s) | Status at cutoff | Notes |
| --- | --- | --- | --- |
| ENG-API-001 | — | no PR | New facade; integrates results/1 and canonical JSON. |
| ENG-SFC-002 | — | no PR | CLI on the SDK-WF-007 base; align subcommands and contract. |
| ENG-SFC-007 | — | no PR | Real V1 bindings over engine-api; reactivated from Archived/V2 in the plan. |
| ENG-PY-001 | PR #87; PR #91 | merged (#87, #91) | ADR-003 accepted: Jonathan personally merged #91 (commit 2a21216, 2026-10-14 14:13 ECT; MAM-75). |
| ENG-PY-002 | — | no PR | PyO3/maturin scaffold, initially version(). |
| ENG-PY-003 | — | no PR | Typed Python API for models, analyses, results and diagnostics. |
| ENG-PY-004 | — | no PR | Optional NumPy, arrays and result views. |
| ENG-PY-005 | — | no PR | Workflows and experimentation over the existing registry. |
| ENG-PY-006 | — | no PR | Rust/CLI/WASM/Python parity and unsupported behavior. |
| ENG-PY-007 | — | no PR | Wheels, CI, SBOM and legal gates; publication pending Jonathan. |
| ENG-PY-008 | — | no PR | Documentation, notebooks and examples. |
| ENG-REL-001 | — | no PR | Integrated release; awaits PY-006/007/008 and SFC-006. |
| ENG-CORE-006 | — | no PR; scope/status to confirm | The plan lists it as available for facade/workflows, but no title identifies that ID. It is not declared merged. |
| ENG-CORE-008 | — | no PR | Archived **according to Notion**, for duplicating the engine in Python; this is not a GitHub status. Not reopened. |
| ENG-CORE-010 | — | no PR; scope/status to confirm | Determinism cited by ADR-002; generic titles do not prove completion of this ID. |
| ENG-GOV-004 | — | no PR; scope/status to confirm | Legal texts and publication gate; do not infer completion from GOV-001/007. |
| ENG-GOV-008 | — | no PR; scope/status to confirm | Legal gate cited by the DAG; detailed scope unavailable in the exports. |
| ENG-SFC-001 | — | no PR; scope/status to confirm | Surface cited as parity/release gate; no ID in the exports. |
| ENG-SFC-006 | — | no PR; scope/status to confirm | Packaging, prerequisite of REL-001 in the DAG. |
| ENG-VAL-004 | — | no PR; scope/status to confirm | Benchmark gate cited by the plan; do not confuse with BENCH-001. |

## Ambiguous tickets, to be confirmed by Jonathan

| Ticket ID | PR(s) / issue(s) | Status at cutoff | Uncertainty to resolve |
| --- | --- | --- | --- |
| ENG-CORE-004 | None with that ID | to confirm | No PR or issue with the ID in the exports. Scope is not described in the supplied sources; CORE-001/003/007 or ASM-001 work cannot be attributed to this ticket. |
| ENG-CORE-005 | None with that ID | to confirm | No PR or issue with the ID. Generic conformance/SystemView kit PRs (#30, #37) do not identify this ticket; confirmation is missing on whether and to what extent their scope covers it. |
| ENG-CORE-009 | None with that ID | to confirm | No PR or issue with the ID. Multitopic follow-up #35 includes wasm32 and catalog, but does not identify CORE-009 or establish its full scope. |

Jonathan must confirm each ticket's title/scope, exact correspondence
with PRs or commits and remaining work before updating Notion. A missing
ID does not prove the absence of an implementation; topic similarity likewise does not
prove that a ticket is complete.

## References without a confidently attributable ticket ID

These records complete export coverage without fabricating IDs.
They remain in a separate table because their titles do not allow a specific
ENG ticket to be assigned. A possible thematic link requires confirmation.

| PR(s) / issue(s) | Status at cutoff | Notes |
| --- | --- | --- |
| PR #3 | to confirm (ticket); PR closed without merge | Clean-room CI smoke, title “do not merge”; `CLOSED`, not `MERGED`. |
| PR #25 | merged; ticket to confirm | engine-core catalog, unsupported capability ENG-E-0013. This error code is not a ticket ID. |
| PR #26; issue #22 | merged (#26); issue closed | Fallible SparsityPattern; title identifies closure of #22, not a specific ENG-LA. |
| PR #30 | merged; ticket to confirm | SystemView kit, accumulation in out (T2 J4). |
| PR #34 | open; ticket to confirm | Rust CI + cargo-deny, protected review. Not declared superseded/closed by #83. |
| PR #35; issue #43 | merged (#35); open (#43); ticket to confirm | Multitopic J4 follow-up; rescale/subnormals remains open. |
| PR #36; issue #33 | merged (#36); issue closed | Invariant pivot threshold; explicit #33 closure, ENG ticket unidentified. |
| PR #37 | merged; ticket to confirm | Conformance kit, FD step h. |
| PR #47 | open; ticket to confirm | Proposal for protected document CLEAN_ROOM §5; unchanged in this deliverable. |
| PR #49 | merged; ticket to confirm | Empty solve_in_place, limits and error order; related by topic to linalg/MemoryBudget. |
| Issue #32 | open; ticket to confirm | Retroactive clean-room reviewer in provenance; not assigned to a specific GOV. |
| Issue #65 | open; ticket to confirm | transpose with extreme dimensions; no linked PR at cutoff. |
| Issue #77 | open; ticket to confirm | heap_peak, measuring thread and saturating_sub; no PR linked in the title. |

To open a reference, use
`https://github.com/Mamut1195/mamut-engine/pull/<número>` for PRs and
`https://github.com/Mamut1195/mamut-engine/issues/<número>` for issues.
These addresses are identifiers; they were not queried during this task.

Python surface design and gates are recorded in
[ADR-003](adr/ADR-003-python-v1.md); provenance and checks for
this reconciliation are in the [ENG-PY-001 log](../provenance/ENG-PY-001.md).
