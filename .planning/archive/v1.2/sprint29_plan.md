# Sprint 29 — Plan d'execution detaille

**Ecrit** : 2026-04-26
**Kickoff ref** : `sprint29_kickoff.md` (meme session)
**Tip master** : `102bc9f`

---

## 1. Etat verifie a l'entree

| Suite | Count | Status |
|---|---|---|
| Rust nextest | 828 | all pass |
| Rust doctests | pass | |
| Python SDK | 195 | all pass |
| Python coord | 391+36f+6s | 36 fail = stale PyO3 wheel |
| Python gov | 46 | all pass |
| Vitest | 269 | all pass |
| Playwright | 41+2f | 2 fail = env |
| Size-limit | 7/7 | |
| Clippy | 0 warnings | |
| Ruff | 0 issues | |
| **Total** | **~1814** | |

---

## 2. Decisions Day 0 (gelees) — rappel

- **D1** : Process isolation broker/executor split, raw serde_json +
  tokio UDS/NP, JSON-RPC 2.0. Nouveau binaire `nexus-executor`.
- **D2** : Cold-start benchmark Ollama 7B RTX 5080 < 5s, prereq Phase C.
- **D3** : TraceProvider opentelemetry 0.31 (PLAN-ADAPT vs roadmap 0.27).
  Crate `nexus-trace-core`, 3 backends.
- **D4** : THREAT_MODEL §9 per-mode residual risks, 6 sous-sections.
- **D5** : Responsible disclosure + scope freeze audit. D3 Windows RPC
  → S30, C4 sandbox → S30, audit engagement → S30.

---

## 3. Research consulte

- G9 WebSearch 2026-04-26 : jsonrpsee v0.26.0, opentelemetry 0.31.0,
  Trail of Bits audit prep checklist, W3C Trace Context, IPC benchmarks
- G9 Explore 2026-04-26 : daemon monolithique 8700 LOC, zero subprocess
  spawning, SecurityEvent 12 variantes (manque ExecutorCrash/BrokerCrash),
  zero opentelemetry, THREAT_MODEL §9 = regles update pas per-mode
- PROCESS_ARCHITECTURE.md (S28 Phase C, 540 LOC) : design complet
  broker/executor, 3 JSON-RPC methods, cold-start < 5s, pool N=1

---

## 4. Dependencies inter-phases

```
Phase A (P2 batch + benchmark) — autonome
    │
    └─► cold-start result valide budget < 5s ──► Phase C (split code)
                                                    │
Phase B (THREAT_MODEL §9 + disclosure) — autonome   │
    │                                                │
    └─► SECURITY.md existe ──► Phase E (audit prep)  │
                                                     │
Phase C (broker/executor split) ──────────────────► Phase D (TraceProvider)
    │  (traceparent header dans JSON-RPC              │  (wire dans broker + executor)
    │   preparé Phase C, consumed Phase D)            │
    │                                                 │
    └──────────────────────────────────────────────► Phase E (wrap-up)
```

Phase A et Phase B sont independantes. Phase C depend du resultat
benchmark Phase A. Phase D depend de Phase C (wire tracing dans les
deux processus). Phase E consolide tout.

---

## 5. Phase A — P2 batch S28 audit (8 items) + cold-start benchmark

### 5.1 Scope

Absorber les 8 P2 carry de l'audit S28 et mesurer le cold-start
Ollama 7B pour valider le budget < 5s (prereq Phase C).

### 5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-worker-core/src/llm/llama_cpp.rs` | P2-REVIEW-1 : commentaire `#[allow(too_many_arguments)]` justificatif |
| `crates/nexus-worker-core/src/llm/llama_cpp.rs` | P2-REVIEW-2 : commentaire load assumption sampler rebuild |
| `crates/nexus-events-core/src/lib.rs` | P2-B-2 : test direct `init_platform_emitter` |
| `docs/security/HARDENING_ROADMAP.md` | P2-D-1 : Note realisme S29-S30 |
| `docs/security/EXTERNAL_AUDIT_SCOPE.md` | P2-D-2 : §2.7 version verification at RFP time |
| `crates/nexus-executor/benches/cold_start.rs` | Benchmark cold-start (nouveau) |
| `crates/nexus-executor/Cargo.toml` | Crate init minimal pour benchmark |

