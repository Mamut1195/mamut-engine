# Clean-room log · ENG-GOV-002 · web access incident in reviewer sessions

- Ticket / branch: ENG-GOV-002, `j2/clean-room-incident`.
- Record date: 2026-10-11 (America/Guayaquil). Times are local and come from the Codex session filenames (`~/.codex/sessions/2026/10/{10,11}/`).
- Agent and model: Codex CLI 0.162.0, model `gpt-6.1-sol`.
- Nature: read-only audit of Codex sessions. This record does not correct code; it documents facts and mitigation.

## 1. Fact

J4 reported that Codex, used as a reviewer, attempted to open OpenSees, CalculiX and LAPACK source code on the web. `docs/CLEAN_ROOM.md` §5 prohibits opening OpenSees and CalculiX; LAPACK is not listed in §5, but the protocol calls for reviewing similarity with it (§9). The audit of the 27 mamut-engine Codex sessions recorded up to 07:40 on 2026-10-11 (8 in `/workspace/j4rev/*`, 19 in working clones) confirms the report.

Cause: the Codex sandbox (`workspace-write`, `network_access=false`, or `read-only`) only limits shell commands. The model's web tool (`web__run`) is another route and remained active with `web_search` unconfigured. No session had `web_search` in `~/.codex/config.toml` or `--search`.

## 2. Reviewer sessions with access to prohibited sources

Reviewer role, read-only (`sandbox: read-only`), launched by J4.

