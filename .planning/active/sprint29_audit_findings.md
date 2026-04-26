# Sprint 29 — Audit findings (S30 Phase 0)

**Auditeur** : session fraîche 2026-04-26 (Opus 4.6 1M)
**Tip audité** : `41363ab` (chore(sprint29): Phase E — wrap-up)
**Timebox observé** : ~25 min (4 agents parallèles Explore)
**Méthode** : 4 agents indépendants (Track A IPC, Track B THREAT_MODEL,
Track C TraceProvider, Track D+E+F+G+Meta) + vérifications manuelles
croisées sur findings clés.

---

## Verdict global : PASS

0 P0, 0 P1, 6 P2, 3 P3.
G4 calibration satisfaite (6 P2 documentés dont 1 nouveau).
S30 Phase A peut démarrer directement.

---

## Track A — Process isolation broker/executor (Phase C `6a23ebf`)

### IPC-1 JSON-RPC 2.0 spec compliance — PASS

- `JsonRpcRequest` : `jsonrpc: "2.0"` (constante `JSONRPC_VERSION`
  `ipc.rs:10`), `id: u64` présent, `method` + `params` présents.
- `JsonRpcResponse` : `result`/`error` mutuellement exclusifs via
  `skip_serializing_if` (`ipc.rs:24-31`), `id` présent.
- `JsonRpcNotification` : pas de champ `id` — test
  `test_json_rpc_notification_roundtrip` (`ipc.rs:252`) vérifie
  `!json.contains("\"id\"")`.
- `_traceparent` optionnel via `#[serde(rename)]` (`ipc.rs:18-20`).

### IPC-2 task_token isolation — PASS

- HMAC-SHA256 : `ipc_broker.rs:101-111`, input = `master_token ||
  task_id || timestamp.to_le_bytes()`. Crates `hmac` + `sha2`.
- Test `test_task_token_ephemeral` (`ipc_broker.rs:325-339`) : tokens
  différents pour task_id différents, timestamps différents ;
  déterministe pour mêmes inputs ; output 64 hex = 32 bytes.
- Executor (`main.rs`) : zéro import de `master_token`, keypair
  Ed25519 ni signing key. Reçoit uniquement `task_token` dérivé.

### IPC-3 crash backoff — PASS

- Constantes `MIN_BACKOFF = 1s`, `MAX_BACKOFF = 30s`
  (`ipc_broker.rs:117-118`).
- `record_crash()` : `current_delay * 2` capped `.min(MAX_BACKOFF)`
  (`ipc_broker.rs:134-137`).
- Test `test_executor_crash_backoff` (`ipc_broker.rs:342-369`) :
  mesure 1→2→4→8→16→30→30s.
- `SecurityEvent::ExecutorCrash` émis à chaque crash
  (`ipc_broker.rs:263-270`), confirmé par test
  `test_executor_crash_security_event` (`ipc_broker.rs:526-549`).

### IPC-4 shutdown graceful — PASS

- Executor : reçoit `executor.shutdown`, envoie
  `{"ack": true}` (`main.rs:111-130`), retourne `Ok(())` = exit 0
  (`main.rs:55-57`).
- Test `test_executor_shutdown_graceful` (`ipc_broker.rs:486-523`)
  vérifie ack + pas de crash post-shutdown.
- Broker distingue shutdown intentionnel (ack reçu) vs crash
  (connexion perdue).

### IPC-5 task_runner stub — PASS (P2 carry confirmé)

- `task_runner.rs:5-14` : 12 LOC, retourne `output: String::new()`,
  `output_token_ids: Vec::new()`, `duration_ms: 0`. Pas d'accès aux
  champs `prompt`, `watermark_config`, `grammar`. Aucun system call,
  subprocess, réseau. Defense-in-depth OK.
- P2 carry S30 documenté dans `sprint29_phase_C_review.md:60`.

### IPC-6 traceparent propagation — PASS

- Champ `_traceparent: Option<String>` dans `JsonRpcRequest`
  (`ipc.rs:18-20`), sérialisé/désérialisé via `#[serde(rename)]`.
- Broker passe `traceparent` dans `send_task()`
  (`ipc_broker.rs:204-220`).
