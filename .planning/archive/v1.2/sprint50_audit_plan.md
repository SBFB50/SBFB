# Sprint 50 — Audit plan (Sprint 49 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S49).
**Tip d'entree** : `0cbfaab` (S49 Phase B, dernier feat commit).
**Documents source** : `sprint49_kickoff.md` (D1..D4) +
`sprint49_plan.md` (§Phase A, §Phase B) +
`sprint49_verification.md` (28/28 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint49_plan.md,
(3) sprint49_kickoff.md §D1..D4. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint49_audit_findings.md`.

## Track A — Project doc lifecycle (Phase A)

- [ ] A-1 : verifier que `runtime.rs` cree ou reopen un doc
  iroh-docs au demarrage (DocsClient::list_docs + open_doc/
  create_doc).
- [ ] A-2 : verifier que `DaemonHttpState` a `project_doc:
  Option<Arc<DocHandle>>` et `task_dispatch_tx:
  Option<TaskEntrySender>`.
- [ ] A-3 : verifier que le dispatch loop est spawned (tokio::spawn)
  avec le project_doc + author.

## Track B — Dispatch loop MPSC (Phase A)

- [ ] B-1 : verifier `dispatch_loop.rs` drain le channel MPSC
  et ecrit via `doc.set(author, key, value)`.
- [ ] B-2 : verifier que `coordinator_submit_task` dans http.rs
  envoie l'entry au channel via `try_send` apres le persist DB.
- [ ] B-3 : verifier le test `dispatch_loop_writes_to_doc` passe
  et verifie la presence de l'entry dans le doc.

## Track C — CLI subcommands (Phase B)

- [ ] C-1 : verifier `cli.rs` a les 4 subcommands (Init,
  Invite(InviteCommand), Quarantine(QuarantineCommand),
  Capability(CapabilityCommand)).
- [ ] C-2 : verifier `main.rs` matche les 4 subcommands dans le
  match et route vers les handlers.
- [ ] C-3 : verifier les handlers operent en mode offline (open
  CoordinatorDb directement, pas de daemon running requis — G1
  D3 ack).
- [ ] C-4 : verifier les 8 parsing tests couvrent les variantes.

## Track D — Process / meta

- [ ] D-1 : G8 preflights 2/2 presents (A + B tous EXECUTE).
- [ ] D-2 : scope cuts 12/12 respectes (diff --stat).
- [ ] D-3 : Phase reviews 2/2 presents (A + B).
- [ ] D-4 : Delta tests cumule coherent : Rust +9.
- [ ] D-5 : Sprint impair — pas de phase dette (confirme).

## Track E — Doc coherence

- [ ] E-1 : CLAUDE.md etat actuel — verifier S49 + compteurs.
- [ ] E-2 : SPRINT_LOG.md row S49 — verifier presente.
- [ ] E-3 : memory nexus_grid_pivot.md — verifier tip mis a jour.

---

## Carries S50

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S48 canary reload size cap | 1/3 | S48 review |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 1/3 | S48 review |
| P2-AUDIT-A-1-S48 carry doc accuracy | 1/3 | S48 audit |
| P2-REVIEW-A-1-S49 dispatch loop JoinHandle | 1/3 | NEW |
| P2-REVIEW-B-1-S49 CLI handler integration tests | 1/3 | NEW |

**Note S50 pair** : S50 est pair → phase dette obligatoire
(§6.2.1 Regle 1). 0 item a 2/3. 0 item a 3/3.

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S50 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint49): ... avant S50 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D4 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S49 (decision sprint, pas audit)
- app-gov conversion (defer S50, scope cut documente)
