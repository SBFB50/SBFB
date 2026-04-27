# Sprint 32 — Design Review Board — Decision Scoring Report

**Date** : 2026-04-27
**Reviewer** : READ-ONLY source verification + scoring report (Haiku 4.5 Agent)
**Scope** : Factual verification of D1-D5 source recency + alternative comparisons
**Method** : crates.io API queries + GitHub releases + code inspection + Cargo.toml audit

---

## Scoring System

- **✅ GREEN** : Source recent (≤90d) + alternative verified or N/A
- **⚠️ YELLOW** : Source present but outdated OR alternative not compared
- **❌ RED** : No source OR choice contradicted by actual state

---

## Decisions Reviewed

### **D1 — iroh stack upgrade simultaneous 4 crates (0.97→0.98/0.100)**

**Claim** : iroh 0.98.0 (published 2026-04-17) + iroh-docs 0.98 + iroh-gossip 0.98 + iroh-blobs 0.100. 8 breaking changes cited. Dependency matrix confirms 4 crates must upgrade together.

**Source Verification**:

| Claim | Source | Verified | Age |
|-------|--------|----------|-----|
| iroh 0.98.0 exists, latest stable | crates.io API /crates/iroh/0.98.0 | ✅ | 10 days (2026-04-17) |
| iroh 0.98.1 also exists (patch) | crates.io API /crates/iroh/0.98.1 | ✅ Published 2026-04-20 | Not mentioned in kickoff |
| iroh-docs 0.98 requires iroh ^0.98 + iroh-blobs ^0.100 + iroh-gossip ^0.98 | crates.io deps API | ✅ | Confirmed |
| iroh-blobs 0.99 requires iroh ^0.97 (incompatible) | crates.io deps API | ✅ | Confirms "4 crates together" |
| 8 breaking changes cited | GitHub n0-computer/iroh CHANGELOG | Trustworthy source format | N/A |
| Code uses SecretKey::from_bytes(), not generate() | nexus-core-rs/src/node.rs line 245 | ✅ | No migration needed |

**Angle Blind Spot** :
- **iroh 0.98.1 (patch, 2026-04-20) not mentioned** — source cites only 0.98.0. Planner should verify patch scope.

**Scoring** : **⚠️ YELLOW**
- Source recent (10d) ✅
- Matrix confirmed ✅
- **BUT: 0.98.1 exists but unremarked** ⚠️

---

### **D2 — rusqlite 0.32→0.36 + arti-client 2.0 feature activation**

**Claim** : rusqlite 0.36 minimum for arti-client 2.0. libsqlite3-sys 0.34. bundled feature supported.

**Source Verification**:

| Claim | Verified | Status |
|-------|----------|--------|
| rusqlite 0.36 uses libsqlite3-sys ^0.34 | ✅ | crates.io confirmed |
| rusqlite 0.32 uses libsqlite3-sys ^0.30 | ✅ | crates.io confirmed |
| rusqlite 0.36 bundled feature exists | ✅ | crates.io features list |
| arti-client 2.0.0 exists, dated 2026-02-07 | ❌ **MISMATCH** | **No version 2.0 in crates.io** |
| Cargo.toml workspace says arti-client version | ✅ | Comment line 434: "arti-client = \"0.41\"" |

**CRITICAL FINDING**:

Kickoff §4 D2 states: *"arti-client 2.0.0 (2026-02-07)"*

Actual crates.io state: arti-client latest is **0.41.0** (2026-03-30)

Cargo.toml workspace comment (line 434): *"arti-client = \"0.41\""*

**The kickoff conflates version numbering.** arti-client uses 0.x semver (no 2.0 major).

**Impact** :
- ❌ Version number in decision body is **factually incorrect**
- ✅ Dependency conflict resolution logic is sound
- ✅ bundled feature supported
- ✅ 0.36 vs 0.39 tradeoff reasonable

**Scoring** : **❌ RED**
- Source metadata **incorrect** (version 2.0.0 does not exist) ❌
- **Cannot trust decision body without planner clarification on arti-client target version**
- Dependency resolution logic sound but citation needs correction

---

### **D3 — Wire max_tokens in executor GenerationRequest**

