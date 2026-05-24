# Audit Gate Checks Prompt

Run the Phase 0 audit gate for Sprint `{N-1}` before Sprint `{N}` Phase A
begins. This is the vendor-neutral port of the nexus audit gate process. Any
provider may execute it if it can read files, run shell commands, and write the
required planning artifact.

Write the result to:

`.planning/active/sprint{N-1}_audit_findings.md`

This audit covers the complete diff of Sprint `{N-1}` (all phases, all
commits). It is not a phase review; it is a retrospective quality gate.

Stay provider-neutral and ASCII only. Every claim must cite a file path, command
output, commit SHA, or explicit assumption.

## Required Sources

```bash
git log --oneline <sprint_start_sha>..HEAD
git diff --stat <sprint_start_sha>..HEAD
cat .planning/active/sprint{N}_audit_plan.md
cat .planning/active/sprint{N-1}_kickoff.md
cat .planning/active/sprint{N-1}_plan.md
cat .planning/active/sprint{N-1}_verification.md
ls .planning/active/sprint{N-1}_phase_*_review.md
ls .planning/active/sprint{N-1}_phase_*_codex_review.md
cat docs/rust/PATTERNS.md
cat docs/shell/PATTERNS.md
cat docs/security/THREAT_MODEL.md
cat docs/security/HARDENING_ROADMAP.md
```

Read `audit_plan.md` first to understand the tracks assigned to this audit.
Then form your own opinion from the code before reading PATTERNS.md (opinion-
first pattern to avoid anchoring).

## 9 Tracks

Execute each track independently. Classify findings as P0/P1/P2/P3.

### Track A — Suites Verification

Check that all test suites pass and counts match the sprint plan.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit && npm run build && npm run size && bash scripts/scan-en-strings.sh)
```

Compare observed counts to the plan `§1 Etat verifie` table. Flag regressions
(count dropped) as P1. Flag count mismatches with plan as P2.

### Track B — Security

Scan the full sprint diff for security patterns:

```bash
git diff <sprint_start_sha>..HEAD -- '*.rs' '*.ts' '*.tsx' '*.py' | rg -n "unsafe|unwrap\(\)|panic!|todo!|unimplemented!|eval\(|innerHTML|dangerouslySetInnerHTML|child_process|exec\(|Command::new|allow-same-origin"
rg -n "AKIA|ghp_|pat_|sbfb_[A-Za-z0-9_]+|password|secret|token" --glob '!*.md' --glob '!*.lock' <changed-paths>
rg -n "serde\(default\)|canonical|JCS|PeerCreds|loopback|sandbox|sign|verify|provenance" <changed-paths>
```

Check each finding against `docs/security/THREAT_MODEL.md` T0-T5. New `unsafe`
without `SAFETY:` comment is P1. Hardcoded secrets are P0. New trust boundary
crossing without documentation is P1.

### Track C — Patterns

Read `docs/rust/PATTERNS.md` and `docs/shell/PATTERNS.md` AFTER forming an
opinion from the code diff. Check for:

- New patterns introduced without documentation.
- Existing patterns violated in new code.
- Tech debt items (T-NN) created or resolved.
- Canonical bytes handling consistency.
- `serde_json` vs JCS usage.

Pattern violations in new code are P2. Undocumented new patterns are P2.

### Track D — Scope

Compare the sprint diff to the kickoff scope cuts and plan sections:

```bash
rg -n "Scope cuts|Non-goals" .planning/active/sprint{N-1}_kickoff.md .planning/active/sprint{N-1}_plan.md
git diff --name-only <sprint_start_sha>..HEAD
```

For each scope cut, verify no code touches that area. Scope leak (code in a
cut area without explicit reopening) is P1. Scope cut documented but code
present is P1.

### Track E — Tests Delta

Compare announced delta tests (each phase commit body `## Delta tests`) to
actual counts:

```bash
git log <sprint_start_sha>..HEAD --format=%B | rg -A 5 "## Delta tests"
cargo nextest run --workspace --locked 2>&1 | tail -5
(cd web && npm run test:unit 2>&1 | tail -5)
```

Announced +N but actual +M where M < N is P2. Zero-delta phase without
justification is P2. Test that proves less than claimed (mock-only for
integration claim) is P2.

### Track F — Review Files

Check that every phase has the required artifacts:

```bash
ls .planning/active/sprint{N-1}_phase_*_preflight.md
ls .planning/active/sprint{N-1}_phase_*_review.md
ls .planning/active/sprint{N-1}_phase_*_codex_review.md
```

For each review file:
- Verdict must be exactly `## Verdict: PASS` (not `PASS-PENDING`, not spaced).
- Codex review must be raw output (not rewritten by Claude).
- Sprint and phase identity must match across preflight, review, codex, and
  commit subject.

