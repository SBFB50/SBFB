# Recherche RRV (Recherche Reseau Verifiable) — Sprints 70-72

**Date:** 2026-05-18
**Statut:** recherche approfondie, alimente les roadmaps S70-72
**Scope:** S70 RRV LocalOnly, S71 Proof Cards, S72 SearchManifest Opt-In
**Confiance globale:** MEDIUM-HIGH (code source lu, prior research croisse,
recherche externe sur ecosysteme effectuee)

**Documents prealables lus:**
- `.planning/codebase/ARCHITECTURE.md`
- `.planning/codebase/STRUCTURE.md`
- `.planning/codebase/protocol_wire_formats.md`
- `.planning/codebase/APPS_BRIDGE_DOCS.md`
- `.planning/research/chat_ia_reseau_recherche_reseau_rnd.md`
- `.planning/research/rrv_scoped_search_compute_groups.md`
- `.planning/research/sbfb_project_factory_rrv_oss_research.md`
- `.planning/research/p2panda_public_protocol_briques.md`
- Code source: `browse.rs`, `public_feed.rs`, `publish.rs`, `apps.rs`, `db.rs`
- `docs/protocol/PUBLIC_FEED_SPEC.md`

---

## 1. Etat actuel — donnees locales cherchables

### 1.1 Inventaire des donnees existantes

L'analyse du code source revele 7 familles de donnees deja presentes
localement sur chaque noeud daemon:

| Famille | Stockage | Champs cles | Source code |
|---------|----------|-------------|-------------|
| **BrowseEntry** | DashMap in-memory (BrowseAggregator) | project_id, project_name, category, description, curator_pubkey, curator_name, source, status, archive_hash, repo_url, provenance_hash, is_open_source | `browse.rs` |
| **FeedEntry** | SQLite `public_feed` table | seq, op (ReleasePublished/SourceBecameStale), author_pubkey, timestamp, entry_hash, prev_hash, signature, payload JSON | `public_feed.rs` + `db.rs` |
| **ProvenanceRecord** | SQLite `provenance` (via CoordinatorDb) | repo_url, commit_sha, artifact_hash, node_id, timestamp, signature, app_version | `provenance.rs` |
| **CuratorListEntry** | DashMap in-memory (CuratorRuntime) | curator_pubkey, curator_name, revision, entries[] (project_id, name, category, description) | `curator.rs` + `iroh_runtime.rs` |
| **ProjectAnnouncement** | Gossip wire (transient, cached in BrowseAggregator) | node_id, project_name, category, description, apps[], archive_ticket, repo_url, provenance_hash, is_open_source | `publish.rs` |
| **TaskRecord / KudosEntry** | SQLite `tasks` + `kudos` tables | task_id, status, project_id, model, worker_node_id, amount, entry_hash | `db.rs` + `types.rs` |
| **App archive** (zip) | iroh-blobs MemStore | Contenu decompresse: fichiers HTML/JS/CSS/JSON, SBFB.json manifest | `blob_serve.rs` |

### 1.2 Donnees de metadata dans SBFB.json

Le manifest SBFB.json des apps contient:

```json
{
  "node_id": "<daemon_ed25519_node_id_hex>",
  "name": "<project_name>",
  "version": "<semver>"
}
```

C'est minimal. Il n'y a pas encore de champs `description`, `category`,
`license`, `capabilities`, `dependencies`, ou `keywords` dans le manifest
embarque dans le zip. Ces metadonnees transitent via le `ProjectAnnouncement`
et les `CuratorList`, pas via SBFB.json.

**GAP RRV-1:** SBFB.json manque de champs structurels pour la recherche.
L'enrichir avec `description`, `category`, `license`, `keywords`,
`capabilities` est un prerequis S70.

### 1.3 Browse et discovery actuels

Le mecanisme de decouverte actuel (`GET /api/daemon/browse`) fonctionne ainsi:

1. `BrowseAggregator.aggregate()` itere les curator lists cachees
2. Pour chaque projet, probe la reachability via iroh endpoint
3. Ajoute les projets annonces directement via gossip
4. Retourne un `Vec<BrowseEntry>` trie par (project_id, curator_pubkey)

C'est un listing exhaustif sans aucune recherche textuelle, sans filtrage
par mot-cle, sans ranking de pertinence, sans pagination intelligente.
`GET /api/v1/apps` ajoute un filtre par category et open_source mais
pas de full-text search.

**GAP RRV-2:** Aucune recherche textuelle n'existe. Le browse est un
dump complet.

### 1.4 Le feed comme source de recherche

Le feed public (`public_feed` table SQLite) contient:
- `ReleasePublished` avec project_id, repo_url, commit_sha, artifact_hash,
  provenance_hash, is_open_source
- `SourceBecameStale` avec project_id, reason

Chaque entry est signee Ed25519, chainee BLAKE3, avec PoW anti-spam.

Le feed est replayable (`replay_all()`), verifiable (`verify_chain()`),
et supporte la materialisation incrementale via curseur.

**Implication RRV:** Le feed est la source de verite temporelle — il
dit _quand_ chaque release a ete publiee, _quand_ une source est devenue
stale. C'est la base de la "fraicheur" dans les proof cards.

### 1.5 Provenance comme source de preuve

