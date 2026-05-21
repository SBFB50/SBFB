# SBFB Project Factory, RRV local-first, and OSS reuse research

**Date:** 2026-05-17
**Status:** recherche produit/architecture, non engagee en sprint.
Recadrage 2026-05-21 : ce document reste utile pour la vision
Project Factory/RRV `@dev`, mais il ne pilote plus S67-S69. Gate 1
se valide sur `@protocole` + Proof Cards + publish + Babel dogfood ;
`@dev` LocalOnly/source-only est S70+ par defaut.
**Scope:** SBFB Project Factory, RRV `@dev` LocalOnly, Babel dogfood,
open-source building blocks, broker/sandbox, sprint-system generation
**Related docs:**
- `.planning/research/chat_ia_reseau_recherche_reseau_rnd.md`
- `.planning/research/rrv_scoped_search_compute_groups.md`
- `.planning/research/iroh_no_internet_babel_anti_censure.md`
- `.planning/research/babel_translation_protocol.md`
- `docs/apps/GENERATION_COMPOSEE.md`
- `docs/affine-sbfb/04_BABEL_SUR_SBFB.md`
- `docs/architecture/PUBLISH_MODEL.md`
- `docs/architecture/SELF_HOSTED_BUILD.md`
- `docs/agent/PROCESS.md`

---

## 0. Verdict court

Ne pas attendre le RRV complet.

La solution la plus propre est:

```text
1. Construire SBFB Project Factory comme projet/repo separe.
2. Dogfood sur Babel via Factory et les preuves `@protocole`.
3. Ajouter RRV @dev LocalOnly apres le pilote ou en stretch zero-impact.
4. Generer des repos applicatifs avec le process sprint visible.
5. Brancher plus tard le RRV reseau quand SearchManifest/feed/proof labels
   sont suffisamment solides.
```

Ce n'est pas un raccourci sale. C'est le chemin le plus rigoureux, parce que
Project Factory est le meilleur cas d'usage concret pour developper RRV sans
sur-vendre une recherche reseau qui n'existe pas encore.

Decision forte:

```text
Project Factory n'est pas une app iframe toute-puissante.
Project Factory est une UI + broker local + workspace sandbox + index @dev.
L'iframe reste non privilegiee.
Le protocole SBFB reste neutre.
Les apps comme Babel restent des repos separes connectes au protocole.
```

---

## 1. Faits repo actuels

### 1.1 Il n'existe pas encore de code Project Factory

Recherche locale:

```text
rg --files | rg -i "babel|gutenberg|generation|factory|rrv|project.factory|project_factory"
```

Resultats pertinents:

```text
crates/nexus-worker-core/src/llm/factory.rs
docs/apps/GENERATION_COMPOSEE.md
docs/affine-sbfb/04_BABEL_SUR_SBFB.md
```

Interpretation:

- `llm/factory.rs` est une factory backend LLM, pas Project Factory.
- `GENERATION_COMPOSEE.md` est design/pre-implementation.
- Babel existe en docs/research, pas comme app code livree dans le repo.

Donc Project Factory doit etre concu comme nouveau projet ou nouveau module,
pas comme une feature deja partiellement codee.

### 1.2 Ce qui existe deja et doit etre reutilise

| Surface repo | Etat | Utilite pour Project Factory |
|---|---:|---|
| `docs/agent/PROCESS.md` | prod process | contrat sprint vendor-neutral |
| `scripts/agent/agentctl.py` | prod tool | contexte, prompts, gates, auditor-gate |
| `prompts/agent/universal.md` | prod prompt | handoff deep pour agents et phases |
| `web/src/bridge/protocol.ts` | prod bridge | methodes iframe -> host whitelistees |
| `web/src/bridge/useBridge.ts` | prod dispatch | host-side validation et handlers |
| `web/src/pages/BrowsedProject.tsx` | prod iframe | sandbox `allow-scripts` pour apps |
| `docs/architecture/PUBLISH_MODEL.md` | trust model | Local Draft / Unverified / Verified / Stale |
| `docs/architecture/SELF_HOSTED_BUILD.md` | build trust | build sandbox + quorum + SLSA |
| `docs/protocol/PUBLIC_FEED_SPEC.md` | feed trust | feed public append-only et provenance |
| `docs/apps/GENERATION_COMPOSEE.md` | design | vision composee app generation |
| `docs/affine-sbfb/04_BABEL_SUR_SBFB.md` | design | Babel comme app vitrine |

### 1.3 Etat worktree observe pendant cette recherche

```text
 M crates/nexus-coordinator-rs/src/public_feed.rs
?? .planning/active/sprint64_phase_C_preflight.md
?? .planning/research/iroh_no_internet_babel_anti_censure.md
?? .planning/research/rrv_scoped_search_compute_groups.md
```

Implication:

- ne pas melanger cette research avec une phase code Sprint 64;
- ne pas supposer que les deux research non trackees sont canoniques tant
  qu'elles ne sont pas committees;
