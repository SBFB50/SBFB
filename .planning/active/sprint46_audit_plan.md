# Sprint 46 — Audit plan (Sprint 45 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S45).
**Tip d'entree** : `e1c31a5` (S45 Phase B, dernier feat commit).
**Documents source** : `sprint45_kickoff.md` (D1..D4) +
`sprint45_plan.md` (§Phase A, §Phase B) +
`sprint45_verification.md` (28/28 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint45_plan.md,
(3) sprint45_kickoff.md §D1..D4. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint45_audit_findings.md`.

## Track A — Route portage (Phase A)

- [ ] A-1 : invite_api.rs — verifier 3 routes (create, list,
  revoke). Schema invite : scope, expiry_secs, max_uses, note.
  Validation scope worker/observer. Expiry >= 60s.
- [ ] A-2 : quarantine_api.rs — verifier 3 routes (list, flush,
  drop). Status filter (pending/all). TTL 900s.
- [ ] A-3 : http.rs — verifier 6 routes enregistrees avec mod
  declarations dans main.rs.
- [ ] A-4 : invite ID generation — AtomicU64 counter + epoch.
  Collision possible multi-daemon. Verifier documentation carry.

## Track B — Carries resolus (Phase A)

- [ ] B-1 : SHA-256→BLAKE3 — verifier redundancy.rs utilise
  blake3::hash(), pas sha2. Verifier tests existants passent.
- [ ] B-2 : worker_state tokio::fs — verifier tokio::fs::
  read_to_string() dans worker_state_api.rs (pas std::fs).
- [ ] B-3 : list_tasks status validation — verifier VALID_STATES
  dans tasks_api.rs. Verifier 400 retourne sur state invalide.
- [ ] B-4 : TOCTOU canary reload — verifier canary_input.rs :
  mtime set sous lock AVANT lecture fichier.
- [ ] B-5 : silent null diagnostic — verifier diagnostic_api.rs :
  worker_contributions() erreur → 500 (pas vec![]).
- [ ] B-6 : hex case-sensitivity — verifier contributor_api.rs :
  to_ascii_lowercase() sur project_id et node_id_hex.

## Track C — Coordinator Python gut (Phase B)

- [ ] C-1 : 14 fichiers routes Python supprimes — verifier
  aucun n'existe dans api/.
- [ ] C-2 : app.py — verifier include_router ne monte QUE
  events + daemon (pas les routes supprimees).
- [ ] C-3 : 12 fichiers tests Python supprimes — verifier
  coherence.
- [ ] C-4 : 4 fichiers tests Python modifies — verifier
  que les tests restants passent (pas d'import casse).
- [ ] C-5 : modules Python non supprimes — verifier que
  coordinator.py boot encore (importe dispatcher, validator, etc.).
- [ ] C-6 : coord pytest count — verifier 323+23f+6s coherent
  (pas de regression cachee).

## Track D — Dead code Rust (Phase B)

- [ ] D-1 : coord_http_client supprime de DaemonHttpState — verifier
  aucune reference dans http.rs, runtime.rs, tests.
- [ ] D-2 : coord_base_url supprime — verifier.
- [ ] D-3 : resolve_coord_base_url() supprime — verifier.
- [ ] D-4 : COORD_BASE_URL_ENV + DEFAULT_COORD_BASE_URL supprimes.
- [ ] D-5 : test resolve_coord_base_url_respects_env_var supprime.
- [ ] D-6 : reqwest reste dep — verifier consumer (deploy.rs).

## Track E — Process / meta

- [ ] E-1 : G8 preflights 2/2 — verifier coherence
  (A + B tous EXECUTE, 0 DESIGN-CONFLICT).
- [ ] E-2 : scope cuts 8/8 — verifier aucun viole (diff --stat).
- [ ] E-3 : 7 carries resolus — verifier dans le diff.
- [ ] E-4 : G1 design review present — verifier scoring.
- [ ] E-5 : sprint impair = pas de phase dette obligatoire.

## Track F — Doc coherence

- [ ] F-1 : CLAUDE.md etat actuel — verifier S45 + compteurs.
- [ ] F-2 : SPRINT_LOG.md row S45 — verifier presente.
- [ ] F-3 : HARDENING_ROADMAP.md — verifier last_validated +
  compteurs.
- [ ] F-4 : Phase review files 2/2 (A + B presents).
- [ ] F-5 : Phase preflight files 2/2 (A + B presents).

---

## Carries S46

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 10+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-AUDIT-A-1-S43 integration test gap 12 routes | 3/3 | **MANDATORY S46** |
| P2-REVIEW-A-1-S44 as_str/serde coupling | 2/3 | |
| P2-REVIEW-B-1-S44 kudos entries pagination | 2/3 | |
| P2-REVIEW-A-1-S45 diagnostic Err path non teste | 1/3 | NEW |
| P2-REVIEW-A-2-S45 invite ID collision multi-daemon | 1/3 | NEW |
| P2-REVIEW-B-1-S45 modules Python suppression differee | 1/3 | NEW |
| P3-REVIEW-B-2-S44 shell discover self-only | 2/3 | |
| P3-AUDIT-A-1-S44 test pagination handler-level | 2/3 | |
| P3-AUDIT-B-1-S44 diagnostic silent fallback | 2/3 | |

**Resolus S45** : P2-REVIEW-C-1-S40 SHA-256→BLAKE3 (6/3),
P2-REVIEW-B-1-S43 coord dead_code (2/3),
P2-REVIEW-C-1-S44 worker_state tokio::fs (1/3),
P3-REVIEW-C-2-S44 list_tasks status invalide (1/3),
P3-REVIEW-A-1-S43 TOCTOU canary reload (2/3),
P3-AUDIT-A-2-S43 silent null canary_api (2/3),
P3-AUDIT-A-3-S43 hex case-sensitivity (2/3).

**Note S46 pair** : S46 est pair → phase dette obligatoire
(§6.2.1 Regle 1). P2-AUDIT-A-1-S43 integration test gap
atteint 3/3 = **MANDATORY** (§6.2.1 Regle 2).

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S46 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint45): ... avant S46 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D4 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S45 (decision sprint, pas audit)
- Modules Python non supprimes (scope cut documente)

## Livrable attendu

`sprint45_audit_findings.md` avec : verdict global, section par
track, findings P0→P3, commits fix attendus si CONDITIONAL PASS.