- Test `test_traceparent_propagation` (`ipc.rs:256-270`) : vérifie
  présence quand fourni, absence quand omis.
- Intégration test `test_task_execute_roundtrip`
  (`ipc_broker.rs:402-467`) : end-to-end broker→executor.

---

## Track B — THREAT_MODEL §9 + docs sécurité (Phase B `1f79c52`)

### TM-1 §9 cohérence — PASS

- 6 sous-sections §9.1-§9.6 présentes (`THREAT_MODEL.md:384-465`).
- §9.1 consent GPU : schéma consent.json vérifié dans Rust
  (`consent.rs:160-176`), Python (`consent.py:102-115`), TypeScript
  (`consent.ts:37-44`). Trois côtés cohérents.
- §9.2 loopback tiers : cross-ref correct vers
  `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`.
- §9.5 guardrails disabled : documente les risques quand guardrails
  OFF **sans claim faux d'implémentation**. GUARDRAILS_ARCHITECTURE.md
  confirme status `implemented (S24 Phase B)`.

### TM-2 SECURITY.md — PASS

- Email contact : `security@sbfb.network` (ligne 10).
- Timeline explicite : 48h ack, 7j assessment, 14j fix critique,
  30j fix low, 90j disclosure publique (lignes 23-29).
- Scope inclut Rust (`crypto.rs`, `iroh stack`, `nexus-worker*`,
  `nexus-shell-daemon*`) et Python (coordinator loopback FastAPI).

### TM-3 security.txt RFC 9116 — PASS

- `.well-known/security.txt` : Contact ✓, Preferred-Languages `fr, en`
  ✓, Expires `2027-04-26` (= 1 an exact) ✓, Policy ✓, Canonical ✓.
- Test `test_security_txt_valid` (`test_consent.py:162-171`).

### TM-4 BUILDING.md — PASS

- 124 LOC. Rust 1.94+ / Python 3.13+ / Node 22+ / uv 0.7+ / Ollama
  optionnel. Commandes exhaustives (cargo build/test/fmt/clippy,
  uv sync/ruff/pytest, npm install/lint/tsc/test/build). 9 fichiers
  audit-priority listés.

### TM-5 consent annotations pipeline — PASS

- Python : `_LEVEL_THREAT_NOTES` dict + `_LEVEL_RESIDUAL_THREATS`
  dict (`consent.py:87-99`). `_populate_threat_fields()` appelé dans
  GET et POST (`consent.py:191, 198`).
- TypeScript : `level_threat_note` + `residual_threats_acknowledged`
  dans `ConsentConfigSchema` (`consent.ts:42-43`).
- React : tooltip `<Tooltip>` par niveau dans `GpuConsentDialog.tsx`
  (`lignes 209-220`), `data-testid` pour chaque lvl 1-4.
- Tests : `test_consent_residual_threats_field`
  (`test_consent.py:134-159`) + Vitest tooltip
  (`GpuConsentDialog.test.tsx:224-230`).

---

## Track C — TraceProvider opentelemetry 0.31 (Phase D `5787168`)

### TRACE-1 processor pipeline — PASS

- `add_trace_processor` = `lock.push(processor)` (append)
  (`lib.rs:55-58`).
- `set_trace_processors` = shutdown old + replace vec
  (`lib.rs:60-66`).
- Tests `test_multi_processor_pipeline` (`lib.rs:114-126`) +
  `test_set_trace_processors_replaces` (`lib.rs:128-141`) OK.

### TRACE-2 signed canary — PASS

- Ed25519 via `ed25519-dalek 2.1` (`signed.rs:15`).
- Domain separation `DOMAIN_TRACE_EVENT_V1 = b"nexus-trace-event-v1"`
  (`lib.rs:29`).
- `domain_message()` : `domain || 0x00 || JSON` (`signed.rs:59-66`).
- Tests `signed_processor_roundtrip` (`signed.rs:140-152`) +
  `signed_processor_tamper_detect` (`signed.rs:155-170`) OK.

### TRACE-3 OTel bridge — PASS (P3 doc bug)

- `OtelProcessor` bridge vers `SdkTracerProvider` (0.31 API)
  (`otel.rs:8-14`).
