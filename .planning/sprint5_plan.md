# Sprint 5 — Plan détaillé (frontend shell P2P nexus-grid)

**Écrit** 2026-04-10 à partir de `.planning/sprint5_kickoff.md` après
lecture des sources §3. Ce document est la grille Sprint 5 : chaque
commit cite la phase, chaque test est listé ici, chaque décision
Day 0 (D1..D5) est tranchée avec justification. **Aucun code n'est
écrit tant que le user n'a pas validé §2.**

HEAD à l'entrée : `3b5c162` (tip Sprint 4, vérifié §1 kickoff).
Working tree clean modulo `.planning/sprint5_kickoff.md` untracked
(sera commité avec ce plan en `docs(sprint5): kickoff + detailed plan`).

---

## 1. État vérifié à l'entrée

### Ce qui existe et est consommé par Sprint 5

**Coordinator Python** — API surface exacte vérifiée dans
`packages/nexus-coordinator/src/nexus_coordinator/api/` :

| Méthode | Route | Retour |
|---|---|---|
| GET | `/health` | `coordinator.health_payload()` → `{node_id, doc_id, project_name, state}` |
| GET | `/project` | `coordinator.project_payload()` → metadata sans ticket |
| POST | `/tasks/submit` | body `TaskCreateBody` → `{task_id}` |
| GET | `/tasks` | query `state`, `limit` → `{tasks: [...], count}` |
| GET | `/kudos` | query `worker_pubkey_hex?` → `{entries: [...], count}` |
| GET | `/kudos/verify` | → `{ok: bool, first_bad_row_id: int\|null}` |
| POST | `/invite/create` | body `CreateInviteBody` → `{id, wire, scope, expires_at, max_uses, note}` |
| GET | `/invite` | → `{invites: [...], count}` |
| DELETE | `/invite/{invite_id}` | → `{id, revoked: true}` |
| GET | `/app` | → `{apps: [{name, version, description, routes, workers, tabs}], count}` |
| GET | `/app/{name}/manifest` | → `{manifest, routes, workers, tabs}` — le `tabs[i].descriptor` est le retour de `fn(app)` si sync, sinon `{note: "async..."}` |

**Note importante** : la forme effective diffère du kickoff sur 2
points (le kickoff mentionnait `POST /invite/revoke` et
`GET /invite/list`). Le client TS Sprint 5 **doit** suivre la forme
effective ci-dessus (`DELETE /invite/{id}` et `GET /invite`).

**SDK** — types consommés par le shell (depuis
`packages/nexus-sdk/src/nexus_sdk/app.py`) :

- `AppManifest`: `{name, version, author, description, dependencies, license}`
- `WorkerDescriptor`: `{name, model}`
- `TabDescriptor`: `{name, icon}` + `descriptor` retour de fn sérialisé
- `RouteDescriptor`: `{path, methods}`

**Apps installées** (entry point `nexus.apps`) :

- `hello` → 1 route `/hello`, 1 worker `hello_worker`, 1 tab `Hello`
- `gov` → 1 route, 1 worker, 1 tab (migration 19-tab reportée v1.1)

**Worker Rust** — CLI only (`register`, `start`, `join`, `projects`,
`browse`, `stats`, `config`). Pas d'HTTP. Boucle principale dans
`crates/nexus-worker-core/src/engine/runtime.rs`. `--stub-ollama`
existant depuis Sprint 4 Phase D part 2 — réutilisé par les tests
Sprint 5 Phase C.

**Frontend existant** — 14 pages legacy cold-case + 418 lignes
d'AppSidebar avec case selector + stats Neo4j. Inventaire complet
§8 (exécution D5).

### Libs context7 consultées pour ce plan

| Lib | ID | Query (synthèse) |
|---|---|---|
| Zod v3.24.2 | `/colinhacks/zod/v3.24.2` | `z.object({...})`, `.optional()`, `z.infer<typeof S>`, `safeParse` → discriminated union `{success, data} \| {success, error}` |
| React Router 7.9.4 | `/remix-run/react-router/react-router_7.9.4` | `createBrowserRouter([{path, Component, children}])` + `<Outlet />` + `<NavLink to={} className={({isActive})=>...}>`. Data loaders optionnels, non utilisés Sprint 5 (les data sont chargées via React Query dans les composants) |
| Zustand v5.0.12 | `/pmndrs/zustand/v5.0.12` | `create<T>()(persist((set, get) => ({...}), {name, storage: createJSONStorage(() => localStorage), partialize}))` — TypeScript nécessite la forme curried `create<T>()(...)` |

Les autres libs (TanStack Query 5, Tailwind 4, shadcn v4 CLI,
Playwright 1.49+, @nexus-sdk manifest client) seront consultées
via context7 **juste avant** le commit qui les utilise (R1).

---

## 2. Décisions Day 0 — tranchées, en attente de validation user

Ces 5 décisions sont **gelées** une fois validées. Modification
ultérieure = blocker qui arrête la phase et relance une discussion.

### D1 — Registre des coordinateurs actifs → **option (d) retenue**

**Retenu** : un fichier `running.json` par projet, écrit par
`nexus-coordinator start <name>` après boot, supprimé proprement
au shutdown. Le shell ne lit PAS le filesystem directement — il
interroge `GET /shell/discover` sur un coordinateur actif connu,
qui lit lui-même `~/.nexus-grid/projects/*/running.json`.

**Justification** :
- Aligné sur le layout Sprint 4 (`~/.nexus-grid/projects/<name>/`)
- Résilient aux crashes (fichier orphelin → health check timeout →
  shell marque offline)
- Un fichier par projet → aucune concurrence d'écriture
- Aucun nouveau daemon, aucun scan de ports, aucun broadcast mDNS
- Auto-cleanup : `start` écrit au boot + supprime au `finally`
  du main loop dans `cli/commands/start.py`

**Schéma `running.json` (gelé)** :

```json
{
  "schema_version": 1,
  "project_name": "gov",
  "node_id": "8b4f...64hex",
  "doc_id": "doc-...",
  "api_host": "127.0.0.1",
  "api_port": 8765,
  "pid": 12345,
  "started_at": "2026-04-10T14:23:00.123456+00:00",
  "visibility": "private"
}
```

**Contrats** :
- `schema_version: 1` littéral — toute bump est breaking pour le
  shell et doit coincider avec un bump du client TS
- `started_at` en ISO-8601 UTC avec microsecondes (`datetime.now(UTC).isoformat()`)
- `node_id` en hex lowercase 64 chars (Ed25519 pubkey)
- `doc_id` peut être vide si le coordinator n'a pas encore ouvert
  le doc tasks (état transient, ne doit pas arriver en prod)
- `pid` int positif — utilisé Sprint 6 pour un cleanup tool de
  fichiers orphelins
- `visibility` enum `"public" | "private"`

**Écriture** : helper `nexus_coordinator.paths.running_state_path(project_name)`
retourne `~/.nexus-grid/projects/<name>/running.json`. Writer
atomique : écrire `running.json.tmp` puis `os.replace`.

**Lecture** : endpoint `GET /shell/discover` (nouveau Phase A) fait
`projects_root().glob("*/running.json")`, parse chaque entry
(validation Pydantic), retourne la liste. Pas de health-check
inline — c'est le shell qui hit `/health` sur chaque entry.

