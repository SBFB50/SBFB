# Sprint 8 — Audit Plan (à jouer dans une session fraîche)

**Écrit** : 2026-04-11, en fin de Sprint 8, par l'agent qui vient
de livrer les 5 commits feat `d321021` → `9339bb6`.

**Pourquoi ce document** : `.planning/sprint8_verification.md` est
une checklist fail-fast **self-reportée** par l'agent qui a écrit
le code. Tous les 32 rows passent — mais c'est le même agent qui les
a écrites et qui confirme qu'elles passent. Ce n'est pas une
vérification, c'est une auto-attestation. Le pattern
`sprint_audit_gate.md` rend l'audit structurellement obligatoire
avant d'ouvrir Sprint 9 Phase A.

**Principe** : le fail-fast dit « le code compile et les tests
passent ». L'audit indépendant dit « le code fait ce qu'il prétend
faire, la surface testée correspond à la surface exécutée en prod,
et les décisions sont justifiées à la relecture ». Le Sprint 7 audit
gate (qui a produit `sprint7_audit_findings.md`) a démontré que ce
pattern attrape des blind spots invisibles à l'auteur — Sprint 8 est
la première itération sur un sprint **lourd en code utilisateur**
(19 tabs gov, ~10 000 LOC ajoutées) où l'audit aura le plus de
matière.

---

## 0. Mode d'emploi pour la session fraîche

**Avant de commencer**, l'auditeur (agent ou humain) doit :

1. `git log --oneline master ^2ed0955` — lire les 7 commits
   Sprint 8 (1 doc kickoff + 5 feat A..E + 1 doc verification + ce
   doc dans le même commit)
2. Lire dans cet ordre :
   - `MEMORY.md` + `nexus_grid_pivot.md` + `sprint_audit_gate.md` +
     `feedback_approach.md` (memory cross-session)
   - `docs/claude/README.md` (workflow source of truth)
   - `.planning/sprint8_kickoff.md` (kickoff + §4 D1..D5 gelées)
   - `.planning/sprint8_plan.md` §4–9 (phases A..F détaillées) et
     §10 (fail-fast 32 rows cible)
   - `.planning/sprint8_verification.md` (self-report 32 rows)
3. **Ne pas lire** `docs/shell/PATTERNS.md` §P10 ni les sections
   « Sprint 8 » de `docs/rust/PATTERNS.md` avant d'avoir formé un
   avis sur la policy — l'objectif est de **challenger** les choix,
   pas les ratifier. Lecture autorisée seulement APRÈS avoir écrit
   son verdict track-par-track.
4. **Ne pas lire** `.planning/sprint7_audit_findings.md` avant d'avoir
   formé son opinion sur les Sprint 7 tech debt items qui sont
   marqués CLOSED Sprint 8 Phase A — pour vérifier que le « CLOSED »
   est solide, pas qu'il l'aurait été par la simple foi
5. Tenir un journal `.planning/sprint8_audit_findings.md` au fur et
   à mesure. Format par finding :
   `{track, severity, what, evidence, fix}`
6. Sévérités : **P0** (casse prod / data loss / surface attaque),
   **P1** (bloque Sprint 9 Phase A), **P2** (tech debt explicite à
   logger dans `PATTERNS.md`), **P3** (nit, optionnel)

**Timebox suggéré** : 3 h. Audit indépendant, pas re-spec. Si un
track prend plus de 45 min, skipper et noter « timebox » ; la
session fraîche rapporte du signal en priorité sur du volume.

**Scope volumétrique à anticiper** : Sprint 8 est le sprint le plus
gros en surface (~10 000 lignes ajoutées sur 57 fichiers, vs ~1 800
LOC + 35 fichiers Sprint 7). L'auditeur doit **prioriser** : Tracks
B (submit_task), C (@nexus_command), D (AppContext.db) sont les
plus stratégiques parce qu'ils introduisent des surfaces neuves.
Track E (data fidelity gov) est gros mais 3 spot-checks suffisent.
Tracks A, F, G, H, I sont des relectures rapides.

**Format du delivrable final** : une section par track ci-dessous
dans `.planning/sprint8_audit_findings.md`, chacune avec son
verdict PASS / CONCERN / FAIL + la liste des findings. Puis un
**verdict global** (PASS / CONDITIONAL PASS / FAIL) avec les
conditions pour lever un CONDITIONAL. Les P0 + P1 doivent être
corrigés en commits `fix(sprint8): ...` atterissant sur master
**avant** le premier commit Sprint 9 Phase A.

---

## 1. Track A — Retrait du `legacy_descriptor` fallback

**Question centrale** : le fallback Sprint 6 D3 (un tab qui retourne
une dict invalide → coordinator wrappe en `{descriptor, legacy_descriptor: true}`)
est-il **vraiment** retiré sans qu'aucun zombie code path ne survive
quelque part ? Toute trace qui reviendrait par accident en Sprint 9
serait une régression silencieuse.

