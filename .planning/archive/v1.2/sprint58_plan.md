# Sprint 58 — Plan

**Ecrit** : 2026-05-10.
**Tip d'entree** : `4cf8bba`.
**Phases** : A (MANDATORY), B (dette pair), C (AppStorage P2P
namespace), D (live events + sync E2E), E (wrap-up).

---

## Phase A — MANDATORY carries (JITTER-SCOPE + INVITE-U16-WIRE)

### §A Objectif

Fermer les 2 items 3/3 MANDATORY. Items petits, 1 test + 1 doc.

### §A Tasks

1. **Test jittered_republish_duration() bounds**
   - Fichier : `crates/nexus-shell-daemon/src/runtime.rs`
   - Ajouter `#[test] fn jitter_bounds_are_within_range()` dans
     le module `#[cfg(test)]`
   - Boucle 200 iterations, assert chaque duration ∈ [30, 60]
   - CLOSE P2-JITTER-SCOPE

2. **Doc PATTERNS.md §P47 INVITE_FORMAT_VERSION**
   - Fichier : `docs/rust/PATTERNS.md`
   - Section §P47 apres §P46
   - Contenu : historique rename S55 Phase D, u16 range, pre-launch
     policy (version = 2, pas de compat multi-version), post-v1.0
     (bump a chaque break, decoder range)
   - Cross-ref `crates/nexus-worker-core/src/invite.rs:73`
   - CLOSE P2-INVITE-U16-WIRE

### §A Verification

- `cargo nextest run -p nexus-shell-daemon --locked` vert
- `grep "§P47" docs/rust/PATTERNS.md` present

### §A Commit

```
feat(sprint58): Sprint 58 Phase A — MANDATORY carries JITTER-SCOPE + INVITE-U16-WIRE

CLOSE P2-JITTER-SCOPE (3/3 MANDATORY S55 Phase D) :
test jitter_bounds_are_within_range() dans runtime.rs, 200
iterations, asserts Duration ∈ [30s, 60s].

CLOSE P2-INVITE-U16-WIRE (3/3 MANDATORY S55 Phase D) :
§P47 dans PATTERNS.md — historique INVITE_VERSION →
INVITE_FORMAT_VERSION rename + u8→u16, pre-launch policy,
post-v1.0 compat.

Delta tests : +1 Rust (1232→1233).
Scope cuts : aucun.
```

---

## Phase B — Phase dette pair obligatoire

### §B Objectif

Sprint pair → phase dette (§6.2.1 Regle 1). 2 items P2.

### §B Tasks

1. **retain_recent timer dans gossip loop**
   - Fichier : `crates/nexus-shell-daemon/src/runtime.rs`
   - Dans le `tokio::select!` de `start_gossip_loop()` (ou
     equivalent), ajouter un bras `retain_interval.tick() =>`
     qui appelle `browse_limiter.retain_recent()`
   - `retain_interval = tokio::time::interval(Duration::from_secs(60))`
   - Ajouter 1 test verifiant que retain_recent ne panique pas
     apres N appels
   - CLOSE P2-RETAIN-RECENT

2. **scripts/sync-bridge-sdk.sh**
   - Fichier : `scripts/sync-bridge-sdk.sh` (NEW)
   - Copie `web/public/sbfb-bridge.js` vers chaque
     `examples/*/sbfb-bridge.js`
   - Post-copie : SHA256 check (toutes copies identiques)
   - Exit 1 si SHA256 diverge
   - Cross-platform : bash (WSL/Linux/Mac)
   - CLOSE P2-BRIDGE-SYNC

### §B Verification

- `cargo nextest run -p nexus-shell-daemon --locked` vert
- `bash scripts/sync-bridge-sdk.sh` exit 0

### §B Commit

```
feat(sprint58): Sprint 58 Phase B — dette pair retain_recent + bridge sync

CLOSE P2-RETAIN-RECENT (2/3 S56 audit) :
retain_recent() appele toutes les 60s dans la gossip select loop
via tokio::time::interval. Nettoie les entries expirees du
BrowseRequestLimiter.

CLOSE P2-BRIDGE-SYNC (1/3 S57 audit P2) :
scripts/sync-bridge-sdk.sh — copie web/public/sbfb-bridge.js
vers examples/*/ avec verification SHA256.

Delta tests : +1 Rust (1233→1234).
Scope cuts : aucun.
```

---

## Phase C — AppStorage P2P namespace + migration storage_api

### §C Objectif

