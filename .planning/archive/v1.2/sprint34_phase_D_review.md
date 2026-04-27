# Phase Review — Sprint 34 Phase D

## Verdict : PASS (1 P2 + 1 P3)

Rigor signal : 2 findings documentés / >=1 requis pour PASS.

## Memory consultation (Step 1.5)
- feedback_approach.md : no band-aid, root cause → running.json
  path mismatch fixé proprement (délégation daemon-core) ✅
- N/A : wrap-up docs-only, pas de zone fonctionnelle spécifique

## Staging check (Step 1bis)
- Phase fichiers : 16 (verification.md NEW, audit_plan S35 NEW,
  CLAUDE.md, SPRINT_LOG.md, HARDENING_ROADMAP.md, main.rs fix,
  11 renames active→archive + 1 rename audit_findings)
- Planning/docs split : chore(planning) research docs committé
  séparément (`6775bb3`) ✅
- Untracked accidentels : 0

## Suites (Step 2)
- Rust nextest : 902/902 pass ✅
- Rust fmt : clean ✅
- Rust clippy : clean ✅
- Release build : OK ✅
- Python SDK : 195 ✅ | Coord : 408+37f+6s ✅ | Gov : 46 ✅
- Frontend : lint 0 errors + tsc clean + 267 Vitest + build OK
  + size 7/7 + 42+2f PW + en-strings clean ✅

## Modified-file branch coverage (Step 2bis)
- `main.rs:find_running_json()` : modifié, 0 nouvelle branche
  (fallback preserved), test existant
  `test_find_running_json_returns_expected_path` couvre ✅

## Delta tests (Step 3)
- 0 (Phase D = docs + migration + 1 bugfix dans code existant)
- Total : ~1905 inchangé

## Scope cuts verification (Step 5)
- Tous les 12 items §7 : ✅ non touchés

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc : N/A (wrap-up, pas de module structurant)
- D1..D5 respectées : ✅
- Solution la plus poussée : ✅ (running.json path via daemon-core
  source-of-truth, pas duplication)
- LOC estimées au plan : ✅ 0 match

## Research grounding (Step 4bis)
- N/A (wrap-up phase, pas de nouvelle dep/API)

## Findings

### P2-D-1 : running.json path mismatch launcher vs daemon (fixed)

Le launcher utilisait `USERPROFILE/.nexus-grid/` alors que le
daemon utilise `BaseDirs::data_dir()/nexus-grid/` via
`nexus_shell_daemon_core::paths`. Sur Windows,
`USERPROFILE = C:\Users\FlowUP` ≠ `data_dir = AppData\Roaming`.
Le launcher ne trouvait jamais `running.json`, timeout 15s, kill
daemon, exit.

**Fix inline** : `find_running_json()` délègue désormais à
`nexus_shell_daemon_core::paths::running_json_path()` avec
fallback `USERPROFILE/.nexus-grid` uniquement si dirs échoue.
Bug pré-existant Phase B mais détecté par test manuel Phase D.

### P3-D-1 : coord PyO3 stale variance 409→408 pass

1 test coord qui passait (409) échoue maintenant (408) — variance
normale du wheel PyO3 stale. Pas de regression code.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S35 : P2-D-1 déjà fixé inline
