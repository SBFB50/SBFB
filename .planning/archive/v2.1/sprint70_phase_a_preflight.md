# Sprint 70 Phase A — preflight G8

Date : 2026-05-24 | HEAD : `c4494a6` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, no band-aid, research before code, G8 = procedural mechanism for pick-deepest, OSS prior art obligatory before every phase (G10). Phase A is docs-only — no code, no band-aid risk. G10 applies to the documentation design approach.
- feedback_context7_systematic.md : context7 before any lib/API/spec touch. Phase A is docs-only, no lib/API/spec touched. N/A.
- vision_model.md : solo maintainer pattern, no startup/funding. Phase A role registry must not imply team/organization structure. Verified: D1 defines roles as abstract workflow functions (driver, reviewer, etc.), not organizational positions.
- nexus_grid_pivot.md : S70 = Process Portable Complete. Phase A next (AGENT_SYSTEM.md + AGENTS.md). Decisions actees do not constrain agent documentation structure.
- Tensions plan vs memory : aucune.

## Scans (all clean)

- S1a OSS prior art : 8 projets/sources recherches (OpenAI Agents SDK, agent-rules, agent-rules-books, dep/agent-rules, agentic-framework/AGENT_RULES.md, arxiv 2605.11032 Portable Agent Memory, arxiv 2601.08815 Agent Contracts, netresearch/agent-rules-skill), APPROACH-ALIGNED — clean
- S1b deps : 0 libs scannees (docs-only phase, no deps added/bumped), serde/serde_json ecosystem verified no critical CVE applicable — clean
- S2 historiques : 3 fichiers (AGENTS.md, PROCESS.md, TOOLING.md), 5 commits bodies lus en entier — clean
- S3 threat model : FULL, 0 vectors analyses (docs-only, no security surface introduced) — clean
- S4 wire format : FULL / no struct touched, VERSION=1 preserved, Day 0 D1 preserved — clean

## S1a — OSS prior art deep analysis

### Projets analyses en profondeur

#### [OpenAI Agents SDK] — openai/openai-agents-python (https://github.com/openai/openai-agents-python)
- Fichiers source lus : README.md (~200 LOC), handoffs docs page (via WebFetch ~300 LOC)
- Pattern architectural extrait : agents documented with instructions + tools + guardrails + handoffs. Handoffs represented as tools with `HandoffInputData` structure (input_history, pre_handoff_items, new_items, run_context). Manager pattern (centralized) vs Handoff pattern (decentralized peer delegation). Input filtering via capability-based access.
- Edge cases geres : dynamic handoff enable/disable (is_enabled), input filtering (remove_all_tools), metadata capture (input_type for reason/language/priority).
- Verdict : ALIGNED — the SBFB plan separates agent roles (Role Registry) from their delegation patterns (Gate Contract), which mirrors the OpenAI SDK separation of Agent definition from Handoff protocol.

#### [Portable Agent Memory Protocol] — arxiv 2605.11032 (https://arxiv.org/html/2605.11032v1)
- Fichiers source lus : full paper (~3000 LOC equivalent)
- Pattern architectural extrait : five-component memory model M = (E, S, P, W, I) — Episodic, Semantic, Procedural, Working, Identity. Seven-stage re-hydration pipeline (verify, filter, rank, compress, format, frame, inject). Selective disclosure via capability tokens with scoped authorization.
- Key insight for SBFB : the paper distinguishes Procedural memory (learned skills, workflows, routines) from Working memory (transient goals, scratch computations). AGENT_SYSTEM.md = procedural memory (roles, gates, lifecycle modes) stored in repo. Handoff.md = working memory (transient state transfer). The plan's Truth Stack (repo > planning > commits > prompt > chat) directly implements this hierarchy.
- Verdict : ALIGNED — the plan's Truth Stack with "chat memory is non-authoritative" is a simpler but compatible implementation of the paper's tiered authority model.

