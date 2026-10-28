# Clean-room log · ENG-LA-008: MemoryBudget in factorize and generalized_eigen

- Ticket / PR: MemoryBudget wiring (J0); branch `j2/la-008-budget-factorize` (stacked on #49)
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Implemented contract: engine-linalg (MemoryBudget)
- Permitted sources used: no external sources.
- Own derivations: `MemoryBudget::factorize` and `::generalized_eigen`: charge retained+working storage, RAII guard, success leaves only retained storage; error/panic leaves `used` intact; error order independent of the budget (shape → overflow → ENG-E-0013 → backend). The initial claim of «upper bounds» was incorrect for factorize: it omitted a transient matrix, the Box header and the spare capacity of blocks. The correction and the scope of the current bounds are detailed below.
- Third-party code copied: none
- Validation: tests/budget.rs (charge, limit, backend error, different orders).
- Oracle comparison: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4)

## Heap peak correction (J4, 2026-10-13)

The LDLᵀ numerical calculation is preserved. `balanced`, the normalized copy `a` and
`l` remain alive simultaneously: transient working storage consists of **two** matrices,
in addition to the retained L. `blocks` allocates `with_capacity(n)` before the pushes,
because there are at most n pivots; its capacity is thus accounted for even
for small orders or orders other than a power of two. `factor_bytes` also includes
`size_of::<Ldlt>()`, the header allocated by the result's Box.

With U = `size_of::<usize>()` and H = `size_of::<Ldlt>()`, the current bounds are:

- Retained factor: `8n² + (24 + 2U)n + H`.
- Maximum factorize charge: `24n² + (24 + 2U)n + H`.
- generalized_eigen working storage: `72n² + (32 + 2U)n + H`, **excluding the result**.
  The nine matrices cover Cholesky, inverse, products, Jacobi and the spare
  capacity when collecting modes; the additional terms cover auxiliary vectors,
  indices/sorting space and the LDLᵀ header of the mass diagnostic.
- Modal result, charged separately: `8n² + 8n`.

These are bounds on the reference backend's numerical heap storage: Layout
bytes requested from the allocator, excluding existing inputs,
internal allocator metadata and error message construction. No bounds are
promised for arbitrary backends. The budget is released with the same
RAII guard; on successful completion only the retained factor or result remains.

`tests/heap_peak.rs` uses its own GlobalAlloc that delegates to System and counts
successful allocations, deallocations and resizes with AtomicUsize. The unsafe
is limited to the test allocator and documents its contracts; the library does not
incorporate unsafe or new panic/unwrap/expect calls. A single test in its
executable and a Mutex serialize the measurements; the peak is reset using the live
bytes as a baseline before each operation. The deterministic SPD matrices are
built before measuring. Orders 0, 1, 3, 9 and 65 are also checked to
cover headers and capacities outside powers of two, and the retained heap.
The test requires that the actual peak must not exceed the budget charge and that the
old bound underestimate at least one of the required orders.

Native 64-bit measurement (`cargo test --offline -p engine-linalg --test heap_peak -- --nocapture`;
U=8, H=232), in bytes:

| n | factorize peak | Old bound `16n²+40n` | Corrected charge | generalized_eigen peak | generalized_eigen charge |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 | 1856 | 1344 | 2088 | 4224 | 5800 |
| 64 | 100864 | 68096 | 101096 | 263168 | 331496 |
| 256 | 1583104 | 1058816 | 1583336 | 4198400 | 5257448 |

The new factorize peaks are lower than those of J4's probe
(1888/101120/1584128 B) because blocks no longer grows through reallocations.
The old bound falls below all three peaks; the corrected one covers them.
The measured values are regression evidence, in addition to the allocation count
above; peak constants are not used as an oracle in the asserts.

Sources: review of engine-linalg's own code and own measurements; the reviewer's
external probe was neither consulted nor copied. No new dependencies.

Final validation, without network access (`CARGO_NET_OFFLINE=true`):

- `cargo fmt --all` and `cargo fmt --all -- --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK (Rust 1.85.1).
- `cargo test --workspace`: OK, including heap_peak and doctests (Rust 1.85.1).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK (Rust 1.85.1).
- `cargo check --workspace --target wasm32-unknown-unknown`: OK with the stable
  toolchain already installed at `/home/box/.rustup/toolchains/stable-x86_64-unknown-linux-gnu`
  (Rust 1.99.0), explicitly using its cargo/rustc binaries. The rustc 1.85.1
  at `/usr/bin` lacks std for that target; the first attempt returns E0463.
  No components were installed and no network access occurred.
