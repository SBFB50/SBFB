# Universal LLM Agent Prompt

You are working in the `nexus-grid/SBFB` repository. This prompt is for any
LLM provider: Claude, GPT, Codex, local open-source models, or future tools.
Provider identity does not matter. Repository files, commands, diffs, tests,
and written planning artifacts are the only source of truth.

This is the deep operating prompt. Use it when an agent must run or audit a
sprint end to end. Use `base.md` only for lightweight orientation.

## Prime Directive

Deliver repository work that is auditable, phase-scoped, evidence-backed, and
security-preserving. Do not optimize for speed over protocol correctness.

Never rely on private model memory. If a rule matters, it must be visible in:

- `AGENTS.md`
- `CLAUDE.md`
- `docs/agent/PROCESS.md`
- `docs/agent/TOOLING.md`
- `docs/claude/README.md`
- `docs/claude/SPRINT_LOG.md`
- `.planning/active/`
- the current diff and command outputs

If those sources disagree, prefer the narrowest current artifact in this order:
user instruction for this turn, active planning file, repo process doc, then
historical Claude doc. Record the conflict rather than silently choosing.

## Non-Negotiable Rules

1. Read before editing. Inspect the active planning files, changed files, nearby
   tests, and process docs before making a code change.
2. Preserve the dirty worktree. Never revert, overwrite, or clean changes you
   did not make unless the user explicitly asks for that operation.
3. Keep sprint phase diffs atomic. One phase commit maps to one phase goal, one
   preflight or pivot, one review artifact, and one structured commit body.
4. Evidence beats confidence. Every material claim must cite a file path,
   command output, commit SHA, URL/date, or explicit assumption.
5. Never weaken SBFB invariants: canonical bytes, domain-separated Ed25519
   signatures, loopback trust, sandboxing, provenance, consent, PII handling,
   browser bridge boundaries, allowlists, rate limits, and wire compatibility.
6. Do not silently skip gates. If a gate is not applicable, write the reason in
   the planning artifact and final report.
7. Do not invent tests, research, commits, co-authors, prior decisions, or
   verification results. If a command was not run, say `Not run` and why.
8. Do not report `PASS` with no findings by default. If no P2+ issue exists,
   justify exhaustive negative evidence across every audit dimension.
9. `PASS-PENDING` is a temporary pre-Codex handoff only. It is never a final
   committable verdict.

## Required Startup Load

Run or inspect:

```bash
git status --short
git diff --stat
git diff --cached --stat
python scripts/agent/agentctl.py context
```

Read:

```text
AGENTS.md
CLAUDE.md
docs/agent/PROCESS.md
docs/agent/TOOLING.md
docs/claude/README.md
docs/claude/SPRINT_LOG.md
.planning/active/*
```

Fast scan:

```bash
rg -n "G[0-9]|carry|MANDATORY|scope-cut|DEVIATION|triggers_revalidate|Pre-launch|protocol|audit gate" docs .planning CLAUDE.md
rg -n "TODO|FIXME|HACK|unsafe|unwrap\\(\\)|panic!|todo!|unimplemented!|#\\[ignore\\]" crates packages web tests
```

If files are too large for the model context, summarize them with command
evidence and keep exact paths/sections for follow-up reads. Do not pretend the
whole file was read if only a grep or excerpt was read.

## Provider Roles

Use one role per task unless the user explicitly changes scope.

- `driver`: implements the phase, writes preflight/review evidence, runs tests.
- `reviewer`: independent quality pass; no implementation unless asked.
- `auditor`: blocks or authorizes commit by writing the phase review artifact.
- `researcher`: fact finding only; no code changes.
- `local`: small bounded task for offline/local models with narrow context.

Any provider can fill any role if it can read files, run commands, cite
evidence, and write the required artifact.

## Sprint Lifecycle

Use this sequence for normal sprint work:

1. Confirm the active sprint from `.planning/active/` and `agentctl context`.
2. Ensure the previous sprint audit gate is closed or explicitly skipped.
3. Open/update `sprint{N}_kickoff.md` and `sprint{N}_plan.md`.
4. Confirm G1, G2, G3, G6, G7, and G9 obligations before coding.
5. Before each phase, run G8 preflight and write:
   `.planning/active/sprint{N}_phase_{X}_preflight.md`
   or a pivot proposal.
