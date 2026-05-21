# Sprint 67 — Kickoff (Factory Foundation)

**Ecrit** : 2026-05-20 (post-audit gate S66 PASS `3821508`).
**Type** : **sprint impair** — pas de phase dette obligatoire
(Regle 1 §6.2.1 s'applique aux sprints pairs).
Un item 3/3 (Regle 2) a traiter : P2-THREAT-MODEL-FEED-SURFACE.
**Tip master d'entree** : `3821508` (audit findings S66 PASS
0 P0, 0 P1, 3 P2, 1 P3).
**Phase 0 audit Sprint 66** : **DEJA JOUE** — `3821508` PASS.
Aucun fix bloquant requis.
**Version archive** : v2.1 — Protocole Neutre + Factory/RRV.
**Roadmap source** :
`.planning/roadmap_v4_neutral_protocol_factory_rrv.md`.
Sprint 1 sur 3 (Arc 2 Factory + RRV @protocole + Canari, 1/3 —
premier sprint de l'arc).

---

## Sources context7 + WebSearch consultees (pre-gel)

### G2 trigger scan (2026-05-20)

last_validated S66 kickoff 2026-05-19 (1 jour). 6 triggers
evalues :

1. **iroh > 0.98 + iroh-docs > 0.98** : iroh 1.0.0-rc.0,
   iroh-docs 0.99.0. Inchange depuis S66. **Decision** :
   deferred (upgrade iroh 1.0 evaluee a Gate 1 post-S69).

2. **arti-client > 0.41** : arti-client 0.42.0. Inchange.
   **Decision** : deferred. Tor transport pas active.

3. **frost-ed25519 > 3.0** : frost-ed25519 3.0.0. Trigger
   INACTIF (trigger > 3.x, on utilise 3.0.0).

4. **wasmtime LTS bump** : 12 CVE avril 2026 inchanges.
   **Decision** : OS sandbox pour Factory (decision D2 v4).
   wasmtime banni.

5. **pkarr-relay > 0.11.x** : pas de release 0.12. Inchange.

6. **loopback endpoint risky** : S67 ajoute 3 nouvelles routes
   (`/api/daemon/feed/entries`, `/api/daemon/search`,
   `/api/v1/project/{id}/manifest`). Ces routes sont readables
   (GET), pas write, et protegees par le bearer token existant.
   Pas de changement de tier. Documenter dans LOOPBACK_ENDPOINTS.

### context7 queries

1. **context7 `/sqlite/sqlite`** (2026-05-20) : FTS5 CREATE
   VIRTUAL TABLE syntax confirme. Tokenizer `unicode61` par
   defaut. `rank` column via bm25(). `snippet()` pour excerpts.
   `highlight()` pour mark tags. Pas de fuzzy natif (limitation
   acceptee, cf. D1 Tantivy gate post-S75).

2. **context7 `/rusqlite/rusqlite`** (2026-05-20) : Feature
   `bundled` active FTS5 via `-DSQLITE_ENABLE_FTS5`. Confirme :
   le `Cargo.toml` workspace actuel (`rusqlite = { version =
   "0.36", features = ["bundled"] }`) supporte FTS5 sans
   changement de dep. `prepare_cached()` recommande pour les
   queries frequentes.

3. **context7 `/websites/rs_rusqlite_0_39_0_rusqlite`**
   (2026-05-20) : Virtual table API via `vtab` module. FTS5
   n'a pas besoin du module `vtab` cote Rust — les queries
   FTS5 sont du SQL standard (`SELECT ... FROM fts_table
   WHERE fts_table MATCH ?`). Pas de code C custom requis.

### WebSearch queries

1. **WebSearch "rusqlite 0.36 bundled FTS5 SQLITE_ENABLE_FTS5"**
   (2026-05-20) : libsqlite3-sys build.rs confirme que
   `bundled` active `-DSQLITE_ENABLE_FTS5` parmi les flags.
   FTS5 est deja disponible avec la config actuelle.
   Source : github.com/rusqlite/rusqlite/blob/master/libsqlite3-sys/build.rs

2. **WebSearch "SQLite FTS5 tokenizer unicode61 performance"**
   (2026-05-20) : unicode61 est case-insensitive + diacritics
   removal (Unicode 6.1). Performance adequate pour < 50K docs
   (objectif pre-launch). Pas de benchmark p95 public specifique
   mais empiriquement < 5ms sur 10K documents (ref: sqlite.org
   forum, Sling Academy guide).
   Sources : sqlite.org/fts5.html, audrey.feldroy.com/articles/2025-01-13-SQLite-FTS5-Tokenizers

3. **WebSearch "clap CLI Rust 4.x derive subcommands 2026"**
   (2026-05-20) : clap 4.5+ stable, derive API avec
   `#[derive(Parser)]`. Subcommands via `#[command(subcommand)]`.
   Feature `derive` + `env` pour environment variables.
   Source : docs.rs/clap, crates.io/crates/clap

4. **WebSearch "BLAKE3 Rust crate latest version 2026"**
   (2026-05-20) : blake3 1.8.3 (2026-01-08). Deja utilise
   dans le projet via nexus-core-rs (blake3_hash function).
   Pas de breaking change depuis 1.6.x.
   Source : crates.io/crates/blake3

5. **WebSearch "Backstage Software Catalog YAML schema"**
   (2026-05-20) : Backstage utilise catalog-info.yaml avec
   kind/metadata/spec structure. Inspiration pour SBFB.json v2
   (schema_version + metadata + bridge config). Backstage
   supporte custom kinds et entity providers.
   Source : backstage.io/docs/features/software-catalog/descriptor-format

### G9 codebase factual scan (lecture directe code)

1. **deploy.rs** (l.119-128) : `sbfb.node_id != state.node_id`
   verification stricte. `SbfbJson` struct (l.543-548) :
   `node_id: String` obligatoire. La migration D3 (node_id
   optionnel) necessite `node_id: Option<String>` +
   `#[serde(default)]` + suppression du bloc de verification
   (l.119-128). Changement ~10 lignes.

2. **public_feed.rs** (l.52-60) : `PublicFeedOperation` enum
   avec `ReleasePublished` et `SourceBecameStale`. Commentaire
   l.52-54 mentionne `CuratorVouched` comme future variant.
   `try_parse_op` (l.110-112) parse via `serde_json::from_value`.
   Ajout CuratorVouched/CuratorDisendorsed = nouvelles variantes
   + nouveaux payloads + tests validation. ~100 lignes.

3. **http.rs routes** : 68 routes existantes. Pas de
   `/api/daemon/feed/entries` ni `/api/daemon/search`. Les
   routes feed existantes : `/api/daemon/feed/ticket`,
   `/api/daemon/feed/join`, `/api/daemon/feed/status`,
   `/api/daemon/feed/insert`, `/api/daemon/feed/cursor`.

4. **db.rs** : M14 est la derniere migration (key_rotations,
   S66 Phase D). `get_feed_entries()` (l.826) retourne toutes
   les entries. `get_feed_entries_after_seq()` (l.780) supporte
   la pagination. FTS5 necessite M15 (nouvelle virtual table).

5. **Cargo.toml workspace** : rusqlite 0.36 features=["bundled"]
   — FTS5 deja compile. Pas de nouvelle dep requise pour FTS5.

6. **examples/sbfb-explorer/SBFB.json** et
   **examples/sbfb-ideas/SBFB.json** : contiennent `"node_id":
   "PLACEHOLDER"`. A migrer vers SBFB.json v2 (sans node_id).

---

## S1 Constat d'entree

### S1.1 D'ou on part

Sprint 66 a livre la durabilite du daemon : iroh data_dir
persistent (FsStore + blobs + feed namespace + storage namespace
survivent aux restarts), provenance 3 etats (absent/verified/
failed), orphan recovery SQLite, RevocationCache persistent,
THREAT_MODEL feed section (T-FEED-1..4). Arc 1 Fondations
(S65 contrat public + S66 durabilite) est COMPLET.

S67 ouvre l'Arc 2 Factory + RRV @protocole + Canari. C'est le premier
sprint de code nouveau depuis S65 (S66 etait consolidation/
hardening). L'objectif est de poser les fondations techniques
de 3 composants : les primitives daemon neutres manquantes, le
search FTS5 @protocole, et le crate sbfb-factory en tant qu'outil
client externe.

Le daemon doit devenir un "tuyau stupide" fournissant des
primitives generiques. Factory et RRV sont des outils clients
qui consomment ces primitives via HTTP loopback.

### S1.2 Ancrage roadmap v4

Position : Arc 2 Factory + RRV @protocole + Canari, sprint 1/3.
Dependances amont : S66 DONE (persistence prerequis FTS5 +
Factory). Dependances aval : S68 (FTS5 + sbfb-manifest prerequis
Proof Cards), S69 (Factory prerequis Babel).

### S1.3 Compteurs tests entree (tip `3821508`)

| Suite | Count |
|---|---|
| Rust nextest | 1349 |
| Vitest | 269 |
| size-limit | 6/6 |
| **Total** | **~1624** |

### S1.4 Pre-launch protocol policy (rappel)

- `FEED_FORMAT_VERSION` reste a 1. Ajouter `CuratorVouched` et
  `CuratorDisendorsed` comme nouvelles operations dans
  `PublicFeedOperation` ne bump PAS `FEED_FORMAT_VERSION`. Les
  noeuds anciens stockent et propagent les ops inconnues via
  `serde_json::Value` (pattern raw-op P51).
- `*_ANNOUNCEMENT_VERSION` reste a 1. `SearchManifest` n'est
  PAS deploye en S67 (S70+). Pas de version a bumper.
- `SBFB.json` passe de v1 a v2 (`schema_version: 2`). C'est une
  redefinition du canonical, pas un bump compat (pre-launch).
  Les anciens SBFB.json v1 restent parsables via `#[serde(default)]`.
- `#[serde(default)]` reste legitime pour robustesse runtime.
- Pas de tests "legacy decode" pre-launch.

---

## S2 Goal

Sprint 67 pose les fondations techniques de l'Arc 2 : primitives
daemon neutres manquantes (feed read paginee, node_id optionnel,
CuratorVouched/Disendorsed, SBFB.json v2), search FTS5 local
@protocole (migration M15 + module search.rs + API endpoint),
et crate sbfb-factory MVP (sbfb-manifest lib + sbfb-factory CLI
avec create + validate + scan-secrets). Le daemon reste un tuyau
stupide. Factory est un outil client externe.

**Critere SMART : toutes les rows fail-fast vertes au
verification.md, mesure binaire au Phase E wrap-up.**

---

## S3 Phase 0 — Audit gate Sprint 66

Verdict : PASS (`3821508`). 0 P0, 0 P1, 3 P2, 1 P3. Aucun fix
bloquant requis. Les 3 P2 (P2-66-1 tests feed republish sans
assertion iroh-docs, P2-66-2 BlobStore enum pattern undocumented,
P2-66-3 Phase A body format) sont logges en carry/dette.

---

## S4 Decisions Day 0 (D1..D5 gelees)

### D1 — FTS5 search @protocole dans le daemon

**Sources consultees** :
- context7 `/sqlite/sqlite` queried 2026-05-20 : FTS5 CREATE
  VIRTUAL TABLE, tokenizer unicode61, rank/bm25(), snippet().
- context7 `/rusqlite/rusqlite` queried 2026-05-20 : `bundled`
  feature active FTS5. `prepare_cached()` pour queries.
- WebSearch "rusqlite 0.36 bundled FTS5" 2026-05-20 :
  github.com/rusqlite/rusqlite build.rs confirme flags.
- WebSearch "SQLite FTS5 unicode61 performance" 2026-05-20 :
  sqlite.org/fts5.html — adequate < 50K docs.
- Code OSS SQLite FTS5 : sqlite.org/fts5.html (reference spec).
- Code OSS Tantivy : github.com/quickwit-oss/tantivy (alternative
  evaluee).
- Code OSS MeiliSearch : github.com/meilisearch/meilisearch
  (alternative evaluee).

**Retenu** : FTS5 via rusqlite bundled (zero dep ajoutee).

L'index FTS5 est une virtual table SQLite dans coordinator.db.
Migration M15 cree `search_index` FTS5 virtual table indexant
les BrowseEntries (project_name, category, description) et les
FeedEntries (op_type, payload text). Le tokenizer `unicode61`
fournit la case-insensitivity et diacritics removal. Les queries
utilisent MATCH + rank via bm25(). Un module `search.rs` dans
nexus-coordinator-rs encapsule l'insertion/query. L'endpoint
`GET /api/daemon/search?q=...&limit=20&offset=0` retourne des
resultats avec score et excerpt (via `snippet()`).

C'est la couche service daemon (SYNTHESIS §2.6 couche 2, pas la
couche protocole). Pas de wire format SearchManifest en S67 —
c'est S70+.

