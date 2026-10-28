# Clean-room log · ENG-MAT-001

- Ticket / PR: ENG-MAT-001, «Ley uniaxial elástica y sección elástica»; branch `j2/eng-mat-001-elastic`. PR #13 of `mamut-engine` (merged into `main`, 8440441).
- Author(s): Codex gpt-6.1-sol high, orchestrated by J2; date 2026-10-10/11 (America/Guayaquil), according to the identification requested for this delivery. No delegation to other agents.
- Role: implementer; no oracle operator or similarity auditor activity.
- Implemented contract: `uniaxial/1`, kind `elastic`, and `section/1`, kind `elastic_frame`, contract versions 1.0. Crate and methods semantic version 0.1.0.
- Permitted sources used: internal contracts and specifications listed below, approved axis convention and own analytical derivations. No books or external sources were consulted for this implementation. For future bibliographic attribution of the elementary relationships: **PENDING: exact citation**; no authors, pages, chapters or equation numbers are invented.
- Own derivations: [note in this log](#derivaciones-propias-y-fixtures); [fixtures and tests](../crates/engine-laws/tests/elastic.rs). Reference values were calculated by hand, without extracting results from another engine.
- Third-party code copied: none. In-house code, standard library and local dependencies `engine-core` and `engine-contracts`; no external dependencies or changes to those crates.
- Validation: analytical; σ=Eε, N=EAε0, My=EIyκy, Mz=EIzκz and T=GJθ′, consistent tangents, conformance, negative inputs and repeatability.
- Oracle comparison (sibling repo, informative): no (**PENDING ENG-GOV-004**). No external engine was used as an oracle.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket." Nor did I consult code or manuals of CalculiX or other engines, or OpenSees/Stabileo manuals. There was no network access in this session.
- Clean-room reviewer: J4, 2026-10-11; result: 0 high findings, 0 medium, 4 low.

## Internal documents consulted

Before writing the implementation, the traits `UniaxialLaw` and
`SectionLaw`, the examples `LinearElastic` and `ElasticSection2D`, and the
`conformance::check_uniaxial_law` / `check_section_law` kit from `engine-contracts` were read.
`docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md`,
`provenance/ENG-CORE-007.md` and `docs/CONVENCION_EJES.md` were also read.

The authorized specification was consulted directly in
`04_TICKETS.md` (Mamut program ticket document), section ENG-MAT-001, and
`01_ARQUITECTURA_Y_CONTRATOS.md` (Mamut program document), §§4.2 and 4.3.
The core's exchange types, matrix storage and diagnostics,
the workspace manifests and `clean-room/check.sh` were consulted. No applicable
`AGENTS.md` was found in the workspace or its ancestor directories.
The checker consumes the existing prohibited lists; those lists
and all protected paths were left unchanged. No sources or paths of prohibited
engines were opened.

## Architecture decisions

1. `engine-laws` 0.1.0 is added to the workspace. It depends exclusively on the two
   authorized local crates, with version 0.1.0 and path. `Cargo.lock` records
   only the new package and those existing dependencies.
2. `lib.rs` contains general documentation, a common descriptor and private
   validators. `uniaxial.rs` and `section.rs` keep implementations, parameters,
   descriptors and factories separate. `engine-contracts` is unchanged.
3. `UniaxialFactory` and `SectionFactory` are local `Send + Sync` interfaces
   with typed associated parameters. Factories with the same parameter type
   are interchangeable through trait objects; their products are
   `Box<dyn UniaxialLaw>` / `Box<dyn SectionLaw>`. Concrete factories delegate
   all validation to the constructor. No global registry, JSON parser or
   new shared contract is added. Model dispatch is left for the future
   method registry.
4. `LawDescriptor` reports kind, contract and method semantic version.
   The contracts and method version are independent; the initial API
   is 0.1.0 and its evolution must respect semver. There is no mutable global state.
5. `Elastic` stores validated E. `ElasticFrame` preserves immutable properties
   and validated stiffnesses; it supports D2 and D3 with the exact order of `section/1`.
   The laws are pure, without history or preload; empty initial/trial state.
6. Finiteness and positivity of E, G, A, Iy, Iz and J are required even in D2, and of
   Ay/Az when present. All their products are validated: overflow
   produces `ENG-E-0008`; a positive product reduced to zero by
   underflow produces `ENG-E-0010`. Nonfinite inputs use
   `ENG-E-0008`; invalid ranges or lengths use `ENG-E-0010`, like the
   existing contracts. Messages identify the SI quantity involved.
7. G is explicit: no constructor from ν is offered and isotropy is not assumed to
   deduce it. J is the supplied Saint-Venant torsion constant, not a
   calculated polar inertia. There is no geometry calculator, warping, plasticity,
   damage, mass or transformation to end forces.
8. Ay/Az are effective shear areas that already incorporate any correction
   chosen by the consumer. GAy/GAz are exposed as `Option<Real>` accessors
   in N, also in D2; the contract's strains and resultants are not extended.
9. The section assumes principal centroidal axes and homogeneous material:
   the tangent is diagonal, without axial–bending or cross-bending coupling.
   X-Y is horizontal, Z vertical, right-handed triad; D2 remains in X-Y with Z=0.
   The pending D2 elevation proposal is not adopted.
10. `#![deny(missing_docs)]` requires rustdoc throughout the public API. Implementation
    and tests only require memory; they use no filesystem, threads,
    clock, network services or host-specific dependencies. Benchmarks
    are analytical fixtures in code, not performance measurements.

## Derivaciones propias y fixtures

The linear elasticity idealization σ=Eε is adopted, stress and modulus in Pa,
dimensionless total strain. Differentiation gives constant dσ/dε=E.
Energy per unit volume is ½Eε²; there is no history or initial stress.

In the principal centroidal local axes, define
εx(y,z)=ε0+κy z−κz y. Then:

- N=∫σx dA=EAε0, because ∫y dA=∫z dA=0.
- My=∫zσx dA=EIyκy, with Iy=∫z² dA and ∫yz dA=0.
- Mz=−∫yσx dA=EIzκz, with Iz=∫y² dA and ∫yz dA=0.

On a face with normal +x, these moments follow the right-hand rule about
+y and +z. On the opposite face their actions change direction. Positive N
means tension. κy and κz are derivatives of rotations about their local axes;
the sign of the axial distribution makes them conjugate to positive My and Mz.
The section does not decide the sign of actions on the nodes.

For uniform Saint-Venant torsion T=GJθ′ is adopted, where θ′ is the derivative
of rotation about +x. G in Pa, J in m⁴ and θ′ in rad/m give T in N·m.
Energy per unit length is
½(EAε0²+EIyκy²+EIzκz²+GJθ′²). Its gradient gives the resultants and its Hessian
gives the constant diagonal tangent (EA, EIy, EIz, GJ). In D2
only ε0 and κz are retained, with diagonal (EA, EIz). A in m² implies EA in N; Iy, Iz and J
in m⁴ imply EI and GJ in N·m². Curvatures are in m⁻¹.

If the consumer introduces shear strains outside this contract,
the corresponding linear relationships are Vy=GAyγy and Vz=GAzγz. Therefore
GAy/GAz have units N and are exposed only as optional data.

Own fixtures (each includes `source` and `citation` to this note):

- Uniaxial: E=200 Pa, ε=0.02 gives σ=4 Pa; E=200×10⁹ Pa, ε=−0.001
  gives σ=−200×10⁶ Pa; ε=0 gives σ=0 without preload.
- Small section: E=200 Pa, G=80 Pa, A=3 m², Iy=2 m⁴, Iz=5 m⁴,
  J=4 m⁴. Thus EA=600 N, EIy=400 N·m², EIz=1000 N·m², GJ=320 N·m².
  D2 with e=(0.02,−0.03) gives (N,Mz)=(12 N,−30 N·m).
  D3 with e=(0.02,−0.03,0.04,−0.05) gives
  (N,My,Mz,T)=(12 N,−12 N·m,40 N·m,−16 N·m).
  Ay=1.5 m² and Az=1.2 m² give GAy=120 N and GAz=96 N.
- Structural-scale section: E=200×10⁹ Pa, G=80×10⁹ Pa, A=0.03 m²,
  Iy=2×10⁻⁵ m⁴, Iz=5×10⁻⁵ m⁴, J=4×10⁻⁵ m⁴. Multiplying by hand:
  EA=6×10⁹ N, EIy=4×10⁶ N·m², EIz=10⁷ N·m², GJ=3.2×10⁶ N·m².
  With e=(0.002,−0.003,0.004,−0.005), the results are
  (12×10⁶ N,−12000 N·m,40000 N·m,−16000 N·m).

The data are own choices, not published examples or oracle
results. They are not attributed to an external page or equation. For any future
external bibliographic reference: **PENDING: exact citation**.

## Validation coverage and limits

Twelve integration tests and two crate doctests. The public conformance
checkers are used with paths including zero, tension/compression,
opposite signs and return to accepted inputs. Central differences use
increment 1e-7 in each component's units; relative error <1e-6.
All tangent entries are also verified separately with
increment 1e-6 and relative error <1e-6, preventing EA from hiding errors in EI/GJ.
Fixtures are compared with tolerance 1e-12 absolute + 1e-12 relative.

Checks cover diagonal matrices of the required size, symmetry, absence of
history, pure repetition and immutability, and `Send + Sync` as compilation
constraints without creating threads. Factories are exercised through their trait objects,
descriptors, propagated errors and accessors. Shear areas are tested
absent, present and separately; they do not alter any generalized response.

Negative cases: zero, negative, NaN and infinity in each property (including those
inactive in D2); incorrect lengths of strains and history; NaN,
infinity and overflow in each component; overflowed stiffness products
or products reduced to zero. No physical geometry is verified or D3
orientation inferred. Conformance only verifies the samples; it does not prove
accuracy over the entire floating-point range.

## Local verification

Final results of local native verification:

- `~/.cargo/bin/cargo fmt --check`: successful.
- `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: successful.
- `~/.cargo/bin/cargo test`: successful; full workspace with 55
  unit/integration tests and 27 doctests, including 12 and 2 from the new crate.
- `RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps`: successful.
- `bash clean-room/check.sh origin/main`: successful (`Clean-room: OK`).
  Also run with a temporary index and object directory in `/tmp`,
  including all new files of this ticket through intent-to-add,
  without altering the real Git index or objects. Successful result.
- `git diff --check` on that temporary index: successful. The full diff
  contains only `Cargo.toml`, `Cargo.lock`, the five files of
  `crates/engine-laws` and this log. No protected path appears in it.

`cargo test -p engine-laws --offline` was run first to verify the
fixtures and then the complete suite. No formulation corrections were
needed after those runs. There were no downloads or remote CI. The real index
keeps the new files unadded, available for user review.

- wasm32 run: **PENDING**, not executed (attempting `cargo build --target wasm32-unknown-unknown` on the box fails with E0463: the target std is unavailable for the active toolchain; no wasm runner). The tests are written to
  compile to wasm32 without `std::fs`, threads or system time; neither numerical
  wasm validation nor native/wasm equivalence is claimed in this delivery.
- Oracle comparison: no, **PENDING ENG-GOV-004**.
- Clean-room reviewer: J4, 2026-10-11; result: 0 high, 0 medium, 4 low.

No changes to `clean-room/**`, `.github/workflows`, `CODEOWNERS`,
`docs/CLEAN_ROOM.md` or legal files; no `LICENSE` was created.
