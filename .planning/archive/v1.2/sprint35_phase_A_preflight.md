# Sprint 35 Phase A — preflight G8

Date : 2026-04-28 | HEAD : `7430166` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — Phase A cree un crate propre au lieu d'etendre daemon-core, conforme
- feedback_context7_systematic.md : rusqlite deja dans workspace (0.36), pas de nouvelle lib externe, context7 non requis

## Scans (all clean)
- S1a OSS prior art : fondation crate + CI + test harness — pas d'algorithme complexe a valider contre l'OSS, pattern standard Rust workspace lib crate. APPROACH-ALIGNED — clean
- S1b deps : rusqlite 0.36 (existant), tokio/serde/thiserror/tracing (existants workspace), 0 nouvelle dep — clean
- S2 historiques : Sprint 21 Phase A pivot note `/task/submit` vit en Python coordinator, pas en Rust daemon. Coherent avec la migration S35 (on deplace intentionnellement vers Rust). Pas de decision rejected qui contredit la creation de nexus-coordinator-rs. Reversion check : N/A (pas de conflit) — clean
- S3 threat model : fast-path verified. Phase A ne cree pas de nouveau composant securite ni wire format. HARDENING_ROADMAP S35 = Gate 4 horizon, pas de pre-requirement bloquant — clean
- S4 wire format : fast-path verified. `*_VERSION = 1` dans schemas/mod.rs, Phase A ne touche pas canonical.rs ni schemas/. Day 0 preservees — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : instant / 0 projets OSS consultes (fondation crate, pas d'algo) / clean
- S1b : instant / 5 deps verifiees (toutes existantes) / clean
- S2 : 30s / 2 scans (git log + archive grep) / clean
- S3 : fast-path / instant
- S4 : fast-path / instant

## Action
Proceder code phase A.
