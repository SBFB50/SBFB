# Sprint 73 — Kickoff (Recherche reseau cablee — FTS5 fraicheur + SearchResult enrichi + barre shell)

**Ecrit** : 2026-06-03 (post-audit gate S72 PASS `087e781`).
**Type** : **sprint IMPAIR** mais a **forte charge dette** — 12 P2 herites de
l'audit S72, dont **P2-A-1(S71) worker-pump 3/3 MANDATORY** (entre au plan
comme phase, plus en carry — §6.2.1 Regle 2) + **P2-RESULT-TEXT-GUARDRAIL-
ORDER** (priorite haute, traiter tot). Une **phase dette dediee (Phase B)**
est reservee bien que S73 soit impair.
**Tip master d'entree** : `087e781` (audit findings S72 PASS — 0 P0, 0 P1,
12 P2, 10 P3).
**Phase 0 audit Sprint 72** : **DEJA JOUE** — `087e781`
(`chore(planning): Sprint 72 audit findings — PASS (S73 Phase 0)`).
**Aucun fix `fix(sprint72)` requis** (0 P0/P1) ; les 12 P2 sont routes au
plan S73 (§6), pas re-implementes en Phase 0.
**Version archive** : v2.1 — « Protocole Neutre + Factory/RRV » (OPEN). S73
continue le meme arc (3.5 Factory Complete Vision), aucune release publiee
depuis → **reste v2.1**. Migration S72 `active/`→`archive/v2.1/` au present
kickoff (precedent `1803d78`).
**Roadmap source** : `.planning/roadmap_v5_factory_complete_vision.md`.
Sprint **3 sur 6** (S71-S76), Arc 3.5 « Factory Complete Vision ».

---

## Sources context7 + WebSearch consultees (pre-gel)

Recherche G9 multi-agent effectuee AVANT de figer D1..D6 (workflow
`wq01d17lj`, 12 agents, ~1.24M tokens). Dates absolues, versions, URLs.

### OSS prior art — FTS5 hot reindex incremental (D1)

| Source | Date | Finding cle |
|--------|------|-------------|
| sqlite.org/fts5.html | SQLite 3.50.x (current) | `search_index` SBFB est une table FTS5 **standalone** (pas external-content). Pour standalone : `INSERT OR REPLACE INTO ft(rowid, …)` est l'upsert idempotent ; les triggers external-content ne s'appliquent **pas**. `'delete'`/`'rebuild'`/`'optimize'` = commandes speciales. |
| sqlite.org/forum (FTS5 corruption) | thread | Triggers FTS5 mal ecrits (DELETE simple, ou INSERT sans `rowid`) corrompent l'index (~10% des updates). Raison d'eviter les triggers ici. |
| sqlite.org/releaselog/3_43_0 | 2023-08-24 | `contentless_delete=1` (DELETE/upsert sur contentless) dispo depuis 3.43.0 — mais lit NULL sur colonnes non-rowid (casserait le read path SBFB). Rejete. |
| Cargo.lock SBFB (`libsqlite3-sys 0.34.0`, rusqlite 0.36 bundled) | verifie | SQLite 3.50.x embarque → `INSERT OR REPLACE` + WAL garantis sans dep systeme. |
| sqlite.org/wal.html | current | WAL : 1 writer concurrent avec N readers ; readers jamais bloques. Un upsert FTS court (1 stmt) ne bloque pas les lecteurs `/api/daemon/search`. |
| github.com rusqlite/rusqlite discussions #1226 | 2024 | `Connection` non-Sync ; patterns = `Mutex<Connection>` (actuel SBFB) ou pool. Single-conn correct sous charge actuelle ; pool seulement >~50 writes/s (pas atteint). `busy_timeout` defaut 5000ms — rendre explicite. |

### OSS prior art — discovery/recherche decentralisee (D3 SearchManifest)