- Workspace deps : `opentelemetry = "0.31"`, `opentelemetry_sdk =
  "0.31"` (`Cargo.toml:85-86`).
- **P3-AUDIT-1** : `lib.rs:8` docstring dit "OpenTelemetry 0.28+"
  au lieu de "0.31". Pas d'impact fonctionnel.

### TRACE-4 W3C Trace Context — PASS

- Format `00-{32hex}-{16hex}-{2hex}` (`propagation.rs:4, 40-42`).
- Validation stricte : 4 parts, version "00", longueurs vérifiées,
  hex digits (`propagation.rs:44-66`).
- Tests `traceparent_roundtrip` (`propagation.rs:139-144`) +
  `invalid_traceparent_rejected` (`propagation.rs:155-163`,
  4 cas invalides) OK.

### TRACE-5 executor trace log path — PASS (P2 carry confirmé)

- Chemin relatif `"traces/executor.jsonl"` (`main.rs:34`), pas de
  `ShellDaemonPaths`. Daemon utilise chemin absolu
  (`runtime.rs:454`). Asymétrie intentionnelle (isolation processus).
- Carry S30 documenté (`sprint29_phase_D_review.md`).

---

## Track D — P2 batch S28 (Phase A `b1c4148`)

### P2-1 generate_blocking params — PASS

- Commentaire justificatif lignes 258-261 dans `llama_cpp.rs`.
- `#[allow(clippy::too_many_arguments)]` ligne 262.

### P2-2 sampler chain rebuild — PASS

- Commentaire inline lignes 344-348 dans `llama_cpp.rs` : coût ~µs
  vs inference ~5-50ms/token, acceptable < 100 req/s.

### P2-3 init_platform_emitter — PASS

- 2 tests dans `nexus-events-core/src/lib.rs:628-646` :
  `test_init_platform_emitter_does_not_panic` (singleton init) +
  `test_init_platform_emitter_selects_tracing_on_windows`
  (cfg-gated Windows).

### P2-4 cold-start benchmark — PASS

- Commit body `b1c4148` : RTX 5080 + Ollama gemma-4-26B Q4_K_M
  (3.3x plus gros que cible 7B). Warm cache **497 ms** << 5000 ms.
  Pool mode Phase C validé.

---

## Track E — G1 Design Review Board — PASS

- `sprint29_design_review.md` présent dans `.planning/archive/v1.2/`.
- Scoring : D1 ✅, D2 ⚠️, D3 ❌, D4 ⚠️, D5 ✅.
- Rigor signal G4 adéquat (1 ❌ + 2 ⚠️ = pas rubber-stamp).

---

## Track F — Phase review completeness — PASS

- 4/4 preflight : A ✓, B ✓, C ✓, D ✓.
- 3/3 reviews : B ✓, C ✓, D ✓.
- Phase A sans review : justifié (P2 batch + benchmark, pas de
  feature à risque).

---

## Track G — HARDENING drift — PASS

| Item prescrit S29 | Livré | Justification |
|---|---|---|
| External audit engagement | Non | scope-cut §7.13 : S29 prépare, S30 exécute |
| Remediation findings | Non | scope-cut §7.14 : post-findings |
| Public disclosure + security.txt | Oui | Phase B `1f79c52` |
| agents_sudo A2 TraceProvider | Oui | Phase D `5787168` (PLAN-ADAPT 0.27→0.31) |
| agents_sudo B4 THREAT_MODEL §9 | Oui | Phase B `1f79c52` |
| Process isolation broker/executor | Oui | Phase C `6a23ebf` |

Items non livrés justifiés par scope-cuts dans le kickoff §7.

---

## Meta-track — G8 traceability — PASS

| Phase | Preflight | Review | Verdict G8 | Commit |
|---|---|---|---|---|
| A | ✓ | (N/A) | EXECUTE | `b1c4148` |
| B | ✓ | ✓ | EXECUTE | `1f79c52` |
| C | ✓ | ✓ | EXECUTE | `6a23ebf` |
| D | ✓ | ✓ | EXECUTE | `5787168` |

4/4 EXECUTE, 0 DESIGN-CONFLICT, 0 pivot_proposal. Cohérent.

---

## Meta-track — Sprint pair S30 phase dette — PASS

