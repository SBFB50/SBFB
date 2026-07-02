# Phase Review Prompt

You are doing the driver-side phase review before the independent auditor
approves the commit. Stay provider-neutral: use ordinary file reads and shell
commands, not vendor-specific agent or tool syntax. Use the staged diff,
working tree state, `.planning/active/`, and the sprint plan as ground truth.

The final mechanical gate artifact for a phase is
`.planning/active/sprint{SPRINT}_phase_{PHASE}_review.md`. Do not invent
alternate final filenames. If this review is a driver handoff rather than the
independent audit, make that explicit in the report body so the auditor can
replace or complete it before commit. Driver handoff verdicts may use
`PASS-PENDING`; they are not committable. The committable final verdict after
Codex verification must be exactly `## Verdict: PASS`.

## Repo Context

Identify the current sprint and phase from:

```bash
git status --short
git diff --stat HEAD
git diff --name-only HEAD
git log --oneline -20
rg -n "Phase {PHASE}|Scope cuts|Research consulte|Tests plan|Commit cible" .planning/active/
```

Read `.planning/active/sprint{SPRINT}_kickoff.md`,
`.planning/active/sprint{SPRINT}_plan.md`, and any
`.planning/active/sprint{SPRINT}_phase_{PHASE}_preflight.md` or
`.planning/active/sprint{SPRINT}_phase_{PHASE}_pivot_proposal*.md`.
Extract the phase scope, frozen Day 0 decisions, scope cuts, expected delta
tests, G8 status, and commit target.

## Required Review

1. **Staging coherence.** Run `git status --short` and separate phase files
   from planning, debt, generated artifacts, and accidental files. A phase
   commit must be atomic. Planning-only edits, scope-cut work, cache files,
   build outputs, or unrelated refactors must be split before the phase commit.
   Check Rust module coherence with:

   ```bash
   git diff --cached -- '*.rs' | rg '^\+pub mod '
   git diff --name-only --cached
   ```

2. **Scope match and scope-cut verification.** Compare `git diff HEAD` to the
   phase section in the plan. Grep every kickoff scope cut against changed
   files:

   ```bash
   rg -n "^## 6\. Scope cuts|^## 8\. Scope cuts|Scope cuts" .planning/active/sprint{SPRINT}_kickoff.md
   git diff HEAD --name-only
   git diff HEAD -- <file>
   ```

   Any touched scope cut is blocking unless the kickoff and plan were
   explicitly revalidated before code.

