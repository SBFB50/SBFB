# Sprint 19 — Plan d'execution (PoW gossip + TLS pinning + DHT runtime wire)

**Ecrit** : 2026-04-16 (meme commit `chore(planning)` que
`sprint19_kickoff.md`).
**Tip master d'entree** : `1a606a3` (post-S18 audit gate leve).

---

## 1. Etat verifie a l'entree

### 1.1 Commit stack context

```
1a606a3 chore(sprint18): audit-P3 batch             ← TIP COURANT
e223ec7 fix(sprint18): audit-P2 C-1
6fe2dce fix(sprint18): audit-P2 B-1
9661485 fix(sprint18): audit-P2 A-1
0fb8458 fix(sprint18): audit-P2 F-1 + F-2
677556f fix(sprint18): audit-P1 D-1 — wire TokenRotator
4453bfd chore(sprint18): Phase F — wrap-up + verification + audit plan S19
95807b1 feat(sprint18): Phase E3 — Codeberg mirror
04c9621 feat(sprint18): Phase E2 — warrant canary
9f4d19f feat(sprint18): Phase E1 — NVIDIA driver CVE check
94cccb2 feat(sprint18): Phase D — wire+token rotation
9d0ad7a feat(sprint18): Phase C — multi-relai + DHT quorum primitive
4ab0211 feat(sprint18): Phase B — reproducible builds + SLSA
d7ab281 feat(sprint18): Phase A — supply chain CI
```

### 1.2 Compteurs de tests observes

| Suite | Count | Verification |
|---|---|---|
| Rust workspace | 478 | `cargo test --workspace --locked` → `test result: ok` somme |
| Python SDK | 183 | `uv run pytest packages/nexus-sdk/tests/ -q` |
| Python coord | 187 + 3 skipped | `uv run pytest packages/nexus-coordinator/tests/ -q` |
| Python app-gov | 46 | `uv run pytest packages/nexus-app-gov/tests/ -q` |
| Vitest unit | 239 | `cd web && npm run test:unit` |
| Playwright | 38 | `cd web && npx playwright test` |
| size-limit | 7/7 | `cd web && npm run size` |
| SPDX | 246+ | SPDX headers grep cumulatif workspace |
| **Total** | **~1176** | |

### 1.3 Verification lint/format entree

```bash
cargo fmt --all --check          # silencieux (clean)
cargo clippy --workspace --all-targets --locked -- -D warnings  # clean
uv run ruff format --check packages/  # clean
uv run ruff check packages/      # clean
```

---

## 2. Decisions Day 0 (gelees — cf. kickoff §4)

| D | Decision | Implications code |
|---|---|---|
| **D1** | 5 phases A-F + Phase 0 audit deja joue | Ordre A DHT wire → B PoW → C TLS pin → D queue → E pkarr docker → F wrap-up |
| **D2** | PoW Hashcash difficulty 2^18 initial, per-relai ajustable | `pow::solve_challenge` SHA256 single-threaded + `relay_pow_policy.toml` |
| **D3** | TLS pinning via SPKI hash (HPKP-concept) | `tls_pinning::PinValidator` + `~/.sbfb/relay-pins.json` |
| **D4** | Delayed upload queue 0-5min exponential mean=90s | `upload_queue.py` async queue + scheduler 30s flush |
| **D5** | pkarr relay = docker image + ops doc (PAS deploy) | `ghcr.io/SBFB50/pkarr-relay:v1.0` + `PKARR_RELAY_OPS.md` |

**Ne pas rebattre** — figees kickoff S19 §4.

---

## 3. Research consulte

### 3.1 Pre-plan research (context7 + WebSearch)

- **iroh 0.97 pkarr discovery** (context7 `/websites/rs_iroh`) :
  - `PkarrRelayClient::new(pkarr_relay_url: Url)` retourne un
    client HTTP GET sur `/{node_id_z32}` → `SignedPacket`
  - `.resolve(node_id: NodeId) -> Result<SignedPacket,
    DiscoveryError>` est le call per-relay unique que nous
    wrappons en `QuorumResolver`
  - `PkarrClient::builder().relays(&[url]).build()` supporte
    **nativement** 1..N relays mais resolve() retourne le PREMIER
    succes — pas un quorum. Notre wrapper doit instancier N
    `PkarrRelayClient` separes pour avoir N resolutions
    independantes.
  - **Verdict** : Phase A non-bloquee par upstream iroh.
    ~170 LOC realiste.
- **Hashcash SHA256 Rust** (WebSearch) : pas de crate
  officielle, mais `sha2` crate (deja dep via nexus-core-rs)
  expose `Sha256::new().update().finalize()`. Primitive
  Hashcash = ~50 LOC inline dans `pow.rs`.
