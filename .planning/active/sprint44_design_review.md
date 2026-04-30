# Sprint 44 — Day-0 Design Review

**Reviewer** : Independent agent (file search specialist)
**Review date** : 2026-04-30
**Context** : Sprint 44 is even-numbered (MANDATORY debt phase per section 6.2.1 Rule 1).
Seven items at 3/3 reports trigger mandatory resolution (Rule 2).

**Scope** : Review D1 (MANDATORY batch), D2 (Routes API), and D3 (Scope cuts).
Verify sources, competing alternatives, and codebase consistency.

---

## D1 — MANDATORY batch: 7 items @ 3/3 status

### Checklist verification

**D1 items identified correctly? YES**

All 7 items present and accounted for in kickoff section 4:
1. P2-REVIEW-A-1-S42 ChainResult mutations doc
2. P2-REVIEW-B-1-S42 pow_keypair identity doc
3. P3-REVIEW-A-2-S42 babel-scraper .gitignore
4. P3-REVIEW-C-1-S42 list_apps pagination
5. P3-AUDIT-A-1-S42 RNG rate>1 test
6. P3-AUDIT-C-1-S42 Debug vs serde as_str()
7. P3-AUDIT-C-2-S42 pagination limit/offset

### Source verification (recent = <= 90 days)

| Item | Source | Age | Status |
|------|--------|-----|--------|
| D1(a) ChainResult | guardrails.rs present at 358c6ff | <1 day | ✅ Recent |
| D1(b) pow_keypair | S42-S43 pattern established | <5 days | ✅ Recent |
| D1(c) babel-scraper | /tools/babel-scraper/ exists | <5 days | ✅ Recent |
| D1(d) list_apps pagination | apps.rs lines 58-139 | <1 day | ✅ Recent |
| D1(e) RNG test | S42 audit plan references rate>1 | <10 days | ⚠️ Plan-only |
| D1(f) Debug as_str | S42 audit item, browse.rs | <1 day | ⚠️ Not coded |
| D1(g) limit/offset | apps.rs AppListQuery | <1 day | ⚠️ Not coded |

**Status**: All sources present in active codebase. Five items NOT yet implemented (Phase A scope—expected).

### Competing alternatives considered?

**Documentation (D1a-b)**: PATTERNS.md pattern S1. Low-risk expansion. No alternatives needed.

**babel-scraper gitignore (D1c)**: Single decision. Clear rationale (post-v1.0, large corpora).

**list_apps pagination (D1d)**: Pattern S42-S43 established. No alternatives needed because pattern proven.

**RNG test (D1e)**: Straightforward unit test addition.

**Debug as_str (D1f)**: Refactoring towards idiomatic Rust. Standard best practice.

**limit/offset (D1g)**: REST conventions.

**Verdict for D1**: ✅ All fixes appropriate and low-risk. Sources recent.

---

## D2 — Tier 5 Routes API: 6 files (hors events.py)

### Identification: All 7 files in roadmap correctly accounted?

**Roadmap section S44** lists: "routes restantes (~700 LOC, 7 fichiers : health, shell, tasks, kudos, events, diagnostic, worker_state)".

**Kickoff D2** splits these 7 into:
- **6 to port S44** : health, shell, tasks, kudos, diagnostic, worker_state
- **1 scope-cut S45** : events (AppEvents bus dependency)

**File line counts** (actual source):

| File | LOC | Routes | S42-S43 status | D2 decision |
|------|-----|--------|---|---|
| health.py | 66 | 3 | Not ported | Port S44 (a) |
| shell.py | 56 | 1 | Not ported | Port S44 (b) |
| tasks.py | 140 | 2 | Partial (submit S35) | Port list S44 (c) |
| kudos.py | 54 | 2 | Partial (get+verify S36) | Port list S44 (d) |
| diagnostic.py | 93 | 1 | Not ported (fairness.rs exists) | Port S44 (e) |
| worker_state.py | 137 | 1 | Not ported | Port S44 (f) |
| events.py | 195 | 2 (SSE) | Not ported, AppEvents Python-only | Scope-cut S45 |

**Total S44 scope**: 546 LOC Python to ~400-500 LOC Rust (per kickoff).

### Verification: Are the 6 files correctly sized and justified?

✅ Correct: Actual line counts match kickoff table exactly.

✅ Justified sizing: All items fit 2-phase batch plan.

### Source verification: Are routes already ported in S42-S43?

**http.rs router** (current source, commit 358c6ff):