### 5.3 Tests plan

1. `test_init_platform_emitter_selects_tracing_on_windows` — verifie
   que `init_platform_emitter()` retourne `TracingWriter` sur Windows
2. `test_init_platform_emitter_not_none` — verifie que l'emitter
   singleton est initialise
3. Benchmark `cold_start_ollama_7b` — mesure spawn + IPC + model load
   + first token. Assertion < 5s (si Ollama + model dispo).

### 5.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-events-core --locked
# P2-B-2 test vert

# Benchmark cold-start (nécessite Ollama running + model 7B)
# Résultat documenté dans commit body
```

### 5.5 Commit cible

```
feat(sprint29): Sprint 29 Phase A — P2 batch S28 + cold-start benchmark RTX 5080

## Contexte

8 P2 carry de l'audit S28 absorbes :
- P2-REVIEW-1 : commentaire justificatif generate_blocking 12 params
- P2-REVIEW-2 : load assumption sampler rebuild documentee
- P2-B-1 : scope-cut CI Linux/macOS (blocked CI infra, carry S30 2/3)
- P2-B-2 : test direct init_platform_emitter
- P2-C-1 : blob-serve isolation gap documente (broker garde blob-serve)
- P2-C-2 : cold-start benchmark RTX 5080 Ollama 7B
- P2-D-1 : Note realisme S29-S30 HARDENING_ROADMAP
- P2-D-2 : §2.7 version verification at RFP time

## Cold-start benchmark

RTX 5080 + Ollama llama3.1:8b (cache chaud):
- Spawn process + connect IPC : X ms
- Model load : X ms
- First token : X ms
- Total : X ms (< 5000 ms target)

## Fichiers

- crates/nexus-worker-core/src/llm/llama_cpp.rs (commentaires)
- crates/nexus-events-core/src/lib.rs (+2 tests)
- docs/security/HARDENING_ROADMAP.md (Note realisme)
- docs/security/EXTERNAL_AUDIT_SCOPE.md (§2.7)
- crates/nexus-executor/ (init crate + benchmark)

## Delta tests

+2 (init_platform_emitter tests)

## Verification §7.4

cargo fmt/clippy/nextest OK
uv run ruff/pytest OK
web lint/tsc/vitest/build/size/playwright/scan OK

## Scope cuts respectes (kickoff §7)

D3 Windows RPC / C4 sandbox / CI Linux/macOS / Nym / Tor : non touches.

## G8 traceability

Preflight : sprint29_phase_A_preflight.md (verdict: ...)

## Pre-launch protocol

Aucun *_VERSION bump. Aucun nouveau wire format P2P.
```

---

## 6. Phase B — THREAT_MODEL §9 + responsible disclosure docs

### 6.1 Scope

Documenter les risques residuels per-configuration dans THREAT_MODEL
§9 et creer les documents responsible disclosure pour le package
audit externe.

### 6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `docs/security/THREAT_MODEL.md` | Nouveau §9 per-mode residual risks (6 sous-sections), ancien §9 → §10 |
| `web/src/components/GpuConsentDialog.tsx` | Tooltip `level_threat_note` display |
| `packages/nexus-coordinator/src/nexus_coordinator/api/consent.py` | Populate `residual_threats_acknowledged` |
| `SECURITY.md` | Nouveau : responsible disclosure policy |
| `.well-known/security.txt` | Nouveau : RFC 9116 machine-readable |
| `BUILDING.md` | Nouveau : build instructions "batteries included" |

### 6.3 Tests plan

1. `test_consent_residual_threats_field` — verifie que l'endpoint
   consent retourne `residual_threats_acknowledged`
2. `test_security_txt_valid` — verifie format RFC 9116 (Expires,
   Contact, Preferred-Languages)
3. Vitest `GpuConsentDialog` — verifie tooltip `level_threat_note`
   rendu quand present

### 6.4 Critere d'acceptation

```bash
# THREAT_MODEL.md a 10 sections (§1-§10)
grep -c "^## " docs/security/THREAT_MODEL.md  # expect 10+