- **pkarr relay upstream Dockerfile** (WebSearch) : pkarr repo
  GitHub fournit un Dockerfile reference dans `server/` —
  nous forkons le pattern sans re-inventer.
- **iroh rustls custom cert validator** (context7
  `/websites/rs_iroh`) : iroh utilise `rustls` via
  `quinn-proto`, expose `.tls_client_config` via builder
  avance — non-trivial mais accessible. Phase C plan detaillera
  le hook exact session fraiche.

### 3.2 Code registry local

- `crates/nexus-core-rs/src/dht_quorum.rs:40-440` : primitive
  `redundant_resolve<R: QuorumResolver>(resolvers: [Arc<R>; 3],
  node_id: &str, timeout: Duration) -> Result<Vec<u8>,
  QuorumError>` deja livree + testee (13 tests)
- `crates/nexus-shell-daemon-core/src/browse.rs:256-267` : TODO
  comment pointe explicitement le wire point pour S19
- `crates/nexus-core-rs/src/relay_config.rs:~280` : loader
  `relays.json` S18 qui fournit la liste des 3 relays pour
  instancier les resolvers
- `crates/nexus-shell-daemon-core/src/auth.rs:421-607` : pattern
  `TokenRotator` + `notify` file-watcher reutilise pour
  `relay-pins.json` hot reload Phase C

### 3.3 Research a faire session fraiche (par phase)

- **Phase B** : context7 `/websites/rs_iroh_0_95_1_iroh` ou
  `/websites/rs_iroh` pour API subscribe gossip 0.97 exacte
  (injection PoW proof dans subscribe message ou via header
  metadata)
- **Phase C** : context7 `/websites/rs_iroh` pour
  `Endpoint::builder().tls_config()` ou equivalent hook cert
  validator custom
- **Phase E** : WebSearch pkarr-relay Dockerfile upstream
  (version 2026), `pubky/pkarr` releases

---

## 4. Phase A — DHT quorum runtime wire (carry S18 C-1)

### 4.1 Scope

Wire la primitive S18 `dht_quorum::redundant_resolve` dans le
browse aggregator (et le curator runtime si applicable), en
instanciant 3 `PkarrRelayClient` distincts (un par relay de la
federation S18) et en wrappant chacun en `QuorumResolver`.

### 4.2 Fichiers touches

| Fichier | LOC | Role |
|---|---|---|
| `crates/nexus-core-rs/src/pkarr_resolver.rs` (nouveau) | ~80 | `PkarrQuorumResolver` wrapper autour `PkarrRelayClient` impl `QuorumResolver` |
| `crates/nexus-core-rs/src/lib.rs` | +3 | `pub mod pkarr_resolver;` + re-export |
| `crates/nexus-shell-daemon-core/src/browse.rs` | ~30 | remplacer `DiscoveryClient::probe_reachable` direct par `redundant_resolve` + `probe_reachable` |
| `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` | ~20 | passer les 3 `PkarrQuorumResolver` au `CuratorRuntime` au boot |
| Tests : `crates/nexus-core-rs/src/pkarr_resolver.rs` tests module | ~50 | 3 tests primitive + 2 tests integration mock |

**Total** : ~180 LOC code + ~50 LOC tests (dans les fichiers
ci-dessus).

### 4.3 Structure `pkarr_resolver.rs`

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
//! PkarrRelayClient adapter implementing QuorumResolver.
//!
//! Wraps a single pkarr relay client so that N instances
//! (one per relay in our federation) can feed the
//! redundant_resolve() 2/3 quorum primitive.

use crate::dht_quorum::QuorumResolver;
use async_trait::async_trait;
use iroh::NodeId;
use iroh::discovery::pkarr::PkarrRelayClient;
use url::Url;

pub struct PkarrQuorumResolver {
    label: String,
    client: PkarrRelayClient,
}

impl PkarrQuorumResolver {
    pub fn new(relay_url: Url) -> Self {
        let label = relay_url.host_str()
            .unwrap_or("unknown")
            .to_string();
        let client = PkarrRelayClient::new(relay_url);
        Self { label, client }
    }
}

#[async_trait]
impl QuorumResolver for PkarrQuorumResolver {
    fn label(&self) -> &str { &self.label }

    async fn resolve(&self, node_id: &str) -> anyhow::Result<Vec<u8>> {
        let nid = NodeId::from_str(node_id)?;
        let packet = self.client.resolve(nid).await?;
        // Canonical bytes = packet.to_relay_payload() (format
        // stable pkarr upstream, matche byte-for-byte entre relays
        // qui servent le meme record)
        Ok(packet.to_relay_payload().to_vec())
    }
}
```

### 4.4 Tests plan

1. `test_pkarr_resolver_label` : `PkarrQuorumResolver::new(
   "https://relay.iroh.network")` → `label() == "relay.iroh.
   network"`
