# Sprint 49 — Audit findings

**Auditeur** : session fraiche (pas la session qui a code S49).
**Tip d'entree** : `c72cf93` (HEAD, Phase C wrap-up).
**Documents source** : sprint50_audit_plan.md, sprint49_kickoff.md,
sprint49_plan.md, sprint49_verification.md.
**Date** : 2026-05-01.

---

## Verdict : **PASS** (0 P0, 0 P1, 1 P2, 2 P3)

G4 rigor signal satisfait (>=1 P2+ documente).
Sprint 50 Phase A demarre directement.

---

## Track A — Project doc lifecycle (Phase A)

- [x] **A-1** : `runtime.rs` cree ou reopen un doc iroh-docs au
  demarrage (L482-513). Utilise `DocsClient::new(node.docs())`,
  `list_docs()`, `open_doc(first_id)` / `create_doc()`. Le pattern
  create-if-not-exists est correct. ✅

- [x] **A-2** : `DaemonHttpState` a `project_doc:
  Option<Arc<DocHandle>>` (http.rs:152) et `task_dispatch_tx:
  Option<TaskEntrySender>` (http.rs:156). `project_doc` est marque
  `#[allow(dead_code)]` car lu via l'Arc clone passe au dispatch
  loop, pas via le HTTP state — acceptable (le champ sera lu par
  de futurs endpoints). ✅

- [x] **A-3** : le dispatch loop est spawned via `tokio::spawn(
  crate::dispatch_loop::run(...))` a runtime.rs:522-527. Le
  JoinHandle est dropped (carry P2-REVIEW-A-1-S49 1/3 documente).
  ✅

## Track B — Dispatch loop MPSC (Phase A)

- [x] **B-1** : `dispatch_loop.rs` drain le channel MPSC
  (`while let Some(entry) = rx.recv().await`, L25) et ecrit via
  `doc.set(author, key.as_bytes().to_vec(), value)` (L34). Le sole
  writer pattern (G1 D2 ack) est respecte. ✅

- [x] **B-2** : `coordinator_submit_task` dans http.rs envoie
  l'entry au channel via `try_send` (http.rs:1348-1351) APRES le
  persist DB (`submit_task(&db, &keypair, submission)` L1346). Le
  sequencement DB-first → channel est correct pour la durabilite.
  ✅

- [x] **B-3** : test `dispatch_loop_writes_to_doc` present
  (dispatch_loop.rs:71-102). Le test boot un node iroh reel, cree
  un doc, envoie une entry via le channel, et verifie la presence
  dans le doc via `get_many_by_prefix`. Couvre la boucle E2E
  channel → doc write → read back. ✅

## Track C — CLI subcommands (Phase B)

- [x] **C-1** : `cli.rs` a les 4 subcommands : `Init` (L122),
  `Invite(InviteCommand)` (L128), `Quarantine(QuarantineCommand)`
  (L135), `Capability(CapabilityCommand)` (L140). Les 3 enums
  derives (InviteCommand, QuarantineCommand, CapabilityCommand)
  sont correctement definis. ✅

- [x] **C-2** : `main.rs` matche les 4 subcommands dans le match
  (L101-104) et route vers `handle_init`, `handle_invite`,
  `handle_quarantine`, `handle_capability`. ✅

- [x] **C-3** : les handlers operent en mode offline (G1 D3 ack) :
  `handle_init` ouvre `CoordinatorDb::open` directement (main.rs:580),
  `handle_invite` fait de meme (main.rs:592) via `InviteLedger::new`,
  `handle_quarantine` (main.rs:645) via `QuarantineQueue::new`,
  `handle_capability` (main.rs:685) via `CapabilityStore::load`.
  Aucun daemon running requis. ✅

- [x] **C-4** : 8 parsing tests couvrent les variantes :
  `parses_init` (L507), `parses_invite_create` (L512),
  `parses_invite_list` (L519), `parses_invite_revoke` (L531),
  `parses_quarantine_list` (L544), `parses_quarantine_flush` (L553),
  `parses_capability_enable` (L565), `parses_capability_list` (L580).
  ✅

## Track D — Process / meta

- [x] **D-1** : G8 preflights 2/2 presents :
  sprint49_phase_A_preflight.md (EXECUTE) et
  sprint49_phase_B_preflight.md (EXECUTE). ✅

