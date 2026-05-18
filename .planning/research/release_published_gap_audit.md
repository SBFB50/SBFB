# Audit : Gap ReleasePublished dans deploy-from-repo

**Date :** 2026-05-18
**Auteur :** Audit cible sur gap identifie dans `s65_s75_factory_babel_canary_research.md` §5
**Confiance :** HIGH — analyse directe du code source, zero extrapolation

---

## 1. Trace complete du flow deploy-from-repo

### 1.1. Entree HTTP

Route : `POST /api/v1/deploy-from-repo` (http.rs:352 -> deploy.rs:65)
Handler : `deploy::deploy_from_repo`

### 1.2. Etapes sequentielles dans deploy.rs

| Etape | Lignes | Action | Etat |
|-------|--------|--------|------|
| 1 | 71 | `normalize_clone_url` — strip `.git`, query, fragment | OK |
| 2 | 72-74 | Validation : `starts_with("http")` — accepte **http ET https** | **GAP** (cf. §4.3) |
| 3 | 76-83 | Validation `commit_sha` optionnel (40 hex) | OK |
| 4 | 85-89 | `is_repo_public()` — HEAD request, verifie 200 | OK |
| 5 | 92-113 | `tempdir` + `clone_repo` (git clone --depth 1) + checkout SHA optionnel | OK |
| 6 | 104-113 | Verification taille clone < 500 MB | OK |
| 7 | 115-128 | Lecture `SBFB.json` — extraction `node_id` + `version` optionnel | OK |
| 8 | 119-128 | **Contrainte : `sbfb.node_id == state.node_id`** — rejet si mismatch | **GAP** (cf. §4.2) |
| 9 | 130-135 | Verification `index.html` a la racine | OK |
| 10 | 137-143 | Resolution commit SHA (git rev-parse si pas fourni) | OK |
| 11 | 145-154 | `zip_directory` — creation archive (exclut .git, symlinks) | OK |
| 12 | 156-157 | `blake3_hash(zip_bytes)` -> `artifact_hash_hex` (64 hex chars) | OK |
| 13 | 159-166 | `generate_provenance()` — signature Ed25519 SLSA L1 | OK |
| 14 | 166 | `prov.app_version = sbfb.version.clone()` | OK |
| 15 | 169-194 | Contributor attestation best-effort (Couche 2 Sybil) | OK |
| 16 | 196-209 | Injection `provenance.json` dans le zip | OK |
| 17 | 211-221 | `BlobsClient::add_bytes(zip_bytes)` -> blob store -> `hash_hex` | OK |
| 18 | 223 | `provenance_blake3_hex(&prov)` -> `prov_hash` | OK |
| 19 | 227-235 | `db.insert_provenance_record(&state.node_id, &prov)` — persistence SQLite | OK |
| 20 | 237-250 | `publish_announcement()` — **Browse + gossip** | OK |
| 21 | - | **AUCUNE insertion feed `ReleasePublished`** | **GAP CRITIQUE** |
| 22 | 252-261 | Response JSON `{ deployed, hash, provenance_hash, commit_sha }` | OK |

### 1.3. Detail de `publish_announcement()` (deploy.rs:316-379)

Cette fonction fait exactement 2 choses :

1. **Gossip broadcast** : construit un `ProjectAnnouncement`, wrappe avec PoW, broadcast via gossip sender. Cible = decouverte temps-reel des noeuds.

2. **Browse entry** : construit un `BrowseEntry` avec `project_id: state.node_id.clone()` et l'ajoute au browse aggregator local.

Elle ne touche PAS au public feed.

### 1.4. Resume des destinations post-deploy

