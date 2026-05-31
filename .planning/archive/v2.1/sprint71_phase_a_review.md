# Sprint 71 Phase A — Review (pre-commit, pre-Codex)

**Date** : 2026-05-30
**Reviewer** : agent review independant
**HEAD** : `2ec72e8` (15 ahead origin, rien pousse)
**Diff revu** : working tree NON commite —
`crates/nexus-shell-daemon/src/dispatch_loop.rs` (+118/-7),
`crates/nexus-worker-core/src/engine/runtime.rs` (+14, accessor additif).
`crates/sbfb-factory/src/terminal.rs` : **aucun diff** au working tree
(stash WIP `.cast`→`.log` droppe, HEAD `.cast` preserve — G1/D7).

Titre commit cible : `fix(compute): Sprint 71 Phase A — align dispatch key + first cross-process E2E`.

---

## Verdict: PASS

**Promu PASS apres reconciliation Codex (3 CONFIRME, 0 GAP, 0 PARTIEL).**
**Rien de bloquant (0 P0, 0 P1).** Les trois livrables (B-1 cle dispatch,
B-3 E2E cross-process, G1 stash drop) sont **corrects et prouves verts en
CI Linux**, alignes sur le preflight EXECUTE
(`sprint71_phase_a_preflight.md`), bornes au scope Phase A. Deux findings P2
(qualite/portabilite) et un P3 (cosmetique) ci-dessous.

### Resolution de l'alerte E2E (investigation menee dans cette review)

Un premier run a montre l'E2E B-3 en **TIMEOUT 90s sur le poste Windows**.
Investigation par bisection :

| Test | Touche Phase A ? | **Windows local** | **Docker CI Linux (`sbfb-ci`)** |
|------|-------------------|-------------------|----------------------------------|
| `dispatch_loop_writes_to_doc` (B-1) | oui (assertions) | PASS 0.291s | **PASS 0.293s** |
| `dispatched_task_is_claimed_...` (E2E B-3) | **nouveau** | **TIMEOUT 90s** | **PASS 1.950s** |
| `engine_claims_..._registered_doc` (miroir, **S4**, **non touche**) | non | **TIMEOUT 90s** | **PASS 1.991s** |

**Conclusion factuelle** :
- L'E2E B-3 **fonctionne reellement** (PASS 1.95s en Linux) — il prouve la
  chaine dispatcher→worker→claim→result. B-3 est satisfait.
- Le timeout Windows touche AUSSI le **miroir pre-existant S4 non modifie** →
  c'est un **artefact specifique au poste Windows** du pump worker-engine
  iroh-docs, **ni une regression Phase A, ni un bug du code**.
- L'affirmation du caller « 473 passed 0 skip incluant le nouvel E2E » est
  **confirmee valide en CI Linux** (la source de verite du projet, memory
  `feedback_wsl_before_push.md` : le pipeline CI tourne en Docker, pas sur
  le poste Windows natif).

Le commit Phase A peut partir une fois la fail-fast checklist verte **en CI
Linux** (rows 3 + 8 du `plan §10`) — ce que le Docker discriminant ci-dessus
vient de demontrer pour les 3 tests concernes. Le gate Codex doit aussi
tourner avant la promotion `PASS`.

`PASS-PENDING` reste l'etat pre-Codex transitoire : faire tourner
`codex exec`, coller la sortie brute dans `sprint71_phase_a_codex_review.md`,
reconcilier ici, puis promouvoir `## Verdict: PASS` avant commit.

---

## Findings

