# Sprint 7 — Plan détaillé (P2P Discovery Layer)

**Écrit** : 2026-04-11 à partir de `.planning/sprint7_kickoff.md`
après gel des décisions Day 0 D1..D5. Ce document est la grille
d'exécution Sprint 7 : chaque commit cite la phase, chaque test
est listé ici, chaque fichier touché est nommé. **Aucun code
Phase A..F n'est écrit avant que cette grille soit commitée**
(`docs(sprint7): kickoff + plan`).

**HEAD entrée** : `2926383`. Working tree clean modulo
`.planning/sprint7_kickoff.md` + ce fichier (commités ensemble en
ouverture via `docs(sprint7): kickoff + plan`).

**Goal Sprint 7 (une phrase)** : livrer
`nexus-shell-daemon-core` + `nexus-shell-daemon` (premier binaire
P2P long-lived de SBFB), les primitives curator list signées
Ed25519 + PyO3, le wrapping gossip subscribe + fetch_ticket côté
Rust, la résolution pkarr DHT, et les pages `/browse` + `/curators`
câblées via proxy coordinator — sans casser l'usage
coordinator-only du shell.

---

## 1. État vérifié à l'entrée

### 1.1 Sprint 6 livré + audit gate DONE (source : `sprint6_audit_findings.md`)

- Sprint 6 fermé à `504c6aa`, puis 11 commits de gate jusqu'à
  `2926383` : 1 findings doc + 8 `fix(sprint6): ...` + 1 tech debt
  doc + 1 TS fix follow-up
- **193 Rust** tests workspace (62 core-rs lib + 11 worker bin lib
  + 10 worker e2e + 105 worker-core lib + 5 doctests)
- **82 Python** tests + 1 skipped (SDK 32 + coord 47 + app-gov 3)
- **99 Vitest** tests web/ (+22 post-audit : 10 useCommandPalette
  + 6 cross_lang + 1 projectStore invariant + 5 RouteErrorBoundary)
- **10 Playwright** specs en ~16 s contre un coordinator live
- `cargo fmt --all --check` + `cargo clippy --workspace -- -D warnings` clean
- `ruff format --check` + `ruff check` clean
- `tsc --noEmit -p tsconfig.app.json` clean
- `npm run lint` clean (0 err + 5 T1 warnings tolérés)
- `npm run build` : main 457 / vendor-react 189 / vendor-ui 31
  / css 93 KB, zéro warning ; budgets D5 serrés (vendor-ui 50 KB
  post-fix E-2)
- `bash web/scripts/scan-en-strings.sh` clean
- `npm audit` : 0 vulns all levels

### 1.2 Consommé directement par Sprint 7

**Rust workspace** (`crates/`) :

| Crate | LOC | Rôle Sprint 7 |
|---|---|---|
| `nexus-core-rs` | ~3350 | **Étendu Phase B** : +`curator.rs` + `DOMAIN_CURATOR_LIST_V1` |
| `nexus-core-py` | ~1050 | **Étendu Phase B** : +`sign_curator_list` + `verify_curator_list_entry` |
| `nexus-worker-core` | ~3500 | **Inchangé** — pattern source pour le split A |
| `nexus-worker` | ~800 | **Inchangé** — pattern source pour le split A |
| `nexus-shell-daemon-core` | 0 | **Nouveau Phase A+C+D** (headless library, ~1200 LOC cible) |
| `nexus-shell-daemon` | 0 | **Nouveau Phase A** (binary + axum HTTP + clap, ~600 LOC cible) |

**Python coordinator** (`packages/nexus-coordinator/`) :

| Fichier | Rôle Sprint 7 |
|---|---|
| `api/daemon.py` *(nouveau Phase E)* | Router avec 5 routes proxy `/daemon/*` vers le shell-daemon local |
| `api/app.py` | Patch Phase E : `include_router(daemon_router)` |
| `paths.py` | Patch Phase E : `shell_daemon_registry_path()` (`~/.nexus-grid/shell-daemon/running.json`) |

**SDK Python** (`packages/nexus-sdk/`) : **inchangé** en Sprint 7.
L'extension `AppContext.submit_task` + `@nexus_command` sont
gelées dans D4/D5 mais **pas implémentées** — Sprint 8 Phase A.

**Frontend** (`web/src/`) :

| Fichier | Rôle Sprint 7 |
|---|---|
| `api/daemon.ts` *(nouveau Phase E)* | Client typé Zod pour les 5 routes `/daemon/*` du coordinator |
| `pages/Browse.tsx` | **Rewrite Phase E** — remplace le stub "arrive Sprint 6" par React Query + TabView-style render |
| `pages/Curators.tsx` | **Rewrite Phase E** — remplace le stub par listing + add/remove curator UI |
| `components/AppShell.tsx` | **Inchangé** (les routes existent déjà dans `App.tsx`) |

**Docs** :

| Fichier | Rôle Sprint 7 |
|---|---|
| `docs/shell/PATTERNS.md` | Phase F : +P9 (daemon HTTP proxy via coordinator), update T4/T5 status |
| `docs/rust/PATTERNS.md` | Phase F : +section Sprint 7 canonical (DOMAIN_CURATOR_LIST_V1, attribution-match, topic blake3 convention) |

### 1.3 Recherche consultée (détails §3)

| Lib | Usage Sprint 7 |
|---|---|
| `iroh 0.97` (`gossip`, `blobs`, `discovery`) | Phase C+D — déjà wrapped Sprint 2 ; pas de nouveaux drifts attendus |
| `iroh-docs 0.97` | **Pas Sprint 7** — curator lists via gossip+blobs seulement |
| `axum 0.7`/`0.8` | Phase A — HTTP server daemon (nouveau dans le workspace) |
| `tower-http` CORS | Phase A — CORSMiddleware regex loopback |
| `sysinfo` 0.32 | Phase A — pid liveness check pour singleton enforcement |
| `httpx` (déjà dep coord) | Phase E — proxy coordinator → daemon |
| `dashmap` 6.x | Phase C — curator list cache RAM-only |

---

## 2. Décisions Day 0 (D1..D5 gelées, cf `sprint7_kickoff.md` §4)

Résumé une-ligne par décision, détails dans le kickoff :

- **D1 — IPC HTTP loopback + running.json + proxy coordinator**.
  Le shell React ne parle **jamais** directement au daemon ; tout
  passe via `/daemon/*` sur le coordinator actif. CORSMiddleware
  loopback regex côté daemon en plus (défense en profondeur).
- **D2 — Daemon singleton**. Un seul `nexus-shell-daemon` par
  user. Boot refuse si running.json + pid vivant, écrase si
  stale.
- **D3 — Curator list schema gelé**. `CuratorList` +
  `CuratorListEntry` pattern exact `Claim`/`ClaimEntry`,
  `DOMAIN_CURATOR_LIST_V1`, topic unique
  `blake3("nexus-grid/curator/v1")[..32]`.
