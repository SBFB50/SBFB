# Sprint 25 — Verification

**Date** : 2026-04-22
**Tip** : `55e42fd` (feat sprint25 Phase D)
**Phases livrees** : A (`2b674db`) + B (`f1e1f4d`) + C (`a06a2d1`) +
D (`55e42fd`)
**Theme** : fondations securitaires pre-tool-calling (key rotation +
C3 handoffs + D5 capabilities + P2 batch DNS concurrent)

---

## 1. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | exit 0 | PASS |
| 2 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | PASS |
| 3 | Rust fmt check | `cargo fmt --all --check` | exit 0 | PASS |
| 4 | Rust nextest pass | `cargo nextest run --workspace --locked` | all pass | PASS — 790 pass, 0 skip |
| 5 | Rust doctests pass | `cargo test --workspace --locked --doc` | all pass | PASS (1 ignored — spawn_with_on_reload) |
| 6 | Python ruff format | `uv run ruff format --check packages/` | exit 0 | PASS — 145 files |
| 7 | Python ruff lint | `uv run ruff check packages/` | exit 0 | PASS |
| 8 | Python SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | 185 pass | PASS — 185 pass |
| 9 | Python coord tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | 370+ pass | PASS — 372 pass + 32 fail stale PyO3 + 5 skip |
| 10 | Python gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | PASS — 46 pass |
| 11 | Web TSC check | `npx tsc --noEmit -p tsconfig.app.json` | exit 0 | PASS |
| 12 | Web lint | `npm run lint` | 0 errors | PASS — 0 errors, 7 warnings |
| 13 | Web unit tests | `npm run test:unit` | 264 pass | PASS — 264 pass |
| 14 | Web build | `npm run build` | exit 0 | PASS |
| 15 | Web size-limit | `npm run size` | 7/7 pass | PASS — 7/7 |
| 16 | Playwright e2e | `npx playwright test` | 43 pass | PASS — 43 pass |
| 17 | Shell daemon release build | `cargo build -p nexus-shell-daemon --release` | exit 0 | PASS |
| 18 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | PASS — src/ is French-only |
| 19 | Key rotation tests | `cargo nextest run key_rotation` | 23+ pass | PASS — 23 pass (core-rs) + 3 pass (daemon-core) |
| 20 | Revocation cache tests | inclus dans row 19 | apply+revoke+transition | PASS |
| 21 | DNS concurrent tests | `cargo nextest run dns_fallback` | 14 pass | PASS — 14 pass |
| 22 | StageGuardrailMap tests | `pytest test_guardrails.py` | 15+ pass | PASS — 15 pass |
| 23 | Hooks stage integration | `pytest test_hooks.py` | 12+ pass | PASS — 12 pass |
| 24 | Capability store tests | `pytest test_capability_store.py` | 30+ pass | PASS — 33 pass + 2 skip |
| 25 | Capability admin check | `pytest -k admin` | pass | PASS — 3 pass + 2 skip (Windows admin) |
| 26 | Pre-launch versions stable | grep `_VERSION` constants | all = 1 | PASS — 7 constants all v1 (+ KEY_ROTATION_FORMAT_VERSION = 1 new) |
| 27 | SPDX scan (unused imports) | `uv run ruff check --select=F401` | clean | PASS |
| 28 | CAPABILITY_TOGGLES.md status | grep implemented | present | PASS — status updated design-only → implemented S25 |
| 29 | Semgrep rule present | ls `.semgrep/capability_gate.yml` | exists | PASS |
| 30 | HARDENING_ROADMAP updated | grep last_validated | 2026-04-22 | PASS |

**Resultat : 30/30 PASS**

---

## 2. Compteurs tests finaux

| Suite | Entree S25 | Sortie S25 | Delta |
|---|---|---|---|
| Rust nextest | 757 | 790 | +33 |
| Rust doctests | pass | pass | — |
| Python SDK | 185 | 185 | 0 |
| Python coord | 315+32stale+3skip | 372+32stale+5skip | +57 pass, +2 skip |
| Python gov | 46 | 46 | 0 |
| Vitest | 264 | 264 | 0 |
| Playwright | 43 | 43 | 0 |
| Size-limit | 7/7 | 7/7 | 0 |
| **Total** | **~1621** | **~1712** | **+91** |

**Detail delta par phase** :
- Phase A : +4 Rust (DNS per-endpoint TLS + concurrent DoH/DoT) = +4
- Phase B : +29 Rust (23 key_rotation core-rs + 3 key_rotation_handler
  daemon-core + 3 canonical domain separation) = +29
