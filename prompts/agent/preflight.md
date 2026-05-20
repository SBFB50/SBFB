# Phase Preflight Prompt

Run the G8 phase preflight before the first code edit of Sprint `{SPRINT}`
Phase `{PHASE}`. This is a vendor-neutral port of the nexus phase preflight
process: any provider may execute it if it can read files, run shell commands,
cite evidence, and write the required planning artifact.

Write the result to:

`.planning/active/sprint{SPRINT}_phase_{PHASE}_preflight.md`

If the result is `DESIGN-CONFLICT`, do not write implementation code. Instead,
write `.planning/active/sprint{SPRINT}_phase_{PHASE}_pivot_proposal.md` and ask
for arbitration.

This artifact never authorizes a commit by itself. A later phase commit still
requires Codex verification, a final review line exactly `## Verdict: PASS`,
and a 9-section commit body containing `## Codex verification`.

## Non-Negotiables

- Provider neutral: do not use Claude-only tool names, subagent syntax, memory
  tool syntax, or vendor-specific claims.
- ASCII only.
- Every factual claim must cite one of: a repo file path, a command run and its
  relevant output, a URL/date from research, or an explicit assumption.
- The sprint plan is a starting point, not blind authority. If research proves a
  better approach, classify and document the adaptation.
- Preserve SBFB invariants: signing, canonical bytes, loopback trust, sandbox
  boundaries, SBFB bridge behavior, allowlists, provenance, and protocol schema
  stability.

## Required Local Sources

Read these before issuing a verdict:

```bash
cat docs/claude/README.md
cat docs/claude/TOOLING.md
cat docs/agent/PROCESS.md
ls .planning/active
cat .planning/active/sprint{SPRINT}_kickoff.md
cat .planning/active/sprint{SPRINT}_plan.md
git rev-parse --short HEAD
git log --oneline -10
```

If the phase is not present in `sprint{SPRINT}_plan.md`, treat it as an ad-hoc
phase. Use the user-provided commit body or the current diff as source of truth:

```bash
git status --short
git diff --stat
git diff --name-only
git diff -- .planning/active/sprint{SPRINT}_plan.md
```

If there is no plan section, no commit body, and no diff evidence, stop and ask
which source of truth to use. Do not silently skip G8.

## Step 1: Scope Extraction

Extract from the Phase `{PHASE}` plan section:

- target files and modules
- crates/packages/apps affected
- dependencies to add or bump in `Cargo.toml`, `pyproject.toml`, or
  `package.json`
- external APIs, standards, or protocol specs touched
- wire formats, `*_VERSION`, `DOMAIN_*`, canonical serialization, or schemas
- security claims such as Sybil resistance, anti-DPI, sandboxing, credentials,
  revocation, signing, or loopback trust
- named test suites and acceptance commands

Confirm the phase identity is exact. If the plan, preflight filename, review
filename, commit subject, or body point to different phases, stop and classify
the mismatch instead of borrowing evidence from another phase.

Useful commands:

```bash
rg -n "Phase {PHASE}|{PHASE}\\.|Fichiers|Files|Tests|Acceptance|Commit" .planning/active/sprint{SPRINT}_plan.md
rg -n "Day 0|D[1-5]|Scope cuts|Risk|HARDENING|protocol" .planning/active/sprint{SPRINT}_kickoff.md
rg -n "canonical|schema|VERSION|DOMAIN_|sign|loopback|sandbox|provenance|SBFB|bridge" crates/ packages/ web/ docs/ configs/
```

For broad phases with more than 10 likely target files, group files by module
and sample history with `git log --max-count=100` per group. Do not downgrade
crypto, wire-format, network-exposed, or security-component scans.

## Step 1.5: Memory And Process Constraints

Consult repo-visible process docs first. If an external memory file is available
and explicitly referenced by local docs, cite it as an external local source;
otherwise record the constraint as an assumption instead of inventing content.

Commands:

```bash
rg -n "G8|preflight|PLAN-ADAPT|DESIGN-CONFLICT|Day 0|wire format|audit gate" docs/claude/README.md docs/claude/TOOLING.md docs/agent/PROCESS.md
rg -n "scope-cut|rejected|DEVIATION|threat-model|Day 0" .planning/active .planning/archive docs
```

