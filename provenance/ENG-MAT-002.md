# Clean-room log · ENG-MAT-002

- Ticket / PR: ENG-MAT-002, uniaxial law `multilinear_elastic`; local delivery without commit or push.
- Author(s): J2 (agent, Grok Bot), attribution requested by the user; implementation carried out by Codex, model GPT-6; date 2026-10-11 (America/Guayaquil). No delegation to other agents.
- Role: implementer, with own analytical derivations and fixtures.
- Implemented contract: `uniaxial/1` (version 1.0), kind `multilinear_elastic`, method 0.1.0, within `engine-laws`.
- Permitted sources used: user's ENG-MAT-002 request, internal code and documentation listed below; no external sources, books, manuals or network access.
- Own derivations: [note in this log](#derivaciones-propias-y-fixtures) and [analytical tests](../crates/engine-laws/tests/multilinear.rs).
- Third-party code copied: none. Only standard library and the existing local dependencies `engine-core` and `engine-contracts`; no new crates or dependencies.
- Validation: analytical; exact piecewise values and extrapolation, tangents through finite differences, continuity, symmetry/asymmetry, gaps, numerical extremes, rejection of invalid inputs and uniaxial conformance.
- Oracle comparison (sibling repo, informative): no. No external engine was run or consulted.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket." Nor were external sources or manuals of other engines consulted. There was no network access.
- Clean-room reviewer: PENDING (J4).

## Internal documents consulted

`provenance/TEMPLATE.md`, `provenance/ENG-MAT-001.md`, `docs/CLEAN_ROOM.md`,
`CONTRIBUTING.md`, `clean-room/README.md` and `clean-room/check.sh`; existing implementation
and tests of `engine-laws`; trait `UniaxialLaw` and
`conformance::check_uniaxial_law` kit of `engine-contracts`; catalog and errors of
`engine-core`; workspace manifests. No applicable `AGENTS.md` was found
in the repo or its ancestors. No sources of prohibited engines were opened.

## Implementation decisions

The API is exported from `engine_laws::uniaxial`, with private implementation in
`src/uniaxial/multilinear.rs`. It includes law, typed parameters, descriptor and
factory implementing the existing `UniaxialFactory`. Like ENG-MAT-001,
there is no global registry, parser or changes to `engine-contracts`. The table and
slopes are validated once and kept immutable; initial,
committed and trial state has zero length. Law and factory are `Send + Sync`.

Explicit ordered points `(ε, σ)` in SI are received; ε is dimensionless and positive
in tension and σ is in Pa. There must be at least two points and an exact origin
`(0, 0)`, without tolerance or automatic correction. Reflected points define an
odd symmetric curve; independent points define an asymmetric curve.
One-sided tables with the origin at one end are also allowed: the end
segment extends to the other side, except when the mode disables it.

The modes are both sides, compression-only and tension-only. The inactive side
gives exactly σ = 0 and tangent = 0. The origin belongs to the active side;
compression-only chooses the left slope there if present, and tension-only
the right one if present. At the other vertices the right segment is chosen, except
the last point, which uses the last segment. `initial_tangent()` always matches
`respond(0).tangent`, including one-sided tables.

At a vertex with different slopes or at gap opening there is no
two-sided derivative. The contract requires declaring these points; the existing kit
accepts the choice with samples whose ε ± h interval is differentiable,
and separately checks the initial tangent against the response at the origin.
It is not claimed that the central difference matches a one-sided derivative
at a corner. Vertices are tested directly without altering the kit.

Exact plateaus are allowed and negative slopes are rejected with
`ENG-E-0010` and segment context. The same code covers invalid lengths, order,
origin and history, following the existing trait's range. A
positive slope reduced to zero by underflow is also rejected
with `ENG-E-0010`, to avoid confusing it with a valid plateau. `ENG-E-0008` covers
nonfinite inputs and overflows in differences, quotient, product and
sum. The message identifies the quantity; no codes are added to the catalog.
The entire table is validated, even if a mode disables one side, and
ε and history are validated before responding in the gap.

Segment search is binary. The nearest end is used as an anchor
to reduce cancellation; tabulated points are returned exactly.
A plateau avoids calculating unnecessary offsets and allows evaluating
extreme finite strains. The library implementation contains no
`panic!`, `unwrap` or `expect`. Private indices are bounded by the
constructor invariants; the user cannot mutate points or slopes.

## Derivaciones propias y fixtures

For two consecutive points, Δε = εᵢ₊₁ − εᵢ > 0,
Δσ = σᵢ₊₁ − σᵢ ≥ 0 and mᵢ = Δσ / Δε. The unique affine function passing
through both points is σ(ε) = σᵢ + mᵢ(ε − εᵢ), with dσ/dε = mᵢ.
Using the right end as anchor gives the equivalent expression
σᵢ₊₁ + mᵢ(ε − εᵢ₊₁). Both reach the same stress at each vertex,
so the curve is continuous. End extensions preserve that
continuity and the slope of the corresponding outer segment. In the gap
σ = 0 is defined; since the origin has σ = 0, the transition is also continuous.

The in-house asymmetric fixture uses points
`(-3, -10), (-1, -6), (0, 0), (1, 4), (3, 6)`; the slopes are
`2, 6, 4, 1` Pa. Exact evaluations, with binary-representable data:

| ε | σ (Pa) | Chosen tangent (Pa) |
|---|---|---|
| −4 | −12 | 2 |
| −3 | −10 | 2 |
| −2 | −8 | 2 |
| −1 | −6 | 6 |
| −0.5 | −3 | 6 |
| 0 | 0 | 4 |
| 0.5 | 2 | 4 |
| 1 | 4 | 1 |
| 2 | 5 | 1 |
| 3 | 6 | 1 |
| 4 | 7 | 1 |

For symmetry `(-3, -6), (-1, -4), (0, 0), (1, 4), (3, 6)` are used.
For plateaus: `(-2, -4), (-1, -4), (0, 0), (1, 4), (2, 4)`.
Minimal tables `(0, 0), (1, 2)` and `(-1, -2), (0, 0)` reproduce σ = 2ε
for both signs before applying the gap. The line σ = ε is tested with
extremes `±f64::MAX` and the smallest positive subnormal; plateaus are extrapolated
to those finite extremes without unnecessary multiplication or subtraction.

Central differences away from vertices use h = 10⁻⁶ and relative error
less than 10⁻⁶. Continuity at all vertices and modes is verified
exactly with h = 1/1024 and the slopes of the respective sides.
Fixture values carry `source` and `citation` to this note; they are
own choices and derivations, without results from another engine.

## Local verification

Checks run without network access, with `CARGO_NET_OFFLINE=true`.
The final runs use the installed stable toolchain (`rustc 1.99.0`,
2026-09-28), prepending `/home/box/.cargo/bin` to PATH; the wasm target was already
installed and nothing was downloaded.

- `cargo fmt` and `cargo fmt --check`: successful.
- `cargo clippy --all-targets -- -D warnings`: successful.
- `cargo test --workspace`: successful; 161 unit/integration tests
  and 35 doctests. The new law contributes 12 integration tests and one doctest;
  the complete crate passes 24 integration tests and 3 doctests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: successful.
- `cargo check --workspace --target wasm32-unknown-unknown`: successful.
  This is a compilation check; no numerical tests were run in wasm.
- `bash clean-room/check.sh origin/main`: successful (`Clean-room: OK`),
  including the three new files through `git add -N` in a temporary index and
  object directory under `/tmp`, without touching the real Git index or
  objects.
- `git diff --check` with the same temporary index: successful. The diff only
  contains the four `engine-laws` files and this log.

No changes to protected paths, manifests or dependencies. No commit is created
or push made. Human clean-room review remains PENDING (J4).
