# Sprint 32 Phase A — preflight G8

Date : 2026-04-27 | HEAD : `1d5e385` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : research before code, context7 obligatoire, pick deepest — satisfait par research kickoff (context7 + crates.io API + WebSearch iroh releases)
- feedback_context7_systematic.md : context7 queried sur iroh 0.98 Endpoint builder + SecretKey API — N/A direct car notre code utilise `SecretKey::from_bytes()` pas `generate()`

## Scans (all clean)

- S1a OSS prior art : Phase A = migration deps mecanique (iroh 0.97→0.98), pas de probleme de design a challenger. Les 8 breaking changes documentes au kickoff sont des adaptations API standard. APPROACH-ALIGNED — clean
- S1b deps : 4 libs scannees (iroh, iroh-docs, iroh-gossip, iroh-blobs). WebSearch RustSec 2026 + CVE : 0 advisory sur iroh ou satellite crates. context7 iroh docs confirme API Endpoint::builder() + SecretKey. Notre code utilise `from_bytes` pas `generate` = #4075 non-impactant — clean
- S2 historiques : 9 fichiers scannes, git log --grep DEVIATION/rejected/scope-cut. Mentions S7/S9/S16 "iroh 0.97 pinne" = Day 0 #3 explicitement levee S32 D5. S18 DEVIATION = warrant canary, non lie. 0 finding contradictoire — clean
- S3 threat model : fast-path verified. Phase A = migration deps interne, 0 nouveau composant securite, 0 nouveau wire format. HARDENING_ROADMAP trigger "iroh > 0.97" ACTIF adresse ce sprint — clean
- S4 wire format : fast-path verified. `*_VERSION = 1` dans schemas/mod.rs. Phase A ne touche ni canonical.rs ni schemas. Pre-launch protocol preservee. Day 0 D1..D5 preservees — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : context7 /websites/rs_iroh queried (Endpoint builder, SecretKey generate) / finding : APPROACH-ALIGNED
- S1b : WebSearch RustSec + CVE iroh 2026, WebSearch iroh-blobs/docs/gossip breaking changes / 0 advisory / clean
- S2 : 9 fichiers, ~20 commits scannes + archive v1.0+v1.1+v1.2 grep / clean
- S3 : fast-path / THREAT_MODEL.md headers + HARDENING S32 line verified
- S4 : fast-path / VERSION=1 + Day 0 preserved + pre-launch policy confirmed

## Action
Proceder code phase A.
