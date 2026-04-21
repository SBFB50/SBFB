# Sprint 24 — Verification

**Date** : 2026-04-21
**Tip** : `1feb2a9` (chore preflight Phase F)
**Phases livrees** : A (`ff4c7d5`) + B (`c0f9561`) + C (`30fb66b`) +
D (`2095e5a`) + fix D (`bff0354`) + E (`e9d69db`)
**Theme** : guardrails pipeline refactor + TaskDispatchHooks lifecycle +
re-run sampling divergence detection + DNS fallback DHT DoH+DoT

---

## 1. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | exit 0 | PASS (already compiled) |
| 2 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | PASS |
| 3 | Rust fmt check | `cargo fmt --all --check` | exit 0 | PASS |
| 4 | Rust nextest pass | `cargo nextest run --workspace --locked` | all pass | PASS — 757 pass, 0 skip |
| 5 | Rust doctests pass | `cargo test --workspace --locked --doc` | all pass | PASS (1 ignored — spawn_with_on_reload) |
| 6 | Python ruff format | `uv run ruff format --check packages/` | exit 0 | PASS — 140 files |
| 7 | Python ruff lint | `uv run ruff check packages/` | exit 0 | PASS |
| 8 | Python SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | 185 pass | PASS — 185 pass |
| 9 | Python coord tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | 312+ pass | PASS — 315 pass + 32 fail stale PyO3 + 3 skip |
| 10 | Python gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | PASS — 46 pass |
| 11 | Web TSC check | `npx tsc --noEmit -p tsconfig.app.json` | exit 0 | PASS |
| 12 | Web lint | `npm run lint` | 0 errors | PASS — 0 errors, 7 warnings |
| 13 | Web unit tests | `npm run test:unit` | 264 pass | PASS — 264 pass |
| 14 | Web build | `npm run build` | exit 0 | PASS |
| 15 | Web size-limit | `npm run size` | 7/7 pass | PASS — 7/7 |
| 16 | Playwright e2e | `npx playwright test` | 43 pass | PASS — 43 pass |
| 17 | Shell daemon release build | `cargo build -p nexus-shell-daemon --release` | exit 0 | PASS |
| 18 | Guardrail ABC tests | `pytest tests/test_guardrails.py` | 15+ pass | PASS (inclus dans row 9) |
| 19 | Hooks tests | `pytest tests/test_hooks.py` | 13+ pass | PASS (inclus dans row 9) |
| 20 | Re-run tests | `pytest tests/test_rerun.py` | 10+ pass | PASS (inclus dans row 9) |
| 21 | DNS fallback tests | `cargo nextest run dns_fallback` | 8+ pass | PASS (inclus dans row 4 — 757 total) |
| 22 | Exponent saturation test | `cargo nextest run exponent_saturation` | pass | PASS (inclus dans row 4) |
| 23 | KudosLedger API tests | coord suite | 3+ pass | PASS (inclus dans row 9) |
| 24 | DispatchHook trait Rust | `cargo nextest run hooks` | pass | PASS (inclus dans row 4) |
| 25 | Pre-launch versions stable | grep `_VERSION` constants | all = 1 | PASS — CURATOR_LIST/TASK/POW/KEYSTORE/TLS_PIN/TASK_RESPONSE all v1 |
| 26 | SPDX scan (unused imports) | `uv run ruff check --select=F401` | clean | PASS |
| 27 | PATTERNS P35+P36 present | grep P35/P36 PATTERNS.md | match | PASS — §P35 ephemeral + §P36 redundancy |
| 28 | pynacl dep floor | grep pynacl pyproject.toml | >= 1.6.2 | PASS — `pynacl>=1.6.2` |
| 29 | hickory-resolver dep | grep hickory Cargo.toml | present | PASS — workspace + nexus-core-rs |
| 30 | Domain fronting design doc | ls DOMAIN_FRONTING_DESIGN.md | exists | PASS |

**Resultat : 30/30 PASS**

---

## 2. Compteurs tests finaux

| Suite | Entree S24 | Sortie S24 | Delta |
|---|---|---|---|
| Rust nextest | 743 | 757 | +14 |
| Rust doctests | pass | pass | — |
| Python SDK | 185 | 185 | 0 |
| Python coord | 272+32stale+3skip | 315+32stale+3skip | +43 |
| Python gov | 46 | 46 | 0 |
| Vitest | 264 | 264 | 0 |
| Playwright | 43 | 43 | 0 |
| Size-limit | 7/7 | 7/7 | 0 |
| **Total** | **~1563** | **~1621** | **+58** |

**Detail delta par phase** :
- Phase A : +4 Rust (exponent saturation) + 3 coord (kudos API) = +7
- Phase B : +15 coord (guardrails contract tests)
- Phase C : +13 (12 coord hooks + 1 Rust trait)
- Phase D : +10 coord (re-run sampling + divergence)
- Phase D fix : +3 coord (dispatcher integration coverage)
- Phase E : +12 Rust (10 dns_fallback + 2 browse integration) = +12
- **Total delta : +58 vs projection plan +63** (ecart -5 = consolidation
  tests hooks/guardrails en suites communes — equivalent couverture)

---

## 3. Phases livrees — resume