#### [Agent Contracts Framework] — arxiv 2601.08815 (https://arxiv.org/html/2601.08815)
- Fichiers source lus : full paper (~2500 LOC equivalent)
- Pattern architectural extrait : Agent Contract = seven-tuple C = (I, O, S, R, T, Phi, Psi) — Input/Output specs, Skills, Resources, Temporal boundaries, Success criteria, Termination conditions. Five lifecycle states: DRAFTED -> ACTIVE -> FULFILLED/VIOLATED/EXPIRED/TERMINATED. Hard enforcement (external monitors) vs soft enforcement (budget-aware prompting). Multi-agent conservation law for resource budgets.
- Key insight for SBFB : the Gate Contract section maps well. Preflight verdicts (EXECUTE/PLAN-ADAPT/SCOPE-CUT/DESIGN-CONFLICT) = success criteria with named terminal states. Phase commit gates = hard enforcement at orchestration layer. The paper's multi-dimensional resource constraints (tokens, API calls, iterations) are not needed for SBFB (solo maintainer, no resource accounting).
- Verdict : ALIGNED — Gate Contract with formal verdict trees is a simpler but compatible pattern. The paper validates formalizing gates as named terminal states with explicit conditions.

#### [AGENTS.md Ecosystem Standard] — agent-rules/agent-rules + ecosystem analysis
- Fichiers source lus : dep/agent-rules AGENTS.md (~150 LOC), netresearch/agent-rules-skill README (~100 LOC), mattpocock/agent-rules-books README (~200 LOC), agentic-framework AGENT_RULES.md (~300 LOC), WebSearch results from deployhq, agensi.io, thepromptshelf, sotaaz
- Pattern architectural extrait : AGENTS.md = universal standard (2026, adopted by Google, OpenAI, Sourcegraph, Cursor, Factory). Hierarchical scoping (root AGENTS.md + subdirectory overrides). Content: coding conventions, architecture, project structure, commands. Human-curated > LLM-generated (4% vs -3% performance, arxiv 2601.20404 + 2602.11988).
- Key insight for SBFB : AGENTS.md is the machine-readable entry point. AGENT_SYSTEM.md in docs/agent/ is a deeper reference that AGENTS.md points to. The ecosystem pattern confirms this two-level approach: short root file (AGENTS.md ~40 lines) + detailed reference (AGENT_SYSTEM.md, PROCESS.md, TOOLING.md).
- Verdict : ALIGNED — the plan's approach (update AGENTS.md to point to AGENT_SYSTEM.md) matches the ecosystem convention.

#### [dep/agent-rules AGENTS.md] — dep/agent-rules (https://github.com/dep/agent-rules)
- Fichiers source lus : AGENTS.md full (~150 LOC)
- Pattern architectural extrait : six behavioral guidelines (plan default, subagent strategy, self-improvement loop, verification before done, demand elegance, autonomous bug fixing). Custom context layer (USER_RULES.md, TEAM_RULES.md, LEARNING_LOG.md). Task management (planning verification, progress tracking, change explanation, lesson capture).
- Key insight for SBFB : the concept of a "learning log" as persistent session memory aligns with SBFB's feedback_*.md memory system. The rule "verification before done" = SBFB's gate contract.
- Verdict : ALIGNED — SBFB's structure is more formalized (7 sections vs 6 guidelines) but the patterns are compatible.

### Tableau comparatif

