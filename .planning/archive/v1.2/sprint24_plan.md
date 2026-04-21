# Sprint 24 — Plan d'execution detaille

**Ecrit** : 2026-04-21
**Kickoff** : `.planning/active/sprint24_kickoff.md`
**D1..D5 gelees** : cf. kickoff §4

---

## 1. Etat verifie a l'entree

- **Tip** : `91589ea`
- **Working tree** : propre (migration sprint23_audit_findings.md
  staggee)
- **Compteurs tests** :

| Suite | Count |
|---|---|
| Rust nextest | 743 |
| Python SDK | 185 |
| Python coord | 272 pass + 32 fail stale + 3 skip |
| Python gov | 46 |
| Vitest | 264 |
| Playwright | 43 |
| Size-limit | 7/7 |
| **Total** | **~1563** |

- clippy : 0 warnings
- ruff : clean

---

## 2. Decisions Day 0 (gelees) — rappel

- **D1** : B1 guardrails Guardrail ABC + GuardrailChain + retrofit
  4 primitives coord-side
- **D2** : A1 TaskDispatchHooks 5 events lifecycle injectables
- **D3** : Re-run sampling 1-5% + DivergenceScorer hook
- **D4** : DNS fallback DHT DoH+DoT via hickory-resolver
- **D5** : Key rotation + C3 handoffs → S25

---

## 3. Research consulte

- context7 `/openai/openai-agents-python` guardrails API v0.14.3
- Design doc `docs/security/GUARDRAILS_ARCHITECTURE.md` (S22)
- Research `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md`
  clusters A+B+C
- HARDENING_ROADMAP §3 S24 line
- sprint23_audit_findings.md P2 items
- `hickory-resolver` crate (anciennement trust-dns-resolver) pour
  D4 — mature, Apache-2.0, DoH+DoT support natif

---

## 4. Dependency graph inter-phases

```
Phase A (cleanup) — independant
Phase B (guardrails) — independant
Phase C (hooks) ← Phase B (GuardrailChain emit vers hooks)
Phase D (re-run) ← Phase C (DivergenceScorer = hook consumer)
Phase E (DNS) — independant du reste
Phase F (wrap-up) ← toutes
```

---

## 5. Phase A — P2 cleanup batch S23 audit + PATTERNS §P35/P36

### 5.1 Scope

Absorber les 7 P2 du gate S23 + ecrire les 2 PATTERNS manquants.
Chaque fix est chirurgical, pas de refactor large.

### 5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-core-rs/src/pow.rs:484` | B-1 : saturer exponent avant cast i32 |
| `packages/nexus-coordinator/src/nexus_coordinator/redundancy.py:44` | C-2 : commentaire deviation SHA-256 vs BLAKE3 |
| `docs/rust/PATTERNS.md` | P35 ephemeral worker lifecycle + P36 redundancy voting |
| `packages/nexus-coordinator/pyproject.toml` | E-2 : dep floor `pynacl >= 1.6.2` |
| `packages/nexus-coordinator/src/nexus_coordinator/kudos.py` | E-3 : methodes publiques `get_total_kudos()` + `get_top_contributors(n)` |
| `docs/shell/PATTERNS.md` | F-1bis : section PyO3 rebuild procedure |
| `docs/security/HARDENING_ROADMAP.md` frontmatter | last_validated → 2026-04-21 |

### 5.3 Tests plan

1. `test_escalating_difficulty_exponent_saturation` — exponent =
   `u64::MAX` → difficulte saturee au max (pas wrap)
2. `test_kudos_get_total` — credit 3 entries, `get_total_kudos()`
   retourne somme
3. `test_kudos_get_top_contributors` — credit 5 entries, top(3)
   retourne les 3 meilleurs tries
4. `test_kudos_get_top_empty_ledger` — ledger vide → liste vide,
   pas d'erreur