**Rejete** :
- **Tantivy** (~0.22) : crate ~5MB compile, index fichier
  separe, nouvelle dep significative. FTS5 est deja disponible
  sans dep ajoutee via rusqlite bundled. Tantivy reserve pour
  gate post-S75 si > 50K docs ou si fuzzy/stemming deviennent
  bloquants. Source : SYNTHESIS §4.3, roadmap v4 D1.
- **MeiliSearch** : serveur externe separe, overhead operationnel
  (process daemon + process MeiliSearch). Incompatible avec le
  pattern "daemon minimal" et la cible pilote ferme 2-3 personnes.
  Source : github.com/meilisearch/meilisearch architecture doc.
- **sqlite-vec** : embeddings vector search. Hors scope MVP
  (pas de modele embeddings local configure). Feature flag P1
  post-pilote. Source : SYNTHESIS §6.2.

**Implications code** : `crates/nexus-coordinator-rs/src/db.rs`
(M15 migration), `crates/nexus-coordinator-rs/src/search.rs`
(NEW), `crates/nexus-shell-daemon/src/http.rs` (endpoint search),
`crates/nexus-shell-daemon/src/runtime.rs` (boot indexation).

### D2 — sbfb-manifest crate partage + SBFB.json v2

**Sources consultees** :
- WebSearch "Backstage Software Catalog YAML schema" 2026-05-20 :
  backstage.io — entity model metadata/spec/relations. Inspiration
  pour SBFB.json v2 structure.