### D2 — Rendu des tabs d'app → **option (d) pour Sprint 5 + design (a) pour Sprint 6**

**Retenu Sprint 5** : le shell ne rend AUCUN tab custom. Il affiche
la liste des tabs d'une app (`name, icon`) et, au clic, ouvre un
panneau avec `<pre>{JSON.stringify(descriptor, null, 2)}</pre>`
enveloppé dans une `Card` shadcn.

**Pour les tabs async** : l'endpoint
`/app/{name}/manifest` retourne actuellement
`{note: "async descriptor — call /app/{name}/tabs/{tab} to invoke"}`
pour un tab async. Phase B ajoute l'endpoint **manquant**
`GET /app/{name}/tabs/{tab_name}/descriptor` qui invoque
le fn async et retourne le résultat. Le shell poll cet endpoint
sur demande (via un bouton « Rafraîchir » sur le panneau tab).

**Design Sprint 6 (documenté, non codé)** : vocabulaire
schema-driven avec kinds `{kind: "markdown"|"table"|"metric"|"chart"|"form"|"list"}`
et composants shadcn correspondants dans le shell. Cadre de
discussion posé ici :

```typescript
// Sprint 6 cible — aucune ligne de code Sprint 5
type TabDescriptor =
  | { kind: "markdown"; content: string }
  | { kind: "table"; columns: string[]; rows: string[][] }
  | { kind: "metric"; label: string; value: number | string; delta?: number }
  | { kind: "chart"; series: Array<{ name: string; points: Array<[number, number]> }> }
  | { kind: "form"; fields: Array<{ name: string; type: "text"|"number"|"select"; options?: string[] }>; submit_path: string }
  | { kind: "list"; items: Array<{ title: string; subtitle?: string; href?: string }> };
```

Sprint 6 migrera `hello-world-app` puis `nexus-app-gov` vers ce
vocabulaire.

**Justification** :
- (c) dynamic import casse `hello-world-app < 100 LOC` (build
  pipeline par app)
- (b) iframe crée une dette permanente (CSS mismatch, postMessage)
- (a) schema-driven est propre mais nécessite une itération de
  design qu'on n'a pas le budget Sprint 5 pour bien faire
- (d) garde le contrat `/app/{name}/manifest` actuel et donne un
  résultat visible user (il peut lire le descriptor brut)

### D3 — Source de données `/my-network` → **option (c) retenue (state.json via worker)**

**Retenu** : le worker Rust étend son main loop pour flusher
`~/.nexus-grid/worker/state.json` toutes les 5s. Le shell n'accède
PAS au FS — il interroge `GET /worker-state` sur un coordinateur
actif, qui lit le fichier (wrapper lecture + fallback
`{running: false}`). Si `state.json` est vieux (> 15s), le
coordinator retourne `{running: true, stale: true, state: {...}}`.

**Justification** :
- (a) axum mini-API : ajoute ~150 LOC + une dep Rust. Plus propre
  long-terme mais coût Sprint 5 élevé et duplique la gestion de
  CORS/auth qu'on n'a pas à faire via proxy
- (b) exec CLI : fragile, process spawn à chaque poll
- (c) state.json flush : ~80 LOC Rust (state_writer module), zéro
  nouvelle dep, le contrat est versionné, le worker peut tourner
  sans coordinateur (le fichier reste à disposition pour Sprint 6
  quand un shell-daemon arrivera)
- (d) scope cut : perd la démo `/my-network` qui est un critère de
  succès user Sprint 5

**Schéma `~/.nexus-grid/worker/state.json` (gelé)** :

```json
{
  "schema_version": 1,
  "node_id": "worker-pubkey-hex-64",
  "worker_version": "0.3.0",
  "uptime_secs": 1234,
  "started_at": "2026-04-10T14:00:00+00:00",
  "last_updated_at": "2026-04-10T14:20:30.500000+00:00",
  "gpu": {
    "name": "NVIDIA GeForce RTX 5080",
    "memory_total_mb": 16384,
    "memory_used_mb": 5123,
    "utilization_pct": 42,
    "temperature_c": 61,
    "power_draw_w": 180
  },
  "projects_served": [
    {
      "project_name": "gov",
      "doc_id": "doc-hex...",
      "kudos_total": 1234,
      "tasks_completed": 56
    }
  ],
  "last_task": {
    "task_id": "task-uuid",
    "project_name": "gov",
    "prompt_preview": "First 120 chars of the prompt, UTF-8 safe...",
    "status": "completed",
    "completed_at": "2026-04-10T14:20:25Z"
  }
}
```

**Contrats** :
- `schema_version: 1` littéral
- `gpu` est `null` si NVML n'a pas trouvé de device (MacBook, VM)
- `projects_served` est vide `[]` si aucun projet n'est enrolled
- `last_task` est `null` si le worker n'a encore rien traité
- Toutes les dates ISO-8601 UTC
- Écriture atomique : `state.json.tmp` → `rename`
- Flush cadence : **5s** (paramétrable via `WorkerConfig.state_flush_secs`,
  default 5)
- `last_updated_at` permet au lecteur de détecter un staleness

**Lecture** : `GET /worker-state` du coordinator lit
`~/.nexus-grid/worker/state.json`. Si absent → `{running: false}`.
Si présent → valide via Pydantic model `WorkerStateV1`, compare
`last_updated_at` à now, retourne
`{running: true, stale: (age > 15), state: {...}}`.

**Endpoint `/worker-state` est global** (pas per-project) — tout
coordinateur peut le servir, le shell pick le premier actif.

### D4 — Sidecar → **scope cut Sprint 5, nexus-shell-daemon Sprint 6+**

**Retenu** : aucun nouveau process Sprint 5. Tout ce que le shell
consomme sort soit d'un coordinateur actif (HTTP), soit d'un
fichier lu par proxy via un coordinateur actif (`/shell/discover`,
`/worker-state`). Si aucun coordinateur n'est actif, onboarding
explicite « lance ton premier projet ».

**Conséquences** :
- `/browse` (DHT pkarr) → stub Sprint 5, design `nexus-shell-daemon`
  Sprint 6
- `/curators` → stub Sprint 5, arrive avec le gossip flow Sprint 6
- `/my-projects`, `/project/:name`, `/my-network` → complets Sprint 5

**Onboarding zero-coordinator** : si
`localStorage.knownCoordinators` est vide, le shell affiche une
page « Bienvenue — entre l'URL de ton premier coordinateur »
(default input prerempli `http://127.0.0.1:8765`) ou un CTA
« Comment lancer mon premier coordinateur » qui ouvre un modal
avec les commandes CLI copiables.

### D5 — Legacy `web/` → **option (a) suppression nette**

**Retenu** : `git rm` en un commit dédié
`refactor(web): drop legacy cold-case UI, archive via git history`.
Aucun dossier `web/legacy/`. Les primitives shadcn
(`web/src/components/ui/*`) sont conservées.

**Inventaire exact (§8)** :

**Fichiers supprimés** (≈40 fichiers) :

