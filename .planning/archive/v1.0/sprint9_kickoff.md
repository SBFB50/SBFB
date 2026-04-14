# Sprint 9 — Kickoff (SDK infra primitives + audit leftovers)

**Écrit** : 2026-04-12, en ouverture de Sprint 9 après
fermeture de l'audit gate Sprint 8 (verdict CONDITIONAL PASS
levé par 2 commits fix landed sur master).

**Tip master d'entrée** : `c50157d` (post `docs(sprint8):
audit findings from Sprint 9 Phase 0 gate`).

---

## 1. Constat d'entrée

### 1.1 État du repo

- Sprints 0–8 **CLOSED**. Sprint 8 Phase 0 audit gate (session
  fraîche qui a joué `sprint8_audit_plan.md`) a produit
  `sprint8_audit_findings.md` avec verdict **CONDITIONAL PASS**
  (0 P0, 2 P1, 5 P2, 13 P3).
- Les 2 P1 sont **fixés** sur master :
  - `14c199f fix(sprint8): enforce read-only on AppDatabaseClient`
    (D-FX-1) — `read_only: bool = True` ajouté au ctor,
    connexion via `file:<path>?mode=ro` URI, short-circuit
    `DatabaseError` au début de `execute()` en
    defense-in-depth, coordinator loader passe `read_only=True`
    pour la DB legacy `nexus/gov/govdata.db`. 3 nouveaux tests
    `test_db.py`.
  - `da69a8b fix(sprint8): correct verification row 11 false
    claims` (V-FX-1) — `.planning/sprint8_verification.md` row
    11 ré-écrite pour refléter la réalité post-fix (plus de
    `asyncio.Lock` mensonge, plus de `schema_introspection`
    inventé).
- Les 5 P2 sont **loggés en tech debt** :
  - `a8b4d50 docs(sprint8): tech debt T11/T12 + H-3 promotion
    from Sprint 9 audit gate` — `docs/shell/PATTERNS.md`
    T8..T12 documentées, `docs/rust/PATTERNS.md` H-3 promue
    de P3 à **P1 Sprint 9**.
- `c50157d docs(sprint8): audit findings from Sprint 9 Phase 0
  gate` — le findings doc committed officiel, clôt la gate.
- Les 13 P3 sont laissés tels quels (rouverts par Sprint 9
  polish en Phase E si le budget le permet, sinon reportés).

### 1.2 Compteurs de tests à l'entrée (tip `c50157d`)

| Suite | Count | Baseline Sprint 8 entrée | Delta Sprint 8 |
|---|---|---|---|
| Rust workspace | **309** | 304 | +5 (P2 hygiene Sprint 7) |
| Python SDK | **71** | 40 | +31 (commands/db/AppContext + 3 read-only post-fix) |
| Python coordinator | **63** + 1 skipped | 57 + 1 | +6 net |
| Python app-gov | **30** | 3 | +27 (19 tabs + 4 commands + 4 routing) |
| Vitest unit | **142** | 114 | +28 |
| Playwright | **24** | 13 | +11 |
| size-limit | **4/4 green** | 4/4 | main 474.5 / 475 (**0.5 kB headroom — T10/T9 à traiter**) |
| `npm audit` | 0 high/crit | 0 | — |

Tout vert. Le seul signal rouge structural est la headroom
main bundle **0.5 kB** qui bloque tout premier commit React
Sprint 9 si D6 n'est pas jouée en Phase A (cf. §4).

### 1.3 Tech debt ouverte à l'entrée

**Sprint 7 tech debt (ouverte, non touchée Sprint 8)** :

- **E-1** `probe_reachable` 2s timeout
  (`crates/nexus-core-rs/src/discovery.rs`, `DEFAULT_PROBE_TIMEOUT`)
- **C-4** gossip backpressure sans `tokio::sync::Semaphore`
  (`crates/nexus-shell-daemon-core/src/iroh_runtime.rs`)
- **D-3** subscriptions persist ordering insert-before-persist
  (`CuratorRuntime::subscribe`)
- **H-3** `nexus_core` wheel editable install drift — **P1
  promu par audit gate Sprint 8**, bloque sessions fresh

**Sprint 8 audit Sprint 9 tech debt nouvelle (T8..T12)** :

- **T8** (CardTitle a11y, promoted from Sprint 7 F-3) — shadcn
  vendored `CardTitle` est un `<div>`, pas un heading
- **T9** main bundle 0.5 KB headroom — bloquant structurel
- **T10** coordinator `httpx.AsyncClient` per-call sans
  `Limits` (Sprint 7 G-1)
- **T11** `CommandPalette.runAppCommand` swallows errors via
  `console.error` (Sprint 8 C-FX-1)
- **T12** `NexusApp.commands()` ordering depends on `dir(cls)`
  alphabetical (Sprint 8 C-FX-2)

### 1.4 Surprise constatée à la lecture de `web/vite.config.ts`

Le fichier porte déjà un `rolldownOptions.output.manualChunks`
avec 5 buckets : `vendor-react`, `vendor-graph` (G6/sigma/graphology/
reagraph/react-force-graph), `vendor-charts` (recharts/nivo),
`vendor-map` (leaflet/react-leaflet), `vendor-ui` (@radix-ui).
**Trois de ces buckets référencent des packages qui ont été
retirés en Sprint 5 Phase 0** (`refactor(web): drop legacy
cold-case UI`, -24 top-level deps, -17 747 LOC).

Donc `vendor-graph`, `vendor-charts`, `vendor-map` sont des
**chunks morts** — le build ne matche aucun ID, les chunks
sortent vides ou n'existent même pas (Rolldown skip les
chunks vides). C'est bénin mais confusant et ça dérive de
l'intention. **Phase A nettoie ça en même temps que D6.**

---

## 2. Goal en une phrase

Livrer les **4 primitives SDK infra déférées par Sprint 8 D5**
(`AppContext.storage`, `AppContext.events`, file upload + CAS,
DB migration runner) avec **un vrai consommateur par primitive
dans nexus-app-gov**, débloquer les deux blockers structurels
(**H-3 wheel drift** et **T9/T10 main bundle headroom**), et
nettoyer les 6 items de tech debt Sprint 7 encore ouverts + les
5 P2 Sprint 8.

Ce n'est **pas** un sprint branding ni release prep — ces items
sont glissés vers Sprint 10+.

---

## 3. Phase 0 — Audit gate de Sprint 8 (DONE)

La Phase 0 de Sprint 9 a été jouée dans une session Claude
Code fraîche (sans historique Sprint 8) qui a lu
`.planning/sprint8_audit_plan.md` comme feuille de route. Le
findings doc `.planning/sprint8_audit_findings.md` est landed
sur master via `c50157d`.

**Verdict** : CONDITIONAL PASS → LEVÉ.

**Commits de gate** (ordre chronologique) :

```
c50157d docs(sprint8): audit findings from Sprint 9 Phase 0 gate
a8b4d50 docs(sprint8): tech debt T11/T12 + H-3 promotion from Sprint 9 audit gate
da69a8b fix(sprint8): correct verification.md row 11 false claims
14c199f fix(sprint8): enforce read-only on AppDatabaseClient
449f404 docs(sprint8): verification + audit plan for Sprint 9
```

Phase 0 est **fermée**. Aucun commit Sprint 9 Phase A ne peut
démarrer avant que ce kickoff + son plan soient landed
(`docs(sprint9): kickoff + plan`, commit unique couvrant les
deux fichiers). À partir de Phase A, tous les commits feat
atterrissent sur un tip qui contient déjà les fix + findings.

---

## 4. Décisions Day 0 (D1..D6 gelées, non rebattables)

Les 6 décisions suivantes ont été arbitrées le 2026-04-12
après recherche documentaire (3 agents parallèles + Context7
MCP fetches sur pydantic-settings, anyio, FastAPI UploadFile,
Pydantic discriminated unions, Zod, Alembic, Vite, React
Router v6, react.dev sunsetting-CRA). Les références OSS
supports sont capturées dans `.planning/sprint9_plan.md` §3
Research.

Elles **ne seront pas rebattues pendant Sprint 9**. Une
objection technique nouvelle qui émergerait en cours de sprint
se note comme item « à rouvrir Sprint 10 Day 0 » sans
interrompre les phases en cours.

### D1 — `AppContext.storage` : JSON file KV + typed namespaces

**Retenu** :

- Fichier `projects/<p>/apps/<a>/storage.json` (créé lazy au
  premier set, layout validé par un header `{"schema": 1,
  "payload": {...}}`)
- Chargé en mémoire au boot de l'app
- Persist via **atomic rename** (`tmpfile + os.replace`) —
  pattern canonique (diskcache, TinyDB, pydantic-settings)
- **Write coalescing** via `asyncio.call_later(500ms, flush)`
  si un write est en attente — pattern TinyDB `CachingMiddleware`
- **`asyncio.Lock` par app** pour protéger la re-entry entre
  handlers concurrents (pas de race corruption sur un fichier
  shared)
- **Flush-on-shutdown** via FastAPI `lifespan` context (le
  coordinator cancel la task de flush différée et écrit
  synchrone avant shutdown)
- **API non-typée** : `await ctx.storage.get(key)` /
  `set(key, value)` / `delete(key)` / `keys(prefix=None)` /
  `clear()`, values JSON-serializable
- **API typée optionnelle** : `ctx.storage.namespace("filters",
  PoliticiansFilter)` retourne un `TypedNamespace[Schema]` qui
  wrappe `Schema.model_validate()` au get et
  `Schema.model_dump()` au set, raise `StorageSchemaError` sur
  mismatch
- Scope : **per-app, per-project**. Une app ne peut pas lire le
  storage d'une autre app ni d'un autre projet

**Rejeté** :

- SQLite (redondant avec `ctx.db`, overkill pour un état UI
  léger)
- iroh-docs (trop lourd, l'état est strictement local au
  coordinator)
- Redis / KV externe (dépendance, ops layer)
- File locking inter-process (`fcntl`/`msvcrt`) — pas utile car
  coordinator D1 Sprint 7 = singleton strict single-process

**Implications code** :

- Nouveau module `packages/nexus-sdk/src/nexus_sdk/storage.py`
  (~280 LOC)
- `AppContext.storage: AppStorage | None` wiré par le
  coordinator loader après `on_start`
- `Coordinator.start()` step nouveau : instancier `AppStorage(
  storage_path)` avant `on_start(ctx)`
- `Coordinator.stop()` : drainer `AppStorage.flush_on_shutdown()`
  pour chaque app
- Consommateur gov : tab Politiciens filter state
  (`chamber`, `date_range`, `search`) persisté via
  `ctx.storage.namespace("filters.politicians", FilterSchema)`,
  testé Playwright (set filter → reload → filter conservé)

### D2 — `AppContext.events` : anyio memory streams + fnmatch glob + context manager

**Retenu** :

- Wrapper autour de **`anyio.create_memory_object_stream[
  EventEnvelope](max_buffer_size=1024)`** — pas d'`asyncio.Queue`
  maison
- `EventEnvelope` Pydantic frozen : `{topic: str, payload: dict,
  timestamp: datetime, trace_id: str}`. `trace_id` généré via
  `uuid4().hex[:16]`, `timestamp` = UTC ISO 8601 au moment du
  `publish()`
- `await ctx.events.publish(topic, payload)` — fire-and-forget,
  dispatch aux subscribers matching synchrone (pas de await
  dans la boucle de dispatch)
- `async with ctx.events.subscribe(pattern) as stream: async for
  envelope in stream:` — context manager qui register/unregister
  le `MemoryObjectSendStream`. **Pas de weak refs** (anti-pattern
  pour coroutines async)
- **Topic glob matching** via `fnmatch.fnmatch` (stdlib) —
  patterns shell-style `politician.*` / `*.refreshed` /
  `file.upload.**` (attention : `fnmatch` n'interprète pas
  `**`, on utilise `*` sur un seul segment, la doc le précise)
- **Overflow policy** configurable (`drop_oldest` default,
  `drop_newest`, `block`) via `send_nowait` + `catch WouldBlock` :
  - `drop_oldest` : drain un item du `ReceiveStream` interne,
    retry `send_nowait` — log un `warning` au premier drop par
    subscriber (avec throttle 1/min)
  - `drop_newest` : skip le publish, log warning throttled
  - `block` : convertit en `await send()` — **à éviter** parce
    qu'un subscriber lent bloque le bus entier, uniquement pour
    des cas test
- Scope : **per-app, in-process only**, no persistence, no
  cross-app, no cross-node. Events perdus au restart coordinator.
  Cross-app / cross-node explicite **P1 Sprint 10+**

**Rejeté** :

- `asyncio.Queue` maison avec weak refs (re-inventé la roue,
  weak refs sur coroutines posent des problèmes de GC)
- iroh-gossip (trop lourd, cross-node hors scope v1)
- MQTT topic hierarchy `+`/`#` (plus expressif mais dépendance
  + parseur, glob `fnmatch` suffit)
