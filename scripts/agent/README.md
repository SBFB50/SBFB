# Agent Scripts

`agentctl.py` is the portable entry point for the vendor-neutral agent process.
It is intentionally stdlib-only so it can run from PowerShell, Git hooks, CI,
Claude, GPT, or local LLM wrappers.

Common commands:

```bash
python scripts/agent/agentctl.py context
python scripts/agent/agentctl.py prompt --kind universal --depth deep
python scripts/agent/agentctl.py prompt --kind base
python scripts/agent/agentctl.py prompt --kind preflight --sprint 35 --phase A --depth deep
python scripts/agent/agentctl.py verify-on-write --file scripts/agent/agentctl.py
python scripts/agent/agentctl.py precommit-lightcheck
python scripts/agent/agentctl.py precommit-lightcheck --scope staged
python scripts/agent/agentctl.py precommit-lightcheck --scope message --message-file .git/COMMIT_EDITMSG
python scripts/agent/agentctl.py auditor-gate --message-file .git/COMMIT_EDITMSG
python scripts/agent/agentctl.py install-hooks
```

Subcommands:

- `context`: print repo, active sprint, process paths, and git status.
- `prompt`: assemble a provider-neutral prompt from `prompts/agent/`.
- `verify-on-write`: run scoped Rust, Python, or frontend lint for one file.
- `precommit-lightcheck`: inspect staged diff and/or commit message.
- `auditor-gate`: block phase commits without a final review artifact whose
  verdict line is exactly `## Verdict: PASS`; `PASS-PENDING` is not
  committable.
- `install-hooks`: print the `git config core.hooksPath .githooks` command.

Phase commit prompts also require independent Codex verification and a
9-section markdown commit body containing `## Codex verification`. `agentctl.py`
is not the whole process contract; use it with `docs/agent/PROCESS.md` and
`prompts/agent/commit-body.md`.