- si Project Factory devient sprint, commencer par kickoff/preflight dedies.

---

## 2. Distinction protocole / factory / app / runtime

Le point architectural critique est de respecter la separation defendue dans
les discussions:

```text
Le protocole est une application neutre en elle-meme.
Les applications sont des repos separes que l'on connecte au protocole.
```

### 2.1 Ce qui appartient au protocole SBFB/Nexus

Seulement des primitives generiques:

- lecture de blobs/docs par hash, ticket ou namespace;
- soumission de tasks generiques;
- provenance generique;
- catalogue/feed generique;
- app storage multi-app;
- capability model;
- verification de publication;
- tests d'integration avec fixture app.

Le protocole ne doit pas connaitre "Babel" comme logique metier.
Le protocole ne doit pas connaitre "Project Factory" comme metier non plus,
sauf via primitives generiques qu'une factory peut appeler.

### 2.2 Ce qui appartient a SBFB Project Factory

Project Factory doit etre un repo/projet dedie:

- UI de creation;
- broker local privilegie;
- templates de projets;
- moteur d'index local `@dev`;
- sandbox de workspace;
- preview sandbox;
- generation de `.planning/`, `AGENTS.md`, hooks, prompts;
- connecteurs vers les primitives du protocole;
- dogfood Babel et autres apps.

Project Factory peut etre une app SBFB du point de vue UX, mais son autorite
reelle doit rester dans un broker local controle par le shell/daemon.

### 2.3 Ce qui appartient a Babel

Babel doit rester un repo applicatif:

- corpus;
- reader;
- traduction;
- validation humaine;
- Babel Shelf;
- manifests de sources;
- workflows OCR/traduction;
- UI metier.

Babel consomme les primitives SBFB:

- `task_submit` pour traduction/indexation/validation;
- app storage pour bibliotheque locale et etat;
- provenance pour sources, chunks, drafts, revues;
- blobs/docs pour corpus et manifests;
- feed/catalogue pour publication et decouverte.

### 2.4 Ce qui appartient a `.sbfb/apps/babel`

`C:\Users\FlowUP\.sbfb\apps\babel` doit etre runtime/cache, pas repo source.

Usage correct:

- app installee;
- cache de blobs;
- etat local;
- DB locale runtime;
- artefacts telecharges;
- donnees utilisateur.

Usage incorrect:

- developper le code source principal de Babel;
- stocker le process sprint canonique;
- faire de ce dossier le repo de reference;
- y mettre des secrets ou outils privilegies.

---

## 3. Decision "attendre RRV complet ?"

### 3.1 Ne pas attendre pour le noyau local

Attendre le RRV complet cree un probleme circulaire:

```text
RRV complet a besoin de vrais objets, vrais manifests, vrais usages.
Project Factory fournit justement ces objets et ces usages.
```

Le bon ordre:

```text
Project Factory local-first -> Babel dogfood @protocole
-> RRV @dev LocalOnly/source-only -> SearchManifest -> RRV network
-> Generation Composee reseau.
```

### 3.2 Attendre seulement pour les promesses reseau

Il faut attendre avant de promettre:

- recherche globale reseau par defaut;
- ranking reseau fiable;
- `Verified by workers` automatique;
- discovery de shards publics;
- reputation complexe;
- generation composee multi-repo reseau sans proof cards.

Ce qui peut etre fait maintenant, hors Gate 1 ou en stretch non bloquant:

- recherche locale verifiable;
- citations fichier/ligne/hash/commit;
- index de code/docs/manifests;
- proof cards locales;
- templates de repo;
- broker sandbox;
- preview iframe;
- publication Local Draft -> Unverified -> Verified;
- Babel comme premier projet cree via Factory, mais le code produit Babel
  reste du ressort du dogfood utilisateur tant que le protocole/Factory
  servent correctement create/publish/proof.

---

## 4. Architecture cible

### 4.1 Vue logique

```text
User
  |
  v
Factory UI
  - chat/action surface
  - template selector
  - diff preview
  - proof cards
  - preview frame
  |
  v
Factory Broker local
  - capability checks
  - FS/git/shell gate
  - dependency policy
  - sandbox orchestration
  - audit log
  - protocol connector
  |
  +--> Workspace Sandbox
  |      - repo worktree
  |      - devcontainer
  |      - build/test
  |      - no secrets
  |
  +--> Local RRV Index @dev
  |      - files/docs/manifests
  |      - AST/symbols
  |      - capabilities
  |      - risks/tests
  |
  +--> Preview Sandbox
  |      - static app iframe
  |      - same SBFB CSP model
  |
  +--> SBFB Protocol
         - blobs/docs
         - tasks
         - provenance
         - feed/catalogue
         - app storage
```

### 4.2 Les quatre zones de securite

