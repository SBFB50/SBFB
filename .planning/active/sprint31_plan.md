# Sprint 31 — Plan d'execution detaille

**Ecrit** : 2026-04-26 (meme session que kickoff).
**Tip master** : `3e1cac0`.

---

## §1 Etat verifie a l'entree

```
Rust nextest    : 864 tests, 864 passed, 0 skipped
Rust clippy     : 0 warnings
Rust fmt        : clean
SDK pytest      : 195 passed (1 flaky intermittent)
Coord pytest    : 394 passed + 36 failed (PyO3 stale) + 6 skipped
Gov pytest      : 46 passed
Vitest          : 269 passed (24 files)
Playwright      : 41 passed + 2 failed (env)
size-limit      : 7/7 pass
en-strings      : clean
```

---

## §2 Decisions Day 0 (gelees)

| ID | Decision | Implications code |
|---|---|---|
| D1 | task_runner reel : OllamaBackend dans executor | task_runner.rs, main.rs, Cargo.toml |
| D2 | §9.5 output filter E2E : post-verify coordinator | verify.py ou result_guardrails.py, tests |
| D3 | Tor transport phase 1 : arti-client 2.0 coordinator outbound | tor_transport.rs, tor_client.py, configs/ |
| D4 | P2 batch S30 + G2 HARDENING update | WebAppFrame delete, docs, HTTP FROST tests |
| D5 | iroh 0.98 scope-cut S32 | Aucun changement iroh |

---

## §3 Research consulte

- **arti-client 2.0** : `TorClient::create_bootstrapped(config)` async
  tokio, retourne `DataStream` (AsyncRead + AsyncWrite). Config TOML
  `TorClientConfig::default()`. Deps : `arti-client = "2.0"`,
  `tor-rtcompat = "2.0"`. Pas de SOCKS daemon requis. Source :
  docs.rs, Tor Project blog.

- **OllamaBackend** : `nexus-worker-core/src/llm/ollama.rs` — HTTP
  POST Ollama, GBNF schema enforcement, defensive JSON validation,
  ~170 LOC production. Endpoint `http://localhost:11434/api/generate`.
  Deps : `ollama-rs = "0.2.6"`, `schemars = "0.8.21"`.

- **OutputFilter** : `output_filter.py:187-341` — `filter()` retourne
  `FilterVerdict`. 2 attack layers : invisible text strip + prompt
  echo EED 0.85. Hot-reload policy 50ms debounce.
  `OutputSafetyGuardrail` adapter :344-388.

- **GuardrailChain** : `guardrails.py` — pipeline declaratif ABC
  `Guardrail.check(ctx, text) → GuardrailOutcome`. Pattern existant
  pour PII, rate-limit. Output filter s'insere comme nouveau
  guardrail dans la chain.

---

## §4 Dependencies inter-phases

```
Phase A (task_runner) → independant
Phase B (output filter) → independant de A
Phase C (Tor transport) → independant de A, B
Phase D (batch + G2) → peut referencer les livrables A/B/C dans docs
Phase E (wrap-up) → depend de A, B, C, D
```

Pas de dependance sequentielle forte entre A, B, C — l'ordre est
choisi par priorite carry (A = 2/3 MANDATORY, B = 2/3 MANDATORY,
C = feature, D = batch).

---

## §5 Phase A — task_runner reel

### §5.1 Scope

Rewrite `task_runner.rs` pour appeler OllamaBackend au lieu de
retourner un resultat vide. Le broker envoie `task.execute` via IPC
JSON-RPC au executor, qui delegue a `task_runner::execute_task()`.
Aujourd'hui cette fonction retourne des zeros. Apres Phase A, elle
appelle Ollama et retourne un vrai resultat LLM.

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-executor/src/task_runner.rs` | Rewrite : build OllamaBackend, call generate(), map response |
| `crates/nexus-executor/src/main.rs` | Ajouter CLI `--ollama-endpoint` arg, init backend on boot |
| `crates/nexus-executor/Cargo.toml` | Ajouter deps : ollama-rs, schemars, tokio features |

### §5.3 Tests plan

1. `test_execute_task_stub_mode` — sans Ollama, mode stub retourne
   resultat vide (backward compat)
2. `test_execute_task_ollama_mock` — mock HTTP response Ollama,
   verifie mapping GenerateResponse → TaskExecuteResult (text,
   prompt_tokens, completion_tokens, duration)
3. `test_execute_task_error_path` — Ollama unreachable → JSON-RPC
   error retournee au broker
4. `test_cli_ollama_endpoint_parsed` — `--ollama-endpoint` arg
   parse correctement

### §5.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-executor --locked
cargo clippy -p nexus-executor --all-targets --locked -- -D warnings
```

