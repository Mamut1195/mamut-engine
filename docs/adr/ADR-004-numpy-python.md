# ADR-004: result tables and optional NumPy in Python

- **Ticket:** MAM-82 · ENG-PY-004.
- **Status:** decision implemented; clean-room review **PENDING (J4)**.
- **Date:** 2026-10-20 (America/Guayaquil).
- **Background:** [ADR-003](ADR-003-python-v1.md),
  [ENG-PY-003](../../provenance/ENG-PY-003.md).

## Context

`results/1` transports JSON lists produced exclusively by Rust.
Notebooks need matrices with explicit identity and units, without depending
on the lifetime of backend buffers. Models may mix elements with
different numbers of DOFs; document identities do not necessarily have
the same order as response rows. A failed status may lack
physical responses while retaining useful diagnostics and convergence.

## Decision

1. `Results.to_arrays()` returns `ArrayResults` with independent lists and
   metadata; `Results.to_numpy()` returns the same contract with `ndarray`.
   Both check for staleness before accessing the snapshot. `ArrayResults.document`
   is a full copy of `results/1`, preserving textual status, fingerprint,
   identities, active DOFs, connectivity, diagnostics and untabulated fields.
   `status` and `partial` retain the semantics of `Results`. Status is not converted
   into an f64 quantity, nor is a failure presented as success.
2. NumPy is the extra `mamut-engine[numpy]` (`numpy>=1.23`). Importing it only
   occurs inside `to_numpy`; importing the package, resolving annotations,
   running analyses and exporting lists work without NumPy. Its absence raises
   `MissingNumPyError`, a subclass of `EngineError`, code `PY-E-NUMPY`, with the
   name of the extra. An internal NumPy installation error is not disguised
   as a missing dependency. The type `values: Any` allows public
   annotations to be resolved without importing the optional dependency.
   NumPy is distributed under the BSD-3-Clause license, compatible with AGPL-3.0 and
   with the commercial license, and is not vendored.
3. Each `ResultMatrix` exposes `values`, `shape`, `row_ids`, `row_indices`,
   `dofs`, `units` and `frame`. IDs and indices are kept as tuples of exact
   Python integers; they are never converted to floats. Labels and units are
   tuples of strings. Only numerical values are f64 matrices.
4. The conversion is `numpy.asarray(listas, dtype=numpy.float64, order="C")`,
   reshape to the explicit shape and `.copy(order="C")`. Even empty
   matrices have two dimensions, their own memory (`OWNDATA`, `base is None`)
   and C order. Outputs are mutable, independent of each other, the JSON and
   the backend. They survive destruction of Results or subsequent changes
   to the model; those changes block new exports through staleness.
   Lists are also copied. There are no zero-copy views or borrows of
   mutable engine buffers. Metadata containers are local copies;
   modifying an export does not modify the snapshot or another export.
5. Tables follow the responses without reordering, interpolating,
   completing them or computing forces. End forces are grouped by
   element type to avoid ragged matrices or invented padding.
   An unknown type requires new explicit labels and yields `Unsupported`
   (`ENG-E-0013`); an incompatible shape or identity yields `InvalidInputError`.
   The original JSON remains available. The detailed contract is in the
   [binding README](../../bindings/python/README.md#tablas-y-numpy-opcional).
6. No Rust dependencies are added and Cargo.toml/Cargo.lock are not changed.
   The existing JSON/list transport is sufficient; adding rust-numpy would duplicate
   infrastructure and complicate ownership without evidence of need.
   Python only validates structure and converts representation; Rust retains
   all the physics and arithmetic of the analysis.
7. No GIL release is added. The Python/NumPy projection requires live Python
   objects. The previous `py.detach` is retained exclusively around
   Rust calls: it captures an immutable `&str` whose lifetime spans the call, an Arc
   containing an atomic token and Rust event accumulation; it does not access Python objects
   or execute Python callbacks during detach. Output conversion and
   callbacks run after reacquiring the GIL.

## Consequences and verification

Copies consume memory and time proportional to the output. `document`
also retains the JSON snapshot with lists; minimal memory or
solve acceleration is not promised. Only tabulated fields are converted to ndarray;
nullable prescribed values, end data, modal participation and convergence remain
in `document`, avoiding NaN as a substitute for null. Zeros for absent
components and modal reactions are the engine's original placeholders.

The installed tests compare against independent Rust references, reject
NaN/Inf, incorrect shapes and identities, verify copies/C order/dtype,
empty outputs and operation without NumPy. NumPy tests use
`pytest.importorskip("numpy")`; the basic API only needs stdlib.
The wheel is built twice in separate targets and SHA-256 equality is
required through the existing verifier. It is not published on PyPI.

The [performance plan](../PYTHON_BOUNDARY_BENCHMARKS.md) separates solve,
transport, parsing and copies, and retains samples without CI thresholds.
