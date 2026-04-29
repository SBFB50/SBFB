# Sprint 41 — Audit plan (Sprint 40 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S40).
**Tip d'entree** : `0b9df49` (S40 Phase C, dernier feat commit).
**Documents source** : `sprint40_kickoff.md` (D1..D5) +
`sprint40_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint40_verification.md` (34/34 fail-fast).

## Track A — Securite / canary_input

- [ ] A-1 : Ed25519 sign/verify — verifier que signable_json()
  produit le meme canonical JSON que le Python (sort_keys).
  Tester sign+verify+tamper roundtrip.
- [ ] A-2 : Levenshtein similarity — verifier que
  strsim::normalized_levenshtein donne les memes resultats que
  rapidfuzz sur 3 samples (exact match, partial, zero).
- [ ] A-3 : CanaryInputGuardrail Tripwire — verifier que le mode
  Tripwire est coherent avec PiiInputGuardrail S39.

## Track B — Architecture / Tier 3 modules

- [ ] B-1 : redundancy SHA-256 — verifier parite hash avec Python
  hashlib.sha256 sur 2 samples
- [ ] B-2 : watermark PRF — verifier que prf_score() coordinator
  produit les memes scores que prf_score() worker (crosscheck
  crates/nexus-worker-core/src/llm/watermark.rs)
- [ ] B-3 : rerun anti-loop — verifier que is_rerun() empeche
  re-run de re-run
- [ ] B-4 : honeypot eclipse threshold — verifier seuils 0.8 +
  3 rotations consecutives

## Track C — Tests / coverage

- [ ] C-1 : delta tests cumule 991→1023 (+32) — verifier chaque
  test teste une branche reelle
- [ ] C-2 : canary_input 13 tests — verifier couverture composants
- [ ] C-3 : Tier 3 16 tests — verifier couverture 4 modules

## Track D — Process / meta

- [ ] D-1 : G8 preflights Phase A + B + C — verifier coherence
- [ ] D-2 : scope cuts 12/12 — verifier aucun viole
- [ ] D-3 : dette pair 5 items — verifier tous resolus

## Track E — Dependencies

- [ ] E-1 : +toml +sha2 +hmac dans Cargo.toml — verifier versions
  workspace, pas de RustSec advisory
- [ ] E-2 : pas de nouvelle dep transitive inattendue

## Track F — Doc coherence

- [ ] F-1 : HARDENING_ROADMAP compteurs — verifier 1023 Rust / ~2026 total
- [ ] F-2 : CLAUDE.md etat actuel — verifier S40 CLOSED
- [ ] F-3 : Phase review files present : 3/3 (A + B + C)
- [ ] F-4 : Phase preflight files present : 3/3 (A + B + C)

## Carries S41

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 6+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S39 Tripwire vs Mutation | 1/3 | trait extension post-v1.0 |
| P2-REVIEW-B-1-S39 warn threshold | 1/3 | seuil cadence post-v1.0 |
| P2-REVIEW-B-1-S40 rand_range non-random | 1/3 | rand crate usage post-v1.0 |
| P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 | 1/3 | alignment post-v1.0 |
| P3-REVIEW-A-2-S39 LOC kickoff | 1/3 | cosmetic |
| P3-REVIEW-B-2-S39 persist error silent | 1/3 | robustness post-v1.0 |
| P3-AUDIT-A-1-S39 URL single-quote | 1/3 | cosmetic |
| P3-REVIEW-B-1-S40 Manager multiple Mutex | 1/3 | cleanup post-v1.0 |
| P3-REVIEW-C-1-S40 rerun deterministic hash | 1/3 | same pattern Phase B |
