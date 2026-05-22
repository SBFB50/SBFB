# S70 Process Portable Complete - intake

**Date:** 2026-05-22
**Status:** live intake for S69 Phase E and S70 kickoff, not an active sprint plan
**Owner:** S69 Phase E wrap-up / S70 kickoff

## 1. Decision

S70 must make the Nexus agent process fully portable before RRV total, Factory
process packaging, or broad `@dev` source indexing.

The goal is not to replace the current Claude process in one unsafe jump. The
goal is to turn the existing process into a repo-native operating contract that
Claude, Codex, GPT, local LLMs, humans, RRV and later Factory can all consume.

S70 therefore becomes:

```text
Process Portable Complete
+ Gate 1 dogfood as verification surface
+ RRV/Factory contract as consumer contract
```

Not:

```text
RRV total
Factory process UI
SearchManifest
remote/private compute
large OSS ingestion
```

## 1.1 Promotion status

- Promoted by: pending `.planning/active/sprint69_verification.md`.
- Active execution plan: pending `.planning/active/sprint70_audit_plan.md`.
- Research-only after promotion: yes. Once S69 Phase E writes the active audit
  plan, this file becomes an input reference, not an executable plan.

## 2. Why this file exists

Recent research created many correct but competing directions:

- Factory creates app templates and publish evidence.
- Babel is the first canary app created through Factory.
- RRV should expose `@protocole`, then later `@dev`, `@web`, and proof-centric
  question flows.
- An app can be verified without being created by Factory if it publishes the
  same evidence pack.
- RRV can use local or central LLMs, but only as answer composers over bounded
  evidence.
- The current sprint process already works, but its deepest implementation is
  still Claude-centric while the portable layer is thinner.

Without a S70 process sprint, the next roadmap step can drift into either a
theoretical RRV or a Factory UI that cannot package the actual process.

## 3. Truth stack

S70 kickoff should read these in order:

1. `.planning/active/sprint69_verification.md` and
   `.planning/active/sprint70_audit_plan.md` once S69 Phase E writes them.
2. `docs/agent/PROCESS.md` and `docs/agent/TOOLING.md`.
3. `prompts/agent/universal.md`, `preflight.md`, `phase-review.md`,
   `phase-auditor.md`, `commit-body.md`.
4. `scripts/agent/agentctl.py`, `.githooks/`, `.claude/hooks/`.
5. `CLAUDE.md` and `docs/claude/README.md`.
6. `.planning/research/rrv_sprint_intake_s70.md`.
7. `.planning/research/rrv_llm_runtime_and_app_boundary.md`.
8. `.planning/research/SYNTHESIS_factory_rrv_protocol.md`.
9. `.planning/roadmap_v4_neutral_protocol_factory_rrv.md`.

Research files are inputs. `.planning/active/` remains the execution surface.

## 4. Current fact pattern

What exists:

- vendor-neutral process doc: `docs/agent/PROCESS.md`;
- portable tooling doc: `docs/agent/TOOLING.md`;
- prompt assembly: `scripts/agent/agentctl.py prompt`;
- active sprint orientation: `scripts/agent/agentctl.py context`;
- portable Git hooks: `.githooks/pre-commit`, `.githooks/commit-msg`;
- Claude deep process: `docs/claude/README.md`, `CLAUDE.md`,
  `.claude/agents/*.md`;
- Codex review artifacts in `.planning/active/sprint{N}_phase_{X}_codex_review.md`.

What is still missing or weak:

- no `docs/agent/AGENT_SYSTEM.md` map;
- no dedicated `prompts/agent/handoff.md`;
- no `agentctl status-sprint`;
- no `agentctl lint-planning`;
- no `agentctl audit-commit --rev HEAD`;
- no agent/process CI workflow proving prompt assembly and negative gates;
- exact `## Verdict: PASS` is stricter in docs than current enforcement;
- phase commit gates have known bypass risk around `chore(sprintN)` phase titles;
- Claude native hooks still contain stale sprint-specific assumptions;
- `nexus-phase-auditor` vs `nexus-phase-review-deep` routing is not fully aligned;
- product/security/release/memory roles exist as dimensions, not portable roles.

## 5. S70 phase order

### Phase A - Canon portable

Create the system map:

- `docs/agent/AGENT_SYSTEM.md`;
- role registry: `driver`, `researcher`, `reviewer`, `auditor`, `product`,
  `security`, `release`, `memory`;
- provider mapping: Claude, Codex, GPT, local LLM, human;
- lifecycle modes: sprint start, phase preflight, implementation, review,
  Codex audit, research intake, release audit, product decision, security review,
  memory carry;
- artifact contract: which role writes which file;
- non-goals: no private model memory as authority, no RRV parallel process.

Acceptance:

```bash
rg -n "Truth Stack|Role Registry|Lifecycle Modes|Gate Contract|Non-Goals" docs/agent/AGENT_SYSTEM.md
```

### Phase B - Handoff portable

Create the transfer prompt and wire it:

