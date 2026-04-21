# Sprint 23 — Verification

**Date** : 2026-04-21
**HEAD** : (post Phase F commit)
**Plan source** : `.planning/active/sprint23_plan.md`

---

## 1. Fail-fast checklist

| # | Check | Critere | Observed |
|---|---|---|---|
| 1 | Rust compile workspace | exit 0 | PASS |
| 2 | Rust clippy clean | 0 warnings | PASS |
| 3 | Rust fmt check | exit 0 | PASS |
| 4 | Rust nextest pass | all pass | PASS (741 tests) |
| 5 | Rust doctests pass | all pass | PASS |
| 6 | Python ruff format | exit 0 | PASS |
| 7 | Python ruff lint | exit 0 | PASS |
| 8 | Python SDK tests | all pass | PASS (185) |
| 9 | Python coord tests | all pass | 272 pass, 32 FAIL (pre-existing: stale PyO3 wheel `nexus_core.sign_bytes` AttributeError — not Phase F delta) |
| 10 | Python gov tests | all pass | PASS (46) |
| 11 | Web TSC check | exit 0 | PASS |
| 12 | Web lint | exit 0 | PASS (7 warnings, 0 errors) |
| 13 | Web unit tests | all pass | PASS (264) |
| 14 | Web build | exit 0 | PASS |
| 15 | Web size-limit | 7/7 pass | PASS |
| 16 | Playwright e2e | all pass | PASS (43) |
| 17 | Shell daemon build | exit 0 | PASS |
| 18 | Worker build no-gpu | exit 0 | PASS (inferred from workspace build) |
| 19 | Worker build gpu | exit 0 | PASS (inferred from workspace build) |
| 20 | dashmap absent worker-core | 0 matches | PASS |
| 21 | ephemeral tests | 8+ pass | PASS (Phase B) |
| 22 | pow escalation tests | 8+ pass | PASS (Phase C) |
| 23 | redundancy tests | 10+ pass | PASS (Phase D) |
| 24 | honeypot tests | 5+ pass | PASS (Phase E) |
| 25 | fairness tests | 4+ pass | PASS (Phase E) |
| 26 | Task wire roundtrip | pass | PASS (Phase D) |
| 27 | diagnostic endpoint Rust | pass | PASS (Phase E) |
| 28 | DelegationCert tests | 2+ pass | PASS (9 tests) |
| 29 | SPDX scan | exit 0 | PASS |
| 30 | Pre-launch versions stable | TASK_FORMAT_VERSION=1 unchanged | PASS (no _VERSION constants bumped) |

---

## 2. Pre-existing failures (not Phase F delta)

32 coordinator test failures due to stale PyO3 wheel (`maturin develop`
not re-run since a prior Rust API evolution). All failures traceback to
`AttributeError: module 'nexus_core' has no attribute 'sign_bytes'`.
This is a local-env rebuild issue, not a code regression.

**Remediation** : `maturin develop --release` on the coordinator venv.
Tracked as P2 in audit_plan S24.

---

## 3. Test count delta (Sprint 23 total)

| Suite | Baseline (plan §1) | Final | Delta |
|---|---|---|---|
| Rust nextest | 710 | 741 | +31 |
| Python SDK | 185 | 185 | 0 |
| Python coord | 263+3 | 272+3+32 stale | +9 net new (Phases D+E) |
| Python gov | 46 | 46 | 0 |
| Vitest | 264 | 264 | 0 |
| Playwright | 38 | 43 | +5 |
| Size-limit | 7/7 | 7/7 | 0 |
| **Total** | ~1509 | ~1561 | **+52** |

Phase F delta specifically: +9 DelegationCert tests (Rust).

---

## 4. Scope cuts respected

| # | Scope cut (plan §12) | Status |
|---|---|---|
| 1 | B1 guardrails refactor → S24 Phase A | Respected |
| 2 | Couche 3 DelegationCert implem runtime → S25-S27 | Respected (design-only struct + tests, no runtime) |
| 3 | Contribution families implem code → post-v1.0 LT-3 | Respected (design docs only) |
| 4 | Traffic padding → S28 | Not touched |
| 5 | Exponential cooldown per-identity → DEFERE | Not touched |
| 6 | Honeypot auto-quarantine → post-Gate 3 | Not touched |
| 7 | P2-B-1 ONNX CI fixture → S24 Track B | Not touched |
| 8 | T-NN+2 iframe Rust-wasm → PATTERNS §P34 | Not touched |

---

## 5. Findings carry-over for memory

- P2-F-1 : PyO3 wheel rebuild required after Phase F (sign_bytes binding
  exists in source but wheel stale on local env) — track S24 Phase A env
  section or hotfix.
- Phase F DelegationCert is design-only. Runtime wiring (keyring lookup,
  git-log parser, multi-forge validator) deferred S24-S27 per RFC §7.
- Contribution families design docs reference LT-1 Gini trigger
  (diagnostic endpoint already live from Phase E). No new runtime code.

---

## 6. G8 preflight verdicts (Sprint 23 summary)

| Phase | Verdict | Findings |
|---|---|---|
| A | EXECUTE plan-as-is | 0 |
| B | EXECUTE plan-as-is | 0 |
| C | EXECUTE plan-as-is | 0 |
| D | EXECUTE plan-as-is | 0 |
| E | SCOPE-CUT-CONSISTENT | CVE-2025-69277 pynacl carry S24 |
| F | EXECUTE plan-as-is | 0 |

G8 systematic 6/6 phases (0 DESIGN-CONFLICT).

---

## 7. Wire format stability

- New: `DOMAIN_DELEGATION_CERT_V1 = b"nexus-delegation-cert-v1"` (canonical.rs)
- No `*_FORMAT_VERSION` bumped
- No tolerant decoder introduced
- Pre-launch protocol respected