2. `test_pkarr_resolver_invalid_node_id` : `resolve("not-hex")`
   → Err
3. `test_pkarr_resolver_resolve_via_mock_server` : mock HTTP
   server repond avec SignedPacket valide → Ok(bytes)
4. `test_browse_quorum_2_of_3_agree` (integration) : 3 mock
   relays, 2 retournent meme packet, 1 different → accept
5. `test_browse_quorum_eclipse_detected` (integration) : 3 mock
   relays, 1 retourne packet, 2 timeout → `NoMajority`, log
   warn, `probe_reachable` NOT called (eclipse defense active)

### 4.5 Critere d'acceptation

- `cargo test -p nexus-core-rs pkarr_resolver` → 3 tests vert
- `cargo test -p nexus-shell-daemon-core browse` → tests
  browse avec quorum vert
- `cargo clippy -p nexus-core-rs -p nexus-shell-daemon-core
  --all-targets -- -D warnings` clean
- Grep `dht_quorum::redundant_resolve` retourne 2+ call sites
  prod (browse.rs + iroh_runtime.rs ou equivalent curator)
- `browse.rs` TODO comment ligne 256-267 **supprime** (remplace
  par une phrase courte "wired via PkarrQuorumResolver S19")

### 4.6 Commit cible

```
feat(sprint19): Phase A — DHT quorum runtime wire (browse aggregator + curator)

Cable dht_quorum::redundant_resolve primitive S18 (13 tests
inchanges) au runtime browse aggregator en instanciant 3
PkarrRelayClient iroh 0.97 distincts (un par relay de la
federation S18) wrappes en PkarrQuorumResolver impl
QuorumResolver.

Fichiers :
- crates/nexus-core-rs/src/pkarr_resolver.rs (nouveau)
- crates/nexus-core-rs/src/lib.rs (module export)
- crates/nexus-shell-daemon-core/src/browse.rs (probe path wire)
- crates/nexus-shell-daemon-core/src/iroh_runtime.rs (boot injection)

Eclipse-by-DHT defense passe de primitive-prete a runtime-active.
Gate 1 verification.md §Gate 1 row "DHT redundant lookup" passe
de [~] a [x] (flip docs/sprint18_verification.md en Phase F S19
wrap-up).

Tests delta : +5 (3 primitive pkarr_resolver + 2 integration
browse quorum scenarios). Total Rust 478 → 483.

Closes S18 audit finding C-1 (P2 carry-over).

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 5. Phase B — PoW Hashcash gossip subscribe

### 5.1 Scope

Primitive Hashcash SHA256 + integration au path `iroh_gossip::
GossipClient::subscribe()`. Config `relay_pow_policy.toml`
charge au boot. Difficulty 2^18 default ajustable per-relay.

### 5.2 Fichiers touches

| Fichier | LOC | Role |
|---|---|---|
| `crates/nexus-core-rs/src/pow.rs` (nouveau) | ~180 | primitive SHA256 Hashcash solve/verify, difficulty bits, nonce iter |
| `crates/nexus-core-rs/src/relay_pow_policy.rs` (nouveau) | ~100 | TOML loader pattern `relay_config.rs` |
| `crates/nexus-core-rs/src/lib.rs` | +6 | exports modules |
| `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` | ~40 | wrap `GossipClient::subscribe` avec PoW gate |
| Tests `pow.rs` module | ~90 | 10 tests (solve+verify + edge cases) |
| Tests `relay_pow_policy.rs` module | ~40 | 4 tests (load, default, per-relay override) |
| Tests integration gossip subscribe | ~30 | 2 tests (happy path + reject invalid proof) |
| Bench `benches/pow.rs` (nouveau) | ~30 | `cargo bench --bench pow` verify timing |

**Total** : ~320 LOC code + ~160 LOC tests + ~30 LOC bench.

### 5.3 Primitive Hashcash

```rust
pub struct HashcashChallenge {
    pub topic: String,
    pub difficulty: u32,  // 2^difficulty target
    pub issued_at: u64,   // unix secs, anti-replay
}

pub struct HashcashProof {
    pub challenge: HashcashChallenge,
    pub nonce: u64,
    pub hash: [u8; 32],
}

pub fn solve(challenge: &HashcashChallenge, timeout: Duration)
    -> Result<HashcashProof, PowError>
{
    let target_zeros = challenge.difficulty;
    let deadline = Instant::now() + timeout;
    for nonce in 0u64.. {
        if Instant::now() > deadline {
            return Err(PowError::Timeout);
        }
        let h = sha256_of(&challenge, nonce);
        if leading_zero_bits(&h) >= target_zeros {
            return Ok(HashcashProof { challenge: challenge.clone(), nonce, hash: h });
        }
    }
    unreachable!()
}

