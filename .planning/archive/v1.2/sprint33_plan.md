# Sprint 33 — Plan d'execution detaille

**Ecrit** : 2026-04-27 (post-kickoff, pre-Phase A)
**Tip master d'entree** : `242200e`

---

## §1 Etat verifie a l'entree

| Suite | Count |
|---|---|
| Rust (cargo nextest) | 883 pass, 0 fail |
| Rust doctests | 0 fail (1 ignored) |
| Rust clippy | 0 warnings |
| Rust fmt | clean |
| Release build daemon | Finished |
| SDK pytest | 195 pass |
| Coordinator pytest | 406 pass + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 pass |
| Vitest | 267 pass |
| Playwright | 42 pass + 2 fail (env) |
| size-limit | 7/7 |
| en-strings | clean |

---

## §2 Decisions Day 0 (gelees) — rappel synthetique

- **D1** : CORS opt-in `--cors-origin` daemon Rust + coordinator Python
- **D2** : Multi-node test 2-daemon localhost, crate `nexus-test-harness`
- **D3** : Deploy systemd + `install-node.sh` Linux-first
- **D4** : P2-REVIEW-A-1 hook LOC guard dans lightcheck
- **D5** : Fail-fast rows 30-33 multi-noeuds permanentes

---

## §3 Research consulte

- Sprint 33 multi-node research (`.planning/research/sprint33_multinode_research.md`,
  commit `1a60033`, 3 agents paralleles deploy-readiness / process /
  cross-compile) — 247 lignes, 4 blockers identifies, D1..D5 proposes.
- Codebase CORS exploration : `http.rs:285` loopback_cors_layer() +
  `app.py:121` allow_origin_regex localhost-only. Confirme le blocker.
- tower-http 0.6 CorsLayer API : `AllowOrigin::list()` pour origins
  statiques, `AllowOrigin::predicate()` pour dynamique. Deja dans
  workspace `Cargo.toml`.
- FastAPI CORSMiddleware : `allow_origins` list ou `allow_origin_regex`
  string. Deja utilise `app.py:120`.
- iroh 0.98 multi-instance : `NEXUS_GRID_ROOT` env + keypair distinctes
  = isolation complete. Confirme par research §3.1.

---

## §4 Dependency graph inter-phases

```
Phase A (CORS + LOC guard + nits)
  └→ Phase B (deploy infra — uses CORS fix in systemd ExecStart examples)
  └→ Phase C (test harness — tests the CORS-enabled daemon)
       └→ Phase D (wrap-up)
```

Phase A est prerequis pour B (les templates systemd documentent
`--cors-origin`) et pour C (les tests multi-daemon ont besoin de
CORS pour les health checks cross-origin si applicable).

---

## §5 Phase A — CORS fix + P2-REVIEW-A-1 + P3 nits

### §5.1 Scope

**Partie 1 — CORS daemon Rust** : Etendre `loopback_cors_layer()` dans
`http.rs` pour accepter une liste d'origins supplementaires passees via
CLI. Le parametre `cors_origins: Vec<String>` est ajoute a la fn de
construction du router axum. Si la liste est vide, le comportement
loopback-only actuel est preserve (zero regression). Si non-vide,
chaque origin est validee (scheme http/https + host + optional port)
et ajoutee a l'allowlist aux cotes de loopback.

CLI : `nexus-shell-daemon start --cors-origin http://192.168.1.10:8080`
(repeatable). Env fallback : `NEXUS_DAEMON_CORS_ORIGINS` (comma-separated).

**Partie 2 — CORS coordinator Python** : Remplacer le
`allow_origin_regex` hardcode par une construction dynamique dans
`create_app()`. Parametre `cors_origins: list[str] | None`. Si None,
seul le regex loopback est utilise. Si fourni, les origins sont ajoutees
a la liste `allow_origins` de CORSMiddleware.

CLI : `nexus-coordinator start <name> --cors-origin http://...`
(repeatable). Env fallback : `NEXUS_COORD__NETWORK__CORS_ORIGINS`
(comma-separated).

**Partie 3 — P2-REVIEW-A-1 MANDATORY** : Ajouter un check dans
`phase-precommit-lightcheck.sh` qui grep les fichiers `sprint*_plan.md`
dans le staging area pour les patterns :
- `~[0-9]+ LOC` (ex: `~500 LOC`)
- `~[0-9]+ lignes` (ex: `~300 lignes`)
- `environ [0-9]+ LOC`
- `budget LOC`
- `LOC total`