- [x] **D-2** : scope cuts 12/12 respectes. `diff --stat` confirme
  5 fichiers modifies, tous dans `crates/nexus-shell-daemon/` (+497
  insertions, -17 deletions). Aucun fichier app-gov, events.py,
  MCP, PyO3, coordinator Python, SDK Python, CI/CD, ou frontend
  touche. ✅

- [x] **D-3** : Phase reviews 2/2 presents : Phase A PASS (1 P2,
  1 P3), Phase B PASS (1 P2, 1 P3). ✅

- [x] **D-4** : delta tests cumule +9 Rust (Phase A +1
  dispatch_loop_writes_to_doc, Phase B +8 CLI parsing). Kickoff
  entry 1186 → sortie 1195 = +9 confirme. ✅

- [x] **D-5** : sprint impair — pas de phase dette. Confirme dans
  kickoff §Type et §6. ✅

## Track E — Doc coherence

- [x] **E-1** : CLAUDE.md etat actuel — "Sprints 0-49 CLOSED" ✅,
  "~1947 tests total (1195 Rust ...)" ✅, compteurs coherents avec
  verification.md. ✅

- [x] **E-2** : SPRINT_LOG.md row S49 presente avec details
  complets (theme, tip, delta tests, carries, fichiers). ✅

- [ ] **E-3** : memory nexus_grid_pivot.md tip = `0cbfaab` (Phase B)
  mais HEAD = `c72cf93` (Phase C). La session wrap-up n'a pas mis
  a jour le tip memory apres le commit Phase C. ⚠️ → P2-AUDIT-A-1-S49

---

## Findings

### P2

**P2-AUDIT-A-1-S49 — Memory tip stale post-Phase C** (1/3)

Le fichier `nexus_grid_pivot.md` dans la memory Claude a son tip a
`0cbfaab` (Phase B) au lieu de `c72cf93` (Phase C wrap-up). La
session qui a produit le commit Phase C n'a pas mis a jour la memory
conformement a la regle feedback_memory_update.md ("toujours update
apres chaque feat phase, AVANT de rendre la main"). Phase C est un
`chore` (pas `feat`), ce qui peut expliquer l'omission — mais le
tip devrait refleter le HEAD final du sprint quand la session se
termine.

**Action recommandee** : corriger le tip dans nexus_grid_pivot.md
au prochain acces memory (cette session ou S50 kickoff).

### P3

**P3-AUDIT-A-2-S49 — Verification table entry count off by 1**

`sprint49_verification.md` §2 indique "Entree S49 : 1187 Rust
nextest" mais le kickoff §1.3 (tip `ebf14e7`) mesure 1186. Le delta
"+9" est correct quand calcule depuis l'entree kickoff (1186→1195)
mais le nombre d'entree dans la table de verification est inexact
par 1. Probablement un copier du count Phase A (1187 = post-Phase A)
utilise comme entree sprint.

**P3-AUDIT-A-3-S49 — Plan/code serialization dans dispatch_loop.rs**

Le plan §A.2 specifie `entry.canonical_bytes()` pour la
serialisation dans le doc, mais le code utilise
`serde_json::to_vec(&entry)` (dispatch_loop.rs:27). Les deux
approches sont acceptables pour le stockage doc (canonical bytes
sert a la signature, pas au stockage). La TaskEntry est deja signee
avant d'atteindre le dispatch loop. Deviation du pseudo-code plan,
pas un bug fonctionnel.

---

## Carries S50 (confirmes)

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S48 canary reload size cap | 1/3 | S48 review |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 1/3 | S48 review |
| P2-AUDIT-A-1-S48 carry doc accuracy | 1/3 | S48 audit |
| P2-REVIEW-A-1-S49 dispatch loop JoinHandle | 1/3 | S49 Phase A review |
| P2-REVIEW-B-1-S49 CLI handler integration tests | 1/3 | S49 Phase B review |
| P2-AUDIT-A-1-S49 memory tip stale post-Phase C | 1/3 | NEW S49 audit |

8 carries S50 (dont 1 NEW audit S49).
0 item a 2/3. 0 item a 3/3.
S50 pair → phase dette obligatoire (§6.2.1 Regle 1).

---

## Out of scope (confirme)

- D1..D4 gelees du kickoff (non rebattues)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S49 (decisions sprint)
- app-gov conversion (S50)
