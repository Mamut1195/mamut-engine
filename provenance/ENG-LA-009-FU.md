# Clean-room log · engine-linalg: J4 follow-up (empty RHS, documented limits, error order)

- Ticket / PR: follow-up to #36/#38 (J0/J4); branch `j2/linalg-empty-rhs`
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Implemented contract: internal linear algebra backend (engine-linalg)
- Permitted sources used: no external sources.
- Own derivations: (a) `solve_in_place` with n=0 returns without traversing columns (previously did not terminate with B 0×usize::MAX); (b) documented the range limit ~1e300 per column of b (underflow of the small component); (c) `MemoryBudget::solve_dense` validates shape before accounting: the error order (shape → overflow → budget ENG-E-0013) no longer depends on the budget.
- Third-party code copied: none
- Validation: tests `empty_factorization_solve_with_huge_rhs_columns_terminates`, `rhs_spanning_more_than_1e300_...`, `error_order_does_not_depend_on_the_budget`.
- Oracle comparison: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4)

## ENG-LA-009-FU · lowest_modes anti-revert tests (MAM-70, #75)

- Follow-up: J4 r19 of #58; branch `j2/eng-la-009-fu`, based on `main`.
- Author: Codex · 2026-10-15 (America/Guayaquil). No delegation.
- Scope: four integration tests, one per point, in
  `crates/engine-linalg/tests/lowest_modes_anti_revert.rs`.
- Sources: exclusively `engine-linalg`'s own contracts and code and
  own analytical derivations. No network access, external oracles, consultation
  or execution of Stabileo/OpenSees or other third-party algebra libraries.
- Third-party code copied: none. New dependencies: none.
- Library: no bug was found; all four defenses were already present.
  The mutations were temporary and were restored byte for byte. No diff is left
  in `crates/engine-linalg/src/`.
- No changes to `docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` or `vendor/**`.
  The new fixtures use only elementary arithmetic; they need no transcendentals.

### Analytical oracles and isolation of regressions

For diagonal K and M=I, det(K−λM)=∏(Kᵢᵢ−λ), so the roots are
the diagonal entries and the eigenvectors are the canonical axes. The inertia
of K−σM is obtained by counting the signs of Kᵢᵢ−σ. The backend is not used as
an oracle for the expected values.

1. **Ascending order** — `anti_revert_rejects_unsorted_eigenpairs`.
   K=diag(1,2,4), M=I; the healthy case returns [1,2,4]. A test backend
   swaps the first two roots **and their columns**, providing [2,1,4].
   The pairs remain valid and, when requesting all three, Sturm counts three
   because the last root is still 4. Only the ordering guard must reject
   the result, with ENG-E-0007 and the diagnostic «sin ordenar».
2. **Inertia sum equal to order** —
   `anti_revert_rejects_inertia_whose_sum_differs_from_order`.
   For the same K, σ=2 gives diag(−1,0,2), inertia (1,1,1) and count 1.
   In `lowest_modes(count=1)`, the shift lies between 1 and 2 and the
   correct inertia is (2,1,0). Values (0,1,0), (2,1,1),
   (usize::MAX,1,0) and (2,1,usize::MAX) are injected: all preserve the negative count 1,
   but violate the sum. ENG-E-0007 and «inercia incompatible» are required through both
   `lowest_modes` and `sturm_count`; deficit, excess and
   overflow in each of the two checked sums are covered.
3. **Relative margin of σ** —
   `anti_revert_relative_shift_includes_nearby_distinct_root`.
   K=diag(s,s(1+5·10⁻⁸)), M=I, s∈{10⁻¹²,1,10¹²}. The midpoint has
   count 1: these are distinct roots. For count=1, σ=s(1+10⁻⁷) includes both
   roots, so ENG-E-0007 and Sturm=2 are required. The floor
   16·tol_cero≈9.1·10⁻¹³·s is much smaller than the separation 5·10⁻⁸·s.
   Requesting both roots must work and preserve them (relative error
   <10⁻¹⁴). The three scales check that the margin is relative.
4. **Zeroing of rigid modes** —
   `anti_revert_rigid_modes_are_exactly_zero_for_both_signs`.
   K=diag(−10⁻¹⁵,+10⁻¹⁵,1), M=I. The first two values are nonzero,
   of both signs, but |λ|<128·ε·3≈8.53·10⁻¹⁴. It is checked that the raw dense
   output preserves those signs and that `lowest_modes`, for count=2 and 3,
   returns exactly [0,0], counts two rigid modes and preserves the canonical
   vectors and the elastic root 1. A fixture with exact zeros would not detect
   regression of this assignment.

### Temporary mutations and observed results

The complete new file was run first: **4 passed, 0 failed**. For each
mutation only `crates/engine-linalg/src/modes.rs` was changed, the indicated
test was run, the original content was restored with `finally` and the same
test was run again. Command for each isolated run:

```bash
cargo test -p engine-linalg --test lowest_modes_anti_revert NOMBRE -- --exact
```