Si detecte dans un fichier staged, le hook affiche un warning et bloque
le commit avec reference §6.7. Pas de block sur `HARDENING_ROADMAP.md`
(qui utilise des bornes indicatives admises §6.7 exception).

**Partie 4 — P3 nits batch** : Fix 8 commentaires stale :
1. `nexus-core-rs/src/attestations/age_witness.rs` lignes 6, 21 : "iroh 0.97" → "iroh 0.98"
2. `nexus-core-rs/src/gossip.rs` ligne 723 : "iroh 0.97" → "iroh 0.98"
3. `nexus-core-rs/src/discovery.rs` lignes 4, 117 : "iroh 0.97" → "iroh 0.98"
4. `nexus-core-rs/src/tls_pinning.rs` ligne 32 : "iroh 0.97" → "iroh 0.98"
5. `nexus-shell-daemon/src/http.rs` ligne 1035 : "iroh 0.97" → "iroh 0.98"
6. `coordinator.py` ligne 370 : "arti-client 2.0" → "arti-client 0.41"

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/http.rs` | CORS layer extension + origin validation |
| `crates/nexus-shell-daemon/src/main.rs` | CLI flag `--cors-origin` + env parse |
| `packages/nexus-coordinator/src/nexus_coordinator/api/app.py` | `create_app()` cors_origins param |
| `packages/nexus-coordinator/src/nexus_coordinator/cli/commands/start.py` | CLI option `--cors-origin` |
| `.claude/hooks/phase-precommit-lightcheck.sh` | LOC guard check |
| `crates/nexus-core-rs/src/attestations/age_witness.rs` | Comment fix "0.97" → "0.98" |
| `crates/nexus-core-rs/src/gossip.rs` | Comment fix "0.97" → "0.98" |
| `crates/nexus-core-rs/src/discovery.rs` | Comment fix "0.97" → "0.98" |
| `crates/nexus-core-rs/src/tls_pinning.rs` | Comment fix "0.97" → "0.98" |
| `crates/nexus-shell-daemon/src/http.rs` | Comment fix "0.97" → "0.98" |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` | Comment fix "arti-client 2.0" → "0.41" |

### §5.3 Tests plan

Daemon Rust CORS :
1. `test_cors_loopback_default_allows_localhost` — sans `--cors-origin`, `http://localhost:3000` est accepte
2. `test_cors_loopback_default_rejects_external` — sans `--cors-origin`, `http://192.168.1.10:8080` est rejete
3. `test_cors_custom_origin_allows_configured` — avec `--cors-origin http://192.168.1.10:8080`, cet origin est accepte
4. `test_cors_custom_origin_preserves_loopback` — avec `--cors-origin`, localhost reste accepte
5. `test_cors_rejects_unconfigured_external` — avec `--cors-origin http://a.com`, `http://b.com` est rejete

Coordinator Python CORS :
6. `test_cors_default_localhost_only` — sans option, seul localhost passe
7. `test_cors_custom_origin_accepted` — avec `--cors-origin`, l'origin configure passe
8. `test_cors_custom_preserves_localhost` — avec `--cors-origin`, localhost reste accepte

### §5.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-coordinator/tests/ -q
# Tous verts, 0 regression
```

### §5.5 Commit cible

```
feat(sprint33): Sprint 33 Phase A — CORS external access + LOC guard + P3 nits

CORS daemon Rust : --cors-origin CLI flag opt-in, loopback-only par
defaut preserve. Extension loopback_cors_layer() avec AllowOrigin
dynamique. 5 tests unitaires CORS (default/custom/reject).

CORS coordinator Python : --cors-origin CLI option, allow_origins
dynamique dans create_app(). 3 tests pytest CORS.

P2-REVIEW-A-1 MANDATORY (3/3) : hook LOC guard dans
phase-precommit-lightcheck.sh grep ~NNN LOC / budget LOC patterns
dans sprint*_plan.md staged. Enforcement mecanique §6.7.

P3 nits : 7 commentaires stale "iroh 0.97" → "0.98" (5 fichiers Rust)
+ 1 commentaire "arti-client 2.0" → "0.41" (coordinator.py).

Delta tests : +8 (+5 Rust CORS, +3 Python CORS)
Cumul : 888 Rust / 409+36f+6s coord / ~1891 total

