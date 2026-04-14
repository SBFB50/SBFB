# Sprint 9 — Plan d'exécution détaillé

**Écrit** : 2026-04-12, juste après `sprint9_kickoff.md`,
avant le premier commit feat du sprint.

**But** : donner à l'agent exécuteur (souvent la même session
qui écrit ce doc) une feuille de route ligne-par-ligne pour
chaque phase A..F, avec fichiers à toucher, tests à écrire,
critères d'acceptation et commit cible.

---

## 1. État vérifié à l'entrée

### 1.1 Tip master

**HEAD d'entrée Sprint 9** : `c50157d`

Commit stack Sprint 8 Phase 0 audit gate (chronologique) :

```
c50157d docs(sprint8): audit findings from Sprint 9 Phase 0 gate
a8b4d50 docs(sprint8): tech debt T11/T12 + H-3 promotion from Sprint 9 audit gate
da69a8b fix(sprint8): correct verification.md row 11 false claims
14c199f fix(sprint8): enforce read-only on AppDatabaseClient
449f404 docs(sprint8): verification + audit plan for Sprint 9
```

### 1.2 Compteurs (source : `sprint8_verification.md` + fix
post-gate)

| Suite | Observé tip `c50157d` | Commande de re-vérif |
|---|---|---|
| `cargo fmt --all --check` | clean | `cargo fmt --all --check` |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | clean | idem |
| `cargo test --workspace --locked` | **309 passed** (80 core-rs + 28 daemon bin + 6 daemon e2e + 64 daemon-core + 11 worker bin + 10 worker e2e + 105 worker-core + 5 doctests) | idem |
| `pytest packages/nexus-sdk/tests/` | **71 passed** (40 baseline + 13 commands + 11 db **post-fix D-FX-1** + 7 sdk extensions) | idem |
| `pytest packages/nexus-coordinator/tests/` | **63 passed + 1 skipped** | idem |
| `pytest packages/nexus-app-gov/tests/` | **30 passed** | idem |
| `npm run test:unit` | **142 passed** / 9 files | idem |
| `npx playwright test` | **24 passed** / 18 files | idem |
| `npm run size` | **4/4 green** : main 474.5 / 475, vendor-react 189.6 / 210, vendor-ui 31.5 / 50, css 94.3 / 100 | idem |
| `tsc --noEmit` | 0 error | idem |
| `eslint` | 0 error, **5 T1 warnings** (shadcn fast-refresh, déjà trackés) | idem |
| `ruff format --check && ruff check packages/ examples/` | clean | idem |
| `scan-en-strings.sh` | clean | `bash web/scripts/scan-en-strings.sh` |
| `npm audit --audit-level=high` | 0 | idem |

**Note importante** : la row 11 de `sprint8_verification.md`
(SDK `test_db.py`) a été réécrite par `da69a8b` pour refléter
les **11 tests passants post-D-FX-1 fix** (8 Sprint 8 originaux
+ 3 nouveaux read-only). Le total SDK est donc **71** et non
**68** comme le rapportait la version originale du
`sprint8_verification.md`.

### 1.3 Phase A doit re-mesurer ces compteurs

Le premier acte de la Phase A Sprint 9 est de rouler
`./scripts/verify.sh` (ajouté en Phase A même) puis de
re-capturer les compteurs dans `sprint9_verification.md`
§Métriques « avant → après ». Si un compteur dérive du tableau
1.2 ci-dessus sur un tip qui n'a pas encore commit Phase A (par
ex. une flakyness test ou un clippy warning qui apparaît sur
une nouvelle version de rust), c'est un signal rouge à résoudre
AVANT d'écrire la moindre ligne Phase A.

---

## 2. Décisions Day 0 (gelées — rappel synthétique)

Rappel court des 6 décisions Day 0 de `sprint9_kickoff.md` §4.
Pour les détails (rationale, alternatives rejetées), voir le
kickoff. Ce résumé sert à ce que l'agent exécuteur n'ait pas
à switcher de fichier pendant l'écriture de code.

- **D1 — AppContext.storage** : JSON file `storage.json` +
  atomic rename + coalesce flush 500 ms + `asyncio.Lock` per
  app + flush-on-shutdown lifespan + typed namespaces via
  pydantic-settings-like accessor + per-app/per-project scope
- **D2 — AppContext.events** : wrapper
  `anyio.create_memory_object_stream[EventEnvelope](1024)` +
  `fnmatch` glob + `async with subscribe(pattern) as stream`
  context manager + overflow `drop_oldest` default +
  envelope `{topic, payload, timestamp, trace_id}` + per-app
  in-process only
- **D3 — File upload + CAS + TabView v2** : endpoint multipart
  `max_part_size=50 MB` + chunked read + SHA256 incrémental +
  CAS `<sha256[:2]>/<sha256>` + manifest JSON adjacent + dédup
  pre-write + magic bytes validation post-write (5 whitelisted)
  + allowlist per app via `@nexus_app_files` + bump TabView
  `schema_version: Literal[1,2]` discriminated union + Zod
  mirror + `extra="forbid"` préservé + cross-lang fixture v2
  + `<FileUploadBlock>` drag-drop natif avec progress SSE
  (consommateur D2 events)
- **D4 — Migration runner** : scan `app_root/migrations/
  NNN_<slug>.sql` lexico + `BEGIN IMMEDIATE` per migration +
  SHA256 tamper detection + table `_nexus_migrations(version,
  slug, sha256, applied_at)` + CLI `migrate --plan/--apply` +
  opt-in via `manifest.migrations_dir` + pas de `repair` CLI
  + forward-only
- **D5 — `scripts/setup.sh` + `scripts/verify.sh` + git hook
  opt-in + doc README** (H-3 CLOSED en Phase A)
- **D6 — `createBrowserRouter` + `lazy` propriété + `manualChunks`
  avec guards + 8 budgets recalibrés + retrait des chunks morts
  `vendor-graph`/`vendor-charts`/`vendor-map`** (T9 CLOSED en
  Phase A)

---

## 3. Research consulté

Recherche documentaire faite 2026-04-12 avant ce plan, en
parallèle via 3 agents général-purpose qui ont combiné
recherche web (GitHub, docs projets) et fetches Context7 MCP
(`mcp__context7__resolve-library-id` + `query-docs`).

### 3.1 Pour D1 (AppContext.storage)

