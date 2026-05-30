# Sprint 70 Phase F — preflight G8

Date : 2026-05-25 | HEAD : `c12aadb` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : "pick deepest, no band-aid, research before code" — Phase F is process tooling refactoring, approach aligned with this principle (hardening gates, closing bypasses, provider portability).
- feedback_context7_systematic.md : "context7 obligatoire avant code touchant lib/API" — Phase F does not introduce new external libs. The only lib-touching code is process.rs (clap, regex, serde_json, axum) which are already in use since Phase C/D. No new API surface.
- vision_model.md : N/A (Phase F does not touch funding/governance/partnerships).
- feedback_codex_gate_strict.md : relevant — Phase F hardens the Codex gate to include `chore(sprintN) Phase` commits. Aligned with this memory.
- feedback_body_section_headers.md : relevant — Phase F hardens FINAL_PASS_RE to refuse `## Verdict : PASS` (spaced colon). Aligned with this memory.
- nexus_grid_pivot.md : D4 "hooks dynamises + provider config" is a frozen Day 0 decision. Phase F implements D4. No contradiction.
- Tensions plan vs memory : none detected.

## S1a — OSS prior art deep analysis

### Problem statement

"How do mature OSS projects handle multi-provider LLM agent configurations and portable prompt templates?"

### Projects analysed

Phase F is internal process tooling (agent wrappers, bash hooks, Rust CLI process code). It does not introduce a new architectural primitive, crypto component, or wire format. The relevant OSS prior art is the pattern of provider-agnostic LLM configurations.

#### LangChain — provider-agnostic wrapper pattern
- Pattern: BaseLLM abstraction with provider-specific adapters (OpenAI, Anthropic, etc.). Prompts are template strings, not vendor-specific instructions.
- Alignment with Phase F: The plan refactors `.claude/agents/` to be thin Claude-specific wrappers referencing portable `prompts/agent/*.md` files. This is the same pattern: portable template + vendor adapter.

#### MCP-Agent (lastmile-ai/mcp-agent)
- Pattern: Settings objects built programmatically for dynamic configuration. Provider config as declarative structure.
- Alignment with Phase F: `PROVIDER_CONFIG.md` defines driver/verifier combinations in a table — declarative, not procedural.

#### Claude Code subagents documentation (code.claude.com/docs/en/sub-agents)
- Pattern: YAML frontmatter for tool access/model, markdown body for system prompt. Agent = metadata + instructions.
- Alignment with Phase F: The refactored agents keep YAML frontmatter (tools, model) and delegate body logic to `prompts/agent/`.

### Finding S1a

- Classification : APPROACH-ALIGNED
- Evidence : The wrapper pattern (thin Claude-specific agent referencing portable prompt) matches the standard provider-agnostic pattern in LangChain, MCP-Agent, and Claude Code's own subagent architecture.
- Impact on plan : none. Plan follows established patterns.

## S1b — Deps/libs versions + CVE

### Libs scanned

Phase F modifies `crates/sbfb-factory/src/process.rs` which uses:
- `regex` (workspace) — no CVE found (RustSec search 2026-05-25)
- `serde_json` (workspace) — no CVE found
- `clap` (workspace) — no CVE found
- `axum` (workspace) — no CVE found (RustSec search 2026-05-25)
- `tokio` (workspace) — RUSTSEC-2025 broadcast channel Sync issue (not relevant, Phase F does not use broadcast)
- `tower-http` (workspace) — no CVE found

No new dependencies are introduced by Phase F. No version bumps planned.

### Specs checked

No spec touched by Phase F. No RFC revision relevant.

### Finding S1b

- Classification : clean
- 0 CVE critical/high on deps in scope
- 0 breaking changes anticipated

## S2 — Decision chain reconstruction

### Files scanned

- `.claude/hooks/process-task-gate.sh` : 22 commits total, 5 bodies read in detail
- `.claude/hooks/process-supervisor-stop.sh` : 22 commits total (shared history), 3 bodies read
- `.claude/hooks/phase-precommit-lightcheck.sh` : 22 commits total, 4 bodies read
- `.claude/hooks/phase-auditor-gate.sh` : 22 commits total, 3 bodies read
- `crates/sbfb-factory/src/process.rs` : 8 commits total, 3 bodies read
- `.claude/agents/*.md` : 17 commits total, 6 bodies read