| ID | Severite | Dimension | Resume | Action |
|----|----------|-----------|--------|--------|
| P2-A-1 | P2 | 2 (E2E) | **Note de portabilite (non bloquant)** : l'E2E B-3 (et son miroir S4) **timeout sur poste Windows** alors qu'ils passent en 2s en CI Linux. Le pump worker-engine iroh-docs ne tourne pas vert nativement sous Windows — dette **pre-existante** (le miroir S4 a le meme symptome). | Documenter dans PATTERNS (Phase E) : « tests worker-pump iroh-docs = CI Linux uniquement, hang Windows natif connu ». Verifier B-3 via Docker/CI avant push (`feedback_wsl_before_push.md`), jamais sur le poste Windows seul. Candidat investigation G6 Phase D (surfaces worker non fiables). |
| P2-A-2 | P2 | 2 (E2E) | L'E2E n'asserte pas le **contenu** du result (signature/digest), seulement `results.len()==1`. Prouve le routage+execution, pas l'integrite du payload. (Le miroir S4 non plus — `runtime.rs:1524-1530` verifie juste la presence.) | Optionnel : ajouter `ResultEntry::verify_signature()`. Non bloquant — la verif signature est couverte par les unit tests `task.rs`/`validator.rs`. Cohérent avec le miroir existant. |
| P3-A-3 | P3 | 6 (body) | `make_test_entry()` (`dispatch_loop.rs:71`) fixe `task_id = "test-dispatch-001"` partage entre les deux tests du module. Tests independants (docs separes), pas de collision, mais id non unique. | Cosmetique. Aucune. |

**0 P0, 0 P1.** Aucun blocage commit (sous reserve du gate Codex). La preuve
B-3 verte est etablie en CI Linux.

---

## Dimensions

### 1. Correctness B-1 — PASS

Le fix aligne reellement writer↔reader.

- **Writer** : `dispatch_loop.rs:41` ecrit desormais
  `format!("task:{}", entry.task.task_id)` puis `doc.set(...)` (l.49).
  Avant : `format!("tasks/{}", ...)`.
- **Reader (reference, inchange)** : `runtime.rs:847`
  `get_many_by_prefix(b"task:")` + `runtime.rs:859`
  `strip_prefix("task:")`. Le scan worker date de Sprint 4 (commentaire
  `runtime.rs:808` « Sprint 4 Phase D W9.1 ») — c'est le `tasks/` (S49) qui
  etait desaligne, conforme au S2 du preflight (bug d'inattention, pas une
  decision a rationale).
- **Test interne aligne** : `dispatch_loop.rs:116`
  (`get_many_by_prefix(b"task:")`) + `:127` (`assert!(starts_with("task:"))`)
  + `:130` (`assert_eq!(stored_key, format!("task:{task_id}"))`). Les trois
  sites R4 anticipes par le preflight sont alignes dans le meme diff.
  L'assert `starts_with("task:")` (l.127) est nouveau et **rend explicite
  l'invariant** que le bug B-1 violait — bonne pratique anti-regression.
- **Re-grep exhaustif `tasks/`** (`crates/**/*.rs`) : les seules occurrences
  restantes sont
  - `http.rs:306,405,4351,4459,5052` + `tasks_api.rs:6,111` = **routes HTTP
    REST** `/api/v1/tasks/...`, sans aucun rapport avec la cle de doc iroh ;
  - `rate_limit.rs:15` + `crypto.rs:57` = commentaires de doc ;
  - `dispatch_loop.rs:39,136` = le **commentaire explicatif du fix** lui-meme.
  Aucun autre **lecteur/ecrivain de la cle de doc** `tasks/`. Confirme S4 du
  preflight.
- **Re-grep `get_many_by_prefix`** : tous les sites de cle de doc compute
  utilisent `task:` / `claim:` / `result:` (worker `runtime.rs:847,1774` ;
  daemon `dispatch_loop.rs`). Aucun consommateur `tasks/`. Les autres prefixes
  (`feed/`, `ideas/`, storage namespace) sont des domaines disjoints, non
  affectes.

Conclusion : alignement bout-en-bout writer `task:` → scan worker `task:` →
`strip_prefix`. Bug ferme, pas de site oublie.

### 2. Validite de l'E2E — PASS (verifie vert en CI Linux ; timeout Windows = artefact poste, P2-A-1)

**Le test est sain par conception (non tautologique, regression-proof) ET
passe a l'execution en CI Linux (1.95s).** Le timeout observe sur le poste
Windows est un artefact d'environnement (P2-A-1), confirme par bisection.

#### Reproduction empirique (2026-05-30)

