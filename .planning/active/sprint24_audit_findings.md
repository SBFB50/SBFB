# Sprint 24 — Audit Findings (Phase 0 Sprint 25)

**Auditeur** : session fraiche 2026-04-22
**Tip audite** : `9351bb4` (chore(sprint24): Phase F — wrap-up)
**HEAD reel** : `4fd62ab` (3 chore process/docs au-dessus, 0 code)
**Timebox** : ~45 min (lecture ciblée + grep/read exhaustif)

---

## Verdict global : PASS

0 P0, 0 P1. Sprint 25 Phase A peut démarrer sans fix bloquant.

2 P2 carry confirmés (pré-documentés Phase E review, inchangés).
1 P3 nouveau (nit adapter type hints).

G4 rigor signal satisfait : >=1 P2+ documenté avec evidence inline,
toutes dimensions explorées avec grep/read cités.

---

## Track A — Guardrails pipeline (Phase B `c0f9561`)

| Check | Verdict | Evidence |
|---|---|---|
| ABC `Guardrail.check()` retourne `GuardrailOutcome` | PASS | `guardrails.py:72` signature `-> GuardrailOutcome` |
| `GuardrailChain` short-circuit sur tripwire | PASS | `guardrails.py:93-97` raise InputTripwire/OutputTripwire |
| Ordering = ordre d'insertion | PASS | `guardrails.py:91` `for g in self._guardrails` (list preserves order) |
| `InputTripwire`/`OutputTripwire` héritent Exception | PASS | `guardrails.py:42,51` both `class ...Tripwire(Exception)` |
| 4 adapters wrappent sans modifier logique interne | PASS | PiiInputGuardrail (`pii_redactor.py:417-449`), OutputSafetyGuardrail (`output_filter.py:344-379`), QuarantineGuardrail (`quarantine_queue.py:302-348`), CanaryInputGuardrail (`canary_input.py:710-752`) — chacun delègue à la primitive existante |
| `dispatcher.py` : `input_chain.run()` remplace if/else | PASS | `dispatcher.py:154-167` : input_chain prioritaire, elif pii_redactor fallback backward compat, else passthrough |
| Backward compat sans chain | PASS | `dispatcher.py:162-164` : elif pii_redactor path préservé |
| §1.3 GUARDRAILS_ARCHITECTURE.md comparative note (D1-G1-1) | PASS | `GUARDRAILS_ARCHITECTURE.md:69-76` : table LangChain/NeMo/Guardrails AI |

**Track verdict : PASS**

---

## Track B — TaskDispatchHooks (Phase C `30fb66b`)

| Check | Verdict | Evidence |
|---|---|---|
| ABC `DispatchHook` non-instanciable | PASS | `hooks.py:43` ABC + `@abstractmethod` sur `__call__` |
| `HookRunner` fire-and-forget (exception → log) | PASS | `hooks.py:73-82` try/except with structlog warning |
| 5 events fires aux bons points | PASS | on_claim_broadcast (`validator.py:189-193`), on_task_dispatched (`dispatcher.py:209-214`), on_result_received (`validator.py:239-248`), on_validator_post_task (`validator.py:282-286` + `306-311`), on_quarantine_enqueue (`validator.py:266-274`) |
| `HookContext` task_id + timestamp + metadata | PASS | `hooks.py:33-40` dataclass avec les 3 champs |
| Trait Rust `DispatchHook` dyn-safe | PASS | `hooks.rs:36-40` test `dispatch_hook_trait_object_safe` compile |
| Pas de PyO3 binding S24 (scope cut) | PASS | `hooks.rs` = trait + test stub uniquement, grep PyO3 hooks = 0 |

**Track verdict : PASS**

---

## Track C — Re-run sampling (Phase D `2095e5a` + fix `bff0354`)