Missing review file for a phase commit is P1. `PASS-PENDING` in final review
is P1. Phase identity mismatch is P1.

### Track G — Carry-Overs

Check carry discipline:

```bash
rg -n "carry|P2-|P3-|LT-|T-NN" .planning/active/sprint{N-1}_verification.md .planning/active/sprint{N-1}_plan.md
rg -n "carry|P2-|P3-|LT-" CLAUDE.md
```

For each carry item:
- Is it documented in verification.md?
- Is it routed to the next sprint audit plan?
- Has it appeared in 3+ reports (G7 escalation)?
- Is the exit condition defined?

Carry without routing is P2. 3-report carry without escalation is P2.

### Track H — HARDENING

Check HARDENING_ROADMAP compliance:

```bash
rg -n "S{N-1}|sprint.?{N-1}" docs/security/HARDENING_ROADMAP.md
rg -n "zone rouge|R-iroh|R-wasmtime|R-libcrux|R-pyodide" CLAUDE.md docs/security/
```

Pre-requirements for the audited sprint that are not delivered are P1.
Zone rouge status changes without documentation are P1.

### Track I — Meta-Process

Check process discipline:

```bash
git log <sprint_start_sha>..HEAD --format="%s"
git log <sprint_start_sha>..HEAD --format="%B" | rg -c "## Contexte|## Fichiers|## Delta tests|## Verification|## Scope cuts|## G8 traceability|## Pre-launch protocol|## Codex verification|## Carry closure"
```

For each phase commit:
- Subject matches `type(scope): Sprint N Phase X — title`.
- Body has exactly 9 markdown sections.
- `## Codex verification` section is present.
- No `--no-verify` or `--amend` on phase commits.
- No emoji in commit messages.

Missing body section is P2. Wrong commit format is P2. Skipped hook evidence
is P1.

## Verdict

Aggregate findings:

| Verdict | Condition |
|---------|-----------|
| `PASS` | 0 P0, 0 P1, at least 1 P2+ documented or exhaustive negative evidence |
| `CONDITIONAL PASS` | 0 P0, 0 P1, conditions to respect documented |
| `FAIL` | Any P0, or any P1, or >= 3 P1 |

G4 rigor signal: 0 P0/P1 AND 0 P2+ is `CONCERN` (not `PASS`). A clean audit
must explain the exhaustive negative evidence.

## Output Template

```markdown
# Sprint {N-1} Audit Findings

Date: YYYY-MM-DD
Auditor: <provider>
Sprint: {N-1}
Diff: `<start_sha>..<end_sha>`
Verdict: **PASS | CONDITIONAL PASS | FAIL**

## Track A — Suites
- Rust nextest: <count> (plan: <count>)
- Vitest: <count> (plan: <count>)
- Findings: <list or none>

## Track B — Security
- Patterns scanned: <count>
- Findings: <list or none>

## Track C — Patterns
- PATTERNS.md alignment: <status>
- Tech debt: <items created/resolved>
- Findings: <list or none>

## Track D — Scope
- Scope cuts verified: <count>/<total>
- Findings: <list or none>

## Track E — Tests Delta
- Announced: +<N> Rust, +<N> Vitest
- Observed: +<N> Rust, +<N> Vitest
- Findings: <list or none>

## Track F — Review Files
- Phases: <count>
- Reviews present: <count>/<count>
- Codex reviews present: <count>/<count>
- Findings: <list or none>

## Track G — Carry-Overs
- Items carried: <count>
- 3-report escalations: <count>
- Findings: <list or none>

## Track H — HARDENING
- Pre-requirements checked: <status>
- Zone rouge status: <status>
- Findings: <list or none>

## Track I — Meta-Process
- Phase commits: <count>
- Body format: <compliant count>/<total>
- Findings: <list or none>

## Summary

| Severity | Count | Items |
|----------|-------|-------|
| P0 | <N> | <ids> |
| P1 | <N> | <ids> |
| P2 | <N> | <ids> |
| P3 | <N> | <ids> |

## Conditions (if CONDITIONAL PASS)
- <condition 1>
- <condition 2>

## Carry-Over To Sprint {N}
- <item>: <owner>, <trigger>, <exit condition>
```

## Rules

- Form your own opinion from the diff BEFORE reading PATTERNS.md or prior
  reviews (opinion-first avoids anchoring bias).
- Do not soften findings to avoid conflict. A P1 is a P1.
- Every finding must cite file:line or commit SHA.
- Exhaustive negative evidence is required for a clean track; do not write
  "no findings" without listing what was checked.
- The audit is retrospective. Do not fix code; document what needs fixing.
  Only P0/P1 items trigger fix commits `fix(sprint{N-1}): ...` before the
  next sprint proceeds.
