# Sprint 24 Phase A — nexus-phase-auditor review

HEAD pre-commit: `91589ea` (working tree staged, pre-commit)
Draft commit body: "feat(sprint24): Phase A — P2 cleanup batch S23 audit + PATTERNS §P35 ephemeral + §P36 redundancy"
Timebox: LIGHT-AUDIT (preflight EXECUTE, no rouge-ligne triggers)

## Verdict : PASS

0 P0/P1. 1 P3 nit documented.

### Acknowledged by G8 preflight (not re-derived)

- S1 SOTA 2026 : pynacl >= 1.6.2 dep floor correct for CVE-2025-69277 ; no new libs introduced
- S2 historiques : git log scan on all Phase A files — 0 DEVIATION/rejected/scope-cut conflict
- S3 threat model : Phase A = fixes P2 + doc patterns + dep floor + read-only API ; 0 primitive nouvelle, 0 threat regression
- S4 wire format : 0 `_VERSION` touched, `canonical.rs` not touched, pre-launch invariants preserved

---

## Dimensions

### Security

- [x] semgrep scan : no semgrep config present — fallback grep applied
- [x] `unsafe` blocks : 0 new unsafe blocks in diff (grep confirmed, pow.rs only adds arithmetic + `.min()`)
- [x] `unwrap()` in pow.rs diff : existing test-only unwraps in `#[cfg(test)]` blocks (lines 540-680) — not new; acceptable in test context
- [x] loopback/wire/zip : not touched by Phase A
- [x] `canonical.rs` : not touched
- [x] secrets grep : 0 secrets / tokens / keys in diff (grepped `AKIA|ghp_|pat_|sbfb_[a-z]+` — 0 matches)
- [x] `pynacl >= 1.6.2` dep floor in pyproject.toml:70 — confirmed set correctly

### Patterns

- [x] P35 ephemeral worker lifecycle : `docs/rust/PATTERNS.md` §P35 (lines 2053-2079) added — documents `EphemeralState` state machine, `Idle→Running→RestartPending→Idle`, `max_tasks_before_restart`, `cuda_wipe_enabled`, test guarantee. Consistent with `nexus-worker-core/src/ephemeral.rs` design (S23 Phase B).
- [x] P36 redundancy voting : `docs/rust/PATTERNS.md` §P36 (lines 2081-2106) added — documents `RedundancyDispatcher`, SHA-256 deviation from D3 BLAKE3 spec with rationale, `redundancy_factor` serde(default) + serde(skip) in TaskCanonical, reference to fix `34c77ce`. Coherent with `redundancy.py::hash_result_bytes` docstring (redundancy.py:43-52).
- [x] PyO3 rebuild procedure : `docs/shell/PATTERNS.md` Sprint 24 section (lines 2089-2114) added — `unset CONDA_PREFIX + VIRTUAL_ENV=$PWD/.venv maturin develop --release` command correct, 32 stale failures explained. Matches `CLAUDE.md §Commandes clés`.
- [x] No pattern drift introduced (diff adds documentation only for these patterns, no new code pattern untracked).

### G8 traceability

- [x] `.planning/active/sprint24_phase_A_preflight.md` exists (added as untracked A in git status)
- [x] Verdict : EXECUTE plan-as-is (scans S1-S4 all clean, 2026-04-21)
- [x] No DESIGN-CONFLICT — plan §5 coherent with code changes
- [x] SCOPE-CUT-CONSISTENT not triggered (EXECUTE verdict) — carry items tracked in kickoff §8 table

### Scope-cuts

Scope cuts from kickoff §7 grepped against diff:

- `key rotation` / `revocation` : mentions in PATTERNS.md lines 1308/1320 are **pre-existing** TLS-pinning section (S19 content), not introduced by this diff. Confirmed by `git diff HEAD -- docs/rust/PATTERNS.md` — those lines not in the diff hunk.
- `GuardrailChain` : not in diff
- `C3 handoffs` : not in diff
- `cross-process` : not in diff
- `domain fronting` : not in diff
- `redundancy persist SQLite` / `quarantine curator alerting` : not in diff
- `iroh neighborhood` : not in diff
- [x] 0 scope cut leaks detected

### Tests-delta

| Suite | Announced | Real | Status |
|---|---|---|---|
| Rust nextest | +1 (`exponent_saturation_i32`) | +1 (744 total, confirmed `pow::tests::escalating_difficulty_exponent_saturation_i32` PASS) | MATCH |
| Python coord | +3 (`get_total_kudos`, `get_total_kudos_empty`, `get_top_contributors`) | +3 (275 pass, was 272; 32 stale unchanged, 3 skip unchanged) | MATCH |
| All other suites | unchanged | not re-run (no touches) | N/A |

Rust total 744 confirmed. Coord 275 pass + 32 stale + 3 skip confirmed. Delta correct.

### Research-grounding

- [x] `Cargo.toml` : no changes in diff (confirmed `git diff HEAD -- Cargo.toml` empty)
- [x] `pyproject.toml` : only `pynacl` version floor bump `>= 1.5` → `>= 1.6.2` (CVE-2025-69277 mitigation). Traced in preflight S1 + kickoff §3 ("sprint23_audit_findings.md P2 items") + plan §5.2.
- [x] `uv.lock` : bump reflects pynacl floor change only — no new packages added
- [x] No new external API / crypto spec introduced

### Horizon long-terme + documentation amont

- [x] Phase A is a P2 cleanup batch — no new structural module introduced. No design doc required.
- [x] D1..D5 in kickoff §4 all enumerate alternatives rejected with rationale (D1 Strategy/Middleware/if-else, D2 event-bus/subclass/AOP, D3 self-report/global-replication, D4 plain-DNS/DNSCrypt/custom, D5 include-both/rotation-only/C3-only).
- [x] Solution choices in Phase A are minimal (`.min()` saturation, stdlib SHA-256, pyproject floor, 2 SQL aggregates) — no tech shortcut vs deeper option.
- [P3] kickoff §D5 mentions `~500 LOC`, `~700 LOC`, `~2400 LOC`, `~3600 LOC` (lines 226-233) as scope-sizing rationale for deferral decisions. Per rule, "LOC retrospective (mesure de gap a posteriori pour décider scope-cut) est légitime" — these are scope-cut size justifications used to justify deferral, not deliverable LOC estimates. Classification: P3 nit (borderline, legitimate usage confirmed).

---

## Findings

- **P3** : kickoff §D5 lines 226-233 use LOC counts as scope-sizing rationale for scope-cut decisions. Technically legitimate (scope-cut sizing, not deliverable estimate). No action required; note for future kickoffs to use "scope size" terminology rather than "LOC" to avoid ambiguity with the §6.7 prohibition.

---

## Recommendation

Commit autorisé. 0 P0/P1. Tests delta verified exact. G8 gate clear. Scope cuts clean.
