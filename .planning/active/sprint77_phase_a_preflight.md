# Sprint 77 — Phase A — Preflight G8 (Convergence delivery WAN)

> Prérequis DUR de tout le sharding S77. Approche rouge-d'abord puis fork (D1).
> Généré par la synthèse adversariale des 5 scans factuels (S1a OSS iroh-live-sync,
> S1b deps, S2 historique, S3 threat, S4 wire) + relecture ligne-par-ligne de la source
> iroh-docs/iroh-gossip 0.98 installée (`~/.cargo/.../iroh-docs-0.98.0`).
> Orchestration Workflow ultracode : 5 scans parallèles + synthèse adversariale (6 agents,
> ~1.09M tokens). Run `wf_73733ee6-2c0`.

## Verdict

**PLAN-ADAPT.**

Le plan §4 Phase A tient intégralement (rouge-d'abord 2-nœuds, diagnostic-puis-fork, fix
câblage OU BLOCK-PO, 0 bump wire) et reste dans le scope. Mais l'**approche concrète doit
dévier** sur deux points établis par l'OSS :

1. **L'hypothèse d'entrée « le worker ne subscribe jamais, donc poll = pas de live-sync »
   est RÉFUTÉE.** `import(ticket)` appelle déjà `doc.start_sync(nodes)` en interne
   (`iroh-docs api.rs:220-224`), et `start_sync` ouvre le replica avec
   `.sync().subscribe(replica_events_tx)` **et** insère le namespace dans le sync-set
   (`live.rs:409-414`). L'app-facing `doc.subscribe()` n'ajoute qu'un consommateur de
   stream (`live.rs:334-341` — n'insère PAS le namespace). **« Ajouter subscribe côté
   worker » NE corrige donc PAS la convergence** ; ce serait fixer le mauvais maillon.

2. **La cause-racine réelle est la non-formation / non-maintien du voisinage gossip** du
   namespace entre worker et coordinateur sur transport réel. Le broadcast incrémental
   `LocalInsert → gossip.broadcast` du coordinateur est gaté par `is_syncing` (`live.rs:714`).

Le plan nommait ce candidat (« (c) gossip topic du doc non joint ») sans nommer la primitive
0.98 corrective ni le fait que subscribe est un faux levier. Le code suit l'approche corrigée
(observer NeighborUp/SyncFinished + re-former/maintenir le join gossip via `start_sync` +
ré-résolution pkarr). **Day-0 (iroh 0.98 pinné) intact, band-aids D1 respectés, 0 bump wire.**

**DESIGN-CONFLICT exclu** : l'exemple doré `two_nodes_docs_sync.rs` converge réellement en
post-subscribe incrémental sous 0.98 même-process (S1a l'a exécuté, `=== SYNC OK ===`,
EXIT=0). La convergence n'est pas intrinsèquement cassée — la cause est câblage/transport.

## Cause-racine (ranking)

### H1 (PRINCIPALE, likelihood haute, fixable 0.98) — Voisinage gossip non formé/non maintenu

Le coordinateur ouvre le `project_doc` via `create_doc`/`open_doc`
(`runtime.rs:642-659`, `OpenOpts::default` sync=false, `actor.rs:240-249`) et ne fait que
`subscribe()` (`result_sync.rs:192`) — **jamais `start_sync`**. Le broadcast du `task:`
incrémental est gaté par `is_syncing(namespace)` (`live.rs:711-718`), et `is_syncing` n'est
inséré QUE par `start_sync` (`live.rs:409-414`). Côté coordinateur, `is_syncing` ne devient
vrai que lorsqu'un neighbor gossip se forme et déclenche `sync_with_peer` / un accept réussi.

Le worker (`runtime.rs:350`) fait `import_ticket` (= `import` = `start_sync`) une seule fois
au boot : il joint le gossip avec le coordinateur en bootstrap (`gossip.rs:45-75`
`subscribe_with_opts(JoinOptions::with_bootstrap(peer_ids))`) en seedant `memory_lookup`
avec les **adresses figées du ticket** (`live.rs:464-487`). Si le worker ne peut pas
dialer/maintenir le coordinateur (adresses ticket périmées au reboot, NAT rebind, relay
change, hole-punch manqué, swap binaire sur état persistant), le swarm gossip du namespace
reste vide : le coordinateur ne broadcaste rien (`is_syncing=false`) et le worker ne reçoit
aucun `Op::Put` incrémental. Seule la sync bulk initiale (au moment de l'import) livre des
données ⇒ exactement **« recv:0, gossip neighborhood non formé »**, reproduit LAN+WAN
(`verification.md §5.1`).

Pourquoi l'exemple marche : node B (`import` → `start_sync` → `join_peers` → `gossip.join`)
dial A ; A reçoit le neighbor entrant → `gossip.progress()` émet NeighborUp (`live.rs:305-307`)
→ A `sync_with_peer` → A ouvre son replica avec sync, insère le namespace → A devient
`is_syncing` → les `LocalInsert` ultérieurs de A sont broadcastés. En même-process LAN le
neighbor est trivial ; en WAN/NAT réel il ne se forme/maintient pas.

### H2 (CONTRIBUTRICE, corrélée H1, likelihood moyenne, fixable 0.98) — `is_syncing` faux côté coordinateur au write

`accept_request` retourne `Reject(NotFound)` si le namespace n'est pas dans le sync-set
(`state.rs:90-100` + `state.rs:154-158` : `entry()` → `None` si non-syncing, doc-comment
« If the namespace is not syncing return None »). Sur PULL, le coordinateur ne connaît pas a
priori l'EndpointId du worker → il ne peut pas `start_sync` proactivement vers lui ; il dépend
entièrement de la formation du neighbor déclenchée par le dial entrant du worker. **Subsumé
par H1.** ⚠️ Faux-vert : forcer `is_syncing` côté coordinateur (ex. `start_sync` au boot sans
pair) pourrait masquer H1 en LAN tout en laissant le WAN cassé.

### H3 (AMPLIFICATEUR, likelihood moyenne, fixable 0.98) — Pas de gate readiness, pas de re-join, état persistant

`import` (start_sync/join) est appelé **une seule fois** au boot worker (`runtime.rs:350`),
sans attente de NeighborUp ni re-bootstrap du gossip du doc de tâches. `joined()` /
`subscribe_and_join` (iroh-gossip) existent mais grep SBFB = 0. La boucle reconnect de
`result_sync` porte sur `subscribe()` (events), pas sur le join gossip. Le worker utilise un
**FsStore persistant** (vs MemStore exemple/test) : `start_sync` recharge `get_sync_peers`
(`live.rs:416-441`, peers possiblement périmés). Le swap binaire S75→S76 à chaud sur
`docs.redb` aggrave (connexion sync morte non ré-établie).

### H4 (RÉFUTÉE, likelihood faible) — Le polling au lieu de subscribe est la cause

**Hypothèse d'entrée du brief + S2/S4 primaires, RÉFUTÉE par l'OSS.** `import(ticket)`
appelle `start_sync(nodes)` (`api.rs:220-224`) ; `start_sync` ouvre le replica avec
`.sync().subscribe(replica_events_tx)` (`live.rs:409-414`) — le worker EST déjà abonné aux
RemoteInsert au niveau engine, indépendamment de l'app-facing `doc.subscribe()`.
`ToLiveActor::Subscribe` (`live.rs:334-341`) n'ajoute qu'un consommateur et N'INSERE PAS le
namespace. Le poll `get_many_by_prefix` lit le replica local, tenu à jour par la sync gossip ;
le mode de lecture n'est pas le problème. Le test hermétique `worker_result_syncs`
(worker poll-only, sans subscribe) PASSE — preuve que poll-sans-subscribe suffit quand le
neighbor est vivant.

### H5 (PEU PROBABLE, à n'invoquer qu'en dernier ressort) — Convergence intrinsèquement cassée en 0.98 (DESIGN-CONFLICT)

**CONTRE :** l'exemple doré converge réellement en 0.98 (S1a exécuté : `node B observed
InsertRemote` + `=== SYNC OK ===`, EXIT=0), write incrémental post-subscribe inclus ; les 2
tests cross-node hermétiques always-on passent (1804 Win/1808 Docker) ; le changelog
0.100/iroh-1.0-rc ne contient aucun fix live-sync ciblé. À ne conclure QUE si le red-test avec
VRAI discovery reste rouge après fix de re-join explicite via l'API publique.

## Red test recommandé

`convergence_incremental_task_reaches_remote_replica` (`crates/nexus-shell-daemon/src`,
module test 2-nœuds, modèle `result_sync.rs:394` + exemple `two_nodes_docs_sync.rs:146-189`).

**Différence cardinale vs tous les tests cross-node existants** : écrire `task:` **APRÈS**
que le receveur (worker) a importé+joint, pour exercer la livraison LIVE incrémentale (pas la
sync bulk initiale — tous les tests actuels écrivent AVANT le boot worker, d'où « vert en
test, rouge en live »).

Séquence : (1) A crée le `project_doc` (`create_doc` + `share_write`) ; (2) B `import_ticket`
(= start_sync, join gossip) ; (3) **attendre la formation du neighbor** (observer
`LiveEvent::NeighborUp` via `doc.subscribe` côté B, budget borné — PAS un sleep aveugle) ;
(4) A écrit `task:{id}` via `dispatch_loop::run` (le VRAI writer, pas un `doc.set` à la main) ;
(5) assert B voit l'entrée (`get_many_by_prefix(b"task:")` ou InsertRemote) < budget.

Activer TRACE `iroh_docs::engine::live` + `iroh_gossip::net` (NeighborUp/Down/SyncFinished
sent/recv) pour capturer le maillon réel. **Discrimination rouge-d'abord** : sans le fix de
re-join le test échoue (recv:0, pas de NeighborUp maintenu) ; il doit DISTINGUER
subscribe-vs-poll (le worker reste poll-based pour le claim) pour ne pas valider par accident
le faux levier H4. Accompagnement : `convergence_boot_catchup_still_works` (non-régression
bulk) + `convergence_remote_write_visible_to_local_subscriber` (symétrie result: inverse).

⚠️ **Limite hermétique** : un test in-process même-machine garde la connexion triviale et ne
reproduit PAS un drop WAN/NAT. Le critère d'acceptation Phase A inclut donc aussi le harness
`b3` stage≥claim cross-machine (T2, §4.4), pas seulement le test Rust vert.

## Fix path

Fix câblage SBFB **côté worker** (`crates/nexus-worker-core/src/engine/runtime.rs`),
0 bump wire, dans l'API publique iroh-docs 0.98 — **pas de fork iroh nécessaire** pour la
convergence (le fork D1 reste disponible mais non requis) :

1. Sur le doc de tâches importé, ouvrir un `doc.subscribe()` dédié à l'**OBSERVABILITE** du
   voisinage (NeighborUp/NeighborDown/SyncFinished) — sans changer le mode de lecture poll
   du claim.
2. **Maintenir/re-former le join gossip** : sur NeighborDown ou absence de neighbor au-delà
   d'un budget, re-déclencher `Doc::start_sync(vec![coordinator_addr])` en **RE-RESOLVANT**
   l'EndpointAddr courant du coordinateur via pkarr (presets::N0 publie+résout déjà) plutôt
   que de rester figé sur les adresses du ticket (`live.rs:464-487`).
3. Borner par backoff (miroir `result_sync.rs:187-217`).

**Discrimination par le red-test** : rouge SANS re-join, vert AVEC re-join+pkarr ⇒ H1/H3
confirmés. Rouge même avec re-join explicite via `start_sync` adresse fraîche ⇒ H5/BLOCK-PO.

**Garde-fous (S3/S4)** : ne JAMAIS toucher la clé doc prefixée `task:` (contrat lecteur B-1
S71) ni les bytes canonical (signatures Track I S76) ; fix strictement sur la session
sync/join. Préserver subscribed-only + cap 16 (`node.rs:68`). Le drain du stream LiveEvent ne
doit pas faire backpressure (hot-path boot P54) : consommer en best-effort, le claim reste
poll-based.

## BLOCK-PO trigger

Le test rouge-d'abord avec **VRAI discovery iroh** (pkarr/relay, pas un handshake in-process
pré-partagé) reste ROUGE (B ne reçoit jamais l'entrée `task:` incrémentale, recv:0, NeighborUp
jamais émis pour le namespace) APRES application du fix câblage maximal réalisable via l'API
publique 0.98 — c.-à-d. après (a) re-join explicite via `Doc::start_sync(adresse
coordinateur re-résolue pkarr)`, (b) absence persistante de NeighborUp/SyncFinished dans les
logs TRACE malgré un dial réussi, et (c) confirmation que l'exemple doré converge alors que le
même pattern d'API échoue sur transport réel.

**Observation requise pour conclure** : NeighborUp absent OU SyncFinished recv:0 répété
malgré join explicite + adresse fraîche joignable, prouvant que la primitive gossip 0.98 ne
forme/maintient pas le voisinage du namespace en WAN/NAT réel.

**Action** : STOP + `pivot_proposal` + arbitrage PO ; NE PAS masquer le fork ni forcer un
upgrade iroh 1.0 (hors Day-0). Probabilité jugée FAIBLE (H5).

## Scope & wire

- **Scope** : Phase A reste strictement dans §4 (diagnostic-puis-fork). Les 3 band-aids
  rejetés D1 sont respectés et NON proposés : (a) poll au lieu de subscribe — non proposé
  (le fix n'EST PAS subscribe ; subscribe = observabilité, le poll du claim reste) ;
  (b) canal HTTP push parallèle — non proposé ; (c) relais N0 hot-path — non proposé
  (le re-join via `start_sync` utilise la résolution pkarr/relay native d'iroh, pas un relais
  N0 inséré). Aucun SCOPE-CUT.
- **Wire** : **0 bump confirmé** (S4 high). La clé `task:` est une clé de DOCUMENT (doc.set =
  set_bytes), hors bytes canonical/signature (`dispatch_loop.rs:41-49` ;
  `canonical.rs:74,77,80` DOMAIN_TASK/RESULT/CLAIM_V1). Le fix touche uniquement la session
  sync/join (transport) : aucune structure d'enveloppe FeedEntry, aucune nouvelle op feed
  (pas de bump FEED_FORMAT_VERSION — politique pre-launch raw-op), aucune version wire figée
  touchée (TASK/FEED/PROJECT_ANNOUNCEMENT/ANNOUNCEMENT/SEED/CURATOR_LIST/NODE_DIRECTORY/POW/
  KEY_ROTATION/TASK_RESPONSE = 1 ; INVITE=2 pré-existant inchangé). `start_sync`/`subscribe`/
  `get_sync_peers` = primitives 0.98 déjà exposées ; aucune nouvelle dépendance, aucun fork.

## Risques

1. **Faux-vert hermétique** : les tests 2-nœuds tournent en même-process LAN avec connexion
   triviale — ils ne reproduisent PAS un drop WAN/NAT. Vérifier le red-test RÉELLEMENT rouge
   avant le fix (revert prouve rouge) ET la convergence cross-machine (b3 stage≥claim).
2. **Masquer H1 par H2** : un fix qui force `is_syncing` côté coordinateur peut passer en LAN
   en laissant le WAN cassé. Le levier propre reste côté worker (maintenir le neighbor).
3. **Confondeurs du live attempt** : binaire swap à chaud sur état persistant + tâches stale
   S75 (signature-invalides sous worker S76, Track I). Repartir de homes worker+coordinateur
   FRAIS (pas de `docs.redb` hérité) avant de conclure que le bug persiste après fix.
4. **Diagnostic S76 non capturé en TRACE** : « gossip neighborhood non formé » cohérent avec
   H1 mais jamais instrumenté NeighborUp/Down/SyncFinished. Phase A doit d'abord reproduire
   AVEC logs TRACE iroh_docs::engine::live + iroh_gossip::net, sinon risque de fixer le
   mauvais maillon.
5. **Backpressure stream** : le `subscribe()` d'observabilité ajoute un stream LiveEvent à
   drainer ; le drop ferme la session — consommer best-effort, ne pas bloquer le hot-path
   boot (P54).
6. **Store persistant** : le red-test doit refléter le FsStore si on veut reproduire le cas
   reboot-coordinateur (adresses ticket périmées + peers-per-doc stale).

## Scans (synthèse par scan)

- **S1a (OSS iroh-live-sync)** — Scan le plus probant, exécution réelle de l'exemple inclus.
  Établit que `import` appelle `start_sync` (`api.rs:220-224`), que `is_syncing` gate le
  broadcast (`live.rs:710-718`) et n'est inséré que par `start_sync` (`live.rs:414`), que
  l'exemple PASSE (EXIT=0). Refute H4. H1 principale. **Confirmé par relecture** :
  `start_sync` ouvre `.sync().subscribe(replica_events_tx)` (engine-level, pas app-facing),
  `ToLiveActor::Subscribe` n'insère pas le namespace, NeighborUp → sync_with_peer, accept
  rejette NotFound si non-syncing (`state.rs:154-158`).
- **S1b (deps)** — Confirme `start_sync`/`get_sync_peers`/`set_download_policy` exposés en
  0.98, `fixable_in_0_98=yes`, pas de BLOCK iroh 1.0. Note le gap : aucun test ne couvre la
  convergence live incrémentale (T2 gate S77). Aligné.
- **S2 (historique)** — Documente le bug live 2026-06-19 (verification.md §5.1) et l'asymétrie
  coordinateur-subscribe / worker-poll. **Hypothèse primaire « worker ne subscribe pas » à
  NUANCER** : la relecture OSS montre que subscribe n'est PAS le levier (import fait déjà
  start_sync) ; le vrai différentiel est le maintien du voisinage gossip, que S2 cite
  justement en H2 (connexion non maintenue après bulk). Risk_flags pertinents conservés
  (faux-vert hermétique, confondeurs swap+stale, homes frais).
- **S3 (threat)** — Aucune surface sécurité nouvelle. Frontière d'admission = DocTicket
  write-capable minté par l'invite loopback authentifié (`invite_api.rs:78-100`), inchangée.
  Validator unique acceptation result, dedup+quorum inchangés. **Préserver subscribed-only +
  cap 16** (`node.rs:68`) — garde-fou intégré au fix path.
- **S4 (wire)** — Confirme 0 bump wire (high) : clé `task:` = clé doc hors canonical
  (`dispatch_loop.rs:41-49`, `canonical.rs:74-80`), versions figées à 1 (INVITE=2
  pré-existant). Identifie correctement que le worker ne subscribe pas + que le test cross-node
  ne couvre que result B→A (pas task A→B incrémentale). Hypothèse « import_and_subscribe » à
  nuancer de la même manière que S2 (subscribe = observabilité, pas le levier de convergence).
  Garde-fous wire conservés (ne pas toucher clé task: ni canonical).
