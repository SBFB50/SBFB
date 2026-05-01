# Sprint 50 Phase A — preflight G8

Date : 2026-05-01 | HEAD : `ac5764e` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — Phase A fixe 2 carries P2 existants (JoinHandle + CLI tests), wiring structural. Conforme.
- feedback_context7_systematic.md : N/A — pas de nouvelle lib, pas d'API externe.

## Scans (all clean)
- S1a OSS prior art : le domaine "tokio JoinHandle lifecycle management" est un pattern standard Rust async. L'approche (store handle, join at shutdown) est le pattern recommande par tokio. APPROACH-ALIGNED — clean
- S1b deps : 0 nouvelle dep. runtime.rs et main.rs utilisent tokio + nexus-coordinator-rs deja en place. 0 delta — clean
- S2 historiques : 1 hit git log (S39 PiiInput wire http.rs) — non pertinent au JoinHandle/CLI tests. S49 preflight avait deja scanne runtime.rs S2 = clean. 0 decision rejetee traversee — clean
- S3 threat model : fast-path verified — Phase A ne cree pas de composant securite. JoinHandle = lifecycle interne daemon. CLI tests = exercent DB existante. HARDENING_ROADMAP : pas de ligne S50 — clean
- S4 wire format : fast-path verified — Phase A ne touche pas canonical.rs ni schemas. 0 VERSION bump. Day 0 preservees — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : 20s / tokio standard pattern / APPROACH-ALIGNED
- S1b : 10s / 0 nouvelle dep
- S2 : 30s / 2 fichiers, 1 commit (non pertinent)
- S3 : fast-path / 15s
- S4 : fast-path / 15s

## Action
Proceder code Phase A.