- Code OSS Backstage : github.com/backstage/backstage
  `catalog-info.yaml` format. Pattern schema_version + kind +
  metadata + spec.
- Code OSS F-Droid : gitlab.com/fdroid/fdroiddata, metadata
  format YAML. Pattern package/version/antiFeatures.
- SYNTHESIS §3.7-§3.8 : struct Rust proposee (Annexe A), champs
  v2, compat descendante.
- deploy.rs (l.543-557) : `SbfbJson` struct actuelle. Lecture
  factuelle du code.
- WebSearch "serde default backward compatible JSON schema
  evolution Rust" 2026-05-20 : docs.rs/serde — `#[serde(default)]`
  pattern pour evolution sans breaking.

**Retenu** : crate `sbfb-manifest` dans le workspace, lib pure
sans dep reseau.

`sbfb-manifest` est un crate Rust pur (deps: serde, serde_json,
thiserror) qui definit `SbfbManifest` (struct parsant SBFB.json
v1 et v2), `validate()` (verifie schema_version, champs requis
v2, bridge methods allowlist), et `BridgeMethodAllowlist` (enum
des methodes bridge connues). Le crate est utilise par le daemon
(dans deploy.rs pour validation au deploy) ET par sbfb-factory
(pour generation et validation locale). C'est le point de
partage entre daemon et Factory (SYNTHESIS §3.2 architecture).

