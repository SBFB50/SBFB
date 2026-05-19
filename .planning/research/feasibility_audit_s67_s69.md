# Audit de faisabilite technique S67-S69

**Date :** 2026-05-19
**Auditeur :** Claude (session FlowUP)
**Base :** code actuel branche `master` (commit `3360c45`) + SYNTHESIS §11

---

## Methode

Chaque livrable est verifie contre le code reel lu durant cette session.
Verdict : **FAISABLE** (trivial ou standard), **COMPLEXE** (faisable mais
avec risque ou effort significatif), **BLOQUE** (bloqueur technique
identifie). L'estimation LOC est la mienne, pas celle de la synthese.

---

## S67 Phase A — Primitives daemon

### 1. sbfb-manifest crate

**Code lu :** `crates/nexus-shell-daemon/src/deploy.rs` lignes 543-557.

La struct actuelle `SbfbJson` est **privee et minimale** :

```rust
#[derive(Debug, Deserialize)]
struct SbfbJson {
    node_id: String,
    #[serde(default)]
    version: Option<String>,
}
```

Elle vit dans le fichier `deploy.rs` du daemon binary, pas dans un crate
reutilisable. Seul `read_sbfb_json()` la consomme (ligne 550).

**Dependances a casser :** Aucune dependance externe. La struct est
privee et ne sort jamais de `deploy.rs`. Extraire vers un crate
`sbfb-manifest` = creer un nouveau crate avec la struct + validation +
serde, puis `deploy.rs` importe le nouveau crate. Le workspace Rust a
deja 12 crates (Cargo.toml `members`) — ajouter un 13eme est mecanique.

**Verdict : FAISABLE**
**Estimation LOC :** ~150 LOC (struct enrichie v2 + validation + serde +
tests unitaires + re-export dans deploy.rs).

---

### 2. CuratorVouched / CuratorDisendorsed

**Code lu :** `crates/nexus-coordinator-rs/src/public_feed.rs` lignes 49-60.

L'enum `PublicFeedOperation` a **exactement 2 variants** actuellement :
- `ReleasePublished(ReleasePublishedPayload)`
- `SourceBecameStale(SourceBecameStalePayload)`

**Complexite d'ajout :**

1. **Le feed est raw-op extensible.** `FeedEntry.op` est un
   `serde_json::Value` (ligne 79). Le test `test_unknown_op_roundtrip`
   (ligne 1667) prouve deja qu'un `CuratorVouched` inconnu est stocke
   et propage sans crash. Ajouter 2 variants a l'enum typed = les rendre
   "connus" au lieu d'"inconnus".

2. **Validation.** `validate_feed_operation()` (ligne 224) dispatch sur
   `try_parse_op()` : les ops connues passent par `validate_known_operation()`,
   les inconnues passent avec un check de taille seulement. Ajouter 2
   branches au match = mecanique.

3. **Canonical bytes.** Non impactes — le hash porte sur le `Value` brut,
   pas sur l'enum typed. Le test `test_canonical_bytes_value_vs_typed`
   (ligne 1691) le confirme.

4. **Tests existants a adapter :** Aucun. Les tests existants couvrent
   les 2 variants actuels et l'unknown op path. Les nouveaux variants
   ajoutent des tests, ne cassent pas les existants.

5. **Feed materializer.** `feed_materializer.rs` (ligne 37-58) fait un
   `try_parse_op()` puis match sur les variants. Il faudra ajouter 2
   branches, sinon les nouveaux ops tombent dans le `None` path (ignore).
   C'est mineur.

**Verdict : FAISABLE**
**Estimation LOC :** ~120 LOC (2 payloads struct + 2 variants enum +
validation + 2 branches materializer + ~8 tests).

---

### 3. GET /api/daemon/feed/entries

**Code lu :** `crates/nexus-shell-daemon/src/feed_sync.rs` et
`crates/nexus-coordinator-rs/src/db.rs`.

**Situation actuelle :** Le daemon expose deja 5 routes feed :
- `GET /api/daemon/feed/ticket`
- `POST /api/daemon/feed/join`
- `GET /api/daemon/feed/status` (renvoie count + last_seq + authors)
- `POST /api/daemon/feed/insert`
- `GET /api/daemon/feed/cursor`