- **D4 — T4 Option B** (wire `AppContext.submit_task`). Signature
  gelée Sprint 7 Day 0, **implémentation Sprint 8 Phase A**.
- **D5 — T5 `@nexus_command`**. Decorator + `CommandDescriptor`
  + coordinator route + Zod mirror **designed et gelés Sprint 7
  Day 0**, implémentation Sprint 8 Phase A.

## 3. Research consulté (détails)

### 3.1 `nexus-worker-core` / `nexus-worker` pattern split

**Pattern extrait** (lu dans `crates/nexus-worker-core/src/lib.rs`
+ `Cargo.toml` + `crates/nexus-worker/src/main.rs`) :

- **Library crate (`*-core`)** : UI-free, aucune dep `clap` /
  `ratatui` / `crossterm` / `dialoguer`. Expose des structs
  config + runtime via tokio. Tests unit sans terminal.
- **Binary crate** : thin wrapper, uniquement parse CLI +
  setup tracing + call into core. Chaque handler est
  ≤ 50 LOC.
- **Cargo.toml** : binary dep le core via `path = "../nexus-*-core"`,
  les deux partagent workspace deps
- **`main.rs` pattern** :
  ```rust
  #[tokio::main]
  async fn main() -> Result<()> {
      let cli = Cli::parse();
      let paths = ShellDaemonPaths::resolve(cli.config.clone())?;
      let _log_guard = logging::init_logging(...)?;
      match cli.command {
          Command::Start { .. } => handle_start(&paths).await,
          Command::Stop => handle_stop(&paths).await,
          ...
      }
  }
  ```

**À reproduire dans Sprint 7 Phase A** : même split exact,
même `VERSION` const, même `PROJECT_QUALIFIER` / `PROJECT_ORGANIZATION`
/ `PROJECT_APPLICATION`, même `#![forbid(unsafe_code)]` +
`#![deny(rust_2018_idioms)]`.

### 3.2 `nexus-coordinator::registry` pattern

**Pattern extrait** (lu dans
`packages/nexus-coordinator/src/nexus_coordinator/registry.py`) :

- Pydantic `RunningState` avec `schema_version: Literal[1]`
- Atomic write via `running.json.tmp` → `os.replace`
- Discover via `projects_root.glob("*/running.json")`
- Remove best-effort dans le `finally:` du CLI `start`

**À reproduire dans Sprint 7 Phase A** : version Rust du
`RunningState` dans `nexus-shell-daemon-core::registry`, même
schéma (`schema_version: 1`, `node_id`, `api_host`, `api_port`,
`pid`, `started_at` RFC 3339). Path fixé à
`~/.nexus-grid/shell-daemon/running.json` (pas `projects/` —
c'est global per-user).

### 3.3 PyO3 `sign_claim` / `verify_claim_entry` pattern

**Pattern extrait** (lu dans
`crates/nexus-core-py/src/lib.rs:877-898`) :

```rust
#[pyfunction]
fn sign_claim(claim_json: &str, secret: &Bound<'_, PyBytes>) -> PyResult<String> {
    let sk: [u8; SECRET_KEY_BYTES] = array32(secret, "secret")?;
    let kp = KeyPair::from_secret_bytes(&sk);
    let claim: Claim = serde_json::from_str(claim_json)
        .map_err(|e| PyValueError::new_err(format!("bad claim json: {e}")))?;
    let entry = ClaimEntry::sign(claim, &kp).map_err(|e| py_err("sign_claim", e))?;
    serde_json::to_string(&entry).map_err(|e| py_err("serialize", e))
}
```

**À reproduire dans Sprint 7 Phase B** : strict clone avec
`CuratorList` + `CuratorListEntry::sign` + `DOMAIN_CURATOR_LIST_V1`.
Re-export dans `#[pymodule] fn nexus_core(...)` à côté des
`sign_claim`/`verify_claim_entry`.

### 3.4 `GossipClient` + `TopicHandle` Sprint 2 layer

**Pattern extrait** (lu dans
`crates/nexus-core-rs/src/gossip.rs`) :

- `GossipClient::new(node.gossip())` — sans lifetime (Sprint 2
  audit P1 fix)
- `join_topic([u8; 32], Vec<String>)` → `TopicHandle`
- `TopicHandle::broadcast(Vec<u8>)` + `next_event()` →
  `Option<GossipEvent>`
- `GossipEvent::Message { content, delivered_from }` est le
  variant à matcher

**À consommer dans Sprint 7 Phase C** : le daemon owns une task
tokio qui `join_topic(blake3("nexus-grid/curator/v1")[..32], vec![])`
+ loop `next_event().await`, parse chaque `content` comme
`{"list_hash": "hex", "ticket": "blobaa..."}`, appelle
`BlobsClient::fetch_ticket(endpoint, memory_lookup, ticket_str)`.

### 3.5 `BlobsClient::fetch_ticket` Sprint 2 layer

**Pattern extrait** (lu dans
`crates/nexus-core-rs/src/blobs.rs:134-157` et le test
`two_nodes_fetch_blob_via_ticket`) :

```rust
let blobs = BlobsClient::new(node.blobs_store());
let hash = blobs.fetch_ticket(
    node.endpoint(),
    node.memory_lookup(),
    &ticket_str,
).await?;
let body = blobs.get_bytes(hash).await?;
```

**À consommer dans Sprint 7 Phase C** : exact flow pour chaque
message gossip reçu. Body = `serde_json::from_slice::<CuratorListEntry>`,
puis `entry.verify_signature()?`, puis dédup par
`(curator_pubkey, revision)`.

### 3.6 iroh 0.97 `presets::N0` / pkarr discovery

**Pattern extrait** (lu dans `crates/nexus-core-rs/src/discovery.rs` +
le doc module) :

> iroh 0.97 `presets::N0` wires pkarr DHT discovery automatically.
> Every node we boot publishes its NodeAddr into the pkarr DHT
> and subscribes to lookups for node ids it tries to dial.

**À consommer dans Sprint 7 Phase D** : pas de publish explicite
en Sprint 7 (scope cut). Resolve = `Endpoint::lookup(EndpointId)`
— **vérifier l'API exacte 0.97 via context7** (query
`mcp__context7__query-docs` ID `/websites/rs_iroh`) en début de
Phase D. Si l'API a drifté, adapter le wrapper.

### 3.7 `axum 0.7` + `tower-http` CORS + loopback binding

Recherche context7 à faire en début Phase A (non urgent — axum
est stable depuis 2024). Défaut raisonnable : axum 0.7 avec
`TcpListener::bind("127.0.0.1:0")` pour port éphémère, récupérer
le port réel via `listener.local_addr()?.port()`. `tower-http`
`CorsLayer` avec `allow_origin(predicate)` acceptant
`"http://127.0.0.1:*"` + `"http://localhost:*"` uniquement (regex
impossible en tower-http, fallback sur closure).

