# Sprint 33 — Research multi-nœuds (pre-kickoff)

**Écrit** : 2026-04-27 (post-S32 Phase B, pre-S32 Phase C)
**Source** : 3 agents parallèles (deploy-readiness, process-analysis, cross-compile)
**Consommé par** : S33 kickoff §Sources research

---

## 1. État actuel — jamais testé multi-nœuds

Le projet a ~1877 tests, tous single-machine. Aucun test ne spawn
2 daemons, aucun test ne vérifie discovery pkarr cross-réseau,
aucun test ne pipe une tâche d'un nœud à l'autre. Les e2e.rs
existants (`nexus-shell-daemon/tests/e2e.rs`, `nexus-worker/tests/e2e.rs`)
spawnent un binaire isolé — pattern transposable à 2 nœuds.

---

## 2. Architecture de déploiement multi-nœuds

```
VPS 1 (Linux)     : Coordinator (--host 0.0.0.0 --port 8765) + pkarr publish
VPS 2 (Linux)     : Worker headless (join via invite nx1...)
PC Windows (dev)  : Daemon + Coordinator + Worker GPU (Ollama RTX 5080)
Mac               : Worker (test cross-platform)
iPhone/Android    : Browser → shell React via http://<VPS1>:8765
```

### Binaires à déployer par machine

| Machine | nexus-shell-daemon | nexus-coordinator | nexus-worker | nexus-launcher |
|---|---|---|---|---|
| VPS 1 | ✅ | ✅ (FastAPI) | optionnel | non |
| VPS 2 | ✅ | non | ✅ | non |
| PC Windows | ✅ | ✅ | ✅ | ✅ |
| Mac | ✅ | non | ✅ | optionnel |
| Mobile | — | — | — | — (browser) |

---

## 3. Findings deploy-readiness

### 3.1 Daemon (nexus-shell-daemon)

- Entry : `crates/nexus-shell-daemon/src/main.rs`
- CLI : `start [--headless]`, `stop`, `status`, `canary publish/verify/frost`
- Config : `~/.nexus-grid/shell-daemon/config.toml` → `[network] api_host, api_port`
- Défaut : `127.0.0.1:0` (port éphémère loopback)
- Override VPS : `api_host = "0.0.0.0"`, `api_port = 9000`
- Multi-instance : possible via `NEXUS_GRID_ROOT` env différent par instance
- `running.json` écrit au boot avec host+port réel
- UDS (Unix) : `~/.sbfb/run/daemon.sock` mode 0600
- Named Pipes (Windows) : `\\.\pipe\sbfb-daemon` DACL user-only

### 3.2 Worker (nexus-worker)

- Entry : `crates/nexus-worker/src/main.rs`
- CLI : `register [--name]`, `start [--tui|--headless|--stub-ollama]`, `join <invite>`, `projects list/enable/disable`
- Config Linux : `~/.config/nexus-grid/worker.toml`
- Config macOS : `~/Library/Application Support/dev.FlowUP.nexus-grid/worker.toml`
- Config Windows : `%APPDATA%\FlowUP\nexus-grid\config\worker.toml`
- Keypair : `<data_dir>/worker.key` (Ed25519, généré par `register`)
- Allowlist : `<data_dir>/allowlist.sqlite3` (rusqlite, schéma v1/v2)
- Join flow : `nexus-worker join nx1...` → decode invite → verify sig → check expiry → enroll SQLite

### 3.3 Coordinator (nexus-coordinator)

- Entry : `packages/nexus-coordinator/src/nexus_coordinator/cli/commands/start.py`
- CLI : `nexus-coordinator start <name> [--host HOST] [--port PORT]`
- Config : `~/.nexus-grid/<project>/coordinator.toml`
- Env override : `NEXUS_COORD__NETWORK__API_HOST=0.0.0.0`
- Défaut : `127.0.0.1:8765`
- VPS : `--host 0.0.0.0 --port 8765`
- CORS : regex `^https?://(127\.0\.0\.1|localhost)(:\d+)?$` — BLOQUANT pour accès externe !
- Bearer token : obligatoire sur HTTP, skippé sur UDS (PeerCredsVerified)
- iroh Node : keypair persistent, Doc namespace stable

### 3.4 Discovery (pkarr DHT + relay)

- `crates/nexus-core-rs/src/discovery.rs` : iroh 0.98 preset N0 active pkarr auto
- Chaque nœud publie `NodeAddrInfo` (node_id + relay URL + socket addrs)
- Relay config : `SBFB_CUSTOM_RELAYS` env > `~/.sbfb/relays.json` > défaut n0 (NA/EU/AP)
- Pkarr relay auto-hébergé : `docker/pkarr-relay/` (Dockerfile + config.toml, port 6881)
- TLS pinning relay : `~/.sbfb/relay-pins.json` (SPKI SHA-256, hot-reload notify)

### 3.5 Invite flow

- Mint : coordinator signe payload v2 (project_id, coordinator_pubkey, coordinator_addr, tasks_doc_ticket, scope, expires_at)
- Encode : `nx1` + Base32 RFC4648 (~200-400 chars)
- Join : worker decode → verify Ed25519 sig → check expiry → enroll allowlist SQLite
- Scopes : Worker (default, claime tasks) ou Observer (read-only)

### 3.6 Blockers identifiés pour multi-nœuds

1. **CORS coordinator** : regex localhost-only → accès browser externe bloqué.
   Fix requis S33 : ajouter option `--cors-origin` ou `NEXUS_COORD__NETWORK__CORS_ORIGINS`.
2. **Bearer token** : généré localement → worker distant n'a pas le token.
   Flow actuel : invite contient coordinator_addr, worker dial via iroh (pas HTTP).
   OK pour iroh-docs, mais l'API HTTP coordinator reste inaccessible de l'extérieur sans token.
