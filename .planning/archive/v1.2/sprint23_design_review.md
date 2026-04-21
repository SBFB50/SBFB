# Sprint 23 — Design Review Board (G1)

**Reviewer** : agent Explore indépendant (session fraîche, contexte
minimal).
**Date** : 2026-04-20.
**Input** : draft D1..D5 kickoff S23.
**Méthode** : WebSearch + codebase grep + validation sources.

---

## Scoring

| D | Thème | Source récence | Alternatives | G1 Rust | Score |
|---|---|---|---|---|---|
| D1 | Ephemeral workers | ✅ ≤30j (CUDA 2026) | ✅ 3 énumérées | ✅ cudarc | ✅ PASS |
| D2 | PoW escalation | ⚠️ Tor 2023 (3a) | ⚠️ Equi-X léger | ✅ Pure Rust | ⚠️ CONDITIONAL |
| D3 | Redundancy voting | ⚠️ SecureDrop vague | ✅ 3 énumérées | ✅ Python+Rust | ⚠️ CONDITIONAL |
| D4 | Honeypot Eclipse | ✅ Tor/ETH 2016-18 | ✅ 3 énumérées | ✅ Python+Rust | ✅ PASS |
| D5 | B1 timing | ✅ Design doc ✓ | ✅ 3 options | ✅ PyO3 | ✅ PASS |

---

## Findings

### ⚠️ D2-G1-1 — Equi-X re-évaluation recomandée S24

D2 rejette Equi-X comme "over-engineered" sur données 2023. En 2026,
GPU botnets sont plus efficients ; Monero RandomX (base Equi-X) prouvé
ASIC-resistant depuis 2019. La ramp géométrique SHA256 suffit pour le
threat model actuel (réseau <200 nodes, Gate 2), mais pourrait être
insuffisante post-Gate 3 si Sybil massive cible le réseau.

**Action planner** : accepter D2, ajouter trigger re-validation S24
audit checklist "PoW choice si Sybil volume > threshold empirique
post-Gate-2".

### ⚠️ D3-G1-1 — SecureDrop cite est stylistica, pas technique

SecureDrop n'a pas de consensus voting formel en production. Meilleur
précédent : **BOINC/Folding@Home result validator majority** (10+ ans
production, result redundancy ×3 comparaison hash).

**Action planner** : accepter D3, amender note design Phase D avec
référence BOINC result validation pattern.

---

## Verdict

**0 ❌ (aucun design conflict).** 2 ⚠️ acceptables avec notes.
Procéder Phase A.

Sources consultées :
- Tor PoW spec (spec.torproject.org/hspow-spec)
- NVIDIA CUDA Runtime API Memory Management (docs.nvidia.com)
- BOINC result validation (boinc.berkeley.edu/trac/wiki/ValidationIntro)
- Filecoin Expected Consensus (spec.filecoin.io)
- Tor Sybil paper (arXiv 1602.07787, USENIX Security 2016)
- Ethereum Eclipse (IACR 2018/236)
- Barrack AI blog 2026 (GPU memory never cleared)
- cudarc crate (docs.rs/cudarc/0.12)
