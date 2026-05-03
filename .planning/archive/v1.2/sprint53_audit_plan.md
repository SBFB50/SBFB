# Sprint 53 — Audit plan (Sprint 52 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S52).
**Tip d'entree** : `374bf59` (S52 Phase B, dernier feat commit).
**Documents source** : `sprint52_kickoff.md` (D1..D4) +
`sprint52_plan.md` (§Phase A, §Phase B) +
`sprint52_verification.md` (23/23 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint52_plan.md,
(3) sprint52_kickoff.md §D1..D4. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint52_audit_findings.md`.

## Track A — Dispatch shutdown fix (Phase A)

- [ ] A-1 : verifier que `dispatch_shutdown` field existe dans
  `DaemonRuntime` struct (runtime.rs).
- [ ] A-2 : verifier que `dispatch_loop::run()` prend un
  `oneshot::Receiver<()>` et utilise `tokio::select!`.
- [ ] A-3 : verifier que `shutdown()` envoie le signal AVANT
  de join le dispatch_handle.
- [ ] A-4 : verifier que le test `dispatch_loop_writes_to_doc`
  passe le shutdown receiver.

## Track B — Docs legacy deletion (Phase A)

- [ ] B-1 : verifier que `git ls-files docs/BENCHMARK.md
  docs/ARCHITECTURE.md docs/DATABASE_SCHEMA.md ...` retourne 0.
- [ ] B-2 : verifier que VISION_USE_CASES.md est dans .gitignore
  (21e fichier non tracke).
- [ ] B-3 : verifier que 0 reference aux 20 fichiers supprimes
  existe dans crates/ web/ .github/.

## Track C — CLAUDE.md coherence (Phase A + C)

- [ ] C-1 : verifier que la ligne stale
  `P2-REVIEW-A-1-S51 release-attest.sh dead code` est absente.
- [ ] C-2 : verifier que carries S53 = 3 items (rand, iroh
  transitives, unsafe set_var 2/3) + LT items.
- [ ] C-3 : verifier que "Sprints 0-52 CLOSED" est present.

## Track D — CI Woodpecker (Phase B)

- [ ] D-1 : verifier que `.woodpecker/ci-linux.yml` existe et
  reproduit le bloc CI Linux (Rust fmt/clippy/test + Frontend
  tsc/lint/vitest/build/size).
- [ ] D-2 : verifier la syntaxe YAML Woodpecker (steps list,
  image, commands).
- [ ] D-3 : verifier que le pipeline ne contient PAS de release
  matrix ni cosign (scope strict CI quotidienne).

## Track E — Self-hosted build design (Phase B)

- [ ] E-1 : verifier que `docs/architecture/SELF_HOSTED_BUILD.md`
  existe avec strategie 3 etages.
- [ ] E-2 : verifier que le doc dit explicitement que task_type
  "build" n'est PAS une extension triviale du worker LLM.
- [ ] E-3 : verifier que LT-7 existe dans ROADMAP_COMMITMENTS
  avec status "pre-v1.0 obligatoire".

## Track F — GHA release.yml matrix fix (Phase B)

- [ ] F-1 : verifier que release.yml utilise `os: [ubuntu-latest,
  macos-latest, windows-latest]` (array, pas include-only).
- [ ] F-2 : verifier que `binary: [nexus-worker, nexus-shell-daemon,
  nexus-launcher]` est separe de include.
- [ ] F-3 : verifier que include ajoute os-label + shell par OS.

## Track G — Process / meta

- [ ] G-1 : G8 preflights 2/2 presents (A + B).
- [ ] G-2 : Phase reviews 2/2 presents (A + B).
- [ ] G-3 : Scope cuts 8/8 respectes.
- [ ] G-4 : Delta tests cumule coherent : 0.
- [ ] G-5 : Sprint pair, phase dette Phase A (confirme).
- [ ] G-6 : 3 carries CLOSED Phase A.
- [ ] G-7 : HARDENING_ROADMAP last_validated S52.
- [ ] G-8 : Phase B pivot documente dans preflight (pas silencieux).

---

## Carries S53

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 2/3 | S51 review |
| P2-REVIEW-A-1-S52 nextest timeout profiling | 1/3 | NEW S52 |
| P2-REVIEW-B-1-S52 Woodpecker E2E validation | 1/3 | NEW S52 |
| P2-REVIEW-B-2-S52 GHA 9/9 re-run confirm | 1/3 | NEW S52 |

**Note S53 impair** : pas de phase dette obligatoire.
P2-REVIEW-B-1-S51 unsafe set_var a 2/3 — si non adresse S53,
3/3 MANDATORY S54.

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S53 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint52): ... avant S53 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D4 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S52 (decision sprint, pas audit)
- Phase B pivot (decision utilisateur documentee, pas audit)