### 3.8 `sysinfo` 0.32 pid liveness check

`System::new()` + `system.refresh_process(Pid::from(pid))` →
`Option<&Process>`. Alternative moins-dep : sur Unix
`unsafe { libc::kill(pid, 0) }` + check errno ESRCH ; sur Windows
`OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid)` +
`GetExitCodeProcess`. **Décision** : utiliser `sysinfo` (cross-platform
sans `unsafe`), coût dep raisonnable (~0.5 MB compilé).

---

## 4. Phase A — `nexus-shell-daemon-core` + `nexus-shell-daemon` crates

### 4.1 Fichiers ajoutés (library crate)

- `crates/nexus-shell-daemon-core/Cargo.toml`
  - `name = "nexus-shell-daemon-core"`, workspace inherits
  - deps: `nexus-core-rs` (path), `tokio`, `serde`, `serde_json`,
    `time`, `directories`, `tracing`, `anyhow`, `thiserror`,
    `dashmap`, `sysinfo`
  - **exclus délibérément** : `clap`, `ratatui`, `crossterm`,
    `axum`, `tower-http` (UI et HTTP vivent dans le binary crate)
  - `[dev-dependencies]` : `tempfile`
- `crates/nexus-shell-daemon-core/src/lib.rs` (~80 LOC)
  - doc module expliquant pourquoi headless-first (même ton que
    `nexus_worker_core/lib.rs`), `pub mod config;`, `pub mod paths;`,
    `pub mod registry;`, `pub mod state;`, `pub const VERSION: &str`,
    reprise des consts `PROJECT_*`
- `crates/nexus-shell-daemon-core/src/config.rs` (~120 LOC)
  - `struct ShellDaemonConfig { logging: LoggingConfig, network: NetworkConfig }`
  - `NetworkConfig { api_host: String, api_port: u16 }` — 
    `api_host` default `"127.0.0.1"`, `api_port` default `0`
    (éphémère)
  - `struct ShellDaemonPaths { root: PathBuf, config_file: PathBuf, log_dir: PathBuf, running_json: PathBuf }`
  - `impl ShellDaemonPaths::resolve(cli_config_override: Option<PathBuf>)` qui respecte
    `NEXUS_GRID_ROOT` env (pattern `nexus_worker_core::paths::nexus_grid_root`)
  - Tests : default config, env override, file override
- `crates/nexus-shell-daemon-core/src/paths.rs` (~40 LOC)
  - `pub fn nexus_grid_root() -> Option<PathBuf>` — **duplique**
    `nexus_worker_core::paths::nexus_grid_root` délibérément
    (pas de crosscrate dep)
  - `pub fn shell_daemon_dir() -> Option<PathBuf>` → `<root>/shell-daemon`
  - `pub fn running_json_path() -> Option<PathBuf>` → `<root>/shell-daemon/running.json`
  - Tests : path resolution avec et sans env override
- `crates/nexus-shell-daemon-core/src/registry.rs` (~200 LOC)
  - `pub const SCHEMA_VERSION: u32 = 1;`
  - `struct RunningState { schema_version: u32, node_id: String, api_host: String, api_port: u16, pid: u32, started_at: String }`
    (même shape que `nexus_coordinator::registry.RunningState` sauf
    `project_name` absent et `visibility` absent — un daemon est
    global per-user)
  - `pub fn write_running(state: &RunningState, path: &Path) -> Result<()>` atomic
    (pattern `state_writer.rs::serialize_to`)
  - `pub fn remove_running(path: &Path)` best-effort
  - `pub fn read_running(path: &Path) -> Option<RunningState>` best-effort
  - `pub fn check_stale_or_bail(path: &Path) -> Result<StaleOutcome>` where
    `enum StaleOutcome { NoFile, Stale { pid: u32 }, Live { pid: u32 } }`
    — utilise `sysinfo` pour le pid check
  - Tests : write/read/remove roundtrip, stale detection, live
    detection (via own pid), schema version constant stable
- `crates/nexus-shell-daemon-core/src/state.rs` (~120 LOC)
  - `pub struct DaemonStateSnapshot { schema_version: u32, node_id: String, daemon_version: String, uptime_secs: u64, started_at: String, last_updated_at: String, subscribed_curators: Vec<String>, known_lists: u32, known_browse_entries: u32 }`
  - `impl DaemonStateSnapshot::from_inputs(inputs: StateInputs)` builder
  - Tests : snapshot shape + schema_version stable

### 4.2 Fichiers ajoutés (binary crate)

- `crates/nexus-shell-daemon/Cargo.toml`
  - `name = "nexus-shell-daemon"`, `[[bin]]` entry
  - deps : `nexus-shell-daemon-core` (path), `nexus-core-rs` (path),
    `tokio`, `clap`, `tracing`, `tracing-subscriber`, `tracing-appender`,
    `axum`, `tower`, `tower-http` (features `cors`), `anyhow`,
    `thiserror`, `hex`, `serde_json`
  - **Deliberately excluded** : `ratatui`/`crossterm` (pas de TUI
    pour un daemon)
  - `[dev-dependencies]` : `tempfile`, cfg-unix `libc`
- `crates/nexus-shell-daemon/src/main.rs` (~120 LOC)
  - `#[tokio::main] async fn main() -> Result<()>` qui parse clap,
    resolve paths, init logging, dispatch sur subcommands
  - Subcommands : `Start { headless: bool }`, `Stop`, `Status`,
    `Config(ConfigCommand::{Get, Set})` — mêmes variants que
    nexus-worker adapté
- `crates/nexus-shell-daemon/src/cli.rs` (~180 LOC)
  - Clap derive, même structure que `nexus-worker/src/cli.rs`
  - Tests : `cli_definition_is_valid`, `parses_start`,
    `parses_stop`, `parses_status`, `parses_config_get_set`,
    `global_config_flag_attaches`
- `crates/nexus-shell-daemon/src/logging.rs` (~80 LOC)
  - **Clone direct** de `nexus-worker/src/logging.rs`
- `crates/nexus-shell-daemon/src/http.rs` (~250 LOC en Phase A
  minimal, +300 LOC en Phase C/D)
  - `fn build_router(state: Arc<DaemonState>) -> Router`
  - Phase A routes : `GET /health` renvoie
    `{"status":"ok","schema_version":1,"daemon_version":...}`,
    `GET /info` renvoie le `DaemonStateSnapshot` courant
  - `CorsLayer::new().allow_origin(predicate |origin, _| loopback_ok(origin))`
  - Tests : via `axum::http::Request::builder()` + `tower::ServiceExt::oneshot`
