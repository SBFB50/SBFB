# Phase Review — Sprint 46 Phase C

## Verdict : PASS (2 P2, 1 P3)

Rigor signal : 3 findings (2 P2 + 1 P3), >=1 P2 requis pour PASS rigoureux. 0 P0, 0 P1.

## Memory consultation
- feedback_approach.md : pick deepest — migration complete vers daemon, pas dual-mode band-aid ✅
- feedback_cd_web_trap.md : subshell pour web/ commandes — respecte ✅
- Tensions : aucune

## Staging check (Step 1bis)
- Phase fichiers : 7 (coordinator.ts, daemon.ts, daemon.test.ts, KudosTab.tsx, TasksTab.tsx, ProjectDetail.tsx, BrowsedProject.test.tsx)
- Planning/docs split : chore(planning) preflight fait separement (7cad387) ✅
- Untracked accidentels : 0 ✅

## Suites
- cargo fmt : 0 diff ✅
- cargo clippy : 0 warnings ✅
- Rust nextest : 1168/1168 (inchange, phase frontend) ✅
- Release build : ok ✅
- SDK pytest : 195 ✅
- Coord pytest : 323 + 23f (PyO3 stale) + 6s ✅
- Gov pytest : 46 ✅
- Frontend lint : 0 errors ✅
- Frontend tsc : 0 errors ✅
- Vitest : 268 -> 267 (-1, test proxy envelope supprime — chemin de code retire) ✅
- Build : ok ✅
- size-limit : 5/5 sous budget ✅
- scan-en-strings : clean ✅

## Commit body validation
- Format titre : ✅
- Delta tests : Vitest 268->267 (-1), justifie (test proxy envelope retire)
- Scope cuts honoured : ✅ 13/13
- Co-Authored-By : ✅

## Modified-file branch coverage (Step 2bis, G9)
- coordinator.ts : renommage classes + paths mis a jour, pas de nouvelle branche — PASS ✅
- daemon.ts : callDaemon() remplace callProxy() — teste par daemon.test.ts (4 tests mis a jour) ✅
- KudosTab.tsx : adaptation champs KudosEntry — pas de nouvelle branche ✅
- TasksTab.tsx : state→status, submitted_at→created_at — pas de nouvelle branche ✅
- ProjectDetail.tsx : worker_pubkey_hex→worker_node_id — 1 ligne ✅

## Research grounding (Step 4bis)
- S1a : APPROACH-ALIGNED, refactoring API paths standard ✅
- S1b : 0 nouveau package npm ✅

## Horizon long-terme (Step 4ter)
- N/A (phase refactoring, pas nouveau module) ✅

## Scope cuts verification
- events.py SSE streaming : 0 fichier diff ✅ (useAppEvents.ts non touche)
- App runtime migration Rust : 0 ✅ (routes /app/* gardees)
- 11 autres scope cuts : 0 ✅

## Findings

- **P2-REVIEW-C-1-S46** : les schemas Zod (HealthSchema, TaskRowSchema, KudosEntrySchema, ShellDiscoverResponseSchema) ont ete mis a jour pour matcher les reponses daemon. Les routes app-specific (/app/*) restent pointees vers le coordinator Python et utilisent potentiellement les anciens schemas. Si le coordinator Python est arrête sans que les routes app soient migrées vers Rust, les schemas casseront. Carry-over S47 (scope cut "App runtime migration Rust").
- **P2-REVIEW-C-2-S46** : les aliases de retro-compatibilite `CoordinatorProtocolError`/`CoordinatorHttpError` sont gardes comme re-exports deprecies. A supprimer quand tous les consommateurs externes (tests Playwright, scripts) sont migres. Carry-over S47.
- **P3** : delta Vitest -1 (267 vs 268 baseline). Le test supprime (`503 body unreadable proxy envelope`) testait un chemin de code qui n'existe plus (proxy envelope parsing). Le scenario 503 est toujours couvert par le test `returns kind=unavailable on 503`. Nit documentaire.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S47 : P2-REVIEW-C-1-S46 (app-specific schema drift 1/3), P2-REVIEW-C-2-S46 (deprecated aliases cleanup 1/3)