### A1 — Grep canary `_coerce_tab_view`

**Méthode** :
1. `grep -rn '_coerce_tab_view' packages/nexus-coordinator/` →
   doit retourner 0 match
2. Cross-check : ouvrir
   `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py`
   et vérifier que le code path principal est `TabView.model_validate(raw)`
   suivi d'une `HTTPException(422, detail=...)` sur ValidationError,
   PAS un try/except qui retombe sur un dict-style fallback
3. Lire les 6 nouveaux tests dans `tests/test_apps.py` : un test
   doit explicitement vérifier qu'un tab qui retourne une dict
   invalide reçoit un 422, pas un 200 avec `legacy_descriptor: true`

**Signal d'audit** :
- **P0** si un dict invalide retourné par un tab fait toujours
  passer la requête en 200
- **P1** si le code path 422 existe mais bypass-é par une autre
  branche
- **P3** si le grep `legacy_descriptor` matche encore des chaînes
  qui sont effectivement mortes (commentaires, docstrings, error
  messages historiques)

### A2 — Coordinator side : la route ne re-introduit pas un retry

**Méthode** :
1. Lire `apps.py::_handle_tab_descriptor` ou son équivalent : il ne
   doit pas catcher `ValidationError` puis essayer une voie de
   secours
2. Vérifier que `import legacy_descriptor` ou similaire n'existe
   nulle part

### A3 — Shell side : le client gère le 422 propre­ment

**Méthode** :
1. `web/src/api/coordinator.ts` : la fonction qui appelle
   `/app/{name}/tabs/{tab}/descriptor` doit traiter le 422 comme
   une vraie erreur (toast / banner), pas comme un cas attendu
2. `web/src/components/project/AppsTab.tsx` doit afficher un état
   d'erreur visible si le tab refuse de produire une TabView, pas
   un fallback `<pre>JSON.stringify(rawError)</pre>`

### A4 — Tests qui verrouillent le contrat

**Méthode** :
1. Compter dans `tests/test_apps.py` les `assert ... not in body`
   sur `legacy_descriptor` → doit y avoir au moins 2 (un sur tab
   valide, un sur tab invalide qui fail à 422)
2. Vérifier qu'au moins un test passe **un tab qui retourne une
   dict invalide** et asserte 422, pas seulement un tab qui retourne
   une `TabView` correcte

**Verdict track** : PASS / CONCERN / FAIL sur l'absence totale de
fallback path.

---

## 2. Track B — `AppContext.submit_task` contract end-to-end

**Question centrale** : le contrat gelé Sprint 7 D4 est-il implémenté
à la lettre, est-il câblé end-to-end (Python `submit_task` →
`/app/{name}/tasks/submit` → coordinator dispatcher → worker →
result), et le `ButtonBlock.task_submit` action React est-elle
réellement consommée par un test (Vitest ou Playwright) qui ne
mocke pas la couche transport ?

### B1 — Signature Python frozen vs réelle

**Méthode** :
1. Ouvrir `packages/nexus-sdk/src/nexus_sdk/app.py` et localiser
   `AppContext.submit_task`
2. Comparer **mot-pour-mot** avec la signature gelée Sprint 7 D4 :
   ```python
   async def submit_task(
       self,
       worker: str,
       payload: dict[str, Any],
       *,
       priority: int = 5,
       parent_task_id: str | None = None,
   ) -> str: ...
   ```
3. Tout argument supplémentaire, tout renommage, toute valeur par
   défaut différente est un finding (P1 si changement de contrat,
   P2 si extension non documentée)

### B2 — `resolve_worker` cross-app + ambiguous

**Méthode** :
1. Lire la doc string et l'implémentation de
   `NexusApp.resolve_worker(routing_key)`
2. Tracer les 3 chemins :
   - `routing_key = "<app>.<worker>"` → résolution cross-app
   - `routing_key = "<worker>"` (unambiguous dans l'app courante) →
     résolution local
   - `routing_key = "<worker>"` (ambigu : 2 apps ont un worker du
     même nom) → doit raise (R5 mitigation)
3. Vérifier le test `test_resolve_worker_cross_app` ET
   `test_resolve_worker_ambiguous_raises` dans
   `packages/nexus-sdk/tests/test_sdk.py`

**Signal d'audit** :
- P1 si l'ambiguité retourne silencieusement le « premier vu »
- P2 si la résolution n'est pas déterministe (e.g. dépend de l'ordre
  d'enregistrement, qui dépend de l'ordre du `dict`)

### B3 — Coordinator route + dispatcher integration

**Méthode** :
1. Lire `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py`
   pour la route `POST /app/{name}/tasks/submit`
2. Vérifier qu'elle :
   - Parse le body via Pydantic (`extra="forbid"` souhaitable)
   - Appelle `app.context.submit_task(...)` plutôt que de re-implémenter
     la résolution worker côté coordinator
   - Retourne `{task_id: str}` avec status 200 si OK, 404 si worker
     unknown, 422 si payload invalide