Scope cuts respectes : pas de VPS deploy, pas de wildcard CORS,
pas de stop/status CLI, pas de Docker.
```

---

## §6 Phase B — Deploy infrastructure

### §6.1 Scope

**Partie 1 — Templates systemd** : 3 fichiers unit dans
`configs/systemd/` :
- `nexus-daemon.service` : ExecStart avec `--cors-origin` exemple
  commente, User=nexus, WorkingDirectory=/opt/nexus-grid,
  Restart=on-failure, KillSignal=SIGINT (graceful shutdown)
- `nexus-worker.service` : ExecStart `nexus-worker start --headless`,
  After=nexus-daemon.service, User=nexus
- `nexus-coordinator.service` : ExecStart
  `nexus-coordinator start <project> --host 0.0.0.0 --port 8765`,
  After=nexus-daemon.service, User=nexus, Environment for uv/Python

Pas de socket activation, pas de hardening systemd avance
(ProtectSystem, PrivateTmp). Templates minimalistes, documentation
inline via commentaires.

**Partie 2 — Install script** : `scripts/install-node.sh` qui :
1. Detecte OS (Linux/macOS/unknown) + distro (apt/dnf/brew)
2. Installe deps systeme (build-essential, libssl-dev, pkg-config,
   libdbus-1-dev pour Linux)
3. Installe Rust via rustup si absent
4. Installe uv si coordinator demande
5. Clone le repo (ou git pull si deja present)
6. Build `--release` les binaires demandes (daemon, worker, coordinator)
7. Genere keypair daemon si absente (`nexus-shell-daemon init`)
8. Copie les templates systemd et les enable (Linux only)
9. Affiche un resume des actions effectuees

Le script est interactif (questions oui/non pour chaque composant)
mais supporte `--yes` pour mode non-interactif.

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `configs/systemd/nexus-daemon.service` | NEW — template systemd daemon |
| `configs/systemd/nexus-worker.service` | NEW — template systemd worker |
| `configs/systemd/nexus-coordinator.service` | NEW — template systemd coordinator |
| `scripts/install-node.sh` | NEW — script installation multi-OS |

### §6.3 Tests plan

Pas de tests automatises pour les templates systemd (fichiers de config
statiques). Le script install est teste manuellement.

Tests de validation :
1. `shellcheck scripts/install-node.sh` — lint bash (0 errors)
2. Verification syntaxe systemd : `systemd-analyze verify configs/systemd/*.service`
   (si systemd disponible en CI, sinon skip)

### §6.4 Critere d'acceptation

```bash
shellcheck scripts/install-node.sh  # 0 errors
# Templates systemd syntaxiquement valides (verif manuelle)
# Script executable (chmod +x)
```

### §6.5 Commit cible

```
feat(sprint33): Sprint 33 Phase B — deploy infra systemd + install script

3 templates systemd (daemon, worker, coordinator) dans configs/systemd/.
User=nexus, Restart=on-failure, KillSignal=SIGINT.
ExecStart avec --cors-origin exemple commente.

scripts/install-node.sh : detection OS (Linux apt/dnf, macOS brew),
installation Rust + uv, clone/pull repo, build --release, keypair init,
copie systemd units. Mode interactif + --yes non-interactif.

Delta tests : 0 (infra statique)
Cumul : inchange ~1891 total

Scope cuts respectes : pas de Docker daemon/worker, pas de Snap/Flatpak,
pas de Nix flake, pas de launchd macOS genere, pas de CI build merge.
```

---

## §7 Phase C — Multi-node test harness + smoke test

### §7.1 Scope

**Partie 1 — Crate nexus-test-harness** : Nouveau crate workspace
`crates/nexus-test-harness/` avec :

```rust
pub struct DaemonCluster {
    nodes: Vec<DaemonHandle>,
}

pub struct DaemonHandle {
    proc: Child,
    root: TempDir,
    http_port: u16,
    node_id: Option<String>,
}

impl DaemonCluster {
    pub async fn spawn(n: usize) -> Result<Self> { ... }
    pub async fn shutdown(&mut self) -> Result<()> { ... }
}

impl DaemonHandle {
    pub async fn health_check(&self) -> Result<bool> { ... }
    pub fn http_url(&self) -> String { ... }
}
```

Chaque `DaemonHandle` :
- Cree un `TempDir` comme `NEXUS_GRID_ROOT`
- Spawn `nexus-shell-daemon start --headless` sur un port ephemere
  (port 0 → OS assigne)
- Lit `running.json` pour recuperer le port reel
- Expose `health_check()` qui fait un GET sur `/health`

**Partie 2 — Script smoke test** : `scripts/test-multi-node.sh` qui :
1. Build les binaires en debug (ou utilise les binaires existants)
2. Cree 2 repertoires temporaires
3. Spawn 2 daemons sur des ports ephemeres
4. Attend que les deux repondent sur `/health`
5. Verifie que les deux ont des node_id differents
6. Arrete les deux daemons (SIGINT)
7. Exit 0 si succes, 1 si echec

**Partie 3 — Tests d'integration** : Dans
`crates/nexus-test-harness/tests/` :
1. `test_two_daemons_boot_and_respond` — row 30, smoke test
2. `test_cross_daemon_discovery` — row 31, un daemon decouvre l'autre
   via iroh (si pkarr localhost fonctionne, sinon via direct NodeAddr)
3. `test_cross_daemon_blob_transfer` — row 32, publish blob daemon 1,
   fetch depuis daemon 2
4. `test_cross_daemon_task_stub` — row 33, soumettre tache daemon 1,
   recevoir resultat daemon 2 (stub, pas Ollama reel)

**Partie 4 — P2-REVIEW-C-2 tentative** : Si le test harness supporte
le spawn d'un blob-serve HTTP, ajouter un test Playwright qui verifie
les headers COEP/COOP/CORP/CSP servis par le daemon reel (pas le mock
S32). Si non realisable (blob-serve requiert un blob reel publie),
documenter l'exemption et carry 3/3 MANDATORY S34.

### §7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-test-harness/Cargo.toml` | NEW — crate config |
| `crates/nexus-test-harness/src/lib.rs` | NEW — DaemonCluster + DaemonHandle |
| `crates/nexus-test-harness/tests/multi_daemon.rs` | NEW — tests integration |
| `Cargo.toml` (workspace) | Ajouter nexus-test-harness aux members |
| `scripts/test-multi-node.sh` | NEW — smoke test script |

### §7.3 Tests plan

1. `test_two_daemons_boot_and_respond` — spawn 2 daemons, health check
   both, verify distinct node_ids
2. `test_cross_daemon_discovery` — daemon 1 decouvre daemon 2 via
   iroh discovery (fallback direct NodeAddr si pkarr localhost echec)
3. `test_cross_daemon_blob_transfer` — daemon 1 publie blob, daemon 2
   fetch via iroh-blobs ticket
4. `test_cross_daemon_task_stub` — daemon 1 soumet tache, daemon 2
   worker stub retourne resultat, daemon 1 recoit

### §7.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
bash scripts/test-multi-node.sh
# Tous verts
```

### §7.5 Commit cible

```
feat(sprint33): Sprint 33 Phase C — multi-node test harness + 2-daemon smoke

Nouveau crate nexus-test-harness : DaemonCluster + DaemonHandle,
spawn N daemons sur ports ephemeres avec NEXUS_GRID_ROOT isoles.
health_check() via GET /health + running.json port discovery.

scripts/test-multi-node.sh : smoke test 2-daemon localhost (build,
spawn, health check, distinct node_ids, shutdown).

4 tests integration multi-daemon :
- test_two_daemons_boot_and_respond (row 30)
- test_cross_daemon_discovery (row 31)
- test_cross_daemon_blob_transfer (row 32)
- test_cross_daemon_task_stub (row 33)

[P2-REVIEW-C-2 : status TBD — COEP E2E avec daemon reel OU carry 3/3]

Delta tests : +4 Rust integration multi-daemon
Cumul : ~1895 total (892 Rust + tests existants inchanges)

Scope cuts respectes : pas de VPS CI, pas de Docker, pas d'Ollama reel
cross-node, pas de mobile browser test.
```

---

## §8 Phase D — Wrap-up

Commit `chore(sprint33): Phase D — wrap-up + verification + audit plan
S34 + migration`. Contenu standard :

1. `sprint33_verification.md` : fail-fast 32+ rows dont rows 30-33
2. `sprint33_carry_summary.md` : carries S34 avec compteur incremente
3. `sprint34_audit_plan.md` : tracks A-C + meta-track multi-node
4. `SPRINT_LOG.md` row S33
5. `CLAUDE.md §Etat actuel` update
6. `docs/security/HARDENING_ROADMAP.md` update last_validated S33
7. Memory update `nexus_grid_pivot.md` + `MEMORY.md`
8. Migration `active/` → `archive/v1.2/` via git mv

---

## §9 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | 0 errors | |
| 2 | Rust nextest pass | `cargo nextest run --workspace --locked` | 883+ pass, 0 fail | |
| 3 | Rust doctests pass | `cargo test --workspace --locked --doc` | 0 fail | |
| 4 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 5 | Rust fmt clean | `cargo fmt --all --check` | no output | |
| 6 | Release build daemon | `cargo build -p nexus-shell-daemon --release` | Finished | |
| 7 | Python ruff format | `uv run ruff format --check packages/` | clean | |
| 8 | Python ruff check | `uv run ruff check packages/` | pass | |
| 9 | SDK 195 pass | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 pass | |
| 10 | Coord 406+ pass | `uv run pytest packages/nexus-coordinator/tests/ -q` | 406+ pass + 36f stale | |
| 11 | Gov 46 pass | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | |
| 12 | Frontend lint | `cd web && npm run lint` | 0 errors | |
| 13 | Frontend tsc | `npx tsc --noEmit -p tsconfig.app.json` | clean | |
| 14 | Vitest 267+ pass | `npm run test:unit` | 267+ pass | |
| 15 | Frontend build | `npm run build` | success | |
| 16 | size-limit 7/7 | `npm run size` | 7/7 pass | |
| 17 | Playwright | `npx playwright test` | 42+ pass | |
| 18 | en-strings | `bash scripts/scan-en-strings.sh` | clean | |
| 19 | FORMAT_VERSION v1 | `grep const.*_VERSION.*= crates/nexus-core-rs/src/` | all = 1 | |
| 20 | HARDENING compteurs | HARDENING_ROADMAP.md last_validated S33 | updated | |
| 21 | Planning docs | kickoff + plan + design_review + preflights + reviews | complets | |
| 22 | CORS daemon default | test_cors_loopback_default_rejects_external | pass | |
| 23 | CORS daemon custom | test_cors_custom_origin_allows_configured | pass | |
| 24 | CORS coord default | test_cors_default_localhost_only | pass | |
| 25 | CORS coord custom | test_cors_custom_origin_accepted | pass | |
| 26 | LOC guard hook | `echo '~500 LOC' > test.md && ...` | blocks commit | |
| 27 | iroh comments clean | `grep -r "iroh 0.97" crates/` | 0 matches | |
| 28 | arti comment clean | `grep "arti-client 2.0" packages/` | 0 matches | |
| 29 | shellcheck install | `shellcheck scripts/install-node.sh` | 0 errors | |
| 30 | 2-daemon smoke | `bash scripts/test-multi-node.sh` | both respond | |
| 31 | Cross-node discovery | test_cross_daemon_discovery | pass | |
| 32 | Cross-node blob | test_cross_daemon_blob_transfer | pass | |
| 33 | Cross-node task | test_cross_daemon_task_stub | pass | |

**Cible : 33/33 rows verts.**

---

## §10 Git plan

| # | Scope | Titre |
|---|---|---|
| 0 | chore(planning) | sprint 33 kickoff + plan + design review |
| A | feat(sprint33) | Sprint 33 Phase A — CORS external access + LOC guard + P3 nits |
| B | feat(sprint33) | Sprint 33 Phase B — deploy infra systemd + install script |
| C | feat(sprint33) | Sprint 33 Phase C — multi-node test harness + 2-daemon smoke |
| D | chore(sprint33) | Phase D — wrap-up + verification + audit plan S34 + migration |

---

## §11 Scope cuts

Copie de kickoff §7 :

1. VPS deployment effectif
2. Mobile browser testing
3. iroh relay over Tor
4. Nym mixnet
5. TEE H100 attestation
6. DKG distribue FROST
7. CI multi-node VPS
8. Docker daemon/worker
9. stop/status CLI
10. Build CI merge (build-binaries.yml)
11. Cross-node task execution reel (Ollama)
12. Output filter client-side

---

## §12 Risks

Copie de kickoff §9 : R1-R6.

---

## §13 Checkpoint de cloture

Sprint 33 est ferme quand :
1. 33/33 rows fail-fast verts
2. 4 commits feat + 1 commit chore planning + 1 commit chore wrap-up
3. `sprint33_verification.md` + `sprint34_audit_plan.md` ecrits
4. PATTERNS.md + SPRINT_LOG.md + CLAUDE.md + memory mis a jour
5. Migration `active/` → `archive/v1.2/` effectuee
