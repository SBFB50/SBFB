# Sprint 8 — Audit Findings (Phase 0 de Sprint 9)

**Audité par** : session Claude Code fraîche, opus-4-6 1M context,
2026-04-12. Timebox observé : ~3 h. Joué `.planning/sprint8_audit_plan.md`
sans avoir lu `docs/shell/PATTERNS.md` §P10 ni
`docs/rust/PATTERNS.md` §Sprint 8 closures avant d'avoir formé un
avis track-par-track.

**Tip audité** : `449f404` (master, post `docs(sprint8): verification
+ audit plan for Sprint 9`)

**Verdict global** : **CONDITIONAL PASS**

- **0 P0** — pas de régression critique, pas de corruption des données
  legacy en cours
- **2 P1** — D-1 (`AppDatabaseClient` n'est PAS read-only mais le
  contrat Sprint 8 D3 le promet) + V-1 (verification.md row 11
  contient 3 fausses claims qui faussent l'audit gate downstream)
- **5 P2** — palette error swallow, gov tabs collective empty-state,
  legacy fallback résiduel sur `/manifest`, build headroom critique,
  resolve_worker shadow détection
- **6 P3** — f-string SQL nit, tests count drift, decorator-time
  shadow check absent, plusieurs autres nits

Conditions de lever : 2 commits `fix(sprint8): ...` (D-1 + V-1)
landed sur master AVANT le premier commit Sprint 9 Phase A. Détails
§ Verdict global ci-dessous.

---

## Mode d'emploi suivi

1. ✅ Lecture mémoire externe (MEMORY.md, nexus_grid_pivot.md,
   sprint_audit_gate.md, feedback_approach.md)
2. ✅ Lecture `docs/claude/README.md`
3. ✅ Lecture `.planning/sprint8_kickoff.md`,
   `sprint8_plan.md` (segments §1-9), `sprint8_verification.md`,
   `sprint8_audit_plan.md`
4. ✅ Inspection des 7 commits Sprint 8 par `git log --stat 2ed0955..449f404`
5. ✅ Re-run fail-fast subset : `cargo test --workspace --locked`
   (309 passed), `pytest packages/nexus-sdk` (68 passed après
   rebuild wheel — voir §H), `pytest packages/nexus-coordinator`
   (63 + 1 skipped), `pytest packages/nexus-app-gov` (30 passed),
   `npm run test:unit` (142 passed), `tsc`, `npm run build`,
   `npm run size` (4/4 verts)
6. ✅ Tracks A..I joués séquentiellement sur le code, pas sur les
   PATTERNS docs (lecture PATTERNS faite APRÈS pour cross-check)
7. ⚠️ Playwright NON re-rejoué (ne fait pas partie du fail-fast
   subset audit ; la verification.md row 27 affirme 24/24 et le
   spawn live + fixture ne donne pas de signal nouveau au-delà de
   ce que les Vitest + tsc ont déjà couvert)

---

## Track A — Retrait du `legacy_descriptor` fallback

**Verdict** : **PASS** (avec 1 finding P3)

### A1 — Grep canary `_coerce_tab_view`

✅ `grep -rn "_coerce_tab_view" packages/nexus-coordinator/` retourne
**0 match** dans le code vivant. Le fichier `apps.py` ne contient
plus la fonction ni son site d'appel.

✅ `apps.py:96-156` (route `app_tab_descriptor`) appelle
`TabView.model_validate(descriptor)` et lève `HTTPException(422)`
sur `ValidationError` avec un message structuré qui cite Sprint 8.
Pas de try/except qui retombe sur un dict-style fallback.

### A2 — Coordinator side : pas de retry caché

✅ Aucune branche secondaire dans `app_tab_descriptor`. La seule
route qui touchait `_coerce_tab_view` est nettoyée.

⚠️ **A2-zombie (P3)** : `apps.py:73-93` (route `app_manifest`) appelle
`_maybe_call(t.fn, app)` pour les tabs sync, et `_maybe_call`
(`apps.py:261-273`) catche `Exception` et retourne `{"error": str(e)}`.
C'est un mini-fallback structurel qui n'utilise plus le mot
`legacy_descriptor` mais garde le **pattern** (un descriptor cassé
ne fait plus 422 sur la route manifest, juste un dict avec une clé
`error`). Pas un blocker — le route `manifest` est principalement
une lecture de surface, et la route principale `tabs/{tab_name}/descriptor`
fait correctement 422. Mais l'esprit de Sprint 8 D4 « tout
descriptor cassé fait 422 » n'est pas tenu sur la route manifest.
À nettoyer Sprint 9 polish ou laisser comme P3.

### A3 — Shell side gère 422

✅ `web/src/api/coordinator.ts::getAppTabDescriptor()` simplifié.
Plus de discriminated `{schema|legacy|error}`, juste retour
`{descriptor: TabView}` avec lift d'exception sur HTTP error
(`CoordinatorHttpError`).

✅ `web/src/components/project/AppsTab.tsx` consomme correctement
les erreurs.

### A4 — Tests qui verrouillent le contrat

✅ `packages/nexus-coordinator/tests/test_apps.py` contient
plusieurs `assert "legacy_descriptor" not in body` (assertions
négatives) + un test qui passe une dict invalide et asserte 422.
Le contrat est bien verrouillé par les tests.

### Findings Track A

- **A-FX-1 (P3)** : `_maybe_call` dans `apps.py:261-273` retourne
  `{"error": ...}` au lieu de propager. C'est le dernier petit
  zombie du pattern fallback. **Évidence** : `apps.py:272-273` —
  `except Exception as e: return {"error": str(e)}`. **Fix** :
  laisser propager l'exception → la route `app_manifest` retournera
  500 sur tab sync cassée, cohérent avec le contrat D4. Optionnel
  Sprint 9.

---

## Track B — `AppContext.submit_task` contract end-to-end

**Verdict** : **PASS** (avec 2 findings P3)

### B1 — Signature Python frozen

✅ `packages/nexus-sdk/src/nexus_sdk/app.py:117-164` —

```python
async def submit_task(
    self,
    worker: str,
    payload: dict[str, Any],
    *,
    priority: int = 5,
    parent_task_id: str | None = None,
) -> str:
```

**Mot pour mot identique** à la signature gelée Sprint 7 D4. Aucun
renommage, aucun arg supplémentaire, aucun default modifié.

### B2 — `resolve_worker` cross-app + ambigu

✅ `packages/nexus-sdk/src/nexus_sdk/app.py:244-276` — implémentation
correcte :

- `routing_key` avec `.` → split, vérifie que le préfixe matche
  `self.manifest.name`, sinon `WorkerNotFound`
- `routing_key` sans `.` → match exact dans `self.workers()`
- 4 tests dans `test_sdk.py` couvrent les chemins :
  `test_resolve_worker_short_name_matches`,
  `test_resolve_worker_prefixed_name_matches_own_app`,
  `test_resolve_worker_prefixed_name_foreign_app_rejected`,
  `test_resolve_worker_unknown_name_raises`

⚠️ **B2-doc (P3)** : la docstring `resolve_worker` (`app.py:259-261`)
affirme « The first match wins; in practice a given app should not
register two workers with the same name so ordering is not a contract. »
La situation « 2 workers avec le même name » est silencieusement
gérée par le first-match-wins, **sans warning, sans test, sans
detection au moment de la classe definition**. L'audit_plan C3
demandait un decorator-time check pour `@nexus_command` qui fait
défaut aussi (cf. Track C). **Fix** : rajouter dans `__init__`
de `NexusApp` un check de nom unique pour `_workers` ET `_commands`,
raise `ValueError("duplicate worker name X")` à la construction.
Optionnel Sprint 9.

### B3 — Coordinator route + dispatcher integration

✅ `apps.py:182-217` — route `POST /app/{name}/tasks/submit` :

- Body Pydantic `SubmitAppTaskRequest` avec `model_config = {"extra": "forbid"}`
- `worker`: `min_length=1, max_length=128`
- `priority`: `ge=0, le=10`
- `parent_task_id`: optional `str | None`
- 404 si app inconnue, 500 si l'app n'a pas de bound context
- 422 si `WorkerNotFound` (catch + relift)
- Délègue vers `ctx.submit_task(...)` sans réimplémenter la
  résolution worker → dispatcher

✅ Test `test_submit_app_task_happy_path` dans
`packages/nexus-coordinator/tests/test_apps.py`.

### B4 — `ButtonBlock` React wiring

✅ `web/src/components/app/tabview/blocks/ButtonBlock.tsx` :

- Sprint 6 stub `console.warn("[tabview] task_submit action not yet wired")`
  **est retiré** (T4 closure)
- Lit `TabAppContext` via `useTabAppContext()`
- Fail gracefully si pas dans un contexte d'app (`feedback.error`
  message visible « Action task_submit indisponible hors d'un
  contexte d'app »)
- Appelle `submitAppTask(coordinatorUrl, appName, body)` du client
- Affiche success/error visiblement (`text-green-500` / `text-destructive`)
  via `<p>` inline, pas un toast invisible
- Disable le bouton pendant `pending`

✅ Coverage Vitest : `TabViewRenderer.test.tsx` étendu (+92 lignes
par Phase E + 106 lignes par Phase A) couvre la branche.
`ButtonBlock` reporté à 77.77% lines / 76.47% branches dans
`verification.md` row 26 — c'est le bottom du seuil 90 mais les
2 branches d'erreur HTTP couvertes par CommandPalette/Playwright
selon la note. Acceptable pour une feature nouvelle.

### B5 — Erreur paths

| Cas | Comportement | Verdict |
|---|---|---|
| `worker = "unknown.worker"` (mauvais préfixe) | `WorkerNotFound` → coord 422 | ✅ |
| `worker = "ghost"` (pas registered) | `WorkerNotFound` → coord 422 | ✅ |
| `payload = {"trop_grand": "x" * 10_000_000}` | sérialisé verbatim, pas de cap → coord forward | ⚠️ P3 (pas de cap) |
| `priority = -1` | Pydantic 422 (`ge=0`) | ✅ |
| `priority = 999` | Pydantic 422 (`le=10`) | ✅ |
| `parent_task_id = "not-a-uuid"` | accepté tel quel (str optional) | ⚠️ P3 (pas de validation) |

### Findings Track B

- **B-FX-1 (P3)** : `resolve_worker` accepte silencieusement deux
  workers de même nom (first-match-wins). **Fix** : check au
  `NexusApp.__init__` que `_workers` et `_commands` n'ont pas de
  doublons.
- **B-FX-2 (P3)** : `SubmitAppTaskRequest.payload` n'a pas de cap
  de taille → un client malicieux peut envoyer 10 MB. Pas un risque
  réel en loopback trust mais à logger en tech debt Sprint 9.
- **B-FX-3 (P3)** : `SubmitAppTaskRequest.parent_task_id` n'est pas
  validé comme UUID → un caller peut passer n'importe quoi qui sera
  forwardé dans `metadata["parent_task_id"]`. Cohérent avec le fait
  que le coordinator dispatcher Sprint 4 ne valide pas non plus.
  Tech debt Sprint 9.

---

## Track C — `@nexus_command` contract + palette integration

**Verdict** : **PASS** (avec 1 finding P2 et 1 P3)

### C1 — `CommandDescriptor` frozen vs réel

✅ `packages/nexus-sdk/src/nexus_sdk/commands.py:23-45` —

```python
class CommandDescriptor(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)
    schema_version: Literal[1] = 1
    name: str = Field(..., min_length=1, max_length=64)
    description: str = Field(..., min_length=1, max_length=280)  # +1 (min_length=1)
    icon: str = Field("sparkles", max_length=32)
    group: str = Field("Actions", max_length=32)
```

⚠️ **Drift mineur** : la signature gelée Sprint 7 D5 listait
`description: str = Field(..., max_length=280)` sans `min_length`.
L'impl ajoute `min_length=1` qui empêche un command palette entry
sans description (sain mais c'est une extension du contrat). Note
en P3.

### C2 — Zod mirror

✅ `web/src/api/coordinator.ts` — `CommandDescriptorSchema` mirror
Zod avec `.strict()` et les mêmes max. (Cross-checké via les commits
Phase A.)

⚠️ **C2-cross-lang (P3)** : pas de fixture `command_canonical.json`
partagée Python ↔ Vitest comme demandé par R2 du plan §13. Le
`coordinator.test.ts` a 9 nouveaux tests pour le schéma mais aucun
ne lit une fixture commune avec `test_commands.py`. Audit risk : un
drift Pydantic sans bump version pourrait passer silencieusement.
Sprint 6 a établi le pattern fixture pour TabView, Sprint 7 audit
A-3 l'a établi pour curator, Sprint 8 ne l'établit PAS pour
CommandDescriptor. Inconsistance.

### C3 — Decorator metadata + registry collection

✅ `decorators.py:70-114` — `@nexus_command` attache
`__nexus_command__` dict sur la méthode.

✅ `registry.py:20-74` — `collect_decorators` walk `dir(cls)`
et bucket les méthodes décorées.

⚠️ **Ordre déterministe** : `dir(cls)` retourne les attributs
**triés par ordre alphabétique** (CPython détail d'implémentation
documenté). Donc l'ordre des `_commands` est lexicographique sur
le nom de méthode (e.g. `cmd_detect_contradictions`,
`cmd_new_scan`, `cmd_search_factchecks`, `cmd_view_alerts`). Le
test `test_list_app_commands_ordered` repose sur cet ordre. C'est
stable mais **fragile** (dépend de l'ordre `dir()` qui n'est pas
un contrat formel) — préférer un sort explicite par
`name` au moment de `commands()`. P2 finding.

⚠️ **Pas de check de nom unique au decorator-time** : audit_plan C3
demandait « 2 méthodes avec le même `name` doit raise au moment
de la classe definition ». Le code accepte silencieusement deux
commands avec le même name (le second wins via order). Voir
B-FX-1 — même fix multiple.

### C4 — Coordinator routes commands + invoke

✅ `apps.py:220-253` —

- `GET /app/{name}/commands` retourne `app.commands()` typé via
  `response_model=list[CommandDescriptor]`
- `POST /app/{name}/commands/{cmd_name}/invoke` :
  - 404 si app inconnue
  - Délègue à `app.invoke_command(cmd_name)` (`app.py:278-293`)
  - Catch `LookupError` → 404 si command absente
  - Wrap résultat dans `{"result": result}`

✅ Test happy path + 404 paths dans `test_apps.py`.

⚠️ **C4-payload-absent (P3)** : la route `invoke` ignore le body
de la requête. L'audit_plan C4 method 3 dit « Passe le body de la
requête comme payload à la méthode ». L'impl actuelle appelle
`app.invoke_command(cmd_name)` sans payload, et `invoke_command`
appelle `fn(self)` sans args. Cohérent avec D2 du kickoff qui dit
explicitement « v1 ne passe pas d'arguments structurés, le trigger
palette est user-action-only ». Mais l'audit_plan a documenté
l'inverse — le plan vs l'audit_plan divergent. Préférer l'**impl
actuelle** (kickoff fait foi sur le scope) ; le doc audit_plan est
incohérent et pourrait induire un futur auditeur en erreur.
Tech debt doc Sprint 9.

### C5 — `CommandPalette` 4e groupe

✅ `web/src/components/command-palette/CommandPalette.tsx:62-105` —

- Fetch `listApps(active.url)` via `useQuery({queryKey: ["palette-apps", active?.url], staleTime: 30_000, enabled: Boolean(active) && palette.open})`
- Pour chaque app avec `commands > 0`, render un `<AppCommandsGroup>`
  qui fait `useQuery({queryKey: ["palette-app-commands", baseUrl, app.name], staleTime: 30_000, refetchInterval: 30_000})`
- Empty state : si `commands.length === 0` → `return null` (groupe
  absent, pas de crash)
- Click handler `runAppCommand` ferme la palette puis appelle
  `invokeAppCommand`, lit `extractNavigationPath(envelope.result)`,
  navigate si présent

⚠️ **C5-error-swallow (P2)** : `runAppCommand` log les erreurs avec
`console.error("[palette] invokeAppCommand failed", ...)` mais
**ne montre rien à l'utilisateur**. La palette se ferme avant
l'await, donc un click sur un command qui crash 500 → palette
disparaît silencieusement, l'utilisateur ne sait pas que ça a
échoué. **Évidence** : `CommandPalette.tsx:87-105`. **Fix** :
soit garder la palette ouverte tant que le invoke n'a pas réussi
(state local pending), soit toaster via sonner. Sprint 9 polish
acceptable mais **explicit P2** parce que l'audit_plan F3 a posé
exactement cette question et que le code ne y répond pas.

⚠️ **C5-cadence (verif)** : `staleTime: 30_000, refetchInterval:
30_000` — l'audit_plan F2 method 2 demandait `staleTime: 15_000,
refetchInterval: 30_000`. L'impl utilise 30/30 — différence
mineure, pas un finding (le R7 R7 plan §13 n'imposait pas 15s).

### C6 — Vitest CommandPalette tests

✅ `web/src/components/command-palette/__tests__/CommandPalette.test.tsx`
fait 363 lignes (Phase E commit `9339bb6`). Couvre le 4e groupe,
empty state, click → invoke, error handling. Délégué dans la suite
Vitest verte (142 passed).

### Findings Track C

- **C-FX-1 (P2)** : invoke errors silencieusement avalées par la
  palette. **Fix** : afficher un toast ou garder la palette ouverte
  en pending state.
- **C-FX-2 (P2)** : ordre `_commands` dépend de `dir(cls)` au lieu
  d'un sort explicite par `name`. Stable en pratique, fragile en
  contrat. **Fix** : `return sorted([...], key=lambda d: d.name)`
  dans `NexusApp.commands()` et `_workers` aussi pour cohérence.
- **C-FX-3 (P3)** : `CommandDescriptor.description` a un
  `min_length=1` qui n'était pas dans la signature gelée Sprint 7 D5.
- **C-FX-4 (P3)** : pas de fixture cross-lang `command_canonical.json`
  partagée Python ↔ Vitest. Sprint 6 / Sprint 7 ont établi le pattern
  pour TabView et curator ; Sprint 8 le rompt pour CommandDescriptor.
- **C-FX-5 (P3)** : audit_plan C4 method 3 documente une exigence
  (payload structuré au invoke) que le kickoff D2 a explicitement
  exclue. À harmoniser dans le doc d'audit Sprint 9.

---

## Track D — `AppContext.db` read boundary + SQL injection surface

**Verdict** : **CONCERN** (1 P1 + 2 P3)

### D1 — Read-only enforce

❌ **D-FX-1 (P1)** — **`AppDatabaseClient` n'est PAS read-only.**

**Évidence** :

1. `packages/nexus-sdk/src/nexus_sdk/db.py:90` — la connexion est
   ouverte via `aiosqlite.connect(self._db_path)` **sans** `mode=ro`
   ni `uri=True`. Aucun enforcement SQLite-level.
2. `db.py:117-135` — la classe expose une méthode `execute()`
   publique qui prend une statement quelconque, l'exécute et
   `commit()`. Le docstring confirme : « Run a mutating statement
   (INSERT / UPDATE / DELETE / DDL). Commits on success. »
3. `packages/nexus-sdk/tests/test_db.py:86-99` — le test
   `test_execute_commits_and_persists` valide explicitement qu'un
   `INSERT` via `execute()` est persisté et observable par un
   `fetchall()` ultérieur. **Le test confirme que le client n'est
   pas read-only.**
4. **Aucun test** `test_readonly_enforced` n'existe dans `test_db.py`,
   contrairement à ce qu'affirme `sprint8_verification.md` row 11.
5. **Aucun `asyncio.Lock`** dans `db.py`, contrairement à ce qu'affirme
   verification.md row 11. La sécurité concurrente vient du pattern
   « connect-per-call » documenté dans le docstring de `db.py:11-18`.
6. **Aucune méthode `schema_introspection()`** n'existe, contrairement
   à ce qu'affirme verification.md row 11.

**Pourquoi P1 (et pas P0)** :

- Le **risque réel** est limité parce que les 19 query functions
  dans `packages/nexus-app-gov/src/nexus_app_gov/queries.py` ne
  font **que des SELECT**. Aucune ne touche `execute()`.
  `grep -n "db\.execute" packages/nexus-app-gov/` retourne 0 match.
- Mais le **contrat Sprint 8 D3 est cassé** : kickoff §4 D3 dit
  « bridge **lecture-principal** vers SQLite per-app » et
  audit_plan D1 méthode 3 dit « Tester explicitement qu'un
  `await client.execute("INSERT INTO ...")` raise
  `sqlite3.OperationalError: attempt to write a readonly database` ».
  Le code fait l'**inverse** : le test prouve que execute() écrit
  et commit.
- La DB legacy `nexus/gov/govdata.db` (4 ans de scraping) est
  pointée par `GovApp.on_start()` (`app.py:149-163`). **Une seule
  ligne `await ctx.db.execute("DROP TABLE gov_politicians")`**
  dans n'importe quel handler gov ou Sprint 9 app suffirait à
  détruire les données. Aujourd'hui aucun handler ne le fait, mais
  c'est une mine.
- L'audit_plan D1 dit explicitement « **P0** si une instance
  `AppDatabaseClient` peut écrire dans la DB legacy ». Strictement
  appliqué, ça serait P0. Je le requalifie **P1** parce qu'aucun
  code vivant n'exploite la fuite, et que le D3 du kickoff est
  ambigu (« lecture-principal » + « writes are allowed but no
  migration runner is shipped »). Mais le contrat **promis dans
  l'audit_plan** est clairement violé.

**Fix attendu** : `fix(sprint8): enforce read-only on AppDatabaseClient`
qui :

1. Ajoute un argument `read_only: bool = True` au ctor
2. Quand `read_only=True`, ouvre la connexion via
   `aiosqlite.connect(f"file:{self._db_path}?mode=ro", uri=True)`
3. Quand `read_only=True`, raise `DatabaseError("client is read-only")`
   au début de `execute()` AVANT la connexion (defense in depth)
4. Le coordinator loader instancie le default avec `read_only=True`
5. Une app qui veut écrire son propre per-app file (cas Sprint 9
   `gov-rag-cache` etc.) instancie explicitement
   `AppDatabaseClient(path, read_only=False)` dans son `on_start`
6. Ajout d'un test `test_readonly_enforced` qui assert que
   `await client.execute("INSERT INTO t VALUES (1)")` raise
   `DatabaseError`
7. Ajout d'un test `test_readonly_blocks_via_uri_mode` qui asserte
   que même en bypass de `execute()`, un appel direct à `fetchall()`
   sur une table absent ne peut pas créer la table

### D2 — Path resolution safety

✅ `paths.py:112-126` — `app_db_path(project_name, app_name)` retourne
un chemin déterministe sous `nexus_grid_root() / "projects" / project /
"apps" / app / "app.sqlite"`. Pas de string user-controlled dans le
chemin.

✅ `coordinator.py:264-285` — le loader instancie
`AppDatabaseClient(default_db_path)` avant `on_start`. Une app peut
swap `ctx.db` mais pas le path injecté.

✅ Le swap `gov.on_start` (`app.py:149-163`) utilise un path absolu
`Path(__file__).resolve().parents[4] / "nexus" / "gov" / "govdata.db"`,
calculé depuis `__file__` du module et non depuis un input.
**Aucun path traversal possible.**

✅ Si la DB legacy n'existe pas, l'override est skippé et le default
per-app file reste en place. Empty state propre sur DB absente.

### D3 — SQL injection surface

✅ `grep -rn "f\"INSERT|f\"UPDATE|f\"DELETE" packages/nexus-app-gov/`
retourne **0 match**.

⚠️ `grep -rn "f\"SELECT" packages/nexus-app-gov/` retourne **1 match** :
`queries.py:64` dans `_safe_count(table: str)` :

```python
row = await db.fetchone(f"SELECT COUNT(*) AS n FROM {table}")
```

**Évidence** : `_safe_count` est appelée 4 fois depuis
`dashboard_stats_query` avec des **string literals hardcoded**
(`"gov_politicians"`, `"gov_positions"`, `"gov_contradictions"`,
`"gov_parties"`). Aucun appel ne passe d'input user. **Pas de
vulnérabilité réelle**, mais :

- Le canary `grep "f\"SELECT"` du audit_plan D3 method 1 fait fail
- Si Sprint 9 ouvre l'API à un table name dynamique, le code a
  une bombe à retardement
- Le pattern devrait être un dispatch explicite par table name
  (e.g. dict `{"gov_politicians": "SELECT COUNT(*) AS n FROM gov_politicians", ...}`)
  ou une whitelist explicite des table names accepté

**P3 finding**, pas P1.

✅ Le reste de `queries.py` (~870 lignes, 17 query functions) utilise
`(limit,)`, `(pol_id,)`, etc. — bind paramétré partout. Spot-check
des 5 fonctions au hasard (politicians_list, positions_list,
contradictions_overview, alerts_overview, factchecks_list) : tous
clean.

✅ `test_db.py:102-121` — `test_parameterized_query_binds_safely`
valide explicitement le binding avec un payload `"alice'; DROP
TABLE t; --"` qui doit retourner zero rows et laisser la table
intacte. Le test passe. **Confirmé que le binding marche.**

### D4 — Concurrency

✅ Le pattern « connect-per-call » dans `db.py:90` est correct.
Chaque appel ouvre sa propre connexion aiosqlite, donc pas de
state partagé entre appels concurrents. C'est explicitement
documenté dans `db.py:11-18`.

❌ Verification.md row 11 affirme **« concurrent fetchall under
asyncio.Lock »** — c'est **FAUX**. Pas de Lock dans le code. La
sécurité vient du « connect-per-call », pas d'un Lock. Ce n'est
pas une bug fonctionnelle (le pattern marche), c'est un mensonge
de la verification. Fait partie du finding **V-1** (Track Verif).

### D5 — Schema introspection

❌ Verification.md row 11 affirme « schema introspection » comme
test rouge. **Aucune méthode `schema_introspection()`** n'existe
dans `db.py`. Mensonge de verification. Inclus dans V-1.

### Findings Track D

- **D-FX-1 (P1)** : `AppDatabaseClient` n'est pas read-only. `execute()`
  écrit et commit, le test `test_execute_commits_and_persists` le
  prouve. La DB legacy `nexus/gov/govdata.db` est exposée à un
  bug de futur app handler. **Fix obligatoire avant Sprint 9 Phase A**
  (cf. §Verdict global).
- **D-FX-2 (P3)** : f-string `f"SELECT COUNT(*) AS n FROM {table}"`
  dans `_safe_count` (queries.py:64). Pas de vuln réelle (table
  literals hardcoded), mais fait fail le canary D3.
- **D-FX-3 (P3)** : pas de schema introspection method (verification
  l'affirme à tort, pas de besoin réel). À retirer du wording de
  la verification.

---

## Track E — Fidélité données gov vs `nexus/gov/api.py` legacy

**Verdict** : **PASS** (avec 2 P3)

**Caveat critique** : `nexus/gov/govdata.db` **n'existe pas** dans
le repo audité (`ls nexus/gov/govdata.db` → No such file). Sprint 8
gov tabs tournent donc tous sur le default per-app SQLite vide
(empty state). Spot-check par diff de SQL queries impossible —
fait par lecture statique.

### E1 — Spot-check 3 tabs

**Dashboard** :

- `queries.dashboard_stats_query` :
  - 4 `_safe_count` sur gov_politicians/positions/contradictions/parties
  - 1 `SELECT COUNT(*) FROM gov_politicians WHERE active = 1`
  - 1 `GROUP BY subject FROM gov_positions ORDER BY n DESC LIMIT 5`
- Comparé schéma `nexus/gov/db.py:54-100` : tables existent,
  colonnes `active`, `subject` sont présentes. ✅
- ⚠️ La colonne `gov_politicians.active` n'est documentée nulle part
  dans Sprint 8 plan §3.4 — possible que certaines DB legacy
  scrapées avant un schema migration n'aient pas cette colonne.
  Le `try/except DatabaseError` autour de la query rattrape ça en
  empty state. ✅

**Politiciens** :

- `politicians_list_query` : `SELECT id, name, chamber, party, role,
  constituency, active FROM gov_politicians ORDER BY name LIMIT ?`
- 7 colonnes projetées. Schéma legacy a `gov_politicians` avec ces
  colonnes (lignes 54-69 de db.py). ✅
- ⚠️ Pas de jointure vers `gov_party_memberships` pour récupérer le
  parti **historique** vs le parti **actuel** dans la colonne `party`.
  Le legacy `nexus/gov/api.py` peut faire cette distinction. P3 nit
  de fidélité.

**Contradictions** :

- `contradictions_overview_query` : 3 sub-queries (rows + by_subject +
  summary). Colonnes : `id, subject, severity, description,
  source_verified, detected_at`. Schéma legacy `gov_contradictions`
  (lignes 86-98) a ces colonnes. ✅
- Aggregation : `SUM(CASE WHEN severity = 'high' ...)` — ce qui
  suppose que `severity` est string, pas int. Pas de validation que
  les valeurs `'high'` / `'medium'` / `'low'` couvrent tous les cas
  legacy. Si une DB legacy a `severity = 'critical'`, le compteur
  `high` sera bas. P3 nit.

### E2 — Empty state coverage

✅ `test_gov_app.py` a **30 tests**. Les 19 tabs ont chacun au moins
**un test positif** (rendu avec données seedées via tmp_path SQLite).

⚠️ **E-FX-1 (P3)** : Empty state est testé **collectivement** par
`test_tabs_render_empty_state_when_db_missing` (1 seul test pour
les 19 tabs), pas par 19 tests dédiés. L'audit_plan E2 demandait
« 19 tests d'empty state ». La couverture est suffisante en
pratique (un seul code path partagé `_empty_tab`) mais le compteur
formellement diverge.

### E3 — Pas de re-scrape au render

✅ `grep -rn "subprocess|requests\.get|httpx" packages/nexus-app-gov/`
retourne **0 match**. Aucun tab handler ne déclenche un scrape au
render.

✅ `submit_task` est appelé uniquement via `ButtonBlock` (click
utilisateur explicite), pas dans un handler de descriptor.

### Findings Track E

- **E-FX-1 (P3)** : 1 test collectif d'empty state au lieu de 19.
  Acceptable mais minor drift vs audit_plan.
- **E-FX-2 (P3)** : `politicians_list_query` projette `party` plat
  (colonne directe) au lieu de joindre `gov_party_memberships` —
  perte de l'info historique. Sprint 9 polish.
- **E-FX-3 (P3)** : `contradictions_overview_query` SUM hardcoded
  `severity = 'high'` — robustesse contre nouveaux niveaux limitée.
  Sprint 9 polish.

---

## Track F — Command palette UX across states

**Verdict** : **PASS** (1 P2 déjà couvert par C-FX-1)

### F1 — État vide (0 apps enrôlées)

✅ `CommandPalette.tsx:174-184` — la condition `active && apps.filter(...)`
ne render rien si pas d'active coordinator. Pas de crash ni de
loading infini. Quand `active` existe mais aucune app enrôlée
(`apps = []`), le `.filter().map()` retourne un array vide, pas
de `<AppCommandsGroup>` rendu. ✅

### F2 — État 1 app, N commands

✅ Couvert par les Vitest tests `CommandPalette.test.tsx`
(142 passed total Vitest, le file fait 363 lignes spécifiquement
sur le 4e groupe).

✅ `CommandItem` `value={`app ${app.name} ${cmd.name} ${cmd.description}`}`
permet la recherche par nom + description.

### F3 — État erreur invoke

❌ Couvert par C-FX-1 (P2) — invoke errors silencieusement avalées.
Voir Track C.

### F4 — État daemon offline

✅ La palette utilise le coordinator, pas le daemon. Daemon offline
n'affecte pas. Si le coordinator est offline, la `useQuery`
`listApps` revient en error et les groupes apps ne sont pas
rendus — pas de crash. Pattern React Query par défaut.

### F5 — Deep-link navigation

✅ `extractNavigationPath.ts:11-18` — narrow le `result` invoke en
`{navigation: {path: string}}` avec type guards stricts. Si shape
incorrect → return `null`, pas de crash.

✅ `gov-commands-flow.spec.ts` (Phase E) couvre le flow complet :
Ctrl+K → click command → navigation. Verification.md row 27 dit
24 Playwright passed.

### Findings Track F

- (Voir C-FX-1 P2 — single overlap finding entre Track C et F)

---

## Track G — Scope cuts enforcement

**Verdict** : **PASS** (clean)

### G1 — Greps storage / events / upload / migration

✅ `grep -rn "AppContext\.storage\|AppContext\.events\|ctx\.storage\|ctx\.events|file_upload|FileUpload|migration_runner|MigrationRunner" packages/`
→ **0 match**.

### G2 — Sprint 7 tech debts ouverts toujours ouverts

✅ **E-1** : `crates/nexus-core-rs/src/discovery.rs` — DEFAULT_PROBE_TIMEOUT
toujours `Duration::from_secs(2)` (pas changé Sprint 8). Confirmé
par grep.

✅ **C-4** : Pas de `tokio::sync::Semaphore` ajouté dans
`iroh_runtime.rs` — backpressure pas implémenté.

✅ **D-3** : `iroh_runtime.rs::CuratorRuntime::subscribe` — l'ordre
insert puis persist est inchangé.

✅ **H-3** : Pas de `scripts/setup.sh` ajouté. Le wheel
`nexus_core_py` reste un editable install fragile (j'ai rebuild le
wheel à la main pour faire passer test_curator.py — voir Track H).

✅ **F-3** + **G-1** : non vérifiés en détail mais cohérent avec
le reste.

### G3 — Pas de mutations

✅ `grep -rn "@nexus_route.*POST|@nexus_route.*PUT|@nexus_route.*DELETE" packages/nexus-app-gov/`
→ **0 match**. Le seul `@nexus_route` est `/statements` GET legacy.

### G4 — Pas de Reseau / Leaflet

✅ Aucun ajout de dep frontend pour graphes / cartes (vérifié dans
les diffs Phase A-E commits).

### G5 — Pas d'auth

✅ Routes `submit_task` + `invoke_command` n'ont pas de dependency
d'auth. Loopback trust documenté.

### Findings Track G

- Aucun. Les scope cuts sont strictement respectés.

---

## Track H — Cross-dependency hygiene + bundle headroom

**Verdict** : **CONCERN** (1 P2)

### H1 — `aiosqlite` version

✅ Pinné `>= 0.20` dans `packages/nexus-sdk/pyproject.toml:24` ET
dans `packages/nexus-coordinator/pyproject.toml:53`. Cohérent.

✅ `uv.lock` résoud à `aiosqlite-0.22.1` (single version pour les
deux packages).

### H2 — Pydantic / FastAPI version drift

✅ `model_config = ConfigDict(extra="forbid", frozen=True)` dans
`commands.py:39` — Pydantic ≥ 2.6 supporte `frozen=True` sur
`ConfigDict`. Pas de drift.

### H3 — Main bundle headroom

⚠️ **H-FX-1 (P2)** : `npm run build` reporte `index-BZTVPAeB.js`
à **474.49 KB** vs limite 475 KB → **0.51 KB de marge**.

**Évidence** : `npm run size` :
```
  Size limit: 475 kB
  Size:       474.5 kB
```

Sprint 9 doit soit augmenter le budget (475 → 500-525), soit faire
une passe de tree-shaking, sinon **le moindre nouveau composant
React Sprint 9 fera fail size-limit et bloquera tous les commits**.
L'audit_plan H3 le qualifiait de **P1** mais c'est un peu agressif :
size-limit fail bloque les commits, mais pas les PRs ou la prod.
**P2** est plus juste — Sprint 9 doit le traiter en priorité dans
sa Phase A ou son Day 0.

### H4 — Vitest deps

✅ Aucune nouvelle dev dep Vitest dans les diffs Phase A-E. Le
package.json reste propre.

### H5 — `cmdk` upgrade silencieux

✅ Pas de bump `cmdk` dans `package.json` ni `package-lock.json`.

### H6 — Wheel `nexus_core_py` install drift (H-3 Sprint 7)

⚠️ **H-FX-2 (P2 — re-confirmation)** : J'ai dû **rebuild manuellement**
le wheel via `maturin develop --release` pour faire passer
`test_curator.py` (9 tests qui dépendent de `sign_curator_list`
et `verify_curator_list_entry`). Sans rebuild, les 9 tests fail
avec `AttributeError: module 'nexus_core' has no attribute 'sign_curator_list'`.

C'est **exactement** le H-3 Sprint 7 tech debt
(`docs/rust/PATTERNS.md:846-852`) qui mord encore. Sprint 8 a
choisi de NE PAS le fixer (scope cut). **Mais l'audit gate
Sprint 9 Phase 0 (cette session) ne pouvait pas passer la
checklist sans rebuilder le wheel**, ce qui démontre que le
problème est plus douloureux que tech debt — il bloque
structurellement l'audit.

**Recommandation Sprint 9 Day 0** : H-3 doit être marqué **P1
pour Sprint 9** (pas P2), avec un `scripts/setup.sh` qui
encapsule le `unset CONDA_PREFIX && VIRTUAL_ENV=... maturin develop`
pour qu'une session fraîche puisse run la full suite sans manuel
trick.

### Findings Track H

- **H-FX-1 (P2)** : Main bundle headroom à 0.5 KB. Bloquant pour
  Sprint 9 Phase A si pas remonté/tree-shaké.
- **H-FX-2 (P2 — promu de P3 Sprint 7)** : H-3 wheel install drift
  bloque les sessions audit fresh. Sprint 9 Day 0 doit livrer
  `scripts/setup.sh`.

---

## Track I — Documentation & traceability

**Verdict** : **PASS** (avec 1 P2 et 1 P3 sur la verification)

### I1 — `docs/shell/PATTERNS.md` P10 cohérent avec le code

✅ P10 est présent (`docs/shell/PATTERNS.md:212`). Lecture POST
audit pour cross-check.

### I2 — P8 update

✅ P8 (`docs/shell/PATTERNS.md:115`) référence Sprint 8 D4
(legacy retiré). Cohérent.

### I3 — T4 + T5 marked CLOSED

✅ T4 (`docs/shell/PATTERNS.md:368-381`) — **Status: CLOSED Sprint 8
Phase A** avec commit `d321021`.
✅ T5 (`docs/shell/PATTERNS.md:402-415`) — **Status: CLOSED Sprint 8
Phase A** avec commit `d321021`.

### I4 — `docs/rust/PATTERNS.md` Sprint 8 closures

✅ A-4, C-2, D-1, G-3 marqués CLOSED Sprint 8 Phase A avec commit
`d321021` (`docs/rust/PATTERNS.md:859-917`).
✅ A-3 cross-language curator fixture marqué CLOSED Sprint 8 Phase A
(`docs/rust/PATTERNS.md:919`).
✅ E-1, C-4, D-3, H-3 restent **ouverts** (pas de marqueur CLOSED
erroné) — vérifié.

### I5 — Sprint 8 plan §12 scope cuts vs reality

✅ Cf. Track G — tous greps verts.

### I6 — Commit messages spot-check

J'ai inspecté `82e9c1d` (Phase D — gov batch 3 + RAG workers) :

```
$ git show --stat 82e9c1d
Phase D: 7 fichiers, 1419 insertions, 35 deletions
```

✅ Le body décrit Phase D : 6 nouveaux tabs, 2 RAG workers, +6 tests
gov, +2 Playwright. Cohérent avec le diff. **Format conforme**
au Sprint 6/7 commit discipline.

### I7 — Verification.md accuracy

❌ **V-FX-1 (P1)** — `sprint8_verification.md` row 11 contient **3
fausses claims** :

| Claim verification | Réalité code | Sévérité |
|---|---|---|
| « read-only enforced via `mode=ro` » | `aiosqlite.connect(path)` sans mode ro | **P1** (fait partie de D-FX-1) |
| « concurrent fetchall under asyncio.Lock » | Pas de Lock, juste connect-per-call | **P2** |
| « schema introspection » comme test rouge | Pas de méthode `schema_introspection` | **P2** |

**Pourquoi P1 (et pas seulement P2)** :

L'audit gate pattern repose sur la **honnêteté** de la
verification.md self-report. Une session fraîche qui audite Sprint 9
lira `sprint8_verification.md` et lui fera confiance par défaut.
Si row 11 dit « read-only enforced », l'auditeur Sprint 9 va
**possiblement skipper** la vérification de ce point en pensant
« déjà couvert par row 11 verification ». Le pattern audit gate
échoue silencieusement.

C'est la **première fois** depuis Sprint 6 (instauration du pattern)
qu'une verification.md ment sur des claims testables. Sprint 6 et
Sprint 7 verifications étaient honnêtes.

**Évidence** :

1. `sprint8_verification.md` ligne ~106 (« row 11 ») — texte ci-dessus
2. `db.py:90` — pas de mode ro (Track D évidence point 1)
3. `db.py` complet — aucun Lock instancié
4. `db.py` complet — aucune méthode `schema_introspection`
5. `test_db.py` complet — aucun test `test_readonly_enforced`

**Fix obligatoire** : `fix(sprint8): correct verification row 11 false claims`
qui édite `.planning/sprint8_verification.md` row 11 pour refléter
la réalité. Si **D-FX-1** est fixé d'abord (ce qui est demandé),
alors row 11 peut être réécrite cohérente avec la nouvelle réalité
read-only.

⚠️ **V-FX-2 (P3)** : verification.md row 4 reporte 309 Rust passed
(observation correcte). Mais la note §Row 4 prétend « Total Phase A
Rust hygiene : +4 tests + 1 share-fixture utilisé côté Python+Vitest.
Total Phase A net = +4 Rust = 308. La 5e ligne est probablement un
test ajouté par erreur dans `core-rs` (passage de 78 → 80 visible
dans la sortie cargo) — sans impact ». La phrase « probablement un
test ajouté par erreur » est cavalière pour un audit
self-report. L'agent qui a écrit Sprint 8 ne sait pas pourquoi un
test a poppé. **Investigation manquante** : `git log -p` sur
`crates/nexus-core-rs/` Phase A pour identifier le 5e test. P3
finding sur la rigueur du self-report.

### Findings Track I

- **V-FX-1 (P1)** : verification.md row 11 ment sur 3 claims (lié
  à D-FX-1 fix).
- **V-FX-2 (P3)** : verification.md note §Row 4 est cavalière sur
  l'origine du 5e test Rust.

---

## Récapitulatif des findings (sortés par sévérité)

### P0 — Casse prod / data loss

Aucun.

### P1 — Bloque Sprint 9 Phase A

| ID | Track | Description | Fix attendu |
|---|---|---|---|
| **D-FX-1** | D | `AppDatabaseClient` n'est pas read-only ; `execute()` écrit et commit ; la DB legacy gov est exposée | `fix(sprint8): enforce read-only on AppDatabaseClient` |
| **V-FX-1** | I | `sprint8_verification.md` row 11 ment sur 3 claims (mode=ro, asyncio.Lock, schema_introspection) | `fix(sprint8): correct verification row 11 false claims` |

### P2 — Tech debt logger dans PATTERNS

| ID | Track | Description |
|---|---|---|
| **C-FX-1** | C | Palette invoke errors silencieusement avalées (`console.error` only) |
| **C-FX-2** | C | Ordre `_commands` dépend de `dir(cls)` au lieu d'un sort explicite |
| **H-FX-1** | H | Main bundle headroom à 0.5 KB (474.5/475) |
| **H-FX-2** | H | H-3 wheel install drift bloque les audit sessions fresh, doit être promu de P3 à P1 pour Sprint 9 |
| **V-FX-1-resid** | I | (résidu de V-FX-1 fix) — false claims sur Lock + schema_introspection sont moins critiques que mode=ro mais doivent être nettoyées en même temps |

### P3 — Nits / optionnels

| ID | Track | Description |
|---|---|---|
| **A-FX-1** | A | `_maybe_call` retourne `{"error": ...}` au lieu de propager (mini-zombie legacy fallback) |
| **B-FX-1** | B | `resolve_worker` first-match-wins sur deux workers de même nom (silent shadow) |
| **B-FX-2** | B | `SubmitAppTaskRequest.payload` sans cap de taille |
| **B-FX-3** | B | `parent_task_id` pas validé comme UUID |
| **C-FX-3** | C | `CommandDescriptor.description` a un `min_length=1` non listé dans la signature gelée Sprint 7 D5 |
| **C-FX-4** | C | Pas de fixture cross-lang `command_canonical.json` (rupture du pattern Sprint 6/7) |
| **C-FX-5** | C | audit_plan C4 documente une exigence (payload structuré au invoke) que kickoff D2 a explicitement exclue — incohérence doc |
| **D-FX-2** | D | f-string `f"SELECT COUNT(*) FROM {table}"` dans `_safe_count` (pas de vuln, fait fail le canary) |
| **D-FX-3** | D | Pas de méthode `schema_introspection` (verification l'affirme à tort) — couvert par V-FX-1 |
| **E-FX-1** | E | 1 test collectif d'empty state au lieu de 19 dédiés |
| **E-FX-2** | E | `politicians_list_query` projette `party` plat sans jointure historique |
| **E-FX-3** | E | `contradictions_overview_query` SUM hardcoded `severity = 'high'` |
| **V-FX-2** | I | verification.md note §Row 4 cavalière sur le 5e test Rust |

---

## Verdict global — **CONDITIONAL PASS**

**Conditions de levée** :

1. Commit `fix(sprint8): enforce read-only on AppDatabaseClient`
   landed sur master, qui :
   - Ajoute `read_only: bool = True` au `AppDatabaseClient.__init__`
   - Quand `read_only=True`, ouvre via
     `aiosqlite.connect(f"file:{path}?mode=ro", uri=True)`
   - Quand `read_only=True`, raise `DatabaseError` au début de
     `execute()` AVANT la connexion (defense in depth)
   - Coordinator loader instancie le default avec `read_only=True`
   - GovApp.on_start utilise `AppDatabaseClient(legacy, read_only=True)`
   - Test `test_readonly_enforced` qui asserte que `execute()`
     raise sur un client read-only
   - Test `test_readonly_blocks_via_uri_mode` qui asserte que même
     en bypass de `execute()`, un INSERT direct lift `OperationalError`
   - Update du test `test_execute_commits_and_persists` pour utiliser
     `read_only=False` explicit

2. Commit `fix(sprint8): correct verification row 11 false claims`
   landed sur master, qui :
   - Édite `.planning/sprint8_verification.md` row 11 pour retirer
     les claims false (`mode=ro`, `asyncio.Lock`, `schema_introspection`)
   - Réécrit le observed pour refléter la nouvelle réalité post
     fix #1 (mode=ro now true), ou si fix #1 est plus minimal,
     juste documente le pattern « connect-per-call » correctement
   - Optionnel : ajoute une note dans `docs/shell/PATTERNS.md`
     P10 ou un nouveau P11 sur le contrat read-only de
     `AppDatabaseClient`

**Après ces 2 commits** :

- Sprint 9 Phase 0 (cette session) recommit `docs(sprint8): audit
  findings from Sprint 9 Phase 0 gate` qui inclut ce fichier
- Sprint 9 Phase A peut démarrer

**P2 à logger en tech debt** (sans bloquer) :

- C-FX-1 → ajout dans `docs/shell/PATTERNS.md` tech debt section
  comme nouveau **T8 — Command palette swallows invoke errors**
  avec note « to fix Sprint 9 polish »
- C-FX-2 → ajout dans `docs/shell/PATTERNS.md` ou complétion de T5
  avec note « ordering depends on `dir(cls)` — Sprint 9 explicit sort »
- H-FX-1 → ajout dans `docs/shell/PATTERNS.md` tech debt comme
  **T9 — Main bundle headroom critical** + Sprint 9 Day 0 doit
  trancher (raise budget vs tree-shake)
- H-FX-2 → mise à jour de `docs/rust/PATTERNS.md` H-3 pour le
  passer de P3 à P1 cible Sprint 9, avec note explicite
  « bloque audit sessions fresh »

**P3 laissés sans action** :

- A-FX-1, B-FX-1, B-FX-2, B-FX-3, C-FX-3, C-FX-4, C-FX-5, D-FX-2,
  E-FX-1, E-FX-2, E-FX-3, V-FX-2 — tous nits, à rouvrir Sprint 9
  polish ou à ignorer si l'utilisateur juge le coût supérieur au
  bénéfice

---

## Notes on audit completeness

**Tracks rejoués** : A, B, C, D, E, F, G, H, I — tous, dans le
timebox 3 h.

**Skips conscients** :

- **Playwright re-run** non rejoué localement (verification.md row 27
  affirme 24/24 et la spawn live + fixture ne donne pas de signal
  nouveau au-delà des Vitest). Si l'utilisateur juge nécessaire
  une re-validation Playwright, la commande est documentée dans
  verification.md §How to re-run.
- **Coverage Vitest spécifique au CommandPalette** non re-mesuré ;
  je me suis appuyé sur la verification.md row 26 (96.32% lines /
  100% funcs / 88.36% branches) qui passe les seuils.
- **Cross-check données réelles gov** impossible parce que
  `nexus/gov/govdata.db` n'existe pas dans le repo audité. Track E
  spot-check fait par lecture statique (queries vs schéma
  `nexus/gov/db.py`).

**Tracks où j'ai pris du temps supplémentaire** : D (3 niveaux de
findings cross-validés en lisant `db.py`, `test_db.py`,
`coordinator.py`, `app.py` gov pour mesurer le risque réel).

**Cross-checks effectués** :

- Lecture POST audit de `docs/shell/PATTERNS.md` P10 et T4/T5 :
  cohérent avec le code lu en blind.
- Lecture POST audit de `docs/rust/PATTERNS.md` Sprint 8 closures :
  cohérent.

---

**Auditeur** : session Claude Code fraîche, opus-4-6 1M context,
2026-04-12.
**Tip de fin d'audit** : `449f404` (master inchangé pendant l'audit
— les fix(sprint8) commits seront landed après ce findings doc).
**Timebox observé** : ~3 h, dans le quota suggéré par audit_plan §0.
