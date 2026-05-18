# Audit de faisabilite : Factory en S67 (remontee depuis S73)

**Date :** 2026-05-18
**Mode :** Feasibility + Architecture
**Confiance globale :** HIGH (analyse exhaustive du code source, croisement avec
7 documents de recherche existants, aucune source externe requise)

---

## 0. Verdict court

**La remontee est techniquement faisable mais a un cout structurel eleve.**

Le pivot propose (Factory S67-S69 au lieu de S73-S75) est motive par un
argument produit valide : prouver que Factory fonctionne avant de construire
RRV et la Gouvernance. Cependant, l'analyse exhaustive du code revele que
le cout est superieur a ce que le pivot document laisse entendre, et que
la Gouvernance deplacee cree un trou de confiance dans le pilote.

**Recommandation : variante D (CuratorVouched minimal en S65, Factory en S67).**
C'est le seul chemin qui ne sacrifie pas la confiance pour la vitesse.

---

## 1. Prerequisites techniques de Factory — inventaire exhaustif

### 1.1 deploy.rs — le flow deploy complet (753 lignes)

**Etat actuel :** Deux endpoints. `deploy-from-repo` est le chemin verifie :
clone repo → lit SBFB.json → verifie node_id == daemon → verifie index.html →
zip → BLAKE3 → provenance Ed25519 → blob store → gossip announce → Browse entry.

**Ce qui existe et fonctionne :**
- Clone repo public (git clone --depth 1)
- Validation commit_sha (40 hex)
- Verification repo public (HEAD request)
- Validation taille clone (< 500 MB)
- Lecture SBFB.json (struct `SbfbJson { node_id, version }`)
- Verification index.html a la racine
- Zip creation (exclut .git, symlinks, path traversal)
- BLAKE3 hash de l'archive
- Generation provenance SLSA L1 signee Ed25519
- Attestation contributeur (best-effort Couche 2)
- Injection provenance dans le zip
- Blob store + gossip announce + Browse entry
- 11 tests unitaires couvrant les cas normaux et limites

**Ce qui manque pour Factory :**

| Gap | Impact | Effort |
|-----|--------|--------|
| `node_id` obligatoire dans SbfbJson → bloque templates portables | BLOQUANT | ~5 lignes (Option D du constraint research) |
| SbfbJson ne lit que `node_id` + `version` → pas de schema_version, name, bridge, etc. | BLOQUANT pour SBFB.json v2 | ~80 lignes (nouveau struct + validation) |
| `repo_url` accepte `http` mais le feed exige `https` | INCOHERENCE | ~3 lignes (renforcer la validation) |
| deploy-from-repo ne cree PAS d'entree feed `ReleasePublished` | GAP FONCTIONNEL | ~40 lignes (appel insert_feed_operation apres le deploy) |
| Pas de validation des bridge methods declarees | GAP SECURITE | ~30 lignes (allowlist check) |
| Pas de scan secrets dans le repo clone | GAP SECURITE | ~100 lignes (regex scanner basic) |

**Verdict :** Le deploy pipeline est solide mais necessite 4-5 modifications
pour supporter Factory. L'effort total pour deploy.rs est ~260 LOC.

### 1.2 provenance.rs — generation provenance (212 lignes)

**Etat actuel :** Complet et bien teste. `generate_provenance()` prend
repo_url, commit_sha, artifact_hash, node_id_hex, keypair et produit un
ProvenanceRecord signe. `verify_provenance()` verifie la signature.
7 tests dont tamper detection et wrong-key rejection.

**Ce qui manque pour Factory :**

Rien. La provenance est generique. Elle signe avec le node_id du daemon
(pas de SBFB.json). Le champ `app_version` est deja `Option<String>`.
Factory utilise exactement le meme chemin : `deploy-from-repo` → provenance.

**Verdict :** Zero modification requise.

### 1.3 public_feed.rs — entries feed (1556 lignes)

**Etat actuel :** Complet et robuste. `PublicFeedOperation` enum avec
`ReleasePublished` et `SourceBecameStale`. Validation stricte (project_id
hex-64, repo_url HTTPS, commit_sha hex-40, artifact_hash hex-64). Hash-chain
BLAKE3 + Ed25519 per-entry. PoW 16-bit anti-spam. Rate limiting per-author.
40+ tests dont adversariaux.

**Ce qui manque pour Factory :**

| Gap | Impact | Effort |
|-----|--------|--------|
| `deploy-from-repo` ne cree pas de feed entry `ReleasePublished` | Le deploy ne laisse pas de trace dans le feed | ~40 LOC dans deploy.rs |
| Pas de `CuratorVouched` (prevu S67 Gouvernance) | Les apps Factory n'ont pas d'endorsement curator | PROBLEME DE SEQUENCAGE |

**Verdict :** Le feed est pret. Le seul gap est le cablage deploy → feed,
qui est un ajout simple. Le probleme CuratorVouched est un probleme de
sequencage, pas de code.

### 1.4 blob_serve.rs — service des apps (485 lignes)

**Etat actuel :** BlobServeCache avec LRU in-memory (DashMap, max 32 entries).
`load()` decompresse un zip, `get_file()` sert un fichier. Path traversal
rejecte. Zip bomb protection (100 MB max decompresse). CSP
`connect-src 'none'`, COOP `same-origin`, COEP `require-corp`. 16 tests.

**Ce qui manque pour Factory :**

| Gap | Impact | Effort |
|-----|--------|--------|
| blob_serve ne sert que les archives en MemStore (volatiles) | Sans S66 (FsStore), les apps Factory disparaissent au restart | S66 PREREQUIS |
| Pas de chemin pour servir un workspace local (preview) | Factory a besoin d'un preview sandbox avant publish | ~60 LOC (endpoint `/blob-serve/preview/{session}`) |

**Preview sandbox :** Le blob_serve actuel ne sert que les blobs iroh. Pour
Factory, le broker doit pouvoir zipper un workspace local, le charger dans
BlobServeCache avec un hash ephemere, et le servir dans le meme iframe
sandbox. C'est architecturalement propre : meme chemin CSP/COOP/COEP, meme
validate_zip_path, juste une source differente (local au lieu de blob iroh).