3. Tester le happy path via `test_apps.py::test_submit_app_task_happy_path`
4. Vérifier qu'un task soumis via cette route est effectivement
   visible dans la queue dispatcher : grep `dispatcher.submit_task`
   ou `tasks_doc.set_bytes` dans le call chain

### B4 — `ButtonBlock` React wiring

**Méthode** :
1. Ouvrir `web/src/components/app/tabview/blocks/ButtonBlock.tsx`
2. Vérifier que la branche `action.kind === "task_submit"` :
   - Lit le `TabAppContext` via `useContext(TabAppContext)`
   - Appelle `submitAppTask(coordinatorUrl, appName, body)` du client
     `coordinator.ts`
   - Affiche un état success/error visible (pas de `console.warn`,
     pas de toast invisible)
3. Vérifier la coverage Vitest sur `ButtonBlock.tsx` (le verification.md
   row 26 reporte 77.77% lines / 76.47% branches — c'est faible
   pour un composant aussi central) — relire pour comprendre les
   branches non-couvertes
4. Spot-check le Playwright spec `gov-rag-search.spec.ts` : il doit
   cliquer sur le bouton task_submit et vérifier qu'un appel HTTP
   réel atteint le coordinator (pas un mock)

**Signal d'audit** :
- P1 si `ButtonBlock` retombe en `console.warn` dans certains cas
- P2 si le coverage Vitest est faussement gonflé par des tests qui
  ne traversent pas la branche réelle
- P2 si le test Playwright couvre seulement le « tab affiche le
  bouton » sans vérifier l'effet de bord HTTP

### B5 — Erreur paths

**Méthode** :
1. Tester ce qui se passe quand `submit_task` est appelé avec :
   - `worker = "unknown.worker"` → doit raise / 404
   - `payload = {"trop_grand": "x" * 10_000_000}` → comportement ?
   - `priority = -1` → validation ? clamp ?
   - `parent_task_id = "not-a-uuid"` → validation ?
2. Vérifier que les erreurs sont user-visibles côté shell, pas
   silencieusement loggées

**Verdict track** : PASS / CONCERN / FAIL sur la fidélité du contrat
+ wiring end-to-end.

---

## 3. Track C — `@nexus_command` contract + palette integration

**Question centrale** : le `@nexus_command` decorator + le
`CommandDescriptor` Pydantic + le Zod mirror + la 4e palette group
forment-ils un contrat **stable et complet**, ou y a-t-il des trous
qui forceront un Sprint 9 fix-up ?

### C1 — `CommandDescriptor` frozen vs réel

**Méthode** :
1. Lire `packages/nexus-sdk/src/nexus_sdk/commands.py::CommandDescriptor`
2. Comparer mot-pour-mot avec la signature gelée Sprint 7 D5 :
   ```python
   class CommandDescriptor(BaseModel):
       model_config = ConfigDict(extra="forbid", frozen=True)
       schema_version: Literal[1] = 1
       name: str = Field(..., min_length=1, max_length=64)
       description: str = Field(..., max_length=280)
       icon: str = Field("sparkles", max_length=32)
       group: str = Field("Actions", max_length=32)
   ```
3. Toute dérive (champ ajouté, max changé, default différent) est
   un finding P1 (sortir du contrat frozen est un blocker Sprint 9
   audit)

### C2 — Zod mirror dans `coordinator.ts`

**Méthode** :
1. `web/src/api/coordinator.ts::CommandDescriptorSchema`
2. Cross-check chaque field, chaque max, chaque default
3. Vérifier que `.strict()` est posé (sinon une dérive Pydantic
   sans bump version passerait silencieusement)
4. Vérifier qu'un test Vitest dans `coordinator.test.ts` lit une
   fixture `command_canonical.json` partagée avec Python (pattern
   Sprint 6 cross-lang) — si absent, c'est un **P2** (R2 plan §13
   demandait ce pattern, mais le verification.md row 25 ne mentionne
   qu'une fixture curator, pas une command)

### C3 — Decorator metadata + registry collection

**Méthode** :
1. Lire `decorators.py::nexus_command` — doit attacher
   `__nexus_command__: CommandDescriptor` sur la méthode
2. Lire `registry.py::NexusApp.commands()` — doit walk les
   méthodes de la classe et collect les `__nexus_command__`
3. Vérifier que l'ordre de retour est **déterministe** (sorted
   par name asc) — sinon le test
   `test_list_app_commands_ordered` ne peut pas être stable
4. Vérifier qu'un command ne peut PAS shadow un command d'une autre
   méthode : 2 méthodes avec le même `name` sur la même classe doit
   raise au moment de la classe definition (decorator-time check)

**Signal d'audit** :
- P1 si l'ordre n'est pas déterministe
- P2 si le shadow est silencieux (le second wins)

### C4 — Coordinator routes `commands` + `invoke`