| Mutation | Temporary library change | Observed failure | After restoring |
| --- | --- | --- | --- |
| M1 / order | Remove the `if all.eigenvalues.windows(2)...` block and its error. | The test receives `Ok` with [2,1,4] instead of rejection; `unwrap_err` fails. Exit 101, 1 failed. | Exit 0, 1 passed. |
| M2 / inertia | Remove validation of `positive.checked_add(negative).and_then(...null) != Some(n)`; preserve only the return of negatives (and `let _ = (positive, null)` to avoid warnings). | Inertia (0,1,0) of order 3 is accepted and `Ok` with [1] is returned; `unwrap_err` fails. Exit 101, 1 failed. | Exit 0, 1 passed. |
| M3 / margin | Replace `(last * (1.0 + 1e-7)).max(last + 16.0 * zero_tol)` with `last + 16.0 * zero_tol`. | At s=10⁻¹² a single mode is accepted and `Ok` with [10⁻¹²] is returned; `unwrap_err` fails. Exit 101, 1 failed. | Exit 0, 1 passed. |
| M4 / zeros | Add `*eigenvalue = all.eigenvalues[col]` in the rigid branch, preserving the increment of `rigid_mode_count`. | The exact comparison receives [−10⁻¹⁵,+10⁻¹⁵] versus [0,0]. Exit 101, 1 failed. | Exit 0, 1 passed. |

All four failures were assertions from the new tests, with no compilation
errors. SHA-256 of `src/modes.rs` before and after the four mutations:
`9451d04d90ba44b5e22b8d93d311596a58bf166b871dfeb7f25117649eafd43e`.
`git diff --exit-code -- crates/engine-linalg/src` finished with exit 0.
The `unwrap`/`unwrap_err` belong exclusively to test code;
no `panic`, `unwrap` or `expect` are introduced into the library.

### Final verification of this session

All Cargo runs use `CARGO_NET_OFFLINE=true` and the exported `CARGO_TARGET_DIR`
`/workspace/.cargo-target/mamut-engine`; installed Rust/Cargo 1.99.0 toolchain
(`/home/box/.cargo/bin` is prepended to PATH, since `/usr/bin/cargo` is 1.85.1).

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK, exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK, exit 0. |
| `cargo test --workspace --no-fail-fast` | OK, exit 0; 549 tests passed (including doctests), 0 failures, 0 ignored; includes the four new tests. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace` | OK, exit 0. |
| `cargo check --workspace --target wasm32-unknown-unknown` | OK, exit 0; compilation check, no wasm execution. |
| `bash clean-room/check.sh origin/main` | `Clean-room: OK`, exit 0; includes the new file in the diff through `git add -N`. |
| `git diff --check` | OK, exit 0. |

## J4 r40 · ENG-LA-009-FU2: overflow that wraps to the order (MAM-70, #122)

- Branch: `j2/eng-la-009-fu2`, based on `main`; follow-up to the
  anti-revert tests incorporated in #118.
- Author: Codex · 2026-10-15 (America/Guayaquil). No delegation.
- Sources: only the repository's own code and contracts and elementary
  integer arithmetic. No network access, external oracles or copied third-party code.
  New dependencies: none.
- Change: `(usize::MAX, n + 1, 0)` is added to test
  `anti_revert_rejects_inertia_whose_sum_differs_from_order`, with `n = k.order()`
  (3). ENG-E-0007 and «inercia incompatible» are required through `lowest_modes` and
  the public API `sturm_count`; private functions need not be exposed.
- Oracle: `usize::MAX = 2^w - 1`, so the mathematical sum with `n + 1`
  is `2^w + n`, out of range. `checked_add` must reject it; `wrapping_add`
  returns `n` and bypasses the guard. The earlier overflow cases wrap
  to values other than `n` and do not detect this substitution.

### Verified temporary mutation

With `CARGO_NET_OFFLINE=true`, `PATH=/home/box/.cargo/bin:$PATH` and
`CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6` exported:

```bash
cargo test -p engine-linalg --test lowest_modes_anti_revert anti_revert_rejects_inertia_whose_sum_differs_from_order -- --exact
```

1. Original library: exit 0, **1 passed**.
2. Temporarily replace
   `positive.checked_add(negative).and_then(|v| v.checked_add(null)) != Some(n)`
   with `positive.wrapping_add(negative).wrapping_add(null) != n` in `src/modes.rs`.
   Compilation completes successfully and the test fails with exit **101**:
   `sturm_count` returns `Ok(4)`; the test's `unwrap_err` fails.
   `lowest_modes` reaches the subsequent mode count check, whose
   diagnostic also fails to satisfy the required rejection for incompatible inertia.
3. The library is restored in a `finally` block and the command is repeated:
   exit 0, **1 passed**.

SHA-256 of original and restored `src/modes.rs`:
`9451d04d90ba44b5e22b8d93d311596a58bf166b871dfeb7f25117649eafd43e`.
`git diff --exit-code -- crates/engine-linalg/src` finishes with exit 0.
No bug was found in the library and no diff is left in it; no
`panic`, `unwrap` or `expect` are added to library code. No changes to
`docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` or `vendor/**`.

### Final verification of r40

All Cargo commands run without network access, with the exported variables
indicated above and the installed toolchain 1.99.0.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK, exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK, exit 0. |
| `cargo test --workspace` | OK, exit 0; 569 tests passed, including doctests, 0 failures, 0 ignored. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace` | OK, exit 0. |
| `cargo check --workspace --target wasm32-unknown-unknown` | OK, exit 0; compilation checked, no wasm execution. |
| `bash clean-room/check.sh origin/main` | `Clean-room: OK`, exit 0. |
| `git diff --check` | OK, exit 0. |