## Step 2: S1 Research Scan

S1 has two required scans.

### S1a OSS Prior Art

Question: how do mature open-source projects solve the same problem this phase
is about?

Identify the functional domain from the phase. Search current OSS prior art and
cite URLs and dates. Use general web search, repository search, package docs, or
local cached docs available to the provider. Examples of reference families:

- compute verification: BOINC, Folding@Home, Golem, Truebit
- LLM safety/guardrails: NeMo Guardrails, Guardrails AI, LangChain,
  openai-agents-python
- P2P networking: libp2p, iroh, IPFS, BitTorrent
- crypto/identity: age, Keyoxide, OpenPGP.js, FROST
- DNS/transport: hickory-resolver, stubby, dnscrypt-proxy

Classify S1a findings:

- `APPROACH-ALIGNED`: plan matches mature OSS practice.
- `APPROACH-NOVEL`: OSS does not do this, but nexus has a justified P2P/SBFB
  reason.
- `APPROACH-NAIVE`: mature OSS evidence shows the planned approach is
  fundamentally flawed.
- `LIB-EXISTS`: a mature, license-compatible library already covers the need.

`APPROACH-NAIVE` or `LIB-EXISTS` is blocking for the original plan and maps to
`PLAN-ADAPT`, not `DESIGN-CONFLICT`.

### S1b Dependencies, CVEs, Release Notes

For every dependency, spec, or external API touched, check versions, advisories,
and relevant release notes:

```bash
rg -n "^(name|version)|\\[dependencies\\]|{crate_or_lib}" Cargo.toml crates/**/Cargo.toml
rg -n "{package_or_lib}" pyproject.toml packages/**/pyproject.toml web/package.json
cargo tree -i {crate_name}
uv tree
(cd web && npm ls {package_name})
```

Search release notes and advisory sources for the current year. A critical/high
CVE affecting crypto, wire, network, sandbox, or signing code is blocking. A
major breaking release on an API the phase uses is blocking unless the plan
already accounts for it.

## Step 3: S2 Historical Decisions

Scan decisions crossed by the target files and domain:

```bash
git log --all --grep="DEVIATION\\|rejected\\|scope-cut\\|deliberate\\|threat-model" -- <target-files>
git log --all --oneline -- <target-files>
rg -n "DEVIATION|rejected for|scope-cut at|threat-model|do not|never|avoid" .planning/archive .planning/active docs
```

For each potential conflict, perform a reverse-commit check:

```bash
git log --all --oneline <rejected_sha>..HEAD -- <files-in-finding>
git log --all --grep="<rejected_sha>" --oneline
git show <candidate_sha> --no-patch --format=%B
```

Classify as:

- confirmed reversion: document, non-blocking
- ambiguous reversion: concern, likely `SCOPE-CUT-CONSISTENT`
- no reversion and rationale still valid: blocking `DESIGN-CONFLICT`

## Step 4: S3 Local Patterns, Contracts, Threat Model

Run S3 as a fast path unless the phase introduces a new security component or
new wire format. If it does, perform a full scan.

Commands:

```bash
rg -n "^### T[0-9]|^## " docs/security/THREAT_MODEL.md
rg -n "S{SPRINT}|Phase {PHASE}|pre-requirement|hardening" docs/security/HARDENING_ROADMAP.md .planning/active
rg -n "PeerCreds|loopback|allowlist|sandbox|capability|revocation|credential|key rotation|sign|verify" crates/ packages/ web/ docs/
rg -n "PATTERNS|canonical|JCS|serde\\(default\\)|unsafe|bridge" docs crates/ packages/ web/
```

Map the phase primitive to T0-T5 where applicable. Blocking S3 findings include
a regression of an already-covered threat or a missing HARDENING_ROADMAP
pre-requirement for the current sprint. Non-blocking findings include documented
future gaps that are not regressions.

## Step 5: S4 Protocol, Security, Wire Scan

Escalate to full S4 if target files include `canonical.rs`, `schemas/`, any
`*_VERSION`, `DOMAIN_*`, signing domain, canonical bytes, or protocol schema.