| Source | Date | Finding cle |
|--------|------|-------------|
| f-droid.org (index-v2 + Security_Model) | 2023-03 / 2024-2026 | Index **signe par depot** (RSA/SHA-256), diffs incrementaux RFC 7396. Anti-spam = signature du mainteneur (curation par depot). Pas de crawl global : on cherche dans les depots ajoutes. Analogue feed-local SBFB. |
| ipshipyard.com (DHT Provide Sweep) + blog.ipfs.tech (delegated routing caching) | 2025 | Provider records IPFS **expirent a 24h** → re-announce continu obligatoire (cout operationnel reel de l'announce). 2026 : caching/probing sur noeuds specialises (pas chaque noeud). |
| dl.acm.org ARES 2024 (« Sybil Attack Strikes Again ») | 2024 | Mainline DHT + IPFS DHT : censure/DoS d'un contenu **depuis une seule machine** (Sybil). Le broadcast opt-in reseau-large ouvre exactement cette surface. |
| github.com nostr-protocol/nips/50 (NIP-50) | + Purgatory sept. 2024 | Recherche **deleguee aux relays** specialises avec anti-spam dedie. Pas chaque noeud qui propage son index → modele « noeud-index », pas broadcast. |
| radicle.dev/guides/protocol (Heartwood) | 2024-2025 | Announce **scope par interet** (on recoit les annonces des depots qu'on seede), seed nodes pour le scaling. = feed-local enrichi par interet. |
| handbook.scuttlebutt.nz (gossip) | 2024 | Replication par **proximite sociale** (follows). Anti-spam par construction, mais couverture bornee. = pur feed-local-replique. |
| github.com Pubky/pkarr + iroh discovery | 2024-2025 | SBFB a deja pkarr (Ed25519 sur Mainline BEP44) + gossip + curator lists pour la **decouverte de noeuds/projets**. S73 ne concerne QUE l'indexation full-text, pas la decouverte. |

### OSS prior art — iroh-docs pump Windows hang (D6, P2-A-1 3/3)

| Source | Date | Finding cle |
|--------|------|-------------|
| docs.rs/tokio attr.test + runtime/index | tokio current (2026) | Sur runtime `current_thread`, une tache `spawn` n'avance **que** quand le futur principal yield ; les taches de fond continues **exigent** `multi_thread`. C'est le mecanisme du hang worker-pump. |
| github.com tokio-rs/tokio #2499 + #7049 | open (2024+) | Classe de **deadlocks Windows-only** tokio (shutdown single-thread / thread_local drop). L'ordonnancement Windows fait deadlocker ce que Linux tolere (racy mais passe). |
| iroh.computer/blog (Async Rust Challenges in Iroh) | iroh eng. blog | Le store iroh-docs est un **acteur sur thread dedie** avec son propre runtime single-thread (blocking sync IO). Le wakeup cross-thread/cross-runtime rend le flavor du runtime de test load-bearing pour la liveness. |
| `crates/nexus-core-rs/examples/two_nodes_docs_sync.rs` + `nexus-worker/src/main.rs:41` | repo | **Fix prouve in-repo** : le seul sync 2-noeuds qui marche + tous les binaires prod utilisent `multi_thread`. Les 2 tests qui hang sont les **seuls outliers `current_thread`**. |
| github.com n0-computer/iroh-docs CHANGELOG (0.99.0) | 2026-05-08 | 0.99.0 « Drain Actor::tasks JoinSet » (fix teardown) — mais 0.99 = breaking (tracks iroh 1.0.0-rc.0 + redb@4), **interdit par le pin gele iroh 0.98** (R-iroh-audit P0). Donc fix amont **non adoptable** maintenant → le fix multi_thread reste la voie. |

### Versions deps confirmees (lockfile)

`rusqlite 0.36` (`libsqlite3-sys 0.34.0` bundled = SQLite 3.50.x), `iroh
0.98`/`iroh-docs 0.98` (pin gele), `axum 0.8.9`, `tokio` (workspace),
`reqwest 0.12.28`. **Aucune nouvelle dep S73** : tout le travail reutilise
l'existant (FTS5 deja la, feed_sync deja la, BrowseEntry deja enrichi). Front
`web/` : React + Vite + TS + Tailwind + shadcn + Zustand + React Query
(barre recherche D4). Front Operator `tools/factory-operator/` : model-picker
(D-dette, intentions non-Claude).

**Decision crypto/spec nouvelle ?** Non pour le code livre S73 (feed-local =
indexation locale + DTO + UX ; guardrail D5 = reordonnancement d'un filtre
existant). Le design note SearchManifest (D3) **specifie** la crypto correcte
du futur wire mais ne la code pas. La checklist `[DETER]` Rust-first
s'applique a D1/D5/D6 (cf. `sprint73_design_review.md`).

---

## §1 Constat d'entree

### §1.1 D'ou on part

S72 a CLOSE (PASS, `087e781`) en livrant le **routage provider multi-LLM**
(`ExecutionTarget` Claude/Ollama/Network, NetworkProvider submit→poll) + le
cablage `provider` de bout en bout + l'UX intentions `/execute`. Le chat
Factory peut maintenant **soumettre** une sous-tache au reseau. S73 pose le
**chemin de decouverte** : **chercher** une app/un projet sur le reseau
**avant** de la forker (S74) ou de soumettre.

Le constat clé de la cartographie : l'infra recherche existe deja a
**~90-97%** (livree S67-S69 RRV @protocole, verifiee master propre). FTS5
`search_index` (M15), route `GET /api/daemon/search`, bm25, sanitize
anti-injection — **tout est la**. Les manques sont chirurgicaux :

1. **Fraicheur** : `rebuild_from_feed()` n'est appele **qu'au boot**
   (`runtime.rs:778`) ; `feed_sync.rs:260` insere le feed distant **sans
   reindexer** → projets recus via gossip **invisibles jusqu'au reboot**.
2. **Enrichissement** : `SearchResult` = 7 champs **sans le triplet
   provenance** (`repo_url`+`commit_sha`+`archive_hash`+`provenance_hash`) →
   un hit ne peut pas declencher un fork (prerequis S74). La donnee existe
   deja dans `BrowseEntry` et le payload feed, juste pas routee vers l'index.
3. **Shell** : la Command Palette (Ctrl+K) est de la **navigation**, pas de
   la recherche ; le bridge SDK a une methode `search` mais **aucun
   composant front ne la consomme**.

C'est strictement du **cablage + durcissement**, pas de la fondation.

### §1.2 Ancrage roadmap v2.1 (Arc 3.5)

Arc 3.5 « Factory Complete Vision » (roadmap v5, CANON), 6 sprints S71-S76.
Position : **sprint 3/6**.

```
S71 assainir+securite+reconciliation (DONE, fonde TOUT)
  └─ S72 quick win: chat Factory route les taches (DONE)
       └─ S73 recherche reseau cablee (FTS5 fraicheur + SearchResult enrichi)  ← ICI
            └─ S74 atelier: rouvrir/forker un projet reseau
                 └─ S75 GPU partage PROUVE cross-machine
                      └─ S76 STRETCH: sharding pipeline
```

**Dependances aval** : S73 enrichit `SearchResult` avec le triplet
provenance que **S74 reutilisera pour forker** (`repo_url@commit_sha` forge,
ou `archive_hash` blob en repli — PO-5). Sans cet enrichissement, un hit de
recherche ne peut pas declencher un fork. S73 NE livre PAS les commandes
`search/open/fork` (S74), NE livre PAS la notion de projet cible distinct
(S74), NE livre PAS le GPU cross-machine (S75).

### §1.3 Compteurs tests entree (tip `087e781`)

| Suite | Count |
|---|---|
| Rust nextest | **1544** (compte canonique CI Linux ; Windows natif = 1543 passed + 1 flake env `operator_sprint_history_endpoint` = P2-OPERATOR-TIMEOUT, traite Phase B) |
| Vitest (`web/`) | 279 |
| size-limit | 6/6 |
| **Total** | **~1829** |

Re-mesure exacte au `plan.md §1` sur le SHA reel post-kickoff.

### §1.4 Pre-launch protocol policy (rappel)

Rien n'est pousse vers origin (30 ahead). **Reconciliation locale libre**.

- `FEED_FORMAT_VERSION = 1`, `PROJECT_ANNOUNCEMENT_VERSION = 1`,
  `TASK_FORMAT_VERSION` restent **inchanges**. S73 ne touche **aucun wire
  format reseau** : l'enrichissement `SearchResult` et la migration FTS5 M17
  sont **locaux daemon** (`search_index` n'est PAS synchronise via
  iroh-docs/gossip — chaque noeud reconstruit son index depuis le feed
  qu'il a recu). D3 **defere** le wire SearchManifest → aucun bump.
- Migration SQLite M17 (recreation table virtuelle FTS5 + colonnes) =
  schema **local**, pas un wire format. Le `db.rs` comment confirme « local
  persistence only ». Index integralement reconstructible depuis le feed.
- `#[serde(default)]` legitime pour la robustesse runtime : les 4 nouveaux
  champs `SearchResult` sont `Option<String>` (un vieux client / une vieille
  ligne d'index → `None`, pas un 422). Tolerance runtime, pas compat
  historique.
- Pas de tolerant decoder multi-version. Pas de test « legacy decode ».

---

## §2 Goal

> Sprint 73 cable la **recherche reseau** de SBFB : les projets recus via le
> feed gossip deviennent **cherchables a l'instant** ou ils arrivent (reindex
> FTS5 a chaud incremental sur le chemin `feed_sync`, plus seulement au
> boot), `SearchResult` porte le **triplet provenance**
> (`repo_url`+`commit_sha`+`archive_hash`+`provenance_hash`) pour qu'un hit
> puisse declencher un fork (S74), et le **shell expose une barre de
> recherche** cablee sur `GET /api/daemon/search` (champ dedie dans Browse).
> La decision **SearchManifest** est tranchee : **feed-local-replique** pour
> S73 (le reseau partage deja le feed gossip — couverture equivalente en
> pilote ferme, zero surface Sybil nouvelle), le SearchManifest reseau-large
> etant **defere** sous sa forme correcte (noeud-index opt-in signe,
> design note capture) jusqu'a un signal empirique post-launch (PO-13 honore).
> Avant d'etendre la surface de recuperation reseau, **Phase A corrige
> l'invariant de securite** P2-RESULT-TEXT-GUARDRAIL-ORDER (guardrail AVANT
> persist sur les 2 chemins) ; **Phase B (dette) ferme P2-A-1 worker-pump
> 3/3** par un fix root-cause cross-platform + le lot dette test + le
> durcissement NetworkProvider/Operator complet.
> **Critere SMART : 100% des rows fail-fast vertes au
> `sprint73_verification.md §Fail-fast checklist`, mesure binaire au Phase F
> wrap-up.** La fail-fast checklist (28-34 rows executables, cf. plan
> §Fail-fast) EST la source of truth mesurable du goal.

---

## §3 Phase 0 — Audit gate Sprint 72

**DEJA JOUE.** `sprint72_audit_findings.md` (`087e781`), 9 tracks A-I,
verdict **PASS**. Diff audite `0b4e7f3..95cae05` (audit multi-agent
independant : workflow ~531s/713k tok sur DIFF BRUT + verif adversariale +
spot-verif main thread). Resume :

- **0 P0, 0 P1** — aucun `fix(sprint72)` requis avant Phase A.
- **12 P2** : 8 carries anticipes confirmes + **4 nouveaux** decouverts par
  l'audit independant : **P2-RESULT-TEXT-GUARDRAIL-ORDER** (headline,
  priorite haute → S73 Phase A), P2-HARDENING-ROADMAP-META-STALE,
  P2-PREFLIGHT-TRANSITIVE-DEPTH, P2-PREFLIGHT-WIRE-CONTRACT-DEPTH. Les 3
  candidats P1 de l'audit_plan §6 ont tous ete **refutes** (gate avant
  dispatch OK, route /result authentifiee GET-only, G1 5/5).
