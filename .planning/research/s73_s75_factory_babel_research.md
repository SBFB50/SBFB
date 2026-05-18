# Recherche S73-S75 : Code Factory + Babel Dogfood

**Date :** 2026-05-18
**Mode :** Ecosystem
**Confiance globale :** MEDIUM (architecture bien documentee en interne,
briques externes verifiees par web, Babel runtime NLLB non encore teste)

---

## 1. Etat des lieux : ce que le projet possede deja

### 1.1 Apps existantes et leur structure

Le projet contient trois exemples d'apps, dont deux fonctionnels sur la
plateforme actuelle :

**Protocol Explorer** (`examples/sbfb-explorer/`) — 5 fichiers, app
educative vanilla JS, utilise 4 methodes bridge (`getNodeStatus`,
`getIdentityPubkey`, `getBrowseList`, `verifyRelease`).

**Ideas Hub** (`examples/sbfb-ideas/`) — 5 fichiers, app CRUD
collaborative avec vote P2P, utilise 7 methodes bridge (full storage
CRUD + identite + sync polling via `onStorageUpdate`).

**Hello World** (`examples/hello-world-app/`) — 2 fichiers, LEGACY
Python SDK pre-pivot, inutilisable sur la plateforme actuelle.

Structure commune d'une app SBFB deployable :

```
my-app/
  index.html          # OBLIGATOIRE (entry point)
  app.js              # logique application
  style.css           # styles
  sbfb-bridge.js      # copie canonique depuis web/public/
  SBFB.json           # manifest minimal
```

### 1.2 Manifest SBFB.json actuel

Format extremement minimal :

```json
{
  "node_id": "PLACEHOLDER",
  "name": "sbfb-explorer",
  "version": "1.0.0"
}
```

Seulement 3 champs. Le `node_id` est remplace au deploy par le node_id
du daemon. Aucune declaration de permissions, bridge methods utilisees,
dependances, licence, categorie ou description. Ce manifest est
insuffisant pour une Factory qui doit savoir ce qu'une app requiert.

### 1.3 Bridge SDK

Le SDK (`web/public/sbfb-bridge.js`, 398 lignes) est une classe ES5
standalone sans dependances. 13 methodes bridge disponibles couvrant :
core (task_submit, storage_get/set), PII (pii_redact), extensions S56
(storage_list/delete, identity_pubkey, node_status, browse_list),
sync S58 (storage_version), verification S63 (provenance_get/verify,
feed_cursor_get). Plus : events push, heartbeat watchdog, lifecycle
destroy.

Distribution actuelle : copie manuelle via `scripts/sync-bridge-sdk.sh`
avec verification SHA256 post-copie.

### 1.4 Deploy verifie

Pipeline complete : clone repo → verifier `SBFB.json` (node_id match)
→ verifier `index.html` existe → zip (exclure `.git/`, symlinks) →
BLAKE3 hash → signer provenance Ed25519 → injecter `provenance.json`
→ stocker via iroh-blobs → annoncer via gossip. 18 etapes documentees,
753 lignes Rust.

### 1.5 Contraintes iframe sandbox

| Restriction | Cause |
|---|---|
| Pas de fetch/XHR/WebSocket | CSP `connect-src 'none'` |
| Pas de localStorage/IndexedDB | Origine opaque (sandbox sans allow-same-origin) |
| Pas de forms | sandbox sans `allow-forms` |
| Pas de CDN | Tout bundled dans le zip |
| Chemins relatifs obligatoires | `base: "./"` pour Vite |
| postMessage seul canal | Bridge SDK unique interface |

### 1.6 Recherche existante sur la Factory

Le document `.planning/research/sbfb_project_factory_rrv_oss_research.md`
(1267 lignes, 2026-05-17) est exhaustif. Il couvre :

- Architecture 4 zones (Factory UI / Broker / Workspace Sandbox / Preview)
- 10 gates de securite (G0-G10)
- Matrice OSS avec priorites (Copier P0, Backstage inspiration P0, etc.)
- Templates pack design avec `factory.project.json` et `factory.provenance.json`
- RRV `@dev` LocalOnly avec schema SQLite FTS5
- Babel dogfood MVP scope
- Roadmap PF-0 a PF-5 + RRV-N
- 14 questions ouvertes

**Verdict de cette recherche precedente :** "Ne pas attendre RRV complet.
Construire Factory local-first, sandboxed, avec RRV @dev LocalOnly."

---

## 2. Scaffolding et app factories : panorama externe

### 2.1 Outils de scaffolding par templates

**Recommandation : giget pour le telechargement, Copier pour la generation.**

| Outil | Forces | Faiblesses | Pertinence SBFB |
|---|---|---|---|
| **giget** (UnJS) | 3M DL/semaine, API programmatique, registre, multi-forge | Pas de templating Jinja, juste copie | HAUTE — telecharger templates depuis repos Git |
| **degit** (Rich Harris) | Simple, 500K DL/semaine | Non maintenu depuis 2023 | BASSE — giget est le successeur |
| **tiged** | Fork maintenu de degit | Moins d'adoption que giget | BASSE |
| **Copier** (Python) | Templating Jinja2, updates versionnes, hooks pre/post | Dependance Python | HAUTE — generer repos avec variables |
| **Cookiecutter** | Populaire (Python), hooks, Jinja2 | Pas d'update versionne, plus ancien | MOYENNE — Copier est plus moderne |
| **create-vite** | Officiel Vite, templates React/Vue/Svelte | Templates generiques, pas extensible facilement | BASSE — trop generique |
| **Nx generators** | Schema JSON, AST manipulation, workspace-aware | Lourd, ecosysteme Nx specifique | BASSE — trop opinione |
| **Plop** | Micro-generators, Handlebars, leger | Pas de projet-complet, composants seulement | MOYENNE — complement pour sous-generators |

