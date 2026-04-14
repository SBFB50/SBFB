# Sprint 8 — Plan détaillé (nexus-app-gov v1.1 migration 19 tabs)

**Écrit** : 2026-04-11 à partir de `.planning/sprint8_kickoff.md`
après gel des décisions Day 0 D1..D5. Ce document est la grille
d'exécution Sprint 8 : chaque commit cite la phase, chaque test est
listé ici, chaque fichier touché est nommé. **Aucun code Phase A..F
n'est écrit avant que cette grille soit commitée**
(`docs(sprint8): kickoff + plan`).

**HEAD entrée** : `2ed0955`. Working tree clean modulo
`.planning/sprint8_kickoff.md` + ce fichier (commités ensemble en
ouverture via `docs(sprint8): kickoff + plan`).

**Goal Sprint 8 (une phrase)** : migrer `nexus-app-gov` de 1 tab stub
à 19 tabs TabView-native, implémenter `AppContext.submit_task` et
`@nexus_command` (Sprint 7 D4/D5 frozen), brancher `AppContext.db`
comme bridge SQLite, retirer `legacy_descriptor` fallback, sans
ajouter les 4 infra items déférés à Sprint 9.

---

## 1. État vérifié à l'entrée

### 1.1 Sprint 7 livré + audit gate DONE (source : `sprint7_audit_findings.md`)

- Sprint 7 fermé à `9cc0796`, puis 1 commit de gate jusqu'à
  `2ed0955` : findings doc PASS (706 lignes, 17 findings)
- **304 Rust** tests workspace (62 core-rs lib + 6 doctests + 105
  worker-core lib + 11 worker bin + 10 worker e2e + 62 shell-daemon-core
  + 27 shell-daemon bin + 6 shell-daemon e2e + 13 curator primitives
  + 2 probe_reachable)
- **100 Python** tests + 1 skipped (SDK 40 + coord 57+1 + app-gov 3)
- **114 Vitest** tests web/
- **13 Playwright** specs en ~16 s contre un coordinator live
- `cargo fmt --all --check` + `cargo clippy --workspace --all-targets
  --locked -- -D warnings` clean
- `ruff format --check` + `ruff check` clean
- `tsc --noEmit -p tsconfig.app.json` clean
- `npm run lint` clean (0 err + 5 T1 warnings tolérés)
- `npm run build` : main 466 / vendor-react 189 / vendor-ui 31 /
  css 93 KB, zéro warning ; budgets D5 serrés
- `bash web/scripts/scan-en-strings.sh` clean
- `npm audit` : 0 vulns all levels

### 1.2 Consommé directement par Sprint 8

**Rust workspace** (`crates/`) :

| Crate | LOC | Rôle Sprint 8 |
|---|---|---|
| `nexus-core-rs` | ~3850 | **Étendu Phase A** : A-4 string caps dans `curator.rs` |
| `nexus-core-py` | ~1050 | **Inchangé** |
| `nexus-worker-core` | ~3500 | **Inchangé** |
| `nexus-worker` | ~800 | **Inchangé** |
| `nexus-shell-daemon-core` | ~1800 | **Étendu Phase A** : C-2 error split, D-1 process_name tighten |
| `nexus-shell-daemon` | ~900 | **Étendu Phase A** : G-3 deny_unknown_fields sur DTOs |

**Python SDK** (`packages/nexus-sdk/`) :

| Fichier | Rôle Sprint 8 |
|---|---|
| `src/nexus_sdk/app.py` | Phase A : extend `AppContext` (submit_task + app_name + db), ajouter `NexusApp.resolve_worker` + `commands()` |
| `src/nexus_sdk/decorators.py` | Phase A : +`nexus_command` decorator |
| `src/nexus_sdk/commands.py` *(nouveau)* | Phase A : +`CommandDescriptor` Pydantic |
| `src/nexus_sdk/db.py` *(nouveau)* | Phase B : +`AppDatabaseClient` aiosqlite wrapper |
| `src/nexus_sdk/registry.py` | Phase A : collect_decorators sait lire `commands` en plus de routes/workers/tabs |
| `src/nexus_sdk/__init__.py` | Phase A : export `nexus_command`, `CommandDescriptor`, `AppDatabaseClient` |

**Python coordinator** (`packages/nexus-coordinator/`) :

| Fichier | Rôle Sprint 8 |
|---|---|
| `api/apps.py` | Phase A : retrait `_coerce_tab_view` + `legacy_descriptor_sweep` (D4); +3 routes (`POST /app/{name}/tasks/submit`, `GET /app/{name}/commands`, `POST /app/{name}/commands/{cmd}/invoke`) |
| `loader.py` | Phase B : initialisation `ctx.db` avec path résolu + override support |
| `paths.py` | Phase B : `app_db_path(project_name, app_name)` helper |
| `dispatcher.py` | Phase A : accepte un `parent_task_id` optionnel |

**Python app-gov** (`packages/nexus-app-gov/`) :

| Fichier | Rôle Sprint 8 |
|---|---|
| `src/nexus_app_gov/app.py` | **Rewrite complet** : 19 `@nexus_tab` handlers + 3-5 `@nexus_command` + override `on_start` pour pointer `ctx.db` sur le fichier legacy |
| `src/nexus_app_gov/queries.py` *(nouveau)* | Queries SQL typées pour chaque tab |
| `src/nexus_app_gov/tabs/*.py` *(nouveau)* | Un module par tab si l'`app.py` devient trop gros (pattern à décider Phase B) |
| `src/nexus_app_gov/prompts.py` | Phase D : +prompts pour `rag_search` / `rag_ask` workers |

**Frontend** (`web/src/`) :

| Fichier | Rôle Sprint 8 |
|---|---|
| `api/coordinator.ts` | Phase A : `submitAppTask`, `listAppCommands`, `invokeAppCommand`, `CommandDescriptorSchema`, simplification `getAppTabDescriptor` (retrait `legacy`) |
| `api/daemon.ts` | Phase A : A-3 cross-lang fixture consumer |
| `components/CommandPalette.tsx` | Phase E : ajout 4e groupe "App" qui fetch toutes les commands des apps enrollées |
| `components/app/tabview/blocks/ButtonBlock.tsx` | Phase A : wire `task_submit` action via `submitAppTask` (retrait `console.warn` stub) |
| `components/app/tabview/schema.ts` | Phase A : reste inchangé (schema_version 1 preserved) |
| `pages/Curators.tsx` | Phase A : F-1 refresh button |

**Docs** :

| Fichier | Rôle Sprint 8 |
|---|---|
| `docs/shell/PATTERNS.md` | Phase F : +P10 (command palette app-contributed), update P8 (legacy retired), fermeture T4 + T5 |
| `docs/rust/PATTERNS.md` | Phase F : mise à jour si patterns Rust touchés Phase A (C-2 / D-1) |

### 1.3 Recherche consultée (détails §3)