Tous les tests existants + 4 nouveaux passent. Clippy clean.

### §5.5 Commit cible

```
feat(sprint31): Sprint 31 Phase A — task_runner reel executor wire LlmBackend Ollama

Scope : task_runner.rs rewrite — appelle OllamaBackend.generate() au lieu
de retourner un resultat vide. CLI --ollama-endpoint. Carry P2-REVIEW-C-1
(2/3) resolu.

Fichiers touches :
- crates/nexus-executor/src/task_runner.rs (rewrite ~80 LOC)
- crates/nexus-executor/src/main.rs (CLI arg + init ~40 LOC)
- crates/nexus-executor/Cargo.toml (deps ollama-rs, schemars)

Tests :
- +4 Rust (stub_mode, ollama_mock, error_path, cli_parse)
- Delta cumule S31 : +4 Rust

Scope cuts respectes : pas de llama.cpp, pas de dual backend.
```

---

## §6 Phase B — §9.5 output filter E2E

### §6.1 Scope

Injecter `OutputFilter.filter()` dans le result dispatch path du
coordinator. Post-verification signature 3-layer, pre-mark_completed.
Results invalides marques `rejected`, 0 kudos credit. Nettoyage
WebAppFrame.tsx orphelin (meme commit car concurrent web/ area).

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `packages/nexus-coordinator/src/nexus_coordinator/result_guardrails.py` | NEW : wire OutputSafetyGuardrail dans result dispatch path |
| `packages/nexus-coordinator/src/nexus_coordinator/api/verify.py` | Appeler result_guardrails post-verify |
| `packages/nexus-coordinator/tests/test_result_guardrails.py` | NEW : tests output filter E2E |
| `web/src/components/app/WebAppFrame.tsx` | DELETE (orphelin S11) |
| `web/src/components/app/WebAppFrame.test.tsx` | DELETE (test du composant orphelin) |

### §6.3 Tests plan

1. `test_output_filter_invisible_text_rejected` — resultat avec
   invisible chars → tripwire → rejected
2. `test_output_filter_prompt_echo_rejected` — resultat qui repete
   le prompt → tripwire → rejected
3. `test_output_filter_clean_passthrough` — resultat propre → pass
4. `test_output_filter_context_threading` — verify system_prompt +
   user_prompt passes au filter
5. `test_output_filter_policy_disabled_passthrough` — policy disabled
   → all pass (no false positive)

Frontend : Vitest count -2 (WebAppFrame.test.tsx supprime).

### §6.4 Critere d'acceptation

```bash
uv run pytest packages/nexus-coordinator/tests/test_result_guardrails.py -q
uv run pytest packages/nexus-coordinator/tests/ -q
cd web && npm run test:unit
```

5 nouveaux tests coord passent. Vitest inchange (net -2 WebAppFrame
tests + 0 nouveaux = regression count).

Note : la suppression de WebAppFrame.test.tsx reduit le count Vitest.
C'est un nettoyage intentionnel d'un test orphelin, pas une regression.

### §6.5 Commit cible

```
feat(sprint31): Sprint 31 Phase B — output filter E2E wire + WebAppFrame cleanup

Scope : OutputFilter.filter() injecte dans result dispatch post-verify.
Results invalides (invisible text, prompt echo) marques rejected, 0 kudos.
Pattern PII guardrail. Carry P2-REVIEW-B-2 (2/3) resolu.
WebAppFrame.tsx + test supprimes (orphelin S11, allow-same-origin non-conforme).
P3-AUDIT-1 (1/3) resolu.

Fichiers touches :
- packages/nexus-coordinator/src/nexus_coordinator/result_guardrails.py (NEW ~100 LOC)
- packages/nexus-coordinator/src/nexus_coordinator/api/verify.py (edit ~20 LOC)
- packages/nexus-coordinator/tests/test_result_guardrails.py (NEW ~120 LOC)
- web/src/components/app/WebAppFrame.tsx (DELETE)
- web/src/components/app/WebAppFrame.test.tsx (DELETE)

Tests :
- +5 coord (invisible_text, echo, clean, context, disabled)
- -2 Vitest (WebAppFrame orphelin supprime)
- Delta cumule S31 : +4 Rust, +5 coord, -2 Vitest
```

---