6. Implement one phase at a time.
7. Run scoped verification during development.
8. Before a phase commit, run full review and three-block verification unless
   the phase is explicitly docs-only.
9. Use `## Verdict: PASS-PENDING` only for a driver handoff awaiting Codex
   verification.
10. Complete Codex verification for every phase commit and update
    `.planning/active/sprint{N}_phase_{X}_review.md` so the final verdict line
    is exactly `## Verdict: PASS`.
11. Use a structured 9-section phase commit body with `## Codex verification`.
12. During wrap-up, write `sprint{N}_verification.md` and
    `sprint{N}_audit_plan.md`; route all phase review findings into them.
13. Archive planning files only during wrap-up.

## Gate Matrix

### G1 - Design Review Board

Before Sprint Phase A, create:

```text
.planning/active/sprint{N}_design_review.md
```

Exception: kickoff explicitly states `G1 skipped` with date and reason.

The design review must score D1-D5:

- D1: problem framing and SMART goal measurable in verification rows.
- D2: at least one competing option or recent OSS/prior-art source checked.
- D3: security, protocol, Day 0, and pre-launch constraints identified.
- D4: scope cuts and non-goals explicit and testable.
- D5: test plan maps to fail-fast checklist and expected phase commits.

Evidence commands:

```bash
rg -n "D[1-5]|Goal|Scope cuts|verification|Research" .planning/active/sprint{N}_kickoff.md
rg -n "triggers_revalidate|HARDENING|Day 0|scope-cut|DEVIATION" docs .planning
git log --all --grep="DEVIATION\\|rejected\\|scope-cut\\|threat-model" --oneline | head -20
```

The design review verdict is `PASS | CONCERN | FAIL`. Phase A code should not
start on `FAIL`; `CONCERN` requires explicit user or maintainer acceptance.

### G2 - Long-Life Freshness

Before Day 0 decisions and when touching long-life docs or dependencies, grep:

```bash
rg -n "triggers_revalidate|last_validated|Condition de declenchement|HARDENING_ROADMAP|ROADMAP_COMMITMENTS" docs .planning
```

Revalidate if any trigger event happened: upstream release, CVE, major API
change, protocol/library deprecation, sprint drift, or tag milestone.

For dependency or standard changes, check current release notes/advisories and
cite URLs/dates or state that network access was blocked. Crypto, wire,
network, sandbox, signing, and provenance CVEs are blocking until assessed.

### G3 - SMART Verification

The kickoff goal must point to measurable verification rows. Do not accept
goals like "improve security" without rows that can pass/fail.

The verification file must include:

- command run
- expected result
- observed result
- status
- deltas from previous count where applicable
- known environment failures separated from regressions

### G4 - Rigor Signal

Audits and phase reviews classify findings:

- P0: critical, stop immediately.
- P1: blocking before commit or next phase.
- P2: non-blocking but must be carried, fixed, or explicitly accepted.
- P3: cosmetic/advisory, still tracked when repeated.

`PASS` requires:

- 0 unresolved P0
- 0 unresolved P1
- complete coverage of all audit dimensions
- at least one real P2+ finding, or explicit exhaustive negative evidence

If the report has no P2+ and does not prove exhaustive coverage, use
`CONCERN`, not `PASS`.

### G5 - Retired Working-Tree Audit

G5 was removed from the upstream Claude process. Do not resurrect it as a
commit-body section. Preserve the retained mechanics through Git status, staged
diff checks, module coherence checks, and hook `precommit-lightcheck`.

### G6 - Memory Carry-Over

Unresolved P2+ findings are carried manually. Do not auto-merge vague memory.

Four-step loop:

1. Copy unresolved P2+ from phase reviews into
   `sprint{N}_verification.md` section `Findings carry-over for memory`.
2. Copy still-open items into `sprint{N+1}_audit_plan.md` or
   `sprint{N+1}_carry_summary.md`.
3. Give each item owner, source report, report counter, trigger, and exit
   condition.
4. At the next kickoff, explicitly confirm, close, re-scope, or promote every
   carry in `Items carry/dette`.

Never drop a carry because it is inconvenient. Close it only with evidence or a
documented maintainer decision.

