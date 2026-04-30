# Sprint 42 — Audit plan (Sprint 41 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S41).
**Tip d'entree** : `300b8d3` (S41 Phase C, dernier feat commit).
**Documents source** : `sprint41_kickoff.md` (D1..D5) +
`sprint41_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint41_verification.md` (34/34 fail-fast).

## Track A — Securite / identity modules

- [ ] A-1 : contributor_registry record() idempotent — verifier que
  INSERT OR IGNORE + UNIQUE constraint preserve l'anchor
  first_deploy_ts sur re-record.
- [ ] A-2 : invite wire field — verifier que le champ wire (token
  signe) est stocke mais jamais execute par le ledger.
- [ ] A-3 : capability_store SHA-256 integrity — verifier que
  tampered file → all-OFF fallback fonctionne (roundtrip
  write/load/tamper/reload).

## Track B — Architecture / queues

- [ ] B-1 : quarantine TTL flush_expired — verifier cutoff calcul
  now - ttl_secs correct et que les entrees fraiches survivent.
- [ ] B-2 : upload jitter distribution — verifier que
  pseudo_random_f64 produit une distribution exploitable (pas
  toujours la meme valeur, bornes respectees).
- [ ] B-3 : DB migration #4 — verifier que les 2 tables
  (quarantine_messages + delayed_uploads) sont creees avec index.

## Track C — Tests / coverage

- [ ] C-1 : delta tests cumule 1023→1059 (+36) — verifier chaque
  test teste une branche reelle
- [ ] C-2 : fairness 8 tests — verifier Gini edge cases (empty,
  single, uniform, skewed)
- [ ] C-3 : pow_counter 4 tests — verifier daily reset
- [ ] C-4 : contributor_registry 4 tests — verifier idempotence
- [ ] C-5 : invite 4 tests — verifier revoke double-call
- [ ] C-6 : capability_store 5 tests — verifier tamper detection
- [ ] C-7 : quarantine 5 tests — verifier flush/drop/expire
- [ ] C-8 : upload 6 tests — verifier jitter + status transitions

## Track D — Process / meta

- [ ] D-1 : G8 preflights Phase A + B + C — verifier coherence
- [ ] D-2 : scope cuts 12/12 — verifier aucun viole
- [ ] D-3 : jalon "Python supprimable" — verifier 7/7 modules
  presents dans coordinator-rs

## Track E — Dependencies

- [ ] E-1 : +chrono dans Cargo.toml — verifier version workspace
- [ ] E-2 : DB migrations sequentielles (#2, #3, #4) — verifier
  pas de collision ou skip

## Track F — Doc coherence

- [ ] F-1 : HARDENING_ROADMAP compteurs — verifier 1059 Rust / ~2062 total
- [ ] F-2 : CLAUDE.md etat actuel — verifier S41 CLOSED
- [ ] F-3 : Phase review files present : 3/3 (A + B + C)
- [ ] F-4 : Phase preflight files present : 3/3 (A + B + C)

## Carries S42

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 6+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S39 Tripwire vs Mutation | 2/3 | trait extension post-v1.0 |
| P2-REVIEW-B-1-S39 warn threshold | 2/3 | seuil cadence post-v1.0 |
| P2-REVIEW-B-1-S40 rand_range non-random | 2/3 | rand crate usage |
| P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 | 2/3 | alignment post-v1.0 |
| P2-REVIEW-A-1-S41 conn() pub encapsulation | 1/3 | Phase A review |
| P2-REVIEW-C-1-S41 pseudo_random jitter | 1/3 | Phase C review |
| P3-REVIEW-A-2-S39 LOC kickoff | 2/3 | cosmetic |
| P3-REVIEW-B-2-S39 persist error silent | 2/3 | robustness post-v1.0 |
| P3-AUDIT-A-1-S39 URL single-quote | 2/3 | cosmetic |
| P3-REVIEW-B-1-S40 Manager multiple Mutex | 2/3 | cleanup post-v1.0 |
| P3-REVIEW-C-1-S40 rerun deterministic hash | 2/3 | same pattern Phase B |
| P3-REVIEW-B-1-S41 MintRequest ergonomie | 1/3 | Phase B review |

**Note S42 pair** : S42 est un sprint pair → phase dette obligatoire
(§6.2.1 Regle 1). Les items a 2/3 de S39-S40 atteindront 3/3 apres
S42 s'ils ne sont pas resolus dans la phase dette.