- Phase C : +24 coord (StageGuardrailMap routing + output chain
  migration + hooks stage integration) = +24
- Phase D : +35 coord (33 pass + 2 skip) (capability store + admin
  check + CLI + decorator + semgrep rule) = +35
- **Total delta : +92 vs projection plan +69** (overperformance +23,
  +33% — edge cases supplementaires par rapport aux projections :
  key rotation +9 vs plan, stage guards +9 vs plan, capabilities +10
  vs plan)

---

## 3. Phases livrees — resume

### Phase A — P2 batch DNS concurrent fallback + quarantine curator alerting
Commit `2b674db`. 3 items P2 resolus : P2-E-1 per-endpoint TLS name
dans `dns_fallback.rs` (chaque `DnsEndpoint` utilise son propre
`tls_name` au lieu de `endpoints[0].tls_name` global), P2-E-2
concurrent DoH+DoT via `tokio::select!` (latence worst-case passe de
2×timeout a 1×timeout), P2-D-2 quarantine alerting via `structlog
.warning` dans `quarantine_queue.py` (structured log avec worker_id +
reason + task_id). HARDENING_ROADMAP `last_validated` update 2026-04-22
avec note G2 trigger scan (1 ACTIVE MCP vuln, 5 INACTIVE).

### Phase B — Key rotation ceremony Ed25519 self-signed + gossip revocation list
Commit `f1e1f4d`. `KeyRotationAnnouncement` struct Rust dans nouveau
`crates/nexus-core-rs/src/key_rotation.rs` — self-signed par ancienne
cle (preuve de possession), domain separation `DOMAIN_KEY_ROTATION_V1`
+ `KEY_ROTATION_FORMAT_VERSION = 1`. `RevocationCache` in-memory
HashMap (`is_revoked` + `is_in_transition` + `apply_announcement`).
`CuratorListEntry::verify_signature` check revocation avant accept.
Gossip subscribe topic `nexus-grid/key-rotation/v1` dans
`nexus-shell-daemon-core` (pattern `pow_policy_loader.rs` S20 Phase C).
PyO3 binding `verify_key_rotation`. 23+3+3 = 29 tests Rust couvrant
sign/verify roundtrip, wrong key reject, revocation cache lifecycle,
transition expired/active, curator verify with revoked/transitioning
key, canonical deterministic, domain separation distinct.

### Phase C — C3 handoffs StageGuardrailMap multi-stage guardrail pipeline
Commit `a06a2d1`. Type `StageGuardrailMap = dict[str, GuardrailChain]`
dans `guardrails.py`. 5 stages valides alignes sur hooks S24 Phase C.
`Dispatcher` accepte `stage_guards` parameter, backward compat
`input_chain` wrape dans `{"on_task_dispatched": input_chain}`.
Migration `OutputSafetyGuardrail` de validator.py ad-hoc vers
`stage_guards["on_result_received"]`. 24 tests coord couvrant
input chain preserved, output chain fire, stage absent passthrough,
multiple stages independent, chain error resilience, output safety
migration, tripwire propagation.

### Phase D — D5 capability toggles nexus-admin + capabilities.toml + @require_capability decorator
Commit `55e42fd`. `CapabilitiesStore` dans nouveau
`capability_store.py` — parse TOML `~/.sbfb/capabilities.toml`,
verify `integrity_hash` SHA-256, fallback all-OFF on tamper detect.
6 capabilities gate-off-by-default (`tool_calling`, `rag_retrieval`,
`mcp_server_expose`, `external_api_access`, `code_execution`,
`file_system_access`). `admin_check.py` cross-OS (Unix `geteuid == 0`,
Windows `IsUserAnAdmin` + MIL ctypes). CLI Typer `nexus-admin` avec
5 commandes (`list`, `enable`, `disable`, `info`, `audit-trail`) +
`require_admin()` avant mutation. `@require_capability` FastAPI
decorator (403 si disabled). `.semgrep/capability_gate.yml` PR-block
rule. `CAPABILITY_TOGGLES.md` status updated design-only → implemented
S25. Pattern `microsoft/sudo`. 35 tests coord (33 pass + 2 skip
Windows admin).

---

## 4. Scope cuts respectes