| Lib | Usage Sprint 8 |
|---|---|
| `aiosqlite` (déjà dep coord) | Phase B — `AppDatabaseClient` wrapper |
| `sqlite3` stdlib | Phase B — schema introspection côté tests |
| Pydantic v2 | Phase A — `CommandDescriptor` model |
| React Query | Phase E — palette fetches commands avec `refetchInterval` |
| shadcn `Command*` (déjà vendored) | Phase E — palette 4e groupe |

---

## 2. Décisions Day 0 (D1..D5 gelées, cf `sprint8_kickoff.md` §4)

Résumé une-ligne par décision, détails dans le kickoff :

- **D1 — `AppContext.submit_task` impl Sprint 7 D4 frozen**. Wire
  vers `ComputeClient` via `NexusApp.resolve_worker(routing_key)`
- **D2 — `@nexus_command` impl Sprint 7 D5 frozen**. Decorator +
  `CommandDescriptor` Pydantic + 2 coord routes + Zod mirror +
  palette 4e groupe
- **D3 — `AppContext.db` bridge aiosqlite per-app**. Lecture
  principale, schema override via `on_start`. Pas de migration runner
- **D4 — Legacy descriptor fallback removal complet**. `_coerce_tab_view`
  + `legacy_descriptor_sweep` retirés, tabs retournent TabView
  valide ou HTTP 422
- **D5 — Scope: 19 tabs read-heavy + 4 infra items DÉFÉRÉS Sprint 9**.
  Pas de storage/events/upload/migration cette sprint

## 3. Research consulté (détails)

### 3.1 `NexusApp` + `@nexus_*` decorators pattern

**Pattern extrait** (lu dans `packages/nexus-sdk/src/nexus_sdk/app.py`
+ `decorators.py` + `registry.py`) :

- `@nexus_route(path, methods)` / `@nexus_worker(name, model)` /
  `@nexus_tab(name, icon)` attachent un dict attribut sur la méthode
- `collect_decorators(cls)` parcourt `dir(cls)` et bucket chaque
  méthode dans routes/workers/tabs selon les attributs
- `NexusApp.__init__` appelle `collect_decorators(type(self))` pour
  peupler `_routes`, `_workers`, `_tabs`
- `routes()` / `workers()` / `tabs()` retournent des descriptors
  typés

**À ajouter Sprint 8 Phase A** :

- `@nexus_command(name, description, icon, group)` avec le même
  pattern d'attribut `__nexus_command__` sur la méthode
- `collect_decorators` étendu pour bucket `commands` en plus des
  trois existants
- `NexusApp.commands() -> list[CommandDescriptor]` qui transforme
  les méthodes collectées

### 3.2 `ComputeClient::submit_task` surface actuelle

**Pattern extrait** (`compute_client.py`) :

```python
async def submit_task(
    self,
    *,
    task_type: str,
    prompt: str,
    model: str,
    system_prompt: str = "",
    priority: int = 5,
    metadata: dict[str, str] | None = None,
    task_id: str | None = None,
) -> SubmittedTask
```

`task_type` et `model` sont des strings libres que le dispatcher
route vers les workers enregistrés.

**Adapter Sprint 8 Phase A** : `AppContext.submit_task(worker,
payload, ...)` résout `worker` vers un `WorkerDescriptor` et délègue :

```python
async def submit_task(
    self, worker: str, payload: dict[str, Any],
    *, priority: int = 5, parent_task_id: str | None = None,
) -> str:
    desc = self._app.resolve_worker(worker)
    prompt = json.dumps(payload, sort_keys=True)
    task = await self.compute.submit_task(
        task_type=worker,
        prompt=prompt,
        model=desc.model,
        priority=priority,
        metadata={"parent_task_id": parent_task_id or ""},
    )
    return task.task_id
```

### 3.3 `_coerce_tab_view` actuel (pour confirmer D4 retrait)

**Pattern extrait** (`coordinator/api/apps.py:121-160`) :

```python
def get_tab_descriptor(name: str, tab_name: str) -> dict:
    descriptor = _invoke_tab_fn(name, tab_name)
    return _coerce_tab_view(descriptor, name, tab_name)

def _coerce_tab_view(descriptor, app_name, tab_name) -> dict:
    # TODO(Sprint 8): remove the legacy_descriptor fallback
    try:
        validated = TabView.model_validate(descriptor)
        return {"descriptor": validated.model_dump(), "legacy_descriptor": False}
    except ValidationError:
        logger.warning(...)
        return {"descriptor": descriptor, "legacy_descriptor": True}
```

**À retirer Sprint 8 Phase A** :

- Supprimer `_coerce_tab_view` entièrement
- `get_tab_descriptor` raises `HTTPException(422, "tab descriptor
  invalid: {e}")` au lieu du fallback
- Retirer `legacy_descriptor_sweep` helper + son appel dans
  `Coordinator.start()`
- Retirer les 4 tests associés
- Simplifier le Zod mirror dans `web/src/api/coordinator.ts` (plus
  de discriminated `{schema|legacy|error}`, juste `{descriptor: TabView}`
  ou HTTP error)

### 3.4 Legacy `nexus/gov/api.py` — mapping vers les 19 tabs

Les 45 endpoints legacy sont groupés en domaines. Mapping pressenti
vers les 19 tabs :

| # | Tab | Legacy endpoints consommés |
|---|---|---|
| 1 | Dashboard | `/stats` (aggregates) |
| 2 | Politiciens | `/politicians`, `/politicians/search` |
| 3 | Politicien detail | `/politicians/{id}`, `/politicians/{id}/positions`, `/politicians/{id}/contradictions` |
| 4 | Biography | `/politicians/{id}/biography` |
| 5 | Positions | `/positions` (liste), `/politicians/{id}/positions` |
| 6 | Subjects | `/subjects` |
| 7 | Contradictions (upgrade) | `/contradictions`, `/politicians/{id}/contradictions` |
| 8 | Scan | `/scan/status`, `/scans` |
| 9 | Workers | `/workers` |
| 10 | Pipeline | `/pipeline` |
| 11 | Social | `/social`, `/politicians/{id}/social` |
| 12 | Press | `/press`, `/politicians/{id}/press` |
| 13 | Transcriptions | `/transcriptions`, `/politicians/{id}/transcriptions` |
| 14 | Alerts | `/alerts` |
| 15 | Affairs | `/affairs`, `/politicians/{id}/affairs` |
| 16 | Laws | `/laws`, `/politicians/{id}/declarations` |
| 17 | Factchecks | `/factchecks`, `/politicians/{id}/factchecks` |
| 18 | Search (RAG) | `/search` (RAG via ChromaDB — worker submit) |
| 19 | Ask (RAG) | `/ask` (RAG open question — worker submit) |

Sprint 8 **ne porte pas** les endpoints eux-mêmes — il porte les
**queries SQL** qui back ces endpoints, directement via
`ctx.db.fetchall(...)`. Les apps consomment la SQLite legacy sans
passer par les `APIRouter` legacy. C'est la simplification qui rend
le chantier tenable : au lieu de mapper 45 endpoints → 19 tabs, on
fait 19 tabs qui lisent la DB directement.

