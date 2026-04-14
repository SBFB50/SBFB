# Sprint 4 kickoff — Coordinateur Python + SDK + migration app gov

**À utiliser dans une session Claude fraîche ouverte dans
`C:\Users\FlowUP\Documents\Code\nexus`**. Document self-contained:
tout le contexte nécessaire est ici ou explicitement pointé vers
un fichier à lire en entier. Pas de "cherche ailleurs", pas de
devine, pas de training data pour les API de libs — `context7`
pour toute bibliothèque non-triviale avant d'écrire une ligne
contre elle.

---

## 1. Mission (une phrase)

Livrer un coordinateur Python qui utilise les bindings `nexus_core`
(PyO3) pour créer, signer et dispatcher des tâches LLM à des workers
`nexus-worker` externes via iroh-docs, avec un SDK `nexus-sdk`
consommable par des apps tierces, et une première app (`gov`)
migrée comme preuve de concept. Sortie : un coordinateur + un
worker Rust externe forment un réseau P2P fonctionnel, e2e,
sans serveur central.

## 2. État à l'entrée (vérifié aujourd'hui 2026-04-10)

Branche `master`, HEAD = `f68d997` (docs sprint3 verification
checklist). Sprint 0/1/2/3 tous CLOSED. Working tree clean à
l'exception de `.planning/audit_sprint2/` (9 shards d'audit Sprint 2
hors workspace cargo, inertes).

