# ENG-LANG-007 — Tranche 3 (clean-room tests and workflows)

- Ticket: MAM-843 (ENG-LANG-007). Date: 2026-10-27. Base: origin/main b707c07 (includes #228).
- Authorization: owner (Jonathan) authorized on 2026-10-27 translating only these three files, with identical behaviour.
- Scope: `clean-room/tests/run.sh`, `.github/workflows/clean-room.yml`, `.github/workflows/rust.yml`. Text written by Codex; reviewed by J2's helper.

| File | Lines changed | Kind |
| --- | ---: | --- |
| clean-room/tests/run.sh | 45 | comments; 5 printed messages (10 in total with clean-room.yml) |
| .github/workflows/clean-room.yml | 13 | comments; 5 printed messages |
| .github/workflows/rust.yml | 1 | comment |

## Printed strings changed (old -> new)

None of these is compared by any script, test, workflow or doc (checked with rg over the repo).

- clean-room/tests/run.sh: `FALLA %s: esperado %s, obtenido %s` -> `FAIL %s: expected %s, got %s`
- clean-room/tests/run.sh: `FALLA %s: falta el diagnóstico de identificador con integridad válida` -> `FAIL %s: missing identifier diagnostic with valid integrity`
- clean-room/tests/run.sh: `FALLA or_incident_real: CI sin historial de 43b5f45 (¿clon superficial?)` -> `FAIL or_incident_real: CI lacks history for 43b5f45 (shallow clone?)`
- clean-room/tests/run.sh: `aviso: historial sin 43b5f45; solo se prueba la réplica sintética` -> `warning: history lacks 43b5f45; only the synthetic replica is tested`
- clean-room/tests/run.sh: `%d pruebas, %d fallos` -> `%d tests, %d failures`
- .github/workflows/clean-room.yml: `::warning::La base no tiene validador clean-room; se usa el del PR (arranque).` -> `::warning::The base has no clean-room validator; using the PR validator (bootstrap).`
- .github/workflows/clean-room.yml: `Sin archivos protegidos modificados.` -> `No protected files changed.`
- .github/workflows/clean-room.yml: `Archivos protegidos modificados:` -> `Changed protected files:`
- .github/workflows/clean-room.yml: `Aprobado por $OWNER.` -> `Approved by $OWNER.`
- .github/workflows/clean-room.yml: `::error::Cambios en archivos protegidos sin aprobación vigente de @$OWNER.` -> `::error::Changes to protected files without current approval from @$OWNER.`

## Left in Spanish

- All `name:` keys of workflows, jobs and steps (kept to leave check and step names unchanged).
- Test-case input fixtures in `clean-room/tests/run.sh` (the contents fed to `check.sh`): they are test data whose wording is what the validator must detect.
- `check.sh` output (out of scope).
- Usage/example text left untranslated: `clean-room/tests/run.sh:2` (`[ruta-a-clean-room]`, usage placeholder) and `clean-room/tests/run.sh:16` (`$tmp/NOMBRE`).
- The inline comment at `clean-room/tests/run.sh:228` (`# Sin salto final para other_file/untracked/ignored.`): translated in the follow-up commit.

## Verification

- `bash -n clean-room/tests/run.sh`: OK.
- `clean-room/tests/run.sh` on main and branch: both exit 0, 144 tests; output differs only in the summary line.
- Parsed YAML trees main vs branch: same keys and structure; differences only inside `run:` blocks (shell comments and the messages above).
- `clean-room/check.sh origin/main`: OK. `scripts/changelog-check.sh origin/main`: no fragment required. `git diff --check`: OK.
- shellcheck 0.11.0 and actionlint 1.7.7 were run by J4 (offline): the same findings on main and on the branch (7 pre-existing SC2094/SC2016 and the self-hosted runner labels); nothing new.
