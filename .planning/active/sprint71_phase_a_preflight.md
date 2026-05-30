# Sprint 71 Phase A Preflight

Date: 2026-05-30
HEAD: `2ec72e8`
Verdict: **EXECUTE**

> Note process : le kickoff/plan citent le tip d'entree `d5ddb95` (12 ahead).
> Le tip reel au moment du preflight est `2ec72e8` (15 ahead) — la Phase 0
> audit-absorb (`sprint70_audit_findings.md`) et la migration docs S70 ont
> deja ete commitees en `chore(planning)` (`2ec72e8`, `1190d18`). Aucune
> divergence de scope : ce sont des chores planning, pas du code feat.

## Evidence Rules
- Claim policy : chaque affirmation cite un chemin/ligne, une sortie de
  commande, une URL/date, ou une hypothese explicite.
- Local sources read :
  - `prompts/agent/preflight.md` (procedure portable, integrale)
  - `.planning/active/sprint71_kickoff.md` (D1, D7, R4, R2)
  - `.planning/active/sprint71_plan.md` (§5 Phase A, §10 fail-fast)
  - `.planning/active/sprint70_audit_findings.md` (B-1, B-3, G1)
  - `crates/nexus-shell-daemon/src/dispatch_loop.rs` (writer + test interne)
  - `crates/nexus-worker-core/src/engine/runtime.rs` (scan worker + E2E in-process)
  - `crates/nexus-shell-daemon/src/tasks_api.rs` (route REST GET task)
  - `crates/nexus-shell-daemon/src/http.rs:1444` (submit handler → channel)
  - `crates/nexus-test-harness/src/lib.rs` (DaemonCluster) + `tests/multi_daemon.rs`
  - `crates/nexus-worker-core/src/llm/ollama.rs` (StubBackend) + `llm/mod.rs` (GenerateParams)
  - `docs/security/THREAT_MODEL.md` (S3)
  - memory `feedback_approach.md` (pick-deepest, anti-pattern S24 LLM stochastique)
- Commands run (extraits pertinents ci-dessous dans chaque scan).

## Scope
- Plan source : `.planning/active/sprint71_plan.md §5 (Phase A, lignes 131-191)`.
- Target files :
  - `crates/nexus-shell-daemon/src/dispatch_loop.rs` (writer cle, l.35 ; **+ test interne l.111,121** — voir S4)
  - `crates/nexus-worker-core/src/engine/runtime.rs` (scan worker l.833,845 — **reference, inchange**)
  - `crates/nexus-test-harness/tests/compute_e2e.rs` (NOUVEAU, B-3)
  - `crates/sbfb-factory/src/terminal.rs` (decision G1/D7 — stash)
- Deps/APIs/specs Phase A : **aucune nouvelle dep**. Les 3 deps off-sprint
  (`portable-pty`, `async-stream`, `futures`) sont scope Phase B / G13
  (`plan §6`, `kickoff §1`), pas Phase A. Le harness `nexus-test-harness`
  a deja `tokio`/`reqwest`/`serde`/`tempfile`/`anyhow`/`zip`
  (`crates/nexus-test-harness/Cargo.toml`) — suffisant pour B-3.
- Security/protocol surfaces : cle de doc iroh-docs interne
  coordinator→worker (transport applicatif). `TASK_FORMAT_VERSION` reste 1.
  Pas de canonical bytes touche (la cle de doc n'est PAS du canonical signe —
  voir S4).
- Tests expected (`plan §5 A.3`) :
  1. `dispatch_loop::tests::dispatched_key_uses_task_prefix`
  2. `compute_e2e::dispatch_to_worker_roundtrip`
  3. `compute_e2e::coordinator_worker_ollama_validation` (gate Ollama, skip propre)
  4. `terminal::tests::session_extension_consistent` (seulement si plaintext retenu)

## S1a OSS Prior Art
- Domain : dispatch de travail cross-process + test E2E avec runtime LLM
  optionnel (skip propre si absent).
- Sources (consultees 2026-05-30) :
  - BOINC work dispatch / client-server :
    https://github.com/BOINC/boinc/wiki/Server-trouble%E2%80%90shooting
    (modele serveur dispatch → client claim, le client poll un work-unit
    par cle stable ; la cle de routage est interne, pas un asset de securite).
  - Rust integration testing avec service externe (Ollama) :
    https://oneuptime.com/blog/post/2026-01-26-rust-integration-tests/view
    et https://github.com/pepperoni21/ollama-rs — pattern : tests d'integration
    echouent clairement si le service est absent ; idiome `#[ignore]` + env-var
    gate + health-check au demarrage, OU stub deterministe in-process.