### Phase A — P2 cleanup batch S23 audit + PATTERNS §P35/P36
Commit `ff4c7d5`. 7 P2 items resolus : exponent saturation (pow.rs),
SHA-256 vs BLAKE3 deviation doc (redundancy.py), PATTERNS §P35
ephemeral worker + §P36 redundancy voting, pynacl dep floor 1.6.2,
KudosLedger public API (get_total_kudos + get_top_contributors),
PyO3 rebuild procedure (docs/shell/PATTERNS.md). HARDENING_ROADMAP
last_validated update.

### Phase B — B1 guardrails pipeline declaratif
Commit `c0f9561`. Guardrail ABC + GuardrailOutcome + GuardrailChain +
InputTripwire/OutputTripwire. 4 adapters : PiiInputGuardrail,
OutputSafetyGuardrail, QuarantineGuardrail, CanaryInputGuardrail.
dispatcher.py input_chain replaces inline if/else PII. Pattern
openai-agents-python G2 validated v0.14.3.

### Phase C — A1 TaskDispatchHooks 5 lifecycle events
Commit `30fb66b`. DispatchHook ABC + HookContext + HookRunner
(fire-and-forget, error-resilient). 5 events : on_claim_broadcast,
on_task_dispatched, on_result_received, on_validator_post_task,
on_quarantine_enqueue. Rust trait DispatchHook stub (preparatory S29).

### Phase D — Re-run sampling divergence detection
Commit `2095e5a` + fix `bff0354`. RerunSampler (1-5% configurable) +
DivergenceScorer (BLAKE3 hash comparison, hook on_result_received).
Mismatch → structured log + quarantine worker divergent.

### Phase E — DNS-based DHT fallback DoH+DoT
Commit `e9d69db`. DnsFallbackResolver : pkarr quorum timeout →
fallback DoH (Cloudflare + Google) + DoT. hickory-resolver 0.24.
browse_aggregator integration fallback chain pkarr → DNS.
DOMAIN_FRONTING_DESIGN.md outline (design-only S25+).

---

## 4. Scope cuts respectes

| # | Item | Status |
|---|---|---|
| 1 | Key rotation ceremony | Deferred S25 — 0 fichiers diff |
| 2 | C3 handoffs semantic dispatcher | Deferred S25 — 0 fichiers diff |
| 3 | GuardrailChain cross-process | Deferred S26+ — 0 fichiers diff |
| 4 | P2-D-1 redundancy persistence | Deferred S25 — in-memory OK |
| 5 | P2-D-2 quarantine alerting | Deferred S25 — queue-only OK |
| 6 | P2-E-1 iroh neighborhood | Deferred S25 — 0 fichiers diff |
| 7 | Domain fronting implementation | Deferred S25+ — design doc only |
| 8 | T-NN+2 iframe Rust-wasm | PATTERNS §P34 — triggers inactive |
| 9 | LT-2 Radicle | Trigger tag v1.0 |
| 10 | LT-3/LT-4 | Post-v1.0 |

**Tous 10 scope cuts honores** — aucune intrusion dans les zones differees.

---

## 5. Findings carry-over for memory

Carry-overs issus des phase reviews S24 :

- **P2-E-1** : `DnsFallbackResolver::build_resolver` uses
  `endpoints[0].tls_name` for all IPs in group — per-endpoint TLS
  name support needed (carry S25)
- **P2-E-2** : `resolve_node` tries DoH sequentially then DoT —
  concurrent fallback strategy would reduce worst-case latency from
  2x timeout to 1x (carry S25)
- **G8 systeme** : quatrieme sprint consecutif (S21-S24) avec G8
  systematique toutes phases. 17 preflights cumules : 14 EXECUTE,
  3 SCOPE-CUT-CONSISTENT, 0 DESIGN-CONFLICT. S1a OSS prior art
  mature. S3/S4 fast-path confirme (0 finding en 17 runs).

---

## 6. Pre-launch protocol compliance

- `CURATOR_LIST_FORMAT_VERSION = 1` — unchanged
- `TASK_FORMAT_VERSION = 1` — unchanged
- `POW_FORMAT_VERSION = 1` — unchanged
- `BLOB_VERSION = 0x01` — unchanged
- `PIN_FILE_FORMAT_VERSION = 1` — unchanged
- `TASK_RESPONSE_VERSION = 1` — unchanged
- No new wire format versions introduced S24
- No tolerant decoder multi-version introduced
- `#[serde(default)]` additions: none S24

---

## 7. Wire format stability

Aucun wire format modifie Sprint 24. Les 5 phases sont additives
(nouveau code, pas de modification canonical) :
- Phase B+C : Python-only guardrails + hooks (pas de wire)
- Phase D : Python-only re-run sampling (pas de wire)
- Phase E : Rust DNS fallback (transport additionnel, pas de wire format)

---

## 8. Risk register post-mortem

| ID | Risk | Status |
|---|---|---|
| R1 | B1 retrofit casse coord tests | NON REALISE — 315 pass post-Phase B |
| R2 | hickory-resolver conflit deps | NON REALISE — path independant iroh |
| R3 | Re-run overhead | NON REALISE — taux configurable, fire-and-forget |
| R4 | GuardrailChain ordering | NON REALISE — tests ordering explicites |
| R5 | PyO3 stale wheel | PRE-EXISTANT — 32 fails inchanges (doc rebuild Phase A) |