### 5.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-core-rs --locked
uv run ruff check packages/ && uv run ruff format --check packages/
uv run pytest packages/nexus-coordinator/tests/ -q
```

### 5.5 Commit cible

```
feat(sprint24): Phase A — P2 cleanup batch S23 audit + PATTERNS §P35 ephemeral + §P36 redundancy

- B-1: saturate exponent before i32 cast in escalating_difficulty()
  (pow.rs:484 — exponent.min(i32::MAX as u64))
- C-2: document SHA-256 vs BLAKE3 deviation in redundancy.py
  (functionally equivalent for hash comparison, Ed25519 sigs do
  crypto integrity)
- F-1: PATTERNS.md §P35 ephemeral worker lifecycle pattern +
  §P36 redundancy voting pattern
- E-2: pyproject.toml dep floor pynacl >= 1.6.2 (CVE-2025-69277)
- E-3: KudosLedger.get_total_kudos() + get_top_contributors(n)
  public API for fairness diagnostic
- F-1bis: docs/shell/PATTERNS.md §PyO3 rebuild procedure
- HARDENING_ROADMAP last_validated → 2026-04-21

Delta tests: +4 Rust (exponent saturation) + 3 coord (kudos API)
Cumul: 747 Rust / 275+3+32stale coord / ~1570 total

Scope cuts respected: cf. kickoff §7 (10 items)
```

---

## 6. Phase B — B1 guardrails pipeline refactor

### 6.1 Scope

Creer le framework guardrails unifie :
1. ABC `Guardrail` + dataclasses `GuardrailOutcome` / `HookPayload`
2. `GuardrailChain` — pipeline ordonne avec short-circuit
3. Exceptions `InputTripwire` / `OutputTripwire`
4. Adapters : `PiiInputGuardrail` wrappant `PiiRedactor`,
   `OutputSafetyGuardrail` wrappant `OutputFilter`,
   `QuarantineGuardrail` wrappant `QuarantineQueue`,
   `CanaryInputGuardrail` wrappant `CanaryInputInjector`
5. Integration `dispatcher.py` : remplacer le if/else PII par
   `input_chain.run(ctx, req)`

### 6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `packages/.../guardrails.py` (NEW) | ABC + GuardrailOutcome + GuardrailChain + exceptions |
| `packages/.../pii_redactor.py` | Ajouter classe `PiiInputGuardrail(Guardrail)` adapter |
| `packages/.../output_filter.py` | Ajouter classe `OutputSafetyGuardrail(Guardrail)` adapter |
| `packages/.../quarantine_queue.py` | Ajouter classe `QuarantineGuardrail(Guardrail)` adapter |
| `packages/.../canary_input.py` | Ajouter classe `CanaryInputGuardrail(Guardrail)` adapter |
| `packages/.../dispatcher.py` | Remplacer inline PII par `GuardrailChain` |
| `tests/test_guardrails.py` (NEW) | Contract tests |

(`.../` = `packages/nexus-coordinator/src/nexus_coordinator/`)

### 6.3 Tests plan

Contract tests (chaque guardrail teste contre le meme contrat) :
1. `test_guardrail_abc_not_instantiable` — ABC raises TypeError
2. `test_guardrail_outcome_passed` — outcome.passed = True
3. `test_guardrail_outcome_tripwire` — outcome.tripwire = True
4. `test_chain_empty_passes` — chain vide → passe
5. `test_chain_single_pass` — 1 guardrail pass → chain pass
6. `test_chain_single_trip` — 1 guardrail trip → InputTripwire raised
7. `test_chain_short_circuit` — 2 guardrails, 1er trip → 2e jamais appele
8. `test_chain_ordering` — ordre d'execution = ordre d'insertion
Per-adapter :
9. `test_pii_input_guardrail_redacts` — PII detectee → outcome.passed
   (redaction, pas tripwire sauf config stricte)
10. `test_output_safety_guardrail_clean` — output safe → pass
11. `test_output_safety_guardrail_trip` — invisible text → tripwire
12. `test_quarantine_guardrail_trip` — quarantine condition → trip
13. `test_canary_input_guardrail_injects` — canary prompt injected
Integration :
14. `test_dispatcher_uses_input_chain` — dispatcher with chain runs
    all input guardrails
15. `test_dispatcher_no_chain_fallback` — dispatcher without chain
    behaves as before (backward compat)

### 6.4 Critere d'acceptation

```bash
uv run pytest packages/nexus-coordinator/tests/test_guardrails.py -v
uv run pytest packages/nexus-coordinator/tests/ -q  # 272+ pass net stale
uv run ruff check packages/
```

### 6.5 Commit cible

```
feat(sprint24): Phase B — B1 guardrails pipeline declaratif Guardrail ABC + GuardrailChain + retrofit 4 primitives