# SECURITY.md existe
test -f SECURITY.md

# security.txt valide
grep "Expires:" .well-known/security.txt

# BUILDING.md existe avec sections build
grep "cargo build" BUILDING.md

# Tests
uv run pytest packages/nexus-coordinator/tests/ -q
cd web && npm run test:unit
```

### 6.5 Commit cible

```
feat(sprint29): Sprint 29 Phase B — THREAT_MODEL §9 per-mode risks + responsible disclosure

## Contexte

Pre-audit prep : agents_sudo B4 (HARDENING_ROADMAP §3 S29) +
Trail of Bits checklist (Review Goals + batteries included).

## Fichiers

- docs/security/THREAT_MODEL.md (+§9, §9→§10 rename, ~200 LOC)
- web/src/components/GpuConsentDialog.tsx (tooltip, ~50 LOC)
- packages/nexus-coordinator/src/nexus_coordinator/api/consent.py (~20 LOC)
- SECURITY.md (nouveau, ~50 LOC)
- .well-known/security.txt (nouveau, ~15 LOC)
- BUILDING.md (nouveau, ~100 LOC)

## Delta tests

+3 (consent field + security.txt format + GpuConsentDialog tooltip)

## Verification §7.4

Idem Phase A.

## Scope cuts respectes (kickoff §7)

Tous non touches.

## G8 traceability

Preflight : sprint29_phase_B_preflight.md (verdict: ...)

## Pre-launch protocol

Aucun *_VERSION bump. Docs seulement.
```

---

## 7. Phase C — Process isolation MVP : broker/executor split

### 7.1 Scope

Le coeur technique du sprint. Implementer le split broker/executor
specifie dans PROCESS_ARCHITECTURE.md. Le broker refactore garde
toutes les responsabilites reseau/identite. Le nouvel executor
binaire gere le compute.

**Architecture IPC** :
- Canal : UDS `~/.sbfb/run/executor-{pid}.sock` (Linux/macOS),
  Named Pipe `\\.\pipe\sbfb-executor-{pid}` (Windows)
- Protocole : JSON-RPC 2.0 via raw serde_json
- Methods : `task.execute`, `health.report` (notification),
  `executor.shutdown`
- Securite : token ephemere per-task `task_token` (HMAC-SHA256),
  executor n'a pas acces a la keypair ni au master token

### 7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-executor/src/main.rs` | Nouveau binaire : CLI parse, IPC client connect, task handler, health heartbeat, shutdown handler |
| `crates/nexus-executor/src/ipc.rs` | Client IPC : JSON-RPC 2.0 message decode/encode, UDS/NP transport, traceparent extraction |
| `crates/nexus-executor/src/task_runner.rs` | Task dispatch vers worker-core engine, result packaging |
| `crates/nexus-executor/Cargo.toml` | Deps : serde_json, tokio, clap, nexus-worker-core, nexus-events-core |
| `crates/nexus-shell-daemon-core/src/ipc_broker.rs` | Nouveau module : spawn executor subprocess, IPC server JSON-RPC 2.0, health monitoring, crash backoff, task routing |
| `crates/nexus-shell-daemon-core/src/lib.rs` | Export ipc_broker module |
| `crates/nexus-shell-daemon-core/src/runtime.rs` | Ajouter executor lifecycle au boot sequence (spawn, monitor, shutdown) |
| `crates/nexus-events-core/src/lib.rs` | Ajouter variantes `ExecutorCrash { pid, exit_code, restart_count }`, `BrokerCrash { reason }` |
| `Cargo.toml` (workspace) | Ajouter `nexus-executor` au workspace members |

### 7.3 Tests plan

1. `test_json_rpc_request_roundtrip` — serde encode/decode task.execute
2. `test_json_rpc_response_roundtrip` — serde encode/decode result
3. `test_json_rpc_notification_roundtrip` — health.report sans id
4. `test_executor_spawn_and_connect` — spawn subprocess, connect IPC,
   echange health.report
