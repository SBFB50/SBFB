# Sprint 64 Phase A — preflight G8

Date : 2026-05-16 | HEAD : `7eb7409` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code, context7 systematic — N/A Phase A est une migration additive + timeout wrapper, patterns standards.
- feedback_context7_systematic.md : tokio timeout + iroh-docs subscribe API — tokio 1.40 stable (training data fiable), iroh-docs 0.98 pinned (pas de delta possible).

## Scans (all clean)
- S1a OSS prior art : domaine = "SQLite migration additive + async subscribe timeout". Patterns universels (rusqlite ALTER TABLE, tokio::time::timeout). APPROACH-ALIGNED — clean.
- S1b deps : tokio 1.40 (workspace, stable), iroh-docs 0.98 (pinned). 0 nouvelle dep. 0 CVE. Clean.
- S2 historiques : 4 fichiers scannes (db.rs, deploy.rs, http.rs, feed_sync.rs), 1 commit pertinent (S55 Phase C quorum) mais sans rapport avec version storage ni timeout. Archive v2.0 scan : 0 finding. Clean.
- S3 threat model : fast-path verified. Pas de nouveau composant securite. M13 = colonne SQLite additive locale. Timeout = mesure defensive. HARDENING_ROADMAP aligned. Clean.
- S4 wire format : fast-path verified. Aucun fichier canonical.rs/schemas touche. VERSION=1 preserve. Day 0 D1-D5 non rebattues. Clean.

## Telemetrie preflight
- Duree totale : ~3m
- S1a : 30s / 0 projets OSS consultes (patterns triviaux) / finding : clean (APPROACH-ALIGNED)
- S1b : 30s / 2 libs scannees (tokio, iroh-docs) / finding : clean
- S2 : 60s / 4 fichiers scannes / finding : clean
- S3 : fast-path / 30s
- S4 : fast-path / 30s

## Action
Proceder code phase A.
