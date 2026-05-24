# Vendor-Neutral Agent Process

This document is the provider-neutral source of truth for agent work in this
repository. Claude, GPT, and local LLMs must follow the same process: the model
may change, but the repo contracts, checks, and evidence do not.

## Objectives

- Keep sprint delivery traceable from kickoff to commit.
- Preserve SBFB security and protocol invariants before speed.
- Make agent output auditable through files in `.planning/active/`.
- Avoid vendor lock-in: prompts, hooks, and gates live in the repository.
- Preserve the Claude process guarantees G1-G10 while removing the dependency on
  Claude-specific prompt, hook, and subagent syntax.

## Provider Roles

- `driver`: primary implementation agent. Edits files, runs focused tests, writes
  the phase evidence.
- `reviewer`: independent quality pass. Checks security, scope, stale tests,
  and protocol drift.
- `researcher`: fact-finding only. Produces references and tradeoffs, no code.
- `local`: offline or open-source fallback. Use for bounded edits and reviews
  after the prompt context has been assembled by `agentctl`.

Any provider can fill any role if it can read the repo, run commands, and write
the required planning artifact.

## Sprint Workflow

1. Open or update `.planning/active/sprint{N}_kickoff.md`.
2. Confirm G1, G2, G3, G6, G7, and G9 kickoff obligations when applicable.
3. Run G8 preflight before coding:
   `python scripts/agent/agentctl.py prompt --kind preflight --sprint N --phase A`.
4. Implement one phase at a time. Keep diffs small enough to review.
5. Run scoped verification after each meaningful edit.
6. Before commit, run the review prompt and complete three-block verification
   unless the phase is explicitly docs-only.
7. Treat `PASS-PENDING` as a temporary driver handoff only. It means "ready for
   independent Codex verification", not "ready to commit".
8. Before every phase commit, complete Codex verification and update
   `.planning/active/sprint{N}_phase_{X}_review.md` so the final verdict line is
   exactly `## Verdict: PASS`.
9. Commit with `feat(sprintN): Sprint N Phase X ...` or another standard type
   and a 9-section body containing `## Codex verification`.
10. Archive planning files only during wrap-up.

## Guarantee Matrix

| Gate | Preserved By | Required Artifact Or Evidence |
| --- | --- | --- |
| G1 Design Review Board | `precommit-lightcheck` Phase A gate + prompt discipline | `sprint{N}_design_review.md` or explicit `G1 skipped` |
| G2 triggers_revalidate | base/preflight prompt | grep of `triggers_revalidate` docs and impacted long-life artifacts |
| G3 SMART verification | kickoff/process prompt | kickoff goal references to `verification.md` rows |
| G4 rigor signal | phase-review and phase-auditor prompts | PASS requires 0 P0/P1 and at least one real P2+ or exhaustive negative evidence |
| G5 retired working-tree audit | Git hooks + lightcheck | G5 was removed upstream; staged/module/commit checks cover the retained mechanics |
| G6 memory carry-over | review/process prompt | unresolved P2+ copied into next planning memory/carry section |
| G7 carry escalation | process prompt | 3-report carry counter and debt-phase decision |
| G8 phase preflight | preflight prompt + auditor traceability | `sprint{N}_phase_{X}_preflight.md` or pivot proposal |
| G9 factual research gate | kickoff/process prompt | D-decisions cite recent stack/current-source evidence before being frozen |
| G10 OSS prior art | preflight prompt | S1a OSS prior art section with sources or blocked status |

## Gates

`scripts/agent/agentctl.py` provides the portable gates:

- `verify-on-write --file PATH`: scoped Rust, Python, or frontend lint.
- `precommit-lightcheck`: staged Rust module coherence, LOC-plan guard, wire
  format warnings, and Phase A design-review gate.
- `auditor-gate --message-file .git/COMMIT_EDITMSG`: blocks phase commits unless
  the matching final phase review exists with `## Verdict: PASS`.