### 3.5 `nexus/gov/db.py` — DB legacy layout

**Pattern extrait** (lu : les 47 fichiers `nexus/gov/*.py`) :

- SQLite file default : `nexus/gov/govdata.db` (à confirmer au
  démarrage Phase B via `db.py::DEFAULT_DB_PATH`)
- Tables principales : `politicians`, `positions`, `contradictions`,
  `subjects`, `scan_logs`, `alerts`, `social_posts`, `transcripts`,
  `press_releases`, `affairs`, `laws`, `declarations`, `factchecks`
- Indexes FTS5 sur plusieurs tables pour Search/Ask

**Décision Phase B** : `GovApp.on_start(ctx)` override `ctx.db` pour
pointer vers `nexus/gov/govdata.db` explicit :

```python
async def on_start(self, ctx: AppContext) -> None:
    legacy_db_path = Path(__file__).parent.parent.parent.parent / "nexus" / "gov" / "govdata.db"
    if legacy_db_path.exists():
        ctx.db = AppDatabaseClient(legacy_db_path)
    # else: le default per-app path est utilisé (DB vide, les queries retourneront [])
    self._ctx = ctx
```

Si la DB legacy n'existe pas (fresh install sans le legacy scrape),
les tabs rendent des empty states gracefully. C'est acceptable pour
Sprint 8 — le cas "DB vide" est déjà le cas dev natif.

### 3.6 Sprint 6 `CommandPalette.tsx` actuel

**Pattern extrait** (lu dans Sprint 6 commit `3d87ec3`) :

```typescript
const groups = [
  { heading: "Navigation", items: navItems },
  { heading: "Projets", items: projectItems },
  { heading: "Actions", items: actionItems },
];
```

Chaque item est un `{label, onSelect, icon, testid}`. Le 4e groupe
Sprint 8 ajoutera :

```typescript
{
  heading: "App: Gov",
  items: govCommands.map(cmd => ({
    label: cmd.description,
    icon: cmd.icon,
    onSelect: () => invokeAppCommand(coordUrl, "gov", cmd.name),
    testid: `cmd-gov-${cmd.name}`,
  })),
}
```

Multipliez par N apps enrollées → N groupes `App: <name>` en queue
de liste.

### 3.7 shadcn Command* primitives

`web/src/components/ui/command.tsx` (vendored) expose `Command`,
`CommandDialog`, `CommandInput`, `CommandList`, `CommandEmpty`,
`CommandGroup`, `CommandItem`, `CommandSeparator`. Tous utilisés
déjà par `useCommandPalette` Sprint 6. Aucune modification vendored
requise en Sprint 8 — on ajoute juste des `CommandGroup` dynamiques.

---

## 4. Phase A — SDK core + P2 hygiène Sprint 7 (~1800 LOC)

### 4.1 Fichiers ajoutés

- `packages/nexus-sdk/src/nexus_sdk/commands.py` (~80 LOC)
  - `class CommandDescriptor(BaseModel)` — Pydantic v2 frozen
    avec `model_config = ConfigDict(extra="forbid", frozen=True)`,
    `schema_version: Literal[1] = 1`, `name`/`description`/`icon`/`group`
    + validateurs de longueur
- `packages/nexus-sdk/tests/test_commands.py` (~100 LOC)
  - 8 tests : construction valide, extra field rejected, name trop
    long rejected, description max 280, icon default "sparkles",
    group default "Actions", serialization roundtrip, frozen
    (assignment after construction raises)

### 4.2 Fichiers modifiés

- `packages/nexus-sdk/src/nexus_sdk/decorators.py`
  - +`nexus_command(name, *, description, icon="sparkles", group="Actions")`
    qui attache un dict `__nexus_command__` sur la fonction
  - ~30 LOC
- `packages/nexus-sdk/src/nexus_sdk/registry.py`
  - `collect_decorators` gagne un bucket `commands` en parallèle
    de `routes`/`workers`/`tabs`. Retourne un 4-tuple.
  - ~20 LOC
- `packages/nexus-sdk/src/nexus_sdk/app.py`
  - `AppContext` gagne `db: AppDatabaseClient` (cf. Phase B) +
    `app_name: str` + méthode `async submit_task(worker, payload,
    *, priority, parent_task_id) -> str` qui appelle
    `self._app.resolve_worker(worker)` et délègue à
    `self.compute.submit_task(...)`
  - `NexusApp.__init__` : collect_decorators retourne maintenant
    4 listes, stocke `self._commands`
  - `NexusApp.resolve_worker(routing_key) -> WorkerDescriptor` —
    parse `"<app>.<worker>"` ou `"<worker>"`, retourne le descriptor
    correspondant ou raise `WorkerNotFound`
  - `NexusApp.commands() -> list[CommandDescriptor]` — transforme
    `self._commands` en descriptors Pydantic-validés
  - ~60 LOC
- `packages/nexus-sdk/src/nexus_sdk/__init__.py`
  - Export `nexus_command`, `CommandDescriptor`, `AppDatabaseClient`
    (Phase B), `WorkerNotFound`
- `packages/nexus-sdk/tests/test_decorators.py` (existe)
  - +5 tests : `nexus_command` apposition d'attribut, propagation
    via `collect_decorators`, `NexusApp.commands()` retourne les
    descriptors, resolve_worker success path, resolve_worker
    `WorkerNotFound`

### 4.3 Coordinator routes nouvelles (Phase A)

- `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py`
  - **Retrait D4** :
    - supprimer `_coerce_tab_view` (lines 124-160 actuelles)
    - supprimer `legacy_descriptor_sweep` (lines 170+)
    - supprimer `_normalize_tab_descriptor` si présent
    - `get_tab_descriptor` → raise `HTTPException(422, detail=...)`
      sur `ValidationError` au lieu du fallback
    - tests retirés : `test_coerce_*`, `test_legacy_descriptor_*`,
      `test_legacy_descriptor_sweep_*`
  - **Ajout D1** :
    ```python
    @router.post("/{name}/tasks/submit")
    async def submit_app_task(
        name: str,
        body: SubmitAppTaskRequest,
    ) -> dict[str, str]:
        app = _registry.get_app(name)
        task_id = await app.context.submit_task(
            body.worker, body.payload,
            priority=body.priority,
            parent_task_id=body.parent_task_id,
        )
        return {"task_id": task_id}
    ```
    + `class SubmitAppTaskRequest(BaseModel)` avec validation
  - **Ajout D2** :
    ```python
    @router.get("/{name}/commands", response_model=list[CommandDescriptor])
    async def list_app_commands(name: str) -> list[CommandDescriptor]:
        app = _registry.get_app(name)
        return app.commands()

    @router.post("/{name}/commands/{cmd_name}/invoke")
    async def invoke_app_command(
        name: str, cmd_name: str,
    ) -> dict[str, Any]:
        app = _registry.get_app(name)
        result = await app.invoke_command(cmd_name)
        return {"result": result}
    ```
    + `NexusApp.invoke_command(cmd_name)` helper qui trouve la
    méthode décorée, l'appelle, et serialize le résultat