- **10 P3** : nits cosmetiques (imprecisions doc PATTERNS §P53/§P54/§P55,
  etiquetage prose, residu Cargo.lock inerte) — balayables opportunement.
- Tous les 12 P2 sont routes au plan S73 (§6 + §7 audit_findings).

---

## §4 Decisions Day 0 (D1..D6 gelees)

### D1 — Reindex FTS5 a chaud : upsert incremental par `feed seq`, pas rebuild

**Sources consultees** : sqlite.org/fts5.html + /wal.html (current),
rusqlite #1226 (2024), Cargo.lock (`libsqlite3-sys 0.34` = SQLite 3.50.x).
Code lu : `search.rs:34-49` (`index_entry()`), `search.rs:95-128`
(`rebuild_from_feed()` boot-only), `feed_sync.rs:113-281`
(`ingest_doc_entry()`, insert `db.insert_feed_entry(&row)` ligne 260),
`db.rs:211-222` (M15 FTS5 schema), `runtime.rs:773-782` (boot rebuild).

**Retenu** : ajouter un **upsert FTS5 incremental** appele **immediatement
apres** un `db.insert_feed_entry(&row)` reussi (feed_sync.rs:~261), **dans le
meme scope de lock** `Arc<Mutex<CoordinatorDb>>` (donc meme transaction
courte WAL — invisible aux lecteurs `/api/daemon/search` concurrents).
L'upsert est **idempotent** : nouvelle fonction `search::upsert_feed_entry(db,
seq, …)` avec `INSERT OR REPLACE INTO search_index(rowid, project_id,
project_name, category, description, op_type, source_type) VALUES (?seq, …)`.
Le `seq` (INTEGER monotone retourne par `insert_feed_entry`) devient le
`rowid` FTS5 → une entree re-arrivee est un **no-op rewrite**, jamais un
doublon (2e ligne de defense apres la dedup-par-`entry_hash` deja en place
`feed_sync.rs:204-217`). L'extraction JSON (`project_id`/`project_name`/
`description` depuis `feed_entry.op`) est **factorisee** dans un helper
partage avec `rebuild_from_feed()` (eviter la derive). `rebuild_from_feed()`
**reste** comme chemin de **reparation/migration** (full rebuild explicite),
plus sur le chemin chaud. `busy_timeout` rendu explicite
(`conn.busy_timeout(Duration::from_secs(5))`).

**Rejete** :
- *Triggers external-content FTS5 (pattern officiel SQLite)* : ne s'applique
  qu'aux tables external-content (`content='t1'`). `search_index` est
  **standalone** et indexe un payload JSON parse en Rust, pas une table SQL
  1:1. Convertir = materialiser une table miroir + 3 triggers (plus de
  surface + risque de corruption documente si un trigger omet `rowid` ou
  fait un DELETE simple). Sur-ingenierie. Rejete.
- *Table contentless-delete (`content=''`, `contentless_delete=1`)* : lit
  NULL sur toute colonne non-rowid → casse le read path actuel `search()`
  qui SELECT `project_name`/`category`/`description` directement. Rejete.
- *Rebuild complet `rebuild_from_feed()` a chaque ingest* : O(N) reecriture
  de tout l'index par entree recue → tient le write lock proportionnellement
  a la taille du feed, **amplification DoS** sous T-CURATOR-VOUCH/T-FEED-SPAM
  (THREAT_MODEL §11). Acceptable seulement comme commande de reparation.
  Rejete sur le chemin chaud.
- *Pool de connexions read-only (r2d2/deadpool) maintenant* : premature —
  le feed n'atteint pas ~50 writes/s. Le `Arc<Mutex<CoordinatorDb>>` single-
  conn est correct sous charge actuelle. Differe (mesure d'abord).

**Implications code** : `crates/nexus-coordinator-rs/src/search.rs`
(`upsert_feed_entry()` NEW + helper extraction partage) ;
`crates/nexus-shell-daemon/src/feed_sync.rs:~261` (appel upsert apres insert
Ok, meme lock scope) ; `db.rs` (busy_timeout explicite a l'open). Zero impact
wire (search_index local-only).

### D2 — Enrichir `SearchResult` avec le triplet provenance : colonnes UNINDEXED + migration M17

**Sources consultees** : code lu `search.rs:7-16` (SearchResult 7 champs),
`browse.rs:170-225` (BrowseEntry **a deja** repo_url/archive_hash/
provenance_hash/is_open_source), `public_feed.rs:32-40`
(ReleasePublishedPayload porte repo_url/commit_sha/artifact_hash/
provenance_hash), `provenance.rs:18-29` (ProvenanceRecord signe persistant),
`deploy.rs:250-426` (data flow deploy : feed + provenance_record + browse).

**Retenu** : ajouter 4 champs `Option<String>` (+ `is_open_source: bool`) a
`SearchResult` (serde defaults) : `repo_url`, `commit_sha`, `archive_hash`,
`provenance_hash`. Cote schema, migration **M17** : FTS5 **ne supporte pas
ALTER TABLE ADD COLUMN** → **CREATE** une nouvelle table virtuelle
`search_index` avec les colonnes ajoutees **en UNINDEXED** (retournees, pas
full-text-matchables — coherent avec `project_id`/`op_type`/`source_type`
deja UNINDEXED) puis repopuler via `rebuild_from_feed()` (l'index est
**integralement reconstructible** depuis `public_feed`, source de verite —
aucune donnee unique perdue). `index_entry()`/`upsert_feed_entry()` (D1)
etendent leur signature pour passer le triplet ; l'extraction depuis
`ReleasePublishedPayload` JSON se fait dans le helper partage (D1). `search()`
SELECT les nouvelles colonnes (offsets 7-10) ; `search_handler` (http.rs:
1989-2001) les serialise dans la reponse JSON.

**Rejete** :
- *Colonnes INDEXED pour le triplet* : un `commit_sha`/`archive_hash` est un
  hash, pas un token de langage naturel — un `MATCH` full-text dessus n'a
  aucun sens, et l'indexation gonfle l'index 20-30% (research). Le triplet
  sert au **fork** (retourne), pas a la recherche. Rejete (UNINDEXED).
- *JOIN vers BrowseEntry/provenance_records au query-time* : `search()`
  retourne `SearchResult` independamment, aucun chemin de join existant ;
  denormaliser dans l'index (comme BrowseEntry le prouve deja) est plus
  simple et plus rapide. Rejete.
- *Endpoint de provenance separe (lookup apres hit)* : defait le but « un
  hit peut forker » — le fork a besoin du triplet **dans** le resultat de
  recherche, pas d'un 2e aller-retour. Rejete.

**Implications code** : `search.rs:7-16` (SearchResult +4 champs), `:34-49`
(signature index_entry/upsert), `:51-93` (SELECT + query_map offsets), `:95-128`
(rebuild_from_feed extrait le triplet) ; `db.rs:211-222` (migration M17
DROP/recreate + colonnes UNINDEXED) ; `http.rs:1989-2001` (JSON reponse).
Zero wire (additif local, `#[serde(default)]`).