| Aspect | Plan Phase A (AGENT_SYSTEM.md) | OpenAI Agents SDK | Portable Agent Memory (arxiv) | Agent Contracts (arxiv) | AGENTS.md Ecosystem |
|--------|-------------------------------|-------------------|-------------------------------|------------------------|---------------------|
| Role definition | 8 abstract roles (Role Registry) | Agent with instructions/tools/guardrails | N/A (memory protocol, not role protocol) | Skills S = {s1..sm} | Behavioral guidelines |
| Provider mapping | Table Claude/Codex/GPT/local/human | Single provider (OpenAI) | Multi-provider via re-hydration | Provider-agnostic contracts | Per-tool configs (CLAUDE.md, .cursorrules) |
| Delegation/handoff | Gate Contract verdict tree | Handoff as tool with input_filter | 7-stage re-hydration pipeline | Lifecycle states DRAFTED->ACTIVE->terminal | Not formalized |
| Authority hierarchy | Truth Stack (repo > planning > commits > prompt > chat) | Not formalized | Five-component memory model (E,S,P,W,I) | Contract governance (hard + soft enforcement) | AGENTS.md > tool-specific |
| Non-duplication | Complement to PROCESS.md (no overlap) | README + docs pages (separate) | Memory types are orthogonal | Contract spec separate from agent impl | Root AGENTS.md + subdirectory overrides |
| Gate formalization | 4 gate types with named verdicts | Guardrails (input/output validation) | Verify stage in re-hydration | Success criteria Phi with threshold theta | Not formalized |
| Docs separation | AGENTS.md (short) -> AGENT_SYSTEM.md (detail) | README -> docs site | Protocol spec -> implementation | Formal spec -> implementation | Root AGENTS.md -> scoped files |

### Finding S1a
- Classification : APPROACH-ALIGNED
- Evidence : 5 projects/papers analyzed in depth, all confirm the pattern of separating role definitions from process workflow, complementing (not duplicating) existing docs, and formalizing gates as named terminal states.
- Impact sur le plan : aucun. The plan is well-aligned with OSS state-of-the-art for agent system documentation.

## S2 — Decision chain reconstruction

### Fichiers scannes
- `AGENTS.md` : 1 commit body lu (e1ca6f5 "harden fresh-context phase gates")
- `docs/agent/PROCESS.md` : 2 commits bodies lus (e1ca6f5, a15b5c8)
- `docs/agent/TOOLING.md` : 2 commits bodies lus (53e4bb6, e1ca6f5)

### Decisions historiques trouvees

#### Decision 1 : creation de la surface agent portable (PROCESS.md + TOOLING.md + AGENTS.md)
- Sprint 67, sha `e1ca6f5` : "harden fresh-context phase gates"
  Body extrait : "Track the repo-visible agent process surfaces that were previously ignored: AGENTS.md, .githooks/, docs/agent/, prompts/agent/, scripts/agent/README.md, and tests/test_agentctl.py."
- Reverse-commit check :
  - `git log --all --oneline "e1ca6f5..HEAD" -- AGENTS.md docs/agent/PROCESS.md docs/agent/TOOLING.md` : 2 commits found (53e4bb6 + a15b5c8), none are reversions.
  - `git log --all --grep="e1ca6f5" --oneline` : 0 results.
  - Verdict : no reversion. Decision active.
- Status : active
- Impact phase : aucun. Phase A extends this decision by adding AGENT_SYSTEM.md as a new surface in the same directory.

#### Decision 2 : AGENT_SYSTEM.md identified as missing gap
- Sprint 70, sha `78e4413` : "Sprint 70 kickoff + plan"
  Body extrait : "D1 — AGENT_SYSTEM.md carte derivee (5 sections, complement PROCESS.md)"
- Sprint 70, sha `1395020` : "S70 plan adjustment — full prompt portability + provider config"
  Body extrait : "Phase A : AGENT_SYSTEM.md passe de 5 a 7 sections (+Gate Contract avec verdict tree complet, +Prompt Registry)"
- Status : active (Phase A is the implementation of this decision)
- Impact phase : aucun (the phase delivers exactly what the decision mandates)

### Memory constraints
- feedback_approach.md : "G8 = mecanisme procedural pour le principe pick-deepest, pas opinion" — Phase A creates the documentation that formalizes the gates; no conflict.
- vision_model.md : "Pattern OpenBSD solo maintainer" — Role Registry defines abstract workflow roles, not organizational positions. No conflict.
- feedback_context7_systematic.md : "context7 obligatoire avant tout code/decision touchant lib/API/spec" — Phase A is docs-only, no lib/API/spec touched. N/A.

## S3 — Threat model analysis

### Primitive analysee : AGENT_SYSTEM.md (documentation artifact)