pub fn verify(proof: &HashcashProof) -> bool {
    let h = sha256_of(&proof.challenge, proof.nonce);
    h == proof.hash &&
        leading_zero_bits(&h) >= proof.challenge.difficulty
}
```

### 5.4 Tests plan

Primitive :
1. `solve_then_verify_happy` : difficulty 16, solve <100ms, verify ok
2. `verify_rejects_tampered_nonce` : mutate proof.nonce → false
3. `verify_rejects_tampered_difficulty` : lower difficulty in proof → false
4. `verify_rejects_tampered_hash` : mutate proof.hash → false
5. `solve_difficulty_0_trivial` : any nonce ok
6. `solve_timeout_fires` : difficulty 32, timeout 50ms → PowError::Timeout
7. `issued_at_diff_changes_solution` : same topic, different timestamps → different proof
8. `leading_zero_bits_boundary_255` : test 7 bits, 8 bits
9. `different_topics_different_solutions` : proof pour topic A ne verify pas pour topic B
10. `sha256_canonical_stable` : canonical bytes Hashcash = fixed encoding

Policy :
1. `policy_default_loads_2_18` : absent file → default
2. `policy_per_relay_override` : TOML avec `[relay."https://foo"]
   difficulty = 16` → override applique
3. `policy_invalid_toml_fail_loud` : Err explicite
4. `policy_missing_relay_uses_default` : relay inconnu → 2^18

Integration :
1. `gossip_subscribe_with_pow_happy` : fake relay exige 2^12, client resout + subscribe ok
2. `gossip_subscribe_invalid_proof_rejected` : mock relay reject, client pense subscribe ok mais messages pas delivery (log warn)

Bench : `cargo bench --bench pow` verify difficulty 18 < 500ms
(warn si >500ms), difficulty 16 < 100ms (reasonable baseline).

### 5.5 Commit cible

```
feat(sprint19): Phase B — PoW Hashcash gossip subscribe (difficulty 2^18 per-relai)

Cost-of-identity minimal pour Sybil-resistance bootstrap. Tout
subscribe gossip exige un HashcashProof valide, difficulty 2^18
initial (~100ms CPU moderne 2026) configurable per-relai via
~/.sbfb/relay_pow_policy.toml.

Fichiers :
- crates/nexus-core-rs/src/pow.rs (nouveau, ~180 LOC)
- crates/nexus-core-rs/src/relay_pow_policy.rs (nouveau, ~100 LOC)
- crates/nexus-shell-daemon-core/src/iroh_runtime.rs (subscribe wrap)
- benches/pow.rs (nouveau, bench timing)

Tests delta : +16 (10 primitive + 4 policy + 2 integration).
Total Rust 483 → 499.

Prerequis S21 rate-limit per-consumer (HARDENING_ROADMAP §3 S19 +
dependency graph §6).

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 6. Phase C — TLS cert pinning relays

### 6.1 Scope

SPKI hash pin pour chaque relay iroh. Loader `~/.sbfb/relay-
pins.json`. Bootstrap pins pre-charges pour les 3 relays n0 (SHA
extracts au moment du kickoff S19 via `openssl s_client`).

### 6.2 Fichiers touches

| Fichier | LOC | Role |
|---|---|---|
| `crates/nexus-core-rs/src/tls_pinning.rs` (nouveau) | ~140 | `PinValidator` + SPKI extract + validate against pinset |
| `crates/nexus-core-rs/src/lib.rs` | +3 | module export |
| `crates/nexus-core-rs/src/node.rs` (edit) | ~20 | injection validator dans `Endpoint::builder` via rustls custom cert validator (hook iroh 0.97 si expose, sinon forked connect path avec TODO upstream PR) |
| `~/.sbfb/relay-pins.json` bootstrap (doc) | - | 3 entries pour relay.iroh.network + fallback 1 + fallback 2 |
| `docs/release/RELAY_PIN_BOOTSTRAP.md` (nouveau) | ~80 | procedure regeneration SPKI via openssl, rotation, user-override |
| Tests `tls_pinning.rs` module | ~60 | 5 tests primitive + 3 tests integration |

**Total** : ~160 LOC code + ~60 LOC tests + ~80 LOC docs.

### 6.3 Primitive SPKI pin

```rust
pub struct RelayPin {
    pub relay_url: String,    // "https://relay.iroh.network"
    pub spki_sha256: String,  // "base64url-..." 44 chars
    pub added_at: String,     // RFC3339
    pub source: PinSource,    // Bootstrap | UserOverride
}

pub struct PinValidator {
    pins: HashMap<String, String>,  // url → spki_sha256
}

impl PinValidator {
    pub fn validate(&self, relay_url: &str, cert_der: &[u8])
        -> Result<(), PinError>
    {
        let pinned = self.pins.get(relay_url)
            .ok_or(PinError::NoPin(relay_url.to_string()))?;
        let actual_spki = extract_spki_sha256(cert_der)?;
        if actual_spki == *pinned {
            Ok(())
        } else {
            Err(PinError::SpkiMismatch {
                relay_url: relay_url.to_string(),
                pinned: pinned.clone(),
                actual: actual_spki,
            })
        }
    }
}
```