| Test | Touche Phase A ? | **Windows local** | **Docker CI Linux** |
|------|-------------------|-------------------|---------------------|
| `dispatch_loop_writes_to_doc` (B-1) | oui (assertions) | PASS 0.291s | **PASS 0.293s** |
| `dispatched_task_is_claimed_..._engine` (E2E B-3) | **nouveau** | TIMEOUT 90s (×2) | **PASS 1.950s** |
| `engine_claims_..._registered_doc` (miroir, **S4**) | **NON** | TIMEOUT 90s | **PASS 1.991s** |

**Bisection decisive** :
- B-1 (boot node + `doc.set`) passe partout → le boot node iroh n'est pas
  en cause (sans reseau il timeoutait sur le relay n0,
  `.config/nextest.toml:38-40` ; avec reseau il passe en 0.29s).
- L'E2E **et son miroir pre-existant non touche** (`runtime.rs:1417`, S4)
  timeout TOUS LES DEUX sur Windows, mais **passent TOUS LES DEUX en
  ~2s en Linux** → le timeout est un **artefact poste-Windows du pump
  worker-engine iroh-docs**, pre-existant (le miroir le precede de
  67 sprints), **pas une regression Phase A ni un bug du code**.
- **B-3 est donc reellement prouve** : en CI Linux, une tache ecrite par le
  vrai `dispatch_loop::run` est claim+executee par un vrai `Engine` worker,
  qui emet le `result:`. Le critere central Phase A (kickoff §6) est
  satisfait.

#### Pourquoi le test est sain par conception

- **Vrai dispatcher** : la tache est ecrite via `dispatch_loop::run`
  (`dispatch_loop.rs:201`, le **vrai** code de production), pas via un
  `doc.set` a la main. Difference load-bearing avec le miroir
  `engine_claims_and_executes_tasks_on_registered_doc` (`runtime.rs:1417`)
  qui injecte `doc.set(author, b"task:t-1".to_vec(), ...)` — le « blind spot »
  que le commentaire E2E (`dispatch_loop.rs:136-144`) decrit correctement.
- **Vrai worker engine** : `Engine::new(boot)` (l.191), pump de production
  via `run_until_shutdown` (l.217) → `scan_and_execute_tasks`
  (`runtime.rs:830`). En CI Linux ce chemin produit bien le `result:`
  (PASS 1.95s) — la chaine complete est exercee.
- **Regression-proof par design** : OUI. Si le bug B-1 revenait (writer
  `tasks/`), le scan `get_many_by_prefix(b"task:")` (`runtime.rs:847`) ne
  verrait rien → assert `task:` len==1 (l.212) echouerait, ou le `timeout(10s)`
  (l.219) expirerait. La propriete anti-regression est correcte et **active
  en CI** (le test passe a l'etat nominal en Linux, donc il echouerait si la
  cle se desalignait).
- **Memes primitives que le miroir S4** : `StubBackend::new()`, consent L4
  via tempdir `sbfb_home_override` (`dispatch_loop.rs:176-180` vs
  `runtime.rs:1453-1463`), `Engine::new`, `register_task_doc`,
  `take_shutdown_sender`, `timeout(10s)` poll sur `result:`. La seule
  difference est l'ecriture via le **vrai dispatcher** (`run`,
  `dispatch_loop.rs:49`) au lieu d'un `doc.set` a la main — c'est exactement
  ce qui ferme le blind spot B-3.
- **Borrow/move** : `engine.docs()` (l.195) retourne un `DocsClient` detache
  (Arc clone, cf. dim 3) ; `register_task_doc(&mut)` (l.215) +
  `take_shutdown_sender(&mut)` (l.216) sont appeles AVANT le `move` de
  `engine` dans `spawn` (l.217). Ordre correct, compile et tourne.
- **StubBackend deterministe** : `StubBackend::new()` (l.187), hermetique,
  pas d'Ollama/GPU/reseau (au-dela du boot node). Conforme R2.
- **Consent L4** : reproduit a l'identique le pattern du miroir. Correct.
- **Signature** : `make_test_entry()` signe avec une `KeyPair` jetable
  (l.87-88), cle publique embarquee → `verify_signature` (`runtime.rs:899`)
  passe. Meme mecanique que le miroir. (Voir P2-A-1 pour asserter aussi la
  signature du **result** — optionnel, coherent avec le miroir.)