- blinker (sync seulement, pas async-friendly)
- aiopubsub (peu maintenu)

**Implications code** :

- Nouveau module `packages/nexus-sdk/src/nexus_sdk/events.py`
  (~320 LOC)
- `nexus-sdk/pyproject.toml` : ajouter `anyio >= 4.0` en dep
  (déjà transitif via FastAPI mais on l'épingle directement)
- `AppContext.events: AppEvents | None` wiré par le coordinator
  loader
- Consommateur gov : nouveau worker `gov.refresh_party_cache`
  qui publish `party.refreshed` sur `ctx.events` après avoir
  re-fetché le party data, tab Politiciens subscribe au topic
  et re-fetch sa query React Query. Testé Playwright e2e
  (trigger worker via palette → attendre event → grid mis à
  jour sans reload manuel)

### D3 — File upload + CAS + TabView schema v2

**Retenu — endpoint + CAS** :

- Endpoint `POST /app/{name}/files/upload` multipart, avec
  `max_part_size=50 * 1024 * 1024` passé explicitement (défaut
  FastAPI/multipart est **1 MB** — piège documenté)
- **Chunked read** obligatoire : `while chunk := await
  file.read(8192)`, SHA256 incrémental via
  `hashlib.sha256().update(chunk)`, écriture streamée vers un
  tmpfile, puis **rename atomique** vers
  `projects/<p>/apps/<a>/uploads/<sha256[:2]>/<sha256>`
- **Sharding `sha256[:2]/<sha256>`** — pattern canonique
  (git, Restic), évite les limites FS sur gros volumes
- **Dédup pre-write** : si `sha_path.exists()` avant rename,
  skip l'écriture, retourner le handle existant (pattern
  Restic)