| Catégorie | Fichiers |
|---|---|
| Pages | `web/src/pages/*.tsx` (14 : Dashboard, Evidence, Entities, Hypotheses, Graph, Timeline, Investigation, Suspects, Wiki, Reports, ImageSearch, Benchmark, GovernmentPage, NetworkPage) |
| API clients | `web/src/api/client.ts`, `web/src/api/compute.ts`, `web/src/api/government.ts` |
| Stores | `web/src/stores/caseStore.ts`, `web/src/stores/systemStore.ts`, `web/src/stores/eventStore.ts` |
| Hooks legacy | `web/src/hooks/useCase.ts`, `useSystemStats.ts`, `useApi.ts`, `useSSE.ts`, `useGovernment.ts`, `useCompute.ts` |
| Composants métier | `web/src/components/InvestigationMap.tsx`, `InvestigationTimeline.tsx`, `PipelineTools.tsx`, `Hemicycle.tsx`, `CommandPalette.tsx` (custom), `Layout.tsx`, `AppSidebar.tsx`, `Sidebar.tsx`, `TopBar.tsx`, `DataTable.tsx`, `MetricCard.tsx`, `ScoreBar.tsx`, `Card.tsx` (custom, pas ui/), `Badge.tsx` (custom, pas ui/), `LoadingSpinner.tsx`, `Toast.tsx` |
| Gov cold-case | `web/src/components/gov/` entier (20 fichiers) |
| Compute cold-case | `web/src/components/compute/` entier (8 fichiers) |
| Types | `web/src/types/d3-parliament-chart.d.ts` |

**Fichiers conservés (non touchés par le rm)** :

- `web/src/components/ui/*` (21 primitives shadcn, incluant `input-group.tsx` et `sidebar.tsx`)
- `web/src/lib/utils.ts` (cn helper)
- `web/src/hooks/use-mobile.ts` (shadcn companion)
- `web/src/main.tsx` (re-écrit Phase A, pas supprimé)
- `web/src/App.tsx` (re-écrit Phase A, pas supprimé)
- `web/src/index.css` (à auditer Phase A pour retirer var cold-case)
- `web/vite.config.ts`, `web/tsconfig*.json`, `web/eslint.config.js`
- `web/package.json` (modifié, voir §2.5.2)

**Dépendances retirées** (`npm uninstall` dans le même commit D5) :

```
@antv/g6
@nivo/chord @nivo/core @nivo/heatmap @nivo/radar @nivo/sankey @nivo/treemap
@number-flow/react
@react-sigma/core
axios
d3-parliament-chart
graphology graphology-communities-louvain graphology-metrics
leaflet @types/leaflet
lenis
moment
motion
react-calendar-timeline
react-force-graph-2d react-force-graph-3d
react-leaflet
reagraph
recharts
sigma
```

**Dépendances ajoutées** (Phase A) :

```
zod@^3.24.2                    # coordinator API schema validation
@playwright/test@^1.49.0       # dev dep, Phase B+
```

**Dépendances conservées non-touchées** (déjà lean pour le shell) :

- `react@19.2`, `react-dom@19.2`
- `react-router-dom@7.14`
- `@tanstack/react-query@5.96`
- `zustand@5.0` (persist middleware déjà embedded)
- `tailwindcss@4.2` + `@tailwindcss/vite` + `tw-animate-css`
- `shadcn@4.2` (CLI), `cmdk@1.1` (command palette primitive)
- `class-variance-authority`, `clsx`, `tailwind-merge`
- `lucide-react` (icônes shadcn)
- `@radix-ui/*` (primitives shadcn)
- `@base-ui/react` (primitive shadcn v4 path)
- `@fontsource-variable/geist`
- devDeps : `@eslint/js`, `eslint`, `eslint-plugin-react-hooks`,
  `eslint-plugin-react-refresh`, `globals`, `typescript`,
  `typescript-eslint`, `vite`, `@vitejs/plugin-react`, `@types/react`,
  `@types/react-dom`, `@types/node`

---

## 3. Day 0 — commit plan avant Phase A

### Commit 1 — `docs(sprint5): kickoff + detailed plan`

Stage:
- `.planning/sprint5_kickoff.md` (déjà untracked)
- `.planning/sprint5_plan.md` (ce fichier)

Pas de code, juste docs. Ce commit gèle les décisions D1..D5.

### Commit 2 — `refactor(web): drop legacy cold-case UI, archive via git history` (D5 execution)

**Aucun ajout dans ce commit**, uniquement :
- `git rm` de la liste §2.5.2 « Fichiers supprimés »
- `npm uninstall` des 24 deps cold-case
- `npm install` pour régénérer `web/package-lock.json`

Après ce commit, `web/src/` est :
```
web/src/
├── components/
│   └── ui/           # 21 primitives shadcn
├── hooks/
│   └── use-mobile.ts # shadcn companion
├── lib/
│   └── utils.ts      # cn helper
├── index.css
├── main.tsx          # existe encore mais à réécrire Phase A
└── App.tsx           # existe encore mais à réécrire Phase A
```

`cd web && npm run build` **ne passera pas** après ce commit parce
que `main.tsx`/`App.tsx` importent des modules supprimés. C'est
attendu — le commit suivant (Phase A) réécrit ces 2 fichiers.

### Pas d'autre commit Day 0

Phase A commence immédiatement après Commit 2. Les extensions
backend (`running.json` writer, `/shell/discover`, `/worker-state`,
worker `state_writer`) sont des commits Phase A, pas Day 0.

---

## 4. Phase A — Shell chrome + registry + coordinator extensions (≈ Day 1-3)

**Objectif** : le shell boote, voit un coordinateur via
`/shell/discover`, affiche un placeholder pour les 4 pages
(Projects, Network, Browse, Curators). Le worker flush son état.

### 4.1 Fichiers Rust (worker-core extension pour D3)

**Nouveau** :
- `crates/nexus-worker-core/src/engine/state_writer.rs` (~120 LOC)
  - struct `WorkerStateSnapshot` derive Serialize matching §2.3 schema
  - fn `serialize_to(path: &Path) -> Result<()>` avec atomic write
  - fn `from_engine(engine: &Engine, last_task: Option<&TaskRecord>) -> Self`
  - tests :
    - `state_writer_emits_valid_snapshot` — build snapshot, round-trip serde
    - `state_writer_atomic_write_survives_permission_denied` — simule EACCES, assert no corrupted file
    - `state_writer_schema_version_is_1` — compile-time constant
    - `state_writer_includes_null_gpu_when_nvml_absent`

**Modifié** :
- `crates/nexus-worker-core/src/engine/runtime.rs`
  - Ajouter un appel à `state_writer::serialize_to(&path)` dans la
    boucle principale, toutes les 5s (utilise `tokio::time::interval`
    ou un compteur simple basé sur le tick existant)
  - Path : nouveau helper `nexus_grid_paths::worker_state_path()`
    qui retourne `~/.nexus-grid/worker/state.json`
  - **Ne pas bloquer la boucle** sur l'écriture (spawn tokio task
    si nécessaire)

- `crates/nexus-worker-core/src/engine/mod.rs` : re-export `state_writer`

- `crates/nexus-worker-core/src/paths.rs` (ou équivalent existant,
  à vérifier) : helper `worker_state_dir()` et `worker_state_file()`

