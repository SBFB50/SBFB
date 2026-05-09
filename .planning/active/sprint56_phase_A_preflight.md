# Sprint 56 Phase A — preflight G8

Date : 2026-05-09 | HEAD : `d60e533` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest technical option — SQLite
  migration dans DB existante est l'option la plus integree
- feedback_context7_systematic.md : N/A — pas de nouvelle lib
  (rusqlite deja dans workspace)

## Scans (all clean)
- S1a OSS prior art : 3 projets recherches (IPFS local datastore,
  libp2p gossipsub in-memory, iroh-docs replication), APPROACH-ALIGNED
  — persistence locale d'annonces P2P est un pattern standard. Clean.
- S1b deps : 0 nouvelle dep. rusqlite + rusqlite_migration deja
  workspace. 0 delta. Clean.
- S2 historiques : db.rs 0 match DEVIATION/rejected. Commit
  `77a8f78` (S53 Phase F) a introduit l'outbox Vec en memoire avec
  carry documente pour persistence. Pas de decision de rejet de la
  persistence. Clean.
- S3 threat model : fast-path verified. Phase A ajoute persistence
  locale (opaque bytes deja signes PoW). Pas de nouveau composant
  securite. Pas de regression T0-T5. HARDENING_ROADMAP S56 N/A. Clean.
- S4 wire format : fast-path verified. 0 fichier canonical.rs/schemas
  touche. VERSION=1 preservees. Day 0 preservees. Outbox stocke des
  enveloppes deja serialisees (Vec<u8> opaque). Clean.

## Telemetrie preflight
- Duree totale : ~2m
- S1a : 30s / 3 projets OSS / finding : APPROACH-ALIGNED
- S1b : 10s / 0 libs nouvelles / finding : clean
- S2 : 30s / 2 fichiers + git log / finding : clean
- S3 : fast-path / 20s
- S4 : fast-path / 20s

## Action
Proceder code phase A.
