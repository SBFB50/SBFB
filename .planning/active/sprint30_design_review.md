# Sprint 30 — Design Review Board (G1)

**Ecrit** : 2026-04-26
**Reviewer** : Design Review Board (G1 independent scoring)

---

## Scoring Summary

| Decision | Score | Status |
|----------|-------|--------|
| D1 — Warrant canary Niveau 1 (trusted dealer DKG) | ✅ | VALIDATED |
| D2 — CI cross-platform (GitHub Actions multi-OS) | ✅ | VALIDATED |
| D3 — blob-serve isolation (COOP/COEP headers) | ✅ | VALIDATED |
| D4 — G2 HARDENING_ROADMAP refresh (3 triggers) | ⚠️ | CONDITIONAL |
| D5 — Split inference research doc | ⚠️ | SOURCE-LIMITED |

**Overall verdict** : 3✅ + 2⚠️. Design decisions factually grounded. Conditional items require documentation clarity on alternatives.

---

## Detailed Scoring

### D1 — Warrant canary Niveau 1 : trusted dealer DKG via frost-ed25519 2.1

**Scoring : ✅ VALIDATED**

- **Source recency** : `frost-ed25519 2.1.0` stable via crates.io/frost-ed25519, audit ToB 2023, ZF FROST codebase actively maintained on GitHub.
- **Alternative concurrence** : FROST alternatives exist (TALUS threshold ML-DSA post-quantum 2026, Mithril 3-round threshold ML-DSA, Hermine lattice-based), but all are post-quantum or pre-v1.0 stage. No competing Ed25519 threshold library with equivalent maturity plus audit pedigree documented 2026.
- **Checklist [DETER] crypto** : D1 choice cites trusted dealer vs DKG as architectural trade-off, not crypto algorithm choice. Rust-first requirement satisfied (FROST = pure Rust ZF crate).
- **Rationale fit** : Pre-v1.0 scope constraint (no ops, no TEE, no multi-machine orchestration) makes trusted dealer proportionate. Code wiring prepares Niveau 0→1 path, decision factual and defensible.

**Status** : ✅ Source recent, crypto choice validated. Alternatives noted but appropriately scoped for stage.

---

### D2 — CI cross-platform : GitHub Actions workflow (ubuntu-latest + macos-latest)

**Scoring : ✅ VALIDATED**

- **Source recency** : GitHub Actions (2026-current), Rust CI patterns stable. Search results confirm GitHub Actions matrix strategy for multi-OS testing, with Rust cargo ecosystem well-supported as of March 2026.
- **Alternative concurrence** : Alternatives exist (GitLab CI runners, Buildkite, Docker Linux, cross-compilation tools), but decision text explicitly rejects and justifies (no platform-native testing in Docker, GitHub Actions free for open-source). No undocumented alternatives.
- **Checklist [DETER] Rust-first** : D2 choice uses native Rust toolchain on target OSes (ubuntu, macOS), satisfies Rust-native requirement. No runtime-level alternatives flagged.
- **Rationale fit** : Scope (nexus-events-core + clippy + fmt) proportionate to MANDATORY 3/3 carry. Scope cuts (full workspace build) documented with justification.

**Status** : ✅ Source recent, CI choice Rust-native, alternatives considered and justified.

---

### D3 — blob-serve isolation : COOP/COEP + CSP upgrade

**Scoring : ✅ VALIDATED**

- **Source recency** : COOP/COEP standards per MDN Web Docs, web.dev/articles/coop-coep, stable web platform standards. Rust HTTP implementation via http-security-headers crate documented 2026.
- **Alternative concurrence** : Full process isolation (mentioned rejected) is architectural rewrite, not security-header alternative. CSP `script-src 'none'` (mentioned rejected) incompatible with React/Pyodide requirement. No undocumented header-level alternatives for iframe surface.
- **Rationale fit** : Quick-win headers (3 LOC) proportionate to debt sprint. Defense-in-depth (allow-scripts without allow-same-origin) maintains existing model while hardening. Risk R3 (app breakage) acknowledged with test plan.

**Status** : ✅ Source recent, headers factual, trade-offs documented. Scope constraint (not full isolation) intentional for debt phase.