3. **Systemd/Docker** : aucune unit/Dockerfile pour daemon/worker/coordinator.
   Templates à écrire S33.
4. **`stop`/`status` CLI** : stubs non implémentés. Graceful shutdown = Ctrl+C only.

---

## 4. Findings cross-compile

### 4.1 CI existante

- `rust-ci.yml` : fmt + clippy + tests sur 3 OS (ubuntu/windows/macos-14)
- `build-worker.yml` : nexus-worker sur 7 cibles (dont 4 via `cross`)
- `release.yml` : 3 binaires natifs (ubuntu/macos/windows) + SLSA cosign

### 4.2 Recommandation : build natif, pas cross-compile

- VPS Linux : `ssh vps && git pull && cargo build --release`
- Mac : `git pull && cargo build --release`
- Windows : build natif local (déjà le cas)
- Pas de cross-compile Windows→Linux ni Windows→macOS

### 4.3 Deps platform-specific (toutes gérées)

| Dep | Strategy | Status |
|---|---|---|
| keyring (OS keyring) | Runtime probe, skip si absent | ✅ |
| nvml-wrapper (NVIDIA) | dlopen runtime, fail-open | ✅ |
| Named Pipes | `#[cfg(windows)]` module entier | ✅ |
| UDS / SO_PEERCRED | `#[cfg(unix)]` module entier | ✅ |
| libsystemd | `[target.'cfg(target_os = "linux")']` | ✅ |
| oslog | `[target.'cfg(target_os = "macos")']` | ✅ |
| rusqlite bundled | Compile SQLite C depuis sources | ✅ |

### 4.4 Gap identifié

`build-worker.yml` ne couvre que nexus-worker. Daemon et launcher absents.
→ Fusionner en `build-binaries.yml` (3 binaires × 7 targets) dans S33.

---

## 5. Findings process analysis

### 5.1 Fail-fast checklist actuelle : 29 rows, 0 multi-nœuds

Rows candidates à ajouter (permanentes) :

```
| 30 | 2-daemon localhost smoke | scripts/test-multi-node.sh | both HTTP respond | |
| 31 | Cross-node discovery     | pkarr resolve node2 from node1 | peer found     | |
| 32 | Cross-node blob transfer | publish app node1 → fetch node2 | hash match     | |
| 33 | Cross-node task pipe     | submit task node1 → result node2 | result valid  | |
```

Row 30 = minimal (S33 Phase A). Rows 31-33 = complet (S33 Phase B+).

### 5.2 Quand exécuter

| Trigger | Scope |
|---|---|
| Phase touche gossip/blobs/discovery/wire format | MANDATORY rows 30-33 |
| Sprint pair phase dette | Row 30 minimum |
| Phase pure frontend/docs | Skip (comme Playwright quand pas de change web) |
| Audit gate Phase 0 | Row 30 systématique |

### 5.3 Crate nexus-test-harness (long-terme)

```rust
pub struct DaemonCluster { nodes: Vec<DaemonHandle> }
pub struct DaemonHandle { proc: Child, root: TempDir, http_port: u16 }

impl DaemonCluster {
    pub async fn spawn(n: usize) -> Self { ... }
    pub async fn await_gossip_sync(&self, topic: &str, timeout: Duration) -> Result<()> { ... }
    pub async fn submit_task_cross_daemon(&self, from: usize, to: usize) -> Result<()> { ... }
}
```

Tests : `test_gossip_multi_node.rs`, `test_blobs_discovery.rs`,
`test_project_announcement_sync.rs`, `test_task_cross_daemon.rs`.

### 5.4 Impact audit gate

Nouveau track dans audit_plan S33+ :

```
Track X — Multi-node (triggered si P2P code change)
- Row 30 smoke test exécuté ?
- Gossip sync validé cross-daemon ?
- Carry S+1 si coverage partielle
```

---

## 6. Deps VPS à installer

### Ubuntu/Debian
```bash
sudo apt update && sudo apt install -y \
  build-essential pkg-config libssl-dev libdbus-1-dev git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

### macOS
```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

### Python (coordinator)
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
# ou pip install uv
```

---

## 7. Proposition D1..D5 Sprint 33

**D1** : Build strategy — natif par machine (SSH + cargo build), pas cross-compile.
Fusionner CI en `build-binaries.yml` (3 binaires × 7 targets).

**D2** : Script d'installation (`scripts/install-node.sh`) — détecte OS,
installe Rust + deps, clone repo, build, génère keypair, crée systemd unit
(Linux) ou launchd plist (macOS).

**D3** : Flow de connexion — VPS1 coordinator (visibility=public, pkarr),
VPS2/Mac/PC workers (join via invite). Fix CORS pour accès browser externe.
Fix bearer token flow pour workers distants.

**D4** : Fail-fast multi-nœuds — rows 30-33 permanentes. Script smoke test
`scripts/test-multi-node.sh` (2 daemons localhost). Crate `nexus-test-harness`
pour tests P2P avancés.

**D5** : Mobile browser test — iPhone/Android accèdent au shell React via
IP publique VPS. Valide rendu iframe sandbox + responsive + touch events.

---

## 8. Risques identifiés

| ID | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | CORS localhost-only bloque browser externe | High | Fix --cors-origin S33 Phase A |
| R2 | NAT résidentiel empêche QUIC direct | Medium | Relay fallback iroh (déjà en place) |
| R3 | Ollama absent sur VPS (pas de GPU) | Low | --stub-ollama ou CPU-only |
| R4 | Build time VPS (~15 min) ralentit itération | Low | Build --release une fois, dev sur PC |
| R5 | iroh relay n0 down = discovery fail | Medium | Self-hosted pkarr relay (docker/) |
