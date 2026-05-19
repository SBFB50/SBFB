# Factory comme client externe — Gap Analysis

**Date :** 2026-05-19
**Contexte :** Le PO veut que Factory devienne un outil client
externe au protocole SBFB, pas un module integre au daemon. Le
protocole reste neutre — Factory est un consommateur comme un
autre.

**Methode :** Lecture exhaustive du code daemon actuel
(`http.rs`, `deploy.rs`, `blob_serve.rs`, `feed_sync.rs`,
`publish.rs`, `public_feed.rs`, `provenance.rs`, `apps.rs`,
`cli.rs`), des specs (`FACTORY_GATES.md`, `SBFB_JSON_V2.md`),
et de la roadmap S67-S69.

---

## 1. Inventaire des besoins Factory

### 1.1 Table besoin par besoin

| # | Besoin Factory | Gate(s) | Classification | Primitive daemon requise | Justification |
|---|----------------|---------|----------------|--------------------------|---------------|
| 1 | **Classifier une app** (type, scope, bridge methods) | FG0, FG1 | :x: Pas du protocole | Aucune | Purement local. Factory choisit un type et des bridge methods selon l'intention dev. Le daemon n'a pas a connaitre la classification — il recoit le manifest final. |
| 2 | **Generer un projet depuis un template** | FG2 | :x: Pas du protocole | Aucune | Copie de fichiers + substitution de variables + `git init`. Zero interaction reseau. Un crate Rust standalone ou un binaire CLI fait ca sans daemon. |
| 3 | **Valider un manifest SBFB.json v2** | FG3 | :wrench: Primitive manquante | `POST /api/v1/manifest/validate` OU lib partagee `sbfb-manifest` | Le daemon a besoin de cette validation en interne (deploy.rs). Factory aussi. Deux options : (a) route HTTP daemon qui valide et retourne les erreurs, (b) crate Rust partage (`sbfb-manifest`) que Factory et le daemon importent tous les deux. L'option (b) est plus propre car la validation est un calcul pur sans etat. Voir §2.1. |
| 4 | **Generer un diff avant application** | FG4 | :x: Pas du protocole | Aucune | Comparaison locale entre deux arbres de fichiers. Factory calcule le diff en memoire et le presente au developpeur. Le daemon ne sait rien des diffs. |
| 5 | **Tester dans un sandbox iframe (CSP violations, same-origin)** | FG5 | :wrench: Primitive manquante | `POST /api/v1/preview/load` + `GET /blob-serve/{hash}/{path}` (existe deja) | Le chemin blob-serve existe. Ce qui manque : un moyen d'injecter des bytes zip dans le blob store sans passer par le deploy complet. `POST /api/v1/deploy` (deploy_private) fait ca mais publie un blob permanent + n'a pas de TTL. Besoin d'un endpoint ephemere "preview" qui charge un zip dans le cache blob-serve sans le persister dans iroh-blobs. Voir §2.2. |
| 6 | **Scanner les secrets** | FG6 | :x: Pas du protocole | Aucune | Regex scan sur un repertoire local. Pas de reseau, pas de daemon. Le scanner peut etre un module dans le binaire `sbfb-factory` ou meme un crate partage reutilisable par le daemon a terme (publish gate defense-in-depth). |
| 7 | **Scanner les deps (CVE)** | FG6 | :x: Pas du protocole | Aucune | Lecture package.json/lockfile + base CVE embarquee. Calcul local pur. |
| 8 | **Preview live d'une app** | FG7 | :white_check_mark: Partiellement couvert + :wrench: primitive manquante | Preview ephemere via blob-serve (meme primitive que #5) | Blob-serve sait servir un zip dans un iframe sandbox avec CSP + COOP/COEP. Ce qui manque : un moyen d'injecter un zip temporaire (non-persiste dans iroh-blobs) avec un TTL de ~30 min. Voir §2.2. |
| 9 | **Generer la provenance SLSA L1** | FG8 | :white_check_mark: Deja couvert | `POST /api/v1/deploy-from-repo` le fait en interne | Le daemon clone, zip, signe, genere provenance.json. Factory peut simplement appeler `POST /api/v1/deploy-from-repo` avec les bons parametres. Cependant, si Factory veut decomposer le process (zip local → signer separement → publier), il faudrait exposer la signature Ed25519 comme primitive. Voir §2.3. |
| 10 | **Publier une app sur le reseau** | FG9 | :white_check_mark: Deja couvert | `POST /api/v1/deploy-from-repo` (verified) ou `POST /api/v1/deploy` (private) | Les deux endpoints existent et fonctionnent. Le deploy-from-repo produit provenance + blob + gossip announce + feed entry ReleasePublished. Factory peut les appeler directement. |
| 11 | **Inserer une entree feed** (ReleasePublished) | FG9 | :white_check_mark: Deja couvert | `POST /api/daemon/feed/insert` (avec `X-SBFB-Feed-Internal: 1`) | L'endpoint existe depuis S65 Phase A. Accepte n'importe quelle `serde_json::Value` comme `op` via le raw-op extensible. Deploy-from-repo l'appelle deja automatiquement. |
| 12 | **Inserer CuratorVouched / CuratorDisendorsed** | FG10 | :wrench: Primitive manquante (types) + :white_check_mark: transport couvert | `POST /api/daemon/feed/insert` + nouveaux types `CuratorVouched`/`CuratorDisendorsed` dans `PublicFeedOperation` | L'endpoint feed/insert existe et accepte des raw ops. Mais les types `CuratorVouched` et `CuratorDisendorsed` ne sont pas encore definis dans `PublicFeedOperation` (enum n'a que `ReleasePublished` et `SourceBecameStale`). La validation `validate_feed_operation()` ne les connait pas. Ils seront stockes comme ops inconnues mais pas valides. Voir §2.4. |
| 13 | **Auditer les operations Factory** (log JSONL) | — | :x: Pas du protocole | Aucune | Ecriture locale d'un fichier JSONL dans le workspace de l'app. Purement un concern Factory. |
| 14 | **Generer factory.template.lock** | FG2, FG3 | :x: Pas du protocole | Aucune | Fichier de lockfile genere par Factory dans le projet cree. Hash BLAKE3 du template + version + date. Purement local. |
| 15 | **Generer factory.provenance.json** | FG8 | :x: Pas du protocole | Aucune | Metadonnees de creation (generator version, template hash, variables hash). Distinct de provenance.json SLSA L1 du deploy. Purement local. |
| 16 | **Publish gate checklist** (index.html, manifest, secrets, bridge methods) | FG5-FG8 | :x: Pas du protocole (logique) + :wrench: validation manifest partagee | Reutilise la lib de validation manifest (meme item #3) | La checklist est une composition de verifications locales. Factory orchestre, le daemon n'a pas besoin de connaitre la notion de "gate". |
| 17 | **Retrait node_id du manifest** | FG3 | :wrench: Primitive manquante | Modification de `SbfbJson` struct dans deploy.rs : `node_id` → `Option<String>` | Aujourd'hui `deploy.rs:119` fait `sbfb.node_id != state.node_id` — check bloquant. Les templates Factory ne connaissent pas le node_id a l'avance. Il faut rendre node_id optionnel. |
| 18 | **Lister les templates disponibles** | FG0, FG2 | :x: Pas du protocole | Aucune | Catalogue local de templates embarques dans le binaire Factory. |
| 19 | **Domain packs** (Babel) | FG2 ext. | :x: Pas du protocole | Aucune | Extension du systeme de templates avec fixtures + config. Purement local a Factory. |

### 1.2 Resume de la classification

| Classification | Nombre | Items |
|----------------|--------|-------|
| :x: Pas du protocole | 12 | #1, #2, #4, #6, #7, #13, #14, #15, #16, #18, #19 |
| :white_check_mark: Deja couvert | 4 | #9, #10, #11 (+ #12 transport) |
| :wrench: Primitive manquante | 4 | #3, #5/8, #12 (types), #17 |

---

## 2. Primitives daemon manquantes — detail

### 2.1 Validation manifest partagee (SBFB.json v2)

**Probleme actuel :** deploy.rs definit une struct `SbfbJson` minimale
(`node_id: String, version: Option<String>`) en interne (ligne 543-548).
Elle ne parse que `node_id` et `version`. La spec SBFB_JSON_V2.md
definit 15+ champs. Il n'y a aucun validateur schema v2 dans le code.

**Ce que Factory a besoin :** Valider un SBFB.json v2 complet avant
de laisser le developpeur publier. Verifier les champs requis (name,
display_name, description), la coherence bridge.methods vs allowlist,
le format name (kebab-case, max 64 chars), etc.

**Ce que le daemon a besoin :** Le meme validateur dans deploy.rs pour
remplacer la struct minimale actuelle. Et a terme, pour que tout
client puisse valider un manifest sans le daemon.

**Solution proposee :**

Creer un crate Rust `sbfb-manifest` (ou un module dans
`nexus-coordinator-rs`) avec :
- `struct SbfbManifest` complete (v1 + v2)
- `fn validate(manifest: &SbfbManifest) -> Result<(), Vec<ManifestError>>`
- `fn parse(json: &str) -> Result<SbfbManifest, ParseError>`
- Exported pour Factory ET pour deploy.rs
- Zero dependance reseau, zero dependance daemon

**Alternative :** Route HTTP `POST /api/v1/manifest/validate` qui
prend un JSON body et retourne les erreurs. Moins propre (requiert
un daemon running) mais plus simple pour un client non-Rust. Les
deux ne sont pas exclusifs.

**Impact roadmap :** S67 Phase A doit creer ce code. C'est deja
dans le plan (roadmap S67 Phase A : "Implementer SBFB.json v2 dans
deploy.rs"). Il suffit de l'extraire en crate/module partage au lieu
de le coder inline dans deploy.rs.

### 2.2 Preview ephemere (blob-serve temporaire)

**Probleme actuel :** Le daemon expose `GET /blob-serve/{hash}/{path}`
qui sert des fichiers depuis un zip stocke dans iroh-blobs. Le zip doit
etre dans le blob store (appel `blobs.get_bytes(hash_bytes)` si pas
en cache LRU). Les deux endpoints de deploy (`deploy_private` et
`deploy_from_repo`) persistent le blob dans iroh-blobs de facon
permanente.

Factory a besoin d'une preview ephemere : servir un zip dans un
iframe sandbox SANS le persister dans iroh-blobs, avec un TTL de
~30 minutes, pour que le developpeur voie son app avant publication.

**Solution proposee :**

Nouvelle route daemon :
```
POST /api/v1/preview/load
Body: raw zip bytes
Response: { "preview_hash": "<hex>", "expires_at": "<iso8601>" }
```

Cette route :
1. Valide le zip (index.html present, path traversal check)
2. Calcule le BLAKE3 hash
3. Charge le zip directement dans le `BlobServeCache` (LRU memoire)
   SANS l'ajouter a iroh-blobs
4. Retourne le hash temporaire

Le client (Factory) ouvre ensuite
`http://127.0.0.1:{port}/blob-serve/{preview_hash}/index.html`
dans un iframe ou un onglet. Le zip est en cache memoire, servi
avec les memes headers CSP/COOP/COEP que le deploy normal.

Eviction : le cache LRU existant (`BlobServeCache` avec
`max_entries=32`) suffit. Si on veut un TTL explicite, ajouter un
champ `expires_at` dans `insertion_order` et evicter au check.

**Impact roadmap :** S68 Phase D dans le plan original. Avec
Factory-as-client, c'est une primitive daemon neutre (pas specifique
a Factory). Tout client qui veut preview une archive peut l'utiliser.

### 2.3 Signature Ed25519 decomposee (optionnel, stretch)

**Probleme actuel :** Le daemon genere la provenance en interne
dans `deploy_from_repo` (clone → zip → signe → inject provenance
dans zip → blob store). Le process est monolithique. Si Factory
veut generer le zip localement et juste demander une signature au
daemon, il n'y a pas de primitive pour ca.

**Analyse :** Ce besoin est OPTIONNEL. Factory peut simplement
appeler `POST /api/v1/deploy-from-repo` et laisser le daemon
faire tout le travail. Le workflow decompose (Factory zip + daemon
sign) est une optimisation pour eviter un double transfert du zip.

**Si on le veut :**
```
POST /api/v1/sign
Body: { "data_hex": "<BLAKE3 hash a signer>" }
Response: { "signature_hex": "<Ed25519 sig>", "pubkey_hex": "<node pubkey>" }
```

**Risque :** Exposer une primitive de signature generique est
dangereux (le client peut faire signer n'importe quoi). Il faut
au minimum domain-separate (prefixer les bytes avec un tag
`"sbfb-provenance-v1:"`) et/ou accepter uniquement un hash
BLAKE3 d'un artifact connu du blob store.

**Recommandation :** Ne pas implementer en S67-S69. Le deploy
monolithique via `deploy-from-repo` est suffisant et plus sur.
Revisiter si un use case concret le justifie post-pilote.

### 2.4 CuratorVouched / CuratorDisendorsed dans le feed

**Probleme actuel :** `PublicFeedOperation` enum dans
`crates/nexus-coordinator-rs/src/public_feed.rs` n'a que deux
variantes :
```rust
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
}
```

Les types `CuratorVouched` et `CuratorDisendorsed` ne sont pas
definis. Le code les mentionne en commentaire (ligne 52-54 :
"Future variants (`CuratorVouched`, `BuildQuorumReached`, ...)")
mais ne les implemente pas.

L'endpoint `POST /api/daemon/feed/insert` accepte des raw ops
(`serde_json::Value`) via le champ `op`. Un client peut envoyer
un JSON `{"op_type": "CuratorVouched", ...}` et il sera stocke
et propage comme op inconnue (raw-op extensibility S65). MAIS
`validate_feed_operation()` ne connait pas ce type et ne validera
pas les champs (project_id hex-64, curator_pubkey hex-64, scope
max 280 chars, etc.).

**Solution proposee :**

Ajouter les deux variantes a l'enum :
```rust
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
    CuratorVouched(CuratorVouchedPayload),
    CuratorDisendorsed(CuratorDisendorsedPayload),
}
```

Avec les payloads :
```rust
pub struct CuratorVouchedPayload {
    pub project_id: String,    // hex-64
    pub curator_pubkey: String, // hex-64
    pub scope: String,          // max 280 chars
    pub comment: Option<String>, // max 280 chars
}

pub struct CuratorDisendorsedPayload {
    pub project_id: String,
    pub curator_pubkey: String,
    pub reason: String,
    pub comment: Option<String>,
}
```

Et la validation dans `validate_known_operation()`.

`FEED_FORMAT_VERSION` reste a 1 (raw-op, pas de bump). Les noeuds
anciens (sans ces types) stockent et propagent les ops inconnues.

**Impact roadmap :** Deja planifie S67 Phase A. Pas de changement.

---

## 3. Architecture proposee pour `sbfb-factory`

### 3.1 Forme du livrable

**Un binaire CLI Rust separe**, pas un module daemon. Installe a
cote du daemon (`sbfb-factory` ou `sbfb create` comme alias).

```
sbfb-factory create --template static-storage --name my-app
sbfb-factory validate manifest ./my-app/SBFB.json
sbfb-factory diff ./my-app
sbfb-factory scan-secrets ./my-app
sbfb-factory preview ./my-app --daemon http://127.0.0.1:PORT
sbfb-factory publish ./my-app --daemon http://127.0.0.1:PORT
sbfb-factory audit ./my-app
```

### 3.2 Crate layout

```
crates/
  sbfb-manifest/              # NOUVEAU — lib pure, zero reseau
    src/lib.rs                # SbfbManifest, parse, validate
    src/bridge_allowlist.rs   # methodes bridge autorisees
    Cargo.toml                # deps: serde, serde_json, thiserror

  sbfb-factory/               # NOUVEAU — binaire CLI
    src/main.rs               # clap CLI, orchestration gates
    src/templates/             # templates embarques (include_str!)
    src/diff.rs               # diff engine (fichiers tree)
    src/secret_scanner.rs     # regex scan secrets
    src/audit_log.rs          # JSONL writer
    src/preview.rs            # HTTP client → daemon preview/load
    src/publish.rs            # HTTP client → daemon deploy-from-repo
    src/template_lock.rs      # factory.template.lock gen
    src/provenance_local.rs   # factory.provenance.json gen
    Cargo.toml                # deps: sbfb-manifest, clap, reqwest,
                              #       blake3, serde, zip, walkdir

  nexus-coordinator-rs/       # EXISTANT — ajout CuratorVouched
    src/public_feed.rs        # + 2 variantes enum + payloads

  nexus-shell-daemon-core/    # EXISTANT — ajout preview cache
    src/blob_serve.rs         # + methode load_ephemeral (ou reutilise load)

  nexus-shell-daemon/         # EXISTANT — ajout routes
    src/http.rs               # + POST /api/v1/preview/load
    src/deploy.rs             # import sbfb-manifest au lieu du struct local
```

### 3.3 Dependances

```
sbfb-factory ──depends-on──> sbfb-manifest (validation locale)
sbfb-factory ──HTTP client──> daemon (preview, publish, feed)

nexus-shell-daemon ──depends-on──> sbfb-manifest (validation deploy)
nexus-coordinator-rs ──unchanged──> (fournit public_feed types)
```

`sbfb-factory` n'importe PAS `nexus-shell-daemon-core`,
`nexus-coordinator-rs`, ou `nexus-core-rs`. Il parle au daemon
uniquement via HTTP. C'est la separation nette : Factory est un
CLIENT du protocole.

### 3.4 Workflow complet

```
Developpeur
    |
    v
sbfb-factory create --template static-storage --name mon-app
    |  (local : copie template, gen SBFB.json v2, git init)
    |  (local : factory.template.lock, factory.provenance.json)
    v
  ... code de l'app ...
    |
    v
sbfb-factory validate manifest ./mon-app/SBFB.json
    |  (local : sbfb-manifest::validate)
    v
sbfb-factory scan-secrets ./mon-app
    |  (local : regex patterns)
    v
sbfb-factory diff ./mon-app
    |  (local : compare workspace vs template ou vs derniere version)
    v
sbfb-factory preview ./mon-app --daemon http://127.0.0.1:PORT
    |  (local : zip le repertoire)
    |  (HTTP : POST /api/v1/preview/load → hash)
    |  (ouvre : http://127.0.0.1:PORT/blob-serve/{hash}/index.html)
    v
sbfb-factory publish ./mon-app --daemon http://127.0.0.1:PORT
    |  (lit running.json pour le token + port)
    |  (HTTP : POST /api/v1/deploy-from-repo si repo public)
    |  (HTTP : POST /api/v1/deploy si zip direct)
    |  (le daemon gere : provenance, blob, gossip, feed entry)
    v
App publiee sur le reseau
```

### 3.5 Auth

Factory doit s'authentifier aupres du daemon comme tout client
loopback. Il lit `~/.sbfb/shell-daemon/running.json` pour
obtenir le port et le token bearer, puis envoie `X-SBFB-Token`
sur chaque requete. Identique a ce que fait le shell React via
`GET /auth/token`.

---

## 4. API daemon — routes existantes reutilisables

| Route daemon | Methode | Usage Factory | Etat |
|---|---|---|---|
| `GET /health` | GET | Verifier que le daemon tourne | :white_check_mark: Existe |
| `GET /auth/token` | GET | Recuperer le bearer token | :white_check_mark: Existe |
| `POST /api/v1/deploy-from-repo` | POST | Publier une app verified | :white_check_mark: Existe |
| `POST /api/v1/deploy` | POST | Publier un zip direct | :white_check_mark: Existe |
| `GET /blob-serve/{hash}/{path}` | GET | Preview iframe | :white_check_mark: Existe |
| `POST /api/daemon/feed/insert` | POST | Inserer ops feed | :white_check_mark: Existe |
| `GET /api/daemon/feed/status` | GET | Verifier feed sync | :white_check_mark: Existe |
| `GET /api/daemon/browse` | GET | Verifier app visible dans Browse | :white_check_mark: Existe |
| `GET /api/v1/apps/{id}` | GET | Verifier detail app post-deploy | :white_check_mark: Existe |
| `GET /api/v1/project/{id}/provenance` | GET | Verifier provenance post-deploy | :white_check_mark: Existe |
| `POST /api/v1/preview/load` | POST | Preview ephemere | :wrench: A creer |
| `POST /api/v1/manifest/validate` | POST | Validation manifest v2 (optionnel) | :wrench: Optionnel (lib suffisante) |

---

## 5. Nouvelles primitives daemon necessaires

### 5.1 OBLIGATOIRE : Preview ephemere

**Route :** `POST /api/v1/preview/load`
**Auth :** Bearer token (meme gate que les routes authed)
**Body :** Raw bytes (zip archive)
**Response :** `{"preview_hash": "<hex>", "ttl_seconds": 1800}`
**Semantique :**
- Valide le zip (index.html present, path traversal)
- Calcule BLAKE3 hash
- Charge dans `BlobServeCache` en memoire (pas iroh-blobs)
- Retourne le hash
- Le client ouvre `/blob-serve/{hash}/index.html` normalement
- Eviction par LRU ou TTL

**Effort :** ~40-60 LOC handler + ~10 LOC ajustement cache.

### 5.2 OBLIGATOIRE : node_id optionnel dans deploy

**Route :** `POST /api/v1/deploy-from-repo` (modification)
**Changement :** Rendre le check `sbfb.node_id != state.node_id`
conditionnel. Si SBFB.json n'a pas de `node_id`, accepter.
**Effort :** ~5 LOC (le struct `SbfbJson` a deja `node_id: String`,
passer a `Option<String>`, supprimer le check bloquant).

### 5.3 OBLIGATOIRE : CuratorVouched/CuratorDisendorsed types

**Fichier :** `crates/nexus-coordinator-rs/src/public_feed.rs`
**Changement :** Ajouter 2 variantes a `PublicFeedOperation`, 2
structs payload, validation dans `validate_known_operation()`.
**Effort :** ~80-100 LOC.

### 5.4 RECOMMANDE : Crate sbfb-manifest partage

**Forme :** Nouveau crate dans le workspace.
**Contenu :** Struct `SbfbManifest` complete, parse v1+v2,
validation, bridge allowlist.
**Effort :** ~200-300 LOC.
**Alternative si presse :** Module dans `nexus-coordinator-rs`.

### 5.5 OPTIONNEL : Route validate manifest

**Route :** `POST /api/v1/manifest/validate`
**Usage :** Pour des clients non-Rust qui ne peuvent pas importer
le crate `sbfb-manifest`.
**Effort :** ~20 LOC handler (delegue a sbfb-manifest).

---

## 6. Ce qui NE change PAS au daemon

- **blob_serve.rs** : Inchange. Le cache LRU et la validation
  path restent identiques. La preview ephemere reutilise la meme
  methode `load()`.
- **publish.rs** : Inchange. `ProjectAnnouncement` reste le meme.
- **gossip subscribe** : Inchange.
- **curator runtime** : Inchange.
- **feed sync iroh-docs** : Inchange (la primitive feed/insert
  gere deja des raw ops).
- **provenance.rs** : Inchange (deploy-from-repo l'utilise deja).
- **auth** : Inchange (bearer + Host + Origin loopback).
- **CORS** : Inchange (Factory est un client loopback).
- **Le shell React** : Inchange (la page `/factory` est une page
  React dans le shell, pas dans Factory CLI. Si Factory veut une
  UI web, c'est une page du shell qui appelle les routes daemon.
  Le CLI et l'UI React sont deux interfaces differentes pour le
  meme workflow).

---

## 7. Impact sur la roadmap S67-S69

### 7.1 S67 — Factory Foundation

| Phase originale | Impact | Changement |
|---|---|---|
| **Phase A** — SBFB.json v2 + retrait node_id + CuratorVouched | Inchange dans le scope. L'extraction en crate `sbfb-manifest` est additionnelle (~0.5 jour). Les types CuratorVouched/Disendorsed restent dans coordinator-rs. Le retrait node_id reste dans deploy.rs. | Ajouter la creation du crate `sbfb-manifest` comme sous-tache. |
| **Phase B** — Template engine + 3 templates | **Change de localisation.** Au lieu d'un module dans `nexus-shell-daemon-core`, c'est un module dans le nouveau crate `sbfb-factory`. Le template engine est identique (substitution, copie bridge SDK, git init). | Creer le crate `sbfb-factory` au lieu d'ajouter du code au daemon. Meme effort. |
| **Phase C** — CLI `sbfb create` | **Change de localisation.** Au lieu d'une sous-commande du daemon CLI, c'est le binaire `sbfb-factory` avec clap. Meme CLI, meme UX, different binary. | Le bin s'appelle `sbfb-factory` (ou `sbfb create` via alias). Meme effort. |
| **Phase D** (stretch) | Inchange — c'est du daemon (aggregation Browse, stale detection). | — |

### 7.2 S68 — Broker / Preview / Publish Gate

| Phase originale | Impact | Changement |
|---|---|---|
| **Phase A** — Broker architecture + routes API factory/* | **Gros changement.** Le "broker" n'est plus un module daemon. Les routes `/api/v1/factory/*` n'existent plus dans le daemon. La seule nouvelle route daemon est `POST /api/v1/preview/load`. Le reste de la logique (diff, apply, audit) est dans `sbfb-factory`. | Remplacer "module factory_broker dans nexus-shell-daemon-core" par "logique dans sbfb-factory + 1 route daemon preview". Effort reduit cote daemon, deplace vers Factory. |
| **Phase B** — Diff + publish gate | **Change de localisation.** Le diff engine et la publish gate checklist sont dans `sbfb-factory`, pas dans le daemon. Le daemon expose seulement les primitives (preview/load, deploy-from-repo). | Meme logique, different crate. |
| **Phase C** — UX confiance (badges, timeline, dissent) + Factory UI | La partie "UX confiance" (Browse, Curators) reste dans le shell React. La "page /factory" dans le shell appelle les routes daemon existantes + eventuellement des routes de `sbfb-factory` si on choisit d'exposer le factory CLI comme serveur local. | Ou bien : la page /factory invoque directement `sbfb-factory preview`, `sbfb-factory publish` via un petit HTTP local que Factory expose. Ou bien : la page /factory appelle les routes daemon et la logique Factory est cote client React. Decision a prendre. |
| **Phase D** — Preview sandbox + proof pack | Preview sandbox utilise la primitive daemon `POST /api/v1/preview/load` + `GET /blob-serve/{hash}/{path}`. Le proof pack est genere par `sbfb-factory`. | Naturel avec l'architecture client. |

### 7.3 S69 — Babel Reader Canari + Pilote

| Phase originale | Impact | Changement |
|---|---|---|
| Toutes les phases | **Impact minimal.** Babel est cree via `sbfb-factory create --domain-pack babel`, deploye via `sbfb-factory publish --daemon ...`. Le daemon reste le meme. Le pilote teste le daemon + Factory CLI ensemble. | La seule difference : le testeur utilise `sbfb-factory` comme outil CLI separe au lieu d'une sous-commande daemon. |

### 7.4 Effort net

| Aspect | Roadmap originale (Factory module daemon) | Roadmap revisee (Factory client externe) |
|---|---|---|
| Code daemon ajoute | ~800-1200 LOC (broker, templates, CLI, diff, scan, preview, audit) | ~150-200 LOC (preview/load route + node_id optionnel + CuratorVouched types) |
| Nouveau crate(s) | 0 | 2 (sbfb-manifest ~200 LOC, sbfb-factory ~1000 LOC) |
| Code total | ~800-1200 LOC daemon | ~1200-1400 LOC repartis |
| Tests daemon | ~30-40 | ~10-15 (primitives uniquement) |
| Tests Factory | 0 (inline daemon) | ~25-35 (tests du CLI et de la logique) |
| Complexite daemon | Monte significativement (le daemon connait les templates, les diffs, les gates) | Inchangee (le daemon fournit des primitives neutres) |
| Reusabilite | Nulle (logique enfouie dans le daemon) | Forte (sbfb-manifest reutilisable par tout client, sbfb-factory remplacable) |

---

## 8. Decision architecturale requise : page /factory dans le shell

Le plan original prevoit une page React `/factory` dans le shell
(S68 Phase C). Deux approches avec Factory-as-client :

**Option A — Page shell pure :** La page `/factory` appelle
directement les routes daemon (deploy, preview, feed). La logique
"gate" (classification, scope, template, diff, scan) est en
JavaScript cote client. Factory CLI et la page React sont deux
interfaces pour le meme workflow, sans code partage.

**Option B — Factory comme serveur local :** `sbfb-factory serve`
expose un petit HTTP sur un port ephemere. La page `/factory` dans
le shell appelle ce serveur local pour les operations Factory
(create, diff, validate, scan) et le daemon pour les operations
protocole (preview, deploy, feed). Plus complexe mais partage le
code entre CLI et UI.

**Option C — CLI only, pas de page shell :** Factory est un outil
CLI uniquement (comme `cargo`, `npm`). La page `/factory` est
reportee ou supprimee. Simple, mais reduit l'accessibilite pour
les developpeurs non-CLI.

**Recommandation :** Option A pour S67-S69 (MVP). La page /factory
appelle les routes daemon directement. Les gates FG0-FG3 sont des
formulaires React qui valident via `sbfb-manifest` (compile en WASM
ou reimplemente en TypeScript). Les gates FG4-FG7 appellent les
routes daemon. Factory CLI reste pour les devs qui preferent le
terminal.

---

## 9. Synthese des changements

### Ce que le daemon gagne (primitives neutres, pas specifiques Factory)

1. **`POST /api/v1/preview/load`** — N'importe quel client peut
   previewer un zip dans blob-serve sans le publier.
2. **`node_id` optionnel dans deploy** — Toute app sans node_id
   est deployable (portabilite templates).
3. **`CuratorVouched` + `CuratorDisendorsed` dans feed** — Tout
   client peut inserer des endorsements.
4. **Crate `sbfb-manifest`** — Tout client peut valider un manifest
   SBFB.json v2 sans daemon.

### Ce que le daemon NE gagne PAS

- Pas de module `factory_broker`
- Pas de routes `/api/v1/factory/*`
- Pas de template engine
- Pas de diff engine
- Pas de secret scanner
- Pas de CLI `sbfb create` dans le daemon
- Pas d'audit log JSONL dans le daemon

Le daemon reste un **protocole P2P neutre**. Factory est un
**outil client** qui orchestre des primitives daemon + de la
logique locale.
