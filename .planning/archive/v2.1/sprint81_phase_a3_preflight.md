# Sprint 81 Phase A3 — Préflight G8 (Workflow ultracode)

> **Verdict : PLAN-ADAPT + SPLIT A3a / A3b.** La DIRECTION du plan (mesurer la
> baseline transport LIVE 0.98 + fermer le blocker WAN task-delivery 0-bump avant le
> bump, C10) est ratifiée. Mais la LETTRE « fix WAN task-delivery (root-cause S77,
> 0-bump) » repose sur une prémisse imprécise sur DEUX plans, re-vérifiée au code
> (source iroh-docs 0.98 installée + call-graph SBFB) :
> 1. Un fix keepalive **existe déjà** et est **câblé** depuis S77 Phase A (`36cf1cc`,
>    `doc_sync.rs` + `nexus-worker-core/src/engine/runtime.rs:741`), **jamais prouvé LIVE**
>    (T2 RIG-ABSENT / `.b3_last_result.json` jamais joué). Le blocker n'est pas
>    « non-fixé », il est « livré-mais-jamais-mesuré-live » pour sa facette worker.
> 2. **La facette DOMINANTE observée (redémarrage / hot-binary-swap du coordinateur) est
>    un gap DISTINCT côté COORDINATEUR, toujours NON-fixé**, que le keepalive worker ne
>    peut PAS compenser (le coordinateur **rejette** les syncs du worker). C'est la
>    root-cause re-établie de première main ci-dessous (§2/§5).
>
> Workflow G8 : 5 scans (S1a code+root-cause / S1b deps / S2 historique / S3 threat /
> S4 wire) + 5 vérifications adversariales + synthèse main-thread de première main.
> Faits décisifs re-lus au code par le main thread contre `iroh-docs-0.98.0`
> (registre cargo) et le call-graph SBFB (§5). Même pattern faux-prémisse qui a fait
> finir A et A2 en PLAN-ADAPT.

## 1. Rappel de la lettre du plan (sprint81_plan.md §Phase A3 + kickoff C2/C10)