| Destination | Fait ? | Comment |
|-------------|--------|---------|
| Blob store (iroh-blobs) | OUI | etape 17 |
| Provenance record (SQLite `provenance_records`) | OUI | etape 19 |
| Contributor attestation (SQLite) | OUI (best-effort) | etape 15 |
| Browse aggregator (in-memory) | OUI | etape 20 |
| Gossip broadcast (`ProjectAnnouncement`) | OUI | etape 20 |
| **Public feed SQLite (`public_feed` table)** | **NON** | **MANQUANT** |
| **Public feed iroh-docs (`sbfb-feed` namespace)** | **NON** | **MANQUANT** |

---

## 2. L'operation ReleasePublished existe-t-elle ?

### 2.1. Definition dans l'enum

**OUI — completement definie et operationnelle** dans `public_feed.rs:54-58` :

```rust
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
}
```

Avec payload `ReleasePublishedPayload` (public_feed.rs:31-39) :
- `project_id: String` — hex-64
- `repo_url: String` — doit commencer par `https://`
- `commit_sha: String` — hex-40
- `artifact_hash: String` — hex-64
- `provenance_hash: Option<String>` — hex-64 optionnel
- `is_open_source: bool`

### 2.2. Validation

**OUI — stricte** dans `validate_feed_operation()` (public_feed.rs:204-250) :
- `project_id` : hex-64 exact
- `repo_url` : doit commencer par `https://`
- `commit_sha` : hex-40 exact
- `artifact_hash` : hex-64 exact
- `provenance_hash` si present : hex-64 exact
- `is_open_source: true` exige `provenance_hash` present (spec §2.1)
- Taille JSON < 64 KB

### 2.3. Tests

**OUI — extensivement testee** :
- Serde roundtrip (public_feed.rs:544-552)
- Validation stricte positive et negative (public_feed.rs:756-864)
- Insert + persist + replay (public_feed.rs:627-641)
- Hash-chain integrity (public_feed.rs:665-677)
- Signature verification (public_feed.rs:719-736)
- Adversarial : fork-bomb spam (public_feed.rs:1184-1207), payload oversized (public_feed.rs:1209-1226), bad URLs (public_feed.rs:1228-1258), bad hashes (public_feed.rs:1260-1288), seq gap (public_feed.rs:1290-1368), cross-author forgery (public_feed.rs:1370-1407), Ed25519 forgery (public_feed.rs:1412-1446), BLAKE3 tamper (public_feed.rs:1449-1497), PoW difficulty (public_feed.rs:1499-1512), future timestamp (public_feed.rs:1514-1554)
- E2E multi-daemon sync (multi_daemon.rs:340-478)
- E2E offline catchup (multi_daemon.rs:480-589)
- E2E replay idempotent (multi_daemon.rs:590+)
- Materializer (feed_materializer.rs:258-462)

### 2.4. Utilisation actuelle

**Uniquement via le endpoint HTTP `POST /api/daemon/feed/insert`** (feed_sync.rs:445-519).
Ce endpoint :
1. Recoit un `FeedInsertRequest { op: PublicFeedOperation }` en JSON
2. L'insere dans SQLite via `insert_feed_operation()`
3. Publie dans iroh-docs via `publish_feed_entry_to_docs()`
4. Rollback si la publication iroh-docs echoue

Il n'est appele par RIEN dans deploy.rs.

---

## 3. Diagnostic du gap

### 3.1. Nature du gap

**Gap de wiring** — toutes les briques existent independamment, mais le chemin `deploy-from-repo -> feed insert` n'est pas cable.

Le gap est entre l'etape 19 (provenance persisted) et l'etape 20 (Browse/gossip) de deploy.rs. Il faudrait inserer une etape 19bis qui :
1. Construit un `ReleasePublishedPayload` avec les donnees du deploy
2. Appelle `insert_feed_operation()` sur le coordinator DB
3. Publie l'entry dans iroh-docs via `publish_feed_entry_to_docs()`

### 3.2. Donnees disponibles dans deploy.rs a l'endroit du gap

Au moment ou le gap se situe (apres etape 19, avant etape 20), toutes les donnees necessaires sont deja calculees :

