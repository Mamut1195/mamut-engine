# ADR-003: V1 native Python package

- **Task:** ENG-PY-001
- **Status:** Accepted (Jonathan, 2026-10-14)
- **Date:** 2026-10-14 (America/Guayaquil)

## Context

MAMUT Engine needs a local interface for notebooks, scripts and
experimentation without duplicating structural computation. Jonathan's V1 plan
of 2026-10-13 incorporates Python and retains a single numerical implementation
in Rust. It supersedes the previous exclusion of Python bindings for V1; ENG-SFC-007
is reactivated for V1 and ENG-CORE-008 remains archived because it proposed another
engine in Python/NumPy.

The sources are the local exports `ENG-PY-001-ADR.txt` (objective, boundaries
and DoD) and `V1-python-DAG.txt` (architecture, DAG and gates), and the plan approved
by J0, `engine-cli-py-plan.md`, in `/workspace/fu/tickets/`. They are used as
design data. The format follows [ADR-001](ADR-001-fundacionales.md) and
[ADR-002](ADR-002-algebra-lineal.md). Approval of the plan does not imply
approval of this ADR, implementation, merge or publication.

The local baseline for this delivery is `main`/`origin/main`
`fd5319f824b3f11e7816343a3dcd4e1f71c28385`, without a remote update. The
workspace contains 14 crates and does not yet include `engine-api`,
`bindings/python` or `pyproject.toml`. The
[ticket↔PR/issue reconciliation](../TICKETS_PRS.md) records a GitHub snapshot
more recent than some observations in the plan: #61 and #62 are listed as merged,
#64 open and #83 merged in the exports. These statuses do not prove a complete product
API or verified wheel CI.

The [clean-room protocol](../CLEAN_ROOM.md) remains in effect. This task is
exclusively documentation: it specifies boundaries and future verification, without
changing algorithms, contracts, manifests or dependencies.

## Decision

1. **A single Rust core.** The numerical crates own the
   computations, physical validation and official methods. The core does not depend on
   Python, PyO3, NumPy, web clients or Harness. CLI, MCP and WASM also do not
   depend on the Python binding. Local execution does not require a cloud, login,
   Platform or Harness.
2. **Shared facade `engine-api` (ENG-API-001).** It will be the public Rust
   contract for validation, execution, capabilities, schemas, version and error
   explanation. It will own the serde/canonical JSON adaptation and integration of
   `engine-input/1` and `results/1`; the numerical crates will not depend on serde
   as a result of this integration. Python will consume that facade, without turning
   analysis or assembly internals into another public API. The actual binding
   awaits the facade; the scaffold may expose only `version()`.
3. **PyO3/maturin in the same repository.** Propose `bindings/python/`
   as the `mamut-engine-py` crate of type `cdylib`, with PyO3 0.25.1 and
   `abi3-py310`, `pyproject.toml` with maturin and the
   `python/mamut_engine/{__init__.py, _native.pyi, py.typed}` package inside the
   binding. It will be imported as `mamut_engine`; the distribution name
   `mamut-engine` is subject to availability and approval. The binding
   will remain **outside `default-members`** with an explicit primary list.
   The engine's native and wasm builds will retain a selection without Python;
   when incorporating the binding, engine checks must be separated
   from package checks, since `--workspace` also includes members outside
   `default-members`.
4. **Typed API and explicit capabilities.** The proposed interface is
   detailed below. Models, analyses, results and diagnostics retain
   versioned contracts; an absent capability returns `Unsupported`, without
   fallback or a Python solver. Workflows reuse the Rust registry and
   existing SDK (SDK-WF-003), with orchestration outside the solver.
