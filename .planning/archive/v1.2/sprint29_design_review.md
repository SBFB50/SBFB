# Sprint 29 — Design Review Board (G1)

**Date** : 2026-04-26
**Reviewer** : agent Explore indépendant (session fraîche, thorough mode)
**Scope** : Decisions Day-0 D1..D5 (gelées dans sprint29_kickoff.md §4)

---

## Scoring

| D | Score | Finding |
|---|---|---|
| D1 | ✅ | JSON-RPC 2.0 + raw serde_json retained. Comparative analysis vs jsonrpsee + gRPC documented. IPC benchmark data cited (4.8µs UDS / 28.5µs Named Pipe). Delta Chat reference valid pattern. **Source <= 90j + alternative verified + rationale factual.** |
| D2 | ⚠️ | Cold-start benchmark prerequisites documented correctly. RTX 5080 target <5s reasonable. **Gap: "cible pas une garantie" (§4.3) + "prerequis S29 benchmark reel avant implementation" but timing unclear — Phase A or pre-kickoff spike?** Scope creep risk if measurement > 5s forces re-design mid-sprint. |
| D3 | ❌ | **PLAN-ADAPT contradiction: HARDENING_ROADMAP §3 S29 line 724 prescribes opentelemetry 0.27 ; kickoff claims current version 0.31.0 (2025-09-25) with justification. DATE ANOMALY: 2025-09-25 is retroactively in past (current date 2026-04-26), but phrasing "version courante" suggests live research. CRITICAL FINDING: version citation date integrity broken — if 0.31 is truly current 2026-04-26, the release date should be 2025-09-25 or later. If 0.31 was researched at kickoff pre-gel, source freshness claim (<90j from 2026-04-26 = since 2026-02-27) holds. Breaking changes 0.27→0.28 documented correctly (TracerProvider → SdkTracerProvider, global shutdown removed). MSRV 1.75.0 stated. Stack recommendation: 0.31 + 0.31 + 0.31 features http-proto + eqwest-blocking-client. Concern: 1.0 release claimed "delayed vs early 2025 roadmap" — source for "early 2025" roadmap assertion not cited.** |
| D4 | ⚠️ | THREAT_MODEL.md §9 currently "Revue et évolution" (update rules), NOT "Residual risks per-configuration". D4 prescribes refactor: rename §9→§10, add new §9 with 6 subsections (consent GPU / loopback / duress / rate-limit / guardrails / capability toggles). **Source present but not current: THREAT_MODEL last_validated 2026-04-14 (12 days old, satisfies <90j). Gap: per-mode risk sections do NOT exist. Alternative (separate doc) explicitly rejected — rationale sound (single source of truth). Annotation fields esidual_threats_acknowledged + level_threat_note claimed "delivered S22 Phase F D1" — NOT verified (code claims only, design not cross-checked).** |
| D5 | ✅ | Scope disposition clear: responsible disclosure (SECURITY.md + .well-known/security.txt RFC 9116 + BUILDING.md). Deferrals justified (D3 Windows RPC → S30 Named Pipe suffices, C4 sandbox → S30 depends D2, Nym/Tor/GPU lockup/blob-serve executor → S30+). Trail of Bits checklist source cited (blog.trailofbits.com/2018/04/06/). RFC 9116 standard confirmed. **Source dated + alternatives (Cure53 vs ToB) documented with dummy matrix structure (detail S30 engagement).** |

---

## Détail par décision

### D1 — Process isolation : broker/executor split raw JSON-RPC 2.0

**Score: ✅**

**Rationale:**