### 6.4 Tests plan

Primitive :
1. `extract_spki_sha256_pem` : cert PEM → SPKI hash fixe (vector
   test hardcoded depuis openssl cli)
2. `extract_spki_sha256_der` : cert DER → meme hash
3. `validate_match_ok` : pin SPKI == cert SPKI → Ok
4. `validate_mismatch_err` : pin SPKI != cert SPKI → SpkiMismatch
5. `validate_no_pin_err` : relay absent pinset → NoPin (fail-closed)

Integration :
1. `loader_parse_pin_json_ok` : JSON valide → PinValidator avec
   2 pins
2. `loader_missing_file_empty_pinset` : pas de fichier → empty
   pinset (tous relays refusent — documented behavior S19 pre-
   release, pre-launch on whitelist les 3 n0 defaults)
3. `loader_invalid_json_fail_loud` : Err explicite

### 6.5 Commit cible

```
feat(sprint19): Phase C — TLS cert pinning relays (SPKI hash validate)

Pin le public key SPKI hash de chaque relay iroh pour resister a
CA compromise. Config ~/.sbfb/relay-pins.json bootstrap avec les
3 relays n0 connus S19 kickoff. Rotation procedure documentee
docs/release/RELAY_PIN_BOOTSTRAP.md §rotation.

Fichiers :
- crates/nexus-core-rs/src/tls_pinning.rs (nouveau)
- crates/nexus-core-rs/src/node.rs (edit injection builder)
- docs/release/RELAY_PIN_BOOTSTRAP.md (nouveau)

Tests delta : +8 (5 primitive + 3 integration).
Total Rust 499 → 507.

Note : hook iroh 0.97 custom cert validator a confirmer session
fraiche Phase C. Si non-expose, forked connect path + TODO PR
upstream (aligne VALIDATED_BLUEPRINT couche 3).

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 7. Phase D — Delayed upload queue

### 7.1 Scope

Queue async coord-side qui randomly delaye chaque publish task
submit de 0-5 minutes (exponential distribution mean=90s).
Scheduler interne flush 30s. Metric log INFO pour diagnostic.

### 7.2 Fichiers touches

| Fichier | LOC | Role |
|---|---|---|
| `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py` (nouveau) | ~200 | async queue + scheduler + jitter randomization |
| `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py` (edit) | ~20 | pipe dans queue au lieu direct gossip emit |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` (edit) | ~15 | boot/shutdown hook queue scheduler |
| Tests `tests/test_upload_queue.py` (nouveau) | ~120 | 10 tests (jitter range, scheduler timing, persistence, concurrent) |
| Tests integration `tests/test_api_tasks_delayed.py` (edit existing) | ~40 | 2 tests full-loop submit → poll status |
| `docs/shell/PATTERNS.md` edit | ~20 | section "Delayed upload queue" |

**Total** : ~235 LOC code + ~160 LOC tests + ~20 LOC docs.

### 7.3 Primitive queue

```python
import asyncio
import random
from datetime import datetime, timedelta
from typing import Callable, Awaitable

class UploadQueue:
    def __init__(
        self,
        emit_fn: Callable[[dict], Awaitable[None]],
        mean_jitter_s: float = 90.0,
        max_jitter_s: float = 300.0,
        flush_interval_s: float = 30.0,
    ):
        self.emit_fn = emit_fn
        self.mean = mean_jitter_s
        self.max = max_jitter_s
        self.flush_interval = flush_interval_s
        self.queue: list[tuple[datetime, dict]] = []
        self._lock = asyncio.Lock()
        self._task: asyncio.Task | None = None

    def schedule(self, task: dict) -> None:
        jitter = min(random.expovariate(1.0 / self.mean), self.max)
        delivery_at = datetime.utcnow() + timedelta(seconds=jitter)
        self.queue.append((delivery_at, task))

    async def _flush_loop(self) -> None:
        while True:
            await asyncio.sleep(self.flush_interval)
            await self._flush_due()

    async def _flush_due(self) -> None:
        now = datetime.utcnow()
        async with self._lock:
            due = [(t, task) for t, task in self.queue if t <= now]
            self.queue = [(t, task) for t, task in self.queue if t > now]
        for _, task in due:
            await self.emit_fn(task)

    async def start(self) -> None:
        self._task = asyncio.create_task(self._flush_loop())

    async def shutdown(self) -> None:
        if self._task:
            self._task.cancel()
        async with self._lock:
            for _, task in self.queue:
                await self.emit_fn(task)
```