**Decision recommandee pour S73 :**

Le projet est Rust-first avec des apps principalement vanilla JS/HTML.
Copier (Python) ajoute une dependance Python juste pour le scaffolding.
L'alternative pragmatique est un **generateur Rust natif** embarque dans
le daemon ou un outil CLI `sbfb` qui :

1. Telecharge un template depuis un repo Git (pattern giget) OU utilise
   des templates embarques dans le binaire
2. Remplace les variables (nom, node_id, version, permissions) via
   substitution simple (pas besoin de Jinja2 pour 5-10 variables)
3. Copie le bridge SDK canonique
4. Genere le `SBFB.json` enrichi
5. Initialise le repo Git local

Copier reste pertinent si le nombre de templates explose (>10) ou si
des hooks complexes sont necessaires. Pour le MVP avec 3-5 templates,
un generateur Rust minimaliste suffit et evite la dependance Python.

### 2.2 AI-assisted code generation (reference)

**Bolt.new :** app generation complete dans le browser via WebContainers
(StackBlitz). L'IA genere, le user preview immediatement. Pattern de
reference pour le preview-before-publish. Utilise Claude pour la
generation. Confiance : MEDIUM (source web, verifie multi-sources).

**v0.dev (Vercel) :** generation UI React, l'utilisateur voit le rendu
et peut iterer. Pattern de reference pour le diff-review avant commit.

**Factory.ai :** 3 niveaux de trust — manual approval (Low), allow safe
commands (Medium), allow all (High). Pattern directement applicable au
broker SBFB avec ses gates G0-G10.

**Lecons pour SBFB Code Factory :**

- L'utilisateur doit toujours voir le diff AVANT application
- Le preview live est essentiel (iframe sandbox = deja la)
- La generation ne doit jamais publier sans gate humain
- Les niveaux de trust (manual / semi-auto / full auto) sont standards
- Le "vibe coding" sans verification est un anti-pattern explicite

### 2.3 Sandbox / broker patterns externes

**Flatpak Portals :** modele de reference pour le broker. L'app demande
un acces via un portal, le systeme demande permission a l'utilisateur,
le portal donne un file descriptor si autorise. Pattern "ask, don't
guess". Directement applicable au broker Factory.

**WebContainers (StackBlitz) :** Node.js complet dans le browser via
WASM. Requiert SharedArrayBuffer et cross-origin isolation. SBFB a
deja COOP/COEP sur blob-serve. Potentiellement utilisable pour un
build preview, mais ajoute 20+ MB de WASM. Overkill pour MVP.

**Sandpack (CodeSandbox) :** composant React pour code editing live.
Supporte React, Next, Vite, Astro. Pourrait etre integre dans le
shell SBFB pour le preview. Confiance : MEDIUM.

**Decision pour S74 :** Le broker SBFB doit suivre le pattern Flatpak
portals — l'iframe (app Factory ou app user) demande une action, le
broker (dans le daemon) mediate, l'utilisateur confirme via le shell.
Pas de WebContainers au MVP (trop lourd, SBFB a deja le blob-serve
pour le preview iframe).

### 2.4 Plugin ecosystems (reference models)

**VS Code Marketplace :** signature a la publication, verification a
l'installation. Mais recherche OX Security 2025 montre que le modele
est contournable — le badge "verified" prouve seulement ownership d'un
domaine. SBFB fait mieux avec Ed25519 + SLSA L1 provenance.

**Obsidian (2025-2026) :** passage de review manuelle a scans automatises
par version + safety scorecard (maintenance, frequence update, risques).
Bon modele pour un catalogue d'apps SBFB. SBFB peut ajouter un
scorecard similaire base sur le feed public et la provenance.

**Home Assistant HACS :** community store sans review de securite,
quality scale en tiers. Le modele SBFB est plus strict (provenance
obligatoire pour Verified Release).

**Decision :** Le modele SBFB est deja meilleur que la plupart des
ecosystemes plugins grace a la provenance Ed25519 + SLSA L1. Le
manque est le manifest enrichi (permissions declarees, capabilities)
et le scorecard automatique.

---

## 3. Architecture Code Factory — recommandations

### 3.1 S73 : Templates

#### Manifest SBFB.json enrichi

Le manifest actuel est trop pauvre. Proposition de manifest v2 :

```json
{
  "schema_version": 2,
  "node_id": "PLACEHOLDER",
  "name": "my-app",
  "version": "0.1.0",
  "display_name": "Mon Application",
  "description": "Description courte",
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

Le champ `bridge.methods` est critique : il permet au shell de
pre-valider les capabilities, d'afficher un badge "cette app utilise
le stockage P2P et la verification de provenance", et de refuser
une app qui declare des methodes inexistantes.

`schema_version: 2` permet la coexistence avec les apps existantes
(`schema_version: 1` ou absent = ancien format, compatibilite).

#### Systeme de templates recommande

**Approche : templates Git-based + generateur Rust embarque.**

Pas de Copier Python, pas de Cookiecutter. Le daemon SBFB est deja
Rust. Le generateur est un module Rust dans `nexus-shell-daemon` ou
un binary separe `sbfb-create`.

Templates stockes dans un repo Git SBFB officiel
(`github.com/SBFB50/sbfb-templates`) ou embarques dans le binaire :

```
sbfb-templates/
  static-minimal/      # HTML pur + bridge
    template.json       # metadata template
    files/
      index.html        # {{name}}, {{description}} placeholders
      style.css
      app.js
      SBFB.json
  react-vite/           # React + Vite + bridge
    template.json
    files/
      index.html
      vite.config.ts
      src/App.tsx
      src/main.tsx
      package.json
      SBFB.json
  pyodide/              # Python/Pyodide + bridge
    template.json
    files/
      index.html
      main.py
      SBFB.json