- `packages/nexus-coordinator/tests/test_apps.py`
  - Tests retirés : `test_coerce_tab_view_*` (3 tests)
  - Tests ajoutés : `test_submit_app_task_happy_path`,
    `test_submit_app_task_unknown_app_404`,
    `test_submit_app_task_bad_worker_422`,
    `test_list_app_commands_ordered`,
    `test_invoke_app_command_runs_method`,
    `test_invoke_app_command_unknown_raises_404`,
    `test_tab_descriptor_raises_422_on_invalid_schema` (remplace
    le test legacy_descriptor)

### 4.4 Frontend — D1 + D2 + D4 + F-1 (~800 LOC)

- `web/src/api/coordinator.ts`
  - **D4** : `getAppTabDescriptor()` simplifié — plus de
    discriminated `{schema|legacy|error}`, juste retourne le
    `TabView` directement (les 422 lèvent une exception via
    `CoordinatorHttpError`)
  - **D1** : `submitAppTask(coordUrl, appName, worker, payload,
    *, priority?, parent_task_id?) -> {task_id: string}` typé Zod
  - **D2** : `CommandDescriptorSchema`,
    `listAppCommands(coordUrl, appName) -> CommandDescriptor[]`,
    `invokeAppCommand(coordUrl, appName, cmdName) -> any`
- `web/src/components/app/tabview/blocks/ButtonBlock.tsx`
  - Retrait du `console.warn("[tabview] task_submit action not yet
    wired")` Sprint 6 stub
  - Wire `action.kind === "task_submit"` → appel de
    `submitAppTask(coordUrl, appName, action.worker,
    action.payload)` avec toast success/error via sonner
  - Props : `coordinatorUrl` + `appName` passés depuis le tab
    parent via un React context `TabAppContext`
  - +1 context file `web/src/components/app/tabview/TabAppContext.tsx`
    (~40 LOC)
- `web/src/pages/Curators.tsx`
  - **F-1** : +bouton Refresh mirroré de Browse.tsx (~10 LOC)
- `web/src/api/__tests__/coordinator.test.ts` (nouveau)
  - 6 tests : `submitAppTask` body shape, `listAppCommands` parse,
    `invokeAppCommand` URL encoding, etc.
- `web/src/components/app/tabview/__tests__/ButtonBlock.test.tsx`
  (existe)
  - Update : le `task_submit` test n'assert plus `console.warn`,
    il assert que `submitAppTask` est appelé avec les bons args

### 4.5 Sprint 7 P2 hygiène absorbées (~400 LOC)

- **A-3 cross-lang curator fixture** : créer
  `packages/nexus-sdk/tests/snapshots/curator_canonical.json`
  (signé avec une keypair test fixe), lire depuis
  `test_curator.py::test_canonical_fixture_verify` ET depuis
  `web/src/api/__tests__/daemon.test.ts::parse canonical fixture`
  (via `resolveJsonModule`). ~60 LOC.
- **A-4 CuratorProjectRef string caps** : ajouter dans
  `curator.rs::CuratorListEntry::verify_signature` un check qui
  itère sur `entries` et rejette si un champ dépasse le cap
  (`project_id <= 128`, `project_name <= 128`, `category <= 64`,
  `description <= 280`). +1 test `verify_rejects_oversized_fields`.
  Mirror Zod dans `web/src/api/daemon.ts::CuratorProjectRefSchema`
  avec `z.string().max(128/64/280)`. ~30 LOC.
- **C-2 NotSubscribed vs EnvelopeMismatch split** :
  `CuratorRuntimeError::AnnouncementAttributionMismatch` splitée
  en `NotSubscribed { announcement }` et
  `EnvelopeMismatch { announcement, entry }`. `handle_announcement`
  différencie : `NotSubscribed` → debug, `EnvelopeMismatch` → warn
  avec pubkey de l'attaquant. +1 test qui assert les deux chemins
  produisent des variants distincts. ~40 LOC.
- **D-1 process_name_matches tightening** : remplacer le
  `norm(observed).contains(&norm(expected))` par un test qui
  normalise puis vérifie égalité exacte OR préfixe suivi d'un
  séparateur (`-`, `_`, `.`). +1 test
  `process_name_rejects_prefix_extension` (ex
  `nexus-shell-daemon-launcher.exe` → false). ~15 LOC.
- **F-1 Curators refresh button** (déjà listé en 4.4 F-1 ci-dessus)
- **G-3 deny_unknown_fields sur daemon HTTP DTOs** : ajouter
  `#[serde(deny_unknown_fields)]` sur `SubscribeCuratorRequest`,
  `SubscriptionsResponse`, `CuratorsListResponse`,
  `BrowseListResponse`. +1 test HTTP qui POST un body avec un
  champ extra et assert 422. ~20 LOC.

### 4.6 Critères d'acceptation Phase A

- `cargo test --workspace --locked` passe, **304 → ~310** (+~6 tests
  Sprint 7 hygiène : curator string caps, error split, process_name
  tighten, daemon deny_unknown)
- `uv run pytest packages/nexus-sdk/tests/ -q` passe, **40 → ~55**
  (+8 test_commands, +5 test_decorators/test_app updates, +2 test
  submit_task wiring)
- `uv run pytest packages/nexus-coordinator/tests/ -q` passe,
  **57+1 → ~63** (+6 nouveaux - 3 legacy retirés)
- `cd web && npm run test:unit` passe, **114 → ~122** (+6
  coordinator.test.ts, +2 ButtonBlock test updates)
- `cd web && npx playwright test` passe, **13 → 14** (+1
  `button-task-submit-flow.spec.ts`)
- `cargo fmt --all --check + cargo clippy --workspace -- -D warnings`
  clean
- `ruff format --check + ruff check` clean
- `tsc + npm run lint + npm run build + npm run size` clean

### 4.7 Commit Phase A

**Target** : `feat(sdk,coordinator,web,shell-daemon): Sprint 8
Phase A — SDK extensions (submit_task + @nexus_command) + legacy
descriptor removal + Sprint 7 P2 hygiene`.

Estimation LOC : **~1800** (SDK ~400 + coord ~400 + web ~400 +
Sprint 7 hygiene ~400 + tests ~200).

---

## 5. Phase B — `AppContext.db` + gov Batch 1 (~1600 LOC)

### 5.1 Fichiers ajoutés (SDK)

- `packages/nexus-sdk/src/nexus_sdk/db.py` *(nouveau ~180 LOC)*
  - `class AppDatabaseClient` avec `__init__(self, db_path: Path)`,
    `async fetchall/fetchone/execute` qui wrappent `aiosqlite`
  - Connection pooling : reopen per-request ou cache un
    `aiosqlite.Connection` dans `__aenter__`/`__aexit__` ?
    Décision : cache une connection lazy avec lock async
  - Row factory : `aiosqlite.Row` → `dict[str, Any]`
  - Error handling : `DatabaseError` wrapper sur `aiosqlite.Error`