Creer un namespace iroh-docs dedie au storage applicatif et
migrer storage_api.rs pour router les operations de sbfb-ideas
vers iroh-docs au lieu du HashMap+SQLite local.

### §C Tasks

1. **Namespace storage au boot**
   - Fichier : `crates/nexus-shell-daemon/src/runtime.rs`
   - Apres le project doc (runtime.rs:547), creer/ouvrir un
     2eme namespace dedie au storage
   - **PAS de `list_docs().first()`** : utiliser la table
     `storage_namespaces` (M8) pour retrouver le NamespaceId
     par nom d'app. Si absent, creer un nouveau namespace et
     persister l'ID dans la table.
   - Passer le `Doc` (Arc) dans `DaemonHttpState`

2. **DaemonHttpState enrichi — etat mutable**
   - Fichier : `crates/nexus-shell-daemon/src/http.rs`
   - L'etat storage doit etre mutable : POST /join installe un
     nouveau namespace APRES le demarrage du daemon. Un simple
     `Option<Arc<DocHandle>>` est trop statique.
   - Ajouter une structure `StorageNamespaceState { doc: Arc<DocHandle>,
     author: AuthorId, ticket: String, version: AtomicU64 }` et
     stocker `storage_namespaces: Arc<RwLock<HashMap<String,
     StorageNamespaceState>>>` dans `DaemonHttpState`.
   - Au boot, pre-remplir avec les namespaces connus de la table M8.
   - POST /join insere un nouveau namespace dans la map.

3. **Router storage_api.rs vers iroh-docs**
   - Fichier : `crates/nexus-shell-daemon/src/storage_api.rs`
   - **Semantique multi-auteur** : iroh-docs stocke des entries
     indexees par (author, key). Plusieurs auteurs sur la meme
     cle = plusieurs entries distinctes.
   - `storage_set` : si app repliquee (`sbfb-ideas`), ecrire
     via `doc.set(local_author, key, json)`. Chaque noeud ecrit
     sous son propre AuthorId. Sinon, HashMap+SQLite existant.
   - `storage_get` : si app repliquee, lire via
     `doc.get_many(Query::key_exact(key))` puis prendre l'entree
     la plus recente tous auteurs confondus (latest timestamp).
     **PAS `get_exact(local_author, key)`** — ca ne retournerait
     que l'entree du noeud local, pas celle d'un autre noeud.
   - `storage_list` : si app repliquee, lire via
     `doc.get_many(Query::key_prefix(prefix))`. Retourne entries
     de TOUS les auteurs (vue agregee multi-noeuds). Filtrer les
     tombstones (`{ "deleted": true }` / `{ "retracted": true }`)
     avant de retourner.
   - `storage_delete` : si app repliquee, ecrire tombstone
     `{ "deleted": true }` sous le local AuthorId. Seul l'auteur
     d'une entree peut la supprimer (per-author ownership).
   - Le HashMap+SQLite local reste le fallback pour les apps
     non repliquees.
   - **Schema app preservee** : le schema actuel de sbfb-ideas
     (`ideas/{uuid}` + `votes/{ideaId}/{pubkey}`) fonctionne
     avec iroh-docs. Le pubkey dans la cle de vote est une
     **cle applicative** (node_id daemon via identity_pubkey),
     distincte de l'AuthorId iroh-docs (identite docs). Les
     deux coexistent : AuthorId = dimension auteur iroh-docs,
     pubkey dans la cle = identifiant applicatif. Pas de
     migration d'app requise pour S58 MVP.
   - **Deduplication reads** : pour `storage_list("ideas/")`,
     grouper par key et garder latest-per-key (via
     `Query::single_latest_per_key().key_prefix(prefix)`) pour
     eviter les doublons multi-auteur sur les cles `ideas/*`.
     Pour `storage_list("votes/")`, retourner toutes les entries
     (chaque auteur = 1 vote distinct).

4. **Migration DB : table storage_namespaces (M8)**
   - Fichier : `crates/nexus-coordinator-rs/src/db.rs`
   - Table `storage_namespaces(app_name TEXT PRIMARY KEY,
     namespace_id BLOB NOT NULL, doc_ticket TEXT)`
   - Helpers : `get_storage_namespace(app_name)`,
     `set_storage_namespace(app_name, namespace_id, ticket)`

