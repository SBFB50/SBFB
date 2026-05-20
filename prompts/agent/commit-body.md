# Commit Body Prompt

Draft a nexus-grid sprint phase commit message from facts only. Use the staged
diff, sprint plan, G8 artifact, review evidence, and command output. Stay
provider-neutral and ASCII-only. Do not claim tests, research, scope cuts, or
co-authorship that are not evidenced.

## Gather Facts

Use concrete repo paths and commands:

```bash
git status --short
git diff --cached --stat
git diff --cached --name-only
git diff --cached
rg -n "Phase {PHASE}|Commit cible|Tests plan|Scope cuts|Research consulte" .planning/active/sprint{SPRINT}_plan.md .planning/active/sprint{SPRINT}_kickoff.md
rg -n "Verdict|G8|S1a|Research|Scope" .planning/active/sprint{SPRINT}_phase_{PHASE}_preflight.md .planning/active/sprint{SPRINT}_phase_{PHASE}_pivot_proposal*.md
rg -n "Verdict|Findings|Delta|Branch Coverage" .planning/active/sprint{SPRINT}_phase_{PHASE}_review.md
```

If a fact is missing, write `Not evidenced` rather than filling the gap. The
mechanical gate artifact is
`.planning/active/sprint{SPRINT}_phase_{PHASE}_review.md`; cite that exact path
when referring to the final review. A final committable phase review must
contain the exact line `## Verdict: PASS`. `PASS-PENDING` is only a temporary
pre-Codex handoff and blocks commit.

## Subject

Use:

```text
<type>(sprint{SPRINT}): Sprint {SPRINT} Phase {PHASE} - <short result>
```

Choose only `feat`, `fix`, `docs`, `test`, `chore`, or `refactor` for a sprint
phase commit. This matches the portable phase-gate regex. Keep code identifiers
and commit titles in English. Avoid decorative punctuation so the subject
remains ASCII.

## Body Format

Use this complete 9-section markdown structure. Do not add a tenth top-level
section and do not reuse older 8-header templates:

```text
## Context
- <Why this phase exists; tie to sprint plan and phase goal.>

## Changes
- <path>: <specific role of the change, not just a filename>.
- <path>: <specific role of the change>.

## Tests
- Rust workspace: <before> -> <after> (+<delta> Phase {PHASE}) via <command or Not run: reason>.
- Python SDK: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Python coordinator: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Python app-gov: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Vitest unit: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Playwright: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Frontend build/size/i18n: <result> via <commands or Not run: reason>.

## G8 traceability
- Preflight or pivot: .planning/active/sprint{SPRINT}_phase_{PHASE}_<artifact>.md, verdict <verdict>.
- Review gate: .planning/active/sprint{SPRINT}_phase_{PHASE}_review.md, final verdict `## Verdict: PASS`.
- Research grounding: <sources from plan/preflight or Not evidenced>.

## Codex verification
- Codex pass: <session/agent and date, or Not run: commit blocked>.
- Final review: .planning/active/sprint{SPRINT}_phase_{PHASE}_review.md contains exactly `## Verdict: PASS`.
- Verification commands: <commands and result, or Not run with reason>.
- Security delta: <none, or exact security/protocol change and mitigation>.

## Pre-launch protocol
- Format/version impact: <none or exact VERSION/DOMAIN/canonical evidence>.
- Decoder/canonical policy: <preserved or explicit decision>.

## Scope cuts
- Honoured: <copy the relevant kickoff scope cuts checked against the diff>.
- Reopened: <none, or exact planning evidence and approval>.

## Carry closure / Unblock
- Closed carries: <ids and evidence>.
- New carries: <ids, owner, trigger, exit condition>.
- Unblocked items: <if any>.

## Risk
- <Residual risk, carry-over P2/P3, or "No unresolved P0/P1 in review gate.">
- <Security/protocol note if signing, provenance, sandbox, loopback, schemas, canonical bytes, deploy, or configs were touched.>
```

## Rules

- The body must mention every staged file or intentional file group. Do not
  hide generated, planning, lockfile, schema, deploy, or config changes.
- Delta tests must match current command output and the phase review. If a
  suite was not run, say why; do not imply the three-block verification passed.
- Scope cuts must come from `.planning/active/sprint{SPRINT}_kickoff.md` or the
  copied scope-cut section in the plan.
- G8 traceability is required for normal sprint phases. Missing G8 should be
  called out as a blocker, not papered over in the body.
- `## Codex verification`, `## Pre-launch protocol`, and
  `## Carry closure / Unblock` are required sections even when the answer is
  `none`; do not collapse them into generic risk text.
- `Security delta` is required inside `## Codex verification` even when it says
  `none`; do not add it as a separate top-level section.
- Co-Authored-By is conditional. Include it only for an actual co-author and
  exact configured identity; do not hard-code a vendor line in this neutral
  prompt, and do not count it as one of the 9 body sections.
- If the review gate verdict is not exactly `## Verdict: PASS`, draft the body
  as a proposed message but state that commit is blocked until Codex
  verification updates the final review file.
- Verify the sprint and phase labels match across subject, G8 artifact, review
  path, and body. A Phase B artifact cannot authorize a Phase C commit.
