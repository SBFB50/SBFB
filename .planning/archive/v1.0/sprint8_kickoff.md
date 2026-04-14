# Sprint 8 — Kickoff (nexus-app-gov v1.1 migration 19 tabs)

**Écrit** : 2026-04-11
**HEAD entrée** : `2ed0955` (master tip post-audit-gate Sprint 7 PASS)
**Auteur** : session de démarrage Sprint 8 après lecture de
`MEMORY.md` + `nexus_grid_pivot.md` + `sprint_audit_gate.md` +
`docs/claude/README.md`, puis jeu de la Phase 0 audit gate Sprint 7
(verdict PASS, findings doc `2ed0955`), puis reconnaissance du split
`nexus-sdk::AppContext`/`ComputeClient`, de la stub `GovApp`
(1 route + 1 worker + 1 tab), de la surface legacy `nexus/gov/`
(45 endpoints API + 32 workers + 12 models), et du `_coerce_tab_view()`
fallback Sprint 6 marked `TODO(Sprint 8)` dans
`packages/nexus-coordinator/src/nexus_coordinator/api/apps.py`.

---

## 1. Constat d'entrée — Phase 0 audit gate DONE

Sprint 7 a été **audité en Phase 0 de Sprint 8** par une session
Claude Code fraîche jouant `.planning/sprint7_audit_plan.md` (9 tracks
A..I). **Verdict : PASS** (0 P0 / 0 P1 / 10 P2 / 5 P3).

`.planning/sprint7_audit_findings.md` détaille les 17 findings ; le
commit `2ed0955 docs(sprint7): audit findings from Sprint 8 Phase 0
gate` a atterri sur master **AVANT** ce kickoff, sans commit
`fix(sprint7): ...` préalable — le verdict PASS ne nécessite aucun
gate blocking.

Sur les 10 P2 :

- **4 pré-auto-confessés** dans `docs/rust/PATTERNS.md:820-858`
  (E-1 probe timeout / C-4 gossip backpressure / D-3 subscriptions
  persist order / H-3 wheel install drift) — restent tech debt
  Sprint 9, pas de traitement Sprint 8
- **6 nouveaux détectés par l'audit** (A-3 cross-lang curator
  fixture / A-4 CuratorProjectRef string caps / C-2 NotSubscribed
  vs EnvelopeMismatch split / D-1 process_name_matches tighten /
  F-1 Curators refresh button / G-3 daemon DTOs deny_unknown_fields)
  — **traités comme hygiène Phase A** du Sprint 8 (~100 LOC), ce qui
  absorbe proprement la dette avant que le chantier gov ne démarre

### Test counts à l'entrée (verifiés par l'audit)

| Suite | Count | Delta vs Sprint 7 |
|---|---|---|
| Rust workspace | 304 | 0 |
| Python SDK | 40 | 0 |
| Python coordinator | 57 + 1 skipped | 0 |
| Python app-gov | 3 | 0 |
| Vitest web/ | 114 | 0 |
| Playwright | 13 | 0 |
| `npm audit` | 0 vulns | — |
| `size-limit` | 4/4 green | — |

Le working tree est clean à `2ed0955` (modulo `M CLAUDE.md` + `??
docs/claude/` pré-existants à la session, hors audit scope). Aucun
P1/P2 non-fixé ne bloque l'ouverture de ce kickoff.

## 2. Goal Sprint 8 (une phrase)

Migrer `nexus-app-gov` de **1 tab stub** à **19 tabs TabView-native**
couvrant l'API government legacy (politiciens, contradictions,
graphe, social, alertes, affaires, lois, factchecks, RAG),
implémenter les 2 extensions SDK frozen Sprint 7 D4/D5
(`AppContext.submit_task`, `@nexus_command`), brancher
`AppContext.db` comme bridge lecture vers la SQLite gov existante,
retirer le `legacy_descriptor` fallback, **sans** introduire les 4
autres primitives infra (`storage` / `events` / file upload / DB
migration runner) qui attendent un vrai consommateur Sprint 9+.

