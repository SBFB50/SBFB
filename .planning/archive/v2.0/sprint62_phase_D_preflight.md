# Sprint 62 Phase D — preflight G8

Date : 2026-05-14 | HEAD : `c1b5b0c` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest option, research before code, OSS prior art obligatoire (G10)
- feedback_context7_systematic.md : context7 obligatoire avant code touchant lib/API — governor queried ✓

## Scans (all clean)
- S1a OSS prior art : 4 projets recherchés (Waku RLN, Secure Scuttlebutt, libp2p specs #374, Hashcash ref P2P_THREATS.md), APPROACH-ALIGNED — plan suit le sequencing P2P_THREATS.md §1.5 "PoW Hashcash per-identity = premier palier anti-spam", GCRA rate-limit = pattern prouvé (governor crate, déjà 2 instances codebase). Waku RLN = ZK-based economic spam protection (beaucoup plus lourd, scope post-launch). SSB = social graph filtering (pas de PoW). Plan Phase D = minimal anti-spam cohérent avec la posture pre-launch.
- S1b deps : governor 0.10.2 (workspace pin), confirmé latest via crates.io + context7. 0 delta, 0 CVE — clean
- S2 historiques : git log feed_sync.rs + public_feed.rs + storage_limiter.rs = 0 DEVIATION/rejected. Archive v*/ = 0 mention feed anti-spam. Memory feedback = 0 contrainte feed/spam/PoW — clean
- S3 threat model : fast-path verified. P2P_THREATS.md §1.4 documente PoW Hashcash "Low-Med impact T1-T2, offloadable T3+" = cohérent scope "anti-spam minimal". Pas de nouveau composant de sécurité (rate limiter suit pattern storage_limiter.rs). HARDENING_ROADMAP = pas de pre-requirement S62 — clean
- S4 wire format : FEED_FORMAT_VERSION=1 inchangé. FeedEntryCanonical non touché (PoW = transport-level, pas signed content). `pow_nonce: Option<u64>` avec `#[serde(default)]` = runtime tolerance légitime (remote entries carry PoW, local entries self-trust → None default). Day 0 préservées — clean

## Telemetrie preflight
- Durée totale : ~3m
- S1a : ~2m / 4 projets OSS consultés (Waku RLN, SSB, libp2p, Hashcash) / finding : APPROACH-ALIGNED
- S1b : ~30s / 1 lib scannée (governor 0.10.2) / finding : clean
- S2 : ~15s / 3 fichiers, 0 commits scannés / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Procéder code phase D.