Projets consultés :
- **diskcache 5.6.3** (https://github.com/grantjenks/python-diskcache)
  — KV SQLite-backed atomic. Pattern « connect-per-call » ou
  cache connection, évidences sur la primitive atomic write
- **TinyDB 4.8.2** (https://github.com/msiemens/tinydb) —
  **`CachingMiddleware` est la référence canonique pour le
  write coalescing** (flush N writes puis persist). Pattern
  repris pour la flush 500 ms
- **sqlitedict 2.1.0** (https://github.com/RaRe-Technologies/sqlitedict)
  — dict Python → SQLite, pickle-based (**à ne pas imiter**,
  pickle n'est pas portable)
- **shelve** stdlib — dbm-backed, pickle only, à éviter
- **pydantic-settings 2.x** (https://github.com/pydantic/pydantic-settings)
  — Context7 query `/pydantic/pydantic-settings` a confirmé
  `JsonConfigSettingsSource` qui charge un JSON → `BaseSettings`
  typed. **Pattern à reprendre** pour `ctx.storage.namespace(
  key, Schema)` : instancier un `BaseSettings` sous-class par
  namespace avec `json_file=<path>`, le wrapper `get`/`set`
  utilise `model_validate()` / `model_dump()`

Patterns retenus :
- **P1.a** `tmpfile + os.replace` : POSIX + Windows atomic
  rename (même volume/inode). `f.flush(); os.fsync(f.fileno())`
  optionnel pour crash-consistency forte — on le skippe car
  l'état storage est « soft state » (filter UI), crash tolerable
- **P1.b** Coalescing via `asyncio.call_later(0.5, flush)` —
  un seul timer par app, cancel + reschedule à chaque set,
  flush synchrone dans shutdown lifespan
- **P1.c** `asyncio.Lock` per app protège la re-entry
  concurrente sur le dict in-memory (pas de race corruption
  même si le dispatch FastAPI parallélise 2 handlers sur la
  même app)

Pièges :
- **P1.x** Ne pas partager un `AppStorage` entre apps — le
  lock est per-instance. Le coordinator loader instancie un
  `AppStorage(path)` distinct par app
- **P1.y** `shelve` / `sqlitedict` pickle-based sont des
  anti-patterns (non portables)

### 3.2 Pour D2 (AppContext.events)

Projets consultés :
- **anyio 4.11** (Context7 query `/agronholm/anyio`) —
  `anyio.create_memory_object_stream[T](max_buffer_size)`
  donne une paire `(send_stream, receive_stream)` typée avec
  buffer borné. **Clonable** via `receive_stream.clone()` pour
  fan-out. Pattern idéal à wrapper au lieu d'`asyncio.Queue`
  maison
- **blinker 1.9** (https://github.com/pallets-eco/blinker) —
  sync signal dispatch avec weak refs natives. Pattern API
  signal (`signal.send(...)`) inspirant mais **sync only**,
  non utilisable directement pour asyncio
- **aiopubsub 2.0.1** (https://github.com/qntln/aiopubsub) —
  asyncio pub/sub glob matching via `fnmatch`, peu maintenu
  mais confirme le choix fnmatch
- **pypubsub 4.0.3** — sync topic hierarchy, référence de doc
  sur la hiérarchie MQTT-like (rejetée au profit de fnmatch)
- **MQTT** topic hierarchy `politician/refreshed` + wildcards
  `+` et `#` — plus expressif mais parseur custom requis. Rejeté

Patterns retenus :
- **P2.a** Wrapper `create_memory_object_stream[EventEnvelope](1024)`
  par subscriber, stocker les `send_stream` dans `dict[pattern,
  list[MemoryObjectSendStream]]`
- **P2.b** Dispatch : pour chaque subscriber matching, `try:
  send_stream.send_nowait(envelope) except anyio.WouldBlock:`
  — en overflow, si `drop_oldest` : `receive_stream.receive_nowait()`
  + retry. Si `drop_newest` : skip + log warning. Si `block` :
  `await send_stream.send(envelope)` — documenté comme risky
- **P2.c** Context manager `async with subscribe(pattern) as
  stream:` — register dans `__aenter__`, unregister (`aclose()`
  sur les deux ends) dans `__aexit__`. **Pas de weak refs** —
  `weakref.ref` sur une coroutine async pose un problème de GC
  (la coroutine inline est GC'd immédiatement), et de toute
  façon le context manager gère la lifetime proprement
- **P2.d** Envelope structuré Pydantic frozen avec `trace_id`
  généré par `uuid4().hex[:16]` et `timestamp` `datetime.now(
  timezone.utc)` au moment du publish
- **P2.e** `fnmatch.fnmatch` stdlib suffit pour nos <100
  subscribers par app — pas besoin de trie compilée

Pièges :
- **P2.x** Fan-out bloquant : `await send_stream.send()` dans
  la boucle de dispatch bloque tous les autres subscribers sur
  le subscriber le plus lent. **Toujours utiliser send_nowait
  dans le dispatcher**
- **P2.y** Weak refs sur coroutines async ne marchent pas :
  la coroutine inline `async def handler(): ...` est GC'd dès
  que la référence forte disparaît. Le pattern sûr est le
  context manager

### 3.3 Pour D3 (File upload + CAS + schema v2)

Projets CAS :
- **Git objects** (https://github.com/git/git) — pattern 2-chars
  sharding `ab/cdef...`, référence canonique pour la limite FS
  64K entries par répertoire (ext3/4)
- **Restic 0.17.3** (https://github.com/restic/restic) — sharding
  `sha256[:2]/<sha256>`, dédup pre-write via
  `if blob_exists: skip`, pattern identique à notre design.
  Doc https://restic.readthedocs.io/
- **huggingface-hub 0.27** (https://github.com/huggingface/huggingface_hub)
  — cache CAS via `blobs/<sha256>` flat (sans sharding) +
  `snapshots/<commit>/<name>` symlinks. **Anti-pattern du symlink-only** —
  notre JSON manifest porte plus d'info
- **pytorch.hub** — cache flat + re-hash à la lecture pour
  garantie d'intégrité. Pattern à reprendre pour la vérif au
  boot (mais pas au download path)

Projets file upload Python :
- **FastAPI UploadFile** (Context7 query `/tiangolo/fastapi`)
  — confirme 3 pièges critiques :
  1. `await file.read()` charge tout en RAM (50 MB → RAM
     exhaustion sur burst)
  2. `max_part_size` défaut **1 MB** côté python-multipart,
     doit être passé explicitement
  3. `file.content_type` = header client, **non fiable**
- **python-multipart** — le parseur sous-jacent, doc des
  limites

Patterns retenus :
- **P3.a** Sharding `sha256[:2]/<sha256>` (git, Restic)
- **P3.b** Manifest JSON adjacent plus riche que symlink
- **P3.c** Chunked read `while chunk := await file.read(8192)`
  + `hashlib.sha256().update(chunk)` streamé vers tmpfile, puis
  rename atomique
- **P3.d** Magic bytes validation post-write : lire les ~16
  premiers octets, matcher contre whitelist hardcoded. Pas de
  dep `python-magic`
- **P3.e** Dédup pre-write : `if sha_path.exists(): return
  existing_handle` avant le disk write

Pièges :
- **P3.x** `file.content_type` client-controlled seul = faille
  de sécurité (client peut envoyer `image/png` avec un EXE)
- **P3.y** `await file.read()` ou `await request.body()` charge
  tout en RAM
- **P3.z** `max_part_size` défaut 1 MB silencieux — upload > 1 MB
  raise 413 sans explication claire

Projets schema evolution :
- **Pydantic 2 discriminated union** (Context7 query
  `/pydantic/pydantic`) — `Annotated[Union[V1, V2], Field(
  discriminator="schema_version")]` avec `Literal[1]` / `Literal[2]`
  sur chaque version. Dispatch O(1)
- **Zod 4 discriminatedUnion** (https://zod.dev/) — `z.discriminatedUnion(
  "schema_version", [v1, v2])` mirror TypeScript
- **Protobuf evolution** (https://protobuf.dev/programming-guides/proto3/#updating)
  — règle « fields ajoutés avec default sont forward-compat,
  retrait requiert `reserved` »
- **Avro schema resolution** (https://avro.apache.org/docs/current/spec.html#schema_resolution)
  — « Reader's Schema wins » : un shell v2 qui lit un descriptor
  v1 utilise ses propres defaults sur les champs absents

Patterns schema retenus :
- **P3.f** Discriminated union sur `schema_version` — pattern
  canonique Pydantic 2, dispatch O(1), messages d'erreur
  précis
- **P3.g** **`extra="forbid"` sacré** : le seul mécanisme qui
  garantit qu'un descriptor v2 reçu par un parser v1 échoue
  proprement au lieu d'être silencieusement dropé (champs en
  trop = ValidationError au lieu de « ignore unknown »). **Ne
  pas retirer sous prétexte de simplifier la compat**
- **P3.h** Fixture cross-lang `tabview_v2_canonical.json` :
  pattern établi Sprint 7 A-3 pour curator, à rejouer ici
- **P3.i** Forward compat testée : `test_v1_descriptor_validates_under_v2`
  + `test_v1_still_parses_under_v1` assertent que v1 descriptors
  continuent à fonctionner (default clause du discriminator, pas
  de champ requis new sous v1 only)
- **P3.j** Backward compat testée : `test_v2_file_upload_rejects_under_v1`
  assertion explicite qu'un descriptor v2 avec `file_upload_block`
  reçu par un parser v1 raise `ValidationError` avec
  `schema_version` mentionné

### 3.4 Pour D4 (Migration runner)

Projets consultés :
- **Alembic** (Context7 query `/sqlalchemy/alembic`) — tracking
  via table `alembic_version` (1 row), upgrade/downgrade
  graphes, **pas de SHA tracking du contenu** (ne détecte pas
  le tampering). On reprend la **structure tracking table**,
  pas l'ORM SQLAlchemy ni les branches ni les downgrades
- **Sqitch** (https://sqitch.org/) — **référence exacte** pour
  SHA tracking / tamper detection via `sqitch.plan` + table
  `changes(sha1, ...)`. Le modèle deploy/verify/revert est
  overkill pour notre v1 (on ne ship que deploy forward-only)
- **Flyway** (https://documentation.red-gate.com/flyway/) —
  CRC32 checksum (faible, collisions triviales) + mode `repair`
  qui réécrit silencieusement. **Anti-pattern** — notre design
  utilise SHA256 et n'a **pas de CLI repair**
- **Yoyo-migrations** (https://ollycope.com/software/yoyo/) —
  table `_yoyo_migration(id, ctime, hash)`, sync only, pas
  d'aiosqlite. Instructif pour le schéma table
- **dbmate** (https://github.com/amacneil/dbmate) — runner Go
  léger, pas de checksum, trop minimal
- **datasette** (https://github.com/simonw/datasette,
  `datasette/database.py`) — mini-runner aiosqlite qui scanne
  scripts SQL numérotés et les applique. **Architecture
  identique à la nôtre**, sans tampering. On ajoute le SHA256
  au-dessus

SQLite lock :
- `BEGIN IMMEDIATE` : RESERVED lock au début, EXCLUSIVE au
  commit. Bloque autres writers, readers OK. **Choix retenu**
- `BEGIN EXCLUSIVE` : EXCLUSIVE immédiat, bloque readers.
  **Overkill** pour singleton coord
- `PRAGMA locking_mode=EXCLUSIVE` : lock permanent de la
  connexion, **trop agressif**
- Source : https://www.sqlite.org/isolation.html

Patterns retenus :
- **P4.a** `BEGIN IMMEDIATE` per migration, rollback auto sur
  exception (`async with db.begin():` ou pattern
  try/except + `db.rollback()`)
- **P4.b** SHA256 stocké dans `_nexus_migrations.sha256`,
  re-vérification au boot → `MigrationTamperedError` si
  divergence. Pattern vient de Sqitch
- **P4.c** Table `_nexus_migrations(version INT PRIMARY KEY,
  slug TEXT, sha256 TEXT, applied_at TEXT)` — `version` INT
  permet `ORDER BY version` natif
- **P4.d** Dry-run = pure function qui retourne la liste des
  pendantes sans ouvrir de transaction (pattern Alembic
  `--sql`)
- **P4.e** Forward-only — pas de downgrade, rollback = nouvelle
  migration qui undo

Pièges :
- **P4.x** Flyway `repair` anti-pattern : CLI qui reset les
  checksums en silence. **À ne pas implémenter**
- **P4.y** CRC32 collisions triviales — SHA256 obligatoire en
  2025 (surcoût perf négligeable pour des fichiers < 10 KB)
- **P4.z** `PRAGMA locking_mode=EXCLUSIVE` dure toute la
  connexion — trop agressif pour un runner ephemeral

### 3.5 Pour D6 (code splitting)

Projets consultés :
- **Vite** (Context7 query `/websites/vite_dev`) — docs
  `rollupOptions.output.manualChunks` + Rolldown
  `advancedChunks` preview
- **React Router v6.30** (Context7 query `/websites/reactrouter_6_30_3`)
  — propriété `lazy: () => import(...)` native sur `createBrowserRouter`,
  exporter `Component` dans les modules lazifiés
- **React** (Context7 query `/websites/react_dev`) — article
  https://react.dev/blog/2025/02/14/sunsetting-create-react-app
  confirme que `createBrowserRouter` + `lazy` est le pattern
  canonique 2025 post-CRA
- **Excalidraw** (https://github.com/excalidraw/excalidraw,
  `excalidraw-app/vite.config.mts`) — chunker par feature
  lourde (locales, mermaid, codemirror), React externalisé
  en CDN. On prend « chunker par feature », on ne prend pas
  le CDN
- **lobe-chat** (https://github.com/lobehub/lobe-chat,
  `plugins/vite/sharedRendererConfig.ts`) — pattern `sharedManualChunks`
  avec guard `if (!id.includes('node_modules')) return;`,
  chunks nommés `vendor-icons`, `vendor-motion`, etc. **Le
  plus proche de notre cas**
- **AppFlowy-Web** (https://github.com/AppFlowy-IO/AppFlowy-Web,
  `vite.config.ts`) — `manualChunks` prod-only +
  `rollup-plugin-visualizer` via `ANALYZE_MODE=true`. Pattern
  à reprendre pour le tooling
- **tldraw** (https://github.com/tldraw/tldraw) — pas de
  manualChunks, tout sur le tree-shaking + imports dynamiques
  in-code. Leçon : **route-based et feature-based se combinent**

Patterns retenus :
- **P6.a** Migration `createBrowserRouter` + `lazy` native
  (pattern canonique 2025)
- **P6.b** Guard `if (!id.includes('node_modules')) return;`
  en tête de `manualChunks` pour éviter qu'un fichier `src/`
  atterrisse dans un chunk vendor (pattern lobe-chat)
- **P6.c** Retrait des chunks morts `vendor-graph`/`vendor-charts`/
  `vendor-map` (dep pre-pivot supprimées Sprint 5)
- **P6.d** Feature chunks nommés (`tabview`, `palette`,
  `upload`) pour invalider le cache de façon granulaire
- **P6.e** `rollup-plugin-visualizer` via `ANALYZE_MODE=true
  npm run build` pour vérifier le résultat

Pièges :
- **P6.x** Mettre `<Suspense>` autour de `AppShell` fait
  disparaître sidebar + palette pendant la navigation
- **P6.y** Chunker shadcn `src/components/ui/` dans `vendor-ui`
  — shadcn est copié dans `src/`, pas un dep npm
- **P6.z** `manualChunks` qui crée un import circulaire entre
  deux chunks → Rollup crash en prod. Toujours guard sur
  `node_modules` pour éviter qu'un fichier source atterrisse
  dans un vendor chunk

---

## 4. Phase A — Unblock: scripts + code splitting + Sprint 7/8 cleanup

**Rationale** : Phase A regroupe les items qui doivent landed
en premier pour débloquer les commits suivants. H-3 bloque les
sessions fresh (audit gate a dû rebuild manuellement), T9
bloque le premier commit React qui ajouterait > 0.5 KB, et les
4 tech debt fixes (T8, T10, T11, T12) sont des oneliners qui
gagnent à être groupés.

### 4.1 Fichiers ajoutés / modifiés

| Fichier | Action | LOC (estim) | Rôle |
|---|---|---|---|
| `scripts/setup.sh` | +new | ~90 | D5 setup script — hash Cargo.lock + maturin develop si diff |
| `scripts/verify.sh` | +new | ~80 | D5 verify script — full fail-fast suite |
| `.githooks/post-merge` | +new | ~25 | D5 git hook opt-in — rappelle `./scripts/setup.sh` |
| `docs/claude/README.md` | edit | ~40 lignes touchées | §4.3 réécrite pour pointer `./scripts/verify.sh` |
| `README.md` | edit | ~30 lignes ajoutées | Section « Quick start » |
| `docs/rust/PATTERNS.md` | edit | ~15 lignes modifiées | H-3 → CLOSED avec SHA Phase A |
| `web/src/App.tsx` | rewrite | ~55 LOC (net) | D6 `createBrowserRouter` + `lazy` native |
| `web/src/pages/Browse.tsx` | edit | +3 LOC | `export const Component = BrowsePage;` à la fin |
| `web/src/pages/Curators.tsx` | edit | +3 LOC | Idem |
| `web/src/pages/Network.tsx` | edit | +3 LOC | Idem |
| `web/src/pages/OnboardingEmpty.tsx` | edit | +3 LOC | Idem |
| `web/src/pages/ProjectDetail.tsx` | edit | +3 LOC | Idem |
| `web/src/pages/Projects.tsx` | edit | +3 LOC | Idem |
| `web/src/pages/AppTabPage.tsx` | edit | +3 LOC | Idem |
| `web/vite.config.ts` | rewrite | ~80 LOC (net) | D6 manualChunks avec guards + retrait morts + feature chunks |
| `web/.size-limit.json` | rewrite | ~60 LOC | D6 8 budgets recalibrés |
| `web/package.json` | edit | +1 devDep | `rollup-plugin-visualizer ^5` |
| `web/src/components/ui/card.tsx` | edit | ~5 LOC | **T8** CardTitle `<div>` → `<h3>` |
| `web/src/components/command-palette/CommandPalette.tsx` | edit | ~40 LOC | **T11** `runAppCommand` error inline state (pattern Sprint 8 `ButtonBlock`) |
| `web/src/components/command-palette/__tests__/CommandPalette.test.tsx` | edit | +~30 LOC | **T11** tests couvre error state |
| `packages/nexus-sdk/src/nexus_sdk/registry.py` | edit | ~20 LOC | **T12** `sorted(..., key=lambda d: d["name"])` explicite pour `workers` / `tabs` / `commands` |
| `packages/nexus-sdk/tests/test_commands.py` | edit | +~15 LOC | **T12** assertion explicite sur sort key |
| `packages/nexus-coordinator/src/nexus_coordinator/api/daemon.py` | edit | ~25 LOC | **T10** module-level `httpx.AsyncClient` singleton avec `Limits(max_connections=10)`, lifespan managed |
| `packages/nexus-coordinator/tests/test_daemon_proxy.py` | edit | +~10 LOC | **T10** test Limits respected |
| `docs/shell/PATTERNS.md` | edit | ~60 LOC | **P12** code splitting doc, T8/T10/T11/T12 → CLOSED |

### 4.2 Tests à écrire / renforcer

- `scripts/setup.sh` : test manuel (pas de test unitaire
  automatisé pour un bash script — le test est « roule-le sur
  une fresh checkout, observe qu'il produit le wheel »). Doc
  dans le commit body
- `scripts/verify.sh` : test manuel idem
- **T11 CommandPalette errors** : 3 nouveaux Vitest tests
  - `test_invoke_command_error_shows_inline_state`
  - `test_invoke_command_error_allows_retry`
  - `test_invoke_command_success_closes_palette`
- **T12 commands ordering** : 1 Vitest + 1 pytest
  - `test_commands_ordered_by_name_explicitly` — build une app
    avec commands `["z", "a", "m"]` et assert le return de
    `NexusApp.commands()` est bien `["a", "m", "z"]` (pas
    `["a", "m", "z"]` par side effect de `dir`)
- **T10 httpx Limits** : 1 pytest
  - `test_daemon_proxy_shares_httpx_client` — assert que 2
    calls concurrents utilisent le même singleton
- **D6 code splitting** : Vitest ne teste pas directement, mais
  le build + size-limit sont les tests. Playwright spec
  existante `stub-pages.spec.ts` doit passer identique (toutes
  les routes chargent)

### 4.3 Critère d'acceptation

- `./scripts/verify.sh` exit 0 (full suite verte, +~10 tests
  cumulatifs Phase A)
- `npm run size` : main ≤ 350 KB (cible D6) OU si impossible en
  une passe, ≤ 425 KB avec commit body qui documente pourquoi
  + promesse Phase B de tree-shaker
- `docs/rust/PATTERNS.md` H-3 status ligne : `Status: CLOSED
  Sprint 9 Phase A (commit <SHA>)`
- `docs/shell/PATTERNS.md` T8, T10, T11, T12 : status
  CLOSED + P12 ajouté
- Rolldown build propre : pas de warning sur chunks circulaires
- Playwright Browse / Curators / Network / Projects / ProjectDetail
  pages se chargent — 24 specs toujours verts

### 4.4 Commit cible

```
feat(web,sdk,coordinator,scripts): Sprint 9 Phase A — setup/verify scripts + createBrowserRouter code splitting + Sprint 7/8 P2 cleanup (H-3 + T8 + T10 + T11 + T12 CLOSED)
```

---

## 5. Phase B — `AppContext.storage` + gov Politiciens filter persist

### 5.1 Fichiers

| Fichier | Action | LOC (estim) | Rôle |
|---|---|---|---|
| `packages/nexus-sdk/src/nexus_sdk/storage.py` | +new | ~280 | D1 primitive : `AppStorage` class + `TypedNamespace` wrapper + write coalescing + atomic rename + asyncio.Lock |
| `packages/nexus-sdk/src/nexus_sdk/__init__.py` | edit | +~8 LOC | Export `AppStorage`, `TypedNamespace`, `StorageSchemaError` |
| `packages/nexus-sdk/src/nexus_sdk/app.py` | edit | +~12 LOC | `AppContext.storage: AppStorage | None = None` field |
| `packages/nexus-sdk/tests/test_storage.py` | +new | ~320 | 20 tests couverts §5.2 |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` | edit | +~35 LOC | Instancie `AppStorage(storage_path)` per app avant `on_start`, drain flush dans `stop()` |
| `packages/nexus-coordinator/src/nexus_coordinator/paths.py` | edit | +~8 LOC | Ajoute `app_storage_path(project, app_name)` |
| `packages/nexus-coordinator/tests/test_coordinator.py` | edit | +~15 LOC | Test lifespan flush-on-shutdown |
| `packages/nexus-app-gov/src/nexus_app_gov/filters.py` | +new | ~40 | `PoliticiansFilter` Pydantic model |
| `packages/nexus-app-gov/src/nexus_app_gov/app.py` | edit | +~30 LOC | Tab Politiciens handler lit / écrit via `ctx.storage.namespace("filters.politicians", PoliticiansFilter)` |
| `packages/nexus-app-gov/tests/test_gov_app.py` | edit | +~25 LOC | 3 tests : filter persist, filter default, filter update |
| `web/src/api/coordinator.ts` | edit | +~20 LOC | Nouveau endpoint `POST /app/{name}/storage/set` + `GET /app/{name}/storage/get` OU via tab descriptor refresh (plan : via re-render du tab descriptor, moins invasive) |
| `web/src/pages/ProjectDetail.tsx` OU nouveau | edit | ~15 LOC | UI filter inputs qui trigger `submitAppTask` ou re-render tab |
| `web/e2e/gov-politicians-filter-persist.spec.ts` | +new | ~80 LOC | Playwright : set filter → reload → filter conservé |
| `docs/shell/PATTERNS.md` | edit | ~30 LOC | **P13** — AppContext.storage pattern (typed namespaces, per-app scope, atomic rename, coalescing) |

### 5.2 Tests à écrire (SDK)

Nommage + scénario pour les 20 tests de `test_storage.py` :

1. `test_storage_get_missing_key_returns_none` — empty state
2. `test_storage_set_get_roundtrip_string` — happy path
3. `test_storage_set_get_roundtrip_nested_dict` — serialization
4. `test_storage_delete_key_removes_from_state`
5. `test_storage_keys_returns_sorted_list`
6. `test_storage_keys_with_prefix_filter`
7. `test_storage_clear_removes_all_keys`
8. `test_storage_set_triggers_deferred_flush` — via
   `asyncio.call_later(0.5, flush)` — mocker le timer
9. `test_storage_multiple_sets_coalesce_into_one_flush` —
   setter 5x, attendre 0.6s, vérifier 1 seul `os.replace`
10. `test_storage_flush_on_shutdown_writes_pending` —
    lifespan shutdown force immediate flush
11. `test_storage_atomic_rename_uses_tmpfile` — mocker
    `os.replace`, vérifier qu'on écrit un tmpfile distinct
    puis on rename
12. `test_storage_concurrent_set_via_asyncio_lock` — 2
    tasks qui font `set` concurrent, vérifier l'état final
    cohérent (pas de corruption JSON)
13. `test_storage_reentry_same_task_does_not_deadlock` — un
    handler qui appelle `set` dans un `set` handler
14. `test_storage_namespace_typed_get_returns_validated_model`
15. `test_storage_namespace_typed_set_accepts_model_instance`
16. `test_storage_namespace_typed_set_rejects_invalid_dict` —
    raise `StorageSchemaError`
17. `test_storage_namespace_typed_get_returns_default_on_missing_key`
18. `test_storage_namespace_untyped_fallback` — `get` sans
    namespace return raw value
19. `test_storage_persists_across_restart` — instance 1 set,
    instance 2 créé sur même path, get retourne value
20. `test_storage_missing_file_creates_lazy_on_first_set`

### 5.3 Tests gov

1. `test_politicians_filter_loads_default_empty_state`
2. `test_politicians_filter_set_persisted`
3. `test_politicians_filter_roundtrip_via_app_context`

### 5.4 Playwright spec

`web/e2e/gov-politicians-filter-persist.spec.ts` :

1. Navigate to `/project/default/app/gov/tabs/Politiciens`
2. Set chamber filter = "Assemblée", date_range = "2024-2026",
   search = "Dupont"
3. Wait for tab descriptor re-render with filter applied
4. `page.reload()`
5. Assert the filter inputs still show "Assemblée", "2024-2026",
   "Dupont"
6. Assert the grid reflects the filter (limited row count)

### 5.5 Critère d'acceptation

- `./scripts/verify.sh` exit 0
- SDK : **91 tests** (71 → +20, delta +20)
- Coord : **64 tests** (+1 lifespan flush)
- app-gov : **33 tests** (+3)
- Playwright : **25 specs** (+1)
- Budget main respecté (mesurable : le test de régression D6)
- `docs/shell/PATTERNS.md` P13 ajouté

### 5.6 Commit cible

```
feat(sdk,coordinator,app-gov,web): Sprint 9 Phase B — AppContext.storage + typed namespaces + gov Politiciens filter persist consumer
```

---

## 6. Phase C — `AppContext.events` + gov party.refreshed consumer + SSE endpoint

### 6.1 Fichiers

| Fichier | Action | LOC (estim) | Rôle |
|---|---|---|---|
| `packages/nexus-sdk/src/nexus_sdk/events.py` | +new | ~320 | D2 primitive : `AppEvents` wrapping anyio memory stream, `EventEnvelope` Pydantic, glob matching, overflow policy, context manager |
| `packages/nexus-sdk/pyproject.toml` | edit | +1 dep | `anyio >= 4.0` explicite |
| `packages/nexus-sdk/src/nexus_sdk/__init__.py` | edit | +~5 LOC | Export `AppEvents`, `EventEnvelope`, `EventOverflowPolicy` |
| `packages/nexus-sdk/src/nexus_sdk/app.py` | edit | +~8 LOC | `AppContext.events: AppEvents | None = None` |
| `packages/nexus-sdk/tests/test_events.py` | +new | ~380 | 25 tests couverts §6.2 |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` | edit | +~15 LOC | Instancie `AppEvents()` per app |
| `packages/nexus-coordinator/src/nexus_coordinator/api/events.py` | +new | ~180 | SSE router `GET /app/{name}/events?pattern=...` qui subscribe au bus per-app et streame `EventEnvelope` JSON |
| `packages/nexus-coordinator/src/nexus_coordinator/api/app.py` | edit | +~5 LOC | Mount du router events |
| `packages/nexus-coordinator/tests/test_events_sse.py` | +new | ~140 | 3 tests : stream basic, pattern filter, disconnect cleanup |
| `packages/nexus-app-gov/src/nexus_app_gov/workers.py` | edit | +~45 LOC | Nouveau worker `gov.refresh_party_cache` (stub qui lit legacy + publish) |
| `packages/nexus-app-gov/src/nexus_app_gov/app.py` | edit | +~15 LOC | Subscribe le tab Politiciens au topic `party.refreshed` via SSE client-side |
| `packages/nexus-app-gov/tests/test_gov_events.py` | +new | ~80 | 3 tests : worker publishes, tab re-fetches, empty state si no event |
| `web/src/hooks/useAppEvents.ts` | +new | ~90 | Hook React : ouvre EventSource sur `/app/{name}/events?pattern=...`, invalide React Query cache sur match |
| `web/src/hooks/__tests__/useAppEvents.test.ts` | +new | ~110 | 6 Vitest tests : open stream, pattern filter, invalidate cache, cleanup on unmount, reconnect on error, multiple subscribers |
| `web/src/pages/ProjectDetail.tsx` | edit | +~8 LOC | Use `useAppEvents('party.refreshed', ...)` pour tab Politiciens |
| `web/e2e/gov-party-refresh-event.spec.ts` | +new | ~90 LOC | Trigger worker via palette → attendre SSE event → grid update sans reload |
| `docs/shell/PATTERNS.md` | edit | ~40 LOC | **P14** — AppContext.events asyncio pub/sub (anyio + glob + context manager + SSE bridge) |

### 6.2 Tests SDK (25)

1. `test_publish_without_subscribers_is_noop`
2. `test_subscribe_receives_matching_event`
3. `test_subscribe_filters_non_matching_event`
4. `test_glob_pattern_star_matches_single_segment` —
   `politician.*` match `politician.refreshed` mais pas
   `politician.party.refreshed`
5. `test_glob_pattern_prefix_wildcard` — `*.refreshed`
6. `test_multi_subscribers_receive_fanout`
7. `test_envelope_has_topic_payload_timestamp_trace_id`
8. `test_envelope_trace_id_is_unique_per_publish`
9. `test_envelope_timestamp_is_utc_iso8601`
10. `test_bounded_queue_blocks_when_full_policy_block`
11. `test_bounded_queue_drops_oldest_by_default` — 1024 events
    publiés, un seul subscriber, lent → drop_oldest log
    `warning` throttled
12. `test_bounded_queue_drops_newest_policy` — pareil mais
    drop_newest
13. `test_context_manager_registers_on_enter` — `async with
    subscribe(...)` appel registry add
14. `test_context_manager_unregisters_on_exit` — assertion
    post-exit que le subscriber n'est plus dans le registry
15. `test_context_manager_unregisters_on_exception` — raise
    dans le body
16. `test_multiple_context_managers_coexist`
17. `test_fnmatch_pattern_validation_on_subscribe_raises_on_invalid`
18. `test_publish_is_sync_send_nowait_not_await` — vérifier
    que `publish()` retourne sans `await send()` bloquant
19. `test_subscribe_after_publish_misses_event` — pas de
    replay (in-memory only)
20. `test_envelope_payload_is_serializable_dict` — rejette
    un payload non-JSON
21. `test_overflow_drop_oldest_logs_warning_once_per_minute`
22. `test_subscriber_with_slow_consumer_does_not_block_others`
23. `test_per_app_scope_isolation` — app A publish, app B
    subscribe — B ne reçoit rien
24. `test_shutdown_closes_all_subscribers_gracefully`
25. `test_event_bus_stats_reports_subscribers_count`

### 6.3 Tests coord SSE

1. `test_events_sse_streams_envelope_on_publish`
2. `test_events_sse_filters_by_pattern`
3. `test_events_sse_disconnect_unregisters_subscriber`

### 6.4 Critère d'acceptation

- `./scripts/verify.sh` exit 0
- SDK : **116 tests** (91 → +25)
- Coord : **67 tests** (+3 SSE)
- app-gov : **36 tests** (+3)
- Vitest : **148 tests** (+6 useAppEvents)
- Playwright : **26 specs** (+1)
- Budget main + nouveau chunk impact vérifié < budget

### 6.5 Commit cible

```
feat(sdk,coordinator,app-gov,web): Sprint 9 Phase C — AppContext.events anyio pub/sub + SSE endpoint + gov party.refreshed consumer
```

---

## 7. Phase D — Migration runner + gov 001_documents.sql consumer

### 7.1 Fichiers

| Fichier | Action | LOC | Rôle |
|---|---|---|---|
| `packages/nexus-sdk/src/nexus_sdk/migrations.py` | +new | ~240 | D4 primitive : scan lexico, SHA256 tracking, BEGIN IMMEDIATE, `_nexus_migrations` table, `MigrationTamperedError` |
| `packages/nexus-sdk/src/nexus_sdk/app.py` | edit | +~5 LOC | `AppManifest.migrations_dir: Path | None = None` field |
| `packages/nexus-sdk/src/nexus_sdk/__init__.py` | edit | +~4 LOC | Export `MigrationRunner`, `MigrationTamperedError`, `PendingMigration` |
| `packages/nexus-sdk/tests/test_migrations.py` | +new | ~320 | 18 tests §7.2 |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` | edit | +~25 LOC | Nouveau step boot : scan + apply migrations après `on_start`, avant dispatcher tick |
| `packages/nexus-coordinator/src/nexus_coordinator/cli.py` | edit | +~60 LOC | Nouveau sous-commande Typer `migrate --project --app --plan/--apply` |
| `packages/nexus-coordinator/tests/test_cli_migrate.py` | +new | ~90 | 5 tests CLI |
| `packages/nexus-app-gov/src/nexus_app_gov/migrations/001_documents.sql` | +new | ~15 lignes SQL | Crée `gov_documents` table |
| `packages/nexus-app-gov/src/nexus_app_gov/app.py` | edit | +~30 LOC | `GovApp.on_start` instancie 2 `AppDatabaseClient` : `ctx.db_gov` read-only sur govdata.db + `ctx.db_app` writable sur `projects/.../apps/gov/app.sqlite` |
| `packages/nexus-sdk/src/nexus_sdk/app.py` | edit | +~8 LOC | `AppContext.dbs: dict[str, AppDatabaseClient]` (remplace `AppContext.db` single par un dict — backward-compat via property `ctx.db` qui retourne `dbs["default"]`) |
| `packages/nexus-app-gov/tests/test_gov_migrations.py` | +new | ~100 | 4 tests : runner applique 001, idempotence, SHA verification |
| `docs/shell/PATTERNS.md` | edit | ~30 LOC | **P15** — Migration runner (Sqitch-inspired, forward-only, SHA256 tamper detection, pas de repair) |

### 7.2 Tests SDK (18)

1. `test_runner_applies_single_migration_happy_path`
2. `test_runner_is_idempotent_on_second_boot`
3. `test_runner_detects_tampered_migration_raises` —
   applique 001, édite le fichier, reboot → `MigrationTamperedError`
4. `test_runner_rollbacks_failing_statement_leaves_clean_state`
   — 001 a 2 statements, la 2e fail → table pas créée, row
   pas dans `_nexus_migrations`
5. `test_runner_applies_in_lexico_order` — 003 avant 002
   avant 001 dans le dir listing → runner les applique dans
   l'ordre numérique
6. `test_runner_dry_run_does_not_touch_db`
7. `test_runner_creates_tracking_table_on_first_run`
8. `test_runner_skip_if_migrations_dir_none`
9. `test_runner_skip_if_migrations_dir_empty`
10. `test_runner_extracts_version_from_filename_prefix` —
    `001_init.sql` → version=1
11. `test_runner_extracts_slug_from_filename` — slug = "init"
12. `test_runner_refuses_read_only_client` — raise
    `ValueError("migrations runner requires a writable client")`
13. `test_runner_forward_only_rejects_version_backward_jump`
    — applied 003, now scanning dir finds only 001 002 →
    raise (missing 003)
14. `test_runner_applied_at_is_utc_iso8601`
15. `test_runner_sha256_stored_matches_file_content`
16. `test_runner_concurrent_runs_blocked_by_begin_immediate`
    — spawn 2 runners sur le même client, assert un seul
    wins (l'autre raise OperationalError busy)
17. `test_migration_tampered_error_message_cites_file_and_hashes`
18. `test_runner_logs_info_on_apply_error_on_tamper`

### 7.3 Tests CLI coord (5)

1. `test_cli_migrate_plan_lists_pending`
2. `test_cli_migrate_apply_happy_path`
3. `test_cli_migrate_unknown_app_exits_1`
4. `test_cli_migrate_all_apps_when_no_app_arg`
5. `test_cli_migrate_refuses_to_run_on_unkown_project`

### 7.4 Tests gov (4)

1. `test_gov_migration_001_creates_documents_table`
2. `test_gov_migration_is_idempotent_on_coordinator_restart`
3. `test_gov_dbs_contains_db_gov_and_db_app`
4. `test_gov_db_gov_is_read_only_db_app_is_writable`

### 7.5 Critère d'acceptation

- `./scripts/verify.sh` exit 0
- SDK : **134 tests** (116 → +18)
- Coord : **72 tests** (+5 CLI)
- app-gov : **40 tests** (+4)
- `nexus-coordinator migrate --project default --app gov --plan`
  affiche les migrations pendantes sur stdout
- `_nexus_migrations` table exists dans `gov/app.sqlite`
  après premier boot

### 7.6 Commit cible

```
feat(sdk,coordinator,app-gov): Sprint 9 Phase D — DB migration runner (SHA256 tamper detection, CLI plan/apply) + gov 001_documents.sql consumer
```

---

## 8. Phase E — File upload + CAS + TabView v2 + gov Documents tab + Sprint 7 Rust tech debt

### 8.1 Fichiers

| Fichier | Action | LOC | Rôle |
|---|---|---|---|
| `packages/nexus-sdk/src/nexus_sdk/files.py` | +new | ~340 | D3 primitive : `AppFileStore`, `FileHandle`, `FileManifest`, chunked read, magic bytes validation, dédup pre-write |
| `packages/nexus-sdk/src/nexus_sdk/decorators.py` | edit | +~35 LOC | `@nexus_app_files(accept=[...])` class-level decorator |
| `packages/nexus-sdk/src/nexus_sdk/view.py` | rewrite partial | ~200 LOC touché | Refacto en `TabViewV1` / `TabViewV2` / `AnyTabView` discriminated union, new `file_upload_block()` constructor |
| `packages/nexus-sdk/tests/snapshots/tabview_schema.json` | rename | — | → `tabview_v1_schema.json` (backward-compat checkpoint) |
| `packages/nexus-sdk/tests/snapshots/tabview_v2_canonical.json` | +new | ~60 LOC | Cross-lang fixture v2 |
| `packages/nexus-sdk/tests/test_files.py` | +new | ~280 | 20 tests §8.2 |
| `packages/nexus-sdk/tests/test_view_v2.py` | +new | ~180 | 12 tests §8.3 |
| `packages/nexus-sdk/src/nexus_sdk/__init__.py` | edit | +~6 LOC | Export `AppFileStore`, `FileHandle`, `FileManifest`, `nexus_app_files`, `file_upload_block`, `TabViewV1`, `TabViewV2`, `AnyTabView` |
| `packages/nexus-coordinator/src/nexus_coordinator/api/files.py` | +new | ~220 | Router `POST /app/{name}/files/upload` multipart chunked |
| `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py` | edit | ~15 LOC | `TabView.model_validate` → `AnyTabView.model_validate` |
| `packages/nexus-coordinator/src/nexus_coordinator/api/app.py` | edit | +~3 LOC | Mount files router |
| `packages/nexus-coordinator/tests/test_files.py` | +new | ~180 | 10 tests §8.4 |
| `packages/nexus-coordinator/tests/test_apps.py` | edit | +~20 LOC | Update `_coerce_tab_view` → `AnyTabView.model_validate`, 2 new tests v2 |
| `web/src/components/app/tabview/schema.ts` | rewrite partial | ~130 LOC touché | Zod `z.discriminatedUnion("schema_version", [...])` |
| `web/src/components/app/tabview/blocks/FileUploadBlock.tsx` | +new | ~180 LOC | React block : drag-drop natif, progress via D2 SSE, thumbnail preview |
| `web/src/components/app/tabview/TabBlockRenderer.tsx` | edit | +~10 LOC | Switch case `file_upload_block` (v2 only) |
| `web/src/components/app/tabview/__tests__/FileUploadBlock.test.tsx` | +new | ~150 LOC | 8 Vitest tests |
| `web/src/components/app/tabview/__tests__/schema_v2_cross_lang.test.ts` | +new | ~60 LOC | Lit `tabview_v2_canonical.json` côté Vitest, assert roundtrip |
| `packages/nexus-app-gov/src/nexus_app_gov/app.py` | edit | +~50 LOC | Nouveau 20e tab « Documents » : liste `gov_documents` via `ctx.dbs["app"].fetchall()`, render `file_upload_block(...)` v2 |
| `packages/nexus-app-gov/tests/test_gov_documents.py` | +new | ~120 | 6 tests gov Documents |
| `web/e2e/gov-documents-upload.spec.ts` | +new | ~110 LOC | Upload PDF → progress bar SSE → visible dans la liste |
| `crates/nexus-core-rs/src/discovery.rs` | edit | ~20 LOC | **E-1** fix : `DEFAULT_PROBE_TIMEOUT` configurable via env |
| `crates/nexus-core-rs/tests/probe_timeout.rs` | +new | ~40 LOC | **E-1** test env override |
| `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` | edit | ~60 LOC | **C-4** `tokio::sync::Semaphore(32)` autour du gossip handler + **D-3** try-persist-first rewrite de `subscribe` |
| `crates/nexus-shell-daemon-core/tests/backpressure.rs` | +new | ~60 LOC | **C-4** test 100 messages burst bloqué par semaphore |
| `crates/nexus-shell-daemon-core/tests/subscribe_persist_first.rs` | +new | ~50 LOC | **D-3** test rollback RAM sur persist fail |
| `docs/rust/PATTERNS.md` | edit | ~30 LOC | E-1, C-4, D-3 → CLOSED |
| `docs/shell/PATTERNS.md` | edit | ~50 LOC | **P16** — File upload + CAS + magic bytes + TabView v2 evolution, **P17** — D2/D3 wiring via SSE progress |

### 8.2 Tests SDK files (20)

1. `test_store_happy_path_returns_file_handle`
2. `test_store_writes_to_cas_sharded_path`
3. `test_store_creates_manifest_json_adjacent`
4. `test_store_dedupe_skip_if_sha256_exists`
5. `test_store_chunked_read_large_file_50mb`
6. `test_store_computes_sha256_incrementally`
7. `test_open_returns_async_iterator_bytes`
8. `test_open_raises_on_missing_sha256`
9. `test_manifest_returns_none_if_not_found`
10. `test_manifest_includes_size_and_content_type`
11. `test_magic_bytes_png_accepted`
12. `test_magic_bytes_jpeg_accepted`
13. `test_magic_bytes_pdf_accepted`
14. `test_magic_bytes_webp_accepted`
15. `test_magic_bytes_svg_accepted`
16. `test_magic_bytes_unknown_raises_and_deletes_file` —
    EXE signature `MZ` → delete + raise
17. `test_magic_bytes_png_signature_with_png_content_type_header_but_actual_exe_rejects`
    — adversarial case
18. `test_delete_soft_removes_manifest_keeps_cas_file`
19. `test_concurrent_store_same_sha256_dedup_safe` —
    2 tasks qui store le même contenu, une seule entry
    CAS, 2 manifests distincts
20. `test_missing_allowlist_decorator_raises`

### 8.3 Tests SDK view_v2 (12)

1. `test_v1_descriptor_validates_under_v1_schema` — backward
   sanity
2. `test_v1_descriptor_validates_under_anytabview` — forward
   compat discriminated
3. `test_v2_descriptor_with_file_upload_block_parses`
4. `test_v2_descriptor_parsed_as_v2_instance` — type check
5. `test_v2_file_upload_block_rejected_under_v1` — backward
   compat explicite : ValidationError
6. `test_v2_extra_forbid_preserved_on_all_blocks`
7. `test_v2_extra_field_raises_validation_error`
8. `test_cross_lang_fixture_v2_roundtrip_python_side` — lit
   `tabview_v2_canonical.json`, parse, re-dump, assert égal
9. `test_v2_file_upload_block_constructor_helper`
10. `test_v2_file_upload_block_accept_validation` — `accept=
    ["image/*"]` valide
11. `test_v2_file_upload_block_max_size_bytes_validation`
12. `test_schema_version_literal_enforced` — un descriptor
    avec `schema_version: 3` → ValidationError

### 8.4 Tests coord files (10)

1. `test_upload_happy_path_returns_201_with_sha256`
2. `test_upload_cap_50mb_rejects_larger`
3. `test_upload_multipart_max_part_size_explicit_50mb`
4. `test_upload_content_type_mismatch_rejects_415` — magic
   bytes lies
5. `test_upload_app_without_files_decorator_404`
6. `test_upload_dedup_returns_existing_sha_with_header`
7. `test_upload_streams_progress_to_events_bus` — spawn
   subscriber `file.upload.progress`, upload 100 KB,
   assert N events received
8. `test_manifest_endpoint_returns_metadata`
9. `test_open_endpoint_streams_bytes`
10. `test_upload_writes_to_cas_sharded_path_via_coordinator`

### 8.5 Tests gov documents (6)

1. `test_documents_tab_empty_state_when_no_uploads`
2. `test_documents_tab_lists_uploaded_via_db_app`
3. `test_documents_tab_descriptor_uses_v2_schema`
4. `test_documents_tab_renders_file_upload_block`
5. `test_gov_app_accepts_pdf_and_images`
6. `test_gov_documents_migration_creates_table`

### 8.6 Playwright spec

`web/e2e/gov-documents-upload.spec.ts` :

1. Navigate to `/project/default/app/gov/tabs/Documents`
2. Assert empty state "Aucun document"
3. Drag a small PDF fixture into the drop zone
4. Assert progress bar appears + reaches 100%
5. Wait for SSE event `file.upload.progress` via event listener
6. Assert the list shows the new document row
7. Click the document → vérifier le preview fonctionne

### 8.7 Critère d'acceptation

- `./scripts/verify.sh` exit 0
- SDK : **166 tests** (134 → +32 : +20 files + +12 view_v2)
- Coord : **82 tests** (+10 files)
- app-gov : **46 tests** (+6)
- Rust : **312 tests** (309 → +3 : +1 probe timeout env, +1
  semaphore backpressure, +1 subscribe persist-first)
- Vitest : **156 tests** (+8 FileUploadBlock)
- Playwright : **27 specs** (+1 documents upload)
- `docs/rust/PATTERNS.md` E-1, C-4, D-3 → CLOSED
- `docs/shell/PATTERNS.md` P16, P17 ajoutés

### 8.8 Commit cible

```
feat(sdk,coordinator,app-gov,web,core-rs,shell-daemon-core): Sprint 9 Phase E — file upload + CAS + TabView v2 bump + gov Documents tab + Sprint 7 E-1/C-4/D-3 tech debt
```

---

## 9. Phase F — verification.md + audit_plan.md for Sprint 10

### 9.1 Fichiers

| Fichier | Action | LOC | Rôle |
|---|---|---|---|
| `.planning/sprint9_verification.md` | +new | ~600 | Self-report fail-fast ~38 rows, format Sprint 6/7/8, HEAD entrée/sortie, commit stack, how to re-run, métriques delta, surface nouvelle livrée par phase, scope cuts respectés, checkpoint clôture |
| `.planning/sprint9_audit_plan.md` | +new | ~700 | Plan d'audit que Sprint 10 Phase 0 jouera. 10 tracks : A code splitting sanity, B storage primitive + gov consumer, C events primitive + SSE + gov consumer, D migrations + tamper + gov consumer, E file upload + CAS + magic bytes + v2 evolution, F schema v2 forward/backward compat, G scripts hygiene, H bundle headroom + chunks, I Sprint 7 Rust closures, J doc consistency |
| `docs/shell/PATTERNS.md` | edit | — | (déjà touché phases A-E, récap si nécessaire en fin) |
| `docs/rust/PATTERNS.md` | edit | — | Idem |
| Memory `nexus_grid_pivot.md` | edit | — | Sprint 9 CLOSED, tip master, compteurs, transition Sprint 10 scope |

### 9.2 Critère d'acceptation

- Phase F est **strictement doc** — aucun code change
- fail-fast suite identique au tip Phase E commit (rien à re-run)
- 2 fichiers `.planning/sprint9_*.md` livrés (verification +
  audit_plan), 2 PATTERNS updates, memory update
- Checkpoint clôture Sprint 9 vérifié

### 9.3 Commit cible

```
docs(sprint9): verification + audit plan for Sprint 10
```

---

## 10. Fail-fast checklist (à rejouer au verification.md)

Table canonique : `| # | Check | Commande | Critère | Observed |`
— 38 rows. La colonne `Observed` reste vide dans ce plan,
remplie en `sprint9_verification.md` Phase F.

| # | Check | Commande | Critère | Observed |
|---|---|---|---|---|
| 1 | Rust build | `cargo build --workspace --locked` | exit 0, 0 warning | |
| 2 | Rust fmt | `cargo fmt --all --check` | exit 0 | |
| 3 | Rust clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 | |
| 4 | Rust tests | `cargo test --workspace --locked` | ≥ 312 (309 + 3 Phase E E-1/C-4/D-3) | |
| 5 | Rust — E-1 probe timeout env | `cargo test -p nexus-core-rs discovery::tests::probe_timeout_env_override` | 1 pass | |
| 6 | Rust — C-4 backpressure | `cargo test -p nexus-shell-daemon-core iroh_runtime::tests::gossip_semaphore_backpressure` | 1 pass | |
| 7 | Rust — D-3 subscribe persist-first | `cargo test -p nexus-shell-daemon-core iroh_runtime::tests::subscribe_persist_first_rollback` | 1 pass | |
| 8 | SDK full suite | `uv run pytest packages/nexus-sdk/tests/ -q` | ≥ 166 (71 + 95 new) | |
| 9 | SDK — storage | `uv run pytest packages/nexus-sdk/tests/test_storage.py -q` | ≥ 20 pass | |
| 10 | SDK — events | `uv run pytest packages/nexus-sdk/tests/test_events.py -q` | ≥ 25 pass | |
| 11 | SDK — migrations | `uv run pytest packages/nexus-sdk/tests/test_migrations.py -q` | ≥ 18 pass | |
| 12 | SDK — files | `uv run pytest packages/nexus-sdk/tests/test_files.py -q` | ≥ 20 pass | |
| 13 | SDK — view_v2 | `uv run pytest packages/nexus-sdk/tests/test_view_v2.py -q` | ≥ 12 pass | |
| 14 | SDK — cross-lang v2 fixture | `uv run pytest packages/nexus-sdk/tests/test_view_v2.py::test_cross_lang_fixture_v2_roundtrip_python_side -q` | 1 pass | |
| 15 | Coord full suite | `uv run pytest packages/nexus-coordinator/tests/ -q` | ≥ 82 (63 + 19 new) | |
| 16 | Coord — SSE events | `uv run pytest packages/nexus-coordinator/tests/test_events_sse.py -q` | ≥ 3 pass | |
| 17 | Coord — files upload | `uv run pytest packages/nexus-coordinator/tests/test_files.py -q` | ≥ 10 pass | |
| 18 | Coord — CLI migrate | `uv run pytest packages/nexus-coordinator/tests/test_cli_migrate.py -q` | ≥ 5 pass | |
| 19 | Coord — lifespan flush | `uv run pytest packages/nexus-coordinator/tests/test_coordinator.py::test_lifespan_flushes_app_storage -q` | 1 pass | |
| 20 | app-gov full suite | `uv run pytest packages/nexus-app-gov/tests/ -q` | ≥ 46 (30 + 16 new) | |
| 21 | app-gov — filter persist | `uv run pytest packages/nexus-app-gov/tests/test_gov_app.py -k filter -q` | ≥ 3 pass | |
| 22 | app-gov — party.refreshed event | `uv run pytest packages/nexus-app-gov/tests/test_gov_events.py -q` | ≥ 3 pass | |
| 23 | app-gov — migrations 001 | `uv run pytest packages/nexus-app-gov/tests/test_gov_migrations.py -q` | ≥ 4 pass | |
| 24 | app-gov — documents tab | `uv run pytest packages/nexus-app-gov/tests/test_gov_documents.py -q` | ≥ 6 pass | |
| 25 | ruff | `uv run ruff format --check packages/ examples/ && uv run ruff check packages/ examples/` | exit 0 | |
| 26 | tsc strict | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 | |
| 27 | ESLint | `cd web && npm run lint` | 0 err, ≤ 5 T1 warnings | |
| 28 | Vitest unit | `cd web && npm run test:unit` | ≥ 156 (142 + 14 new) | |
| 29 | Vitest coverage | `cd web && npm run test:coverage` | lines ≥ 90, funcs ≥ 90, branches ≥ 85, stmts ≥ 90 | |
| 30 | Vite build | `cd web && npm run build` | exit 0, 0 warning | |
| 31 | size-limit budgets | `cd web && npm run size` | 8/8 green | |
| 32 | — main chunk | (row 31 subpart) | main ≤ 350 KB OR ≤ 425 KB with rationale | |
| 33 | — feature chunks | (row 31 subpart) | tabview ≤ 80, palette ≤ 40, upload ≤ 30, vendor-query ≤ 50 | |
| 34 | Playwright | `cd web && npx playwright test` | ≥ 27 pass (24 + 3 new : filter, party-refresh, documents-upload) | |
| 35 | scan-en-strings | `cd web && bash scripts/scan-en-strings.sh` | exit 0 | |
| 36 | npm audit | `cd web && npm audit --audit-level=high` | 0 high/crit | |
| 37 | setup.sh idempotent | `./scripts/setup.sh && ./scripts/setup.sh` | exit 0, 2nd run skip | |
| 38 | verify.sh full run | `./scripts/verify.sh` | exit 0 | |

**Note** : row 32/33 sont des sub-assertions de row 31. Si le
budget main passe mais feature chunks échouent, c'est row 33
qui fail, pas row 31 — granularité utile pour l'auditeur
Sprint 10.

---

## 11. Git plan

Ordre des commits Sprint 9 (6 feat + 1 docs final — sans
compter le `docs(sprint9): kickoff + plan` d'ouverture) :

1. `docs(sprint9): kickoff + plan` — ouverture, ce fichier +
   le kickoff (pré-Phase A)
2. `feat(web,sdk,coordinator,scripts): Sprint 9 Phase A —
   setup/verify scripts + createBrowserRouter code splitting
   + Sprint 7/8 P2 cleanup (H-3 + T8 + T10 + T11 + T12 CLOSED)`
3. `feat(sdk,coordinator,app-gov,web): Sprint 9 Phase B —
   AppContext.storage + typed namespaces + gov Politiciens
   filter persist consumer`
4. `feat(sdk,coordinator,app-gov,web): Sprint 9 Phase C —
   AppContext.events anyio pub/sub + SSE endpoint + gov
   party.refreshed consumer`
5. `feat(sdk,coordinator,app-gov): Sprint 9 Phase D — DB
   migration runner (SHA256 tamper detection, CLI plan/apply)
   + gov 001_documents.sql consumer`
6. `feat(sdk,coordinator,app-gov,web,core-rs,shell-daemon-core):
   Sprint 9 Phase E — file upload + CAS + TabView v2 bump +
   gov Documents tab + Sprint 7 E-1/C-4/D-3 tech debt`
7. `docs(sprint9): verification + audit plan for Sprint 10`

Chaque commit body suit la discipline `docs/claude/README.md`
§4.1 : fichiers touchés avec rationale, delta de tests cumulé
pour chaque suite, scope cuts honorés, co-author line.

**Interdit** :

- `git commit --amend` (jamais)
- `git push --force` (jamais)
- Band-aid fixes sous prétexte de passer les tests
- Mélange doc + code dans la Phase F

Si un fix post-commit est nécessaire (test flakiness révélé
après le feat commit, clippy warning sur un rustc update,
etc.), le fix vit dans un commit séparé `fix(sprint9): ...`.

---

## 12. Scope cuts (copie kickoff §6)

Repris ici pour que l'agent exécuteur n'ait pas à switcher de
fichier pendant l'écriture de code.

### 12.1 Ce que Sprint 9 ne livre PAS

- Pas de branding, renommage, docs public — Sprint 10+
- Pas de release v1.0, PyPI publish, npm publish — Sprint 10
- Pas de 3 VPS bootstrap — Sprint 10
- Pas de cross-app events (`AppContext.events` per-app
  in-process only) — Sprint 10+ si consommateur le demande
- Pas de cross-node events — Sprint 11+
- Pas de `AppContext.storage` cross-app — per-app strict
- Pas de downgrade migration runner — forward-only
- Pas de CLI `repair` migration — anti-pattern Flyway
- Pas de cloud storage / S3 / blob store uploads — CAS
  filesystem local only
- Pas de streaming au-delà du endpoint upload (les apps
  passent par `ctx.files.open` pour streamer)
- Pas de `python-magic` dep — whitelist hardcoded 5 magics
- Pas de toast lib (sonner/react-hot-toast) — T11 via pattern
  inline Sprint 8
- Pas de Rolldown `advancedChunks` — Sprint 10+
- Pas de RSC, SSR, SPA only
- Pas de route loader React Router `loader` — React Query
  directement
- Pas de module fédéré / micro-frontend

### 12.2 Sprint 7 tech debt pas traitée

- F-2 CommandPalette loading state P3 — Sprint 10+
- G-3 daemon DTOs `deny_unknown_fields` — CLOSED Sprint 8
  Phase A, hors scope

### 12.3 Sprint 8 audit P3 laissés tels quels (12 items)

A-FX-1, B-FX-1 à B-FX-3, C-FX-3 à C-FX-5, D-FX-2 à D-FX-3,
E-FX-1 à E-FX-3, V-FX-2 — reportés Sprint 10+ ou ignorés
selon coût/bénéfice. Cf. `sprint8_audit_findings.md` §P3.

---

## 13. Risks (R1..R10)

### R1 — D6 main budget 350 KB pas atteint en une passe

**Probabilité** : moyenne. Le bundle actuel est 474.5 KB et
la refacto `createBrowserRouter` + chunks déplace ~150 KB
vers des feature chunks lazy. Marge théorique ~50 KB sous
325 KB, mais les imports circulaires ou une config incorrecte
peuvent bouffer cette marge.

**Mitigation** :

- Si main > 350 KB post Phase A, commit `fix(sprint9): relax
  main budget to 425 KB pending tree-shake pass` avec
  rationale explicite + `rollup-plugin-visualizer` screenshot
  dans le body
- Jamais au-dessus de 425 KB
- Tree-shake `lucide-react` icons (~30 importés, ~10 rendus)
  comme premier levier facile
- Si vraiment bloqué → split `AppTabPage.tsx` en chunk dédié

### R2 — anyio memory stream primitive ne supporte pas exactement le pattern clone-per-subscriber

**Probabilité** : faible. Context7 a confirmé que
`MemoryObjectReceiveStream.clone()` existe et est le pattern.

**Mitigation** :

- Si clone pose problème : fallback sur
  `dict[pattern, list[(send, receive)]]` — chaque subscriber
  crée sa propre paire et le bus dispatch sur tous les
  `send_stream` matching
- Test unitaire `test_multi_subscribers_receive_fanout`
  validera tôt

### R3 — FastAPI multipart `max_part_size` override ne fonctionne pas comme espéré

**Probabilité** : faible. Les docs Context7 sont claires.

**Mitigation** :

- Si `UploadFile` param ne respecte pas le `max_part_size`,
  fallback sur `request.stream()` manuel
- Test `test_upload_multipart_max_part_size_explicit_50mb`
  validera tôt

### R4 — Magic bytes whitelist PNG/JPG/WEBP/SVG/PDF incomplète pour gov Documents

**Probabilité** : moyenne. Les documents gov peuvent être des
DOC/DOCX/XLS.

**Mitigation** :

- Décider Phase E scope : si gov ship que PDF+image, la whitelist
  suffit. Sinon, ajouter ZIP (pour DOCX/XLSX) avec sniffing du
  content zipé
- Documenter explicitement dans le kickoff check qu'uniquement
  PDF+image sont en scope Sprint 9

### R5 — Migration runner SHA256 re-hash au boot est lent pour gros fichiers

**Probabilité** : très faible. Les migrations font typiquement
< 10 KB. SHA256 d'un fichier 10 KB = ~10 μs.

**Mitigation** :

- Si un jour une migration fait > 1 MB, le runner log un
  warning et skip le re-hash (cache en-mémoire invalidé
  seulement au restart)
- Pas de fix immédiat nécessaire Sprint 9

### R6 — Gov app impose désormais 2 `AppDatabaseClient` (db_gov + db_app) et le refacto casse les tests existants

**Probabilité** : haute. Les 30 tests gov actuels consomment
`ctx.db` single.

**Mitigation** :

- `AppContext` ajoute `dbs: dict[str, AppDatabaseClient]`
  ET garde `db: AppDatabaseClient | None` via property
  backward-compat qui retourne `dbs.get("default")` — les
  tests existants continuent à marcher sur `ctx.db`
- Le gov app override `on_start` pour populer `dbs["gov"]` et
  `dbs["app"]`, le handler tab utilise `ctx.dbs["gov"]` explicite
- Audit Sprint 10 Phase 0 verra ce pattern et peut demander
  de retirer la property backward-compat (décision deferred)

### R7 — SSE endpoint `/app/{name}/events` fuite un subscriber si le client disconnect brutalement

**Probabilité** : moyenne. C'est un classic pattern SSE.

**Mitigation** :

- `test_events_sse_disconnect_unregisters_subscriber` — test
  explicite avec `httpx.AsyncClient.stream()` et abort
- Cleanup dans `finally:` de la génératrice SSE, pas dans
  `except`
- Heartbeat keep-alive toutes les 30 s pour détecter les
  clients morts

### R8 — Gov tab Documents avec `file_upload_block` v2 casse la rendering v1 shell si Sprint 10+ une app ancienne ressort

**Probabilité** : faible. Aucun consommateur v1-only prévu.

**Mitigation** :

- Test explicite `test_v2_file_upload_block_rejected_under_v1`
  qui lift une `ValidationError` structurée avec le nom du
  champ (`schema_version`) dans le message
- Doc `P15` dans `docs/shell/PATTERNS.md` : « si une app v1
  reçoit un v2 descriptor, le route handler retourne 422 et
  le shell render une erreur `UnsupportedSchemaVersion` »
- `AnyTabView.model_validate` est strict — pas de silent
  dropping

### R9 — Code splitting `createBrowserRouter` casse le Playwright global-setup

**Probabilité** : moyenne. Playwright global-setup fait
`page.goto('/my-projects')` et s'attend à des routes
statiques.

**Mitigation** :

- Le routeur v6 lazy ne change pas le comportement client-side
  — les routes restent les mêmes URLs. Le lazy est un détail
  d'impl interne
- Si une spec échoue, probablement parce qu'elle attend un
  chargement synchrone — ajouter un `await page.waitForSelector(
  '[data-testid="page-loaded"]')` sur les pages impactées
- Test en local Phase A AVANT commit

### R10 — `scripts/verify.sh` met >10 min à tourner (Playwright inclus)

**Probabilité** : haute. Playwright sur 27 specs + build npm
+ cargo test workspace = facile 8-10 min.

**Mitigation** :

- `./scripts/verify.sh` est le full run — les devs peuvent
  rouler des subsets pendant le dev et lancer verify avant
  le commit final
- Option `--quick` au script qui skip Playwright pour les
  iterations rapides (Phase A optional)
- CI full run obligatoire pour mergeable PRs (mais on n'a pas
  de CI GitHub Actions sur ce projet, donc c'est la
  responsabilité du dev local)

---

## 14. Checkpoint de clôture (Sprint 9 fermé si ...)

Le sprint est fermé (Phase F livrée, ready for Sprint 10 Phase
0 audit gate) **uniquement si les 10 conditions suivantes sont
toutes vraies** :

1. **Fail-fast checklist 38/38 verte** dans
   `sprint9_verification.md` (voir §10 ci-dessus)
2. **7 commits landed** sur master : 1 `docs(sprint9): kickoff
   + plan` + 5 feat A..E + 1 `docs(sprint9): verification +
   audit plan for Sprint 10`
3. **4 primitives D1..D4 livrées** avec consommateur réel dans
   `nexus-app-gov` : storage (filter persist), events (party
   refresh), migrations (gov 001_documents.sql), files (gov
   Documents tab)
4. **H-3 wheel drift CLOSED** dans `docs/rust/PATTERNS.md` + script
   `scripts/setup.sh` landed + testé sur fresh checkout
5. **T8, T10, T11, T12 CLOSED** dans `docs/shell/PATTERNS.md`
   avec SHA Phase A
6. **Sprint 7 E-1, C-4, D-3 CLOSED** dans `docs/rust/PATTERNS.md`
   avec SHA Phase E
7. **Main bundle budget** : main ≤ 350 KB OR ≤ 425 KB avec
   rationale commit body + plan Phase B tree-shake. Pas au-dessus
8. **`.planning/sprint9_verification.md`** livré (self-report)
9. **`.planning/sprint9_audit_plan.md`** livré (10 tracks A..J)
10. **Memory `nexus_grid_pivot.md` mise à jour** avec Sprint 9
    CLOSED tip, compteurs de tests finaux, transition Sprint 10
    scope

**Si une condition manque** : pas de `docs(sprint9): verification
+ audit plan for Sprint 10` — le sprint reste ouvert jusqu'à
résolution.

**Si toutes les conditions sont vraies** : Sprint 9 est CLOSED.
Sprint 10 peut ouvrir sa Phase 0 audit gate dans une session
fraîche qui jouera `.planning/sprint9_audit_plan.md`.
