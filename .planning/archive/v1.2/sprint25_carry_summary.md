# Sprint 25 — Carry summary

**Ecrit** : 2026-04-22 (ouverture Sprint 25).
**Source** : `sprint24_verification.md §4+§5` + `sprint24_audit_findings.md §Findings`.
**Note** : ce fichier aurait du etre genere par S24 Phase F (meme
gap process que S22→S23). L'info existait dans verification §4
(scope cuts) + §5 (carry-over for memory) + audit_findings. Correction
process S25.

---

## 1. Cap G7 — 2/2 slots utilises

### Slot 1 — Key rotation ceremony

- **ID** : S24-D5-1
- **Description** : Ed25519 key rotation ceremony + revocation list gossip topic
- **Source** : S24 kickoff §4 D5 scope-cut + §7 item 1
- **Commit source** : scope-cut S24 (jamais implemente)
- **Severite** : P2 (infrastructure securite, pas Gate-blocker)
- **Owner** : S25 Phase B

### Slot 2 — C3 handoffs semantic dispatcher

- **ID** : S24-D5-2
- **Description** : GuardrailChain s'applique par stage lifecycle (input, output, validation) via mapping hooks → chains
- **Source** : S24 kickoff §4 D5 scope-cut + §7 item 2
- **Commit source** : scope-cut S24 (depend B1+A1 stables)
- **Severite** : P2 (amelioration architecturale, pas Gate-blocker)
- **Owner** : S25 Phase C

---

## 2. P2 tech debt (hors cap G7)

| ID | Description | Source | Status |
|---|---|---|---|
| P2-E-1 | DnsFallbackResolver TLS name per-endpoint | S24 Phase E review `72bc0b1` | Phase A S25 |
| P2-E-2 | DoH→DoT concurrent fallback | S24 Phase E review `72bc0b1` | Phase A S25 |
| P2-D-1 | Redundancy persistence SQLite | S23 audit → S24 scope-cut §7 item 4 | Defer S26 (refactor significatif) |
| P2-D-2 | Quarantine curator alerting | S23 audit → S24 scope-cut §7 item 5 | Phase A S25 |
| P2-E-1-iroh | iroh neighborhood enrichment | S23 audit → S24 scope-cut §7 item 6 | Defer S26 |

---

## 3. Hors cap — items long-terme

| ID | Description | Status |
|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | Triggers inactifs |
| LT-2 | Radicle activation | ROADMAP_COMMITMENTS, trigger tag v1.0 |
| LT-3 | Contribution family Sybil matrix | Post-v1.0 |
| LT-4 | OS biometric gate | Post-v1.0 |
