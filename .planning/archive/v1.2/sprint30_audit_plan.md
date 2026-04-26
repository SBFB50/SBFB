# Sprint 29 — Audit plan (pour S30 Phase 0)

**Date** : 2026-04-26
**Tip sortie S29** : sera le commit Phase E (post-migration)
**Auditeur** : session fraiche S30 Phase 0 (pas la meme session)

---

## 1. Mode d'emploi pour la session fraiche

### Ordre de lecture impose

1. `.planning/archive/v1.2/sprint29_kickoff.md` — D1..D5 gelees
2. `.planning/archive/v1.2/sprint29_plan.md` — fail-fast checklist + phases A-E
3. Ce document (audit_plan) — tracks + methodes
4. Le code (via grep/read/explore)

### Fichiers a NE PAS lire avant d'avoir forme une opinion

- `.planning/archive/v1.2/sprint29_verification.md` — self-report biaise
- Memory files — ne pas importer le contexte du sprint

### Timebox suggere

2-3h. Priorite : Track A (code neuf) > Track B (docs) > Track C (integration) > Meta-tracks.

### Delivrable final

`sprint29_audit_findings.md` dans `.planning/active/` avec verdict PASS / CONDITIONAL PASS / FAIL.

---

## 2. Dimensions d'audit

### Track A — Process isolation broker/executor (Phase C `6a23ebf`)

Le coeur technique du sprint. Nouveau crate `nexus-executor` + module `ipc_broker.rs`.

1. **IPC-1 JSON-RPC spec compliance** : verifier que les messages JSON-RPC
   respectent la spec 2.0 (id present pour requests, absent pour
   notifications, `jsonrpc: "2.0"` present). Grep `JsonRpcRequest` /
   `JsonRpcResponse` / `JsonRpcNotification` dans executor/ipc.rs et
   ipc_broker.rs. Comparer avec RFC 7616.

2. **IPC-2 task_token isolation** : verifier que le token per-task est
   derive via HMAC-SHA256 et qu'il n'est PAS reutilisable entre deux tasks.
   Test `test_task_token_ephemeral` doit confirmer unicite. Verifier que
   l'executor n'a PAS acces au master token ni a la keypair Ed25519.

3. **IPC-3 crash backoff** : verifier que le backoff exponentiel cap a 30s
   est implemente. Test `test_executor_crash_backoff` doit mesurer les
   delais. Verifier que SecurityEvent::ExecutorCrash est emis.

4. **IPC-4 shutdown graceful** : verifier que `executor.shutdown` method
   produit un ack + exit propre (exit code 0). Verifier que le broker
   ne re-spawn pas apres un shutdown intentionnel vs un crash.

5. **IPC-5 task_runner stub** : verifier que `task_runner.rs` est un stub
   (retourne resultat vide). C'est un **P2 carry** documente — le finding
   est attendu. L'auditeur doit confirmer que le stub ne peut PAS
   executer du code arbitraire (defense-in-depth).

6. **IPC-6 traceparent propagation** : verifier que `_traceparent` field
   est present dans chaque JSON-RPC request du broker vers l'executor.
   Test `test_traceparent_propagation` doit confirmer.

### Track B — THREAT_MODEL §9 + docs securite (Phase B `1f79c52`)

1. **TM-1 §9 coherence** : verifier que les 6 sous-sections §9.1-§9.6
   referencent les bons sprints et mecanismes existants dans le code.
   §9.1 (GPU consent) → verifier existence consent.json schema.
   §9.2 (loopback tiers) → verifier coherence avec LOOPBACK_ENDPOINTS_TRUST_TIERS.md.
   §9.5 (guardrails disabled) → **P2 attendu** : output filter pas wire
   end-to-end (design-only S23). L'auditeur doit confirmer que le §9.5
   documente le risque sans claim faux d'implementation.

2. **TM-2 SECURITY.md** : verifier que la policy de disclosure est
   actionnable (email, PGP optionnel, timeline explicite). Verifier
   que le scope inclut les crates Rust et les packages Python.

3. **TM-3 security.txt** : verifier RFC 9116 compliance (Expires,
   Contact, Preferred-Languages). Expires > 1 an.

4. **TM-4 BUILDING.md** : verifier que les instructions permettent de
   builder le projet from scratch (toutes les deps listees, commandes
   exhaustives). Executer les commandes et verifier qu'elles fonctionnent.

5. **TM-5 consent annotations** : verifier que `residual_threats_acknowledged`
   et `level_threat_note` sont propagees depuis l'endpoint consent
   Python → frontend GpuConsentDialog.tsx. Test `test_consent_residual_threats_field`
   doit couvrir la pipeline.

### Track C — TraceProvider opentelemetry 0.31 (Phase D `5787168`)

1. **TRACE-1 processor pipeline** : verifier que `add_trace_processor`
   et `set_trace_processors` ont la semantique annoncee (add = append,
   set = replace). Test `test_multi_processor_pipeline` et
   `test_set_trace_processors_replaces`.

2. **TRACE-2 signed canary** : verifier que `SignedCanaryProcessor` utilise
   Ed25519 avec domain separation `DOMAIN_TRACE_EVENT_V1`. Verifier que
   le tamper detection fonctionne. Tests `signed_processor_roundtrip` +
   `signed_processor_tamper_detect`.

3. **TRACE-3 OTel bridge** : verifier que `OtelProcessor` bridge vers
   `opentelemetry_sdk::trace::SdkTracerProvider` (0.31, pas 0.27).
   Verifier que la dep workspace est `opentelemetry = "0.31"`.