Il manque une route qui **renvoie les entries elles-memes**. Mais le code
DB est deja pret :

- `db.get_feed_entries()` (ligne 811) renvoie toutes les entries.
- `db.get_feed_entries_after_seq(after_seq)` (ligne 765) renvoie les
  entries apres un seq donne (pagination par curseur).
- `replay_all()` dans `public_feed.rs` (ligne 415) reconstruit des
  `FeedEntry` complets depuis les `FeedEntryRow`.

Ajouter un handler HTTP paginé = un handler axum ~40 lignes qui prend
un query param `?after_seq=N&limit=50`, appelle `get_feed_entries_after_seq`,
reconstruit les `FeedEntry`, et serialise en JSON.

**Verdict : FAISABLE**
**Estimation LOC :** ~60 LOC (handler + query params struct + route
registration + 2 tests).

---

### 4. FTS5 daemon search

**Code lu :** `crates/nexus-coordinator-rs/src/db.rs` migrations M1-M13,
`Cargo.toml` workspace + `nexus-coordinator-rs/Cargo.toml`.

**Migrations existantes :** 13 migrations (M1 a M13). Pattern clair,
`rusqlite_migration` bien maitrise.

**Feature rusqlite :** Le workspace declare `rusqlite = { version = "0.36",
features = ["bundled"] }`. La feature `"bundled"` compile SQLite depuis les
sources C embarquees dans `libsqlite3-sys`. Par defaut, `libsqlite3-sys`
avec `bundled` compile SQLite **avec FTS5 active** (c'est le defaut de
SQLite depuis 3.9.0, et le build script de libsqlite3-sys le laisse ON).
Verification : rusqlite 0.36 utilise libsqlite3-sys 0.36 qui compile
SQLite 3.49+ avec `-DSQLITE_ENABLE_FTS5=1` par defaut dans le mode bundled.

