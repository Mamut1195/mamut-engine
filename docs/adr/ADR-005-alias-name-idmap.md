# ADR-005: textual aliases and reversible identity mapping

- **Ticket:** MAM-993. Blocks MAM-940 and MAM-948.
- **Status:** proposed; review PENDING (J4); decisions recorded from Jonathan 2026-10-31
- **Date:** 2026-10-31 (America/Guayaquil).
- **Background:** [engine-input/1 schema](../../crates/engine-api/schemas/engine-input-1.json)
  (line 22); Python examples; Jonathan's decisions dated 2026-10-31.

## Context

`engine-input/1` defines node, member and spring IDs as integers in
`0..4294967295`. Python examples such as `nodes["N2"]` and `Node(id='N1')`
use text, exposing a mismatch between example notation and contract identity.
Jonathan's decision of 2026-10-31 retains the integer contract and adds textual
aliases with reversible mapping in TOML and Python. This ADR records that
decision and explicitly identifies unresolved details; it implements no changes.

## Decision

1. Adopt **Alt 1**: add an optional string field `name` to each node, member and
   spring in `mamut-toml/1` and the Python layer. Each entity carries an integer
   `id`, which remains its contract identity. `engine-input/1`, `results/1` and
   `input_sha256` remain unchanged: `name` never enters the JSON sent to the
   engine. Alias metadata therefore does not alter the engine input hash.

2. Store the mapping in a separate canonical JSON document, `idmap/1`, with
   these exact required fields and collection entry shapes:

   ```json
   {
     "format": "idmap/1",
     "nodes": [{"name": "N1", "id": 1}],
     "members": [{"name": "M1", "id": 1}],
     "springs": [{"name": "S1", "id": 1}]
   }
   ```

   Each collection is a list of objects containing string `name` and integer
   `id` in the contract range. Canonically order each list by natural `name`
   order: only ASCII `0`–`9` are digits;
   numeric runs compare by value without conversion to an integer; identifiers
   containing digits precede identifiers without digits; no Unicode
   normalization occurs. The final tie-break compares UTF-8 bytes (H1 = A).
   The normative text for this natural-order rule is defined in MAM-939,
   which has not yet been merged.
   An optional `input_sha256` field identifying the associated engine input is
   a proposal: **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**.
   Publication of `idmap/1` as its own schema is
   **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**.
   Schema rules for `additionalProperties`, uniqueness of `name` and `id`,
   an explicit `id` range, validity of empty lists, and exact JSON
   canonicalization (for example RFC 8785) remain
   **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**.

3. Within each collection, the recorded mapping is a bijection: each `name`
   maps to exactly one `id`, and each mapped `id` maps to exactly one `name`.
   It must be deterministic (same input, same ID), reversible (`id -> name`
   returns identical UTF-8 bytes), and stable across import -> export -> import.
   The mapping has separate node, member and spring collections. Whether a
   node and member may share a name is
   **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**.
   Although Alt 1 gives every entity an integer ID, the ticket leaves the
   rules for coexistence of aliases and explicit IDs in one model
   **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**.
   The exact assignment rule is likewise
   **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**; the ticket supplies
   criteria rather than an assignment rule.

4. Apply H2: a maximum of 256 UTF-8 bytes per `name` (alias).
   In Alt 1, `id` is an integer in `0..4294967295`.
   Exceeding 256 bytes yields `CLI-E-DOCUMENT`, exit 2.

5. Export to `engine-input/1` JSON carries no aliases. Whether
   `export --to toml` reconstructs `name` from `idmap/1` is
   **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**. The ticket's
   import -> export -> import stability criterion needs this behavior resolved.