- `prompt --kind KIND`: assembles model-ready instructions from repo files.

Claude hooks may still call their native shell scripts, but Git hooks and other
agents should use `agentctl`.

Enable the portable hooks once per clone:

```bash
git config core.hooksPath .githooks
```

`python scripts/agent/agentctl.py install-hooks` prints the same command for
operators that want a discoverable CLI entry point.

## G1 Design Review Production

Before Sprint Phase A, produce `.planning/active/sprint{N}_design_review.md`
unless the kickoff explicitly says `G1 skipped` with date and reason. The
review must be independent from the sprint author and score D1-D5:

- D1: problem framing and SMART goal are measurable in `verification.md`;
- D2: at least one competing option or recent OSS/prior-art source was checked;
- D3: security, protocol, and Day 0 constraints are identified;
- D4: scope cuts and non-goals are explicit and testable;
- D5: test plan maps to the fail-fast checklist and expected phase commits.

Use this minimal command set as evidence:

```bash
rg -n "D[1-5]|Goal|Scope cuts|verification|Research" .planning/active/sprint{N}_kickoff.md
rg -n "triggers_revalidate|HARDENING|Day 0|scope-cut|DEVIATION" docs .planning
git log --all --grep="DEVIATION\|rejected\|scope-cut\|threat-model" --oneline | head -20
```

The artifact must end with `## Verdict: PASS | CONCERN | FAIL`. Phase A code
should not start on `FAIL`; `CONCERN` needs explicit user or maintainer
acceptance.

## G6/G7 Carry Discipline

At phase review and sprint wrap-up, unresolved P2+ findings are carried
manually, not auto-merged. Use this four-step loop:

1. Copy unresolved P2+ from `sprint{N}_phase_*_review.md` into
   `sprint{N}_verification.md` carry-over section.
2. Copy still-open items into `sprint{N+1}_audit_plan.md` or the next active
   carry summary with owner, trigger, and exit condition.
3. If the same carry appears in three reports, escalate it to a named debt
   phase or document why it is closed.
4. On even-numbered debt sprints, check `docs/release/ROADMAP_COMMITMENTS.md`
    and close, re-scope, or re-carry each long-horizon item explicitly.

## G9 Factual Research Gate

Before freezing D1-D5 decisions in kickoff or design review, verify assumptions
against current local or external evidence when the decision touches inference,
networking, storage, crypto, security, process isolation, browser boundaries, or
new dependencies. This is separate from G8: G9 protects Day 0 decisions before
they become the plan; G8 re-checks the phase immediately before coding.

The kickoff or design review must include `Research consulte` or equivalent
evidence with source, version/date, and impact. If the provider cannot access
the web or registry, record `blocked` and mark the affected decision as
`CONCERN` instead of inventing current facts.

## Prompt Contract

8 portable prompts live in `prompts/agent/`. They can be assembled by
`sbfb-factory process prompt --kind {kind}` (Rust) or
`agentctl.py prompt --kind {kind}` (Python legacy).

| Kind | File | Purpose |
|------|------|---------|
| `base` | `base.md` | Short orientation and invariants |
| `universal` | `universal.md` | Complete sprint process vendor-neutral |
| `handoff` | `handoff.md` | Inter-provider transfer (9 sections) |
| `preflight` | `preflight.md` | G8 pre-code: 5 scans S1-S4, verdict tree |
| `phase-review` | `phase-review.md` | Post-code: 11 dimensions review |
| `phase-auditor` | `phase-auditor.md` | Independent audit: 7 dimensions |
| `commit-body` | `commit-body.md` | 9-section commit body with validation |
| `audit-gate` | `audit-gate-checks.md` | 9 tracks sprint audit, P0-P3 |

Aliases: `review` -> `phase-review`, `auditor` -> `phase-auditor`,
`audit` -> `audit-gate`.

Prompts must reference files and commands, not private model memory.