**Pas modifié** : `crates/nexus-worker/` (le binaire) ne change pas
sauf si un nouveau flag CLI est nécessaire (probablement non —
state_writer est toujours on, pas de flag).

### 4.2 Fichiers Python (coordinator extensions pour D1 + D3 proxy + D4 discover)

**Modifié** :
- `packages/nexus-coordinator/src/nexus_coordinator/paths.py`
  - Ajouter `running_state_path(project_name: str) -> Path` →
    `project_dir(project_name) / "running.json"`
  - Ajouter `worker_state_path() -> Path` →
    `nexus_grid_root() / "worker" / "state.json"` (MIRROIR du
    path Rust — critique que les deux côtés matchent)

- `packages/nexus-coordinator/src/nexus_coordinator/cli/commands/start.py`
  - Dans `_run()`, après `await coord.start()` et AVANT
    `server_task = asyncio.create_task(...)` → écrire
    `running.json` via un nouveau helper `write_running_state(coord)`
  - Dans le bloc `finally:`, après `await coord.stop()` → supprimer
    le fichier via `unlink(missing_ok=True)`
  - Atomic write : tempfile + `os.replace`

- `packages/nexus-coordinator/src/nexus_coordinator/api/app.py`
  - Ajouter `include_router(shell_router)` et `include_router(worker_state_router)`

**Nouveau** :
- `packages/nexus-coordinator/src/nexus_coordinator/registry.py` (~80 LOC)
  - `class RunningState(BaseModel)` — schema §2.1, `schema_version: Literal[1]`
  - `def write_running_state(coord: Coordinator) -> None` — atomic write
  - `def remove_running_state(project_name: str) -> None` — best-effort unlink
  - `def discover_running() -> list[RunningState]` — glob + parse,
    skip malformed entries avec log warning

- `packages/nexus-coordinator/src/nexus_coordinator/api/shell.py` (~60 LOC)
  - `router = APIRouter(prefix="/shell", tags=["shell"])`
  - `GET /shell/discover` → `{schema_version: 1, coordinators: [...], count: N}`
  - Réponse inclut un champ `self: {project_name, node_id, api_port}`
    indiquant quel coordinateur a servi la requête (utile au shell
    pour dédupliquer)

- `packages/nexus-coordinator/src/nexus_coordinator/api/worker_state.py` (~80 LOC)
  - `router = APIRouter(tags=["worker"])`
  - `GET /worker-state` → `{running: bool, stale?: bool, state?: WorkerStateV1}`
  - `class WorkerStateV1(BaseModel)` — schema §2.3, strict validation
  - Lecture via `worker_state_path()`, fallback `{running: false}` si
    absent ou parse failure (log warning)
  - Staleness check : `now - last_updated_at > 15s` → `stale: true`

- `packages/nexus-coordinator/tests/test_registry.py` :
  - `test_running_json_written_on_start` — spawn coord subprocess,
    assert file exists, parse matches schema
  - `test_running_json_removed_on_clean_stop` — SIGINT, assert file gone
  - `test_running_json_atomic_write_no_corruption` — concurrent writes
    race, assert final state valid
  - `test_stale_running_json_ignored_by_discover_on_parse_error` —
    write garbage to file, call `discover_running()`, assert warning logged

- `packages/nexus-coordinator/tests/test_shell_discover.py` :
  - `test_shell_discover_returns_running_coordinators` — boot 2 coordinators
    sur ports 18765 + 18766, hit `/shell/discover` sur l'un, assert
    les 2 sont listés
  - `test_shell_discover_excludes_self_marked_entry` — vérifie le field `self`
  - `test_shell_discover_empty_when_no_running_file` — tmp_path vide, assert `count: 0`

- `packages/nexus-coordinator/tests/test_worker_state_proxy.py` :
  - `test_worker_state_returns_false_when_file_absent` — assert `{running: false}`
  - `test_worker_state_parses_valid_snapshot` — write fixture à
    `worker_state_path()`, hit endpoint, assert body
  - `test_worker_state_marks_stale_when_old` — write fixture avec
    `last_updated_at` 20s dans le passé, assert `stale: true`
  - `test_worker_state_handles_malformed_json` — write garbage,
    assert `{running: false}` + warning

### 4.3 Fichiers Frontend

**Supprimés par Commit 2 (D5)** — voir §2.5.2. Après Commit 2, on
repart de `main.tsx` et `App.tsx` vides.

**Réécrits** :
- `web/src/main.tsx` (~25 LOC) :
  - `createRoot(document.getElementById("root")!)` + `<StrictMode>`
  - `<QueryClientProvider>` + `<RouterProvider>` (si data router) OU
    `<BrowserRouter>` + `<Routes>` (si declarative)
  - **Décision**: déclaratif suffit Sprint 5 (pas de loaders, data
    via React Query dans les composants — plus simple pour le MVP)
  - Font `@fontsource-variable/geist`

- `web/src/App.tsx` (~50 LOC) :
  - 5 routes plates : `/` (redirect `/my-projects`), `/my-projects`,
    `/project/:name`, `/my-network`, `/browse`, `/curators`
  - Wrapper `<AppShell>` fournit sidebar + header + outlet
  - Pas de loaders (React Query gère le fetching)

**Nouveau** :

- `web/src/index.css` (modifié — audit pour retirer var cold-case
  comme `--bg-primary` custom etc. et ne garder que les tokens
  tailwind v4 + shadcn)

- `web/src/api/coordinator.ts` (~250 LOC) — **client TS strict**
  - Schémas zod pour toutes les réponses coordinator :
    - `HealthSchema = z.object({ node_id: z.string(), doc_id: z.string(), project_name: z.string(), state: z.string() })`
    - `ProjectSchema`, `TasksListSchema`, `KudosListSchema`,
      `KudosVerifySchema`, `InviteRecordSchema`, `AppSummarySchema`,
      `AppManifestResponseSchema`, `ShellDiscoverSchema`, `WorkerStateResponseSchema`
  - Types exportés : `export type Health = z.infer<typeof HealthSchema>` etc.
  - Fonctions : `getHealth(baseUrl)`, `getProject(baseUrl)`,
    `listTasks(baseUrl, {state?, limit?})`, `submitTask(baseUrl, body)`,
    `listKudos(baseUrl, {workerPubkey?})`, `verifyKudos(baseUrl)`,
    `createInvite(baseUrl, body)`, `listInvites(baseUrl)`,
    `revokeInvite(baseUrl, inviteId)`, `listApps(baseUrl)`,
    `getAppManifest(baseUrl, name)`, `shellDiscover(baseUrl)`,
    `getWorkerState(baseUrl)`
  - Chaque fn : `fetch` → `await res.json()` → `Schema.safeParse(data)` →
    `if (!parsed.success) throw new CoordinatorProtocolError(...)`
  - `class CoordinatorProtocolError extends Error` avec champ `issues` de zod
  - **Aucun `fetch` nu** ailleurs dans le shell