**Méthode** :
1. `GET /app/{name}/commands` → retourne `list[CommandDescriptor]`
2. `POST /app/{name}/commands/{cmd}/invoke` → appelle la méthode
   décorée
3. Vérifier que `invoke` :
   - Valide que `cmd` existe sur l'app, sinon 404
   - Passe le body de la requête comme `payload` à la méthode
   - Retourne le résultat de la méthode (probablement un dict
     sérialisable JSON)
4. Tester l'erreur path : appel d'un command absent → 404,
   payload invalide → 422

### C5 — `CommandPalette` React 4e groupe

**Méthode** :
1. `web/src/components/command-palette/CommandPalette.tsx`
2. Vérifier que le 4e groupe « Apps » :
   - Fetch via React Query `listAppCommands(coordinatorUrl, appName)`
     pour CHAQUE app enrôlée du store
   - `staleTime: 15_000`, `refetchInterval: 30_000` (R7 mitigation
     pour ne pas hammer le coord)
   - Affiche un état vide « Pas de commands disponibles » plutôt
     qu'une absence de groupe (UX consistency)
   - Sur click : appelle `invokeAppCommand()` ET / OU navigue vers
     `extractNavigationPath(command)` si le command porte un
     metadata `target_tab`
3. Lire `extractNavigationPath.ts` — comment fait-il pour mapper
   un command name vers un URL `app/{appName}/tab/{tabName}` ?
   Le mapping est-il dans le command metadata ou hardcoded gov-side ?

**Signal d'audit** :
- P1 si le 4e groupe ne s'affiche pas avec 0 apps enrôlée (crash
  React au lieu de empty state)
- P2 si le polling cadence n'est pas configuré comme R7 demandé
- P2 si `extractNavigationPath` est hardcoded gov-only et ne
  scaling pas pour Sprint 9 cold-case / forensics

### C6 — Vitest CommandPalette tests

**Méthode** :
1. Compter les tests Vitest dans `CommandPalette.test.tsx`
2. Vérifier qu'au moins un test couvre :
   - Navigation group (render)
   - Projects group (render)
   - Actions group (render)
   - Apps group avec 0 apps (empty state)
   - Apps group avec 1 app, N commands (render + click)
   - Apps group avec invoke error (toast / banner visible)

**Verdict track** : PASS / CONCERN / FAIL sur la complétude du contrat
`@nexus_command`.

---

## 4. Track D — `AppContext.db` read boundary + SQL injection surface

**Question centrale** : `AppDatabaseClient` est-il vraiment read-only,
le path resolution est-il safe (pas de path traversal), et la surface
SQL est-elle paramétrée partout (pas un seul `f"SELECT ... {user_input}"`
dans le code gov) ?

### D1 — Read-only enforce

**Méthode** :
1. Lire `packages/nexus-sdk/src/nexus_sdk/db.py::AppDatabaseClient`
2. Vérifier que la connexion aiosqlite est ouverte avec :
   - URI mode (`file:{path}?mode=ro`)
   - OU `aiosqlite.connect(path, ..., uri=True, mode='ro')`
3. Tester explicitement (test ou audit-time REPL) qu'un
   `await client.execute("INSERT INTO ...")` raise
   `sqlite3.OperationalError: attempt to write a readonly database`
4. Lire le test `test_db.py::test_readonly_enforced` (verification.md
   row 11 le mentionne) pour vérifier que ce n'est pas un mock

**Signal d'audit** :
- **P0** si une instance `AppDatabaseClient` peut écrire dans la DB
  legacy. La promesse Sprint 8 D2 est read-only ; un write
  accidentel via un `executescript` raté est un risque réel parce
  que la DB legacy contient des données précieuses (4 ans de scraping)
- P2 si le mode read-only est obtenu par convention (« on n'expose
  pas execute() ») plutôt que par enforcement SQLite-level

### D2 — Path resolution safety

**Méthode** :
1. Lire `coordinator.py` ou `paths.py` pour comment le path de la
   DB legacy est résolu
2. Vérifier que c'est un path absolu calculé depuis le `__file__`
   du coordinator + `nexus/gov/govdata.db`, **pas** depuis un
   user-controlled string
3. Vérifier qu'un app malicieux ne peut PAS demander une DB
   arbitraire via, par exemple, `AppContext.db_path = "/etc/passwd"`
4. Tester ce qui arrive si la DB legacy n'existe pas (R1 mitigation) :
   doit retourner empty state, pas crasher

### D3 — SQL injection surface

**Méthode** :
1. `grep -rn "f\"SELECT\|f\"INSERT\|f\"UPDATE\|f\"DELETE" packages/nexus-app-gov/`
   → doit retourner 0 match
2. `grep -rn '%s.*%' packages/nexus-app-gov/src/nexus_app_gov/queries.py`
   → doit retourner 0 match (pas de string formatting sur SQL)
3. Lire `queries.py` exhaustivement (~874 LOC) — chaque appel
   `await db.fetchall(sql, params)` doit avoir `sql` literal et
   `params` un tuple/dict
