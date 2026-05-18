# Sprint 65 Phase A — preflight G8

Date : 2026-05-18 | HEAD : `1b3143d` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code, OSS prior art obligatoire (G10)
- feedback_context7_systematic.md : context7 avant tout code touchant lib/API/spec
- sprint14_keyoxide_decision.md : deploy from source, ne jamais re-introduire upload zip public

## Scans (all clean)
- S1a OSS prior art : 3 projets recherches (CloudEvents CNCF, CT RFC 9162, Sigstore Rekor), APPROACH-ALIGNED — le pattern `op_type` discriminant + Value extensible + store/forward unknown ops est identique au design CloudEvents (CNCF standard). Rekor v2 confirme que l'extensibilite initiale est le bon default (simplification ulterieure possible). Clean.
- S1b deps : serde_json stable (Value = core API), jcs crate existant via nexus_core_rs::canonical_bytes, 0 CVE, 0 delta — clean
- S2 historiques : 5 fichiers scannes, 2 commits trouves (S64 cross-review feed, S55 http.rs quorum) — aucun ne concerne raw-op ni auth tier. Memory feedback clean (aucun "never" sur les zones touchees). Clean.
- S3 threat model : FULL scan (nouveau composant securite : auth tier guard feed_insert). THREAT_MODEL.md T0-T5 scannes. Le guard est defense-in-depth sur loopback existant (bearer + Host + Origin). Pas de regression. HARDENING_ROADMAP pas de pre-requirement S65. Limitation header non-crypto documentee D4 design review (⚠️ SCOPE-CUT-CONSISTENT vers T1 post-pilote S69). Clean.
- S4 wire format : FULL scan (FeedEntry.op change de type Rust). FEED_FORMAT_VERSION = 1 preserve. JSON wire identique pour ops connues (serde tag = "op_type" produit le meme JSON que Value avec op_type key). Canonical bytes invariant couvert par test plan L8 (test_canonical_bytes_value_vs_typed). Day 0 D1 mandate ce changement. Pre-launch policy preservee. Clean.

## Telemetrie preflight
- Duree totale : ~4m
- S1a : ~2m / 3 projets OSS consultes (CloudEvents, CT RFC 9162, Rekor) / finding : APPROACH-ALIGNED
- S1b : ~30s / 2 libs scannees (serde_json, jcs) / finding : clean
- S2 : ~30s / 5 fichiers, 2 commits scannes / finding : clean
- S3 : FULL / ~30s / no regression, defense-in-depth improvement
- S4 : FULL / ~30s / VERSION=1, Day 0 preserved, canonical invariant test planned

## Action
Proceder code phase A.