4. **TRACE-4 W3C Trace Context** : verifier que l'injection/extraction
   du traceparent est conforme au format W3C
   (`{version}-{trace-id}-{parent-id}-{flags}`). Tests
   `traceparent_roundtrip` + `invalid_traceparent_rejected`.

5. **TRACE-5 executor trace log path** : **P2 attendu** : verifier que
   le chemin est relatif (`"traces/executor.jsonl"`) et non resolu
   depuis `ShellDaemonPaths`. Confirmer le carry S30.

### Track D — P2 batch S28 (Phase A `b1c4148`)

1. **P2-1 generate_blocking params** : verifier que le commentaire L258
   justifie les 12 parametres et que `#[allow(clippy::too_many_arguments)]`
   est present L262.

2. **P2-2 sampler chain** : verifier commentaire inline sur le cout
   de rebuild sampler chain.

3. **P2-3 init_platform_emitter** : verifier que les 2 tests couvrent
   les 2 branches cfg (Windows/non-Windows). Sur Windows : verifier
   que `TracingWriter` est selectionne.

4. **P2-4 cold-start benchmark** : verifier que le resultat est documente
   dans le commit body `b1c4148`. Verifier < 5000 ms.

### Track E — G1 Design Review Board

Verifier que `sprint29_design_review.md` existe dans
`.planning/archive/v1.2/`. Present avec scoring D1-D5 = OK.
Absent sur sprint non-trivial = **P1**.

### Track F — Phase review completeness

- [ ] Phase review files present: 3/4 (B, C, D — Phase A pas de review)

Phase A n'a pas de fichier review. Verifier si Phase A est une phase
triviale (P2 batch + benchmark) justifiant l'absence. Si oui = OK.
Si non = **P2**.

Phase reviews findings routes dans ce document :

| Finding | Source | Track | Status |
|---|---|---|---|
| P2-REVIEW-B-1 consent.py mutation pattern | Phase B review | B (TM-5) | carry S30 1/3 |
| P2-REVIEW-B-2 §9.5 output filter not wired | Phase B review | B (TM-1) | carry S30 1/3 |
| P2-REVIEW-C-1 task_runner.rs stub | Phase C review | A (IPC-5) | carry S30 1/3 |
| P3-REVIEW-C-1 types IPC dupliques design | Phase C review | A | cosmetique |
| P2-REVIEW-D-1 executor trace log path | Phase D review | C (TRACE-5) | carry S30 1/3 |
| P3-REVIEW-D-1 plan file path inexact | Phase D review | C | cosmetique |

### Track G — HARDENING drift

Comparer HARDENING_ROADMAP §3 ligne S29 (items prescrits) vs livre :

| Item prescrit | Livre ? | Justification si non |
|---|---|---|
| External audit engagement | Non | scope-cut kickoff §7.13 : S29 prepare, S30 execute |
| Remediation findings | Non | scope-cut kickoff §7.14 : post-findings |
| Public disclosure + security.txt | Oui | Phase B SECURITY.md + security.txt |
| agents_sudo A2 TraceProvider | Oui | Phase D (PLAN-ADAPT 0.27→0.31) |
| agents_sudo B4 THREAT_MODEL §9 | Oui | Phase B |
| Process isolation broker/executor | Oui | Phase C (absorbe de S28 D2) |

### Meta-track — G8 traceability

1. Verifier que les 4 phases A-D ont chacune un
   `sprint29_phase_{X}_preflight.md` dans `.planning/archive/v1.2/`.
2. Verifier que les 3 phases B-D ont chacune un
   `sprint29_phase_{X}_review.md`.
3. Verifier la coherence verdict G8 × commit (4 EXECUTE → 4 commits
   phase livres, 0 DESIGN-CONFLICT → 0 pivot_proposal).

### Meta-track — Sprint pair S30 phase dette

S30 est un sprint pair → phase dette obligatoire (§6.2.1 Regle 1).
Candidats obligatoires :
- **P2-B-1-S28 CI Linux/macOS writers** : 3/3 reports → **MANDATORY** per §6.2.1 Regle 2.
  L'auditeur doit verifier que le kickoff S30 reserve une phase pour cet item.

Candidats recommandes :
- P2-C-1-S28 blob-serve isolation gap (2/3)
- P2-REVIEW-C-1 task_runner.rs stub (1/3)
- P2-REVIEW-D-1 executor trace log path (1/3)

---

## 3. Calibration rigor G4

L'audit DOIT trouver au minimum 1 P2+ pour verdict PASS. Sinon
verdict CONCERN et re-audit dimension supplementaire.

---

## 4. Pre-launch protocol check

Verifier :
- `*_VERSION = 1` partout (aucun bump)
- `DOMAIN_TRACE_EVENT_V1` = evenements locaux, pas wire P2P
- `DOMAIN_IPC_REQUEST_V1` / `DOMAIN_IPC_RESPONSE_V1` = internes broker↔executor
- Aucun tolerant decoder multi-version introduit
- Aucun test "legacy decode" zombie introduit
- `#[serde(default)]` avec rationale runtime tolerance

---

## 5. Out of scope pour l'audit

Ne PAS rebattre :
- D1..D5 gelees (process isolation raw JSON-RPC, cold-start benchmark,
  opentelemetry 0.31, THREAT_MODEL §9, scope disposition audit)
- Les 15 scope cuts kickoff §7
- Le choix raw serde_json vs jsonrpsee
- Le choix opentelemetry 0.31 vs 0.27
- Pin iroh 0.97

---

## 6. Verdict global attendu

- **PASS** : 0 P0, 0 P1 → S30 Phase A demarre direct
- **CONDITIONAL PASS** : 1-3 P1 fixables → S30 Phase A bloque
  tant que les `fix(sprint29): ...` ne sont pas landed
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle
