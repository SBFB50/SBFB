# Codebase Structure

**Analysis Date:** 2026-05-18

## Directory Layout

```
nexus-grid/
├── Cargo.toml                          # Workspace root (12 members)
├── CLAUDE.md                           # Project instructions for Claude
├── crates/
│   ├── nexus-core-rs/                  # Foundation: iroh, crypto, wire types
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                  # 25 public modules, re-exports
│   │   │   ├── node.rs                 # Node + create_node()
│   │   │   ├── crypto.rs              # KeyPair, Blake3Chain, verify()
│   │   │   ├── canonical.rs           # JCS canonical_bytes + 14 domains
│   │   │   ├── task.rs                # Task, ResultPayload, Claim (signed)
│   │   │   ├── docs.rs               # DocsClient, DocHandle
│   │   │   ├── gossip.rs             # GossipClient, TopicHandle, PoW join
│   │   │   ├── blobs.rs              # BlobsClient (MemStore)
│   │   │   ├── curator.rs            # CuratorList, CuratorListEntry (signed)
│   │   │   ├── pow.rs                # Hashcash SHA256, solve/verify
│   │   │   ├── pow_gossip.rs         # PowEnvelope, caches
│   │   │   ├── keystore.rs           # Argon2id+AES-GCM double-layer
│   │   │   ├── verification.rs       # 3-layer verifier
│   │   │   ├── discovery.rs          # DiscoveryClient, probe_reachable
│   │   │   ├── key_rotation.rs       # Rotation announcements, revocation
│   │   │   ├── dht_quorum.rs         # Redundant pkarr resolution
│   │   │   ├── pkarr_resolver.rs     # Pkarr relay client
│   │   │   ├── relay_config.rs       # Custom relay map
│   │   │   ├── relay_pow_policy.rs   # Per-topic PoW policy
│   │   │   ├── tls_pinning.rs        # SPKI SHA256 pin validation
│   │   │   ├── dns_fallback.rs       # DoH/DoT via hickory-resolver
│   │   │   ├── tor_transport.rs      # Tor via arti-client (feature-gated)
│   │   │   ├── hooks.rs              # Pre/post execution hooks
│   │   │   ├── error.rs              # NexusError (8 variants)
│   │   │   ├── attestations/         # AgeWitness, ContributorAttestation, DelegationCert
│   │   │   │   ├── mod.rs
│   │   │   │   ├── age_witness.rs
│   │   │   │   ├── contributor.rs
│   │   │   │   ├── delegation.rs
│   │   │   │   └── forge_parser.rs
│   │   │   └── schemas/              # TaskResponse with JsonSchema derive
│   │   │       ├── mod.rs
│   │   │       └── task_response.rs
│   │   ├── benches/                   # pow.rs, keystore.rs (criterion)
│   │   ├── examples/                  # two_nodes_docs_sync.rs
│   │   └── tests/                     # keystore_integration.rs, relay_federation.rs
│   │
│   ├── nexus-coordinator-rs/          # Business logic (SQLite + dispatch)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 # 20 public modules
│   │   │   ├── db.rs                  # CoordinatorDb (13+ migrations)
│   │   │   ├── dispatcher.rs          # submit_task()
│   │   │   ├── validator.rs           # Result validation
│   │   │   ├── kudos_ledger.rs        # BLAKE3 hash-chain kudos
│   │   │   ├── public_feed.rs         # FeedEntry, FeedEntryCanonical
│   │   │   ├── feed_materializer.rs   # Feed state materializer
│   │   │   ├── quarantine_queue.rs    # Suspicious message quarantine
│   │   │   ├── output_filter.rs       # LLM output filtering
│   │   │   ├── pii_redactor.rs        # PII regex detection
│   │   │   ├── provenance.rs          # SLSA L1 provenance
│   │   │   ├── capability_store.rs    # Per-node capabilities
│   │   │   ├── guardrails.rs          # Pre/post guardrail hooks
│   │   │   ├── honeypot.rs            # Honeypot detection
│   │   │   ├── redundancy.rs          # Multi-worker voting
│   │   │   ├── rerun.rs               # Task re-execution
│   │   │   ├── watermark_detector.rs  # SynthID z-test
│   │   │   ├── invite.rs              # Invite token CRUD
│   │   │   ├── fairness.rs            # Gini coefficient
│   │   │   ├── forge.rs               # Multi-forge Git ops
│   │   │   ├── pow_counter.rs         # Escalating difficulty counts
│   │   │   ├── types.rs               # TaskSubmission, TaskRecord, etc.
│   │   │   ├── canary_input.rs        # Canary input management
│   │   │   ├── canary_registry.rs     # Canary registry
│   │   │   ├── contributor_registry.rs # Contributor attestation registry
│   │   │   ├── upload_queue.rs        # Blob upload queue
│   │   │   └── error.rs              # CoordinatorError
│   │   └── tests/
│   │       └── multi_daemon.rs        # Multi-daemon integration test
│   │
│   ├── nexus-shell-daemon-core/       # Shell engine library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 # 20 public modules
│   │   │   ├── iroh_runtime.rs        # CuratorRuntime (gossip absorb)
│   │   │   ├── browse.rs             # BrowseAggregator (probe + cache)
│   │   │   ├── blob_serve.rs         # BlobServeCache (LRU zip decompress)
│   │   │   ├── auth.rs               # Bearer + Host + Origin middleware
│   │   │   ├── publish.rs            # ProjectAnnouncement wire format
│   │   │   ├── config.rs             # ShellDaemonConfig from TOML
│   │   │   ├── registry.rs           # running.json singleton
│   │   │   ├── state.rs              # DaemonStateSnapshot v1
│   │   │   ├── trust_web.rs          # Trust web (SQLite 7d TTL)
│   │   │   ├── trust_cache.rs        # Trust score caching
│   │   │   ├── bootstrap_allowlist.rs # Pre-v1.0 bootstrap nodes
│   │   │   ├── browse_limiter.rs     # GCRA rate limiter (browse)
│   │   │   ├── feed_limiter.rs       # Rate limiter (feed)
│   │   │   ├── storage_limiter.rs    # Rate limiter (storage)
│   │   │   ├── pow_policy_loader.rs  # Hot-reload PoW policy
│   │   │   ├── key_rotation_handler.rs # Key rotation gossip handler
│   │   │   ├── ipc_broker.rs         # Executor IPC broker
│   │   │   ├── transport_probe.rs    # Connectivity diagnostics
│   │   │   ├── paths.rs              # ~/.nexus-grid/ layout
│   │   │   └── canary/               # Warrant canary subsystem
│   │   │       ├── mod.rs
│   │   │       ├── signer.rs         # Ed25519CanarySigner
│   │   │       ├── frost.rs          # FrostCanarySigner (threshold)
│   │   │       ├── dkg.rs            # DKG ceremony
│   │   │       ├── ceremony.rs       # Ceremony orchestration
│   │   │       ├── attestation.rs    # Canary attestation
│   │   │       └── duress_ack.rs     # Duress acknowledgement
│   │   └── tests/
│   │       └── pow_wire.rs
│   │
│   ├── nexus-shell-daemon/            # Shell daemon binary (HTTP + gossip)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs               # CLI dispatch (clap)
│   │   │   ├── runtime.rs            # DaemonRuntime::start() boot sequence
│   │   │   ├── http.rs               # DaemonHttpState + build_router()
│   │   │   ├── cli.rs                # Clap subcommands
│   │   │   ├── deploy.rs             # deploy-from-repo (verified deploy)
│   │   │   ├── feed_sync.rs          # Feed synchronization
│   │   │   ├── dispatch_loop.rs      # Task dispatch loop
│   │   │   ├── validator_loop.rs     # Result validation loop
│   │   │   ├── apps.rs               # App metadata
│   │   │   ├── tasks_api.rs          # Task HTTP endpoints
│   │   │   ├── storage_api.rs        # iroh-docs bridge endpoints
│   │   │   ├── kudos_api.rs          # Kudos endpoints
│   │   │   ├── invite_api.rs         # Invite endpoints
│   │   │   ├── canary_api.rs         # Canary endpoints
│   │   │   ├── contributor_api.rs    # Contributor endpoints
│   │   │   ├── quarantine_api.rs     # Quarantine endpoints
│   │   │   ├── health_api.rs         # GET /health
│   │   │   ├── diagnostic_api.rs     # Peer diagnostics
│   │   │   ├── shell_api.rs          # Shell-specific endpoints
│   │   │   ├── consent.rs            # Consent management
│   │   │   ├── worker_state_api.rs   # Worker state relay
│   │   │   ├── files.rs              # Static file serving
│   │   │   ├── logging.rs            # tracing-appender daily rotation
│   │   │   ├── panic.rs              # PanicWipeService
│   │   │   ├── noop_identity.rs      # Duress-mode noop
│   │   │   ├── named_pipe_server.rs  # Windows Named Pipe (cfg(windows))
│   │   │   └── uds_server.rs         # Unix Domain Socket (cfg(unix))
│   │   └── tests/
│   │       ├── e2e.rs                # End-to-end daemon tests
│   │       └── loopback_token.rs     # Auth token tests
│   │
│   ├── nexus-worker-core/             # Worker engine library
│   │   ├── Cargo.toml                 # Features: llm_llama_cpp, gpu-ephemeral
│   │   ├── src/
│   │   │   ├── lib.rs                 # 12 public modules
│   │   │   ├── engine/
│   │   │   │   ├── mod.rs             # Re-exports
│   │   │   │   ├── state.rs           # WorkerState enum + StateMachine
│   │   │   │   ├── runtime.rs         # Engine async loop
│   │   │   │   └── state_writer.rs    # WorkerStateSnapshot to disk
│   │   │   ├── llm/
│   │   │   │   ├── mod.rs             # LlmBackend trait
│   │   │   │   ├── ollama.rs          # OllamaBackend (HTTP)
│   │   │   │   ├── llama_cpp.rs       # LlamaCppBackend (feature-gated)
│   │   │   │   ├── factory.rs         # Backend factory from config
│   │   │   │   ├── schema_bridge.rs   # JSON Schema format bridging
│   │   │   │   └── watermark.rs       # HMAC-SHA256 watermark injection
│   │   │   ├── gpu/
│   │   │   │   ├── mod.rs             # GpuMonitor trait
│   │   │   │   ├── nvml.rs            # NvmlBackend
│   │   │   │   ├── noop.rs            # NoopBackend
│   │   │   │   └── profile.rs         # GPU profiling
│   │   │   ├── config.rs             # WorkerConfig from worker.toml
│   │   │   ├── consent.rs            # 4-level consent + caps + hot-reload
│   │   │   ├── allowlist.rs          # SQLite project allowlist
│   │   │   ├── rate_limit.rs         # GCRA multi-tier rate limiter
│   │   │   ├── rate_limit_policy_loader.rs # Policy hot-reload
│   │   │   ├── invite.rs             # nx1 + BASE32 invite tokens
│   │   │   ├── ephemeral.rs          # VRAM wipe (gpu-ephemeral feature)
│   │   │   ├── build_executor.rs     # Build task execution
│   │   │   └── paths.rs              # WorkerPaths via directories crate
│   │
│   ├── nexus-worker/                  # Worker binary (CLI + TUI)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs               # CLI dispatch
│   │   │   ├── cli.rs                # start, register, config
│   │   │   ├── tui.rs                # ratatui + crossterm dashboard
│   │   │   └── logging.rs            # tracing-appender
│   │   └── tests/e2e.rs
│   │
│   ├── nexus-launcher/                # Launcher binary (daemon + browser + tray)
│   │   ├── Cargo.toml
│   │   ├── build.rs                   # Windows resource (winresource)
│   │   └── src/
│   │       ├── main.rs               # Tray icon, daemon spawn, browser open
│   │       ├── unlock.rs             # sbfb init/unlock (keystore)
│   │       ├── auth.rs               # Bearer token management
│   │       ├── driver_check.rs       # NVIDIA driver + NVD CVE check
│   │       ├── token_rotation.rs     # 24h token rotation
│   │       └── tray.rs               # System tray icon
│   │
│   ├── nexus-events-core/             # Security audit events
│   │   ├── Cargo.toml                 # Platform deps: libsystemd, oslog
│   │   └── src/lib.rs                 # SecurityEvent (14), EventWriter trait
│   │
│   ├── nexus-executor/                # Isolated compute process
│   │   ├── Cargo.toml
│   │   ├── benches/cold_start.rs
│   │   └── src/
│   │       ├── main.rs               # IPC loop, heartbeat, dispatch
│   │       ├── task_runner.rs         # Ollama-backed execution
│   │       └── ipc.rs                # JSON-RPC message types
│   │
│   ├── nexus-trace-core/              # Trace infrastructure
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                 # TraceEvent, TraceProcessor trait
│   │       ├── batch_log.rs           # BatchLogProcessor (JSONL rotation)
│   │       ├── otel.rs               # OtelProcessor (OpenTelemetry 0.31)
│   │       ├── signed.rs             # SignedCanaryProcessor (Ed25519)
│   │       └── propagation.rs        # W3C Trace Context
│   │
│   └── nexus-test-harness/            # Multi-daemon test harness
│       ├── Cargo.toml
│       ├── src/lib.rs                 # DaemonHandle spawn/health/shutdown
│       └── tests/
│           ├── multi_daemon.rs        # Multi-daemon integration
│           ├── cross_daemon_blob.rs   # Cross-daemon blob transfer
│           └── blob_serve_coep.rs     # COOP/COEP header tests
│
├── web/                               # React frontend (TypeScript)
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.app.json
│   ├── src/
│   │   ├── pages/                     # Browse, Curators, Network, etc.
│   │   ├── components/
│   │   ├── stores/                    # Zustand
│   │   └── api/                       # React Query + fetch
│   └── scripts/scan-en-strings.sh
│
├── examples/
│   ├── hello-world-app/               # Minimal SBFB app example
│   ├── sbfb-explorer/                 # Protocol Explorer (5 sections)
│   └── sbfb-ideas/                    # Ideas Hub (vote + storage P2P)
│
├── docs/
│   ├── claude/README.md               # Workflow source of truth
│   ├── claude/SPRINT_LOG.md           # Sprint history table
│   ├── rust/PATTERNS.md               # Rust patterns + tech debt
│   ├── shell/PATTERNS.md              # Shell/coordinator patterns
│   └── security/                      # THREAT_MODEL.md, RUNTIME_ISOLATION.md
│
├── .planning/
│   ├── active/                        # Current sprint docs
│   ├── archive/v1.0/                  # S0-S13
│   ├── archive/v1.1/                  # S14-S15
│   ├── archive/v1.2/                  # S16-S64
│   ├── research/                      # Research documents
│   └── codebase/                      # THIS directory (architecture docs)
│
└── tools/png-to-icns/                 # macOS icon conversion
```

