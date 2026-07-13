# Sprint 82 — Phase A preflight (G8)

Phase : A « Convergence cold-boot catch-up » (escalade boot-SEED,
carry P1 S81-G-ESC-1, OVERDUE 3/3, GATE PLEIN, PO-1=B).
Invariant unifié Day-0 D2 (GELÉ) : « le broadcast gossip est un HINT
non fiable ; l'état durable synchronisé est la VÉRITÉ ; tout
consommateur cold-boot RÉCONCILIE contre cet état une fois le
neighborhood formé. »

## Verdict: PLAN-ADAPT

L'approche **ANCRE** (re-drive de `run_boot_seed_driver` à l'ingest
annuaire) est **saine et codable** telle quelle, sous 4 contraintes de
conception concrètes (débounce, ordre duress au wrapper, idempotence par
set pinné, sérialisation vs boot-driver en vol). Aucune décision Day-0 /
figée n'est violée (S2), 0 bump wire (S4), 0 dep (S1b), iroh reste
=1.0.1 (S1b).

L'approche **WORKER** telle que **littéralement rédigée** dans le
livrable (« réconciliation d'état forcée `start_sync(peers)` au
`run_until_shutdown` APRÈS import + keepalive borné ») est
**substantiellement DÉJÀ PRÉSENTE dans le code** et serait un **no-op
partiel** si implémentée naïvement : `import_ticket` appelle déjà
`doc.start_sync(ticket.nodes)` une fois au boot, et le keepalive S77
Phase A ré-émet DÉJÀ `start_sync` immédiatement au 1er tick de
`run_until_shutdown` (via `last_rejoin` pré-vieilli + `interval` premier
tick immédiat) puis en backstop toutes les 15 s tant que le neighbor est
absent. Le gap live S81-K est RÉEL mais son vrai levier est la **latence
de formation du neighborhood au cold-boot**, pas un `start_sync`
manquant. La Phase A doit donc **RENFORCER** la convergence cold-boot,
pas ré-ajouter un `start_sync` déjà émis. → coder, mais adapter
l'approche WORKER (voir « Approche validée »).

Sévérité : PAS de DESIGN-CONFLICT — aucune violation Day-0/invariant
prouvée au code ; les 5 constats sont des risques de conception mitigés
+ une prémisse plan réfutée, ce qui est exactement PLAN-ADAPT.

---

## S1a — OSS prior-art (sémantique start_sync)