| Champ `ReleasePublishedPayload` | Source dans deploy.rs | Format | Compatible feed ? |
|---|---|---|---|
| `project_id` | `state.node_id` | Ed25519 pubkey hex (64 chars) | OUI — hex-64 |
| `repo_url` | `repo_url` (normalise) | `http://...` ou `https://...` | **PARTIEL** — feed exige `https://` |
| `commit_sha` | `commit_sha` (lowercase) | hex-40 | OUI |
| `artifact_hash` | `artifact_hash_hex` | BLAKE3 hex-64 | OUI |
| `provenance_hash` | `prov_hash` | BLAKE3 hex-64 | OUI |
| `is_open_source` | `true` (hardcode dans `AnnouncementParams`) | bool | OUI |

### 3.3. Effort requis

**PETIT** — estimation 20-40 lignes de code dans deploy.rs :

```rust
// Apres etape 19 (provenance persisted), avant etape 20 (publish_announcement)

// Etape 19bis: Insert ReleasePublished into public feed
if let Some(feed_state) = &state.feed_sync_state {
    let op = nexus_coordinator_rs::public_feed::PublicFeedOperation::ReleasePublished(
        nexus_coordinator_rs::public_feed::ReleasePublishedPayload {
            project_id: state.node_id.clone(),
            repo_url: repo_url.clone(),  // ATTENTION: doit etre https://
            commit_sha: commit_sha.clone(),
            artifact_hash: artifact_hash_hex.clone(),
            provenance_hash: Some(prov_hash.clone()),
            is_open_source: true,
        },
    );

    let keypair = Arc::clone(&state.pow_keypair);
    let author_pubkey = hex::encode(keypair.public_bytes());

    let entry = {
        let db = state.coordinator_db.lock().unwrap_or_else(|p| p.into_inner());
        nexus_coordinator_rs::public_feed::insert_feed_operation(
            &db,
            op,
            &author_pubkey,
            |data| keypair.sign(data).to_vec(),
        )
    };

    match entry {
        Ok(entry) => {
            if let Err(e) = crate::feed_sync::publish_feed_entry_to_docs(feed_state, &entry).await {
                warn!(error = %e, "feed publish failed after deploy (non-fatal)");
                // Rollback l'entry feed si possible
                if let Ok(db) = state.coordinator_db.lock() {
                    let _ = db.delete_feed_entry_if_tail(&entry.entry_hash);
                }
            }
        }
        Err(e) => {
            warn!(error = %e, "feed insert failed after deploy (non-fatal)");
        }
    }
}
```

**Prerequis** : `publish_feed_entry_to_docs` doit etre `pub` — c'est deja le cas (feed_sync.rs:50).

### 3.4. Decision architecturale : fatal ou best-effort ?

Le deploy actuel traite le gossip broadcast et le browse entry comme best-effort silencieux (pas de retour d'erreur si ca echoue). La provenance DB insert est aussi best-effort (debug log, pas d'erreur 500).

**Recommandation : best-effort** — coherent avec le pattern existant. L'archive + provenance sont les artefacts primaires. Le feed est un index public secondaire. Un echec feed ne doit pas bloquer un deploy reussi.

En revanche, l'erreur doit etre tracee pour monitoring (warn, pas debug).

---

## 4. Autres gaps du publish path

### 4.1. `SBFB.json.node_id` contrainte de deploy

**CONFIRME — gap reel.**

deploy.rs:119-128 :
```rust
if sbfb.node_id != state.node_id {
    return error_response(StatusCode::BAD_REQUEST, ...);
}
```

Chaque app deployee doit avoir le `node_id` exact du daemon dans `SBFB.json`. Cela bloque :
- Templates portables (Factory genere un scaffold avec `PLACEHOLDER` -> rejet)
- Apps multi-daemon (un meme repo deploye sur 2 daemons differents)
- CI/CD classique (le node_id change a chaque rebuild)