5. **Shared units, axes and traceability.** Public input uses explicit SI,
   rejects NaN/Inf and preserves the original data. The convention remains
   [X-Y horizontal, Z up](../AXES_CONVENTION.md): D2 has Ux/Uy/Rz and D3
   Ux/Uy/Uz/Rx/Ry/Rz. 2D elevation remains pending; there will be no silent
   mapping to X-Z. An adapter must declare and test any
   approved conversion. Results and the manifest will record engine version,
   commit, target, contracts, methods, input hash and diagnostics.
   Canonical JSON and zero normalization are centralized in the facade.
6. **Compatibility separate from the ABI.** The package and facade follow
   semver; each document declares its `schema`. An unknown major version
   is rejected with a typed error. `capabilities()` identifies the schemas and
   methods actually supported. `abi3-py310` proposes CPython 3.10 or higher;
   by itself it does not guarantee compatibility of contracts, platforms,
   architectures or other interpreters. Each distributed combination must
   be verified in ENG-PY-007.
7. **Memory, GIL, progress and cancellation.** Rust retains ownership of
   models, matrices and results during computation. Python conversions and
   views will have explicit lifetimes; NumPy will be optional in
   ENG-PY-004, with shape/dtype validation and copies when safe access cannot
   be guaranteed. `run` releases the GIL during Rust work
   where safe, without accessing objects or executing Python callbacks
   inside the solver. Progress and cancellation are coordinated in the facade
   between steps; adapter callbacks require reacquiring the GIL outside
   computation. Cancellation produces `Cancelled`; partial results
   are identified and are not presented as success. Library errors
   are fallible and typed, without `panic!`, `unwrap` or `expect`.
8. **Dual license and human publication.** The package retains
   **AGPL-3.0 + commercial license from MAMPRO SAS**, recommended on 2026-10-08,
   ratified on 2026-10-13 and confirmed by Jonathan on 2026-10-15
   (AGPL-3.0-only; holder MAMPRO SAS), and within the scope requested for this
   ADR. The generic SDK's Apache-2.0 does not extend to the engine; PolyForm is not
   adopted either. The AGPL allows commercial use subject to its obligations. The
   legal texts, formal ownership, CLA and commercial offering require
   review in ENG-GOV-001/004/007/008; this ADR does not draft legal texts or
   resolve the historical outstanding ownership item in ADR-001. **Nothing will
   be published on PyPI without Jonathan's express approval.**

### V1 interface and responsibility

This matrix is a specification awaiting implementation, not a list
of Python functions already available.

| Python interface | Rust responsibility / contract | Ticket and boundary |
| --- | --- | --- |
| `version()` | Engine identity, version and build | ENG-PY-002: minimal scaffold; ENG-API-001: common identity |
| `capabilities()` | Supported methods, dimensions and schemas | ENG-API-001 / ENG-SFC-007; verified capabilities only |
| `schema(kind)` | Schemas `engine-input/1` and `results/1` | ENG-API-001; unknown major version rejected |
| `validate(model)` | Input validation and domain diagnostics | ENG-API-001 / ENG-PY-003; validation does not amount to proving stability |
| `run(input, *, cancel=None, progress=None)` | Execution and status through the facade | ENG-SFC-007 / ENG-PY-003; explicit `Unsupported`, `NotConverged`, `Unstable`, `Cancelled` |
| `explain(code)` and `EngineError(code, entities, cause)` | `ENG-E` catalog, entities and cause | ENG-API-001 / ENG-PY-003; no loss of context |
| Typed models, cases, analyses and `Results` | Rust contracts and `results/1`, with views over results | ENG-PY-003/004 and ENG-RES-001; no recomputing results in Python |
| Optional NumPy matrices and arrays | Rust data with documented shape/dtype/lifetime | ENG-PY-004; no unsafe shared buffers |
| Workflows and experimentation | Existing method and workflow registry | ENG-PY-005; no Python scripts running in solvers |

## Rejected alternatives

- **Second engine in Python/NumPy (ENG-CORE-008):** it would duplicate computation,
  validation and maintenance; it remains archived.