| Zone | Autorite | Role | Regle |
|---|---:|---|---|
| Factory UI | faible | interaction utilisateur | jamais FS/git/shell direct |
| Factory Broker | forte locale | autorisation et execution | allowlist + audit + confirmations |
| Workspace Sandbox | moyenne bornee | codegen/build/test | pas de secrets, FS borne, reseau limite |
| Preview Sandbox | faible | tester l'app produite | meme isolation qu'une app publiee |

### 4.3 Flux "creer Babel"

```text
1. User: "cree Babel, bibliotheque P2P traduction + validation humaine"
2. Factory UI cree une intention.
3. Broker selectionne template "sbfb-app".
4. RRV @dev cherche les patterns/protocol docs locaux utiles.
5. Broker genere un plan et un diff virtuel.
6. User valide les capabilities:
   - fs.write_project
   - git.local
   - shell.build_test
   - deps.install si necessaire
7. Workspace sandbox cree le repo Babel.
8. Tests/build/lint passent dans sandbox.
9. Preview iframe sert l'app.
10. Provenance locale Factory est ecrite.
11. Publication:
    - Local Draft pour dev
    - Unverified si zip direct
    - Verified Release via deploy-from-repo
```

### 4.4 Flux "poser une question @dev"

```text
@dev @Babel ou est geree la validation des chunks ?
```

Pipeline:

```text
scope parser
  -> privacy gate
  -> local index query
  -> lexical + vector + AST ranking
  -> proof card builder
  -> answer with citations
  -> action suggestions
```

Resultat attendu:

```text
La validation des chunks est decrite dans:
- docs/affine-sbfb/04_BABEL_SUR_SBFB.md:45
- .planning/research/babel_translation_protocol.md:<ligne>

Preuve:
- source locale
- commit courant
- file hash
- no network query

Actions:
- ouvrir fichier
- creer task "schema validation chunks"
- generer test fixture
```

---

## 5. RRV `@dev` LocalOnly

### 5.1 Objectif

Le premier RRV ne doit pas etre un moteur global. Il doit etre un moteur local
de recherche-action verifiable.

Objectif utilisateur:

```text
Je peux demander ou se trouve une capacite, quelle preuve l'app a, quel code
expose quel bridge, quels risques existent, et quoi faire ensuite.
```

### 5.2 Scopes supportes au MVP

| Scope | MVP | Source |
|---|---:|---|
| `@current` | oui | app/projet courant |
| `@Babel` | oui si repo/index existe | repo Babel |
| `@dev` | oui | code/docs/manifests/tests |
| `@network` | non MVP | attend SearchManifest/feed |
| `@web` | sidecar optionnel | SearXNG ou recherche externe |
| `@private:<group>` | non MVP | attend groupes prives |

### 5.3 Objets indexes

| Objet | Champs minimaux |
|---|---|
| `Project` | id, path, repo_url, commit_sha, template_id, status |
| `Release` | artifact_hash, provenance_hash, source_state, build_state |
| `SourceFile` | path, language, size, content_hash, last_seen |
| `CodeChunk` | file_id, range, text_hash, text, symbols, embedding_ref |
| `Symbol` | name, kind, file_id, range, exported, references |
| `Capability` | method/name, direction, schema_ref, source_file, risk |
| `PermissionSurface` | fs/git/shell/network/deps/publish flags |
| `RiskFinding` | tool, rule, severity, file, range, status |
| `LineageEdge` | source_project, target_project, source_chunk, reason |
| `SearchManifest` | future public summary, signed, opt-in |

### 5.4 Stockage MVP

MVP recommande:

```text
search_index.sqlite
  - FTS5 pour lexical
  - tables metadata/proof
  - sqlite-vec optionnel derriere feature flag
```

Pourquoi pas vector DB serveur au debut:

- trop lourd;
- moins local-first;
- ajoute de l'ops;
- complique les tests;
- ne resout pas la preuve.

Evolution:

```text
MVP: SQLite FTS5 + metadata proof
MVP+: sqlite-vec optional
Mid: tree-sitter symbols
Mid+: Tantivy si index code local devient gros
Later: SearchManifest + shards reseau opt-in
```

### 5.5 Schema conceptuel

```sql
CREATE TABLE projects (
  project_id TEXT PRIMARY KEY,
  path TEXT NOT NULL,
  repo_url TEXT,
  commit_sha TEXT,
  template_id TEXT,
  template_version TEXT,
  created_at INTEGER NOT NULL
);

CREATE TABLE files (
  file_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  path TEXT NOT NULL,
  language TEXT,
  content_hash TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  indexed_at INTEGER NOT NULL
);

CREATE VIRTUAL TABLE chunks_fts USING fts5(
  chunk_id UNINDEXED,
  project_id UNINDEXED,
  file_id UNINDEXED,
  path,
  language,
  text,
  tokenize = 'unicode61'
);

CREATE TABLE chunks (
  chunk_id TEXT PRIMARY KEY,
  file_id TEXT NOT NULL,
  start_line INTEGER NOT NULL,
  end_line INTEGER NOT NULL,
  text_hash TEXT NOT NULL,
  proof_json TEXT NOT NULL
);

CREATE TABLE capabilities (
  capability_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  schema_ref TEXT,
  file_id TEXT,
  start_line INTEGER,
  risk_level TEXT NOT NULL
);

CREATE TABLE risk_findings (
  finding_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  tool TEXT NOT NULL,
  severity TEXT NOT NULL,
  rule_id TEXT NOT NULL,
  file_id TEXT,
  start_line INTEGER,
  status TEXT NOT NULL
);
```

