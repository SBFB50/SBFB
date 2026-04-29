# Sprint 40 — Audit plan (Sprint 39 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S39).
**Tip d'entree** : `09d490f` (S39 Phase C, dernier feat commit).
**Documents source** : `sprint39_kickoff.md` (D1..D5) +
`sprint39_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint39_verification.md` (31/31 fail-fast).

## Track A — Securite / PII redactor

- [ ] A-1 : regex patterns — verifier que les 7 patterns Rust
  matchent les memes inputs que les 7 patterns Python (email, phone,
  credit card, IBAN, SSN, IP, URL). Tester sur les memes fixtures.
- [ ] A-2 : Luhn validation — verifier que luhn_valid() Rust produit
  les memes resultats que _luhn_valid() Python sur un echantillon
  (Visa 4111..., Amex 3782..., invalide 1234...)
- [ ] A-3 : PiiInputGuardrail — verifier que le wire dans submit_task
  bloque effectivement une soumission contenant un email (test HTTP
  integration absent Phase C)

## Track B — Architecture / CanaryRegistry

- [ ] B-1 : persistence atomique — verifier que persist() ecrit
  tmp + rename (pas write direct qui peut corrompre)
- [ ] B-2 : freshness classification — verifier les seuils
  WARN_THRESHOLD_DAYS=30, ALARM_THRESHOLD_DAYS=45 vs Python
- [ ] B-3 : coerce_canary_payload v→version rename — verifier
  que le rename fonctionne et qu'un payload sans "v" ni "version"
  echoue

## Track C — Tests / coverage

- [ ] C-1 : delta tests cumule 968→991 (+23) — verifier chaque test
  teste une branche reelle pas un stub
- [ ] C-2 : PII tests 14/14 — verifier que les 7 patterns ont chacun
  au moins 1 test
- [ ] C-3 : CanaryRegistry tests 9/9 — verifier observe/freshness/
  health/persist/coerce couverture

## Track D — Process / meta

- [ ] D-1 : G8 preflights Phase A + B + C — verifier coherence
  preflight vs code livre
- [ ] D-2 : scope cuts 12/12 — verifier que aucun scope cut n'est
  viole
- [ ] D-3 : P2-REVIEW-A-1-S37 launcher logging — verifier que la
  resolution est bien complete (test couvre invariant)

## Track E — Dependencies

- [ ] E-1 : regex crate dans Cargo.lock — verifier version et
  pas de RustSec advisory
- [ ] E-2 : pas de nouvelle dep transitive inattendue

## Track F — Doc coherence

- [ ] F-1 : HARDENING_ROADMAP compteurs — verifier 991 Rust / ~1994 total
- [ ] F-2 : CLAUDE.md etat actuel — verifier coherence
- [ ] F-3 : Phase review files present : 3/3 (A + B + C)
- [ ] F-4 : Phase preflight files present : 3/3 (A + B + C)

## Carries S40

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 6+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P3-grammar executor | 3/3+ | defer Rust pipeline S40 |
| P3-watermark executor | 3/3+ | defer Rust pipeline S40 |
| P2-REVIEW-A-1-S38 result_event_tx dead code | 2/3 | wire gossip S40+ |
| P2-REVIEW-B-1-S38 substring O(n*m) | 2/3 | perf post-v1.0 |
| P2-REVIEW-C-1-S38 chain Arc singleton | 2/3 | perf post-v1.0 |
| P3-AUDIT-A-2b-S38 lowercase divergence | 2/3 | doc post-v1.0 |
| P2-REVIEW-A-1-S39 Tripwire vs Mutation | 1/3 | trait extension post-v1.0 |
| P2-REVIEW-B-1-S39 warn threshold | 1/3 | seuil cadence post-v1.0 |
| P2-REVIEW-C-1-S39 HTTP integration tests | 1/3 | post-v1.0 |
| P3-REVIEW-A-2-S39 LOC kickoff | 1/3 | cosmetic |
| P3-REVIEW-B-2-S39 persist error silent | 1/3 | robustness post-v1.0 |
