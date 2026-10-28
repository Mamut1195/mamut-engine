# Clean-room log · ENG-GEO-001

- Ticket / PR: ENG-GEO-001 (Soil profile for analysis); PR #14, merged into `main` (3e3c2b8); follow-up on branch `j2/geo-001-followup`.
- Author(s): J2 (agent, Grok Bot), 2026-10-11 (America/Guayaquil), label requested by the user. Recorded historical implementation: Codex gpt-6.1-sol, high effort, under J2's direction. Metadata update in ENG-MASS-001: Codex based on GPT-6.
- Role: specifier and implementer; no oracle operator or similarity auditor activity.
- Implemented contract: crate `engine-geo` 0.1.0, registered in the workspace, depending only on `engine-core` by path. Own trait `StressProfile: Send + Sync`, usable as `dyn StressProfile`, contract `stress-profile/1` (major 1, minor 0). `StratifiedProfile` validates contiguous layers by thickness, groundwater level and water unit weight; computes total vertical stress, hydrostatic pore pressure and effective stress. `GeoResult<T> = Result<T, Diagnostic>`, with core catalog codes and context in `hint`. It does not implement `element/1` or any structural `kind`.
- Permitted sources used:
  - Internal documents read first: `README.md`, `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md`, `provenance/ENG-CORE-003.md`, `crates/engine-core` (types, `Diagnostic`, `ENG-E-*` catalog, units), `crates/engine-contracts` (contracts and public types) and `docs/CONVENCION_EJES.md`. `CONTRIBUTING.md` and the workspace configuration were also reviewed. Protected documents and the core were not modified.
  - Standard soil mechanics equations provided in J2's specification: `σv = Σ γi hi`, `u = γw (z − zw)` below the groundwater level, `u = 0` above and `σ'v = σv − u`. Implementation and numerical examples derived here from scratch.
  - General conceptual reference: Terzaghi's effective stress principle, treated by Braja M. Das, *Principles of Geotechnical Engineering*, Cengage, general chapter on stresses in a soil mass (*Stresses in a Soil Mass*). **PENDING: verify edition/page**; chapter/section number, equation and book example also PENDING. The book text was not consulted and no numerical example in this log is attributed to it.
  - Bibliographic check limited to searching for the publisher listing of [Cengage, Principles of Geotechnical Engineering, SI Edition](https://au.cengage.com/c/principles-of-geotechnical-engineering-si-edition-10e-das/9780357420485/): search results identify a 10.ª edition listing and the general chapter title. Opening the listing failed due to redirection; this does not verify an edition/page for the equations or replace the pending consultation of the book. No data or code listings from external sources were used.
- Own derivations: piecewise integration and manual calculations in the following sections; [rustdoc and API](../crates/engine-geo/src/lib.rs), [implementation](../crates/engine-geo/src/profile.rs) and [tests](../crates/engine-geo/src/tests.rs).
- Third-party code copied: none. Own implementation with the standard library and internal `engine-core` API; no additional dependencies, no motor-geotecnico code or data.
- Validation: analytical, with hand-calculated results below and independent literals in the tests; 14 unit tests and 3 doctests in the new crate. Book (edition, example, page): **PENDING: verify edition/page**. Published example: none; cases A–H are own. Complete workspace: 57 unit tests and 28 doctests successful. Format, clippy, tests and rustdoc verification detailed below; no remote CI execution.
- Oracle comparison (sibling repo, informational): no. OpenSees, CalculiX and other engines were not run or consulted.
- Declaration: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'. Own code, API design, documentation and fixtures.
- Clean-room reviewer: PENDING (J4). Historical record: J4 review (2026-10-11), 0 high and 4 low findings; Jonathan's signature and approval remain pending. Das/Terzaghi/Cengage reference: **PENDING** (edition and page unverified).

## Derivation by self-weight integration

`z` is used as depth positive downward, with surface at `z = 0`.
For the approved global convention, Z points upward and
`z = Z_superficie − Z`. The profile uses thicknesses; it does not import global elevations or
interpret the horizontal D2 plane as a vertical plane. Soil compression
is taken as positive; future integration with structural stress positive in
tension must reverse the sign and convert units explicitly.

For a horizontal section of area A, the weight of a segment of height h is
`W = γ A h`. Dividing by A gives `Δσv = γ h`. Adding the segments above the
queried section gives `σv(z) = Σ γi hi`.

In a layer between depths a and b, for `z > a`, let `c = min(z, b)`.
The heights integrated here are:

```text
h_sobre = max(min(c, zw) − a, 0)
h_bajo  = max(c − max(a, zw), 0)
Δσv     = γ h_sobre + γsat h_bajo
```

For `z <= a` the layer contributes no weight. The heights are disjoint and their sum
is `c − a`; the partition works both at an interface and within a
layer. γsat is the saturated total unit weight: γw is not added to it again.

Hydrostatic equilibrium with zero gauge pressure at `zw` gives
`u(z) = γw max(z − zw, 0)`. The effective stress principle is applied
as `σ'v(z) = σv(z) − u(z)`. Suction above zw is not computed.
Each height contributes zero at its start, so σv is continuous at interfaces
and at zw, although its slope changes. u and σ'v are also continuous.

