# Handoff Prompt

Transfer sprint context between providers or sessions. This prompt produces a
structured point-in-time snapshot so the receiving agent can resume work without
inheriting stale chat memory or unverified assumptions.

The handoff is not an authority. It is a navigational aid that points to repo
files. If the handoff says X but the repo says Y, the repo wins (Truth Stack
rank 1 > rank 5).

Stay provider-neutral. Do not reference Claude-specific tools, memory syntax,
or subagent APIs. ASCII only.

## When To Use

- Switching from one LLM provider to another mid-sprint.
- Resuming work in a fresh session after context loss.
- Handing a phase from driver to reviewer or auditor.
- Transferring between a cloud provider and a local model.
- Creating a context-pack for `sbfb-factory operator serve`.

## Gather Facts

```bash
git rev-parse --short HEAD
git status --short
git log --oneline -10
git diff --stat
git diff --cached --stat
ls .planning/active/
rg -n "Phase|Verdict|carry|scope.cut|Day 0" .planning/active/sprint*_plan.md .planning/active/sprint*_phase_*_review.md .planning/active/sprint*_phase_*_preflight.md
rg -n "Sprint|Phase|OPEN|CLOSED" CLAUDE.md | head -10
```

Do not invent facts. If a command fails or a file is missing, write
`Not evidenced` for that section.

## Output Structure

Write the handoff as a markdown document with exactly 9 sections. Each section
must cite at least one repo file path or command output.

```markdown
# Handoff — Sprint {N} Phase {X}

Date: YYYY-MM-DD
HEAD: `<sha>`
Provider source: <provider producing this handoff>
Provider target: <intended receiver or "any">

## 1. Sprint context

- Sprint: {N}
- Goal: <one-line from kickoff>
- Day 0 decisions: D1 <summary>, D2 <summary>, ...
- Plan: `.planning/active/sprint{N}_plan.md`
- Kickoff: `.planning/active/sprint{N}_kickoff.md`

## 2. Progress

- Phases completed: <list with commit SHAs>
- Current phase: {X} — <status: not started / in progress / review pending / blocked>
- Tip commit: `<sha>` `<subject>`
- Working tree: clean | dirty (<file count>)

## 3. Changed files

Files touched in the current phase or since the last handoff:

- `<path>`: <role of the change>
- `<path>`: <role of the change>

Evidence: `git diff --name-only <base>..<tip>` or `git status --short`.

## 4. Test evidence

| Suite | Before | After | Delta | Command | Last run |
|-------|--------|-------|-------|---------|----------|
| Rust nextest | <N> | <N> | +<N> | `cargo nextest run --workspace --locked` | <date or Not run> |
| Rust doctests | ok | ok | +0 | `cargo test --workspace --locked --doc` | <date or Not run> |
| Vitest | <N> | <N> | +<N> | `(cd web && npm run test:unit)` | <date or Not run> |
| size-limit | 6/6 | 6/6 | +0 | `(cd web && npm run size)` | <date or Not run> |

## 5. Verdict state

| Gate | File | Verdict | Notes |
|------|------|---------|-------|
| Preflight G8 | `sprint{N}_phase_{X}_preflight.md` | <verdict or missing> | |
| Review | `sprint{N}_phase_{X}_review.md` | <verdict or missing> | |
| Codex | `sprint{N}_phase_{X}_codex_review.md` | <verdict or missing> | |

## 6. Stop conditions

The receiving agent must stop and hand back or escalate if:

- A Day 0 decision (D1-D5) needs to be changed.
- A `DESIGN-CONFLICT` verdict is encountered.
- The scope drifts beyond the phase plan section.
- A P0 or P1 finding cannot be resolved within the phase.
- The working tree diverges from the plan by more than 3 unplanned files.

## 7. Assumptions NOT to inherit

List claims from the sending session that the receiver must re-verify rather
than trust:

- <assumption 1>: re-verify by <command or file check>
- <assumption 2>: re-verify by <command or file check>

If the sender has no unverified assumptions, write "None — all claims are
evidenced in repo files listed above."

## 8. Active carries

| ID | Description | Owner | Trigger | Exit condition | Reports |
|----|-------------|-------|---------|----------------|---------|
| <id> | <description> | <owner> | <trigger> | <exit condition> | <count> |

Source: `.planning/active/sprint{N}_plan.md` scope cuts and kickoff carries.

## 9. Next actions

1. <concrete next step with file path or command>
2. <concrete next step>
3. <concrete next step>

Expected commit: `<type>(scope): Sprint {N} Phase {X} — <title>`
```

## Rules

- Every section must reference at least one repo path or command output.
- `chat_history_authoritative: false` — the receiver must not treat chat history
  from the sender as ground truth.
- If the handoff is produced for a context-pack (`sbfb-factory operator serve`),
  include `private chat history is non-authoritative` verbatim.
- Do not embed full file contents. Point to paths; let the receiver read them.
- Do not speculate about future phases beyond the next 1-3 actions.
- Carry items must include report count for G7 escalation tracking.
