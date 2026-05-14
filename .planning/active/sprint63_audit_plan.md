# Sprint 63 — Audit plan (auditer Sprint 62)

**Ecrit** : 2026-05-14 (Sprint 62 Phase D wrap-up).
**Sprint a auditer** : Sprint 62 (feed sync P2P + anti-spam minimal).
**Tip master Sprint 62** : Phase D commit (ce sprint).

---

## §1 Perimetre de l'audit

Sprint 62 a livre en 4 phases + 6 fix inter-phases :
- Phase A : dette pair F2/F3/F4/F6 (S61 audit P2) + P2-NSIS-UNINSTALL
- Phase B : feed sync foundation iroh-docs (FeedSyncState, boot_feed_namespace, spawn_feed_subscribe, endpoints feed/ticket + feed/join)
- Phase C : catch-up offline + multi-daemon E2E (import_and_subscribe atomique, blob read retry backoff, backfill on join)
- Phase D : anti-spam minimal (FeedRateLimiter GCRA 5/min, PoW 16-bit sur FeedEntry, integration subscribe handler)
- 6 fix : 3 P1 review croisee Phase B + 3 fix Phase C (backfill join, per-author chain hash, publish error propagation, blob read retry, spec is_open_source alignment)
- Delta tests : +17 Rust (1282 → 1299), +0 Vitest

---

## §2 Dimensions a auditer

### 2.1 Feed sync correctness

- `publish_feed_entry_to_docs()` publie toujours avec PoW nonce valide
- `ingest_doc_entry()` rejette entries sans PoW ou avec PoW invalide
- `ingest_doc_entry()` rejette entries d'auteurs au-dessus du rate limit
- Backfill on join applique les memes checks PoW + rate-limit que le live stream
- `import_and_subscribe` atomique : pas de fenetre entre import et subscribe

### 2.2 PoW design

- `pow_nonce` absent de `FeedEntryCanonical` (transport-level uniquement)
- `FEED_FORMAT_VERSION` reste a 1 (pre-launch policy)
- `#[serde(default)]` sur `pow_nonce` documente comme runtime tolerance
- `FEED_POW_DIFFICULTY = 16` : equilibre cout/securite coherent avec P2P_THREATS.md §1.4

### 2.3 Rate limiter isolation

- `FeedRateLimiter` independant de `StorageWriteLimiter` (pas de cross-contamination)
- `retain_recent()` spawned correctement (5 min interval)
- Quota 5/min coherent avec le plan et P2P_THREATS.md sequencing

### 2.4 Wire format stability

- `FeedEntry` serde roundtrip avec et sans `pow_nonce`
- Backward compat : JSON sans pow_nonce deserialise en `None`
- Canonical bytes non impactes par ajout pow_nonce

### 2.5 Items carry a verifier

- P2-IMAGE-DEP image 0.25 (3/3 MANDATORY S63)
- P2-PLAYWRIGHT-REFACTOR global-setup (3/3 MANDATORY S63)
- P2-G-1 exe lock intermittent (reouvert)
- P2-FEED-INSERT-NO-AUTH-TIER (S64+)
- subscribe JoinHandle non trackee

---

## §3 Metriques attendues

| Metrique | Valeur attendue |
|---|---|
| Tests Rust | >= 1299, 0 fail |
| Vitest | >= 258 |
| size-limit | 6/6 |
| clippy | 0 warnings |
| Phases | 4 (A-D) + 6 fix = 10 commits |