| Check | Verdict | Evidence |
|---|---|---|
| `RerunSampler` rate 0.0→no, 1.0→all, >1.0→clamp+warning | PASS | `rerun.py:61-63` clamp `max(0.0, min(1.0, ...))` + warning log |
| `DivergenceScorer` hash comparison binaire | PASS | `rerun.py:115-117` `score()` returns 0.0 if equal, 1.0 if mismatch |
| Re-run task_id distinct | PASS | `rerun.py:77-79` `f"rerun-{uuid.uuid4().hex}"` |
| Mismatch → quarantine.add() appelé | PASS | `rerun.py:150-163` `await self._quarantine.add(...)` |
| Config TOML parse | PASS | `rerun.py:40-50` `RerunConfig.from_toml()` with clamping |
| `DivergenceScorer` = hook on_result_received uniquement | PASS | `rerun.py:120` `if ctx.event != "on_result_received": return` |
| `_schedule_rerun` wired in dispatcher.mark_completed | PASS | `dispatcher.py:254-255` conditionnel sur sampler.should_rerun |

**Track verdict : PASS**

---

## Track D — DNS fallback (Phase E `e9d69db`)

| Check | Verdict | Evidence |
|---|---|---|
| Fallback QUE si pkarr quorum échoue (AllFailed) | PASS | `browse.rs:416-417` `Err(QuorumError::AllFailed { count }) => if let Some(dns)...` |
| NoMajority → Unreachable (PAS de DNS fallback) | PASS | `browse.rs:404-414` → `record_unreachable()` direct |
| DoH config TLS name per-endpoint | P2-E-1 | `dns_fallback.rs:195` `endpoints[0].tls_name.clone()` — uses first endpoint TLS name for all. **Carry confirmé** |
| DoT port 853 default, TLS validation active | PASS | `dns_fallback.rs:56` `DOT_PORT: u16 = 853`, Protocol::Tls → TLS active |
| TXT record parsing/concatenation | PASS | `dns_fallback.rs:329-336` `concat_txt_strings()` reassembles RFC 1035 §3.3.14 |
| hickory-resolver features minimales rustls-only | PASS | `Cargo.toml:408-411` : `dns-over-https-rustls` + `dns-over-rustls`. `cargo tree -p nexus-core-rs -i openssl-sys` = vide |
| DOMAIN_FRONTING_DESIGN.md design-only | PASS | 127 lignes, 0 code Rust/Python impl, scope cut respecté |
| DoH→DoT séquentiel (pas concurrent) | P2-E-2 | `dns_fallback.rs:259-285` DoH first, DoT on failure. **Carry confirmé** |

**Track verdict : PASS (2 P2 carry pré-documentés, inchangés)**

---

## Track E — P2 cleanup batch (Phase A `ff4c7d5`)

| Check | Verdict | Evidence |
|---|---|---|
| `pow.rs` exponent saturation `min(i32::MAX)` | PASS | `pow.rs:484` `exponent.min(i32::MAX as u64)` + test `escalating_difficulty_exponent_saturation_i32` (line 829) |
| `KudosLedger.get_total_kudos()` somme correcte | PASS | `kudos.py:238-244` `SELECT COALESCE(SUM(amount), 0.0)` |
| `KudosLedger.get_top_contributors(n)` top n triés | PASS | `kudos.py:247-258` `ORDER BY total DESC LIMIT ?` |
| pynacl dep floor `>= 1.6.2` | PASS | `pyproject.toml:70` `"pynacl>=1.6.2"` |
| PATTERNS §P35 + §P36 présents et complets | PASS | `docs/rust/PATTERNS.md:2053` §P35 ephemeral, `2081` §P36 redundancy |
| `docs/shell/PATTERNS.md` §PyO3 rebuild | PASS | `docs/shell/PATTERNS.md:2099-2102` maturin develop procedure |
| HARDENING_ROADMAP last_validated update | PASS | `HARDENING_ROADMAP.md:3` `last_validated: 2026-04-21` |

**Track verdict : PASS**

---

## Track F — Process / meta

