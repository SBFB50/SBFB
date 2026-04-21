# Sprint 23 — Audit Plan pour Sprint 24

**Redige** : 2026-04-21
**Sprint audite** : Sprint 23 (Ephemeral workers + escalating PoW +
honeypot + redundancy voting + contribution families foundation)
**Commits** : de `2438c59` (plan) a HEAD Phase F inclus.

---

## 1. Checklist d'audit (Phase 0 Sprint 24)

L'auditeur independant (nouvel agent, session fraiche) doit :

1. Lire ce document + `sprint23_kickoff.md` D1-D5 + `sprint23_plan.md`
2. Verifier chaque commit Phase A-F contre le plan (scope, fichiers,
   tests annonces vs reels)
3. Scanner pour regressions, dead code, security smells
4. Emettre `sprint23_audit_findings.md` avec verdict PASS / CONDITIONAL
   PASS / FAIL + items P0-P3

---

## 2. Dimensions a auditer

### Track A — Ephemeral lifecycle (Phase B)

- [ ] `ephemeral.rs` state machine transitions couvrent tous les paths
- [ ] Feature gate `gpu-ephemeral` compile sans/avec correctement
- [ ] `cudarc` gated derriere cfg, pas de panic si GPU absent
- [ ] Worker restart signal integre correctement dans runtime.rs
- [ ] Config TOML sample coherent avec struct EphemeralConfig

### Track B — PoW escalating (Phase C)

- [ ] Geometric ramp overflow-safe (saturate, pas wrap)
- [ ] Daily reset at midnight UTC correct (timezone edge cases)
- [ ] Per-(consumer, model) isolation reelle (pas de cross-leak)
- [ ] PowCounter SQLite thread-safe sous concurrent access
- [ ] Dynamic difficulty dans gossip subscribe compile proprement

### Track C — Redundancy voting (Phase D)

- [ ] `redundancy_factor` dans Task wire = `#[serde(default)]` pour
  robustesse runtime (Python omission → 1)
- [ ] `redundancy_factor` EXCLU du canonical bytes (dispatch-only)
- [ ] BLAKE3 hash comparison deterministic (canonical bytes, not raw JSON)
- [ ] Quarantine outliers route vers queue existante (pas silencieux)
- [ ] Dispatcher routing : factor=1 → passthrough, factor>1 → redundancy

### Track D — Honeypot eclipse (Phase E)

- [ ] Ed25519 keypair generation pour canary peers = ephemeral (pas persiste)
- [ ] 80% / 3 rotations threshold documenté et testé aux limites
- [ ] Fairness Gini math correct (edge case ledger vide → 0.0, pas NaN)
- [ ] Diagnostic endpoints proteges par loopback bearer auth
- [ ] Pas de fuite de canary secret keys dans les logs/responses

### Track E — DelegationCert (Phase F)

- [ ] Domain separation effective (test present)
- [ ] Serde roundtrip JSON correct (test present)
- [ ] Validation fingerprint/algo stricte (pas d'input non-sanitise)
- [ ] Design docs self-contained (pas de reference cassee)
- [ ] Pre-launch protocol respecté (pas de VERSION bump)

### Track F — Process / meta

- [ ] G8 preflight systematique 6/6 (documents presents dans active/)
- [ ] Commit bodies contiennent delta tests cumule + scope cuts
- [ ] Pas de dead code introduit (unused imports, unreachable branches)
- [ ] PATTERNS.md mis a jour si patterns nouveaux (P35 ephemeral, P36
  redundancy attendus)
- [ ] SPDX headers presents sur tous les fichiers nouveaux

---

## 3. Items connus (carry-over Sprint 23)

| ID | Severite | Description | Source |
|---|---|---|---|
| P2-D-1 | P2 | RedundancyDispatcher persistence SQLite (in-memory only S23) | Phase D scope cut |
| P2-D-2 | P2 | Quarantine curator alerting (queue only, no notification) | Phase D scope cut |
| P2-E-1 | P2 | iroh neighborhood enrichment (diagnostic basic only) | Phase E scope cut |
| P2-E-2 | P2 | pynacl dep floor >=1.6.2 (CVE-2025-69277 carry) | Phase E G8 |
| P2-E-3 | P2 | KudosLedger public API (fairness reads ledger internals) | Phase E scope cut |
| P2-F-1 | P2 | PyO3 wheel rebuild stale (sign_bytes AttributeError) | Phase F verification |
| T-NN+2 | carry | iframe Rust-wasm (PATTERNS §P34) | S22 carry |
| LT-2 | carry | Radicle sortie cap G7 (trigger tag v1.0) | S22 carry |

---

## 4. Recommandations Sprint 24

- **Theme propose** : Re-run sampling + DNS fallback + key rotation
  (HARDENING_ROADMAP §3 S24 line)
- **Phase A** : absorber P2-F-1 PyO3 rebuild + P2-E-2 pynacl dep floor
  + PATTERNS.md §P35/P36 updates (cleanup batch)
- **B1 guardrails** : scope cut S23 → peut devenir S24 Phase B/C si
  budget le permet (arbitrage user kickoff S24)
- **DelegationCert runtime** : pas S24. Le RFC dit S24 = git-log parser
  offline, pas le wiring complet.

---

## 5. Format attendu pour findings

```markdown
# Sprint 23 — Audit Findings

Verdict : PASS | CONDITIONAL PASS (list conditions) | FAIL

## P0 — Bloquants
(items qui cassent le runtime ou violent une invariante security)

## P1 — Significatifs
(items qui degradent la qualite mais ne bloquent pas)

## P2 — Mineurs
(cleanup, naming, docs gaps)

## P3 — Observations
(style, suggestions d'amelioration non-blocking)
```