Commands:

```bash
rg -n "_VERSION\\s*[:=]\\s*[0-9]+|DOMAIN_|canonical_bytes|serde\\(default\\)|schema" crates/nexus-core-rs crates/ packages/ docs/
head -120 crates/nexus-core-rs/src/canonical.rs
rg -n "Pre-launch protocol|Day 0|D[1-5]|wire format|canonical|schema" CLAUDE.md docs .planning/active
```

Verify:

- `*_VERSION` remains `1` unless a blocking CVE requires a bump.
- no tolerant multi-version decoder is introduced pre-launch without explicit
  decision evidence
- `serde(default)` is justified as runtime tolerance, not silent wire drift
- signing domains and canonical bytes remain stable
- Day 0 decisions from kickoff are preserved

Blocking S4 findings map to `DESIGN-CONFLICT`.

## Verdict Vocabulary

Use exactly one:

- `EXECUTE`: no findings; implement the plan as written.
- `PLAN-ADAPT`: S1a found `APPROACH-NAIVE` or `LIB-EXISTS`; implement the
  evidence-backed corrected approach and document the delta.
- `SCOPE-CUT-CONSISTENT`: only non-blocking findings; proceed with documented
  limits or carry-over.
- `DESIGN-CONFLICT`: S1b/S2/S3/S4 has a blocking finding; stop coding and emit
  a pivot proposal.

Aggregation:

- any blocking S1a finding -> `PLAN-ADAPT`
- any blocking S1b/S2/S3/S4 finding -> `DESIGN-CONFLICT`
- only non-blocking findings -> `SCOPE-CUT-CONSISTENT`
- no findings -> `EXECUTE`

## Output Template

```markdown
# Sprint {SPRINT} Phase {PHASE} Preflight

Date: YYYY-MM-DD
HEAD: `<git rev-parse --short HEAD>`
Verdict: **EXECUTE | PLAN-ADAPT | SCOPE-CUT-CONSISTENT | DESIGN-CONFLICT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read: <paths>
- Commands run: <commands with short relevant outputs>

## Scope
- Plan source: `.planning/active/sprint{SPRINT}_plan.md` <section/lines>
- Target files: <list>
- Deps/APIs/specs: <list or none>
- Security/protocol surfaces: <list or none>
- Tests expected: <list>

## S1a OSS Prior Art
- Domain: <domain>
- Sources: <URLs/repos/docs with dates>
- Finding: APPROACH-ALIGNED | APPROACH-NOVEL | APPROACH-NAIVE | LIB-EXISTS
- Impact: <none or adaptation required>

## S1b Dependencies, CVEs, Release Notes
- Scanned: <libs/specs>
- Commands/sources: <evidence>
- Finding: clean | <finding>

## S2 Historical Decisions
- Commands: <git log/rg commands>
- Decisions crossed: <sha/path/rationale/reversion status>
- Finding: clean | <finding>

## S3 Local Patterns And Threat Model
- Threats/contracts checked: <T0-T5 or N/A>
- HARDENING_ROADMAP status: <evidence>
- Finding: clean | <finding>

## S4 Protocol And Wire Invariants
- Wire/security files checked: <paths>
- VERSION/domain/canonical status: <evidence>
- Day 0 status: preserved | conflict
- Finding: clean | <finding>

## Plan Adaptation
Required only for PLAN-ADAPT.
- Original plan: <cite>
- Evidence requiring adaptation: <cite>
- Corrected approach: <concrete implementation direction>
- File/test delta: <delta>

## Risks And Scope Cuts
- Blocking risks: <none or list>
- Non-blocking risks: <carry-over>
- Scope cuts still honored: <cite kickoff>

## Action
- EXECUTE: proceed with Phase {PHASE} as planned.
- PLAN-ADAPT: proceed with corrected approach; commit body must cite this file.
- SCOPE-CUT-CONSISTENT: proceed and track carry-over.
- DESIGN-CONFLICT: stop; see pivot proposal.
```

For `DESIGN-CONFLICT`, the pivot proposal must include the conflict, factual
evidence, options A/B/C, default recommendation, guardrails checked, and the
user decision needed before code starts.