`node_id` devient `Option<String>` avec `#[serde(default)]`.
La verification `sbfb.node_id != state.node_id` dans deploy.rs
(l.119-128) est supprimee. Un warning log est emis si `node_id`
est present (deprecation signal). Les exemples existants
(sbfb-explorer, sbfb-ideas) migrent vers v2 (sans node_id).

**Rejete** :
- **Validation inline dans deploy.rs** (status quo) : la
  validation serait dupliquee entre daemon et Factory. Un crate
  partage est le pattern OSS convergent (Backstage catalog-info,
  F-Droid metadata, AT Proto Lexicon schemas). Source : SYNTHESIS
  §3.2, roadmap v4 H4.
- **Schema YAML au lieu de JSON** : SBFB.json est historique.
  Changer le format = migration + changement dans le deploy path
  + changement dans les exemples. Benefice marginal. JSON reste.
  Source : decision conservatrice pre-launch.
- **Validation stricte v2 only** : casserait les SBFB.json v1
  existants. La compat descendante via `#[serde(default)]` est
  explicitement prescrite par le pre-launch protocol. Source :
  CLAUDE.md §Pre-launch protocol policy.

**Implications code** : `crates/sbfb-manifest/` (NEW crate),
`crates/nexus-shell-daemon/src/deploy.rs` (refactor SbfbJson →
import sbfb-manifest), `Cargo.toml` workspace (nouveau membre),
`examples/sbfb-explorer/SBFB.json` (migration v2),
`examples/sbfb-ideas/SBFB.json` (migration v2).

### D3 — CuratorVouched/CuratorDisendorsed feed operations

**Sources consultees** :
- SYNTHESIS §2.5 : CuratorVouched/CuratorDisendorsed identifies
  comme P0 dans les primitives daemon manquantes.
- Code local public_feed.rs (l.52-60) : commentaire mentionne
  explicitement `CuratorVouched` comme future variant.
- Code OSS SSB : github.com/ssbc/ssb-db2 — feed append-only
  avec types de messages extensibles. Pattern convergent : le feed
  ne connait pas la semantique, les clients interpretent.
- Code OSS AT Protocol : github.com/bluesky-social/atproto —
  records Lexicon extensibles dans repos. Le PDS stocke et
  propage sans interpreter.
- SYNTHESIS §12.2 pattern #3 : "La creation de contenu est une
  primitive generique : le protocole offre write(bytes),
  l'application decide write(what)."

**Retenu** : 2 nouvelles variantes dans `PublicFeedOperation`.

`CuratorVouched` et `CuratorDisendorsed` sont des operations
feed qui enregistrent qu'un curator endorsement/disendorsement
a ete emis. Le payload contient : `curator_pubkey`, `project_id`,
`reason` (optionnel pour vouch, obligatoire pour disendorsement).
La semantique est minimale : le feed log l'evenement, les clients
l'interpretent. C'est coherent avec la decision D9 (CuratorVouched
minimal) de la roadmap v4.

