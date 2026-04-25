# Sprint 28 — Design Review Board (G1)

**Date** : 2026-04-25
**Reviewer** : agent Explore independant (session fraiche)
**Scoring** : D1 ✅, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ⚠️

---

## D1 — Watermark end-to-end wiring : ✅

Source solide (G9 2026-04-25, grep confirme zero call site). Alternatives
documentees (Ollama API + Tournament Sampling + detection-only). Risks
identifies (R-S28-1 llguidance conflict). Scope ~30-50 LOC realiste.

## D2 — Platform event writers : ⚠️

Source presente (G9 stubs L166-189). Mais alternative `tracing-journald`
(Tokio ecosystem, subscriber layer auto-routing journald) non comparee
dans le document. `libsystemd` et `oslog` crates cites mais pas declares
Cargo.toml yet. Testing gap (Windows dev) acknowledged R-S28-2.

## D3 — Process isolation design : ⚠️

Source codebase reelle (monolithe confirme grep). Mais JSON-RPC 2.0 vs
gRPC rejet sans analyse quantifiee (latency/overhead). Cold-start
Ollama benchmark prereq defere S29. Design doc speculative OK per risk
register R-S28-4. Coherence HARDENING_ROADMAP preservee.

## D4 — External audit scope : ✅

Source recente (G9 + HARDENING_ROADMAP S29). Scope matrix Cure53/ToB
clear. Prerequisites (D3 + B4) listed. Auto-audit alternative rejetee
proprement.

## D5 — Scope disposition Nym + MIG : ⚠️

Source presente (G9 2026-04-25) mais non-sourcee externes :
VALIDATED_BLUEPRINT.md cite sans provision independante. Nym SDK
latences (200-800ms) non referencees (arXiv/GitHub/blog). NVIDIA MIG
specs (consumer != MIG) assertion non datee. Alternative Rust-native
Nym/mixnet absente (DETER #2 unchecked). HARDENING_ROADMAP update
promised Phase D.