### D3 — SearchManifest : DEFER ; S73 = feed-local-replique (+ design note forme correcte)

**Decision (Checkpoint §11, arbitrage 2026-06-03)** : **feed-local-replique**
pour S73, **defer** SearchManifest reseau-large.

**Sources consultees** (7 modeles, research G9) : F-Droid (index signe par
depot), IPFS DHT (provider records expirent 24h + delegated routing 2025-26),
ARES 2024 (Sybil mono-machine sur DHT ouvert), Nostr NIP-50 (recherche
deleguee aux relays), Radicle Heartwood (announce scope par interet), SSB
(replication proximite sociale), pkarr/iroh (decouverte noeuds deja en
place). Code lu : `public_feed.rs:82-118` (FeedEntry.op = raw `Value`, 4 ops
existantes, `FEED_FORMAT_VERSION=1`), CLAUDE.md:354-366 (pre-launch raw-op),
scan historique G8 (decision deferee « selon audit S72 », toujours ouverte).

**Retenu** : les deux vrais gaps S73 (fraicheur D1 + enrichissement D2) se
resolvent **entierement** cote feed-local — le feed est deja gossip-replique
en DB locale (`feed_sync.rs`), la recherche est deja une FTS5 bm25 locale
(`search.rs`). On **indexe ce que le gossip a deja replique**. En pilote
ferme (zero noeud tiers en prod), la couverture feed-local **≈ couverture
reseau-large** (tout le pilote partage le meme feed). SearchManifest
n'apporterait **aucun gain de couverture** aujourd'hui tout en ouvrant la
surface Sybil/spam (ARES 2024) + un cout d'announce continu (provider records
24h). Les systemes matures **n'font pas** « chaque noeud propage son index a
tous » — ils utilisent des **noeuds-index specialises** (relays Nostr,
delegated routing IPFS, index signe F-Droid).

**Mitigation « le plus pousse » (sans scaffolding)** : capturer la
**conception correcte** du futur SearchManifest dans
`.planning/research/s73_searchmanifest_index_node_design.md` (Phase D) :
noeud-index **opt-in** signe Ed25519 (modele relay/seed-node), anti-spam
signature curator + reputation kudos, **default OFF** (requetes utilisateur
**jamais** envoyees au reseau), critere de declenchement = federation
partielle post-launch (noeuds rejoignant sans le feed gossip complet). Le
design dur est fait et pret a coder, sans code protocole speculatif. PO-13
(« cabler les deux a terme ») **honore** : feed-local maintenant, noeud-index
opt-in plus tard.

**Rejete** :
- *Implementer SearchManifest-broadcast maintenant* : zero gain de couverture
  en pilote ferme + surface Sybil/spam + cout announce + c'est precisement la
  forme que les systemes matures **abandonnent**. Construire ce qu'on devrait
  jeter. Rejete.
- *Squelette raw-op desactive* : scaffolding speculatif (CLAUDE.md « pas de
  scaffolding ») ; le raw-op est deja extensible quand on voudra l'ajouter —
  inutile de poser un type mort. Rejete (le design note capture le futur sans
  code mort).

**Implications code** : **aucun code wire S73** (defer). NEW
`.planning/research/s73_searchmanifest_index_node_design.md` (design note).
`FEED_FORMAT_VERSION` reste 1 (la porte reste ouverte via raw-op).