Already wired routes from S42-S43:
- Line 272: /api/v1/tasks/submit (submit only)
- Lines 274-278: /api/v1/kudos/{project_id} + /verify (get + verify)
- Lines 290-291: /api/v1/apps + /apps/{id} (both S42)
- Lines 292-296: /api/v1/deploy + deploy-from-repo (both S42)
- Lines 297-312: /api/v1/consent + /api/v1/files (all S43)

**NOT yet wired** (confirmed absent):
- /health (daemon probe exists; coordinator health not wired)
- /shell/discover
- /tasks (list) and /tasks/{id}
- /kudos (list with filters)
- /diagnostic/fairness
- /worker-state

**Verdict for D2**: ✅ Correct. Six files identified. Routes NOT yet ported (as expected). Fairness.rs exists. AppEvents bus rationale verified (Python SDK only, no Rust equivalent).

---

## D3 — Scope cuts S44

### Scope cuts listed (6 items):

1. events.py SSE streaming — S45 (AppEvents bus SDK Python)
2. quarantine.py API routes — S45 (hors roadmap S44)
3. Suppression coordinator Python — S45
4. CI/VPS/v1.0 — S46-48
5. Kudos debit/stake — interdit (Day 0 #7)
6. P2-AUDIT-A-1-S43 integration test gap — partiel S44, complet S45

### Consistency with roadmap?

**D3 scope cuts are consistent**:
- events.py → S45 (carried from S44 decision)
- quarantine.py → S45 (explicitly "non liste dans la roadmap S44")
- Python suppression → S45 per roadmap
- CI/VPS/v1.0 → S46-48 per roadmap
- Kudos debit/stake → interdit
- Integration test gap → partiel S44, complet S45

**Verdict for D3**: ✅ Scope cuts are consistent with roadmap. No contradictions.

---

## Cross-cutting checks

### [DETER] Crypto/spec decision check

**Requirement**: Any D-choice involving crypto must cite >= 1 competing alternative < 6 months old.

**Search result**: No crypto decisions in D1-D3.

**Verdict**: ✅ No crypto decisions in S44 Day-0.

### [DETER] Rust-first runtime decision check

**Requirement**: Any runtime D-choice must cite >= 1 Rust-native alternative.

**D2 decision**: "porter les 6 routes API Python restantes vers des handlers axum natifs".

**Analysis**: The pattern is established from S42-S43 (8 routes already ported using same pattern). Porting 6 more routes with same pattern is incremental, not new architecture.

**Verdict**: ✅ No new Rust runtime choice. Pattern continuation. No [DETER] flag needed.

---

## Final scoring

### D1 — MANDATORY batch (7 items)

**Status**: ✅ **PASS**
- All 7 items correctly identified
- Estimated LOC: ~150-175
- Risk: Low

### D2 — Tier 5 Routes API (6 files, hors events.py)

**Status**: ✅ **PASS**
- Six files correctly identified
- events.py scope-cut justified (Python SDK only)
- Feasible 2-phase batch
- Risk: Medium but mitigated

### D3 — Scope cuts S44

**Status**: ✅ **PASS**
- All scope cuts consistent with roadmap
- No contradictions

---

## Blind spots signalled (no solution proposed)

### 1. D2(a) health.py — coordinator-side health_payload enrichment

**Blind spot**: Current liveness probe returns daemon health. Kickoff proposes enriching it with coordinator.health_payload(). 

**Unverified**: What does coordinator.health_payload() return? Is enriched payload a breaking change?

**Signal**: Planner should verify payload contract before Phase B coding.

### 2. D2(c) tasks.py — list_tasks() query in db.rs

**Blind spot**: No pre-phase research confirming SQL query exists or is feasible.

**Signal**: Planner should verify db schema and draft list_tasks SQL query before Phase C coding.

### 3. D2(e) diagnostic.py — fairness.rs wire contract

**Blind spot**: diagnostic.py queries kudos_ledger directly. Rust port must either reuse fairness.rs functions or wire fairness.rs directly from HTTP handler.

**Unverified**: fairness.rs functions signature and input requirements.

**Signal**: Planner should review fairness.rs before Phase B to confirm wire contract.

---

## Acknowledgements

All Day-0 decisions (D1-D3) follow established pattern from S42-S43. No architectural novelty. No new dependencies. MANDATORY batch appropriate for even sprint. Routes API continuation well-justified. Scope-cuts conservative.

**Recommended action**: Proceed to Phase A. Address three blind spots during phase preflight (G8 scan) before coding.