**Impact Factory** : la Factory devra injecter le `node_id` du daemon courant dans `SBFB.json` juste avant le deploy. C'est faisable mais contraint le workflow.

**Audite separement** : oui (confirme dans le doc canary research). Ce gap est cadre dans factory_deploy_constraint_research.md.

### 4.2. `project_id`/Browse lie au `node_id`

**CONFIRME — gap de design.**

deploy.rs:363 :
```rust
project_id: state.node_id.clone(),
```

Et http.rs:987 (handler publish) :
```rust
project_id: state.node_id.clone(),
```

Et runtime.rs:1374 (gossip ingest) :
```rust
project_id: ann.node_id.clone(),
```

**Consequence** : toutes les apps publiees par le meme daemon ont le MEME `project_id` (= le `node_id` du daemon). Il n'y a pas de mecanisme pour distinguer 2 apps differentes publiees par le meme noeud dans le Browse ou le feed.

Cela signifie que pour le public feed :
- Un `ReleasePublished` avec `project_id = node_id` represente "le daemon a publie quelque chose" — pas "l'app X version Y est publiee"
- Si le meme daemon publie Babel puis un autre app, les 2 `ReleasePublished` ont le meme `project_id`
- Le materializer remplace `latest_release_hash` a chaque nouvelle publication — la premiere app "disparait" de la vue materialisee

**Ce n'est pas bloquant pour le canari** (un seul daemon publie une seule app) mais c'est un gap de design pour le multi-app.

**Severity** : P2 pre-multi-app, P3 pour canari mono-app.

### 4.3. Protocole `http` vs `https`

**CONFIRME — gap reel et bloquant pour le wiring feed.**

deploy.rs:72 :
```rust
if !repo_url.starts_with("http") {
    return error_response(...);
}
```

Accepte `http://` ET `https://`.

public_feed.rs:218 :
```rust
if !p.repo_url.starts_with("https://") {
    return Err("repo_url must start with https://".to_string());
}
```

Exige `https://` uniquement.

**Consequence** : si un utilisateur deploie avec `http://github.com/user/repo`, le deploy reussit (archive + provenance creees) mais l'insertion feed `ReleasePublished` echouerait avec l'erreur `repo_url must start with https://`.

**Resolution** : 2 options, je recommande la premiere :
1. **Durcir deploy.rs** : remplacer `starts_with("http")` par `starts_with("https://")`. Coherent avec la politique securite du projet.
2. **Normaliser dans le chemin feed** : upgrader `http://` -> `https://` avant l'insertion feed. Mauvaise idee — masque un probleme de securite.

`normalize_clone_url()` (forge.rs:22-33) ne touche PAS au scheme — il strip `.git`, query, fragment. Il faudrait soit modifier `normalize_clone_url` soit ajouter une validation explicite dans deploy.rs.

### 4.4. Replication storage pour nouvelle app

**NON AUDITE ICI** — hors scope de l'audit feed. Le gap mentionne dans le doc canary concerne le fait que `storage_get/storage_set` pour une app Babel necessiterait un namespace iroh-docs dedie, et le mecanisme P2P complet de replication storage n'est pas encore prouve pour une app tierce.

Ce gap est cadre dans la roadmap comme prerequis S66 (durabilite).

---

## 5. Impact sur le sequencage

### 5.1. Analyse par sprint

| Sprint | Pertinence | Justification |
|--------|------------|---------------|
| **S65** | **OUI — prioritaire** | Le S65 porte sur le "contrat public" du feed. L'insertion automatique `ReleasePublished` lors du deploy est la manifestation concrete de ce contrat. Sans ce wiring, le feed est un silo deconnecte du deploy path. |
| S66 | Non | La durabilite concerne le blob store et le storage, pas le wiring feed. |
| S67 | Trop tard | Factory a S67 depend du wiring. Si le wiring n'est pas fait avant, Factory ne peut pas produire un publish path complet. |
| S68 | Non | Le proof pack depend du feed, mais le gap est en amont du proof pack. |