## 3. Phase 0 — Audit gate de Sprint 7 (DONE)

Status : **terminé avant ce kickoff**. Références :

- `.planning/sprint7_audit_plan.md` (plan joué)
- `.planning/sprint7_audit_findings.md` (verdict PASS + 17 findings)
- `2ed0955 docs(sprint7): audit findings from Sprint 8 Phase 0 gate`

Phase 0 est **fermée** et ne consomme plus aucun commit Sprint 8.
Elle est listée ici uniquement pour la traçabilité du pattern
`sprint_audit_gate.md` — à partir de Sprint 9, Phase 0 jouera
`.planning/sprint8_audit_plan.md` que **Phase F de Sprint 8 doit
écrire** (cf §5 Phase F).

## 4. Décisions Day 0 (D1..D5 — gelées)

### D1 — `AppContext.submit_task` (impl Sprint 7 D4 frozen)

**Retenu** : la signature gelée Sprint 7 Day 0 D4 est implémentée
telle quelle — aucune dérive, aucun renommage de paramètre.

```python
# packages/nexus-sdk/src/nexus_sdk/app.py (extension Sprint 8 Phase A)

@dataclass
class AppContext:
    compute: ComputeClient
    project_name: str
    db: AppDatabaseClient  # D3 nouveau
    app_name: str          # nouveau — nécessaire pour resolve_worker
    extras: dict[str, Any] = field(default_factory=dict)

    async def submit_task(
        self,
        worker: str,
        payload: dict[str, Any],
        *,
        priority: int = 5,
        parent_task_id: str | None = None,
    ) -> str:
        """Submit a task to the coordinator's dispatcher.

        `worker` is a routing key format `"<app>.<worker>"` or
        just `"<worker>"` if unambiguous in the current app. The
        runtime resolves it via `NexusApp.resolve_worker()` to
        the `WorkerDescriptor` registered with `@nexus_worker`,
        grabs the model declared there, serializes `payload` as
        JSON into the `prompt` field, and delegates to
        `ComputeClient.submit_task(...)`.
        """
```

**Résolution du routing key** (ajoute ~30 LOC dans `nexus_sdk/app.py`) :

```python
class NexusApp(ABC):
    def resolve_worker(self, routing_key: str) -> WorkerDescriptor:
        """Resolve a routing key to a concrete WorkerDescriptor.

        Accepts `<app>.<worker>` (cross-app) or `<worker>` if
        the current app has a worker by that name. Raises
        `WorkerNotFound` otherwise.
        """
```

Le ButtonBlock React wire le `task_submit` action via un nouveau
endpoint coordinator `POST /app/{name}/tasks/submit` qui parse le
body comme `{worker, payload, priority, parent_task_id}`, résout
l'app cible via `coordinator.loader.get_app(name)` et appelle
`app.context.submit_task(...)` en interne.

**Rejeté** :

- Dupliquer `ComputeClient` avec une surface séparée → double
  maintenance, risque de drift
- Changer le contrat `/tasks/submit` du coordinator pour accepter un
  routing key → casse les workers existants (Sprint 4 dispatcher)
- Faire du `submit_task` une méthode `ComputeClient` → violerait la
  D4 frozen signature (args positional)

**Implications** :

- `nexus_sdk.app.AppContext` gagne `submit_task` + `app_name` +
  `db` (D3)
- `NexusApp.resolve_worker(routing_key) -> WorkerDescriptor`
  nouveau
- `coordinator/api/apps.py` gagne `POST /app/{name}/tasks/submit`
- `web/src/components/app/tabview/blocks/ButtonBlock.tsx` wire le
  `action.kind === "task_submit"` branch pour appeler la nouvelle
  route + toast success/error via shadcn
- `web/src/api/coordinator.ts` gagne `submitAppTask(coordUrl,
  appName, worker, payload)` typé Zod
- Test : un tab button click → `task_id` retourné, observable via
  `GET /tasks`
- Sprint 6 tech debt **T4 fermée** en Phase A