**Verdict :** Modification mineure. Le pattern existant est reutilisable.

### 1.5 Bridge complet (protocol.ts + useBridge.ts)

**Etat actuel :** 13 methodes bridge (task_submit, storage_get/set/list/delete,
pii_redact, identity_pubkey, node_status, browse_list, storage_version,
provenance_get, provenance_verify, feed_cursor_get). Chaque methode est
dispatched vers l'API coordinator avec timeout 10s. Heartbeat watchdog.
Push events host → iframe.

**Ce qui manque pour Factory :**

Rien pour le MVP Babel Reader. Les methodes existantes couvrent :
- `storage_get/set/list/delete` → progression lecture, bookmarks
- `identity_pubkey` → identite lecteur
- `provenance_get/verify` → provenance visible
- `feed_cursor_get` → position dans le feed
- `browse_list` → catalogue apps
- `node_status` → sante du noeud
- `task_submit` → traduction mock future

**Babel n'a besoin d'aucune nouvelle methode bridge.** C'est un argument
fort pour la faisabilite : Factory genere du code qui utilise des API
existantes.

**Verdict :** Zero modification requise pour le MVP.

### 1.6 Browse.tsx — apparition des apps (392 lignes)

**Etat actuel :** Netflix-style grid. Chaque `BrowseEntry` a project_id,
project_name, category, description, status, archive_hash, repo_url,
provenance_hash, is_open_source, source (direct/curator). Navigation vers
`/browse/{project_id}`.

**Ce qui manque pour Factory :**

| Gap | Impact | Effort |
|-----|--------|--------|
| Pas de champ `display_name` ni `description` enrichi | L'UI affiche project_name brut | ~20 LOC (propagation SBFB.json v2 → BrowseEntry) |
| Pas de badge "Genere par Factory" | Pas de distinction visuelle | ~10 LOC (badge conditionnel sur factory_template dans metadata) |

**Verdict :** Modifications cosmetiques. Le Browse fonctionne tel quel.

### 1.7 Apps existantes (sbfb-explorer, sbfb-ideas)

**Etat actuel :** Deux apps HTML/CSS/JS vanilla. Chaque app a :
- `index.html` (point d'entree)
- `app.js` (logique, bridge calls)
- `style.css` (dark theme)
- `SBFB.json` (`{"node_id": "PLACEHOLDER", "name": "...", "version": "..."}`)
- `sbfb-bridge.js` (SDK client bridge)

**Pattern reutilisable pour Factory :** Les apps sont le template naturel.
Factory doit generer exactement cette structure. Le contenu de `app.js`
change selon le domain pack, mais la structure reste identique :
`index.html + app.js + style.css + SBFB.json + sbfb-bridge.js`.

**Verdict :** Les apps existantes SONT les templates implicites. Factory
formalise ce qui existe deja.

---

## 2. S65-S66 comme prerequisites Factory — verification

### 2.1 P2-FEED-INSERT-NO-AUTH-TIER (S65) — est-ce un prerequis Factory ?

**OUI, BLOQUANT.**

`feed_sync.rs:445-487` accepte n'importe quel caller authentifie (bearer
token) pour inserer des operations feed, sans verification de tier. Si
Factory publie des apps qui creent des entries feed `ReleasePublished`, un
process malveillant avec le bearer token pourrait injecter de fausses
releases dans le feed.

Le fix est de ~30-50 LOC (verifier auth tier >= T1 avant insert). Il DOIT
etre fait avant tout feed insert automatique, ce qui inclut le cablage
deploy → feed que Factory necessite.

**Sans ce fix, une app Factory deployee + feed entry = une vulnerabilite
d'injection feed permanente.**

### 2.2 Persistence S66 — est-ce un prerequis Factory ?

**Phase A (iroh-docs data_dir) : NON pour Factory seule, OUI pour le pilote.**

Factory genere un repo, l'utilisateur le deploie via deploy-from-repo. Le
deploy utilise iroh-blobs pour stocker l'archive. Si le daemon restart,
l'archive disparait (MemStore). Mais Factory ne CREE pas de donnees
iroh-docs directement — elle utilise le chemin deploy standard.

**Phase B (iroh-blobs FsStore) : FORTEMENT SOUHAITABLE mais pas strictement
bloquant.**

Sans FsStore, les apps Factory deployees disparaissent au restart. Pour
un dev qui itere localement, c'est acceptable (re-deploy). Pour le pilote
S69 avec des testeurs, c'est inacceptable (apps perdues). La question est :
le pilote Babel S69 peut-il tolerer des restarts qui perdent les apps ?

**Reponse : Non.** Le pivot document dit explicitement : "utilisable 24h" et
"gate S66 : app archive + provenance + feed survivent aux restarts".

**Phase C (feed republish) : FORTEMENT SOUHAITABLE.**

Sans feed republish au boot, les entries feed des apps Factory ne sont
pas re-syncees vers les peers apres restart. C'est un probleme pour le
pilote mais pas pour Factory elle-meme (qui genere, pas qui sync).

**Verdict final sur S66 :** Factory peut DEMARRER en S67 sans S66 complet.
La generation de templates et le deploy fonctionnent en memoire. Mais le
PILOTE S69 (Babel canari) est STRICTEMENT impossible sans S66.

**Sequencage recommande :** S65 → S66 → S67 RESTE le bon ordre. Factory
ne gagne rien a se lancer avant que la persistence soit en place, car
Babel S69 est le test d'acceptance de Factory, et Babel necessite S66.

### 2.3 Factory peut-elle demarrer sans persistence blobs (S66 Phase B) ?

**Oui, en mode dev-only.** Factory genere le template, l'utilisateur deploie,
l'app apparait dans Browse tant que le daemon tourne. Au restart, re-deploy.
C'est le workflow des apps existantes (Explorer, Ideas Hub deployes
manuellement).

