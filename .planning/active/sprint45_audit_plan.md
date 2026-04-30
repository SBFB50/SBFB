# Sprint 45 — Audit plan (Sprint 44 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S44).
**Tip d'entree** : `9942d70` (S44 Phase C, dernier feat commit).
**Documents source** : `sprint44_kickoff.md` (D1..D3) +
`sprint44_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint44_verification.md` (30/30 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint44_plan.md,
(3) sprint44_kickoff.md §D1..D3. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint44_audit_findings.md`.

## Track A — MANDATORY batch (Phase A)

- [ ] A-1 : ChainResult doc §P42 — verifier section presente dans
  PATTERNS.md, contrat mutations documente.
- [ ] A-2 : pow_keypair doc §P43 — verifier equivalence
  pow_keypair = iroh node = provenance signer documentee.
- [ ] A-3 : babel-scraper .gitignore — verifier
  `tools/babel-scraper/` n'apparait plus dans `git status`.
- [ ] A-4 : list_apps pagination — verifier limit/offset/total_count
  dans AppListQuery + AppListResponse. Defaut 50, max 500.
- [ ] A-5 : RNG test — verifier test injector_rate_probabilistic
  presente dans canary_input.rs.
- [ ] A-6 : Debug as_str — verifier as_str() sur BrowseStatus et
  BrowseSource, format!("{:?}").to_lowercase() supprime.
- [ ] A-7 : pagination fusionne avec A-4 — verifier coherence.
- [ ] A-8 : prefix route /api/v1/contributor/ — verifier 3 routes +
  test + doc comment mis a jour.

## Track B — Health + shell + kudos + diagnostic (Phase B)

- [ ] B-1 : health_api.rs — verifier 1 route GET /api/v1/coordinator/
  health. Response : node_id, daemon_version, api_host, api_port,
  uptime_secs.
- [ ] B-2 : shell_api.rs — verifier 1 route GET /api/v1/shell/discover.
  Schema version 1, self-only.
- [ ] B-3 : kudos_api.rs — verifier 2 routes (entries + leaderboard).
  list_kudos_entries(worker_node_id) query dans db.rs.
- [ ] B-4 : diagnostic_api.rs — verifier 1 route GET /api/v1/diagnostic/
  fairness. Wire fairness.rs compute_gini/top_k/churn. Precision 4 dec.
- [ ] B-5 : db.rs — verifier 3 nouvelles queries
  (list_kudos_entries, worker_contributions, active_workers_since).
- [ ] B-6 : routes enregistrees dans http.rs — 5 routes Phase B.

## Track C — Tasks + worker_state (Phase C)

- [ ] C-1 : tasks_api.rs — verifier 2 routes (list + get).
  list_tasks(status, limit) query dans db.rs. Defaut limit 100,
  max 500.
- [ ] C-2 : worker_state_api.rs — verifier 1 route GET /api/v1/worker/
  state. Lecture state.json via nexus_grid_root(). 5 branches
  reponse (no file, invalid JSON, schema mismatch, fresh, stale).
  Staleness 15s.
- [ ] C-3 : db.rs — verifier query list_tasks(status, limit).
- [ ] C-4 : routes enregistrees dans http.rs — 3 routes Phase C.

## Track D — Process / meta

- [ ] D-1 : G8 preflights Phase A + B + C — verifier coherence
  (3/3 EXECUTE, 0 DESIGN-CONFLICT).
- [ ] D-2 : scope cuts 6/6 — verifier aucun viole (diff --stat).
- [ ] D-3 : 7/7 MANDATORY items resolus — verifier dans le diff.
- [ ] D-4 : sprint pair phase dette Phase A — verifier (§6.2.1 R1).

## Track E — Doc coherence

- [ ] E-1 : HARDENING_ROADMAP compteurs — verifier 1127 Rust / ~2130 total
- [ ] E-2 : CLAUDE.md etat actuel — verifier S44 CLOSED + carries S45
- [ ] E-3 : SPRINT_LOG.md — verifier row S44 presente
- [ ] E-4 : Phase review files present : 3/3 (A + B + C)
- [ ] E-5 : Phase preflight files present : 3/3 (A + B + C)
- [ ] E-6 : PATTERNS.md §P42 + §P43 — verifier contenu coherent

---

## Carries S45

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 10+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 | 6/3 | exemption dep S45 |
| P2-REVIEW-B-1-S43 coord dead_code cleanup | 2/3 | |
| P2-AUDIT-A-1-S43 integration test gap 12 routes | 2/3 | |
| P2-REVIEW-A-1-S44 as_str/serde coupling | 1/3 | NEW |
| P2-REVIEW-B-1-S44 kudos entries pagination | 1/3 | NEW |
| P2-REVIEW-C-1-S44 worker_state tokio::fs | 1/3 | NEW |
| P3-REVIEW-A-1-S43 TOCTOU canary reload | 2/3 | |
| P3-AUDIT-A-2-S43 silent null canary_api | 2/3 | |
| P3-AUDIT-A-3-S43 hex case-sensitivity | 2/3 | |
| P3-REVIEW-B-2-S44 shell discover self-only | 1/3 | NEW |
| P3-REVIEW-C-2-S44 list_tasks status non valide | 1/3 | NEW |

**Resolus S44** : P2-REVIEW-A-1-S42 ChainResult (doc §P42),
P2-REVIEW-B-1-S42 pow_keypair (doc §P43),
P3-REVIEW-A-2-S42 babel-scraper (.gitignore),
P3-REVIEW-C-1-S42 list_apps aggregate (pagination),
P3-AUDIT-A-1-S42 couverture RNG rate>1 (test),
P3-AUDIT-C-1-S42 Debug vs serde (as_str),
P3-AUDIT-C-2-S42 pagination limit/offset (fusionne apps),
P3-REVIEW-C-1-S43 prefix route contributor (/api/v1/).

**Note S45 impair** : S45 est impair → pas de phase dette obligatoire
(§6.2.1 Regle 1). Items P2 a 2/3 approchent MANDATORY (3/3 au
prochain carry).

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S45 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint44): ... avant S45 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D3 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S44 (decision sprint, pas audit)
- events.py SSE (scope-cut explicite, dep SDK Python)

## Livrable attendu

`sprint44_audit_findings.md` avec : verdict global, section par
track, findings P0→P3, commits fix attendus si CONDITIONAL PASS.