5. `test_task_execute_roundtrip` — broker envoie task, executor
   retourne result via IPC
6. `test_executor_crash_detection` — kill executor, broker detecte
   deconnexion + log ExecutorCrash
7. `test_executor_crash_backoff` — crash repetitif → backoff exponentiel
   mesure (1s, 2s, 4s)
8. `test_task_token_ephemeral` — token per-task different a chaque
   requete, non-reutilisable
9. `test_executor_shutdown_graceful` — broker envoie shutdown, executor
   retourne ack + exit propre
10. `test_traceparent_propagation` — traceparent header present dans
    JSON-RPC request, extractible cote executor
11. `test_executor_crash_security_event` — ExecutorCrash emis via
    emit_event
12. `test_security_event_executor_crash_serde` — roundtrip serde
    ExecutorCrash variante

### 7.4 Critere d'acceptation

```bash
# Nouveau crate compile
cargo build -p nexus-executor

# Tests IPC
cargo nextest run -p nexus-executor --locked
cargo nextest run -p nexus-shell-daemon-core --locked

# SecurityEvent new variants
cargo nextest run -p nexus-events-core --locked

# Workspace complet reste vert
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# Benchmark cold-start Phase A valide
```

### 7.5 Commit cible

```
feat(sprint29): Sprint 29 Phase C — process isolation broker/executor split JSON-RPC 2.0 IPC

## Contexte

Premier split processus du daemon shell. Le broker (nexus-shell-daemon)
garde identite + reseau + state. L'executor (nexus-executor) gere
compute + GPU. IPC via JSON-RPC 2.0 sur UDS/Named Pipe.

Implements PROCESS_ARCHITECTURE.md (S28 Phase C design doc).

## Architecture

Broker (refactored nexus-shell-daemon):
- Keypair Ed25519, bearer auth, gossip, curator pipeline
- Module ipc_broker : spawn executor, IPC server, health monitor
- Crash detection + backoff exponentiel 1s→30s

Executor (new crate nexus-executor):
- CLI : --ipc-path <socket> [--spawn-on-demand]
- Task execution via worker-core engine
- Health heartbeat 10s
- Pas d'acces keypair ni master token

## IPC Protocol

3 methods JSON-RPC 2.0 :
- task.execute (broker→executor) : model, prompt, watermark_config, task_token
- health.report (executor→broker notification) : status, gpu_util, vram, uptime
- executor.shutdown (broker→executor) : reason, grace_period_ms

W3C Trace Context : traceparent header dans chaque requete.
Token ephemere task_token : HMAC-SHA256(master_token, task_id, timestamp).

## Fichiers

- crates/nexus-executor/ (nouveau crate, ~500 LOC)
- crates/nexus-shell-daemon-core/src/ipc_broker.rs (nouveau, ~400 LOC)
- crates/nexus-shell-daemon-core/src/runtime.rs (update, ~50 LOC)
- crates/nexus-events-core/src/lib.rs (ExecutorCrash/BrokerCrash, ~30 LOC)
- Cargo.toml workspace member

## Delta tests

+12 (IPC roundtrip, spawn, crash, backoff, token, shutdown, traceparent, SecurityEvent)

## Verification §7.4

cargo fmt/clippy/nextest OK
uv run ruff/pytest OK
web lint/tsc/vitest/build/size/playwright/scan OK

## Scope cuts respectes (kickoff §7)

D3 Windows RPC : non touche (Named Pipe S16 utilise).
C4 task-scoped sandbox : non touche (defer S30).
blob-serve executor dedie : non touche (defer S30+).

## G8 traceability

Preflight : sprint29_phase_C_preflight.md (verdict: ...)

## Pre-launch protocol

Aucun *_VERSION bump. Formats IPC = internes broker↔executor, pas wire P2P.
ExecutorCrash/BrokerCrash = events locaux, pas gossip.
```

---

## 8. Phase D — TraceProvider opentelemetry 0.31

### 8.1 Scope