5. **Ticket Write generation + import (OBLIGATOIRE)**
   - Au boot, si le namespace est nouveau, generer un DocTicket
     Write via `doc.share_write()`
   - Serialiser le ticket dans la table storage_namespaces ET
     le rendre disponible via endpoint obligatoire
     `GET /api/daemon/storage/ticket/:app`
   - **Endpoint import** : `POST /api/daemon/storage/join` accepte
     un DocTicket serialise et appelle `import_and_subscribe()`.
     Daemon B utilise cet endpoint pour rejoindre le namespace
     de daemon A. Requis par le test E2E Phase D ET par le
     scenario production (nouveau noeud rejoint le reseau).
   - L'embed dans l'archive zip = etape manuelle S58 (le verified
     deploy automatique est S59)

6. **Tests Rust**
   - Test CRUD iroh-docs storage (set + get + list + delete/
     tombstone) via nexus-test-harness ou unit tests
   - Test `storage_get` multi-auteur : ecrire sous 2 AuthorIds,
     lire retourne la plus recente
   - Test tombstone filtering dans `storage_list`
   - Test routing : app repliquee → iroh-docs, app non repliquee
     → HashMap

### §C Verification

- `cargo nextest run --workspace --locked` vert
- `cargo clippy --workspace --all-targets --locked -- -D warnings`

### §C Commit

```
feat(sprint58): Sprint 58 Phase C — AppStorage P2P iroh-docs namespace + migration

Migration M8 : table storage_namespaces dans coordinator.db.
storage_api.rs route les operations de sbfb-ideas vers
iroh-docs namespace dedie. HashMap+SQLite reste le fallback
pour apps non repliquees.

Schema app preservee (ideas/{uuid} + votes/{ideaId}/{pubkey}),
compatible iroh-docs multi-auteur. Reads via get_many +
latest-per-key. Tombstone filtering sur list/get.

Ticket Write genere au boot + endpoint obligatoire
GET /api/daemon/storage/ticket/:app + import endpoint
POST /api/daemon/storage/join (mutation runtime).

Delta tests : +N Rust (1234→1234+N).
Scope cuts :
- Ticket embed dans archive zip = manuelle S58 (verified deploy S59)
- Generalisation manifest = S59+ (hardcode sbfb-ideas S58)
```

---

## Phase D — AppStorage P2P live events + sync test E2E

### §D Objectif

Rendre la replication effective : quand un noeud ecrit, les
autres noeuds recoivent l'update en temps reel.

### §D Tasks

1. **Subscribe InsertRemote sur storage namespace**
   - Fichier : `crates/nexus-shell-daemon/src/runtime.rs`
   - Apres le boot du storage namespace, appeler
     `doc.subscribe()` et spawn un task qui ecoute les
     `LiveEvent::InsertRemote`
   - Sur chaque InsertRemote, emettre un event interne
     (pas de cache HashMap local pour les apps repliquees —
     iroh-docs EST le store, les reads passent par le Doc)

2. **Notification live : polling endpoint MVP**
   - **Pourquoi pas SSE** : l'iframe a `connect-src 'none'`
     (CSP sandbox). L'iframe ne peut pas ouvrir de connexion
     HTTP/SSE vers le daemon. Mais le shell React (host) peut.
     Un futur SSE passerait par : daemon SSE → shell React →
     postMessage → iframe. Pour S58, polling est pragmatique.
   - **Endpoint daemon** : `GET /api/daemon/storage/:app/version`
     retourne un compteur atomique `AtomicU64` incremente a
     chaque `LiveEvent::InsertRemote` recu sur le namespace.
   - **Protocole bridge complet** :
     1. Iframe appelle `bridge.onStorageUpdate("sbfb-ideas", cb)`
     2. SDK `sbfb-bridge.js` lance un `setInterval(3000)` qui
        appelle `_call("storage_version", { app: "sbfb-ideas" })`
     3. Cet appel traverse postMessage → shell React host
     4. `useBridge.ts dispatch("storage_version")` fait
        `authFetch("/api/daemon/storage/sbfb-ideas/version")`
     5. Response postMessage retourne la version au SDK iframe
     6. Si version differente du dernier poll → SDK appelle le
        callback enregistre
   - **Fichiers touches** : `storage_api.rs` (endpoint version),
     `web/src/bridge/protocol.ts` (methode storage_version),
     `web/src/bridge/useBridge.ts` (dispatch handler),
     `web/public/sbfb-bridge.js` (onStorageUpdate + interval)

3. **sbfb-bridge.js : onStorageUpdate callback**
   - Fichier : `web/public/sbfb-bridge.js`
   - Ajouter `storage_version` dans `BridgeMethodSchema`
   - Ajouter `onStorageUpdate(appName, callback)` : lance
     un interval qui poll storage_version via _call, compare
     avec la derniere valeur, invoque le callback si change
   - Propager vers les 2 copies examples/ via sync-bridge-sdk.sh