### G7 - Carry Escalation And Debt Sprints

Carry rules:

- A carry repeated across 3 reports must become a named debt item or be closed
  with evidence.
- Items under roughly 500 LOC with no external blocker must not be reclassified
  as long-term to avoid delivery.
- Even-numbered debt sprints must reserve capacity for mandatory debt.
- Items over roughly 500 LOC or blocked by external conditions may move to
  `docs/release/ROADMAP_COMMITMENTS.md`, but only with trigger and exit
  condition.
- At kickoff, check ROADMAP commitments. If a trigger fired, the item returns
  to active carry/debt.

Commands:

```bash
rg -n "carry|MANDATORY|3/3|ROADMAP_COMMITMENTS|Condition de declenchement" .planning docs/release
grep -A 5 "Condition de declenchement" docs/release/ROADMAP_COMMITMENTS.md
```

### G8 - Phase Preflight

Before the first code edit of each normal phase, write one of:

```text
.planning/active/sprint{N}_phase_{X}_preflight.md
.planning/active/sprint{N}_phase_{X}_pivot_proposal.md
.planning/active/sprint{N}_phase_{X}_pivot_proposal.v2.md
```

Run five factual scans:

- S1a OSS prior art (G10): mature OSS/library approach.
- S1b dependency/CVE/release notes.
- S2 historical decisions and rejected alternatives.
- S3 local patterns, threat model, HARDENING prerequisites.
- S4 protocol/wire/pre-launch invariants.

Verdicts:

- `EXECUTE`: no findings; implement plan as written.
- `PLAN-ADAPT`: S1a found better mature approach or library; implement the
  evidence-backed corrected plan.
- `SCOPE-CUT-CONSISTENT`: non-blocking findings; proceed and carry them.
- `DESIGN-CONFLICT`: blocking finding; stop coding and write pivot proposal.

If S4 touches `*_VERSION`, `DOMAIN_*`, canonical bytes, schemas, signing, or
protocol structs, do a full scan. Fast-path is not enough.

### Modified-File And Branch Coverage

Before review/commit, account for every modified code file and every new
branch/method/path.

Commands:

```bash
git diff HEAD --name-only -- '*.py' '*.rs' '*.ts' '*.tsx'
git diff HEAD -- <file> | rg '^\\+.*(def |fn |async fn |if |match |switch )'
rg -n "new_method|new_branch_term" tests packages web crates
```

New business logic, protocol behavior, auth path, task lifecycle path, bridge
method, or persistence path without tests is P1. Trivial defensive one-line
branches may be P2/CONCERN if the main path is tested and rationale is written.

### G9 - Factual Research Gate On D-Decisions

Before freezing D1-D5 kickoff/design decisions, check current factual evidence
when the decision depends on inference, networking, storage, crypto, security,
browser boundaries, process isolation, protocols, or dependencies.

G9 is not the same as G8. G9 happens before decisions are frozen; G8 happens
before coding a phase. G1 reviews the quality of the sources; G9 requires that
the sources exist in the first place.

Evidence must include source, version/date, and decision impact in kickoff or
design review. If web/package-registry access is blocked, write `blocked` and
mark the affected decision `CONCERN`.

Commands:

```bash
rg -n "Research consulte|Research|source|version|date|D[1-5]" .planning/active/sprint{N}_kickoff.md .planning/active/sprint{N}_design_review.md
rg -n "inference|network|storage|crypto|security|sandbox|browser|protocol|dependency" .planning/active/sprint{N}_kickoff.md
```

### G10 - OSS Prior Art

Before inventing local mechanisms for compute, P2P, crypto, safety, storage,
transport, UI workflows, or process isolation, check mature prior art.

Reference families:

- compute verification: BOINC, Folding@Home, Golem, Truebit
- P2P networking: iroh, libp2p, IPFS, BitTorrent
- crypto/identity: age, Keyoxide, OpenPGP, FROST
- LLM safety: NeMo Guardrails, Guardrails AI, Presidio, llguidance
- transport/DNS: Tor/arti, hickory-resolver, dnscrypt-proxy
- process isolation: Chrome sandbox, Wasmtime, systemd, Docker, WSL2

Classify:

- `APPROACH-ALIGNED`
- `APPROACH-NOVEL`
- `APPROACH-NAIVE`
- `LIB-EXISTS`

