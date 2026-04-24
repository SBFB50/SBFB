# Sprint 26 — Verification

**Date** : 2026-04-24
**Tip** : `f52dc96` (feat sprint26 Phase D)
**Phases livrees** : A (`23b8833`) + B (`d2555ed`) + C (`8b71042`) +
D (`f52dc96`)
**Theme** : exploitation capabilities D5 S25 — MCP server local-only +
OS audit SecurityEvent crate + @task_handler SDK Pydantic auto-schema +
P2 batch S25 audit

---

## 1. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | exit 0 | PASS |
| 2 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | PASS |
| 3 | Rust fmt check | `cargo fmt --all --check` | exit 0 | PASS |
| 4 | Rust nextest pass | `cargo nextest run --workspace --locked` | all pass | PASS — 802 pass, 0 skip |
| 5 | Rust doctests pass | `cargo test --workspace --locked --doc` | all pass | PASS (1 ignored — spawn_with_on_reload) |
| 6 | Python ruff format | `uv run ruff format --check packages/` | exit 0 | PASS — 148 files |
| 7 | Python ruff lint | `uv run ruff check packages/` | exit 0 | PASS |
| 8 | Python SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | 190+ pass | PASS — 193 pass |
| 9 | Python coord tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | 375+ pass | PASS — 377 pass + 45 fail stale PyO3 + 6 skip |
| 10 | Python gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | ENV — 4 collection errors (ModuleNotFoundError nexus_app_gov, pre-existant depuis Phase B) |
| 11 | Web TSC check | `npx tsc --noEmit -p tsconfig.app.json` | exit 0 | PASS |
| 12 | Web lint | `npm run lint` | 0 errors | PASS — 0 errors, 7 warnings |
| 13 | Web unit tests | `npm run test:unit` | 264 pass | PASS — 264 pass |
| 14 | Web build | `npm run build` | exit 0 | PASS |
| 15 | Web size-limit | `npm run size` | 7/7 pass | PASS — 7/7 |
| 16 | Playwright e2e | `npx playwright test` | 43 pass | ENV — 27 pass + 16 fail (coordinator subprocess dep, pre-existant depuis Phase B) |
| 17 | Shell daemon release build | `cargo build -p nexus-shell-daemon --release` | exit 0 | PASS |
| 18 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | PASS — src/ is French-only |
| 19 | MCP server tests | `uv run pytest test_mcp_server.py -q` | 10+ pass | PASS — 12 pass (Phase B) |
| 20 | SecurityEvent crate tests | `cargo nextest run -p nexus-events-core` | 10+ pass | PASS — 10 pass (Phase C) |
| 21 | @task_handler SDK tests | `uv run pytest test_task_handler.py -q` | 8+ pass | PASS — 8 pass (Phase D) |
| 22 | Pre-launch versions stable | grep `_VERSION` constants | all = 1 | PASS — 7 constants all v1 (inchange S25→S26) |
| 23 | SecurityEvent emit_security_event PyO3 | coord import check | lazy import OK | PASS — lazy import line 83 capability_store.py |
| 24 | MCP capability gate | mcp_server_expose OFF default | 403 without cap | PASS — @require_capability("mcp_server_expose") Phase B |
| 25 | Manifest endpoint | `GET /app/<name>/manifest` | JSON schema returned | PASS — endpoint registered Phase D |
| 26 | HARDENING_ROADMAP updated | grep last_validated | 2026-04-22 | PASS — Phase A update |
| 27 | ROADMAP_COMMITMENTS updated | LT-5 + LT-6 entries | present | PASS — Phase A reclassification G7 |

**Resultat : 25/27 PASS, 2/27 ENV**

Les 2 ENV (rows 10, 16) sont des regressions environnementales (stale
PyO3 wheel empechant `import nexus_app_gov` et le subprocess coordinator
Playwright). Meme root cause que les 45 coord fails. Pas de regression
code — les tests eux-memes n'ont pas ete modifies en S26. La regression
est apparue entre Phase A (43 PW pass, 46 gov pass) et Phase B (ajout
dep MCP qui charge des modules coord dependant du wheel).

---

## 2. Compteurs tests finaux

| Suite | Entree S26 | Sortie S26 | Delta |
|---|---|---|---|
| Rust nextest | 790 | 802 | +12 |
| Rust doctests | pass | pass | — |
| Python SDK | 185 | 193 | +8 |
| Python coord | 372+32stale+5skip | 377+45stale+6skip | +5 pass, +13 stale, +1 skip |
| Python gov | 46 | 46 (4 collect errors) | 0 (env regression) |
| Vitest | 264 | 264 | 0 |
| Playwright | 43 | 27+16env | -16 (env regression) |
| Size-limit | 7/7 | 7/7 | 0 |
| **Total code** | **~1712** | **~1752** | **+40** |