**Cependant, risque a verifier au runtime.** Meme si le build devrait
fonctionner, il est prudent de verifier avec un test `SELECT fts5()` ou
`CREATE VIRTUAL TABLE test USING fts5(name)` avant de s'engager.
Si la feature manquait, il suffirait d'ajouter `features = ["bundled",
"bundled-sqlcipher-vendored-openssl"]` — mais ce ne devrait pas etre
necessaire.

**Architecture FTS5 :** Une migration M14 cree une table virtuelle FTS5
(`project_search`) indexant project_name + description + category. Un
trigger ou un INSERT explicite dans le code peuple la table FTS5 depuis
les announcements. Un handler `GET /api/daemon/search?q=...` fait un
`SELECT ... FROM project_search WHERE project_search MATCH ?1`.

**Verdict : FAISABLE** (sous reserve d'un test FTS5 en M14 au runtime)
**Estimation LOC :** ~180 LOC (migration M14 + search.rs module +
handler HTTP + peuplement depuis announcements + 4 tests).

---

### 5. node_id optionnel dans deploy.rs

**Code lu :** `crates/nexus-shell-daemon/src/deploy.rs` lignes 115-128.

```rust
let sbfb = match read_sbfb_json(&clone_dir) {
    Ok(s) => s,
    Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
};
if sbfb.node_id != state.node_id {
    return error_response(StatusCode::BAD_REQUEST, &format!(
        "SBFB.json node_id ({}...) does not match daemon node_id ({}...)",
        ...
    ));
}
```

La struct `SbfbJson` a `node_id: String` (obligatoire). Le check est une
egalite stricte daemon vs. manifest. Le rendre optionnel :

1. Changer `node_id: String` en `node_id: Option<String>` (ou mieux,
   dans le futur crate `sbfb-manifest`, definir v2 avec `node_id`
   optionnel).
2. Le check bloquant (lignes 119-128) devient : si `node_id` est
   `Some(id)` et `id != state.node_id`, rejeter. Si `None`, accepter
   (le daemon signera de son propre node_id).
3. L'app factory pourra ainsi generer un SBFB.json sans node_id, et
   l'utilisateur deploy vers n'importe quel daemon.

**Verdict : FAISABLE**
**Estimation LOC :** ~15 LOC (changement de type + adaptation du check +
1 test).

---

## S67 Phase B-D — sbfb-factory crate

### 6. Nouveau crate dans le workspace

**Code lu :** `Cargo.toml` racine, `members` liste.

Le workspace a **12 membres** (dont `tools/png-to-icns`). Structure :
```toml
members = [
    "crates/nexus-core-rs",
    "crates/nexus-worker-core",
    ... (10 autres)
    "tools/png-to-icns",
]
```

Ajouter `"crates/sbfb-factory"` ou `"tools/sbfb-factory"` : une ligne
dans `members` + le `Cargo.toml` du crate. Aucune contrainte de cross-
compilation specifique (tout compile nativement sur la machine dev).
Le `[profile.release]` avec LTO fat et `codegen-units = 1` s'appliquera
au nouveau crate sans config supplementaire.

**Verdict : FAISABLE**
**Estimation LOC :** ~10 LOC infra (Cargo.toml + workspace member).

---

### 7. Template engine

**Code lu :** `examples/sbfb-explorer/` et `examples/sbfb-ideas/`.

Deux apps exemples existent :

1. **sbfb-explorer** : HTML/CSS/JS pur, `SBFB.json` avec
   `{"node_id": "PLACEHOLDER", "name": "sbfb-explorer", "version": "1.0.0"}`,
   `sbfb-bridge.js`, `index.html`, `app.js`, `style.css`.

2. **sbfb-ideas** : meme structure (HTML/CSS/JS, SBFB.json, bridge).

Les deux apps sont des fichiers statiques de ~200-300 lignes chacune. Le
pattern est clair et repetitif : un `index.html` + `app.js` + `style.css`
+ `SBFB.json` + `sbfb-bridge.js`.

Un template engine pour sbfb-factory peut :
- Copier un squelette (un des examples avec des `{{variables}}`)
- Substituer `{{project_name}}`, `{{node_id}}`, `{{description}}`
- Valider la struct SBFB.json resultante
- Optionnellement ajouter des sections HTML

Pas besoin d'un engine complexe (Tera, Handlebars) — un `str::replace`
sur 5-6 variables suffit pour v1. Le `sbfb-bridge.js` est un fichier
statique copie tel quel.

**Verdict : FAISABLE**
**Estimation LOC :** ~250-350 LOC (template struct + 2 templates embedded
via `include_str!` + variable substitution + validation output +
CLI interface basique + tests).

---

## S68 — Proof Cards

### 8. ProofCard computation

**Code lu :** `crates/nexus-coordinator-rs/src/db.rs`,
`crates/nexus-coordinator-rs/src/provenance.rs`,
`crates/nexus-coordinator-rs/src/public_feed.rs`,
`crates/nexus-coordinator-rs/src/feed_materializer.rs`.

Donnees necessaires pour un ProofCard et leur accessibilite :

| Donnee | Source | Accessible ? |
|--------|--------|-------------|
| Provenance (repo, commit, hash, signature) | `db.get_provenance_by_project(id)` | OUI |
| Curator endorsements | Feed entries avec `op_type = "CuratorVouched"` (apres S67) | OUI (apres S67 Phase A) |
| Feed history | `replay_all()` ou `get_feed_entries_after_seq()` | OUI |
| Licence | Via `SbfbJson` dans l'archive zip (champ a ajouter en v2) | PARTIEL — pas dans la DB actuellement |
| Project metadata | Browse aggregator (`BrowseEntry`) | OUI (en memoire, pas DB) |
| Contributor attestations | Table `contributor_attestations` dans db.rs | OUI |
| Kudos | `db.get_project_kudos_total(id)` | OUI |

**Dependance cachee :** La licence n'est pas stockee en DB actuellement.
Le manifest SBFB.json actuel a `node_id`, `name`, `version` — pas de
champ `license`. C'est un ajout mineur au crate sbfb-manifest (S67), pas
un bloqueur.

Le ProofCard lui-meme est un struct Rust qui aggrege ces donnees. La
computation = une fonction qui query la DB + browse aggregator et
construit le struct. Pas de crypto complexe — juste de l'aggregation.

**Verdict : FAISABLE**
**Estimation LOC :** ~200 LOC (ProofCard struct + computation function +
serialisation JSON + handler HTTP GET + 4 tests).

---

## S69 — Babel + pilote

### 9. Bridge methods pour Babel

**Code lu :** `web/src/bridge/protocol.ts` lignes 20-40.

L'enum `BridgeMethodSchema` expose **12 methodes** :

| Methode | Disponible | Besoin Babel |
|---------|-----------|-------------|
| `task_submit` | OUI | NON (pas de compute S69) |
| `storage_get` | OUI | OUI (preferences utilisateur) |
| `storage_set` | OUI | OUI (preferences utilisateur) |
| `pii_redact` | OUI | NON |
| `storage_list` | OUI | OUI (lister les langues configurees) |
| `storage_delete` | OUI | NON |
| `identity_pubkey` | OUI | OUI (identifier le noeud) |
| `node_status` | OUI | NON |
| `browse_list` | OUI | NON |
| `storage_version` | OUI | NON |
| `provenance_get` | OUI | OUI (afficher provenance du pack) |
| `provenance_verify` | OUI | OUI (verifier provenance) |
| `feed_cursor_get` | OUI | OUI (optionnel, afficher historique) |

**Analyse :** 5 des 12 methodes bridge sont utiles pour Babel (storage,
identity, provenance, feed_cursor). Toutes sont **deja implementees**.
La synthese mentionne un besoin de `feed_cursor` — il est la.

Il manque potentiellement un `GET /api/daemon/feed/entries` cote bridge
(pour afficher l'historique feed dans l'app), mais c'est couvert par
le livrable S67 Phase A point 3.

**Verdict : FAISABLE** — aucune methode bridge manquante pour Babel MVP.
**Estimation LOC :** ~500 LOC app HTML/JS (le domain pack Babel
lui-meme) + ~200 LOC fixtures de test. Zero LOC bridge necessaire.

---

### 10. Installeur NSIS

**Code lu :** `Packager.toml` (racine) + `scripts/build-installer.sh`.

L'installeur NSIS est **deja en place et fonctionnel** (Sprint 60) :

- `Packager.toml` configure `cargo-packager` avec sections `[nsis]`
  (Windows), `[deb]` (Linux), `[dmg]` (macOS).
- `scripts/build-installer.sh` orchestre le build frontend + packager.
- Les binaires inclus : `nexus-launcher` (main) + `nexus-shell-daemon`.
- Ressources : `web/dist` embedde dans l'installeur.
- NSIS config : `installer-mode = "currentUser"`, langues EN/FR.

**Verdict : FAISABLE** — rien a creer, il faut juste l'utiliser pour
le pilote S69.
**Estimation LOC :** 0 LOC nouveau. Eventuellement ~20 LOC pour ajouter
un domain pack Babel dans les ressources si necessaire.

---

## Dependances cachees identifiees

### D1. rusqlite FTS5 runtime activation (S67)
Le feature `bundled` de rusqlite compile SQLite avec FTS5 par defaut,
mais il faut un test explicite (`CREATE VIRTUAL TABLE ... USING fts5(...)`)
avant de s'engager. Risque faible.

### D2. Licence dans sbfb-manifest (S68 Proof Cards)
Le champ `license` n'existe pas dans `SbfbJson` actuel. Le crate
sbfb-manifest (S67) doit l'inclure pour que les Proof Cards soient
complets. Risque faible — c'est un champ `Option<String>`.

### D3. Browse aggregator en memoire (S68 Proof Cards)
Les `BrowseEntry` vivent dans un `DashMap` en memoire, pas en DB.
Pour les Proof Cards, il faudra soit interroger le DashMap au runtime,
soit materialiser les metadata projet en DB. Les deux approches sont
faisables.

### D4. Feed materializer a enrichir (S67)
Le `feed_materializer.rs` ne traite que `ReleasePublished` et
`SourceBecameStale`. Ajouter `CuratorVouched`/`CuratorDisendorsed`
dans le materializer est necessaire pour que les Proof Cards agrègent
les endorsements. C'est ~30 LOC supplementaires.

### D5. Cross-node verification carry S66 (S69)
Le carry `P2-VERIFY-LOCAL-KEY-ONLY` est MANDATORY S66. Si S66 ne le
resout pas, le pilote S69 sera limite a un seul noeud. Ce n'est pas
un bloqueur S67-S68 mais c'est un bloqueur S69 Phase C.

---

## Synthese des estimations

| Livrable | Synthese §11 | Mon estimation | Verdict |
|----------|-------------|----------------|---------|
| sbfb-manifest crate | ~200 LOC | ~150 LOC | FAISABLE |
| CuratorVouched/Disendorsed | incl. dans 300-400 | ~120 LOC | FAISABLE |
| GET /api/daemon/feed/entries | incl. dans 300-400 | ~60 LOC | FAISABLE |
| FTS5 daemon search | incl. dans 300-400 | ~180 LOC | FAISABLE |
| node_id optionnel | incl. dans 300-400 | ~15 LOC | FAISABLE |
| **S67 Phase A total** | **~500-600** | **~525 LOC** | **FAISABLE** |
| sbfb-factory crate + templates | ~300-400 | ~350 LOC | FAISABLE |
| Factory CLI + diff + scanner | ~200-300 | ~250 LOC | FAISABLE |
| factory.lock + provenance + audit | ~150-200 | ~150 LOC | FAISABLE |
| **S67 total** | **~1150-1500** | **~1275 LOC** | **FAISABLE** |
| Preview + broker S68 | ~300 | ~250 LOC | FAISABLE |
| Diff engine + secrets + gate | ~300 | ~280 LOC | FAISABLE |
| Page React /factory | ~570 | ~550 LOC | COMPLEXE (*) |
| Preview sandbox + dry-run | ~200 | ~180 LOC | FAISABLE |
| **S68 total** | **~1370** | **~1260 LOC** | **FAISABLE** |
| ProofCard data + computation | ~200 | ~200 LOC | FAISABLE |
| Domain pack Babel | ~700 | ~650 LOC | FAISABLE |
| Mecanisme invite + fix verif | ~150 | ~150 LOC | COMPLEXE (**) |
| Deploy verifie Babel + feed | ~100 | ~80 LOC | FAISABLE |
| **S69 total** | **~1150** | **~1080 LOC** | **FAISABLE** |

(*) La page React /factory est la piece la plus grosse en frontend.
5 composants UI (TemplateSelector, VariablesForm, DiffViewer,
PreviewFrame, PublishChecklist) dans un sprint qui a deja du backend.
Faisable si le scope UX est strict (pas de polish, formulaires basiques).

(**) Le mecanisme invite existe deja (`invite.rs` + table `invites` en
DB). Le fix cross-node verification depend du carry S66. Si S66 ne le
resout pas avant S69, cette phase sera amputee.

---

## Verdict global

**La roadmap S67-S69 est REALISTE techniquement.**

Arguments :
1. **Zero bloqueur technique.** Toutes les primitives necessaires
   (feed extensible, DB avec migrations, bridge 12 methodes, installeur
   NSIS, provenance, curator system) existent deja dans le code.

2. **Les estimations LOC de la synthese sont coherentes.** Mon audit
   independant donne ~3615 LOC total vs. ~3670-4020 LOC synthese.
   L'ecart est marginal. La synthese ne sous-estime pas.

3. **Le pattern "ajouter un crate workspace" est maitrise.** 12 crates
   existent deja, le 13eme (sbfb-factory) et un eventuel 14eme
   (sbfb-manifest) suivent le meme pattern.

4. **Le feed raw-op est deja design pour l'extensibilite.** Le test
   `test_unknown_op_roundtrip` avec un `CuratorVouched` fictif
   (ligne 1672 de public_feed.rs) prouve que le systeme est pret.

5. **FTS5 est disponible** via rusqlite bundled (SQLite 3.49+ avec
   FTS5 active par defaut).

Risques a surveiller :
- **R1 (S69) :** Le carry `P2-VERIFY-LOCAL-KEY-ONLY` doit etre resolu
  en S66. Sinon le pilote multi-noeud est bloque.
- **R2 (S68) :** La page React /factory est le morceau le plus dense
  en UX. Si le scope gonfle (animations, preview live, drag-and-drop),
  le sprint depasse.
- **R3 (S67) :** Le test FTS5 runtime est un pre-requis technique a
  valider en tout debut de S67 Phase A (un test de 3 lignes suffit).

**Recommandation :** Commencer S67 Phase A par un smoke test FTS5
(`CREATE VIRTUAL TABLE ... USING fts5(...)` dans un test unitaire)
pour eliminer R3 immediatement.