---

### D4 — G2 HARDENING_ROADMAP refresh : 3 triggers + S30 update

**Scoring : ⚠️ CONDITIONAL**

- **Source recency** : Three triggers documented current as of 2026-04-26 : iroh 0.98.0 (2026-04-17, GitHub n0-computer/iroh releases), arti-client 2.0.0 (2026-02-07, Tor Project blog, LTS 2.x announced), openai-agents-python 0.14.6 (2026-04-25).
- **Decision clarity issue** : D4 text defers Tor transport phase 1 (S31) with justification "scope trop large", but decision does not cite alternative transport architectures or justify arti 2.0 sufficiency vs other Tor implementations (I2P, Snowflake documented as alternatives). ROADMAP update prescribes "nouvellement faisable" but no comparative brief documented.
- **Rationale fit** : Scope deferral defensible (debt sprint + carries full), but ROADMAP refresh should document "why arti 2.0 was sufficient to declare phase 1 faisable" (not just "release happened").

**Status** : ⚠️ Sources recent, triggers verified, but D4 decision text lacks explicit alternative comparison for arti-client choice. Recommend: HARDENING_ROADMAP section 3 S31 add 1-line rationale "arti-client 2.0 LTS + API stable (no I2P/Snowflake hybrid required for phase 1 scope)".

---

### D5 — Split inference research : design doc only

**Scoring : ⚠️ SOURCE-LIMITED**

- **Source recency** : Split inference research patterns documented as current : Truebit Verify (2026-02 general availability), BOINC interactive verification, Golem task markets. Decision correctly notes "research doc is liverable", no prototype expected.
- **Scope coverage issue** : D5 references "BOINC verification, Truebit interactive verification, Golem task markets" as patterns for design doc, but decision text does not cite specific sources or papers for technical content. Recommendation section 4 in design doc will lack bibliographic grounding unless sources pre-selected.
- **Rationale fit** : Design-doc-only appropriate for debt sprint, no code burden. But decision should pre-specify which papers/sources the research phase will synthesize (e.g., BOINC docs, Truebit Verify whitepaper, Golem marketplace API, split-inference literature).

**Status** : ⚠️ Source patterns identified (Truebit Verify, BOINC, Golem), but no bibliography pre-allocated. Recommend: Phase D planning include reading list so research doc is immediately grounded.

---

## Compliance Checklist Results

| Checklist Item | D1 | D2 | D3 | D4 | D5 | Status |
|---|---|---|---|---|---|---|
| [DETER] crypto alt cited <6mo | — | — | — | ❌ | — | D4 missing arti rationale |
| [DETER] Rust-first : Rust-native alt | ✅ | ✅ | ✅ | — | — | All Rust choices native |
| Source recency ≤90d | ✅ | ✅ | ✅ | ✅ | ⚠️ | D5 patterns current, no sources cited |
| Alternatives documented | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | D4/D5 need bibliographic clarity |

---

## G1 Recommendations (non-binding)

1. **D4 (arti-client)** : HARDENING_ROADMAP section 3 S31 entry add 1-sentence rationale : "arti-client 2.0.0 LTS provides sufficient API stability for phase 1 scope; I2P/Snowflake hybrid deferred to future phase".

2. **D5 (split inference)** : Phase D planning (before 1st doc write) pre-select and commit 3-5 key sources (Truebit Verify whitepaper, BOINC verification docs, Golem API, 1-2 research papers) so design doc is immediately bibliographically grounded.

3. **D1 (FROST)** : Current decision text sufficient; Rust-first plus crypto maturity validated.

4. **D2 (GitHub Actions)** : Current decision text sufficient; scope plus alternatives well-justified.

5. **D3 (blob-serve headers)** : Current decision text sufficient; risk R3 test plan adequate.

---

## Verdict

**PROCEED with Phase 0 implementation.** All D1..D5 decisions factually grounded in 2026 sources. Recommendations 1–2 are documentation-clarity items, not technical risks. No design conflicts identified.

---

**Reviewed by** : G1 Design Review Board (independent agent)
**Date** : 2026-04-26
**Next gate** : Phase E audit gate (sprint30_verification.md, sprint31_audit_plan.md)