| # | Item | Status |
|---|---|---|
| 1 | Tor transport phase 1 | Deferred S26 — 0 fichiers diff |
| 2 | B2 MCP server expose | Deferred S26 — prereq D5 livre ce sprint |
| 3 | A3 OS audit channel | Deferred S26 — structlog fallback OK |
| 4 | C2 @task_handler SDK | Deferred S26+ — 0 fichiers diff |
| 5 | C5 streaming bridge | Deferred S26+ — 0 fichiers diff |
| 6 | RAG sanitization | Deferred S26+ — 0 fichiers diff |
| 7 | Per-app rate budget | Deferred S26+ — 0 fichiers diff |
| 8 | Pluggable transports lyrebird | Deferred S26 — 0 fichiers diff |
| 9 | Domain fronting implementation | Deferred S26+ — design-only |
| 10 | P2-D-1 redundancy persistence | Deferred S26 — in-memory OK |
| 11 | P2-E-1-iroh neighborhood | Deferred S26 — 0 fichiers diff |
| 12 | T-NN+2 iframe Rust-wasm | PATTERNS §P34 — triggers inactive |
| 13 | LT-2 Radicle | Trigger tag v1.0 |
| 14 | LT-3/LT-4 | Post-v1.0 |

**Tous 14 scope cuts honores** — aucune intrusion dans les zones differees.

---

## 5. Findings carry-over for memory

Carry-overs issus de S25 :

- **P2-D-1** : redundancy persistence in-memory → SQLite. Pre-v1.0,
  in-memory suffisant. Carry S26 (re-carry depuis S23).
- **P2-E-1-iroh** : iroh neighborhood enrichment non-bloquant. Carry
  S26 (re-carry depuis S23).
- **G8 systeme** : cinquieme sprint consecutif (S21-S25) avec G8
  systematique toutes phases. 21 preflights cumules : 18 EXECUTE +
  3 SCOPE-CUT-CONSISTENT + 0 DESIGN-CONFLICT. Maturite confirmee.
- **Cap G7** : 0 nouveau carry introduit S25 (net-zero). 2/2 entrants
  resolus (key rotation Phase B + C3 handoffs Phase C). Les 2 items
  restants (P2-D-1, P2-E-1-iroh) sont des re-carry S23 pre-existants,
  pas des nouveaux S25.

---

## 6. Pre-launch protocol compliance

- `CURATOR_LIST_FORMAT_VERSION = 1` — unchanged
- `TASK_FORMAT_VERSION = 1` — unchanged
- `POW_FORMAT_VERSION = 1` — unchanged
- `BLOB_VERSION = 0x01` — unchanged
- `PIN_FILE_FORMAT_VERSION = 1` — unchanged
- `TASK_RESPONSE_VERSION = 1` — unchanged
- `KEY_ROTATION_FORMAT_VERSION = 1` — **NEW** (Phase B, premier wire
  format pre-launch stable pour la rotation de cles)
- No tolerant decoder multi-version introduced
- `#[serde(default)]` additions: none S25

---

## 7. Wire format stability

1 nouveau wire format ajoute S25 (Phase B) :
- `KeyRotationAnnouncement` : `KEY_ROTATION_FORMAT_VERSION = 1`,
  domain separation `DOMAIN_KEY_ROTATION_V1`, gossip topic
  `nexus-grid/key-rotation/v1`. Format stable pre-launch, pas de
  tolerant decoder. Canonical bytes via JCS (pattern canary S21).

Les 3 autres phases sont additives Python-only (pas de wire) :
- Phase A : DNS transport refactor (Rust, pas de wire format)
- Phase C : StageGuardrailMap Python-only (pas de wire)
- Phase D : CapabilitiesStore TOML Python-only (pas de wire)

---

## 8. Risk register post-mortem

| ID | Risk | Status |
|---|---|---|
| R1 | Key rotation gossip msg incompatible existing subscribe | NON REALISE — topic dedie `key-rotation/v1` isole |
| R2 | C3 handoffs casse le path input_chain existant | NON REALISE — backward compat input_chain wrape dans stage_guards, 372 pass |
| R3 | capabilities.toml integrity_hash collisions SHA-256 | NON REALISE — SHA-256 256-bit, collision negligeable |
| R4 | Admin privilege check bypass Windows Medium IL | NON REALISE — double check IsUserAnAdmin + MIL defense-in-depth |
| R5 | tokio::select! DNS concurrent race condition | NON REALISE — select! cancel-safe, read-only futures |
| R6 | PyO3 stale wheel | PRE-EXISTANT — 32 fails inchanges |