### D2 — `@nexus_command` (impl Sprint 7 D5 frozen)

**Retenu** : la signature complète gelée Sprint 7 Day 0 D5 est
implémentée telle quelle — decorator + `CommandDescriptor` Pydantic
+ coord routes + Zod mirror + palette 4e groupe.

```python
# packages/nexus-sdk/src/nexus_sdk/decorators.py (extension Sprint 8 Phase A)

def nexus_command(
    name: str,
    *,
    description: str,
    icon: str = "sparkles",
    group: str = "Actions",
) -> Callable[[F], F]:
    """Mark a coroutine on a NexusApp as a command palette entry."""
```

```python
# packages/nexus-sdk/src/nexus_sdk/commands.py (nouveau Sprint 8)

class CommandDescriptor(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)
    schema_version: Literal[1] = 1
    name: str = Field(..., min_length=1, max_length=64)
    description: str = Field(..., max_length=280)
    icon: str = Field("sparkles", max_length=32)
    group: str = Field("Actions", max_length=32)
```

**Invocation** : `POST /app/{name}/commands/{cmd_name}/invoke` avec
body `{}` (v1 ne passe pas d'arguments structurés, le trigger
palette est user-action-only). Retour :

- `null` → no-op (commande exécutée, pas de changement UI)
- `{"navigation": {"path": "/..."}}` → le shell navigue vers `path`
- Toute autre forme → traité comme no-op + warning log

**Rejeté** :

- Arguments structurés au command invoke → déferré à v1.2 si un
  consommateur concret en demande
- Push-based commands (coordinator triggers shell) → hors scope
  palette
- Commands non-async → uniforme avec le reste du SDK

**Implications** :

- `nexus_sdk/decorators.py` +~30 LOC (`nexus_command`)
- `nexus_sdk/commands.py` +~50 LOC (`CommandDescriptor`)
- `NexusApp.commands() -> list[CommandDescriptor]` ajoutée via
  introspection (pattern `routes()`/`workers()`/`tabs()`)
- `coordinator/api/apps.py` gagne 2 routes
  (`GET /app/{name}/commands`, `POST /app/{name}/commands/{cmd}/invoke`)
- `web/src/api/coordinator.ts` gagne `CommandDescriptorSchema` +
  `listAppCommands(coordUrl, appName)` +
  `invokeAppCommand(coordUrl, appName, cmdName)` typés Zod
- `web/src/components/CommandPalette.tsx` ajoute le 4e groupe
  "App" qui merge les commands retournées pour tous les apps
  enrollés sur le coordinator actif (fetch via React Query avec
  `refetchInterval: 30s`)
- Test : un palette entry click → command invoked → navigation ou
  noop
- Sprint 6 tech debt **T5 fermée** en Phase A

### D3 — `AppContext.db` : bridge lecture SQLite per-app

**Retenu** : plutôt que le `AppContext.storage` KV primitif du
memory `nexus_grid_pivot.md`, Sprint 8 introduit un `AppContext.db`
typé bridge **lecture-principal** vers un fichier SQLite per-app.
Le fichier résout à
`paths.nexus_grid_root() / "projects" / <project> / "apps" / <app> / "app.sqlite"`
par défaut, mais une app peut override dans `on_start()` pour
pointer vers un fichier existant (ce qu'on fera pour gov pour
réutiliser la base legacy `nexus/gov/govdata.db`).

```python
# packages/nexus-sdk/src/nexus_sdk/db.py (nouveau Sprint 8 Phase B)

class AppDatabaseClient:
    """Thin aiosqlite wrapper with dict-based row access.

    Apps use this in tab handlers and `@nexus_route` handlers
    to query their own per-app SQLite database. Pure read
    path in Sprint 8 — writes are allowed (aiosqlite supports
    them) but no migration runner is shipped; apps create
    their schema in `on_start`.
    """

    def __init__(self, db_path: Path) -> None: ...

    async def fetchall(
        self, query: str, params: Sequence[Any] | None = None
    ) -> list[dict[str, Any]]: ...

    async def fetchone(
        self, query: str, params: Sequence[Any] | None = None
    ) -> dict[str, Any] | None: ...

    async def execute(
        self, query: str, params: Sequence[Any] | None = None
    ) -> None: ...
```