- `crates/nexus-shell-daemon/src/runtime.rs` (~180 LOC)
  - `struct DaemonRuntime { state: Arc<DaemonState>, iroh_node: Option<Node>, ... }`
  - `async fn start(paths, config) -> Result<DaemonRuntime>` qui :
    1. `check_stale_or_bail(&paths.running_json)?`
    2. Boot `nexus_core_rs::create_node().await?`
    3. Écrit `running.json`
    4. Bind TCP listener sur `(api_host, api_port)` → port éphémère
    5. Met à jour `running.json` avec le port réel
    6. Spawn axum `serve(listener, router)` dans une task
    7. Retourne le `DaemonRuntime` (ctrl-c handler à wire dans main.rs)
  - `async fn shutdown(self) -> Result<()>` ordered : stop HTTP,
    shutdown iroh, remove running.json
  - Tests unit : start-stop roundtrip avec `NEXUS_GRID_ROOT=<tmp>`
- `crates/nexus-shell-daemon/tests/e2e.rs` (~180 LOC)
  - Pattern `crates/nexus-worker/tests/cli_e2e.rs` : spawn le
    binaire compilé avec `--config <tmp>`, curl le `/health`,
    kill via ctrl-c (Unix) ou close (Windows)
  - Test `start_writes_running_json_and_responds_to_health`
  - Test `stop_removes_running_json`
  - Test `second_start_fails_when_first_still_running`

### 4.3 Fichiers modifiés

- `Cargo.toml` (workspace)
  - `members += ["crates/nexus-shell-daemon-core", "crates/nexus-shell-daemon"]`
  - `[workspace.dependencies]` += `axum = "0.7"`, `tower = "0.5"`,
    `tower-http = { version = "0.6", features = ["cors"] }`,
    `sysinfo = "0.32"`, `dashmap = "6"`

### 4.4 Critères d'acceptation Phase A

- `cargo build -p nexus-shell-daemon-core -p nexus-shell-daemon` clean
- `cargo test -p nexus-shell-daemon-core -p nexus-shell-daemon` green
- `cargo test --workspace` green (les 193 anciens restent verts)
- `cargo fmt --all --check` + `cargo clippy --workspace -- -D warnings` clean
- `nexus-shell-daemon start --headless` : boot ≤ 3 s, écrit
  `~/.nexus-grid/shell-daemon/running.json`, répond `/health → 200`
- `curl http://127.0.0.1:<port>/health` renvoie le JSON attendu
- Ctrl-C supprime `running.json` et shutdown iroh proprement
- Un second `start` refuse poliment (`daemon already running (pid N)`)
- Un `running.json` avec un pid mort est écrasé silencieusement

### 4.5 Commit Phase A

**Target** : `feat(shell-daemon): Sprint 7 Phase A — headless
daemon + HTTP skeleton`.

Estimation LOC : **~1900** (library ~1000 + binary ~900 incl tests).

## 5. Phase B — Curator list primitives Rust + PyO3

### 5.1 Fichiers ajoutés

- `crates/nexus-core-rs/src/curator.rs` *(nouveau ~340 LOC)*
  - Doc module ton identique à `task.rs`
  - `pub const CURATOR_LIST_FORMAT_VERSION: u16 = 1;`
  - `struct CuratorList { version, curator_pubkey, curator_name, created_at, revision, entries }`
  - `struct CuratorProjectRef { project_id, project_name, category, description }`
  - `impl CuratorList { pub fn new(...) -> Self }`
  - `struct CuratorListEntry { list, curator_pubkey, signature }` avec
    `#[serde(with = "BigArray")]` sur la signature
  - `impl CuratorListEntry::sign(list, keypair) -> Result<Self>`
  - `impl CuratorListEntry::verify_signature(&self) -> Result<()>` avec
    le double check attribution (`list.curator_pubkey ==
    self.curator_pubkey`) **et** longueur `entries ≤ 256`
    (protection DoS sur validation)
  - Tests (6 tests copiés du pattern `task.rs::tests`) :
    - `curator_entry_sign_and_verify`
    - `curator_entry_rejects_tampered_list`
    - `curator_entry_rejects_attribution_mismatch`
    - `curator_entry_rejects_wrong_signer`
    - `curator_entry_rejects_oversized_entries`
    - `curator_and_task_canonical_bytes_do_not_collide` (domain
      separation)

### 5.2 Fichiers modifiés

- `crates/nexus-core-rs/src/canonical.rs` — +`DOMAIN_CURATOR_LIST_V1: &[u8] = b"nexus-curator-list-v1";`
  + mise à jour du `//! # Why a domain prefix?` pour lister la
  nouvelle constante + test `different_domains_yield_different_bytes_for_curator`
- `crates/nexus-core-rs/src/lib.rs` — `pub mod curator;` +
  re-exports `CuratorList`, `CuratorListEntry`, `CuratorProjectRef`
- `crates/nexus-core-py/src/lib.rs` — +2 pyfunctions
  (`sign_curator_list` + `verify_curator_list_entry`, strict clone
  du pattern `sign_claim` lignes 877-898) + `m.add_function(wrap_pyfunction!(...)?)?;`
  dans le `#[pymodule]`
- `packages/nexus-sdk/tests/test_curator.py` *(nouveau ~120 LOC)*
  - `test_sign_and_verify_roundtrip`
  - `test_verify_rejects_tampered`
  - `test_verify_rejects_attribution_mismatch`
  - `test_canonical_bytes_cross_lang_stable` — freeze une curator
    list connue en JSON, signer en Python, vérifier que bytes
    canoniques Rust (via `verify_curator_list_entry`) l'acceptent

### 5.3 Critères d'acceptation Phase B

- Les 6 nouveaux tests Rust passent
- Les 4 nouveaux tests Python passent
- Une entry signée Python vérifie Rust-side et vice-versa
  (via wheel `nexus-core-py` recompilé)
- `cargo test --workspace` : **199 Rust tests** (193 + 6)
- `uv run pytest packages/nexus-sdk/tests/ -q` : **36 Python
  tests** SDK (32 + 4)

### 5.4 Commit Phase B

**Target** : `feat(core-rs,core-py,sdk): Sprint 7 Phase B —
curator list Ed25519 primitives + PyO3 bindings`.

Estimation LOC : **~600** (curator.rs + canonical.rs patch +
PyO3 + tests).

## 6. Phase C — Shell-daemon iroh runtime (gossip subscribe + fetch_ticket)

### 6.1 Fichiers ajoutés