Les variantes sont ajoutees a l'enum `PublicFeedOperation` dans
public_feed.rs. `validate_feed_operation` est etendu pour valider
les champs (hex-64 pubkey, hex-64 project_id, reason length
check). Les noeuds anciens stockent et propagent ces ops sans
les interpreter (pattern raw-op P51).

**Rejete** :
- **Ops CuratorVouched dans les curator lists DashMap** (status
  quo) : les listes DashMap sont des snapshots, pas un log. Le
  feed est le log verifiable. Un endorsement dans le feed est
  une trace publique, auditable, verifiable par signature. Les
  listes DashMap sont un cache derive. Source : SYNTHESIS §12.2.
- **Wire format dedie (pas dans le feed)** : un type de message
  gossip separe serait une nouvelle surface protocole a geler.
  Le feed extensible est concu pour absorber les nouvelles
  operations. Source : decision D4 roadmap v4.
- **Reporter a S70 avec SearchManifest** : les Proof Cards S68
  ont besoin des vouches dans le feed pour calculer le score de
  curation. Reporter = bloquer S68. Source : SYNTHESIS §3.4,
  roadmap v4 H5.

**Implications code** : `crates/nexus-coordinator-rs/src/public_feed.rs`
(variantes + payloads + validation), `crates/nexus-coordinator-rs/src/feed_materializer.rs`
(materialization des nouvelles ops),
`crates/nexus-shell-daemon/src/http.rs` (test handler si
expose via API).

### D4 — Feed entries read paginee (GET /api/daemon/feed/entries)

**Sources consultees** :
- SYNTHESIS §2.5 : feed read paginee identifiee comme P0 dans
  les primitives manquantes.
- Code local db.rs (l.780-826) : `get_feed_entries_after_seq()`
  et `get_feed_entries()` existent deja. La pagination est
  triviale (LIMIT/OFFSET SQL).
- Code OSS AT Protocol : github.com/bluesky-social/atproto —
  `com.atproto.sync.getRepo` + `com.atproto.repo.listRecords`
  patterns de pagination cursor-based.
- Code OSS SSB : github.com/ssbc/ssb-db2 — `createLogStream`
  avec opts `limit`, `since` (sequence-based).
- SYNTHESIS §12.2 pattern #7 : "Evenements comme contrat
  d'integration : les protocoles matures exposent un flux
  d'evenements que les outils consomment."

**Retenu** : endpoint GET /api/daemon/feed/entries avec
pagination sequence-based.

