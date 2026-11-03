# Repository history and migration

Snapshot dated **2026-11-03**, based on the parent coordinator's verification.
This offline documentation followup did not query GitHub or check external links.

- **Original repository:** `Mamut1195/mamut-engine-private-archive-20261103`,
  GitHub repository ID `1412637886`, verified **PRIVATE** by the parent.
  Renamed from `Mamut1195/mamut-engine`, it preserves all original PRs,
  issues, logs and 84 branches.
- **New repository:** `Mamut1195/mamut-engine`, GitHub repository ID
  `1444444923`. It reuses the original address but is a different repository.
  The parent verified completed publication of sanitized history:
  it is now **PUBLIC**, with `private=false`.

At verification on 2026-11-03, the parent confirmed anonymous HTTPS HEAD
returned HTTP 200 and `main` was
`8d3c66fdee13d5660391aaac8fa6223b60e89a61` (a snapshot at verification,
not a claim about the current HEAD). The parent cloned from GitHub,
confirmed `fsck` passed and all 14 excluded historical blobs had reachable
count 0, and verified the LICENSE blob was identical to the original.
The repository license remains **AGPL-3.0-only**.

GitHub Actions remains disabled pending integration/review, with 0 persistent
runners. The workflow token is read-only, `canapprove=false`, and approval
policy is `all_external_contributors`. These parent-verified checks do not
establish hosted CI success or independent review approval.

The migration strategy preserves the full original repository privately
and sanitizes copied history for publication. Archive access remains
subject to existing private permissions.

PR and issue numbers and commit SHAs from before the migration on
2026-11-03 must be looked up in the private archive. Reuse of the original
URL does not transfer historical PR or issue numbering to the new repository.
For example, historical
[PR #254](https://github.com/Mamut1195/mamut-engine-private-archive-20261103/pull/254)
belongs to the private archive. Use these archive URL templates:

- PRs: `https://github.com/Mamut1195/mamut-engine-private-archive-20261103/pull/<number>`
- Issues: `https://github.com/Mamut1195/mamut-engine-private-archive-20261103/issues/<number>`

No old-to-new commit map has been provided; do not infer SHA mappings.
The root coordinator may provide a machine-readable map later.

Branches or history from old clones **must not be merged or pushed into
the sanitized repository or future public history**, because this can
reintroduce removed blobs. Start from a fresh checkout of the new repository
or a clean base in its sanitized history. Transfer changes through controlled
clean patches or cherry-picks, with the root coordinator's review of the
resulting history; do not merge wholesale old history.