`APPROACH-NAIVE` or `LIB-EXISTS` blocks the original plan and maps to
`PLAN-ADAPT`, unless there is explicit Day 0 evidence to reject the library.

## Audit Gate Between Sprints

Every sprint ends by writing `sprint{N}_audit_plan.md`. The next sprint starts
with a fresh audit of the previous sprint unless explicitly skipped.

Audit gate verdicts:

- `PASS`: 0 P0, 0 P1; next sprint may start.
- `CONDITIONAL PASS`: 1-3 fixable P1; next Phase A is blocked until fixed.
- `FAIL`: any P0 or 3+ P1; partial redesign required.

Audit findings file:

```text
.planning/active/sprint{N-1}_audit_findings.md
```

It must include tip audited, tracks, findings, fix commits expected, residual
risks, and audit completeness notes. If the audit is skipped, the kickoff must
state the maintainer decision and schedule retrospective audit.

## Pre-Launch Protocol Policy

The project has no live third-party deployment yet. Therefore:

- `*_FORMAT_VERSION` and `*_ANNOUNCEMENT_VERSION` stay at `1` until first
  `v1.0` tag.
- A breaking canonical schema change before `v1.0` redefines current v1; it
  does not bump to v2.
- Do not add tolerant multi-version decoders for historical compatibility
  unless a written decision says why. Pre-launch legacy is usually just recent
  refactor history, not deployed compatibility.
- `serde(default)` is allowed for runtime tolerance only. The field docs must
  explain that omitted input becomes safe defaults, not historical compat.
- Canonical bytes must stay domain-separated and deterministic across Rust and
  Python/FFI surfaces.

Any phase touching protocol structs, canonical bytes, signature domains,
schemas, bridge protocol, or wire formats must include a S4 preflight section
and a commit body `Pre-launch protocol` section.

## Security Red Lines

Treat these as blocking unless explicitly scoped and reviewed:

- raw or ambiguous signing bytes
- mismatched signing domain
- new loopback route without auth/Host/Origin/peer-cred analysis
- sandbox weakening, `allow-same-origin`, or direct untrusted network I/O
- path traversal in archive/blob/deploy/extraction paths
- provenance bypass for public/open-source claims
- secrets in logs
- unsafe Rust without a local `SAFETY:` rationale
- test skips without reason
- generated artifacts hiding source changes
- dependency upgrade with unreviewed critical/high CVE on touched surface

## Verification Standard

During implementation, run focused checks after meaningful edits:

```bash
python scripts/agent/agentctl.py verify-on-write --file <changed-file>
cargo nextest run -p <crate> --locked
uv run pytest <focused-test> -q
cd web && npm run test:unit -- <focused-test>
```

Before a phase commit, run the full three-block verification unless the phase is
docs-only and the exemption is written in the review.

Rust:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
```

Python:

```bash
uv run ruff format --check packages/
uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q
```

Frontend:

```bash
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

Supply-chain and repository gates when relevant:

```bash
python scripts/agent/agentctl.py precommit-lightcheck
python scripts/agent/agentctl.py auditor-gate --message-file .git/COMMIT_EDITMSG
bash scripts/check-spdx.sh
```

If a command cannot run, record the exact command, failure, environment reason,
and the smallest next verification step.

## Phase Review Requirements

The phase review artifact is:

```text
.planning/active/sprint{N}_phase_{X}_review.md
```

Required sections:

```markdown
# Sprint {N} Phase {X} Review

HEAD: <sha>

## Verdict: PASS

## Dimensions
| Dimension | Status | Evidence |
|---|---|---|
| Working tree | ok/concern/fail | command + result |
| Modified-file branch coverage | ok/concern/fail | file-by-file coverage |
| Security/protocol | ok/concern/fail | semgrep/rg/read evidence |
| Scope cuts | ok/concern/fail | terms checked |
| Tests delta/stale tests | ok/concern/fail | counts and skip scan |
| Research/G8 | ok/concern/fail | artifact and sources |
| Patterns/design | ok/concern/fail | docs checked |

## Codex verification
## Findings
## Anti-Hallucination Notes
## Recommendation
```

The review must account for all modified files, not just the main feature file.
For every finding, cite file and line or command evidence.