- Finding : **APPROACH-ALIGNED**.
  - Le projet implemente DEJA les deux primitives du pattern OSS :
    (1) gate par env-var `SBFB_INTEGRATION=1` (`tests/multi_daemon.rs:13-15`),
    (2) `StubBackend::with_forced_output` deterministe in-process
    (`crates/nexus-worker-core/src/llm/ollama.rs:243,438`).
  - Le plan A.3 test 3 dit explicitement « gate sur disponibilite Ollama
    (skip propre sinon, cf. R2) » — conforme a l'etat de l'art.
- Impact : aucune adaptation. (S1a est NON bloquant et ne pourrait de toute
  facon mener qu'a PLAN-ADAPT, pas DESIGN-CONFLICT.)
- Note pick-deepest (memory `feedback_approach.md` #54) : l'anti-pattern S24
  (hash binaire BLAKE3 sur sortie LLM stochastique = 100% faux positifs) est
  precisement traite par D2 greedy seed-fixe — mais c'est **Phase B**, hors
  scope Phase A. Pour Phase A, le test E2E peut s'appuyer sur le `StubBackend`
  deterministe (`with_forced_output`) pour la validation `redundancy=1` sans
  dependre du non-determinisme GPU.

## S1b Dependencies, CVEs, Release Notes
- Scanned : aucune dep ajoutee en Phase A. Verifie via `plan §5 A.2`
  (aucune ligne Cargo.toml dans les fichiers touches) et
  `crates/nexus-test-harness/Cargo.toml` (deps existantes suffisantes).
- Les 3 deps off-sprint (`Cargo.toml:91 async-stream`, `:93 futures`,
  `portable-pty`) sont **explicitement deferrees Phase B / G13**
  (`plan §6 B.1(3)`, `kickoff §1` « passent au preflight G8/S1b en Phase B »).
- Finding : **clean** pour Phase A. (Le scan CVE de ces 3 deps est un livrable
  trace du preflight Phase B, pas Phase A.)

## S2 Historical Decisions
- Commands :
  - `git log --oneline --all -- crates/nexus-shell-daemon/src/dispatch_loop.rs`
    → 4 commits : `abc16a6` (S55 timing), `1d010b0` (S54 edition 2024),
    `22695ed` (S52 shutdown), `63875d9` (S49 — **creation du fichier**).
  - `git log -L 35,35:.../dispatch_loop.rs` → la ligne `format!("tasks/{}", ...)`
    existe **depuis la creation du fichier** (`63875d9`, S49 Phase A) ; les
    commits ulterieurs ne l'ont jamais touchee (seulement le shutdown/edition).
  - `git show 63875d9 --no-patch --format=%B` → le body decrit « dispatch loop
    MPSC qui ecrit les TaskEntry dans le doc » mais **ne mentionne jamais le
    choix du prefixe de cle** `tasks/` ni ne le confronte au scan worker
    `task:`. La section G8 du body dit « EXECUTE plan-as-is (752b85d) ».
  - `git log --all --grep='DEVIATION|rejected|scope-cut|threat-model' -- dispatch_loop.rs`
    → 0 hit. Aucun grep DEVIATION/threat-model sur `tasks/`.
- Reverse-commit check :
  - Le worker scanne `task:` depuis Sprint 4 (`runtime.rs:794` commentaire
    « Sprint 4 Phase D W9.1 », `runtime.rs:833,845`). Le `tasks/` (S49) est
    donc **posterieur** au `task:` (S4) de 45 sprints. Le writer a ete ajoute
    desaligne, jamais l'inverse.
  - Aucun commit `63875d9..HEAD` ne re-aligne ni ne re-debat le prefixe :
    `git log --all -S 'tasks/' -- dispatch_loop.rs` ne retourne que `63875d9`.
- Decisions crossed : aucune. Le `tasks/` n'est **pas** une decision documentee
  avec rationale — c'est un **bug d'inattention** introduit a la creation du
  dispatch loop (S49), ou l'auteur a ecrit un nouveau prefixe sans verifier le
  prefixe que le worker scannait deja depuis S4.
- Finding : **clean** (non bloquant). C'est un bug pur, pas une reversion d'une
  decision valide. D1 (changer le writer vers `task:`) ferme le bug et aligne
  sur le chemin worker rode. Conforme au classement S2 « confirmed reversion /
  no valid rationale » → non bloquant.

## S3 Local Patterns And Threat Model
- Threats/contracts checked : la cle de doc compute est interne au transport
  iroh-docs entre le coordinator (dispatch_loop, **sole writer**,
  `dispatch_loop.rs:4-6`) et le worker (scan, `runtime.rs:833`).
- La securite du chemin compute repose sur la **signature Ed25519 du contenu**
  (`TaskEntry::sign` / `verify_signature`), PAS sur le nom de la cle :
  - le worker re-verifie `task_entry.verify_signature()` apres deserialisation
    (`runtime.rs:885-888`) — quelle que soit la cle, une tache non signee par
    le coordinator est rejetee.
  - `docs/security/THREAT_MODEL.md` (STRIDE l.180 « Peer nie avoir envoye une
    task » ; l.196 « Autre user local lit les tasks via UDS ») traite la
    signature et les peer-creds, jamais le prefixe de cle de doc.
- HARDENING / T0-T5 : changer `tasks/`→`task:` ne touche aucune surface de
  menace (T0-T5, AD5 RCE bridge, caps W/VRAM). Aucune regression d'une menace
  deja couverte.
- Finding : **clean**. Surface basse confirmee (conforme a l'attente du
  prompt : « la cle dispatch est interne au transport iroh-docs ... surface
  basse, confirme »).

## S4 Protocol And Wire Invariants — LE SCAN CRITIQUE
Objectif : prouver qu'APRES le fix `tasks/`→`task:`, il n'existe **aucun autre
lecteur/ecrivain de `tasks/`** qui casserait, et que `task:` est le prefixe
canonique partout. (Risque R4 du kickoff.)

### Sites de cle de doc compute (grep exhaustif sur `crates/**/*.rs`)

| Site | Fichier:ligne | Role | Forme | Action Phase A |
|------|---------------|------|-------|----------------|
| WRITER (production) | `dispatch_loop.rs:35` | `format!("tasks/{}", entry.task.task_id)` puis `doc.set(...)` (l.43) | `tasks/{id}` | **FIX → `task:{id}`** (D1) |
| WRITER (test interne) | `dispatch_loop.rs:111` | `doc.get_many_by_prefix(b"tasks/")` (assert sur le writer) | `tasks/` | **ALIGNER → `b"task:"`** (sinon R4) |
| WRITER (test interne) | `dispatch_loop.rs:121` | `assert_eq!(stored_key, format!("tasks/{task_id}"))` | `tasks/{id}` | **ALIGNER → `task:{id}`** (sinon R4) |
| READER (production) | `runtime.rs:833` | `doc.get_many_by_prefix(b"task:")` | `task:` | inchange (reference) |
| READER (production) | `runtime.rs:845` | `.strip_prefix("task:")` | `task:` | inchange (reference) |

### Confirmation : aucun AUTRE lecteur/ecrivain de `tasks/` (clé de doc)

- `grep "tasks/"` sur `crates/**/*.rs` → **uniquement** `dispatch_loop.rs`
  (l.35 prod, l.111 + l.121 test interne). Aucun autre site n'ecrit ni ne lit
  le prefixe `tasks/` comme **cle de doc**.
- Faux positifs ecartes (NON concernes par le fix) :
  - `http.rs:306,405` `/api/v1/tasks/submit` et `/api/v1/tasks/{task_id}` =
    **routes HTTP REST**, pas des cles de doc iroh. Distincts par nature.
  - `tasks_api.rs:122` `get_task` lit la **DB SQLite du coordinator**
    (`db.get_task(&task_id)`), pas le doc iroh — verifie en lisant le handler
    entier. La route REST n'a aucun rapport avec le prefixe de cle de doc.
  - `task.rs`, `TaskEntry`, `task_canonical_bytes` = la **structure signee**,
    independante du nom de cle de doc.
- Le worker enregistre le doc dans `task_docs` puis scanne `task:` ; le
  submit handler (`http.rs:1444`) envoie le `TaskEntry` dans le channel MPSC
  qui aboutit au `dispatch_loop` — c'est le **seul** chemin d'ecriture de la
  cle de doc. Une fois `dispatch_loop.rs:35` aligne sur `task:`, la cle est
  **alignee bout-en-bout** : writer prod `task:` → scan worker `task:` →
  `strip_prefix("task:")`. Les claims (`claim:`) et results (`result:`) ont
  leurs propres prefixes, ecrits par le worker (`runtime.rs:1024,1087`), non
  affectes.

### Invariants wire
- `TASK_FORMAT_VERSION` reste `1` (aucune raison CVE de bump). Verifie : la cle
  de doc n'est PAS versionnee — c'est une cle de routage de transport, pas un
  champ d'enveloppe canonique signe. Le fix ne touche pas `canonical.rs`.
- Pas de decodeur multi-version introduit. D1 rejette explicitement la
  « lecture tolerante des deux prefixes » (`kickoff §5 D1 rejete`) = pas de
  band-aid, cohérent avec la policy pre-launch.
- Pre-launch protocol (`CLAUDE.md`, `kickoff §2.3`) : 15 ahead origin, rien
  pousse, aucun noeud tiers ne parle ce protocole. **Edition libre** : pas de
  migration, pas de bump, pas de decodeur range. La correction B-1 change le
  wire applicatif librement.
- Day 0 status : **preserved**. D1 (prefixe `task:` unique) respecte ; aucune
  decision gelee contredite.
- Finding : **clean wire**, avec **1 note d'execution non bloquante** (voir
  Risques) : le test interne `dispatch_loop.rs:111,121` DOIT etre aligne sur
  `task:` dans le MEME commit que le fix l.35, sinon le test casse (R4).

## Risks And Scope Cuts
- Blocking risks : **aucun**. Les 5 scans sont clean ou non bloquants.
- Non-blocking risks (carry-over Phase A) :
  - **R4 (S4 note)** : `dispatch_loop.rs:111` (`get_many_by_prefix(b"tasks/")`)
    et `:121` (`assert_eq! ... format!("tasks/{task_id}")`) sont un **test
    interne** qui assert la forme `tasks/`. Le fix l.35 DOIT etre accompagne
    de l'alignement de ces deux lignes vers `task:` dans le meme commit, sinon
    le test `dispatch_loop_writes_to_doc` echoue. C'est l'unique site « test
    in-process injectant tasks/ » que le kickoff R4 anticipe. Mitigation deja
    appliquee par ce preflight : les deux lignes sont identifiees nommement.
  - **R2 (E2E flaky)** : le test 3 (`coordinator_worker_ollama_validation`)
    doit gater sur disponibilite Ollama (idiome `SBFB_INTEGRATION` /
    health-check) OU s'appuyer sur `StubBackend::with_forced_output` pour un
    E2E deterministe sans Ollama. Le harness `DaemonCluster`
    (`nexus-test-harness/src/lib.rs`) spawn des daemons reels mais **n'a pas**
    de helper worker-process ni de helper submit compute — l'auteur Phase A
    devra soit ajouter ce helper, soit prouver le round-trip de cle via un doc
    partage in-test (suffisant pour `dispatch_to_worker_roundtrip`, test 2).
    Le « cross-process » strict (worker process separe) peut etre limite a
    `redundancy=1` / machine dev (scope cut #11 → S75 pour le cross-machine
    reel). Non bloquant : c'est un choix d'implementation du test, pas un
    conflit de design.
- Scope cuts honored (`kickoff §8`, `plan §12`) — Phase A n'en touche aucun :
  - #10 GPU partage cross-machine → S75 (Phase A prouve 1-tache cross-process,
    pas le GPU partage).
  - #11 quorum redundancy>1 cross-MACHINE reel → S75 (Phase A = `redundancy=1`).
  - #13 logprobs/watermark → V2 (Phase A n'y touche pas ; greedy seed = Phase B).
  - LT-7 worker quorum E2E (`kickoff §7`) : Phase A livre l'E2E cross-process
    partiel ; le quorum cross-machine reste S75.