`ProvenanceRecord` (SLSA L1) contient le triplet `(repo_url, commit_sha,
artifact_hash)` signe par le coordinateur. `verify_provenance()` verifie
la signature Ed25519 sur les canonical bytes.

Le bridge expose deja deux methodes:
- `provenance_get(project_id)` → record + provenance_hash
- `provenance_verify(project_id)` → verified: bool + record

**Implication RRV:** La provenance est le coeur de la verification.
Chaque resultat de recherche peut afficher "open source verifie" /
"non verifie" / "source stale" en utilisant ces donnees existantes.

### 1.6 Ce qui manque pour RRV

| Gap | Description | Sprint cible |
|-----|-------------|--------------|
| RRV-1 | SBFB.json enrichi (keywords, license, capabilities) | S70 Phase A |
| RRV-2 | Aucune recherche textuelle | S70 Phase A-B |
| RRV-3 | Pas d'index local (ni FTS ni inverted index) | S70 Phase A |
| RRV-4 | Pas de citations fichier:ligne:hash | S70 Phase C |
| RRV-5 | Pas de model proof card | S71 Phase A |
| RRV-6 | Pas de SearchManifest wire format | S72 Phase A |
| RRV-7 | Pas de `SearchManifestPublished` feed operation | S72 Phase B |

---

## 2. Recherche externe — moteurs de recherche

### 2.1 Tantivy (Rust-native full-text search)

**Source:** github.com/quickwit-oss/tantivy, docs.rs/tantivy, crates.io
**Confiance:** HIGH (documentation officielle + code source verifiable)

Tantivy est une bibliotheque de recherche full-text ecrite en Rust,
inspiree d'Apache Lucene. C'est une _library_, pas un serveur — elle
s'integre directement dans le binaire du daemon.

Caracteristiques cles:
- Scoring BM25 pour le ranking par pertinence
- Requetes booleennes, phrases, fuzzy, prefix
- FST (Finite State Transducers) pour dictionnaire de termes compact
- Tokenizer configurable avec stemming pour 17 langues latines
- ~2x plus rapide que Lucene en latence de requete
- Support Windows, macOS, Linux
- Stable Rust, pas de nightly requis
- Index sur disque (MMapDirectory) ou en memoire (RamDirectory)

Taille typique de l'index: ~30% de la taille des donnees source
(compresse via FST + compression entiere).

Schema Tantivy pour SBFB:

```rust
let mut schema_builder = Schema::builder();
schema_builder.add_text_field("project_name", TEXT | STORED);
schema_builder.add_text_field("description", TEXT | STORED);
schema_builder.add_text_field("category", TEXT | STORED);
schema_builder.add_text_field("keywords", TEXT);
schema_builder.add_text_field("repo_url", STORED);
schema_builder.add_text_field("license", STRING | STORED);
schema_builder.add_text_field("content", TEXT);      // README, code
schema_builder.add_text_field("file_path", STRING | STORED);
schema_builder.add_u64_field("timestamp", INDEXED | STORED);
schema_builder.add_text_field("project_id", STRING | STORED);
schema_builder.add_text_field("artifact_hash", STRING | STORED);
schema_builder.add_bytes_field("provenance_hash", STORED);
```

### 2.2 SQLite FTS5

**Source:** sqlite.org/fts5, blog.sqlite.ai
**Confiance:** HIGH (technologie mature, deja dans la stack)

SQLite FTS5 est l'extension de recherche full-text integree a SQLite.
Le projet utilise deja rusqlite pour le CoordinatorDb.

Avantages vs Tantivy:
- Zero dependance supplementaire (rusqlite expose deja FTS5)
- Transactions ACID avec les tables metier (feed, provenance)
- Requetes jointes avec les donnees existantes
- Tokenizer BM25 integre

Limites vs Tantivy:
- Pas de fuzzy search natif
- Pas de scoring aussi sophistique
- Pas d'index incremental optimise pour gros volumes
- Pas de compression FST
- Tokenizer moins configurable (pas de stemming multi-langue natif)

### 2.3 Decision: Tantivy pour S70, pas FTS5

**Recommandation:** Utiliser **Tantivy** comme moteur d'indexation local.

Rationale:
1. **Performance:** Tantivy est 2x plus rapide que Lucene, FTS5 ne
   compete pas a gros volume
2. **Qualite de recherche:** BM25 + fuzzy + phrase queries + stemming
   17 langues — necessaire pour un "moteur de recherche" digne du nom
3. **Independence:** L'index Tantivy est un fichier separe du
   coordinator.db. Pas de risque de corrompre les tables metier
4. **Extensibilite:** Tantivy supporte des champs custom, des facettes,
   des filtres booleen — tout ce dont les proof cards ont besoin
5. **Embarque:** Tantivy est une lib, pas un serveur. S'integre dans
   le daemon comme iroh ou rusqlite
6. **Ecosystem:** ParadeDB, Quickwit, Turso utilisent tous Tantivy
   comme moteur interne — c'est le standard Rust

FTS5 reste utile pour des requetes simples sur les tables existantes
(par exemple "trouver un feed entry par project_name") mais n'est
pas le bon outil pour un moteur de recherche a part entiere.

### 2.4 Sonic (Rust search backend)

**Source:** github.com/valeriansaliou/sonic
**Confiance:** MEDIUM