### 5.6 Ranking initial

```text
score =
  0.35 * bm25
+ 0.30 * vector
+ 0.15 * proof
+ 0.10 * availability
+ 0.10 * safety
```

Pour un MVP sans embeddings:

```text
score =
  0.55 * bm25
+ 0.20 * path/symbol match
+ 0.15 * proof
+ 0.10 * safety
```

Important:

- le score ne doit jamais transformer une source web en preuve SBFB;
- `@web` peut influencer pertinence, pas niveau de confiance;
- les resultats doivent afficher leur label de preuve.

### 5.7 Labels de preuve

| Label | Sens |
|---|---|
| `Local indexed` | lu dans workspace local |
| `Local generated` | cree par Factory, non publie |
| `Local tested` | build/test local passe |
| `SBFB verified` | provenance + artifact + source OK |
| `SBFB unverified` | vu/publie sans chaine complete |
| `SBFB stale` | provenance existe mais source non reverifiable |
| `Web external` | source externe non verifiee SBFB |
| `Web claim` | affirmation web non prouvee |
| `Verified by workers` | verification active executee par workers |

### 5.8 Actions RRV MVP

- ouvrir fichier;
- inspecter preuve;
- utiliser comme source;
- forker;
- generer draft Factory;
- lancer audit local;
- lancer build/test;
- creer issue/tache;
- preparer publication;
- demander verification reseau plus tard.

---

## 6. Open-source building blocks

### 6.1 P0: a reutiliser ou adapter en priorite

| Brique | Projet | Source | Role Factory | Decision |
|---|---|---|---|---|
| Templates | Copier | <https://copier.readthedocs.io/> | generation repo versionnable | P0 moteur concret |
| Catalog model | Backstage Catalog | <https://backstage.io/docs/features/software-catalog/> | metadata/catalogue projet | P0 inspiration modele |
| Scaffolder model | Backstage Templates | <https://backstage.io/docs/features/software-templates> | steps, parameters, dry-run | P0 inspiration UX/contrats |
| Coding agent ref | OpenHands | <https://github.com/OpenHands/OpenHands> | reference agent + sandbox | P0 reference, pas coeur |
| Dev env | Dev Containers | <https://github.com/devcontainers/spec> | workspace reproductible | P0 standard |
| Local search | SQLite FTS5 | <https://www.sqlite.org/fts5.html> | index lexical local | P0 simple |
| Code parse | tree-sitter | <https://github.com/tree-sitter/tree-sitter> | AST/symbols | P0 pour @dev |
| SAST | Semgrep CE | <https://semgrep.dev/docs/introduction> | scan rapide | P0 local |
| SBOM | Syft | <https://github.com/anchore/syft> | SBOM artifacts | P0 provenance |
| Scan | Trivy | <https://github.com/aquasecurity/trivy> | vuln/misconfig/secrets | P0 mais pin strict |
| Provenance | in-toto | <https://github.com/in-toto/in-toto> | attestation chain | P0 vocabulary/format |
| Test preview | Playwright | <https://playwright.dev/docs/intro> | e2e preview | P0 |
| CI | Woodpecker | <https://woodpecker-ci.org/> | self-hosted CI aligned SBFB | P0 |

### 6.2 P1: utiles mais pas coeur MVP

| Brique | Projet | Source | Role | Decision |
|---|---|---|---|---|
| Templates alt | Cookiecutter | <https://cookiecutter.readthedocs.io/> | fallback templates | P1 |
| Generator JS | Plop | <https://plopjs.com/documentation/> | micro-generators | P1 |
| Search Rust | Tantivy | <https://docs.rs/tantivy/> | gros index full-text | P1 apres SQLite |
| Vector local | sqlite-vec | <https://github.com/asg017/sqlite-vec> | embeddings locaux | P1 feature flag |
| Code search ref | Zoekt | <https://github.com/sourcegraph/zoekt> | symbol/path ranking | P1 inspiration |
| Semantic SAST | CodeQL | <https://codeql.github.com/> | analyse profonde | P1 lourd |
| Pipelines | Dagger | <https://docs.dagger.io/> | build/test portable | P1 |
| GHA local | nektos/act | <https://github.com/nektos/act> | tester workflows GH | P1 si GitHub |
| Policy | OPA | <https://www.openpolicyagent.org/> | policies capabilities | P1 si besoin |
| Config schema | CUE | <https://cuelang.org/> | validation templates | P1 si complexite |
| Signing | cosign | <https://github.com/sigstore/cosign> | signature artifacts | P1 |
| Web sidecar | SearXNG | <https://docs.searxng.org/> | `@web` privacy sidecar | P1, jamais trust core |