| Session (local time) | Directory | What was attempted | What was actually opened |
|---|---|---|---|
| 2026-10-11 00:18 (`01a12966-5ee1`) | `j4rev/engine-pr18` (review of #18, ENG-LA-007) | web search «netlib lapack DSYTF2…» and opening faer's `COPYING.EIGEN.MPL2` | Opening `COPYING.EIGEN.MPL2` failed. The search returned snippets (≤200 words per result) from LAPACK code pages on netlib (`dsytf2.f`, `dsytf2_rook.f`, `dsytrf.f`, `dsytri.f`) and mirrors. No full page was opened. |
| 2026-10-11 07:28 (`01a12af0-1524`) | `j4rev/engine-retro` (retrospective review) | OpenSees `Truss.cpp`; CalculiX `shape2l.f`, `shape3l.f`; LAPACK `dgemm.f` | **Successfully opened:** OpenSees `SRC/element/truss/Truss.cpp` (raw.githubusercontent, 1582 lines) and LAPACK `BLAS/SRC/dgemm.f` (408 lines). CalculiX failed («Failed to fetch»), as did `netlib.org/lapack/double/dgemm.f` and `dhondt.de/ccx_2.23.src.tar.bz2`. `OpenSees/SRC/element/Element.cpp` and a CalculiX mirror also appeared as search results. |
| 2026-10-11 07:28 (`01a12af0-1527`, log C) | `j4rev/engine-retro` | OpenSees `SparseGenRowLinSOE.cpp` (raw, GitHub and doxygen); CalculiX `mafillsm.f`; LAPACK `dsytrf.f`, `dsytf2.f` | **Successfully opened:** CalculiX `src/mafillsm.f` (976 lines) and LAPACK `SRC/dsytf2.f` (610 lines). OpenSees `SparseGenRowLinSOE.cpp` failed via raw, doxygen and the GitHub URL («Failed to fetch» / «Internal Error»). A prior search also returned the OpenSees file index (doxygen) and CalculiX mirrors. |
| 2026-10-11 07:28 (`01a12af0-151b`, log A) | `j4rev/engine-retro` | no web access of its own | none (no web tool calls) |

The remaining mamut-engine reviewer sessions (#13, #14, #20) did not use the web tool. The #24 session (2026-10-11 01:17) only opened documentation from `doc.rust-lang.org` and `docs.kernel.org`.

### 2.1 Retrospective review: correspondence with J4 logs and use in verdicts

> **Note (2026-10-15):** the logs cited in this section (`/workspace/j4rev/cx-retro-{A,B,C}.txt`) no longer exist: the folder was lost on 2026-10-15. Line citations are retained as a historical record and cannot be reverified against those files.

J4's logs (`/workspace/j4rev/cx-retro-{A,B,C}.txt`) only record the `web search:` lines (query or requested URL), not the returned content. Content was verified in the session transcripts (`~/.codex/sessions/2026/10/11/rollout-…01a12af0-{151b,1524,1527}…jsonl`, `web__run` outputs).

| Log | Session | Requested URL (log line) | Result according to transcript | Content |
|---|---|---|---|---|
| A | `…151b` | none | no web calls | — |
| B | `…1524` | `raw.githubusercontent.com/OpenSees/OpenSees/master/SRC/element/truss/Truss.cpp` (5154); search `site:github.com/OpenSees/OpenSees Truss.cpp…` (5086); search `'Truss::getTangentStiff'` (5255) | **returned content** (1582 lines) | OpenSees C++ source code |
| B | `…1524` | `raw.githubusercontent.com/Reference-LAPACK/lapack/master/BLAS/SRC/dgemm.f` (only recorded as a call in the transcript; the log does not list it) | **returned content** (408 lines) | LAPACK Fortran source code (reference BLAS) |
| B | `…1524` | `raw.githubusercontent.com/Dhondtguido/CalculiX/master/src/shape2l.f` (5533, 5562) and `shape3l.f` | failed («Failed to fetch») | none |
| B | `…1524` | `netlib.org/lapack/double/dgemm.f`, `dhondt.de/ccx_2.23.src.tar.bz2`, `docs.rs/…/matown.rs.html`, `docs.rs/…/vec_storage.rs.html` | failed / inaccessible | none |
| C | `…1527` | `…/OpenSees/master/SRC/system_of_eqn/linearSOE/sparseGEN/SparseGenRowLinSOE.cpp` (6417), `opensees.berkeley.edu/…SparseGenRowLinSOE_8cpp-source.html` (6483), `github.com/OpenSees/…SparseGenRowLinSOE.cpp` (6937) | all three failed | none from OpenSees; the search at 5981 returned only index results (e.g. doxygen `files.html`) |
| C | `…1527` | `raw.githubusercontent.com/Dhondtguido/CalculiX/master/src/mafillsm.f` | **returned content** (976 lines) | CalculiX Fortran source code |
| C | `…1527` | `raw.githubusercontent.com/Reference-LAPACK/lapack/master/SRC/dsytf2.f` (and `dsytrf.f`) | `dsytf2.f` **returned content** (610 lines) | LAPACK Fortran source code |

Summary of the facts: OpenSees (`Truss.cpp`), LAPACK (`dgemm.f`, `dsytf2.f`) and CalculiX (`mafillsm.f`) **did return source code** in sessions B and C. The other attempts to open CalculiX and OpenSees failed. J4's belief that CalculiX and LAPACK worked is confirmed (`mafillsm.f`, `dsytf2.f`, `dgemm.f`); OpenSees `Truss.cpp`, which was not on that list, also worked.

Use in verdicts (final text of each log):

- **A** (`cx-retro-A.txt:4182`, repeated at 4195): states «no se realizó comparación externa de similitud con OpenSees/CalculiX/faer/nalgebra/LAPACK». Consistent with the transcript: it opened nothing.
- **B** (`cx-retro-B.txt:5582`, repeated at 5603): «Sin indicios distintivos de copia en la comparación parcial realizada; no se completó el contraste de fuentes de CalculiX/faer». It does not name OpenSees or LAPACK as comparison sources, although the session had opened `Truss.cpp` and `dgemm.f`. The verdict describes the comparison as partial; which part was made against which source is not recorded.
- **C** (`cx-retro-C.txt:6957`, repeated at 6976): cites the URLs of `mafillsm.f` and `dsytf2.f` and states «no detecté similitud sustantiva»; «OpenSees quedó sin verificar por fallo de acceso». **This verdict did rely on CalculiX and LAPACK source code opened by the session**, contrary to the rule in §5 (CalculiX) and without LAPACK being listed as a permitted source.

None of the three logs contains a finding that cites or reproduces lines from those sources. The findings (e.g. #21, #23, #11) are described with the repo's own numerical examples. Whether ideas from those sources influenced the formulation of the findings was not verified.

There is no record of any of these sessions copying text into the repo: they were read-only and wrote no files. What could have happened is that the reviewer incorporated ideas seen in those sources into their findings; the findings traveled as text to the authors of the corrections.

## 3. Author sessions

Of the 19 sessions in working clones, 17 were author sessions (`sandbox: workspace-write`, `network_access=false`) and 2 were read-only (2026-10-10 09:01 and 14:09, no web use). None ran `curl`, `wget`, `git clone` or `gh api` from the shell. The author sessions nevertheless had the web tool available. Six used it:

| Session (local time) | Directory / branch | Web sources opened |
|---|---|---|
| 2026-10-10 20:22 | `mamut-engine` (ENG-GOV-004 memo, no engine code) | `opensees.berkeley.edu/OpenSees/copyright.php` and an OpenSees forum thread on copyright terms (`viewtopic.php?p=122700`); AGPL pages (failed). These are license pages, not code. Scope of §5 («fragmentos de código fuente en foros»): J4 must confirm that the thread contains no code. |
| 2026-10-10 23:16 and 23:20 | `mamut-engine-e` (`j2/eng-core-007-traits`) | publisher pages for the book by McGuire, Gallagher and Ziemian (Wiley, Bucknell) |
| 2026-10-10 23:37 and 23:50 | `mamut-engine-g` (`j2/eng-la-007-linalg-backend`, #18) | crates.io, docs.rs and faer README on codeberg; `nalgebra`, `sprs`, `scirs2-sparse` (crate metadata and documentation) |
| 2026-10-10 23:55 | `mamut-engine-i` (`j2/eng-geo-001-soil-profile`) | publisher page (Cengage) for Das's book |

**No author session opened OpenSees, CalculiX or LAPACK code.** Verified against the URLs of each call and against the outputs: there are no matches for those sources except for the license exception above and LAPACK mentions in faer documentation. The ASM-001 (`mamut-engine-n`) and ELM-001 (`mamut-engine-p`) author sessions did not use the web tool.

Tickets whose author sessions used the web without opening prohibited sources: ENG-CORE-007, ENG-LA-007 (#18) and ENG-GEO-001. No PR is affected by §5 according to this record. Those *reviewed* by sessions with web access to prohibited sources were: #18 and the PRs in the retrospective review (table 2).

### 3.1 OpenCode incident in mamut-contracts (2026-10-09)

On 2026-10-09, around 21:24 ECT, two OpenCode sessions in mamut-contracts used the network: one `webfetch` and three `urllib` calls from bash to `rfc8785.txt`, plus one `websearch` about RFC 9745. These were public RFCs, not prohibited code, but they violate the rule in §10 (reviewers without network or external sources). The 118 OpenCode sessions in engine, j4rev and audit did not use the network. Mitigation: OpenCode permissions and operating system network restriction (§10), **PENDING** Jonathan's decision.

## 4. Mitigation

1. Codex always runs without network or web tool (`docs/CLEAN_ROOM.md` §10). Configuration verified with `codex-cli 0.162.0`: `web_search = "disabled"`, `[sandbox_workspace_write] network_access = false` and `[features] apps = false` in `~/.codex/config.toml`, as in §10 (via command line: `-c web_search=disabled -c sandbox_workspace_write.network_access=false --disable apps`). Tested: with all three, the session has no web tool and the shell still works. `web_search=disabled` alone was **not** sufficient.
2. J4 repeats the review of the sessions in point 2 (if the finding depends on what was seen in OpenSees/CalculiX/LAPACK, it is derived again without network) and confirms the forum thread in point 3.
3. The PRs reviewed by those sessions (#18 and the retrospective review: #6, #7, #8, #10, #11, #19, #21, #23, #25) are validated with an offline provenance review; code is not rewritten unless J4 finds similarity.
4. OpenCode, as a reviewer, follows the same rule (§10): noninteractive invocation on local files, without web tools; Codex and OpenCode never open OpenSees, CalculiX or LAPACK code.
5. Human clean-room reviewer: PENDING (Jonathan's signature).
