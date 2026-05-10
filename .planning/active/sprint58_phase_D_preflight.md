# Sprint 58 Phase D — preflight G8

Date : 2026-05-10 | HEAD : `41e9e1f` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid, research before code — N/A tension
- feedback_context7_systematic.md : context7 obligatoire avant code touchant lib/API — iroh-docs queried ci-dessous

## Scans (all clean)
- S1a OSS prior art : 3 projets/sources recherches (iroh-docs, CRDT best practices 2026, real-time sync frameworks), APPROACH-ALIGNED — polling MVP justifie par CSP sandbox `connect-src 'none'` sur iframe, plan documente le chemin SSE futur (daemon → shell → postMessage → iframe). CRDTs + subscribe/push = SOTA, notre contrainte sandbox rend le polling pragmatique pour le MVP
- S1b deps : iroh-docs 0.98 pinne, API `Event::RemoteInsert` + `SubscribeResponse` confirmee via context7 (`/websites/rs_iroh-docs_iroh_docs`), 0 delta — clean
- S2 historiques : 7 fichiers cibles, 0 commit DEVIATION/rejected/scope-cut pertinent a la zone storage sync — clean
- S3 threat model : fast-path verified, pas de nouveau composant securite ni wire format. Storage namespace = interne daemon, pas P2P inter-projets. HARDENING_ROADMAP pas de ligne S58 — clean
- S4 wire format : fast-path verified, `*_VERSION` = 1 inchanges (`schemas/mod.rs` doc comment seul), canonical.rs non touche, Day 0 D1..D4 preservees — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : ~1m / 3 sources consultees / finding : APPROACH-ALIGNED
- S1b : ~30s / 1 lib scannee (iroh-docs 0.98) / finding : clean
- S2 : ~15s / 7 fichiers + archive scan / finding : clean
- S3 : fast-path / ~10s
- S4 : fast-path / ~10s

## Action
Proceder code phase D.