## §7 Phase C — Tor transport phase 1

### §7.1 Scope

Premiere integration Tor via arti-client 2.0. Scope phase 1 :
anonymiser les connexions HTTP sortantes du coordinator (task dispatch
vers workers, gossip publish HTTP, fetch operations). Configuration
opt-in disabled par defaut. Fallback direct si Tor bootstrap echoue.

### §7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-core-rs/src/tor_transport.rs` | NEW : TorTransport wrapper (create_bootstrapped, connect, health_check) |
| `crates/nexus-core-rs/src/lib.rs` | Export tor_transport module |
| `crates/nexus-core-rs/Cargo.toml` | Deps arti-client, tor-rtcompat (feature-gated `tor`) |
| `crates/nexus-core-py/src/lib.rs` | PyO3 binding tor_connect(host, port) |
| `packages/nexus-coordinator/src/nexus_coordinator/tor_client.py` | NEW : TorClientWrapper async (connect, is_available, health) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py` | Wire tor_client pour outbound HTTP si enabled |
| `configs/tor.toml.sample` | NEW : [tor] enabled = false, bootstrap_timeout_s = 30 |
| `docs/security/HARDENING_ROADMAP.md` | Update §3 S31 entry (delivre Phase C) |

### §7.3 Tests plan

1. `test_tor_transport_create_default_config` — TorTransport::new
   avec config default compile et init sans panic
2. `test_tor_transport_connect_mock` — mock connection retourne
   DataStream (via trait abstraction)
3. `test_tor_transport_fallback_on_failure` — Tor bootstrap echoue →
   fallback direct sans erreur fatale
4. `test_tor_config_disabled_noop` — config `enabled = false` →
   aucun bootstrap Tor, toutes connexions directes
5. `test_tor_config_parse_toml` — `tor.toml.sample` parse
   correctement
6. `test_tor_client_python_available` — PyO3 binding accessible
   depuis Python, retourne booleen
7. `test_tor_client_coordinator_outbound` — coordinator wire :
   request HTTP via tor_client quand enabled

### §7.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-core-rs --locked -E 'test(tor)'
cargo nextest run -p nexus-core-py --locked -E 'test(tor)'
cargo clippy --workspace --all-targets --locked -- -D warnings
uv run pytest packages/nexus-coordinator/tests/test_tor_client.py -q
```

7 nouveaux tests passent. Clippy clean. Feature gate `tor` compile.

### §7.5 Commit cible

```
feat(sprint31): Sprint 31 Phase C — Tor transport phase 1 arti-client 2.0 coordinator outbound

Scope : TorTransport wrapper (arti-client 2.0, tokio async) dans nexus-core-rs.
PyO3 binding tor_connect(). Coordinator wire outbound HTTP via TorClientWrapper
quand config [tor] enabled = true. Fallback direct si Tor indisponible.
HARDENING_ROADMAP §3 S31 prescrit "Tor transport phase 1" — livre scope
coordinator outbound (pas iroh relay, cf. D3 kickoff).

Fichiers touches :
- crates/nexus-core-rs/src/tor_transport.rs (NEW ~200 LOC)
- crates/nexus-core-py/src/lib.rs (binding ~30 LOC)
- packages/nexus-coordinator/src/nexus_coordinator/tor_client.py (NEW ~80 LOC)
- packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py (edit ~20 LOC)
- crates/nexus-core-rs/Cargo.toml (arti-client, tor-rtcompat feature tor)
- configs/tor.toml.sample (NEW)

Tests :
- +5 Rust (create, connect_mock, fallback, config_disabled, config_parse)
- +2 coord (python_available, coordinator_outbound)
- Delta cumule S31 : +9 Rust, +7 coord, -2 Vitest
```

---

## §8 Phase D — P2 batch S30 + G2 HARDENING update

### §8.1 Scope

Batch des items P2/P3 a 1/3 + G2 HARDENING_ROADMAP update S31.

### §8.2 Fichiers touches

| Fichier | Role |
|---|---|
| `docs/security/VALIDATED_BLUEPRINT.md` | Refresh Couche 6 : Kirchenbauer → SynthID, spaCy → GLiNER |
| `docs/security/SPLIT_INFERENCE_DESIGN.md` | Ajouter confidence_score field mention |
| `docs/security/HARDENING_ROADMAP.md` | Update last_validated S31, S31 entry (Tor delivered, iroh deferred) |
| `crates/nexus-shell-daemon/tests/` | HTTP integration tests FROST endpoints (4 tests) |

