# Sprint 39 — Audit plan (Sprint 38 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S38).
**Tip d'entree** : `16ad15e` (S38 Phase C, dernier feat commit).
**Documents source** : `sprint38_kickoff.md` (D1..D5) +
`sprint38_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint38_verification.md` (32/32 fail-fast).

## Track A — Securite / output filter

- [ ] A-1 : invisible text scanner — verifier que les ranges Unicode
  couvrent les memes categories que le Python (zero-width, PUA, tags)
  et que la whitelist bidi est identique (U+202A-E, U+2066-69)
- [ ] A-2 : prompt echo EED — verifier que strsim normalized_levenshtein
  produit des resultats comparables a rapidfuzz Python sur les memes
  inputs (seuil 0.85)
- [ ] A-3 : guardrail wire dans submit_result — verifier que le
  tripwire path bloque effectivement le credit kudos (pas de credit
  avant guardrail check)

## Track B — Architecture / validator_loop

- [ ] B-1 : broadcast channel capacity 64 — verifier que c'est
  suffisant et que RecvError::Lagged est gere (log + continue)
- [ ] B-2 : idempotence — verifier que set_task_result() WHERE status
  garantit qu'un double submit via HTTP + validator_loop ne credite
  qu'une fois
- [ ] B-3 : result_event_tx dead code — verifier que le champ est
  correctly allow(dead_code) et que le broadcast sender est stocke
  sans leak

## Track C — Tests / coverage

- [ ] C-1 : delta tests cumule 946→967 (+21) — verifier chaque test
  teste une branche reelle pas un stub
- [ ] C-2 : output_filter tests — verifier que les 10 tests couvrent
  les 3 layers (invisible, echo cascade, EED)
- [ ] C-3 : guardrails tests — verifier que les 6 tests couvrent
  empty/pass/flag/tripwire + OutputSafety integration

## Track D — Process / meta

- [ ] D-1 : G8 preflights Phase A + B + C — verifier coherence
  preflight vs code livre (pas de drift plan→code non documente)
- [ ] D-2 : scope cuts — verifier que les 12 scope cuts §7 du kickoff
  ne sont pas violes
- [ ] D-3 : MANDATORY 3/3 ferme — verifier que validator_loop est
  effectivement spawne et fonctionnel (pas juste declare)

## Track E — Dependencies

- [ ] E-1 : strsim 0.11 dep directe coordinator-rs — verifier que
  Cargo.lock est a jour et que strsim est dans le lockfile
- [ ] E-2 : verifier que strsim n'a pas de RustSec advisory

## Track F — Doc coherence

- [ ] F-1 : HARDENING_ROADMAP compteurs — verifier 967 Rust / ~1970 total
- [ ] F-2 : CLAUDE.md etat actuel — verifier coherence avec verification.md
- [ ] F-3 : Phase review files present : 3/3 (A + B + C)
- [ ] F-4 : Phase preflight files present : 3/3 (A + B + C)
- [ ] F-5 : PATTERNS.md P33 rowid — verifier presence et coherence

## Carries S39

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 6+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P3-grammar executor | 3/3+ | defer Rust pipeline S40 |
| P3-watermark executor | 3/3+ | defer Rust pipeline S40 |
| P2-REVIEW-A-1-S38 result_event_tx dead code | 1/3 | wire gossip S39+ |
| P2-REVIEW-B-1-S38 substring O(n*m) | 1/3 | perf post-v1.0 |
| P2-REVIEW-C-1-S38 chain Arc singleton | 1/3 | perf post-v1.0 |
| P2-REVIEW-A-1-S37 launcher logging test | 2/3 | Phase A partial |