## Decision G1 / D7 — WIP terminal stash@{0}

Commande : `git stash show -p stash@{0}` (diff integral lu).

### Constat factuel
Le stash modifie **uniquement** `crates/sbfb-factory/src/terminal.rs`
(+88/-15, 1 fichier). Il fait DEUX choses, et **rien d'autre** :
1. `session_log_path` : change l'extension `.cast` → `.log`
   (`terminal.rs:27` au HEAD).
2. **Supprime** `write_asciicast_header` (HEAD l.30) et
   `write_asciicast_event` (HEAD l.41) et **ajoute** un `PlainTextWriter`
   (ANSI stripper).

### Pourquoi le stash est un demi-travail INCOHERENT (drop recommande)
- Les fonctions supprimees par le stash sont **encore appelees** au HEAD :
  `write_asciicast_header(f, cols, rows)` (`terminal.rs:133`) et
  `write_asciicast_event(f, start, &chunk)` (`terminal.rs:143`). Le stash ne
  modifie PAS ces call-sites → **le build casse** (fonctions appelees mais
  supprimees). C'est exactement le constat audit G1 (`sprint70_audit_findings.md:84-88`
  « cassait le build : write_asciicast_* supprimes mais appeles »).
- Le nouveau `PlainTextWriter` n'est **cable nulle part** : aucun `.feed()`,
  aucun `.finish()`, aucun `PlainTextWriter::new(...)` dans le diff. Code mort
  ajoute.
