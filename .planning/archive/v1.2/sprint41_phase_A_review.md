# Phase Review — Sprint 41 Phase A

## Verdict : PASS (1 P2 + 1 P3)

Rigor signal : 2 findings documentes (>=1 requis pour PASS).

## Memory consultation
- feedback_approach.md : pick deepest — N/A (formules math standard)
- fairness_vision.md : metriques observationnelles, pas kudos-v2 — respecte
- feedback_kudos_non_monetary.md : pas de cost/deposit — N/A

## Staging check (Step 1bis)
- Phase fichiers : 10 (2 NEW + 5 fmt + 1 dep + 1 migration + 1 mod)
- Cargo.lock : +chrono dep directe (minimal, attendu)
- Untracked : tools/babel-scraper/ (pre-existant, hors scope)
- Planning split : N/A (pas de doc planning modifie)

## Suites
- cargo fmt : PASS
- cargo clippy : PASS (4 fixes canary_input.rs lints Rust 1.94)
- Rust nextest : 1035 (+12 vs 1023) PASS
- Release build : PASS
- Python ruff : PASS
- Frontend lint+tsc+vitest : PASS (inchange)

## Delta tests
- Plan : +8, reel : +12 (8 fairness + 4 pow_counter)
- Ecart +4 : gini_empty_and_single, top_k_share_empty,
  churn_rate_empty_previous, separate_consumer_model_pairs
  (couverture meilleure que prevu, pas de test manquant)

## Modified-file branch coverage (G9)
- canary_input.rs : clippy style fixes, 0 nouvelle branche — PASS
- db.rs : +1 migration + conn() pub — migration testee via
  open_in_memory() dans pow_counter tests — PASS
- guardrails/honeypot/watermark/http : reformatage uniquement — PASS

## Scope cuts verification (12/12)
- Wire HTTP handlers S42-44 : 0 route ajoutee — PASS
- Background loops S42-44 : 0 tokio::spawn — PASS
- Wire dispatcher S42 : 0 modification dispatcher — PASS
- 9 autres scope cuts : 0 match — PASS

## Horizon long-terme
- Design doc : N/A (modules < 100 LOC, pas structurants)
- D1..D5 alternatives : documentees kickoff §4 — PASS
- Solution poussee : Gini = formule standard, pas de shortcut — PASS
- LOC estime au plan : 0 match grep — PASS

## Research grounding
- S1a preflight : Gini APPROACH-ALIGNED, pow_counter trivial — PASS
- Deps : chrono 0.4 workspace, rusqlite 0.36 workspace — PASS

## Findings

- **P2** : conn() rendu pub dans db.rs retire l'encapsulation
  CoordinatorDb. Tout module peut executer du SQL arbitraire.
  Acceptable pre-v1.0 (7 modules S41 en dependent), mais post-v1.0
  un pattern typed-query (ex: methods sur CoordinatorDb par domaine)
  serait preferable. Carry S42.
- **P3** : canary_input.rs clippy fixes (io::Error::other,
  is_some_and, collapsed if) sont des fixes pre-existants declenches
  par mise a jour lints Rust 1.94, pas du code Phase A.

## Recommendation
- Ready to commit : oui
- Carry S42 : P2-REVIEW-A-1-S41 conn() pub encapsulation