- **Manifest JSON adjacent** `<sha256>.json` avec `{sha256,
  size, content_type, original_name, uploaded_at, uploaded_by,
  app_name}` — plus riche que symlink HuggingFace, plus léger
  qu'une DB
- **Validation content-type multi-couches** :
  1. Header multipart `file.content_type` consulté pour log
     (client-controlled, pas fiable seul)
  2. **Magic bytes validation post-write** : lire les ~16
     premiers octets du fichier écrit, matcher contre une
     whitelist hardcoded (PNG `89 50 4e 47`, JPEG `ff d8 ff`,
     WEBP `52 49 46 46 ... 57 45 42 50`, PDF `25 50 44 46 2d`,
     SVG `3c 3f 78 6d 6c` ou `3c 73 76 67`). Si mismatch :
     delete le fichier + raise 415 Unsupported Media Type
- **Allowlist par app** via décorateur class-level
  `@nexus_app_files(accept=["image/*", "application/pdf"])` sur
  la classe `NexusApp`. Si l'app n'a pas le décorateur → upload
  route retourne 404 (« app does not accept file uploads »)
- SDK helper `ctx.files`:
  - `await ctx.files.store(upload: UploadFile) -> FileHandle`
  - `async def open(sha256: str) -> AsyncIterator[bytes]`
  - `await ctx.files.manifest(sha256) -> FileManifest | None`
  - `await ctx.files.delete(sha256)` (soft delete = remove
    manifest only, fichier CAS reste pour dédup)

**Retenu — TabView schema v2 bump** :

- `schema_version: Literal[1, 2]` avec
  `Annotated[Union[TabViewV1, TabViewV2], Field(
  discriminator="schema_version")]` côté Pydantic 2
- Zod mirror `z.discriminatedUnion("schema_version",
  [TabViewV1Schema, TabViewV2Schema])`
- **`extra="forbid"` préservé absolument** sur les deux
  versions — c'est le seul mécanisme qui garantit qu'un shell
  v1 qui reçoit un descriptor v2 rejette avec une erreur
  structurée au lieu de dropper silencieusement. Anti-pattern
  explicite : « ne pas retirer `extra="forbid"` pour simplifier
  la compat »
- **Forward compat** : un descriptor v1 doit toujours valider
  sous v2 schema (testé par `test_v1_descriptor_validates_under_v2`)
- **Backward compat** : un shell v1 qui reçoit un v2 descriptor
  avec `file_upload_block` doit lift `UnsupportedBlockKindError`
  avec `schema_version` cité dans le message, pas crasher ni
  silencieusement dropper le block. L'auditeur Sprint 10 Phase 0
  le verifiera explicitement
- **Cross-lang fixture v2** : `packages/nexus-sdk/tests/snapshots/
  tabview_v2_canonical.json` shared, lue par Python
  `test_view.py::test_v2_canonical_roundtrip` ET Vitest
  `schema.test.ts::v2_cross_lang` (pattern Sprint 7 A-3, déjà
  en place pour curator)
- Nouveau block kind `file_upload_block` uniquement valide sous
  `schema_version >= 2`, porte `{kind, upload_endpoint, accept,
  max_size_bytes, label?, help?, target_storage_key?}`. Le
  `upload_endpoint` est `/app/{name}/files/upload` rendu relatif
  par le backend
- Renderer React `<FileUploadBlock>` en nouveau component avec
  drag-and-drop (HTML5 native, pas de dep `react-dropzone`),
  progress bar **connectée aux events D2** via SSE
  `/app/{name}/events?pattern=file.upload.progress` (consommateur
  réel de D2), preview thumbnail pour les images

**Rejeté** :

- `await file.read()` d'un coup (charge 50 MB en RAM —
  memory exhaustion)
- Streaming chunked sans cap (pas borné = DoS vector)
- Cloud storage / S3 (off-scope v1)
- Blob en DB (bad shape, ne bénéficie pas du dédup CAS)
- Lib `python-magic` (FFI libmagic, complique l'install Windows
  — on hardcode 5 magic numbers à la main)
- `react-dropzone` / `filepond` / `uppy` (dépendance lourde
  pour drag-and-drop que HTML5 natif fait en 30 lignes)

**Implications code** :

- Nouveau module `packages/nexus-sdk/src/nexus_sdk/files.py`
  (~340 LOC)
- Nouveau router `packages/nexus-coordinator/src/
  nexus_coordinator/api/files.py` (~220 LOC)
- `AppContext.files: AppFileStore | None` wiré par le
  coordinator loader
- `packages/nexus-sdk/src/nexus_sdk/view.py` : ajouter
  `TabViewV1` + `TabViewV2` + `AnyTabView` discriminated union,
  ajouter `file_upload_block()` constructor helper, bump
  snapshot `tabview_schema.json` → `tabview_v1_schema.json` +
  nouveau `tabview_v2_schema.json`
- `packages/nexus-coordinator/src/nexus_coordinator/api/
  apps.py` : `TabView.model_validate` devient
  `AnyTabView.model_validate` pour consommer les deux versions
- `web/src/components/app/tabview/schema.ts` : Zod union
  discriminée sur `schema_version`
- `web/src/components/app/tabview/blocks/FileUploadBlock.tsx`
  (~180 LOC nouveau)
- Consommateur gov : nouveau tab **« Documents »** (20e tab
  qui porte le compteur Sprint 8 de 19 → 20), liste les PDFs
  uploadés via FileUploadBlock, stocke le manifest dans une
  per-app SQLite créée par la migration runner D4 (chaîne
  technique volontaire : files ↔ events ↔ migrations)

### D4 — DB migration runner : SQL files lexico + SHA256 tamper detection + BEGIN IMMEDIATE

**Retenu** :

- Scan `app_root / "migrations" / "NNN_<slug>.sql"` par ordre
  **lexicographique** du nom de fichier. Convention : `001_init.sql`,
  `002_add_documents.sql`. Version = int extrait du préfixe
- Chaque migration appliquée dans une transaction
  **`BEGIN IMMEDIATE`** (pas `BEGIN EXCLUSIVE` — suffisant
  pour notre singleton coord D1 Sprint 7). Rollback auto si
  une statement fail dans le fichier