**Pour gov** : `GovApp.on_start()` override l'initialisation du
`ctx.db` pour pointer vers le fichier SQLite legacy existant
(`nexus/gov/govdata.db` ou le path résolu par
`nexus/gov/db.py::default_db_path()` si défini). Le schema legacy est
consommé **read-only** via `fetchall`. Pas de migration wholesale du
module `nexus/gov/db.py` — celui-ci reste disponible comme helper
pour un éventuel ETL séparé Sprint 9+.

**Rejeté** :

- ORM / abstraction layer (SQLAlchemy, Tortoise) → over-engineered
  pour des SELECT plats
- iroh-docs per-app comme storage → heavyweight pour de la lecture
  local, pas de besoin de sync P2P en Sprint 8
- `AppContext.storage: dict[str, Any]` KV primitif → trop limité
  pour les queries multi-colonnes que les 19 tabs gov font
- Exposer directement un `aiosqlite.Connection` → fuite
  d'abstraction, le wrapper `fetchall/fetchone/execute` stabilise
  le contrat

**Implications** :

- `nexus_sdk/db.py` nouveau (~150 LOC + 8 tests)
- `AppContext.db` initialisé dans `on_start` par le loader du
  coordinator (chemin par défaut) OU par l'app elle-même si elle
  veut override
- `nexus_sdk/app.py::AppContext` gagne le champ
  `db: AppDatabaseClient`
- `GovApp.on_start()` override pour pointer vers le fichier legacy
- Les 19 tabs gov consomment `ctx.db.fetchall("SELECT ...")`
  directement

### D4 — Legacy descriptor fallback — removal complet

**Retenu** : le chemin `_coerce_tab_view()` qui emballe un mauvais
descriptor dans un envelope `{"descriptor": ..., "legacy_descriptor":
true}` est **retiré entièrement**. Tout app DOIT retourner un
`TabView` pydantic-valide. Un parse failure résulte en HTTP 422 +
WARNING log `"app {} tab {} descriptor invalid: {error}"`, pas en
fallback silencieux.

**Rejeté** :

- Garder le fallback comme filet de sécurité → contradiction avec
  Sprint 6 D-1 tech debt explicit `# TODO(Sprint 8): remove`