Phase A3 « Baseline transport LIVE 0.98 + fix WAN task-delivery (C10) [0-bump] ».
Livrables (plan `:88-91`) : (1) artefact **JSON b3 par palier COMMITTÉ** (baseline 0.98,
re-jouable à l'identique post-bump = différentiel propre) ; (2) run Win `SBFB_INTEGRATION=1`
archivé (les tests `multi_daemon` relay-gated early-returnent verts EN SILENCE en CI) ;
(3) **Ollama installé sur le Mac** ; (4) **copie du store VPS rapatriée** (ressource
Phase F) ; (5) **fix WAN task-delivery** (root-cause S77, 0-bump). Delta `+1..4` Rust.
0-bump strict. Split autorisé si le fix est jugé trop gros (précédent : split E').
C10 ratifié PO 2026-07-02 : fixer AVANT le bump pour que le palier quorum b3 ait un
PASS atteignable (aucun b3 quorum PASS complet n'a jamais existé — S81 vise le premier).

## 2. Pourquoi PLAN-ADAPT (prémisse imprécise, root-cause re-établie au code)

### 2.1 Un fix keepalive existe déjà (facette worker), jamais validé live

`spawn_doc_sync_keepalive` (`nexus-core-rs/src/doc_sync.rs:165`) ré-émet
`start_sync(peers)` sur `NeighborDown` (fast-path) et via un backstop périodique
(`DEFAULT_CHECK_INTERVAL=15s`, `DEFAULT_MIN_REJOIN_INTERVAL=5s`, cooldown `:140-142`).
Câblé en PROD, unique call-site : `nexus-worker-core/src/engine/runtime.rs:741` (dans
`run_until_shutdown`, capture des peers du ticket `:367`, teardown `:810`). Livré par
`36cf1cc` (S77 Phase A). **Red→green prouvé UNIQUEMENT hermétiquement** via
`leave()` (`doc_sync.rs:307` `keepalive_rejoins_doc_after_neighbor_loss`) : in-process
le neighbor se forme trivialement. La preuve WAN a été explicitement routée au harness
b3 (T2, Phase K) resté **RIG-ABSENT** → efficacité WAN **INCONNUE**. Le S76-D dedup
`(worker_pubkey,task_id)` (`result_sync.rs:130-132`) est sur le chemin RÉSULTAT,
orthogonal à la delivery.

### 2.2 La root-cause DOMINANTE est côté COORDINATEUR et n'est PAS fixée (finding neuf)

Re-vérifié contre la source `iroh-docs-0.98.0` installée :

- Le broadcast incrémental (`LocalInsert -> gossip.broadcast`) est **gaté par
  `is_syncing(namespace)`** (`engine/live.rs:714` : `if self.state.is_syncing(&namespace)`).
- `is_syncing` = `self.0.contains_key(namespace)` (`engine/state.rs:60`), peuplé
  **uniquement** par `insert()` (`state.rs:65`), appelé **uniquement** dans `start_sync`
  (`live.rs:414`).
- L'**acceptation d'un sync entrant** exige aussi le namespace dans le sync-set :
  `accept_request` (`state.rs:90-100`) fait `let Some(state) = self.entry(namespace, node)
  else { return AcceptOutcome::Reject(AbortReason::NotFound) }` ; `entry()` (`state.rs:154`)
  rend `None` si le namespace n'est pas dans `self.0`. **Un coordinateur hors sync-set
  REJETTE toute demande de sync du worker.**
- **Comment le coordinateur entre-t-il dans son sync-set ?** UNIQUEMENT via `share()`
  côté serveur : le handler `doc_share` (`api/actor.rs:407`) appelle
  `self.start_sync(doc_id, vec![])`. C'est **transitoire, en mémoire**.
- **Le call-graph SBFB** : le coordinateur ouvre le project doc au boot via
  `open_doc` (`nexus-shell-daemon/src/runtime.rs:650`) ou `create_doc` (`:656`). **Ni
  l'un ni l'autre n'appelle `start_sync`** (`open_doc` wrapper `docs.rs:159-166` = pur
  `open` ; `create_doc` `docs.rs:106-113` = pur `create`). `share_write()` n'est appelé
  **qu'à la demande** au mint d'invite (`invite_api.rs:80`, `local_worker.rs:307`),
  **jamais au boot**. `subscribe()` (result_sync `:192`) n'entre PAS dans le sync-set
  (handler `doc_subscribe` = stream de subscribers, pas `start_sync`). Grep prod
  confirme : **le SEUL `start_sync` de tout le repo est le keepalive worker**
  (`doc_sync.rs:143`) — **0 `start_sync` côté coordinateur/daemon**.

**Conséquence (le blocker exact `worker=0 / pending / worker_node_id:null`)** : après un
**redémarrage / hot-binary-swap** du coordinateur (le symptôme précis observé en
`sprint76_verification.md` §5.1) — ou tout boot où aucune invite n'est re-mintée dans
la session — le project doc rouvert n'est **pas** dans le sync-set. Donc (a) les écritures
`task:` incrémentales du dispatch loop sont **LocalInsert non-broadcastées** (gate
`live.rs:714`), et (b) les syncs du worker déjà-enrôlé sont **REJETÉS** (`NotFound`). Le
keepalive **worker** ne peut rien : il ré-émet `start_sync` **depuis le worker**, mais le
coordinateur rejette le sync. Seul le bulk initial (quand le coordinateur était encore
in-sync-set d'un `share` récent) livre ; l'incrémental sur restart ne converge jamais.

### 2.3 Pourquoi ce n'est PAS un DESIGN-CONFLICT

Aucune décision Day-0 D1..D8 ne fige un « boot sans `start_sync` coordinateur ». Le fix
candidat (un `start_sync(project_doc, vec![])` au boot, miroir de ce que `share_write`
fait déjà) **réutilise la primitive `start_sync`** — la même que le keepalive worker —,
est **0-bump** (aucun wire/JCS/`DOMAIN_*`/ALPN/`FORMAT_VERSION`), et **ne met AUCUN
relais N0 dans le hot-path** (`start_sync(vec![])` n'appelle personne : il rend le doc
serveur/broadcast ; le worker dial via pkarr `presets::N0` inchangé). Le verrou
anti-recentralisation S74/S75 est préservé. Bande-aids D1 rejetés (poll, HTTP push,
relais hot-path) restent rejetés. → **PLAN-ADAPT**, pas DESIGN-CONFLICT.

### 2.4 Correction des conflations héritées (à ne PAS coder dans A3)

- **RE-DRIVE-ON-INGEST / SeedAnnounced peer_count:0 / seeder catalog_len:0** ne sont
  PAS le chemin task-delivery. Ce sont le boot-seed-driver + node-directory + seed-announce
  (`nexus-shell-daemon/src/runtime.rs` `run_boot_seed_driver`/`reannounce_directory_at_boot`,
  S75 Phase E). Sous-systèmes disjoints, « reliés-par-classe » (convergence gossip WAN
  non exercée live) mais **jamais le même bug**. Les fermer ne ferme pas A3 et
  réciproquement. Carries distincts (§10), hors A3.
- **A2 (self-heal namespace boot, `23f3be8`, `runtime.rs:2456-2633`) ne touche PAS la
  convergence gossip.** La mémoire `idea_hub` conflate « self-heal ×2 = convergence
  cassée » — FAUX. A2 = Err-handling `open_doc`. La convergence WAN reste l'enjeu propre
  d'A3.
- **A3 n'est PAS dans un vide de couverture.** `dispatch_loop.rs` a des tests hermétiques
  in-process 2-nœuds de task-delivery : `convergence_incremental_task_reaches_remote_replica`
  (`:396`, un `task:` écrit par le VRAI dispatch loop atteint la réplique distante),
  `convergence_boot_catchup_still_works` (`:441`), `convergence_remote_write_visible_to_local_subscriber`
  (`:478`) + le E2E `dispatched_task_is_claimed_and_executed_by_worker_engine` (`:182`).
  **Ce qui MANQUE = un test reproduisant le mode redémarrage-coordinateur** (open_doc sans
  share) — précisément le trou que A3b comble hermétiquement. `multi_daemon` n'a qu'un
  `test_cross_daemon_task_stub` (`:738`, stub /info, pas de dispatch réel).

## 3. Approche corrigée + décision SPLIT (supersede la lettre)

**Ordre imposé : OBSERVER avant de CODER.** Le keepalive worker existe et est testé
hermétiquement ; le gap coordinateur est code-évident mais sa **suffisance live est
non prouvée**. Le plan sanctionne le split (précédent E'). La bisectabilité est
l'invariant cardinal S81. → **SPLIT A3a (observationnel/infra, rig-gated) / A3b (fix
code coordinateur, hermétique-mergeable + confirmation live)**.

### A3a — Baseline transport LIVE 0.98 + ressources rig (0 fix code)

Commit `chore(acceptance): Sprint 81 Phase A3a — baseline transport b3 LIVE 0.98
committée + ressources rig (0-bump)`.

1. **Jouer `b3_live_pc_vps.sh` palier 1 sur HEAD 0.98** (keepalive worker en place,
   fix coordinateur PAS encore) → observer le stage réel du blocker via l'auto-diagnostic
   delivery-vs-result (`b3_live_pc_vps.sh:388-399`). **D'abord écarter le confondeur
   PROJECT_ID** (le harness documente `:26-30` qu'un ancien BLOCK WAN était un
   faux-positif project-mismatch → raison du `RIG-ABSENT` exit 3). Baseline HONNÊTE
   attendue = **BLOCK{delivery}** sur le mode restart (ce N'EST PAS un échec de phase :
   c'est le différentiel avant-fix, cf. C10 « aucun b3 quorum PASS n'a jamais existé »).
2. **Committer un artefact baseline CURATED** (schéma **verdict-only**, calqué sur
   `.planning/archive/v2.1/sprint80_t2_acceptance.json`) sous **`.planning/`** — PAS le
   chemin raw gitignoré (`.gitignore:148`). Chemin proposé
   `.planning/active/sprint81_t2_baseline_098.json`, clé par palier, axe différentiel
   `iroh_baseline:"0.98"`. **Whitelist** `status/palier/redundancy/delay_s/claim_s/verdict`
   + `diagnosis` réduit aux **catégories fermes** (`task never reached worker replica` /
   `reached but no result`) — **DROP `last_response`, DROP tout `${RESP}`, 0 IP/SSH/token/
   username/clé/path absolu**, node_ids pseudonymisés (`nodeVPS/nodePC/nodeMac`) si
   load-bearing sinon omis (S3 R1/R4 ; note S3 : le path `$WORKER_LOG` injecté en
   `diagnosis` est ABSOLU `b3_live_pc_vps.sh:391,394` → le scrubber DOIT l'omettre).
   Laisser `scripts/acceptance/.b3_last_result.json` + `.b3_worker.log` gitignorés.
3. **Ollama installé sur le Mac** (prérequis DUR du palier 2 quorum, indépendant du WAN ;
   sans lui `#30` reste BLOCK quoi qu'il arrive côté delivery).
4. **Copie du store VPS rapatriée** (ressource Phase F) : stockée sous `data/` (déjà
   ignoré `.gitignore:4`) ou hors-repo (scratchpad), **jamais committée**. **Ajouter une
   règle `.gitignore *.redb`** (belt-and-suspenders : `*.db` `:5` NE matche PAS `.redb` —
   vérifié : `foo.redb` non ignoré) car `docs.redb` porte le NamespaceSecret = capacité
   d'écriture (S3 R2).
5. **Run Win `SBFB_INTEGRATION=1` archivé** (sortie nextest scrubbée du préfixe home).

### A3b — Fix convergence coordinateur (code, 0-bump, hermétique red→green)

Commit `fix(daemon): Sprint 81 Phase A3b — coordinateur re-entre son sync-set iroh-docs
au boot du project doc (0-bump)`.

1. **Après open/create du project doc** (`nexus-shell-daemon/src/runtime.rs:660-666`),
   appeler `project_doc.start_sync(vec![])` (ou `share_write()` jeté) pour re-entrer le
   sync-set : le coordinateur **broadcaste** ses écritures `task:` incrémentales
   (`live.rs:714` passe) ET **accepte** les syncs du worker (`accept_request` trouve
   l'entrée). Réutilise la primitive iroh-docs, aucun nouveau wire. Miroir exact de ce que
   `doc_share` (`actor.rs:407`) fait déjà à la demande.
2. **Test hermétique red→green (finding-driven, couvre le mode que RIEN ne teste)** :
   boot node A over store S, `share_write` → `set task:` → converge (baseline verte) ;
   **shutdown A, boot A' sur le MÊME store, `open_doc` SANS share**, `set task:2` →
   **CONTROL rouge : ne converge pas** (reproduit `worker=0` sur restart) ; **appliquer le
   boot `start_sync`** → converge. Delta `+1..2` Rust (façon `boot_*_namespace_persistent_reopen`).
3. **Re-jouer `b3_live_pc_vps.sh` palier 1** post-fix → différentiel **BLOCK→PASS** (
   rig-gated ; le code + test mergent indépendamment du rig). Si palier 2 quorum reste
   BLOCK faute d'Ollama-Mac au moment du run, le documenter honnêtement (BLOCK{quorum} vs
   BLOCK{delivery}), jamais maquiller en RIG-ABSENT si le rig transport est présent.

**Garde-fous A3b (non négociables)** : 0-bump strict (`task:` = clé document `set_bytes`,
hors canonical ; `TASK_FORMAT_VERSION=1`, `DOMAIN_TASK/RESULT/CLAIM_V1`, ALPN,
`FEED_FORMAT_VERSION` intacts) ; 0 relais N0 hot-path ; `task:` aligné S71-B1 (`2f9238d`)
intact ; NE PAS ré-implémenter/reverter le keepalive worker (complémentaire) ; si le fix
passe des peers non-vides à `start_sync`, hériter du cooldown + known-peer bound §15.3
(ici `vec![]` = pas de dial, pas d'amplification).

### Pourquoi le split, pas un A3 unifié

(a) Bisectabilité (invariant cardinal S81) : un commit baseline-BLOCK + un commit fix
qui flippe BLOCK→PASS = le différentiel le plus propre possible. (b) A3b (code+test)
merge **sans rig** ; A3a est rig-gated — ne pas bloquer le fix derrière la dispo
matériel. (c) Les 5 scans recommandent indépendamment le split. (d) Précédent E' cité
par le plan. (e) Isole la surface `start_sync`/`is_syncing` que le bump iroh 1.0 migre
(re-validation Phase B/C).

## 4. Restitution des scans (fan-out 5 + adversarial 5)

| Scan | Verdict-hint | Findings clés | Adversarial |
|---|---|---|---|
| S1a code+root-cause | PLAN-ADAPT | keepalive DÉJÀ livré+câblé `36cf1cc` (`runtime.rs:741`), jamais live ; S76-D orthogonal ; chemin task: tracé ; b3 gitignoré | 13/13 CONFIRMED ; **P1-2e-moitié REFUTED** (tests hermétiques convergence existent `dispatch_loop.rs:396/441/478`) ; anchor corrigé `engine/runtime.rs` |
| S1b deps | 0-dep | pins 0.98 intacts ; `=1.0.1` reste max_stable (docs/gossip/blobs 0.101/0.101/0.103) ; fix = logique in-crate | 13/15 CONFIRMED ; **fact9 REFUTED** (`nexus-coordinator-rs` ne déclare PAS iroh) ; gate advisories ROUGE (8 transitives hors iroh) = Phase G |
| S2 historique | reframe | blocker WAN jamais fermé ; b3 quorum PASS jamais existé ; **découverte : keepalive spawné one-shot au boot** sur task_docs de construction | 12/12 CONFIRMED contre source upstream ; **CONFLATION corrigée** : RE-DRIVE-ON-INGEST ≠ task-delivery |
| S3 threat | feu vert conditionné | §15.3 couvre déjà le chemin (0 frontière admission neuve) ; artefact = policy-conflict gitignore ; `*.redb` non couvert | CONFIRMED ; correction : path `diagnosis` ABSOLU (fuite pire) ; fix-direction pas encore réglée |
| S4 wire | 0-bump ATTEIGNABLE | inventaire wire figé ; aucune struct task ne porte de champ delivery ; S77 = preuve 0-wire | **REFUTED la reco « symmetric coordinator keepalive »** (mécanisme mal-décrit) ; **is_syncing NON flippé par le dial entrant** (source-refuted) ; fix = UNCERTAIN → mesurer d'abord |

Convergence des 5 adversariaux : la prémisse plan « fix WAN (root-cause S77) » est
imprécise ; observer-avant-coder ; split A3a/A3b ; ne jamais ré-implémenter le keepalive ;
delta `+1..4` peut être plus proche de `+1..2` (test coordinateur) que d'un gros fix.

## 5. Contre-vérification main thread (adversariale, de première main)

Contre `iroh-docs-0.98.0` (registre cargo) + call-graph SBFB, lus moi-même :

1. `engine/state.rs:60` `is_syncing = self.0.contains_key(namespace)` ; `:65` `insert`
   (seul peuplement) ; `:72-82` `start_connect` → `entry()` None → `abort connect:
   namespace is not in sync set` → false — CONFIRMÉ.
2. `engine/live.rs:414` `self.state.insert(namespace)` DANS `start_sync` (`:409` gardé
   `if !is_syncing`) ; `:714` broadcast `LocalInsert` gaté `if is_syncing` ; `:366`
   `sync_with_peer` → `if !start_connect { return }` — CONFIRMÉ.
3. `engine/state.rs:90-100` `accept_request` → `entry()` None → `Reject(NotFound)`
   (`:97`) — un coordinateur hors sync-set REJETTE le sync entrant — CONFIRMÉ.
4. `api/actor.rs:407` `doc_share` appelle `self.start_sync(doc_id, vec![])` — **le
   coordinateur entre son sync-set via `share`, pas via le dial entrant** ; le
   doc-comment `doc_sync.rs:20-22` (« relies entirely on the worker's incoming dial to
   flip is_syncing ») est IMPRÉCIS — CONFIRMÉ.
5. `nexus-shell-daemon/src/runtime.rs:643-666` : project doc ouvert par
   `open_doc`/`create_doc`, **aucun `start_sync`/`share` au boot** ; `docs.rs:159-166`
   `open_doc` = pur open, `:106-113` `create_doc` = pur create — CONFIRMÉ.
6. Grep prod `\.start_sync(` sur `crates/` = **1 seul hit non-test** : `doc_sync.rs:143`
   (keepalive worker) — 0 côté coordinateur — CONFIRMÉ.
7. `nexus-worker-core/src/engine/runtime.rs:736-749` : keepalives spawnés **one-shot** au
   boot sur `self.task_docs` (peuplé à la construction `:352-376`) ; un doc importé
   APRÈS `run_until_shutdown` n'en reçoit pas — fenêtre-boot analogue — CONFIRMÉ.
8. `dispatch_loop.rs:396/441/478` tests hermétiques convergence task: existent (le happy
   path est couvert ; le mode restart-coordinateur ne l'est pas) — CONFIRMÉ.
9. `.gitignore:148/149/150` raw b3 + worker.log ignorés ; `:4` `data/` ; `:5` `*.db` ne
   matche PAS `.redb` (vérifié `git check-ignore foo.redb` = non ignoré) ; T2 committé =
   `.planning/archive/v2.1/sprint80_t2_acceptance.json` verdict-only — CONFIRMÉ.

## 6. Plan de tests (delta cible A3b +1..2 solides)

1. `coordinator_reentering_sync_set_delivers_incremental_after_reopen` (A3b, hermétique) :
   node A `share_write`+`set task:1` converge (baseline) → shutdown A → boot A' même store
   → `open_doc` SANS share → `set task:2` → **CONTROL : ne converge pas** (reproduit
   `recv:0` sur restart) → boot `start_sync` appliqué → converge. Harness `create_node()`
   in-process (PAS `multi_daemon` networked), `#[tokio::test(multi_thread)]`. Précédents :
   `dispatch_loop.rs:396` + `boot_*_namespace_persistent_reopen` (`runtime.rs:4127/4139`).
2. (best-effort) `coordinator_reject_becomes_accept_after_boot_start_sync` — assert
   qu'un sync worker rejeté (NotFound) est accepté après le boot `start_sync`. Si le
   harness ne peut pas observer le reject/accept proprement, documenter le gap plutôt que
   flaky.

A3a n'ajoute **aucun** test hermétique (baseline live) ; le harness gagne le check
préflight PROJECT_ID (confondeur `b3:26-30`).

## 7. Risques

- **META (dominant, précédent A/A2)** : ne pas coder à l'aveugle. Le keepalive worker
  existe ; réconcilier avec `36cf1cc` avant tout Edit ; le fix A3b est côté COORDINATEUR,
  complémentaire, jamais un revert/duplicata du keepalive.
- **Suffisance live non prouvée** : la root-cause coordinateur est code-évidente mais son
  efficacité WAN dépend du run A3a. Il subsiste des facettes purement live (NAT rebind,
  relay change, stale ticket addr) que le keepalive worker cible et que seul le rig peut
  départager. A3a mesure ; A3b prouve hermétiquement le mode restart ; le rig confirme.
- **Dépendance rig dure** : A3a (VPS + PC 5080 + Mac + Ollama) ne se joue pas sans le rig.
  Ollama-Mac absent bloque le palier 2 quoi qu'il arrive côté delivery.
- **Baseline HONNÊTE = BLOCK attendu** : ne pas maquiller un BLOCK{delivery} en RIG-ABSENT
  si le rig transport est présent. Le confondeur PROJECT_ID DOIT être écarté d'abord.
- **Artefact committé = policy-conflict** : `.gitignore:143-153` traite le raw b3 comme
  secret-bearing → committer un artefact CURATED verdict-only sous `.planning/`, jamais
  le raw. `*.redb` à ajouter au gitignore.
- **Surface migrée par le bump** : le mécanisme `is_syncing`/`start_sync`/broadcast est
  vérifié 0.98 SEULEMENT. Toute logique A3b le concernant se re-valide contre iroh-docs
  0.101 au bump (Phase B/C) — même discipline que le matcher « Replica not found » (A2).
- **Delta-tests** : `+1..2` réaliste (test coordinateur). NE PAS inventer de tests pour
  atteindre `+4` ; risque de faux-vert (les convergence tests existent déjà).

## 8. Wire check (0-bump + test-acteur §6.12)

0-bump CONFIRMÉ : A3b = un `start_sync(vec![])` au boot (control-flow transport) +
logs + test ; A3a = harness shell + artefact JSON planning. Aucune sérialisation/JCS ;
`task:` reste clé document ; `TASK_FORMAT_VERSION=1`, `DOMAIN_TASK/RESULT/CLAIM_V1`,
`FEED_FORMAT_VERSION=1`, ALPN `sbfb/seed/0`+`sbfb/shard/1`, format string `DocTicket`
inchangés. **Test-acteur** : l'artefact b3 baseline n'est PAS une frontière docs-contrat/
LLM — lecteurs = un humain (`verification.md`) + l'audit gate + le harness lui-même ;
aucun runtime distinct ne le parse comme contrat d'interop. C'est un artefact de test
committé (tranche test-acteur = revieweur humain), pas une API loopback lue par un
runtime tiers → aucune étiquette requise.

## 9. Commit shapes

- **A3a** : `chore(acceptance): Sprint 81 Phase A3a — baseline transport b3 LIVE 0.98
  committée + ressources rig (0-bump)` — body : run palier 1 HEAD, confondeur PROJECT_ID
  écarté, verdict baseline (BLOCK{delivery} attendu), artefact curated `.planning/`,
  Ollama-Mac, store VPS `data/`, `*.redb` gitignore, run `SBFB_INTEGRATION=1` archivé.
- **A3b** : `fix(daemon): Sprint 81 Phase A3b — coordinateur re-entre son sync-set
  iroh-docs au boot du project doc (0-bump)` — body : root-cause (open_doc/create_doc ne
  start_sync pas ; `share` transitoire ; reject NotFound sur restart), fix boot
  `start_sync(vec![])`, test hermétique red→green mode-restart, différentiel b3 BLOCK→PASS,
  keepalive worker inchangé/complémentaire, 0-bump + garde-fous.

## 10. Carries (hors scope A3, tracés)

1. **Phase B/C** — re-calibrer le raisonnement `is_syncing`/`start_sync`/broadcast/accept
   contre **iroh-docs 0.101** au bump (le fix A3b en dépend ; analogue au matcher
   « Replica not found » A2). Veille wire pré-1.0.
2. **Phase G** — (a) nommer la classe « perte silencieuse warn-only » dans THREAT_MODEL
   (carry A2, déjà routé) ; (b) note d'amplification §15.3 pour tout `start_sync`
   coordinateur avec peers non-vides (ici `vec![]` = pas de dial) ; (c) **re-jouer
   `cargo deny check advisories` APRÈS le bump** — 8 advisories transitives ROUGE
   aujourd'hui (anyhow/hickory/quick-xml/rustls-webpki, hors iroh ; exposition SBFB
   nulle : 0 `downcast_mut`), la pile DNS/TLS/XML bouge au bump ; entrée `deny.toml`
   ignore `RUSTSEC-2026-0097` périmée à nettoyer.
3. **Phase K** — libellé T1 honnête (relay-gated early-return silencieux) ; le test
   hermétique mode-restart A3b alimente T1 sous-test (1) convergence.
4. **Dette / hors A3** — fenêtre-boot du keepalive worker (spawné one-shot au boot sur
   `task_docs` de construction ; docs importés mid-run non couverts, `runtime.rs:736-749`)
   → dette convergence post-launch (fold possible dans A3b si trivial, sinon tracer).
5. **Distincts, NON A3** (ne PAS conflater) — RE-DRIVE-ON-INGEST (boot-seed-driver
   one-shot), SeedAnnounced peer_count:0, seeder catalog_len:0, PULL-3 cross-tier :
   chemin DÉCOUVERTE/SEED, pas task-delivery (carries S78/S75 existants).
