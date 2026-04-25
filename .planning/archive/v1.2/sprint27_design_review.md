# Sprint 27 — Design Review Board scoring report (G1)

**Date** : 2026-04-25
**Reviewer** : agent Explore independant (session fraiche, contexte minimal)
**Sprint** : 27
**Decisions reviewees** : D1..D5 (Day-0 frozen)

---

## Scoring table

| D-ID | Decision | Scoring | Finding | Severity |
|------|----------|---------|---------|----------|
| D1 | SynthID-inspired z-test watermark | ✅ | Source recente (Nature Oct 2024) + alternative verifiee (BIRA Sept 2025) | — |
| D2 | Couche 3 git-log parser offline | ✅ | RFC 4880/8709 standard, git-log --show-signature proven | — |
| D3 | Trust-web ONG bootstrap seeds | ⚠️ | Arch sound mais DelegationCert pattern non independamment verifie | MED |
| D4 | P2 batch S26 audit (7 items) | ✅ | Routine cleanup, pas de dep externe | — |
| D5 | Gate 3 showcase Gate 3 scope docs | ✅ | Doc update, pas de risque technique | — |

---

## Detail par D-decision

### D1 : Output Watermark — SynthID-inspired z-test Detection ✅

**Evidence validee** :
- SynthID-Text (Nature 2024, oct. 2024) : Tournament Sampling + z-test confirmes. Open source Hugging Face.
- BIRA Attack (arXiv:2509.23019, sept 2025) : >99% evasion rates sur Kirchenbauer KGW confirme.
- Kirchenbauer KGW (arXiv:2301.10226, ICML 2023) : green/red-list biasing, vulnerable BIRA confirme.

Alternatives comparees : MarkLLM (EMNLP 2024, Python-only), Full Tournament Sampling (overhead CDF). Documentes.

### D2 : Couche 3 Multi-forge — Git-log Offline Parser ✅

**Evidence validee** :
- git log --show-signature (RFC 4880 OpenPGP, RFC 8709 SSH) : universel.
- Radicle (Ed25519 refs signes, 1.7.0+ fix replay attacks) : LT-2 path documente.

Alternatives comparees : API polling (rate limits), blockchain (overhead). Documentes.

### D3 : Trust-web Bootstrap — ONG Seed Keys ⚠️

**Finding** : DelegationCert primitive (S23 `attestations/delegation.rs`) n'a
pas ete independamment verifiee contre les standards d'attestation externes
(C2PA Claim structures, WebAuthn patterns). L'architecture (anchor + transitive
decay) est sound, mais le format de signature manque de spec formelle pre-S28.

**Recommendation** : avant Phase C, documenter le format signature DelegationCert
(issuer, delegatee, trust_level, expiry) avec mapping C2PA. Garantit migration
S28 ONG post-placeholder.

### D4 : P2 Batch S26 Audit ✅

Routine fixes (<30 LOC chacun). Pas de dep externes.

### D5 : Gate 3 showcase Gate 3 Scope ✅

Documentation-only. Pas de risque technique.

---

## Checklists

### [DETER] Crypto/Spec

| D | Requirement | Verified |
|---|-------------|----------|
| D1 | SynthID vs Kirchenbauer (BIRA rejected) | ✅ Nature 2024 + arXiv:2509 |
| D2 | git-log (RFC 4880/8709) vs API polling | ✅ RFC standard |
| D3 | ONG anchor vs CA / Keybase | ⚠️ Pattern sound, spec mapping C2PA manquant |

### [DETER] Rust-first

| D | Requirement | Verified |
|---|-------------|----------|
| D1 | SynthID injection Rust llama.cpp vs Python MarkLLM | ✅ MarkLLM rejete (Python-only) |
| D2 | git-log parser Rust vs shell/Python | ✅ Rust native |
| D3 | trust-web gossip Rust daemon-core | ✅ Rust documente |

---

## Recommendations

- **HIGH** (D3 pre-Phase C) : documenter format signature DelegationCert
  avec mapping C2PA (`spec.c2pa.org/specifications/1.4/attestations/`).
- **MED** (R-S27-4) : tester conflit llama.cpp logit-bias + grammar
  (llguidance). Issue #13605 signale instabilite server-mode.
- **LOW** (LT-2) : Radicle 1.7.0+ (replay-attack fix) comme prerequis
  avant tag v1.0.

---

## Verdict : MOSTLY PASS (4/5 ✅ + 1/5 ⚠️)

Le sprint peut proceder. D3 ⚠️ require une action pre-Phase C
(documentation format DelegationCert).

---

## Sources

- Nature 2024 : `nature.com/articles/s41586-024-08025-4`
- BIRA arXiv : `arxiv.org/abs/2509.23019`
- Kirchenbauer ICML 2023 : `arxiv.org/abs/2301.10226`
- C2PA Attestation Spec : `spec.c2pa.org/specifications/1.4/attestations/`
- llama.cpp Issue #13605 : `github.com/ggml-org/llama.cpp/issues/13605`