- `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` *(nouveau ~380 LOC)*
  - `pub struct IrohRuntime { node: Node, curator_cache: Arc<DashMap<String, CuratorListEntry>>, attention: Arc<RwLock<HashSet<String>>> }`
  - `pub async fn start(state: Arc<DaemonState>) -> Result<IrohRuntime>` :
    1. `let node = nexus_core_rs::create_node().await?;`
    2. `let gossip = GossipClient::new(node.gossip());`
    3. `let topic_id = blake3::hash(b"nexus-grid/curator/v1").as_bytes().clone();`
    4. `let topic = gossip.join_topic(topic_id, vec![]).await?;`
    5. Spawn task `Self::recv_loop(node, topic, cache, attention)`
    6. Return IrohRuntime
  - `async fn recv_loop(...)` :
    - `while let Some(event) = topic.next_event().await? { match event { GossipEvent::Message { content, .. } => ... } }`
    - Parse `content` comme
      `struct Announcement { list_hash: String, ticket: String, curator_pubkey: String }` (JSON)
    - Filter par `attention.read().contains(&announcement.curator_pubkey)`
      (skip si pas subscribé)
    - `let hash = BlobsClient::new(node.blobs_store()).fetch_ticket(node.endpoint(), node.memory_lookup(), &announcement.ticket).await?`
    - `let body = blobs.get_bytes(hash).await?`
    - `let entry: CuratorListEntry = serde_json::from_slice(&body)?`
    - `entry.verify_signature()?`
    - Dédup : insert/update dans cache seulement si
      `entry.list.revision > existing.list.revision`
    - Log `info!(curator = %entry.curator_pubkey_hex, revision = %entry.list.revision, "curator list accepted")`
    - Tampered / bad sig → `warn!` et skip
  - `pub fn subscribe(&self, curator_pubkey_hex: String)` —
    ajoute au set d'attention
  - `pub fn unsubscribe(&self, curator_pubkey_hex: String)` —
    retire du set + drop l'entry du cache
  - `pub fn snapshot_curator_lists(&self) -> Vec<CuratorListEntry>` —
    clone les entries courantes
- Tests (via un helper 2-node test fixture, ~200 LOC de test) :
  - `curator_subscribe_receives_broadcast` — node A broadcast un
    message announcement, node B subscribe et reçoit l'entry
  - `curator_list_revision_dedup` — node A broadcast v1 puis v2,
    node B ne garde que v2
  - `curator_list_tampered_rejected` — tamper la signature,
    subscriber skip silencieusement (log warning capturable)
  - `curator_list_wrong_curator_filtered` — subscribe à curator X,
    recevoir un message pour curator Y → ignore

### 6.2 Fichiers modifiés

- `crates/nexus-shell-daemon-core/src/lib.rs` — `pub mod iroh_runtime;`
- `crates/nexus-shell-daemon-core/Cargo.toml` — +`nexus-core-rs`
  déjà présent, +`blake3 = { workspace = true }` (ajouter à workspace)
- `crates/nexus-shell-daemon/src/runtime.rs` — le `DaemonRuntime`
  wraps `IrohRuntime` et retourne une ref pour les handlers HTTP
- `crates/nexus-shell-daemon/src/http.rs` — +3 routes :
  - `GET /curators` → retourne `{curator_lists: [CuratorListEntry...]}`
    (schema_version 1)
  - `POST /curators/subscribe` → body
    `{curator_pubkey: "hex"}`, appelle `iroh_runtime.subscribe()`,
    200 empty
  - `DELETE /curators/{pubkey}` → appelle `unsubscribe()`, 200 empty
  - Tests via `tower::ServiceExt` avec un `DaemonRuntime` stubé
    (`iroh_runtime` mocké via un trait `CuratorListStore`)

### 6.3 Critères d'acceptation Phase C

- Les 4 tests iroh_runtime 2-node passent en ≤ 10 s cumulés
- Les tests HTTP curator stub passent
- `nexus-shell-daemon start` boot iroh + subscribe au topic
  curator v1 sans erreur (check via log)
- `curl -X POST localhost:<port>/curators/subscribe -d '{"curator_pubkey":"<hex>"}' -H 'content-type:application/json'`
  retourne 200, et après un broadcast adjacent le `GET /curators`
  retourne l'entry

### 6.4 Commit Phase C

**Target** : `feat(shell-daemon): Sprint 7 Phase C — gossip
subscribe + fetch_ticket curator pipeline`.

Estimation LOC : **~900** (iroh_runtime ~380 + http ext ~200 +
tests ~320).

## 7. Phase D — Pkarr DHT browse

### 7.1 Fichiers ajoutés

- `crates/nexus-shell-daemon-core/src/browse.rs` *(nouveau ~220 LOC)*
  - `pub struct BrowseEntry { project_id, project_name, category, description, curator_pubkey, first_seen_at, last_checked_at, status }`
  - `pub enum BrowseStatus { Reachable, Unreachable, Unknown }`
  - `pub struct Browser { node: Arc<Node>, lookup_cache: Arc<DashMap<String, BrowseEntry>> }`
  - `pub async fn refresh_from_curators(&self, entries: &[CuratorListEntry])` :
    pour chaque `CuratorProjectRef`, tenter `node.endpoint().lookup(EndpointId::from_str(&project_id)?).await`
    (API iroh 0.97 à confirmer via context7 au démarrage Phase D)
  - `pub fn snapshot(&self) -> Vec<BrowseEntry>`
  - **Fallback gracieux** : si l'API `lookup` n'existe pas verbatim
    en 0.97 (drift possible), utiliser `Endpoint::connect(EndpointId)`
    + catch timeout ≤ 2 s = Unreachable, connection success =
    Reachable + immediate `disconnect`
- Tests :
  - `browse_refresh_marks_reachable_on_connect` — 2 nodes,
    l'un crée l'autre dans sa curator list, après refresh l'entry
    apparaît `Reachable`
  - `browse_refresh_marks_unreachable_on_timeout` — project_id
    fabriqué jamais booté

### 7.2 Fichiers modifiés

- `crates/nexus-shell-daemon-core/src/lib.rs` — `pub mod browse;`
- `crates/nexus-shell-daemon/src/http.rs` — +1 route :
  - `GET /browse` → retourne `{entries: [BrowseEntry...]}` (shape
    Zod-compatible, `schema_version: 1`)
- `crates/nexus-shell-daemon/src/runtime.rs` — spawn un
  refresh loop périodique (default: 60 s) qui itère sur le
  curator cache et rafraîchit le browser

### 7.3 Critères d'acceptation Phase D

- Les 2 tests browse passent
- `GET /browse` renvoie un tableau avec les statuts attendus
  après un subscribe curator + refresh tick
- Pas de publish pkarr (scope cut §6 kickoff)

### 7.4 Commit Phase D

**Target** : `feat(shell-daemon): Sprint 7 Phase D — pkarr DHT
browse resolution`.

Estimation LOC : **~400** (browse.rs + tests + http ext).

## 8. Phase E — Coordinator proxy + Web pages câblées

### 8.1 Fichiers ajoutés (Python coordinator)

