# Base Agent Prompt

You are working in the `nexus-grid/SBFB` repository. This prompt is
vendor-neutral: Claude, GPT, and local LLMs must follow the same repository
process. Model-specific memory is never the source of truth; files, commands,
and written evidence are.

## Non-Negotiable Operating Rules

1. Read before editing. Start from `AGENTS.md`, `docs/agent/PROCESS.md`, the
   active `.planning/active/` files, and the exact files you will touch.
2. Preserve the dirty worktree. Never revert or overwrite changes you did not
   make unless the user explicitly asks for that operation.
3. Keep phase diffs atomic. A sprint phase commit should map to one phase goal,
   one review artifact, and one structured commit body.
4. Evidence beats confidence. Every material claim must cite a file path,
   command output, commit SHA, or explicit assumption.
5. Do not weaken SBFB invariants: canonical bytes, Ed25519 signatures, loopback
   trust, sandboxing, provenance, consent, PII handling, browser bridge
   boundaries, and wire compatibility.
6. Do not silently skip gates. If a gate is not applicable, write the reason in
   the planning artifact.
7. `PASS-PENDING` is only a temporary pre-Codex handoff. A phase commit needs
   Codex verification and a final review line exactly `## Verdict: PASS`.

## Required Context Load

Read or query these files before sprint work:

```text
AGENTS.md
CLAUDE.md
docs/agent/PROCESS.md
docs/agent/TOOLING.md
docs/claude/README.md
docs/claude/SPRINT_LOG.md
.planning/active/*
```

For implementation, also inspect the touched files and nearby tests:

```bash
git status --short
git diff --stat
git diff --cached --stat
rg -n "TODO|DEVIATION|scope-cut|threat-model|triggers_revalidate" .planning docs crates packages web
```

## Canonical Gate Map

- G1 Design Review Board: required before Sprint Phase A unless kickoff states
  `G1 skipped` with date and reason. Artifact:
  `.planning/active/sprint{N}_design_review.md`.
- G2 Long-life freshness: docs with `triggers_revalidate` must be checked when
  upstream releases, CVEs, or sprint drift could invalidate assumptions.
- G3 SMART goal: kickoff goals must point to measurable rows in
  `sprint{N}_verification.md` fail-fast checklist.
- G4 Rigor signal: an audit with 0 P0/P1 and 0 P2+ findings is normally
  `CONCERN`, not `PASS`, unless the review documents exhaustive negative
  evidence across all required dimensions.
- G5 Retired working-tree audit: upstream G5 was removed; keep the retained
  mechanics through `git status`, staged diff checks, and portable hooks.
- G6 Memory carry-over: unresolved P2+ findings must be manually carried into
  the next sprint memory/planning artifact, not auto-merged blindly.
- G7 Carry escalation: repeated carry-over for 3 reports requires escalation or
  a debt phase, especially on planned debt sprints.
- G8 Phase preflight: before the first code edit of a planned phase, run the
  five factual scans in `prompts/agent/preflight.md` and write
  `sprint{N}_phase_{X}_preflight.md` or a pivot proposal.
- G9 Factual research gate: before freezing D1-D5/Day-0 decisions, cite current
  stack evidence for inference, network, storage, crypto, security, protocol,
  process-isolation, browser-boundary, or dependency assumptions.
- G10 OSS prior art: preflight must check mature OSS approaches before inventing
  local mechanisms for compute, P2P, crypto, safety, storage, or UI workflows.

## Work Modes

### Sprint Phase Driver

Use this sequence:

```bash
python scripts/agent/agentctl.py prompt --kind preflight --sprint {N} --phase {X}
# write/read .planning/active/sprint{N}_phase_{X}_preflight.md
# implement
python scripts/agent/agentctl.py verify-on-write --file <changed-file>
python scripts/agent/agentctl.py prompt --kind phase-review --sprint {N} --phase {X}
python scripts/agent/agentctl.py prompt --kind phase-auditor --sprint {N} --phase {X}
python scripts/agent/agentctl.py precommit-lightcheck
```

Do not commit a phase until `.planning/active/sprint{N}_phase_{X}_review.md`
contains the exact final line `## Verdict: PASS` after Codex verification.
If the driver review is ready but Codex verification has not happened yet, use
`## Verdict: PASS-PENDING` and treat the phase as blocked for commit.

### Independent Reviewer

Use the staged diff and changed files as ground truth:

```bash
git diff --cached --name-only
git diff --cached --stat
git diff --cached -U0
```

If the driver summary conflicts with the diff, trust the diff and mark the
summary stale.

### Researcher

Do not edit code. Produce sources, tradeoffs, and direct repo references. If web
or package registry access is unavailable, state that the OSS/CVE scan is
blocked and mark the relevant decision `CONCERN` or `DESIGN-CONFLICT` rather
than guessing.

### Local LLM

Keep tasks narrow. Provide the assembled prompt, the changed files, the staged
diff, and the relevant planning artifacts. Require file-path citations in the
response. Do not ask a small local model to infer global architecture from
memory.

## Verification Standard

During implementation, scoped checks are acceptable for fast feedback. Before a
phase commit, the review prompt must require the complete three-block
verification unless the phase is docs-only and the exemption is explicit:

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

If a command cannot run because dependencies are missing or the environment is
offline, record the exact failure and the smallest next verification step.

Phase commit bodies must use exactly 9 markdown sections and include
`## Codex verification`. Do not reuse older 8-section templates or add
`Security delta` as a separate tenth header; record security delta inside
`## Codex verification`.

## Completion Report

Finish with:

- files changed;
- gates/artifacts written;
- commands run and results;
- P0/P1/P2/P3 residual findings;
- carry-over items for the next sprint if any;
- explicit note if no P2+ finding was found and why `PASS` is still justified.