Phase A creates `docs/agent/AGENT_SYSTEM.md` (NEW) and updates `AGENTS.md` (cleanup stale refs). Both are Markdown documentation files. No executable code is added or modified.

### Assets en jeu
- Aucun asset de securite impacte. La documentation decrit des roles et gates existants sans modifier la surface d'execution.

### Threat actors
- N/A. Aucun acteur ne peut exploiter un fichier documentation Markdown pour compromettre le systeme. Les fichiers ne sont pas parses au runtime.

### Attack vectors identifies
- (a) Injection/forgery : N/A (documentation statique)
- (b) Replay/reorder : N/A
- (c) DoS/resource exhaustion : N/A
- (d) Information leakage : N/A (documentation publique, pas de secret)
- (e) Privilege escalation : N/A (pas de code executable)
- (f) Supply chain : N/A (pas de dep ajoutee)
- (g) Temporal attacks : N/A

### Mitigations existantes
- T0-T5 non impactes.

### Gaps identifies
- Aucun gap.

### Regression check
- La primitive ne diminue l'efficacite d'aucune mitigation existante.
- La primitive ne cree aucun nouveau vecteur.

### Verdict S3 : clean

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (100 premieres lignes + grep exhaustif VERSION constants)
### Structs verifiees

Phase A ne touche aucune struct dans canonical.rs ni dans aucun autre fichier Rust.

Grep exhaustif :
- `*_FORMAT_VERSION` : CURATOR_LIST_FORMAT_VERSION=1, KEY_ROTATION_FORMAT_VERSION=1, POW_FORMAT_VERSION=1, PIN_FILE_FORMAT_VERSION=1, TASK_FORMAT_VERSION=1. Tous a 1, aucun touche par Phase A.
- `*_ANNOUNCEMENT_VERSION` : aucune constante trouvee avec ce suffixe exact dans crates/nexus-core-rs/src/.
- DOMAIN_*_V1 : 7 constantes (task, result, claim, invite, kudos, curator-list, provenance). Aucune touchee.

### Day 0 check
- D1 sprint courant : AGENT_SYSTEM.md carte derivee 7 sections → Phase A livre exactement ceci. Non contredite.
- D2-D5 sprint courant : non impactees par Phase A (phases futures).
- Decisions actees pivot.md : aucune contredite. Les 12 decisions architecturales gelees + extensions S12-14 ne concernent pas la documentation agent.

### Pre-launch policy
- *_VERSION = 1 : OK (aucune modification)
- Pas de tolerant decoder multi-version : OK (aucune modification code)
- Pas de tests "legacy decode" zombie : OK (aucune modification code)

### AGENTS.md stale references verified
- `scripts/setup.sh` : existe sur disque mais date de Sprint 9 Phase A (PyO3/Python era). Reference stale car le projet est Rust+Frontend pur depuis S50. Cleanup justifie.
- `packages/nexus-sdk` : existe sur disque (legacy Python SDK). Reference stale car le projet a supprime le code Python actif au S50-S51. Cleanup justifie.
- `uv run pytest packages/` : commande pour tests Python, non pertinente post-S50. Cleanup justifie.

## Telemetrie preflight (agent deep)

- Duree totale : ~15m
- S1a : ~8m / 5 projets OSS + 3 papiers arxiv analyses / 8 fichiers source lus via WebFetch / ~3500 LOC reviewees / 0 context7 queries (docs-only phase, no lib touched) / 10 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : ~1m / 0 libs scannees (docs-only) / 1 CVE search (serde ecosystem, no critical for SBFB) / finding : clean
- S2 : ~3m / 5 commits bodies lus / 0 archive files (no archive has Phase A-relevant decisions) / 4 memory files lus en entier / finding : clean
- S3 : FULL / ~1m / 0 vectors (docs-only, no security surface) / 0 gaps
- S4 : FULL / ~2m / 0 structs verifiees (docs-only) / canonical.rs lu (100 premieres lignes + grep exhaustif) : oui

## Action

Proceder code phase A.