Guardrail ABC + GuardrailOutcome + GuardrailChain + InputTripwire/
OutputTripwire. Pattern openai-agents-python (G2 validated v0.14.3).

4 adapters: PiiInputGuardrail (PiiRedactor wrap),
OutputSafetyGuardrail (OutputFilter wrap),
QuarantineGuardrail (QuarantineQueue wrap),
CanaryInputGuardrail (CanaryInputInjector wrap).

dispatcher.py: input_chain replaces inline if/else PII.

Delta tests: +15 coord (8 contract + 4 adapter + 2 integration + 1 abc)
Cumul: 747 Rust / 290+3+32stale coord / ~1585 total

Scope cuts respected: no cross-process chain (rate-limit Rust +
PII iframe TS stay independent)
```

---

## 7. Phase C — A1 TaskDispatchHooks lifecycle events

### 7.1 Scope

1. ABC `DispatchHook` + dataclass `HookContext`
2. `HookRunner` composite (multi-hook fire-and-forget, error-resilient)
3. 5 events : `on_claim_broadcast`, `on_task_dispatched`,
   `on_result_received`, `on_validator_post_task`,
   `on_quarantine_enqueue`
4. Integration `dispatcher.py` : fire events aux 5 points
5. Trait Rust `DispatchHook` stub dans `nexus-core-rs` (pas de
   PyO3 binding S24 — stub preparatoire S29)

### 7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `packages/.../hooks.py` (NEW) | ABC DispatchHook + HookContext + HookRunner |
| `packages/.../dispatcher.py` | Ajouter `hooks: list[DispatchHook]` + fire aux 5 points |
| `packages/.../validator.py` | Fire `on_result_received` + `on_validator_post_task` |
| `crates/nexus-core-rs/src/hooks.rs` (NEW) | Trait `DispatchHook` Rust stub |
| `crates/nexus-core-rs/src/lib.rs` | `pub mod hooks;` |
| `tests/test_hooks.py` (NEW) | Hook tests |

### 7.3 Tests plan

1. `test_hook_abc_not_instantiable` — ABC raises TypeError
2. `test_hook_context_fields` — HookContext a tous les champs
3. `test_runner_single_hook_fires` — 1 hook registered, 1 event
4. `test_runner_multi_hook_ordering` — 2 hooks, fire en ordre
5. `test_runner_error_resilience` — hook raises → log, pas crash
6. `test_runner_no_hooks_noop` — 0 hooks → pas d'erreur
7. `test_on_claim_broadcast_fires` — event specifique
8. `test_on_task_dispatched_fires`
9. `test_on_result_received_fires`
10. `test_on_validator_post_task_fires`
11. `test_on_quarantine_enqueue_fires`
12. `test_dispatcher_integration_hooks` — dispatcher wired, events
    visible sur mock hook
Rust stub :
13. `test_dispatch_hook_trait_object_safe` — trait compilable, dyn-safe

### 7.4 Critere d'acceptation

```bash
uv run pytest packages/nexus-coordinator/tests/test_hooks.py -v
uv run pytest packages/nexus-coordinator/tests/ -q
cargo nextest run -p nexus-core-rs --locked
```

### 7.5 Commit cible

```
feat(sprint24): Phase C — A1 TaskDispatchHooks 5 lifecycle events + HookRunner composite + dispatcher integration

