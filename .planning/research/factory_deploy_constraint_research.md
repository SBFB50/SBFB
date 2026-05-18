# Recherche : contrainte deploy.rs / SBFB.json pour la Code Factory

**Date :** 2026-05-18
**Mode :** Feasibility + Architecture
**Confiance globale :** HAUTE (analyse directe du code source, aucune
source externe necessaire)

---

## 1. Etat actuel exhaustif du deploy verifie

### 1.1 Le fichier deploy.rs (753 lignes)

Localisation : `crates/nexus-shell-daemon/src/deploy.rs`

Deux endpoints :
- `POST /api/v1/deploy` -- deploy prive (upload zip brut, pas de
  provenance, pas de verification SBFB.json)
- `POST /api/v1/deploy-from-repo` -- deploy verifie public (clone +
  SBFB.json + provenance SLSA L1)

### 1.2 Flow complet de deploy-from-repo (18 etapes)

```
 1. Recevoir JSON { repo_url, commit_sha?, project_name, category, description, apps }
 2. normalize_clone_url() -- strip .git, trailing /, fragments
 3. Valider que repo_url commence par "http"
 4. Valider commit_sha si fourni (40 hex chars)
 5. HEAD request → verifier que le repo est public (is_repo_public)
 6. tempdir + git clone --depth 1 --single-branch
 7. Si commit_sha: git fetch --depth 1 origin <sha> + git checkout FETCH_HEAD
 8. Verifier taille clone < 500 MB
 9. **read_sbfb_json()** → parser SBFB.json                          ← ICI
10. **sbfb.node_id != state.node_id** → REJECT si mismatch           ← ICI
11. Verifier index.html existe a la racine
12. git rev-parse HEAD (si commit_sha pas fourni)
13. zip_directory() → creer le zip (exclut .git/, symlinks, path traversal)
14. BLAKE3 hash du zip → artifact_hash                                ← ZIP CONTIENT SBFB.json
15. generate_provenance(repo_url, commit_sha, artifact_hash, node_id, keypair)
16. Contributor attestation (best-effort, Couche 2 Sybil)
17. add_to_zip(provenance.json) → injecter provenance dans le zip
18. blob store → gossip announce → browse entry
```

### 1.3 Le struct SbfbJson

```rust
#[derive(Debug, Deserialize)]
struct SbfbJson {
    node_id: String,
    #[serde(default)]
    version: Option<String>,
}
```

**Observations critiques :**
- Le struct ne lit que `node_id` et `version`. Le champ `name` des
  SBFB.json existants est **ignore** par serde (pas dans le struct).
- `version` est `Option<String>` avec `#[serde(default)]` -- absent = None.
- `node_id` est un `String` **obligatoire** -- absent = erreur parse.
- Aucun `schema_version` n'est lu.

### 1.4 La verification node_id (lignes 119-128)

```rust
if sbfb.node_id != state.node_id {
    return error_response(
        StatusCode::BAD_REQUEST,
        &format!(
            "SBFB.json node_id ({}...) does not match daemon node_id ({}...)",
            &sbfb.node_id[..16.min(sbfb.node_id.len())],
            &state.node_id[..16.min(state.node_id.len())],
        ),
    );
}
```

C'est une **egalite stricte**. Le node_id dans SBFB.json DOIT etre
identique au `state.node_id` du daemon (64 char hex, cle publique
Ed25519 du noeud iroh). Aucune exception, aucun placeholder, aucun
wildcard.

### 1.5 Impact sur le hash de l'artefact

**Fait critique :** Le zip est cree a l'etape 13, le BLAKE3 hash est
calcule a l'etape 14. Le SBFB.json est DANS le repo clone, donc DANS
le zip, donc **le node_id fait partie du artifact_hash**. Si le
node_id change, le artifact_hash change.

