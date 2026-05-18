# Architecture

**Analysis Date:** 2026-05-18

## Pattern Overview

**Overall:** Layered headless-first P2P architecture with iroh 0.98 as the networking substrate. Every binary crate (worker, daemon, launcher, executor) depends on a corresponding `-core` library crate that contains all logic, keeping binaries as thin CLI/HTTP wrappers.

**Key Characteristics:**
- Headless-first: every crate pair (`-core` lib + binary) is split so the engine is fully testable without CLI, HTTP, or TUI
- Single signing surface: all signed payloads flow through `nexus_core_rs::canonical::canonical_bytes` (RFC 8785 JCS + domain separation prefix) so cross-language verification is deterministic
- iroh 0.98 protocol stack: every node carries iroh-docs (replicated KV logs), iroh-gossip (topic pub/sub), and iroh-blobs (content-addressed storage), wired through an iroh Router by ALPN
- SQLite for all persistent state (coordinator DB, worker allowlist, trust cache, feed store)
- Ed25519 everywhere: node identity, task signing, curator list signing, PoW binding, key rotation, warrant canary, feed entries
- Zero central server: the shell daemon IS the coordinator, the worker IS the executor -- no external backend

## Workspace Members (12 crates)

```
crates/
  nexus-core-rs/            # Foundation: iroh wrapper, crypto, wire types, PoW, keystore
  nexus-coordinator-rs/     # Business logic: dispatch, validation, kudos, feed, quarantine
  nexus-shell-daemon-core/  # Shell engine: curator runtime, browse, blob-serve, canary, auth
  nexus-shell-daemon/       # Binary: HTTP server, gossip subscribe, CLI subcommands
  nexus-worker-core/        # Worker engine: state machine, LLM backends, GPU, consent, rate-limit
  nexus-worker/             # Binary: CLI + ratatui TUI
  nexus-launcher/           # Binary: spawn daemon, open browser, tray icon, identity unlock
  nexus-events-core/        # SecurityEvent audit system (JSONL, journald, oslog)
  nexus-executor/           # Binary: isolated compute process (broker/executor IPC)
  nexus-trace-core/         # Trace pipeline: BatchLog, OpenTelemetry 0.31, signed traces
  nexus-test-harness/       # Multi-daemon integration test harness
tools/
  png-to-icns/              # macOS icon conversion tool
```

## Dependency Graph (crate-level)

```
nexus-core-rs  (foundation — iroh stack + crypto)
  |
  +-- nexus-coordinator-rs  (depends: nexus-core-rs, rusqlite)
  |
  +-- nexus-worker-core  (depends: nexus-core-rs, nexus-events-core)
  |     +-- nexus-worker  (binary — nexus-worker-core, nexus-core-rs)
  |
  +-- nexus-shell-daemon-core  (depends: nexus-core-rs, nexus-events-core)
  |     +-- nexus-shell-daemon  (binary — ALL crates: core, coordinator, worker-core, daemon-core, events, trace)
  |     +-- nexus-launcher  (binary — nexus-shell-daemon-core, nexus-core-rs)
  |
  +-- nexus-trace-core  (depends: ed25519-dalek, opentelemetry)
  +-- nexus-events-core  (standalone — serde + chrono + platform audit)
  +-- nexus-executor  (binary — nexus-trace-core, ollama-rs)
  +-- nexus-test-harness  (depends: tokio, reqwest, tempfile, zip)
```

**The shell daemon binary is the central hub** -- it imports all major crates because it hosts the coordinator logic, worker state relay, curator runtime, HTTP API, and gossip transport.

## Layers

**Foundation Layer (nexus-core-rs):**
- Purpose: All iroh protocol wrappers, cryptographic primitives, wire format types, and protocol helpers
- Location: `crates/nexus-core-rs/src/`
- Contains: 25 modules (node, crypto, canonical, task, docs, gossip, blobs, curator, pow, keystore, verification, discovery, attestations, key_rotation, tls_pinning, dns_fallback, tor_transport, relay_config, relay_pow_policy, pkarr_resolver, dht_quorum, hooks, schemas, pow_gossip, error)
- Depends on: iroh 0.98, iroh-docs 0.98, iroh-gossip 0.98, iroh-blobs 0.100, ed25519-dalek, blake3, sha2, serde_jcs, argon2, aes-gcm, frost-ed25519, hickory-resolver
- Used by: Every other crate in the workspace