```

Chaque `template.json` :

```json
{
  "id": "static-minimal",
  "version": "0.1.0",
  "display_name": "Application statique minimale",
  "description": "HTML + CSS + JS avec bridge SBFB",
  "variables": [
    {"name": "project_name", "prompt": "Nom du projet", "default": "my-app"},
    {"name": "description", "prompt": "Description", "default": ""},
    {"name": "author", "prompt": "Auteur", "default": ""},
    {"name": "bridge_methods", "prompt": "Methodes bridge", "type": "multi_select",
     "options": ["storage_get", "storage_set", "storage_list", "storage_delete",
                 "task_submit", "identity_pubkey", "node_status", "browse_list",
                 "provenance_get", "provenance_verify"]}
  ],
  "post_create": ["git init", "git add .", "git commit -m 'Initial scaffold via sbfb create'"],
  "content_hash": "blake3:..."
}
```

#### Templates concrets a livrer S73

1. **static-minimal** : HTML pur, bridge SDK, SBFB.json enrichi, style.css
   theme SBFB (dark, CSS vars GitHub-dark), app.js skeleton avec bridge
   init/destroy. ~100 LOC total. Cas d'usage : page informative,
   dashboard simple.

2. **static-storage** : Comme static-minimal + storage CRUD complet
   (pattern Ideas Hub : list/get/set/delete + onStorageUpdate polling).
   ~300 LOC. Cas d'usage : app collaborative.

3. **react-vite** : React 19 + Vite + TypeScript + `base: "./"` +
   hook `useSBFBBridge()` + bridge SDK en global. Build `npm run build`
   → dist/ → deploy. ~400 LOC. Cas d'usage : app complexe avec state.

4. **pyodide-notebook** : HTML + Pyodide loader + bridge JS-Python
   interop. Python code dans `<script type="text/python">` ou `main.py`.
   ~200 LOC. Cas d'usage : calcul scientifique, NLP.

5. **wasm-app** (stretch) : HTML + WASM loader + bridge. Skeleton pour
   Rust/C++ compile en WASM. ~150 LOC.

#### CLI `sbfb create`

```bash
# Interactif
sbfb create

# Non-interactif
sbfb create --template static-storage --name babel-reader --description "Lecteur P2P"