4. Spot-check 5 fonctions au hasard avec un payload qui contient
   `'; DROP TABLE politicians; --` → l'erreur attendue est un row
   absent, PAS un crash ni une mutation

**Signal d'audit** :
- **P0** si une fonction `fetch_*` accepte du SQL user-input
- **P1** si une fonction utilise du string formatting sur des
  identifiers (table name, column name) qui viennent indirectement
  du user

### D4 — `asyncio.Lock` concurrency

**Méthode** :
1. Lire l'implémentation du lock dans `db.py`
2. Vérifier que `fetchall`, `fetchone`, `execute` (s'il existe)
   prennent tous le même lock
3. Tester `test_concurrent_fetchall` (verification.md row 11 le
   mentionne) → doit valider que 2 calls en parallèle sont
   sérialisés, pas crashed
4. Mesurer la latence : un lock global per-`AppDatabaseClient`
   sérialise les queries, ce qui peut être un bottleneck si une
   tab fait 10 queries en parallèle. Est-ce acceptable pour
   Sprint 8 ? Pour Sprint 9 ? **P2** si oui

### D5 — Schema introspection scope

**Méthode** :
1. `db.py` expose-t-il `schema_introspection()` ou similaire ?
2. Si oui, vérifier qu'il ne fait que `SELECT name FROM sqlite_master`
   et n'ouvre pas une porte à `pragma_*` ou autre
3. Sinon, c'est OK — mais l'absence est notable parce que le SDK
   pourrait vouloir l'exposer Sprint 9 pour les apps qui veulent
   se connecter à des DBs inconnues

**Verdict track** : PASS / CONCERN / FAIL sur le boundary read-only
+ SQL injection surface.

---

## 5. Track E — Fidélité des données gov vs `nexus/gov/api.py` legacy

**Question centrale** : Les 19 tabs gov rendent-ils **vraiment** les
mêmes données que les 45 endpoints legacy `nexus/gov/api.py` ? Sprint 8
a porté la lecture via `AppContext.db`, mais la traduction SQL → TabView
peut subtilement diverger des endpoints existants.

### E1 — Spot-check 3 tabs (timebox 30 min)

**Méthode** :
1. Choisir 3 tabs au hasard parmi les 19 (suggestion : Dashboard,
   Politiciens, Contradictions — un de chaque batch A/B/C)
2. Pour chacun :
   - Lire la fonction `fetch_*` dans `queries.py`
   - Comparer avec l'endpoint legacy `nexus/gov/api.py` correspondant
     (e.g. `/api/dashboard`, `/api/politicians`, `/api/contradictions`)
   - Vérifier que les colonnes SELECTées sont les mêmes, que les
     filtres `WHERE` correspondent, que les agrégations `GROUP BY`
     sont identiques
3. Si la DB legacy a des données réelles (le repo a peut-être un
   `nexus/gov/govdata.db` populated), exécuter les deux et comparer
   les rows ligne-à-ligne
4. Si la DB est vide, vérifier que les deux retournent un empty
   state structuré identique (pas de `None` vs `[]` divergence)

**Signal d'audit** :
- **P1** si une divergence de colonne change le rendu UX (e.g.
  l'endpoint legacy retournait `created_at` ISO mais la nouvelle
  query retourne `created_at` epoch)
- P2 si une agrégation diffère (count vs sum)
- P3 nit si un ORDER BY est différent

### E2 — Empty state coverage

**Méthode** :
1. Vérifier que les 19 tabs ont chacun un test app-gov qui couvre
   l'empty state (DB vide)
2. Compter dans `tests/test_gov_app.py` le nombre de
   `def test_*_empty_state` ou équivalent
3. Cible : 19 tests d'empty state. Si moins, lister les tabs
   non-couverts comme P3

### E3 — Pas de re-scrape déclenché par le rendu

**Méthode** :
1. `grep -rn 'subprocess\|requests.get\|httpx' packages/nexus-app-gov/`
   → doit retourner 0 match (sauf imports defensifs)
2. Vérifier qu'aucun tab handler n'appelle `submit_task` au
   moment du render — `submit_task` doit être déclenché par un
   click button explicite (pattern Recherche / Question)

**Signal d'audit** :
- **P0** si un tab déclenche un scrape au render (UX killer +
  rate limit risk)

**Verdict track** : PASS / CONCERN / FAIL sur la fidélité des données.

---

## 6. Track F — Command palette UX across states

**Question centrale** : la palette se comporte-t-elle correctement
dans **tous** les états, ou y a-t-il un crash dans un état tordu
(0 apps, daemon offline, invoke retourne 500, etc.) ?

### F1 — État vide (0 apps enrôlées)

**Méthode** :
1. Tester (Vitest ou Playwright) la palette quand le store
   `projectStore` n'a aucune app enrôlée
2. Le 4e groupe « Apps » doit s'afficher avec « Pas de commands »
   ou être absent (pas un crash ni un loading spinner infini)

### F2 — État 1 app, N commands

**Méthode** :
1. Tester avec gov enrôlé (4 commands déclarés Phase E)
2. Le groupe doit lister les 4 commands triés par name asc
3. Chaque command doit être cliquable
4. Click → soit navigation vers `/app/gov/tab/{target_tab}` soit
   `invokeAppCommand` HTTP call

### F3 — État erreur invoke

**Méthode** :
1. Mock le coordinator pour retourner 500 sur `invoke`
2. Click sur un command → l'erreur doit être visible (toast,
   inline error, ou modal)