### 7.4 Tests plan

Primitive (mock emit_fn) :
1. `schedule_within_max_range` : 100 schedules, all jitter <= 5min
2. `schedule_median_around_mean` : 1000 schedules, median ~90s +/- 20s
3. `flush_due_releases_ready_tasks` : schedule t+0, flush immediate → emit
4. `flush_due_holds_future_tasks` : schedule t+10min, flush maintenant → 0 emit
5. `scheduler_flush_every_30s` : run scheduler 90s, verify 3 flush calls
6. `shutdown_flushes_all_pending` : 5 tasks pending, shutdown → 5 emit calls
7. `concurrent_schedule_safe` : 100 threads schedule, no race
8. `restart_persistence_none_by_design` : S19 queue is in-memory only (docs warn queue loss on crash, persistence = Sprint 20+ tech debt)

Integration :
1. `api_submit_pipes_to_queue` : POST /project/task/submit → queue schedule (mock scheduler freeze time)
2. `api_submit_full_loop_with_flush` : submit + advance time 90s + flush → gossip emit observed

### 7.5 Commit cible

```
feat(sprint19): Phase D — delayed upload queue (0-5min exponential jitter)

Anti-correlation traffic embryonnaire. Chaque task submit coord-
side est randomly delayed (exponential mean=90s, max 5min) avant
d'etre emis via iroh-gossip. Scheduler interne flush 30s. UX
impact : DnD Forge task response latency +30-90s median (documente
PATTERNS.md).

Fichiers :
- packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py
- packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py (edit)
- packages/nexus-coordinator/src/nexus_coordinator/coordinator.py (edit)

Tests delta : +12 pytest (10 primitive + 2 integration).
Total coord 187+3 → 199+3.

Queue in-memory pre-launch (persistence = tech debt S20+).
Document PATTERNS.md §delayed-upload-queue.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 8. Phase E — pkarr relay self-hosted image + ops doc

### 8.1 Scope

Docker image buildable + workflow GHA push `ghcr.io/SBFB50/
pkarr-relay:<version>`. Ops doc deploiement Hetzner CX11. Smoke
test CI docker build. Zero code Rust dans ce repo.

### 8.2 Fichiers touches

| Fichier | LOC | Role |
|---|---|---|
| `docker/pkarr-relay/Dockerfile` (nouveau) | ~40 | FROM rust:1.94-slim AS builder, cargo install pkarr-relay, runtime image |
| `docker/pkarr-relay/README.md` (nouveau) | ~30 | quick-start docker run |
| `.github/workflows/build-pkarr-image.yml` (nouveau) | ~60 | trigger push master + tag v*, push ghcr.io, Trivy scan inline |
| `docs/release/PKARR_RELAY_OPS.md` (nouveau) | ~280 | ops deploy Hetzner (provisioning + systemd + nginx + LE + smoke test + monitoring + SPKI rotation cross-ref Phase C) |
| `tests/ci-smoke/pkarr-relay-healthcheck.sh` (nouveau) | ~30 | bash script curl /healthz + assert response |

**Total** : ~440 LOC (config + docs + shell), zero Rust.

### 8.3 Dockerfile shape

```dockerfile
# syntax=docker/dockerfile:1
FROM rust:1.94-slim AS builder
WORKDIR /build
RUN cargo install pkarr-relay --version 2.1.* --root /build/dist
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/dist/bin/pkarr-relay /usr/local/bin/
EXPOSE 6881/udp 6882/tcp
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:6882/healthz || exit 1
ENTRYPOINT ["/usr/local/bin/pkarr-relay"]
CMD ["--http-port", "6882", "--dht-port", "6881"]
```

### 8.4 Workflow GHA shape

```yaml
name: build-pkarr-image
on:
  push:
    branches: [master]
    tags: ["v*"]
  workflow_dispatch:
permissions:
  contents: read
  packages: write
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4  # (TODO pin SHA pattern S20+)
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - uses: docker/build-push-action@v5
        with:
          context: docker/pkarr-relay
          push: true
          tags: ghcr.io/sbfb50/pkarr-relay:${{ github.ref_name }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
      - name: Trivy scan
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ghcr.io/sbfb50/pkarr-relay:${{ github.ref_name }}
          severity: CRITICAL,HIGH
          exit-code: 1
```

### 8.5 Commit cible

```
feat(sprint19): Phase E — pkarr relay self-hosted docker image + ops doc

Premier pas vers federation pkarr non-solo. Livre l'image docker
packagee + doc deploy Hetzner CX11 ~5 EUR/mois pour qu'un
maintainer ou contributeur spin up un relai en 30min. Deploy reel
= decision ops separee (pas ce sprint).

Fichiers :
- docker/pkarr-relay/Dockerfile
- docker/pkarr-relay/README.md
- .github/workflows/build-pkarr-image.yml
- docs/release/PKARR_RELAY_OPS.md
- tests/ci-smoke/pkarr-relay-healthcheck.sh

Zero code Rust dans repo. Cross-ref Phase C §SPKI rotation pour
regenerer pin apres deploy.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 9. Phase F — Consolidation + verification + audit plan S20

### 9.1 Scope

- Update `CLAUDE.md §Etat actuel` : Sprint 19 CLOSED + Eclipse-
  by-DHT defense active + commits stack
- Update `docs/claude/SPRINT_LOG.md` : row S19 v1.2
- Update memory `nexus_grid_pivot.md` frontmatter
- `.planning/active/sprint19_verification.md` : fail-fast 24+ rows
- `.planning/active/sprint19_audit_plan.md` : tracks A-E + meta-
  track Radicle-v1.0 **re-carried**
- Flip `sprint18_verification.md §Gate 1 row "DHT redundant
  lookup"` de `[~]` a `[x]` (consequence Phase A wire)
- Migration planning `.planning/active/sprint19_*.md` →
  `.planning/archive/v1.2/`

### 9.2 Commit cible

```
chore(sprint19): Phase F — wrap-up + verification + audit plan S20 + migrate planning
```

---

## 10. Fail-fast checklist (28 rows)

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | `git rev-parse --short HEAD` vaut un SHA Phase F final | `git rev-parse --short HEAD` | 7-char SHA | — |
| 2 | Range S19 commits >= 6 (5 phases + F) | `git log --oneline 1a606a3..HEAD \| wc -l` | `>= 6` | — |
| 3 | `.planning/active/` vide post-F | `ls .planning/active/ \| wc -l` | `0` | — |
| 4 | `.planning/archive/v1.2/sprint19_*` = 4 minimum (kickoff/plan/verification/audit_plan) + phase reviews | `ls .planning/archive/v1.2/sprint19_*.md \| wc -l` | `>= 4` | — |
| 5 | Rust tests 478 → >= 523 | `cargo test --workspace --locked` somme | `>= 523` | — |
| 6 | `cargo fmt --all --check` silent | `cargo fmt --all --check` | exit 0, stdout vide | — |
| 7 | `cargo clippy -D warnings` clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 | — |
| 8 | Python SDK 183 unchanged | `uv run pytest packages/nexus-sdk/tests/ -q` | `183 passed` | — |
| 9 | Python coord 187 → 199+3 | `uv run pytest packages/nexus-coordinator/tests/ -q` | `>= 199 passed, 3 skipped` | — |
| 10 | Python app-gov 46 unchanged | `uv run pytest packages/nexus-app-gov/tests/ -q` | `46 passed` | — |
| 11 | `uv run ruff format --check packages/` clean | idem | exit 0 | — |
| 12 | `uv run ruff check packages/` clean | idem | exit 0 | — |
| 13 | Vitest 239 unchanged | `cd web && npm run test:unit` | `239 passed` | — |
| 14 | Playwright 38 unchanged | `cd web && npx playwright test` | `38 passed` | — |
| 15 | size-limit 7/7 | `cd web && npm run size` | all pass | — |
| 16 | Frontend build ok | `cd web && npm run build` | zero warnings | — |
| 17 | `scan-en-strings.sh` clean | `bash web/scripts/scan-en-strings.sh` | exit 0 | — |
| 18 | Grep `dht_quorum::redundant_resolve` retourne 2+ prod sites | `grep -r "redundant_resolve\|QuorumResolver" crates/ --include="*.rs" \| grep -v "test\|mod " \| wc -l` | `>= 2` | — |
| 19 | Phase A `browse.rs` TODO comment ligne 256-267 supprime | `grep -c "TODO.*S19.*audit" crates/nexus-shell-daemon-core/src/browse.rs` | `0` | — |
| 20 | PoW primitive module present | `test -f crates/nexus-core-rs/src/pow.rs` | exit 0 | — |
| 21 | `bench pow` s'execute < 2s | `cargo bench --bench pow 2>&1 \| grep "time:"` | present | — |
| 22 | TLS pinning module present | `test -f crates/nexus-core-rs/src/tls_pinning.rs` | exit 0 | — |
| 23 | `relay-pins.json` bootstrap doc presente | `test -f docs/release/RELAY_PIN_BOOTSTRAP.md` | exit 0 | — |
| 24 | Upload queue module present | `test -f packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py` | exit 0 | — |
| 25 | Dockerfile pkarr-relay present | `test -f docker/pkarr-relay/Dockerfile` | exit 0 | — |
| 26 | Workflow build-pkarr-image.yml present | `test -f .github/workflows/build-pkarr-image.yml` | exit 0 | — |
| 27 | `PKARR_RELAY_OPS.md` present | `test -f docs/release/PKARR_RELAY_OPS.md` | exit 0 | — |
| 28 | `sprint18_verification.md §Gate 1 row "DHT redundant lookup"` flip `[~]` → `[x]` | `grep -E '\[x\].*DHT redundant' .planning/archive/v1.2/sprint18_verification.md` | 1 match | — |

Observed rempli en Phase F.

---

## 11. Git plan (commits ordonnes)

| # | Commit | Phase | SHA attendu |
|---|---|---|---|
| 1 | `chore(planning): close S18 audit gate + open Sprint 19 — PoW gossip + TLS pinning + DHT wire` | Planning | post-`1a606a3` |
| 2 | `feat(sprint19): Phase A — DHT quorum runtime wire (browse aggregator + curator)` | A | — |
| 3 | `feat(sprint19): Phase B — PoW Hashcash gossip subscribe (difficulty 2^18 per-relai)` | B | — |
| 4 | `feat(sprint19): Phase C — TLS cert pinning relays (SPKI hash validate)` | C | — |
| 5 | `feat(sprint19): Phase D — delayed upload queue (0-5min exponential jitter)` | D | — |
| 6 | `feat(sprint19): Phase E — pkarr relay self-hosted docker image + ops doc` | E | — |
| 7 | `chore(sprint19): Phase F — wrap-up + verification + audit plan S20 + migrate planning` | F | — |

7 commits S19 (1 planning + 5 feat + 1 wrap-up).

---

## 12. Scope cuts (repete pour accessibilite)

Cf. kickoff §6. En resume :
- Encryption at rest keypair → S20 big-rock
- Duress PIN + panic wipe → S20
- Rate-limit sliding-window per-consumer → S21 (depend S19 PoW)
- Kudos-weighted admission → S22
- Structured output grammar → S20
- Client-side redaction SDK → S21
- Federated ONG pkarr concrets → S22+ partnership
- ML-DSA-65 + ML-KEM-1024 PQC → S26+
- Domain fronting + Tor bridges → S24-25
- `actions/checkout@v4` pin SHA → sprint ops futur

---

## 13. Risks (R1..R5) + mitigation

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Phase C : iroh 0.97 n'expose PAS hook cert validator custom accessible | M | M | Phase C fork le connect path local + TODO PR upstream. Fallback : pinning desactive si hook manquant, warn log explicite. **Check session fraiche Phase C** via context7 avant implementer. |
| R2 | Phase B : PoW difficulty 2^18 trop lent sur CPU faible (raspberry pi) | L | M | Bench `cargo bench --bench pow` verifie timing. Si >500ms, reviser a 2^17 (~50ms). Per-relay override deja dans D2. |
| R3 | Phase D : delayed upload queue in-memory = task lost si coord crash mid-queue | M | L | Documente tech debt S20 persistence. Pre-launch acceptable (pas de traffic prod). PATTERNS.md warn. |
| R4 | Phase E : pkarr-relay upstream release bump breaks Dockerfile cargo install | L | L | Pin version `pkarr-relay --version 2.1.*` (minor range). Rebuild CI trigger re-verifie au prochain push. |
| R5 | Phase A : iroh `PkarrRelayClient::resolve` returns different bytes pour meme SignedPacket sur 2 relays (format serialize instable) | L | H | Primitive `redundant_resolve` compare byte-for-byte. Si instabilite observed, compare par `(pubkey, payload_hash)` canonique — investigation session fraiche Phase A via test integration. Fallback bench : si relays retournent bytes differents systematic, escalate finding P0 et re-design Phase A. |

---

## 14. Checkpoint de cloture

Sprint 19 ferme quand :

1. 7 commits S19 landed (1 planning + 5 feat + 1 wrap-up)
2. 28/28 fail-fast checklist verte
3. `sprint19_verification.md` + `sprint19_audit_plan.md` ecrits
4. CLAUDE.md + SPRINT_LOG.md + memory `nexus_grid_pivot.md`
   updated post-Phase F
5. `sprint18_verification.md §Gate 1 row "DHT redundant lookup"`
   flip `[~]` → `[x]`
6. Planning files `sprint19_*.md` migres `active/` → `archive/v1.2/`
7. Meta-1 Radicle-v1.0 tracking **re-carried** explicitement dans
   `sprint19_audit_plan.md §meta-track`
8. `.planning/active/` vide
9. Memory frontmatter description sync tip final

---

**Note de placement** : ce plan est ecrit **meme commit** que
`sprint19_kickoff.md` (pattern S18 `1f5cf42` planning open).
Migrations S18 staged pre-commit via `git mv`.