4. **Ideas Hub app.js : refresh on update**
   - Fichier : `examples/sbfb-ideas/app.js`
   - Appeler `bridge.onStorageUpdate("sbfb-ideas", loadAll)`
   - Afficher un indicateur "derniere sync : il y a Ns"

5. **Test E2E sync 2 noeuds**
   - Fichier : `crates/nexus-test-harness/tests/multi_daemon.rs`
   - Test `test_cross_daemon_storage_sync` :
     1. Daemon A demarre, cree namespace storage
     2. Daemon A ecrit `ideas/test-1` via
        `POST /app/sbfb-ideas/state/ideas/test-1`
     3. Test recupere le ticket via
        `GET /api/daemon/storage/ticket/sbfb-ideas` sur daemon A
     4. Daemon B demarre, importe le ticket via
        `POST /api/daemon/storage/join` sur daemon B
     5. Poll `GET /app/sbfb-ideas/state?prefix=ideas/` sur
        daemon B avec timeout 30s
     6. Assert : daemon B voit `ideas/test-1` dans la reponse
   - Gate `SBFB_INTEGRATION=1` (meme pattern que gossip E2E)
   - Le test utilise les endpoints HTTP des 2 daemons (pas
     d'appels internes directs — preuve que le chemin prod
     fonctionne)

6. **Update sbfb-bridge.js copies**
   - Run `scripts/sync-bridge-sdk.sh` pour propager les changes

7. **Anti-spam carry S59**
   - Documenter dans le commit body : anti-spam couches 2-3
     (rate-limit per-author + validation applicative) = dette
     explicite S59. Pre-v1.0 acceptable (reseau controle).

### §D Verification

- `cargo nextest run --workspace --locked` vert
- Test E2E storage sync vert (avec SBFB_INTEGRATION=1)

### §D Commit

```
feat(sprint58): Sprint 58 Phase D — AppStorage P2P live events + sync E2E

Subscribe InsertRemote sur storage namespace. Compteur version
atomique incremente par InsertRemote. Endpoint GET
/api/daemon/storage/:app/version.

Bridge polling MVP : storage_version methode bridge → shell
authFetch → daemon endpoint. SDK onStorageUpdate(app, cb)
lance interval 3s, compare version, invoke callback.

Test E2E : 2 daemons, noeud A ecrit ideas/test-1, daemon B
importe ticket via POST /api/daemon/storage/join, recoit
l'entree via iroh-docs sync.

Anti-spam couches 2-3 = dette explicite S59 (pre-v1.0
acceptable, reseau controle).

Delta tests : +N Rust (cumule).
Scope cuts :
- SSE temps reel → S59+ (polling MVP S58)
- Indicateur "noeuds connectes" si metadata indisponible
```

---

## Phase E — Wrap-up + verification + audit plan S59

### §E Objectif

Cloturer le sprint, documenter l'etat.

### §E Tasks

1. **Fail-fast checklist** : 26+ rows executables
2. **CLAUDE.md** : S58 CLOSED, carries S59
3. **HARDENING_ROADMAP** : last_validated S58
4. **SPRINT_LOG** : row S58
5. **sprint59_audit_plan.md** : 7+ tracks auditant S58
6. **Memory update** : nexus_grid_pivot.md tip + compteurs

### §E Commit

```
chore(sprint58): Phase E — wrap-up + verification + audit plan S59
```

---

## §5 Dependencies inter-phases

```
Phase A (MANDATORY) ←── aucune dep
Phase B (dette)     ←── aucune dep
Phase C (namespace) ←── aucune dep technique, mais logiquement
                        apres A+B pour garder le flux atomique
Phase D (events)    ←── Phase C (namespace doit exister)
Phase E (wrap-up)   ←── Phase D (cumule complet)
```

Phases A et B sont independantes et pourraient theoriquement
etre mergees, mais la discipline 1 commit = 1 phase impose la
separation MANDATORY/dette.

---

## §6 Pre-launch protocol policy check

Aucun `*_FORMAT_VERSION` n'est touche par ce sprint :
- `TASK_FORMAT_VERSION` : inchange (dispatch unaffected)
- `INVITE_FORMAT_VERSION` : inchange (doc only en Phase A)
- Storage namespace keys : schema applicatif interne, pas de
  wire format P2P

Le namespace iroh-docs pour le storage est un detail
d'implementation du daemon, pas un protocole inter-projets.
Les cles `ideas/`, `votes/` ne transitent pas par le gossip.