**Claim** : GenerationOptions::default().num_predict(params.max_tokens). Task_runner silently drops max_tokens.

**Source Verification**:

| Claim | Verified |
|-------|----------|
| Task_runner drops max_tokens field | ✅ nexus-executor/src/task_runner.rs line 16 confirms only model+prompt used |
| TaskExecuteParams has max_tokens | ✅ nexus-executor/src/ipc.rs line 106 |
| ollama-rs 0.2 supports GenerationOptions::num_predict() | ✅ crates.io 0.2.6 exists |

**Scoring** : **✅ GREEN**
- Gap verified ✅
- ollama-rs 0.2 API exists ✅
- No contradictions ✅

---

### **D4 — P2 batch: HARDENING compteurs + Tor log + FROST tests + Playwright**

**Claim** : P2-AUDIT-2 compteurs ~401→406 coord; P3-AUDIT-3 Tor log disabled vs unavailable; P3-AUDIT-2 FROST error tests; P2-REVIEW-B-1-S30 Playwright COEP 2/3 (MANDATORY S33).

**Source Verification**:

| Item | Verified |
|-------|----------|
| HARDENING_ROADMAP compteurs stale (401 vs 406) | ✅ Confirmed in audit_findings.md |
| Tor boot log issue documented | ✅ Confirmed in audit_findings.md |
| FROST error paths gap confirmed | ✅ Happy-path only, error paths untested |
| Playwright COEP 2/3 carry status | ✅ Confirmed in kickoff table |

**Angle Blind Spot** :
- **Playwright env failure root cause not re-verified** — kickoff assumes "coordinator not running" still valid, but no fresh env check pre-phase.

**Scoring** : **⚠️ YELLOW**
- Sources confirmed ✅
- **Playwright env assumption not re-checked** ⚠️ (mitigation plan sound, contingency exists)

---

### **D5 — Formal lift of Day 0 #3 (pin iroh 0.97)**

**Claim** : Day 0 #3 originally "iroh 0.97 pinned, upgrade voluntary" — implies dedicated sprint. S32 is the dedicated sprint. Post-S32: pin moves to 0.98.

**Source Verification**:

| Claim | Verified |
|-------|----------|
| Day 0 #3 exists, states "iroh 0.97 pin" | ✅ CLAUDE.md line 180 |
| Original phrasing "upgrade voluntary" | ✅ Confirmed in kickoff §4 D5 |
| LT-6 trigger met 2026-04-17 | ✅ iroh 0.98.0 release date |
| S32 is pair sprint for debt | ✅ Kickoff §6.2.1 Rule 1 |
| No contradiction with other Day 0 decisions | ✅ CLAUDE.md §decisions read |

**Scoring** : **✅ GREEN**
- Source recent ✅
- Trigger verified ✅
- No contradictions ✅
- Semantic reformulation sound ✅

---

## Summary Scorecard

| Decision | Score | Key Issue | Blocker ? |
|----------|-------|-----------|-----------|
| **D1** iroh stack | ⚠️ YELLOW | iroh 0.98.1 patch (2026-04-20) unremarked; scope should be verified | No |
| **D2** rusqlite + arti | ❌ RED | **Version 2.0.0 factually wrong** (actual: 0.41); must clarify before commit | **YES** |
| **D3** max_tokens | ✅ GREEN | All sources verified; gap confirmed; scope appropriate | No |
| **D4** P2 batch | ⚠️ YELLOW | Playwright env failure root cause not re-verified; contingency plan sound | No |
| **D5** Day 0 #3 lift | ✅ GREEN | Wording sound; trigger verified; no contradictions | No |

---

## Planner Action Items

1. **D2 URGENT** : Clarify arti-client target version in decision body (state 0.41, not 2.0.0). Correct commit body before merge.

2. **D1 MINOR** : Optional: acknowledge iroh 0.98.1 patch; verify no additional breaking changes beyond 0.98.0.

3. **D4 CONTINGENT** : Phase C pre-launch: if Playwright env fails with different root cause, re-evaluate exemption credibility.

4. **D5** : Proceed as-is.

---

**Reviewer Note** : One critical citation error (D2 version number) and two minor gaps identified. Technical logic is sound; issue is source documentation clarity. No decision needs reconsideration, only clarification.
