# Sprint 32 Phase B — preflight G8

Date : 2026-04-27 | HEAD : `90aff27` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest technical option, research before code, context7 obligatoire — N/A (dep upgrade standard, pas de design choice)
- feedback_context7_systematic.md : context7 queries effectuées sur rusqlite + arti-client — contrainte satisfaite

## Scans (all clean)
- S1a OSS prior art : 2 projets recherchés (rusqlite, arti/Tor Project), APPROACH-ALIGNED — rusqlite = canonical Rust SQLite binding, arti-client = official Tor Project Rust implementation. Aucune approche naive, aucune lib alternative pertinente. Clean.
- S1b deps : 2 libs scannées (rusqlite 0.36, arti-client 0.41). rusqlite : dernier advisory RUSTSEC-2021-0128 fixé 0.26.2+, API bundled stable 0.32→0.36, 0 delta. arti-client 0.41 : RUSTSEC-2024-0339/0340 (tor-circmgr vanguards) fixés dans versions ultérieures incluses par arti-client 0.41. 0 CVE bloquant — clean.
- S2 historiques : 6 fichiers cibles scannés, 16 commits trouvés — aucun ne rejette rusqlite upgrade ou arti-client dep activation. S31 Phase C a explicitement planifié l'activation S32. S32 kickoff D2 confirme. Archive scan : 0 mention rejection rusqlite/arti/tor. Memory feedback : 0 contrainte violée — clean.
- S3 threat model : fast-path verified. Phase B n'introduit PAS de nouveau composant de sécurité (tor_transport.rs créé S31 Phase C). Activation de la dep réelle = même surface API, même comportement. HARDENING_ROADMAP aligned (arti-client trigger ACTIF, résolu par cette phase) — clean.
- S4 wire format : fast-path verified. 0 fichier canonical.rs/schemas/ touché. rusqlite = storage interne, arti-client = runtime dep réseau. VERSION=1 partout, Day 0 D2 preserved (0.36 pas 0.39) — clean.

## Telemetrie preflight
- Durée totale : ~3m
- S1a : ~1m30 / 2 projets OSS consultés (context7 rusqlite + arti-client) / finding : clean (APPROACH-ALIGNED)
- S1b : ~1m / 2 libs scannées (WebSearch RustSec rusqlite + arti-client) / finding : clean
- S2 : ~30s / 16 commits scannés / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Procéder code phase B.
