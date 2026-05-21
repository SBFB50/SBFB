# Sprint 67 - Factory/RRV Execution Decision

Date: 2026-05-20
Source checkout: `e1ca6f5` before this note
Purpose: decide whether Sprint 67 Phase C/D Factory work is still on the
correct Roadmap v4 path, and whether the next execution context should be
Claude Code or Codex.

2026-05-21 addendum: later PO recadrage keeps this Sprint 67 decision valid
but narrows the downstream interpretation. Gate 1 is not an `@dev` gate.
`@dev` AST/tree-sitter/source-only work is S70+ by default, or a strict
zero-impact stretch after the `@protocole` Proof Cards + publish + Babel
dogfood path is safe.

## Verdict Court

Phase C/D Factory is not off-roadmap.

It is explicitly part of the current Roadmap v4 and Sprint 67 plan, but only
under a strict interpretation: `sbfb-factory` is a local client-side CLI MVP,
outside the daemon, limited to create/validate/template/lock/security checks in
Phase C, then local provenance/debt closure in Phase D.

The work that must not be pulled into Sprint 67 is:

- Factory publish path.
- `/factory` UI.
- Proof Cards.
- SearchManifest.
- Babel generation or pilot.
- RRV `@dev` AST/tree-sitter/indexing.
- RRV `@web`.
- Any daemon-owned Factory/broker module.

Recommended next action: launch a fresh Claude Code context as the primary
process driver for **Sprint 67 Phase C preflight only**, pass this file as an
input, and do not code before G8 returns `EXECUTE`, `PLAN-ADAPT`, or an
equivalent non-conflict verdict. Keep Codex as independent verification/gate.

## Evidence

### Roadmap Canon

- `CLAUDE.md:195-204` says Roadmap v4 is canon and places S67 on daemon
  primitives, `@protocole` FTS5, and `sbfb-factory` MVP. It also states the
  frozen order: Factory outside daemon, `@protocole` first, then `@dev`, then
  `@web`.
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md:34-39` states the
  daemon is a dumb pipe, Factory leaves the daemon, and RRV order is
  `@protocole` -> `@dev` -> `@web`.
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md:63-67` freezes D2
  Factory outside daemon and D6 `@protocole` before `@dev` before `@web`.
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md:148-150` assigns S67
  to primitives + `@protocole` + `sbfb-factory` MVP, S68 to Proof Cards/publish
  gate, and S69 to Babel via Factory.
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md:163-168` classifies
  `sbfb-factory CLI` as mandatory, while `@dev index` is movable.
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md:286-288` says
  `sbfb-manifest` feeds both deploy and `sbfb-factory`, and `sbfb-factory` must
  never import `nexus-shell-daemon-core`.
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md:317-319` places
  `@protocole` first and makes `@dev` post-pilot/S70+ by default unless it is
  a non-blocking stretch.

### Sprint 67 Plan

- `.planning/active/sprint67_plan.md:24-33` freezes D1 FTS5 `@protocole` and
  D5 `sbfb-factory` CLI create + validate.
- `.planning/active/sprint67_plan.md:36-65` defines the dependency graph:
  Phase A foundation, Phase B FTS5, Phase C `sbfb-factory`, Phase D provenance
  and residual debt.
- `.planning/active/sprint67_plan.md:210-276` defines Phase C as
  `sbfb-factory crate + template engine`, with create/validate, template lock,
  secret scanner, path traversal tests, and a phase commit target.
- `.planning/active/sprint67_plan.md:280-328` defines Phase D as Factory
  provenance and residual debt closure.
- `.planning/active/sprint67_plan.md:427-438` moves preview, advanced diff,
  Babel generation, `@dev` index, Factory audit log, and publish path out of
  S67.
- `.planning/active/sprint67_plan.md:415` requires no daemon dependency from
  `sbfb-factory`.

### Kickoff

- `.planning/active/sprint67_kickoff.md:151-160` says S67 opens Arc 2 and
  establishes daemon primitives, FTS5 `@protocole`, and `sbfb-factory` as an
  external client tool.
- `.planning/active/sprint67_kickoff.md:457-478` retains a local CLI crate
  with create/validate and explicitly excludes daemon/core coupling.
- `.planning/active/sprint67_kickoff.md:591-604` moves preview, diff,
  `/factory`, Proof Cards, Babel, `@dev`, audit log, and publish path to S68+ or
  S69.
