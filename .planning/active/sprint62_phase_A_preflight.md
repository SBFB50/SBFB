# Sprint 62 Phase A — preflight G8

Date : 2026-05-14 | HEAD : `30c2f94` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code, OSS prior art obligatoire, no band-aids
- feedback_context7_systematic.md : context7 avant code touchant lib/API — Phase A utilise Ed25519/BLAKE3/rusqlite existants, pas de nouvelle dep → pas de query context7 requise (deps internes SBFB)

## Scans (all clean)
- S1a OSS prior art : 3 projets recherches (SSB Secure Scuttlebutt, AT Protocol/Bluesky, Wirken), **APPROACH-ALIGNED** — le modele per-author append-only feed avec Ed25519 signatures + hash chain independant par auteur est le pattern SSB eprouve (ssb-db). Notre A5 (verify_chain multi-auteur = verification independante par auteur) correspond exactement au modele SSB ou chaque identite a un feed unique verifie separement. Pas de LIB-EXISTS (SSB = JS, notre format est specifique) — clean
- S1b deps : 0 nouvelle dep. ed25519-dalek, blake3, rusqlite deja en usage. Pas de bump majeur en scope — clean
- S2 historiques : 4 fichiers cibles scannes (public_feed.rs, feed_materializer.rs, PUBLIC_FEED_SPEC.md, Packager.toml), 0 commit DEVIATION/rejected/scope-cut sur ces fichiers. Archive scan : decisions historiques S18-S21 sur warrant canary / Ed25519 key storage, hors-scope Phase A (feed store, pas canary) — clean
- S3 threat model : fast-path verified. Phase A durcit l'existant (validation + atomicite + verification), n'introduit pas de nouveau composant de securite ni nouveau wire format. HARDENING_ROADMAP pas d'entree S62 specifique. Couverture threats inchangee — clean
- S4 wire format : **FULL SCAN** (public_feed.rs contient `FEED_FORMAT_VERSION`). FEED_FORMAT_VERSION = 1 preserve, aucun bump prevu. Phase A ajoute validation + atomic tx + multi-author verify sur le format existant, ne modifie pas la structure FeedEntry/FeedEntryCanonical. Day 0 D1-D5 preservees. Pre-launch protocol policy respectee — clean

## Telemetrie preflight
- Duree totale : ~3m
- S1a : ~2m / 3 projets OSS consultes (SSB, AT Protocol, Wirken) / finding : APPROACH-ALIGNED (clean)
- S1b : ~30s / 3 deps existantes scannees / finding : clean
- S2 : ~30s / 4 fichiers + archive / finding : clean
- S3 : fast-path / ~15s
- S4 : full / ~30s / FEED_FORMAT_VERSION=1 preserve

## Action
Proceder code phase A.