La provenance est injectee APRES le calcul du hash (etape 17, via
`add_to_zip`). Donc le provenance.json n'est PAS dans le hash de
l'artefact -- c'est voulu (la provenance est ajoutee par le builder,
pas par l'auteur).

**Consequence pour la verification tiers :** Un tiers qui veut verifier
que "le code du repo = l'archive" va :
1. Cloner le repo au meme commit
2. Reconstruire le zip
3. Calculer le BLAKE3
4. Comparer avec l'artifact_hash de la provenance

Si le SBFB.json dans le repo contient un node_id specifique, le tiers
reproduit exactement le meme hash. Si le node_id est different (ex:
le tiers a un autre daemon), le hash sera DIFFERENT, et la verification
echouera.

**C'est la propriete de securite fondamentale du deploy verifie :**
l'artefact est lie a un noeud specifique via SBFB.json.

---

## 2. Etat actuel de SBFB.json

### 2.1 Les 3 exemples

**sbfb-explorer/SBFB.json :**
```json
{
  "node_id": "PLACEHOLDER",
  "name": "sbfb-explorer",
  "version": "1.0.0"
}
```

**sbfb-ideas/SBFB.json :**
```json
{
  "node_id": "PLACEHOLDER",
  "name": "sbfb-ideas",
  "version": "0.1.0"
}
```

**hello-world-app/ :** Pas de SBFB.json (legacy Python, pre-pivot).

### 2.2 Etat de deploiement actuel des exemples

Les deux exemples utilisent `"PLACEHOLDER"` comme node_id. Cela signifie
qu'en l'etat, **aucun des deux ne peut etre deploye via deploy-from-repo**
car `"PLACEHOLDER" != state.node_id`. Pour deployer, le developpeur
doit :
1. Editer SBFB.json et remplacer "PLACEHOLDER" par son vrai node_id
2. Committer le changement
3. Pusher sur un repo public
4. Appeler `POST /api/v1/deploy-from-repo`

C'est un workflow manuel et fragile. Le "PLACEHOLDER" est un hack
de developpement, pas un mecanisme supporte.

### 2.3 Champs lus vs champs presents

| Champ | Present dans JSON | Lu par Rust | Utilise par | Obligatoire |
|---|---|---|---|---|
| `node_id` | OUI | OUI (`SbfbJson.node_id`) | Verification deploy L119 | OUI |
| `name` | OUI | NON (ignore par serde) | Rien | NON |
| `version` | OUI | OUI (`SbfbJson.version`) | `prov.app_version` L166 | NON (Option) |

Le champ `name` est present dans les JSON mais **jamais lu**. C'est un
champ decoratif.

---

## 3. Le probleme Factory — analyse precise

### 3.1 Le cycle normal (dev individuel)

```
1. Le dev cree l'app (code + SBFB.json avec SON node_id)
2. Le dev commit + push sur repo public
3. Le dev appelle deploy-from-repo avec l'URL du repo
4. Le daemon clone, verifie node_id == daemon, signe provenance
5. Le dev est l'auteur ET le deployeur
```

Le node_id dans SBFB.json sert de **declaration d'intention** : "cette
app est destinee a etre deployee par le noeud X". C'est un mecanisme
de securite qui empeche un attaquant de cloner un repo et de deployer
une app au nom de quelqu'un d'autre.

### 3.2 Le cycle Factory

```
1. L'utilisateur demande a Factory de creer une app depuis un template
2. Factory genere le code + SBFB.json
3. ... QUEL node_id dans SBFB.json ?
4. L'utilisateur veut deployer le resultat
```

**3 sous-scenarios Factory :**

**A. Factory locale (daemon genere pour lui-meme) :**
L'utilisateur utilise son propre daemon pour generer l'app. Factory
connait `state.node_id`. Elle peut ecrire le bon node_id dans SBFB.json.
Pas de probleme.

**B. Factory comme template Git :**
L'utilisateur clone un template depuis un repo Git. Le template ne peut
pas contenir le node_id de l'utilisateur (il n'est pas connu au moment
ou le template est cree). L'utilisateur doit modifier SBFB.json
manuellement apres clonage.

**C. Factory comme service reseau :**
Un noeud Factory genere une app pour un autre noeud. Factory ne connait
pas le node_id du destinataire. Meme probleme que B.

### 3.3 Le vrai probleme : reproducibilite + attribution

Le node_id dans SBFB.json sert deux roles contradictoires :
1. **Attribution** : identifier qui a le droit de deployer cette app
2. **Reproducibilite** : le hash de l'artefact depend du node_id

Pour un deploy verifie, un tiers doit pouvoir reproduire le meme hash.
Si le node_id est specifique au deployeur, alors le meme code source
deploye par deux personnes differentes produit deux hashes differents.
C'est **voulu** dans le modele actuel (chaque deployeur signe sa propre
version), mais c'est **incompatible** avec un template partage.

---

## 4. Analyse des 5 options

### Option A : node_id placeholder remplace par Factory

**Mecanisme :** Le template contient `"node_id": "PLACEHOLDER"` (ou
`"$FACTORY"` ou similaire). Factory remplace le placeholder par le vrai
node_id au moment de la generation. Le SBFB.json committe dans le repo
contient le vrai node_id. deploy.rs ne change pas.

**Avantages :**
- Zero changement dans deploy.rs
- Le deploy verifie fonctionne tel quel
- Le repo Git contient le vrai node_id (verification tiers intacte)