Infrastructure tracing backend-agnostic. Crate `nexus-trace-core`
avec 3 backends (BatchLog, OTLP, SignedCanary). Wire dans broker
et executor.

### 8.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-trace-core/src/lib.rs` | Trait TraceProvider + TraceProcessor, API set/add_trace_processors |
| `crates/nexus-trace-core/src/batch_log.rs` | BatchLogProcessor : JSON structured → fichier |
| `crates/nexus-trace-core/src/otel.rs` | OtelProcessor : bridge opentelemetry 0.31 OTLP |
| `crates/nexus-trace-core/src/signed.rs` | SignedCanaryProcessor : Ed25519-signed trace events |
| `crates/nexus-trace-core/src/propagation.rs` | W3C Trace Context extract/inject pour JSON-RPC |
| `crates/nexus-trace-core/Cargo.toml` | Deps : opentelemetry 0.31, opentelemetry_sdk, serde, ed25519-dalek |
| `crates/nexus-shell-daemon-core/src/runtime.rs` | Wire TraceProvider init au broker startup |
| `crates/nexus-executor/src/main.rs` | Wire TraceProvider init au executor startup |
| `Cargo.toml` (workspace) | Ajouter opentelemetry deps + nexus-trace-core member |

### 8.3 Tests plan

1. `test_batch_log_processor_write_and_read` — write events, read JSONL
2. `test_batch_log_processor_rotation` — depasse seuil taille → rotate
3. `test_otel_processor_export_mock` — mock OTLP exporter recoit spans
4. `test_signed_processor_roundtrip` — sign event → verify Ed25519
5. `test_signed_processor_tamper_detect` — modify payload → verify fail
6. `test_trace_context_inject_extract` — inject traceparent dans HashMap,
   extract de l'autre cote
7. `test_trace_context_from_json_rpc` — extract traceparent depuis
   JSON-RPC request metadata `_traceparent`
8. `test_multi_processor_pipeline` — 2 processors enregistres, event
   route aux 2
9. `test_set_trace_processors_replaces` — set() remplace add()
10. `test_domain_trace_event_v1` — domain separation string correcte

### 8.4 Critere d'acceptation

```bash
# Nouveau crate compile
cargo build -p nexus-trace-core

# Tests
cargo nextest run -p nexus-trace-core --locked

# Workspace complet
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
cargo build -p nexus-executor
```

### 8.5 Commit cible

```
feat(sprint29): Sprint 29 Phase D — TraceProvider opentelemetry 0.31 backend-agnostic

## Contexte

agents_sudo A2 (HARDENING_ROADMAP §3 S29). Pre-audit tracing
infrastructure. PLAN-ADAPT : opentelemetry 0.31 (vs roadmap 0.27,
4 versions en retard, breaking changes 0.28).

## Architecture

Trait TraceProvider + TraceProcessor (pattern Observer).
3 backends :
- BatchLogProcessor : JSON structured → fichier (default)
- OtelProcessor : OTLP bridge opentelemetry 0.31
- SignedCanaryProcessor : Ed25519-signed DOMAIN_TRACE_EVENT_V1

W3C Trace Context : traceparent inject/extract pour JSON-RPC IPC.
Cross-process Rust (broker) ↔ Rust (executor) via _traceparent field.

## Fichiers

- crates/nexus-trace-core/ (nouveau crate, ~600 LOC)
- crates/nexus-shell-daemon-core/src/runtime.rs (wire, ~20 LOC)
- crates/nexus-executor/src/main.rs (wire, ~15 LOC)
- Cargo.toml workspace (opentelemetry 0.31 deps)

## Delta tests

+10 (processor pipeline, signed roundtrip, traceparent, domain)

## Verification §7.4

Idem Phase C.

## Scope cuts respectes (kickoff §7)

opentelemetry 1.0 pin : non touche (1.0 pas publie, on utilise 0.31).

## G8 traceability

Preflight : sprint29_phase_D_preflight.md (verdict: ...)

## Pre-launch protocol

DOMAIN_TRACE_EVENT_V1 = nouveau domain design-only pre-launch stable.
Events locaux uniquement, pas de wire P2P gossip. Aucun *_VERSION bump
sur formats existants.
```