- `.planning/active/sprint67_kickoff.md:681-684` accepts that S67 Factory is
  local-only create + validate, with no publish.

### Current Code State

Read-only checks before this note:

- `git rev-parse --short HEAD` -> `e1ca6f5`.
- `crates/sbfb-factory` -> absent.
- `.planning/active/sprint67_phase_c_preflight.md` -> absent.
- `.planning/active/sprint67_phase_c_review.md` -> absent.
- `.planning/active/sprint67_phase_c_codex_review.md` -> absent.
- `Cargo.toml` already contains `crates/sbfb-manifest`, but not
  `crates/sbfb-factory`.

Therefore Phase C has not started in code. The repo is ready for Phase C
preflight, not for Phase C implementation without preflight.

## What Was Actually Delivered

Phase A delivered the prerequisites that Factory needs:

- `sbfb-manifest` is in the Rust workspace.
- The daemon validates `SBFB.json` through the shared manifest crate.
- `node_id` became optional/deprecated instead of a hard deployment blocker.
- Feed primitives exist for later RRV and Proof Card flows.

Phase B delivered `@protocole` search:

- FTS5 search module and daemon route.
- Browser bridge search method.
- THREAT_MODEL feed items closed.
- Final review is PASS with Codex verification.

Phase B also left legitimate carries:

- rusqlite/SQLite CVE tracking in S68.
- Incremental search indexing is absent.
- Browse entries are not indexed yet.

Those carries matter for S68/S69 promises, especially Babel discoverability, but
they do not block a local-only Phase C create/validate MVP.

## Main Risk Before Coding Phase C

The biggest technical risk is not Factory itself. It is contract drift between
`docs/protocol/SBFB_JSON_V2.md` and the current `sbfb-manifest` implementation.

The spec describes fields such as `display_name`, `description`, `license`,
`lang`, `bridge.events`, `tech`, and `requirements`. The current shared crate is
more minimal. If Factory generates a manifest that the crate accepts but the
protocol spec considers incomplete, S67 would create false confidence.

The Phase C preflight must decide one of these paths:

- `EXECUTE`: extend `sbfb-manifest` enough for the Phase C template contract,
  then build Factory against it.
- `PLAN-ADAPT`: keep the manifest minimal but document exactly which v2 fields
  are generated, ignored, or deferred, with tests proving the chosen contract.
- `DESIGN-CONFLICT`: stop and re-plan before implementation.

## Scope Autorise Pour Phase C

Allowed in Phase C:

- New Rust crate `crates/sbfb-factory`.
- Workspace registration.
- CLI with `create --template static --name <name> [--output <dir>]`.
- CLI with `validate <path>`.
- Static embedded template.
- `SBFB.json` generation through `sbfb-manifest`.
- `factory.template.lock` generation.
- Path traversal and symlink rejection.
- Small secret scanner if bounded and tested.
- Focused crate tests and workspace compile.
- No daemon import.

Allowed only if Phase C preflight explicitly accepts it:

- Minimal expansion of `sbfb-manifest` for fields Factory must generate.
- Direct dependency on `walkdir` or a stdlib-only traversal choice.
- Whether `zip` is necessary for `validate <path>` in S67.
- Whether `ed25519-dalek` stays deferred until Phase D/S68+.

## Scope Interdit En Phase C

Forbidden in Phase C unless the active plan is formally adapted:

- Factory publish path to daemon.
- `factory.provenance.json` if kept in Phase D.
- Proof Cards.
- Preview endpoint or ephemeral preview.
- React `/factory` UI.
- Babel app generation.
- RRV `@dev` code/AST/tree-sitter index.
- RRV `@web`.
- SearchManifest.
- Networked trust/discovery.
- Any dependency from `sbfb-factory` to `nexus-shell-daemon` or
  `nexus-shell-daemon-core`.

## Phase D Boundary

Phase D is still coherent, but it must remain local provenance and residual debt
closure, not Proof Cards or publish proof.

Important boundary:

- `factory.template.lock` is planned in Phase C.
- `factory.provenance.json` is planned in Phase D.
- The kickoff mentions provenance in the create flow, while the detailed plan
  splits it into Phase D. Phase C preflight should record this split explicitly
  to avoid accidental scope creep.
- Full signature/decomposed proof should not become an S67 blocker unless the
  phase plan is updated. Roadmap v4 keeps deeper signature/proof work outside
  the S67-S69 minimal path.

## Claude Or Codex