DispatchHook ABC + HookContext + HookRunner (fire-and-forget,
error-resilient). 5 events: on_claim_broadcast, on_task_dispatched,
on_result_received, on_validator_post_task, on_quarantine_enqueue.

dispatcher.py + validator.py: fire hooks at 5 dispatch lifecycle
points. Rust trait stub crates/nexus-core-rs/src/hooks.rs
(preparatory S29 TraceProvider).

Delta tests: +13 (12 coord + 1 Rust trait)
Cumul: 748 Rust / 302+3+32stale coord / ~1598 total

Scope cuts respected: no PyO3 binding S24 (Rust trait = stub only,
Python-first implementation)
```

---

## 8. Phase D — Re-run sampling + divergence detection

### 8.1 Scope

1. `RerunSampler` : selectionne aleatoirement `sample_rate`% des
   tasks completees pour re-dispatch
2. `DivergenceScorer` : hook `on_result_received`, compare hash
   canonical resultat original vs re-run
3. Auto-report : divergence detectee → log structured + quarantine
   worker divergent via `QuarantineQueue`
4. Config TOML : `rerun_sample_rate` (float 0.01-0.05, default 0.01)

### 8.2 Fichiers touches

| Fichier | Role |
|---|---|
| `packages/.../rerun.py` (NEW) | RerunSampler + DivergenceScorer |
| `packages/.../dispatcher.py` | Wire RerunSampler pre-dispatch |
| `configs/rerun_sampling.toml.sample` (NEW) | Config sample |
| `tests/test_rerun.py` (NEW) | Tests re-run |

### 8.3 Tests plan

1. `test_sampler_rate_0_never_reruns` — rate=0 → aucun re-run
2. `test_sampler_rate_1_always_reruns` — rate=1.0 → tous re-run
3. `test_sampler_rate_distribution` — rate=0.05, 1000 tasks →
   ~50 re-runs (tolerance statistique ±20)
4. `test_divergence_scorer_identical` — meme hash → score 0
5. `test_divergence_scorer_mismatch` — hash differents → score 1.0
6. `test_divergence_scorer_triggers_quarantine` — mismatch →
   quarantine enqueue fire
7. `test_divergence_scorer_as_hook` — registered, fires on
   on_result_received
8. `test_rerun_task_distinct_id` — re-run task a un task_id
   different du original
9. `test_rerun_config_parse` — TOML valide parse
10. `test_rerun_config_invalid_rate` — rate > 1.0 → clamp + warn

### 8.4 Critere d'acceptation

```bash
uv run pytest packages/nexus-coordinator/tests/test_rerun.py -v
uv run pytest packages/nexus-coordinator/tests/ -q
```

### 8.5 Commit cible

```
feat(sprint24): Phase D — re-run sampling 1-5% divergence detection + auto-report curator + quarantine divergent

RerunSampler (random selection configurable 1-5%) +
DivergenceScorer (hook on_result_received, BLAKE3 hash
comparison canonical result). Mismatch → structured log +
quarantine worker divergent.

Config: configs/rerun_sampling.toml.sample (rerun_sample_rate).

Delta tests: +10 coord
Cumul: 748 Rust / 312+3+32stale coord / ~1608 total

Scope cuts respected: no fuzzy divergence scoring S24 (binary
hash comparison only, fuzzy threshold deferred S25)
```

---

## 9. Phase E — DNS-based DHT fallback (DoH + DoT)

### 9.1 Scope

1. `DnsFallbackResolver` : quand `PkarrQuorumResolver` timeout
   (10s quorum echoue), tente resolution via DNS TXT records
   DoH (RFC 8484) + DoT (RFC 7858)
2. Integration browse aggregator : fallback chain pkarr → DNS
3. Design doc outline `docs/security/DOMAIN_FRONTING_DESIGN.md`
   (design-only, pas d'implementation)

### 9.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-core-rs/src/dns_fallback.rs` (NEW) | DnsFallbackResolver + DoH + DoT |
| `crates/nexus-core-rs/src/lib.rs` | `pub mod dns_fallback;` |
| `crates/nexus-core-rs/Cargo.toml` | dep `hickory-resolver` |
| `crates/nexus-shell-daemon-core/src/browse_aggregator.rs` | Fallback chain integration |
| `docs/security/DOMAIN_FRONTING_DESIGN.md` (NEW) | Design doc outline |

