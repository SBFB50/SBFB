# Sprint 49 Phase A — preflight G8

Date : 2026-05-01 | HEAD : `752b85d` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid, research before code — Phase A fait du wiring structural (dispatcher.rs + validator_loop existants vers project doc), pas de band-aid
- feedback_context7_systematic.md : context7 sur iroh-docs 0.98 — consulte, API create/open/list/author_default confirmee alignee avec le plan

## Scans (all clean)
- S1a OSS prior art : le domaine "daemon coordinating tasks via CRDT document" est le use case natif d'iroh-docs (le projet iroh lui-meme). L'approche du plan (create/reopen doc, write TaskEntry via set_bytes, subscribe LiveEvent pour results) est le pattern standard documente. APPROACH-ALIGNED — clean
- S1b deps : iroh-docs 0.98 pinne (Day 0 #3). context7 `/websites/rs_iroh-docs_iroh_docs` confirme API stable (Docs::create, open, list, author_default, Doc subscribe/set_bytes). 0 nouvelle dep ajoutee. 0 delta — clean
- S2 historiques : 4 fichiers scannes (runtime.rs, http.rs, dispatcher.rs). 4 hits git log (S39 PiiInput, S29 TraceProvider, S20 encryption, S7 daemon skeleton) — aucun pertinent au pattern coordinator-in-daemon. 0 decision rejetee traversee. Archive scan : 1 hit (S22 warrant canary scheduler) — non applicable (threat-model forbidding, pas coordinator pattern). Memory feedback scan : 0 hit sur dispatch/coordinator/lifecycle — clean
- S3 threat model : fast-path verified — Phase A ne cree pas de nouveau composant de securite ni nouveau wire format. HARDENING_ROADMAP : pas de ligne S49 prescrite. Threat model inchange — clean
- S4 wire format : fast-path verified — Phase A ne touche pas canonical.rs ni schemas/. TaskEntry format existant reutilise sans modification. VERSION=1 preserve. Day 0 preserved — clean

## Telemetrie preflight
- Duree totale : ~3m
- S1a : 30s / iroh-docs (projet reference natif) / finding : APPROACH-ALIGNED
- S1b : 60s / 1 lib scannee (iroh-docs context7) / finding : clean
- S2 : 45s / 4 fichiers, 4 commits scannes / finding : clean
- S3 : fast-path / 15s
- S4 : fast-path / 15s

## Action
Proceder code phase A.