3. La palette ne doit pas freeze ni crasher

### F4 — État daemon offline

**Méthode** :
1. La palette n'utilise pas le daemon (elle utilise le coordinator),
   donc daemon offline ne doit pas l'affecter
2. Mais : si le coordinator est lui-même offline, la palette doit
   afficher un état dégradé ou skipper le 4e groupe sans crash

### F5 — Deep-link navigation

**Méthode** :
1. `extractNavigationPath` est-il testé ?
2. Le test `gov-commands-flow.spec.ts` doit cliquer sur un command
   et vérifier que la navigation atterrit sur la bonne tab
3. Vérifier les deep-links Phase E `gov-final-polish.spec.ts`
   couvrent au moins 2 tabs distincts

**Verdict track** : PASS / CONCERN / FAIL sur la couverture UX.

---

## 7. Track G — Scope cuts enforcement (4 infra primitives reportées)

**Question centrale** : Sprint 8 a explicitement **scope-cut** 4
primitives infra (storage, events, file upload, migration runner) +
6 Sprint 7 tech debts (E-1, C-4, D-3, H-3, F-3, G-1). L'auditeur
vérifie que **rien** n'a fuité.

### G1 — Greps storage / events / upload / migration

**Méthode** :
1. `grep -rn 'AppContext\.storage\|AppContext\.events\|ctx\.storage\|ctx\.events' packages/`
   → doit retourner 0 match
2. `grep -rn 'file_upload\|FileUpload\|upload_file' packages/nexus-coordinator/ packages/nexus-sdk/`
   → 0 match attendu
3. `grep -rn 'migration_runner\|MigrationRunner\|alembic\|migrate_db' packages/nexus-sdk/ packages/nexus-coordinator/`
   → 0 match attendu (le coordinator utilise déjà aiosqlite mais
   pas de runner d'app-side)

**Signal d'audit** :
- **P1** si une de ces 4 primitives a fuité dans le code Sprint 8

### G2 — Sprint 7 tech debts ouverts toujours ouverts

**Méthode** :
1. **E-1** : `crates/nexus-core-rs/src/discovery.rs` —
   `DEFAULT_PROBE_TIMEOUT` doit toujours être `Duration::from_secs(2)`
   (pas changé Sprint 8)
2. **C-4** : `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` —
   `process_announcement_bytes` doit toujours être appelé
   séquentiellement (pas de `tokio::sync::Semaphore` ajouté)
3. **D-3** : `iroh_runtime.rs::CuratorRuntime::subscribe` —
   l'ordre `attention.insert` puis `persist_subscriptions()?` doit
   être inchangé (verification.md le confirme par grep)
4. **H-3** : pas de `scripts/setup.sh` ajouté, le wheel
   `nexus_core_py` reste un editable install fragile
5. **F-3** : `web/src/components/ui/card.tsx` — `CardTitle` doit
   toujours être un `<div>` (pas de `as=` prop ajoutée)
6. **G-1** : `packages/nexus-coordinator/src/nexus_coordinator/api/daemon.py`
   — `_forward` doit toujours créer
   `async with httpx.AsyncClient(timeout=timeout)` par call (pas
   de pooling)

Si l'un de ces items a été **partiellement traité** (e.g. un
commentaire ajouté mais pas de fix), c'est une fuite de scope cut
qui mérite un finding P2 (« Phase A devait être hygiène stricte,
pas grattage opportuniste »).

### G3 — Scope cut « pas de mutations »

**Méthode** :
1. `grep -rn '@nexus_route.*POST\|@nexus_route.*PUT\|@nexus_route.*DELETE' packages/nexus-app-gov/`
   → doit retourner 0 match (Sprint 8 D6 : gov reste READ-ONLY
   via `ctx.db`)
2. Vérifier qu'aucun tab handler n'écrit dans la DB legacy

### G4 — Scope cut « pas de Reseau / Leaflet »

**Méthode** :
1. `grep -rn 'leaflet\|react-leaflet\|reseau\|graph_render' web/`
   → 0 match
2. Pas de nouvelle dep frontend pour graphes / cartes

### G5 — Scope cut « pas d'auth » sur submit_task / invoke

**Méthode** :
1. Lire les routes `POST /app/{name}/tasks/submit` et
   `POST /app/{name}/commands/{cmd}/invoke`
2. Vérifier qu'elles n'ont **pas** de dependency d'auth (Bearer
   token, cookie session, API key) — c'est intentionnel (loopback
   trust, même modèle que `/tasks/submit` Sprint 4)