---

## 9. Phase E — Wrap-up

Livrables de sortie standard :
- `sprint29_verification.md` (fail-fast checklist 30+ rows)
- `sprint30_audit_plan.md` (tracks audit S29)
- `sprint29_carry_summary.md`
- Migration `.planning/active/` → `.planning/archive/v1.2/`
- Update `CLAUDE.md` (compteurs, etat actuel)
- Update `docs/claude/SPRINT_LOG.md` (row S29)
- Update memory (`nexus_grid_pivot.md`, `MEMORY.md`)
- Scope freeze audit : documenter tip commit dans
  `EXTERNAL_AUDIT_SCOPE.md §2.7`

---

## 10. Fail-fast checklist

| # | Check | Phase | Commande | Critere | Observed |
|---|---|---|---|---|---|
| 1 | Commentaire justify generate_blocking 12 params | A | `grep "12 param\|too_many_arg" crates/nexus-worker-core/src/llm/llama_cpp.rs` | commentaire present | |
| 2 | Commentaire load assumption sampler rebuild | A | `grep "load assumption\|100 req" crates/nexus-worker-core/src/llm/llama_cpp.rs` | commentaire present | |
| 3 | Test init_platform_emitter | A | `cargo nextest run -p nexus-events-core -E "test(init_platform)"` | vert | |
| 4 | Note realisme S29-S30 HARDENING_ROADMAP | A | `grep -c "Note realisme" docs/security/HARDENING_ROADMAP.md` | ≥ 1 | |
| 5 | §2.7 version verification EXTERNAL_AUDIT_SCOPE | A | `grep "verify at" docs/security/EXTERNAL_AUDIT_SCOPE.md` | present | |
| 6 | Cold-start benchmark < 5s | A | benchmark result dans commit body | < 5000 ms | |
| 7 | THREAT_MODEL.md §9 per-mode 6 sous-sections | B | `grep -c "^### 9\." docs/security/THREAT_MODEL.md` | ≥ 6 | |
| 8 | THREAT_MODEL.md §10 = ancien §9 | B | `grep "Revue et evolution" docs/security/THREAT_MODEL.md` | sous §10 | |
| 9 | GpuConsentDialog tooltip threat_note | B | `grep "level_threat_note\|threat_note" web/src/components/GpuConsentDialog.tsx` | present | |
| 10 | SECURITY.md existe | B | `test -f SECURITY.md` | OK | |
| 11 | security.txt RFC 9116 | B | `grep "Expires:" .well-known/security.txt` | present | |
| 12 | BUILDING.md instructions build | B | `grep "cargo build" BUILDING.md` | present | |
| 13 | crate nexus-executor compile | C | `cargo build -p nexus-executor` | OK | |
| 14 | IPC JSON-RPC task.execute roundtrip | C | `cargo nextest run -p nexus-executor -E "test(json_rpc)"` | vert | |
| 15 | Executor spawn + connect | C | `cargo nextest run -p nexus-executor -E "test(spawn)"` | vert | |
| 16 | Crash detection + backoff | C | `cargo nextest run -p nexus-shell-daemon-core -E "test(crash)"` | vert | |
| 17 | Task token ephemeral | C | `cargo nextest run -E "test(task_token)"` | vert | |
| 18 | Executor shutdown graceful | C | `cargo nextest run -E "test(shutdown)"` | vert | |
| 19 | traceparent dans JSON-RPC | C | `cargo nextest run -E "test(traceparent)"` | vert | |
| 20 | SecurityEvent ExecutorCrash serde | C | `cargo nextest run -p nexus-events-core -E "test(executor_crash)"` | vert | |
| 21 | crate nexus-trace-core compile | D | `cargo build -p nexus-trace-core` | OK | |
| 22 | BatchLogProcessor write+read | D | `cargo nextest run -p nexus-trace-core -E "test(batch_log)"` | vert | |
| 23 | OtelProcessor mock export | D | `cargo nextest run -p nexus-trace-core -E "test(otel)"` | vert | |
| 24 | SignedCanaryProcessor roundtrip | D | `cargo nextest run -p nexus-trace-core -E "test(signed)"` | vert | |
| 25 | Trace context inject/extract | D | `cargo nextest run -p nexus-trace-core -E "test(trace_context)"` | vert | |
| 26 | Multi-processor pipeline | D | `cargo nextest run -p nexus-trace-core -E "test(multi)"` | vert | |
| 27 | Rust fmt + clippy clean | all | `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warn | |
| 28 | Rust nextest workspace pass | all | `cargo nextest run --workspace --locked` | all pass | |
| 29 | Rust doctests pass | all | `cargo test --workspace --locked --doc` | pass | |
| 30 | Release build daemon OK | all | `cargo build -p nexus-shell-daemon --release` | OK | |
| 31 | Python ruff + pytest pass | all | `uv run ruff format --check packages/ && uv run ruff check packages/ && uv run pytest packages/nexus-sdk/tests/ -q && uv run pytest packages/nexus-coordinator/tests/ -q && uv run pytest packages/nexus-app-gov/tests/ -q` | pass | |
| 32 | Frontend lint + tsc + vitest + build + size | all | `cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit && npm run build && npm run size` | pass | |
| 33 | Playwright pass | all | `cd web && npx playwright test` | 41+ pass | |
| 34 | scan-en-strings clean | all | `cd web && bash scripts/scan-en-strings.sh` | clean | |

---

## 11. Git plan

| # | Commit | Phase |
|---|---|---|
| 1 | `chore(planning): sprint 29 kickoff + plan + design review` | pre-A |
| 2 | `chore(planning): sprint 29 Phase A — G8 preflight + review` | A |
| 3 | `feat(sprint29): Sprint 29 Phase A — P2 batch S28 + cold-start benchmark` | A |
| 4 | `chore(planning): sprint 29 Phase B — G8 preflight + review` | B |
| 5 | `feat(sprint29): Sprint 29 Phase B — THREAT_MODEL §9 + responsible disclosure` | B |
| 6 | `chore(planning): sprint 29 Phase C — G8 preflight + review` | C |
| 7 | `feat(sprint29): Sprint 29 Phase C — process isolation broker/executor split` | C |
| 8 | `chore(planning): sprint 29 Phase D — G8 preflight + review` | D |
| 9 | `feat(sprint29): Sprint 29 Phase D — TraceProvider opentelemetry 0.31` | D |
| 10 | `chore(sprint29): Phase E — wrap-up + verification + audit plan S30 + migration` | E |

---

## 12. Scope cuts (copie kickoff §7)

1. D3 Windows RPC → S30
2. C4 task-scoped sandbox → S30
3. CI Linux/macOS writers → S30 (P2-B-1 carry 2/3)
4. Nym mixnet → S30+
5. Tor transport → S30+
6. Arti library-embed → S30+
7. Domain fronting → S30+
8. GPU lockup defense → S30+
9. C1 SQLiteSession → S30+
10. Streaming bridge C5 → S30+
11. blob-serve executor dedie → S30+
12. Full Gate 3 showcase app → post-Gate 3
13. Audit execution → S30
14. Remediation audit → post-findings
15. opentelemetry 1.0 pin → post-1.0 release

---

## 13. Risks (R1..R5)

Copie kickoff §9. Mitigations inline dans chaque phase.

---

## 14. Checkpoint de cloture

1. 34/34 fail-fast checklist verts (§10)
2. 10 commits dans le git plan (§11)
3. 2 nouveaux crates (`nexus-executor`, `nexus-trace-core`)
4. 4 nouveaux docs (`SECURITY.md`, `BUILDING.md`, `security.txt`,
   THREAT_MODEL §9)
5. Cold-start < 5s valide
6. `sprint29_verification.md` + `sprint30_audit_plan.md` ecrits
7. PATTERNS.md a jour si nouveaux patterns
8. Memory (`nexus_grid_pivot.md`, `MEMORY.md`) a jour
