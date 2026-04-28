# Sprint 36 Phase A — preflight G8

Date : 2026-04-28 | HEAD : `148c65f` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : "no band-aid, pick deepest" — singleton DB = solution deep (pas open_in_memory per-request)
- feedback_context7_systematic.md : N/A — pas de nouvelle dep (axum, rusqlite, tokio deja dans workspace)

## Scans (all clean)
- S1a OSS prior art : Phase A = infra refactoring (DaemonHttpState integration). Pattern Arc<Mutex<T>> standard axum — pas de recherche OSS specifique requise (pattern trivial prouve 35 sprints). APPROACH-ALIGNED — clean
- S1b deps : 0 nouvelle dep. rusqlite 0.36 + axum 0.8 + tokio 1.x inchanges — clean
- S2 historiques : 3 fichiers scannes (db.rs, http.rs, main.rs). 1 hit S30 Phase B blob-serve COOP/COEP — sans rapport avec coordinator DB. Archive S34-S35 : 0 rejection sur coordinator integration. Memory feedback : 0 conflit — clean
- S3 threat model : fast-path verified. Pas de nouveau composant securite. HARDENING_ROADMAP pas de ligne S36. Phase A = refactoring interne — clean
- S4 wire format : fast-path verified. VERSION=1 preserve (schemas/mod.rs). Phase A ne touche ni canonical.rs ni schemas/. Day 0 D1-D5 preservees — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : trivial / 0 projets OSS consultes / APPROACH-ALIGNED (pattern standard)
- S1b : trivial / 0 libs scannees (0 nouvelle dep)
- S2 : ~30s / 1 commit scanne / clean
- S3 : fast-path / ~10s
- S4 : fast-path / ~10s

## Action
Proceder code Phase A.