**Non, pour le pilote.** Le gate S69 exige "utilisable 24h" et "no P0/P1".
Un daemon qui perd ses apps au restart est un P0 evident.

**Conclusion :** Factory S67 peut livrer un `sbfb create` fonctionnel sans
S66. Mais S68 (broker, preview, publish gate) a besoin de blob-serve
stable, et S69 (Babel canari) a besoin de la persistence complete.
L'economie de sequencage est donc de 0 sprints : S66 doit quand meme
etre fait avant S69.

---

## 3. Factory S67 — contenu reel vs. code existant

### 3.1 Template engine

**Existe-t-il un pattern de scaffolding dans le code ?**

Non. Aucun code de generation de templates. Les apps `examples/sbfb-explorer/`
et `examples/sbfb-ideas/` sont des fichiers statiques crees manuellement.
Le bridge SDK `sbfb-bridge.js` est un fichier commun aux deux apps (contenu
identique), copie manuellement.

**Ce que Factory doit creer :**

Un template engine minimal n'est pas un moteur Tera/Handlebars. C'est un
copy + substitution de variables dans des fichiers. Le pattern :

```
1. Lire le template (dossier avec fichiers + template.json)
2. Pour chaque fichier : copier, substituer les variables ({{name}}, {{version}})
3. Generer SBFB.json v2 avec les valeurs
4. Copier sbfb-bridge.js
5. Init git repo
6. Ecrire factory.template.lock + factory.provenance.json
```

**Effort estime :** ~300-400 LOC Rust pour le module de generation. Pas de
dependance externe requise (String::replace, std::fs::copy, serde_json).

### 3.2 SBFB.json v2

**Impact sur deploy.rs :**

Le research `factory_deploy_constraint_research.md` documente precisement
le changement. Le struct `SbfbJson` passe de :

```rust
struct SbfbJson {
    node_id: String,
    version: Option<String>,
}
```

A un struct v2 avec ~15 champs (`schema_version`, `name`, `display_name`,
`description`, `category`, `license`, `lang`, `bridge`, `tech`,
`requirements`). Le `node_id` devient `Option<String>` et n'est plus
verifie (Option D).

**Effort :** ~80 LOC (struct + validation) + ~40 LOC (tests) + ~20 LOC
(migration apps existantes). Total ~140 LOC.

**Risque :** Faible. La compat descendante est assuree par `#[serde(default)]`
sur tous les champs. Les SBFB.json v1 existants parsent sans probleme.

### 3.3 factory.template.lock

**Format :**

```json
{
  "schema_version": 1,
  "template_id": "static-storage",
  "template_version": "0.1.0",
  "template_hash": "<BLAKE3 du dossier template>",
  "generated_at": "<ISO 8601>",
  "generator_version": "1.0.0",
  "variables": {
    "name": "babel-reader",
    "version": "0.1.0"
  }
}
```

**Effort :** ~30 LOC (struct serde + write). Trivial.

### 3.4 factory.provenance.json

**Format :**

```json
{
  "schema_version": 1,
  "generator_node_id": "<hex>",
  "template_hash": "<BLAKE3>",
  "variables_hash": "<BLAKE3 du JSON variables>",
  "output_hash": "<BLAKE3 du workspace genere>",
  "generated_at": "<ISO 8601>",
  "signature": "<Ed25519 hex>"
}
```

**Effort :** ~50 LOC (struct serde + signing + write). Le pattern est
identique a `provenance.rs` : canonical bytes + sign.

### 3.5 factory.audit.jsonl

Un fichier append-only de log structuree :

```jsonl
{"ts":"...","action":"create","template":"static-storage","variables":{...}}
{"ts":"...","action":"generate","files":["index.html","SBFB.json",...],"hash":"..."}
{"ts":"...","action":"lock","template_hash":"...","output_hash":"..."}
```

**Effort :** ~20 LOC (struct serde + append). Trivial.

### 3.6 Sprint skeleton genere

Les fichiers `.planning/active/sprint01_plan.md` etc. sont des templates
textuels avec variables. Le contenu est generique :

```markdown
# Sprint 01 — ${display_name} Initial Release

## Objectif
Premiere release deployable de ${display_name}.

## Phases
- Phase A : Structure initiale
- Phase B : Fonctionnalites de base
- Phase C : Tests et verification
- Phase D : Deploy verifie
```

**Effort :** ~50 LOC (3-4 fichiers template, ~15 LOC chacun).

### 3.7 Module broker — quel crate, quels patterns existants ?

**Crate recommande :** `nexus-shell-daemon-core` (le meme crate qui contient
blob_serve.rs, browse.rs, publish.rs). Le broker est un module de la lib
daemon, pas un crate separe.

**Patterns reutilisables :**

