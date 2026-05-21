# Sprint 67 Phase A — preflight G8

Date : 2026-05-20 | HEAD : `d477d81` | Verdict : **EXECUTE plan-as-is**

## G1 pre-condition (Phase A)
sprint67_design_review.md present dans .planning/active/ — OK.

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest technical option, research before code, no band-aids
- feedback_context7_systematic.md : context7 obligatoire pour libs touchees (serde, thiserror deja dans workspace — pas de query context7 requise, versions stables)
- Tensions plan vs memory : aucune

## Scans (all clean)
- S1a OSS prior art : 3 domaines recherches (manifest format → Backstage catalog-info.yaml, curator endorsement → AT Protocol likes/reposts, paginated feed → SSB/AT Proto sequence-based pagination), APPROACH-ALIGNED — le plan suit les patterns etablis. Kickoff D1-D5 ont deja fait la recherche profonde (context7 + WebSearch 2026-05-20) — clean
- S1b deps : 3 libs scannees (serde 1.0, serde_json 1.0, thiserror 1.0), toutes deja dans workspace.dependencies, 0 nouvelle dep a ajouter, 0 CVE — clean
- S2 historiques : 10 fichiers cibles, 10 commits scannes (S64 adversarial feed, S55 quorum, S40 dette HTTP, S39 wire PII, S36 result endpoint, S30 COOP/COEP, S7 daemon creation), 0 DEVIATION/rejected en conflit avec Phase A — clean
- S3 threat model : fast-path verified. Phase A n'introduit pas de nouveau composant securite ni de nouveau wire format VERSION. CuratorVouched/CuratorDisendorsed passent par le pipeline validation existant (verify_entry + validate_feed_operation + rate limiter per-author). GET /api/daemon/feed/entries = read-only, bearer-protected. THREAT_MODEL §10 (Feed surface T-FEED-1..4) couvre deja la surface. HARDENING_ROADMAP pas de ligne S67 — clean
- S4 wire format : FEED_FORMAT_VERSION = 1 preserve. Ajout CuratorVouched/CuratorDisendorsed = nouvelles variantes PublicFeedOperation, PAS de bump version (raw-op pattern P51, serde_json::Value forward compat). SBFB.json v1→v2 = redefinition pre-launch (schema_version: 2), pas de bump compat. Ancien v1 reste parsable via #[serde(default)]. Day 0 preservees : D2 (sbfb-manifest = crate partage), D3 (node_id retire), D9 (CuratorVouched minimal) — clean

## Telemetrie preflight
- Duree totale : ~3m
- S1a : kickoff D1-D5 deja couverts (context7 + WebSearch 2026-05-20) / 3 domaines / finding : clean (APPROACH-ALIGNED)
- S1b : <30s / 3 libs / finding : clean
- S2 : <60s / 10 commits scannes / finding : clean
- S3 : fast-path / <30s
- S4 : full (PublicFeedOperation + SBFB.json) / <60s / finding : clean

## Action
Proceder code Phase A.
