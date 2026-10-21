# Method sheet · <name> · <ticket>

Complete before implementation. Do not leave pending fields in a sheet used
to validate the method. The guide contains a complete sheet for the pilot.

- Author, model if AI, date and role:
- Own kind, contract and method version:
- Module/crate and intended consumer:
- Existing methods reviewed and why this one is distinct:

## Permitted sources

For each source: author, title, edition/year, section/page/equation and theoretical
text available locally. For an original derivation: author, date, title,
location and numbered equations. Do not invent locators; if theory is missing,
state the uncertainty before implementation. No third-party listings or copied tables.

## Assumptions, units and domain

- Physics and limitations:
- Total inputs, SI units and signs:
- Parameters, units, ranges and validation:
- State: order, length, units, initial values and admissible states:
- Outputs and units:
- Numerical domain, operation order, overflow/underflow:
- Nondifferentiable points and convention at each boundary:

## Original derivation

1. Starting equations and citation for each.
2. Development of the procedure, without referencing other engines.
3. Consistent derivative with respect to input, at fixed committed history.
4. Initial tangent, invariants, state acceptance and discard.
5. Analytical limits and recovery of simple cases.

## Cases established before execution

| Case / source / citation | SI parameters | Input and history | Analytical expected values | Tolerance and justification |
| --- | --- | --- | --- | --- |
| <complete> | <complete> | <complete> | <own evaluation with equation> | <complete> |

Separate smooth samples for FD, boundaries, cycles and negative tests.
Justify h with truncation/rounding error and verify kit limits.

## Integration and evidence

- Exports, descriptor and factory:
- Conformance appropriate to the trait:
- Native benchmark and, if applicable, ENG-CORE-009 fixture/1:
- Checks, tests passed/failed and logs under `$CARGO_TARGET_DIR`:
- Comparison of the 16 hashes across targets without modifying the reference:
- Log, clean-room review and validation limits:
