# Sprint 5 kickoff — Frontend shell P2P pour nexus-grid

**À utiliser dans une session Claude fraîche ouverte dans
`C:\Users\FlowUP\Documents\Code\nexus`**. Document self-contained :
tout le contexte nécessaire est ici ou explicitement pointé vers
un fichier à lire en entier. Pas de « cherche ailleurs », pas de
devine, pas de training data pour les API de libs — `context7`
pour toute bibliothèque non-triviale avant d'écrire une ligne
contre elle.

---

## 1. Mission (une phrase)

Livrer un shell React / TypeScript qui parle aux `nexus-coordinator`
locaux de l'utilisateur (un ou plusieurs, un par projet) via leur
API FastAPI, qui permet à l'utilisateur de voir ses projets, son
worker local, le réseau public (si design retenu), et qui héberge
dynamiquement les tabs déclarés par les apps installées via leur
manifest `/app/{name}/manifest`. Sortie : un shell local qui rend
réellement le produit nexus-grid utilisable pour un non-dev
(lance un coordinateur, ouvre le shell, voit le projet, clique un
tab d'app et ça marche), sans jamais toucher au backend legacy
`nexus/main.py` ni aux 14 anciennes pages cold-case.

## 2. État à l'entrée (vérifié 2026-04-10 après Sprint 4)

### Branche et commits

Branche `master`, HEAD attendue : le tip de Sprint 4 tel que
vérifié dans `.planning/sprint4_verification.md` (actuellement
`3b5c162 docs(sprint4): verification prompt for a fresh-context
audit`, 10 commits Sprint 4 au-dessus de `f68d997` Sprint 3
verification). Working tree clean modulo `.planning/audit_sprint2/`
(gitignored, inertes). `.planning/sprint4_verification.md` doit
être relu avant toute action — il liste ce qui est DÉJÀ livré et
utilisable depuis Sprint 4.

### Ce qui existe côté backend P2P (utilisable tel quel)

**Coordinator Python** (`packages/nexus-coordinator/`) — 27 tests
verts, 1 skip Windows POSIX perms :

- `GET /health` — liveness, retourne `{ node_id, doc_id, project_name, state }`
- `GET /project` — métadonnées projet (name, visibility, doc_id,
  author_id, **pas** de ticket complet)
- `POST /tasks/submit` — soumet une `TaskEntry` signée au doc `tasks`
- `GET /tasks/{task_id}` — état local (pending/claimed/completed)
  depuis la table `task_state` SQLite
- `GET /kudos` — ledger hash-chain per-project (entries, total par
  worker, chain integrity booléen)
- `GET /kudos/verify` — `{ valid: bool, first_bad_row: int|null }`
- `POST /invite/create` — génère un invite v2 `nx1v2...` signé
  portant `tasks_doc_ticket`
- `POST /invite/revoke`, `GET /invite/list`
- `GET /app` — liste des apps installées avec `routes`, `workers`,
  `tabs` (compteurs)
- `GET /app/{name}/manifest` — manifest complet + routes +
  workers + tabs descriptor (avec fallback note pour async
  descriptors)
- `/app/{name}{path}` — chaque `@nexus_route(path)` d'une app
  est reachable ici avec les méthodes déclarées

Le coordinateur boote sur `127.0.0.1:8765` par défaut (override
via `--port`) et sert un **seul** projet par process. Un user qui
run 3 projets = 3 coordinateurs simultanés sur 3 ports.

**SDK** (`packages/nexus-sdk/`) — 6 tests verts :

- `NexusApp` ABC + `AppManifest` Pydantic 2
- `@nexus_route(path, methods)`, `@nexus_worker(name, model)`,
  `@nexus_tab(name, icon)` décorateurs
- `ComputeClient(coordinator_url)` pour submit tasks
- `discover_apps()` via `importlib.metadata.entry_points(group="nexus.apps")`

**Apps minimales** :
- `packages/nexus-app-gov/` (3 tests) — 1 route / 1 worker / 1 tab,
  `POLITICAL_CONTRADICTION_PROMPT` déplacé ici depuis `nexus/engine/`
- `examples/hello-world-app/` — 45 LOC, 1 route / 1 worker / 1 tab,
  entry point `nexus.apps.hello`

**Worker Rust** (`crates/nexus-worker/` binary + `nexus-worker-core/`
library) — 94 + 10 tests verts. W9.1 task pump opérationnel
(`Engine::scan_and_execute_tasks`). `--stub-ollama` flag existe
pour les tests headless. **Pas d'API HTTP** sur le worker — seul
le CLI existe (`register`, `start`, `join`, `projects`, `browse`,
`stats`, `config`). Voir §4 décision D3 pour l'impact.

### Ce qui existe côté web/ (à ne surtout pas croire utilisable)

Le répertoire `web/` est la **legacy cold-case frontend** liée au
backend monolithique Python `nexus/main.py` sur port 8000 :

- `web/src/App.tsx` — 14 routes (`/`, `/evidence`, `/entities`,
  `/hypotheses`, `/graph`, `/timeline`, `/investigation`,
  `/suspects`, `/wiki`, `/reports`, `/images`, `/benchmark`,
  `/government`, `/network`)
- `web/src/pages/` — 14 pages cold-case : **aucune** n'est
  portable telle quelle contre un coordinateur nexus-grid.
  Elles hit `http://localhost:8000/api/...` via
  `web/src/api/client.ts` et consomment des endpoints qui
  n'existent pas dans le coordinateur P2P.
- `web/src/components/AppSidebar.tsx` (418 lignes) — sidebar
  avec 4 groupes hardcodés (Investigation, Gouvernement,
  Puissance Citoyenne, Outils), case selector, badge « Cold
  Case Intel », header « NEXUS », stats Neo4j + Chroma. Tout est
  à retirer ou à réécrire.
- `web/src/stores/caseStore.ts` — state pour « cold case
  sélectionné », concept qui n'existe plus
- `web/src/api/compute.ts` + `web/src/api/government.ts` — clients
  HTTP pour `nexus/api/*` legacy, à supprimer
- `web/src/components/gov/`, `web/src/components/compute/`,
  `InvestigationMap.tsx`, `InvestigationTimeline.tsx`,
  `PipelineTools.tsx`, `Hemicycle.tsx` — composants cold-case
  ou gov legacy, à supprimer ou migrer dans
  `packages/nexus-app-{coldcase,gov}/frontend/` (cf. §4 décision
  D5 sur le sort de la legacy)

**Ce qui est réutilisable dans `web/`** :

- `web/src/components/ui/` — 20 primitives shadcn (badge, button,
  card, command, dialog, dropdown-menu, input, progress,
  scroll-area, select, separator, sheet, sidebar, skeleton,
  tabs, textarea, toggle, toggle-group, tooltip) — à conserver
  tel quel
- `web/src/components/Layout.tsx` — layout shell générique (à
  auditer pour déps cold-case, probablement minimalement
  adaptable)
- `web/src/lib/` — utilitaires (cn, fetchers génériques)
- `web/vite.config.ts` + `web/tsconfig*.json` + `web/eslint.config.js`
  + `web/package.json` — toolchain Vite + React 19 + TS + Tailwind 4
  + shadcn. **Conservée intégralement**. Dépendances cold-case
  lourdes à retirer (@antv/g6, leaflet, react-leaflet, recharts,
  reagraph, sigma, nivo/*, react-force-graph-2d/3d,
  react-calendar-timeline, d3-parliament-chart, graphology*) — voir §4 décision D5.

### Ce qui n'existe PAS encore

- Aucun code shell nexus-grid dans `web/` (le fait qu'il existe
  `web/` avec 14 pages legacy ne compte pas)
- Aucun registre de coordinateurs actifs (`~/.nexus-grid/registry.json`
  ou équivalent) — le coordinateur ne s'auto-déclare nulle part
- Aucun daemon de découverte réseau (pas d'accès DHT / pkarr
  depuis le shell sauf via un coordinateur actif)
- Aucun worker HTTP API (le worker Rust est CLI-only)
- Aucun backend curator lists (cf. `.planning/sprint4_verification.md`
  §« What's NOT in this sprint » : « Curator list gossip flow —
  not part of Sprint 4 »)
- Aucun contrat de rendu des tabs d'app côté frontend (le
  manifest expose `name, icon, descriptor` mais rien ne dit
  comment le tab doit être rendu dans le shell)

## 3. Sources de vérité à lire AVANT d'agir

Lecture obligatoire, dans cet ordre, avant d'écrire le plan
détaillé Sprint 5 :

1. **`.planning/sprint4_verification.md`** en entier — c'est la
   liste authoritative de ce que Sprint 4 a livré et de ce qu'il
   a explicitement scope-cut (§« What's NOT in this sprint »).
   Sprint 5 DOIT hériter de ces scope cuts et ne PAS les
   re-ouvrir sans justification écrite.

2. **`.planning/sprint4_kickoff.md`** — spécifiquement les sections
   §6 (règles opérationnelles R1..R7) et §4 (logique « Day 0 »).
   Sprint 5 adopte les mêmes règles (adaptées au frontend), et la
   notion de « Day 0 blockers » y est définie.

3. **`C:\Users\FlowUP\.claude\plans\magical-marinating-phoenix.md`**
   - Chercher le plan Sprint 5 officiel (grep `Sprint 5`
     dans le fichier) pour le scope day-par-day théorique
   - Lignes des décisions architecturales figées (20 items) :
     elles contiennent notamment la règle « single writer v1.0
     iroh-docs », « kudos per-project » et « curator lists
     signées Ed25519 propagées via iroh-blobs + iroh-gossip —
     pas de Git » — toutes ont un impact direct sur le design du
     shell (respectivement : qui peut écrire quoi, où on affiche
     les kudos, comment on affiche les curators)

4. **`packages/nexus-coordinator/src/nexus_coordinator/api/`** en
   entier (app.py, apps.py, health.py, invites.py, kudos.py,
   tasks.py) — la surface API actuelle du coordinateur. C'est
   le contrat que le shell va consommer. Relire chaque `async
   def` avant de coder son client.

5. **`packages/nexus-sdk/src/nexus_sdk/app.py`** — `AppManifest`,
   `NexusApp`, `TabDescriptor`, `WorkerDescriptor`. Le shell
   doit consommer ces types donc il faut les connaître
   exactement.

6. **`examples/hello-world-app/src/hello_world_app/__init__.py`**
   (45 LOC) + **`packages/nexus-app-gov/src/nexus_app_gov/__init__.py`**
   — les 2 seules apps qui existent. Le shell est testé contre
   elles en premier (et pas contre un clone de `nexus/gov/`
   legacy 19-tab qui n'a pas été migré).

7. **`web/package.json`** — inventaire exact des deps React
   actuelles. Décision D5 (§4) se prend depuis cette liste, pas
   à l'aveugle.

8. **`web/src/components/ui/sidebar.tsx`** — le shadcn Sidebar
   primitive que l'AppSidebar legacy utilise déjà. Réutilisable
   tel quel dans le nouveau shell, à condition de retirer tout
   le contenu cold-case au-dessus.

9. **`CLAUDE.md`** en entier — spécifiquement la section
   « Langue » (réponse et UI en français), et le bloc
   architecture v2 (pour comprendre ce que le monolithe legacy
   faisait et pourquoi c'est mort).

10. **Ne PAS lire** (sauf pour la décision D5 qui explicitement
    les référence) :
    - `nexus/` Python legacy sauf `nexus/gov/` en extrait minimal
    - `web/src/pages/*.tsx` sauf en référence pour « est-ce qu'il
      y a un composant réutilisable là-dedans ? » (réponse
      attendue : non, sauf primitives shadcn déjà isolées dans
      `web/src/components/ui/`)
    - `docker-compose.yml` — Neo4j / ChromaDB / Robin
      n'existent plus dans le modèle P2P ; le shell n'en parle
      plus
    - `nexus/main.py` — mort, le shell ne doit jamais l'évoquer

## 4. Day 0 — 5 décisions critiques à trancher AVANT toute ligne de code

Le scope Sprint 5 contient 5 choix architecturaux qui, pris au
hasard au milieu d'une phase, cassent les 3 autres phases. Ils se
prennent TOUS Day 0, sont écrits dans `.planning/sprint5_plan.md`
avec justification, et sont soumis au user pour validation
explicite avant le premier commit Phase A.

### D1 — Registre des coordinateurs actifs

**Problème** : le shell doit savoir quels coordinateurs l'utilisateur
a lancés sur sa machine. Un coordinateur écoute sur un port
(`8765` par défaut, override) mais rien ne l'enregistre
globalement.

**Options** :

- (a) **Fichier registry** `~/.nexus-grid/registry.json` maintenu
  par le coordinateur : `nexus-coordinator start <name>` y
  append une entry `{name, port, node_id, pid, started_at}`,
  `stop` l'enlève, crash cleanup via TTL + re-heartbeat.
- (b) **Scan port range** 8765–8799 sur `127.0.0.1` depuis le
  shell, hit `/health` sur chaque, collecte les réponses.
  Moche et lent.
- (c) **Broadcast local mDNS** — overkill, ajoute zeroconf au stack.
- (d) **Fichier par coordinateur** dans `~/.nexus-grid/projects/<name>/running.json`
  écrit au boot, supprimé au shutdown ; le shell glob le dossier
  parent. Variante de (a) sans problème de concurrence d'écriture.

**Recommandation** : **(d)** — un fichier par projet, le shell
`glob('~/.nexus-grid/projects/*/running.json')`, parse chaque
entry. Résilient aux crashes (stale file → health check
timeout → marqué offline). Pas de problème de lock sur un
fichier partagé. Aligné sur le layout `~/.nexus-grid/projects/<name>/`
déjà utilisé par Sprint 4 (`iroh-data/`, `coordinator.toml`).

**Action Day 0** : écrire la décision dans `sprint5_plan.md`,
décider du schéma exact (fields + perms fichier), décider qui
écrit (coordinateur Python, extension `start`/`stop` dans
`packages/nexus-coordinator/src/nexus_coordinator/cli/`).
**Cette extension est un mini-fix backend qui tombe dans Sprint
5 Phase A, pas Sprint 6**. Ne pas essayer de pousser le shell à
scanner les ports — c'est le band-aid à éviter.

### D2 — Contrat de rendu des tabs d'app

**Problème** : le manifest d'une app expose des tabs avec
`name`, `icon`, `descriptor` (le résultat d'une fonction
Python). Rien ne dit comment le **contenu visuel** du tab est
rendu dans le shell React.

**Options** :

- (a) **Schema-driven rendering** : le descriptor Python retourne
  un JSON structuré (`{kind: 'table', rows: [...]}` ou
  `{kind: 'chart', series: [...]}`), le shell a un rendu
  générique par `kind`. Pro : zéro JS à écrire par app. Con :
  limite forte sur la richesse UI, force un vocabulaire
  fermé de composants.
- (b) **Iframe** : chaque app sert son tab à
  `/app/{name}/tab/{tab_id}/html`, le shell l'embarque en iframe,
  communication via `postMessage`. Pro : isolation forte. Con :
  style mismatch (chaque iframe a son propre CSS), intégration
  UX clunky, auth cross-frame pénible.
- (c) **Dynamic ES module import** : le manifest d'une app
  expose un `frontend_bundle_url` (ex:
  `http://127.0.0.1:8765/app/gov/static/bundle.js`) et chaque tab
  un `component: "GovernmentTab"`. Le shell fait
  `const mod = await import(url); const C = mod[component]; <C coordinatorUrl={url} />`.
  Pro : intégration native React, partage du theme shadcn via
  externals. Con : chaque app doit shipper un bundle JS compilé,
  CSP à penser, versioning.
- (d) **Sprint 5 ne rend AUCUN tab** : le shell ne fait que
  **lister** les tabs du manifest avec leur nom + icône, et
  cliquer dessus ouvre un modal qui affiche le JSON descriptor.
  Le vrai rendu tombe en Sprint 6 avec une décision mûrie sur
  (a) ou (c).

**Recommandation** : **(d) pour Sprint 5 MVP + (a) à design dans
`sprint5_plan.md` comme cible Sprint 6**. Raison : (c) ajoute un
build pipeline par app, ce qui casse `examples/hello-world-app`
< 100 LOC. (b) est une dette permanente. (a) est clean mais mérite
une itération de design qu'on n'a pas le budget Sprint 5 pour
faire sérieusement (définition exhaustive du vocabulaire :
`table`, `chart`, `form`, `markdown`, `heatmap`, `graph`, ...
plus les variants par `kind`). Sprint 5 **livre** le rendu JSON
brut + la liste navigable ; Sprint 6 **conçoit** le vocabulaire
schema-driven et migre hello-world-app puis gov. Le contrat
`/app/{name}/tab/{tab_id}` est déjà posé par Sprint 4 dans
`apps.py:55` → `/{name}/manifest` (qui retourne le descriptor
inline sync ou la note async), il suffit d'ajouter un endpoint
d'invocation async dédié pour les descriptors async (ce qui est
déjà un TODO implicite dans `_maybe_call`).

**Action Day 0** : écrire la décision, coder **aucun** rendu de
tab custom en Sprint 5, prévoir un rendu fallback générique
`<pre>{JSON.stringify(descriptor, null, 2)}</pre>` enveloppé dans
une Card shadcn. Le vocabulaire schema-driven Sprint 6 est listé
dans le plan avec 6-8 `kind` cibles et cité pour validation user
mais pas codé.

### D3 — Source de données pour /my-network (worker HTTP API ?)

**Problème** : la page `/my-network` doit afficher l'état du
worker Rust local (GPU utilisation, tâches en cours, kudos
gagnés per-project). Le worker est CLI-only et n'expose pas
d'HTTP.

**Options** :

- (a) **Ajouter une mini API HTTP au `nexus-worker`** — un
  endpoint `/status` lié à un `--api` flag. Nécessite
  `axum` ou `hyper` côté Rust. 100-200 LOC.
- (b) **Exec le CLI `nexus-worker stats --json`** depuis un
  sidecar Python côté shell — le shell hit un endpoint Python
  qui spawn le binaire et parse sa sortie. Couplage process
  fragile.
- (c) **Lecture de fichiers d'état** écrits par le worker dans
  `~/.nexus-grid/worker/state.json` (à créer via W10 Sprint 3 ou
  extension Sprint 5). Zéro nouvelle dépendance Rust, mais le
  worker doit flush régulièrement.
- (d) **Sprint 5 ne rend pas /my-network avec des données réelles**
  — la page affiche un mock + « bientôt » et la vraie data arrive
  en Sprint 6 ou via un v1.1 worker API.

**Recommandation** : **(c)** — le worker Rust étend son boucle
principale pour sérialiser toutes les N secondes un snapshot JSON
dans `~/.nexus-grid/worker/state.json` (node_id, uptime,
registered_projects avec kudos totaux, last_task, GPU snapshot
via NVML déjà intégré Sprint 3). Le shell lit ce fichier via un
sidecar Python minimal (voir D4) ou via un endpoint du
coordinateur si l'utilisateur a un coordinateur actif (préférer
via sidecar car le worker peut tourner sans coordinateur local).