### Decisions historiques trouvees

#### Decision 1 : NEXUS_SKIP_PHASE_AUDITOR removed (no bypass)

- Sprint 67, sha `e1ca6f5` : "Remove the NEXUS_SKIP_PHASE_AUDITOR bypass path"
  Body : "require staged diff whitespace checks, require 9 canonical phase body sections, and keep Codex verification mandatory for phase commits"
- Reverse-commit check : no reversion found
- Status : active
- Impact phase : none (Phase F extends this by also closing the `chore(sprintN) Phase` bypass)

#### Decision 2 : Zero exemption (no LOC/docs-only/content-based bypass)

- Sprint 67, sha `543eb45` : "Directive PO : toutes les phases recoivent le meme traitement maximal sans exception"
  Body : "Supprime tous les seuils LOC, tous les timeboxes, toutes les exemptions docs-only/dette/cosmetique pour Codex et G8"
- Reverse-commit check : no reversion found
- Status : active
- Impact phase : ALIGNED. Phase F extends this principle to `chore(sprintN) Phase` commits. The plan's approach of including `chore` in the gate guard directly implements this decision.

#### Decision 3 : Body format enforcement with exact headers

- Sprint 66, sha `349f998` : "body-format enforcement + lightcheck checks 4-9"
  Body : "Agent nexus-phase-auditor : dimension 7 Body-format. Hook lightcheck : checks 4-9. Template commit_body_phase.txt."
- Sprint 70, Phase A sha `92a4d19` : `## Scope cuts` not `## Scope cuts respectes` (memory feedback_body_section_headers.md)
- Reverse-commit check : no reversion found
- Status : active
- Impact phase : Phase F hardens FINAL_PASS_RE to reject `## Verdict : PASS` (spaced colon). This is a direct extension of the exact-headers principle.

#### Decision 4 : Superviseur process optionnel (D17)

- CLAUDE.md : "Superviseur process optionnel, hooks = backstop mecanique (D17)"
- This is a frozen architectural decision. Phase F does not change the superviseur optionality — it only dynamizes the hooks that serve as backstop.
- Status : active, not contradicted
- Impact phase : none

#### Decision 5 : post-commit-memory.sh suppressed

- docs/claude/README.md line 2494 : "Hooks supprimes (0 valeur causale prouvee) : [...] post-commit-memory"
- `install-claude-tooling.sh` still references `.claude/hooks/post-commit-memory.sh` (line 82-106)
- `docs/claude/TOOLING.md` line 138 describes auto-install of this hook
- The file `.claude/hooks/post-commit-memory.sh` does NOT exist on disk (verified)
- Status : suppressed, but references are stale
- Impact phase : Phase F correctly identifies this as needing cleanup. Plan §9.1(b) says "corriger/supprimer post-commit-memory.sh casse" and plan §9.2 lists `scripts/install-claude-tooling.sh` and `docs/claude/TOOLING.md` as UPDATE targets.

### Memory constraints

- feedback_approach.md : "pick deepest" — Phase F is the deepest option (close all bypass paths, dynamize, not just patch one hook)
- feedback_codex_gate_strict.md : "zero exemption, toutes les phases" — Phase F extends this to chore(sprintN) commits
- feedback_body_section_headers.md : "headers EXACTS" — Phase F tightens FINAL_PASS_RE to refuse spaced colon

No memory constraint conflicts with the plan.

## S3 — Threat model analysis

### Primitive analysee : Agent refactor + hooks + provider config

### Assets en jeu

- A1 Process integrity (medium) : commit gates prevent unreviewed code from entering the repo
- A2 Audit trail (medium) : planning artifacts trace every phase decision
- A3 Provider portability (low) : prompts work across providers

### Threat actors

- TA1 Agent model bypass (automated, Claude/GPT/Codex) : model forgets or ignores a gate
- TA2 Developer shortcut (human) : `git commit --no-verify` or `chore(sprintN)` to dodge gates

### Attack vectors