Sonic est un backend de recherche leger qui fonctionne comme serveur
TCP separe. Il retourne des identifiants, pas des documents.

**Verdict:** Non adapte pour SBFB. Sonic est un serveur externe, pas
une library embarquee. L'architecture SBFB est "zero serveur externe"
(daemon = everything). Tantivy embedded est le bon choix.

### 2.5 Meilisearch

**Source:** meilisearch.com, github.com/meilisearch/meilisearch
**Confiance:** MEDIUM

Meilisearch est un serveur de recherche complet ecrit en Rust,
puissant pour les APIs web. Utilise Tantivy en interne pour l'indexation.

**Verdict:** Non adapte. Meilisearch est un serveur HTTP autonome,
pas une library embarquee. SBFB a besoin d'un composant embarque
dans le daemon, pas d'un second serveur.

---

## 3. Recherche externe — recherche P2P et verifiable

### 3.1 YaCy (P2P search engine)

**Source:** yacy.net, github.com/yacy/yacy_search_server, Wikipedia
**Confiance:** MEDIUM

YaCy est un moteur de recherche P2P qui distribue son index via DHT.
Chaque peer contribue a un index partage. Architecture Java, DHT pour
le routing, reverse word indexing.

**Lecons pour SBFB:**
- Le modele YaCy de "tout-le-monde-crawle" ne convient pas a SBFB.
  SBFB n'a pas de web a crawler — les apps sont des archives zip
  avec provenance