Résultat de `.planning/sprint3_verification.md` lancé aujourd'hui :
10/11 lignes du tableau fail-fast passent strictement (seule ligne
"working tree" en fail soft à cause des shards d'audit untracked).
Preuve forte : `nexus-worker start --headless` boote iroh endpoint
+ relay n0, ollama healthcheck OK (2 modèles sur :11434), GPU
monitor NVML découvre RTX 5080 16 GB. La machinerie Sprint 3 est
opérationnelle.

**Crates existantes** :

- `crates/nexus-core-rs/` — library iroh wrapper, 56 tests lib + 5
  doctests (gossip/docs/blobs/node/verifier + task.rs avec
  `canonical_bytes`, Task, TaskEntry, Claim, ResultPayload,
  ResultEntry, sign/verify)
- `crates/nexus-core-py/` — bindings PyO3 (**634 lignes** dans
  `src/lib.rs`). Expose Node, Doc, Gossip, Blobs, Verifier, plus
  des free functions sign/verify sérialisant Task/Result en JSON.
  Buildable via maturin en wheel. À lire en entier avant Phase A
  pour savoir ce qui est déjà là et ce qu'il faut ajouter.
- `crates/nexus-worker-core/` — core worker headless (config,
  allowlist, engine, gpu, invite, ollama, state machine)
- `crates/nexus-worker/` — binaire CLI clap (register/start/join/
  projects/browse/stats/config) + TUI ratatui optionnelle + logging
  JSON rotating + 11 e2e tests

**N'existe PAS encore** : `packages/`, `apps/`, `examples/`,
aucun package Python du monorepo uv n'est créé. Le `pyproject.toml`
racine existe (workspace uv) mais il n'y a encore aucun membre.

## 3. Sources de vérité à lire AVANT d'agir

Lecture obligatoire, dans cet ordre, avant d'écrire le plan détaillé
Sprint 4 :

1. **`C:\Users\FlowUP\.claude\plans\magical-marinating-phoenix.md`**
   - Lignes **715-737** : 10 items de tech debt Sprint 2 dont
     les 2 P1 qui bloquent Sprint 4 Day 1
   - Lignes **797-818** : plan Sprint 4 officiel (14 jours)
   - Lignes **927-951** : 20 décisions architecturales figées
     qu'il ne faut PAS re-débattre
   - Lignes **1046-1051** : critères de sortie Sprint 4

2. **`docs/rust/PATTERNS.md`** section "Tech debt logged"
   (ligne ~556) — les 10 items d'audit avec leur diagnostic
   technique précis

3. **`crates/nexus-core-rs/src/task.rs`** — en particulier
   - ligne 49-100 : struct `Task` avec son ordre de declaration
     (NON alphabétique → c'est le bug P1)
   - ligne 129-168 : `TaskEntry` + `sign` / `verify_signature`
   - ligne 309 : fonction `canonical_bytes` actuelle
   - ligne 319+ : tests existants à porter vers le nouveau format

4. **`crates/nexus-worker-core/src/engine/runtime.rs`**
   - ligne 1-43 : doc d'intro qui décrit le scope W9 et annonce
     les `TODO(W9.1)` markers
   - ligne 383 : le `TODO(W9.1)` concret qui attend Sprint 4

5. **`crates/nexus-core-py/src/lib.rs`** (634 lignes) — pour
   cartographier ce qui est déjà exposé à Python et ce qu'il
   reste à ajouter pour que le coordinateur puisse publier des
   `task:*` entries, s'abonner à un `results` stream, et signer
   les Tasks avec l'API canonique commune.

6. **`crates/nexus-core-rs/src/gossip.rs`** lignes 104-107 — le
   field `&'a Gossip` qui fait que `GossipClient` porte un
   lifetime et ne peut pas être stocké dans un handle Python
   long-lived (P1 bloquant Phase A).

Ne PAS lire : `nexus/` legacy Python (sauf `nexus/gov/` en Phase D
pour la migration), `web/`, `docker-compose.yml`, Sprint 0/1/2/3
plans déjà clos.

## 4. Day 0 — 2 blockers P1 à fixer AVANT toute ligne de coordinateur

### Blocker #1 : `canonical_bytes` cross-langue

**Diagnostic** : aujourd'hui `crates/nexus-core-rs/src/task.rs:309`
utilise `serde_json::to_vec(&value)` qui sérialise les struct
fields en ordre de **déclaration** (`version, task_id, task_type,
prompt, system_prompt, model, priority, created_at, parent_task_id,
metadata`). Python `json.dumps(obj, sort_keys=True)` sérialise en
ordre **alphabétique**. La première fois que le coordinateur
Python signera une `Task` pour la publier, le worker Rust qui
`verify_signature()` recalculera des bytes différents et la
vérification Ed25519 échouera silencieusement — le worker
rejettera toutes les tâches sans warning clair.

Aggravant : les 2 comments actuels dans `task.rs` (lignes 300-317)
affirment que "serde_json + json.dumps(sort_keys=True) agree on
the wire format". **C'est faux.** Ce commentaire doit être
corrigé dans le même commit que le fix.

**Fix global (PAS band-aid)** : adopter **RFC 8785 JSON
Canonicalization Scheme (JCS)** comme format canonique unique des
deux côtés. JCS est un standard IETF qui garantit des bytes
identiques pour tout JSON "well-formed" par construction (lex-sort
des keys à tous les niveaux, escaping normalisé, nombres canoniques).

Étapes :

1. Via **context7** : confirmer l'existence et l'API actuelle de
   - Rust : crate `serde_jcs` (version courante, signature de la
     fonction top-level)
   - Python : package `jcs` sur PyPI (version, API)

2. Si `serde_jcs` convient : ajouter au `Cargo.toml` workspace,
   réécrire `canonical_bytes` en `serde_jcs::to_vec(&value)`,
   corriger les doc-comments faux, lancer la suite de tests
   existants (`task_canonical_bytes_is_deterministic`,
   `metadata_order_does_not_matter`, `task_roundtrip_through_canonical_bytes`).

3. **Ajouter un test cross-langue** : générer une fixture JSON
   canonique depuis Python `jcs` (script one-shot dans
   `tests/fixtures/canonical_task.hex`) et ajouter un test Rust
   qui appelle `canonical_bytes(&sample_task)` et compare à la
   fixture hex. Ce test est la garantie de non-régression.

4. **Exposer dans les bindings PyO3** : `nexus_core.canonical_bytes`,
   `nexus_core.sign_task(task_dict, secret_key)`,
   `nexus_core.verify_task(task_entry_dict)`. Le coordinateur Python
   utilisera ces functions **exclusivement** — jamais de sérialisation
   manuelle côté Python.

5. **Bonus — domain separation prefix** (audit P3 item 3) : pendant
   qu'on touche à `canonical_bytes`, ajouter un prefix par type
   (`b"nexus-task-v1\0"`, `b"nexus-result-v1\0"`, `b"nexus-claim-v1\0"`).
   Aujourd'hui `canonical_bytes(&claim)` et `canonical_bytes(&task)`
   produisent des structures similaires → risque de confusion
   cryptographique en v1.2. Fix trivial maintenant, douloureux
   plus tard. Cost : 1 commit, 3 tests.

**Si `serde_jcs` n'existe pas ou est unmaintained** : fallback sur
`rmp-serde` (MessagePack canonical) ou CBOR (RFC 8949 §4.2.1 canonical
encoding). Binary mais deterministic par construction. Trancher
avec le user avant de coder.

**Alternative rejetée** : réordonner les fields Rust en alphabétique.
Fragile (tout ajout de field casse la compat), invisible dans la
code review, ne couvre pas les maps nested dans des sous-structs.

### Blocker #2 : `GossipClient<'a>` lifetime

**Diagnostic** : `crates/nexus-core-rs/src/gossip.rs` ligne 104-107
— le struct a un field `&'a Gossip`. Le Sprint 2 s'en est sorti
parce que les bindings PyO3 ne stockent jamais un `GossipClient`
across la frontière FFI (les tests n'ont couvert que des usages
éphémères). Le coordinateur Python Sprint 4 **doit** pouvoir
maintenir un handle `Gossip` long-lived pour :

- Écouter en continu les annonces de curator lists (gossip topic
  `curators-v1`)
- Publier les `BlobTicket` de sa propre curator list quand elle est
  mise à jour
- Optionnellement recevoir des heartbeats de workers via gossip

**Fix global** : retirer le paramètre de lifetime du struct.
`Gossip` est déjà cloneable bon marché (Arc interne dans iroh
0.97). Remplacer `&'a Gossip` par `Gossip` owned, ajuster les
méthodes du impl block, vérifier que les tests gossip existants
passent, vérifier que le nouveau field peut traverser la FFI
(ajouter un test PyO3 qui stocke un `GossipClient` dans un
`#[pyclass]` et l'utilise après un round-trip).

### Day 0 — commit plan

- Commit 1 : `fix(core-rs): P1 canonical_bytes cross-language via RFC 8785 JCS`
- Commit 2 : `fix(core-rs): P1 GossipClient owned Gossip (no lifetime)`
- Commit 3 : `feat(core-py): expose canonical_bytes + sign_task + verify_task`
- Commit 4 : `test(core-rs): cross-language canonical fixture + domain prefix`

Ces 4 commits tombent **avant** le premier commit de Phase A.
Si un des 4 échoue, arrêter et discuter avec le user. Ne PAS
commencer la Phase A tant qu'ils ne sont pas verts.

## 5. Plan Sprint 4 — 4 phases, 14 jours

Le plan officiel jour-par-jour est dans phoenix.md lignes 801-814.
Les jours sont **indicatifs**, pas contraignants. Structure réelle
en 4 phases qui se ferment chacune sur un critère passable :

### Phase A — Coordinateur core (≈ Day 1-2)

Créer `packages/nexus-coordinator/` avec `pyproject.toml` + layout
`nexus_coordinator/` (snake_case module name), ajouter au workspace
uv racine, `uv sync` vert.

- `nexus_coordinator/coordinator.py` :
  - Charge une clé Ed25519 persistante. **Décision à prendre Day 1**:
    (a) `keyring` 25.7+ (prod-friendly, native keychain) ; (b) fichier
    `~/.nexus-grid/projects/<name>/coord.key` perm 600 (simple, marche
    partout, aligné sur le worker Sprint 3) ; (c) hybride avec `keyring`
    primaire et fallback fichier. Recommandation : (b) pour v1.0,
    aligne sur le worker, upgrade vers (a) en v1.1.
  - Crée un `Node` iroh via `nexus_core.Node` (ou `Node.with_secret`),
    persistant dans `~/.nexus-grid/projects/<name>/iroh-data/`
  - Crée deux docs per-project : `tasks` (single-writer =
    coordinateur) et `results` (multi-writer, les workers écrivent).
    **Décision à prendre Day 1** : un seul doc avec préfixes
    `task:*` / `result:*` / `claim:*` (simple, 1 ticket partagé) OU
    docs séparés (permissions plus propres). Le plan dit 2 docs.
    Aligne-toi sur le plan sauf raison technique forte.
  - Publie `Node.id()` dans la DHT iroh-pkarr si `project.visibility
    == "public"`
  - Lifespan FastAPI qui expose `/health`, `/project`, `/tasks`,
    `/results`, `/kudos`, `/invite` sur `127.0.0.1:8765` par défaut
    (via context7: patterns fastapi lifespan + dependency injection)

- `nexus_coordinator/config.py` : `CoordinatorConfig` Pydantic
  v2 + `pydantic-settings` `BaseSettings` avec env var override
  `NEXUS_COORD__*` (parallèle du `NEXUS_WORKER__*` Sprint 3 W3).
  TOML persistent dans `~/.nexus-grid/projects/<name>/coordinator.toml`.

- Tests d'intégration : spawn un Node iroh réel in-process, crée
  un project, vérifier que les 2 docs sont créés, que l'API FastAPI
  répond (via `httpx.AsyncClient` dans un `pytest.fixture` asyncio).

**Critère de fermeture Phase A** : `uv run nexus-coordinator start
test --port 8765` boote, logs `iroh endpoint ready node_id=...`,
`curl 127.0.0.1:8765/health` retourne 200 avec project name et
node_id.

### Phase B — Dispatcher + Validator + Kudos (≈ Day 3-5)

- `nexus_coordinator/dispatcher.py` : soumission de `TaskEntry` signées
  au `tasks` doc. Utilise `nexus_core.sign_task()` (exposé Day 0).
  Track interne pending/claimed/completed via SQLite local (table
  `task_state (task_id PK, state, submitted_at, claimed_by_pubkey,
  claimed_at, completed_at, result_hash)`). Retry si le worker ne
  claim jamais avant `claim_timeout_secs`. Expose
  `POST /tasks/submit`.

- `nexus_coordinator/validator.py` : subscribe le `results` doc via
  `nexus_core.Doc.subscribe()` (via context7 : iroh-docs LiveEvent
  API dans PATTERNS.md "Recette iroh-docs"). Pour chaque
  `ResultEntry` nouveau :
  1. Charger la Task correspondante depuis le `tasks` doc
  2. Appeler `nexus_core.Verifier.verify_3_layers(task, result)`
     — Layer 1 = signature Ed25519, Layer 2 = model digest,
     Layer 3 = logprob hash
  3. Si OK → créditer kudos via kudos.py, marquer task completed
  4. Si KO → log trust decrement, laisser task re-dispatchable
- `nexus_coordinator/kudos.py` : ledger SQLite per-project
  `kudos_ledger (id PK, worker_pubkey, task_id, tokens, quality,
  trust_mul, amount, prev_hash, entry_hash, sig)`. Hash chain
  append-only : `entry_hash = sha256(prev_hash || canonical_bytes(entry))`.
  Formule : `kudos = tokens × quality_factor × trust_multiplier`.
  Fonction `verify_chain_integrity()` qui replay la chain depuis
  la tête et détecte toute modification de 1 byte.

Tests : (a) submit 10 tasks, vérifier qu'elles apparaissent dans
le doc, (b) simuler 10 results valides via le même `nexus_core.Doc`,
vérifier que les 10 kudos entries sont ajoutées avec hash chain
valide, (c) modifier 1 byte dans le ledger → `verify_chain_integrity`
retourne False.

**Critère de fermeture Phase B** : un dispatcher + un validator +
un kudos ledger tournent en boucle interne (in-process, sans worker
externe) sur 10 tasks simulées. Hash chain vérifiable.

### Phase C — Invite + CLI (≈ Day 6-7)

- `nexus_coordinator/invite.py` : génération de liens d'invitation
  pour les workers.

  **Décision critique Day 1** : format d'invite.
  - Le Sprint 3 W8 a créé un format `nx1...` base32 dans
    `crates/nexus-worker-core/src/invite.rs` qui encode JSON +
    Ed25519 signature, scope `Worker` ou `Observer`, avec
    `coordinator_endpoint_addr + project_namespace + scope + expiry`.
  - Sprint 4 doit **étendre** ce format (pas créer un 2e) pour y
    ajouter le `doc_ticket_for_tasks: String` (un `DocTicket`
    iroh-docs sérialisé en string). Ce field est ce qui
    **débloquera W9.1** dans le runtime worker.
  - Version bump : `v1` → `v2` avec champ optionnel OR version
    bump majeure et refus des v1 par les workers Sprint 4+. Choisir
    **v2 optional** pour compat ascendante.
  - Le format `nx://...` URL mentionné dans le plan phoenix n'est
    qu'un alias UX ; le payload reste le format base32 `nx1...`
    wrappé dans un `nx://` pour le copy-paste depuis un navigateur.

- `nexus_coordinator/cli.py` : Typer 0.12+ (via context7 : patterns
  sub-commands + rich integration). Commands :
  - `nexus-coordinator init <name> [--public] [--description TEXT]`
  - `nexus-coordinator configure <name>`
  - `nexus-coordinator start <name> [--port 8765]`
  - `nexus-coordinator stop <name>` (graceful)
  - `nexus-coordinator invite create <project> --scope worker --expiry 7d --max-uses 10`
  - `nexus-coordinator invite revoke <invite_id>`
  - `nexus-coordinator invite list <project>`
  - `nexus-coordinator stats <project>` (tasks, kudos, workers connectés)
  - `nexus-coordinator kudos verify <project>` (hash chain integrity)

Tests CLI : subprocess tempfile fixtures (pattern Sprint 3 W12
`crates/nexus-worker/tests/e2e.rs`). Spawn le CLI Python via
`subprocess.run`, assert exit code + stdout + fichiers générés.

**Critère de fermeture Phase C** : un invite `nx1v2...` est généré
par le coordinateur, passé à `nexus-worker join <invite>`, le
worker le parse, extrait le `doc_ticket_for_tasks`, et peut
importer le tasks doc via `nexus_core.DocsClient.import_and_subscribe`.

### Phase D — SDK + migration gov + W9.1 drop-in + e2e (≈ Day 8-14)

- `packages/nexus-sdk/nexus_sdk/app.py` : `NexusApp` ABC avec
  hooks `on_start()`, `on_stop()`, `routes()`, `workers()`, `tabs()`.
  `AppManifest` Pydantic v2 : `name, version, author, routes,
  worker_prompts, frontend_tabs, dependencies, license`.

- `packages/nexus-sdk/nexus_sdk/decorators.py` : `@nexus_route(path,
  methods)`, `@nexus_worker(name, model)`, `@nexus_tab(name, icon)`.
  Collecte via registry global per-module.

- `packages/nexus-sdk/nexus_sdk/compute_client.py` :
  `ComputeClient(coordinator_url)`, `.submit_task(prompt, model,
  system_prompt, priority)` → POST vers `/tasks/submit` du
  coordinateur, retourne un `Future[ResultEntry]`.

- `packages/nexus-sdk/nexus_sdk/loader.py` : discovery des apps
  installées via `importlib.metadata.entry_points(group="nexus.apps")`.
  Retourne la liste des `NexusApp` disponibles.

- **Drop-in W9.1** : maintenant que Phase A écrit vraiment dans
  le `tasks` doc et que Phase C embarque un `DocTicket` dans
  l'invite v2, **remplir** le `TODO(W9.1)` dans
  `crates/nexus-worker-core/src/engine/runtime.rs:383` :
  1. Au `join`, parser l'invite v2, extraire `doc_ticket_for_tasks`,
     stocker dans l'allowlist (nouveau champ `task_doc_ticket`
     dans la table `projects`)
  2. Au boot, pour chaque projet enabled, `docs_client.import_and_subscribe(ticket)`
  3. Poll tick : `doc.get_many_by_prefix("task:")`, filtrer les
     tâches libres, claim atomique via `Claim::sign()` + écriture
     `claim:<task_id>` dans le doc, exécuter `ollama.generate()`,
     signer `ResultEntry` et l'écrire dans le `results` doc
  4. Ce drop-in nécessite d'étendre les bindings PyO3 ET l'API
     Rust pour le claim atomique si elle n'est pas encore là
     (à vérifier dans `task.rs` existing `Claim::new()`).

- `packages/nexus-app-gov/` : migration de `nexus/gov/` vers SDK.
  Créer `GovApp(NexusApp)`, déclarer les 19 tabs existants via
  `@nexus_tab`, les 31 workers via `@nexus_worker`, le manifest.
  **Point critique signalé dans phoenix.md ligne 960** :
  `nexus/engine/__init__.py` réexporte `POLITICAL_CONTRADICTION_PROMPT`.
  Sortir ce prompt de `nexus/engine/` (qui deviendra partie de
  `nexus-core` pur) et le garder uniquement dans `nexus-app-gov`.
  Refactor inévitable.

- `tests/e2e/test_coordinator_dispatches_10_tasks.py` : le test
  d'acceptance final. Spawn un coordinator Python (subprocess
  Python), spawn un `nexus-worker` externe (subprocess Rust binary)
  avec fixture config pointant vers un tempdir, publier 10
  TaskEntry signées, attendre 10 ResultEntry, vérifier toutes les
  signatures, timeout 120s. Ce test est **le** critère de succès
  Sprint 4.

- `examples/hello-world-app/` : exemple SDK minimal, <100 lignes
  Python, 1 route + 1 worker + 1 tab, `pyproject.toml` avec
  `entry_points[nexus.apps]`. Doit tourner dans le coordinateur
  via loader.py.

**Critère de fermeture Phase D** :

- `nexus-coordinator start gov` lance l'app gov
- Un `nexus-worker` externe (binaire Sprint 3) rejoint via invite
  v2 et commence à traiter des tâches LLM générées par gov
- `examples/hello-world-app/` < 100 lignes et tourne
- E2E test 10 tasks passe en <120s
- W9.1 TODO est remplacé par du vrai code, `grep TODO(W9.1)` dans
  `runtime.rs` retourne 0 match (OU 1 match de type
  `TODO(W9.2)` pour le prochain scope cut)

## 6. Règles opérationnelles (non négociables)

### R1 — Context7 obligatoire avant tout code contre une lib

Requêter **context7** pour toute bibliothèque non-triviale avant
d'écrire du code contre elle. Minimum obligatoire :

| Lib | Usage Sprint 4 |
|---|---|
| `serde_jcs` (Rust) | Day 0 canonical fix |
| `jcs` (Python) | Day 0 canonical fix |
| `pyo3` 0.22 | extension des bindings |
| `pyo3-async-runtimes` | bridge tokio/asyncio |
| `pydantic` 2.6+ | models partout (Task, ResultEntry, Manifest) |
| `pydantic-settings` 2.13+ | CoordinatorConfig |
| `fastapi` 0.111+ | API coordinateur |
| `typer` 0.12+ | CLI coordinator |
| `structlog` 25.5+ | logging JSON |
| `aiosqlite` 0.20+ | kudos ledger async |
| `keyring` 25.7+ | (si décision keyring Day 1) |
| `iroh-docs` 0.97 | cross-check contre le cargo registry local `C:/Users/FlowUP/.cargo/registry/src/index.crates.io-*/iroh-docs-0.97.0/` parce que context7 peut indexer des versions >0.97 |

**Ne JAMAIS deviner une signature d'API.** Ne JAMAIS écrire du
code basé uniquement sur la training data du modèle, en particulier
pour les libs qui ont bougé entre la knowledge cutoff et aujourd'hui
(2026-04-10). Si context7 retourne une signature qui ne matche
pas ce à quoi on s'attend, **ARRÊTER** et relire la réponse au
lieu de coder "ce qu'on croit être juste".

### R2 — Pas de fix pansement, cause racine uniquement

Si un bug est détecté en cours de Sprint 4, fixer la **cause racine**,
pas le symptôme. Si le symptôme est dans un module différent de celui
qu'on implémente, créer une entry tech debt dans
`docs/rust/PATTERNS.md` OU `docs/coordinator/PATTERNS.md` (à créer
Phase A) ET décider explicitement : fix maintenant OU tag Sprint 5.
Ne JAMAIS commiter :

- `try/except: pass` sans justification écrite
- `unwrap()` ou `expect()` non commenté (sauf dans un test)
- Valeur hardcodée "temporaire"
- `# type: ignore` sans référence à un issue tracker
- Commit message de la forme "WIP", "fix", "tmp", "hack"

### R3 — Global, deep, pas local

Avant d'écrire du code pour une phase, **relire le plan des 4 phases
en entier** et vérifier qu'aucune décision prise dans la phase
en cours ne contraint les phases suivantes de façon problématique.

Exemples concrets :

- Le format de l'invite en Phase C **détermine** si W9.1 pourra
  lire un DocTicket dans `join` → décision à prendre Day 1, pas
  Day 6
- Le choix un-doc-avec-prefix vs deux-docs en Phase A **détermine**
  la structure du dispatcher en Phase B → décision Day 1
- Le schema SQLite kudos en Phase B **détermine** l'API
  `/api/kudos` en Phase D → décision Day 1

Ces 3 décisions doivent être prises et écrites dans le plan détaillé
**avant** de commencer Phase A. Le plan détaillé est livrable Day 0.

### R4 — Tests d'intégration prioritaires

Le but de Sprint 4 est un réseau P2P fonctionnel bout-en-bout. Un
test unitaire qui mock iroh ne prouve rien. Tout fichier Python
livré doit avoir **au moins un** test d'intégration qui passe par
un `Node` iroh **réel** (in-process via `nexus_core`), pas mocké.

Acceptable :

- Tests unitaires purs pour la logique kudos (hash chain, formule
  quality factor)
- Tests unitaires purs pour les parsers (invite v2 deserialization)
- Tests unitaires pour les Pydantic models (validation cases)

Tout le reste passe par un Node iroh réel. Le pattern de test est
déjà dans `crates/nexus-core-rs/examples/two_nodes_docs_sync.rs`
pour Rust et doit être porté en Python comme helper pytest
(`tests/helpers/iroh_fixture.py`).

### R5 — Commits atomiques par phase

Message format : `feat(coordinator|sdk|app-gov|core-rs|core-py):
Sprint 4 Phase <A|B|C|D> — <résumé concis>`. Un commit par
fichier-major + ses tests. Pas de mega-commits multi-phases. Pas
de commits "WIP". Le plan Sprint 4 du phoenix est la grille —
chaque commit cite la phase (pas le jour, qui est indicatif).

Day 0 a ses propres commits (voir §4).

### R6 — Ne PAS toucher

- **Le code Python legacy dans `nexus/`** — sauf `nexus/gov/` en
  Phase D pour migration. Ne pas refactor `nexus/core/`, `nexus/recon/`,
  `nexus/vision/`, `nexus/forensics/` : ils restent intouchés pour
  Sprint 4 (migration coldcase est différable v1.1 selon le plan).
- **Les crates Rust existantes** sauf :
  - Les 2 P1 fixes Day 0 (`task.rs` + `gossip.rs`)
  - Le drop-in W9.1 Phase D dans `runtime.rs`
  - L'extension du format invite v2 dans `invite.rs`
  - L'extension nécessaire de `nexus-core-py` pour exposer les
    nouvelles APIs
- **`web/`** — Sprint 5
- **`docker-compose.yml`, `docs/BENCHMARK.md`, `docs/COMPUTE_STATUS.md`**
  — déjà clos Sprint 0
- **`magical-marinating-phoenix.md`** — source de vérité figée,
  relecture uniquement

### R7 — Verification continue

À la fin de chaque phase, lancer :

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --exclude nexus-core-py --profile ci --locked
uv run pytest packages/ -v
```

Tous verts → fermer la phase et commiter la tête. Un seul rouge →
stop et fix avant d'avancer.

## 7. Critères de sortie Sprint 4 (tableau fail-fast)

À produire en fin de Sprint 4 sous forme de
`.planning/sprint4_verification.md` analogue à
`.planning/sprint3_verification.md` :

| # | Check | Commande | Critère |
|---|---|---|---|
| 1 | canonical_bytes cross-lang | `cargo test -p nexus-core-rs canonical_bytes_matches_python_fixture` | green |
| 2 | GossipClient owned | `grep -n "GossipClient<" crates/nexus-core-rs/src/gossip.rs` | 0 match |
| 3 | PyO3 exposes sign/verify | `python -c "import nexus_core; nexus_core.sign_task"` | no AttributeError |
| 4 | Coordinator boots | `uv run nexus-coordinator start test --port 8765` | logs "iroh endpoint ready", /health 200 |
| 5 | Dispatcher | 10 tasks submitted via API, visible in tasks doc | green |
| 6 | Validator | 10 results processed, 10 kudos entries created | green |
| 7 | Kudos chain integrity | `nexus-coordinator kudos verify test` | valid; flip 1 byte → invalid |
| 8 | Invite v2 roundtrip | coord creates invite, worker joins, allowlist has doc_ticket | green |
| 9 | SDK hello-world | `wc -l examples/hello-world-app/**/*.py` | <100 LOC total |
| 10 | Gov migration | `nexus-coordinator start gov` → 19 tabs exposed in manifest | green |
| 11 | W9.1 drop-in | `grep TODO(W9.1) crates/nexus-worker-core/src/engine/runtime.rs` | 0 match |
| 12 | E2E 10 tasks | `pytest tests/e2e/test_coordinator_dispatches_10_tasks.py` | 10 results in <120s |
| 13 | Format | `cargo fmt --all --check` + `uv run ruff check packages/` | exit 0 |
| 14 | Full Rust tests | `cargo nextest run --workspace --exclude nexus-core-py --profile ci --locked` | all green (>161 expected) |
| 15 | Full Python tests | `uv run pytest packages/ -v` | all green |

## 8. Première action (ne rien lancer avant ceci)

Dans l'ordre strict :

1. Vérifier working tree clean (`git status`). Si `.planning/audit_sprint2/`
   encore untracked, demander au user quoi en faire avant de commencer
   (cf. divergence notée dans la verif Sprint 3).
2. Lire en entier :
   - `C:\Users\FlowUP\.claude\plans\magical-marinating-phoenix.md`
     lignes 715-818 et 927-951 et 1046-1051
   - `docs/rust/PATTERNS.md` section Tech debt (ligne ~556)
   - `crates/nexus-core-rs/src/task.rs` en entier
   - `crates/nexus-core-rs/src/gossip.rs` en entier
   - `crates/nexus-core-py/src/lib.rs` en entier (634 lignes)
   - `crates/nexus-worker-core/src/engine/runtime.rs` en entier
3. Requêter **context7** pour `serde_jcs` Rust et `jcs` Python,
   confirmer existence + API. Si l'un des deux manque ou est
   unmaintained, remonter au user avant de trancher le format
   canonique.
4. Écrire `.planning/sprint4_plan.md` — plan détaillé pour les 4
   phases avec, pour chaque phase :
   - Liste exhaustive des fichiers à créer (chemin absolu)
   - Dépendances `[tool.uv]` / `[dependencies]` à ajouter
   - Liste des décisions architecturales prises Day 1 (format invite,
     1-doc vs 2-docs, keyring vs fichier, etc.) avec justification
   - Liste des tests attendus (intégration + unitaires)
   - Critère de fermeture de la phase
5. **Montrer ce plan au user et attendre validation avant de coder
   quoi que ce soit**. Spécifiquement : les 3 décisions critiques
   (format invite v2 vs v1 breakage, 1-doc vs 2-docs, canonical
   format serde_jcs vs alternative) doivent être approuvées
   explicitement.
6. **Ensuite seulement** : Day 0 commits (4 commits §4), puis
   Phase A, B, C, D dans l'ordre.

---

**Rappel final** : Sprint 4 ferme l'architecture P2P de bout en
bout. Après Sprint 4, n'importe qui peut installer `nexus-coordinator`
via pip, `init` un projet, générer une invite, la partager, et un
worker externe Rust rejoint et traite ses tâches. C'est le passage
de "plomberie" à "produit". Pas de raccourci.