### §8.3 Tests plan

1. `test_frost_trusted_dealer_http` — POST /api/canary/frost/
   trusted-dealer retourne 200 avec shares JSON
2. `test_frost_round1_http` — POST /api/canary/frost/round1
   retourne commitments
3. `test_frost_round2_http` — POST /api/canary/frost/round2
   retourne signature share
4. `test_frost_aggregate_http` — POST /api/canary/frost/aggregate
   retourne signature aggregee

### §8.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon --locked -E 'test(frost_http)'
cargo clippy --workspace --all-targets --locked -- -D warnings
```

4 nouveaux tests HTTP FROST passent. Docs updates coherentes.

### §8.5 Commit cible

```
feat(sprint31): Sprint 31 Phase D — P2 batch S30 carries + G2 HARDENING update

Scope : P2 batch S30 (VALIDATED_BLUEPRINT Couche 6 refresh, SPLIT_INFERENCE
confidence_score, HTTP FROST integration tests) + HARDENING_ROADMAP G2 update
(last_validated S31, S31 entry : Tor delivered, iroh deferred S32, carries
resolved).

Fichiers touches :
- docs/security/VALIDATED_BLUEPRINT.md (edit ~10 LOC)
- docs/security/SPLIT_INFERENCE_DESIGN.md (edit ~5 LOC)
- docs/security/HARDENING_ROADMAP.md (update S31 entry)
- crates/nexus-shell-daemon/tests/ (4 HTTP FROST tests ~80 LOC)

Tests :
- +4 Rust (frost_http trusted_dealer, round1, round2, aggregate)
- Delta cumule S31 : +13 Rust, +7 coord, -2 Vitest

