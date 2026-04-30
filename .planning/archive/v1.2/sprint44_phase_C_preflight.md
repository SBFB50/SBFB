# Sprint 44 Phase C — preflight G8

Date : 2026-04-30 | HEAD : `7100d24` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : "pick deepest, no band-aid" — Phase C
  porte 2 routes API existantes, pattern S42-S43 + Phase B etabli.
- feedback_context7_systematic.md : 0 nouvelle dep. N/A.

## Scans (all clean)
- S1a OSS prior art : port de routes Python existantes vers
  axum Rust. Pattern etabli S35-S44B (20+ routes portees). Pas de
  decision architecturale nouvelle. APPROACH-ALIGNED.
- S1b deps : 0 nouvelle dep, 0 bump — clean.
- S2 historiques : 3 fichiers scannes (http.rs, main.rs, db.rs).
  Commits DEVIATION/rejected sur http.rs (S36/S39/S40) et main.rs
  (S18/S7) — non-lies aux routes tasks/worker_state. db.rs clean.
  Clean.
- S3 threat model : fast-path verified. Phase C n'introduit
  aucun nouveau composant securite ni wire format. Routes = lecture
  etat (task list/get, worker state proxy). Clean.
- S4 wire format : fast-path verified. canonical.rs non touche.
  VERSION=1 inchange. Day 0 preservees. Clean.

## G1 blind spot adresse (design review)
- D2(c) tasks.py list_tasks() SQL : db.rs a deja get_task et
  insert_task. Besoin d'une query list_tasks(status, limit).
  Schema existant tasks(task_id, status, ...) suffisant.

## Telemetrie preflight
- Duree totale : ~2m
- S1a : N/A (port pattern etabli)
- S1b : ~30s / 0 libs nouvelles / clean
- S2 : ~30s / 3 fichiers, 5 commits scannes / clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code Phase C.