**Action Day 0** : trancher. Si (c) retenu, écrire le contrat
du `state.json` dans le plan AVANT de coder le shell. Si (d),
mocker et marquer Sprint 6. Si (a), spinner une micro-décision
design « quel framework Rust HTTP » — probablement `axum` 0.7
qui est déjà dans l'écosystème tokio — via context7. (a) est
plus propre long-terme, (c) est plus économique en LOC Sprint 5
et aligné sur le principe « pas de nouveau daemon / framework
sauf nécessaire ». **Recommandation ferme : (c)**. Si le user
préfère (a), mettre ça dans le plan comme scope supplémentaire
explicite +1 jour.

### D4 — Sidecar shell (pour discovery + lecture du worker state)

**Problème** : le shell ne peut pas lire
`~/.nexus-grid/worker/state.json` depuis le navigateur (le JS
d'une SPA ne peut pas accéder au filesystem local via HTTP). Il
ne peut pas non plus interroger la DHT iroh-pkarr pour peupler
`/browse` sans un Node iroh actif. Il y a donc un besoin d'un
**sidecar local**.

**Options** :

- (a) **Un des coordinateurs actifs fait sidecar** : on pick
  arbitrairement le premier coordinateur dans le registry et on
  lui ajoute des endpoints « globaux » (`/shell/worker-state`,
  `/shell/browse-dht`). Couplage gênant : si aucun coordinateur
  n'est actif, le shell est muet.
- (b) **Nouveau process Python** `nexus-shell-daemon` (≈ 300 LOC)
  qui lance un Node iroh minimal via `nexus_core`, expose
  `/worker-state`, `/browse`, `/curators`, `/projects` sur un
  port fixe (ex: `8760`), tourne en background détaché par le
  shell au premier lancement. Pro : autonome, pas dépendant
  d'un coordinateur actif. Con : nouveau process, nouveau
  point de démarrage pour l'utilisateur.
- (c) **Tauri** (wrapper desktop Rust) — le shell tourne dans
  une webview Tauri qui a accès au FS et peut exec le worker.
  Décision lourde : le shell devient une desktop app. Attrayant
  pour la distribution single-binary mais hors scope 8-10 j
  Sprint 5.
- (d) **Scope cut** : pas de /browse, pas de /my-network data
  réelle, Sprint 5 livre seulement `/my-projects` et `/project/:name`
  qui tapent directement les coordinateurs actifs sans sidecar.
  /browse et /curators marqués « Sprint 6 » dans le shell avec
  un placeholder.

**Recommandation** : **combinaison (d) initial + (b) futur Sprint 6+**.
Le MVP Sprint 5 ne rend réellement que `/my-projects` et
`/project/:name` parce qu'ils ne nécessitent aucun sidecar
(lecture directe de chaque coordinateur actif via son HTTP API
existante). `/my-network` affiche le `state.json` via **lecture
directe** si on retient (c) sur D3 (dans ce cas, le shell n'a
pas le FS mais le **coordinateur actif** peut exposer un
endpoint proxy `/worker-state` qui lit le fichier — petite
addition sprint 5). `/browse` et `/curators` sont stubs renvoyant
vers Sprint 6. Le `nexus-shell-daemon` autonome est reporté à
Sprint 6 avec un design complet dans `sprint5_plan.md` (pas dans
le code).

**Conséquence Day 0** : Sprint 5 n'introduit **aucun nouveau
process**. Tout ce que le shell consomme sort d'un coordinateur
actif (via son HTTP) ou d'un fichier local lu par proxy via un
coordinateur actif. Si aucun coordinateur n'est actif, le shell
affiche un onboarding « Lance ton premier projet via
`nexus-coordinator init <name> && start <name>` ».

### D5 — Sort de la legacy `web/src/pages/*.tsx`

**Problème** : 14 pages cold-case + 418 lignes d'AppSidebar avec
case selector et stats Neo4j/Chroma + 418 lignes de deps React
cold-case lourdes dans `package.json`. Garder tout cela sous la
main « au cas où » pendant Sprint 5 = band-aid qui pourrit le
repo. Tout jeter = perdre des composants potentiellement
réutilisables (shadcn est isolé mais il y a peut-être 3-4
composants utilitaires dans `web/src/components/` qui valent la
peine).

**Options** :

- (a) **Suppression nette** : `git rm -r web/src/{pages,api,stores,hooks,components/{gov,compute}}.tsx`
  et les composants InvestigationMap, InvestigationTimeline,
  PipelineTools, Hemicycle. Nouveau App.tsx et AppSidebar from
  scratch. Plus toutes les deps cold-case dans package.json
  (antv, leaflet, nivo, recharts, reagraph, sigma, force-graph,
  graphology, react-calendar-timeline, d3-parliament-chart).
  Radical mais propre. Perte : rien d'utile, tout est cold-case.
- (b) **Migration vers `packages/nexus-app-coldcase/frontend/`**
  — on crée le placeholder package Sprint 5 et on y déplace
  tout. Risque : `nexus-app-coldcase` n'est pas scoped Sprint 5
  (c'est v1.1 per phoenix), créer son skeleton = scope creep.
- (c) **Archive Git** : `git mv web/src/{pages,api,stores,...}
  web/legacy/` + note `README.md` « archived for future
  migration ». Garde tout sous la main mais hors du bundle
  vite. Compromis.

**Recommandation** : **(a) suppression nette**. Raison : Sprint
5 impose un reset mental total. Si le shell voit `web/legacy/`
à côté, il y a un risque énorme de « oh je vais juste adapter
celle-ci vite fait » = band-aid. Les 14 pages cold-case ont été
conçues pour un backend monolithique mort ; elles ne sont pas
réutilisables même en référence. Leur git history reste
disponible via `git log -- web/src/pages/Evidence.tsx` si
quelqu'un en a besoin un jour. La migration cold-case v1.1
regardera git blame, pas un dossier `legacy/` qui traîne. Les
deps package.json cold-case sont retirées dans le même commit —
`npm install` post-Sprint 5 doit être lean.

**Action Day 0** : écrire l'inventaire exact des fichiers à
supprimer (liste ≈ 40 fichiers), inventaire exact des deps à
`npm uninstall`, inventaire exact de ce qui reste (≈ 25
composants ui + Layout + lib + config). Soumettre au user pour
validation avant le premier `git rm`.

---

**Rappel** : D1, D2, D3, D4, D5 sont tous écrits dans
`sprint5_plan.md` AVANT le premier commit. Aucune phase ne
commence tant que le user n'a pas validé les 5 décisions.

## 5. Plan Sprint 5 — 4 phases, ≈ 8-10 jours

Les jours sont indicatifs, pas contraignants. Structure réelle
en 4 phases qui se ferment chacune sur un critère passable.

### Phase A — Shell chrome + registry + client coordinator (≈ Day 1-3)

**Backend** :

- Étendre `packages/nexus-coordinator/src/nexus_coordinator/cli/`
  (commandes `start`, `stop`) pour écrire / supprimer
  `~/.nexus-grid/projects/<name>/running.json` selon D1.
  Schéma exact dans le plan. Ajouter tests : `test_running_json_written_on_start`,
  `test_running_json_removed_on_clean_stop`,
  `test_stale_running_json_ignored_by_healthcheck`.
- Ajouter `GET /worker-state` au coordinateur (proxy lecture de
  `~/.nexus-grid/worker/state.json` avec fallback `{running:
  false}` si absent) selon D3/D4 combinés.
- **Si D3 = (c)** : étendre `crates/nexus-worker-core/src/engine/runtime.rs`
  (ou un nouveau module `state_writer.rs`) pour flusher un
  snapshot JSON toutes les 5s dans
  `~/.nexus-grid/worker/state.json`. Contrat strict, versionné
  (`schema_version: 1`). Tests Rust : `state_writer_emits_valid_snapshot`,
  `state_writer_survives_permission_denied`.

**Frontend** :

- **D5 exécuté** : `git rm` en un commit dédié (message
  `refactor(web): drop legacy cold-case UI, archive via git history`).
  Nouveau `web/src/App.tsx` minimal (4 routes).
- `web/src/api/coordinator.ts` : client TS typé du coordinateur.
  Utiliser `zod` 3.23+ (via **context7**) pour valider chaque
  response → types propagés dans React Query. Endpoints couverts
  : `/health`, `/project`, `/tasks`, `/kudos`, `/kudos/verify`,
  `/invite/*`, `/app`, `/app/{name}/manifest`, `/worker-state`.
  **Aucun** fetch cru dans le reste du code.
- `web/src/stores/projectStore.ts` : Zustand (via **context7**
  pour la syntaxe 5.x) — state `{activeCoordinatorName, knownCoordinators}`
  persisté dans `localStorage` via middleware `persist`. Les
  known coordinators sont **reconciliés** au boot via un fetch
  à `/health` sur chaque entry du registry lu par un sidecar…
  → voir D4 : **Sprint 5 ne lit pas directement le filesystem**.
  À la place, le shell poll un endpoint proxy
  `GET /shell/discover` sur **chaque coordinateur actif connu**
  + un fallback manuel « Add coordinator » (URL input). Le
  premier launch demande à l'utilisateur « Entre l'URL de ton
  premier coordinateur » (par défaut `http://127.0.0.1:8765`).
  Une fois un coordinateur connu, il peut publier les autres
  via `GET /shell/discover` qui lit le registry côté backend
  (ce endpoint s'ajoute au coordinateur, lecture du
  `~/.nexus-grid/projects/*/running.json`). **Cette
  indirection est la seule option propre sans sidecar.**
- `web/src/components/AppShell.tsx` : nouveau layout + sidebar.
  4 entrées de nav (Projects, Network, Browse, Curators) +
  coordinator picker dans le header (dropdown listant les
  `knownCoordinators`). Reutilise `web/src/components/ui/sidebar.tsx`,
  `web/src/components/ui/dropdown-menu.tsx`, `web/src/components/ui/command.tsx`.
  Header affiche `nexus-grid` (pas « NEXUS Cold Case Intel »)
  et un status dot basé sur le `/health` du coordinateur
  actuellement sélectionné.

**Critère de fermeture Phase A** :

- `pytest packages/nexus-coordinator/tests/` vert, tous les
  tests running.json + /worker-state + /shell/discover passent
- `cargo test -p nexus-worker-core` vert, tests state_writer
  passent (si D3 = (c))
- `cd web && npm install && npm run build` vert, 0 warnings
  TS ni eslint
- `cd web && npm run dev` → shell accessible sur `http://127.0.0.1:3002`,
  affiche l'onboarding « Entre ton premier coordinateur » si
  aucun n'est connu

### Phase B — Page /my-projects + détail /project/:name (≈ Day 4-5)

**Frontend seulement** (backend déjà suffisant).

- `web/src/pages/Projects.tsx` : liste des coordinateurs connus.
  Pour chaque : card shadcn avec nom, node_id (tronqué 8 chars),
  visibility, doc_id, state, healthy dot. Actions : « Ouvrir »,
  « Retirer de la liste ». CTA en haut : « + Add coordinator »
  et « + New project » (le second ouvre un modal
  avec instructions CLI — on ne spawn pas de process depuis le
  browser, cf. D4).
- `web/src/pages/ProjectDetail.tsx` (route `/project/:name`) :
  tabs shadcn avec 5 panneaux :
  - **Overview** : manifest racine du projet (`/project` +
    `/health` combinés), worker list (dérivée du kudos ledger
    → `unique(worker_pubkey)`), compteurs tasks.
  - **Tasks** : table paginée des tâches soumises (`/tasks`
    — ce endpoint n'existe pas encore, **à ajouter en Phase A**
    comme backend patch pour lister les task_state en DB ;
    décision à trancher Day 0).
  - **Kudos** : ledger visualisé, intégrité affichée via
    `/kudos/verify` (✅ valid OR ❌ invalid first_bad_row=N).
  - **Invites** : liste via `/invite/list`, actions create /
    revoke. Dialog shadcn pour le formulaire.
  - **Apps** : liste via `/app`, chaque app expand vers son
    manifest complet `/app/{name}/manifest`. Chaque tab
    descriptor est affiché dans un `<pre>` formatté (rendu D2
    = (d)). Chaque route exposée par l'app est cliquable et
    hit `/app/{name}{route.path}` avec affichage de la
    response brute.

Tests d'intégration : Playwright 1.49+ (via **context7** pour
API) — spin un coordinateur Python réel via
`uv run nexus-coordinator init test && start test --port 18765`
depuis un `beforeAll`, charge hello-world-app comme entry point
dans l'env de test, fetch la home, vérifie que `/my-projects`
voit le coordinateur, `/project/test` affiche 1 app (hello),
clique sur l'app, vérifie que le manifest est rendu.

**Critère de fermeture Phase B** : le test Playwright
« shell-sees-coordinator-and-app » passe end-to-end avec un vrai
coordinateur + un vrai hello-world-app + aucun mock.

### Phase C — Page /my-network + wiring worker state (≈ Day 6-7)

- **Backend** : si D3 = (c), vérifier que le worker flush bien
  toutes les 5s, vérifier que `/worker-state` du coordinateur
  remonte les bonnes données. Ajouter un test e2e
  `tests/e2e/test_worker_state_roundtrip.py` : spawn
  `nexus-worker start --stub-ollama`, attendre 10s, lire
  `~/.nexus-grid/worker/state.json`, vérifier les fields attendus,
  vérifier que le coordinateur `/worker-state` le retourne.
- **Frontend** : `web/src/pages/Network.tsx` — affiche
  `/worker-state` du coordinateur actif (polling 2s via React
  Query). Cards :
  - **Worker identity** : node_id, uptime, version
  - **GPU** : NVML snapshot (memory, utilization, temp, power)
  - **Projects served** : liste des projects enrolled par le
    worker, avec kudos total per-project (lu depuis le
    `state.json` côté worker qui pousse ces données)
  - **Last task** : task_id, prompt tronqué, status, timestamp
- Pas de mocks. Si `state.json` absent, affiche « Worker non
  détecté — lance `nexus-worker start` dans un terminal ». Le
  shell ne spawn rien.

**Critère de fermeture Phase C** :
`npm run dev` + `nexus-worker start --stub-ollama` dans un
terminal + un coordinateur actif → `/my-network` affiche des
données live du worker en < 5s.

### Phase D — /browse + /curators stubs + polish + verification (≈ Day 8-10)

- **/browse** : page stub avec explication du design Sprint 6
  (DHT pkarr query via sidecar, public projects listing). Rend
  un placeholder Card expliquant ce qui vient et pourquoi c'est
  reporté. Pas de code mort, pas de hidden TODO.
- **/curators** : idem, stub. Explique que les curator lists
  arrivent avec le gossip flow Sprint 6. Placeholder.
- **Polish** : dark theme shadcn confirmé, responsive ≥ 1280px,
  i18n fr (tous les textes UI en français, cf. CLAUDE.md).
  Aucun string anglais user-facing sauf node_id / hashes.
  Keyboard shortcuts : `Cmd/Ctrl+K` = command palette (shadcn
  `Command`), `g p` = go projects, `g n` = go network.
- **E2E complet** : Playwright scénario « onboarding → add
  coordinator → voit app → ouvre tab → voit descriptor JSON ».
  Critère de succès Sprint 5.
- **Verification document** :
  `.planning/sprint5_verification.md` sur le modèle de
  `sprint4_verification.md`, 15-18 lignes fail-fast (voir §7).
- **Cleanup** : `npm audit` sans high/critical, bundle size
  analysé via `npx vite-bundle-visualizer` (via **context7**),
  aucune dep cold-case résiduelle.

**Critère de fermeture Phase D** :

- `/browse` et `/curators` rendent un placeholder propre, pas
  de 404
- E2E Playwright scénario onboarding complet passe en < 60s
- `.planning/sprint5_verification.md` rédigé, 100 % rows PASS

## 6. Règles opérationnelles (non négociables)

### R1 — Context7 obligatoire avant tout code contre une lib

Requêter **context7** pour toute bibliothèque non-triviale avant
d'écrire du code contre elle. Minimum obligatoire :

| Lib | Usage Sprint 5 |
|---|---|
| `react` 19.2 | hooks et patterns récents (cache, suspense, useOptimistic) |
| `react-router-dom` 7.14 | data router API, nested routes, loader pattern |
| `@tanstack/react-query` 5.96 | queries, mutations, suspense integration React 19 |
| `zod` 3.23+ | schémas de validation pour coordinator API (ajout ou confirmation version depuis web/package.json) |
| `zustand` 5.x | store projectStore avec persist middleware |
| `tailwindcss` 4.2 | utilitaires v4 (nouvelle config via CSS vs tailwind.config.js) |
| `@base-ui/react` 1.3 | primitives (si utilisées à la place de radix pour certains cas) |
| `shadcn` v4 | nouveaux composants à ajouter via `npx shadcn@latest add ...` (ne JAMAIS copier-coller manuellement une version périmée) |
| `vite` 6.x | build config, plugin react swc, bundle analyzer |
| `playwright` 1.49+ | test runner, fixtures asynchrones, trace viewer |
| `fastapi` 0.111+ | déjà vu Sprint 4 mais re-check les patterns WebSocket / SSE si on en ajoute |
| `pydantic` 2.6+ | schema du running.json côté coordinator |
| `pydantic-settings` 2.13+ | extension config coordinator (paths registry) |
| `structlog` 25.5+ | logging coordinator et worker (pour le state_writer) |
| `axum` 0.7 (SEULEMENT si D3 = (a)) | micro HTTP API worker |

**Ne JAMAIS deviner une signature d'API.** Ne JAMAIS écrire du
code basé uniquement sur la training data du modèle, en particulier
pour les libs JS qui bougent vite (React 19 patterns, Tailwind v4
CSS config, React Router 7 data API, shadcn v4 CLI). Si context7
retourne une signature qui ne matche pas ce à quoi on s'attend,
**ARRÊTER** et relire la réponse au lieu de coder « ce qu'on croit
être juste ».

Re-check contre `web/package.json` en parallèle : les versions
exactes installées sont l'autorité finale. Si context7 est en
avance ou en retard, la version installée gagne. `npm install`
pour confirmer après chaque ajout.

### R2 — Pas de fix pansement, cause racine uniquement

Si un bug est détecté en cours de Sprint 5, fixer la **cause
racine**, pas le symptôme. Si le symptôme est dans un module
différent de celui qu'on implémente, créer une entry tech debt
dans `docs/shell/PATTERNS.md` (à créer Phase A) OU
`docs/coordinator/PATTERNS.md` OU `docs/rust/PATTERNS.md` ET
décider explicitement : fix maintenant OU tag Sprint 6. Ne
JAMAIS commiter :

- `try/catch { /* ignore */ }` sans justification écrite
- `as any` ou `@ts-ignore` / `@ts-expect-error` sans lien vers
  un issue tracker
- Valeur hardcodée « temporaire » (URL, port, path)
- Fetch non-typé (utiliser `coordinator.ts` client uniquement)
- Composant UI avec styling inline permanent (tout dans shadcn
  / Tailwind)
- Commit message de la forme « WIP », « fix », « tmp », « hack »
- Skip de test Playwright avec `.skip()` sans commentaire
  expliquant le scope cut
- Mock de coordinateur dans un test qui devrait passer par un
  vrai coordinateur (cf. R4)

**Spécifique Sprint 5** : interdiction absolue de garder des
références au backend legacy `nexus/api/*` dans le nouveau code
web/. Interdiction de créer des `// TODO: migrate from legacy`
à côté d'un composant nouveau. La migration est **terminée** par
suppression (D5), pas en chantier.

### R3 — Global, deep, pas local

Avant d'écrire du code pour une phase, **relire le plan des 4
phases en entier** et vérifier qu'aucune décision prise dans la
phase en cours ne contraint les phases suivantes de façon
problématique.

Exemples concrets :

- Le schema du `running.json` en Phase A **détermine** ce que
  `/my-projects` peut afficher en Phase B. Si on oublie
  `started_at`, on ne peut pas afficher « uptime ». Décision
  schema Day 0.
- Le contrat du `worker state.json` en Phase A **détermine** ce
  que `/my-network` peut afficher en Phase C. Si on oublie
  `projects_served`, pas de tableau per-project kudos. Décision
  schema Day 0.
- Le contrat du tab rendering (D2) **détermine** comment
  `examples/hello-world-app` et `nexus-app-gov` devront adapter
  leurs tabs en Sprint 6. Si on ship (c) dynamic import, il faut
  déjà penser au versioning du bundle JS. Si on ship (d) JSON
  brut, il faut déjà documenter que Sprint 6 aura un format
  schema-driven pour ne pas piéger des apps tierces Sprint 6.
- Le choix (a) / (b) pour la discovery inter-coordinateurs en
  Phase A **détermine** le code de la dropdown coordinator picker
  en Phase B et l'onboarding initial en Phase D.

Ces 4 décisions (schemas running/worker-state + D2 + discovery
flow) doivent être prises et écrites dans `sprint5_plan.md`
**avant** Phase A. Le plan détaillé est livrable Day 0.

### R4 — Tests d'intégration contre vrai coordinateur + vrai worker

Le but de Sprint 5 est un shell qui marche contre le vrai
backend. Un test unitaire qui mock le coordinator client ne
prouve pas grand-chose. Tout composant qui lit / écrit via
`coordinator.ts` doit avoir **au moins un** test d'intégration
qui passe par un **vrai** coordinateur spawné en subprocess
(pattern `subprocess + httpx` côté Python, pattern `globalSetup
+ spawn` côté Playwright).

Acceptable :

- Tests unitaires purs pour les helpers de formatage
  (`formatNodeId`, `shortenHash`, `formatKudos`)
- Tests unitaires purs pour les Zod schemas (valid + invalid
  cases via `schema.safeParse`)
- Tests composants isolés pour le rendu pur (shadcn primitives
  wrappées)

Tout le reste passe par un coordinateur réel. Le pattern de
spawn est :

```python
# packages/nexus-coordinator/tests/conftest.py style
@pytest.fixture
def live_coordinator(tmp_path):
    subprocess.Popen(["uv", "run", "nexus-coordinator", "start", "test",
                      "--data-dir", str(tmp_path), "--port", "18765"])
    # wait for /health 200
    yield "http://127.0.0.1:18765"
    # stop cleanly
```

Pour Playwright :

```typescript
// web/tests/fixtures.ts
test.beforeAll(async () => {
  coordProcess = spawn("uv", ["run", "nexus-coordinator", "start", "test",
                              "--port", "18765", "--data-dir", tmpDir]);
  await waitForHealth("http://127.0.0.1:18765");
});
```

Pas de mock du port 18765. Pas de MSW. Pas de fetch mocké.

### R5 — Commits atomiques par phase

Message format : `feat(web|coordinator|worker-core): Sprint 5
Phase <A|B|C|D> — <résumé concis>`. Un commit par fichier-major
+ ses tests. Pas de mega-commits multi-phases. Pas de commits
« WIP ». Le plan Sprint 5 est la grille — chaque commit cite la
phase (pas le jour, qui est indicatif).

Cas spéciaux :

- `refactor(web): drop legacy cold-case UI` (D5) est son **propre**
  commit, avant le premier commit Phase A. Il ne contient QUE des
  suppressions. Pas d'ajout dans le même commit.
- Les extensions backend (running.json, /worker-state, /shell/discover)
  sont des commits `feat(coordinator)` distincts des commits
  `feat(web)`. Un même PR peut en chaîner plusieurs, chacun est
  atomique.
- Si D3 = (c), l'extension worker est un commit
  `feat(worker-core): state writer for shell integration`.

### R6 — Ne PAS toucher

- **Le code Python legacy dans `nexus/`** — Sprint 5 ne regarde
  même pas dans ce dossier. Aucune migration, aucune référence.
- **Les apps gov / hello-world côté Python** — pas d'ajout de
  fonctionnalité à `packages/nexus-app-gov/` ni à
  `examples/hello-world-app/`. Sprint 5 consomme leur manifest,
  point final.
- **`crates/nexus-core-rs/`, `crates/nexus-core-py/`** — aucun
  changement. La PyO3 surface Sprint 4 est suffisante.
- **`crates/nexus-worker/` binary** — seul `nexus-worker-core`
  peut gagner le state_writer si D3 = (c). Le binaire lui-même
  ne change pas (sauf pour wire le state_writer dans le main
  loop — ≤ 10 lignes).
- **`docker-compose.yml`** — mort pour le produit nexus-grid,
  ne pas s'en préoccuper.
- **`magical-marinating-phoenix.md`** — source de vérité figée,
  relecture uniquement.
- **`.planning/sprint{0,1,2,3,4}_*.md`** — archives, ne pas
  modifier.

### R7 — Verification continue

À la fin de chaque phase, lancer :

```bash
# Rust (si D3 = (c) ou si un patch worker-core a été fait)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p nexus-core-rs --lib
cargo test -p nexus-worker-core --lib
cargo test -p nexus-worker --test e2e

# Python
uv run ruff format --check packages/ examples/
uv run ruff check packages/ examples/
uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q

# Frontend
cd web
npm run lint
npx tsc --noEmit -p tsconfig.json
npm run build
npx playwright test   # Phase B+
cd ..
```

Tous verts → fermer la phase et commiter la tête. Un seul rouge
→ stop et fix avant d'avancer. **Jamais** de `test.skip()` ou
`it.todo()` comme workaround.

## 7. Critères de sortie Sprint 5 (tableau fail-fast)

À produire en fin de Sprint 5 sous forme de
`.planning/sprint5_verification.md` analogue à
`.planning/sprint4_verification.md` :

| # | Check | Commande | Critère |
|---|---|---|---|
| 1 | running.json écrit au start | `pytest packages/nexus-coordinator/tests/test_registry.py::test_running_json_written_on_start` | pass |
| 2 | running.json retiré au stop | `pytest packages/nexus-coordinator/tests/test_registry.py::test_running_json_removed_on_clean_stop` | pass |
| 3 | /shell/discover liste les running | `pytest packages/nexus-coordinator/tests/test_shell_discover.py` | pass |
| 4 | /worker-state proxy | `pytest packages/nexus-coordinator/tests/test_worker_state_proxy.py` | pass |
| 5 | worker state_writer (si D3=(c)) | `cargo test -p nexus-worker-core --lib state_writer` | pass |
| 6 | legacy cold-case removed | `test -d web/src/pages && ls web/src/pages \| grep -E "(Evidence\|Dashboard\|Investigation)"` | exit 1 (0 matches) |
| 7 | zero legacy deps | `grep -E "(antv/g6\|leaflet\|recharts\|reagraph\|sigma\|nivo\|force-graph\|graphology\|parliament)" web/package.json` | exit 1 (0 matches) |
| 8 | TypeScript strict clean | `cd web && npx tsc --noEmit` | exit 0 |
| 9 | ESLint clean | `cd web && npm run lint` | exit 0 |
| 10 | Build prod clean | `cd web && npm run build` | exit 0, no warnings |
| 11 | Shell onboarding | Playwright `shell-onboarding-empty-state.spec.ts` | pass |
| 12 | Add coordinator flow | Playwright `shell-add-coordinator.spec.ts` | pass |
| 13 | /my-projects renders live | Playwright `my-projects-live.spec.ts` (live coord) | pass |
| 14 | /project/:name manifest | Playwright `project-detail-manifest.spec.ts` | pass |
| 15 | Apps tab listing + descriptor | Playwright `apps-tab-render.spec.ts` (hello + gov) | pass |
| 16 | /my-network reads worker | Playwright `my-network-live.spec.ts` (stub-ollama worker) | pass |
| 17 | /browse et /curators stubs | Playwright `stub-pages.spec.ts` (no 404) | pass |
| 18 | All Rust tests | `cargo test --workspace --exclude nexus-core-py` | all green, ≥ 166 + worker-core state_writer tests |
| 19 | All Python tests | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q` | all green (≥ 27 + registry + shell + worker-state tests) |
| 20 | French UI | grep scan des strings EN dans `web/src/` | 0 strings UI anglais hors identifiants techniques |

## 8. Première action (ne rien lancer avant ceci)

Dans l'ordre strict :

1. Vérifier working tree clean (`git status`). Si quelque chose
   d'inattendu est untracked, demander au user quoi en faire
   avant de commencer.
2. Lire en entier dans l'ordre § 3 :
   - `.planning/sprint4_verification.md`
   - `.planning/sprint4_kickoff.md` §§ 4 et 6
   - phoenix.md (sections Sprint 5)
   - `packages/nexus-coordinator/src/nexus_coordinator/api/` (6 fichiers)
   - `packages/nexus-sdk/src/nexus_sdk/app.py`
   - `examples/hello-world-app/src/hello_world_app/__init__.py`
   - `packages/nexus-app-gov/src/nexus_app_gov/__init__.py`
   - `web/package.json`
   - `web/src/App.tsx` + `web/src/components/AppSidebar.tsx`
     (en lecture seule, pour inventaire de suppression D5)
   - `web/src/components/ui/sidebar.tsx` (le primitif à réutiliser)
   - `CLAUDE.md`
3. Requêter **context7** pour les libs listées §6 R1, au minimum :
   React Router 7 data API, TanStack Query 5 + React 19 suspense,
   Zustand 5 persist middleware, Tailwind 4 CSS config, shadcn
   v4 CLI, Playwright 1.49+ fixtures async, Zod 3.23+.
4. Écrire `.planning/sprint5_plan.md` — plan détaillé des 4
   phases avec, pour chaque phase :
   - Liste exhaustive des fichiers à créer / modifier / supprimer
     (chemin absolu)
   - Dépendances à ajouter ou retirer (`web/package.json`,
     `packages/nexus-coordinator/pyproject.toml`,
     `crates/nexus-worker-core/Cargo.toml`)
   - Les 5 décisions Day 0 (D1..D5) résolues avec justification
   - Liste des tests attendus (Python, Rust si applicable,
     Playwright)
   - Critère de fermeture de la phase
   - Schemas JSON exacts pour `running.json` et `worker state.json`
     et `AppShellDiscoverResponse`
5. **Montrer ce plan au user et attendre validation avant de
   coder quoi que ce soit**. Spécifiquement : les 5 décisions
   critiques (D1..D5) doivent être approuvées explicitement
   avec option de rediscussion si une alternative n'a pas été
   évaluée.
6. **Ensuite seulement** : commit `refactor(web): drop legacy
   cold-case UI` (D5 execution), puis Phase A, B, C, D dans
   l'ordre.

---

**Rappel final** : Sprint 5 livre le **premier produit
utilisable** de nexus-grid. Avant Sprint 5, tout le stack est
de la plomberie (crates Rust, bindings PyO3, coordinator sans
interface, worker sans interface, apps sans UI). Après Sprint 5,
un utilisateur non-dev lance `nexus-coordinator init foo && start foo`,
ouvre le shell, voit son projet, voit ses tâches, voit son
worker qui tourne, clique sur l'app gov et voit son manifest
rendu. C'est le passage de « lib P2P » à « app desktop web ».
Tout ce qui n'aide pas directement cette expérience user est
Sprint 6. Pas de raccourci, pas de band-aid, pas de legacy qui
traîne.