- `packages/nexus-coordinator/src/nexus_coordinator/api/daemon.py` *(nouveau ~200 LOC)*
  - `router = APIRouter(prefix="/daemon", tags=["daemon"])`
  - Helper `_daemon_base_url() -> str` qui lit
    `paths.shell_daemon_registry_path()`, parse le JSON, retourne
    `f"http://{r.api_host}:{r.api_port}"`. Raise `HTTPException(503, detail="shell-daemon not running")` si
    pas de running.json ou pid mort (via `psutil` — déjà transitive
    dep du coord ? sinon add)
  - `GET /daemon/info` proxy → retourne le `DaemonStateSnapshot`
  - `GET /daemon/curators` proxy
  - `POST /daemon/curators/subscribe` proxy avec `request.json()`
  - `DELETE /daemon/curators/{pubkey}` proxy
  - `GET /daemon/browse` proxy
  - Helper interne `_proxy(method, path, body=None)` qui wrap
    `httpx.AsyncClient(timeout=5.0)` et forward verbatim
- `packages/nexus-coordinator/tests/test_daemon_proxy.py` *(nouveau ~180 LOC)*
  - Fixture qui mock `paths.shell_daemon_registry_path()` avec un
    running.json pointant sur un mini HTTP server respond-fixture
  - `test_daemon_info_proxies_snapshot`
  - `test_daemon_curators_list_passthrough`
  - `test_daemon_subscribe_forwards_body`
  - `test_daemon_missing_running_json_returns_503`
  - `test_daemon_stale_pid_returns_503`
  - `test_daemon_upstream_error_bubbles`

### 8.2 Fichiers modifiés (Python coordinator)

- `packages/nexus-coordinator/src/nexus_coordinator/paths.py` —
  `def shell_daemon_registry_path() -> Path:
      return nexus_grid_root() / "shell-daemon" / "running.json"`
- `packages/nexus-coordinator/src/nexus_coordinator/api/app.py` —
  `include_router(daemon_router)`
- `packages/nexus-coordinator/pyproject.toml` — éventuellement
  add `psutil >= 6` si pas déjà transitive

### 8.3 Fichiers ajoutés (frontend)

- `web/src/api/daemon.ts` *(nouveau ~300 LOC)*
  - Zod schemas stricts (`.strict()`) :
    - `DaemonInfoSchema` (match `DaemonStateSnapshot` wire)
    - `CuratorProjectRefSchema`
    - `CuratorListSchema`
    - `CuratorListEntrySchema` (avec `signature: z.string().regex(/^[0-9a-f]{128}$/)` hex 64 bytes)
    - `BrowseEntrySchema` + `BrowseStatusSchema` enum
    - `DaemonCuratorsResponseSchema` wrapper avec `schema_version: z.literal(1)`
    - `DaemonBrowseResponseSchema` idem
  - `class DaemonUnreachableError extends Error` (classifie le
    503 spécifique pour que les pages distinguent
    daemon-offline vs coordinator-offline)
  - `async getDaemonInfo(coordUrl)`, `listCurators(coordUrl)`,
    `subscribeCurator(coordUrl, pubkey)`, `unsubscribeCurator(coordUrl, pubkey)`,
    `listBrowse(coordUrl)`
- `web/src/components/browse/DaemonOfflineBanner.tsx` *(nouveau ~60 LOC)*
  - Card shadcn "Le shell-daemon n'est pas démarré" + code snippet
    `nexus-shell-daemon start` + bouton "Réessayer" qui invalide
    les queries
- `web/src/components/curators/AddCuratorDialog.tsx` *(nouveau ~140 LOC)*
  - Dialog shadcn + `<Input id="curator-pubkey">`, validation hex
    64 char, submit appelle `subscribeCurator`

### 8.4 Fichiers modifiés (frontend)

- `web/src/pages/Browse.tsx` — rewrite complet (~180 LOC):
  - `useQuery({queryKey: ["browse", coordUrl], queryFn: () => listBrowse(coordUrl), refetchInterval: 15000})`
  - Render en cards shadcn listant chaque `BrowseEntry`, badge
    par `status`, filter dropdown par `category` (simple
    `<select>`, pas de cmdk)
  - Empty state → "Aucun projet découvert", CTA "S'abonner à un
    curator depuis /curators"
  - Error state → `DaemonOfflineBanner` si `DaemonUnreachableError`,
    sinon card générique "Erreur coordinator"
- `web/src/pages/Curators.tsx` — rewrite complet (~200 LOC):
  - `useQuery` sur `listCurators`
  - Render TabView-style : pour chaque `CuratorListEntry`, card
    avec `CuratorList.curator_name`, revision, created_at relative
    via `formatRelativeTime`, list des `entries` en bullets
  - Bouton `<AddCuratorDialog>` trigger
  - Bouton "Retirer" par entry avec `useMutation` sur
    `unsubscribeCurator` + invalidate
  - Error / empty states identiques à Browse
- `web/src/components/AppShell.tsx` — **inchangé**
- `web/src/App.tsx` — **inchangé** (routes `/browse` + `/curators`
  existent déjà)

### 8.5 Tests Phase E

- `packages/nexus-coordinator/tests/test_daemon_proxy.py` — 6 tests
- `web/src/api/daemon.test.ts` (Vitest) — ~8 tests de parsing Zod :
  - `parse daemon info`, `parse curator list with section`,
    `rejects drift field`, `rejects bad signature hex`,
    `parse browse entries all statuses`, `parse empty curators`,
    `DaemonUnreachableError from 503`
- `web/tests/curators-subscribe.spec.ts` (Playwright) — spawn
  daemon stub (fake HTTP responder jouant le rôle du vrai daemon
  pour déterminisme), subscribe/unsubscribe flow end-to-end
- `web/tests/browse-daemon-offline.spec.ts` (Playwright) —
  pas de running.json → page affiche DaemonOfflineBanner, bouton
  "Réessayer" retry

### 8.6 Critères d'acceptation Phase E

- Les 6 tests pytest `test_daemon_proxy.py` passent
- Les 8 tests Vitest `daemon.test.ts` passent
- Les 2 tests Playwright passent
- `cd web && npm run test:coverage` : coverage scope élargi à
  `src/api/daemon.ts` (seuil : ≥90% lines — consistent avec
  `format.ts` / `projectStore.ts`)
- `cd web && npm run build && npm run size` : budgets verts
  (main ≤ 475, vendor-react ≤ 210, vendor-ui ≤ 50, css ≤ 100)
- Le shell affiche vraiment une curator list après subscribe via
  la palette (Sprint 6 Ctrl+K) "Ajouter un curator" et un
  project apparaît dans `/browse` après refresh tick
- `bash web/scripts/scan-en-strings.sh` clean (tous les strings
  new sont fr)

### 8.7 Commit Phase E

**Target** : `feat(coordinator,web): Sprint 7 Phase E — /daemon
proxy + Browse/Curators pages live`.

Estimation LOC : **~1500** (coord proxy ~400 + web rewrite ~900
+ tests ~200).

## 9. Phase F — Sortie de sprint

### 9.1 Livrables obligatoires

- `.planning/sprint7_verification.md` — self-report fail-fast
  checklist ≥24 rows (format exact Sprint 6)