### Objective Recommendation

Use Claude Code fresh context as the primary process driver for the next step,
but only for Phase C preflight first.

Use Codex as independent implementation support or review/gate after the G8
decision, not as both driver and verifier in the same phase.

This is not a model-quality claim. It follows the repo process:

- `docs/claude/README.md` is the strongest canonical workflow for fresh Claude
  contexts, teams, supervisor, preflight, review, and reconciliation.
- `docs/agent/PROCESS.md` and `scripts/agent/agentctl.py` make the workflow
  portable and enforceable for Codex/GPT/local LLMs.
- The dual-agent rule only has value if the implementation driver and
  independent verifier are not the same unchecked context.

### When To Continue Coding With Codex

Continue with Codex only after one of these is true:

- Claude G8 Phase C returns `EXECUTE` and the scope is tight.
- Claude G8 returns `PLAN-ADAPT` and you want Codex to implement the adapted,
  bounded plan.
- You choose Codex as driver knowingly, and then use a separate fresh verifier
  context for the independent gate.

Do not start Phase C code directly from this decision file alone. The missing
artifact is `.planning/active/sprint67_phase_c_preflight.md`.

### When To Launch Claude

Launch Claude now if the goal is roadmap/process correctness before code.

Use the canonical README bootstrap, not only a short base prompt. Attach or
reference this file and ask Claude for:

- Cas B sprint in progress.
- Sprint 67 Phase C.
- Preflight only first.
- No implementation before G8.
- Explicit verdict: `EXECUTE`, `PLAN-ADAPT`, or `DESIGN-CONFLICT`.

## Prompt Pret A Envoyer A Claude Team

```text
Nouveau contexte Claude Code sur nexus.

Repo local obligatoire:
C:\Users\FlowUP\Documents\Code\nexus

Commence par le bootstrap v3 de docs/claude/README.md.
Ne lis rien avant le pre-flight si le bootstrap le demande.

Contexte verifie avant transfert:
- HEAD initial avant note: e1ca6f5 chore(process): harden fresh-context phase gates
- Sprint actif: 67
- Phase A: commitee, review PASS + Codex confirme
- Phase B: commitee, review PASS + Codex confirme
- Phase C: pas commencee
- crates/sbfb-factory: absent
- .planning/active/sprint67_phase_c_preflight.md: absent

Lis ensuite ces fichiers, dans cet ordre:
- .planning/active/sprint67_factory_rrv_execution_decision.md
- .planning/active/sprint67_plan.md
- .planning/active/sprint67_kickoff.md
- .planning/roadmap_v4_neutral_protocol_factory_rrv.md
- .planning/research/SYNTHESIS_factory_rrv_protocol.md
- docs/agent/PROCESS.md
- scripts/agent/agentctl.py

Mission:
- Joue le Cas B Sprint en cours.
- Ne code pas avant G8 preflight.
- Verifie objectivement si Sprint 67 Phase C doit rester:
  1. sbfb-factory local-only create+validate,
  2. PLAN-ADAPT avec scope plus petit,
  3. ou DESIGN-CONFLICT/report S68.
- Produis .planning/active/sprint67_phase_c_preflight.md.
- Le preflight doit trancher: manifest v2 minimal vs complet, deps exactes,
  factory.template.lock en C, factory.provenance.json en D, no daemon dep,
  tests d'acceptation, scope cuts S67.
```

## Decision Matrix

| Option | Process rigor | Roadmap risk | Execution risk | Recommendation |
| --- | --- | --- | --- | --- |
| Claude fresh context, preflight only | High | Low | Low | Best next step |
| Claude codes Phase C directly | Medium | Medium | Medium | Do not do before G8 |
| Codex codes Phase C now | Medium | Medium | Medium | Too early without G8 |
| Codex implements after Claude G8 | High | Low | Medium | Good execution path |
| Skip Phase C to S68 | Medium | Medium | Low now, high later | Not justified by v4 evidence |
| Move Factory into daemon | Low | High | High | Reject |

## Final Decision

Proceed with Sprint 67 Phase C, but only through a new Phase C G8 preflight.

The correct next move is not to abandon Factory or move it to a different
roadmap. The correct move is to keep the S67 Factory scope narrow:

- local-only,
- external CLI crate,
- no daemon coupling,
- no publish/UI/Babel/Proof Cards/SearchManifest,
- contract clarity around `SBFB.json` v2,
- review + Codex verification before any phase commit.
