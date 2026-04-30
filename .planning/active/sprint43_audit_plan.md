# Sprint 43 — Audit plan (Sprint 42 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S42).
**Tip d'entree** : `87ee663` (S42 Phase C, dernier feat commit).
**Documents source** : `sprint42_kickoff.md` (D1..D5) +
`sprint42_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint42_verification.md` (28/28 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint42_plan.md,
(3) sprint42_kickoff.md §D1..D5. Ne PAS lire le code source
avant d'avoir parcoure les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint42_audit_findings.md`.

## Track A — Dette pair (Phase A)

- [ ] A-1 : rand::thread_rng() dans canary_input.rs — verifier que
  les tests existants (injector_rate_always, guardrail_tripwire_on_inject)
  exercent le caller. Comparer l'ancienne logique hash-based supprimee.
- [ ] A-2 : rand::thread_rng() dans upload_queue.rs — verifier que
  test jitter_in_range exerce le caller. Distribution non-degeneree.
- [ ] A-3 : GuardrailOutcome::Mutation variant — verifier que le match
  arm dans run() est exhaustif et que chain_mutation_collects_and_passes
  teste le path.
- [ ] A-4 : warn threshold doc PATTERNS.md — verifier presence §P41
  et coherence avec code existant (WARN_THRESHOLD_DAYS=30 / ALARM=45).
- [ ] A-5 (P2-REVIEW-A-1-S42) : ChainResult::mutations est
  Vec<(String, String)> sans semantique target. Verifier que le variant
  Mutation n'est emis par aucun guardrail (scope cut S42 respecte).
  Informational — carry S43+ pour documentation.

## Track B — Deploy API (Phase B)

- [ ] B-1 : forge.rs detection — verifier couverture des 5 ForgeType
  variants (GitHub/GitLab/Codeberg/Gitea/Unknown) par les tests.
- [ ] B-2 : provenance.rs — verifier roundtrip generate+verify, rejet
  wrong key, rejet tampered hash. BLAKE3 deterministic.
- [ ] B-3 : deploy.rs validations — SHA hex validation, zip validation,
  zip creation+append, dir_size, SBFB.json parse+missing. 9 tests.
- [ ] B-4 (P2-REVIEW-B-1-S42) : pow_keypair utilise pour signer
  provenance au lieu d'un keypair dedie. Verifier que c'est le meme
  pattern que le Python (coord.keypair). Carry S43+ documentation.
- [ ] B-5 : deploy handler dans http.rs — verifier que les 2 routes
  POST /api/v1/deploy et POST /api/v1/deploy/private sont enregistrees.

## Track C — Apps API (Phase C)

- [ ] C-1 : apps.rs to_summary/to_detail — verifier que les champs
  JSON correspondent au format attendu (BrowseEntry fields).
- [ ] C-2 : BrowseStatus + BrowseSource enum coverage — verifier que
  status_str et source_str couvrent tous les variants.
- [ ] C-3 : query defaults + filters — verifier que les 8 tests
  couvrent category/open_source/limit/offset filtres.
- [ ] C-4 (P3-REVIEW-C-1-S42) : list_apps fait aggregate() avec
  probe reseau a chaque appel. Verifier que le TTL cache browse
  aggregator amortit. Informational post-v1.0.

## Track D — Process / meta

- [ ] D-1 : G8 preflights Phase A + B + C — verifier coherence
  (3/3 EXECUTE, 0 DESIGN-CONFLICT).
- [ ] D-2 : scope cuts 8/8 — verifier aucun viole (diff --stat).
- [ ] D-3 : 4/4 dette items resolus — verifier dans le diff que
  rand_range, pseudo_random, Mutation, warn threshold sont tous
  traites.

## Track E — Overdue items (P1 signal)

- [ ] E-1 : 5 items P3 a 4/3 (LOC kickoff, persist error,
  URL single-quote, Manager Mutex, rerun hash) — ces items etaient
  a 3/3 MANDATORY a l'entree de S42 et n'ont pas ete resolus.
  Verifier que le kickoff les marquait "si temps" et que Phase A
  n'avait pas de budget.
  Signal **P2** si l'audit estime qu'au moins un etait trivial
  et faisable dans le budget Phase A.
  Signal **P1** si un item < 50 LOC a ete ignore sans raison.
- [ ] E-2 : P2-REVIEW-A-1-S41 conn() pub 3/3 — atteint MANDATORY.
  Verifier si le scope S42 pouvait le traiter (Phase A dette = 4
  items cibles, conn() pas dans la liste D1).
- [ ] E-3 : P3-REVIEW-B-1-S41 MintRequest 3/3 — idem.

## Track F — Doc coherence

- [ ] F-1 : HARDENING_ROADMAP compteurs — verifier 1089 Rust / ~2092 total
- [ ] F-2 : CLAUDE.md etat actuel — verifier S42 CLOSED + carries S43
- [ ] F-3 : SPRINT_LOG.md — verifier row S42 presente
- [ ] F-4 : Phase review files present : 3/3 (A + B + C)
- [ ] F-5 : Phase preflight files present : 3/3 (A + B + C)

---

## Carries S43

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 7+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 | 4/3 | exemption dep S45 |
| P2-REVIEW-A-1-S41 conn() pub encapsulation | 3/3 | **MANDATORY** |
| P2-REVIEW-A-1-S42 ChainResult mutations target | 1/3 | Phase A review |
| P2-REVIEW-B-1-S42 pow_keypair identity doc | 1/3 | Phase B review |
| P3-REVIEW-A-2-S39 LOC kickoff | 4/3 | **OVERDUE** |
| P3-REVIEW-B-2-S39 persist error silent | 4/3 | **OVERDUE** |
| P3-AUDIT-A-1-S39 URL single-quote | 4/3 | **OVERDUE** |
| P3-REVIEW-B-1-S40 Manager multiple Mutex | 4/3 | **OVERDUE** |
| P3-REVIEW-C-1-S40 rerun deterministic hash | 4/3 | **OVERDUE** |
| P3-REVIEW-B-1-S41 MintRequest ergonomie | 3/3 | **MANDATORY** |
| P3-REVIEW-A-2-S42 babel-scraper untracked | 1/3 | Phase A review |
| P3-REVIEW-C-1-S42 list_apps aggregate probe | 1/3 | Phase C review |

**Resolus S42** : P2-REVIEW-A-1-S39 Tripwire (Phase A),
P2-REVIEW-B-1-S39 warn threshold (Phase A),
P2-REVIEW-B-1-S40 rand_range (Phase A),
P2-REVIEW-C-1-S41 pseudo_random (Phase A).

**Note S43 impair** : S43 est impair → pas de phase dette obligatoire
(§6.2.1 Regle 1). Les items OVERDUE et MANDATORY doivent quand meme
etre traites dans le scope du sprint.

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S43 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint42): ... avant S43 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D5 gelees du kickoff (ne pas rebattre)
- SHA-256 vs BLAKE3 exemption (dependance S45 documentee)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S42 (decision sprint, pas audit)

## Livrable attendu

`sprint42_audit_findings.md` avec : verdict global, section par
track, findings P0→P3, commits fix attendus si CONDITIONAL PASS.
