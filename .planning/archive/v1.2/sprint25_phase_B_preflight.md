# Sprint 25 Phase B — preflight G8

Date : 2026-04-22 | HEAD : `2b674db` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, OSS prior art obligatoire (G10), context7 systématique
- sprint14_keyoxide_decision.md : deploy from source (Ed25519 verified) — pas de tension, Phase B étend la couverture Ed25519
- feedback_context7_systematic.md : context7 queried sur ed25519-dalek 2.x API

## Scans (all clean)
- S1a OSS prior art : 5 projets recherchés (KERI, Keybase sigchain, Ceramic Network, SSH rotation, Matrix cross-signing), APPROACH-ALIGNED — le pattern "old key signs rotation announcement + gossip broadcast + transition window" est le standard établi pour les systèmes d'identité décentralisés (KERI rotation events, Keybase reverse_sig, SSH old→new signing)
- S1b deps : ed25519-dalek 2.1 workspace (latest 2.2.0, semver minor, API stable), serde_jcs 0.2, 0 CVE ed25519-dalek 2025-2026 (rustsec.org checked), RUSTSEC-2026-0075 = libcrux-ed25519 différent crate — clean
- S2 historiques : 5 fichiers scannés, 1 commit DEVIATION `04c9621` S18 E2 (canary auto-publish rejeté threat-model : "Ed25519 key accessible auto = compromission"). Pas de conflit : Phase B = rotation manuelle/explicite, pas signature automatisée. Memory feedback scannée, 0 conflit — clean
- S3 threat model : fast-path verified, HARDENING_ROADMAP S25 aligned (key rotation planifié), 0 regression T0-T5 — clean
- S4 wire format : FULL SCAN (canonical.rs dans périmètre). 13 DOMAIN_*_V1 existants, ajout DOMAIN_KEY_ROTATION_V1 = 14e constant même pattern. KEY_ROTATION_FORMAT_VERSION = 1 suit pattern CURATOR_LIST_FORMAT_VERSION / POW_FORMAT_VERSION. Pre-launch policy VERSION=1 préservée. Day 0 D1 implémentée fidèlement — clean

## Telemetrie preflight
- Durée totale : ~3m
- S1a : ~1m30 / 5 projets OSS consultés / finding : APPROACH-ALIGNED (clean)
- S1b : ~30s / 2 libs scannées (ed25519-dalek, serde_jcs) / finding : clean
- S2 : ~30s / 5 fichiers + archive scan / finding : clean
- S3 : fast-path / ~15s
- S4 : full / ~30s

## Action
Procéder code phase B.
