# Clean-room log · ENG-VAL-006

- Ticket / PR: MAM-855 · ENG-VAL-006; documentation only.
- Author(s): Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Role: specifier of the documentation protocol; no Engine implementation.
- Contract implemented: not applicable; amendment of `docs/CLEAN_ROOM.md`, §2/§3/§5/§5.1/§8/§9/§10.
- Permitted sources used: ticket request; `docs/CLEAN_ROOM.md`;
  `provenance/TEMPLATE.md`; clean-room declarations from
  `provenance/ENG-BENCH-001-SMOKE-VPS.md`; Jonathan's decision
  (BENCH-001, MAM-253, 2026-10-20), provided in the request.
- Own derivations: not applicable; no methods or calculations introduced.
- Third-party code copied: none.
- Validation: documentation review and `bash clean-room/check.sh origin/main`:
  `Clean-room: OK` (exit 0). `git diff --check`: OK.
  Cargo and Rust tests omitted under the explicit exception for this task.
- Comparison with oracle (sibling repo, informative): no; no oracles were run
  and none of their values, reports or outputs were incorporated.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo
  ni derivados para este ticket. Tampoco consulté código de CalculiX o LAPACK.
  No usé resultados de estos programas para especificar, probar o implementar
  métodos del Engine. Se trabajó sin red ni fuentes externas."
- Clean-room reviewer: J4 r1 on 5974c9c (APTO with follow-up).

Frame3DD (release 20140514 or later) is incorporated as an external executable
exclusively for `mamut-engine-oracles`, with the binary version and hash pinned,
without reading sources or linking, packaging or distributing with the Engine or MAMPRO.
Option A reserves operation of OpenSees and Frame3DD for Q2 in an
isolated session, under J4 audit; J2 and J0 only receive «difiere en caso X», without values.
The review follow-up aligns §2 and §3 with Frame3DD and option A; specifies
that the operator pins version and hash per campaign and that a version change
requires a new record and J4 audit.

J4 r1 follow-up on PR #178 (2026-10-20): H1 separates direct approval
through Jonathan's personal merge from the background relayed by J0 and cites
the smoke record only as the origin of option A. H2 extends option A
to the oracles in §5.1, with Stabileo pending ENG-GOV-004, and reserves
`results/` for Q2 and J4 to audit. H4 adds Frame3DD to the template
declaration. N1 records the previous review. N2 renames the log to
`provenance/ENG-VAL-006-CLEAN-ROOM.md` (without textual references to the previous name).
H3, H5 and N3 remain pending for an issue.

Reading incident in this follow-up: opening the cited smoke record
accidentally included its section with an OpenSees output. Those values are
neither reproduced nor used; this session is permanently barred from
implementing or correcting the affected methods under §5.1.
J4 audit (2026-10-20): the log of the barred Codex session
01a15b2a-feb4-7742-9b48-e970bcb01e49 is retained outside the repo, with restricted
access, with sha256
259be28dee01d6e7afe356f43569b09c96cbbdae83ca46b10541d24266ec0fe5. It was an
accidental read and was not used afterward. The ban covers only the axial bar and
the elastic truss. The OpenCode r4 log did not read the file.

No English counterpart of the protocol was found. A brief entry is added
to the changelog, which already records documentation changes. Modification of
`docs/CLEAN_ROOM.md` is expressly authorized by MAM-855; no changes are made to
`clean-room/**`, workflows, code, fixtures or `docs/DETERMINISMO.md`.