Units of this entire API: m, kN/m³ and kPa, with
`(kN/m³)·m = kN/m² = kPa`. The core and structural contracts use
Pa; `engine_core::units::convert(kpa, Kilopascal, Pascal)` must be used at
that boundary. `UnitsOut` is not used to label a number in kPa as Pa:
that structure declares base units and does not convert numbers. The core does not
include unit weight units, and is not modified in this ticket.

## Analytical validation: own hand-calculated cases

Unless otherwise stated, weights are Measured and γw = 10 kN/m³,
**explicit test datum**, not a module constant or a datum from a book.
Dry profiles also declare γw and zw explicitly. All
following depths/heights are in m and stresses in kPa.
Expected numbers are literals calculated using the formulas above,
without calling the implementation to produce them.

### A. Dry homogeneous, groundwater level at the base

H = 10, γ = 18, zw = 10; γsat absent, allowed because there is no submerged segment.

| z | σv = γz | u | σ'v = σv − u |
| --- | --- | --- | --- |
| 0 | 18·0 = 0 | 0 | 0 |
| 4 | 18·4 = 72 | 0 | 72 |
| 10 | 18·10 = 180 | 10·(10−10) = 0 | 180 |

Checks inclusive endpoints, σv = γz and γsat not required at the base.

### B. Homogeneous, groundwater level at the surface

H = 10, γ = 18, γsat = 20, zw = 0. All integrated weight is saturated.

| z | σv | u | σ'v |
| --- | --- | --- | --- |
| 0 | 20·0 = 0 | 10·0 = 0 | 0 |
| 4 | 20·4 = 80 | 10·4 = 40 | 80−40 = 40 |
| 10 | 20·10 = 200 | 10·10 = 100 | 200−100 = 100 |

The trait's three scalar shortcuts are also compared at z = 4.

### C. Groundwater level within a layer

H = 10, γ = 18, γsat = 20, zw = 3.

| z | σv | u | σ'v |
| --- | --- | --- | --- |
| 2 | 18·2 = 36 | 0 | 36 |
| 3 | 18·3 = 54 | 10·0 = 0 | 54 |
| 5 | 18·3 + 20·2 = 54+40 = 94 | 10·2 = 20 | 94−20 = 74 |
| 10 | 18·3 + 20·7 = 54+140 = 194 | 10·7 = 70 | 194−70 = 124 |

With ε = 0.000001, just above zw:
`σv(3−ε) = 54−18ε = 53.999982`, `u = 0`, `σ'v = 53.999982`.
Just below: `σv(3+ε) = 54+20ε = 54.000020`,
`u = 10ε = 0.000010`, `σ'v = 54.000010`.
These differences tend to zero: no jump is introduced when partitioning the layer.
The main doctest uses z = 5 and Assumed γw: 94, 20, 74.

### D. Multilayer, groundwater level at the interface

Layer 0: H₀ = 2, γ₀ = 16, no γsat. Layer 1: H₁ = 3, γ₁ = 18,
γsat₁ = 21. zw = 2, total H = 5.

| z | σv | u | σ'v |
| --- | --- | --- | --- |
| 1 | 16·1 = 16 | 0 | 16 |
| 2 | 16·2 = 32 | 0 | 32 |
| 4 | 16·2 + 21·2 = 32+42 = 74 | 10·2 = 20 | 74−20 = 54 |
| 5 | 16·2 + 21·3 = 32+63 = 95 | 10·3 = 30 | 95−30 = 65 |

With ε = 0.000001:
`σv(2−ε) = 32−16ε = 31.999984`, `u = 0`, `σ'v = 31.999984`;
`σv(2+ε) = 32+21ε = 32.000021`, `u = 0.000010`,
`σ'v = 32.000011`. The interface adds no weight or jump.

