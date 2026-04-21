# Sprint 23 — Carry summary

**Écrit** : 2026-04-20 (ouverture Sprint 23).
**Source** : `sprint22_audit_findings.md §3` + `sprint22_verification
.md §5` + `HARDENING_ROADMAP.md §3 S23`.
**Note** : ce fichier aurait dû être généré par S22 Phase F (cf.
README §6.2.1). L'info existait dans audit_findings §3 mais pas
dans un fichier dédié. Correction process S23.

---

## 1. Cap G7 — 0/2 slots utilisés

### Slot 1 — LIBRE

P2 audit cleanup batch (6 items) absorbé Phase A S23 = pas un carry
formel (cleanup batch < 500 LOC trivial, catégorisé gate remediation
pas carry-over).

### Slot 2 — LIBRE

Redundancy voting = item HARDENING_ROADMAP §3 S23 (roadmap planifié),
pas carry report S22. Co-deferré S22→S23 via amendment (pas via cap
G7).

### Hors cap — T-NN+2 iframe Rust-wasm (PATTERNS §P34)

Status inchangé. Triggers (a/b/c/d) non-activés 2026-04-20. Tech
debt tracking, pas carry formel.

---

## 2. Items P2 carry from audit gate S22

| ID | Description | Sev. | Phase S23 |
|---|---|---|---|
| P2-S22A-1 | `dashmap` dep directe stale worker-core | P2 | Phase A |
| P2-S22A-3 | PATTERNS §P33 struct snapshot obsolète | P2 | Phase A |
| P2-B-2 | wrapper.ts L309 commentaire "scaffold" obsolète | P2 | Phase A |
| P2-E-1 | `_reload_policy_locked` suffix trompeur | P2 | Phase A |
| P2-E-2 | LOC estimation pattern README §6.7 amend | P2 | Phase A |
| P2-Meta-hook-1 | bypass_audit_trail.log forward-only clarif | P2 | Phase A |
| P3-C-1 | DOMAIN_PROVENANCE/WARRANT_CANARY re-export | P3 | Phase A |

---

## 3. Items roadmap HARDENING §3 S23

| Item | LOC approx | Phase S23 |
|---|---|---|
| Ephemeral workers restart + VRAM wipe | ~500 | Phase B |
| Escalating PoW geometric ramp | ~300 | Phase C |
| Redundancy voting 3-worker majority | ~400 | Phase D |
| Honeypot Eclipse detection | ~400 | Phase E |
| Fairness observability endpoint | ~120 | Phase E |
| Contribution families design docs | ~400 docs | Phase F |
| Couche 3 DelegationCert format | ~100 | Phase F |

---

## 4. Items déférés (PAS S23)

| Item | Raison | Target |
|---|---|---|
| B1 guardrails refactor | Option B arbitré user | S24 Phase A |
| P2-B-1 ONNX end-to-end CI | infrastructure jsdom | S24 Track B |
| Couche 3 implem runtime | séquencé multi-forge | S25-S27 |
| Traffic padding | aligné Nym mixnet | S28 |
| LT-3 contribution families code | post-v1.0 | LT-3 trigger Gini |

---

## 5. LT reclassifications (rappel)

- **LT-2 Radicle-v1.0** : sorti cap G7 S22 (trigger tag v1.0 only)
- **LT-3 Contribution family Sybil** : design-only S23, code post-v1.0
- **LT-4 OS biometric gate** : post-v1.0 (trigger v1.0 + FROST N1)