- `packages/nexus-sdk/tests/test_db.py` *(nouveau ~140 LOC)*
  - 8 tests : init roundtrip, fetchall dict shape, fetchone None,
    execute insert, parameterized queries, file missing → empty
    results, concurrent fetchall (asyncio.gather), DatabaseError
    on bad SQL

### 5.2 Fichiers modifiés (SDK)

- `packages/nexus-sdk/src/nexus_sdk/app.py`
  - `AppContext.db: AppDatabaseClient` ajouté (init par le loader
    coordinator)
- `packages/nexus-sdk/pyproject.toml`
  - `aiosqlite>=0.20` déjà transitive via coordinator ; ajouter
    en dep directe du SDK pour clarté

### 5.3 Fichiers modifiés (Coordinator)

- `packages/nexus-coordinator/src/nexus_coordinator/loader.py`
  - Initialisation de `ctx.db` dans `load_app(name)` :
    résoudre `paths.app_db_path(project_name, app_name)`, créer
    l'instance `AppDatabaseClient`, l'assigner à `ctx.db` avant
    `app.on_start(ctx)` (qui peut override si l'app le souhaite)
- `packages/nexus-coordinator/src/nexus_coordinator/paths.py`
  - +`def app_db_path(project_name: str, app_name: str) -> Path:
    return nexus_grid_root() / "projects" / project_name / "apps"
    / app_name / "app.sqlite"`
  - +test `test_app_db_path_resolution`

### 5.4 Fichiers rewrite (gov app)

- `packages/nexus-app-gov/src/nexus_app_gov/app.py` (rewrite ~550 LOC)
  - Override `on_start(ctx)` pour pointer `ctx.db` vers
    `nexus/gov/govdata.db` si le fichier existe
  - 6 `@nexus_tab` handlers Batch 1 (Dashboard/Politicians/Detail/
    Biography/Positions/Subjects) — chacun construit un `TabView`
    via les helpers `nexus_sdk.view` et consomme `ctx.db.fetchall(...)`
    pour les données
  - Chaque tab handler : ~60 LOC moyenne (query + TabView build)
- `packages/nexus-app-gov/src/nexus_app_gov/queries.py` *(nouveau ~180 LOC)*
  - Module dédié aux queries SQL typées, importé par `app.py`
  - Exports : `dashboard_stats_query`, `politicians_list_query`,
    `politician_detail_query`, `biography_query`, `positions_query`,
    `subjects_query`
  - Pattern : chaque query est une fonction `async def xxx(db:
    AppDatabaseClient, *args) -> <result_type>` qui retourne des
    dataclasses-like dict
- `packages/nexus-app-gov/tests/test_gov_app.py`
  - Tests retirés : aucun (les 3 existants restent comme regression
    baseline)
  - Tests ajoutés : 6 tests qui mockent `ctx.db` (sqlite3 in-memory
    fixture) et assertent que chaque tab handler produit un
    TabView valide

### 5.5 Critères d'acceptation Phase B

- `uv run pytest packages/nexus-sdk/tests/test_db.py -q` : **8
  passed**
- `uv run pytest packages/nexus-sdk/tests/ -q` : **55 → 63**
  (+8 test_db)
- `uv run pytest packages/nexus-app-gov/tests/ -q` : **3 → 9**
  (+6 tabs)
- `uv run pytest packages/nexus-coordinator/tests/test_apps.py -q` :
  passe, new `test_app_db_initialized_in_loader`
- Playwright : +3 specs `gov-dashboard.spec.ts`,
  `gov-politicians.spec.ts`, `gov-politician-detail.spec.ts` qui
  load les tabs via un coordinator live et assertent la présence
  des blocks clefs
- `cd web && npm run test:coverage` coverage inchangée ≥ 90%

### 5.6 Commit Phase B

**Target** : `feat(sdk,coordinator,app-gov): Sprint 8 Phase B —
AppContext.db + gov batch 1 (Dashboard/Politicians/Biography/
Positions/Subjects)`.

Estimation LOC : **~1600** (SDK db ~320 + coord loader ~80 +
gov rewrite ~550 + queries ~180 + tests ~470).

---

## 6. Phase C — gov Batch 2 (7 tabs) (~1500 LOC)

### 6.1 Fichiers modifiés (gov app)

- `packages/nexus-app-gov/src/nexus_app_gov/app.py`
  - +7 `@nexus_tab` handlers Batch 2 :
    - Contradictions (upgrade du stub Sprint 4)
    - Scan, Workers, Pipeline, Social, Press, Transcriptions
  - Chacun ~60-80 LOC
- `packages/nexus-app-gov/src/nexus_app_gov/queries.py`
  - +7 query functions pour Batch 2 (~200 LOC)
- `packages/nexus-app-gov/tests/test_gov_app.py`
  - +7 tests tab handlers

### 6.2 Critères d'acceptation Phase C

- Chaque des 7 nouveaux tabs retourne un `TabView` valide
- `GET /app/gov/tabs/<tab_name>/descriptor` répond 200 avec une
  forme TabView pour chacun
- Playwright : +2 specs `gov-contradictions-upgrade.spec.ts`,
  `gov-pipeline.spec.ts`
- `uv run pytest packages/nexus-app-gov/tests/ -q` : **9 → 16**

### 6.3 Commit Phase C

**Target** : `feat(app-gov): Sprint 8 Phase C — gov batch 2
(Contradictions/Scan/Workers/Pipeline/Social/Press/Transcriptions)`.

Estimation LOC : **~1500** (app.py ~600 + queries ~300 + tests
~350 + Playwright ~250).

---

## 7. Phase D — gov Batch 3 (6 tabs) + workers RAG (~1500 LOC)

### 7.1 Fichiers modifiés (gov app)

- `packages/nexus-app-gov/src/nexus_app_gov/app.py`
  - +6 `@nexus_tab` handlers Batch 3 : Alerts, Affairs, Laws,
    Factchecks, Search, Ask
  - +2 `@nexus_worker` pour `rag_search` et `rag_ask` :
    ```python
    @nexus_worker(name="rag_search", model="nomic-embed-text")
    async def rag_search(self, ctx: AppContext) -> dict[str, Any]:
        ...

    @nexus_worker(name="rag_ask", model="juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m")
    async def rag_ask(self, ctx: AppContext) -> dict[str, Any]:
        ...
    ```
  - Les workers appellent l'API ollama via `ctx.compute.submit_task`
    pour déléguer le heavy lifting au worker daemon
- `packages/nexus-app-gov/src/nexus_app_gov/queries.py`
  - +6 query functions pour Batch 3
- `packages/nexus-app-gov/src/nexus_app_gov/prompts.py`
  - +`RAG_SEARCH_PROMPT` + `RAG_ASK_PROMPT` templates

### 7.2 Search / Ask flow (D1 end-to-end validation)

Search tab :

1. User types query → TabView button `task_submit` action avec
   worker="gov.rag_search", payload={"query": "..."}
2. `ButtonBlock.tsx` → `submitAppTask(coordUrl, "gov",
   "gov.rag_search", {query})` → coordinator `/app/gov/tasks/submit`
3. `GovApp.context.submit_task("gov.rag_search", {query})` →
   `resolve_worker("gov.rag_search")` → `WorkerDescriptor(model=
   "nomic-embed-text", ...)` → `compute.submit_task(task_type=
   "gov.rag_search", prompt=json.dumps(payload), model=...)`
4. Worker daemon (Rust) pulls task, exécute l'embedding via Ollama,
   stocke le résultat
5. Tab polls `GET /tasks/{id}` pour récupérer le résultat
6. TabView re-render avec les résultats

### 7.3 Critères d'acceptation Phase D

- Les 6 nouveaux tabs + 2 workers passent leur descriptor call
- `gov.rag_search` et `gov.rag_ask` sont résolus par
  `NexusApp.resolve_worker` et retournent un `task_id`
- Playwright : +2 specs `gov-rag-search.spec.ts`,
  `gov-alerts.spec.ts`
- `uv run pytest packages/nexus-app-gov/tests/ -q` : **16 → 22**

### 7.4 Commit Phase D

**Target** : `feat(app-gov): Sprint 8 Phase D — gov batch 3
(Alerts/Affairs/Laws/Factchecks/Search/Ask) + RAG workers`.

Estimation LOC : **~1500**.

---

## 8. Phase E — @nexus_command palette integration + polish (~1200 LOC)

### 8.1 gov @nexus_command entries

- `packages/nexus-app-gov/src/nexus_app_gov/app.py`
  - +4 `@nexus_command` decorations :
    ```python
    @nexus_command(
        name="new_scan",
        description="Lancer un nouveau scan des politiciens",
        icon="refresh",
        group="Gov",
    )
    async def cmd_new_scan(self) -> dict[str, Any]:
        return {"navigation": {"path": "/app/gov/tabs/scan"}}
    ```
  - 4 commands : new_scan, detect_contradictions, search_factchecks,
    view_alerts — chacune navigate vers un tab gov
- `packages/nexus-app-gov/tests/test_gov_app.py`
  - +4 tests : `test_gov_commands_registered`,
    `test_cmd_new_scan_navigates_to_scan`, etc.

### 8.2 CommandPalette.tsx — 4e groupe "App"

- `web/src/components/CommandPalette.tsx`
  - +`useQuery` qui fetch `listAppCommands(coordUrl, appName)` pour
    chaque app enrollée sur le coordinator actif
    (refetchInterval 30s)
  - +merge dans `groups` : un `CommandGroup heading="App: <appName>"`
    par app avec items → `invokeAppCommand(coordUrl, appName,
    cmdName)` puis si `{navigation.path}` → `navigate(path)`
  - +test `CommandPalette.test.tsx` : 4 tests (render avec 0 apps,
    render avec 1 app + 4 commands, select command → navigate,
    select command → invoke then noop)

### 8.3 Playwright end-to-end

- `web/tests/gov-commands-flow.spec.ts` *(nouveau)*
  - Spawn coordinator + gov app, open shell, Ctrl+K, asserter que
    "Détecter contradictions" apparaît sous "App: Gov", click,
    asserter navigation vers `/app/gov/tabs/contradictions`
- `web/tests/gov-final-polish.spec.ts` *(nouveau)*
  - Visual regression sur 3 tabs gov après polish (skeleton →
    loaded → empty)

### 8.4 gov tabs polish

- Skeleton loaders pour chaque tab pendant que `fetchall` en cours
- Empty states avec CTA utiles (« Aucune donnée — lancer un scan
  via la palette »)
- Error states (DB legacy absente → banner explicatif)
- Sort dropdowns sur les tables principales (Politicians, Positions,
  Laws)

### 8.5 Critères d'acceptation Phase E

- 4 gov commands apparaissent dans la palette
- 1 Playwright command-flow passe
- `uv run pytest packages/nexus-app-gov/tests/ -q` : **22 → 26**
- `cd web && npm run test:unit` : **122 → ~128** (+4 CommandPalette
  + ~2 polish regressions)
- `cd web && npx playwright test` : **14 → 16** (+2 end-to-end)
- `cd web && npm run size` budgets verts (main ≤ 475, vendor-react
  ≤ 210, vendor-ui ≤ 50, css ≤ 100)

### 8.6 Commit Phase E

**Target** : `feat(app-gov,web): Sprint 8 Phase E — gov
@nexus_command + palette integration + polish`.

Estimation LOC : **~1200**.

---

## 9. Phase F — Sortie de sprint

### 9.1 Livrables obligatoires

- `.planning/sprint8_verification.md` — self-report fail-fast
  checklist ≥28 rows (format exact Sprint 6/7)
- `.planning/sprint8_audit_plan.md` — 9 tracks minimum avec
  objectifs + méthodes, prêt à être joué par une session fraîche
  Sprint 9 Phase 0. Tracks cibles :
  - **A** — TabView descriptor removal verification (no regression,
    no zombie `legacy_descriptor` code path)
  - **B** — AppContext.submit_task contract + wiring end-to-end
  - **C** — @nexus_command contract + palette integration + error
    paths
  - **D** — AppContext.db read boundary + SQL injection surface
    + path resolution sanity
  - **E** — gov tabs data fidelity vs legacy `nexus/gov/api.py`
    endpoints (spot-check 3 tabs)
  - **F** — Command palette UX across states (0 apps, 1 app, many
    apps, invoke error)
  - **G** — Scope cut verification (grep 0 match sur storage/events/
    upload/migration)
  - **H** — Cross-dependency hygiene (aiosqlite version, dep bumps,
    any new Zod usages)
  - **I** — Documentation coherence (PATTERNS.md P8 update, P10
    new, T4/T5 closed)
- `docs/shell/PATTERNS.md` — update P8 (legacy retired note) +
  ajout P10 (command palette app-contributed) + fermeture tech
  debt T4/T5
- `docs/rust/PATTERNS.md` — update Sprint 7 tech debt section :
  marquer A-4, C-2, D-1, G-3 comme CLOSED Sprint 8 Phase A si les
  P2 sont traités ; laisser les 4 pré-confessés (E-1, C-4, D-3,
  H-3) comme ouverts
- Mise à jour `nexus_grid_pivot.md` memory (manuel, externe au
  repo) : tip Sprint 8, transition Sprint 9 avec les 4 infra items
  déférés

### 9.2 Scan final

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

uv run ruff format --check packages/ examples/
uv run ruff check packages/ examples/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

cd web
npx tsc --noEmit -p tsconfig.app.json
npm run lint
npm run test:unit
npm run test:coverage
npm run build
npm run size
npx playwright test
bash scripts/scan-en-strings.sh
cd ..

grep -rn 'TODO(Sprint8)' crates/ packages/ web/src/ || echo "0 match"
grep -rn 'legacy_descriptor' packages/nexus-coordinator/ web/src/api/ && exit 1 || echo "legacy removed"
grep -rn 'AppContext.storage\|AppContext.events' packages/ && exit 1 || echo "deferred items absent"
```

### 9.3 Commit Phase F

**Target** : `docs(sprint8): verification + audit plan for Sprint 9`.

---

## 10. Fail-fast checklist (cible Sprint 8)

| # | Row | Commande | Attendu |
|---|---|---|---|
| 1 | Rust workspace build | `cargo build --workspace --locked` | exit 0 |
| 2 | Rust fmt | `cargo fmt --all --check` | exit 0 |
| 3 | Rust clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| 4 | Rust tests workspace | `cargo test --workspace --locked` | ≥ 310 (304 + 6 P2 hygiene A-4/C-2/D-1/G-3) |
| 5 | Curator string caps (A-4) | `cargo test -p nexus-core-rs curator::tests::verify_rejects_oversized_fields` | 1 pass |
| 6 | NotSubscribed vs EnvelopeMismatch (C-2) | `cargo test -p nexus-shell-daemon-core iroh_runtime::tests::not_subscribed_and_envelope_mismatch_are_distinct` | 1 pass |
| 7 | process_name_matches tightening (D-1) | `cargo test -p nexus-shell-daemon-core registry::tests::process_name_rejects_prefix_extension` | 1 pass |
| 8 | daemon deny_unknown_fields (G-3) | `cargo test -p nexus-shell-daemon http::tests::subscribe_rejects_extra_fields` | 1 pass |
| 9 | Python SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | ≥ 63 (40 + 8 test_commands + 8 test_db + others) |
| 10 | Python SDK commands tests | `uv run pytest packages/nexus-sdk/tests/test_commands.py -q` | ≥ 8 pass |
| 11 | Python SDK db tests | `uv run pytest packages/nexus-sdk/tests/test_db.py -q` | ≥ 8 pass |
| 12 | Python coord tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | ≥ 63 (57 + 6 nouveaux - 3 legacy retirés) |
| 13 | Python coord submit_task route | `uv run pytest packages/nexus-coordinator/tests/test_apps.py::test_submit_app_task_happy_path -q` | pass |
| 14 | Python coord commands route | `uv run pytest packages/nexus-coordinator/tests/test_apps.py::test_list_app_commands_ordered -q` | pass |
| 15 | Python coord legacy retired | `grep _coerce_tab_view packages/nexus-coordinator/` returns `No files found` | exit 0 (no legacy) |
| 16 | Python app-gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | ≥ 26 (3 baseline + 23 new) |
| 17 | gov Dashboard tab | `uv run pytest packages/nexus-app-gov/tests/ -k test_dashboard -q` | pass |
| 18 | gov Contradictions upgrade | `uv run pytest packages/nexus-app-gov/tests/ -k test_contradictions -q` | pass |
| 19 | gov rag_search worker resolve | `uv run pytest packages/nexus-app-gov/tests/ -k test_rag_search -q` | pass |
| 20 | ruff format + check | `uv run ruff format --check packages/ examples/ && uv run ruff check packages/ examples/` | exit 0 |
| 21 | Web tsc strict | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 |
| 22 | Web ESLint | `cd web && npm run lint` | 0 err, ≤ 5 T1 warnings |
| 23 | Web Vite build | `cd web && npm run build` | exit 0, no warnings |
| 24 | Web size-limit budgets | `cd web && npm run size` | main ≤ 475, vendor-react ≤ 210, vendor-ui ≤ 50, css ≤ 100 |
| 25 | Web Vitest unit | `cd web && npm run test:unit` | ≥ 128 (114 baseline + 14 phase A/E) |
| 26 | Web coverage thresholds | `cd web && npm run test:coverage` | lines ≥ 90, funcs ≥ 90, branches ≥ 85 |
| 27 | Playwright (gov + command flow) | `cd web && npx playwright test` | ≥ 20 pass (13 baseline + 7 new phase B/C/D/E) |
| 28 | Web scan-en-strings | `cd web && bash scripts/scan-en-strings.sh` | exit 0 |
| 29 | npm audit | `cd web && npm audit --audit-level=high` | 0 high/critical |
| 30 | TODO(Sprint8) hanging | `grep -rn 'TODO(Sprint8)' crates/ packages/ web/src/` | 0 match |
| 31 | Deferred items absent | `grep -rn 'AppContext.storage\|AppContext.events' packages/` | 0 match |
| 32 | sprint8_audit_plan.md exists | `test -f .planning/sprint8_audit_plan.md` | exit 0 |

**32 rows** — au-dessus de la cible ≥28. Cible de fermeture
confirmée à 32/32 verts.

## 11. Git plan

Commits cibles sur master (atomiques par phase, pattern Sprint 6/7) :

1. `docs(sprint8): kickoff + plan`
2. `feat(sdk,coordinator,web,shell-daemon): Sprint 8 Phase A — SDK extensions + legacy removal + Sprint 7 hygiene`
3. `feat(sdk,coordinator,app-gov): Sprint 8 Phase B — AppContext.db + gov batch 1 (Dashboard/Politicians/Biography/Positions/Subjects)`
4. `feat(app-gov): Sprint 8 Phase C — gov batch 2 (Contradictions/Scan/Workers/Pipeline/Social/Press/Transcriptions)`
5. `feat(app-gov): Sprint 8 Phase D — gov batch 3 (Alerts/Affairs/Laws/Factchecks/Search/Ask) + RAG workers`
6. `feat(app-gov,web): Sprint 8 Phase E — gov @nexus_command + palette integration + polish`
7. `docs(sprint8): verification + audit plan for Sprint 9`

Total target : **7 commits** (pattern Sprint 6/7). Si un fix
post-phase est nécessaire (pattern Sprint 2 `de9589d` / `ed2ea76`),
commit séparé `fix(sprint8): ...` entre les phases concernées.

## 12. Scope cuts (à respecter strictement)

Répétition de `sprint8_kickoff.md` §6 pour exécution :

### Infra items déférés Sprint 9

- **Pas de `AppContext.storage`** (KV primitif) — Sprint 9
- **Pas de `AppContext.events`** (pub/sub in-process) — Sprint 9
- **Pas de file upload endpoint** — Sprint 9
- **Pas de DB migration runner** — Sprint 9

### Sprint 7 tech debt NON traitée

- **E-1 probe_reachable 2s timeout** — tech debt Sprint 9
- **C-4 gossip backpressure** — tech debt Sprint 9
- **D-3 subscriptions persist order** — tech debt Sprint 9
- **H-3 nexus_core wheel install drift** — tech debt Sprint 9
- **F-3 CardTitle accessibility** — Sprint 9 polish
- **G-1 httpx client per-call + limits** — Sprint 9

### Scope architecture gov

- **Pas de mutations** via `@nexus_route` POST/PUT/DELETE portées
  en Sprint 8. Les 45 endpoints legacy sont consommés **READ-ONLY**
  via `ctx.db`, pas via un port 1-pour-1 du router legacy
- **Pas de re-scrape triggered from gov tabs** — les tabs affichent
  l'état de la DB existante, aucun worker de scraping lancé depuis
  un click utilisateur
- **Pas de Reseau graph / Leaflet map** — v1.2+
- **Pas d'auth** sur `submit_task` ou `invoke_command` — loopback
  trust, même modèle que `/tasks/submit`
- **Pas de push notifications** / websocket real-time — polling
  React Query suffit
- **Pas de mobile responsive < 1280px** — reconfirm Sprint 5 D3

## 13. Risks

- **R1 — Legacy DB legacy absent / corrompu** : si
  `nexus/gov/govdata.db` n'existe pas (fresh dev install) ou est
  corrompu, les 19 tabs rendent des empty states. Mitigation :
  chaque tab handler gère gracefully `fetchall([]) -> empty state
  TabView`. Test unit avec DB in-memory vide pour chaque tab.
- **R2 — `@nexus_command` invocation schema drift** : si le
  `CommandDescriptor` Zod mirror dérive du Pydantic côté Python,
  l'audit Sprint 9 Phase 0 le détectera. Mitigation : un test
  cross-lang fixture comme Sprint 6 TabView (`test_commands.py`
  lit `packages/nexus-sdk/tests/snapshots/command_canonical.json`
  et Vitest `coordinator.test.ts` lit le même fichier).
- **R3 — Legacy `nexus/gov/` import collisions** : le module
  `nexus/gov/` existe toujours dans le monorepo (non-packaged).
  Si `packages/nexus-app-gov/` essaie d'importer `from nexus.gov
  import db`, le resolver Python peut échouer (nexus n'est pas un
  package installé). Mitigation : Phase B utilise `Path(__file__).
  parent.parent.parent.parent / "nexus" / "gov" / "govdata.db"` à
  la racine du repo, sans import Python. Lecture SQL pure via
  `AppDatabaseClient`.
- **R4 — `legacy_descriptor` retrait casse un client externe** :
  aucun app externe consomme aujourd'hui le fallback (verifié par
  grep Sprint 6 audit D-1). Mitigation : le retrait est breaking
  dans le contrat HTTP (les 422 remplacent les 200 avec `{legacy:
  true}`) mais aucun consommateur n'en dépend. Test : le gov
  Contradictions upgrade doit passer en schema-driven sans
  nécessiter le fallback.
- **R5 — `AppContext.submit_task` routing key ambiguity** : si un
  user écrit `"gov.contradiction_detector"` et le gov app a aussi
  un worker `contradiction_detector` via `@nexus_worker`, la
  résolution doit être déterministe. Mitigation : `resolve_worker`
  essaie d'abord le format `"<app>.<worker>"`, fallback sur
  `"<worker>"` dans l'app courante seulement. Test `test_resolve_worker_cross_app`
  + `test_resolve_worker_ambiguous_raises`.
- **R6 — gov app.py devient trop gros** : 19 tabs + 4 commands
  + 2 workers = ~1500 LOC dans un seul fichier. Mitigation : si
  en Phase C on dépasse 800 LOC, splitter en sous-modules
  `tabs/dashboard.py`, `tabs/politicians.py`, etc. et les charger
  depuis `app.py`. Décision: attendre Phase C pour trancher, pas
  pre-optimiser Phase B.
- **R7 — Palette fetch de commands hammer le coordinator** : avec
  N apps × M commands, la palette fait N queries à chaque refresh.
  Mitigation : `refetchInterval: 30_000` + `staleTime: 15_000`
  dans React Query, et un batch endpoint
  `GET /apps/commands` pourrait être ajouté Sprint 9 si N croît.
  Sprint 8 reste sur 1-app (gov) donc OK.
- **R8 — Playwright gov specs flaky si DB legacy absente** : si la
  DB legacy n'est pas peuplée en CI, les tests d'affichage de
  données passent en empty state et peuvent ne plus valider que
  "le tab charge". Mitigation : seeder une mini-DB fixture
  (~50 rows) dans `web/tests/fixtures/gov_seed.sql` exécutée par
  `global-setup.ts` avant les specs gov. +~100 LOC fixture.
- **R9 — aiosqlite concurrency issues** : si 2 tabs fetchall en
  parallèle sur la même `AppDatabaseClient`, aiosqlite peut
  serialiser ou lever une erreur selon la version. Mitigation :
  `AppDatabaseClient` utilise une `asyncio.Lock` interne pour
  sérialiser les queries, test `test_concurrent_fetchall`.
- **R10 — RAG Search/Ask tab Phase D nécessite un worker daemon
  running** : si le worker daemon (nexus-worker) n'est pas démarré,
  `submit_task` retourne OK mais le task reste en `pending`
  éternellement. Le tab polling voit `pending` → no result. Pas
  une régression mais UX dégradée. Mitigation : le tab affiche un
  banner "Nécessite un worker actif — démarre via
  `nexus-worker start`" si le task stagne > 30s.

## 14. Checkpoint de clôture Sprint 8

Sprint 8 est **fermé** quand :

1. Fail-fast §10 : 32/32 vert
2. `git log --oneline master ^2ed0955` affiche 7-10 commits (avec
   éventuels `fix(sprint8): ...` post-phase)
3. `.planning/sprint8_verification.md` commité et lisible
4. `.planning/sprint8_audit_plan.md` commité et lisible (obligatoire
   `sprint_audit_gate.md`)
5. `docs/shell/PATTERNS.md` contient P10 (command palette) + P8
   update (legacy retired) + T4/T5 marked CLOSED
6. `docs/rust/PATTERNS.md` tech debt section updated : 4 items
   Sprint 7 P2 hygiène (A-4/C-2/D-1/G-3) marqués CLOSED ; 4 items
   E-1/C-4/D-3/H-3 restent ouverts pour Sprint 9
7. Aucun `TODO(Sprint8)` dans le code
8. Aucun match grep `legacy_descriptor` ni `_coerce_tab_view` dans
   `packages/nexus-coordinator/` ni `web/src/api/`
9. Aucun match grep `AppContext.storage` ni `AppContext.events`
   dans `packages/` (D5 scope cut enforcé)
10. `MEMORY.md` `nexus_grid_pivot.md` mis à jour avec le tip
    Sprint 8 + transition vers Sprint 9 (~4 infra items déférés)

Après fermeture : **rien**. Sprint 9 ouvrira avec sa propre Phase 0
audit de Sprint 8 (session fraîche jouant
`sprint8_audit_plan.md`). Pas d'écriture préemptive d'un
`sprint9_kickoff.md` — ça violerait le pattern audit gate.