- Le modele de DHT pour distribuer l'index est interessant mais
  lourd (YaCy a des problemes de performance et d'adoption reconnus)
- L'idee de "merge d'index locaux dans un index partage" est bonne
  mais doit etre opt-in pour SBFB (privacy by design)

**Verdict:** Inspiration pour S72 (SearchManifest opt-in), mais pas
de reutilisation directe. L'architecture SBFB est fondamentalement
differente (apps signees, pas de crawl).

### 3.2 IPFS search / IPNI

**Source:** docs.ipfs.tech/concepts/ipni, ipfs-search.com, NLnet
**Confiance:** MEDIUM

IPFS-search indexe le contenu IPFS. IPNI (InterPlanetary Network
Indexer) cree une infrastructure d'indexation independante du DHT
Kademlia.

**Lecons pour SBFB:**
- IPNI montre que l'opt-in volontaire fonctionne mieux que le DHT
  general pour la decouverte de contenu
- Le probleme fondamental d'IPFS ("CIDs ne sont pas semantiquement
  cherchables") est aussi celui d'iroh-blobs
- La solution IPFS est de construire un index au-dessus des CIDs
  et de le publier separement — exactement le pattern SearchManifest

**Verdict:** Le pattern IPNI valide le design SearchManifest opt-in
de SBFB. Les noeuds publient volontairement un index signe, pas de
crawl implicite.

### 3.3 Recherche verifiable — Authenticated Data Structures

**Source:** Certificate Transparency (RFC 6962), Merkle trees, research.swtch.com/tlog
**Confiance:** HIGH (standard industriel, RFC publiee)

Certificate Transparency utilise des Merkle trees pour prouver
l'inclusion d'un certificat dans un log append-only. Le pattern est:

1. Log append-only avec Signed Tree Head (STH)
2. Merkle audit proof = log2(N) hashes pour prouver l'inclusion
3. Consistency proof = prouver qu'une version du log est un superset

**Application a RRV:**
Le feed SBFB est deja un log append-only signe (BLAKE3 hash-chain +
Ed25519). La difference avec CT:

| Aspect | Certificate Transparency | SBFB Feed |
|--------|-------------------------|-----------|
| Structure | Merkle tree | Hash-chain lineaire |
| Inclusion proof | O(log n) hashes | Replay entier |
| Multi-author | Non (log unique) | Oui (per-author chains) |
| Verification | STH + Merkle proof | entry_hash + signature |

**Decision:** Ne PAS migrer vers un Merkle tree pour S70-72. Le
hash-chain lineaire est suffisant pour la taille du reseau pre-launch
(centaines, pas millions d'entries). Un Merkle tree serait un
over-engineering premature. Si le feed depasse 100K entries, envisager
une migration post-v2.0.

Pour les proof cards (S71), l'approche est:
- L'entry_hash + signature ED25519 constituent deja une preuve
  cryptographique d'inclusion dans le feed
- La provenance_hash constitue une preuve de build integrity
- Pas besoin de Merkle proof tant que le feed est replayable

### 3.4 AppStream (Linux app metadata)

**Source:** freedesktop.org/software/appstream, github.com/ximion/appstream
**Confiance:** HIGH (standard freedesktop, utilise par Debian/Fedora/GNOME)

AppStream definit un format XML pour decrire les applications Linux
avec metadonnees structurees: id, name, summary, description,
categories, keywords, licenses, screenshots, releases, reviews.

**Lecons pour SBFB.json enrichi:**
- Le pattern "metadata_license" (licence du fichier de metadonnees
  lui-meme) est bon
- Les categories structurees et les keywords sont essentiels
- Le versioning des releases avec changelogs est utile
- Le format XML est trop lourd pour SBFB — JSON est le bon choix

**Application:** SBFB.json enrichi devrait contenir les champs
inspires d'AppStream: `description`, `categories`, `keywords`,
`license`, `screenshots`, `version_history`.

### 3.5 DOAP (Description of a Project)

**Source:** github.com/ewilderj/doap, Wikipedia
**Confiance:** MEDIUM (standard stable mais peu adopte hors GNOME)

DOAP est un vocabulaire RDF pour decrire les projets logiciels.
Proprietes: homepage, developer, programming-language, os, repository,
bug-database, download-mirror.

**Verdict:** DOAP est trop lie a RDF/Linked Data pour SBFB. Mais les
champs conceptuels sont une bonne reference pour le schema
SearchManifest: programming_language, license, repository, description.

---

## 4. Architecture RRV proposee

### 4.1 S70 — RRV LocalOnly: moteur de recherche local

#### 4.1.1 Moteur d'indexation

**Tantivy** (crate `tantivy` ~0.22) embarque dans le daemon.

L'index Tantivy vit dans `~/.sbfb/search_index/` et est reconstruit
a partir des donnees locales existantes. Pas de nouveau transport, pas
de nouvelle table SQLite pour l'index — Tantivy gere son propre
stockage.

#### 4.1.2 Quoi indexer

Trois niveaux d'indexation, du plus simple au plus ambitieux:

**Niveau 1 — Metadata (MVP, obligatoire S70):**
- `BrowseEntry`: project_name, category, description, curator_name
- `FeedEntry`: op_type, project_id, repo_url, commit_sha
- `ProvenanceRecord`: repo_url, commit_sha, artifact_hash
- `SBFB.json` enrichi: name, version, description, keywords, license

**Niveau 2 — Contenu app (souhaitable S70):**
- README.md des archives zip deployees
- index.html titre et meta descriptions
- Contenu textuel des fichiers .txt, .md dans le zip

**Niveau 3 — Code source (optionnel, S70 ou differe):**
- Fichiers .js, .ts, .py, .rs extraits du zip
- AST/symbols (fonctions, classes, exports)
- Necessiterait un parseur par langage — potentiellement trop lourd
  pour S70 Phase A-D

**Recommandation:** Niveaux 1+2 pour S70, Niveau 3 differe.

#### 4.1.3 Pipeline d'indexation

```text
Source de donnees          Pipeline             Index
-----------------          --------             -----
BrowseAggregator     -->  SearchIndexer     --> tantivy::Index
  .aggregate()             .index_browse()
                                               ~/.sbfb/search_index/
FeedStore            -->  SearchIndexer
  .replay_all()            .index_feed()

ProvenanceRecord     -->  SearchIndexer
  DB query                 .index_provenance()

BlobServeCache       -->  SearchIndexer
  zip decompression        .index_app_content()
```

**Declenchement de la re-indexation:**
- Au boot du daemon (indexation complete)
- A chaque `ProjectAnnouncement` recu via gossip
- A chaque `deploy-from-repo` ou `publish-blob` reussi
- A chaque nouveau FeedEntry insere

L'IndexWriter Tantivy supporte les writes incrementaux via
`add_document()` — pas besoin de tout re-indexer a chaque fois.

#### 4.1.4 Format des citations

Chaque resultat de recherche retourne des citations exactes:

```json
{
  "source_type": "feed_entry",
  "project_id": "abc123...",
  "file_path": null,
  "line": null,
  "entry_hash": "f81ced7d...",
  "timestamp": 1700000000,
  "signature": "ed25519_hex...",
  "text_excerpt": "Release published for sbfb-explorer"
}
```

Pour les fichiers dans les archives:

```json
{
  "source_type": "app_file",
  "project_id": "abc123...",
  "file_path": "README.md",
  "line": 42,
  "content_hash": "blake3_hex...",
  "archive_hash": "iroh_blob_hash...",
  "text_excerpt": "Ce fichier decrit..."
}
```

#### 4.1.5 API locale

Nouveau endpoint daemon:

```
GET /api/daemon/search?q=<query>&scope=local&limit=20&offset=0
```

Response:

```json
{
  "query": "traduction offline",
  "results": [
    {
      "score": 0.87,
      "source_type": "browse_entry",
      "project_id": "abc...",
      "project_name": "Babel",
      "category": "translation",
      "excerpt": "Traduction <mark>offline</mark> P2P...",
      "citations": [
        {
          "source_type": "feed_entry",
          "entry_hash": "f81...",
          "timestamp": 1700000000
        }
      ]
    }
  ],
  "total": 3,
  "took_ms": 2
}
```

Nouveau bridge method (pour les apps iframe):

```javascript
bridge.search({ query: "traduction", scope: "local", limit: 10 })
```

Cela necessite un nouveau method `search` dans le `BridgeMethodSchema`.

#### 4.1.6 UX

**Recommandation:** App SBFB dediee (`sbfb-search`), pas integree
dans le shell React.

Rationale:
- Le shell est deja un iframe host agnostique (pattern etabli)
- Une app de recherche est le meilleur dogfood du bridge
- L'app peut etre itere independamment du shell
- Futur: l'app search pourra etre remplacee ou forkee par la communaute

L'app `sbfb-search` utiliserait les bridge methods existants
(`browse_list`, `provenance_get`, `provenance_verify`) plus le nouveau
`search`.

**Alternative consideree et rejetee:** Integrer un search bar dans le
composant `Browse.tsx` du shell. Rejete parce que cela couplerait la
recherche au shell et briserait le pattern "shell = host agnostique".

### 4.2 S71 — Proof Cards

#### 4.2.1 Structure d'une Proof Card

Une "Proof Card" est un objet JSON enrichi qui accompagne chaque
resultat de recherche. Elle repond a la question "pourquoi devrais-je
faire confiance a ce resultat ?".

```typescript
interface ProofCard {
  // Identite
  project_id: string;
  project_name: string;
  
  // Source
  source: {
    type: "browse" | "feed" | "provenance" | "app_content";
    entry_hash?: string;       // hash de l'entree source
    file_path?: string;        // chemin dans l'archive
    line?: number;             // ligne dans le fichier
  };
  
  // Hash integrity
  hash: {
    archive_hash?: string;     // BLAKE3 du zip
    artifact_hash?: string;    // BLAKE3 du build
    provenance_hash?: string;  // BLAKE3 de provenance.json
    content_hash?: string;     // BLAKE3 du fichier specifique
  };
  
  // Licence
  license: {
    spdx?: string;             // "AGPL-3.0-or-later"
    source: "manifest" | "inferred" | "unknown";
  };
  
  // Fraicheur
  freshness: {
    last_verified_at: number;  // unix timestamp
    age_days: number;          // jours depuis derniere verification
    state: "fresh" | "aging" | "stale" | "unknown";
    // fresh: < 7 jours
    // aging: 7-30 jours
    // stale: > 30 jours ou SourceBecameStale dans le feed
    // unknown: pas de donnee de fraicheur
  };
  
  // Provenance
  provenance: {
    verified: boolean;
    repo_url?: string;
    commit_sha?: string;
    builder_node_id?: string;
    slsa_level: 0 | 1;        // 0 = pas de provenance, 1 = SLSA L1
  };
  
  // Risque
  risk: {
    level: "low" | "medium" | "high" | "critical";
    factors: string[];
    // Exemples de factors:
    // "no_provenance" - pas de provenance attestation
    // "stale_source" - source stale
    // "no_curator" - pas de curator vouching
    // "single_curator" - un seul curator
    // "no_open_source" - pas open source
    // "old_release" - release > 90 jours sans mise a jour
    // "unverified_deploy" - deploy zip direct, pas deploy-from-repo
  };
  
  // Curation
  curation: {
    curator_count: number;
    curator_names: string[];
  };
  
  // Confidence score (0-100)
  confidence: number;
}
```

#### 4.2.2 Calcul du confidence score

Le score de confiance est un entier 0-100 calcule deterministe,
pas une heuristique opaque. Chaque facteur ajoute ou retire des
points de maniere documentee:

```text
Base: 30 points (le resultat existe)

+ 20 si provenance.verified == true
+ 10 si is_open_source == true
+ 10 si freshness.state == "fresh"
+  5 si freshness.state == "aging"
+ 10 si curation.curator_count >= 1
+ 10 si curation.curator_count >= 3
+  5 si license.spdx != null
+  5 si hash.archive_hash != null

- 10 si risk contient "stale_source"
- 15 si risk contient "no_provenance"
- 10 si risk contient "unverified_deploy"
-  5 si risk contient "old_release"

Clamp: min(0, max(100, score))
```

Ce calcul est transparent et reproductible. L'utilisateur peut
comprendre pourquoi un resultat a un score de 85 vs 45 en lisant
les facteurs.

#### 4.2.3 Risques identifies

Le champ `risk.factors` est calcule automatiquement:

| Facteur | Condition | Impact |
|---------|-----------|--------|
| `no_provenance` | provenance_hash absent | -15 |
| `stale_source` | SourceBecameStale dans le feed | -10 |
| `no_curator` | aucun curator ne vouch le projet | -10 |
| `single_curator` | un seul curator | -5 |
| `unverified_deploy` | pas de deploy-from-repo | -10 |
| `old_release` | derniere release > 90 jours | -5 |
| `no_open_source` | is_open_source == false | 0 (info) |

#### 4.2.4 UX des Proof Cards

**Composant React** dans l'app `sbfb-search` (pas dans le shell):

```
+-------------------------------------------+
| Babel - Traduction P2P                    |
| Score: 85/100  [====>            ]        |
+-------------------------------------------+
| Source: verified_release                  |
| Hash:   archive abc123... (BLAKE3)        |
| Licence: AGPL-3.0-or-later               |
| Fraicheur: 3 jours (frais)               |
| Provenance: github.com/SBFB50/babel      |
|   commit: a1b2c3d (verifie Ed25519)      |
| Curators: FlowUP, CommunityDev           |
| Risques: aucun                            |
+-------------------------------------------+
| [Verifier la provenance] [Voir le code]   |
+-------------------------------------------+
```

Le composant est interactif:
- "Verifier la provenance" appelle `bridge.verifyRelease(projectId)`
- "Voir le code" ouvre le repo_url dans un nouvel onglet
- Chaque hash est cliquable pour montrer le detail

### 4.3 S72 — SearchManifest Opt-In

#### 4.3.1 Format du SearchManifest

Le SearchManifest est un document JSON signe qui resume l'index
d'un noeud. Il est publie volontairement.

```json
{
  "v": 1,
  "type": "sbfb.search_manifest",
  "node_id": "<daemon_ed25519_node_id_hex>",
  "created_at": 1700000000,
  
  "projects": [
    {
      "project_id": "<hex-64>",
      "project_name": "Babel",
      "category": "translation",
      "description": "Traduction P2P verifiable",
      "keywords": ["traduction", "offline", "p2p"],
      "license": "AGPL-3.0-or-later",
      "artifact_hash": "<BLAKE3 hex-64>",
      "provenance_hash": "<BLAKE3 hex-64>",
      "is_open_source": true,
      "latest_release_seq": 42,
      "latest_release_hash": "<BLAKE3 hex-64>",
      "curator_count": 2
    }
  ],
  
  "feed_cursor": {
    "last_seq": 150,
    "last_entry_hash": "<BLAKE3 hex-64>"
  },
  
  "index_stats": {
    "project_count": 5,
    "document_count": 1240,
    "total_size_bytes": 524288
  },
  
  "signer_pubkey": "<Ed25519 hex-64>",
  "signature": "<Ed25519 hex-128>"
}
```

**Canonical bytes:** `DOMAIN_SEARCH_MANIFEST_V1 || 0x00 || JCS(manifest)`
Nouveau domain constant a ajouter dans `canonical.rs`.

**Limites par champ:**
- `projects.len()` <= 256 (meme cap que CuratorList)
- `description` <= 280 bytes par projet
- `keywords` <= 10 par projet, 64 bytes chacun
- Total manifest <= 1 MB

#### 4.3.2 Publication opt-in

Le manifest est publie via iroh-blobs + gossip, exactement comme
les CuratorList:

1. Le daemon genere le SearchManifest depuis son index local
2. Signe avec la cle Ed25519 du noeud
3. Stocke comme blob via `BlobsClient::add_bytes()`
4. Annonce le BlobTicket sur un nouveau gossip topic:
   `BLAKE3("nexus-grid/search-manifest/v1")[..32]`

```json
{
  "type": "search_manifest",
  "node_id": "<hex-64>",
  "ticket": "<BlobTicket>"
}
```

**Opt-in explicite:** Le daemon ne publie PAS de manifest par defaut.
L'utilisateur doit activer la publication via:
```
POST /api/daemon/search/publish-manifest
```
Ou via une checkbox dans le shell "Rendre mon index decouvrable".

#### 4.3.3 Verification des manifests recus

Quand un daemon recoit un manifest d'un autre noeud:

1. `fetch_ticket()` pour telecharger le blob
2. Parse JSON, verifier `v == 1`
3. Verifier signature Ed25519 via `canonical_bytes(manifest, DOMAIN_SEARCH_MANIFEST_V1)`
4. Verifier que `node_id` correspond a `signer_pubkey`
5. Verifier `feed_cursor.last_entry_hash` contre le feed local (si disponible)
6. Cacher dans un DashMap similaire au CuratorRuntime

#### 4.3.4 Discovery des manifests

Deux modes de decouverte:

**Mode 1 — Gossip (bootstrap actif):**
Le daemon s'abonne au topic `"nexus-grid/search-manifest/v1"` et
recoit les annonces de manifests des peers.

**Mode 2 — Feed event:**
Le feed public recoit un nouveau type d'operation
`SearchManifestPublished`:

```json
{
  "op_type": "SearchManifestPublished",
  "project_id": "<hex-64>",
  "manifest_hash": "<BLAKE3 hex-64>",
  "manifest_ticket": "<BlobTicket>",
  "project_count": 5
}
```

Cela permet de decouvrir les manifests retroactivement en rejouant
le feed, pas seulement en temps reel via gossip.

#### 4.3.5 Anti-spam sur les manifests

- Rate limiting: 1 manifest par noeud par heure (cache DashMap)
- Taille max: 1 MB
- `projects.len()` <= 256
- Signature Ed25519 obligatoire
- Optionnel: PoW 16-bit (meme difficulte que le feed)

#### 4.3.6 Privacy

**Ce qu'un manifest revele:**
- Quels projets le noeud a indexes (public par design — opt-in)
- Le curseur du feed local (revele la fraicheur du noeud)
- Le node_id du publisheur (deja public via iroh)

**Ce qu'un manifest ne revele PAS:**
- Les requetes de recherche de l'utilisateur (jamais envoyees au reseau)
- Le contenu complet de l'index (seulement les metadonnees)
- Les donnees de stockage privees des apps

**Design privacy-by-default:** La recherche reste locale par defaut.
Les manifests enrichissent seulement la decouverte de projets, ils
ne permettent pas la recherche dans le contenu d'un autre noeud.

---

## 5. Dependances et risques

### 5.1 Dependances inter-sprints

| Dependance | Direction | Impact |
|------------|-----------|--------|
| S65 (go-live, auth-tier feed) | S70 depend de S65 | Le feed doit etre fonctionnel et avec auth-tier avant de l'indexer |
| S66 (durabilite) | S70 depend faiblement de S66 | L'index Tantivy persiste deja sur disque, pas de dependance directe sur la durabilite iroh-blobs |
| S67 (gouvernance) | S72 depend de S67 | Le trust des manifests depend du systeme de curators |
| S70 RRV LocalOnly | S71 depend de S70 | Les proof cards enrichissent les resultats de recherche |
| S71 Proof Cards | S72 depend de S71 | Le manifest publie les proof card metadata |

### 5.2 Risques techniques

| Risque | Probabilite | Impact | Mitigation |
|--------|-------------|--------|------------|
| **Tantivy version break** | LOW | MEDIUM | Pin la version dans Cargo.toml, tester en CI |
| **Index trop gros pour la memoire** | LOW (peu d'apps pre-launch) | LOW | Tantivy utilise mmap, pas de chargement en RAM |
| **Corruption d'index** | LOW | LOW | Rebuild depuis les sources (feed + browse + provenance) |
| **Manifest spoofing** | MEDIUM | MEDIUM | Signature Ed25519 + verification, PoW optionnel |
| **RRV = surface d'attaque injection** | MEDIUM | HIGH | Sanitizer les champs indexes (strip HTML, limit UTF-8, reject NUL bytes) |
| **Privacy leak via manifest** | LOW (opt-in) | MEDIUM | Default off, pas de requetes transmises |
| **Complexite excessive S70** | MEDIUM | HIGH | Commencer par Niveau 1 (metadata only), pas de code source |

### 5.3 Risque produit principal

**RRV ne doit pas devenir un systeme de surveillance.** La recherche
reste locale par defaut. Les manifests sont opt-in. Les requetes de
l'utilisateur ne sont jamais envoyees au reseau. C'est un principe
de design fondamental, pas un nice-to-have.

---

## 6. Plans de phases

### 6.1 S70 — RRV LocalOnly (A-D)

**Phase A — Index local + API**
- Ajouter `tantivy` au workspace Cargo.toml
- Creer `crates/nexus-coordinator-rs/src/search_index.rs`:
  - Schema Tantivy (project_name, description, category, keywords,
    repo_url, artifact_hash, timestamp, etc.)
  - `SearchIndex::create(path)`, `index_browse_entries()`,
    `index_feed_entries()`, `index_provenance_records()`
  - `search(query, limit)` → `Vec<SearchResult>`
- Creer `crates/nexus-shell-daemon/src/search_api.rs`:
  - `GET /api/daemon/search?q=...&limit=...&offset=...`
- Wire dans `http.rs` router
- 6-8 tests unitaires: index creation, search, empty results, special chars

**Phase B — Indexation au boot + incrementale**
- Au boot: indexer browse entries, feed entries, provenance records
- Trigger incrementale: re-indexer a chaque ProjectAnnouncement recu,
  deploy reussi, ou FeedEntry insere
- Enrichir SBFB.json: ajouter `description`, `keywords`, `license`
  (champs optionnels, #[serde(default)])
- Indexer le contenu des README.md dans les archives zip
- Tests: rebuild d'index, indexation incrementale

**Phase C — Bridge method + citations**
- Ajouter `search` au BridgeMethodSchema dans `protocol.ts`
- Implementer le handler dans `useBridge.ts`
- Chaque resultat retourne des citations exactes (source_type,
  entry_hash, file_path, line)
- Tests Vitest: bridge search dispatch, citation format

**Phase D — App sbfb-search MVP**
- Creer `examples/sbfb-search/` (HTML + JS, meme pattern que Explorer)
- Search bar + resultats avec extraits et citations
- Design dark theme coherent avec Explorer/Ideas Hub
- SBFB.json manifest
- Tests: taille bundle, fonctionnalite search

### 6.2 S71 — Proof Cards (A-D)

**Phase A — ProofCard data model + computation**
- Creer `crates/nexus-coordinator-rs/src/proof_card.rs`:
  - `ProofCard` struct (source, hash, license, freshness, provenance,
    risk, curation, confidence)
  - `compute_proof_card(project_id, db, browse_entry)` → ProofCard
  - Confidence score deterministic (formule documentee)
  - Risk factors automatiques
- 8-10 tests: score computation, risk detection, edge cases

**Phase B — API + bridge**
- `GET /api/daemon/proof-card/{project_id}`
- Bridge method `proof_card_get(project_id)` → ProofCard
- Tests: API response format, bridge dispatch

**Phase C — Integration dans search results et Browse**
- L'app sbfb-search affiche les proof cards avec chaque resultat
- Composant ProofCard (HTML/JS): score bar, facteurs, actions
- "Verifier la provenance" interactif via bridge
- Tests Playwright: affichage proof card, verification interactive

**Phase D — Tests adversariaux**
- Proof card spoofing: un projet sans provenance ne peut pas
  afficher un score > 50
- Risk factor injection: injection HTML dans description
  → sanitized avant affichage
- Stale detection: projet avec SourceBecameStale → risk "stale_source"
- Score determinism: meme entrees → meme score (pas de randomness)

### 6.3 S72 — SearchManifest Opt-In (A-D)

**Phase A — SearchManifest format + signing**
- Domain constant `DOMAIN_SEARCH_MANIFEST_V1` dans `canonical.rs`
- Wire format `SearchManifest` dans `nexus-core-rs/src/search_manifest.rs`
- Sign/verify via canonical_bytes pattern existant
- Validation: champs limites, version check, signature Ed25519
- 8-10 tests: sign, verify, reject tampered, reject oversized

**Phase B — Publication opt-in via iroh**
- Gossip topic `"nexus-grid/search-manifest/v1"`
- `POST /api/daemon/search/publish-manifest`
- Stockage blob + annonce gossip
- Rate limiter: 1 publication par heure par noeud
- `SearchManifestPublished` feed operation type ajout (enum variant)
- Tests: publish, gossip announce, rate limit

**Phase C — Discovery + verification**
- Subscribe au gossip topic search-manifest
- Recevoir, parser, verifier les manifests des peers
- Cache DashMap similaire a CuratorRuntime
- `GET /api/daemon/search/manifests` → liste des manifests recus
- Enrichir les resultats de recherche avec les donnees des manifests distants
- Tests: receive, verify, cache, reject forged

**Phase D — Anti-spam + privacy analysis**
- PoW optionnel 16-bit sur la publication de manifests
- Tests adversariaux: spam manifests, manifests surdimensionnes,
  signatures invalides, replay d'ancien manifest
- Documentation privacy: ce qu'un manifest revele vs. ce qu'il ne revele pas
- Audit de la surface d'exposition (quels champs pourraient etre
  utilises pour fingerprinter un noeud)

---

## 7. Stack technique recommande

### 7.1 Dependances Rust a ajouter

| Crate | Version | Purpose | Sprint |
|-------|---------|---------|--------|
| `tantivy` | ~0.22 | Full-text search engine | S70 |

Pas d'autre dependance nouvelle necessaire. Les crates existants
(rusqlite, serde_jcs, ed25519-dalek, blake3, hex, dashmap) couvrent
tous les besoins de S71-S72.

### 7.2 Pas de dependance frontend nouvelle

L'app sbfb-search est vanilla JS comme Explorer et Ideas Hub.
Pas de build step, pas de React, pas de dependance npm.

---

## 8. Estimation de complexite

| Sprint | Phases | Nouveaux fichiers Rust | Nouveaux fichiers Frontend | Tests estimes | Complexite |
|--------|--------|----------------------|---------------------------|---------------|------------|
| S70 | A-D | 3-4 (search_index.rs, search_api.rs) | 5 (app sbfb-search) | +25-30 Rust, +10-15 Vitest | MEDIUM |
| S71 | A-D | 2 (proof_card.rs, proof_card_api.rs) | Enrichissement sbfb-search | +20-25 Rust, +5-10 Vitest | MEDIUM |
| S72 | A-D | 2-3 (search_manifest.rs, manifest handler) | Enrichissement sbfb-search | +25-30 Rust | HIGH |

---

## 9. Questions ouvertes

1. **Faut-il Tantivy ~0.22 ou ~0.23 ?** Verifier la derniere version
   stable et la compatibilite avec le MSRV 1.94 au moment du kickoff S70.

2. **Indexer le code source (Niveau 3) en S70 ou differer ?** Recommandation:
   differer. Le code source necessite des parseurs par langage, tree-sitter
   integration, et complexifie l'index pour une valeur marginale au debut.

3. **SBFB.json enrichi: retro-compatible ?** Oui, tous les nouveaux champs
   sont `#[serde(default)]`. Les anciens manifests restent valides.

4. **SearchManifest dans le feed: faut-il un nouveau FEED_FORMAT_VERSION ?**
   Oui, ajouter un variant a l'enum `PublicFeedOperation` est un breaking
   change (cf. PUBLIC_FEED_SPEC.md §9). Bump a version 2.

5. **Limit du manifest: 256 projets suffisent-ils ?** Pour le reseau
   pre-launch, oui. Post-launch, envisager un manifest pagine ou des
   shards d'index via iroh-blobs.

---

## 10. Prior art dans le repo

Les recherches precedentes couvrent deja certains aspects:

| Document | Contribution |
|----------|-------------|
| `chat_ia_reseau_recherche_reseau_rnd.md` | Vision RRV originale, SearchManifest v1 conceptuel, IndexShard, privacy modes |
| `rrv_scoped_search_compute_groups.md` | Scopes (@dev, @network, @web), sequence produit, non-goals |
| `sbfb_project_factory_rrv_oss_research.md` | Architecture Project Factory + RRV @dev, objets indexes, privacy gates |
| `p2panda_public_protocol_briques.md` | SearchManifestPublished dans le feed, lien avec feed public |

La presente recherche concretise ces visions en architecture implementable
pour S70-72, avec choix techniques arretes (Tantivy, pas FTS5), formats
wire definis, et phases detaillees.

---

## 11. Verdict

Les sprints 70-72 RRV sont faisables avec la stack existante + Tantivy.
Les donnees locales sont deja suffisantes pour un moteur de recherche
utile (browse entries, feed, provenance, contenu des archives). Les
proof cards sont un differenciateur produit fort ("je ne te donne pas
juste un resultat, je te montre pourquoi il est fiable"). Le
SearchManifest opt-in est le bon pattern pour etendre la recherche
au reseau sans sacrifier la privacy.

L'ordre S70 → S71 → S72 est le bon:
1. D'abord chercher localement (fondation)
2. Ensuite montrer le niveau de preuve (differenciateur)
3. Enfin ouvrir au reseau avec controle (extension)

Ne pas inverser cet ordre. Ne pas sauter a S72 sans S70-71 fonctionnels.