### E. Groundwater level below the profile

The same two dry layers: H₀ = 2, γ₀ = 16; H₁ = 3, γ₁ = 18;
zw = 8, γsat absent in both. At z = 2: `σv = 16·2 = 32`,
`u = 0`, `σ'v = 32`. At z = 5:
`σv = 16·2 + 18·3 = 32+54 = 86`, `u = 0`, `σ'v = 86`.
No extrapolation to zw.

The smaller dry doctest uses H = 2, γ = 18, zw = 3:
z = 0 produces (0, 0, 0), z = 2 produces (36, 0, 36),
and z = 2.01 is rejected for being below the base.

### F. Three layers, dry and submerged interfaces

Layers: (H₀ = 2, γ₀ = 16, γsat₀ absent),
(H₁ = 3, γ₁ = 18, γsat₁ = 21),
(H₂ = 4, γ₂ = 19, γsat₂ = 22). zw = 3, total H = 9.

At z = 2: `σv = 16·2 = 32`, `u = 0`, `σ'v = 32`.
At z = 5: `σv = 16·2 + 18·1 + 21·2 = 32+18+42 = 92`,
`u = 10·2 = 20`, `σ'v = 92−20 = 72`.
At z = 9: `σv = 92 + 22·4 = 180`, `u = 10·6 = 60`,
`σ'v = 180−60 = 120`.

With ε = 0.000001, manual values around both interfaces:

| z | σv | u | σ'v |
| --- | --- | --- | --- |
| 2−ε | 32−16ε = 31.999984 | 0 | 31.999984 |
| 2+ε | 32+18ε = 32.000018 | 0 | 32.000018 |
| 5−ε | 92−21ε = 91.999979 | 20−10ε = 19.999990 | 71.999989 |
| 5+ε | 92+22ε = 92.000022 | 20+10ε = 20.000010 | 72.000012 |

### G. ASSUMED parameter report

Case D is reused, also supplying γsat₀ = 20 although layer 0 is dry.
Declared Assumed: γ₀, γsat₀, γsat₁, zw and γw; γ₁ is Measured.
The exact report, without duplicates and with zero-based indices, is:
`LayerGamma(0), LayerGammaSat(0), LayerGammaSat(1), GroundwaterDepth, WaterUnitWeight`.
The same report is verified at z = 0, 1 and 5, including data not
involved in those queries. Numerical results do not change:
z = 5 produces (95, 30, 65). Modifying the returned vector does not alter the profile.

H = 2, γ = 18 Assumed, γsat = 20 Measured, zw = 0
and γw = 10 Measured is also tested: γ is Assumed and reported although unused because
the entire layer is submerged. At z = 2, the formulas give (40, 20, 20).
With all data Measured, the report is empty.

### H. Do not impose a relationship between weights or clip effective stress

Case deliberately outside usual soil values: H = 2, γ = 18,
γsat = 8, zw = 0, γw = 10. All are positive.
At z = 2: `σv = 8·2 = 16`, `u = 10·2 = 20`,
`σ'v = 16−20 = −4`. It is checked that γsat < γ is accepted and that the module
does not introduce a zero effective stress or silently correct the datum.
Physical assessment of those weights is the consumer's responsibility.

## Negative validation and arithmetic

- Empty profile, thicknesses 0, −0 and −1, nonpositive γ/γsat/γw weights: `ENG-E-0010`. A supplied γsat is validated even if unused.
- z = −1, z = −MIN_POSITIVE and z = 10.000001 for H = 10: `ENG-E-0010`; propagation in all three scalar shortcuts is also checked. z = 0 and z = H are valid, without tolerance or extrapolation.
- zw = −0.1: `ENG-E-0010`; groundwater level above ground is explicitly rejected.
- NaN, +∞ and −∞ in thickness, γ, γsat, γw, zw and depth: `ENG-E-0008`; includes unused γsat in a dry profile.
- Missing γ, γw or zw: `ENG-E-0003`. γ is mandatory even in a fully saturated layer; γw is mandatory even if it generates no pore pressure.
- Missing γsat with zw = 0, within a layer or in a deep layer below an interface: `ENG-E-0003`, with position/field in `hint`; the constructor validates the complete profile. The negative doctest uses H = 2, γ = 18, zw = 0 without γsat.
- Sum of two `f64::MAX` thicknesses: non-finite total, `ENG-E-0008`. Thicknesses 1e20 and 1: the second interface collapses in f64, `ENG-E-0010`; that layer is not removed.
- H = 2, γ = MAX, zw = 3: z = 0 gives (0, 0, 0), but `σv(2) = 2·MAX` overflows, `ENG-E-0008`. H = 2, γsat = MAX, zw = 0 also overflows when integrating. Two dry layers H = 1 and γ = 1e308 have finite individual products, but the sum 2e308 overflows and is rejected.
- H = 2, γ = γsat = 1, zw = 0, γw = MAX: `σv(2) = 2` is finite, but `u(2) = 2·MAX` overflows, `ENG-E-0008`. Stresses and pressure are verified before returning a result; neither inf/NaN nor a partial result is returned.
- Diagnostics verify code, severity Error and catalog message, empty entities and `hint` with precise field and catalog limitation. Analytical stress comparisons use absolute tolerance 1e-10 kPa for f64 rounding; endpoints, identities and reports are compared exactly.