Prompt depth is intentional. A vendor-neutral prompt may be longer than a
minimal chat instruction because it replaces Claude-specific skills and memory.
Use concise wording, but keep the operational checks executable.

### Bootstrap Fresh Session

A fresh session receives context in this order:

1. `base.md` — orientation invariante, regles evidence
2. `universal.md` — lifecycle sprint complet, gates G1-G10
3. `sbfb-factory process context` — faits repo live (HEAD, branch, dirty files,
   sprint, phase, active artifacts, AGENT_SYSTEM.md)
4. `handoff.md` — etat point-in-time (phase courante, verdict state, carries,
   next actions)
5. Prompt specialise — prochaine action de gate (preflight, phase-review,
   commit-body, audit-gate, phase-auditor)

Private chat memory is non-authoritative. If a fact is not in the repo files,
runtime context, or handoff, the receiving agent writes `Not evidenced`.

## Codex Runbook

Codex-specific execution discipline lives in `docs/agent/codex-process/`. It is
not a separate source of truth; it is a stricter runbook layered on this
document, `TOOLING.md`, `agentctl.py`, and `.planning/active/`. Use it when a
Codex session needs Claude-level process parity: session start, phase driving,
review/audit, commit gate, domain smoke matrices, and automation backlog.

## Phase Commit Finality

Phase commits have a stricter finality rule than intermediate reviews:

- `PASS-PENDING` is allowed only before Codex verification. It is never a final
  committable verdict.
- The committable final review line is exactly `## Verdict: PASS`. Do not use
  `## Verdict : PASS`, `CONDITIONAL PASS`, or prose equivalents for a phase
  commit gate.
- `CONCERN` and `FAIL` block the phase commit until resolved or explicitly
  reworked into a new reviewed state.
- Codex verification is mandatory for phase commits, including docs-only phase
  commits. Docs-only phases may exempt heavy test suites only when the final
  review and commit body record the reason.
- Sprint and phase identity must match across the preflight/pivot, review,
  commit title, and commit body. Never reuse Phase B evidence for Phase C, or
  any other mismatched phase pair.
- The phase commit body has exactly these 9 markdown sections:
  `## Contexte`, `## Fichiers`, `## Delta tests`, `## Verification`,
  `## Scope cuts`, `## G8 traceability`, `## Pre-launch protocol`,
  `## Codex verification`, and `## Carry closure` or
  `## Carry closure / Unblock`.
  `Security delta` belongs inside `## Codex verification`; do not add it as a
  tenth top-level section.

## Quality Bar

During coding, run the smallest relevant test suite for fast feedback. Before a
phase commit, run the three full verification blocks (Rust, Python, frontend)
unless the phase is docs-only and the exemption is written in the review. Then
complete Codex verification and keep its evidence in the final review and
`## Codex verification` commit-body section. For protocol or security surfaces,
include explicit checks for canonical bytes, signing, loopback trust, sandbox
boundaries, provenance, consent, PII, and browser bridge invariants.

Do not report `PASS` with no findings by default. If no P2+ issue is found, the
review must explain why every audit dimension was exhaustively covered;
otherwise use `CONCERN`.

## Provider Switching

To switch from Claude to GPT or a local LLM, keep the same files and run:

```bash
# Rust (preferred)
cargo run -p sbfb-factory -- process context
cargo run -p sbfb-factory -- process prompt --kind universal --depth deep
cargo run -p sbfb-factory -- process prompt --kind preflight --depth deep

# Python legacy (still supported)
python scripts/agent/agentctl.py context
python scripts/agent/agentctl.py prompt --kind universal --depth deep
python scripts/agent/agentctl.py prompt --kind preflight --sprint 35 --phase A
```

For local/offline providers, use `--provider local` to strip cloud-specific
references (WebSearch, context7):

```bash
cargo run -p sbfb-factory -- process prompt --kind preflight --provider local --depth deep
```

Paste the assembled prompt into the selected model, then require the model to
write the same `.planning/active/` artifacts before commit.