**Note DESIGN-CONFLICT evite** : un doc fossile
(`s70_s72_rrv_research.md:985-987`) affirme « ajouter SearchManifestPublished
= breaking change, bump v2 » — **faux** sous la politique pre-launch actuelle
(raw-op, pas de bump, CLAUDE.md:355-357). Comme S73 defere, le conflit ne
mord pas ; note pour le sprint d'implementation (verifier `PUBLIC_FEED_SPEC.md
§9` au preflight wire). De meme : **Tantivy reste gele hors-scope** (gate
post-S75 >50K docs) — un doc fossile le recommande, **ne pas rouvrir** (FTS5
est l'engine, decision gelee CLAUDE.md:306).

### D4 — Barre de recherche shell : champ dedie dans Browse via `searchBrowse()`

**Decision (Checkpoint §11, arbitrage 2026-06-03)** : **champ dedie dans
Browse** (pas barre header globale, pas palette unifiee).

**Sources consultees** : code lu `Browse.tsx:39-108` (liste in-memory
`listBrowse`, pas de champ recherche), `CommandPalette.tsx` (Ctrl+K =
navigation/commandes, PAS recherche), `useBridge.ts:359-371` (bridge SDK
`search` → `/api/daemon/search`, **aucun composant front ne le consomme**),
`api/daemon.ts:194-311` (pattern `listBrowse` + `DaemonResult` + `authFetch`
+ Zod + React Query), `scan-en-strings.sh:26` (FR-only enforce).

**Retenu** : ajouter `searchBrowse(baseUrl, q, limit, offset)` dans
`web/src/api/daemon.ts` (miroir de `listBrowse`, via `authFetch` +
`DaemonResult<SearchResponse>` + `SearchResponseSchema` Zod). Ajouter un
**champ de recherche dedie au-dessus de la grille d'apps dans Browse**
(`Browse.tsx:~104`), cable via React Query (`queryKey: ['daemon-search',
coordUrl, q]`). Les resultats portent le triplet provenance (D2) → prets pour
le fork S74. La Command Palette **reste navigation** (separation des
intentions). Strings utilisateur **en francais** (placeholders, etats vides,
erreurs — `scan-en-strings.sh` rejette l'anglais). Pas de pagination
boutons en S73 (corpus petit pre-launch ; differe si besoin mesure).

**Rejete** :
- *Barre globale dans le header AppShell* : melange recherche d'apps et
  navigation, et le bouton « Rechercher » du header ouvre deja la palette
  (confusion). Rejete.
- *Recherche full-text dans la Command Palette (Ctrl+K)* : change le contrat
  de la palette (navigation) et melange commandes locales vs apps reseau —
  confusion UX. Rejete.
- *Reutiliser le bridge SDK seul* : les apps peuvent appeler `search`, mais
  les composants shell **ne peuvent pas** (pas de helper public). Il faut le
  wrapper front. Rejete (on l'ajoute).

**Implications code** : `web/src/api/daemon.ts` (`searchBrowse()` +
`SearchResponseSchema`), `web/src/pages/Browse.tsx` (champ recherche +
useQuery + rendu resultats avec provenance), i18n FR. Frontend `web/`
(exemption Rust-first).

### D5 — Guardrail de sortie AVANT persist `result_text` sur les 2 chemins (P2-RESULT-TEXT-GUARDRAIL-ORDER, Phase A)

**Sources consultees** (audit S72 headline) : code lu `validator.rs:25-89`
(`validate_result()` persiste via `set_task_result` lignes 74-80 single +
155 quorum **pendant** la validation), `http.rs:1485-1540`
(`coordinator_submit_result` : `validate_result` ligne 1500 **persiste**,
PUIS guardrail `default_output_chain` ligne 1507 — si rejet, deja persiste,
status=completed, **pas de rollback**), `validator_loop.rs:62-80` (appelle
`validate_result` ligne 62 avec **ZERO guardrail** — chemin gossip Sprint 38),
`tasks_api.rs:160-190` (GET /result lit `result_text` persiste, T0 loopback).

**Retenu** : **split** `validator::validate_result` en deux phases : (1)
`validate_result_pre_guardrail()` (signature + status + quorum, **ne
persiste PAS**) ; (2) `validate_result_post_guardrail()` qui appelle
`set_task_result()` **uniquement apres** passage du guardrail. Chemin HTTP
(`http.rs:1500-1522`) : pre-guardrail → `default_output_chain().run()` →
post-guardrail (persist) ; si rejet, **aucune ligne creee**. Chemin
`validator_loop` (`:62-80`) : **injecter le guardrail** avant persist
(actuellement aucun) — si rejet, log + skip persist + **ne pas crediter
kudos**. Chemin quorum (redundancy>1) : guardrail sur le texte **agree**
(`best_hash`, apres accord quorum ligne 155) avant `set_task_result`.
Corriger les claims **fausses** : `THREAT_MODEL.md §14` (lignes 786-790) +
`LOOPBACK_ENDPOINTS_TRUST_TIERS.md §3.1` (ligne 56) affirment « texte deja
filtre » — reecrire pour refleter l'ordre reel (guardrail AVANT persist sur
les 2 chemins).

**Rejete** :
- *Rollback apres persist* : pas de transaction SQLite couvrant le chemin +
  laisse une fenetre ou GET /result sert du contenu rejete. Rejete.
- *Guardrail seulement sur le chemin HTTP* : `validator_loop` est le chemin
  **gossip-sourced** (resultats reseau non-confiants, Sprint 38) — c'est
  exactement la ou il faut gater. Rejete.
- *Fix doc-only (corriger la claim sans bouger le code)* : la claim est
  fausse ET le code est faux. Band-aid. Rejete.

**Implications code** : `validator.rs:25-89,155` (split pre/post), `http.rs:
1500-1522` (reordonner), `validator_loop.rs:62-80` (injecter gate),
`THREAT_MODEL.md:786-790` + `LOOPBACK_…TRUST_TIERS.md:56` (corriger claims).
Zero wire (timing interne). **Note** : changement de comportement HTTP — sur
rejet, la ligne n'est plus `completed` (verifier les tests d'integration qui
liraient un result apres un 400).

### D6 — worker-pump iroh-docs (P2-A-1 3/3 MANDATORY) : fix root-cause cross-platform multi_thread, pas exemption

**Sources consultees** (research G9 + code) : `dispatch_loop.rs:146-261`
(`dispatched_task_is_claimed_and_executed_by_worker_engine`, bare
`#[tokio::test]` = current_thread, polling `doc.get_many_by_prefix`),
`runtime.rs:864` (pump worker `get_many_by_prefix(b"task:")` sans
timeout/cancel), `docs.rs:291-307` (wrapper get_many_by_prefix), tokio
#2499/#7049 (deadlocks Windows-only current_thread), iroh blog (store =
acteur thread dedie), `examples/two_nodes_docs_sync.rs` +
`nexus-worker/src/main.rs:41` (**multi_thread, le fix prouve in-repo**),
iroh-docs 0.99 CHANGELOG (fix amont mais breaking, **interdit par pin 0.98**).

**Retenu** : **fix root-cause cross-platform** — passer les 2 tests E2E
affectes a `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`.
**Ce n'est PAS un band-aid** : ca fait matcher le runtime de test a la
maniere dont le code tourne **partout ailleurs** (binaire worker
`main.rs:41`, daemon, et le SEUL sync 2-noeuds qui marche
`examples/two_nodes_docs_sync.rs` en multi_thread worker_threads=4). Les
tests qui echouent sont les **seuls outliers current_thread**. **Chemin
unique cross-platform, ZERO `#[cfg(windows)]`** — neutre sur Linux (qui
passait deja), corrige Windows. Defense-en-profondeur : **garder** le
`tokio::time::timeout(10s, …)` (une regression future **echoue vite** au lieu
de hang tout le nextest run) ; preferer `tokio::select!` pour la consommation
du stream. Documenter dans `PATTERNS §P54` : tout test pilotant le pump
iroh-docs concurremment avec un engine spawn **DOIT** etre multi_thread.
**Reproduire sur Windows natif** (`feedback_wsl_before_push`) → vert, puis
Docker Linux → vert → **P2-A-1 CLOSED par fix code**.

**Rejete** :
- *4e report / exemption comme defaut* : 3/3 escalade (G7 Regle 2) interdit le
  report. L'exemption reste le **fallback documente uniquement si** le
  multi_thread ne suffit pas (residu teardown lie au pin 0.98 pre-0.99 actor
  JoinSet-drain, fix amont interdit). Pas le defaut. Rejete comme primaire.
- *`#[cfg(windows)]` band-aid (chemin divergent)* : viole §6.3 (no band-aid)
  + masque le comportement prod. Rejete.
- *Timeout interne/cancellation sur `get_many_by_prefix` comme fix primaire* :
  traite le symptome, pas la cause (le flavor du runtime EST la cause) ; le
  cancellation token n'est pas proprement supporte en iroh-docs 0.98. Garde
  en defense-en-profondeur, pas en fix primaire. Rejete comme primaire.

**Fallback formel (si multi_thread insuffisant)** : `#[cfg_attr(windows,
ignore = "P2-A-1: iroh-docs 0.98 actor pump + tokio current_thread Windows
scheduler; canonical run = CI Linux; revisit at iroh 1.0 upgrade")]` sur les
tests concernes + entree tech-debt formelle `PATTERNS §P54` avec trigger
« re-evaluer a l'upgrade iroh 0.98→1.0 (Gate 1) ». **C'est l'exemption
formelle** que l'audit exige — mais on essaie le fix d'abord.

**Implications code** : `dispatch_loop.rs` + le miroir worker
`runtime.rs` tests (attribut `multi_thread`), `PATTERNS §P54` (note pump
multi_thread + statut carry CLOSED ou exemption formelle). Zero wire/schema.

---

**Acknowledged review findings (G1)** :