## Scope decisions and contract limitations

Horizontal, contiguous layers of constant unit weight are used, declared
by thickness. Alternative elevations, inclined geometry,
cohesion, capillarity, suction, surcharges and impounded water above ground are not implemented.
Flow, non-hydrostatic pore pressure, consolidation,
deformation, strength, stability and horizontal stresses are not modeled.

`zw = 0` saturates the entire profile; `zw = H` and `zw > H` leave all integrated
heights above water. γsat may be absent only in layers without a submerged segment,
and any supplied γsat is still validated. There are no default values,
implicit assumption of γw = 9.81 or 10, or replacement γsat = γ.
γ, zw and γw are always required. Each value carries a declared origin;
the module does not verify whether a measurement was made or its quality.

The report includes **all** present parameters marked Assumed,
including unused ones, so that a shallow query does not hide
profile assumptions. It is ordered by layers (γ and γsat), then zw and γw.
Scalar shortcuts extract a number from `stresses_at`; to obtain
provenance, the consumer must retain the complete `StressResult`.

The core has no codes for missing geotechnical data, physical domain,
depth range or soil geometry. `ENG-E-0003`
(Broken reference) is reused for missing data, and `ENG-E-0010` (Invalid dimension)
for ranges/geometry, following parameter validation practice
in `engine-contracts`. `ENG-E-0008` is also used for non-finite results.
`engine-core` is not changed. Message/severity are preserved; the `hint` reports
the semantic limitation and necessary action. No structural entity is
invented for a layer; `Diagnostic.entities` remains empty.

Query complexity: O(n) in reached layers and O(a) to copy the
a Assumed identifiers of the profile. Integration is not interpolated or
discretized; there is only f64 rounding. Thicknesses that collapse an
interface on accumulation are rejected; stresses are validated when queried.
The profile remains immutable after the constructor and the response is pure.

Semver: package 0.1.0; an incompatible change before 1.0 will increment
the package minor version. Changing units, signs or trait requirements
requires a new major version of the `stress-profile` contract; compatible
extensions change its minor version. Identity and compatibility with minor 7,
and incompatibility with major 2, are checked using the core's `ContractId`.

## f64 precision limitation (follow-up)

With accumulated elevations near 1e16 m the f64 spacing is 2 m: a thickness of 1 m
is lost and rejected (`ENG-E-0010`), and one of 3 m is accepted with the lower
elevation rounded (1e16 + 4). It is an f64 limitation, not a logic
error; it is documented in the rustdoc of `StratifiedProfile::new` and the test
`f64_precision_limit_near_1e16_is_pinned` pins that behavior.

## Local verification

The default Rust 1.85.1 toolchain does not have rustfmt or clippy installed;
the first attempts without override reported missing components.
The components already installed in `stable` (Rust 1.99.0) were used, without
installing tools or modifying global configuration. All commands
use the executables in `~/.cargo/bin`:

```bash
RUSTUP_TOOLCHAIN=stable ~/.cargo/bin/cargo fmt
RUSTUP_TOOLCHAIN=stable ~/.cargo/bin/cargo clippy --all-targets -- -D warnings
RUSTUP_TOOLCHAIN=stable ~/.cargo/bin/cargo test
RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps
```

All successful. In addition, `~/.cargo/bin/cargo test` passed with Rust 1.85.1
(compatible with workspace rust-version 1.85). Both test runs:
57 unit tests and 28 doctests; engine-geo contributes 14 and 3, respectively.
`#![deny(missing_docs)]` is active in engine-geo and rustdoc finishes without warnings.
The final diff review limits files to Cargo.toml, Cargo.lock,
the new crate and this log; there are no modifications to protected paths.