**Inconvenients :**
- Le template dans le repo Git contient "PLACEHOLDER" -- il ne peut
  PAS etre deploye tel quel. C'est deja le cas des exemples existants.
- Factory doit faire une substitution string dans un fichier JSON, ce
  qui est fragile si le JSON est complexe.
- Si l'utilisateur fork le repo du template sans modifier node_id,
  le deploy echoue. Pas de message d'erreur clair.
- Le "PLACEHOLDER" n'est pas un mecanisme officiel du protocole. C'est
  un hack de convention.

**Impact provenance :**
- Aucun impact si le remplacement se fait AVANT le commit dans le repo.
- Le repo contient le vrai node_id, donc le hash est deterministe.

**Impact apps existantes :**
- Aucun. Les apps existantes ont deja "PLACEHOLDER" et elles ne sont
  de toute facon pas deployables en l'etat.

**Verdict : Fonctionnel mais fragile. C'est l'etat actuel de facto.**

### Option B : node_id optionnel dans SBFB.json

**Mecanisme :** Le struct `SbfbJson` rend `node_id` optionnel :

```rust
#[derive(Debug, Deserialize)]
struct SbfbJson {
    #[serde(default)]
    node_id: Option<String>,
    #[serde(default)]
    version: Option<String>,
}
```

deploy.rs remplit le node_id au moment du deploy :
- Si `node_id` est `Some(id)` et `id != state.node_id` → reject
- Si `node_id` est `Some(id)` et `id == state.node_id` → OK
- Si `node_id` est `None` → utiliser `state.node_id`

**Avantages :**
- Les templates peuvent omettre node_id (pas de placeholder)
- Les apps existantes avec node_id continuent de fonctionner
- Le champ devient une **declaration optionnelle**, pas une obligation

**Inconvenients :**
- **CRITIQUE :** Si node_id est absent du SBFB.json dans le repo, le
  zip ne contient pas de node_id. Le artifact_hash est identique quel
  que soit le deployeur. Deux deployeurs differents produisent le
  **meme hash** pour le meme code. C'est une **perte de securite** :
  la provenance signe toujours avec le node_id du deployeur, mais le
  hash ne le contient plus.
- **CRITIQUE :** Un tiers qui reconstruit le zip pour verification ne
  sait pas quel node_id le deployeur avait. La propriete "code du
  repo = archive" reste valide, mais l'attribution est perdue dans
  l'artefact.

**Impact provenance :**
- La provenance contient toujours le node_id du signataire (c'est
  dans la signature). Mais l'artefact lui-meme ne contient plus cette
  attribution si SBFB.json n'a pas de node_id.
- Verification tiers : le tiers peut toujours verifier que le code du
  repo produit le meme zip. L'identite du deployeur n'est plus dans le
  zip mais dans la provenance signee. C'est **suffisant** car la
  provenance est elle-meme dans le zip final (injectee apres le hash).

**Impact apps existantes :**
- Aucun breaking change. `"node_id": "PLACEHOLDER"` → `Some("PLACEHOLDER")`,
  qui serait rejete au deploy (car != state.node_id). Le comportement
  est identique a aujourd'hui.
- L'utilisateur doit soit mettre son vrai node_id, soit retirer le champ.

**Verdict : Propre mais change la semantique du hash. Acceptable car
la provenance porte l'attribution.**

### Option C : node_id separe de SBFB.json

**Mecanisme :** SBFB.json ne contient que les metadonnees app (name,
version, bridge methods). Le node_id est dans un fichier separe
(`DEPLOY.json`, `.sbfb/identity.json`, ou `.sbfb-deploy`).

**Avantages :**
- Separation nette : SBFB.json = manifeste app, DEPLOY.json = identite
  deployeur
- SBFB.json est portable entre deployeurs
- Le template ne contient que SBFB.json (pas de node_id)

**Inconvenients :**
- **Deux fichiers a gerer** au lieu d'un. Confusion pour les devs.
- `.sbfb/identity.json` ou `DEPLOY.json` doit etre dans le .gitignore
  (car il contient une info specifique au deployer). Mais alors il
  n'est PAS dans le repo, et le deploy-from-repo qui clone le repo ne
  le trouve pas. Chicken-and-egg.