Carries resolus ce commit :
- P2-REVIEW-D-1-S30 VALIDATED_BLUEPRINT stale (1/3 → closed)
- P3-REVIEW-D-1-S30 confidence_score (1/3 → closed)
- P2-REVIEW-C-1-S30 HTTP FROST tests (1/3 → closed)
```

---

## §9 Phase E — Wrap-up

### §9.1 Scope

Verification + carry summary + audit plan S32 + SPRINT_LOG + CLAUDE.md
+ memory + migration.

### §9.2 Fichiers touches

| Fichier | Role |
|---|---|
| `.planning/active/sprint31_verification.md` | NEW : fail-fast 30+ rows |
| `.planning/active/sprint31_carry_summary.md` | NEW : carries S32 |
| `.planning/active/sprint32_audit_plan.md` | NEW : audit plan S32 Phase 0 |
| `docs/claude/SPRINT_LOG.md` | Row S31 |
| `CLAUDE.md` | §Etat actuel update |

### §9.3 Commit cible

```
chore(sprint31): Phase E — wrap-up + verification + audit plan S32 + migration
```

---

## §10 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | 0 errors | |
| 2 | Rust nextest all pass | `cargo nextest run --workspace --locked` | 864+ passed, 0 failed | |
| 3 | Rust doctests pass | `cargo test --workspace --locked --doc` | 0 failed | |
| 4 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 5 | Rust fmt clean | `cargo fmt --all --check` | clean | |
| 6 | Release build daemon | `cargo build -p nexus-shell-daemon --release` | Finished | |
| 7 | Python ruff format | `uv run ruff format --check packages/` | clean | |
| 8 | Python ruff check | `uv run ruff check packages/` | clean | |
| 9 | SDK 195/195 pass | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 passed | |
| 10 | Coord pass + stale fail | `uv run pytest packages/nexus-coordinator/tests/ -q` | 394+ passed + 36 fail (stale) | |
| 11 | Gov 46/46 pass | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 passed | |
| 12 | Frontend lint clean | `cd web && npm run lint` | 0 errors | |
| 13 | Frontend tsc clean | `npx tsc --noEmit -p tsconfig.app.json` | clean | |
| 14 | Vitest pass | `npm run test:unit` | 267+ passed | |
| 15 | Frontend build OK | `npm run build` | built | |
| 16 | size-limit pass | `npm run size` | 7/7 under budget | |
| 17 | Playwright pass | `npx playwright test` | 41+ pass + 2 env fail | |
| 18 | en-strings clean | `bash scripts/scan-en-strings.sh` | clean | |
| 19 | task_runner Ollama test | `cargo nextest -p nexus-executor -E 'test(ollama)'` | 2+ passed | |
| 20 | task_runner error test | `cargo nextest -p nexus-executor -E 'test(error)'` | 1+ passed | |
| 21 | output filter E2E tests | `uv run pytest tests/test_result_guardrails.py -q` | 5 passed | |
| 22 | Tor transport tests | `cargo nextest -p nexus-core-rs -E 'test(tor)'` | 5+ passed | |
| 23 | Tor client Python test | `uv run pytest tests/test_tor_client.py -q` | 2 passed | |
| 24 | HTTP FROST tests | `cargo nextest -p nexus-shell-daemon -E 'test(frost_http)'` | 4 passed | |
| 25 | VALIDATED_BLUEPRINT refresh | `grep -c SynthID docs/security/VALIDATED_BLUEPRINT.md` | >= 1 | |
| 26 | HARDENING last_validated S31 | `grep 'last_validated.*S31' docs/security/HARDENING_ROADMAP.md` | found | |
| 27 | WebAppFrame deleted | `test ! -f web/src/components/app/WebAppFrame.tsx` | true | |
| 28 | tor.toml.sample exists | `test -f configs/tor.toml.sample` | true | |
| 29 | FORMAT_VERSION all v1 | `grep _VERSION crates/nexus-core-rs/src/ \| grep -v "= 1"` | 0 matches | |
| 30 | Planning docs complets | kickoff + plan + design_review + preflights + reviews | present | |
| 31 | iroh pin inchange | `grep 'iroh = "0.97"' Cargo.toml` | found | |
| 32 | Tor feature gate | `grep 'feature.*tor' crates/nexus-core-rs/Cargo.toml` | found | |

**Score cible** : **32/32 rows vertes**.

---

## §11 Git plan

```
1. chore(planning): sprint 31 kickoff + plan + design review
2. chore(planning): sprint 31 Phase A — G8 preflight verdict [TBD]
3. feat(sprint31): Sprint 31 Phase A — task_runner reel executor wire LlmBackend Ollama
4. chore(planning): sprint 31 Phase B — G8 preflight verdict [TBD]
5. feat(sprint31): Sprint 31 Phase B — output filter E2E wire + WebAppFrame cleanup
6. chore(planning): sprint 31 Phase C — G8 preflight verdict [TBD]
7. feat(sprint31): Sprint 31 Phase C — Tor transport phase 1 arti-client 2.0 coordinator outbound
8. chore(planning): sprint 31 Phase D — G8 preflight verdict [TBD]
9. feat(sprint31): Sprint 31 Phase D — P2 batch S30 carries + G2 HARDENING update
10. chore(sprint31): Phase E — wrap-up + verification + audit plan S32 + migration
```

---

## §12 Scope cuts (copie kickoff §7)

1. iroh 0.98 upgrade → S32
2. iroh relay over Tor → S32+
3. Nym mixnet → S33+
4. TEE H100 → post-v1.0
5. DKG distribue FROST → post-v1.0
6. Recrutement mainteneurs → post-v1.0
7. Playwright COEP test → S34
8. Onion service hosting → post phase 1
9. Full process isolation blob-serve → LT
10. openai-agents upgrade → pas de dep
11. llama.cpp executor → S32+
12. Output filter client-side → S34

---

## §13 Risks (copie kickoff §9)

| ID | Risque | Mitigation |
|---|---|---|
| R1 | arti deps build time | Feature-gate `tor` |
| R2 | arti bootstrap lent 10-30s | Async non-bloquant, fallback 30s |
| R3 | Ollama pas running en test | Stub-mode + OLLAMA_ENDPOINT env |
| R4 | OutputFilter false positive | Config threshold tunable |
| R5 | iroh scope-cut conteste audit | Justification documentee |
| R6 | PW COEP atteint 2/3 | S32 integre ou exemption |
| R7 | tokio runtime conflit arti/iroh | Shared runtime 1.x, tester Phase C |

---

## §14 Checkpoint de cloture

- [ ] 32/32 fail-fast rows vertes
- [ ] 4 commits feat (A, B, C, D) + planning chore
- [ ] 4 preflights G8 (A, B, C, D)
- [ ] 4 reviews phase (A, B, C, D)
- [ ] sprint31_verification.md ecrit
- [ ] sprint31_carry_summary.md ecrit
- [ ] sprint32_audit_plan.md ecrit
- [ ] SPRINT_LOG.md row S31 ajoute
- [ ] CLAUDE.md §Etat actuel mis a jour
- [ ] Memory mise a jour (nexus_grid_pivot.md + MEMORY.md)
- [ ] active/ migre vers archive/v1.2/