- `prompts/agent/handoff.md`;
- `agentctl prompt --kind handoff`;
- include `AGENT_SYSTEM.md` in `agentctl context`;
- make handoff explicit about active sprint, phase, changed files, test evidence,
  verdict state, stop conditions and assumptions not to inherit.

Acceptance:

```bash
python -m py_compile scripts/agent/agentctl.py
python scripts/agent/agentctl.py prompt --kind handoff --depth deep
python scripts/agent/agentctl.py prompt --kind universal --depth deep
```

### Phase C - Agentctl observability

Make process state inspectable without reading the whole repo:

- `agentctl status-sprint`;
- `agentctl lint-planning`;
- `agentctl audit-commit --rev HEAD`;
- JSON output where useful for future RRV/Factory consumption;
- tests in `tests/test_agentctl.py`.

Acceptance:

```bash
python scripts/agent/agentctl.py status-sprint
python scripts/agent/agentctl.py lint-planning
python scripts/agent/agentctl.py audit-commit --rev HEAD
uv run pytest tests/test_agentctl.py -q
```

### Phase D - Gates, hooks and CI

Close known bypasses and make the portable layer proven:

- remove stale Sprint 67 assumptions from Claude hooks;
- align `nexus-phase-auditor` and `nexus-phase-review-deep` routing;
- enforce exact `## Verdict: PASS`;
- prevent `chore(sprintN): Sprint N Phase X` from bypassing Codex/9-section
  phase gates;
- add process CI for `agentctl`, prompt assembly, hooks and negative fixtures.

Acceptance:

```bash
python -m py_compile scripts/agent/agentctl.py
uv run pytest tests/test_agentctl.py -q
python scripts/agent/agentctl.py precommit-lightcheck --scope staged
python scripts/agent/agentctl.py prompt --kind universal --depth deep
```

### Phase E - Dogfood on Nexus

Run a real small Nexus change or docs/process phase using only the portable
contracts:

- start from `agentctl context`;
- use `handoff` instead of private chat memory;
- write preflight/review/Codex artifacts;
- verify that Claude, Codex or local LLM could resume from files alone.

Acceptance:

```text
The handoff packet plus repo files are enough to continue the phase in another
provider without relying on hidden chat memory.
```

### Phase F - RRV/Factory contract

Define consumers of the process:

- RRV modes are aliases over roles, not authority:
  `@research -> researcher`, `@dev -> driver`, `@audit -> reviewer/auditor`,
  `@security -> security reviewer`, `@product -> product intake`.
- RRV can display process state and evidence, but execution still comes from
  `.planning/active/` and gates.
- Factory can package templates/prompts/process docs later, but it does not own
  verification authority.
- Babel remains an app created with Factory, not the process itself.

Acceptance:

```bash
rg -n "@research|@dev|@audit|@security|@product|Factory|RRV" docs/agent .planning/active .planning/research
```

## 6. Gate 1 dogfood rule

Gate 1 consolidation remains useful in S70, but as dogfood evidence for the
portable process.

S70 should use the real Factory/Babel/Gate 1 verification to prove:

- process status is observable;
- handoff can restart a provider cleanly;
- reviews and Codex audits are tied to files;
- app/protocol proof evidence is not hidden in chat;
- unresolved P2/P3 carries are routed into `.planning/active`.

## 7. RRV/Factory sequencing after S70

After S70:

- RRV can consume `status-sprint`, `lint-planning`, `audit-commit` and
  `AGENT_SYSTEM.md` as its first process-aware corpus.
- Factory can later package the process as templates/contracts for generated
  projects.
- `@dev LocalOnly`, OSS seed source-only, `sbfb-search`, provider router and
  SearchManifest should be planned only after the process contract is no longer
  implicit.

## 8. S69 Phase E requirements

S69 Phase E should write `sprint70_audit_plan.md` with explicit tracks for:

| Track | Question |
| --- | --- |
| A - Portable canon | Does S70 create `AGENT_SYSTEM.md` without duplicating `PROCESS.md`? |
| B - Handoff | Can another provider resume from repo files and `handoff`? |
| C - Agentctl | Are `status-sprint`, `lint-planning`, and `audit-commit` implemented and tested? |
| D - Gates | Are exact PASS, Codex evidence and 9-section bodies enforced? |
| E - Hooks | Are stale Sprint 67 Claude hooks removed or made dynamic? |
| F - CI | Does CI prove the process layer? |
| G - RRV contract | Are future `@` modes aliases over roles, not a parallel system? |
| H - Factory contract | Is Factory a consumer/packager of process artifacts, not verification authority? |
| I - Dogfood | Did a real Nexus change prove the portable flow? |

## 9. Non-goals for S70

- Do not build RRV total.
- Do not build SearchManifest.
- Do not add network/private compute.
- Do not ingest a broad OSS corpus as verified apps.
- Do not build a Factory process UI.
- Do not move process authority into Factory.
- Do not rely on model memory as truth.

## 10. One-line recommendation

S69 Phase E should route S70 as:

```text
S70 = Process Portable Complete + Gate 1 dogfood
```

RRV and Factory should consume the completed process after that.