6. Use Jonathan's approved error categories, with no new codes. The same
   failure has the same output for TOML and JSON:

   | Failure | Existing code | Exit |
   | --- | --- | --- |
   | Syntax, UTF-8, document limits, `name` (alias) over 256 UTF-8 bytes | `CLI-E-DOCUMENT` | 2 |
   | Specification rules, nonexistent alias referenced by `from`/`to`/`node`/`member` | `CLI-E-VALIDATION` | 2 |
   | Identical aliases (same bytes) in the same collection: Council decision (delegated by Jonathan, relayed by J0, 2026-10-31), pending Jonathan's review | `CLI-E-VALIDATION` | 2 |
   | TOML datetime values | `CLI-E-VALIDATION` | 2 |
   | Flags | `CLI-E-USAGE` | 2 |
   | Unsupported version | `CLI-E-UNSUPPORTED` | 3 |
   | I/O or internal failure, including nonexistent file | `CLI-E-INTERNAL` | 70 |

   NaN/inf yield `ENG-E-0008`, as with JSON input.

   Main has 32 codes in `docs/contracts/error-codes-1.json`: 14 `ENG-E`,
   9 `CLI-E`, and 9 API (4 `API-E`, 4 `API-W`, 1 `API-I`); 27 of them are
   error codes (`*-E-*`). The scope of `EngineError` over the 27 error codes
   is decided by MAM-947.

   A nonexistent file remains exit 70 unless a new decision changes it.
   Engine errors retain the same handling as JSON input. Diagnostics never
   repeat file content: only the code, JSON pointer and line:column are shown;
   even truncated names are omitted. Under the council decision above, pending
   Jonathan's review, two identical aliases (same bytes) in the same collection
   yield `CLI-E-VALIDATION`, exit 2, without echoing the value.

7. Naturally equivalent aliases, including `S01` versus `S1` and `0`
   versus `00`, are DISTINCT aliases, ordered by UTF-8 bytes without an error
   (H1 = A), under the council decision in Decision 6, pending Jonathan's review.
   This tie-break concerns text ordering only. `ENG-E-0002`
   ("Duplicate entity identifier", `docs/contracts/error-codes-1.json`) does not
   apply to aliases or equivalent IDs; it applies only to identical integer IDs
   that reach the engine. Aliases never reach the engine; identical aliases in
   the same collection yield `CLI-E-VALIDATION`, exit 2, as recorded in Decision 6.

## Alternatives considered

- **Alt 2: assign integer IDs by natural alias order when an ID is absent.**
  Rejected as the main mechanism: adding or removing an alias can change IDs,
  break stability and change input hashes. Jonathan's 2026-10-31 comment allows
  it "only when the id is missing, with a warning that these are not stable ids",
  while the internal assignment treated it as rejected as the main mechanism.
  Its limited use to supply missing IDs remains
  **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**; this discrepancy
  needs resolution. Warning text/code is
  **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**.
- **Alt 3: put `name` into `engine-input/1` and `results/1`.** Rejected for
  `/1` because it changes the contract and `input_sha256`; reserved for a
  future `/2` discussion, without implying approval for that version.

## Consequences

Human-readable names require a separate mapping document to recover exact
aliases from engine IDs. Engine identity, result identity and input hashing
retain their existing contracts. Missing decisions below prevent treating
this proposal as a complete implementation specification.

Acceptance tests must verify:

- Repeated imports of identical input produce identical IDs and canonical
  `idmap/1` bytes.
- Every recorded pair round-trips `name -> id -> name` with identical UTF-8
  bytes, and `id -> name -> id` with the same integer.
- Import -> export -> import preserves IDs and names once TOML export behavior
  is decided; JSON export excludes aliases and preserves `input_sha256`.
- Natural order gives `S1 < S2 < S10`; equivalent numeric forms use the byte
  tie-break without an error, and Unicode is not normalized.
- A 256-byte name passes the length check; a 257-byte name yields
  `CLI-E-DOCUMENT`, exit 2. Include multibyte UTF-8 to check bytes, not characters.
- The same failure in TOML and JSON yields the approved code and exit status.
  A sentinel embedded in a name never appears in stderr, even in truncated form.

## Open points

- **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**: Alt 2 as a fallback
  for missing IDs, its conflict with the internal assignment, and warning text/code.
- **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**: publication of an
  `idmap/1` schema and the proposed optional associated `input_sha256` field.
- **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**: coexistence of
  aliases and explicit IDs; whether names may be shared between collections.
- **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**: reconstruction of
  names by `export --to toml` to meet import -> export -> import stability.
- **UNKNOWN (DESCONOCIDO) — pending Jonathan's decision**: the exact ID
  assignment rule satisfying the ticket's criteria.
