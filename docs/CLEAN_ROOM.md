# Engine clean-room protocol (ENG-GOV-002)

- **Version:** 0.1 (draft for Jonathan's approval)
- **Ticket:** ENG-GOV-002 · **Depends on:** ENG-GOV-001
- **Project name:** provisional (final name UNKNOWN)
- **Notice:** this document is an operational protocol, not legal advice. The legal notice and licensing position reside only in Notion (ENG-GOV-004). Items marked **PENDING** require a decision or external review.

## 1. Objective

To ensure that every line of the engine can be traced to a permitted source (book, paper or a derivation of one's own), with written evidence. The engine is written from scratch in Rust.

## 2. Golden rule

Anyone writing engine code or specifications **must not open** the source code of OpenSees, Frame3DD, Stabileo or any derivative. OpenSees, Frame3DD and Stabileo are only **run** as black boxes (models in, numbers out), internally, from the private sibling repository `mamut-engine-oracles`, under §5.1 and §8.

Q2 assumes the **oracle operator** role in §3 under option A of §5.1: Q2 does not specify or implement methods or write Engine fixtures; J4 audits. The permanent bar resulting from viewing reports or outputs (§5.1) takes precedence over any role change or clean session provided for in §3.

## 3. Roles

| Role | May view | Must not view | Duties |
|---|---|---|---|
| **Specifier** (person or AI) | Permitted sources (§4), Rust contracts | Code of OpenSees, Frame3DD, Stabileo and the paths in §5 | Writes the method specification (equations and citations) and the benchmark fixtures |
| **Implementer** (person or AI) | Rust contract, method specification, method template, fixtures, log | The same as the specifier, and other FEM engines without permissive licensing | Implements in Rust and makes conformance checks and benchmarks pass |
| **Oracle operator** (internal) | Sibling repository, OpenSees and Frame3DD binaries, OpenSees and Frame3DD user manuals for building the model, Stabileo (npm/WASM package run in the harness) and its API documentation | OpenSees, Frame3DD and Stabileo source code. Does not write engine code or fixtures | Runs OpenSees, Frame3DD and Stabileo as black boxes and compares with our CLI; results only in the sibling repository |
| **Auditor** | Everything, only for comparison | — | Runs code similarity checks and reviews logs. Never writes engine code or tells the implementer “how OpenSees does it” |

The same person or session must not combine the oracle operator or auditor roles with the specifier or implementer roles for the same method. Anyone who has viewed prohibited code or paths (§5) must not write specifications, fixtures or code for any method; they may only act as an auditor or oracle operator. Moving from auditor to specifier requires a new, clean session, recorded in the log.

## 4. Permitted sources

- Theory, equations and algorithms in prose or pseudocode: textbooks, papers and published reports. Papers by OpenSees authors are permitted: they are published science; it is the code that is prohibited.
- Derivations of one's own, documented in the method specification.
- Documentation (not code) of dependencies with a verified permissive license.
- Closed-form solutions and published examples, cited in each fixture.
- Reference books: the canonical list is in Notion (`02_PROTOCOLO_CLEAN_ROOM`, §3). **PENDING:** decide whether to copy it to the repository.
- Code listings from books must not be copied unless their license permits it and it is recorded.
- Copyrighted documents (e.g. API RP 2GEO): a lawful copy is obtained, equations are implemented, text is not copied (**PENDING** ENG-GOV-004).
  - **ENG-GOV-004 waiver limited to API RP 2GEO in ENG-GEO-004 (MAM-45).** On 2026-10-20 Jonathan (via J0) confirmed that his copy of API RP 2GEO, 1st edition (2011) with Addendum 1 (2014), is lawful, authorized implementing the t-z and Q-z curves from §8.4.2 (Fig. 2, Table 2) and §8.4.3 (Fig. 3) in our own code, citing section, figure and table, without copying text or figures, including the numerical vertices that define the curves, as code constants, and waived the ENG-GOV-004 memo **only** for this use. The waiver does not cover other sections of API RP 2GEO, other tickets or other copyrighted documents; the PDF must not enter any repository. Evidence: `provenance/ENG-GEO-004.md`; Jonathan's merge of this change constitutes his direct approval (§5.1).
- Third-party code under permissive licenses: by default, it is not copied. Copying a fragment requires Jonathan's approval, an entry in `THIRD_PARTY_NOTICES` and a note in the log.
- Linear algebra dependencies: only with a permissive license accepted by `cargo-deny` (ENG-GOV-005).

## 5. Prohibited sources and paths (must not be opened)

- The `OpenSees/OpenSees` repository, its forks and derivatives (Xara, OpenSeesPy, Tessera, `openseesjs`…), its developer wiki and source code fragments in forums.
- Frame3DD source code: it must not be opened or read; only running its external binary is permitted under §5.1.
- `lambdaclass/stabileo` (crate `dedaliano-engine`) and `node_modules/stabileo-*`: opening or reading their code, including the contents of `node_modules/stabileo-*`, remains prohibited. Installing and running Stabileo in the dedicated harness in the private repository `mamut-engine-oracles` is permitted under §5.1.
- Within MAMUT: `mamut-structures/src/stabileo-adapter`, `motor-fem/engine`, `/workspace/opensees-web/spike/` and the part of `motor-geo` that uses groundhog.
- Other engines with GPL, LGPL, AGPL or unknown licenses, or no license (CalculiX, code_aster, OOFEM, carapace, anaStruct). Kratos only with approval.
- Numerical linear algebra libraries whose code must not be consulted or copied: LAPACK, Eigen, faer and nalgebra (prose/theory only; dependencies are governed by ENG-GOV-005).
- The OpenSees command manual **as a specification**: it does not define format or names. Only the oracle operator uses it.
- Code listings by Davis (*Direct Methods for Sparse Linear Systems*, CSparse): the prose is read, the code is not copied.

The machine-readable list will reside in `clean-room/forbidden_paths.txt` and `clean-room/forbidden_identifiers.txt` (**PENDING**, to be created under ENG-GOV-005, which consumes them in CI).

### 5.1 Exception: external comparative benchmark (ENG-BENCH-001) — approved

Approval record: Jonathan (account Mamut1195) personally merged PR #74 on GitHub (commit 2613271, 2026-10-13 19:11 ECT), and confirmed on 2026-10-14 10:53 ECT (via J0) that this merge constitutes his direct approval; the shared account prevents him from approving as a reviewer, so his merge is the evidence of direct approval.

Background (does not constitute approval under §5.1; relayed messages alone do not count): on 2026-10-13 J0 relayed that Jonathan Torres (Mamut1195) wrote «APRUBO» (sic) at 18:16 ECT and confirmed it at 18:22 ECT in J0's chat.

Reference: issue #66. This exception provides an exemption from §5 for the described use of OpenSees, Frame3DD and Stabileo; §8 continues to govern validation.

Approval record for the extension to Frame3DD and option A: Jonathan's personal merge (account Mamut1195) of this PR #178 on GitHub constitutes his direct approval (commit and date: recorded upon merging).

Background (does not constitute approval under §5.1; relayed messages alone do not count): Jonathan's decision relayed by J0 (BENCH-001, MAM-253, 2026-10-20). Origin of option A: [provenance/ENG-BENCH-001-SMOKE-VPS.md](../provenance/ENG-BENCH-001-SMOKE-VPS.md), which does not mention Frame3DD. Reference note: ENG-VAL-006 / MAM-855.

- The benchmark resides exclusively in the private sibling repository `mamut-engine-oracles`, in a dedicated harness (§8). No directory or workflow for this benchmark is permitted in `mamut-engine`. It only compares displacements, internal forces/reactions, periods and computation times **a posteriori**.
- OpenSees and Frame3DD (GPL, release 20140514 or later) are run as external binaries/executables. The operator pins a specific version and the SHA-256 hash of each binary for each campaign and records them in the sibling repository before running them; changing the version requires recording it again with its hash and an audit by J4. Stabileo is distributed as **npm/WASM (AGPL)**, not as a binary: it is loaded in the dedicated harness and runs in a separate process, never in the `node_modules` of `mamut-engine`, `mamut-structures` or any MAMPRO repository or package, except in the dedicated harness in the private repository `mamut-engine-oracles`, the only place where it may be installed. Its internal use requires review of the AGPL license (ENG-GOV-004).
- They must not be linked, packaged or distributed with the Engine or with MAMPRO packages; the WASM files must not be redistributed. Their code must not be read or copied: the prohibition on opening source code in §5 remains in force.
- **Option A:** Q2 runs the oracles in §5.1: OpenSees and Frame3DD, and Stabileo when ENG-GOV-004 authorizes its use (until then Stabileo remains pending ENG-GOV-004), as oracle operator (§2/§3), in an isolated session, and J4 audits. J2 and J0 only receive “differs in case X”, without values. **Only Q2** may use the OpenSees and Frame3DD user manuals and Stabileo API documentation to build models; never as an Engine specification. Their input scripts (Tcl/JS/Python) remain in the sibling repository and never enter Engine repositories.
- Outputs and reports are archived in the `results/` folder of the harness in the private repository `mamut-engine-oracles`. Only Q2 and J4 (for auditing) access `results/` (outputs and reports); J2, J0 and authoring sessions (specifier or implementer) never do. The GitHub account `Mamut1195` and the agent machine are shared: isolation is a **declarative control plus an audit by J4**, not a technical access barrier.
- Anyone who views a benchmark report or its outputs is **permanently barred from implementing or correcting that method**, even in subsequent tickets or new sessions. A discrepancy only permits opening a “differs in case X” ticket, without values; the correction must arise from an original analytical derivation, in a clean session, by an author who is not barred.
- No code, data, report or output from these programs may feed into Engine specifications, tests, fixtures or implementation. Their indirect use to adjust tolerances, select or discard test cases or calibrate default values is also prohibited.
- No method is deemed validated by this comparison. Engine validation uses versioned analytical solutions and published examples (§8), never outputs from these programs.

## 6. Own names and format

- Own JSON format (`mamut.engine.model/1`); no Tcl commands or script structure.
- OpenSees identifiers must not appear in schemas, CLI, MCP or `kind`. Initial list:
  - Commands: `geomTransf`, `uniaxialMaterial`, `nDMaterial`, `eleLoad`, `timeSeries`, `fix`, `equalDOF`, `rigidDiaphragm`, `recorder`, `numberer`, `integrator`, `algorithm`, `analyze`.
  - Elements: `elasticBeamColumn`, `dispBeamColumn`, `forceBeamColumn`, `zeroLength`.
  - Materials: `Steel01`, `Steel02`, `Concrete01`, `Concrete02`, `PySimple1`, `TzSimple1`, `QzSimple1`, `PressureDependMultiYield`.
  - Solvers: `ProfileSPD`, `BandGeneral`, `UmfPack`, `LoadControl`, `DisplacementControl`.
- Models are named by author and physics: `steel.menegotto_pinto`, `concrete.kent_park`, `py.matlock_soft_clay`, `py.reese_sand`.
- Importing `.tcl`/`.py` or exporting scripts to OpenSees: not planned before the ENG-GOV-004 memo (**PENDING**).

## 7. Provenance record

For each ticket, `provenance/ENG-XXX.md` based on `provenance/TEMPLATE.md`. CI (ENG-GOV-005, under development) will reject a PR that touches `crates/` without a log; until it is active and protected, the reviewer checks this manually.

## 8. Verification against free oracles

- **Engine validation (in this repository):** analytical solutions and examples published in books or other permitted sources (§4), versioned as fixtures with `source` and a citation; never outputs from OpenSees, Frame3DD or Stabileo.
- **Supplementary comparison:** OpenSees, Frame3DD and Stabileo as black boxes in `mamut-engine-oracles` (private, internal use), under the same rules as §5.1. The benchmark is not a code, CI or release dependency of the Engine; it has no directories or workflows in `mamut-engine`.
- No method is deemed validated by this comparison.
- Internal use and feasibility of these programs, including review of Stabileo's AGPL license: **PENDING** the ENG-GOV-004 memo.

## 9. Similarity audit

Before each release, the auditor compares the engine code with OpenSees, Frame3DD and Stabileo using a similarity tool and manually reviews flagged cases. Tool and threshold: **PENDING**. The result is archived in a private repository.

## 10. AI agent sessions

- Each implementation session starts with a clean context, in a checkout that contains no prohibited paths.
- Agents that have already viewed OpenSees/Stabileo code (e.g. the one from the 2026-10-07 spike) must not implement the engine.
- A model may have seen code during training: this is mitigated by §9; a lawyer assesses the residual risk (legal review **PENDING**).
- Record the agent and model used in the log.
- **Codex always runs without network access and without a web tool, both as author and as reviewer** (incident recorded in `provenance/ENG-GOV-002.md`). The sandbox only restricts the shell; the model's web tool is independent. With `codex-cli 0.162.0`, both must be disabled:
  - In `~/.codex/config.toml`: `web_search = "disabled"`, `[sandbox_workspace_write] network_access = false` and `[features] apps = false`.
  - On the command line, always: `codex exec -c web_search=disabled -c sandbox_workspace_write.network_access=false --disable apps …`. `web_search=disabled` alone is not enough (verified: the session was still opening pages); with `--disable apps` the session has no web tool and the shell continues to work.
  - Reviewers use `-s read-only` and authors use `-s workspace-write`, both with the above. `--search` must not be used.
  - Before trusting a new session, check that it has no web tool (e.g. ask it to open `https://example.com`: it must respond that it cannot).
- **OpenCode (reviewer) follows the same rule:** it is invoked non-interactively (`opencode run … </dev/null`), only on local checkout files (diff and method specification), without asking it to open URLs and with restricted permissions in its configuration: `"permission": {"webfetch": "deny", "websearch": "deny", "edit": "deny", "bash": "deny"}` (alternative: a strict bash allowlist with only `git diff`, `git show`, `git log`, `rg` and `ls`; nothing that executes code: no `python`, `node`, `curl`, `wget` or `cargo`). **Blocking only `webfetch` and `websearch` is not enough**: bash still has network access. The actual guarantee comes from the operating system: the agent's user may only connect to `token-plan.ap-southeast-1.maas.aliyuncs.com:443` (nftables/iptables by UID or a proxy). This is checked with a `webfetch` and a `python -c "import urllib.request…"`, which must fail. Applying this: **PENDING** Jonathan's decision. The only permitted connection is to the model provider; it does not download or consult external code or documentation.
- **Codex and OpenCode never open OpenSees, CalculiX or LAPACK code** (or any other path in §5), even to “compare similarity”: the similarity audit (§9) is performed by J4 or a person, never by the authoring or reviewing session.
- Neither authors nor reviewers consult external sources: no repositories, forums or online documentation. They work only with what is in the checkout and in the method specification. If a permitted external source (§4) is needed, a person supplies it as text in the specification.
- A violation (a session with web access or one that opened a path in §5) is recorded in the ENG-GOV-002 log with facts: session, role, URLs opened and whether the attempt succeeded.

**Standard instruction (paste at the start of each session):**

```text
You will implement <method> in Rust for this repository, under the clean-room protocol (docs/CLEAN_ROOM.md).
Permitted sources: only those cited in the method specification and in docs/CLEAN_ROOM.md §4.
PROHIBITED: searching for, opening, citing, recalling or reproducing source code of OpenSees, Frame3DD, Stabileo
or any derivative or engine from the paths in §5, and using their command, element,
material or solver names (§6). Do not browse those repositories or forums with fragments of their code.
If you cannot implement something using only the permitted sources, STOP and report the uncertainty;
do not improvise from memory of another engine. Do not copy code from books or crates.
When finished, complete provenance/ENG-XXX.md with sources, derivations and declaration.
```

## 11. PR checklist

The template is in `.github/PULL_REQUEST_TEMPLATE.md`. Summary:

- [ ] Log completed and signed by a reviewer
- [ ] No prohibited paths in the diff or session history
- [ ] Prohibited names test passing (ENG-GOV-005)
- [ ] `cargo-deny` passing
- [ ] CLA signed if the author is external (ENG-GOV-007)
- [ ] Fixtures with `source` and a citation
- [ ] Conformance checks and benchmarks passing on native and wasm
- [ ] Jonathan's explicit “Yes” to the merge

## 12. Pending items and open questions

- Final name and repository visibility (ENG-GOV-001).
- Similarity tool and threshold.
- Legal memo (ENG-GOV-004) and subsequent update of this protocol.
- CLA (ENG-GOV-007) and CI (ENG-GOV-005).