- Déprécier avec deprecation warning sur 2 sprints → overhead
  workflow sans bénéfice (aucun app externe consomme aujourd'hui)
- Conserver juste `legacy_descriptor_sweep` boot-time sweep →
  orphelin sans son fallback consumer

**Implications** :

- `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py` :
  retrait de `_coerce_tab_view`, `legacy_descriptor_sweep`,
  `_normalize_tab_descriptor`, tous les tests associés
- Route `GET /app/{name}/tabs/{tab_name}/descriptor` retourne
  directement `{"descriptor": <tabview.model_dump()>}` sans flag
  `legacy_descriptor`
- Zod côté shell :
  `web/src/api/coordinator.ts::getAppTabDescriptor()` simplifié
  — plus de discriminated `{schema|legacy|error}`, juste
  `{descriptor: TabView}` (le `error` path reste pour HTTP 422)
- Tests retirés : `test_coerce_tab_view_*`, `test_legacy_descriptor_*`
- `docs/shell/PATTERNS.md` P8 (TabView seul contrat) gagne une
  note « legacy fallback retired Sprint 8 »
- Sprint 6 tech debt **D-1 + D-3 fermées**

### D5 — Scope : 19 tabs read-heavy + 4 infra items DÉFÉRÉS Sprint 9

**Retenu** : Sprint 8 livre `submit_task` + `@nexus_command` +
`AppContext.db` + legacy removal + 19 tabs gov. **Les 4 autres items
infra listés dans `nexus_grid_pivot.md` §Sprint 8** (`AppContext.storage`
KV, `AppContext.events` pub/sub, file upload endpoint, DB migration
runner) **sont déférés à Sprint 9** pour les raisons suivantes :

1. **Les 19 tabs gov sont read-heavy** — SELECT + display, pas de
   state mutation côté app. `AppContext.db` suffit à les alimenter
2. **Aucun consommateur connu** pour storage/events/upload dans
   cette sprint — pas de tab gov qui fait un upload de fichier ou
   qui a besoin d'un bus pub/sub
3. **Sprint 8 est déjà lourd** : 19 tabs (~2800 LOC React + ~1500
   LOC Python handlers) + 2 SDK impl + legacy removal + 6 P2
   hygiène = ~8000 LOC déjà au budget. Ajouter 4 primitives infra
   sans consommateurs pousserait à >12000 LOC et dégraderait la
   qualité
4. **Sprint 9 peut consacrer un sprint propre** à ces 4 items avec
   de vrais consommateurs (Sprint 10 apps qui les nécessitent)
5. **DB migration runner** est luxueux — gov utilise son schema
   SQLite existant via l'override `on_start` sans toucher aux
   migrations

**Rejeté** :

- Implémenter tout d'un coup → Sprint dépasse 18 jours, qualité
  dégradée, tests insuffisants
- Implémenter storage+events uniquement, défer upload+migration →
  split arbitraire, file upload et migration runner restent en
  limbo
- Descoper les tabs de gov (passer à 12 au lieu de 19) → casse le
  goal SBFB « gov doit être un vrai produit v1.1 pas une démo »

**Implications** :

- Scope cut explicite §6 pour les 4 items
- `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md` tech debt
  sections à updater en Phase F Sprint 8 pour refléter le décalage
- Memory `nexus_grid_pivot.md` roadmap Sprint 9 à mettre à jour en
  fin de Sprint 8
- **Aucun code Sprint 8 ne touche `storage` / `events` / file
  upload / DB migration runner** — l'audit Sprint 9 Phase 0 fera un
  grep dédié pour vérifier

## 5. Phase outline A..F (Sprint 8 proper)

### Phase A — SDK core + P2 hygiène Sprint 7 (~1800 LOC)

Scope :

- **D1** `AppContext.submit_task` impl + `NexusApp.resolve_worker`
- **D2** `@nexus_command` + `CommandDescriptor` + `NexusApp.commands()`
  + coord routes + Zod mirror + palette 4e groupe
- **D4** legacy descriptor fallback removal (2 helpers + 4 tests
  retirés, Zod simplifié côté shell, PATTERNS.md P8 mis à jour)
- **6 P2 hygiène Sprint 7 absorbés** :
  - A-3 cross-lang curator fixture (Python sign → Zod parse)
  - A-4 CuratorProjectRef string length caps (Rust + Zod miroir)
  - C-2 `NotSubscribed` vs `EnvelopeMismatch` error split
  - D-1 `process_name_matches` bounded substring
  - F-1 Curators page refresh button (mirror Browse)
  - G-3 `#[serde(deny_unknown_fields)]` sur daemon HTTP DTOs

**Critère Phase A acceptation** : `cargo test --workspace` passe
(304 → ~310 avec les 6 P2 tests), `uv run pytest packages/nexus-sdk/`
passe (40 → ~55 avec submit_task + @nexus_command), `uv run pytest
packages/nexus-coordinator/` passe (57+1 → ~63 avec les 2 nouvelles
routes), `web/ tsc strict` + `npm run test:unit` (114 → ~122), `npx
playwright test` (13 → ~15 avec palette + task_submit flows).
Commit cible : `feat(sdk,coordinator,web,shell-daemon): Sprint 8
Phase A — SDK extensions + legacy removal + Sprint 7 hygiene`.

### Phase B — `AppContext.db` + gov Batch 1 (6 tabs) (~1600 LOC)

Scope :

- **D3** `AppDatabaseClient` impl + 8 tests SDK
- `GovApp.on_start()` override pour pointer `ctx.db` vers le
  fichier SQLite legacy
- **6 tabs gov** (lecture directe via `ctx.db.fetchall`) :
  1. **Dashboard** — stats agrégées
     (`SELECT count FROM politicians/contradictions/alerts`)
  2. **Politiciens** — liste paginée des députés
  3. **Politicien detail** — fiche individuelle (bio + positions)
  4. **Biography** — onglet détaillé d'une bio
  5. **Positions** — votes + déclarations par sujet
  6. **Subjects** — topics d'analyse

Chaque tab est un `@nexus_tab`-décoré handler qui retourne un
`TabView` pydantic-valide construit via les helpers
`nexus_sdk.view.{heading, text, metric, table, section, ...}`. Les
charts sont faits en `chart_line` / `chart_bar` TabView primitives.

**Critère Phase B acceptation** : les 6 tabs affichent des données
réelles (ou fixture statique si le DB legacy est vide) à
`/app/gov/tabs/<tab_name>/descriptor`, 3 Playwright tests de golden
path, 8 SDK tests de `AppDatabaseClient`. Commit :
`feat(sdk,app-gov): Sprint 8 Phase B — AppContext.db + gov batch 1
(Dashboard/Politicians/Biography/Positions/Subjects)`.

### Phase C — gov Batch 2 (7 tabs) (~1500 LOC)

Scope : **7 tabs** mid-complexity :

7. **Contradictions** — upgrade du stub Sprint 4 vers un TabView
   avec table paginée + metrics + chart de contradictions par
   subject
8. **Scan** — status des jobs de scraping (politicians, laws,
   factchecks)
9. **Workers** — liste des workers legacy + leur état
10. **Pipeline** — état du pipeline ETL
11. **Social** — feed Twitter/Facebook/Insta par politicien
12. **Press** — communiqués de presse scraped
13. **Transcriptions** — transcriptions vidéo / audio

Chaque tab : lecture via `ctx.db` + rendu TabView. Pas de mutations
(pas de lancement de scan — juste affichage état).

**Critère Phase C acceptation** : 7 nouveaux `@nexus_tab` passent
leur descriptor call, 2 Playwright tests golden, gov test suite à
~10 tests. Commit : `feat(app-gov): Sprint 8 Phase C — gov batch 2
(Contradictions/Scan/Workers/Pipeline/Social/Press/Transcriptions)`.

### Phase D — gov Batch 3 (6 tabs) (~1500 LOC)

Scope : **6 tabs** — les plus lourds :

14. **Alerts** — liste des alertes avec filtrage read/unread
15. **Affairs** — dossiers / affaires par politicien
16. **Laws** — propositions de loi + votes
17. **Factchecks** — résultats factcheck existants
18. **Search** — barre de recherche RAG (ChromaDB via un worker
    dédié lancé par `ctx.submit_task("gov.rag_search", {"query":
    "..."})`)
19. **Ask** — question ouverte RAG (même pattern via
    `ctx.submit_task("gov.rag_ask", {"question": "..."})`)

**Search** et **Ask** utilisent `AppContext.submit_task` pour
valider le D1 end-to-end (c'est le premier consommateur réel du
D4 frozen).

**Critère Phase D acceptation** : 6 nouveaux tabs, 2 workers
`gov.rag_search` / `gov.rag_ask` enregistrés via `@nexus_worker`,
Search et Ask affichent les résultats dans un TabView après
soumission de tâche. Commit :
`feat(app-gov): Sprint 8 Phase D — gov batch 3
(Alerts/Affairs/Laws/Factchecks/Search/Ask)`.

### Phase E — Commands + polish + Playwright end-to-end (~1200 LOC)

Scope :

- **3-5 `@nexus_command` entries** sur `GovApp` :
  1. « Nouveau scan politiciens » → navigate `/app/gov/tabs/scan`
  2. « Détecter contradictions » → navigate
     `/app/gov/tabs/contradictions`
  3. « Recherche factchecks » → navigate
     `/app/gov/tabs/factchecks`
  4. « Voir alertes » → navigate `/app/gov/tabs/alerts`
- Palette 4e groupe "App" fetche ces commands pour chaque app
  enrollée et les expose sous le nom de l'app
- `CommandPalette.tsx` : merge side-by-side avec les groupes
  Navigation / Projets / Actions existants
- gov tabs final polish : skeleton loaders, error states, empty
  states, sort dropdowns sur les tables
- Playwright end-to-end : `gov-commands-flow.spec.ts` ouvre la
  palette via Ctrl+K, pick « Détecter contradictions », asserts
  navigation vers le bon tab + le tab load

**Critère Phase E acceptation** : 2 Playwright tests
(`gov-commands-flow` + `gov-batch3-rag`), `npm run test:coverage`
≥ 90% lines sur `CommandPalette.tsx`, `npm run size` budgets tous
verts (main < 475, vendor-react < 210, vendor-ui < 50, css < 100).
Commit : `feat(app-gov,web): Sprint 8 Phase E — gov @nexus_command +
palette integration + polish`.

### Phase F — Sortie de sprint (obligatoire cf `sprint_audit_gate.md`)

**Livrables côte à côte** :

1. `.planning/sprint8_verification.md` — self-report fail-fast
   checklist (format Sprint 6/7, ≥28 rows)
2. `.planning/sprint8_audit_plan.md` — plan d'audit que la session
   fraîche de Sprint 9 Phase 0 jouera. **9 tracks minimum** :
   - A — TabView descriptor removal verification (no regression)
   - B — AppContext.submit_task contract + wiring
   - C — @nexus_command contract + palette integration
   - D — AppContext.db read-only boundary + SQL injection surface
   - E — gov tabs data fidelity vs legacy endpoints
   - F — Command palette UX across states
   - G — Scope cut verification (no storage/events/upload/migration)
   - H — Cross-dependency hygiene (aiosqlite, dep bumps)
   - I — Documentation coherence
3. `docs/shell/PATTERNS.md` — mise à jour P8 (TabView unique
   contrat, legacy retiré), ajout P10 (command palette contrat
   app-contributed), fermeture T4 + T5 tech debt
4. `docs/rust/PATTERNS.md` — ajout section Sprint 8 canonical si
   des patterns Rust ont été touchés par les hygiène P2 Phase A
5. Mise à jour `nexus_grid_pivot.md` memory :
   Sprint 8 CLOSED tip, transition Sprint 9 scope (4 infra items
   déférés + P2 hygiène restante)

**Sans ces deux fichiers planning, Sprint 8 ne peut pas être
fermé.** C'est le point non-négociable de `sprint_audit_gate.md`.

## 6. Scope cuts (à respecter strictement)

### Infra items déférés Sprint 9 (via D5)

- **Pas de `AppContext.storage`** (KV primitif) — Sprint 9
- **Pas de `AppContext.events`** (pub/sub) — Sprint 9
- **Pas de file upload endpoint** — Sprint 9
- **Pas de DB migration runner** — Sprint 9 (gov réutilise son
  schema existant)

### Sprint 7 tech debt NON traitée

- **E-1 probe_reachable 2s timeout** — tech debt Sprint 9
- **C-4 gossip backpressure** — tech debt Sprint 9
- **D-3 subscriptions persist order** — tech debt Sprint 9
- **H-3 nexus_core wheel install drift** — tech debt Sprint 9
- **F-3 CardTitle accessibility** — Sprint 9 polish
- **G-1 httpx client per-call + limits** — Sprint 9

### Scope architecture

- **Pas de Reseau graph / Leaflet map** — Sprint 10+ (v1.2 scope)
- **Pas de real scrape/scan jobs triggerés depuis gov tabs** —
  les tabs sont READ-ONLY sur le DB existant, aucun tab ne lance
  un worker de scraping en Sprint 8
- **Pas de mutations via `nexus_route` POST/PUT/DELETE** —
  les 45 endpoints legacy `nexus/gov/api.py` ne sont PAS tous
  portés : seules les routes lecture (GET) sont consommées via
  `ctx.db`, les mutations attendent un Sprint 9+
- **Pas de re-intro reagraph / D3 / Leaflet** dans le shell —
  les charts restent en `chart_line` / `chart_bar` TabView SVG
  inline
- **Pas de mobile responsive < 1280px** — reconfirm Sprint 5 D3
- **Pas d'auth sur les `@nexus_command` invoke** — loopback
  trust, même modèle que `/tasks/submit`

## 7. Traçabilité scope (Sprint 7 "What's NOT" — suite)

| Item Sprint 7 "What's NOT" | Sprint | Phase | Status |
|---|---|---|---|
| `AppContext.submit_task` implémentation | **8** | A | **D1 impl, frozen signature Sprint 7 Day 0** |
| `@nexus_command` décorateur | **8** | A | **D2 impl, frozen signature Sprint 7 Day 0** |
| Migration d'un tab gov | **8** | B + C + D | **D3 pilot via `AppContext.db`, 19 tabs** |
| Extension `AppContext.storage` / `.events` | **déféré 9** | — | D5 scope cut |
| Extension `AppContext.file_upload` / migration runner | **déféré 9** | — | D5 scope cut |
| Bootstrap peers VPS FlowUP | Sprint 10 | — | scope cut confirmé |
| pkarr publish | Sprint 10 | — | scope cut confirmé |
| Multi-instance daemon | rejeté D2 S7 | — | non rebattable |
| Topic gossip namespacé | rejeté D3 S7 | — | non rebattable |
| Persist SQLite des curator lists | Sprint 9 polish | — | tech debt |
| Browse filter / search UI | Sprint 9 polish | — | tech debt |
| Icônes dynamiques par curator | v1.2 | — | refusé |

## 8. Audit gate pattern — rappel

Sprint 8 est le **deuxième cycle complet** du pattern
`sprint_audit_gate.md` :

- Phase 0 a été jouée → `.planning/sprint7_audit_findings.md` +
  commit `2ed0955` (DONE avant ce kickoff, verdict PASS)
- Phase F sera obligatoire → Sprint 8 doit livrer
  `.planning/sprint8_audit_plan.md` pour que Sprint 9 Phase 0
  puisse jouer son audit sur une session fraîche

Exception possible uniquement si l'utilisateur demande explicitement
de skipper l'audit — dans ce cas, noter « Phase 0 audit skipped per
user decision YYYY-MM-DD » et prévoir un audit rétroactif Sprint 9.

## 9. Checkpoint de validation

Avant d'écrire le code Sprint 8 Phase A (premier commit
`feat(sdk,coordinator,web,shell-daemon): Sprint 8 Phase A — …`),
l'utilisateur doit :