- **SHA256 content tracking** : calcul `sha256(file_bytes)` au
  moment de l'apply, stockage dans `_nexus_migrations.sha256`.
  **À chaque boot**, re-vérification : pour chaque row de
  `_nexus_migrations`, re-hash le fichier local et compare.
  Si divergence → `MigrationTamperedError` avec message
  explicite pointant le fichier et les deux hashes.
  Référence : **Sqitch** (https://sqitch.org/) qui fait le
  même pattern avec SHA1
- Table de tracking : `_nexus_migrations(version INT PRIMARY
  KEY, slug TEXT NOT NULL, sha256 TEXT NOT NULL,
  applied_at TEXT NOT NULL)`. Créée lazy par le runner lui-même
  si absente (ORDER BY version = l'ordre d'application, plus
  simple qu'Alembic qui supporte des branches)
- **Pas de downgrade** — le runner est forward-only. Rollback
  = `git revert` + nouvelle migration qui undo. Pattern
  datasette / yoyo simplifié
- **CLI** `nexus-coordinator migrate --project <name>
  [--app <name>] [--plan | --apply]` :
  - `--plan` : dry-run, liste les migrations pendantes et les
    SHA attendues sans rien écrire
  - `--apply` (défaut) : applique les pendantes
  - Sans `--app` : boucle sur toutes les apps du projet qui
    déclarent `manifest.migrations_dir`
- **Opt-in par app** via `manifest.migrations_dir: Path | None`
  (default `None` = runner skip l'app)
- **Pas de CLI `repair`** — anti-pattern Flyway. Seule action
  sur drift = `MigrationTamperedError` bloquant qui exige une
  intervention humaine consciente (édition manuelle de la row
  `_nexus_migrations` si l'intention est de re-apply)
- **Runner invoqué automatiquement par `Coordinator.start()`**
  APRÈS `on_start` de chaque app mais AVANT le premier tick du
  dispatcher — bloquant au boot, fail-fast si tamper détecté
- Writable `AppDatabaseClient` (`read_only=False`) obligatoire
  pour que le runner puisse écrire — l'app qui veut ship des
  migrations crée son propre `AppDatabaseClient` dans
  `on_start(ctx)` avec `read_only=False` pointé sur une per-app
  DB writable, distincte de la DB legacy read-only éventuelle.
  Pattern gov : `ctx.db_gov = AppDatabaseClient(legacy_path,
  read_only=True)` + `ctx.db_app = AppDatabaseClient(per_app_path,
  read_only=False)`, le runner tourne sur `ctx.db_app`

**Rejeté** :

- Alembic (ORM SQLAlchemy requis, branches + downgrades
  overkill)
- Flyway mode `repair` (trou de sécurité sur tamper
  detection)
- Yoyo-migrations (sync only, pas d'aiosqlite)
- SHA1 (Sqitch l'utilise mais SHA256 est préférable en 2025,
  pas de surcoût perf notable)
- `PRAGMA locking_mode=EXCLUSIVE` (lock permanent trop
  agressif)
- Downgrade bi-directionnel (complexité sans bénéfice pour
  notre cas forward-only single-writer)

**Implications code** :

- Nouveau module `packages/nexus-sdk/src/nexus_sdk/migrations.py`
  (~240 LOC)
- `AppManifest.migrations_dir: Path | None = None` ajouté au
  modèle Pydantic
- `Coordinator.start()` nouveau step entre `_loader_apps()` et
  `on_start()` : scan + apply migrations. Log INFO count
  appliqué, log ERROR + halt sur tamper
- `nexus-coordinator` CLI ajoute un sous-commande `migrate`
  (Typer)
- Consommateur gov : nouveau fichier
  `packages/nexus-app-gov/src/nexus_app_gov/migrations/
  001_documents.sql` qui crée `gov_documents(sha256 TEXT
  PRIMARY KEY, politician_id INT NULL, uploaded_at TEXT NOT
  NULL, title TEXT)` dans la per-app SQLite writable. Le tab
  Documents lit cette table via `ctx.db_app.fetchall()`

### D5 — H-3 wheel drift : `scripts/setup.sh` + `scripts/verify.sh` + git hook opt-in

**Retenu** :

- `scripts/setup.sh` (bash POSIX, tourne sur Git Bash Windows,
  Linux, macOS) :
  1. Assert `.venv/` existe (sinon error + hint `uv sync`)
  2. `unset CONDA_PREFIX CONDA_DEFAULT_ENV` (conflit miniconda
     base classic)
  3. Hash de `Cargo.lock` + `crates/nexus-core-rs/src/**` +
     `crates/nexus-core-py/src/**` via `sha256sum | head -c 16`
  4. Compare au hash stocké `.venv/.nexus-core-hash`
  5. Si divergence OU si `$PWD/.venv/lib/python*/site-packages/
     nexus_core*.so` absent : `VIRTUAL_ENV=$PWD/.venv maturin
     develop --release --manifest-path
     crates/nexus-core-py/Cargo.toml`
  6. Écrit le nouveau hash
  7. Message de succès ou skip

- `scripts/verify.sh` (bash POSIX) roule la full fail-fast
  suite en ordre fixé :
  1. `cargo fmt --all --check`
  2. `cargo clippy --workspace --all-targets --locked
     -- -D warnings`
  3. `cargo test --workspace --locked`
  4. `uv run ruff format --check packages/ examples/`
  5. `uv run ruff check packages/ examples/`
  6. `uv run pytest packages/nexus-sdk/tests/ -q`
  7. `uv run pytest packages/nexus-coordinator/tests/ -q`
  8. `uv run pytest packages/nexus-app-gov/tests/ -q`
  9. `cd web && npx tsc --noEmit -p tsconfig.app.json`
  10. `cd web && npm run lint`
  11. `cd web && npm run test:unit`
  12. `cd web && npm run test:coverage`
  13. `cd web && npm run build`
  14. `cd web && npm run size`
  15. `cd web && npx playwright test`
  16. `cd web && bash scripts/scan-en-strings.sh`
  17. `cd web && npm audit --audit-level=high`
  Fail fast sur premier exit non-zero, retourne le step qui a
  fail. Les devs peuvent commenter des steps locaux pour
  shortcut mais CI doit tout rouler

- **Git hook `.githooks/post-merge`** (opt-in via
  `git config core.hooksPath .githooks`) : diff le hash stocké
  vs l'état actuel et rappelle de rouler `./scripts/setup.sh`
  si nécessaire. **Opt-in** — on ne force pas un hook qui
  modifie l'env des devs

- **Doc** : `docs/claude/README.md` §4.3 réécrite pour pointer
  vers `./scripts/verify.sh` au lieu du bloc bash long. README
  racine mise à jour avec une section « Quick start »

- **PATTERNS update** : `docs/rust/PATTERNS.md` H-3 passe à
  CLOSED Sprint 9 Phase A avec SHA commit

**Rejeté** :

- Vendor le wheel dans le repo (plate-forme-specific, git LFS
  requis, dérive silencieuse)
- Hook `pre-commit` (bloque les commits légitimes, annoyant
  pour les devs)
- `nox` / `tox` (dépendance lourde pour 2 scripts bash)
- `cargo xtask` (pattern Rust, mais on a aussi du Python et
  npm — cross-écosystème, bash est le plus portable)

**Implications code** :

- Nouveau `scripts/setup.sh` (~90 lignes bash)
- Nouveau `scripts/verify.sh` (~80 lignes bash)
- Nouveau `.githooks/post-merge` (~25 lignes bash)
- Edit `docs/claude/README.md` §4.3
- Edit `docs/rust/PATTERNS.md` H-3 (CLOSED)
- Edit `README.md` racine (section Quick Start)

### D6 — Bundle code splitting : createBrowserRouter + manualChunks + dead chunks cleanup

**Retenu** :

- **Migration App.tsx** : `<BrowserRouter>` + `<Routes>`
  declarative → `createBrowserRouter([{...}])` avec propriété
  `lazy: () => import('./pages/Foo')` sur chaque route enfant.
  Le comment actuel dans `App.tsx` « Routing stays declarative
  because every page fetches its data through React Query
  directly » est **contredit par la recherche** : le pattern
  canonique React Router 2025 (`react.dev` sunsetting-CRA
  article 2025-02-14) est `createBrowserRouter` + `lazy`, et
  React Query reste utilisable identiquement. La raison donnée
  dans le comment n'est plus pertinente
- Dans chaque page lazifiée, exporter `Component` (pas
  `default`) — pattern React Router v6.9+ natif. Optionnellement
  exporter `loader`, `ErrorBoundary`
- **AppShell reste l'`element` du parent route** — ne jamais
  mettre `Suspense` autour de `<AppShell />`, sinon la sidebar
  et la palette disparaissent pendant la navigation. Le fallback
  de chargement de route vit dans l'outlet, pas au layout niveau
- **Manual chunks** via `rolldownOptions.output.manualChunks(id)`
  avec :
  - Guard `if (!id.includes('node_modules')) return;` en tête
    (évite qu'un fichier `src/` atterrisse dans un chunk vendor,
    pattern lobe-chat)
  - **`vendor-react`** : `react`, `react-dom`, `react-router-dom`
  - **`vendor-ui`** : `@radix-ui/*` (Radix primitives uniquement —
    shadcn lit son propre code depuis `src/components/ui/` donc
    hors scope ; ce n'est pas changé vs l'actuel)
  - **`vendor-query`** (nouveau) : `@tanstack/react-query`,
    `@tanstack/react-query-devtools`, `zustand`
  - **Chunk features** : `tabview` (`src/components/app/tabview/
    **`), `palette` (`src/components/command-palette/**`),
    `upload` (`src/components/app/tabview/blocks/FileUploadBlock*`)
  - **Retrait** des chunks morts `vendor-graph`, `vendor-charts`,
    `vendor-map` (héritage des deps pre-pivot supprimées Sprint 5)
- **Budgets `web/.size-limit.json`** augmentés / réduits :
  - `main` : **475 KB → 350 KB** (cible ambitieuse post code
    splitting, vérifiable via `npm run size` avant de commiter
    Phase A)
  - `vendor-react` : **210 KB → 170 KB** (l'observé actuel est
    189 KB)
  - `vendor-ui` : **50 KB → 40 KB**
  - Nouveau : `vendor-query` ≤ 50 KB
  - Nouveau : `tabview` ≤ 80 KB
  - Nouveau : `palette` ≤ 40 KB
  - Nouveau : `upload` ≤ 30 KB
  - `css` : **100 KB → 100 KB** (inchangé)
- **Si Phase A échoue à passer main sous 350 KB**, un commit
  `fix(sprint9): relax main budget to 425 KB pending tree-shake
  pass` est acceptable mais doit être immédiatement suivi d'un
  effort de tree-shaking (audit `rollup-plugin-visualizer` via
  `ANALYZE_MODE=true`). **Pas de raise silencieuse sans
  rationale**
- **Tooling** : `rollup-plugin-visualizer` ajouté en devDep,
  activé conditionnellement via `process.env.ANALYZE_MODE ===
  'true'` (pattern AppFlowy-Web), génère un `dist/stats.html`
  inspectable

**Rejeté** :

- Raise silencieuse du budget main de 475 → 525 (re-introduit le
  problème Sprint 10)
- `React.lazy` + `<Suspense>` manuel (ancien pattern, la v6.9+
  préfère la propriété `lazy` directement sur la route)
- Chunker shadcn `src/components/ui/` dans `vendor-ui` (shadcn
  est copié dans `src/`, pas une lib npm — les deux guards de
  lobe-chat le clarifient)
- Externaliser React en CDN (pattern Excalidraw) — trop intrusif
  pour un shell local-only sans bénéfice bandwidth (on shippe le
  shell sur le coordinator loopback, pas un CDN global)
- Migration Rolldown `advancedChunks` (Rolldown encore preview,
  la syntaxe `manualChunks` tient — **tech debt Sprint 10+**
  quand Rolldown devient stable)

**Implications code** :

- Refacto `web/src/App.tsx` : `createBrowserRouter` + `lazy`
- Edit `web/vite.config.ts` : rewrite `manualChunks` avec
  guards + nouveaux chunks + retrait des chunks morts
- Edit `web/.size-limit.json` : 4 budgets → 8 budgets ajustés
- Edit `web/src/pages/*.tsx` : ajouter `export const Component
  = ...` (ou re-export) pour le pattern lazy natif
- Nouveau `web/package.json` devDep : `rollup-plugin-visualizer
  ^5`
- Note dans `docs/shell/PATTERNS.md` : nouveau pattern **P12 —
  Code splitting via createBrowserRouter lazy + manualChunks**
  qui documente la convention, les chunks, les guards

---

## 5. Plan Phase outline A..F

Une phase = un commit atomique `feat(scope): Sprint 9 Phase X
— titre`. Scope tags anticipés :

### Phase A — Unblock: setup/verify scripts + code splitting + Sprint 7/8 cleanup

**Scope** : D5 + D6 + nettoyage tech debt Sprint 7/8 qui peut
landed sans les primitives D1..D4.

- `scripts/setup.sh` + `scripts/verify.sh` + `.githooks/
  post-merge`
- `docs/claude/README.md` §4.3 réécrite
- `docs/rust/PATTERNS.md` H-3 → CLOSED
- `web/src/App.tsx` migration `createBrowserRouter` + `lazy`
- `web/vite.config.ts` rewrite `manualChunks` avec guards,
  retrait chunks morts, ajout feature chunks
- `web/.size-limit.json` 8 budgets recalibrés
- Retrait des chunks vides `vendor-graph`, `vendor-charts`,
  `vendor-map` de la config et éventuelle mise à jour de
  tests Playwright
- **T11 fix** : `CommandPalette.runAppCommand` toast inline
  (réutilise le pattern Sprint 8 `ButtonBlock` — pas de
  nouvelle dep sonner/react-hot-toast)
- **T12 fix** : `NexusApp.commands()` + `workers()` + `tabs()`
  → `sorted(..., key=lambda d: d.name)` explicite, test
  `test_list_app_commands_ordered` renforcé (assertion sur le
  sort key explicite au lieu du side effect de `dir(cls)`)
- **T8 fix** (CardTitle a11y) : edit `web/src/components/ui/
  card.tsx` pour rendre `CardTitle` en `<h3>` au lieu de
  `<div>` (option a, diff clair, audit-friendly)
- **T10 fix** (coord httpx Limits) : module-level singleton
  `httpx.AsyncClient(limits=httpx.Limits(max_connections=10))`
  managé par FastAPI `lifespan`

**Critère** : fail-fast suite complète verte AVEC nouveau
budget main ≤ 350 KB OU un `fix(sprint9): relax main budget`
explicite mais toujours < 425 KB. H-3 CLOSED. T8/T10/T11/T12
CLOSED.

**Commit cible** : `feat(web,sdk,coordinator,scripts): Sprint
9 Phase A — setup/verify scripts + createBrowserRouter code
splitting + Sprint 7/8 P2 cleanup (H-3 + T8 + T10 + T11 +
T12 CLOSED)`

### Phase B — `AppContext.storage` + gov Politiciens filter consumer

**Scope** : D1 primitive + consommateur gov réel.

- `packages/nexus-sdk/src/nexus_sdk/storage.py` (~280 LOC)
- `AppContext.storage: AppStorage | None` field
- `Coordinator.start()` instancie `AppStorage(storage_path)`
  avant `on_start(ctx)`
- `Coordinator.stop()` drain flush
- `packages/nexus-sdk/tests/test_storage.py` (~20 tests :
  get/set/delete/keys/clear, atomic rename, coalescing flush,
  typed namespace validation, lock re-entry, empty state,
  missing file)
- `packages/nexus-app-gov/src/nexus_app_gov/filters.py` —
  `PoliticiansFilter` Pydantic model
- `packages/nexus-app-gov/src/nexus_app_gov/app.py` : tab
  Politiciens handler lit / écrit `ctx.storage.namespace(
  "filters.politicians", PoliticiansFilter)` au render
- `web/src/pages/ProjectDetail.tsx` ou équivalent : UI pour
  set filter (chamber dropdown, date range, search input)
  qui persist
- Playwright spec `gov-politicians-filter-persist.spec.ts` :
  set filter → reload → filter conservé

**Critère** : full suite verte, +20 tests SDK, +1 Playwright,
+3 tests gov, D6 budgets tiennent toujours.

**Commit cible** : `feat(sdk,coordinator,app-gov,web): Sprint
9 Phase B — AppContext.storage + typed namespaces + gov
Politiciens filter persist consumer`

### Phase C — `AppContext.events` + gov party.refreshed consumer

**Scope** : D2 primitive + consommateur gov réel.

- `packages/nexus-sdk/src/nexus_sdk/events.py` (~320 LOC)
- `packages/nexus-sdk/pyproject.toml` : pin `anyio >= 4.0`
- `AppContext.events: AppEvents | None` field
- `Coordinator.start()` instancie `AppEvents()` par app
- `packages/nexus-sdk/tests/test_events.py` (~25 tests :
  publish/subscribe basic, glob matching, bounded queue
  overflow drop_oldest, drop_newest, block mode, context
  manager unsubscribe, envelope fields, weak cleanup check,
  multi-subscriber fan-out)
- `packages/nexus-app-gov/src/nexus_app_gov/workers.py` :
  nouveau worker `gov.refresh_party_cache` qui fetch legacy
  party data, publish `party.refreshed` sur `ctx.events`
- `packages/nexus-app-gov/src/nexus_app_gov/app.py` : tab
  Politiciens subscribe au topic `party.refreshed` et trigger
  re-fetch de sa query React Query côté shell via SSE endpoint
  `/app/{name}/events?pattern=party.refreshed`
- `packages/nexus-coordinator/src/nexus_coordinator/api/
  events.py` : nouveau router SSE `GET /app/{name}/events?
  pattern=...` qui subscribe au bus per-app et streame
  `EventEnvelope` JSON en Server-Sent Events
- `web/src/hooks/useAppEvents.ts` : hook React qui ouvre un
  `EventSource` sur l'endpoint SSE et invalide React Query
  cache sur event matching
- Vitest `useAppEvents.test.ts` (~6 tests)
- Playwright `gov-party-refresh-event.spec.ts` : trigger
  worker via palette → attendre event → grid mise à jour sans
  reload

**Critère** : full suite verte, +25 tests SDK, +6 Vitest,
+1 Playwright, +3 tests coord (SSE endpoint), budget
`main` + `palette` + `tabview` tiennent.

**Commit cible** : `feat(sdk,coordinator,app-gov,web): Sprint
9 Phase C — AppContext.events asyncio pub/sub + SSE endpoint
+ gov party.refreshed consumer`

### Phase D — Migration runner + gov per-app DB + 001_documents.sql

**Scope** : D4 primitive + consommateur gov migration.

- `packages/nexus-sdk/src/nexus_sdk/migrations.py` (~240 LOC)
- `AppManifest.migrations_dir: Path | None = None` field
- `Coordinator.start()` ordered step new : migration scan +
  apply après `on_start`
- `nexus-coordinator` CLI `migrate` sous-commande Typer avec
  `--plan` / `--apply` / `--project` / `--app` options
- `packages/nexus-sdk/tests/test_migrations.py` (~18 tests :
  happy path 001/002/003, idempotent re-run, SHA tamper
  detection, BEGIN IMMEDIATE rollback on fail, lexico order
  enforce, dry-run plan, CLI `--plan` output, missing
  migrations_dir skip, empty migrations_dir skip,
  `_nexus_migrations` table auto-create, version extraction
  from filename, slug extraction, opt-in via manifest,
  forward-only, no-downgrade, read-only client raises)
- `packages/nexus-app-gov/src/nexus_app_gov/migrations/
  001_documents.sql` — 1 fichier qui crée `gov_documents`
  table
- `packages/nexus-app-gov/src/nexus_app_gov/app.py` :
  `GovApp.on_start` crée un `AppDatabaseClient` per-app
  writable séparé de la legacy (2 clients distincts :
  `ctx.db_gov` read-only sur govdata.db, `ctx.db_app`
  writable sur `projects/.../apps/gov/app.sqlite`). Update
  `AppContext` pour porter 2 clients (ou un dict `dbs:
  dict[str, AppDatabaseClient]`)
- Tests gov `test_gov_migrations.py` : runner applique 001,
  création table verifiée, idempotence verifiée

**Critère** : full suite verte, +18 tests SDK, +4 tests
gov, CLI `nexus-coordinator migrate --project ... --plan`
produit un output lisible, budget `main` + nouveau
`vendor-query` ou `tabview` tiennent.

**Commit cible** : `feat(sdk,coordinator,app-gov): Sprint 9
Phase D — DB migration runner (SHA256 tamper detection, CLI
plan/apply) + gov 001_documents.sql consumer`

### Phase E — File upload + CAS + TabView v2 bump + gov Documents tab + Sprint 7 deep tech debt

**Scope** : D3 primitive (upload + CAS + schema v2) +
consommateur gov Documents tab + Sprint 7 tech debt E-1, C-4,
D-3 qui touche le Rust stack.

- `packages/nexus-sdk/src/nexus_sdk/files.py` (~340 LOC)
- `packages/nexus-sdk/src/nexus_sdk/view.py` : refacto en
  `TabViewV1` / `TabViewV2` / `AnyTabView` discriminated
  union, nouveau `file_upload_block()` constructor,
  `tabview_v1_schema.json` snapshot + nouveau
  `tabview_v2_canonical.json` cross-lang fixture
- `@nexus_app_files(accept=...)` class-level decorator
- `packages/nexus-coordinator/src/nexus_coordinator/api/
  files.py` (~220 LOC) : `POST /app/{name}/files/upload`
  multipart chunked read + magic bytes validation
- `packages/nexus-coordinator/src/nexus_coordinator/api/
  apps.py` : `TabView.model_validate` → `AnyTabView.
  model_validate`
- `web/src/components/app/tabview/schema.ts` : Zod
  `z.discriminatedUnion("schema_version", [v1, v2])`
- `web/src/components/app/tabview/blocks/FileUploadBlock.tsx`
  (~180 LOC) : drag-and-drop HTML5 native, progress via D2
  events SSE, preview thumbnail images
- `packages/nexus-app-gov/src/nexus_app_gov/app.py` : nouveau
  tab **Documents** (20e tab) liste `gov_documents` via
  `ctx.db_app.fetchall()`, porte un FileUploadBlock
- Playwright `gov-documents-upload.spec.ts` : upload PDF →
  vérif progress bar → vérif apparait dans la liste après
  flush events
- Tests SDK `test_files.py` (~20 tests : store happy path,
  dedup, manifest, open streaming, magic bytes reject PDF-as-
  PNG, content-type allowlist, missing file, chunked read
  large file, concurrent store same sha)
- Tests SDK `test_view_v2.py` (~12 tests : v1 valide sous v2,
  v2 file_upload_block parse, v2 invalide sous v1, cross-lang
  fixture roundtrip, extra forbid préservé)
- Tests coord `test_files.py` (~10 tests : upload route
  multipart cap, magic bytes reject, allowlist decorator,
  SHA256 dedup header, 404 pour app sans décorateur)
- Tests gov `test_gov_documents.py` (~6 tests)

**Sprint 7 deep tech debt (dans le même commit — scope
volontairement chargé parce que E-1/C-4/D-3 sont des fixes
Rust ciblés qui ne couplent pas avec D3)** :

- **E-1** `DEFAULT_PROBE_TIMEOUT` 2s → configurable via env
  `NEXUS_PROBE_TIMEOUT_MS` avec default 2000, ajouter un test
  qui vérifie le parsing
- **C-4** `tokio::sync::Semaphore(max_inflight=32)` ajouté au
  `CuratorRuntime` handler gossip, test qui vérifie le
  backpressure via 100 messages en burst
- **D-3** `CuratorRuntime::subscribe` : try persist first,
  rollback RAM state sur failure (le commentaire T3 dans
  `PATTERNS.md` documente le pattern attendu)

**Critère** : full suite verte, +20 tests SDK files,
+12 tests SDK view v2, +10 tests coord, +6 tests gov,
+1 Playwright, +3 tests Rust (E-1 + C-4 + D-3), budget
total avec nouveau `upload` chunk ≤ sum(budgets).

**Commit cible** : `feat(sdk,coordinator,app-gov,web,
shell-daemon-core): Sprint 9 Phase E — file upload + CAS +
TabView v2 bump + gov Documents tab + Sprint 7 E-1/C-4/D-3
tech debt`

### Phase F — `verification.md` + `audit_plan.md` pour Sprint 10 + PATTERNS updates

**Scope** : docs de sortie + no code change.

- `.planning/sprint9_verification.md` (self-report fail-fast
  ~38 rows, format Sprint 6/7/8)
- `.planning/sprint9_audit_plan.md` (plan d'audit que session
  fraîche Sprint 10 Phase 0 jouera, 10 tracks A..J)
- `docs/shell/PATTERNS.md` updates :
  - **P12** — Code splitting via `createBrowserRouter` +
    `lazy` + `manualChunks` (Sprint 9 Phase A)
  - **P13** — Event bus primitive via anyio memory streams
    avec context manager (Sprint 9 Phase C)
  - **P14** — File upload CAS sharded + manifest JSON +
    magic bytes validation (Sprint 9 Phase E)
  - **P15** — TabView schema evolution: discriminated union
    on `schema_version`, `extra="forbid"` sacré, cross-lang
    fixture par version (Sprint 9 Phase E)
  - T8, T10, T11, T12 → CLOSED avec SHA
- `docs/rust/PATTERNS.md` updates :
  - H-3 → CLOSED (Sprint 9 Phase A)
  - E-1, C-4, D-3 → CLOSED (Sprint 9 Phase E)
  - Nouvelle section « Sprint 9 patterns » si besoin
- Memory update `nexus_grid_pivot.md` : Sprint 9 CLOSED, tip
  master, compteurs tests, transition Sprint 10 scope
  (release prep + branding + 3 VPS bootstrap)

**Critère** : Phase F est strictement doc, aucun code
change, fail-fast suite identique au commit précédent (rien
de nouveau à re-run), 2 fichiers `.planning/` livrés + 2
PATTERNS docs touchés + memory update.

**Commit cible** : `docs(sprint9): verification + audit plan
for Sprint 10`

---

## 6. Scope cuts (à respecter strictement)

### 6.1 Ce que Sprint 9 ne livre PAS

- **Pas de branding / renommage / docs public** — glissé Sprint
  10+. Le nom SBFB vs nexus-grid reste indéterminé, aucun
  fichier renommé
- **Pas de release v1.0 / PyPI publish / npm publish** — Sprint
  10
- **Pas de 3 VPS bootstrap** — Sprint 10
- **Pas de cross-app events** (`AppContext.events` est per-app
  in-process only) — Sprint 10+ si un consommateur le demande
- **Pas de cross-node events** (pas d'iroh-gossip bridge
  vers `ctx.events`) — Sprint 11+
- **Pas de `AppContext.storage` typé namespace API cross-app**
  — chaque storage est strictement per-app
- **Pas de downgrade migration runner** — forward-only, pattern
  datasette / yoyo simplifié
- **Pas de migration runner `repair` CLI** — anti-pattern Flyway
- **Pas de cloud storage / S3 / blob store** pour les uploads
  — uniquement CAS filesystem local
- **Pas de streaming chunked dans les handlers** au-delà du
  endpoint upload (les apps qui veulent streamer un gros fichier
  passent par `ctx.files.open(sha256)` qui retourne un
  `AsyncIterator[bytes]`)
- **Pas de `python-magic` dep** (FFI libmagic complique Windows)
  — whitelist hardcoded de 5 magic numbers
- **Pas de new toast lib** (sonner/react-hot-toast) — T11 fix
  utilise le pattern inline Sprint 8
- **Pas de Rolldown `advancedChunks`** — tech debt Sprint 10+
  quand Rolldown stable
- **Pas de React Server Components ni SSR** — le shell reste
  SPA local-only
- **Pas de route loader** via React Router v6 `loader` — les
  pages continuent à utiliser React Query directement (plus
  simple, l'article react.dev le permet explicitement)
- **Pas de module fédéré / micro-frontend** — overkill pour
  notre scope

### 6.2 Sprint 7 tech debt NON traitée Sprint 9

- **G-3 daemon DTOs `deny_unknown_fields`** — Sprint 8 Phase A
  l'a traitée (déjà CLOSED), out of scope Sprint 9
- **F-2 CommandPalette loading state** — P3 Sprint 7, non
  touché, tech debt Sprint 10+
- **F-3 CardTitle a11y (T8)** — CLOSED Sprint 9 Phase A

### 6.3 Sprint 8 audit P3 laissés tels quels

Tous les P3 `sprint8_audit_findings.md` §P3 (A-FX-1, B-FX-1,
B-FX-2, B-FX-3, C-FX-3, C-FX-4, C-FX-5, D-FX-2, E-FX-1,
E-FX-2, E-FX-3, V-FX-2) sont considérés comme acceptable
technical debt et **NE sont PAS fixés Sprint 9** sauf si un
fix tombe naturellement dans le périmètre d'une phase (ex :
`resolve_worker` duplicate check sera fait en Phase B si le
test `test_resolve_worker_duplicate_name_raises` est trivial
à ajouter avec le refacto typed namespace ; sinon reporté).

---

## 7. Traçabilité Sprint 8 → Sprint 9 (scope cuts → pris en charge)

Table mappe chaque item « What's NOT » du `sprint8_kickoff.md`
§D5 vers le sprint + phase qui le prend en charge.

| Item Sprint 8 scope cut | Pris en charge par | Raison |
|---|---|---|
| `AppContext.storage` | **Sprint 9 Phase B** | Cœur Sprint 9 |
| `AppContext.events` | **Sprint 9 Phase C** | Cœur Sprint 9 |
| File upload endpoint | **Sprint 9 Phase E** | Cœur Sprint 9 + bump schema v2 |
| DB migration runner | **Sprint 9 Phase D** | Cœur Sprint 9 + tamper detection |
| E-1 probe_reachable 2s timeout | **Sprint 9 Phase E** (deep tech debt) | Touche Rust stack, groupé avec C-4/D-3 |
| C-4 gossip backpressure | **Sprint 9 Phase E** | Idem |
| D-3 subscriptions persist order | **Sprint 9 Phase E** | Idem |
| F-3 CardTitle a11y | **Sprint 9 Phase A** (T8) | Frontend touch, groupé avec code splitting |
| G-1 httpx Limits | **Sprint 9 Phase A** (T10) | FastAPI coord, léger |
| H-3 wheel install drift | **Sprint 9 Phase A** (D5) | Blocker structurel, doit être Phase A |
| T9 bundle headroom | **Sprint 9 Phase A** (D6) | Blocker structurel, doit être Phase A |
| T11 palette error swallow | **Sprint 9 Phase A** | Frontend léger |
| T12 commands() ordering | **Sprint 9 Phase A** | SDK oneline fix |
| Retrait `legacy_descriptor` fallback | **Sprint 8 Phase A** (DONE) | Déjà fait |

---

## 8. Audit gate pattern — rappel

Instauré Sprint 6 après constat que les verifications
self-reportées ont une valeur limitée (§3 de
`docs/claude/README.md` + memory `sprint_audit_gate.md`).

- **Sprint 9 Phase 0** : DONE avant ce kickoff. A produit
  `sprint8_audit_findings.md` + 2 fix + T8..T12 tech debt
  + H-3 promotion. Les 5 commits de gate (`449f404` →
  `c50157d`) sont landed sur master. Sprint 9 Phase A démarre
  sur un tip (`c50157d`) qui est **strictement plus loin**
  que ce que le verification.md Sprint 8 original pointait
- **Sprint 9 Phase F** : sera OBLIGATOIRE → doit livrer
  `.planning/sprint9_verification.md` + `.planning/
  sprint9_audit_plan.md`. Sans ces 2 fichiers, le sprint ne
  peut pas être fermé. Le Sprint 10 Phase 0 jouera
  `sprint9_audit_plan.md` dans une session fraîche et
  produira `sprint9_audit_findings.md` comme blocker de la
  Phase A Sprint 10

Exception possible uniquement si l'utilisateur demande
explicitement de skipper l'audit — dans ce cas, noter « Phase
0 audit skipped per user decision YYYY-MM-DD » et prévoir un
audit rétroactif au sprint suivant. **Aucune exception
demandée pour Sprint 9**.

---

## 9. Checkpoint de validation

Avant d'écrire la moindre ligne de code Phase A, l'utilisateur
(FlowUP) doit valider :

1. **Les 6 décisions Day 0 D1..D6** — en particulier :
   - D2 : wrapper anyio vs asyncio.Queue maison (pushback
     possible si tu préfères la version plus simple)
   - D3 : bump TabView schema v1 → v2 dans le **même commit**
     Phase E vs splitter en Phase E1 (schema bump) + E2
     (file upload) — le plan actuel garde un seul commit
     Phase E pour éviter de faire un bump schema isolé sans
     consommateur
   - D6 : budget main 475 → 350 KB est ambitieux. Si tu préfères
     garder 475 KB et investir moins de temps sur le tree-shake,
     dis-le maintenant
2. **Le split Phase A..F** — 6 phases, ordre A→F figé. Si une
   phase paraît trop lourde (E est la plus dense), pousse back
3. **Le scope cut « pas de toast lib »** (T11 utilise le
   pattern inline Sprint 8) — si tu veux un sonner pour avoir
   un layer toast réutilisable, dis-le avant Phase A
4. **La convention « consommateur réel par primitive dans
   nexus-app-gov »** — ça ajoute ~600 LOC gov et 4 Playwright
   specs. Si tu veux shipper les primitives sans consommateur
   (pure scaffolding), dis-le maintenant ; mais c'est contre
   `feedback_approach.md` et je ne le recommande pas
5. **Les scope cuts stricts §6** — notamment « pas de branding,
   pas de release, pas de VPS »

**Si tout OK** : commit `docs(sprint9): kickoff + plan`
(ce fichier + `sprint9_plan.md`) sur master, puis Phase A
démarre immédiatement en fresh session.

**Si une décision doit être ré-arbitrée** : note-la, on
re-écrit la section concernée, on refait un checkpoint.

---

**Références clés** :
- `.planning/sprint9_plan.md` — feuille de route détaillée
  phase par phase + fail-fast checklist + risks
- `.planning/sprint8_audit_findings.md` — findings Sprint 8
  Phase 0 gate, commits fix landed
- `docs/claude/README.md` — workflow source of truth
- `docs/shell/PATTERNS.md` T8..T12 — nouvelle tech debt
  loggée
- `docs/rust/PATTERNS.md` §H-3 — promotion P1 Sprint 9