L'endpoint retourne les entries feed paginee par `after_seq`
(cursor) et `limit` (default 50, max 200). Parametres
optionnels : `project_id` (filtre par projet), `op_type`
(filtre par type d'operation). Le format de reponse est un
JSON avec `entries: [...]`, `total: N`, `next_seq: Option<u64>`.

Le code backend existe deja : `get_feed_entries_after_seq()`
dans db.rs. L'endpoint est un thin wrapper HTTP qui deserialise
les query params et appelle la DB.

**Rejete** :
- **SSE/WebSocket live stream** : over-engineered pour le MVP.
  Le polling par les clients Factory/RRV est suffisant pour S67.
  SSE prevu comme P2 pour S68+ (SYNTHESIS §2.5, item P2
  "Webhook/subscribe feed"). Source : decision D15 (priorite
  curseur vs stream).
- **GraphQL** : surface d'API non standard, overhead
  implementation, pas de prior art dans les protocoles P2P
  compares (IPFS/SSB/AT Proto utilisent REST ou RPC). Source :
  SYNTHESIS §12.1.
- **Pas d'endpoint (lire via feed/status + feed/cursor)** : les
  endpoints existants ne retournent pas le contenu des entries.
  Factory et RRV ont besoin du contenu pour indexer/afficher.
  Source : SYNTHESIS §2.5 P0 justification.

**Implications code** : `crates/nexus-shell-daemon/src/http.rs`
(handler GET + query params), `crates/nexus-coordinator-rs/src/db.rs`
(ajout filtre project_id/op_type si non existant).

### D5 — sbfb-factory CLI crate externe avec create + validate

**Sources consultees** :
- SYNTHESIS §3.2 : architecture cible sbfb-factory, deps, structure.
- SYNTHESIS §3.3 : moteur de templates (copie fichiers +
  substitution variables, pas Tera/Handlebars).
- SYNTHESIS §3.6 : flux `sbfb-factory create` etape par etape.
- Code OSS Copier : github.com/copier-org/copier — Python
  template engine, copy+substitution pattern. Reference template
  mais la logique interne est plus simple en Rust (include_str!).
- Code OSS Backstage scaffolder :
  github.com/backstage/backstage/tree/master/plugins/scaffolder
  — steps, parameters, dry-run. Inspiration UX mais trop
  complexe pour MVP.
- Code OSS Cookiecutter :
  github.com/cookiecutter/cookiecutter — Python, template
  rendering Jinja2. Meme pattern copie+substitution mais dep
  lourde.
- WebSearch "clap CLI Rust 4.x derive subcommands" 2026-05-20 :
  clap 4.5+ stable, derive API. Source : docs.rs/clap.
- WebSearch "BLAKE3 Rust crate" 2026-05-20 : blake3 1.8.3.
  Deja utilise dans le projet. Source : crates.io/crates/blake3.

**Retenu** : crate sbfb-factory dans le workspace, binaire CLI.

S67 livre le MVP minimal : `sbfb-factory create --template
static --name <name>` (copie template + substitution variables
+ generation SBFB.json v2 + factory.template.lock +
factory.provenance.json + git init) et `sbfb-factory validate
<path>` (validation manifest via sbfb-manifest). Le template
embarque est `static` (HTML pur + sbfb-bridge.js). Un second
template `static-storage` (avec bridge storage) est stretch
si le temps le permet.

Le moteur de templates est interne (include_str! + String
replace) — pas de dep Tera/Handlebars/Copier. Les artefacts
generes incluent : SBFB.json v2, factory.template.lock (template
id/version/hash BLAKE3), factory.provenance.json (generation
lineage hash + signature Ed25519). Un scan-secrets basique
(regex patterns hardcodes) est integre dans `validate`.

sbfb-factory depend de sbfb-manifest (validation) et blake3
(hashing). Il ne depend PAS de nexus-shell-daemon-core ou
nexus-coordinator-rs (decision D2 v4 — Factory hors daemon).
Il communiquera avec le daemon via HTTP en S68+ (publish path).

**Rejete** :
- **Factory dans le daemon** (module interne) : viole la
  neutralite protocolaire (SYNTHESIS §2.1). Le daemon ne doit
  pas connaitre la semantique Factory. Decision D2 gelee
  roadmap v4. Source : SYNTHESIS §3.1, prior art IPFS Cluster,
  Radicle httpd.
- **Copier binaire externe** : dep Python, overhead install,
  incompatible stack Rust-first. Le moteur interne est plus
  simple (< 200 lignes) et controllable. Source : SYNTHESIS §10.1
  Q3.
- **Tera/Handlebars** : overkill pour la substitution simple
  `{{name}}` / `{{version}}`. Les templates SBFB sont des
  fichiers statiques avec quelques placeholders. Source :
  SYNTHESIS §3.3.

**Implications code** : `crates/sbfb-factory/` (NEW crate),
`crates/sbfb-factory/src/main.rs` (clap CLI),
`crates/sbfb-factory/src/templates/` (templates embarques),
`crates/sbfb-factory/src/template_engine.rs` (copie+substitution),
`crates/sbfb-factory/src/secret_scanner.rs` (regex scan),
`Cargo.toml` workspace (nouveau membre).

---

**Acknowledged review findings (G1)** :

Scoring : D1 ok, D2 ok, D3 ok, D4 ok, D5 warning.
Rigor signal G4 satisfait (1 warning sur 5).

D5 warning : la source Copier est la doc PyPI, pas un read du
code source Copier. Le moteur interne est simple et ne reproduit
pas Copier — c'est une simple copie+replace. Le warning est
acknowledge : la comparaison est au niveau du pattern (copie de
fichiers + substitution), pas de l'implementation. Le code
Factory est ecrit from scratch.

---

## S5 Plan Phase outline A..E

### Phase A — Primitives daemon neutres

Scope : sbfb-manifest crate (struct + validation + bridge
allowlist), SBFB.json v2 migration (deploy.rs node_id optionnel,
exemples migres), CuratorVouched/CuratorDisendorsed operations
feed, GET /api/daemon/feed/entries endpoint pagine.

### Phase B — FTS5 search @protocole

Scope : migration M15 FTS5 virtual table, module search.rs
(indexation browse+feed, query), endpoint GET /api/daemon/search,
bridge method `search` (schema Zod + dispatch + SDK),
THREAT_MODEL feed surface 3/3 (P2-THREAT-MODEL-FEED-SURFACE
MANDATORY).

### Phase C — sbfb-factory crate + template engine

Scope : sbfb-factory crate structure (clap CLI + template engine
+ secret scanner), template `static` embarque, commande `create`,
commande `validate`, factory.template.lock generation, tests
unitaires.

### Phase D — Factory provenance + dette residuelle

Scope : factory.provenance.json generation (hash BLAKE3 +
signature Ed25519), P2-66-2 BlobStore pattern P52 dans
PATTERNS.md, P2-66-1 documentation (tests feed republish gap
acknowledgement), test integration sbfb-factory determinisme
(meme inputs = meme hash).

### Phase E — Wrap-up + verification

Scope : verification.md fail-fast, sprint68_audit_plan.md,
CLAUDE.md + SPRINT_LOG.md mise a jour.

---

## S6 Items carry/dette

### Items 3/3 (traitement Sprint 67)

| Item | Reports | Phase S67 | Exit condition |
|---|---|---|---|
| P2-THREAT-MODEL-FEED-SURFACE | 3/3 | Phase B | THREAT_MODEL.md §10 enrichi (search surface + CuratorVouched surface), `grep -q "T-SEARCH\|T-CURATOR-VOUCH" docs/security/THREAT_MODEL.md` |

### Carry absorbes S67

| Item | Reports | Phase S67 | Exit condition |
|---|---|---|---|
| P2-66-2 BlobStore pattern undoc | 1/3 | Phase D | P52 dans PATTERNS.md, `grep -q "P52" docs/rust/PATTERNS.md` |
| P2-66-1 tests feed republish gap | 1/3 | Phase D | Acknowledgement dans PATTERNS.md (documented known limitation) |

### Carries reconduits S68

| Item | Reports | Justification |
|---|---|---|
| P2-A-1 rand blocker | exemption externe | upstream rand 0.9 non publie. Dep transitive via iroh 0.98. Pas d'action possible sans upgrade iroh. |
| P2-AUDIT-2 iroh transitives | exemption externe | herite du pin iroh 0.98. Pre-release transitives (iroh-gossip-proto etc.) inchangees. Evaluate a Gate 1. |
| P2-G-1 exe lock intermittent | monitoring | Non reproductible depuis S62 (4 sprints). Monitoring continu. Si reproductible, escalade immediate. |
| T-NN+2 iframe Rust-wasm | bloque upstream | Toolchain wasm-pack + wasm-bindgen gaps non resolus. Pas de changement upstream depuis S22. Hors scope Factory (Factory = CLI Rust, pas wasm). |
| P2-66-3 Phase A body format | 1/3 → CLOSED | Hook fix en place depuis S66 Phase B. Cause racine corrigee. Pas d'action S67. CLOSED. |

### Attention 3/3 S68

Aucun item n'atteindra 3/3 au S68 (les carries reconduits sont
tous en exemption externe ou monitoring, pas incrementes).

