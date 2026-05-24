# Agent Tooling

This repository keeps agent automation in `scripts/agent/` so the workflow is
usable by Claude, GPT, local LLMs, and humans.

## Commands

```bash
python scripts/agent/agentctl.py context
python scripts/agent/agentctl.py prompt --kind universal --depth deep
python scripts/agent/agentctl.py prompt --kind preflight --sprint 35 --phase A
python scripts/agent/agentctl.py prompt --kind preflight --sprint 35 --phase A --depth deep
python scripts/agent/agentctl.py codex-prompt-path --sprint 35 --phase A
python scripts/agent/agentctl.py codex-prompt-path --sprint 35 --phase A --recheck 1
python scripts/agent/agentctl.py verify-on-write --file crates/nexus-core-rs/src/lib.rs
python scripts/agent/agentctl.py precommit-lightcheck
python scripts/agent/agentctl.py precommit-lightcheck --scope staged
python scripts/agent/agentctl.py precommit-lightcheck --scope message --message-file .git/COMMIT_EDITMSG
python scripts/agent/agentctl.py auditor-gate --message-file .git/COMMIT_EDITMSG
python scripts/agent/agentctl.py install-hooks
```

`context` prints the active sprint, dirty files, and key process paths.

`prompt` prints a model-ready prompt. Use it as the first message for a new
provider session so the model inherits the same process and evidence rules.
Use `--kind universal --depth deep` for a full sprint-capable handoff to any
LLM provider. Use `--kind base` only for lightweight orientation. The default
`--depth standard` includes status and diff stats only. Use `--depth deep` when
handing work to another provider; it adds branch, HEAD, staged/unstaged file
name-status, and the last five commit titles without embedding full diffs.

`codex-prompt-path` prints the stable `.git/` prompt filename for Codex phase
verification. The name includes both sprint and phase, for example
`.git/CODEX_SPRINT35_PHASE_A.txt`, so a later Sprint 36 Phase A prompt does not
overwrite it. Use `--recheck N` for targeted reruns such as
`.git/CODEX_SPRINT35_PHASE_A_RECHECK_01.txt`.

`verify-on-write` runs the smallest relevant linter for one file:

- Rust crate file: `cargo clippy -p <crate> --all-targets --locked -- -D warnings`
- Python file: `uv run ruff check <file>`
- `web/` TypeScript file: `npx --no-install eslint <relative file>`

`precommit-lightcheck` inspects staged files and, when a message file is
provided, the commit message. It blocks missing Rust module files, LOC estimates
in sprint plans, and missing Phase A design review. It warns on wire-format
surfaces and suspicious commit-body file references. Use `--scope staged` for
the pre-commit hook and `--scope message --message-file <path>` for the
commit-msg hook; plain `precommit-lightcheck` runs both scopes for manual use.

`auditor-gate` inspects the commit title. Phase commits such as
`feat(sprint35): Sprint 35 Phase A ...` require a review file with
`## Verdict: PASS`. `PASS-PENDING` is a temporary pre-Codex handoff state and
is not committable.

The portable phase title regex matches the repository phase hooks:
`feat|fix|docs|chore|test|refactor` with scope `sprintN` and a title containing
`Sprint N Phase X`. General repository commits may still use any conventional
scope; sprint-scoped phase commits remain gated by the review artifact.

## Git Hook Installation

The portable hooks are committed in `.githooks/`. Enable them once per clone:

```bash
git config core.hooksPath .githooks
```

Or print the same instruction with:

```bash
python scripts/agent/agentctl.py install-hooks
```

After that, `git commit` runs the same gates for every provider. Claude native
hooks can remain enabled; they are a faster feedback layer, not the source of
truth.

The hook scripts are POSIX `sh` and work under Git Bash on Windows. They look
for `python3`, then `python`, then the Windows `py -3` launcher. Failure modes:

- no Python interpreter: blocks with `[agentctl] BLOCK: no Python interpreter found`
- `pre-commit`: runs `python scripts/agent/agentctl.py precommit-lightcheck --scope staged`
- `commit-msg`: runs `python scripts/agent/agentctl.py precommit-lightcheck --scope message --message-file "$1"` and then `python scripts/agent/agentctl.py auditor-gate --message-file "$1"`
- missing review file, `PASS-PENDING`, or any other non-PASS verdict for a phase commit: blocks with `[phase-auditor-gate] BLOCK`
- missing Rust module file, unstaged module file, sprint plan LOC estimate, or missing Phase A design review: blocks with `[lightcheck] BLOCK`
- wire-format files or missing file references in the commit message: warns with `[lightcheck] WARN` but does not block

The Git hooks do not run Cargo, Ruff, ESLint, Playwright, or Semgrep. Those
remain explicit commands or Claude `verify-on-write` feedback so commits do not
duplicate expensive checks.

For phase commits, the prompt/process layer still requires Codex verification
before commit and a commit body with exactly 9 markdown sections, including
`## Codex verification`. The current hooks gate the final review verdict; they
do not replace the written Codex verification evidence.

## Bypass Policy

The portable phase gates do not support an environment-variable bypass. If a
gate blocks incorrectly, fix the gate or the repo-visible process artifact
before committing. `git commit --no-verify` remains a manual Git escape hatch,
but using it creates a process incident that must be documented in planning and
resolved before the phase is considered clean.

## Rust Prompt Assembly (sbfb-factory)

`sbfb-factory process` provides Rust-native prompt assembly and repo context.
It reads prompt files from `prompts/agent/` and assembles them for any provider.

```bash
# Show repo context as JSON (sprint, phase, HEAD, artifacts, AGENT_SYSTEM)
cargo run -p sbfb-factory -- process context

# Assemble a prompt by kind
cargo run -p sbfb-factory -- process prompt --kind preflight
cargo run -p sbfb-factory -- process prompt --kind handoff --depth deep
cargo run -p sbfb-factory -- process prompt --kind phase-review --provider local

# Available kinds
# handoff, preflight, phase-review, commit-body, audit-gate, phase-auditor
# Aliases: review -> phase-review, auditor -> phase-auditor, audit -> audit-gate

# Available providers
# claude, codex, gpt, local, human
# --provider local strips WebSearch/context7 references
```

`process context` output includes:
- `repo`, `branch`, `head`: current git state
- `agent_system`: whether `docs/agent/AGENT_SYSTEM.md` exists
- `process_docs`: list of process documentation files present
- `prompt_kinds`: list of prompt kinds with existence status
- `sprint`, `phase`: detected from `.planning/active/`
- `active_artifacts`: list of planning artifacts
- `dirty_files`, `staged_files`: current worktree state
- `recent_commits`: last 5 commit subjects

## Local LLM Usage

Local models should receive a small assembled prompt, not the whole repository.
Start with `universal` for full sprint work, or `base` for a small bounded
task. Add `preflight` or `phase-auditor` when the role requires it, then paste
only the relevant files and diffs. Keep local LLM tasks bounded to one phase,
one module, or one review artifact.

Use `--provider local` to strip cloud-specific references:

```bash
cargo run -p sbfb-factory -- process prompt --kind preflight --provider local --depth deep
```