- `.planning/sprint7_audit_plan.md` — 9 tracks minimum avec
  objectifs + méthodes, prêt à être joué par une session fraîche
  Sprint 8 Phase 0. Tracks cibles :
  - **A** — Crypto / canonical bytes (curator list cross-lang)
  - **B** — Curator verify resilience (tampered/DoS/edge sizes)
  - **C** — Shell-daemon HTTP robustness (CORS, 503, singleton)
  - **D** — Singleton enforcement (stale pid cases, race conditions)
  - **E** — Pkarr resolve correctness (reachable/unreachable/timeout)
  - **F** — Shell UX degraded states (daemon offline, empty, error)
  - **G** — Sprint 8 risk assumptions (task_submit impl ready,
    @nexus_command impl ready)
  - **H** — Dependencies + security audit (axum/tower-http/sysinfo/dashmap
    npm/cargo audit)
  - **I** — Documentation coherence (kickoff ↔ plan ↔ verification
    triangulation, commit message fidelity)
- `docs/shell/PATTERNS.md` — +P9 (daemon HTTP proxy via coordinator
  pattern, PAS de direct daemon calls depuis le shell) + update
  T4 / T5 status "signature gelée Sprint 7 Day 0, impl Sprint 8
  Phase A" + éventuelle nouvelle T8..T? si des scope cuts Phase
  A..E apparaissent
- `docs/rust/PATTERNS.md` — section "Sprint 7 canonical" avec :
  - `DOMAIN_CURATOR_LIST_V1` ajouté à la liste des domaines
  - Pattern `CuratorListEntry` (attribution match double-check,
    `entries ≤ 256` DoS guard, `revision` LWW dedup)
  - Pattern `blake3("nexus-grid/curator/v1")` topic convention
    + rappel "v1 unique global, namespacing ajourné v1.2"
  - Pattern `DashMap<curator_pubkey, CuratorListEntry>` RAM-only
    + rappel "persist SQLite daemon = tech debt T?"
- `MEMORY.md` → `nexus_grid_pivot.md` mise à jour avec "Sprint 7
  CLOSED tip `<SHA>`, Phase F livrables en place, Sprint 8 Phase 0
  peut jouer sprint7_audit_plan.md"

### 9.2 Scan final

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `uv run ruff format --check packages/ examples/`
- `uv run ruff check packages/ examples/`
- `uv run pytest packages/ -q`
- `cd web && npx tsc --noEmit -p tsconfig.app.json`
- `cd web && npm run lint`
- `cd web && npm run test:unit`
- `cd web && npm run test:coverage`
- `cd web && npm run build && npm run size`
- `cd web && npx playwright test`
- `cd web && npm audit --omit=dev --audit-level=high`
- `bash web/scripts/scan-en-strings.sh`
- `grep -rn 'TODO(Sprint7)' crates/ packages/ web/` → **0 match**
  (aucun TODO en suspens)

### 9.3 Commit Phase F

**Target** : `docs(sprint7): verification + audit plan for Sprint 8`.

## 10. Fail-fast checklist (cible Sprint 7)