### 9.3 Tests plan

1. `test_dns_fallback_doh_resolve` — mock DoH response → parse TXT
2. `test_dns_fallback_dot_resolve` — mock DoT response → parse TXT
3. `test_dns_txt_pkarr_parse` — TXT record → pkarr packet decode
4. `test_dns_fallback_on_pkarr_timeout` — quorum timeout →
   fallback triggered
5. `test_dns_fallback_config_resolvers` — configure custom DoH/DoT
   endpoints
6. `test_dns_fallback_all_fail` — pkarr + DNS both fail → error
   propagated
7. `test_dns_fallback_disabled` — config disabled → no fallback attempt
8. `test_browse_aggregator_with_dns_fallback` — integration test

### 9.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-core-rs --locked
cargo nextest run -p nexus-shell-daemon-core --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### 9.5 Commit cible

```
feat(sprint24): Phase E — DNS-based DHT fallback DoH+DoT via hickory-resolver + domain fronting design doc

DnsFallbackResolver: pkarr quorum timeout → fallback DoH
(RFC 8484, Cloudflare + Google) + DoT (RFC 7858). TXT record
parsing pkarr-DNS compatible. browse_aggregator integration
fallback chain pkarr → DNS.

Dep: hickory-resolver (Apache-2.0, mature DNS resolver Rust).

docs/security/DOMAIN_FRONTING_DESIGN.md: outline design-only
(legal review prerequisite, implementation S25+).

Delta tests: +8 Rust
Cumul: 756 Rust / 312+3+32stale coord / ~1616 total

Scope cuts respected: no domain fronting implementation
(design doc only), no key rotation (deferred S25)
```

---

## 10. Phase F — wrap-up + verification + audit plan S25

### 10.1 Scope

- `sprint24_verification.md` (30+ rows fail-fast)
- `sprint24_audit_plan.md`
- Migration planning active/ → archive/v1.2/
- SPRINT_LOG.md row S24
- CLAUDE.md updates (test counts, etat actuel)
- Memory update nexus_grid_pivot.md tip + compteurs
- HARDENING_ROADMAP §3 S24 post-delivery update

### 10.2 Commit cible

```
chore(sprint24): Phase F — wrap-up + verification + audit plan S25 + migration planning archive/v1.2/
```

---