- `web/src/stores/projectStore.ts` (~80 LOC)
  - Zustand 5 + persist middleware (forme curried `create<T>()(persist(...))`)
  - State : `{ knownCoordinators: Array<{url: string, nickname?: string}>, activeCoordinatorUrl: string | null }`
  - Actions : `addCoordinator(url, nickname?)`, `removeCoordinator(url)`,
    `setActive(url)`
  - `persist({ name: "nexus-grid:shell:v1", storage: createJSONStorage(() => localStorage) })`
  - Pas de `partialize` — on persiste tout le state

- `web/src/components/AppShell.tsx` (~200 LOC)
  - Layout wrapper : sidebar à gauche + header en haut + `<Outlet />`
  - Reutilise `web/src/components/ui/sidebar.tsx` (primitif shadcn
    conservé par D5)
  - Sidebar navigation (4 entrées) : Projects, Network, Browse, Curators
  - Header : dropdown « Coordinateur actif » + status dot
    (`/health` polling 5s via React Query) + bouton « + Add coordinator »
  - Branding : texte `nexus-grid` sans mention cold-case

- `web/src/components/AddCoordinatorDialog.tsx` (~100 LOC)
  - Dialog shadcn avec input URL + bouton « Tester » + « Ajouter »
  - Tester : `getHealth(url)`, affiche ✓ ou ✗
  - Ajouter : `projectStore.addCoordinator(url)` + `setActive(url)`

- `web/src/pages/OnboardingEmpty.tsx` (~80 LOC)
  - Rendu si `knownCoordinators.length === 0`
  - Explique « Lance ton premier projet » + commandes CLI copiables
  - CTA « J'ai déjà un coordinateur → Entre son URL » → ouvre
    `AddCoordinatorDialog`

- `web/src/pages/Projects.tsx` (~30 LOC, stub Phase A)
  - Affiche « Phase B (coming) » + 1 card listant les coordinators
    connus avec leur nickname et bouton « Ouvrir »
  - Le rendu riche vient Phase B

- `web/src/pages/ProjectDetail.tsx`, `web/src/pages/Network.tsx`,
  `web/src/pages/Browse.tsx`, `web/src/pages/Curators.tsx` :
  stubs minimaux Phase A (les 3 dernières rendent un `<Card>`
  explicatif).

### 4.4 Tests Phase A

**Python** :
- `test_registry.py` (4 tests listés §4.2)
- `test_shell_discover.py` (3 tests listés §4.2)
- `test_worker_state_proxy.py` (4 tests listés §4.2)

**Rust** :
- `state_writer::tests` (4 tests listés §4.1)
- Tester via `cargo test -p nexus-worker-core --lib state_writer`

**TypeScript** :
- `web/src/api/coordinator.test.ts` (unit) :
  - `ShellDiscoverSchema` valide et invalide via `safeParse`
  - `HealthSchema` valide et invalide
  - `CoordinatorProtocolError.issues` populated au parse fail
- `web/src/stores/projectStore.test.ts` (unit) :
  - `addCoordinator` dédupe par URL
  - `setActive` rejette si URL inconnue
  - persist snapshot roundtrip

Pas de Playwright Phase A (arrive Phase B).

### 4.5 Dépendances ajoutées Phase A

**Rust** (`crates/nexus-worker-core/Cargo.toml`) :
- Aucun nouveau crate — `serde`, `serde_json`, `chrono` déjà présents

**Python** (`packages/nexus-coordinator/pyproject.toml`) :
- Aucun nouveau package — `pydantic`, `platformdirs` déjà présents

**Frontend** (`web/package.json`) :
- `zod@^3.24.2` (runtime)
- `vitest@^2.1.0` + `@vitest/ui` (devDep, pour unit tests TS) —
  **vérifier context7 avant** : Vitest 2 est stable, API
  compatible avec Vite 8

### 4.6 Critère de fermeture Phase A

- [ ] `pytest packages/nexus-coordinator/tests/test_registry.py` vert
- [ ] `pytest packages/nexus-coordinator/tests/test_shell_discover.py` vert
- [ ] `pytest packages/nexus-coordinator/tests/test_worker_state_proxy.py` vert
- [ ] Tests Sprint 4 coordinator toujours verts (`pytest packages/nexus-coordinator/tests/ -q` ≥ 27 + nouveaux)
- [ ] `cargo test -p nexus-worker-core --lib state_writer` vert
- [ ] `cargo test --workspace --exclude nexus-core-py` encore vert (≥ 166 + state_writer tests)
- [ ] `cd web && npm install && npm run build` exit 0, 0 warnings
- [ ] `cd web && npm run lint` exit 0
- [ ] `cd web && npx tsc --noEmit` exit 0
- [ ] `cd web && npm run dev` → page `http://127.0.0.1:3002` affiche
      l'onboarding « Lance ton premier projet » si aucun coord actif
- [ ] Smoke test manuel : `uv run nexus-coordinator init smoke && start smoke --port 18765`,
      puis shell + « Add coordinator http://127.0.0.1:18765 »,
      vérifier que le status dot passe au vert et que
      `/my-projects` affiche le stub avec 1 entry

---

## 5. Phase B — /my-projects + /project/:name détail (≈ Day 4-5)

**Objectif** : livrer les 2 pages les plus importantes du shell,
testées e2e contre un vrai coordinateur + vraie app.

### 5.1 Backend (petit patch)

**Modifié** :
- `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py` :
  ajouter `GET /app/{name}/tabs/{tab_name}/descriptor` qui
  invoque le tab fn (sync ou async) et retourne son résultat.
  Si le tab n'existe pas → 404.
  Tests : `test_apps_async_tab_descriptor_endpoint` dans
  `packages/nexus-coordinator/tests/test_apps.py`.

### 5.2 Frontend

**Réécrit (était stub Phase A)** :
- `web/src/pages/Projects.tsx` (~200 LOC) :
  - React Query : `useQuery` pour chaque coordinateur connu, hit
    `/shell/discover` sur le coordinateur actif pour enrichir
  - Card shadcn par coordinateur : nom, `node_id` tronqué 8 chars,
    visibility badge, state, healthy dot (polling `/health` 5s)
  - Actions par card : « Ouvrir » (→ `/project/:name`) + « Retirer »
  - CTA haut : « + Add coordinator » + « + New project »
    (le second ouvre un modal shadcn avec les instructions CLI
    copiables via `lucide-react:Copy`)

- `web/src/pages/ProjectDetail.tsx` (~300 LOC)
  - Route `/project/:name`, charge via
    `useQuery(["project", name], () => getProject(baseUrl))` +
    `useQuery(["health", name], () => getHealth(baseUrl))`
  - `<Tabs>` shadcn avec 5 panels :
    - **Overview** : `/project` + `/health` combinés, counter
      workers distinct (dérivé du kudos ledger :
      `new Set(entries.map(e => e.worker_pubkey_hex))`)
    - **Tasks** : table shadcn paginée via `GET /tasks`
    - **Kudos** : table ledger via `GET /kudos`, en haut
      status intégrité via `GET /kudos/verify` (badge vert ✓ ou
      rouge ✗ avec `first_bad_row_id`)
    - **Invites** : liste via `GET /invite`, formulaire
      `POST /invite/create` en dialog shadcn, action « Revoke » via
      `DELETE /invite/{id}`
    - **Apps** : liste via `GET /app`. Chaque app est un
      `<Accordion>` qui expand vers le manifest
      (`GET /app/{name}/manifest`) affiché en sections :
      - Routes (liste avec méthode + path, chaque route cliquable
        → fetch `/app/{name}{path}` et affiche la réponse brute
        dans un `<Dialog>` avec `<pre>`)
      - Workers (liste name + model)
      - Tabs (liste name + icon ; clic → ouvre un panneau avec
        `<pre>{JSON.stringify(descriptor, null, 2)}</pre>` ; si le
        descriptor est la note async, afficher un bouton
        « Invoquer » qui hit
        `GET /app/{name}/tabs/{tab_name}/descriptor`)