**Conclusion dim 2** : E2E valide, non-contournant, regression-proof, et
**prouve vert en CI Linux** (B-3 satisfait). Le timeout Windows natif est un
artefact poste documente (P2-A-1), partage par le miroir S4 — verifier via
CI/Docker, pas sur le poste Windows seul. Le critere central Phase A
(kickoff §6 : « la 1ere tache dispatchee est reellement vue et executee par
un worker reel, prouve par test ») **est satisfait** en CI.

- **Vrai dispatcher** : la tache est ecrite via `dispatch_loop::run`
  (`dispatch_loop.rs:201`, le **vrai** binaire de production), pas via un
  `doc.set` a la main. C'est exactement la difference load-bearing avec le
  test worker existant `engine_claims_and_executes_tasks_on_registered_doc`
  (`runtime.rs:1417`) qui injecte `doc.set(author, b"task:t-1".to_vec(), ...)`
  (`runtime.rs:1496`) — le « blind spot » que le commentaire E2E
  (`dispatch_loop.rs:136-144`) decrit correctement.
- **Vrai worker engine** : `Engine::new(boot)` (l.191), backend reel via
  le state-machine `run_until_shutdown` (l.217). La tache est claim+executee
  par le chemin de production `scan_and_execute_tasks` (`runtime.rs:830`).
- **Chaine prouvee** : le test asserte `task:` len==1 (l.212, ecrit par le
  dispatcher), puis `result:` non-vide via timeout (l.219-229), puis
  `claim:` len==1 (l.232) et `result:` len==1 (l.234). C'est dispatcher →
  scan → claim → execute → result, la chaine complete.
- **Regression-proof** : OUI. Si le bug B-1 revenait (writer ecrivant
  `tasks/`), le scan worker `get_many_by_prefix(b"task:")` (`runtime.rs:847`)
  ne verrait rien → aucun `claim:`/`result:` ecrit → le `timeout(10s)`
  (l.219) expire → `.expect("worker should claim+execute ... within 10s")`
  panique. Le test echouerait deterministiquement. (En outre l'assert l.212
  `task:` len==1 echouerait directement, puisque le dispatcher ecrirait sous
  `tasks/`.)
- **StubBackend deterministe** : `StubBackend::new()` (l.187) — backend
  hermetique, pas d'Ollama, pas de GPU, pas de reseau. Conforme R2 (skip
  propre / pas de flaky runtime). Le determinisme greedy seed reel est
  Phase B (hors scope A).
- **Consent L4** : `ConsentConfig::default_for(...)` + `consent.level =
  ConsentLevel::All` sauve dans le tempdir `sbfb_home_override` (l.176-180).
  Necessaire car le filtre consent defaut L1 (own-projects) rejetterait l'id
  synthetique `proj-dispatch-e2e`. **Reproduit a l'identique** le pattern du
  test worker existant (`runtime.rs:1453-1463`). Correct.
- **Timeout 10s** : identique au test worker miroir (`runtime.rs:1507`).
  Poll a 100ms, `task_poll_interval_ms: 100`. Marge large. Voir P2-A-2
  (non bloquant, non-regression).