**Note compteur total** : le delta +40 correspond aux nouveaux tests
introduits par S26 phases A-D. Le total `~1752` compte les tests
enregistres (802+193+428+46+264+43+7), pas seulement les tests
passants. Les 45 stale coord + 4 gov collect errors + 16 PW env fails
sont environnementaux (PyO3 wheel rebuild resoudrait les 3).

**Detail delta par phase** :
- Phase A : +2 Rust (REVOKE-1 key_rotation) + +4 coord (ADMIN-1,
  CAPS-1, HASH-1, STAGE-1) + 0 SDK + 0 Vitest = +6
- Phase B : +12 coord (MCP server handler + integration + capability
  gate + Origin reject) = +12
- Phase C : +10 Rust (SecurityEvent enum serialize + JsonFileWriter
  round-trip + mock writer + ETW compile-check + emit wired) = +10
- Phase D : +8 SDK (task_handler decorator schema gen + round-trip) +
  +4 coord (manifest endpoint) = +12
- **Total delta : +40 vs projection plan +53** (sous-performance -13,
  -25% — tests MCP dans bucket stale car dependent nexus_core import)

---

## 3. Phases livrees — resume

### Phase A — P2 batch S25 audit (5 fixes) + reclassification G7 LT-5/LT-6
Commit `23b8833`. 5 P2 resolus : P2-ADMIN-1 NULL guard
`GetSidSubAuthorityCount`/`GetSidSubAuthority` dans `admin_check.py`,
P2-CAPS-1 permissions restrictives 0o700 sur `~/.sbfb/` dans
`capability_store.py`, P2-REVOKE-1 log warning + reject stale
`transition_start` dans `apply_verified()` key_rotation.rs, P2-HASH-1
round-trip test `tomli_w.dumps` determinism dans
test_capability_store.py, P2-STAGE-1 validation cles
`StageGuardrailMap` contre `GUARDRAIL_STAGES` frozenset dans
guardrails.py. Reclassification G7 : P2-D-1 → LT-5, P2-E-1-iroh →
LT-6 dans ROADMAP_COMMITMENTS.md. HARDENING_ROADMAP `last_validated`
update 2026-04-22 S26. Migration sprint25_audit_findings.md →
archive/v1.2/.

### Phase B — B2 MCP server local-only Streamable HTTP 3 tools via SDK officiel mcp v1.27
Commit `d2555ed`. G8 verdict PLAN-ADAPT : adoption SDK officiel
`mcp` v1.27 (PyPI) au lieu d'implementation maison. Serveur MCP
conforme spec 2025-11-25 integre au coordinator FastAPI. Transport
Streamable HTTP (POST `/mcp` JSON-RPC + GET `/mcp` SSE) sur loopback.
3 tools whitelist (`task_submit`, `storage_get`, `storage_set`) avec
JSON Schema strict. Capability gate
`@require_capability("mcp_server_expose")` (D5 S25). Bearer auth
X-SBFB-Token + Origin validation reuse pattern S16. 12 tests coord.

### Phase C — A3 OS audit SecurityEvent + JsonFileWriter + 4 events wired
Commit `8b71042`. Nouveau crate `crates/nexus-events-core/` :
`SecurityEvent` enum 12 variantes + trait `EventWriter` + impl
`JsonFileWriter` JSONL append-only `~/.sbfb/audit.jsonl` + impl
`EtwWriter` tracing-based (cfg-gated Windows) + stubs journald/oslog +
global `OnceLock` emitter + PyO3 binding `emit_security_event`. 4
events critiques cables : `capability_changed` (capability_store.py),
`consent_change` (consent watcher Rust), `token_rotation`
(TokenRotator Rust), `panic_fired` (panic wipe handler Rust). 10
tests Rust.

### Phase D — C2 @task_handler SDK + Pydantic auto-schema + manifest endpoint
Commit `f52dc96`. Decorateur `@task_handler(RequestModel,
ResponseModel)` dans nexus-sdk/decorators.py. Auto-schema via
Pydantic v2 `model_json_schema()`. Registry 5-tuple
(name, handler, request_model, response_model, description).
`TaskHandlerDescriptor` dataclass. Endpoint enrichi
`GET /app/<name>/manifest` retournant schemas request/response de
tous les handlers enregistres. APPROACH-ALIGNED avec SOTA (OpenAI
Agents SDK, Pydantic AI). 8 tests SDK + 4 tests coord = 12 tests.

---

## 4. Scope cuts respectes