- `web/src/components/NewProjectDialog.tsx` (~120 LOC)
  - Dialog shadcn : explique que le shell ne spawn pas de process
    (D4), liste 3 commandes copiables :
    ```bash
    uv run nexus-coordinator init <name>
    uv run nexus-coordinator start <name>
    # Puis dans le shell : « Add coordinator http://127.0.0.1:8765 »
    ```

- `web/src/components/KudosIntegrityBadge.tsx` (~40 LOC)
- `web/src/components/TasksTable.tsx` (~120 LOC) — pagination
  client-side simple (limit=100)
- `web/src/components/InviteCreateDialog.tsx` (~150 LOC)
- `web/src/components/AppManifestView.tsx` (~200 LOC) — accordion
  route/workers/tabs + rendu JSON brut

### 5.3 Tests Phase B

**Playwright** (nouveau) :
- `web/tests/fixtures/coordinator.ts` — helper qui spawn
  `uv run --package nexus-coordinator nexus-coordinator start test --port 18765 --data-dir <tmp>`
  via `globalSetup`, wait for `/health` 200, expose la URL
- `web/tests/shell-onboarding-empty-state.spec.ts` :
  - clear localStorage, open `/`, expect onboarding visible
- `web/tests/shell-add-coordinator.spec.ts` :
  - open add dialog, entre URL du coord de test, « Tester » → ✓,
    « Ajouter » → redirect `/my-projects` avec 1 card
- `web/tests/my-projects-live.spec.ts` :
  - coord de test en place, open `/my-projects`, assert 1 card
    avec project name « test »
- `web/tests/project-detail-manifest.spec.ts` :
  - open `/project/test`, click tab Apps, expand `hello`, assert
    manifest rendu avec 1 route / 1 worker / 1 tab
- `web/tests/apps-tab-render.spec.ts` :
  - click sur le tab `Hello`, assert `<pre>` contient
    `"description": "Hello world"`

Install hello-world-app dans le venv de test via
`uv pip install -e examples/hello-world-app` (dans le globalSetup).

### 5.4 Critère de fermeture Phase B

- [ ] Playwright `shell-onboarding-empty-state.spec.ts` pass
- [ ] Playwright `shell-add-coordinator.spec.ts` pass
- [ ] Playwright `my-projects-live.spec.ts` pass (vrai coord)
- [ ] Playwright `project-detail-manifest.spec.ts` pass
- [ ] Playwright `apps-tab-render.spec.ts` pass
- [ ] `pytest packages/nexus-coordinator/tests/test_apps.py::test_apps_async_tab_descriptor_endpoint` pass
- [ ] `cd web && npm run build` encore vert
- [ ] 0 mock du coordinateur dans aucun test (R4)

---

## 6. Phase C — /my-network + worker state wiring (≈ Day 6-7)

**Objectif** : `/my-network` lit le vrai worker state (flush D3)
via `/worker-state` (proxy D3+D4), polling 2s, données live.

### 6.1 Backend

**Déjà fait Phase A** : `/worker-state` endpoint + worker
`state_writer`. Phase C ajoute un test end-to-end Python qui
spawn un `nexus-worker start --stub-ollama` réel et vérifie que
le fichier `state.json` est écrit et lisible par le coordinator.

**Nouveau test** :
- `packages/nexus-coordinator/tests/test_worker_state_roundtrip.py` :
  - `test_worker_state_roundtrip_with_stub_ollama` :
    - spawn `nexus-worker start --stub-ollama --data-dir <tmp>`
    - attendre 10s
    - spawn coord, hit `GET /worker-state`, assert
      `running: true`, `stale: false`, `state.node_id` non vide,
      `state.gpu` non null si machine a un GPU

### 6.2 Frontend

**Réécrit (était stub Phase A)** :
- `web/src/pages/Network.tsx` (~250 LOC)
  - `useQuery(["worker-state"], () => getWorkerState(baseUrl), { refetchInterval: 2000 })`
  - Si `!data.running` → grande Card « Worker non détecté »
    avec commande copiable `nexus-worker start`
  - Sinon, 4 cards shadcn :
    - **Worker identity** : `node_id` tronqué 16 chars (copy on click),
      `worker_version`, `uptime` formaté humain (1h 23m 45s)
    - **GPU** : nom, progress bar memory, progress bar utilization,
      temp, power. Si `gpu: null` → message « Aucun GPU détecté »
    - **Projects served** : table `name / kudos_total / tasks_completed`
    - **Last task** : `task_id` tronqué, prompt preview, status badge,
      `completed_at` formaté relatif (« il y a 3s »)
  - Si `stale: true` → bandeau warning « Dernière mise à jour > 15s,
    le worker pourrait être figé »

- `web/src/lib/format.ts` (~80 LOC) — helpers purs :
  - `formatNodeId(hex: string, chars: number = 16): string`
  - `formatUptime(secs: number): string`
  - `formatRelativeTime(iso: string): string`
  - `formatBytes(mb: number): string`
  - tests unitaires `web/src/lib/format.test.ts`

### 6.3 Tests Phase C

**Python** :
- `test_worker_state_roundtrip.py` (listé §6.1)

**Playwright** :
- `web/tests/my-network-live.spec.ts` :
  - globalSetup spawn coord ET
    `cargo run -p nexus-worker -- start --stub-ollama --data-dir <tmp>`
  - attente 7s pour le premier flush
  - open `/my-network`, assert 4 cards visibles, assert `node_id`
    non vide, assert `stale: false`

**Unit TS** :
- `web/src/lib/format.test.ts` (4 tests pour les helpers)

### 6.4 Critère de fermeture Phase C

- [ ] `test_worker_state_roundtrip.py` pass avec vrai worker subprocess
- [ ] Playwright `my-network-live.spec.ts` pass
- [ ] `web/src/lib/format.test.ts` pass
- [ ] Démo manuelle : 3 terminaux (coord, worker `--stub-ollama`, dev server)
      → `/my-network` affiche des données live qui se rafraîchissent

---

## 7. Phase D — stubs + polish + verification (≈ Day 8-10)

### 7.1 Stubs Sprint 6

- `web/src/pages/Browse.tsx` (~80 LOC)
  - Card explicative : « La découverte DHT pkarr arrive Sprint 6
    avec le daemon nexus-shell »
  - Mentionne les décisions figées (§2.4) et pointe vers le
    design dans ce plan

- `web/src/pages/Curators.tsx` (~80 LOC)
  - Card explicative : « Les curator lists arrivent avec le gossip
    flow Sprint 6 »
  - Liste les features futures (ajout/retrait curator, signature
    vérifiée Ed25519)