### 6.3 P2: references UX/long terme

| Brique | Projet | Role | Decision |
|---|---|---|---|
| Browser AI app builder | bolt.diy | UX generation app | reference UX seulement |
| Local app builder | Dyad | UX app generation | reference UX seulement |
| SWE benchmark agents | SWE-agent | agent research | reference, pas core |
| IDE assistant | Continue | coding assistant | option user, pas Factory core |
| microVM | Firecracker | isolation forte Linux/KVM | long terme |
| sandboxed container | gVisor | isolation conteneur Linux | long terme |
| bubblewrap | bubblewrap | sandbox Linux desktop | utile Linux, pas Windows-first |

### 6.4 Anti-decisions

Ne pas faire:

- mettre LlamaIndex/Haystack au coeur du protocole;
- imposer Qdrant/Weaviate/Elasticsearch pour MVP local;
- faire confiance a des embeddings fournis par peers;
- donner FS/git/shell directement a l'iframe;
- faire du web crawling automatique;
- melanger `Web external` et `SBFB verified`;
- publier automatiquement une app generee sans gate humain;
- faire de Project Factory une dependance runtime obligatoire de Babel.

---

## 7. Securite: iframe, broker, sandbox

### 7.1 Regle principale

```text
L'iframe est une interface. Le broker est l'autorite.
```

Le code app/iframe peut demander:

- afficher un plan;
- demander une generation;
- demander une preview;
- demander une publication.

Il ne peut pas:

- lire arbitrairement le FS;
- ecrire hors workspace;
- lancer shell;
- modifier git;
- installer deps;
- lire secrets;
- acceder au reseau host directement.

### 7.2 Capabilities minimales

| Capability | Sens | Gate |
|---|---|---|
| `fs.read_project` | lire workspace borne | auto si projet ouvert |
| `fs.write_project` | ecrire workspace borne | diff preview + confirmation |
| `git.local` | init/commit/branch local | confirmation |
| `shell.build_test` | lancer commandes allowlistees | sandbox + logs |
| `deps.install` | installer deps | lockfile + registry allowlist |
| `preview.static` | servir preview | sandbox |
| `publish.unverified` | publier zip direct | warning |
| `publish.verified` | deploy-from-repo | provenance complete |
| `network.web_search` | chercher web | consentement explicite |

### 7.3 Gates de securite Factory

| Gate | Trigger | Exit condition |
|---|---|---|
| G0 Classification | nouvelle action | action classee read/write/build/publish |
| G1 Scope | projet selectionne | path resolu dans workspace autorise |
| G2 Diff preview | write | diff affiche + utilisateur valide |
| G3 Shell | build/test | commande allowlistee + sandbox creee |
| G4 Dependencies | lockfile change | registry allowlist + scripts controles |
| G5 Secrets | avant build/publish | secret scan PASS ou acknowledge bloque |
| G6 Preview | app runnable | iframe sandbox + CSP + Playwright smoke |
| G7 Provenance locale | generation terminee | factory.provenance.json ecrit |
| G8 Git | commit/release | commit local + status connu |
| G9 Publish | release | Local Draft/Unverified/Verified explicite |
| G10 Protocol | bridge/schema change | preflight S4 + review PASS |

### 7.4 Supply chain

Regles:

- lockfiles obligatoires;
- `npm ci --ignore-scripts` en premiere passe quand possible;
- scripts postinstall seulement apres review;
- registry allowlist;
- SBOM Syft;
- scan Trivy/Semgrep;
- provenance in-toto/SLSA-like;
- cache content-addressed;
- zip traversal checks;
- build network-off apres fetch quand possible;
- pins de versions pour outils de securite eux-memes.

Note Trivy:

```text
Trivy reste utile, mais les outils de securite sont aussi une surface
d'attaque. Pin strict, verification d'artefacts, et defense en couches.
```

---

## 8. Templates Project Factory

### 8.1 Template pack minimal

```text
templates/
  sbfb-app-static/
    copier.yml
    template/
      AGENTS.md
      README.md
      SBFB.json
      package.json
      web/
      docs/
      .planning/active/.gitkeep
      docs/agent/PROCESS.md
      scripts/agent/agentctl.py
      prompts/agent/
      .githooks/
```

Le template doit generer:

- repo lisible sans Factory;
- process sprint complet;
- commandes test/build;
- manifest SBFB;
- bridge SDK usage;
- preview smoke test;
- provenance seed.

### 8.2 Artefacts generes par defaut

| Artefact | Role |
|---|---|
| `AGENTS.md` | instructions agent repo-locales |
| `docs/agent/PROCESS.md` | process vendor-neutral |
| `scripts/agent/agentctl.py` | gates et handoff |
| `.planning/active/` | kickoff/plan/preflight/review |
| `.githooks/` | precommit/auditor gate |
| `SBFB.json` | app manifest |
| `factory.project.json` | metadata Factory |
| `factory.template.lock` | template id/version/hash |
| `factory.provenance.json` | generation lineage |
| `tests/smoke/` | smoke minimal |