## Key File Locations

**Entry Points:**
- `crates/nexus-shell-daemon/src/main.rs`: Central daemon binary (most important)
- `crates/nexus-worker/src/main.rs`: Worker binary
- `crates/nexus-launcher/src/main.rs`: Launcher binary
- `crates/nexus-executor/src/main.rs`: Executor binary

**Configuration:**
- `Cargo.toml`: Workspace root with all shared dependency versions
- `crates/*/Cargo.toml`: Per-crate dependencies and feature flags
- `web/vite.config.ts`: Frontend build config
- `web/package.json`: Frontend dependencies

**Core Logic (Rust):**
- `crates/nexus-core-rs/src/`: All cryptographic primitives, wire types, iroh wrappers
- `crates/nexus-coordinator-rs/src/`: All coordinator business logic
- `crates/nexus-shell-daemon-core/src/`: Daemon engine (curator runtime, browse, auth)
- `crates/nexus-worker-core/src/`: Worker engine (state machine, LLM, GPU)

**Testing:**
- `crates/*/tests/`: Integration tests per crate
- `crates/nexus-test-harness/`: Multi-daemon test infrastructure
- `crates/nexus-core-rs/benches/`: Criterion benchmarks (pow, keystore)

## Naming Conventions

**Files (Rust):**
- `snake_case.rs` for all source files (e.g., `key_rotation.rs`, `blob_serve.rs`)
- `mod.rs` for directory modules (e.g., `engine/mod.rs`, `canary/mod.rs`)
- Integration tests: descriptive `snake_case.rs` (e.g., `keystore_integration.rs`, `multi_daemon.rs`)