**Pattern état-de-l'art confirmé.** L'approche ANCRE+WORKER
(reconcilier idempotemment contre l'état durable quand un nœud a
rebooté / manqué un broadcast) correspond au textbook anti-entropy
(range-based set reconciliation d'iroh-docs : convergence en un seul
fingerprint si déjà en phase ; idempotence ⇒ convergence indépendante
de l'ordre/répétition). CONFIRMED.

**Sémantique `start_sync` — vérifiée au code réel :**

- **Wrapper thin, non-nouveau** : `crates/nexus-core-rs/src/docs.rs:410`
  `pub async fn start_sync(&self, peers: Vec<iroh::EndpointAddr>)`
  délègue à iroh-docs. Aucune dep, aucun wire. CONFIRMED.
- **Seul `start_sync` insère le namespace dans le sync-set** ;
  `open_doc`/`create_doc` NON (`docs.rs:388-402`, recalibré 0.101 à
  S81 Phase B). Un nœud hors sync-set (a) ne gossip-broadcast pas ses
  `LocalInsert` (gate `is_syncing`, `engine/live.rs:713`) et (b)
  REJETTE toute sync entrante `AbortReason::NotFound`
  (`engine/state.rs:97`). C'est la **racine du bug cold-boot** et ce
  qui rend le re-drive/reconcile NÉCESSAIRE (pas juste un HINT).
  CONFIRMED.
- **Fire-and-register non-bloquant + idempotent sur un doc déjà
  syncing** (`docs.rs:408-409` « Idempotent on an already-syncing
  doc » ; `doc_sync.rs:139-140`). L'appeler au `run_until_shutdown`
  après import ne bloque pas la boucle worker. CONFIRMED.
- **`start_sync(vec![])`** : l'appelant ne dial rien, MAIS iroh-docs
  merge les pairs PERSISTÉS du store (`register_useful_peer` /
  `get_sync_peers`) et les re-dial (`DirectJoin`), borné par
  `PEERS_PER_DOC_CACHE_SIZE = 5` (`docs.rs:404-408`). CONFIRMED.

**Prémisse plan WORKER — RÉFUTÉE au code :**

- `import_ticket` appelle `doc.start_sync(ticket.nodes)` **exactement
  une fois au boot** (`crates/nexus-core-rs/src/doc_sync.rs:23-24`,
  documentant `api.rs:220-225`). Le worker importe au build
  (`engine/runtime.rs:368` `docs_client.import_ticket(ticket)`), en
  capturant `peers = ticket.nodes.clone()` avant (`:367`). CONFIRMED.
- Le keepalive S77 Phase A **ré-émet déjà** `start_sync` : spawné par
  doc à `run_until_shutdown` (`engine/runtime.rs:742
  spawn_doc_sync_keepalive`, gardé `!peers.is_empty()` `:739-740`) ;
  `last_rejoin` pré-vieilli de `min_rejoin_interval`
  (`doc_sync.rs:190-192`) donc le **1er tick re-join immédiatement** ;
  `interval` premier tick immédiat (`doc_sync.rs:219-222`) ; backstop
  15 s si `neighbors.is_empty()` (`doc_sync.rs:258-264`) ; cooldown
  `min_rejoin_interval=5s` (`doc_sync.rs:148`). Donc au cold-boot le
  worker émet DÉJÀ `start_sync` immédiatement + toutes les 15 s tant
  qu'aucun neighbor. **REFUTED** (le livrable WORKER littéral duplique
  l'existant).
- Le module est **observability-only sur le read-path** : le claim
  reste poll-based (`get_many_by_prefix`), la subscription ne sert
  qu'à détecter un neighbor tombé (`doc_sync.rs:51-55`). Donc le scan
  `task:` re-court DÉJÀ à chaque tick (`engine/runtime.rs:959`), poll
  INCHANGÉ. CONFIRMED.

**Concern S1a #2 (ANCRE thundering-herd)** retenu : `run_boot_seed_driver`
est réseau-LOURD (`fetch_and_pin` / `fetch_and_pin_multi`,
acquisition+pin d'archives entières), bien plus coûteux qu'un
`start_sync`. Un re-drive-on-ingest sans débounce risque un burst de
pins sur rafale d'annonces. → contrainte de conception (voir Approche).

---

## S1b — Deps / CVE (preuve 0 dep, iroh =1.0.1)

**Invariant « 0 dep runtime ajoutée en Phase A » PROUVÉ.**

- Les deux crates touchés dépendent déjà de `nexus-core-rs`
  (`crates/nexus-shell-daemon/Cargo.toml:29`,
  `crates/nexus-worker-core/Cargo.toml:57`) ; le daemon dépend en plus
  de `nexus-worker-core` (`:31`). Threader/appeler `start_sync`,
  `get_many_by_prefix`, `run_boot_seed_driver` n'ajoute AUCUNE dep.
  CONFIRMED.
- `nexus-worker-core` n'a **aucune** dép iroh directe et n'importe
  aucun symbole `iroh::` : il consomme `EndpointAddr`/`start_sync` via
  re-exports `nexus-core-rs` (`engine/runtime.rs:49`). Appeler
  `start_sync` côté worker n'introduit ni dep ni feature-flag iroh.
  CONFIRMED.
- Aucune primitive visée n'active un feature-flag iroh non déjà
  présent (`nexus-shell-daemon/Cargo.toml` iroh `{ workspace = true }`
  sans `features=[...]`). CONFIRMED.
- **iroh reste STRICTEMENT =1.0.1** — `Cargo.toml` workspace :
  `iroh = "=1.0.1"`, `iroh-docs = "=0.101.0"`, `iroh-gossip =
  "=0.101.0"`, `iroh-blobs = "=0.103.0"`. Pins inchangés. CONFIRMED.
  (NB : le `CLAUDE.md` disant « iroh 0.98 » est STALE — S81 a bumpé ;
  la contrainte de tâche « iroh reste =1.0.1 » est la bonne.)
- `hickory-resolver 0.24` (4 RUSTSEC) = **Phase K** (PO-7=A), HORS
  scope A. La Phase A ne touche ni `Cargo.toml` ni hickory → aucun
  advisory nouveau. CONFIRMED / tracké.

---

## S2 — Décisions historiques

**Aucune décision FIGÉE (Day-0 / rejected / scope-cut) violée.** Le
re-drive-on-ingest et le catch-up worker sont des items **DEFERRED**
(carry S75/S76), jamais rejetés, et l'invariant D2 les gouverne
exactement. Les inexactitudes plan sont à réconcilier, pas des
violations. CONFIRMED.

**Driver ONE-SHOT — localisation RÉELLE (pointeur plan corrigé) :**

- Le plan cite `run_boot_seed_driver` à **`http.rs:1819-1826`** →
  **PÉRIMÉ** : `1818-1826` est la queue du doc-comment (paragraphe
  « Best-effort, ONE-SHOT … nothing re-drives until the next daemon
  restart … re-drive-on-ingest is a tracked S76 carry »). La signature
  RÉELLE est **`crates/nexus-shell-daemon/src/http.rs:1828`**
  `pub(crate) async fn run_boot_seed_driver(state, configured)`.
  Localiser par SYMBOLE, pas par ligne. CONFIRMED (re-vérifié).
- Le driver est **réellement ONE-SHOT** : `tokio::spawn` unique dans
  le bloc boot (`crates/nexus-shell-daemon/src/runtime.rs:1162-1192`),
  qui attend `boot_replay_done_rx` (`BOOT_DRIVER_REPLAY_WAIT_SECS`)
  puis appelle `reannounce_directory_at_boot` (`:1180`) +
  `run_boot_seed_driver(&boot_driver_state, &configured)` (`:1184`)
  UNE fois. La boucle gossip (`spawn_gossip_subscribe_task`,
  `runtime.rs:1541`) est une tâche SÉPARÉE qui ne re-drive rien.
  CONFIRMED.
- Décision ONE-SHOT = S75 Phase E (`1486fc9`), DEFERRAL explicite,
  jamais un rejet. Carry routé S76 puis S82
  (`sprint76_audit_plan.md`, « fenêtre morte 1er boot »). CONFIRMED.

**Escalade OVERDUE 3/3 :** S81-G-ESC-1 boot-SEED confirmée
(`sprint82_audit_plan.md:60-64`, routée par `e05338f + 50f05c1 +
8872596`). Règle §6.2.1 : **fermer dans S82 ou re-concevoir — plus
jamais de report sec** ; PO-1=B (`sprint82_kickoff.md:38`). Le fix code
seul (T1 hermétique vert) NE FERME PAS l'escalade sans le T2 LIVE (c)
PASS<30 s (rig Mac+PC+VPS) ; rig indispo ⇒ escalade PO explicite,
JAMAIS un 4e report sec. CONFIRMED.

**Évidence NEUVE S81-K (gap worker RÉEL malgré le mécanisme
existant) :** au run 2 du palier quorum, un worker démarré 3 s avant le
submit n'a JAMAIS reçu l'entrée `task:` incrémentale en 30 s (neighbor
pas formé au moment du broadcast) ; convergé une fois stable +2 min 08
(`sprint82_audit_plan.md:64-72`). Analyse mécanistique : le coordinateur
ouvre le doc via `create/open` et n'appelle jamais `start_sync`
(`doc_sync.rs:19-22`) ; son `is_syncing` ne passe true que lorsque le
dial ENTRANT du worker forme un neighbor ; il **ne re-broadcaste pas**
au NeighborUp tardif — il reconcilie par range-set. Le +2 min 08 = temps
que le `start_sync` re-dialé du worker (toutes les 15 s) + résolution
pkarr/relais mettent à former le neighbor. **Le bottleneck est la
latence de formation du neighbor, pas un `start_sync` absent.** →
corollaire : le vrai levier WORKER est un rejoin cold-boot plus
agressif/borné, PAS un re-scan `get_many_by_prefix` (le poll re-scanne
déjà chaque tick ; re-scanner est inutile tant que la réplique n'a pas
REÇU l'entrée).

**ANCRE = hook genuinement NEUF, non conflictuel :** driver one-shot
(`runtime.rs:1162`) et boucle gossip d'ingest (`:1541`) sont des tâches
distinctes ; `GossipTaskConfig` (`:1515-1536`) ne porte ni
`boot_driver_state` ni `keep_online_projects`. Threader est du code
neuf. CONFIRMED.

---

## S3 — Threat model

**ANCRE — surface d'amplification/DoS doublement gardée :**

- `run_boot_seed_driver` n'itère QUE `configured` (= `opts.seed.
  keep_online_projects`, accept-list opérateur), **jamais un pid venu
  du réseau** (`http.rs:1847 for pid in configured` ; signature
  `configured: &[String]` `:1830` ; call-site `runtime.rs:1163`). Un
  ingest annuaire fournit des pids réseau mais ne les injecte JAMAIS
  dans cette boucle. CONFIRMED.
- **Duress-gate réellement hérité** car EN TÊTE de la primitive
  elle-même (`http.rs:1840-1844`
  `if gossip_publish_in_duress(state.identity_mode) == Noop { return
  0; }`) — placé avant toute résolution/fetch/DB/log. Structurellement
  impossible à oublier à un nouveau call-site **si le wrapper appelle
  le driver comme PREMIÈRE action**. CONFIRMED.
- Ingest annuaire **subscription-gated** (S75 Phase C) : l'arm
  `Ok(entry)` de `handle_directory_announcement`
  (`runtime.rs:1999→2008`) ne s'atteint qu'après abonnement + Ed25519
  + attribution + anti-rollback (via
  `process_directory_announcement_bytes_throttled`,
  `iroh_runtime.rs` Step 4 NotSubscribed avant Step 5 fetch). Un pair
  non-abonné → `debug` drop (`runtime.rs:2015-2016`), jamais le
  re-drive. CONFIRMED.
- Content-addressing BLAKE3 = vérité : guard `h == want_hash` sur les
  2 chemins de fetch (`http.rs:1915` ticket, `:1955` multi-provider),
  `delete_tag` + skip si mismatch (`:1919`/`:1960`). Une annonce forgée
  ne sert jamais d'octets absents. CONFIRMED.

**WORKER — pas de nouvelle surface :**

- `start_sync(peers)` = adresses du ticket importé (`task_doc_peers`,
  capturé `engine/runtime.rs:367`), ensemble borné identique à celui
  que le keepalive S77 dial déjà. Docs sans peers = skip
  (`doc_sync.rs:180-183`). Keepalive throttlé
  (`min_rejoin_interval`) → pas de boucle. CONFIRMED.
- Une `task:` forgée est rejetée par `task_entry.verify_signature()`
  AVANT traitement (`engine/runtime.rs:1011-1014`) ; blob
  content-addressé (`content_hash` `:989`) ; dedup
  `completed_task_ids` + `task_already_handled_on_doc`
  (`:977-986`). La réconciliation accélère la convergence sans
  contourner aucune vérif ; scan `task:` INCHANGÉ. CONFIRMED.

**Résidus de conception à border (non-bloquants) :**

1. **Revision-churn** (ancre abonnée semi-fiable) : un abonné peut
   bumper la révision de son annuaire avec un NOUVEAU hash pour un pid
   configuré → chaque ingest déclenche un pull réel. L'idempotence
   `set_tag` (`http.rs:1892-1902`) borne le RE-PIN (blob déjà détenu,
   0 réseau) mais PAS le coût d'un nouveau hash légitimement advertisé.
   Le débounce/coalescing est donc obligatoire (voir Approche).
2. **Ordre du duress-gate au wrapper** (classe DURESS-BOOT-LEAK) : si
   le wrapper de re-drive lit `keep_online` / résout des pids / logge
   des `project_id` réels AVANT d'appeler le driver, ces lectures/logs
   s'exécutent sous duress et corrèlent l'identité leurre au vrai data
   root (le bug qui a forcé `reannounce_seeds_at_boot` à porter son
   PROPRE gate). Exiger : appel driver EN PREMIER, aucune pré-lecture.
3. **Course boot-driver ↔ re-drive** : partage d'`Arc` entre le
   one-shot boot driver encore en vol (`BOOT_DRIVER_REPLAY_WAIT_SECS`)
   et la boucle d'ingest. Le pin `set_tag` est idempotent, MAIS le
   garde anti-double-emit `was_already_announced`
   (`http.rs:1986/2000`, via la row `keep_online`) a une course
   lecture-avant-écriture : sérialiser via le mutex `coordinator_db`
   ou un flag de garde partagé pour ne pas double-annoncer.

Hors périmètre (non aggravé/mitigé) : une `task:` signée par une clé
coordinateur légitime mais sémantiquement abusive — défense = consent
worker-side L1-L4 + guardrail dispatch, inchangés.

---

## S4 — Wire invariants (0 bump _VERSION)

**Phase A ne touche AUCUN format wire ni canonical.** CLEAN.

- `boot_driver_state` = handle RUNTIME in-memory `Arc<DaemonHttpState>`
  (`runtime.rs:1068 Arc::clone(&http_state)`), PAS une structure
  sérialisée. CONFIRMED.
- `keep_online_projects` = état config local `Vec<String>`, sérialisé
  UNIQUEMENT dans `config.toml` `[seed]`
  (`nexus-shell-daemon-core/src/config.rs` `SeedConfig`), jamais sur le
  wire. CONFIRMED.
- Le re-drive ne réémet que des ops préexistantes : `SeedAnnounced` =
  variante typée de `PublicFeedOperation`, `FeedEntry.op =
  serde_json::Value` raw-op extensible, **0-bump `FEED_FORMAT_VERSION`**
  (S74 Phase F + pre-launch policy CLAUDE.md). CONFIRMED.
- WORKER : clé `task:` garde `format!("task:{task_id}")`
  (`dispatch_loop.rs:41` writer / `engine/runtime.rs:959,971` reader) ;
  poll `get_many_by_prefix(b"task:")` INCHANGÉ. CONFIRMED.
- `start_sync` = thin-wrapper transport iroh-docs, ni wire ni canonical
  (`docs.rs:410`). CONFIRMED.
- Aucune des ~15 constantes `*_VERSION` du workspace n'est mutée
  (`PROJECT_ANNOUNCEMENT_VERSION=1`, `FEED_FORMAT_VERSION=1`,
  `NODE_DIRECTORY_FORMAT_VERSION=1`, etc. — orthogonales au re-drive
  et au catch-up). CONFIRMED.

---

## Approche validée pour le code

### ANCRE (daemon) — re-drive-on-ingest idempotent, borné, duress-safe

1. **Threading** — étendre `struct GossipTaskConfig`
   (`crates/nexus-shell-daemon/src/runtime.rs:1515-1536`) avec :
   - `boot_driver_state: Arc<DaemonHttpState>` (clone de `http_state`,
     déjà disponible au boot en `:1068` sous le même nom) — porte
     node/curator_runtime/browse_aggregator/coordinator_db/
     seed_registry/feed_sync_state, tout ce dont le driver a besoin ;
   - `keep_online_projects: Vec<String>` (clone de
     `opts.seed.keep_online_projects`, déjà cloné en `:1163`).
   Câbler la construction du config au site d'appel de
   `spawn_gossip_subscribe_task` et le destructuring en `:1542-1558`.

2. **Point de re-drive** — dans le SEUL arm `Ok(entry)` de
   `handle_directory_announcement`
   (`crates/nexus-shell-daemon/src/runtime.rs:1999→2008`, l'arm succès
   subscription-gated + Ed25519 + anti-rollback). Passer
   `boot_driver_state` + `keep_online_projects` + l'état de débounce à
   `handle_directory_announcement` (signature actuelle `:1999-2003`
   `(curator_runtime, node, content)` — à étendre). NE PAS re-driver
   dans les arms d'erreur.

3. **Fonction** — appeler `crate::http::run_boot_seed_driver(&
   boot_driver_state, &keep_online_projects)`
   (`crates/nexus-shell-daemon/src/http.rs:1828`, PAS `1819-1826`),
   comme **PREMIÈRE action** du wrapper (le gate duress est en tête de
   la primitive `:1840` → hérité structurellement). AUCUNE pré-lecture
   `keep_online` / résolution de pid / log de `project_id` réel avant
   cet appel (classe DURESS-BOOT-LEAK, S3 #2).

4. **Débounce/coalescing (contrainte DURE, S1a-risk + S3 #1)** — la
   « borne 1 re-drive/batch » DOIT être un vrai COALESCING/cooldown,
   pas une simple anti-concurrence : cooldown temporel (analogue
   `min_rejoin_interval` de `doc_sync.rs`) OU garde par
   `(pid, resolved_hash)` déjà traité, pour qu'une rafale de N annonces
   de révision d'un abonné ⇒ au plus 1 acquisition par app manquante,
   pas N passes séquentielles de `run_boot_seed_driver` (chacune
   ré-itérant toute la liste configurée + `directory_snapshot`).

5. **Sérialisation vs boot-driver en vol (S3 #3)** — protéger le
   double-emit `SeedAnnounced` : le garde `was_already_announced`
   (`http.rs:1986/2000`) lit la row `keep_online` ; sérialiser
   boot-driver + re-drive via le mutex `coordinator_db` (ou un flag de
   garde partagé) pour éviter une double-annonce sur course
   lecture-avant-écriture. Le pin `set_tag` est déjà idempotent.

### WORKER (nexus-worker-core) — RE-SPÉCIFIÉ (le livrable littéral est un no-op)

Fichier réel : `crates/nexus-worker-core/src/engine/runtime.rs` (le
contexte de tâche disait `src/runtime.rs` — imprécis).

- **NE PAS** ajouter un `start_sync(peers)` « après import » : il est
  DÉJÀ émis (import_ticket `:368` une fois + keepalive
  `spawn_doc_sync_keepalive` `:742` immédiat au 1er tick + backstop
  15 s). Un ajout naïf = no-op partiel qui ne ferme pas S81-K.
- **NE PAS** ajouter un re-scan `get_many_by_prefix` post-NeighborUp :
  le poll re-scanne déjà chaque tick (`:959`) ; re-scanner est inutile
  tant que la réplique n'a pas REÇU l'entrée. Poll INCHANGÉ (contrat).
- **Vrai levier** (à concevoir au preflight code, grounded S81-K) :
  raccourcir la latence de formation du neighbor au cold-boot. Piste
  la plus directe et iroh-seule : un **schedule de rejoin cold-boot
  agressif borné** — `check_interval` / `min_rejoin_interval` serrés
  pendant une FENÊTRE INITIALE (jusqu'au 1er `NeighborUp`), puis
  relâchés vers le backstop S77 (15 s). Se paramètre via
  `KeepaliveConfig` (`doc_sync.rs:93-104`, déjà `Clone`) OU une variante
  de `spawn_doc_sync_keepalive` — 0 dep, 0 wire, primitive existante.
  Le mécanisme exact (variante keepalive vs config cold-boot) est un
  choix de conception du preflight code, à valider contre le budget
  T2 live (c) PASS<30 s.
- **Gate testabilité** : T1 hermétique 2-nœuds (rouge-avant-vert sur
  la convergence cold-boot, sur le modèle de
  `doc_sync.rs::keepalive_rejoins_doc_after_neighbor_loss`) + T2 JSON
  live PASS<30 s. Sans le T2 live, l'escalade §6.2.1 NE FERME PAS
  (PO-1=B) — rig indispo ⇒ escalade PO, jamais 4e report sec.

---

## Blockers / risques ouverts

- **PAS de blocker Day-0** — verdict PLAN-ADAPT, pas DESIGN-CONFLICT.
- **WORKER — risque dominant (no-op partiel)** : si le fix ré-ajoute
  un `start_sync` déjà présent, l'escalade OVERDUE 3/3 reste ouverte.
  Le delta doit RENFORCER la formation du neighbor cold-boot. Mécanisme
  exact = décision preflight code, à falsifier par T1 rouge-avant-vert
  + T2 live<30 s.
- **GATE PLEIN + PO-1=B** : la clôture §6.2.1 EXIGE la preuve LIVE (c)
  PASS<30 s (Mac+PC+VPS). Le code + T1 vert ne suffisent pas. Rig
  indisponible ⇒ escalade PO EXPLICITE (compteur 3/3 atteint, 4e report
  sec INTERDIT).
- **ANCRE — thundering-herd/revision-churn** : sans le débounce/
  coalescing du point 4, une rafale d'annonces d'un abonné = N passes
  réseau-lourdes (`fetch_and_pin_multi`) sur le hot-path d'ingest →
  régression latence/DoS. Contrainte DURE.
- **ANCRE — DURESS-BOOT-LEAK** : toute pré-lecture/log de données
  réelles avant l'appel driver fuit la corrélation decoy↔réel sous
  duress. Appel driver EN PREMIER, obligatoire.
- **ANCRE — course double-annonce** vs boot-driver en vol : sérialiser
  le garde `was_already_announced` (mutex `coordinator_db`).
- **Couplage A↔L** : le hook re-drive ajouté en Phase A sera absorbé
  par la décomposition `http.rs` de la Phase L — ne pas l'écraser au
  split refacto.
- **Trust boundary héritée (non-régression)** : le driver pinne le hash
  du PREMIER anchor lexicographique advertising (BLAKE3-seul, pas de
  provenance auteur à l'auto-seed, résidu Sybil-sampling S76,
  `http.rs:1863-1871`). Le re-drive ne doit pas élargir cette surface
  (même résolution, même gate).
- **PEERS_PER_DOC_CACHE_SIZE=5** (`docs.rs:408`) : plafond des pairs
  re-dialés sur `start_sync(vec![])`. Non-bloquant Phase A (le driver
  passe des addrs explicites ; le worker passe les peers du ticket),
  mais à garder en tête pour la convergence b3 si >5 seeders pertinents.
- **hickory-resolver 0.24** conserve 4 RUSTSEC ouverts jusqu'à Phase K
  (PO-7=A) — connu, tracké, HORS scope A, ne pas confondre avec une
  régression introduite par A.