## 11. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | exit 0 | |
| 2 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | Rust fmt check | `cargo fmt --all --check` | exit 0 | |
| 4 | Rust nextest pass | `cargo nextest run --workspace --locked` | all pass | |
| 5 | Rust doctests pass | `cargo test --workspace --locked --doc` | all pass | |
| 6 | Python ruff format | `uv run ruff format --check packages/` | exit 0 | |
| 7 | Python ruff lint | `uv run ruff check packages/` | exit 0 | |
| 8 | Python SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | 185 pass | |
| 9 | Python coord tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | 312+ pass | |
| 10 | Python gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | |
| 11 | Web TSC check | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 | |
| 12 | Web lint | `cd web && npm run lint` | 0 errors | |
| 13 | Web unit tests | `cd web && npm run test:unit` | 264 pass | |
| 14 | Web build | `cd web && npm run build` | exit 0 | |
| 15 | Web size-limit | `cd web && npm run size` | 7/7 pass | |
| 16 | Playwright e2e | `cd web && npx playwright test` | 43 pass | |
| 17 | Shell daemon build | `cargo build -p nexus-shell-daemon --release` | exit 0 | |
| 18 | Guardrail ABC tests | `uv run pytest tests/test_guardrails.py -v` | 15+ pass | |
| 19 | Hooks tests | `uv run pytest tests/test_hooks.py -v` | 13+ pass | |
| 20 | Re-run tests | `uv run pytest tests/test_rerun.py -v` | 10+ pass | |
| 21 | DNS fallback tests | `cargo nextest run -p nexus-core-rs dns_fallback` | 8+ pass | |
| 22 | Exponent saturation test | `cargo nextest run -p nexus-core-rs exponent_saturation` | pass | |
| 23 | KudosLedger API tests | coord suite | 3+ pass | |
| 24 | DispatchHook trait Rust | `cargo nextest run -p nexus-core-rs hooks` | pass | |
| 25 | Pre-launch versions stable | grep `_VERSION` constants | unchanged | |
| 26 | SPDX scan | `uv run ruff check --select=F401` | clean | |
| 27 | PATTERNS P35+P36 present | `grep §P35 docs/rust/PATTERNS.md` | match | |
| 28 | pynacl dep floor | `grep pynacl pyproject.toml` | >= 1.6.2 | |
| 29 | hickory-resolver dep | `grep hickory Cargo.toml` | present | |
| 30 | Domain fronting design doc | `ls docs/security/DOMAIN_FRONTING_DESIGN.md` | exists | |

---

## 12. Git plan

| # | Commit | Phase |
|---|---|---|
| 0 | `chore(planning): open S24 — kickoff + plan + migrate S23 archive + G1 design review` | pre-A |
| 1 | `feat(sprint24): Phase A — P2 cleanup batch S23 audit + PATTERNS §P35/P36` | A |
| 2 | `feat(sprint24): Phase B — B1 guardrails pipeline declaratif + retrofit 4 primitives` | B |
| 3 | `feat(sprint24): Phase C — A1 TaskDispatchHooks 5 events + HookRunner + dispatcher` | C |
| 4 | `feat(sprint24): Phase D — re-run sampling divergence detection + quarantine` | D |
| 5 | `feat(sprint24): Phase E — DNS-based DHT fallback DoH+DoT + domain fronting design` | E |
| 6 | `chore(sprint24): Phase F — wrap-up + verification + audit plan S25 + migration` | F |

---

## 13. Scope cuts (rappel kickoff §7)

1. Key rotation ceremony → S25
2. C3 handoffs semantic dispatcher → S25
3. GuardrailChain cross-process → S26+
4. P2-D-1 redundancy persistence → S25
5. P2-D-2 quarantine alerting → S25
6. P2-E-1 iroh neighborhood → S25
7. Domain fronting implem → S25+
8. T-NN+2 iframe Rust-wasm → PATTERNS §P34
9. LT-2 Radicle → trigger tag v1.0
10. LT-3/LT-4 → post-v1.0

---

## 14. Risks (rappel kickoff §9)

| ID | Risk | Mitigation |
|---|---|---|
| R1 | B1 retrofit casse 272 coord tests | Contract tests first, incremental retrofit |
| R2 | hickory-resolver conflit deps | Path independant iroh |
| R3 | Re-run overhead | Taux configurable 1%, fire-and-forget |
| R4 | GuardrailChain ordering | Ordre explicite config, tests ordering |
| R5 | PyO3 stale wheel | Phase A doc rebuild, CI note |

---

## 15. Checkpoint de cloture

- [ ] 30/30 fail-fast rows vertes
- [ ] 6 commits feat/chore landed (Phase A-F)
- [ ] verification.md ecrit
- [ ] audit_plan S25 ecrit
- [ ] PATTERNS.md §P35 + §P36 presents
- [ ] SPRINT_LOG.md row S24 ajoutee
- [ ] CLAUDE.md mis a jour
- [ ] Memory nexus_grid_pivot.md tip + compteurs updates
- [ ] Planning migre active/ → archive/v1.2/