**Crates:**
- Libraries: `nexus-{domain}-{suffix}` with `suffix` being `rs` (core), `core` (engine), or nothing
- Binaries: `nexus-{role}` (worker, launcher, executor) or `nexus-shell-daemon`

**Modules:**
- Rust modules match filename: `pub mod blob_serve;` -> `src/blob_serve.rs`
- Submodules use directories: `pub mod canary;` -> `src/canary/mod.rs`

**Types (Rust):**
- Structs/enums: `PascalCase` (e.g., `CuratorListEntry`, `WorkerState`, `HashcashProof`)
- Traits: `PascalCase` (e.g., `KeyStore`, `ContributorRegistry`, `AgeAdmissionPolicy`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `CURATOR_LIST_MAX_ENTRIES`, `DOMAIN_TASK_V1`)
- Functions: `snake_case` (e.g., `canonical_bytes`, `submit_task`, `evaluate_age_admission`)

## Where to Add New Code

**New Coordinator Feature:**
1. Add module in `crates/nexus-coordinator-rs/src/{module}.rs`
2. Add `pub mod {module};` in `crates/nexus-coordinator-rs/src/lib.rs`
3. If SQLite needed: add migration in `crates/nexus-coordinator-rs/src/db.rs`
4. Add HTTP endpoint in `crates/nexus-shell-daemon/src/{feature}_api.rs`
5. Wire into `crates/nexus-shell-daemon/src/http.rs` router

