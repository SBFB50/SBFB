# Sprint 31 Design Review — Day 0 Decisions (D1-D5)

**Review Date** : 2026-04-26
**Reviewer** : Agent Explore independant (session fraiche)
**Scope** : 5 Day-0 decisions from sprint31_kickoff.md §4
**Methodologie** : Source verification (code, roadmap, threat model)
+ recency check (<= 90j) + alternative enumeration

---

## D1 — task_runner reel : Wire LlmBackend dans nexus-executor

**Scoring** : ✅

- **Source** : `task_runner.rs` stub confirme (lines 5-18), LlmBackend
  trait + OllamaBackend impl existent dans nexus-worker-core (S20
  Phase D). Code live, recency OK.
- **Alternatives** : 4 enumerees (stub carry, llama.cpp direct, dual
  backend, IPC pass-through). Toutes avec rationale factuel.
- **Crypto/spec** : N/A (pas de primitive crypto).
- **Rust-first** : ✅ OllamaBackend Rust-native (ollama-rs HTTP
  client).

---

## D2 — §9.5 output filter wire E2E

**Scoring** : ✅

- **Source** : `output_filter.py:187-341` code complet, THREAT_MODEL
  §9.5 gap documente explicitement. GuardrailChain pattern etabli
  S21 Phase C.
- **Alternatives** : 4 enumerees (worker-side, client-side, carry
  gap, re-design). Trust boundary argument solide.
- **Crypto/spec** : N/A (pas de primitive crypto dans le filter).
- **Rust-first** : ⚠️ split Rust core / Python coord binding (by
  design S21, pas nouveau). Acceptable.

---

## D3 — Tor transport phase 1 : arti-client 2.0 coordinator outbound

**Scoring** : ⚠️

- **Source** : arti-client 2.0.0 release 2026-02-07 (79j, dans le
  seuil 90j). Blog Tor Project, docs.rs, context7 1597 snippets.
  LTS annonce branche 2.x. 0 CVE RustSec.
- **Alternatives** : 4 enumerees (iroh relay Tor, SOCKS daemon,
  skip Tor, I2P/Nym). Toutes avec rationale.
- **Scope delta** : HARDENING_ROADMAP §3 S31 prescrit "wire iroh
  relay HTTPS fallback over Tor SOCKS5" — le kickoff scope down a
  coordinator outbound HTTP seul. Delta documente dans kickoff §4
  Acknowledged review findings. HARDENING_ROADMAP sera mis a jour
  Phase D pour refleter le scope reel.
- **Crypto/spec** : arti-client = Tor standard, implementation Rust
  officielle Tor Project. ✅
- **Rust-first** : ✅ arti-client Rust-native, tokio async.

**Angle mort** : API marquee "experimental" — contingency si
bootstrap echoue en integration test (fallback SCOPE-CUT option).

---

## D4 — P2 batch S30 carries + G2 HARDENING update

**Scoring** : ✅

- **Source** : Tous items verifies dans le codebase. WebAppFrame.tsx
  orphelin confirme (0 imports production). VALIDATED_BLUEPRINT
  Kirchenbauer → SynthID evolution documentee S27 Phase D.
  SPLIT_INFERENCE_DESIGN.md confidence_score = doc fix trivial.
  HTTP FROST endpoints declares dans routeur, non testes.
- **Alternatives** : Playwright COEP test defere S34 (env instable).
  Justifie.
- **Crypto/spec** : N/A (docs + tests maintenance).

---

## D5 — iroh 0.98 upgrade : SCOPE-CUT S32

**Scoring** : ✅

- **Source** : iroh 0.98.0 release 2026-04-17 (9j). Breaking changes
  documentes (#[non_exhaustive], SecretKey::generate(), relay-v2).
  Cargo.toml confirme pin iroh 0.97 actif.
- **Justification** : 5 raisons factuelles (3 hard deliverables,
  blobs compat inconnue, breaking workspace-wide, Tor fonctionne
  sans, S32 sprint pair dette). Deviation roadmap Alexandria
  documentee, pas silencieuse.
- **ROADMAP_COMMITMENTS** : LT-6 "trigger met, scheduled S32"
  explicite.
- **Angle mort** : S32 kickoff devra verifier compatibilite
  iroh-blobs 0.99 avec iroh 0.98 AVANT de geler D-decision upgrade.

---

## Blind spots identifies

1. **D3 HARDENING_ROADMAP sync** : Phase D doit mettre a jour
   l'entry S31 AVANT cloture pour eviter ambiguite downstream.
2. **iroh-blobs 0.99 compat** : ajouter a S32 Day 0 comme
   pre-requis verification.
3. **arti experimental API** : contingency fallback si bootstrap
   echoue.
4. **Output filter policy file** : `configs/output_filter_policy.
   toml.sample` doit exister ou etre cree Phase B pour que les
   tests passent (hot-reload pattern).

---

## Summary

| D# | Decision | Source | Alternatives | Assessment |
|---|---|---|---|---|
| D1 | task_runner Ollama | Live code | 4 | ✅ |
| D2 | Output filter E2E | Live code + THREAT_MODEL | 4 | ✅ |
| D3 | Tor arti-client 2.0 | 79j (Feb 2026) | 4 | ⚠️ scope delta |
| D4 | P2 batch | Live code | N/A | ✅ |
| D5 | iroh 0.98 SCOPE-CUT | 9j (Apr 2026) | justified | ✅ |

**Rigor signal G4** : 4/5 ✅ + 1/5 ⚠️. Signal >= 1 satisfait.