1. Valider les **5 décisions Day 0 D1..D5** — en particulier **D5**
   qui défère 4 items infra du memory `nexus_grid_pivot.md` à
   Sprint 9. Si tu veux garder les 4 items dans Sprint 8, pousse
   back maintenant et on redécoupe Phase A-F.
2. Valider le **split Phase A..F** et l'ordre (A SDK core + hygiène
   → B DB bridge + gov batch 1 → C gov batch 2 → D gov batch 3 →
   E commands + polish → F sortie)
3. Confirmer les **scope cuts §6** — notamment le fait que les
   tabs sont READ-ONLY sur le DB legacy, qu'aucun tab ne lance un
   worker de scraping, et que les mutations legacy `nexus/gov/api.py`
   (POST/PUT/DELETE) ne sont PAS portées Sprint 8
4. Valider que le plan détaillé `.planning/sprint8_plan.md` (commité
   atomiquement avec ce kickoff) reflète bien ces décisions avec
   la grille d'exécution, la fail-fast checklist ~28 rows, et les
   risques R1..Rn

---

**État** : kickoff rédigé, 5 décisions Day 0 gelées en attente de
validation, plan détaillé dans `.planning/sprint8_plan.md`. Aucun
commit code Sprint 8 ne peut atterrir avant que ces deux docs soient
commités via `docs(sprint8): kickoff + plan` et que l'utilisateur
valide D1..D5.