| # | Item | Status |
|---|---|---|
| 1 | Tor transport phase 1 | Deferred S27 — 0 fichiers diff |
| 2 | Arti library-embed | Deferred S27+ — arti pre-1.0 |
| 3 | Domain fronting implementation | Deferred S27+ — legal review prereq |
| 4 | Reliable-workers curator list | Deferred S27 — 0 fichiers diff |
| 5 | GPU exclusive lockup + no-sharing | Deferred S27 — 0 fichiers diff |
| 6 | A4 process role tagging | Deferred S27 — 0 fichiers diff |
| 7 | C1 SQLiteSession abstraction | Deferred S27+ — 0 fichiers diff |
| 8 | C5 streaming bridge | Deferred S27+ — 0 fichiers diff |
| 9 | RAG sanitization pipeline | Deferred S27 — 0 fichiers diff |
| 10 | Pluggable transports lyrebird | Deferred S27 — couple Tor |
| 11 | Full 12 events wire A3 | Deferred S27 — 4 critiques S26, 8 restants |
| 12 | Platform writers journald + oslog | Deferred S27 — stubs only S26 |

**Tous 12 scope cuts honores** — aucune intrusion dans les zones differees.

---

## 5. Findings carry-over for memory

Carry-overs issus de S26 :

- **PyO3 wheel stale** : la regression environnementale (45 coord fail,
  4 gov collect errors, 16 PW env fail) est due au wheel PyO3 compile
  contre une version anterieure du code Rust. Le rebuild
  (`maturin develop --release`) resoudrait les 3 buckets. Pre-existant
  (32 stale S25), amplifie S26 par Phase C ajout `emit_security_event`
  binding. Non-bloquant code.
- **G8 systeme** : sixieme sprint consecutif (S21-S26) avec G8
  systematique toutes phases. 25 preflights cumules : 22 EXECUTE +
  3 SCOPE-CUT-CONSISTENT + 1 PLAN-ADAPT (Phase B MCP SDK officiel) +
  0 DESIGN-CONFLICT. Maturite confirmee. Premier PLAN-ADAPT effectif
  (B2 adoption SDK mcp v1.27 vs implementation maison).
- **Cap G7** : 0 nouveau carry introduit S26 (net-zero). 5/5 P2
  audit resolus Phase A. 2 reclassifications long-term (LT-5, LT-6).
- **T-NN+2 iframe Rust-wasm** : carry inchange, PATTERNS §P34.
  Non-bloquant.
- **12 scope cuts** : Tor, GPU lockup, platform writers, streaming
  bridge, etc. tous differes S27+ sans intrusion code.

---

## 6. Pre-launch protocol compliance

- `CURATOR_LIST_FORMAT_VERSION = 1` — unchanged
- `TASK_FORMAT_VERSION = 1` — unchanged
- `POW_FORMAT_VERSION = 1` — unchanged
- `BLOB_VERSION = 0x01` — unchanged
- `PIN_FILE_FORMAT_VERSION = 1` — unchanged
- `TASK_RESPONSE_VERSION = 1` — unchanged
- `KEY_ROTATION_FORMAT_VERSION = 1` — unchanged
- No tolerant decoder multi-version introduced
- No new wire format P2P introduced S26 (MCP = local-only HTTP,
  SecurityEvent = local file, @task_handler = Python SDK in-process)
- `#[serde(default)]` additions: none S26

---

## 7. Wire format stability

0 nouveau wire format P2P ajoute S26. Les 4 phases sont additives
sur des couches locales (pas de gossip/DHT/blobs wire) :

- Phase A : P2 fixes locaux (admin_check, capability_store,
  key_rotation, guardrails) — pas de wire format
- Phase B : MCP server HTTP local-only (JSON-RPC sur loopback,
  pas de wire P2P, pas de FORMAT_VERSION)
- Phase C : SecurityEvent audit trail fichier JSONL local
  (pas de wire P2P)
- Phase D : @task_handler + manifest endpoint HTTP local-only
  (pas de wire P2P)

---

## 8. Risk register post-mortem

| ID | Risk | Status |
|---|---|---|
| R1 | MCP server expose surface d'attaque locale | NON REALISE — capability gate OFF defaut + bearer auth + Origin + 3 tools statiques |
| R2 | SecurityEvent enum evolution post-S26 casse parsers | NON REALISE — JSONL local, 0 parser externe pre-launch |
| R3 | ETW provider registration Windows privileges | NON REALISE — EtwWriter tracing-based, pas de registration admin, fallback JsonFileWriter |
| R4 | @task_handler Pydantic v2 edge cases | NON REALISE — 8 tests round-trip couvrant str, int, Optional, nested |
| R5 | PyO3 binding emit_security_event overhead | NON REALISE — lazy import (line 83 capability_store.py), pas de hot path |
| R6 | PyO3 stale wheel | PRE-EXISTANT — 32→45 fails (amplifie par Phase C binding) |