3. Documenter cette absence dans `PATTERNS.md` Sprint 8 update si
   pas déjà fait

**Verdict track** : PASS / CONCERN / FAIL sur la pureté des scope
cuts.

---

## 8. Track H — Cross-dependency hygiene + bundle headroom

**Question centrale** : Sprint 8 a-t-il introduit des deps qui
pourraient bumper bizarrement, et le main bundle a-t-il encore
de la marge avant de fail size-limit ?

### H1 — `aiosqlite` version

**Méthode** :
1. `grep aiosqlite packages/nexus-sdk/pyproject.toml packages/nexus-coordinator/pyproject.toml`
2. La version doit être pinned ou caret-ranged
3. Vérifier qu'il n'y a pas 2 versions différentes installées
   (`uv lock --check` ou similaire)

### H2 — Pydantic / FastAPI version drift

**Méthode** :
1. Sprint 8 introduit `CommandDescriptor` avec `model_config =
   ConfigDict(extra="forbid", frozen=True)` — vérifier que la version
   de Pydantic supporte `frozen=True` sur le `ConfigDict` (≥ 2.6)
2. `pyproject.toml` doit être cohérent

### H3 — main bundle headroom critical

**Méthode** :
1. Re-run `npm run size` et lire la valeur de `main`
2. Verification.md row 24 reporte 474.5 / 475 = **0.5 KB headroom**
3. Sprint 9 doit soit augmenter le budget à 500-525 KB, soit faire
   une passe de tree-shaking. Lister comme **P1** parce que le
   moindre nouveau composant React Sprint 9 fera fail size-limit
   et bloquera les commits

### H4 — Vitest deps

**Méthode** :
1. Sprint 8 a ajouté des tests pour `CommandPalette`, `coordinator.test.ts`,
   `daemon.test.ts cross-lang`, `TabViewRenderer task_submit`
2. Vérifier qu'aucune nouvelle dev dep Vitest n'est arrivée par
   inadvertance (e.g. `vitest-axe`, `vitest-environment-jsdom-something`)
3. `web/package.json` doit être propre

### H5 — `@base-ui/react` ou `cmdk` upgrade silencieux

**Méthode** :
1. La 4e palette group réutilise `cmdk` (vendored shadcn)
2. Vérifier que `cmdk` n'a pas été bumpé silencieusement Sprint 8
3. `web/package-lock.json` ou `package.json` lock check

**Verdict track** : PASS / CONCERN / FAIL sur la hygiène + bundle.

---

## 9. Track I — Documentation & traceability

**Question centrale** : `PATTERNS.md` reflète-t-il le code Sprint 8 ?
P10 (command palette) est-il décrit avec assez de précision pour
qu'un futur sprint puisse cross-checker ? Les T4/T5 sont-ils
réellement marqués CLOSED ?

### I1 — `docs/shell/PATTERNS.md` P10 cohérent avec le code

**Méthode** :
1. Lire P10 (ou son équivalent Sprint 8) APRÈS avoir lu
   `CommandPalette.tsx` et `CommandDescriptor`
2. Vérifier que P10 décrit :
   - Le contrat `@nexus_command` (signature gelée)
   - Le polling cadence React Query (R7 mitigation)
   - L'extension cross-app via `extractNavigationPath`
   - L'absence d'auth (loopback trust)
3. Toute claim non-vraie est un finding

### I2 — `docs/shell/PATTERNS.md` P8 update

**Méthode** :
1. P8 (TabView is the only contract) doit avoir une note Sprint 8 :
   « Le fallback `legacy_descriptor` est retiré commit `d321021`
   Sprint 8 Phase A. La validation est `model_validate` direct,
   422 sur ValidationError »

### I3 — T4 (task_submit) + T5 (@nexus_command) marked CLOSED

**Méthode** :
1. `docs/shell/PATTERNS.md` section tech debt
2. T4 doit avoir « **Status: CLOSED Sprint 8 Phase A** » avec le
   SHA `d321021`
3. T5 idem
4. Si encore en `Status: signature frozen Sprint 7`, c'est un finding
   P2 (oubli de mise à jour Phase F)

### I4 — `docs/rust/PATTERNS.md` Sprint 8 closures

**Méthode** :
1. La section Sprint 7 tech debt doit lister les 4 P2 hygiène fixes
   en CLOSED Sprint 8 Phase A : A-4, C-2, D-1, G-3 — chacun avec
   un SHA pointant vers `d321021`
2. Les 4 items E-1, C-4, D-3, H-3 doivent rester ouverts (pas
   marked CLOSED par erreur)

### I5 — Sprint 8 plan §12 scope cuts vs reality

**Méthode** :
1. Pour chaque item du plan §12 scope cut, grep pour vérifier 0
   touche Sprint 8 (déjà partiellement couvert Track G mais
   re-vérifier ici comme cross-check)