**New Wire Format Type:**
1. Define struct in `crates/nexus-core-rs/src/{type}.rs` with `Serialize, Deserialize`
2. Add domain constant in `crates/nexus-core-rs/src/canonical.rs` (e.g., `DOMAIN_NEWTYPE_V1`)
3. Implement signed entry wrapper with `sign()` / `verify_signature()` pattern
4. Re-export from `crates/nexus-core-rs/src/lib.rs`

**New Worker LLM Backend:**
1. Create `crates/nexus-worker-core/src/llm/{backend}.rs`
2. Implement the backend trait pattern (see `ollama.rs` or `llama_cpp.rs`)
3. Add feature flag in `crates/nexus-worker-core/Cargo.toml` if optional
4. Wire into `crates/nexus-worker-core/src/llm/factory.rs`

**New Security Event:**
1. Add variant to `SecurityEvent` enum in `crates/nexus-events-core/src/lib.rs`
2. Emit via `emit_event()` at the appropriate callsite

**New Shell Daemon Module:**
1. Engine logic: `crates/nexus-shell-daemon-core/src/{module}.rs`
2. HTTP surface: `crates/nexus-shell-daemon/src/{module}_api.rs`
3. Wire both: `lib.rs` for core, `http.rs` + `main.rs` for daemon

**New Frontend Page:**
1. Create `web/src/pages/{PageName}.tsx`
2. Add route in `web/src/App.tsx`
3. Add API functions in `web/src/api/`

## Special Directories

**`.planning/`:**
- Purpose: Sprint planning, architecture docs, research
- Generated: No (manually maintained)
- Committed: Yes
- Layout: `active/` (current sprint), `archive/v{X}/` (closed sprints), `research/`, `codebase/`

**`examples/`:**
- Purpose: Example SBFB apps (published as zip archives, served via blob-serve)
- Contains: `hello-world-app/`, `sbfb-explorer/`, `sbfb-ideas/`
- Generated: No
- Committed: Yes

**`docs/`:**
- Purpose: Developer documentation
- Key files: `claude/README.md` (workflow), `rust/PATTERNS.md` (Rust patterns), `security/THREAT_MODEL.md`
- Generated: No
- Committed: Yes

---

*Structure analysis: 2026-05-18*