- S30 est pair → phase dette obligatoire (§6.2.1 Règle 1).
- **P2-B-1-S28 CI Linux/macOS writers** : 3/3 reports → **MANDATORY**
  S30 per §6.2.1 Règle 2. Documenté dans `sprint29_verification.md`
  lignes 189 et 215-216 + `sprint29_carry_summary.md` ligne 14.

---

## Pre-launch protocol check — PASS

- `CURATOR_LIST_FORMAT_VERSION = 1` (`curator.rs:61`)
- `KEY_ROTATION_FORMAT_VERSION = 1` (`key_rotation.rs:32`)
- `POW_FORMAT_VERSION = 1` (`pow.rs:85`)
- `TASK_FORMAT_VERSION = 1` (`task.rs:61`)
- `PIN_FILE_FORMAT_VERSION = 1` (`tls_pinning.rs:102`)
- `DOMAIN_TRACE_EVENT_V1` : événements locaux, pas wire P2P.
- `DOMAIN_IPC_*` : internes broker↔executor, pas P2P gossip.
- Aucun tolerant decoder multi-version détecté.
- Aucun test "legacy decode" zombie détecté.

---

## Findings sorted by severity

### P2

| ID | Finding | Source | Track | Status |
|---|---|---|---|---|
| P2-B-1-S28 | CI Linux/macOS writers 3/3 **MANDATORY** S30 | S28 Phase B review | G/Meta | carry S30 |
| P2-REVIEW-B-1 | consent.py mutation pattern | Phase B review | B | carry S30 1/3 |
| P2-REVIEW-B-2 | §9.5 output filter not wired e2e | Phase B review | B | carry S30 1/3 |
| P2-REVIEW-C-1 | task_runner.rs stub | Phase C review | A | carry S30 1/3 |
| P2-REVIEW-D-1 | executor trace log path relatif | Phase D review | C | carry S30 1/3 |
| **P2-AUDIT-1** | **HARDENING_ROADMAP.md:734 dit "opentelemetry 0.27" — code livré = 0.31 (PLAN-ADAPT). Doc stale.** | **Cet audit** | **G** | **fix S30 ou doc pass** |

### P3

| ID | Finding | Source | Track |
|---|---|---|---|
| P3-AUDIT-1 | `nexus-trace-core/src/lib.rs:8` docstring "0.28+" → devrait dire "0.31" | Cet audit | C |
| P3-REVIEW-C-1 | Types IPC dupliqués design (cosmétique) | Phase C review | A |
| P3-REVIEW-D-1 | Plan file path inexact (cosmétique) | Phase D review | C |

---

## P2 à logger en tech debt

- **P2-AUDIT-1** : HARDENING_ROADMAP.md:734 "opentelemetry 0.27" →
  rafraîchir vers "0.31" au prochain sprint qui touche ce doc. Pas de
  code change, juste un sed.

---

## P3 laissés sans action

- P3-AUDIT-1 : docstring stale, fix au prochain touch de
  `nexus-trace-core`.
- P3-REVIEW-C-1 / P3-REVIEW-D-1 : cosmétiques, pas de code change.

---

## Notes on audit completeness

- 4 agents Explore indépendants ont couvert les 7 tracks + 2
  meta-tracks + pre-launch protocol.
- Vérifications manuelles croisées sur : HARDENING_ROADMAP 0.27 vs
  0.31 (confirmé stale), `_VERSION` constants (tous = 1), CI
  Linux/macOS carry count (3/3 MANDATORY confirmé), tolerant decoders
  (aucun).
- Pas de lecture de `sprint29_verification.md` (self-report biaisé)
  avant formation d'opinion.
- Pas de re-débat D1..D5 gelées ni scope cuts §7.

---

## Verdict final

**PASS** — Sprint 29 audité, 0 P0/P1.
S30 Phase A peut démarrer. Le kickoff S30 doit :
1. Réserver une phase dette obligatoire (sprint pair, §6.2.1 Règle 1)
2. Absorber P2-B-1-S28 CI Linux/macOS writers (3/3 MANDATORY)
3. Documenter les 5 P2 carry dans §Items carry/dette
4. Fix P2-AUDIT-1 (HARDENING_ROADMAP refresh 0.27→0.31)