### 5.2. Recommandation

**Le wiring `deploy-from-repo -> ReleasePublished` doit etre fait en S65.**

Raisons :
1. C'est petit (20-40 lignes) — pas un sprint entier.
2. C'est une dependance bloquante de S67 (Factory publish path).
3. C'est une dependance du publish path canari (§9 etape 11).
4. Le fix du protocole `http` vs `https` (§4.3) doit aussi etre en S65 — c'est un prerequis du wiring.
5. S65 a deja `P2-FEED-INSERT-NO-AUTH-TIER` dans ses carries — le wiring feed est dans le meme territoire.

### 5.3. Items S65 supplementaires identifies par cet audit

| Item | Priorite | Effort |
|------|----------|--------|
| Wiring `deploy-from-repo -> insert_feed_operation + publish_feed_entry_to_docs` | P1 MANDATORY | Petit (~30 LOC + tests) |
| Fix `repo_url` validation : `starts_with("http")` -> `starts_with("https://")` dans deploy.rs | P1 MANDATORY (prereq du wiring) | Trivial (~1 LOC) |
| Test E2E : deploy -> feed entry created | P1 MANDATORY | Moyen (~50 LOC test) |
| Test negatif : deploy fail -> pas de feed entry | P2 | Petit (~30 LOC test) |
| Test : feed entry contient bon artifact_hash, commit_sha, repo_url | P1 | Moyen (~40 LOC test) |

---

## 6. Tests necessaires

### 6.1. Test unitaire : deploy -> feed entry created (integration-level)

```
Scenario : deploy_from_repo reussit avec feed_sync_state initialise
Given : DaemonHttpState avec feed_sync_state = Some(...)
When : deploy_from_repo() retourne 200
Then : count_feed_entries() == 1
And : la feed entry a op_type = "ReleasePublished"
And : la feed entry a project_id = state.node_id
And : la feed entry a repo_url = <repo normalise>
And : la feed entry a commit_sha = <commit du repo>
And : la feed entry a artifact_hash = blake3(zip)
And : la feed entry a provenance_hash = blake3(provenance.json)
And : la feed entry a is_open_source = true
```

Ce test est DIFFICILE a ecrire en unitaire pur car `deploy_from_repo` fait un vrai `git clone`. Options :
- **Test integration** (avec un vrai repo Git local) : le pattern existe deja dans les tests deploy existants du http.rs (cf. tests deploy_from_repo_non_http_url_returns_400)
- **Test E2E multi-daemon** : ajouter au DaemonCluster existant

### 6.2. Test negatif : deploy fail -> pas de feed entry

```
Scenario : deploy_from_repo echoue (repo invalide, SBFB.json missing, etc.)
Given : DaemonHttpState avec feed_sync_state = Some(...)
When : deploy_from_repo() retourne 400
Then : count_feed_entries() == 0
```

Ce test est plus simple car les echecs arrivent avant le point d'insertion feed.

### 6.3. Test : feed entry contient les bonnes donnees

```
Scenario : verification du contenu de la feed entry
Given : deploy_from_repo reussit
When : on replay le feed
Then : la ReleasePublishedPayload contient :
  - artifact_hash == hex::encode(blake3(zip_sans_provenance))
  - commit_sha == git rev-parse HEAD du repo clone
  - repo_url commence par "https://"
  - provenance_hash est present et non-vide
  - project_id == 64 hex chars
```

### 6.4. Test : feed entry non creee quand feed_sync_state est None

```
Scenario : deploy sans feed initialise (backward compat)
Given : DaemonHttpState avec feed_sync_state = None
When : deploy_from_repo() retourne 200
Then : count_feed_entries() == 0
And : pas d'erreur dans les logs (sauf debug)
```

### 6.5. Test adversarial : deploy avec repo http:// refuse

```
Scenario : durcissement protocole
Given : repo_url = "http://github.com/user/repo"
When : POST /api/v1/deploy-from-repo
Then : 400 Bad Request
And : message contient "https"
```

