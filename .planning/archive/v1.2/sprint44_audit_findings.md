# Sprint 44 — Audit findings (Phase 0 gate S45)

**Auditeur** : session fraiche independante.
**Tip audite** : `9942d70` (S44 Phase C, dernier feat commit).
**Documents lus** : sprint45_audit_plan.md, sprint44_kickoff.md,
sprint44_plan.md, sprint44_verification.md, code source direct.
**Date** : 2026-04-30.

---

## Verdict global : PASS

0 P0, 0 P1 — S45 Phase A peut demarrer directement.
3 P2 (tous carries pre-existants confirmes) + 3 P3 (1 carry + 2
nouveaux). G4 rigor signal satisfait (>=1 P2+ documente).

---

## Track A — MANDATORY batch (Phase A)

| Item | Verdict | Notes |
|---|---|---|
| A-1 ChainResult doc §P42 | PASS | Section presente PATTERNS.md l.2242, contrat mutations documente |
| A-2 pow_keypair doc §P43 | PASS | Section presente l.2263, 3 roles + equivalence Python |
| A-3 babel-scraper .gitignore | PASS | `tools/babel-scraper/` dans .gitignore l.146 |
| A-4 list_apps pagination | PASS | limit/offset/total_count, defaut 50, max 500, skip/take |
| A-5 RNG test | PASS | injector_rate_probabilistic canary_input.rs:785, 2000 iter, 10-35% |
| A-6 Debug as_str | PASS | as_str() sur BrowseStatus + BrowseSource, format!("{:?}") absent |
| A-7 pagination fusionne | PASS | Coherent avec A-4, meme struct AppListQuery |
| A-8 prefix /api/v1/contributor/ | PASS | 3 routes, test, doc comment mis a jour |

### Findings Track A

**P2-REVIEW-A-1-S44** (carry confirme) : `as_str()` et
`serde(rename_all = "lowercase")` retournent les memes valeurs
mais le couplage n'est pas enforce par le compilateur. Un ajout
de variant pourrait causer un drift silencieux. Carry S45.

**P3-AUDIT-A-1-S44** (nouveau) : le test `app_list_query_pagination`
verifie uniquement la deserialisation JSON du query — il ne teste
pas la logique `skip().take()` avec un vrai Vec d'apps. Couverture
implicite via le chemin HTTP integration, mais pas de test unitaire
isole du handler logic.

---

## Track B — Health + shell + kudos + diagnostic (Phase B)

| Item | Verdict | Notes |
|---|---|---|
| B-1 health_api.rs | PASS | 1 route, 5 champs requis + status, uptime correct |
| B-2 shell_api.rs | PASS | 1 route, schema v1, self-only documente |
| B-3 kudos_api.rs | PASS | 2 routes (entries + leaderboard), queries parametrees |
| B-4 diagnostic_api.rs | PASS | 1 route, gini/top_k/churn, precision 4 dec confirmee |
| B-5 db.rs queries | PASS | 3 queries parametrees, 0 risque injection SQL |
| B-6 routes http.rs | PASS | 5 routes Phase B enregistrees + mod declarations |

### Findings Track B

**P2-REVIEW-B-1-S44** (carry confirme) : `list_entries` retourne
toutes les entries sans pagination. Fonctionnel sur petit reseau,
probleme a l'echelle. Carry S45.

**P3-AUDIT-B-1-S44** (nouveau) : `diagnostic_api.rs` l.38 :
`worker_contributions()` erreur DB → silent fallback `vec![]`
au lieu de 500. Une erreur DB ressemble a "zero workers" plutot
qu'a une erreur. Defensif mais peut masquer un probleme reel.

---

## Track C — Tasks + worker_state (Phase C)

| Item | Verdict | Notes |
|---|---|---|
| C-1 tasks_api.rs | PASS | 2 routes, limit defaut 100 cap 500, 3 tests unitaires |
| C-2 worker_state_api.rs | PASS | 1 route, 5 branches reponse, staleness 15s |
| C-3 db.rs list_tasks | PASS | Query parametree, 2 branches with/without status |
| C-4 routes http.rs | PASS | 3 routes Phase C + mod declarations |

### Findings Track C

**P2-REVIEW-C-1-S44** (carry confirme) : `worker_state_api.rs`
utilise `std::fs::read_to_string` dans un handler async. Bloquant
sur le runtime tokio. Migrer `tokio::fs` en S45.

**P3-REVIEW-C-2-S44** (carry confirme) : `list_tasks` avec un
`state` invalide (ex: `?state=garbage`) passe le filtre a SQL
→ retourne 0 resultats au lieu de 400. Fonctionnellement inoffensif
mais semantiquement incorrect.

---

## Track D — Process / meta

| Item | Verdict | Notes |
|---|---|---|
| D-1 G8 preflights 3/3 | PASS | A + B + C tous EXECUTE, 0 DESIGN-CONFLICT |
| D-2 scope cuts 6/6 | PASS | 0 fichier events.py/quarantine.py/debit/stake touche |
| D-3 7/7 MANDATORY resolus | PASS | Tous adresses dans Phase A commit body |
| D-4 sprint pair dette Phase A | PASS | Phase A = dette obligatoire (§6.2.1 R1) |

0 findings.

---

## Track E — Doc coherence

| Item | Verdict | Notes |
|---|---|---|
| E-1 HARDENING_ROADMAP compteurs | PASS | 1127 Rust / ~2130 total, last_validated 2026-04-30 |
| E-2 CLAUDE.md etat actuel | PASS | S44 CLOSED + carries S45 listes |
| E-3 SPRINT_LOG.md row S44 | PASS | Presente, contenu complet |
| E-4 Phase review files 3/3 | PASS | A + B + C + D presents |
| E-5 Phase preflight files 3/3 | PASS | A + B + C presents, tous EXECUTE |
| E-6 PATTERNS.md §P42 + §P43 | PASS | Contenu coherent avec code et kickoff |

0 findings.

---

## Synthese des findings

| ID | Sev | Track | Description | Action |
|---|---|---|---|---|
| P2-REVIEW-A-1-S44 | P2 | A | as_str/serde coupling non-enforce | Carry S45 (1/3) |
| P2-REVIEW-B-1-S44 | P2 | B | kudos entries sans pagination | Carry S45 (1/3) |
| P2-REVIEW-C-1-S44 | P2 | C | worker_state std::fs bloquant async | Carry S45 (1/3) |
| P3-AUDIT-A-1-S44 | P3 | A | test pagination handler-level absent | Carry S45 |
| P3-AUDIT-B-1-S44 | P3 | B | diagnostic silent fallback vec![] | Carry S45 |
| P3-REVIEW-C-2-S44 | P3 | C | list_tasks status invalide → vide | Carry S45 (1/3) |

Tous les P2 sont des carries pre-existants confirmes par
verification.md §5. 2 P3 sont nouveaux (A-1, B-1), 1 P3 est un
carry confirme (C-2).

---

## Coherence audit plan vs findings

Le sprint45_audit_plan.md listait 13 carries S45 :
- 3 P2 NEW (A-1, B-1, C-1) : **confirmes** par cet audit
- 5 P3 NEW (B-2, C-2) + existants : **confirmes** (B-2 = self-only,
  C-2 = status invalide)
- 5 carries herites (rand, iroh, SHA-256, coord dead_code,
  integration test gap) : **hors scope audit S44** (pre-existants,
  pas touches par S44)

## Verdict final

**PASS — 0 P0, 0 P1.** Sprint 45 Phase A peut demarrer.