| # | Row | Commande | Attendu |
|---|---|---|---|
| 1 | shell-daemon-core builds | `cargo build -p nexus-shell-daemon-core --locked` | exit 0 |
| 2 | shell-daemon builds | `cargo build -p nexus-shell-daemon --locked` | exit 0 |
| 3 | shell-daemon-core unit tests | `cargo test -p nexus-shell-daemon-core --locked` | all pass |
| 4 | shell-daemon bin+e2e tests | `cargo test -p nexus-shell-daemon --locked` | all pass |
| 5 | Curator primitives sign/verify | `cargo test -p nexus-core-rs curator::` | ≥ 6 passed |
| 6 | Domain separation for curator | `cargo test -p nexus-core-rs canonical` | all pass incl curator domain |
| 7 | PyO3 curator sign/verify | `uv run pytest packages/nexus-sdk/tests/test_curator.py -q` | ≥ 4 passed |
| 8 | Cross-lang curator canonical | (inside `test_curator.py`) | entry signed Python verifies Rust-side |
| 9 | iroh_runtime 2-node subscribe | `cargo test -p nexus-shell-daemon-core iroh_runtime::` | ≥ 4 passed |
| 10 | Browse pkarr resolve | `cargo test -p nexus-shell-daemon-core browse::` | ≥ 2 passed |
| 11 | All Rust tests unchanged suites | `cargo test --workspace --locked` | ≥ 210 (193 + 6 curator + 4 iroh + 2 browse + ~5 daemon core unit) |
| 12 | Coordinator /daemon proxy | `uv run pytest packages/nexus-coordinator/tests/test_daemon_proxy.py -q` | ≥ 6 passed |
| 13 | All coordinator tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | ≥ 53 + 1 skip |
| 14 | All SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | ≥ 36 |
| 15 | All app-gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | 3 pass (inchangé) |
| 16 | cargo fmt + clippy | `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| 17 | ruff format + check | `uv run ruff format --check packages/ examples/ && uv run ruff check packages/ examples/` | exit 0 |
| 18 | tsc strict | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 |
| 19 | ESLint | `cd web && npm run lint` | 0 err, ≤ 5 T1 warnings |
| 20 | Vite build | `cd web && npm run build` | exit 0, no warnings |
| 21 | size-limit budgets | `cd web && npm run size` | main ≤ 475, vendor-react ≤ 210, vendor-ui ≤ 50, css ≤ 100 |
| 22 | Vitest unit tests | `cd web && npm run test:unit` | ≥ 107 passing (99 + 8 daemon.ts) |
| 23 | Vitest coverage thresholds | `cd web && npm run test:coverage` | lines ≥ 90, funcs ≥ 90, branches ≥ 85, stmts ≥ 90 |
| 24 | Playwright curator + daemon-offline | `cd web && npx playwright test` | ≥ 12 passed (10 + 2 nouveaux) |
| 25 | French-only | `bash web/scripts/scan-en-strings.sh` | exit 0 |
| 26 | Singleton enforced | `nexus-shell-daemon start` twice → second fails | "daemon already running (pid N)" |
| 27 | Daemon-offline UX | `/browse` while daemon stopped → DaemonOfflineBanner | visible + retry button |
| 28 | npm audit | `cd web && npm audit --audit-level=high` | 0 vulns high/critical |
| 29 | PATTERNS.md P9 + T4/T5 updated | `grep -q "P9" docs/shell/PATTERNS.md && grep -q "Sprint 7" docs/shell/PATTERNS.md` | exit 0 |
| 30 | sprint7_audit_plan.md exists | `test -f .planning/sprint7_audit_plan.md` | exit 0 |

**30 rows** — 24+ demandé, on livre 30 pour couvrir toutes les
surfaces neuves + les régressions possibles.

## 11. Git plan

Commits cibles sur master (atomiques par phase, pattern Sprint 6) :

1. `docs(sprint7): kickoff + plan`
2. `feat(shell-daemon): Sprint 7 Phase A — headless daemon + HTTP skeleton`
3. `feat(core-rs,core-py,sdk): Sprint 7 Phase B — curator list Ed25519 primitives + PyO3 bindings`
4. `feat(shell-daemon): Sprint 7 Phase C — gossip subscribe + fetch_ticket curator pipeline`
5. `feat(shell-daemon): Sprint 7 Phase D — pkarr DHT browse resolution`
6. `feat(coordinator,web): Sprint 7 Phase E — /daemon proxy + Browse/Curators pages live`
7. `docs(sprint7): verification + audit plan for Sprint 8`

Total target : **7 commits** (une variante possible : split
Phase A en A1 core + A2 binary si les LOC font un commit trop
gros, mais viser 1 commit / phase). Si un fix post-phase est
nécessaire (pattern Sprint 2 `de9589d` + `ed2ea76`), commit
séparé `fix(shell-daemon): ...` entre les phases concernées.

## 12. Scope cuts (à respecter strictement)

Répétition de `sprint7_kickoff.md` §6 pour exécution :

- **Pas de bootstrap peers VPS** — Sprint 10
- **Pas de publish pkarr** — Sprint 10
- **Pas d'implémentation `AppContext.submit_task`** — Sprint 8 Phase A
- **Pas d'implémentation `@nexus_command`** — Sprint 8 Phase A
- **Pas de migration d'un tab gov** — Sprint 8
- **Pas d'extension `AppContext.storage` / `.events`** — Sprint 8
- **Pas d'Unix socket / named pipe** — D1 figé
- **Pas de multi-instance daemon** — D2 figé
- **Pas de topic gossip namespacé** — D3 figé
- **Pas de persist SQLite des curator lists côté daemon** — RAM only
- **Pas de browse filter / search UI** — Sprint 8/9
- **Pas d'icônes dynamiques par curator** — fixe
- **Pas de multi-writer iroh-docs** — Sprint 10+
- **Pas de re-signature cross-révision** — nouvelle entry complète
  à chaque bump
- **Pas de TUI dans `nexus-shell-daemon`** — headless only (les
  ratatui/crossterm ne sont pas listés dans `Cargo.toml` binary)
- **Pas de ré-introduction de reagraph / leaflet / D3** dans
  Browse/Curators — shadcn cards + TabView-style rendering only
- **Pas d'auth sur le proxy daemon** — loopback only, CORS strict,
  même modèle de confiance que coordinator loopback

## 13. Risks

- **R1 — iroh 0.97 `Endpoint::lookup` API drift** : le code
  discovery.rs mentionne "resolve(node_id) pour proactive peer
  lookup" comme "Sprint 4 will add" mais n'est pas encore câblé.
  Mitigation : Phase D démarre par un `mcp__context7__query-docs`
  sur `/websites/rs_iroh` pour valider la signature exacte. Si
  absent, fallback sur `Endpoint::connect` + timeout 2 s =
  probe, ce qui reste sémantiquement équivalent pour `BrowseStatus`.
- **R2 — axum 0.7 vs 0.8** : axum 0.8 a introduit des breakings
  (`Handler` trait lifetime). Mitigation : pin `axum = "0.7.9"`
  dans workspace + lock la version. Si l'écosystème pousse 0.8,
  Sprint 9 traitera l'upgrade.
- **R3 — `sysinfo` pid check incorrect Windows** : `sysinfo`
  peut marquer un pid réutilisé comme "alive" si la PID a été
  recyclée par l'OS. Mitigation : comparer aussi le nom du
  processus (`process.name() == "nexus-shell-daemon"`) avant de
  refuser le boot. Tests e2e couvrent ce cas Phase A.
- **R4 — `httpx` timeout Phase E coordinator proxy** : un daemon
  lent (gros cache) ferait timer out le proxy. Mitigation :
  `httpx.AsyncClient(timeout=httpx.Timeout(connect=2.0, read=10.0))`
  + tests qui vérifient la bonne erreur 504 remontée au shell.
- **R5 — Curator list DoS par `entries` massif** : un attaquant
  publierait une `CuratorList` avec 1M d'entries. Mitigation :
  `entries.len() > 256` → rejet pendant `verify_signature()` en
  Phase B. Test dédié.
- **R6 — Revision rollback attack** : un curator rejoue une
  ancienne révision. Mitigation : LWW par `revision` + dedup,
  jamais d'écriture si `new.revision ≤ current.revision`. Test
  dédié Phase C.
- **R7 — `DashMap` cache RAM-only perte au restart** : un user
  qui restart le daemon perd toutes ses subscriptions. Mitigation
  acceptée Sprint 7 (scope cut) ; le set d'attention
  (`HashSet<curator_pubkey>` des curators subscribés) **est**
  persisté à `~/.nexus-grid/shell-daemon/subscriptions.json`
  (~30 LOC atomic write), et re-chargé au boot. Les listes
  elles-mêmes re-arrivent via gossip.
- **R8 — Playwright Phase E avec daemon stub** : le stub HTTP
  doit simuler les 5 endpoints daemon. ~120 LOC Node stub.
  Alternative : lancer un vrai daemon compilé en `globalSetup`
  comme le coordinator Sprint 5. **Retenu** : vrai daemon via
  globalSetup, plus fidèle et déjà la convention.
- **R9 — Scan-en-strings flag les nouveaux strings de
  `BrowseStatus`** : "Reachable"/"Unreachable"/"Unknown" sont
  anglais. Mitigation : mapper en fr côté React (dict
  `{reachable: "Accessible", unreachable: "Injoignable",
  unknown: "Inconnu"}`), les strings Rust restent techniques
  (exclus du scan via le pattern existant `crates/` exclude).

## 14. Checkpoint de clôture Sprint 7

Sprint 7 est **fermé** quand :

1. Fail-fast §10 : 30/30 vert
2. `git log --oneline master ^2926383` affiche 7-10 commits
   (avec éventuels `fix(shell-daemon): ...` ou `fix(core-rs): ...`
   post-phase)
3. `.planning/sprint7_verification.md` commité et lisible
4. `.planning/sprint7_audit_plan.md` commité et lisible (obligatoire
   `sprint_audit_gate.md`)
5. `docs/shell/PATTERNS.md` contient P9 et marque T4/T5 "frozen
   Sprint 7, impl Sprint 8"
6. `docs/rust/PATTERNS.md` contient la section "Sprint 7 canonical"
7. Aucun `TODO(Sprint7)` dans le code
8. `MEMORY.md` `nexus_grid_pivot.md` mis à jour avec le tip
   Sprint 7 + transition vers Sprint 8

Après fermeture : **rien**. Sprint 8 ouvrira avec sa propre
Phase 0 audit de Sprint 7 (session fraîche jouant
`sprint7_audit_plan.md`). Pas d'écriture préemptive d'un
`sprint8_kickoff.md` — ça violerait le pattern audit gate.