### I6 — Commit messages spot-check (1 commit)

**Méthode** :
1. Choisir un commit Sprint 8 au hasard (suggestion : Phase B
   `6efda53`)
2. Vérifier que le body contient :
   - Files touched + rationale
   - Delta de tests cumulé
   - Scope cuts respectés
3. Comparer le `git show --stat` au body — doit matcher

**Verdict track** : PASS / CONCERN / FAIL sur la doc traceability.

---

## 10. Verdict global attendu

Trois scénarios possibles quand la session fraîche finit cet audit :

**PASS** : aucun finding P0 ni P1. Les P2/P3 vont dans
`docs/shell/PATTERNS.md` et `docs/rust/PATTERNS.md` tech debt
sections. Sprint 9 Phase A peut démarrer direct.

**CONDITIONAL PASS** : 1 ou 2 findings P1 clairement fixables en
commits `fix(sprint8): ...` dédiés. L'auditeur liste les commits
nécessaires + les critères de lève de condition. Sprint 9 Phase A
ne démarre QU'après les fix + une session de verify rapide. **Cas
le plus probable** vu le volume Sprint 8 (~10 000 LOC sur 57
fichiers — la surface d'attaque audit est large).

**FAIL** : ≥ 1 finding P0, ou ≥ 3 findings P1. L'audit demande
une re-conception partielle. Improbable mais possible si Track D
(SQL injection) ou Track E (data fidelity) trouve une régression
sérieuse.

---

## 11. Out of scope pour l'audit Sprint 8

L'auditeur ne doit PAS challenger :

- **Les D1..D5 gelées Sprint 8 kickoff §4** : `submit_task` Option B,
  `@nexus_command` decorator, `AppContext.db` read-only, retrait
  legacy_descriptor, gov tabs READ-ONLY via `ctx.db`
- **Les 4 primitives infra reportées Sprint 9** : `storage`, `events`,
  file upload, migration runner — leur absence est un scope cut
  intentionnel, pas un finding
- **Les 6 Sprint 7 tech debts encore ouverts** : E-1, C-4, D-3, H-3,
  F-3, G-1 — toutes documentées comme « à traiter Sprint 9 », ne
  pas les rebattre comme P1
- **Le pattern audit gate lui-même** : si l'audit pense que le pattern
  est mal spec, le noter pour une refactor de `docs/claude/README.md`
  Sprint 9, pas comme un finding Sprint 8
- **Les choix d'archi P2P Sprint 7** : les curator lists, le
  daemon, le proxy /daemon — Sprint 8 ne les a pas touché
- **Le scope `nexus-app-coldcase` / `nexus-app-forensics`** : ces
  apps existent dans le repo mais Sprint 8 ne les touche pas. Leur
  état actuel est hors scope audit Sprint 8.

Si l'audit a une raison technique NOUVELLE d'invalider une décision
Day 0, il doit la logger comme **« décision à rouvrir en Sprint 9
Day 0 »** et ne PAS bloquer Sprint 8.

---

## 12. Livrable final attendu

`.planning/sprint8_audit_findings.md` avec :

1. Une section par track (A..I) — verdict + findings
2. Un verdict global (PASS / CONDITIONAL PASS / FAIL)
3. Si CONDITIONAL PASS : liste des commits fix attendus, chacun
   avec son critère de lève
4. Une liste des P2 à logger en tech debt dans `docs/shell/PATTERNS.md`
   et `docs/rust/PATTERNS.md`
5. Une liste des P3 laissés sans action
6. Signature : « audité par session {id}, timebox observée {h}h »

**Sans ce fichier, Sprint 9 Phase A ne peut pas démarrer.** C'est
le point non-négociable de `sprint_audit_gate.md`.

---

## 13. Annexe — checklist de démarrage rapide auditeur

```
[ ] Lire memory/MEMORY.md + nexus_grid_pivot.md + sprint_audit_gate.md
[ ] Lire docs/claude/README.md
[ ] git log --oneline master ^2ed0955 (7 commits Sprint 8)
[ ] Lire .planning/sprint8_kickoff.md (D1..D5 gelées)
[ ] Lire .planning/sprint8_plan.md §4-9 + §10 fail-fast
[ ] Lire .planning/sprint8_verification.md (self-report)
[ ] OUVRIR ce fichier .planning/sprint8_audit_plan.md (la feuille
    de route)
[ ] Créer .planning/sprint8_audit_findings.md vide avec stub
[ ] Track A → Track I dans l'ordre, ~20 min chacun en moyenne
[ ] Track B + C + D sont les plus stratégiques — accepter d'y
    passer 30-40 min chacun
[ ] Verdict global + liste commits fix si CONDITIONAL
[ ] Si P0/P1 : commiter les fix(sprint8): ... AVANT le findings
[ ] Commiter le findings doc en docs(sprint8): audit findings
    from Sprint 9 Phase 0 gate
[ ] Retourner le verdict à l'utilisateur
```

Bonne chance.