- **Direct binding to Rust internals:** it would expose details that may change
  and fragment contracts across Python, CLI, MCP and WASM. Execution of actual scientific
  computations awaits `engine-api`.
- **Mandatory remote service or execution through CLI as the primary Python
  API:** it would add infrastructure or a process protocol to the
  local experience. The CLI remains another interface to the facade.
- **Python in the core or solvers:** it would invert dependencies, affect
  wasm and complicate ownership, GIL and determinism.
- **Exclude Python from V1:** contrary to Jonathan's plan of 2026-10-13.
  Compiling Python to Rust or introducing Lua is not proposed.
- **Apache-2.0/PolyForm for the engine or automatic publication on PyPI:**
  incompatible with the ratified dual license and required approval.

## Consequences

- ENG-PY-001 precedes the ENG-PY-002 scaffold. ENG-SFC-007 needs both the
  scaffold and ENG-API-001; this facade depends on ENG-ANL-001 and
  ENG-CORE-006. The status and scope of the latter cannot be inferred from a PR
  with an ID in the exports: see the reconciliation.
- After the bindings, ENG-PY-003 enables ENG-PY-004 (NumPy/results) and
  ENG-PY-005 (workflows). ENG-PY-006 verifies parity after both;
  ENG-PY-008 also documents models and workflows after both.
- ENG-PY-007 may prepare wheels after the scaffold, without publishing.
  Parity, CLI/WASM interfaces, benchmarks and legal gates are release
  conditions; they do not prevent designing or experimenting with the scaffold. The integrated
  V1 ships through ENG-REL-001 after ENG-PY-006/007/008 and
  ENG-SFC-006, without presenting a partial delivery as a complete V1.
- abi3 wheels are proposed for manylinux2014 x86_64/aarch64, macOS universal2
  and win_amd64, installable without Rust and without internet once downloaded.
  The effective matrix and wheel CI remain pending. The merge of #83
  according to the export does not by itself substantiate that gate.
- ENG-PY-006 must test Rust/CLI/WASM/Python parity with analytical
  test cases, justified tolerances, identical units/signs, hashes and
  error states. Code from OpenSees, Stabileo or derivatives will not be consulted or copied
  nor will they be used as oracles in this delivery.
- ENG-PY-007 must provide executed CI, checksums, SBOM,
  `THIRD_PARTY_NOTICES` and reviewed AGPL/commercial texts before
  distribution. Legal compliance or wheel compatibility is not claimed
  by selecting PyO3/maturin.
- Open GitHub follow-ups remain as such, including
  #69–#72, #75–#78 and #84–#86 even where merged corrections exist. Exposing
  capabilities requires evaluating those outstanding items and actual
  conformance, without inferring it from the presence of code.

## Acceptance (2026-10-14)

Approval record: Jonathan (account Mamut1195) personally merged
PR #91 on GitHub (commit 2a21216, 2026-10-14 14:13 ECT); the shared
account prevents him from approving as a reviewer, so his merge
is the evidence of direct approval (same precedent as
CLEAN_ROOM §5.1 / #74 → #79). Linear ticket: MAM-75 (ENG-PY-001).

Background (relays alone do not count): on 2026-10-14 at 13:57 ECT
J0 communicated that Jonathan approved ADR-003 (MAM-75 / ENG-PY-001).

## Outstanding items

- [x] Jonathan's approval of this ADR — personal merge of PR #91 (commit 2a21216, 2026-10-14 14:13 ECT; MAM-75). Independent clean-room review pending.
- [ ] Confirm ambiguous tickets and ENG-CORE-006 in the reconciliation.
- [ ] Implement and verify the facade, bindings, types, memory and cancellation.
- [ ] Resolve the 2D elevation convention before advertising it as a capability.
- [ ] Verify platforms/CPython, parity and wheel CI with evidence.
- [ ] Complete the legal gates and obtain Jonathan's express approval
      for any publication on PyPI and for the integrated release.