1. V1 chore(sprintN) Phase bypass : a commit typed `chore(sprint70): Sprint 70 Phase A` passes the current guards because `matches!(commit_type, "feat" | "fix" | "docs" | "test" | "refactor")` excludes `chore`. **Currently exploitable.** Phase F closes this.
2. V2 Spaced verdict `## Verdict : PASS` : the current `has_final_pass_verdict` function in process.rs (line 312) accepts both `## Verdict: PASS` and `## Verdict : PASS`. The hook phase-auditor-gate.sh (line 104) also accepts the spaced form via `[[:space:]]*:`. **Currently exploitable** but low impact (both forms are written by review agents, not user-supplied).
3. V3 Stale hook references : hooks referencing "sprint 67" no longer block anything (S67 artifacts are in archive). **Currently degraded** — the hooks provide no protection for the current sprint's factory/preflight checks.
4. V4 DoS (irrelevant) : hooks run locally, no network exposure
5. V5 Injection via hook input : hooks parse JSON stdin via jq/python3. Input comes from Claude Code, not external sources. Low risk.
6. V6 Supply chain (no new deps) : Phase F adds no new dependencies.
7. V7 Temporal/TOCTOU : review files are checked at commit time; file system is local. No race condition concern.

### Mitigations existantes

- T0-T5 : none directly relevant (threat model covers P2P/crypto/network, not process tooling)
- Process gates (hooks) : are the mitigation under improvement

### Gaps identifies

- GAP1 V1 chore bypass : severity HIGH (process gate bypass). Phase F closes this.
- GAP2 V2 spaced verdict : severity LOW (cosmetic, both forms are agent-generated). Phase F closes this.
- GAP3 V3 stale hooks : severity MEDIUM (hooks ineffective for current sprint). Phase F closes this.

### Regression check

- Phase F does not diminish any existing mitigation
- Phase F does not create new attack vectors
- Phase F closes 3 existing gaps

### Verdict S3 : clean (Phase F improves security posture)

## S4 — Wire format deep audit

### canonical.rs : NOT TOUCHED by Phase F

Phase F modifies:
- `.claude/agents/*.md` (agent documentation files — no wire format)
- `.claude/hooks/*.sh` (bash hooks — no wire format)
- `crates/sbfb-factory/src/process.rs` (CLI process tooling — no wire format)
- `docs/agent/PROVIDER_CONFIG.md` (documentation — no wire format)
- `docs/agent/TOOLING.md` (documentation — no wire format)
- `scripts/install-claude-tooling.sh` (install script — no wire format)
- `docs/claude/TOOLING.md` (documentation — no wire format)

### Day 0 check

- D1..D5 sprint 70 : none contradicted by Phase F
  - D1 AGENT_SYSTEM.md : Phase F agents reference it, no modification
  - D2 handoff portable : Phase F uses existing handoff.md, no modification
  - D3 sbfb-factory Rust 3 commands : Phase F extends process.rs (adds provider adaptation, verdict hardening, chore gate), does not contradict D3
  - D4 hooks dynamises : Phase F IMPLEMENTS D4 directly
  - D5 Factory Viewer/Operator + contrat RRV/Factory : Phase F dogfoods Operator/Viewer, does not modify them

### Pre-launch policy

- No `*_VERSION` change
- No wire format modification
- No tolerant decoder
- No legacy decode test

### Decisions actees pivot.md

All 12 decisions actees (plus extensions S12-S14) : none contradicted.
Phase F touches process tooling only, not P2P/crypto/identity/deploy/curator.

### Verdict S4 : clean

## Telemetrie preflight (agent deep)

- S1a : 3 projects analysed (LangChain, MCP-Agent, Claude Code docs) / 0 source files read (process tooling pattern, not code-level) / 0 context7 queries (no new lib) / 3 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : 6 libs scanned (regex, serde_json, clap, axum, tokio, tower-http) / 3 CVE searches / finding : clean
- S2 : 5 decisions reconstructed / 20+ commit bodies read / 5 archive files scanned / 6 memory files read / finding : clean (all decisions aligned, 3 gaps Phase F closes)
- S3 : FULL / 7 vectors analysed / 3 gaps (all closed by Phase F) / 0 regressions
- S4 : FULL / 0 structs to verify (no wire format touched) / canonical.rs not read (not in scope) / Day 0 preserved / pre-launch policy clean

## Action

Proceder code phase F.
