# Clean-room log · ENG-ANL-006

- Ticket / PR: MAM-11, ENG-ANL-006, `buckling.linear`. No pull request. The ticket is not Done.
- Author: implementation sub-agent, Grok 4.7, 2026-11-03 (America/Guayaquil).
- Role: implementer. The four Euler constants below were derived in this session from the beam equation.
- Contract implemented: library method `engine_analysis::buckling::BucklingLinear`, kind `buckling.linear`. No change to element/1, to the public JSON schemas, or to the error catalog.
- Permitted sources used: the task text; `docs/CLEAN_ROOM.md`; `provenance/TEMPLATE.md`; the in-repo APIs and notes `provenance/ENG-ANL-002.md`, `provenance/ENG-ELM-002.md`, `provenance/ENG-ASM-001-KG.md`, and the existing `LinearSystem` / `lowest_modes` path. No network. No book was opened.
- Own derivations: this note, and `crates/engine-analysis/tests/buckling.rs`.
- Third-party code copied: none. No new dependency.
- Validation: analytical. `source.kind = analytical`. Timoshenko–Gere citation, page, and table: **UNKNOWN**. No published example was invented.
- Oracle comparison (sibling repository, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## What the method solves

`BucklingLinear::run` asks `LinearSystem` for the lowest finite factors of

```text
(K_ff + λ Kg_ff) φ = 0
```

at the geometric reference already stored on the system. `Kg_ff` is the assembled reference matrix, or the producer evaluation at `set_geometric_reference`, reduced by `T_fᵀ Kg T_f`. Accepted and trial displacements are not changed.

The existing dense solver factors the second matrix by Cholesky and accepts only a positive definite metric (review note B9 on MAM-11). This method does not add another eigensolver. It rewrites the same equation as

```text
K_ff φ = λ (−Kg_ff) φ
```

and calls `lowest_modes` on that pair. Consequences, specified here rather than hidden:

- Sign. Element axial force is positive in tension, and `Kg` is linear in that force. A compressive reference makes the bending part of `Kg` negative, so `−Kg` is positive on those terms and `λ > 0` scales the reference compression. A negative diagonal of `−Kg` means the reference is not a compression metric and returns ENG-E-0010. Doubling a uniform compressive reference halves `λ`.
- Singular `Kg`. Rows of `−Kg` that are identically zero (axial DOFs, members with no geometric stiffness) are condensed with the existing Guyan path. The diagnostics keep ENG-E-0006 and ENG-E-0014. The shared solver text still says "mass" for the modal problem and is not edited. This call replaces that wording and names the metric `−Kg`.
- Indefinite `Kg`. If `−Kg` is indefinite but has no negative diagonal, Cholesky of the metric returns ENG-E-0010 or ENG-E-0006. The ENG-E-0010 sentence that said the mass is not positive definite is replaced here with the same statement about `−Kg`. There is no shift and no selection of complex or mixed-sign factors.
- Zero `Kg` and `count > 0`: ENG-E-0010. `count = 0` returns an empty spectrum and does not require a definite metric.
- Physical modes. The requested count is the lowest ascending finite factors. A numerically zero factor is a mechanism or rigid mode of `K` and is counted, not deleted. Shapes satisfy `φᵀ(−Kg)φ = 1` on the free DOFs. The global shape is the homogeneous map `u = T_f φ`, with prescribed components left at zero.
- The public JSON method enum does not include `buckling.linear`. That schema change is not applied. See the execution note `SCHEMA-PROPOSAL.md` beside the report for this task.

## Beam equation

Compression `P > 0`, flexural rigidity `EI`, transverse displacement `w`. Equilibrium of the bent axis, linearized, is

```text
EI w'''' + P w'' = 0.
```

Set `k² = P / EI`. The characteristic equation `r²(r² + k²) = 0` gives

```text
w(x) = A + B x + C sin(kx) + D cos(kx).
```

`k = 0` is the rigid field and is excluded. Then `Pcr = C_factor · π² · EI / L²` with the `C_factor` derived below. These are flexural buckling of one prismatic bar. Axial yielding, local buckling, torsional buckling, and shear were not checked. The numbers are not a code-compliance claim.

### Pinned–pinned

`w(0) = w''(0) = w(L) = w''(L) = 0`. Then `A = D = 0`, `B = 0`, and `sin(kL) = 0`. The lowest positive root is `kL = π`, so `C_factor = 1`.

### Fixed–free

Fixed at `x = 0`: `w = w' = 0`. Free at `x = L`: `w'' = 0` and `w''' + k² w' = 0`. The second free condition collapses identically to `k² B = 0`, so `B = 0`. Then `w'(0) = 0` forces `C = 0`, and `w''(L) = 0` forces `cos(kL) = 0`. The lowest root is `kL = π/2`, so `Pcr = π² EI / (4 L²)` and `C_factor = 1/4`.

### Fixed–fixed

`w(0) = w'(0) = w(L) = w'(L) = 0`. Substituting the general solution and writing `θ = kL` produces

```text
sin(θ/2) · (sin(θ/2) − (θ/2) cos(θ/2)) = 0.
```

`sin(θ/2) = 0` gives `θ = 2πn`. The lowest positive value is `θ = 2π`, so `C_factor = 4`, with shape proportional to `cos(2πx/L) − 1`. The other factor is `tan(φ) = φ` with `φ = θ/2`. Its first positive root is the fixed–pinned root below, so `φ > π` and `θ > 2π`. That family is a higher load. The lowest constant is 4.

### Fixed–pinned

Fixed at `x = 0`, pinned at `x = L`: `w(0) = w'(0) = w(L) = w''(L) = 0`. The moment condition at `x = L` cancels the trigonometric terms in `w(L)` and leaves

```text
tan θ = θ,    θ = kL.
```

Equivalently `f(θ) = sin θ − θ cos θ = 0`. On `(0, π/2)`, `tan θ − θ` starts as `θ³/3 > 0` and the tangent stays above the line, so they do not cross. On `(π/2, π)` the tangent is negative while `θ > 0`. Thus there is no positive root at or below `π`. `θ = 0` is the trivial field. On `(π, 3π/2)`, `f(π) = π > 0`, `f(3π/2) = −1 < 0`, and `f'(θ) = θ sin θ < 0`, so there is exactly one root. The fixture isolates it by 80 bisections with `engine_core::math`. The run on 2026-11-03 gave

```text
θ = 4.4934094579090633
f(θ) = 3.775e-15
C_factor = (θ/π)² = 2.0457485159382953
```

No page or table was used to choose these digits. `Pcr = C_factor · π² · EI / L²`.

## Discrete column and numerical comparison

The fixture is a straight D2 bar on `X`, length `L = 4 m`, `E = 200 Pa`, `A = 2 m²`, `I = 3 m⁴`. These are test scales, not a material. Each element stores `Kg` at axial force `N = −1 N` (tension positive), so the eigenvalue `λ` is `Pcr` in newtons. Eight equal Euler–Bernoulli elements are compared with the continuum root. Four equal elements must lie above that root and above the eight-element result: the coarse Hermite space is contained in the refined one, and the consistent geometric stiffness is the energy `N ∫ (w')² dx`.

Relative tolerance on the eight-element factor: `1e-3`. It is wider than the observed discretization error below and narrow enough that a wrong integer constant (for example 2 instead of the fixed–pinned root) fails. One cubic element, pinned–pinned, is not compared with the continuum root. Its two free rotations and `N = −1 N` give exactly `12 EI / L²`:

```text
K = (EI/L) [[4, 2], [2, 4]]
Kg = −(L/30) [[4, −1], [−1, 4]]
```

The mode `[1, −1]` produces `λ = 12 EI / L² = 450 N`. Absolute tolerance `1e-8`. The same bar with `N = −2 N` returns half of that factor.

Observed eight-element factors versus `C_factor · π² · EI / L²`, from `cargo test --offline --locked -p engine-analysis --test buckling` before the diagnostic print was removed. The assertions remain in the test.

| Case | C_factor | Exact Pcr (N) | 8 elements (N) | ratio |
| --- | ---: | ---: | ---: | ---: |
| pinned–pinned | 1 | 370.1101650408509158 | 370.1222921022010155 | 1.0000327660856025 |
| fixed–free | 0.25 | 92.5275412602127290 | 92.5277318864397671 | 1.0000020602106621 |
| fixed–pinned | 2.0457485159382953 | 757.1523208659984903 | 757.2550398772349354 | 1.0001356649229034 |
| fixed–fixed | 4 | 1480.4406601634036633 | 1481.1988538461778262 | 1.0005121405424588 |

Each mode also satisfies `φᵀ(−Kg)φ = 1` within `1e-8` and `‖(K + λ Kg) φ‖ / scale < 1e-8`. Zero `Kg` and a tensile reference (`N = +1 N`) return ENG-E-0010. `count = 0` on zero `Kg` returns an empty list.

## Evidence record

- claim: for the four end conditions, the eight-element factor of `K + λ Kg = 0` at `N = −1 N` is within `1e-3` relative of `C_factor · π² · EI / L²`, with `C_factor` as derived above. The pinned one-element projection equals `450 N` within `1e-8`.
- candidate: worktree implementation of MAM-11 on base `dc2b5b3a27cfc12cfa71345b1f877b515a04c283`. The commit SHA is recorded in the task report after the local commit.
- state: VERIFIED for that numerical claim, after the buckling test exited 0. This is not code compliance and not a comparison with Timoshenko–Gere.
- gap: book citation UNKNOWN. The public schema and wasm execution of this fixture are outside this verification. Clean-room reviewer PENDING.
- inputs: `E`, `A`, `I`, `L`, and `N = −1 N` or `−2 N` as written above. Source: this derivation.
- method: `buckling.linear`, eigenvalues of `K + λ Kg` through `lowest_modes` on `−Kg`, with Guyan condensation of zero metric rows.
- sources: this note only. Timoshenko–Gere was not used.
- toolchain: host `cargo test --offline --locked` for `engine-analysis` and `engine-assembly`. Rust target `x86_64-pc-windows-msvc`. Exact compiler version and exit codes are in the task report.
- units: SI. `λ` is in newtons because the reference compression is `1 N`.
- tolerances: relative `1e-3` on the eight-element continuum comparison; absolute `1e-8` on `450 N`; residual and geometric norm `1e-8`.
- oracle: the closed beam solution and the bisection of `sin θ − θ cos θ`, independent of the element matrices. The residual `(K + λ Kg) φ` only checks that the solver returned an eigenpair of those matrices.
- execution: `crates/engine-analysis/tests/buckling.rs`.
- comparison: the table above. All four ratios pass `1e-3`. The one-element and tension checks passed in the same test binary.

## Open acceptance box

The Linear criterion "Euler en sus 4 condiciones de borde (Timoshenko–Gere)" stays open. The four cases are in the code from this derivation. The book was not consulted, and its citation stays UNKNOWN. The ticket is not Done.

## Publication in run, Python, and TOML (2026-11-04)

This section records the publication asked by `docs/tickets/04-pandeo-en-run.md`. It does not rewrite the library session above. That session did not change the public schema. This publication does.

- Ticket: `docs/tickets/04-pandeo-en-run.md`, MAM-11. The ticket stays open. MAM-11 and MAM-107 are not closed. No time or memory budget is approved.
- Author: implementation sub-agent, Grok 4.7, 2026-11-04 (America/Guayaquil).
- Role: implementer. No signature.
- Contract published: method id `buckling.linear` on the existing `BucklingLinear`. The equation and the solver are unchanged. No second eigensolver. No new dependency. Third-party code copied: none.
- Permitted sources used: the task text; `docs/CLEAN_ROOM.md`; `docs/tickets/04-pandeo-en-run.md`; this note; the in-repo APIs already named above. No network source. No book was opened.
- Public surface. `run` accepts `buckling.linear` with `mode_count` and `reference` (`case`, `factor`). Status is Experimental, so the job requires `options.allow_experimental = true`. `capabilities().methods` reports `engine: true` and `run: true`. The reference path reuses `pdelta::geometric_producer`, `StaticLinear::run_about_with_check` (it does not accept state), `LinearSystem::set_geometric_reference`, then `BucklingLinear::run_with_check`. `λ` multiplies that reference `Kg`. A member load on the reference case returns the existing code ENG-E-0013. No new error code. Results use a new optional object `buckling`. `modes` stays the mass-normalized list and is empty for this method. Each factor carries `normalization` `negative_kg`, meaning `φᵀ(−Kg)φ = 1`. On failure the key is removed. The Python facade and `mamut-toml/1` publish the same method. `method = "buckling"` stays rejected.
- Citation: Timoshenko–Gere citation, page, and table: **UNKNOWN**. The Spanish tests keep **DESCONOCIDO**. No page was invented. `source.kind = analytical`.
- Schema. `engine-input/1` and `results/1` gain the method literal and the definitions `buckling_options`, `buckling_mode`, and `buckling_result`. The published `schemas/` bytes were regenerated from `engine-api` and match. `docs/V1.md` is the inventory of branch `v1-primera`. It still says `buckling.linear` is unpublished. That sentence describes that inventory and was not edited here.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

### Evidence for the publication

- claim: a small in-repo column, passed through `run`, the Python facade, and `mamut-toml/1`, returns the library factors. One cubic pinned–pinned element with `E = 200 Pa`, `I = 3 m⁴`, `L = 4 m`, and a nodal load `Ux = −1 N` returns `450 N` within `1e-8`. The same bar with reference factor 2 returns `225 N`. Eight equal elements stay within relative `1e-3` of `C · π² · EI / L²` for `C = 1`, `C = 1/4`, the fixed–pinned root of `tan θ = θ`, and `C = 4`.
- state: VERIFIED for that numerical claim on this worktree. This is not code compliance and not a comparison with Timoshenko–Gere.
- gap: book citation UNKNOWN. Clean-room reviewer PENDING. The commit SHA is the head of branch `ticket/04-buckling-run` after this note is committed; it is not written here.
- execution, host `x86_64-pc-windows-msvc`, Cargo `--offline --locked`:
  - `cargo test -p engine-api --test buckling`: 6 passed.
  - The rest of the `engine-api` test binaries passed, including `published_schemas_match_the_complete_api`. `schema_tools_and_release_archive_are_reproducible` failed because Git bash on Windows rewrote the path of `scripts/package-schemas.sh` and exited 127. That failure is environmental.
  - `cargo clippy -p engine-api --all-targets -- -D warnings`: no errors.
  - `cargo test -p engine-cli --lib buckling_linear_round_trip_matches_the_one_element_factor`: passed. Round-trip and eigenvalue `450` within `1e-8`.
  - `cargo clippy -p engine-cli --all-targets -- -D warnings` still stops on a pre-existing `unused_mut` in `crates/engine-cli/src/lib.rs`, which is outside this diff. Allowing that one lint, the same clippy finished clean.
  - maturin is not installed, so no wheel was built and none was committed. `cargo build -p mamut-engine-py --features pyo3/extension-module` produced the cdylib. Loaded from a temporary copy of the package, `python -m unittest` on `bindings/python/tests/test_buckling.py` and `bindings/python/tests/test_typing.py` ran 5 tests, all passed, including `450` within `1e-8` and the unknown-page citation. A full discovery of `bindings/python/tests` is not that wheel install: it failed on missing package metadata, missing `MAMUT_RUST_REFERENCES`, and Windows rejection of a non-Win32 helper (`WinError 193`). Those failures are environmental.
- units and tolerances: the same SI scales and tolerances as the library session.
- sources: this note and `crates/engine-analysis/tests/buckling.rs`. Timoshenko–Gere was not used.

The acceptance box above stays open. The ticket is not Done. Clean-room reviewer: PENDING (J4).