### 6.6. Test E2E complet : deploy -> feed -> sync -> verify

```
Scenario : publish path complet multi-daemon
Given : DaemonCluster(2), B joint le feed de A
When : A fait deploy-from-repo
Then : A a 1 feed entry ReleasePublished
And : B recoit la feed entry (poll feed_status count >= 1)
And : verify_chain sur le feed de B passe
And : B peut materialiser un PublicRegistryView avec le projet
```

Ce test serait l'equivalent du publish path canari §9.

---

## 7. Risques du wiring

### 7.1. Mutex contention

Le deploy.rs fait deja 2 acquisitions du `coordinator_db` mutex :
1. Contributor attestation (etape 15)
2. Provenance insert (etape 19)

Ajouter une 3eme acquisition pour le feed insert est acceptable car :
- Chaque acquisition est breve (~1ms SQL)
- `deploy-from-repo` n'est pas un chemin chaud (1-2 appels par jour)
- Le pattern est identique a `feed_insert` dans feed_sync.rs:463-487

### 7.2. Feed entry hash-chain coherence

Le `insert_feed_operation` est transactionnel (BEGIN IMMEDIATE ... COMMIT) et recalcule `prev_hash` a partir de la derniere entry de l'auteur. Il n'y a pas de risque de corruption si 2 deploys arrivent en parallele — le mutex serialise les acces.

### 7.3. iroh-docs publish failure

Le pattern existant dans feed_sync.rs:489-508 gere deja le cas : si `publish_feed_entry_to_docs` echoue, rollback de l'entry SQLite via `delete_feed_entry_if_tail`. Ce meme pattern doit etre replique dans deploy.rs.

### 7.4. Backward compatibility

Si `feed_sync_state` est `None` (feed pas initialise, ancien daemon, tests), le wiring doit etre un no-op silencieux. Le `if let Some(feed_state) = &state.feed_sync_state { ... }` gere ce cas.

---

## 8. Synthese

### Gap confirmes

| # | Gap | Severity | Effort | Sprint cible |
|---|-----|----------|--------|-------------|
| G1 | `deploy-from-repo` ne cree pas de `ReleasePublished` dans le public feed | **P0 bloquant** pour publish path canari | Petit (30 LOC) | **S65** |
| G2 | `deploy-from-repo` accepte `http://` alors que le feed exige `https://` | **P1 bloquant** (prereq G1) | Trivial (1 LOC) | **S65** |
| G3 | `project_id = node_id` pour toutes les apps du meme daemon | P2 design debt | Moyen (refactor) | S67+ (multi-app) |
| G4 | `SBFB.json.node_id` contrainte de deploy bloque templates portables | P2 design debt | Moyen (refactor) | S67 (Factory) |
| G5 | Pas de test E2E deploy -> feed | P1 couverture | Moyen (50 LOC) | **S65** |

### Gap infirmes

| # | Claim du doc canary | Verdict |
|---|---------------------|---------|
| - | "La replication storage generique pour une nouvelle app Babel n'est pas encore prouvee" | Hors scope audit feed — confirme par le doc comme sujet S66 |

### Sequencage recommande dans S65

1. **Fix G2** en premier (1 LOC) — prerequis
2. **Wire G1** (30 LOC) — core feature
3. **Tests G5** (100 LOC) — validation
4. Review G3/G4 — documentation du gap pour S67

### Code path final apres fix

```
POST /api/v1/deploy-from-repo
  -> normalize_clone_url
  -> validate https://     [FIX G2]
  -> clone + SBFB.json + index.html
  -> zip + blake3
  -> generate_provenance + sign
  -> inject provenance.json in zip
  -> blob store
  -> provenance DB insert
  -> feed insert ReleasePublished + iroh-docs publish  [FIX G1]
  -> publish_announcement (Browse + gossip)
  -> 200 OK
```