### 8.3 Exemple `factory.project.json`

```json
{
  "schema_version": 1,
  "project_id": "babel",
  "kind": "sbfb-app",
  "created_by": "sbfb-project-factory",
  "template": {
    "id": "sbfb-app-static",
    "version": "0.1.0",
    "source": "local",
    "content_hash": "blake3:..."
  },
  "repo": {
    "path": "C:/Users/FlowUP/Documents/Code/babel",
    "initial_commit": null
  },
  "protocol": {
    "requires_bridge": true,
    "requires_tasks": true,
    "requires_storage": true,
    "requires_provenance": true
  }
}
```

### 8.4 Exemple `factory.provenance.json`

```json
{
  "schema_version": 1,
  "project_id": "babel",
  "created_at": "2026-05-17T00:00:00Z",
  "intent": "Bibliotheque P2P anti-censure avec traduction IA et validation humaine",
  "inputs": {
    "prompt_hash": "blake3:...",
    "template_hash": "blake3:..."
  },
  "sources": [
    {
      "kind": "repo_doc",
      "path": "docs/affine-sbfb/04_BABEL_SUR_SBFB.md",
      "content_hash": "blake3:..."
    },
    {
      "kind": "repo_doc",
      "path": ".planning/research/rrv_scoped_search_compute_groups.md",
      "content_hash": "blake3:..."
    }
  ],
  "outputs": [
    {
      "path": "SBFB.json",
      "content_hash": "blake3:..."
    }
  ],
  "tools": [
    {
      "name": "copier",
      "version": "pinned"
    }
  ]
}
```

---

## 9. Babel dogfood

### 9.1 Pourquoi Babel est le bon premier projet

Babel force les vrais besoins:

- corpus;
- provenance;
- traduction IA;
- validation humaine;
- lecture offline;
- sync locale;
- compute batch;
- recherche par langue/source/chunk;
- partage anti-censure;
- UX non technique.

Si Factory sait creer Babel proprement, elle valide:

- template SBFB app;
- bridge;
- storage;
- tasks;
- provenance;
- preview;
- publication;
- RRV `@dev`.

### 9.2 Premier scope Babel via Factory

Ne pas commencer par tout Babel.

MVP Babel cree par Factory:

```text
Babel reader local
  - liste de textes fixtures
  - manifest source
  - page lecture
  - stockage progression locale
  - provenance visible
  - task_submit mock ou local pour traduction chunk
  - validation humaine simple: accept/reject/comment
```

Ensuite:

```text
Babel corpus pipeline
  - ingestion Gutenberg/Wikisource fixture
  - source manifest
  - chunking
  - translation task
  - review queue
  - published translation manifest
```

### 9.3 Gutenberg dans cette recherche

Gutenberg est un bon corpus de depart parce qu'il donne:

- textes accessibles;
- metadonnees;
- formats simples;
- volume suffisant pour pipeline.

Mais la Factory ne doit pas coder "Gutenberg" dans le protocole. Elle doit
generer un pipeline de sources generique:

```text
SourceAdapter
  -> SourceManifest
  -> ContentChunks
  -> TranslationTasks
  -> ReviewQueue
  -> PublishedText
```

---

## 10. Roadmap R&D proposee

Cette sequence ne remplace pas un kickoff. Elle traduit la decision en
travail decoupable.

### Phase PF-0 - Spec et repo boundary

Livrables:

- `docs/factory/PROJECT_FACTORY_SPEC.md` ou research promue en spec;
- decision repo separe vs module temporaire;
- threat model Factory;
- matrice OSS;
- template pack design;
- definition capabilities.

Exit:

- aucun code privilegie sans broker;
- protocole/app/runtime boundary validee.

### Phase PF-1 - Template sprint app

Livrables:

- template Copier `sbfb-app-static`;
- `factory.project.json`;
- `factory.template.lock`;
- generation `AGENTS.md`, `.planning/`, `agentctl.py`;
- fixture app minimale;
- tests de generation.

Exit:

- `sbfb factory init demo-app` produit un repo testable;
- `agentctl context` fonctionne dans le repo genere.

### Phase PF-2 - RRV `@dev` LocalOnly

Livrables:

- `search_index.sqlite`;
- index docs/code/manifests;
- FTS5;
- proof cards;
- citations fichier/ligne/hash;
- requetes benchmark.

Exit:

- 20 requetes benchmark passent avec citations;
- zero source reseau implicite.

### Phase PF-3 - Broker + workspace sandbox

Livrables:

- broker local;
- capabilities;
- path allowlist;
- shell allowlist;
- diff preview;
- audit log;
- devcontainer support.

Exit:

- aucun write sans diff preview;
- aucun shell sans sandbox + logs;
- tests traversal/path escape.