# Depuis un template Git
sbfb create --from github.com/SBFB50/sbfb-templates/static-minimal
```

Implementation : sous-commande du daemon ou binary Rust separe.
Le daemon a deja un CLI (`nexus-shell-daemon`). Ajouter `sbfb create`
comme sous-commande est coherent.

#### Validation de template

Un template est lui-meme un artifact verifiable. `template.json` inclut
`content_hash` (BLAKE3 de tous les fichiers template). Le generateur
verifie le hash avant generation. Si le template vient d'un repo Git,
le deploy verifie standard s'applique.

### 3.2 S74 : Broker/Sandbox

#### Architecture broker

Le broker est un module du daemon, pas une app iframe. C'est la
decision architecturale cle deja prise dans la recherche precedente
(§7 du doc Factory/RRV).

```
Factory UI (shell page React /factory)
  |
  | postMessage bridge (methodes dediees factory_*)
  | OU route HTTP /api/v1/factory/*
  |
  v
Factory Broker (dans nexus-shell-daemon-core)
  |
  +-- Template Engine (genere fichiers)
  +-- Diff Generator (calcule diff avant application)
  +-- Preview Manager (sert app dans iframe blob-serve)
  +-- Publish Gate (verifie toutes les conditions avant publish)
  +-- Audit Log (chaque action loguee avec timestamp+auteur)
  |
  v
Workspace (dossier local, borne par path allowlist)
```

**Decision critique :** Le broker NE PASSE PAS par le bridge iframe.
Le broker est un composant du daemon, accede via routes HTTP authentifiees
(`/api/v1/factory/*`) avec le meme bearer token que le reste du daemon.
La Factory UI est une page React du shell (comme `/deploy`), pas une
app iframe sandboxee.

Raison : le broker doit acceder au filesystem, executer git, lancer
des builds. Ce sont des operations privilegiees qu'une iframe sandboxee
ne peut pas faire par design.

#### Workflow diff-review-approve

1. **Intent** : l'utilisateur decrit une modification (texte libre ou
   selection template)
2. **Diff generation** : le broker genere les fichiers en memoire et
   calcule le diff avec l'etat actuel du workspace
3. **Review UI** : le shell affiche le diff dans un composant React
   (diff viewer, syntax highlighting). Chaque fichier est expandable.
4. **Approve/Reject** : l'utilisateur clique "Appliquer" ou "Annuler"
5. **Apply** : le broker ecrit les fichiers sur disque
6. **Git commit** (optionnel) : le broker commit avec message genere
7. **Preview** : le broker zippe le projet, le sert via blob-serve,
   affiche dans iframe sandbox
8. **Publish gate** : quand l'utilisateur demande "Publier", le broker
   verifie : index.html existe, SBFB.json valide, no secrets detectes
   (Semgrep/Trivy), build passes (si applicable), preview OK

#### Sandbox de generation

Pour le MVP, le "sandbox" est simplement un path allowlist :

- Le broker connait le workspace path autorise (ex: `~/Code/my-app/`)
- Toute ecriture hors de ce path est refusee
- Les commandes shell sont allowlistees : `git init`, `git add`,
  `git commit`, `npm install`, `npm run build`, `npm test`
- Les variables d'environnement du build sont nettoyees (pas de secrets)

Pour le long terme, devcontainers ou bubblewrap (Linux) / conteneurs
(Windows) sont envisageables mais overkill pour le MVP.

#### Tracabilite

Chaque action du broker est loguee dans `factory.audit.jsonl` :

```json
{"ts": "2026-05-18T10:00:00Z", "action": "template_generate", "template": "static-storage",
 "project": "babel-reader", "user_confirmed": true}
{"ts": "2026-05-18T10:01:00Z", "action": "file_write", "path": "index.html",
 "content_hash": "blake3:...", "user_confirmed": true}
{"ts": "2026-05-18T10:02:00Z", "action": "preview_serve", "hash": "abc123..."}
{"ts": "2026-05-18T10:05:00Z", "action": "publish_attempt", "state": "verified",
 "provenance_hash": "blake3:..."}
```

### 3.3 S75 : Babel Dogfood

#### Ce dont Babel a besoin du bridge

D'apres le research doc Babel et l'architecture SBFB :

| Besoin Babel | Methode bridge | Etat |
|---|---|---|
| Stocker corpus local | `storage_set` | EXISTE |
| Lire texte source | `storage_get` | EXISTE |
| Lister textes/langues | `storage_list` | EXISTE |
| Supprimer draft | `storage_delete` | EXISTE |
| Identite lecteur | `identity_pubkey` | EXISTE |
| Soumettre traduction | `task_submit` | EXISTE |
| Recevoir resultat | `onEvent("task_result_ready")` | EXISTE |
| Sync P2P corpus | `onStorageUpdate` | EXISTE |
| Verifier provenance | `provenance_verify` | EXISTE |
| PII redaction | `pii_redact` | EXISTE |

**Conclusion : toutes les methodes bridge necessaires a Babel existent
deja.** Aucune extension bridge n'est requise pour le MVP Babel.

#### Workflow Babel end-to-end

```
1. sbfb create --template static-storage --name babel-reader
   → repo babel-reader/ avec scaffold SBFB

2. Developpement app reader :
   - UI : liste de textes, lecteur, toggle langue
   - Data : textes fixtures en JSON dans le zip
   - Storage : progression lecture, bookmarks, preferences

3. Enrichissement avec traduction :
   - L'app affiche un texte source (ex: Le Petit Prince, domaine public)
   - L'utilisateur clique "Traduire en wolof"
   - bridge.submitTask({ prompt: texte, task_type: "translation",
     metadata: { source_lang: "fra_Latn", target_lang: "wol_Latn" } })
   - Un worker avec NLLB-200 traite la tache
   - bridge.onEvent("task_result_ready", cb) recoit la traduction
   - bridge.setStorage("translations/fr-wo/" + chunkId, translation)

4. Validation humaine :
   - L'app affiche traduction machine + original cote a cote
   - L'utilisateur (native speaker) approve/reject/edite
   - bridge.setStorage("reviews/" + translationId + "/" + pubkey, review)
   - Consensus par nombre de reviews positifs

5. Provenance :
   - bridge.verifyRelease("babel-reader") verifie la provenance de l'app
   - Chaque traduction stockee avec metadata (modele, worker, timestamp)
   - Chaque review stockee avec pubkey du reviewer

6. Deploy :
   - sbfb deploy --repo github.com/SBFB50/babel-reader
   - Provenance SLSA L1 generee
   - App disponible sur le reseau via Browse
```

#### NLLB-200 dans le navigateur (offline)

D'apres la recherche web, **Transformers.js** peut executer NLLB-200
distilled 600M (242 MB) entierement dans le browser via ONNX Runtime
WASM. Premier chargement ~30 secondes, ensuite cache IndexedDB.
Supporte 196 langues.

**MAIS** : l'iframe sandbox SBFB bloque IndexedDB (origine opaque) et
`connect-src 'none'` bloque le telechargement du modele. Donc :

- **Option A (worker SBFB) :** La traduction passe par `task_submit` →
  un worker SBFB avec NLLB-200 (Ollama ou ctranslate2) traite la tache.
  C'est le chemin naturel SBFB. Confiance : HAUTE.

- **Option B (Pyodide dans iframe) :** L'app embarque Pyodide + NLLB
  dans le zip. Probleme : le zip ferait >300 MB, et Pyodide dans iframe
  sandbox sans IndexedDB est fragile. Confiance : BASSE.

- **Option C (Transformers.js dans iframe) :** L'app embarque le modele
  ONNX dans le zip. Meme probleme de taille + pas d'IndexedDB pour
  cache. Confiance : BASSE.

**Decision : Option A (worker SBFB).** La traduction est un cas d'usage
parfait pour les workers SBFB. C'est coherent avec l'architecture du
reseau (compute distribue). Le modele NLLB-200 tourne sur GPU (le
worker a deja le pattern Ollama). L'app reste legere (juste UI + bridge).

#### Autres domain packs candidats

| Pack | Description | Complexite | Interet MVP |
|---|---|---|---|
| **Babel Reader** | Lecteur multilingue + traduction IA | MOYENNE | HAUTE — premier dogfood Factory |
| **Repair Notebook** | Manuels reparation offline (iFixit-like) | BASSE | HAUTE — simple, offline first |
| **Offline School** | Cours + exercices + progression | MOYENNE | HAUTE — impact humanitaire |
| **Community Health** | Donnees sante locales | HAUTE | BASSE — sensibilite donnees |
| **Crisis Platform** | Outils de crise (mesh, triage) | HAUTE | BASSE — trop complexe pour dogfood |

**Recommandation S75 :** Commencer par Babel Reader (c'est l'objectif
du sprint). Si le temps le permet, ajouter Repair Notebook comme second
domain pack pour valider que la Factory generalise.

#### Format "Domain Pack"

Un domain pack est un template enrichi avec des donnees et config
specifiques au domaine :

```
domain-packs/
  babel/
    template.json         # extends static-storage
    fixtures/
      texts/              # textes fixture domaine public
        le-petit-prince-fr.json
      languages.json      # langues supportees
    config/
      task_types.json     # types de taches specifiques (translation, review)
      storage_schema.json # structure cles storage attendue
    README.md             # documentation du domain pack
```

Le domain pack n'est PAS un template standalone. C'est une extension
d'un template de base avec des fixtures, des schemas et de la config
metier. La Factory sait appliquer un domain pack par-dessus un template.

---

## 4. Dependencies et sequencing

### 4.1 Pre-requis S73 (Templates)

| Dependency | Source | Etat | Impact |
|---|---|---|---|
| SBFB.json v1 valide | S14 (deploy.rs) | FAIT | Manifest actuel fonctionne |
| Bridge SDK stable | S56-S63 | FAIT | 13 methodes, SDK 398 lignes |
| Deploy from repo | S14 | FAIT | Pipeline complete |
| Sync bridge SDK | scripts/sync-bridge-sdk.sh | FAIT | SHA256 verification |

S73 n'a pas de prerequis bloquant. Le manifest SBFB.json doit etre
enrichi (schema_version 2) mais c'est un livrable de S73, pas un
pre-requis.

### 4.2 Pre-requis S74 (Broker/Sandbox)

| Dependency | Source | Etat | Impact |
|---|---|---|---|
| Templates fonctionnels | S73 | A FAIRE | Broker genere depuis templates |
| Shell UI extensible | web/ (React shell) | FAIT | Ajouter page /factory |
| Daemon HTTP extensible | nexus-shell-daemon | FAIT | Ajouter routes /api/v1/factory/* |
| Blob-serve preview | S12 | FAIT | Preview iframe deja en place |

S74 depend de S73 (templates) mais pas de sprints intermediaires S65-S72.
Cependant, le broker devrait integrer les patterns de governance (S67)
pour le publish gate.

### 4.3 Pre-requis S75 (Babel Dogfood)

| Dependency | Source | Etat | Impact |
|---|---|---|---|
| Factory complete | S73+S74 | A FAIRE | Babel cree via Factory |
| Worker NLLB-200 | Nouveau | A FAIRE | Backend traduction |
| Storage multi-app | S56 | FAIT | Namespace par app |
| Task dispatch | S14+ | FAIT | task_submit + worker pipeline |

**Le vrai bloqueur S75 est le backend NLLB-200 dans nexus-worker-core.**
C'est un nouveau backend LLM (calque du pattern Ollama existant) qui
doit etre concu dans S75 ou un sprint precedent. Le research doc Babel
estime 1 sprint pour ce backend (Sprint Babel A).

### 4.4 Dependances cross-sprint (S65-S72 → S73-S75)

Les sprints S65-S72 (contrat public, durabilite, governance, RRV) ne
sont pas des bloqueurs directs pour S73-S75. Mais :

- **S65 go-live** : si le feed public est live, les apps creees par
  Factory seront visibles immediatement. Pas un bloqueur technique.
- **S67 governance** : les patterns publish gate du broker devraient
  s'aligner avec la governance du reseau. Coordination recommandee.
- **S70-S72 RRV** : le RRV `@dev` LocalOnly prevu dans S73 est
  independant du RRV reseau. Pas de bloqueur.

---

## 5. Pitfalls et risques

### 5.1 Pitfalls critiques

#### P1 : Factory devient protocole metier

**Ce qui va mal :** On ajoute des methodes bridge `factory_*` dans le
protocole core, ou on met du code Factory dans nexus-coordinator-rs.
Le protocole perd sa neutralite.

**Prevention :** Factory est un module du daemon (ou un binaire separe),
jamais dans le coordinator ni dans le protocole core. Les methodes
bridge restent generiques. Factory utilise les routes HTTP du daemon,
pas le bridge iframe.

#### P2 : SBFB.json v2 casse les apps existantes

**Ce qui va mal :** On enrichit le manifest SBFB.json et les deux apps
existantes (Explorer, Ideas Hub) cassent au deploy.

**Prevention :** `schema_version` absent ou 1 = ancien format, accepte
tel quel. `schema_version: 2` = nouveau format avec champs enrichis.
Le deploy.rs doit supporter les deux. Migration des apps existantes
en meme temps que S73.

#### P3 : Le broker ecrit hors du workspace

**Ce qui va mal :** Un bug dans le path allowlist permet d'ecrire des
fichiers hors du dossier projet autorise. Escalation de privileges.

**Prevention :** Le broker utilise `canonicalize()` + prefix check
AVANT toute ecriture. Tests de traversal (`../`, `..\\`, symlinks)
obligatoires. Meme rigueur que `validate_zip_path()` dans blob_serve.rs.

#### P4 : NLLB-200 backend pas pret pour S75

**Ce qui va mal :** Le backend worker NLLB-200 n'est pas assez mature
et S75 Babel n'a pas de traduction fonctionnelle.

**Prevention :** Le MVP Babel peut fonctionner avec des traductions
mock (textes pre-traduits en fixture) pendant que le backend NLLB-200
murit. La traduction live est un stretch goal de S75, pas un hard
requirement pour le dogfood.

### 5.2 Pitfalls moderes

#### P5 : Templates trop rigides

**Prevention :** Garder les templates minimalistes (structure + SDK +
manifest + tests smoke). Ne pas imposer une architecture app complexe.

#### P6 : Preview ne fonctionne pas comme en prod

**Prevention :** La preview utilise exactement le meme chemin que le
deploy : zip → blob-serve → iframe sandbox. Meme CSP, meme sandbox,
memes contraintes.

#### P7 : Domain pack trop ambitieux

**Prevention :** Le domain pack S75 est Babel Reader ONLY (lecteur +
storage + traduction mock). Pas de corpus pipeline, pas de validation
communautaire, pas de multi-source. Ces features sont post-S75.

### 5.3 Pitfalls mineurs

#### P8 : Scaffolding CLI vs UI confusion

**Prevention :** `sbfb create` est CLI-first pour les devs. La
Factory UI (S74) est le pendant visuel pour les non-devs. Les deux
appellent le meme generateur Rust.

#### P9 : Templates Git drift

**Prevention :** Chaque template a un `content_hash` BLAKE3. Le
generateur verifie le hash. `factory.template.lock` dans le projet
genere permet de detecter si le template a change depuis la generation.

---

## 6. Plans de phases

### 6.1 S73 — Code Factory Templates (4 phases)

**Phase A : SBFB.json v2 + validation**
- Spec du manifest enrichi (schema_version 2, bridge.methods, tech, requirements)
- Parser/validateur dans deploy.rs (support v1 + v2)
- Migration des deux apps existantes vers v2
- Tests : v1 compat, v2 parse, v2 reject invalid
- Delta : ~4 tests Rust

**Phase B : Template engine + 3 templates**
- Generateur Rust : substitution variables, copie bridge SDK, init repo
- Template `static-minimal` (HTML pur)
- Template `static-storage` (storage CRUD, pattern Ideas Hub)
- Template `react-vite` (React 19 + bridge hook)
- Tests : generation snapshot, fichiers attendus, SBFB.json valide
- Delta : ~6 tests Rust

**Phase C : CLI `sbfb create`**
- Sous-commande dans nexus-shell-daemon ou binary separe
- Mode interactif (prompts) et non-interactif (flags)
- Telechargement template depuis repo Git (optionnel, pour templates externes)
- Verification content_hash BLAKE3 du template
- Tests : CLI happy path, invalid template, path traversal
- Delta : ~4 tests Rust

**Phase D : Template verification + factory.template.lock**
- `factory.template.lock` genere dans chaque projet cree
- `factory.provenance.json` genere (lineage creation)
- Template lui-meme deployable comme app verifiee (meta-verification)
- Tests : lock hash stable, provenance structure
- Delta : ~3 tests Rust

### 6.2 S74 — Code Factory Broker/Sandbox (4 phases)

**Phase A : Broker architecture + routes API**
- Module `factory_broker` dans nexus-shell-daemon-core
- Routes HTTP : `/api/v1/factory/templates` (list), `/api/v1/factory/create`
  (generate), `/api/v1/factory/diff` (preview changes), `/api/v1/factory/apply`
  (apply changes), `/api/v1/factory/preview` (serve preview)
- Path allowlist + canonicalize
- Audit log (factory.audit.jsonl)
- Tests : path traversal denied, routes auth required
- Delta : ~6 tests Rust

**Phase B : Diff generation + review API**
- Diff engine : calcule fichiers modifies/ajoutes/supprimes entre
  workspace actuel et modifications proposees
- Format diff : JSON structure (pas unified diff text) pour affichage React
- Route `/api/v1/factory/diff` retourne le diff, `/api/v1/factory/apply`
  applique seulement si diff precedemment genere + user_confirmed=true
- Tests : diff calcul correct, apply sans diff = refuse, concurrent apply
- Delta : ~5 tests Rust

**Phase C : Review UI (page React /factory)**
- Page shell `/factory` avec :
  - Template selector (dropdown des templates disponibles)
  - Variables form (nom, description, bridge methods)
  - Diff viewer (fichiers expandables, syntax highlighting via CSS)
  - Approve/Reject buttons
- Composant DiffViewer reutilisable
- Tests Vitest : DiffViewer renders, approve mutation
- Delta : ~4 tests Vitest

**Phase D : Preview sandbox + publish gate**
- Preview : le broker zippe le workspace, le sert via blob-serve, affiche
  dans iframe (meme chemin que deploy)
- Publish gate checklist : index.html existe, SBFB.json v2 valide, bridge
  methods declarees existent, no secrets detected (regex scan), build OK
  (si build_command present dans SBFB.json)
- Route `/api/v1/factory/publish-check` retourne la checklist
- Tests : publish gate pass/fail, preview serve, missing index.html
- Delta : ~5 tests Rust

### 6.3 S75 — Babel Dogfood / Domain Packs (5 phases)

**Phase A : Domain pack format + Babel pack**
- Spec du format domain pack (template.json extended, fixtures, config)
- Babel domain pack : textes fixture (3 textes domaine public, ~5 langues),
  languages.json, task_types.json, storage_schema.json
- Integration dans `sbfb create --domain-pack babel`
- Tests : domain pack parse, fixtures loaded, generated app deployable
- Delta : ~4 tests Rust

**Phase B : Babel reader app via Factory**
- Creer babel-reader via `sbfb create --domain-pack babel`
- UI reader : liste textes, lecteur plein ecran, toggle langue
- Storage : progression lecture (bridge storage_get/set)
- Identity : affichage pubkey lecteur
- Tests : app deploie, bridge storage fonctionne
- Delta : ~3 tests Playwright

**Phase C : Bridge integration (storage + tasks)**
- Storage structure : `texts/{id}`, `translations/{lang}/{id}`,
  `bookmarks/{pubkey}/{id}`, `reviews/{translationId}/{pubkey}`
- task_submit pour traduction (mock backend ou NLLB si pret)
- onEvent pour recevoir resultats
- onStorageUpdate pour sync P2P
- Tests : storage CRUD, task submit mock, sync poll
- Delta : ~4 tests

**Phase D : Deploy verifie + feed publication**
- Deploy babel-reader via deploy-from-repo
- Provenance SLSA L1 generee et verifiable
- Feed entry ReleasePublished
- Verification E2E : deploy → browse → ouvrir → lire
- Tests : E2E deploy, provenance verify, feed entry
- Delta : ~3 tests

**Phase E : Domain pack format spec + second pack (stretch)**
- Formaliser le format domain pack dans un doc spec
- Second domain pack candidat : Repair Notebook (simpler que Babel)
- Tests : second pack genere une app deployable
- Delta : ~2 tests

---

## 7. Stack technique

### 7.1 S73 — Template Engine

| Composant | Technologie | Version | Raison |
|---|---|---|---|
| Generateur | Rust natif | - | Coherent avec le stack existant |
| Substitution | `String::replace` ou mini-template | - | 5-10 variables max, pas besoin de Jinja |
| CLI | clap (deja en place) | 4.x | Sous-commande daemon |
| Hash | blake3 | 1.x | Verification templates |
| Git init | `std::process::Command` | - | `git init` + `git add` + `git commit` |
| Bridge SDK | copie fichier | - | Meme pattern que sync-bridge-sdk.sh |

### 7.2 S74 — Broker

| Composant | Technologie | Version | Raison |
|---|---|---|---|
| HTTP routes | axum (deja en place) | 0.8.x | Extension du daemon existant |
| Path validation | std::fs::canonicalize | - | Path traversal prevention |
| Diff | similar (Rust) | 2.x | Diff computation |
| Audit log | serde_json → fichier JSONL | - | Simple, auditable |
| UI | React page dans le shell | - | Coherent avec l'existant |
| Diff viewer | CSS + HTML pre | - | Pas de lib externe |
| Preview | blob-serve existant | - | Meme pipeline que deploy |

### 7.3 S75 — Babel

| Composant | Technologie | Version | Raison |
|---|---|---|---|
| App | Vanilla JS + bridge | - | Meme pattern qu'Explorer/Ideas |
| Storage | bridge storage API | - | Deja en place |
| Traduction | NLLB-200 via worker | - | Option A (worker SBFB) |
| NLLB runtime | ctranslate2 ou Ollama | - | A determiner selon maturite |
| Textes fixture | JSON dans zip | - | Simple, offline |
| Provenance | deploy-from-repo | - | Pipeline existante |

---

## 8. Implications pour le roadmap

### 8.1 Structure de phases recommandee

1. **S73 Templates** (4 phases A-D) — Fondation. Aucun bloqueur.
   Peut commencer des que le sprint est planifie.

2. **S74 Broker/Sandbox** (4 phases A-D) — Extension du daemon.
   Depend de S73 pour les templates mais pas des sprints RRV.

3. **S75 Babel Dogfood** (5 phases A-E) — Preuve du systeme.
   Depend de S73+S74. Le backend NLLB-200 est un risque. Mitigation :
   traductions mock en fixture.

### 8.2 Rationale d'ordonnancement

- S73 avant S74 : le broker genere depuis des templates, donc les
  templates doivent exister d'abord.
- S74 avant S75 : Babel doit etre cree via Factory pour valider le
  dogfood. Sans le broker, on "triche" en codant Babel a la main.
- Le NLLB-200 backend peut etre travaille en parallele ou dans S75
  Phase C. Si pas pret, Babel fonctionne avec des fixtures.

### 8.3 Drapeaux de recherche supplementaire

- **S73 Phase A** : le format exact du manifest v2 merite un preflight
  avec les apps existantes pour s'assurer de la compat.
- **S75 Phase C** : le backend NLLB-200 worker est un sujet de recherche
  en soi (ctranslate2 vs Ollama vs Transformers.js server-side).
  Recherche specifique necessaire quand le sprint approche.

---

## 9. Evaluation de confiance

| Domaine | Confiance | Raison |
|---|---|---|
| Stack existant (bridge, deploy, blob-serve) | HAUTE | Lu dans le code, teste, 1326 tests |
| Templates scaffolding | HAUTE | Pattern bien compris, recherche externe verifiee |
| Broker architecture | MEDIUM | Design doc interne solide, pas de code existant |
| SBFB.json v2 spec | MEDIUM | Design propose, pas encore confronte au code deploy.rs |
| Preview sandbox | HAUTE | Utilise exactement blob-serve existant |
| NLLB-200 browser | HAUTE | Transformers.js verifie, mais BASSE pour iframe sandbox |
| NLLB-200 via worker | MEDIUM | Pattern Ollama existe, NLLB pas encore integre |
| Domain packs | BASSE | Concept nouveau, pas de precedent interne |
| Babel app | MEDIUM | Research doc exhaustif, pas de code |

---

## 10. Questions ouvertes (de la recherche precedente, repriorisees)

1. **Factory repo ou module daemon ?** — Recommandation : module daemon
   d'abord (crate `nexus-factory` dans le workspace), extraction en repo
   separe quand la surface API est stabilisee.

2. **Premier template : HTML ou React ?** — Recommandation : les DEUX.
   `static-minimal` (HTML pur) et `react-vite`. Le HTML pur est le cas
   simple, React est le cas reel. Ne pas choisir un seul.

3. **Copier ou generateur Rust ?** — Recommandation : generateur Rust
   pour le MVP (3-5 templates, <10 variables). Copier si le nombre de
   templates depasse 10 et que des hooks complexes deviennent necessaires.

4. **Broker Rust ou Python ?** — Recommandation : Rust. Le daemon est
   Rust. Ajouter un broker Python cree une dependance runtime inutile.

5. **Format provenance Factory ?** — Recommandation : format SBFB
   (`factory.provenance.json`) avec mapping in-toto possible plus tard.
   Ne pas imposer in-toto au MVP.

6. **Premier dogfood Babel : reader, ingestion ou traduction ?** —
   Recommandation : reader + storage d'abord. La traduction (task_submit)
   est un stretch goal. L'ingestion corpus est post-S75.

---

## 11. Sources

### Sources internes (repo)

- `examples/sbfb-explorer/` — app reference complexe
- `examples/sbfb-ideas/` — app reference storage P2P
- `web/public/sbfb-bridge.js` — SDK bridge canonique (398 lignes)
- `web/src/bridge/protocol.ts` — schemas Zod bridge
- `crates/nexus-shell-daemon/src/deploy.rs` — pipeline deploy (753 lignes)
- `crates/nexus-coordinator-rs/src/provenance.rs` — provenance SLSA L1
- `docs/apps/REACT_MIGRATION.md` — guide migration React
- `docs/architecture/PUBLISH_MODEL.md` — 4 etats publication
- `.planning/research/babel_translation_protocol.md` — research Babel
- `.planning/research/sbfb_project_factory_rrv_oss_research.md` — research Factory/RRV
- `.planning/codebase/APPS_BRIDGE_DOCS.md` — cartographie apps/bridge
- `.planning/codebase/frontend_architecture.md` — architecture shell React
- `.planning/codebase/protocol_wire_formats.md` — wire formats

### Sources externes

- [giget (UnJS)](https://github.com/unjs/giget) — template downloading, ~3M DL/semaine
- [giget vs degit vs tiged 2026](https://www.pkgpulse.com/blog/giget-vs-degit-vs-tiged-git-template-downloading-nodejs-2026) — comparaison actualisee
- [degit (Rich Harris)](https://github.com/Rich-Harris/degit) — scaffolding original, non maintenu
- [Copier docs](https://copier.readthedocs.io/) — template generation Python
- [Cookiecutter](https://github.com/cookiecutter/cookiecutter) — scaffolding Python mature
- [create-vite](https://vite.dev/guide/) — scaffolding officiel Vite
- [Nx generators](https://nx.dev/extending-nx/recipes/local-generators) — code generation workspace
- [Sandpack (CodeSandbox)](https://sandpack.codesandbox.io/) — browser code sandbox React
- [WebContainer API (StackBlitz)](https://developer.stackblitz.com/platform/api/webcontainer-api) — Node.js dans browser
- [bolt.new](https://github.com/stackblitz/bolt.new) — AI app builder + preview
- [Flatpak sandbox permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html) — portal broker pattern
- [VS Code extension security](https://code.visualstudio.com/docs/configure/extensions/extension-runtime-security) — signature verification model
- [VS Code marketplace trust vulnerabilities (OX Security 2025)](https://www.ox.security/blog/can-you-trust-that-verified-symbol-exploiting-ide-extensions-is-easier-than-it-should-be/) — limites du modele verification
- [Obsidian plugin security 2025-2026](https://obsidian.md/blog/future-of-plugins/) — automated review + safety scorecard
- [Home Assistant quality scale](https://developers.home-assistant.io/docs/core/integration-quality-scale/) — tiers qualite integrations
- [Transformers.js NLLB-200](https://huggingface.co/Xenova/nllb-200-distilled-600M) — NLLB dans browser via ONNX
- [NLLB-200 browser translator](https://drlee.io/break-language-barriers-in-your-browser-build-a-real-time-translation-app-using-nllb-200-and-8dfa57356a6f) — implementation reference
- [Pyodide WASM 2026](https://glinteco.com/en/post/beyond-the-server-running-high-performance-python-in-the-browser-with-pyodide-and-webassembly-2026-guide/) — Python dans browser
- [AI code verification at scale (OpenAI)](https://alignment.openai.com/scaling-code-verification/) — verification automatisee code
- [Factory.ai guide](https://sidbharath.com/blog/factory-ai-guide/) — trust levels AI code generation
- [Developer trust in AI code 2025](https://edmondscommerce.co.uk/research/ai/developer-trust/) — 33% seulement font confiance