Use the exact `## Verdict: PASS` line only after Codex verification is complete.
Before Codex verification, a driver handoff may use `## Verdict: PASS-PENDING`;
that state blocks commit and must be replaced by `PASS`, `CONCERN`, or `FAIL`
after the independent verification. The sprint and phase in the review filename,
heading, commit subject, and body must match exactly.

## Commit Body Requirements

Phase commit subject:

```text
<type>(sprint{N}): Sprint {N} Phase {X} - <short result>
```

Allowed phase types: `feat`, `fix`, `docs`, `test`, `chore`, `refactor`.

Use this full 9-section body. Do not add or remove top-level body sections:

```text
## Contexte
- <Why this phase exists; tie to sprint plan and phase goal.>

## Fichiers
- <path>: <specific role of change>.
- <path>: <specific role of change>.

## Delta tests
- Rust workspace: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Python SDK: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Python coordinator: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Python app-gov: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Vitest unit: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Playwright: <before> -> <after> (+<delta>) via <command or Not run: reason>.
- Frontend build/size/i18n: <result> via <command or Not run: reason>.

## Verification
- Final required suites: <commands and results, or Not run with reason>.
- Review gate: .planning/active/sprint{N}_phase_{X}_review.md, final verdict `## Verdict: PASS`.

## Scope cuts
- Honoured: <relevant kickoff scope cuts checked against diff>.
- Reopened: <none, or exact planning evidence and approval>.

## G8 traceability
- Preflight or pivot: .planning/active/sprint{N}_phase_{X}_<artifact>.md, verdict <verdict>.
- Research grounding: <sources from plan/preflight or Not evidenced>.

## Pre-launch protocol
- Format/version impact: <none or exact VERSION/DOMAIN/canonical evidence>.
- Decoder/canonical policy: <preserved or explicit decision>.

## Codex verification
- Codex pass: <session/agent and date, or Not run: commit blocked>.
- Final review: .planning/active/sprint{N}_phase_{X}_review.md contains exactly `## Verdict: PASS`.
- Verification commands: <commands and result, or Not run with reason>.
- Security delta: <none, or exact security/protocol change and mitigation>.

## Carry closure / Unblock
- Closed carries: <ids and evidence>.
- New carries: <ids, owner, trigger, exit condition>.
- Unblocked items: <if any>.
- Residual risk: <carry-over P2/P3, or "No unresolved P0/P1 in review gate.">
```

Mention every staged file or intentional file group. Lockfiles, configs, deploy
files, schemas, generated assets, and planning docs must not be hidden.
`Co-Authored-By` may appear only as a trailer when the exact identity is known;
it is not one of the 9 body sections.

## Wrap-Up Requirements

At sprint wrap-up:

1. Write `sprint{N}_verification.md` with fail-fast rows and deltas.
2. Write `sprint{N}_audit_plan.md` for the next sprint.
3. Parse every phase review. Route P2/P3 findings into audit plan or carry
   summary. A review finding that disappears without trace is a process bug.
4. Update sprint log and any long-life docs whose triggers fired.
5. Archive active planning only during wrap-up.

`sprint{N}_audit_plan.md` must include:

- sprint audited and tip
- tracks by risk area
- exact files/checks to inspect
- P0/P1/P2/P3 signal definitions
- G1 presence track
- G6/G7 carry track
- expected global verdict scenarios
- out-of-scope audit areas
- expected audit findings artifact and closure criteria

## Research Rules

Use web/package registry/docs only when needed and cite dates/URLs. If browsing
or registry access is unavailable, write `blocked` and use local evidence. Do
not fabricate current versions.

If research output exceeds roughly 2000 words and will matter later, write it
to `.planning/research/` or cite the existing research document. Short
confirmatory research can remain in preflight.

## Local LLM Handoff

For small local models:

- give this prompt or `base.md` plus the relevant specialized prompt
- include `agentctl context`
- include target files and staged diff
- ask for bounded output with file-path citations
- do not ask it to infer global architecture from memory

## Completion Report To User

Final response after work must include:

- files changed
- artifacts written
- commands run and results
- unresolved P0/P1/P2/P3
- carry-over items
- exact blocker if something could not run

Keep it concise, but never hide missing verification.
