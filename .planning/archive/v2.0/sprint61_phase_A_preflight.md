# Sprint 61 Phase A — preflight G8

Date : 2026-05-13 | HEAD : `7b6c205` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, research before code, OSS prior art obligatoire (G10)
- feedback_context7_systematic.md : context7 avant code/decision (N/A — pas de nouvelle dep)
- Tensions plan vs memory : aucune

## Scans (all clean)

- S1a OSS prior art : 3 projets recherches (Secure Scuttlebutt, Certificate Transparency, p2panda), APPROACH-ALIGNED — append-only signed log avec hash-chain et signature par entree est le pattern standard. SSB utilise exactement le meme modele. CT utilise Merkle tree (plus complexe, pour echelle globale, pas necessaire pour feed local). p2panda deja etudie (65-75% recherche dans `p2panda_public_protocol_briques.md`). Clean.
- S1b deps : 0 dep nouvelle. blake3 1.5, ed25519-dalek, serde_jcs deja dans le workspace. 0 CVE. Clean.
- S2 historiques : 10 commits sur canonical.rs (Sprint 4, 7, 10, 14, 18, 19, 20, 22, 23, 25), tous des ajouts de domaines. 0 DEVIATION, 0 rejected, 0 scope-cut sur la zone. Clean.
- S3 threat model : **FULL SCAN** (nouveau wire format DOMAIN_FEED_V1). Feed local = stockage SQLite append-only. Pas de nouveau vecteur reseau (P2P sync = Sprint 2). Hash-chain BLAKE3 + Ed25519 per-entry couvrent T2 data integrity + T3 identity. Feed public = T4 privacy non-concern. T0/T1/T5 non-applicable au stockage local. 0 regression, 0 gap non-documente. Clean.
- S4 wire format : **FULL SCAN** (canonical.rs cible). 14 domaines existants, pattern `b"nexus-{name}-v1"` + doc comment. DOMAIN_FEED_V1 = 15e domaine, meme pattern. `*_VERSION` = uniquement en commentaires doc (key_rotation.rs, schemas/mod.rs), pas de version field a bumper. FEED_FORMAT_VERSION = 1 (nouveau format, post-v1.0 regime). Day 0 D1..D5 preservees. Pre-launch protocol policy : tag v1.0 pose, post-v1.0 regime actif, FEED_FORMAT_VERSION = 1 est correct. Clean.

## Telemetrie preflight

- Duree totale : ~5m
- S1a : 2m / 3 projets OSS / finding : APPROACH-ALIGNED (clean)
- S1b : 30s / 3 libs scannees / finding : clean
- S2 : 1m / 10 commits + archive scan / finding : clean
- S3 : full / 1m / 0 regression
- S4 : full / 30s / 14 domaines + VERSION scan clean

## Action

Proceder code phase A.