### Phase PF-4 - Preview + publish path

Livrables:

- preview iframe;
- Playwright smoke;
- Local Draft;
- Unverified warning;
- Verified Release via deploy-from-repo;
- provenance Factory -> publish model.

Exit:

- app generee previewable;
- publication state explicite.

### Phase PF-5 - Babel dogfood

Livrables:

- repo Babel cree via Factory;
- reader fixture;
- source manifest;
- translation task draft;
- review queue minimale;
- provenance visible.

Exit:

- Babel n'est pas code a la main hors Factory;
- les gaps Factory sont listes par dogfood.

### Phase RRV-N - Reseau plus tard

Preconditions:

- public feed stable;
- `SearchManifestPublished` stable;
- proof labels UX;
- anti-spam;
- worker verification quota;
- stale source handling;
- integration Browse/Protocol Explorer.

Livrables:

- SearchManifest local -> signed;
- opt-in publish;
- shard discovery;
- network query separate button;
- active verification by workers.

---

## 11. Tests et preuves attendus

### 11.1 Tests Factory

| Test | Pourquoi |
|---|---|
| template generation snapshot | eviter drift template |
| generated repo `agentctl context` | process portable |
| path traversal denied | securite FS |
| shell command allowlist denied | securite shell |
| diff preview required | controle humain |
| lockfile change gate | supply chain |
| preview iframe smoke | UX/runtime |
| provenance file hash stable | preuve |

### 11.2 Tests RRV

| Test | Pourquoi |
|---|---|
| FTS query returns file/line | citation minimale |
| changed file reindexed | index incremental |
| deleted file removed | no stale local |
| capability extracted | mode dev utile |
| risk finding indexed | audit |
| `@web` off by default | privacy |
| `@network` unavailable label | honest UX |
| proof label preserved | no trust mixing |

### 11.3 Tests Babel dogfood

| Test | Pourquoi |
|---|---|
| source manifest loads | provenance source |
| reader opens fixture | UX de base |
| storage saves progress | app storage |
| task draft generated | compute path |
| review accept/reject | validation humaine |
| provenance graph visible | trust UX |

---

## 12. Integration avec le protocole existant

### 12.1 Bridge generique a terme

Bridge actuel expose deja des methodes utiles:

- `task_submit`;
- `storage_get`;
- `storage_set`;
- `storage_list`;
- `storage_delete`;
- `storage_version`;
- `provenance_get`;
- `provenance_verify`;
- `feed_cursor_get`.

Pour Project Factory/Babel, les ajouts eventuels doivent rester generiques:

- lire un blob/doc par identifiant;
- lister documents app-scoped;
- soumettre une task typee;
- lire provenance;
- verifier une release;
- obtenir feed/catalogue;
- lire/ecrire app storage.

Ne pas ajouter:

- `babel_translate`;
- `factory_shell_exec` cote iframe;
- `factory_write_file` sans broker;
- methodes metier dans le protocole core.

### 12.2 App storage multi-app

Besoin:

```text
Chaque app a son namespace.
Factory peut creer et tester des apps.
Babel peut stocker corpus/progression/reviews.
Le protocole doit isoler les namespaces et rendre la provenance auditable.
```

Contrat attendu:

- namespace par app id;
- storage version;
- list/get/set/delete;
- quotas;
- event/polling;
- no cross-app read by default;
- migration/versioning.

### 12.3 Catalogue/feed generique

Le feed doit pouvoir annoncer:

- release publiee;
- source stale;
- provenance;
- SearchManifest plus tard;
- capability summary plus tard.

Project Factory consomme ces signaux mais ne les possede pas.

---

## 13. Risques principaux

| Risque | Impact | Mitigation |
|---|---|---|
| Factory devient protocole metier | couplage long terme | repo separe + primitives generiques |
| iframe gagne FS/shell | privilege escalation | broker seul autoritaire |
| templates non versionnes | impossible a auditer | `factory.template.lock` |
| generation sans provenance | code opaque | `factory.provenance.json` obligatoire |
| web externe melange a SBFB | fausse confiance | proof labels non fusionnes |
| RRV reseau trop tot | produit mensonger | LocalOnly d'abord |
| deps install non controle | supply-chain | lockfile, allowlist, scans |
| Babel trop large MVP | blocage | reader + fixture + review minimal |
| `.sbfb/apps` devient repo | confusion source/runtime | repo source dans `Documents/Code` |
| process sprint cache dans modele | non portable | generer process en fichiers |

---

## 14. Questions ouvertes a trancher avant sprint

1. Project Factory doit-il etre un repo sibling de `nexus` ou un module
   temporaire sous `tools/project-factory` avant extraction ?
2. Le premier template doit-il etre Vite static app, React, ou HTML minimal ?
3. Copier doit-il etre appele comme binaire externe ou via bibliotheque
   Python dans un broker ?
4. Le broker doit-il etre Rust des le debut ou Python/Rust hybride pour MVP ?
5. Le RRV local doit-il etre dans le broker Factory ou dans une crate
   generique reutilisable par Nexus ?