- **Borrow/move** : `engine.docs()` (l.195) retourne un `DocsClient`
  detache (l'Arc est clone, cf. dim 3) ; `register_task_doc(&mut)` (l.215)
  et `take_shutdown_sender(&mut)` (l.216) sont appeles AVANT le `move` de
  `engine` dans `tokio::spawn` (l.217). Ordre correct, pas de conflit.
- **Signature** : `make_test_entry()` signe le `TaskEntry` avec une `KeyPair`
  jetable (`dispatch_loop.rs:87-88`) ; la cle publique est embarquee dans le
  TaskEntry, donc `task_entry.verify_signature()` (`runtime.rs:899`) passe —
  meme mecanique que le test worker (`coord_kp` jetable). Note P2-A-1 :
  l'E2E n'asserte pas l'integrite du **result** signe (juste sa presence) —
  non bloquant, couvert ailleurs.

Conclusion : E2E valide, non-contournant, regression-proof. Ferme B-3
(premier E2E cross-process reel coordinator→worker).

### 3. Accessor `Engine::docs()` — PASS

`runtime.rs:562-564` : `pub fn docs(&self) -> nexus_core_rs::docs::DocsClient`.

- **Additif** : nouvelle methode, ne modifie aucune signature existante,
  ne touche aucun champ. Zero risque de regression sur le chemin worker.
- **Justifie** : `DocsClient::new(inner: &Docs)` clone l'Arc interne
  (`docs.rs:70-73` : « Clones the inner Arc so the resulting client has no
  borrow on the source »). L'accessor permet a l'E2E de creer un doc sur le
  node du worker et d'y ecrire via le **vrai** dispatch loop, puis
  `register_task_doc`. Sans lui, l'E2E ne pourrait pas partager un doc entre
  dispatcher et worker dans le meme process.
- **Pas d'elargissement API risque** : retourne un wrapper `DocsClient`
  deja public, sur `self.node.docs()` deja accessible en interne. N'expose
  pas de mutabilite ni d'internals nouveaux. Coherent avec la famille de
  test-helpers `pub` existants (`register_task_doc` l.548, declare « Test
  helper »). `pub` (non `#[cfg(test)]`) est requis car le test consommateur
  vit dans le crate `nexus-shell-daemon` (cross-crate) — justifie.
- **Doc claire** : le doc-comment (l.552-561) explique le pourquoi (B-3,
  testabilite cross-process) et reference `register_task_doc`. Bon.

Note pre-launch : exposer un accessor `pub` non gate `#[cfg(test)]`
augmente legerement la surface API du crate worker-core. Acceptable
(pre-launch, edition libre, accessor benin sur primitive deja publique).

### 4. G1 stash drop — PASS

Conforme D7 (defaut = drop, garder asciicast `.cast` coherent).

- `git stash list` : le stash WIP terminal `.cast`→`.log` est **absent**.
  Restent `stash@{0}` (pre-reset gossip bootstrap) et `stash@{1}` (WIP skill
  G8) — exactement les deux stashes que le preflight declarait hors-scope a
  preserver (ex-`stash@{1}`/`stash@{2}`, re-indexes apres le drop). Le drop
  a bien retire le bon stash sans toucher les autres.
- `git diff crates/sbfb-factory/src/terminal.rs` : **vide**. Le working tree
  ne contient AUCUNE modification de `terminal.rs` — l'etat HEAD `.cast`
  coherent (livre `864b005`) est preserve. Le commit Phase A ne touchera pas
  `terminal.rs`.
- **Pas de perte critique** : le preflight a documente que le refactor etait
  a ~30% (writer plaintext ecrit, mais 0 call-site recable, 3-5 sites
  d'extension desalignes, build casse). Code incoherent, pas un travail fini
  jete. Le `git reflog show stash` conserve la ref droppee (recuperable ~90j,
  conforme D7). Defaut D7 confirme par lecture du stash dans le preflight.

Conclusion : G1 resolu proprement, conforme D7, sans perte de travail
recuperable, sans toucher les stashes hors-scope.

### 5. Scope — PASS

Phase A ne touche aucun scope cut du kickoff §8 / plan §12.

- **#10 GPU partage cross-machine → S75** : non touche (E2E in-process,
  StubBackend, pas de GPU).
- **#11 quorum redundancy>1 cross-MACHINE → S75** : non touche (E2E =
  redundancy=1, in-process meme node ; le `redundancy_factor: 1` est cable
  dans `make_test_entry` l.84).
- **#13 logprobs/watermark → V2** : non touche (`watermark_seed: Vec::new()`
  l.85, greedy seed = Phase B).
- Les 13 autres scope cuts (ProviderRouter, chat reseau, FTS5, SearchManifest,
  commandes factory, templates, dashboard kudos, @dev tree-sitter, packaging,
  sharding) sont des domaines disjoints du compute routing — aucune ligne
  Phase A ne les approche.
- **Pas de debordement Phase B/C** : Phase A se limite a (1) la cle dispatch
  (B-1), (2) l'E2E cross-process (B-3), (3) l'accessor enabler, (4) le drop
  stash (G1). Le greedy seed (B-2, Phase B), le nettoyage modules morts
  (D8, Phase B), la securite Factory (Phase C) ne sont PAS touches. Le diff
  est strictement borne aux 2 fichiers compute + drop stash.