---

## S7 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | Preview ephemere (POST /api/v1/preview/load) | S68 Phase A | Depend de sbfb-factory publish path, pas requis pour create+validate MVP |
| 2 | Diff engine avance sbfb-factory | S68+ | La commande `sbfb-factory diff` est stretch S68, pas requis pour Factory foundation |
| 3 | Page React /factory | S68+ CLI suffit | Decision PO : CLI-first, UI optionnelle (roadmap v4 ajustable) |
| 4 | Proof Cards computation | S68 Phase A | Depend de FTS5 + sbfb-manifest (sortie S67), pas dans ce sprint |
| 5 | SearchManifest wire format | S70+ | Couche protocole (wire format gele), pas couche service daemon. Pas en S67 |
| 6 | Babel dogfood via Factory | S69 | FlowUP cree Babel avec Factory ; le sprint supporte publish/proofs/pilote ferme |
| 7 | @dev index dans sbfb-factory (tree-sitter) | S70+ par defaut | Non requis Gate 1 ; stretch S68-S69 seulement si zero impact @protocole/publish |
| 8 | Bridge method `proof_card_get` | S68+ | Proof Cards pas en S67 |
| 9 | Template `react-vite` | S69+ | 2 templates max S67 (static + stretch static-storage). 3eme template deplacable |
| 10 | Factory audit log (JSONL) | S68+ | Pas requis pour create+validate MVP. Requis pour publish gate S68 |
| 11 | CuratorVouched UI dans le shell | S70+ (Gouvernance Full UI) | Decision D9 v4 : minimal en S65-S67, Full UI en S70 |
| 12 | Publish path sbfb-factory → daemon | S68+ | S67 = create + validate local seulement. Publish = S68 publish gate |
| 13 | Feed format version bump | post-launch | Pre-launch protocol policy. FEED_FORMAT_VERSION = 1 |
| 14 | Fuzzing cargo-fuzz/proptest | post-audit | Hors scope feature sprint |

---

## S8 Tracabilite scope