3. **Three-block verification.** Final pre-commit review requires all three
   verification blocks, regardless of touched language. Red stops the commit;
   do not suggest `--no-verify`, `#[ignore]`, `xfail`, or skipping a suite.

   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo build --workspace --release --locked
   cargo nextest run --workspace --locked
   cargo test --workspace --locked --doc

   uv run ruff format --check packages/
   uv run ruff check packages/
   uv run pytest packages/nexus-sdk/tests/ -q
   uv run pytest packages/nexus-coordinator/tests/ -q
   uv run pytest packages/nexus-app-gov/tests/ -q

   cd web
   npx tsc --noEmit -p tsconfig.app.json
   npm run lint
   npm run test:unit
   npm run build
   npm run size
   npx playwright test
   bash scripts/scan-en-strings.sh
   cd ..
   ```

4. **Delta tests.** Compare the real after-counts to the plan and prior commit
   body. Name any suite with `0` delta and explain why that is acceptable for
   this phase. Use current command output, not memory.

5. **Modified-file branch coverage.** For every existing modified code file,
   identify new methods and branches, then prove at least one test exercises
   each non-trivial path:

   ```bash
   git diff HEAD --name-only -- '*.py' '*.rs' '*.ts' '*.tsx'
   git diff HEAD -- <file> | rg '^\+.*(def |fn |async fn |if |match |switch )'
   rg -n "new_method|new_branch_term" tests packages web crates
   ```

   Untested business logic over roughly ten lines, new wiring, or new protocol
   behavior is a blocking review finding. Defensive one-line branches may be a
   concern if the main path is covered.

6. **Security and protocol surfaces.** Call out any touched code involving
   signing, provenance, canonical bytes, schemas, loopback trust, sandboxing,
   SBFB bridge boundaries, secrets, path traversal, unsafe Rust, or dependency
   pins. Use targeted grep:

   ```bash
   rg -n "unsafe|unwrap\(\)|panic!|todo!|unimplemented!" crates packages web
   rg -n "serde_json::to_string|canonical|JCS|schema|PeerCreds|loopback|sandbox|sign|provenance" <changed-paths>
   semgrep --config .semgrep/sbfb.yml <changed-paths>
   ```

6bis. **Docs-contract frontier check (README §6.12, ACTOR TEST — amendement
   2026-07-02).** For every primitive the phase ships or reshapes, ask « who
   READS this primitive? »: another node = wire, an external client = API —
   INCLUDING a loopback API read by a distinct runtime (e.g. the Operator
   front reading `/api/*` or an SSE contract) —, a network app =
   contract/CSP, another LLM = prompt-kind/knowledge. NEVER substitute the
   « 0 wire bump / not propagated between nodes » test (the conflation that
   silenced the three S80 frontiers for 9 phases). If the answer names a
   frontier: (a) the generated étiquette / parity artifact must be IN this
   commit, and (b) the frontier must be appended to the sprint's
   GUIDE+`llms.txt` closure list (Definition-of-Done (d), §3.3 livrable 3).
   A frontier with neither is a P2 finding; report it in `## Findings`.

7. **Research grounding and G8.** Confirm the phase has a G8 preflight or
   pivot proposal. Check that new dependencies, external APIs, crypto,
   standards, or protocol specs are traced in plan research with source and
   date. Absence is at least a concern; crypto/spec work without research is
   blocking.

8. **Patterns drift.** Check that new code follows documented patterns in
   `docs/rust/PATTERNS.md` and `docs/shell/PATTERNS.md`. Flag new patterns
   introduced without documentation. Check tech debt items (T-NN) created or
   resolved:

   ```bash
   rg -n "T-NN|PATTERN|canonical|JCS|serde_json" docs/rust/PATTERNS.md docs/shell/PATTERNS.md
   git diff HEAD -- docs/rust/PATTERNS.md docs/shell/PATTERNS.md
   ```

9. **Horizon long-terme.** Evaluate whether the phase solution will hold at
   10x scale, with 100 contributors, in 2 years. Flag short-term hacks that
   create architectural debt. Check if the approach matches what mature OSS
   projects do (S1a evidence from preflight).

10. **Livrables check.** Verify every deliverable listed in the plan phase
    section is present in the diff. Missing deliverables are blocking. Extra
    files not in the plan must be justified.

    ```bash
    rg -n "Livrables|Fichier|Description" .planning/active/sprint{SPRINT}_plan.md
    git diff HEAD --name-only
    ```

11. **Carry routing.** Route unresolved P2/P3 findings to
    `sprint{SPRINT}_verification.md` and `sprint{SPRINT+1}_audit_plan.md`.
    Check 3-report carry items for G7 escalation. Every carry must have owner,
    trigger, and exit condition.

12. **Commit body draft.** Draft the commit body from facts in the diff and
    verification output. It must use exactly 9 markdown sections:
    `## Contexte`, `## Fichiers`, `## Delta tests`, `## Verification`,
    `## Scope cuts`, `## G8 traceability`, `## Pre-launch protocol`,
    `## Codex verification`, and `## Carry closure` or
    `## Carry closure / Unblock`. Put `Security delta` inside
    `## Codex verification`, not as a separate top-level section. Include
    `Co-Authored-By` only as a trailer when an applicable policy or contributor
    identity is known.

## Output

Write a concise markdown review using this structure:

```markdown
# Sprint {SPRINT} Phase {PHASE} Review

## Verdict: PASS-PENDING | CONCERN | FAIL

## Scope And Staging
## Three-Block Verification
## Delta Tests
## Modified-File Branch Coverage
## Security And Protocol
## Research And G8
## Scope Cuts
## Codex verification
## Commit Body Draft
## Findings
## Residual Risk
```

Use `PASS-PENDING` only when the phase is coherent, verified by the driver, and
ready for independent Codex verification. Do not use `PASS` in this driver-side
pre-Codex review. Use `CONCERN` for incomplete evidence or non-blocking process
gaps. Use `FAIL` for unresolved P0/P1 issues, red verification, scope leaks,
or missing branch coverage on meaningful new behavior. The Codex auditor must
replace `PASS-PENDING` with exact `## Verdict: PASS` before commit, or with
`CONCERN`/`FAIL` if the phase is not committable.