- Si les deux fichiers sont dans le repo, c'est pire que l'etat actuel
  (deux fichiers a editer au lieu d'un).
- deploy.rs doit lire deux fichiers au lieu d'un.
- La complexite ajoutee ne resout pas mieux que Option B.

**Impact provenance :**
- Meme impact que Option B : le node_id n'est pas dans le hash de
  l'artefact (a moins que DEPLOY.json soit aussi dans le zip).

**Impact apps existantes :**
- Breaking si SBFB.json ne contient plus node_id et que deploy.rs
  l'attend toujours. Migration necessaire.

**Verdict : Sur-ingenierie. Ajoute de la complexite sans benefice clair
par rapport a Option B.**

### Option D : node_id dans le daemon, pas dans le manifeste

**Mecanisme :** SBFB.json ne contient jamais de node_id. deploy.rs
lit le node_id exclusivement depuis `state.node_id`. Le manifeste
declare l'app, pas l'identite du deployeur.

**Avantages :**
- SBFB.json est 100% portable
- Pas de placeholder, pas de substitution, pas de confusion
- Les templates fonctionnent tels quels
- Semantiquement correct : le node_id est une propriete du **deployeur**,
  pas de l'**app**

**Inconvenients :**
- **Perte de la verification d'attribution dans le repo :** Aujourd'hui,
  le node_id dans le repo public prouve "cette app est destinee a ce
  noeud". Sans node_id, n'importe qui peut cloner un repo et deployer
  l'app "au nom" du code source. Mais la provenance signe toujours
  avec SON node_id, donc il n'y a pas d'usurpation -- juste un
  deploiement independant du meme code (ce qui est le modele open source
  normal).
- **Le hash de l'artefact est identique pour tous les deployeurs** du
  meme commit. C'est le meme impact que Option B.
- Migration des apps existantes : il faut retirer node_id de SBFB.json
  ou le rendre ignore.

**Impact provenance :**
- Identique a Option B. L'attribution est dans la provenance signee,
  pas dans l'artefact.

**Impact apps existantes :**
- Les SBFB.json existants contiennent "PLACEHOLDER". Si deploy.rs
  ignore le champ, pas de breaking change. Si le struct ne le parse
  plus, les JSON existants ne cassent pas (serde ignore les champs
  inconnus par defaut... SAUF si on utilise `#[serde(deny_unknown_fields)]`
  qui n'est pas le cas ici).

**Verdict : Semantiquement le plus correct. Le node_id est une propriete
du deployeur, pas de l'app.**

### Option E : node_id comme attribut du deploy, pas restriction

**Mecanisme :** deploy.rs ECRIT le node_id dans SBFB.json au moment
du deploy, quel que soit le contenu initial. Le node_id dans le repo
est ignore ou ecrase.

**Avantages :**
- Les templates fonctionnent (node_id ecrase au deploy)
- Le node_id final dans l'archive correspond toujours au deployeur
- Pas de breaking change (le champ existe toujours)

**Inconvenients :**
- **CRITIQUE :** Le code dans le repo != le code dans l'archive.
  deploy.rs modifie SBFB.json avant de zipper. Un tiers qui clone le
  repo au meme commit obtiendra un SBFB.json different (celui du repo
  vs celui modifie par le deployeur). Le hash sera DIFFERENT. **La
  propriete fondamentale du deploy verifie ("code du repo = archive")
  est cassee.**
- Pour restaurer la verification, il faudrait que le verificateur
  sache qu'il faut ignorer le champ node_id dans SBFB.json lors de la
  comparaison. C'est une complexite enorme.

**Impact provenance :**
- Invalide la verification bit-a-bit. La provenance dit "cet artifact
  hash vient de ce commit", mais le hash ne correspond plus au contenu
  exact du repo.

**Verdict : INACCEPTABLE. Casse la propriete fondamentale du deploy
verifie.**

---

## 5. Matrice de comparaison

| Critere | A (placeholder) | B (optionnel) | C (fichier separe) | D (daemon only) | E (ecrasement) |
|---|---|---|---|---|---|
| Factory locale | OK (substitue) | OK (omis) | OK (fichier separe) | OK (pas de node_id) | OK (ecrase) |
| Factory template Git | FRAGILE (manual edit) | OK (pas de node_id) | OK (un seul fichier) | OK (pas de node_id) | OK (ecrase) |
| Deploy verifie intact | OUI | OUI* | OUI* | OUI* | **NON** |
| Verification tiers | OUI (meme hash) | OUI (meme hash, node_id dans prov) | OUI | OUI | **NON** |
| Attribution dans artefact | OUI (dans SBFB.json) | NON (dans provenance) | NON | NON | OUI (modifie) |
| Migration apps existantes | Aucune | Aucune | Breaking | Aucune** | Aucune |
| Complexite deploy.rs | 0 lignes | ~10 lignes | ~30 lignes | ~5 lignes (supprimer check) | ~20 lignes |
| Semantique correcte | NON (hack) | OUI | MOYENNE | OUI | NON |

*La verification tiers reste valide car le hash est reproductible. L'attribution est dans la provenance signee.

**Si `#[serde(default)]` est ajoute sur node_id, les JSON existants avec
"PLACEHOLDER" sont parses sans erreur (le champ est lu mais non
verifie).

---

## 6. RECOMMANDATION : Option D (node_id dans le daemon, pas dans le manifeste)

### 6.1 Pourquoi Option D

**Argument central :** Le node_id n'est pas une propriete de l'app, c'est
une propriete du deploiement. Un meme code source peut etre deploye par
N noeuds differents. C'est le modele open source standard : le code est
public, n'importe qui peut le builder et le deployer. La provenance
signee par le deployeur porte l'attribution.

L'analogie : un paquet Debian ne contient pas l'identite du miroir qui
le distribue. Le miroir signe le Release file. L'identite du distributeur
est dans la signature, pas dans le contenu.

**Simplification radicale :**
- SBFB.json devient un manifeste pur (nom, version, permissions, tech)
- Pas de placeholder, pas de hack, pas de substitution
- Les templates sont utilisables tels quels
- La Factory n'a pas de logique speciale pour le node_id

### 6.2 Changements necessaires dans deploy.rs

```rust
// AVANT (actuel)
#[derive(Debug, Deserialize)]
struct SbfbJson {
    node_id: String,
    #[serde(default)]
    version: Option<String>,
}
// ... plus tard ...
if sbfb.node_id != state.node_id {
    return error_response(...);
}

// APRES (Option D)
#[derive(Debug, Deserialize)]
struct SbfbJson {
    // node_id lu si present mais NON verifie (backward compat)
    #[serde(default)]
    node_id: Option<String>,
    #[serde(default)]
    version: Option<String>,
}
// ... la verification node_id est SUPPRIMEE ...
// Le node_id du deployeur est dans state.node_id et dans la provenance
```

**5 lignes de changement effectif :**
1. `node_id: String` → `node_id: Option<String>` avec `#[serde(default)]`
2. Supprimer le bloc `if sbfb.node_id != state.node_id` (lignes 119-128)
3. Ajouter un warning log si node_id est present et != state.node_id
   (informatif, pas bloquant)

### 6.3 Impact sur la provenance

**Aucun changement dans provenance.rs.** La provenance est generee avec
`state.node_id` (le node_id du daemon), pas avec `sbfb.node_id`. Le
champ `node_id` dans ProvenanceRecord vient du daemon, pas du manifeste.

Extrait (deploy.rs L159-164) :
```rust
let mut prov = provenance::generate_provenance(
    &repo_url,
    &commit_sha,
    &artifact_hash_hex,
    &state.node_id,      // ← daemon node_id, PAS sbfb.node_id
    &state.pow_keypair,
);
```

La provenance **signe** avec la cle du daemon. L'attribution est
cryptographiquement liee au deployeur, independamment du contenu de
SBFB.json.

### 6.4 Impact sur la verification tiers

Aujourd'hui, un verificateur :
1. Clone le repo au commit exact
2. Reconstruit le zip
3. BLAKE3 hash → compare avec artifact_hash dans provenance

Avec Option D, rien ne change. Le SBFB.json dans le repo n'a plus de
node_id (ou en a un qui est ignore). Le verificateur reconstruit le
meme zip, obtient le meme hash. La verification passe.

**Mieux encore :** Aujourd'hui, si le SBFB.json dans le repo contient
"PLACEHOLDER", un verificateur qui reconstruit le zip obtiendrait un
hash DIFFERENT de celui du deployeur (qui a remplace PLACEHOLDER par
son vrai node_id avant le deploy). Avec Option D, SBFB.json est
identique dans le repo et dans l'archive. La verification tiers est
**amelioree**.

### 6.5 Impact sur les apps existantes

Les deux SBFB.json existants contiennent `"node_id": "PLACEHOLDER"`.

Avec Option D :
- `serde(default)` sur `node_id: Option<String>` → `"PLACEHOLDER"` est
  parse comme `Some("PLACEHOLDER")`.
- La verification est supprimee, donc `"PLACEHOLDER"` n'est plus un
  probleme.
- Les apps sont deployables telles quelles (si le dev pousse le repo).
- Le `"PLACEHOLDER"` est du bruit inoffensif dans le JSON.

**Pour la proprete, migrer les apps existantes :** retirer `node_id` de
SBFB.json lors du sprint S73 Phase A, en meme temps que l'ajout de
`schema_version: 2`.

### 6.6 Impact sur la securite

**Ce qu'on perd :** La garantie "seul le noeud X peut deployer cette
app" disparait du manifeste. Mais cette garantie est **illusoire** dans
le modele open source : le code est public, n'importe qui peut le
cloner et le deployer. La "restriction" par node_id dans SBFB.json
ne protege contre rien qu'un `git clone` + `sed` ne contourne.

**Ce qu'on garde :** La provenance signee par Ed25519 prouve que le
deployeur est bien le noeud X. C'est la seule preuve cryptographique
reelle. Le node_id dans SBFB.json n'est pas signe (il fait partie du
zip, pas de la signature provenance directement), donc il n'a pas de
valeur cryptographique propre.

**Ce qu'on ameliore :** La reproductibilite du hash. Aujourd'hui,
chaque deployeur produit un hash different (car node_id different dans
SBFB.json). Avec Option D, le meme code produit le meme hash pour
tous les deployeurs. C'est meilleur pour la verification tiers et pour
le caching P2P (meme hash = meme blob = meilleure distribution).

---

## 7. Schema SBFB.json v2

### 7.1 Proposition

```json
{
  "schema_version": 2,
  "name": "my-app",
  "version": "0.1.0",
  "display_name": "Mon Application",
  "description": "Description courte de l'application",
  "category": "utility",
  "license": "AGPL-3.0-or-later",
  "lang": "fr",
  "bridge": {
    "methods": ["storage_get", "storage_set", "storage_list"],
    "events": ["task_result_ready"],
    "heartbeat": true
  },
  "tech": {
    "type": "static",
    "build_command": null,
    "entry_point": "index.html"
  },
  "requirements": {
    "min_bridge_version": "1.0.0",
    "offline_capable": true,
    "estimated_size_kb": 50
  }
}
```

### 7.2 Differences avec v1

| Champ | v1 | v2 | Notes |
|---|---|---|---|
| `schema_version` | absent | 2 | Discriminant de version |
| `node_id` | **obligatoire** | **SUPPRIME** | Attribution dans provenance |
| `name` | present (non lu) | present (lu) | Identifiant app |
| `version` | present (optionnel) | present (obligatoire) | Semver |
| `display_name` | absent | nouveau | Nom affichable UI |
| `description` | absent | nouveau | Description courte |
| `category` | absent | nouveau | Classification app |
| `license` | absent | nouveau | SPDX identifier |
| `lang` | absent | nouveau | Langue primaire |
| `bridge` | absent | nouveau | Methodes bridge declarees |
| `tech` | absent | nouveau | Type de techno (static/react/pyodide/wasm) |
| `requirements` | absent | nouveau | Conditions d'execution |

### 7.3 Struct Rust proposee

```rust
#[derive(Debug, Deserialize)]
struct SbfbJson {
    /// v1 compat: present dans les anciens JSON, ignore.
    #[serde(default)]
    node_id: Option<String>,
    
    /// Schema version. Absent ou 1 = v1 (ancien format). 2 = v2.
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    
    /// App identifier. Obligatoire v2, absent v1.
    #[serde(default)]
    name: Option<String>,
    
    /// Semver. Optionnel v1, obligatoire v2.
    #[serde(default)]
    version: Option<String>,
    
    /// Display name for UI.
    #[serde(default)]
    display_name: Option<String>,
    
    /// Short description.
    #[serde(default)]
    description: Option<String>,
    
    /// Category tag.
    #[serde(default)]
    category: Option<String>,
    
    /// SPDX license identifier.
    #[serde(default)]
    license: Option<String>,
    
    /// Primary language (BCP-47).
    #[serde(default)]
    lang: Option<String>,
    
    /// Bridge configuration.
    #[serde(default)]
    bridge: Option<BridgeConfig>,
    
    /// Technology type.
    #[serde(default)]
    tech: Option<TechConfig>,
    
    /// Execution requirements.
    #[serde(default)]
    requirements: Option<RequirementsConfig>,
}

fn default_schema_version() -> u32 { 1 }

#[derive(Debug, Deserialize)]
struct BridgeConfig {
    #[serde(default)]
    methods: Vec<String>,
    #[serde(default)]
    events: Vec<String>,
    #[serde(default)]
    heartbeat: bool,
}

#[derive(Debug, Deserialize)]
struct TechConfig {
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    build_command: Option<String>,
    #[serde(default = "default_entry_point")]
    entry_point: String,
}

fn default_entry_point() -> String { "index.html".to_string() }

#[derive(Debug, Deserialize)]
struct RequirementsConfig {
    #[serde(default)]
    min_bridge_version: Option<String>,
    #[serde(default)]
    offline_capable: bool,
    #[serde(default)]
    estimated_size_kb: Option<u32>,
}
```

### 7.4 Validation deploy.rs v2

```rust
fn validate_sbfb_json(sbfb: &SbfbJson, strict_v2: bool) -> Result<(), String> {
    if sbfb.schema_version == 2 || strict_v2 {
        // v2 : name et version obligatoires
        if sbfb.name.is_none() || sbfb.name.as_deref() == Some("") {
            return Err("SBFB.json v2 requires non-empty 'name'".into());
        }
        if sbfb.version.is_none() || sbfb.version.as_deref() == Some("") {
            return Err("SBFB.json v2 requires non-empty 'version'".into());
        }
        // Valider bridge.methods si present
        if let Some(ref bridge) = sbfb.bridge {
            let valid_methods = [
                "task_submit", "storage_get", "storage_set", "storage_list",
                "storage_delete", "identity_pubkey", "node_status", "browse_list",
                "provenance_get", "provenance_verify", "pii_redact",
                "feed_cursor_get", "storage_version",
            ];
            for method in &bridge.methods {
                if !valid_methods.contains(&method.as_str()) {
                    return Err(format!("SBFB.json: unknown bridge method '{method}'"));
                }
            }
        }
    }
    // node_id est ignore quel que soit le schema_version
    if let Some(ref nid) = sbfb.node_id {
        tracing::debug!(node_id = %nid, "SBFB.json contains node_id (deprecated, ignored)");
    }
    Ok(())
}
```

### 7.5 Compat descendante

| Cas | SBFB.json contenu | Resultat |
|---|---|---|
| App existante v1 | `{"node_id": "PLACEHOLDER", "name": "x", "version": "1.0.0"}` | Parse OK. `schema_version` defaut = 1, pas de validation stricte. node_id ignore. |
| Nouvelle app v2 sans node_id | `{"schema_version": 2, "name": "x", "version": "1.0.0"}` | Parse OK. Validation v2 passe. |
| Nouvelle app v2 avec bridge | `{"schema_version": 2, "name": "x", "version": "1.0.0", "bridge": {"methods": ["storage_get"]}}` | Parse OK. Bridge valide. |
| JSON minimal v1 | `{"node_id": "abc"}` | Parse OK. Version 1, pas de validation stricte. |
| JSON vide | `{}` | Parse OK (tout serde default). Version 1, pas de validation. |

**Aucun breaking change.** Tous les SBFB.json existants continuent de
parser. La validation stricte ne s'applique qu'a `schema_version: 2`.

---

## 8. Migration des apps existantes

### 8.1 sbfb-explorer

```json
// AVANT
{"node_id": "PLACEHOLDER", "name": "sbfb-explorer", "version": "1.0.0"}

// APRES
{
  "schema_version": 2,
  "name": "sbfb-explorer",
  "version": "1.0.0",
  "display_name": "SBFB Protocol Explorer",
  "description": "Application educative : architecture, lifecycle, securite du protocole SBFB",
  "category": "education",
  "license": "AGPL-3.0-or-later",
  "lang": "fr",
  "bridge": {
    "methods": ["node_status", "identity_pubkey", "browse_list", "provenance_verify"],
    "events": [],
    "heartbeat": true
  },
  "tech": {
    "type": "static",
    "entry_point": "index.html"
  },
  "requirements": {
    "min_bridge_version": "1.0.0",
    "offline_capable": true,
    "estimated_size_kb": 30
  }
}
```

### 8.2 sbfb-ideas

```json
// AVANT
{"node_id": "PLACEHOLDER", "name": "sbfb-ideas", "version": "0.1.0"}

// APRES
{
  "schema_version": 2,
  "name": "sbfb-ideas",
  "version": "0.1.0",
  "display_name": "SBFB Ideas Hub",
  "description": "Hub d'idees collaboratif avec vote P2P et stockage distribue",
  "category": "collaboration",
  "license": "AGPL-3.0-or-later",
  "lang": "fr",
  "bridge": {
    "methods": ["storage_get", "storage_set", "storage_list", "storage_delete", "identity_pubkey", "node_status", "storage_version"],
    "events": [],
    "heartbeat": true
  },
  "tech": {
    "type": "static",
    "entry_point": "index.html"
  },
  "requirements": {
    "min_bridge_version": "1.0.0",
    "offline_capable": true,
    "estimated_size_kb": 25
  }
}
```

---

## 9. Sequencing d'implementation recommande

### S73 Phase A (SBFB.json v2 + suppression contrainte node_id)

**Etape 1 : Modifier le struct SbfbJson** (5 minutes)
- `node_id: String` → `node_id: Option<String>` avec `#[serde(default)]`
- Ajouter `schema_version: u32` avec default 1
- Ajouter `name: Option<String>`, `bridge: Option<BridgeConfig>`, etc.

**Etape 2 : Supprimer la verification node_id** (2 minutes)
- Retirer le bloc `if sbfb.node_id != state.node_id` (lignes 119-128)
- Ajouter un `debug!` log si node_id est present (deprecated warning)

**Etape 3 : Ajouter la validation v2** (15 minutes)
- Fonction `validate_sbfb_json()`
- Validation `name` + `version` obligatoires si schema_version == 2
- Validation `bridge.methods` contre la liste des methodes connues

**Etape 4 : Migrer les apps existantes** (10 minutes)
- Mettre a jour `examples/sbfb-explorer/SBFB.json` → v2
- Mettre a jour `examples/sbfb-ideas/SBFB.json` → v2

**Etape 5 : Tests** (20 minutes)
- Test v1 compat (JSON ancien format, parse OK)
- Test v2 avec tous les champs
- Test v2 rejet si name manquant
- Test v2 rejet si bridge.methods invalide
- Test migration apps existantes (deploy pipeline integre)
- Modifier `sbfb_json_node_id_mismatch_detected` → tester que le mismatch
  ne provoque plus de rejet (juste un warning log)

**Total estime : ~1 heure de code, ~4-6 tests.**

### S73 Phase B+ : Le reste des templates
Pas d'impact de la contrainte node_id sur les phases suivantes. Les
templates generent un SBFB.json v2 sans node_id, qui est deploye tel
quel.

---

## 10. Risques et edge cases

### 10.1 Fork malveillant

**Scenario :** Un attaquant fork un repo legitime, ne change rien, et
deploie l'app "au nom" de son propre noeud.

**Avec node_id dans SBFB.json :** L'attaquant doit modifier SBFB.json
avec son propre node_id, ce qui change le hash de l'artefact. Le hash
est donc different du hash de l'app originale. Pas d'usurpation possible.
MAIS : le hash etait deja different car le node_id original n'est pas
celui de l'attaquant. C'est un deploy independant dans tous les cas.

**Sans node_id (Option D) :** L'attaquant fork, ne change rien, deploie.
Le hash est IDENTIQUE a celui du deploiement original. La provenance est
differente (signee par un autre noeud). C'est le modele open source :
deux miroirs du meme code produisent le meme artefact. Ce n'est PAS un
probleme de securite -- c'est le fonctionnement normal. L'utilisateur
choisit a quel noeud faire confiance via la provenance.

**Verdict :** Pas de degradation de securite. La provenance signe par
le deployeur est le seul mecanisme de confiance reel.

### 10.2 Pollution namespace

**Scenario :** Deux apps differentes avec le meme `name` dans SBFB.json.

**Solution :** Le `name` dans SBFB.json n'est PAS un identifiant unique
global. L'identifiant unique est le hash iroh-blobs de l'archive. Le
`name` est un identifiant **par deployeur**. Si deux deployeurs publient
des apps avec le meme nom, elles sont differenciees par le hash et la
provenance.

### 10.3 Regression test node_id

Le test `sbfb_json_node_id_mismatch_detected` (L696-706) verifie
actuellement qu'un mismatch est detecte. Avec Option D, ce test doit
etre modifie ou supprime. Le mismatch est toujours detecte (compare avec
le daemon node_id) mais il ne cause plus de rejet -- juste un log.

---

## 11. Synthese

### Le probleme
Le `node_id` dans SBFB.json est un frein a la portabilite des templates
et au workflow Factory. C'est un mecanisme de securite a valeur
limitee : le node_id n'est pas signe (il fait partie du zip, pas de la
signature cryptographique directe), et il est trivialement contournable
par un attaquant qui a acces au code source.

### La solution
Retirer la contrainte `node_id` de SBFB.json. Le manifeste declare l'app
(nom, version, permissions, tech). L'identite du deployeur est dans la
provenance signee Ed25519, qui est le seul mecanisme cryptographique reel.

### Le cout
5 lignes de changement dans deploy.rs. 1 heure de travail dans S73 Phase A.
Zero breaking change. Amelioration de la reproductibilite des hashes.

### Le benefice
Les templates et la Factory fonctionnent sans hack. SBFB.json devient un
manifeste applicatif propre et extensible. La verification tiers est
amelioree (hash reproductible sans connaitre le node_id du deployeur).