| Item S66 "What's NOT" | Sprint + Phase S67 |
|---|---|
| CuratorVouched/CuratorDisendorsed | Phase A S67 (D3 gelee) |
| BuildQuorumReached feed | Reconduit S68+ (#non-requis pour Factory MVP) |
| Quarantine feed hot path | Reconduit S68+ (#priorite Proof Cards) |
| Age witness gate | Reconduit S70+ (#gouvernance) |
| T1 CONFIRM_PROMPT complet | Reconduit post-pilote S69+ |
| SBFB.json v2 code | Phase A S67 (D2 gelee — sbfb-manifest crate) |
| node_id deprecation deploy.rs | Phase A S67 (D2 gelee — node_id optionnel) |
| Factory template scaffold | Phase C S67 (D5 gelee — sbfb-factory create) |
| Fuzzing cargo-fuzz/proptest | Reconduit post-audit (scope cut #14) |
| CLI verify-release | Reconduit S68+ (#publish gate) |
| VerificationDetail niveau 3 | Reconduit S68+ (#Proof Cards) |
| Playwright E2E re-ecriture | Reconduit S69 (#pilote ferme) |
| Feed format version bump | Reconduit post-launch (scope cut #13) |
| Multi-curator trust overlay | Reconduit S70+ (#gouvernance) |

---

## S9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | FTS5 indexation lente sur gros corpus | Low | Medium | Volume < 500 entries pre-launch. Indexation incrementale. Test performance Phase B |
| R2 | sbfb-factory crate deps conflict workspace | Medium | Medium | Deps minimales (serde, clap, blake3). Pas de dep nexus-shell-daemon-core. Test compilation workspace entier |
| R3 | SBFB.json v2 migration casse exemples existants | Low | High | `#[serde(default)]` compat descendante. Tests de parsing v1 dans v2 struct |
| R4 | CuratorVouched semantique trop legere pour Proof Cards S68 | Medium | Low | Payload minimal suffisant (pubkey + project_id). Extension en S68 si besoin |
| R5 | Template engine trop simple (pas de conditionals) | Medium | Low | Scope cut : S67 = copie + replace. Logique conditionnelle en S68+ si requis |
| R6 | iroh 0.98 bugs blocker persistence (herite S66) | Low | High | E2E restart test S66 vert. Monitoring continu. Decision upgrade a Gate 1 |
| R7 | 3 crates nouveaux dans le workspace (sbfb-manifest + sbfb-factory + search module) alourdit CI | Low | Low | CI compile le workspace entier (deja le cas). Les crates sont petits |
| R8 | Derive @dev/source-only avant pilote | Medium | Medium | @dev reste scope cut S70+ ; Gate 1 = @protocole + Proof Cards + publish + Babel dogfood |

---

## S10 Audit gate pattern — rappel

Phase 0 S66 : DEJA JOUEE. Verdict PASS (`3821508`).

Phase E du sprint devra produire :
- `sprint67_verification.md` (self-report fail-fast)
- `sprint68_audit_plan.md` (plan pour Phase 0 S68)
- Mise a jour `docs/rust/PATTERNS.md` si nouveaux patterns
  (sbfb-manifest, search.rs, template engine)
- Mise a jour `docs/security/THREAT_MODEL.md` (search surface
  + CuratorVouched surface — P2-THREAT-MODEL-FEED-SURFACE 3/3
  MANDATORY)

---

## S11 Checkpoint de validation

1. **D1 — FTS5 vs Tantivy** : FTS5 indexe les BrowseEntries et
   FeedEntries existants, sans nouvelle dep. Tantivy est reserve
   pour post-S75. Acceptes-tu que le search MVP n'ait pas de
   fuzzy matching ni de stemming multilingue ?

2. **D2 — sbfb-manifest crate** : le crate est partage entre
   daemon et Factory. node_id devient optionnel dans SBFB.json.
   Acceptes-tu que les exemples existants (sbfb-explorer,
   sbfb-ideas) migrent vers SBFB.json v2 dans ce sprint ?

3. **D3 — CuratorVouched minimal** : les operations sont dans le
   feed (log verifiable) mais pas dans les listes DashMap (UI).
   L'UI curator vouch est en S70. Acceptes-tu cette separation
   log/UI ?

4. **D4 — Feed entries read** : l'endpoint est read-only, pagine
   par sequence. Pas de live stream SSE/WebSocket en S67.
   Acceptes-tu le polling pour Factory/RRV MVP ?

5. **D5 — sbfb-factory create + validate** : le CLI genere des
   projets et valide des manifests localement. Pas de publish
   path en S67. Acceptes-tu que Factory S67 soit local-only
   (create + validate, pas publish) ?