Scoring (renseigne par `sprint73_design_review.md`) :
**D1 ✅, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ✅, D6 ✅.**
Rigor signal G4 satisfait (**2 ⚠️ sur 6** — cible gold 1-2/5 mise a l'echelle).

- **D2 ⚠️** : migration FTS5 par **DROP/recreate** (FTS5 ne supporte pas
  ALTER ADD COLUMN). Decision : **acknowledge + adjust** — l'index est
  **integralement reconstructible** depuis `public_feed` (DROP+recreate +
  `rebuild_from_feed` au boot repopule tout, aucune donnee unique perdue),
  colonnes **UNINDEXED** (pas de gonflement), migration testee sur replica.
  Le ⚠️ reste (recreation table virtuelle > ADD COLUMN).
- **D3 ⚠️** : **defere** un item nomme par la roadmap (« decider
  SearchManifest »). Decision : **acknowledge + adjust** — la decision EST
  prise (defer), grounded sur 7 modeles + l'etat pre-launch + PO-13, et la
  forme correcte (noeud-index opt-in) est **capturee** en design note. Le ⚠️
  trace que l'item est nomme ; il est tranche, pas oublie.

---

## §5 Plan Phase outline A..F

### Phase A — Securite invariant : guardrail AVANT persist (D5) + lot doc menace

P2-RESULT-TEXT-GUARDRAIL-ORDER (priorite haute, traiter tot — c'est la
surface de recuperation sur laquelle S73 construit). Split
`validate_result` pre/post-guardrail ; `default_output_chain` AVANT
`set_task_result` sur **HTTP + validator_loop** ; quorum sur texte agree.
Corriger les claims fausses `THREAT_MODEL §14` + `LOOPBACK §3.1`. **Lot doc
absorbe** : P2-TIER-MODEL (Operator :3001 en tier formel `LOOPBACK §2/§8`),
P2-HARDENING-ROADMAP-META-STALE (re-cadrage §3 + last_validated). **Critere :
guardrail prouve AVANT persist sur les 2 chemins (tests rejet → 0 ligne
persistee) ; claims doc corrigees.** G1 design_review present (gate Phase A).

### Phase B — Dette (reservee) : worker-pump 3/3 (D6) + dette test + durcissement NetworkProvider/Operator

**Non-negociable, non-convertible en feature.** Items :
- **P2-A-1(S71) worker-pump 3/3 MANDATORY** (D6) : fix `multi_thread` sur les
  2 tests E2E + defense timeout + `PATTERNS §P54`. Reproduire Windows natif +
  Docker Linux. **Exit binaire : carry CLOSED par fix (ou exemption formelle
  ecrite si fix insuffisant).**
- **Lot dette test** : P2-TEST-ZOMBIE (`audit_commit_valid_phase_commit`
  de-hardcoder le SHA S70 via repo git fixture) ; P2-OPERATOR-TIMEOUT
  (serialiser le test-group OU timeout configurable — `operator_server`
  spawn+git lent Windows) ; P2-OPERATOR-NO-TEST-RUNNER (infra Vitest
  `factory-operator` : jsdom + mock EventSource pour la logique SSE/gate/
  reconnect/mapping).
- **Durcissement NetworkProvider/Operator (tout traiter S73, arbitrage
  Checkpoint §11)** : P2-POLL-DIAGNOSTIC-LOSS (memoriser `last_err`, la
  surfacer au timeout au lieu d'un « timed out » generique) ; P2-SYNC-FS-ASYNC
  (`resolve_daemon` `std::fs` → `tokio::fs` ou `spawn_blocking`) ;
  **P2-OLLAMA-MODEL-PICKER** (les intentions Ollama/Network heritent
  `claude-opus-4-8[1m]` inexistant → ajouter un selecteur de modele par
  intention non-Claude dans le front Operator `ExecutionChat.tsx` + backend
  `operator_server` per-provider model).

**Critere : worker-pump CLOSED ; 3 dettes test resolues ; NetworkProvider/
Operator durci (3 items).** Phase non-convertible en feature.

### Phase C — Fraicheur : reindex FTS5 a chaud incremental (D1)

`search::upsert_feed_entry()` NEW (`INSERT OR REPLACE` par `feed seq`) +
helper extraction JSON partage avec `rebuild_from_feed` ; appel apres
`insert_feed_entry` Ok dans `feed_sync.rs` (meme lock scope) ; `busy_timeout`
explicite. `rebuild_from_feed` reste chemin de reparation. **Critere :
une entree feed-distante ingeree est cherchable immediatement (test : ingest
→ search hit sans reboot) ; re-ingest idempotent (pas de doublon) ; lecteurs
search non bloques.**

### Phase D — Enrichissement `SearchResult` triplet provenance (D2) + design note SearchManifest (D3)

Migration M17 (DROP/recreate `search_index` + 4 colonnes UNINDEXED) ;
`SearchResult` +4 champs (serde default) ; `index_entry`/`upsert_feed_entry`/
`rebuild_from_feed` extraient le triplet du `ReleasePublishedPayload` ;
`search()` SELECT + `search_handler` JSON. **+ design note**
`.planning/research/s73_searchmanifest_index_node_design.md` (forme correcte
noeud-index opt-in — D3 mitigation). `PATTERNS` (FTS5 hot reindex + triplet)
ecrit ici (docs indissociables du code). **Critere : un hit search porte
repo_url+commit_sha+archive_hash+provenance_hash ; migration M17 verte
(rebuild repopule) ; design note present.** Depend de Phase C (helper
extraction + upsert).

### Phase E — Barre de recherche shell (D4)

`searchBrowse()` + `SearchResponseSchema` Zod dans `api/daemon.ts` ; champ
recherche dedie dans `Browse.tsx` (React Query) ; rendu resultats avec
provenance ; strings FR. **Critere : `web/` tsc+lint+vitest+build+size+scan-en
propres ; la barre tape `GET /api/daemon/search` ; un terme affiche les hits
enrichis ; strings utilisateur en francais.** Depend de Phase D (endpoint
retourne le triplet).

### Phase F — Wrap-up

`sprint73_verification.md` (fail-fast rempli) + `sprint74_audit_plan.md` +
**P2-PREFLIGHT-TRANSITIVE-DEPTH + P2-PREFLIGHT-WIRE-CONTRACT-DEPTH** (amender
les skills/agents preflight : S1b inspecte le Cargo.toml/lock de la version
**precise** ; S4 trace chaque champ wire jusqu'au producteur/consommateur
avant « inchange ») + `PATTERNS.md` (P3 doc lot leger §P53/§P54/§P55 si pas
deja) + memory update + SPRINT_LOG row + CLAUDE.md. **Critere : 100%
fail-fast verts, 2 docs planning, skills preflight amendees, PATTERNS a jour,
memory a jour.**

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 73 — entrent au plan, plus en carry)

| Item | Reports | Phase S73 | Exit condition |
|---|---|---|---|
| **P2-A-1(S71) worker-pump iroh-docs Windows** | **3/3** (S71→S72→S73) | **Phase B** | Fix root-cause `multi_thread` sur les 2 tests E2E, reproduit Windows natif + Docker Linux vert. OU exemption formelle CI-Linux-only ecrite (`#[cfg_attr(windows, ignore)]` + tech-debt `PATTERNS §P54` trigger iroh 1.0). **Plus jamais carry apres S73.** |

### Carries absorbes S73 (12 P2 audit S72)

| Item | Source | Phase S73 | Exit condition |
|---|---|---|---|
| P2-RESULT-TEXT-GUARDRAIL-ORDER | audit S72 (nouveau, headline) | Phase A | Guardrail AVANT persist sur HTTP + validator_loop ; claims THREAT_MODEL §14 + LOOPBACK §3.1 corrigees. |
| P2-TIER-MODEL | audit S72 | Phase A | Operator :3001 en tier formel `LOOPBACK §2/§8` (rows AD2/AD4). |
| P2-HARDENING-ROADMAP-META-STALE | audit S72 (nouveau) | Phase A | Re-cadrage §3 (note « backlog S18-30 clos » + pointeur threat docs vivants) + last_validated. |
| P2-TEST-ZOMBIE | audit S72 | Phase B | `audit_commit_valid_phase_commit` de-hardcode (repo git fixture). |
| P2-OPERATOR-TIMEOUT | audit S72 | Phase B | Serialiser test-group OU timeout configurable ; passe Windows isole. |
| P2-OPERATOR-NO-TEST-RUNNER | audit S72 | Phase B | Infra Vitest `factory-operator` (jsdom + mock EventSource). |
| P2-POLL-DIAGNOSTIC-LOSS | audit S72 | Phase B | `last_err` memorisee + surfacee au timeout ; test mock 401/500. |
| P2-SYNC-FS-ASYNC | audit S72 | Phase B | `resolve_daemon` `tokio::fs`/`spawn_blocking`. |
| P2-OLLAMA-MODEL-PICKER | audit S72 | Phase B | Selecteur modele par intention non-Claude (front Operator + backend per-provider). |
| P2-PREFLIGHT-TRANSITIVE-DEPTH | audit S72 | Phase F | Skill/agent preflight : S1b inspecte le Cargo.toml de la version precise. |
| P2-PREFLIGHT-WIRE-CONTRACT-DEPTH | audit S72 | Phase F | Skill/agent preflight : S4 trace chaque champ wire avant « inchange ». |

### Carries reconduits S73 (exemptes / hors-scope)

| Item | Reports | Justification (renouvelee 2026-06-03) |
|---|---|---|
| P2-A-1 (rand upstream) | exemption | Blocker amont (crate `rand` fix non publie) — hors scope agent. Toujours non publie. |
| P2-AUDIT-2 (iroh transitives pre-release) | herite | Pin iroh 0.98 (decision gelee). Pas d'upgrade 1.0 stable publie. |
| T-NN+2 (iframe Rust-wasm) | exemption | Depend upstream wasm (PATTERNS §P34). Pas de changement amont. |
| P3-OS-1 (operator_server OR duplique) | pre-existant S70 | Trigger = prochaine modif `handle_artifact_draft`. S73 ne touche pas ce handler. Reconduit. |
| P3 doc PATTERNS §P53/§P54/§P55 | audit S72 (10 P3) | Lot leger ; balaye opportunement Phase F si peu couteux, sinon reconduit. |
| LT-5/LT-7 worker quorum E2E | post-v1.0 / S75 | Hors arc recherche. LT-7 Tier 3 worker quorum → S75 (roadmap v5). |

### ROADMAP_COMMITMENTS (Regle 3 — conditions evaluees 2026-06-03)

| LT | Condition | Etat 2026-06-03 |
|---|---|---|
| LT-2 Radicle | tag v1.0 **pousse** vers origin + GitHub Release | **PENDING** — tag v1.0 pose localement, **PAS pousse** (30 ahead, rien pousse). Condition NON remplie → reste latent. |
| LT-1 | reclass pre-v1.0 (DONE S50) | Non declenche. |
| LT-3 / LT-4 | post-v1.0 (Gini>0.70 / biometric) | Aucune sous-condition remplie. Latent. |
| LT-5 | 1er deploiement multi-worker OU v1.0 go-live | Non declenche (S75 = cross-machine, pas deploiement prod). |
| LT-6 | iroh > 0.97 | **RESOLVED S32** (iroh 0.98). |
| LT-7 | pre-v1.0 (Tier 1+2 DONE) | Gate satisfait. Tier 3 worker quorum E2E → S75. Non declenche S73. |

**Aucun trigger ROADMAP_COMMITMENTS ne se declenche pour S73.**

---

## §7 Scope cuts (exhaustif)

Ce que S73 ne fera PAS, et pour quel sprint c'est garde. Chaque item
re-evalue contre le code actuel (G9, §6.2).

| # | Item | Sprint cible | Rationale (factuel) |
|---|---|---|---|
| 1 | SearchManifest reseau-large (op feed + gossip + wire signe) | post-launch (D3) | Defere — feed-local = couverture equivalente en pilote ferme, zero gain SearchManifest + surface Sybil. Design note capture la forme correcte (noeud-index opt-in). PO-13 honore « a terme ». |
| 2 | `sbfb-factory search/open/fork` (Factory tire du reseau) | S74 | Atelier fork. S73 enrichit `SearchResult` (prerequis fork) mais ne code pas les commandes Factory. |
| 3 | Notion de projet cible distinct du repo nexus (`process::repo_root`) | S74 | `repo_root` pointe toujours nexus (G17). S73 cherche, ne fork pas. |
| 4 | `reseau→atelier` : clone `repo_url@commit` ou reconstruction blob | S74 | Le fork (PO-5). S73 fournit le triplet, S74 l'utilise. |
| 5 | Templates etendus (react, pyodide) | S74 | Atelier. Hors recherche. |
| 6 | GPU partage volontaire prouve cross-machine | S75 | S73 cherche local. La preuve cross-machine = S75. |
| 7 | Quorum redundancy>1 cross-MACHINE reel (B-3 etendu) | S75 | Hors recherche. |
| 8 | Sharding pipeline « gros modele » | S76 STRETCH | Jamais avant preuve S75. |
| 9 | Tantivy (moteur d'indexation alternatif) | gate post-S75 si >50K docs | **Gele** (CLAUDE.md:306). FTS5 reste l'engine. Doc fossile recommandant Tantivy = NE PAS rouvrir. |
| 10 | @dev tree-sitter / index symboles / source-only OSS | post-Gate 1 | Hors chemin critique (decision gelee). Pas un livrable recherche S73. |
| 11 | Rate limit per-client sur endpoint search | S74+ si trafic le justifie | Residual T-SEARCH-DOS (THREAT_MODEL §11, « acceptable pre-launch »). La barre shell S73 augmente le trafic interactif → re-evaluer en Phase E ; si le residual L ne tient plus, livrer le rate limiter (sinon reconduit S74). |
| 12 | Webhook/subscribe feed temps reel (SSE/long-poll push) | S74+ | Le reindex chaud D1 est sur le chemin d'ingest existant (pull/gossip), pas un push SSE. Hors-scope. |
| 13 | Streaming token-par-token worker reseau distant | jamais (PO-14) | Decision gelee S72. |
| 14 | Pagination boutons (prev/next) barre recherche | S74+ si corpus le justifie | Corpus petit pre-launch ; champ recherche simple suffit. Differe si mesure. |

---

## §8 Tracabilite scope

Mapping de chaque item « What's NOT » du sprint precedent (S72 §7 scope
cuts) sur son traitement S73.

| Item S72 « What's NOT » (§7) | Sprint + Phase S73 |
|---|---|
| #2 Pont feed-distant → reindex FTS5 a chaud | **S73 Phase C** (D1) |
| #3 Enrichissement `SearchResult` (triplet provenance) | **S73 Phase D** (D2) |
| #4 Barre recherche shell cablee `GET /api/daemon/search` | **S73 Phase E** (D4) |
| #5 Decision SearchManifest | **S73 D3 — TRANCHEE : defer** (feed-local + design note) |
| #1 Onboarding/packaging atelier | Reconduit **S74** (§7 #2-#5) |
| #6 `sbfb-factory search/open/fork` | Reconduit **S74** (§7 #2) |
| #7 Notion projet cible distinct nexus | Reconduit **S74** (§7 #3) |
| #8 Templates etendus (react, pyodide) | Reconduit **S74** (§7 #5) |
| #9 GPU partage cross-machine | Reconduit **S75** (§7 #6) |
| #10 Quorum redundancy>1 cross-MACHINE | Reconduit **S75** (§7 #7) |
| #11 Sharding pipeline | Reconduit **S76 STRETCH** (§7 #8) |
| #12 Streaming token-par-token WAN | **jamais** (§7 #13, PO-14) |
| #13 logprobs/watermark | Reconduit **V2 compute** (post-S75) |
| #14 Dashboard kudos per-task | Reconduit **S75** |
| #16 Routage multi-cloud generaliste | hors roadmap |

---

## §9 Risk register (R1..R7)

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Le fix `multi_thread` (D6) ne resout pas totalement le hang Windows (residu teardown lie au pin iroh-docs 0.98 pre-0.99) | Moyen | Moyen | D6 : reproduire Windows natif + Docker Linux **avant** de declarer CLOSED (`feedback_wsl_before_push`). Si residu → exemption formelle ecrite (`#[cfg_attr(windows, ignore)]` + tech-debt §P54 trigger iroh 1.0). Le fix OU l'exemption ferme le 3/3 — pas de 4e report. |
| R2 | Migration FTS5 M17 (DROP/recreate) perd des donnees ou corrompt l'index | Faible | Eleve | D2 : l'index est **integralement reconstructible** depuis `public_feed` (DROP+recreate + `rebuild_from_feed` repopule). Tester sur DB replica. Colonnes UNINDEXED (pas de gonflement). Pre-launch : aucun index externe a preserver. |
| R3 | Le reindex chaud (D1) ouvre une amplification DoS (reindex couteux par entree sous feed-spam) | Moyen | Moyen | D1 : upsert **incremental** (1 stmt par entree, O(1)), pas rebuild O(N). Borne par la dedup-`entry_hash` + rate-limit GCRA 5 ops/min deja en place (`feed_sync.rs:223-230`). Le rebuild O(N) reste reserve a la reparation explicite. |
| R4 | Le changement d'ordre guardrail (D5) casse des tests d'integration qui lisent un result apres un 400 | Moyen | Moyen | D5 : sur rejet, la ligne n'est plus `completed` — auditer les tests qui assument `result_text` lisible apres rejet ; adapter. Comportement plus correct (pas de fuite de contenu rejete via GET /result). |
| R5 | La barre recherche shell (D4) introduit des strings anglaises (scan-en-strings rejette au commit) | Moyen | Faible | D4 : toutes les strings utilisateur (placeholder, etats vides, erreurs) en **francais** des l'ecriture ; `scan-en-strings.sh` en critere Phase E. |
| R6 | Defer SearchManifest (D3) est conteste a l'audit S73 comme « livrable roadmap manquant » | Faible | Faible | D3 : la decision EST prise (defer), **documentee** (kickoff §4 + design_review ⚠️ + design note de la forme correcte), grounded sur 7 modeles + pre-launch + PO-13. L'audit verifie que c'est tranche, pas oublie. |
| R7 | Scope creep « tout traiter S73 » (NetworkProvider + dette test + securite + recherche) dilue le sprint | Eleve | Moyen | Phasage strict : A securite, B dette (non-convertible), C-E recherche sequentielle. Chaque phase a un critere binaire. Le model-picker (le plus gros du lot dette) est borne au selecteur per-intention, pas une refonte. Si B deborde, A (securite) + C-E (recherche, le theme) restent livrables. |

---

## §10 Audit gate pattern — rappel

- **Phase 0** : DEJA JOUE (§3) — `sprint72_audit_findings.md` (`087e781`),
  verdict PASS (0 P0, 0 P1, 12 P2, 10 P3). Aucun fix requis.
- **Phase de sortie (F)** : produit les deux livrables obligatoires dans un
  commit `docs(sprint73)` : `sprint73_verification.md` (self-report fail-fast
  rempli) + `sprint74_audit_plan.md` (feuille de route session fraiche S74).
  Sans ces deux fichiers, le sprint ne ferme pas (§3.3).
- Phase F amende les skills/agents preflight (P2-PREFLIGHT-*), met a jour
  `docs/rust/PATTERNS.md` et `docs/shell/PATTERNS.md` si nouveaux patterns.

---

## §11 Checkpoint de validation

6 questions (1 par D-choice) pour arbitrage user AVANT le plan detaille.
**Arbitrage rendu 2026-06-03** (AskUserQuestion) sur les 3 bifurcations
produit ; D1/D2/D5/D6 sont research-decisifs (non rebattus).

1. **D1** — Reindex FTS5 **incremental** par `feed seq` (`INSERT OR REPLACE`,
   idempotent) au point `feed_sync`, pas rebuild O(N) : OK pour figer
   l'upsert incremental (research-decisif, evite l'amplification DoS) ?