| Pattern existant | Localisation | Reutilisable pour Factory |
|-----------------|-------------|--------------------------|
| Path validation | `blob_serve::validate_zip_path()` | OUI — meme rigueur pour les templates |
| Zip creation | `deploy::zip_directory()` | OUI — pour le preview |
| Provenance signing | `provenance::generate_provenance()` | OUI — pour factory.provenance.json |
| BLAKE3 hashing | `crypto::blake3_hash()` | OUI — pour les hashes template/output |
| Canonical bytes | `canonical::canonical_bytes()` | OUI — pour la signature provenance Factory |
| Auth middleware | `auth::auth_required()` | OUI — pour les routes /api/v1/factory/* |
| Browse entry | `browse::BrowseAggregatorHandle` | OUI — pour publier l'app apres deploy |

**Effort :** Le module broker S67 est minimal : il orchestre les fonctions
existantes. Le broker S68 est le vrai effort (diff, preview, publish gate).

**S67 broker = ~150 LOC** (routes list-templates + create + audit log).
**S68 broker = ~500-700 LOC** (diff engine, preview serve, publish gate,
scan secrets, path traversal tests).

---

## 4. S68 (Broker, preview, publish gate) — analyse de faisabilite

### 4.1 UI /factory — effort nouvelle page React

**Pattern existant :** Les pages Browse.tsx (392 LOC), Curators.tsx (327 LOC),
Deploy.tsx (191 LOC) sont des precedents. La page /factory suit le meme
pattern : query API, afficher resultats, formulaire d'action.

**Composants necessaires :**

| Composant | Complexite | LOC estime |
|-----------|-----------|-----------|
| TemplateSelector | Faible (liste + click) | ~60 |
| VariablesForm | Moyenne (champs dynamiques) | ~100 |
| DiffViewer | Haute (fichiers expandables, ajouts/suppressions) | ~200 |
| PreviewFrame | Faible (iframe meme pattern BrowsedProject) | ~50 |
| PublishChecklist | Moyenne (statut par item) | ~80 |
| FactoryPage (layout) | Faible | ~80 |

**Total estime :** ~570 LOC React/TypeScript. C'est un sprint frontend
complet en soi.

### 4.2 Diff preview

**Existe-t-il un pattern diff dans le code ?**

Non. Aucun diff engine. Mais le diff Factory n'est pas un diff Git — c'est
une comparaison fichier-par-fichier entre "workspace actuel" et "modifications
proposees". Le format est JSON :

```json
{
  "files": [
    {"path": "index.html", "action": "modify", "before_hash": "...", "after_hash": "..."},
    {"path": "SBFB.json", "action": "create"},
    {"path": "old-file.txt", "action": "delete"}
  ]
}
```

**Effort :** ~100 LOC Rust (read workspace, compare avec template output,
produire le diff JSON). Pas de dependance externe (`diffy` ou `similar`
optionnels pour du diff inline texte, mais pas obligatoires pour le MVP).

### 4.3 Preview sandbox

**blob-serve peut-il deja servir un workspace local ?**

Non directement. `blob_serve` attend un blob hash qui correspond a une
archive dans iroh-blobs (MemStore/FsStore). Pour preview :

Option A (simple) : zipper le workspace → `BlobServeCache::load()` avec un
hash ephemere → servir via le meme `/blob-serve/{hash}/{path}`. Le hash est
un UUID ou un BLAKE3 du zip. Aucune modification de blob_serve.rs.

Option B (plus propre) : route dediee `/blob-serve/preview/{session}/{path}`
qui sert directement depuis le filesystem avec les memes CSP/COOP/COEP.
~60 LOC dans http.rs.

**Recommandation :** Option A. Zero modification de blob_serve.rs. Le
broker zippe, charge dans le cache, renvoie le hash. Le frontend pointe
l'iframe vers `/blob-serve/{hash}/index.html`.

**Effort :** ~30 LOC dans le broker (zip + cache.load).

### 4.4 Publish gate — deploy-from-repo reutilisable ?

**Oui, a 100%.** Le chemin Factory → publish est :

```
1. Factory genere le workspace local
2. L'utilisateur review (diff + preview)
3. L'utilisateur approve
4. Factory commit + push sur un repo public (ou local pour test)
5. POST /api/v1/deploy-from-repo {repo_url, commit_sha, project_name, ...}
6. deploy.rs fait son travail normal (clone, verify, zip, hash, provenance)
```

Factory ne bypasse pas deploy-from-repo. Elle prepare l'input et delegue
au meme pipeline que tout deploy normal. C'est le design correct : Factory
n'a pas de pouvoir special sur le protocole.

**Ce qui manque :** Un "publish-check" qui execute les validations de
deploy-from-repo sans effectuer le deploy. C'est un dry-run :

```rust
pub async fn deploy_from_repo_check(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<DeployFromRepoRequest>,
) -> Response {
    // Memes validations que deploy_from_repo, mais retourne OK/ERRORS
    // sans stocker, annoncer ou signer
}
```

**Effort :** ~60 LOC (factoriser les validations de deploy_from_repo en
fonctions reutilisables).

### 4.5 Scan secrets

**Existe-t-il un pattern ?**

Non. Mais un scan basique est une liste de regex :

```rust
const SECRET_PATTERNS: &[&str] = &[
    r"(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*['\"][^'\"]+['\"]",
    r"(?i)AKIA[0-9A-Z]{16}",  // AWS access key
    r"-----BEGIN (RSA |EC )?PRIVATE KEY-----",
    r"(?i)bearer\s+[a-zA-Z0-9\-._~+/]+=*",
];
```

**Effort :** ~80 LOC (regex patterns + scanner + tests). Pas de dependance
externe (regex est deja dans le workspace via des deps transitives).

---

## 5. Gouvernance — analyse des 4 options

### Option A : Gouvernance absorbee dans S65-S66

**Description :** Ajouter CuratorVouched + CuratorDisendorsed au feed dans
S65 Phase A (avec le fix auth tier et le bump FEED_FORMAT_VERSION).

**Faisabilite :**

S65 est deja un sprint de 4 phases (taxonomie + badges UI + badge dynamique +
non-regression). Ajouter CuratorVouched/CuratorDisendorsed signifie :
- 2 nouveaux variants dans `PublicFeedOperation` (~50 LOC)
- Validation pour les nouveaux types (~40 LOC)
- Tests unitaires + adversariaux (~100 LOC)
- UI multi-curator dans Browse (~200 LOC React)
- Stale detection timer (~80 LOC)

**Total scope creep :** ~470 LOC supplementaires. S65 passerait de 4 phases
a 6 phases.

**Risque :** HAUT. S65 est deja le sprint le plus important de l'arc 1
(contrat public, base de toute la confiance). Ajouter la gouvernance risque
de diluer le focus et de retarder le sprint.

**Verdict : REJETE.** Scope creep inacceptable pour le sprint fondateur.

### Option B : Gouvernance en S70 (apres pilote, avant RRV)

**Description :** Factory en S67-S69, Gouvernance en S70, RRV en S71-S72.

**Implications :**

- Le pilote S69 se fait SANS CuratorVouched dans le feed.
- Les apps Factory dans Browse n'ont que le badge "Auto-publie" et
  "Provenance". Pas d'endorsement curator.
- RRV S71 n'a pas de CuratorVouched a indexer pour les Proof Cards.
- Le bump FEED_FORMAT_VERSION v1 → v2 se fait en S70 au lieu de S67.

**Probleme central :** Le pilote S69 avec 2-3 testeurs ne beneficie pas de
la gouvernance. Les testeurs voient des apps sans endorsement. La "credibilite
publiquement defendable" est affaiblie — le proof pack S68 ne contient pas
d'endorsements.

**Risque :** MOYEN. Le pilote est ferme (2-3 personnes). L'absence de
CuratorVouched est documentable comme "pre-governance". Mais ca reduit la
valeur du pilote comme demonstration de confiance.

**Verdict : ACCEPTABLE mais sous-optimal.** Le pilote perd de la valeur
comme demonstration.

### Option C : Gouvernance en S73 (dans le hardening)

**Description :** Factory S67-S69, RRV S70-S72, Gouvernance S73, Factory
hardening S74, Babel S75.

**Implications :**

- 8 sprints sans CuratorVouched dans le feed.
- RRV S70-S72 indexe et cherche mais n'a pas d'endorsements a afficher
  dans les Proof Cards.
- Les Proof Cards S71 manquent le critere "curation.curator_count >= 1"
  (+10 points) → scores systematiquement plus bas.
- Le SearchManifest S72 ne peut pas distinguer "app endorsee" vs "app random".
- Factory hardening S74 n'a pas de workflow "curator endorse une app Factory".

**Probleme central :** RRV sans Gouvernance est un moteur de recherche qui
ne peut pas juger la confiance. Les Proof Cards sont incompletes. Le
SearchManifest est un canal de discovery sans filtre de qualite.

**Risque :** HAUT. Ca creve l'arc 2 de sa substance : "Intelligent et
Verifiable" sans "Verifiable" (pas d'endorsements).

**Verdict : REJETE.** L'arc 2 necessite CuratorVouched pour etre complet.

### Option D : CuratorVouched minimal en S65, full governance post-pilote

**Description :**
- S65 Phase A : ajouter les variants `CuratorVouched` et `CuratorDisendorsed`
  au enum `PublicFeedOperation` + validation + tests. Bumper
  FEED_FORMAT_VERSION a 2. PAS d'UI, PAS de multi-curator overlay,
  PAS de stale detection.
- S65 Phase D : ajouter un endpoint CLI/API pour emettre un CuratorVouched
  (pour le mainteneur, pas d'UI).
- S67 : Factory Foundation (comme prevu dans le pivot).
- S69 : Pilote avec CuratorVouched disponible (le mainteneur endorse
  les apps manuellement).
- S70 : Full governance UI (multi-curator overlay, dissent, freshness,
  stale detection).

**Faisabilite :**

Le cout additionnel en S65 est minimal :
- 2 variants dans l'enum (~50 LOC Rust)
- Validation (~40 LOC)
- Tests (~80 LOC)
- Endpoint CLI/API sans UI (~30 LOC)
- Bump FEED_FORMAT_VERSION (~5 LOC)

**Total : ~205 LOC supplementaires.** S65 reste a 4 phases, la Phase A
absorbe les variants (elle fait deja le fix auth tier et la taxonomie).

**Avantages :**
- Le pilote S69 a des endorsements dans le feed (meme si manuels).
- Les Proof Cards S71 ont le champ curation rempli.
- Le SearchManifest S72 beneficie des endorsements.
- Le bump FEED_FORMAT_VERSION est fait une seule fois (S65), pas deux fois
  (S67 + S70).
- La full governance UI est differee a S70 ou le feedback pilote l'informe.

**Risques :**
- S65 est legerement plus charge (~200 LOC, ~2 heures de travail).
- L'endpoint CLI CuratorVouched est "power user only" sans UI.

**Verdict : RECOMMANDE.** C'est le meilleur compromis. Le cout marginal
est faible, le benefice structurel est majeur.

---

## 6. Risques de la remontee Factory

### 6.1 Scope creep — Factory + Babel en 3 sprints, faisable solo maintainer ?

**Historique :** Les sprints S60-S64 ont ete 5 sprints de 4-6 phases chacun,
avec un rythme de ~1 sprint/semaine. Le scope S67-S69 dans le pivot est :

| Sprint | Phases estimees | LOC estime | Crates touches |
|--------|----------------|-----------|----------------|
| S67 (Factory Foundation) | 4-5 | ~800-1000 | nexus-coordinator-rs, nexus-shell-daemon, nexus-shell-daemon-core |
| S68 (Broker, preview, publish gate) | 4-5 | ~1000-1200 | nexus-shell-daemon-core, nexus-shell-daemon, web/ |
| S69 (Babel canari) | 4-5 | ~600-800 | web/ (page /factory), examples/babel-reader/, nexus-shell-daemon |

**Total : 12-15 phases, ~2400-3000 LOC sur 3 sprints.**

**Comparaison :** S64 (hardening public) a livre 5 phases, ~600 LOC, +21
tests Rust. S60 (installer + tray + LT-7 + frontend) a livre 6 phases.

**Verdict :** Le rythme est soutenu mais faisable. Chaque sprint est dans
la plage historique (4-6 phases). Le risque n'est pas la quantite mais
l'etendue : 3 sprints consecutifs avec du code nouveau (pas du refactoring
ou du hardening) demandent une discipline de scope cuts stricte.

**Mitigation :** Scope cuts clairs :
- S67 : templates statiques SEULEMENT (pas react-vite, pas pyodide-notebook)
- S68 : diff JSON SEULEMENT (pas de diff inline texte)
- S69 : Babel avec fixtures SEULEMENT (pas de task_submit traduction)

### 6.2 Le pilote S69 integre Babel canari — double le scope ?

**Analyse :** Le pivot S69 est SPECIFIQUEMENT Babel canari. Ce n'est pas
"S69 pilote + Babel". C'est "S69 = Babel EST le pilote".

Le problem est que l'ancien S69 (Pilote Ferme) dans la roadmap V2 contenait :
- Mecanisme invite (tickets feed)
- Installeur cross-platform teste
- Feedback collector integre (Ideas Hub)
- 8 scenarios de test guides
- Analyse go/no-go

Le nouveau S69 (Babel canari) contient :
- Domain pack Babel
- App generee par Factory
- Fixtures multilingues
- Source manifests
- Storage local
- Reviews minimales
- Provenance visible
- Pilote ferme (2-3 personnes)

**Le scope n'est PAS double.** Le nouveau S69 remplace l'ancien. Les
scenarios de test sont testes sur Babel au lieu d'etre testes sur "rien
en particulier". Le feedback collector (Ideas Hub) est deja deploye. Le
mecanisme invite reste necessaire.

**MAIS :** Le domain pack Babel ajoute un effort specifique (fixtures
multilingues, source manifests, storage schema). C'est ~200-300 LOC de
contenu metier en plus du code protocole.

**Verdict :** Le scope est comparable a l'ancien S69, pas double. Babel
fournit un objet concret de test au lieu de scenarii abstraits.

### 6.3 Factory sans Gouvernance — les apps n'ont pas d'endorsement

**Probleme :** Avec l'Option D (CuratorVouched minimal en S65), le
mainteneur peut emettre manuellement des endorsements pour les apps
Factory. Les testeurs du pilote voient "Reference par sbfb-dev (securite,
qualite)" dans le feed.

**Sans Option D :** Les apps Factory sont "Auto-publie" sans endorsement.
Pour un pilote ferme avec 2-3 personnes, c'est documentable. Pour une
demonstration publique, c'est un trou de credibilite.

**Verdict :** L'Option D resout ce probleme a cout marginal.

### 6.4 Factory sans RRV — les apps ne sont pas cherchables

**Probleme :** Le Browse est une liste. Sans RRV, on ne peut pas chercher
par mot-cle. Avec 3-5 apps dans le Browse, ce n'est pas un probleme.

**RRV est un probleme a 100+ apps.** Pour le pilote S69 avec 2-3 apps
(Explorer, Ideas Hub, Babel), le Browse en liste est suffisant.

**Verdict :** NON-BLOQUANT. RRV peut rester en S70+ sans impact sur
Factory/Babel.

---

## 7. Estimation effort revisee — S67-S69 version pivot

### S67 — Factory Foundation / Sprint OS

**Objectif :** Factory sait creer un repo app minimal avec process de sprint.

**Phases :**

| Phase | Contenu | LOC Rust | LOC TS/HTML | Tests |
|-------|---------|----------|-------------|-------|
| A | SBFB.json v2 struct + validation + deploy.rs update + node_id deprecation | ~140 | 0 | +6-8 |
| B | Template engine (static-minimal, static-storage) + factory.template.lock | ~300 | 0 | +8-10 |
| C | Sprint skeleton genere + factory.provenance.json + factory.audit.jsonl | ~130 | 0 | +4-6 |
| D | CLI `sbfb create` + migration apps existantes vers v2 + cablage deploy → feed | ~200 | ~20 (JSON apps) | +6-8 |

**Totaux S67 :**
- Phases : 4
- LOC Rust : ~770
- LOC autres : ~20
- Tests delta : +24-32
- Crates touches : nexus-coordinator-rs (SBFB.json v2), nexus-shell-daemon (deploy.rs, CLI), nexus-shell-daemon-core (factory module)
- Risque technique : 2/5 (pas de nouvelle dependance, reutilise des patterns existants)

**Ce qui est NOUVEAU (aucun code existant):**
- Template engine (~300 LOC)
- factory.template.lock / factory.provenance.json / factory.audit.jsonl (~200 LOC)
- CLI sbfb create (~100 LOC)

**Ce qui est du REFACTORING (code existant modifie):**
- SbfbJson struct v2 (~80 LOC)
- deploy.rs node_id deprecation (~20 LOC)
- deploy.rs → feed cablage (~40 LOC)
- Apps existantes migration (~20 LOC)

### S68 — Broker, preview, publish gate

**Objectif :** Factory previent une publication fragile.

**Phases :**

| Phase | Contenu | LOC Rust | LOC TS | Tests |
|-------|---------|----------|--------|-------|
| A | Broker module + routes API (/factory/templates, /factory/create, /factory/diff, /factory/apply, /factory/preview) + audit JSONL | ~200 | 0 | +6-8 |
| B | Diff engine JSON + review API + scan secrets regex | ~200 | 0 | +8-10 |
| C | Page React /factory (TemplateSelector, VariablesForm, DiffViewer, PreviewFrame, PublishChecklist) | ~50 | ~570 | +8-12 (Vitest) |
| D | Preview sandbox (zip workspace → blob-serve cache) + publish gate checklist + deploy-from-repo dry-run | ~160 | ~80 | +6-8 |

**Totaux S68 :**
- Phases : 4
- LOC Rust : ~610
- LOC TS : ~650
- Tests delta : +28-38
- Crates touches : nexus-shell-daemon-core (factory_broker), nexus-shell-daemon (routes), web/ (page React)
- Risque technique : 3/5 (DiffViewer est le composant le plus complexe, preview sandbox est nouveau)

### S69 — Babel Reader canari ferme

**Objectif :** Babel est la premiere app reelle produite par Factory.

**Phases :**

| Phase | Contenu | LOC Rust | LOC TS/HTML | Tests |
|-------|---------|----------|-------------|-------|
| A | Domain pack Babel (fixtures, languages.json, storage_schema.json, source_manifests) | ~50 | ~200 (fixtures JSON) | +3-4 |
| B | App babel-reader generee par Factory (UI reader, liste textes, toggle langue, progression) | ~20 | ~500 (HTML/CSS/JS) | +4-6 |
| C | Mecanisme invite pilote + fix P2-VERIFY-LOCAL-KEY-ONLY + guide testeur | ~80 | ~60 | +4-6 |
| D | Deploy verifie Babel + feed ReleasePublished + E2E verification provenance | ~40 | ~30 | +6-8 |
| E | Bilan pilote + go/no-go (documentation, pas de code) | 0 | ~200 (docs) | 0 |

**Totaux S69 :**
- Phases : 5
- LOC Rust : ~190
- LOC TS/HTML : ~990 (dont ~500 app Babel, ~200 fixtures, ~200 docs)
- Tests delta : +17-24
- Crates touches : nexus-shell-daemon (invite), nexus-shell-daemon-core (domain pack), examples/babel-reader/
- Risque technique : 4/5 (premiere exposition a des testeurs externes, bugs imprevisibles)

### Synthese S67-S69

| Sprint | Phases | LOC total | Tests delta | Risque |
|--------|--------|-----------|-------------|--------|
| S67 | 4 | ~790 | +24-32 | 2/5 |
| S68 | 4 | ~1260 | +28-38 | 3/5 |
| S69 | 5 | ~1180 | +17-24 | 4/5 |
| **Total** | **13** | **~3230** | **+69-94** | -- |

**Comparaison avec l'ancien S67-S69 (Gouvernance + Proof Pack + Pilote) :**

| Ancien sprint | Phases estimees | LOC estime |
|---------------|----------------|-----------|
| S67 Gouvernance | 4 | ~800 |
| S68 Proof Pack | 4 | ~600 |
| S69 Pilote | 5 | ~500 |
| **Total ancien** | **13** | **~1900** |

**Le pivot ajoute ~1330 LOC sur 3 sprints** (3230 vs 1900). C'est ~70% de
code en plus. L'essentiel du surplut vient de l'UI React /factory (~570 LOC)
et de l'app Babel (~500 LOC).

---

## 8. Impact sur la roadmap complete — ou va quoi ?

### Roadmap ancien (V2 actuelle)

```
S65 Contrat Public → S66 Durabilite → S67 Gouvernance → S68 Proof Pack →
S69 Pilote → S70 RRV Local → S71 Proof Cards → S72 SearchManifest →
S73 Templates → S74 Broker → S75 Babel
```

### Roadmap pivot (Factory First)

```
S65 Contrat Public (+ CuratorVouched minimal) → S66 Durabilite →
S67 Factory Foundation → S68 Broker/Preview → S69 Babel Canari →
S70 Gouvernance Full UI → S71 RRV Local → S72 Proof Cards →
S73 SearchManifest → S74 Factory Hardening → S75 Pack Produit
```

### Ce qui est deplace

| Element | Ancien sprint | Nouveau sprint | Impact |
|---------|--------------|---------------|--------|
| CuratorVouched (wire format) | S67 | S65 (minimal) | Avance de 2 sprints |
| CuratorVouched (full UI) | S67 | S70 | Retarde de 3 sprints |
| Proof Pack | S68 | Absorbe dans S75 | Retarde de 7 sprints |
| Pilote mecanisme | S69 | S69 (fusionne avec Babel) | Meme sprint |
| Templates | S73 | S67 | Avance de 6 sprints |
| Broker | S74 | S68 | Avance de 6 sprints |
| Babel | S75 | S69 | Avance de 6 sprints |
| RRV Local | S70 | S71 | Retarde de 1 sprint |
| Proof Cards | S71 | S72 | Retarde de 1 sprint |
| SearchManifest | S72 | S73 | Retarde de 1 sprint |

### Ce qui disparait ou est retarde significativement

**Proof Pack (S68 ancien) :** Le proof pack est un livrable important pour
les bailleurs/evaluateurs. Le retarder de 7 sprints (S68 → S75) signifie
que le pilote S69 n'a PAS de proof pack verifiable. Les testeurs ne
peuvent pas examiner un bundle de preuves hors connexion.

**Recommandation :** Integrer un proof pack minimal dans S69 (Phase D : un
dossier avec provenance + feed snapshot + SBOM). Le proof pack complet
(CLI, verify.sh, cosign) reste en S75.

**Gouvernance Full UI (S67 → S70) :** Retardee de 3 sprints. Acceptable
si CuratorVouched minimal est en S65 (Option D).

---

## 9. Analyse de risque synthetique

### Risques specifiques a la remontee Factory

| # | Risque | Prob. | Impact | Mitigation |
|---|--------|-------|--------|------------|
| F1 | Template engine plus complexe que prevu (edge cases fichiers, encoding) | MOYEN | MOYEN | Scope cut : 2 templates seulement (static-minimal, static-storage), pas de react-vite S67 |
| F2 | DiffViewer React est un composant UI complexe | MOYEN | FAIBLE | Composant JSON expandable, pas de diff inline texte |
| F3 | Preview sandbox expose une surface d'attaque locale | FAIBLE | HAUT | Memes CSP/COOP/COEP que blob-serve normal. Hash ephemere, TTL court. |
| F4 | Babel fixtures multilingues necessitent un travail de curation | HAUT | FAIBLE | 3 textes domaine public seulement. Gutenberg Project comme source. |
| F5 | Le pilote S69 sans Proof Pack complet est moins convaincant | MOYEN | MOYEN | Proof pack minimal (dossier, pas CLI) inclus dans S69 Phase D |
| F6 | La gouvernance full UI retardee a S70 cree un trou de confiance | MOYEN | HAUT | CuratorVouched minimal en S65 (Option D) comble le trou |
| F7 | 3 sprints de code nouveau consecutif sans sprint de hardening | MOYEN | MOYEN | Phase D de chaque sprint inclut des tests adversariaux |
| F8 | L'app Babel generee par Factory est plus simple qu'une app codee a la main | FAIBLE | FAIBLE | C'est le but : prouver que Factory genere du code deployable |

### Matrice de decision

| Critere | Ancien S67-S69 | Pivot S67-S69 | Delta |
|---------|----------------|---------------|-------|
| LOC total | ~1900 | ~3230 | +70% |
| Composants UI nouveaux | 2 (gouvernance, proof cards) | 3 (factory page, diff viewer, babel app) | +50% |
| Valeur produit demonstrable | Gouvernance visible, proof pack offline | App reelle generee et deployee | +++ |
| Risque technique | 3/5 (feed v2, multi-curator) | 3/5 (template engine, broker) | = |
| Risque pilote | 3/5 (pas d'app reelle a tester) | 4/5 (app reelle mais plus de code) | +1 |
| Credibilite externe | Haute (proof pack, gouvernance) | Haute (app reelle, Factory fonctionnelle) | = |
| Dependance Gouvernance | Livree | Differee (S70) | Trou si pas Option D |
| Dependance Proof Pack | Livre | Differe (S75) | Trou au pilote |

---

## 10. Recommandation finale

### Sequencage recommande

```
S65 — Contrat Public + CuratorVouched wire format (Option D)
  Phase A : Fix auth tier + CuratorVouched/Disendorsed variants + feed v2 bump
  Phase B : Taxonomie TRUST_TAXONOMY.md + badges UI migration
  Phase C : Badge dynamique post-verification
  Phase D : Non-regression wording + dette pair + endpoint CLI CuratorVouched

S66 — Durabilite (INCHANGE)
  Phase A : iroh data_dir + iroh-docs persistence
  Phase B : iroh-blobs FsStore
  Phase C : Feed republish + feed_join handle
  Phase D : RevocationCache persistence
  Phase E : E2E restart

S67 — Factory Foundation
  Phase A : SBFB.json v2 struct + validation + node_id deprecation
  Phase B : Template engine (static-minimal, static-storage)
  Phase C : factory.template.lock + factory.provenance.json + audit.jsonl
  Phase D : CLI sbfb create + deploy → feed cablage + migration apps v2

S68 — Broker, Preview, Publish Gate
  Phase A : Broker module + routes API + path validation
  Phase B : Diff engine JSON + scan secrets
  Phase C : Page React /factory (composants UI)
  Phase D : Preview sandbox + publish gate + deploy dry-run

S69 — Babel Reader Canari
  Phase A : Domain pack Babel + fixtures
  Phase B : App babel-reader generee + UI
  Phase C : Mecanisme invite + fix cross-node verification
  Phase D : Deploy verifie + feed entry + proof pack minimal
  Phase E : Bilan pilote + go/no-go

S70 — Gouvernance Full UI (anciennement S67)
  Phase A : Multi-curator trust overlay + scope
  Phase B : UX confiance visible (timeline, dissent, freshness)
  Phase C : Stale detection timer
  Phase D : Tests adversariaux gouvernance

S71-S75 — RRV + Hardening + Pack Produit (decales de 1)
```

### Points cles

1. **L'Option D (CuratorVouched minimal en S65) est NON-NEGOCIABLE** si
   Factory monte en S67. Sans elle, le pilote S69 n'a pas d'endorsements,
   les Proof Cards S72 sont incompletes, et le feed v2 bump est fait en S70
   au lieu de S65 (double bump si SearchManifest l'exige aussi).

2. **S66 reste OBLIGATOIRE avant S69.** Factory S67-S68 peut techniquement
   fonctionner sans S66 (mode dev), mais Babel S69 est impossible sans
   persistence (gate "utilisable 24h").

3. **Le proof pack minimal en S69 Phase D** est necessaire pour compenser
   le report du Proof Pack complet a S75.

4. **Le scope total sur 3 sprints est ~70% superieur** a l'ancien plan.
   C'est faisable solo maintainer avec des scope cuts stricts, mais il
   ne faut pas sous-estimer l'effort.

5. **La valeur produit est superieure** : une app reelle deployee via
   Factory est une meilleure demonstration que des specs de gouvernance
   et un proof pack sans objet concret a prouver.

---

## 11. Checklist prerequisites avant S67

Avant de demarrer S67, les items suivants doivent etre resolus :

- [ ] P2-FEED-INSERT-NO-AUTH-TIER fixe (S65 Phase A)
- [ ] CuratorVouched / CuratorDisendorsed variants dans le feed (S65 Phase A)
- [ ] FEED_FORMAT_VERSION bumpe a 2 (S65 Phase A)
- [ ] TRUST_TAXONOMY.md ecrit et applique dans l'UI (S65 Phase B)
- [ ] Badge "Verifie" remplace par "Provenance" dans Browse (S65 Phase B)
- [ ] iroh data_dir cable dans le daemon (S66 Phase A)
- [ ] iroh-blobs FsStore operationnel (S66 Phase B)
- [ ] Feed republish au boot (S66 Phase C)
- [ ] E2E restart test vert (S66 Phase E)

Si un de ces items echoue, S67 NE DOIT PAS demarrer. Le daemon doit etre
durable et le contrat public exact avant d'y construire Factory dessus.

---

## 12. Sources

### Sources code (lues exhaustivement)

- `crates/nexus-shell-daemon/src/deploy.rs` (753 lignes)
- `crates/nexus-coordinator-rs/src/provenance.rs` (212 lignes)
- `crates/nexus-coordinator-rs/src/public_feed.rs` (1556 lignes)
- `crates/nexus-shell-daemon-core/src/blob_serve.rs` (485 lignes)
- `crates/nexus-shell-daemon/src/feed_sync.rs` (120+ lignes lues)
- `crates/nexus-shell-daemon/src/http.rs` (80+ lignes lues)
- `web/src/bridge/protocol.ts` (176 lignes)
- `web/src/bridge/useBridge.ts` (366 lignes)
- `web/src/pages/Browse.tsx` (392 lignes)
- `examples/sbfb-explorer/` (5 fichiers)
- `examples/sbfb-ideas/` (5 fichiers)
- `Cargo.toml` workspace (60 lignes)

### Sources research (croisees)

- `.planning/research/s65_s75_factory_babel_canary_research.md` (850 lignes)
- `.planning/research/factory_deploy_constraint_research.md` (926 lignes)
- `.planning/research/s65_contrat_public_research.md` (584 lignes)
- `.planning/research/s66_durabilite_research.md` (417 lignes)
- `.planning/research/s65_s75_cross_cutting_research.md` (100+ lignes)
- `.planning/roadmap_v2_public_trust_rrv_factory.md` (1667 lignes)

---

*Audit de faisabilite Factory First : 2026-05-18*