- Les sites d'extension ne sont **PAS** alignes — le stash en laisse au moins
  3 desynchronises avec le `.log` :
  - `list_sessions` filtre `Some("cast")` (`terminal.rs:213`) — inchange par
    le stash → ne verrait plus les fichiers `.log`.
  - `operator_server.rs:850` construit `{name}.cast` — hors du fichier touche
    par le stash, donc inchange → 5e site desaligne.
- Bilan : le stash applique la moitie 1 du refactor (writer + extension) sans
  toucher la moitie 2 (call-sites + 3-5 sites d'extension + cablage). C'est
  precisement le « demi-travail incoherent » du critere D7.

### Recommandation (decision finale a l'agent Phase A, pas tranchee ici seul)
**DROP `stash@{0}`, garder l'asciicast `.cast` (etat HEAD coherent).**
- Defaut D7 confirme par la lecture : le plaintext n'est PAS « presque fini et
  coherent » — il est a ~30% (writer ecrit, mais 0 call-site recable, 3-5 sites
  d'extension desalignes, build casse).
- L'etat HEAD `.cast` est coherent et deja livre (`864b005` persistance
  asciicast + session list, `kickoff §5 D7`). La valeur du plaintext (logs
  lisibles) ne justifie pas de terminer un refactor a 5 sites quand l'asciicast
  fonctionne — et un refactor `.cast`→`.log` complet serait un chantier dette
  a part entiere (recabler `handle_terminal_ws`, `list_sessions`,
  `operator_server.rs:850`, label UI front), hors objectif Phase A.
- Si l'agent Phase A retient malgre tout le plaintext (option non recommandee),
  alors le test 4 `terminal::tests::session_extension_consistent` devient
  obligatoire ET les 5 sites (`session_log_path` l.27, call-sites l.133/143,
  `list_sessions` l.213, `operator_server.rs:850`, label UI) doivent etre
  alignes — etat coherent obligatoire, pas d'intermediaire (D7).

`git stash drop stash@{0}` resout G1 (`fail-fast §10` ligne « stash@{0} resolu »)
et laisse `stash@{1}`/`stash@{2}` intacts (hors scope Phase A — pre-reset gossip
+ WIP skill, non lies au terminal).

## Action
- **EXECUTE** : proceder avec Phase A telle que planifiee (`plan §5`).
- Conditions d'execution (non bloquantes, integrees au plan, a respecter au commit) :
  1. **B-1** : changer `dispatch_loop.rs:35` `tasks/{}` → `task:{}` ET aligner
     le test interne `dispatch_loop.rs:111` (`b"tasks/"` → `b"task:"`) et `:121`
     (`format!("tasks/{task_id}")` → `format!("task:{task_id}")`) dans le MEME
     commit (S4 / R4). Le scan worker `runtime.rs:833,845` reste la reference,
     inchange.
  2. **B-3** : test 2 `dispatch_to_worker_roundtrip` peut prouver l'alignement
     de cle via un doc partage in-test ; test 3
     `coordinator_worker_ollama_validation` gate sur Ollama (skip propre,
     `SBFB_INTEGRATION` / health-check) ou s'appuie sur `StubBackend`
     deterministe. Cross-machine reel = S75 (scope cut #11).
  3. **G1 / D7** : `git stash drop stash@{0}` (recommandation forte) ; garder
     l'asciicast `.cast` HEAD coherent.
- Le commit body Phase A cite ce preflight (verdict EXECUTE) en section G8.
