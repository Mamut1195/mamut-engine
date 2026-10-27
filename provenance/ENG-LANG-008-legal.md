# ENG-LANG-008 — Legal documents in English (legal group)

- Ticket: MAM-844 (ENG-LANG-008), legal group.
- Date: 2026-10-27.
- Base: `origin/main`, `a677e530273e3c9dff6cfc85078e7272e1135554`.
- Authorization: Jonathan (2026-10-27 14:50 ECT, via J0) authorized preparing
  this translation in a PR; J4 reviews and Jonathan merges.
- Sources: the existing Spanish texts only. Work and verification were offline.
  No third-party material and no new obligations, scope, deadlines or figures.

## Scope

| File | Lines before | Lines after |
| --- | ---: | ---: |
| `docs/CLEAN_ROOM.md` | 149 | 149 |
| `docs/CLA.md` | 60 | 60 |

File names are unchanged. No repository link targets an anchor in either file,
so no explicit HTML anchors were needed. No script or test parses their content;
`clean-room/check.sh` skips `docs/CLEAN_ROOM.md`.

## Preserved

The meaning is preserved: no clause, item, obligation, deadline, definition or
figure was added or removed. A script compared each file with its base: numbers,
link targets, URLs, heading levels, list items, table rows and inline code
spans are identical. Fenced code blocks are identical except the standard
instruction in docs/CLEAN_ROOM.md (§10), whose text is translated to an
equivalent English wording. The OpenSees identifier list in §6 and the
quoted original wording «APRUBO» (sic) are kept verbatim.