**Business Logic Layer (nexus-coordinator-rs):**
- Purpose: Task dispatch, result validation, kudos ledger, public feed, quarantine, capabilities
- Location: `crates/nexus-coordinator-rs/src/`
- Contains: 20 modules (db, dispatcher, validator, kudos_ledger, public_feed, feed_materializer, quarantine_queue, output_filter, pii_redactor, provenance, capability_store, guardrails, honeypot, redundancy, rerun, watermark_detector, invite, fairness, forge, pow_counter, types, error)
- Depends on: nexus-core-rs, rusqlite + rusqlite_migration
- Used by: nexus-shell-daemon

**Shell Engine Layer (nexus-shell-daemon-core):**
- Purpose: Headless engine for curator runtime, browse aggregation, blob-serve, auth, canary signing
- Location: `crates/nexus-shell-daemon-core/src/`
- Contains: 20 modules (iroh_runtime, browse, blob_serve, auth, canary/*, publish, config, registry, state, trust_web, trust_cache, bootstrap_allowlist, browse_limiter, feed_limiter, storage_limiter, pow_policy_loader, key_rotation_handler, ipc_broker, transport_probe, paths)
- Depends on: nexus-core-rs, nexus-events-core, rusqlite, dashmap, frost-ed25519
- Used by: nexus-shell-daemon, nexus-launcher

**Worker Engine Layer (nexus-worker-core):**
- Purpose: Worker state machine, LLM backends, GPU monitoring, consent, rate limiting
- Location: `crates/nexus-worker-core/src/`
- Contains: 15 modules (engine/state, engine/runtime, engine/state_writer, llm/ollama, llm/llama_cpp, llm/factory, llm/schema_bridge, llm/watermark, gpu/nvml, gpu/noop, gpu/profile, config, consent, allowlist, rate_limit, invite, ephemeral, build_executor, paths)
- Depends on: nexus-core-rs, nexus-events-core, ollama-rs, governor, nvml-wrapper, optional: llama-cpp-2, llguidance, cudarc
- Used by: nexus-worker, nexus-shell-daemon

**Transport/HTTP Layer (nexus-shell-daemon binary):**
- Purpose: axum HTTP server, gossip transport, CLI subcommands, React shell serving
- Location: `crates/nexus-shell-daemon/src/`
- Contains: 25 modules (http, runtime, cli, deploy, feed_sync, dispatch_loop, validator_loop, apps, tasks_api, storage_api, kudos_api, invite_api, canary_api, contributor_api, quarantine_api, health_api, diagnostic_api, shell_api, consent, worker_state_api, files, logging, panic, noop_identity, named_pipe_server, uds_server)
- Depends on: all -core crates, axum 0.8, tower, hyper, tower-http

## Key Modules Detail

### nexus-core-rs Modules

| Module | File | Purpose |
|--------|------|---------|
| `node` | `src/node.rs` | `Node` struct: `Endpoint` + `Docs` + `Gossip` + `MemStore` + `Router` + `MemoryLookup`. `create_node()` / `create_node_with_config(NodeConfig)` boot the full stack. |
| `crypto` | `src/crypto.rs` | `KeyPair` (Ed25519 sign/verify), `Blake3Chain` (append-only hash chain for kudos), `blake3_hash()`, `verify()`. Keys = `[u8; 32]`, signatures = `[u8; 64]`. |
| `canonical` | `src/canonical.rs` | `canonical_bytes<T>(value, domain)` -- RFC 8785 JCS + domain prefix + `0x00` separator. 14 domain constants: `DOMAIN_TASK_V1` through `DOMAIN_FEED_V1`. |
| `task` | `src/task.rs` | `Task`, `TaskEntry` (signed), `ResultPayload`, `ResultEntry` (signed), `Claim`, `ClaimEntry` (signed). Format version 1. `redundancy_factor` excluded from canonical bytes. |
| `docs` | `src/docs.rs` | `DocsClient` / `DocHandle` wrapping iroh-docs. Author CRUD, doc lifecycle, prefix scan, live event subscription, share (read/write DocTickets). |
| `gossip` | `src/gossip.rs` | `GossipClient` / `TopicHandle` / `TopicSender` / `TopicReceiver`. PoW-gated join, age-admission join, non-blocking subscribe. `AgeAdmissionPolicy` trait. |
| `blobs` | `src/blobs.rs` | `BlobsClient` wrapping `MemStore`. `add_bytes`, `get_bytes`, `has`, `fetch_ticket` (download via BlobTicket from remote peer). |
| `curator` | `src/curator.rs` | `CuratorList` / `CuratorListEntry` (signed, max 256 entries). Per-field byte-length caps. `ContributorRegistry` trait. Revocation cache integration. |
| `pow` | `src/pow.rs` | Hashcash SHA256 PoW. `HashcashChallenge` / `HashcashProof`. `solve()`, `verify()`, `verify_at()`. Publisher-bound + topic-bound + time-bound. `EscalatingPolicy` for dynamic difficulty. |
| `keystore` | `src/keystore.rs` | `LocalFileKeyStore` impl `KeyStore`. Double-layer: Argon2id(PIN, 64 MiB/t=3) + OS keyring(kek2) + AES-256-GCM. Blob format v1 (`SBFBK1`). Duress mode (Phase B). `Identity` with `SecretBox` zeroize-on-drop. |
| `verification` | `src/verification.rs` | 3-layer `Verifier`: L1 Ed25519 signature, L2 model digest whitelist (BLAKE3), L3 logprob fingerprint hash. `VerificationReport` + trust delta + ban flag. |
| `discovery` | `src/discovery.rs` | `DiscoveryClient` wrapping `Endpoint`. `my_addr()`, `my_endpoint_addr()`, `probe_reachable()` (iroh-blobs ALPN connect probe under timeout). |
| `key_rotation` | `src/key_rotation.rs` | `KeyRotationAnnouncement` / `SignedKeyRotation`. `RevocationCache` with configurable transition window. |
| `attestations/` | `src/attestations/` | `AgeWitness` (Couche 1 Sybil, min 7d), `ContributorAttestation` (Couche 2), `DelegationCert` (Couche 3 forge binding), `ForgeContribution`. |
| `pow_gossip` | `src/pow_gossip.rs` | `PowEnvelope` (message + PoW proof), `PowSolveCache` (15-min publisher cache), `PowVerifyCache` (receiver amortization). |
| `schemas/` | `src/schemas/task_response.rs` | `TaskResponse` with `#[derive(JsonSchema)]` for structured LLM output. |

### nexus-coordinator-rs Key Modules

| Module | File | Purpose |
|--------|------|---------|
| `db` | `src/db.rs` | `CoordinatorDb` wrapping rusqlite. 13+ migrations. Tables: tasks, kudos, pow_task_counts, contributor_attestations, invites, quarantine_messages, capabilities, canary_inputs, upload_queue, public_feed, apps. |
| `dispatcher` | `src/dispatcher.rs` | `submit_task()`: validate, sign TaskEntry, persist. Build tasks: enforce metadata + redundancy >= 3. |
| `public_feed` | `src/public_feed.rs` | `FeedEntry` / `FeedEntryCanonical` / `PublicFeedOperation` (ReleasePublished, SourceBecameStale). BLAKE3 hash-chain + Ed25519 per-entry. |
| `output_filter` | `src/output_filter.rs` | LLM output filtering: prompt echo detection via edit distance (strsim), PII regex patterns. |
| `watermark_detector` | `src/watermark_detector.rs` | SynthID-style z-test on output token IDs for watermark detection. |
| `redundancy` | `src/redundancy.rs` | Multi-worker redundancy voting (majority result acceptance). |

## Data Flow

### Task Lifecycle

1. React shell `POST /api/daemon/tasks/submit` -> daemon HTTP -> `nexus_coordinator_rs::dispatcher::submit_task()` signs `TaskEntry` with coordinator keypair, persists to SQLite
2. Dispatch loop reads pending tasks from DB, broadcasts signed `TaskEntry` via iroh-gossip
3. Worker receives `TaskEntry`, checks consent/allowlist/rate-limit, signs `ClaimEntry`, writes to iroh-docs
4. Worker engine drives LLM backend (Ollama HTTP or llama.cpp in-process), collects `ResultPayload`
5. Worker signs `ResultEntry`, writes to iroh-docs
6. Coordinator reads result via iroh-docs subscription, runs 3-layer verification, credits kudos or quarantines

### Curator List Flow

1. Curator signs `CuratorListEntry`, stores blob via `BlobsClient::add_bytes()`, broadcasts `{ v: 1, curator: hex, ticket: blob_ticket }` on gossip topic `blake3("nexus-grid/curator/v1")`
2. Shell daemon gossip receiver checks curator pubkey against attention set, fetches blob via `BlobsClient::fetch_ticket()`
3. `CuratorListEntry::verify_signature()` checks version, entry count cap (256), attribution, Ed25519 signature
4. Verified entry stored in `DashMap<curator_pubkey, CuratorListEntry>` with revision dedup (monotonic counter)
5. `BrowseAggregator` unions all entries, probes each project via `DiscoveryClient::probe_reachable()`, serves to React shell

### Identity Unlock Flow

1. `nexus-launcher` calls `LocalFileKeyStore::init(pin)`: generates Ed25519 keypair, derives KEK via Argon2id(PIN, salt, 64 MiB), wraps with AES-256-GCM, stores kek2 in OS keyring, writes blob to `identity.enc`
2. `nexus-launcher` calls `unlock(pin)` or `unlock_differential(pin)` (tries normal then duress): reads blob, re-derives KEK, AEAD open
3. Launcher exports secret as `SBFB_IDENTITY_SECRET_HEX` env var, spawns daemon child
4. Daemon reads + wipes env var, passes bytes to `NodeConfig::with_secret_key()`

### Public Feed Flow

1. Coordinator writes `FeedEntry` with `PublicFeedOperation::ReleasePublished` to local SQLite
2. `FeedEntryCanonical` serialized via JCS with `DOMAIN_FEED_V1`, BLAKE3-hashed, Ed25519-signed
3. Entry hash-chained (`prev_hash` = previous entry's `entry_hash`) for append-only integrity
4. Feed synced to peers via iroh-docs subscription

**State Management:**
- Backend: SQLite (source of truth for coordinator DB, worker allowlist, trust cache)
- In-memory: DashMap (curator lists, browse cache, PoW verify cache)
- Frontend: Zustand + React Query (see `web/` -- not in Rust scope)

## Key Traits and Implementations

| Trait | Location | Implementations |
|-------|----------|----------------|
| `KeyStore` | `nexus-core-rs/src/keystore.rs` | `LocalFileKeyStore` (blob + OS keyring) |
| `ContributorRegistry` | `nexus-core-rs/src/curator.rs` | Coordinator HTTP proxy, in-memory stub (tests) |
| `AgeAdmissionPolicy` | `nexus-core-rs/src/gossip.rs` | `BootstrapAllowlist` (daemon-core), stub (tests) |
| `QuorumResolver` | `nexus-core-rs/src/dht_quorum.rs` | `PkarrQuorumResolver` |
| `DnsFallbackResolve` | `nexus-core-rs/src/dns_fallback.rs` | `DnsFallbackResolver` (hickory DoH/DoT) |
| `EventWriter` | `nexus-events-core/src/lib.rs` | `JsonFileWriter`, `TracingWriter`, `JournaldWriter`, `OsLogWriter` |
| `TraceProcessor` | `nexus-trace-core/src/lib.rs` | `BatchLogProcessor`, `OtelProcessor`, `SignedCanaryProcessor` |
| `GpuMonitor` | `nexus-worker-core/src/gpu/mod.rs` | `NvmlBackend`, `NoopBackend` |
| `LlmBackend` (implied) | `nexus-worker-core/src/llm/` | `OllamaBackend`, `LlamaCppBackend` (feature-gated) |

## HTTP API Surface

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| GET | `/health` | No | Liveness probe |
| GET | `/api/daemon/info` | Bearer | Daemon state snapshot |
| GET | `/api/daemon/curators` | Bearer | Cached curator lists |
| POST | `/api/daemon/curators/subscribe` | Bearer | Subscribe to curator |
| DELETE | `/api/daemon/curators/{pubkey}` | Bearer | Unsubscribe |
| GET | `/api/daemon/browse` | Bearer | Aggregated browse entries with reachability |
| POST | `/api/daemon/publish` | Bearer | Publish project announcement |
| POST | `/api/daemon/publish-blob` | Bearer | Upload zip archive blob |
| POST | `/api/daemon/deploy` | Bearer | Deploy from Git repo (verified deploy) |
| GET | `/api/daemon/default-curators` | Bearer | Config-provided curator list |
| POST | `/api/daemon/panic/wipe` | Bearer | Irreversible identity wipe |
| GET | `/api/daemon/diagnostic/neighborhood` | Bearer | Peer snapshot |
| GET | `/blob-serve/{hash}/{path}` | No | Serve file from decompressed zip archive |
| POST | `/api/daemon/tasks/submit` | Bearer | Submit task for dispatch |
| GET | `/api/daemon/tasks` | Bearer | List tasks |
| POST | `/api/daemon/storage/get` | Bearer | iroh-docs get (iframe bridge) |
| POST | `/api/daemon/storage/set` | Bearer | iroh-docs set (iframe bridge) |
| GET | `/api/daemon/kudos` | Bearer | Kudos ledger query |
| POST | `/api/daemon/invites` | Bearer | Create invite token |
| GET | `/api/daemon/worker-state` | Bearer | Worker state snapshot relay |
| POST | `/api/daemon/consent/set` | Bearer | Set consent level |
| GET | `/api/daemon/apps` | Bearer | App metadata |
| GET | `/api/daemon/feed` | Bearer | Public feed entries |

## Database Schema (Coordinator)

**Location:** `~/.sbfb/coordinator.db` (SQLite WAL mode)

**Schema versioning:** `rusqlite_migration` with 13+ incremental migrations in `crates/nexus-coordinator-rs/src/db.rs`.

**Core tables:**
- `tasks` (task_id PK, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash)
- `kudos` (entry_id PK, worker_node_id, task_id, project_id, amount, created_at, prev_hash, entry_hash)
- `public_feed` (seq PK, op_type, entry_hash, prev_hash, signature, author_pubkey, timestamp, payload_json)
- `apps` (name + project_id composite PK, metadata)
- `contributor_attestations` (UNIQUE project_id + contributor_node_id)
- `invites` (id PK, wire UNIQUE, scope, project_id, expires_at, max_uses, uses_count)
- `quarantine_messages` (id PK, topic, sender_pubkey_hex, payload_json, rate_strikes, pow_status)
- `capabilities` (node_id + name PK, enabled)
- `pow_task_counts` (consumer_id + model_id PK, count, last_reset_utc)

## Error Handling

**Strategy:** `thiserror` for library crates (typed error enums), `anyhow` for binary crates (context chaining).

**Key error types:**
- `nexus_core_rs::NexusError` -- 8 variants: Endpoint, Discovery, Docs, Gossip, Blobs, Crypto, Io, Other
- `nexus_core_rs::keystore::KeyStoreError` / `UnlockError` -- separate types for init vs unlock (different security semantics)
- `nexus_core_rs::pow::PowError` -- 8 variants with `PartialEq` for test assertions
- `nexus_coordinator_rs::CoordinatorError` -- validation, DB, dispatch errors
- `nexus_shell_daemon_core::iroh_runtime::CuratorRuntimeError` -- gossip/fetch/verify errors

## Cross-Cutting Concerns

**Logging:** `tracing` crate in all libraries. Binaries use `tracing-subscriber` with `env-filter` + daily rotating file appender via `tracing-appender`. Non-blocking writer thread.

**Validation:** Domain-specific at every signing boundary. Per-field byte-length caps. Version checks on all wire formats.

**Authentication:** Bearer token + Host header + Origin header on all loopback HTTP. UDS peer credentials (Unix) / Named Pipe DACL (Windows). `TokenRotator` hot-reloads from `tokens.json`.

**Signing:** ALL signed payloads use `canonical_bytes(value, DOMAIN_*_V1)`. 14 domain constants prevent cross-type replay.

**Security events:** `nexus_events_core::SecurityEvent` (14 categories) emitted to platform-specific backends (JSONL, journald, oslog).

## Build Configuration

**Release profile:** opt-level=3, LTO=fat, codegen-units=1, strip=symbols, panic=abort. Deterministic for SLSA attestation.

**Dev profile:** opt-level=0, debug=true, incremental=true. Dependencies at opt-level=1.

**iroh stack pinned:** iroh 0.98 / iroh-docs 0.98 / iroh-gossip 0.98 / iroh-blobs 0.100.

**Feature flags:**
- `nexus-core-rs`: `tor` (optional arti-client)
- `nexus-worker-core`: `llm_llama_cpp`, `llm_llama_cpp_cuda/metal/vulkan`, `gpu-ephemeral`
- `nexus-coordinator-rs`: `test-support`

## Entry Points

| Binary | Location | Subcommands |
|--------|----------|-------------|
| `nexus-shell-daemon` | `crates/nexus-shell-daemon/src/main.rs` | `start`, `stop`, `status`, `config`, `canary`, `frost`, `invite`, `capability`, `quarantine` |
| `nexus-worker` | `crates/nexus-worker/src/main.rs` | `start`, `register`, `config` |
| `nexus-launcher` | `crates/nexus-launcher/src/main.rs` | Spawns daemon + opens browser + tray icon |
| `nexus-executor` | `crates/nexus-executor/src/main.rs` | IPC child process for isolated task execution |

---

*Architecture analysis: 2026-05-18*