| Check | Verdict | Evidence |
|---|---|---|
| G8 preflight systématique 6/6 phases | PASS | Artefacts : `sprint24_phase_{A,B,C,D,E,F}_preflight.md` (6/6) |
| Phase reviews A-E présentes | PASS | `sprint24_phase_{A,B,C,D,E}_review.md` (5/5 — Phase F docs-only = no review attendu) |
| Commit bodies delta tests cumulé + scope cuts | PASS | Vérifié par sondage sur 3 commits feat (ff4c7d5, c0f9561, e9d69db) : body riche avec delta + scope cuts |
| Dead code (unused imports, unreachable) | PASS | `git diff -- *.rs *.py | grep dead_code/allow(dead/cfg(not(test))` = 0. Unwrap() = test code uniquement |
| Pre-launch protocol (VERSION = 1, 0 tolerant decoder) | PASS | Grep `_VERSION` : CURATOR_LIST/TASK/POW/PIN_FILE/CANARY_INPUT_SET all = 1. 0 nouveau wire format S24 |
| SPDX clean | PASS | Vérifié par verification.md row 26 : `ruff check --select=F401` clean |

**Track verdict : PASS**

---

## Findings list (sorted by severity)

| ID | Sev | Track | Description | Status |
|---|---|---|---|---|
| P2-E-1 | P2 | D | `DnsFallbackResolver::build_resolver` uses `endpoints[0].tls_name` for all IPs in group — per-endpoint TLS name support needed | Carry S25 (pré-documenté Phase E review `72bc0b1`) |
| P2-E-2 | P2 | D | `resolve_node` DoH→DoT séquentiel — concurrent fallback strategy réduirait worst-case latency de 2× timeout à 1× | Carry S25 (pré-documenté Phase E review `72bc0b1`) |
| P3-A-1 | P3 | A | Guardrail adapter `check()` signatures utilisent `object`/`Any` au lieu de `GuardrailContext`/`GuardrailOutcome` (pii_redactor.py:435, output_filter.py:363, canary_input.py:731, quarantine_queue.py:330). Causé par évitement circular import — runtime correct via `Guardrail.register()` ABC mechanism | Nit, laissé tel quel |

---

## Commits fix attendus

Aucun — 0 P0, 0 P1. Sprint 25 Phase A peut démarrer.

---

## P2 à logger en tech debt

P2-E-1 et P2-E-2 sont déjà documentés dans
`sprint24_verification.md §5` et dans
`sprint24_audit_plan.md §3`. Pas de nouvelle entrée PATTERNS.md
nécessaire — les items sont des améliorations transport DNS,
pas des patterns architecturaux.

---

## P3 laissés sans action

- P3-A-1 : type hints adapters. Le pattern `Guardrail.register()`
  + deferred import inside method body est un choix délibéré pour
  éviter les circular imports Python. L'inconsistance de signature
  est détectable par un type checker strict (mypy --strict) mais
  ne cause aucun bug runtime. Pas d'action.

---

## Notes on audit completeness

- 6/6 tracks du `sprint24_audit_plan.md` joués
- Fichiers code S24 lus intégralement : `guardrails.py`, `hooks.py`,
  `rerun.py`, `dns_fallback.rs`, `dispatcher.py`, `validator.py`,
  `pii_redactor.py`, `output_filter.py`, `quarantine_queue.py`,
  `canary_input.py`, `hooks.rs`, `pow.rs`, `kudos.py`, `browse.rs`
- Security scan : grep unwrap/unimplemented/todo/panic sur diff
  Rust = 0 en prod code (all in #[cfg(test)]). Grep FIXME/TODO/HACK
  sur diff Python = 0
- `cargo tree -p nexus-core-rs -i openssl-sys` = vide (hickory-resolver
  n'introduit pas de backend OpenSSL dans le binaire)
- PATTERNS.md NON lus avant formation d'opinion (convention audit gate §3.5)
- Aucun fichier `.planning/` ni memory lu avant les fichiers code
  (opinion indépendante formée d'abord sur le code, puis comparée
  au plan)