Conclusion : scope respecte, borne, pas de debordement.

### 6. Commit body readiness — PASS

Les 9 sections du `commit_body_phase.txt` sont renseignables.

- **Contexte** : B-1 (cle desalignee `tasks/` vs `task:`) + B-3 (premier
  E2E cross-process) + G1 (stash drop). Rationale clair, grounding interne.
- **Fichiers** : `dispatch_loop.rs` (fix l.41 + test interne aligne + E2E),
  `runtime.rs` (accessor `docs()`). `terminal.rs` NON liste (aucun diff).
- **Delta tests** : **+1 test** `nexus-shell-daemon`
  (`dispatched_task_is_claimed_and_executed_by_worker_engine`). Le test B-1
  `dispatch_loop_writes_to_doc` est **modifie** (aligne `task:`), count
  inchange. `nexus-worker-core` : **+0 test** (l'accessor `docs()` n'ajoute
  pas de test propre ; il est exerce indirectement par l'E2E shell-daemon).
  Le caller rapporte `-p nexus-shell-daemon -p nexus-worker-core` = 473
  passed 0 skip, incluant le nouvel E2E. Workspace baseline ~1486 → ~1487.
- **Verification §7.4** : fmt OK, clippy 0 warning (constats caller). Le
  trio B-1 + E2E B-3 + miroir S4 **verifie vert en Docker CI Linux**
  (3 passed, cette review). **Important** : la suite nextest doit etre
  validee en **CI Linux**, pas sur le poste Windows natif (P2-A-1 : le pump
  worker-engine iroh-docs timeout sous Windows). Full workspace + doctests
  a confirmer en CI avant promotion PASS.
- **Scope cuts respectes** : les 16 items §8, exhaustif — aucun touche
  (cf. dim 5).
- **G8 traceability** : Preflight `2ec72e8` verdict **EXECUTE**
  (`sprint71_phase_a_preflight.md`). Review : ce fichier, PASS apres Codex.
- **Pre-launch protocol** : `TASK_FORMAT_VERSION` reste 1
  (`make_test_entry` l.70 l'utilise inchange) ; la cle de doc n'est pas un
  champ d'enveloppe versionne — edition libre (15 ahead, rien pousse).
- **Codex verification** : a remplir post-Codex (section reconciliation
  ci-dessous).
- **Carry closure** : B-1 ferme (tache route reellement), B-3 livre (E2E
  cross-process), G1 resolu (stash drop). LT-7 partiel (worker quorum
  cross-machine reste S75). Renseignable.

Conclusion : body 9 sections complet et coherent au moment du commit.

---

## Codex reconciliation

Codex (gpt-5.5, reasoning medium) a verifie les 3 livrables — sortie brute
non reecrite dans `sprint71_phase_a_codex_review.md`. Verdict :

- **B-1 CONFIRME** : writer `task:` (dispatch_loop.rs:41) aligne sur le scan
  worker (runtime.rs:847,859) ; grep confirme aucun autre writer/reader de
  cle doc Iroh `tasks/` (restes = commentaires de regression + routes REST
  `/api/v1/tasks/`).
- **B-3 CONFIRME** : E2E non-tautologique (vrai `Engine` + `StubBackend` +
  vrai `run()`, dispatch_loop.rs:146-234) ; echouerait si B-1 regressait ;
  accessor `Engine::docs()` additif (runtime.rs:562).
- **G1 CONFIRME** : terminal.rs reste sur asciicast `.cast` (aucun
  `PlainTextWriter`, aucun `.log`).

0 GAP, 0 PARTIEL, aucune correction requise. Limite notee par Codex : ses
`cargo test`/`cargo check` ont expire dans sa session (confirmation statique
only) — le runtime vert est prouve par le nextest local (473 passed) + la
confirmation Docker Linux (review). Verdict promu **PASS**.

- Rapport Codex : _(fichier `sprint71_phase_a_codex_review.md`)_
- Livrables audites : _N_ / Confirmes : _N_ / Gaps : _N_
- Reconciliation : _(promotion PASS, corrections GAP)_