### 7.2 Polish

- **i18n français** : audit complet via grep de strings anglaises
  dans `web/src/**/*.{ts,tsx}`. Toutes les strings user-facing en
  français. Exceptions : identifiants techniques (`node_id`,
  `doc_id`, `task_id`, `pubkey`), messages d'erreur protocole
  coordinator (laissés en anglais car ils viennent du backend).
  - Script : `web/scripts/scan-en-strings.sh` (regex grep simple)
  - Enforced via row 20 du tableau fail-fast §8

- **Dark theme shadcn** : vérifier `web/src/index.css` alignment
  avec les tokens shadcn v4 (CSS variables). Retirer toute var
  cold-case (`--bg-primary` custom, etc.)

- **Keyboard shortcuts** : via `cmdk` (déjà dans les primitives
  shadcn) :
  - `Ctrl+K` / `Cmd+K` → command palette (liste coordinators,
    navigation directe aux 4 pages, actions rapides add/remove)
  - `g p` → `/my-projects`
  - `g n` → `/my-network`
  - `g b` → `/browse`
  - `g c` → `/curators`
  - Implémenté via `useHotkey` maison dans `web/src/lib/hotkeys.ts`

- **Responsive ≥ 1280px** : le shell cible desktop, pas mobile.
  Vérifier les pages à 1280×800 et 1920×1080 via Playwright
  viewport. Tolérer le breakage < 1280 (message « Shell desktop
  uniquement »).

- **Bundle size audit** :
  `npx vite-bundle-visualizer` (dev dep temporaire ou via
  `npx` sans install). Cible : < 500 KB gzipped pour le
  shell sans les pages apps-tab descriptors.
  **Pas de tests automatisés** sur la taille — juste un
  check manuel + note dans le verification doc.

- **`npm audit`** : 0 high/critical. Si des vulns apparaissent
  post-D5 dans les deps conservées → créer entry tech debt
  dans `docs/shell/PATTERNS.md` (nouveau fichier Phase D) ET
  décider fix-maintenant vs Sprint 6.

### 7.3 Documentation shell

**Nouveau** :
- `docs/shell/PATTERNS.md` (~200 LOC) — miroir de
  `docs/rust/PATTERNS.md` : patterns React/TS, règles sur
  `coordinator.ts`, interdictions (`as any`, fetch nu),
  notes sur les décisions D1..D5 figées

### 7.4 Verification document

**Nouveau** :
- `.planning/sprint5_verification.md` — tableau 20 lignes fail-fast
  (§8 ci-dessous), chaque ligne avec la commande exacte + le
  résultat observé

### 7.5 Tests Phase D

**Playwright** :
- `web/tests/stub-pages.spec.ts` :
  - open `/browse`, assert Card explicative visible, pas de 404
  - open `/curators`, assert idem
- `web/tests/shell-onboarding-to-manifest-e2e.spec.ts` :
  - scenario complet : clear localStorage, add coord, voit app
    hello, click tab, voit descriptor — critère de succès
    Sprint 5 global

**Script scan** :
- `web/scripts/scan-en-strings.sh` — invoqué par row 20 du
  tableau fail-fast, grep liste limitée de mots anglais courants
  dans les fichiers `.tsx`, whitelist les identifiants techniques

### 7.6 Critère de fermeture Phase D

- [ ] `/browse` et `/curators` rendent un Card placeholder, pas
      de 404
- [ ] E2E Playwright scénario onboarding complet pass en < 60s
- [ ] `.planning/sprint5_verification.md` rédigé, 20/20 rows PASS
- [ ] `npm audit` 0 high/critical
- [ ] 0 string UI anglaise hors identifiants techniques
- [ ] `docs/shell/PATTERNS.md` présent, cité dans les commits

---

## 8. Tableau fail-fast Sprint 5 (livrable final)

À produire dans `.planning/sprint5_verification.md` à la fin de
Phase D, avec colonne « Observed » remplie par les runs réels.