6. Les embeddings doivent-ils etre absents du MVP ou derriere Ollama local ?
7. Quel format exact pour `factory.provenance.json` : in-toto direct,
   SLSA-like minimal, ou format SBFB puis mapping in-toto ?
8. Quel niveau de confirmation utilisateur pour `deps.install` ?
9. Comment integrer les hooks sprint dans un repo genere sans copier du code
   stale depuis `nexus` ?
10. Quel premier dogfood Babel: reader only, ingestion source, ou translation
    task ?

---

## 15. Recommendation finale

La meilleure solution factuelle:

```text
Ne pas attendre RRV complet.
Ne pas coder Project Factory dans Babel.
Ne pas mettre Project Factory comme metier dans le protocole.

Construire un Project Factory repo-visible, local-first, sandboxed,
avec RRV @dev LocalOnly comme noyau.

Promouvoir uniquement les primitives generiques dans Nexus/protocole quand
elles ont ete validees par Factory + Babel dogfood.
```

Ordre recommande:

```text
1. Spec Project Factory + threat model.
2. Template Factory qui genere un repo SBFB avec process sprint.
3. Preview/publish + Proof Cards @protocole.
4. Babel cree avec Factory par le dogfood utilisateur.
5. RRV @dev LocalOnly/source-only sur le repo genere ou un corpus OSS curate.
6. Broker/sandbox si necessaire au write workflow.
7. SearchManifest/RRV reseau plus tard.
```

La raison:

```text
Project Factory donne a RRV un cas d'usage reel.
RRV local donne a Project Factory des preuves et citations.
Babel donne a Factory un produit non trivial.
Le protocole reste neutre.
```

---

## 16. Sources externes consultees

Templates/catalogue:

- Backstage Software Templates: <https://backstage.io/docs/features/software-templates>
- Backstage Software Catalog: <https://backstage.io/docs/features/software-catalog/>
- Copier docs: <https://copier.readthedocs.io/en/stable/generating/>
- Cookiecutter docs: <https://cookiecutter.readthedocs.io/>
- Plop docs: <https://plopjs.com/documentation/>

Agents/dev automation:

- OpenHands GitHub: <https://github.com/OpenHands/OpenHands>
- Dev Containers spec: <https://github.com/devcontainers/spec>

Search/code intelligence:

- SQLite FTS5: <https://www.sqlite.org/fts5.html>
- sqlite-vec: <https://github.com/asg017/sqlite-vec>
- Tantivy docs: <https://docs.rs/tantivy/>
- Tantivy GitHub: <https://github.com/quickwit-oss/tantivy>
- tree-sitter GitHub: <https://github.com/tree-sitter/tree-sitter>
- Semgrep docs: <https://semgrep.dev/docs/introduction>
- CodeQL docs: <https://codeql.github.com/>
- Zoekt GitHub: <https://github.com/sourcegraph/zoekt>

Supply chain/security:

- in-toto GitHub: <https://github.com/in-toto/in-toto>
- in-toto docs: <https://in-toto.io/docs/>
- SLSA levels: <https://slsa.dev/spec/v1.0/levels>
- Sigstore cosign: <https://github.com/sigstore/cosign>
- Syft GitHub: <https://github.com/anchore/syft>
- Trivy GitHub: <https://github.com/aquasecurity/trivy>
- gVisor docs: <https://gvisor.dev/docs/>
- Firecracker GitHub: <https://github.com/firecracker-microvm/firecracker>

CI/test:

- Woodpecker CI: <https://woodpecker-ci.org/>
- Dagger docs: <https://docs.dagger.io/>
- nektos/act: <https://github.com/nektos/act>
- Playwright docs: <https://playwright.dev/docs/intro>

Web sidecar/P2P references:

- SearXNG docs: <https://docs.searxng.org/>
- iroh blobs: <https://docs.iroh.computer/protocols/blobs>

## 17. Sources repo utilisees

- `AGENTS.md`
- `docs/agent/PROCESS.md`
- `prompts/agent/universal.md`
- `scripts/agent/agentctl.py`
- `docs/apps/GENERATION_COMPOSEE.md`
- `docs/apps/CHAT_IA_RESEAU.md`
- `docs/affine-sbfb/04_BABEL_SUR_SBFB.md`
- `docs/architecture/PUBLISH_MODEL.md`
- `docs/architecture/SELF_HOSTED_BUILD.md`
- `docs/architecture/LAUNCHER.md`
- `docs/protocol/PUBLIC_FEED_SPEC.md`
- `web/src/bridge/protocol.ts`
- `web/src/bridge/useBridge.ts`
- `web/src/pages/BrowsedProject.tsx`
- `.planning/research/chat_ia_reseau_recherche_reseau_rnd.md`
- `.planning/research/rrv_scoped_search_compute_groups.md`
- `.planning/research/iroh_no_internet_babel_anti_censure.md`