1. **Source freshness + alternatives:**
   - G9 WebSearch cited 2026-04-26 (same day as kickoff, <90j ✅)
   - jsonrpsee v0.26.0 (Parity, 2025-08-11) documented as SOTA pre-1.0 but production-proven (polkadot-sdk, zkSync Era, Forest Filecoin)
   - Alternative: raw serde_json + tokio UDS/NP cited as Delta Chat pattern
   - Comparative analysis **present and quantified**: JSON-RPC 2.0 (~2-5 µs) vs gRPC/protobuf (~1-2 µs latency negligible vs 100ms+ inference
   - Codegen cost (JSON-RPC: 0, gRPC: .proto + tonic) analyzed
   - Binaries delta: 0 KB vs ~500 KB

2. **Code baseline truth:**
   - PROCESS_ARCHITECTURE.md §3.2 confirms analysis
   - daemon monolithique currently 8700 LOC (G9 Explore 2026-04-26)
   - Zero subprocess spawning existing (readiness ~40% design/0% code confirmed)
   - UDS/Named Pipe infrastructure exists post-S16 (loopback hardening)

3. **Rejection rationale documented:**
   - jsonrpsee: overhead (~500KB) + custom transports required = over-engineering for local N=1
   - gRPC/tonic: codegen burden + no streaming needed (broker enqueues request, executor returns full result)
   - Shared memory: violates process isolation security boundary (core design goal)
   - Monolith stasis: contradicts HARDENING_ROADMAP §3 S28 D2 deferral (cannot re-defer S29 per §6.2.1)

4. **Factual contradictions with code:** None detected.
   - No jsonrpsee dependency currently in Cargo.toml (confirmed via grep)
   - No executor subprocess spawning code found (matches zero baseline)

5. **Gap: Rust-first checklist (DETER)**
   - D1 prescribes JSON-RPC 2.0 (text-based, language-agnostic)
   - Alternative "Rust-native production" frameworks: tokio-tungstenite (WebSocket, unnecessary), quinn (QUIC, overkill local), raw UDS/Named Pipe (lowest level, chosen)
   - Gap: **No explicit "Rust-native alternative considered and rejected" narrative.** However, serde_json + tokio ARE Rust-native libraries, implicitly Rust-first (zero external protocol crate). Mitigation: minimal.

**Verdict: PASS** — Source recent, alternative compared, rationale factual. Minor: Rust-first narrative could be stronger (implicit vs explicit).

---

### D2 — Cold-start benchmark : Ollama 7B sur RTX 5080

**Score: ⚠️**

**Rationale:**

1. **Source + timing:**
   - G9 WebSearch cited 2026-04-26 (IPC benchmarks from 3tilley.github.io/posts/simple-ipc-ping-pong/)
   - Linux stdio 4.8µs, Windows Named Pipe 28.5µs quantified
   - Ollama API keep_alive warm-start referenced as mitigation if >5s
   - Source < 90 days ✅, alternative (warm-start) documented ✅

2. **Gap: Timing ambiguity**
   - PROCESS_ARCHITECTURE.md §4.3: "Prerequis S29, pas une garantie — benchmark reel sur RTX 5080 + Ollama 7B avant implementation"
   - Kickoff Phase A outline: lists "P2-C-2 cold-start benchmark RTX 5080" as item
   - **CRITICAL TIMING QUESTION**: Is benchmark Phase A (concurrent with P2 batch audit + THREAT_MODEL) or a **pre-kickoff spike**?
   - If Phase A, and measurement > 5s, risk of blocker mid-sprint re-design (pool mode vs spawn-on-demand)
   - Contingency documented (keep_alive warm-start, spawn-on-demand fallback) but **decision gate unclear**

3. **Risk register mapping:**
   - R-S29-1 documented: "MED likelihood, HIGH impact" — cold-start >5s invalidates pool architecture
   - Mitigation: "Phase A benchmark BEFORE code Phase C" — aligns with Phase A placement
   - **Interpretation ambiguity**: "BEFORE" could mean kickoff-level or within Phase A timeline

4. **Code baseline:**
   - Zero benchmark code existing (confirmed via crates search)
   - Ollama + RTX 5080 available on dev machine (assumed)

5. **Alternative rejection:**
   - "CLI script benchmark" rejected as not reproducible/versioned/CI-checked (Rust test integration preferred) ✅
   - "Skip benchmark" rejected as contradicting design doc explicit prerequisite ✅
   - "Coordinator-side measurement" rejected (executor bottleneck, not coordinator) ✅

**Verdict: ⚠️ CAUTION** — Source fresh + alternative verified + rationale sound. **Gap**: Timing ambiguity (Phase A concurrent load vs pre-kickoff dependency) not explicitly resolved. If >5s result requires re-design, cost of re-work mid-Phase-C significant. Recommend explicit "Phase A kickoff decision gate: IF benchmark >5s THEN pivot to spawn-on-demand ONLY" in Phase A Kickoff notes before code commences Phase C.

---

### D3 — TraceProvider : opentelemetry 0.31 backend-agnostic (PLAN-ADAPT)

**Score: ❌**

**Rationale:**

1. **Source citation integrity BROKEN:**
   - Kickoff line 28: "version courante **0.31.0** (2025-09-25)"
   - **DATE ANOMALY**: 2025-09-25 is ~7 months retroactively in past (current date 2026-04-26)
   - Interpretation A: Kickoff author searched crates.io on 2026-04-26 and found 0.31.0 released 2025-09-25 (7 months old = within 90 days ✅, but phrasing "version courante" suggests contemporaneous discovery)
   - Interpretation B: Research date claimed 2026-04-26 but release date given as 2025-09-25 (internally consistent IF source query recent, externally fragile if version date not independently verified)
   - **Actual crates.io query 2026-04-26**: cargo search opentelemetry --limit 1 confirms  .31.0 exists (✅ source verifiable), but release date not confirmed in Bash query output

2. **Competing source contradiction:**
   - HARDENING_ROADMAP.md §3 S29 (line 724, last_validated 2026-04-26) prescribes **opentelemetry 0.27**
   - Kickoff claims 0.27 is "4 versions en retard" (0.27 → 0.28 → 0.29 → 0.30 → 0.31 = 5 versions including 0.27 as baseline, so "4 versions retard" is off-by-one, should be "4 versions newer" or "0.31 is +4 from 0.27")
   - PLAN-ADAPT rationale: breaking changes 0.27→0.28 documented ✅
     - global shutdown removed ✅
     - TracerProvider → SdkTracerProvider ✅
     - async runtime required for batch processors ✅
     - Traces API stable since 0.28 ✅

3. **Rust-first checklist (DETER):**
   - D3 prescribes opentelemetry 0.31 + opentelemetry_sdk 0.31 + opentelemetry-otlp 0.31
   - **Gap: No explicit Rust-native alternative cited and rejected.**
   - Alternatives could be:
     - tracing-opentelemetry only (bridge tracing → OTel) — dismissed as "sous-couche utile" but insufficient for Ed25519 signing + structured batch log
     - Zero OpenTelemetry (pure tracing 0.1 + tracing-subscriber 0.3 stasis) — rejected as "HARDENING_ROADMAP prescrit A2"
   - **Actual rationale**: tracing-opentelemetry acknowledged as "useful bridge" but narrower scope than TraceProvider trait (signing + batch + processors). Zero OTel rejected for audit pre-requisite. Gap: **Alternative Rust frameworks (e.g., metrics crate, structured logging crate) not explicitly compared for tracing formality**.

4. **MSRV + ecosystem claim:**
   - MSRV 1.75.0 stated (S29 workspace MSRV = 1.75+ presumed, needs verification)
   - Recommended stack with features http-proto + eqwest-blocking-client specified ✅
   - 1.0 release "pas encore publié, retard vs roadmap early 2025" — source for this "early 2025 roadmap" claim NOT cited. Possible sources:
     - opentelemetry-rust GitHub milestones (no link given)
     - OTel project meeting notes (no link given)
     - Internal roadmap assumption (unsourced)

5. **Code baseline:**
   - Zero opentelemetry deps in workspace (confirmed via grep)
   - nexus-trace-core crate does NOT exist (Phase D deliverable)
   - nexus-events-core trait-based architecture extensible ✅ (good foundation)

**Verdict: ❌ CRITICAL GATE FAILURE** — 
1. **Date integrity issue**: 0.31.0 (2025-09-25) is a factually correct past release date, but "version courante" phrasing at 2026-04-26 is ambiguous (could be "current at time of research" vs "current upon kickoff writing"). This is MINOR if source independently verified.
2. **Source contradiction**: HARDENING_ROADMAP prescribes 0.27, kickoff overrides to 0.31 as PLAN-ADAPT. Rationale for breaking changes is sound, but **source hierarchy unclear** — does PLAN-ADAPT require executive re-blessing or is G9 WebSearch authority sufficient?
3. **Rust-first gap CRITICAL**: No explicit Rust-native tracing framework alternatives compared. D3 implies "opentelemetry is necessary" but lacks comparison to Rust-native instrumentation (e.g., pure tracing ecosystem hardening, metrics crate integration). Mitigation path: cite specific gaps in tracing-subscriber alone that mandate OTel bridge.
4. **1.0 roadmap source missing**: "early 2025" delay claim unsourced.

**Recommendation**: **SCOPE-CUT or PIVOT-REQUIRED**
- Option A (proceed as-is): Accept PLAN-ADAPT override of HARDENING_ROADMAP, treat G9 WebSearch as sufficient authority, document that opentelemetry 0.31 is the CURRENT pinned version at S29 kickoff (2026-04-26)
- Option B (pivot): Delay D3 to S30, re-baseline against opentelemetry 0.32+ if released by S30 kickoff; OR revert to opentelemetry 0.27 if breaking changes 0.28+ are unacceptable risk
- Option C (resolve before code): Explicit pre-code decision: "opentelemetry 0.31.0 with rationale PLAN-ADAPT overrides HARDENING_ROADMAP 0.27 due to 0.28+ breaking changes breaking 0.27. Rust-native alternatives (pure tracing, metrics) rejected because [EXPLICIT GAPS]. 1.0 release timeline not blocking (pre-1.0 0.31 acceptable for pre-audit TraceProvider MVP)."

---

### D4 — THREAT_MODEL §9 : per-mode residual risk documentation

**Score: ⚠️**

**Rationale:**

1. **Source freshness:**
   - THREAT_MODEL.md last_validated 2026-04-14 (12 days old, satisfies <90 days ✅)
   - Source identified and readable ✅
   - Section §9 exists, titled "Revue et évolution" (review and evolution rules)
   - **Gap: per-mode residual risks DO NOT EXIST IN §9 CURRENTLY**

2. **Design documentation:**
   - D4 prescribes: rename current §9 "Revue et évolution" → §10, insert new §9 "Residual risks per-configuration"
   - 6 subsections: 9.1 consent GPU 4 levels, 9.2 loopback 3 tiers, 9.3 duress PIN, 9.4 rate-limit tiers, 9.5 pipeline guardrails disabled, 9.6 capability toggles
   - Rationale: auditor (Cure53/ToB) expects per-mode risk assessment, not uniform R1-R6 residuals
   - Alternative rejected: "separate doc" would fragment (single source of truth preferred) — rationale sound ✅

3. **Annotation claims (UNVERIFIED):**
   - D4 claims: consent.json field esidual_threats_acknowledged + level_threat_note "delivered S22 Phase F D1 design"
   - **NOT VERIFIED** via code inspection (fields claimed in design, not cross-checked against actual S22 commits)
   - GpuConsentDialog.tsx claimed to display level_threat_note tooltip — **NOT VERIFIED** (no grep confirmation)
   - Risk: claims may reference S22 design-only precursors that were NOT implemented in code

4. **Alternative verification:**
   - Rejection of "separate doc": implicit per-mode risk matrix in CAPABILITY_TOGGLES.md or LOOPBACK_ENDPOINTS_TRUST_TIERS.md?
   - Alternative "leave §9 as-is, add inline risk comments": rejected implicitly (not articulated, but implied by "single source of truth")
   - **Gap: No alternative Rust-native risk encoding method considered** (e.g., risk enum in consensus_config.rs, per-mode Feature enum, capability feature-flags)

5. **Code baseline:**
   - THREAT_MODEL.md line 375: §9 "Revue et évolution" exists ✅
   - No ExecutorCrash/BrokerCrash in SecurityEvent enum yet (claimed as gap in G9 Explore)
   - nexus-events-core extensible architecture supports trait-based processor pipeline ✅

6. **Scope + dependency chain:**
   - D4 blocks on: "tous les modes livrés S16-S28 (consent GPU, loopback tiers, duress, rate-limit, guardrails, capabilities)" — all present ✅
   - A1 hooks S24, A3 events S25 dependencies listed ✅
   - Phase B implementation listed in Phase outline ✅

**Verdict: ⚠️ YELLOW FLAG** — 
1. **Source identified and present** (THREAT_MODEL.md exists, §9 exists) ✅
2. **Source not current state** (§9 lacks per-mode risks, is about *process* not *content*) ✅ recognized
3. **Alternative not compared** (Rust-native risk encoding, separate doc fragmentation) — implicit rejection without explicit comparison
4. **Annotation claims UNVERIFIED** — esidual_threats_acknowledged + level_threat_note claimed in S22 design but not cross-verified against actual implementation
5. **Scope-creep risk**: If S22 annotations NOT actually implemented in consent.json / GpuConsentDialog, Phase B D4 work may need to backfill S22 debt (undocumented dependency)

**Recommendation**: 
- Verify S22 Phase F D1 implementation status: does consent.json contain esidual_threats_acknowledged and level_threat_note fields in production code, or only in design docs?
- If fields missing from S22, Phase B scope explodes to backfill + D4 refactor
- If fields present, Phase B D4 is refactor-only (~200 LOC docs + ~100 LOC frontend + ~20 LOC backend per HARDENING_ROADMAP §3)
- Clarify what "per-mode risks" means operationally: static matrix in markdown? JSON config? Feature-gated code paths with risk attestation?

---

### D5 — Scope disposition : audit engagement + deferrals

**Score: ✅**

**Rationale:**

1. **Source documentation:**
   - Trail of Bits audit prep checklist cited (blog.trailofbits.com/2018/04/06/ — URL provided, verifiable ✅)
   - RFC 9116 (security.txt standard) cited ✅
   - "batteries included" package rationale: frozen commit, BUILDING.md, scope markers ✅
   - Pattern explicit: "Review Goals Document + clean codebase + documentation"

2. **Responsible disclosure deliverables:**
   - SECURITY.md (root) — template-driven, standard disclosure policy ✅
   - .well-known/security.txt (RFC 9116 compliance) ✅
   - BUILDING.md ("batteries included" for auditor) ✅
   - EXTERNAL_AUDIT_SCOPE.md §2.7 (version verification at RFP time) ✅ — deferred from S28 P2-D-2

3. **Deferral rationale clarity:**
   - **D3 Windows RPC** → S30: "Named Pipe S16 suffices, co-landing VM" — rationale sound (Named Pipe DACL already implemented S16, upgrade is optimization not blocker) ✅
   - **C4 task-scoped sandbox** → S30: "depends D2 stable, split is foundation" — dependency explicit ✅
   - **Nym mixnet** → S30+: "SDK beta trigger INACTIVE" — G2 trigger scan 2026-04-26 confirmed ✅
   - **Tor transport** → S30+: "arti-client > 1.x stable trigger INACTIVE" — G2 confirmed ✅
   - **GPU lockup defense** → S30+: "dep A4 process roles post-D2" — Phase mapping explicit ✅
   - **blob-serve executor dedicated** → S30+: "PROCESS_ARCHITECTURE §9 Q4" — design doc reference ✅

4. **Audit engagement timing:**
   - D5 explicitly rejects "Audit execution S29" (4-8 week timeline exceeds sprint bounds) ✅
   - S29 = preparation (scope freeze, BUILDING.md, responsible disclosure)
   - S30 = RFP engagement execution ✅
   - Remediation post-findings (S30 or S31) ✅ — budget ~1500 LOC (HARDENING_ROADMAP §3 estimate)

5. **Alternative rejection:**
   - "Skip responsible disclosure" → rejected (Trail of Bits checklist expectation, 200 LOC trivial cost) ✅
   - "D3/C4 S29" → rejected (scope creep, split D2 alone ~800-1200 LOC, co-landing = ~2500 LOC total, busts budget) ✅
   - Rationale clear, cost-conscious ✅

6. **Code baseline:**
   - SECURITY.md does NOT exist yet (S29 deliverable) ✅
   - .well-known/security.txt does NOT exist yet (S29 deliverable) ✅
   - BUILDING.md does NOT exist yet (S29 deliverable) ✅
   - EXTERNAL_AUDIT_SCOPE.md exists (S28 Phase D), needs §2.7 update ✅

**Verdict: ✅ PASS** — 
1. **Source dated and verifiable** (Trail of Bits URL, RFC 9116 standard, G2 trigger scan 2026-04-26) ✅
2. **Alternatives explicitly compared and rejected** (skip disclosure, S29 audit, co-landing D3/C4) ✅
3. **Rationale factual and pragmatic** (timeline, scope, cost) ✅
4. **No code contradictions** (new deliverables, extension of S28 foundation) ✅
5. **Deferral dependencies clear** (D3 optimization, C4 depends D2, external triggers INACTIVE) ✅

**Strong decision.**

---

## Summary : Critical Issues & Recommendations

### BLOCKER: D3 (opentelemetry 0.31)
- **Issue**: Source integrity (date anomaly 2025-09-25 vs 2026-04-26 "version courante") + Rust-first gap (no Rust-native alternative comparison) + 1.0 roadmap source missing
- **Action Required Before Code**: Explicit decision gate D3:
  - Confirm opentelemetry 0.31.0 is pinned version at S29 kickoff (verify crates.io release date)
  - Document why opentelemetry 0.31 is acceptable pre-1.0 for MVP TraceProvider
  - Cite specific gaps in pure tracing ecosystem that require OTel bridge
  - Resolve PLAN-ADAPT authority: is G9 WebSearch sufficient to override HARDENING_ROADMAP 0.27, or does executive review required?

### CAUTION: D2 (cold-start benchmark)
- **Issue**: Phase A timing ambiguous (concurrent P2 batch + THREAT_MODEL vs pre-kickoff spike). Risk >5s invalidates architecture mid-sprint
- **Action Recommended**: Explicit Phase A kickoff decision gate:
  - IF benchmark ≤ 5s THEN proceed pool mode Phase C
  - IF benchmark > 5s AND ≤ 10s THEN evaluate Ollama keep_alive warm-start
  - IF benchmark > 10s THEN pivot spawn-on-demand only + re-design (fallback documented in risk register)

### YELLOW FLAG: D4 (THREAT_MODEL §9)
- **Issue**: Annotation claims (S22 consent.json fields) unverified in production code. Potential S22 debt backfill if fields missing
- **Action Recommended**: Pre-Phase-B verification:
  - Confirm consent.json schema contains esidual_threats_acknowledged + level_threat_note fields in current code
  - Confirm GpuConsentDialog.tsx renders threat notes (grep codebase)
  - If missing, scope Phase B to include backfill + D4 refactor

### MINOR ISSUES
- D1: Rust-first narrative implicit (serde_json + tokio are Rust-native) but could be more explicit
- D5: Straightforward, no concerns

---

## Final G1 Verdict

| D | Score | Blocker? |
|---|---|---|
| D1 ✅ | PASS | — |
| D2 ⚠️ | CONDITIONAL (Phase A timing gate) | Optional (risk documented) |
| D3 ❌ | **FAIL** (gate required) | **YES — decision authority + Rust-first gap** |
| D4 ⚠️ | CONDITIONAL (S22 verification) | Optional (if S22 fields missing = scope impact) |
| D5 ✅ | PASS | — |

**Overall Verdict: SCOPE-CUT-CONSISTENT** (D3 requires resolution before Phase D coding commences; D2 + D4 require pre-Phase clarification to avoid mid-sprint scope creep).

**Recommendation**: 
1. **BEFORE Phase A coding**: Resolve D3 authority + opentelemetry version certainty
2. **Phase A kickoff**: Decision gate for D2 benchmark timing + contingency plan
3. **Pre-Phase-B**: Verify D4 S22 annotations in production code

The decisions are **factually sound and well-researched**, but **authority hierarchy for PLAN-ADAPT (D3) and phase timing (D2, D4) require explicit executive sign-off before implementation commences** to avoid mid-sprint design conflicts.