| # | Check | Commande | Critère |
|---|---|---|---|
| 1 | running.json écrit au start | `pytest packages/nexus-coordinator/tests/test_registry.py::test_running_json_written_on_start` | pass |
| 2 | running.json retiré au stop | `pytest packages/nexus-coordinator/tests/test_registry.py::test_running_json_removed_on_clean_stop` | pass |
| 3 | /shell/discover liste les running | `pytest packages/nexus-coordinator/tests/test_shell_discover.py` | pass (3/3) |
| 4 | /worker-state proxy valide + stale | `pytest packages/nexus-coordinator/tests/test_worker_state_proxy.py` | pass (4/4) |
| 5 | worker state_writer | `cargo test -p nexus-worker-core --lib state_writer` | pass (4/4) |
| 6 | worker state roundtrip e2e | `pytest packages/nexus-coordinator/tests/test_worker_state_roundtrip.py` | pass |
| 7 | legacy cold-case pages removed | `test ! -f web/src/pages/Dashboard.tsx && test ! -f web/src/pages/Evidence.tsx` | exit 0 |
| 8 | zero legacy deps | `grep -E "(antv/g6\|leaflet\|recharts\|reagraph\|sigma\|nivo\|force-graph\|graphology\|parliament\|@number-flow\|axios\|moment\|motion\|lenis)" web/package.json` | exit 1 (0 matches) |
| 9 | TypeScript strict clean | `cd web && npx tsc --noEmit` | exit 0 |
| 10 | ESLint clean | `cd web && npm run lint` | exit 0 |
| 11 | Build prod clean | `cd web && npm run build` | exit 0, no warnings |
| 12 | Shell onboarding empty state | Playwright `shell-onboarding-empty-state.spec.ts` | pass |
| 13 | Add coordinator flow | Playwright `shell-add-coordinator.spec.ts` | pass |
| 14 | /my-projects live | Playwright `my-projects-live.spec.ts` | pass (live coord) |
| 15 | /project/:name manifest | Playwright `project-detail-manifest.spec.ts` | pass |
| 16 | Apps tab render | Playwright `apps-tab-render.spec.ts` | pass |
| 17 | /my-network reads worker live | Playwright `my-network-live.spec.ts` | pass |
| 18 | /browse + /curators stubs | Playwright `stub-pages.spec.ts` | pass |
| 19 | All Rust tests | `cargo test --workspace --exclude nexus-core-py` | ≥ 170 total (166 + 4 state_writer) |
| 20 | All Python coordinator tests | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q` | ≥ 38 total (27 Sprint 4 + ~11 Sprint 5) |
| 21 | French UI | `bash web/scripts/scan-en-strings.sh` | 0 strings EN dans `web/src/` hors whitelist |
| 22 | E2E global shell → app | Playwright `shell-onboarding-to-manifest-e2e.spec.ts` | pass en < 60s |

22 lignes (le kickoff disait « 15-18 » ; on a pris +4 pour couvrir
D1..D5 plus finement).

---

## 9. Questions ouvertes à trancher avec le user

Avant de commencer Day 0, valider explicitement chacun :

### Q1 — D1 : `running.json` schema gelé au v1 ?

Le `schema_version: 1` ci-dessus fige les champs. Ajout de
`description` (user-set) ou `tags[]` → bump v2 obligatoire.
**Décision demandée** : OK pour v1 minimal tel quel, ou ajouter
`description` Phase A ?

### Q2 — D2 : endpoint `GET /app/{name}/tabs/{tab_name}/descriptor` en Phase B ?

L'ajout de cet endpoint est la seule modif non-triviale du
backend apps.py côté Phase B. Alternative : laisser les tabs
async afficher la note et déférer l'invocation à Sprint 6.
**Décision demandée** : ajout Phase B (recommandé — le test
Playwright Apps l'utilise), ou report Sprint 6 ?

### Q3 — D3 : cadence du flush worker state → 5s OK ?

5s est un compromis raisonnable : assez réactif pour /my-network
à 2s polling, pas trop agressif pour éviter d'user les SSD.
Alternative 3s (plus réactif) ou 10s (plus économique).
**Décision demandée** : 5s, autre valeur, ou paramétrable via
`coordinator.toml` Phase A (préféré — on met 5s default et ajout
d'un field `WorkerConfig.state_flush_secs`) ?

### Q4 — D5 : `axios` retiré en même temps ?

`axios` est utilisé par l'ancien `web/src/api/client.ts`. Tous
les nouveaux clients utilisent `fetch` natif + zod. Retirer
`axios` au moment du `npm uninstall` Day 0 Commit 2.
**Décision demandée** : confirmer retrait (recommandé).

### Q5 — Phase D tests : viewport desktop-only OK ?

Les tests Playwright et le shell ciblent 1280×800 minimum, pas
de responsive mobile/tablette. Alignement avec le use case
(desktop dev power-user).
**Décision demandée** : confirmer desktop-only ou exiger
breakpoint mobile (ajoute ~1 jour scope) ?

### Q6 — Phase D : `docs/shell/PATTERNS.md` obligatoire ?

Sprint 4 a créé `docs/rust/PATTERNS.md`, Phase D Sprint 5 peut
créer l'équivalent shell. Alternative : skip et ajouter si un
pattern non-trivial émerge pendant l'implémentation.
**Décision demandée** : créer d'office Phase D (recommandé) ou
lazy-create ?

---

## 10. Ce que le plan NE fait PAS (scope explicitement hors Sprint 5)

Cohérent avec les scope cuts `.planning/sprint4_verification.md` §
« What's NOT in this sprint » :

- **`nexus-shell-daemon`** — sidecar autonome avec Node iroh pour
  DHT discovery + curator gossip. Reporté Sprint 6 avec design
  posé §2.4 D4.
- **Rendu schema-driven des tabs d'app** — vocabulaire listé §2.2
  D2 mais non codé. Sprint 6 l'implémente et migre hello +
  gov.
- **Curator list flow** — signatures Ed25519, ajout/retrait
  curator, subscribe/unsubscribe, stub Sprint 5, Sprint 6 complet.
- **DHT browse (pkarr)** — stub Sprint 5, Sprint 6 complet via
  nexus-shell-daemon.
- **Migration 19-tab `nexus-app-gov`** — toujours reportée v1.1
  conforme à `.planning/sprint4_verification.md` « gov migration
  v1.1 ».
- **Worker HTTP API (axum)** — rejeté D3, remplacé par state.json
  flush + proxy coordinator. Sprint 6+ réévalue si besoin.
- **Spawn de processes depuis le shell** — interdit par D4, le
  shell n'est pas un wrapper Tauri Sprint 5. Sprint 7+ si le
  produit desktop devient prioritaire.
- **Mobile/responsive < 1280px** — desktop-only confirmé Q5.
- **Tests de charge** — pas de benchmark frontend Sprint 5.
  Arrive Sprint 7+ post-release.
- **i18n multi-langue** — français en dur, anglais pour Sprint 7+.
- **Auth/RBAC sur le shell** — le shell parle en localhost au
  coordinateur de l'utilisateur, aucune auth nécessaire. Sprint
  8+ pour un mode hosted.

---

## 11. Règles opérationnelles Sprint 5 (rappel)

Les règles R1..R7 du kickoff §6 sont reprises sans modification :

- **R1** — context7 OBLIGATOIRE avant tout code contre une lib
  non-triviale. Le tableau §1 ci-dessus liste les libs DÉJÀ
  consultées pour ce plan ; chaque phase ajoute ses queries
  (TanStack Query 5 + React 19 Suspense, Tailwind 4 CSS config,
  shadcn v4 CLI, Playwright 1.49+ fixtures async) avant le
  premier commit qui les utilise.
- **R2** — pas de band-aid. Cause racine ou entry tech debt dans
  `docs/shell/PATTERNS.md` (Phase D) OU
  `docs/coordinator/PATTERNS.md` OU `docs/rust/PATTERNS.md` AVEC
  décision fix-maintenant vs Sprint 6. Interdits :
  `try/catch { /* ignore */ }`, `as any`, `@ts-ignore`, fetch
  nu, hardcoded URL/port, commit « WIP/tmp/hack », skip de test,
  mock de coordinateur.
- **R3** — global pas local. Relire le plan des 4 phases avant
  d'écrire une phase. Les schémas gelés §2 ne bougent pas.
- **R4** — tests d'intégration contre vrai coordinator + vrai
  worker. Pattern subprocess Python + spawn Playwright globalSetup.
  Les unit tests sont acceptables pour helpers purs, Zod schemas,
  primitives wrappées.
- **R5** — commits atomiques par phase, format
  `feat(web|coordinator|worker-core): Sprint 5 Phase <A|B|C|D> — <résumé>`.
  Day 0 = 2 commits (`docs` + `refactor` D5). Pas de mega-commit.
- **R6** — NE PAS toucher : `nexus/` legacy, apps gov/hello
  (consommer seulement), `crates/nexus-core-rs`, `crates/nexus-core-py`,
  `crates/nexus-worker/` binaire (sauf wire ≤ 10 lignes pour
  state_writer), `docker-compose.yml`, `phoenix.md`, archives
  Sprint 0-4.
- **R7** — verification continue à la fin de chaque phase :
  ```bash
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test -p nexus-worker-core --lib
  cargo test --workspace --exclude nexus-core-py
  uv run ruff format --check packages/ examples/
  uv run ruff check packages/ examples/
  uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q
  cd web && npm run lint && npx tsc --noEmit -p tsconfig.json && npm run build
  cd web && npx playwright test   # Phase B+
  ```
  Un rouge → stop. Jamais de `test.skip()`.

---

## 12. Prochaine action

1. **Soumettre ce plan au user** pour validation explicite de :
   - D1 (running.json schema §2.1)
   - D2 (rendu tab stub §2.2)
   - D3 (worker state.json flush 5s §2.3)
   - D4 (pas de sidecar Sprint 5 §2.4)
   - D5 (suppression nette §2.5 avec liste exhaustive)
   - Q1..Q6 §9

2. **Après validation** :
   - Commit 1 `docs(sprint5): kickoff + detailed plan` (ce fichier + kickoff)
   - Commit 2 `refactor(web): drop legacy cold-case UI, archive via git history` (D5)
   - Phase A commence

Aucune ligne de code Rust/Python/TS avant Commit 1 + 2.