2. **D2** — Triplet provenance en colonnes **UNINDEXED** (retournees pas
   matchables) + migration FTS5 DROP/recreate (M17) : OK pour UNINDEXED (un
   hash n'est pas un token) + reconstruction depuis le feed (R2) ?
3. **D3 [ARBITRE]** — **Defer SearchManifest**, S73 = feed-local-replique +
   design note de la forme correcte (noeud-index opt-in). **Arbitrage rendu :
   feed-local** (le « plus pousse » techniquement = ne pas coder le broadcast
   que les systemes matures abandonnent ; capturer le design correct).
   Confirme ?
4. **D4 [ARBITRE]** — Barre recherche = **champ dedie dans Browse** (palette
   Ctrl+K reste navigation). **Arbitrage rendu : champ dedie Browse.** OK ?
5. **D5** — Guardrail AVANT persist sur les **2 chemins** (HTTP +
   validator_loop), split validate_result pre/post, claims doc corrigees,
   **Phase A (tot)** : OK pour traiter la securite avant d'etendre la
   recuperation reseau ?
6. **D6** — worker-pump 3/3 = fix **`multi_thread` cross-platform** (pas
   `#[cfg(windows)]`, pas exemption par defaut), fallback exemption formelle
   si insuffisant. **Confirme par la recherche** (root-cause = runtime flavor
   de test). OK ?

**D-dette [ARBITRE]** — Niveau durcissement NetworkProvider S72 :
**arbitrage rendu = tout traiter S73** (poll-diagnostic + sync-fs +
model-picker Ollama inclus, Phase B).
