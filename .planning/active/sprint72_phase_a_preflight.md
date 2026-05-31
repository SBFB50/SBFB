# Sprint 72 Phase A Preflight

Date: 2026-05-31
HEAD: `1803d78`
Verdict: **EXECUTE**

Depth calibration: this is a docs/threat phase closing a carry whose defense is
already implemented and tested in Sprint 71. Per plan checkpoint section 13
("G8 preflight phases code (A docs allege ; C migration = format complet S1b)")
this preflight is the allege variant: S3 (threat-model coverage) and S2
(historical decisions on the Operator surface and the S71 audit trigger) are the
load-bearing scans; S1a/S1b (OSS prior art / deps / CVE) and S4 (wire format)
are confirmed Not-Applicable below rather than forced.

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (portable procedure, source of truth)
  - `.planning/active/sprint72_plan.md` (Phase A section 4)
  - `.planning/active/sprint72_kickoff.md` (sections 3, 4, 5, 6)
  - `.planning/archive/v2.1/sprint71_audit_findings.md` (P2-H-1 finding +
    Carry-Over)
  - `docs/shell/PATTERNS.md` §P35 (lines 2158-2207, Operator hardening
    rationale)
  - `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (full)
  - `docs/security/THREAT_MODEL.md` (section headers + section 5.5)
  - `crates/sbfb-factory/src/auth.rs` (token + Host + Origin defense)
  - `crates/sbfb-factory/src/operator_server.rs` (SENSITIVE_ACTIONS gate, CORS,
    SSE dispatch point)
  - `crates/nexus-shell-daemon/src/http.rs` (tasks/submit auth tier)
- External local source: memory `MEMORY.md` + `nexus_grid_pivot.md` consulted
  for "pick deepest, no band-aid" and "Factory = external crate hors daemon"
  constraints (routing table: feedback_approach + vision/loopback context).
- Commands run (relevant outputs inline):
  - `git rev-parse --short HEAD` -> `1803d78`
  - `git grep -n P2-H-1 .planning` -> plan + kickoff + sprint71_audit_findings
  - `grep -ciE operator docs/security/THREAT_MODEL.md` -> `0` (word "operator"
    absent; only an unrelated "preview" line matched the broad alternation)
  - `grep -niE "operator|3001|spawn|bypassPermissions|sbfb-factory"
    docs/security/THREAT_MODEL.md docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`
    -> 1 unrelated hit (preview line 661), 0 Operator hits in either catalogue
  - `git log --oneline -15 -- docs/security/THREAT_MODEL.md` /
    `... LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (S2 reversion check, below)

## Scope
- Plan source: `.planning/active/sprint72_plan.md` section 4 (Phase A —
  Catalogue menace Operator P2-H-1 + reservation surface), lines 77-119.
- Target files:
  - `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (add Operator `:3001`
    server entry: endpoint + trust tier + write/spawn capability, cross-ref
    PATTERNS §P35; note NetworkProvider S72 as a client of the loopback daemon)
  - `docs/security/THREAT_MODEL.md` (add Operator threat entry:
    CSRF/DNS-rebinding mitigated by S71 G7 token+Host+CORS, autonomous
    spawn-agent mitigated by S71 G2 SENSITIVE_ACTIONS gate; ref PATTERNS §P35)
- Deps/APIs/specs: none. No `Cargo.toml`/`package.json` edit in Phase A.
- Security/protocol surfaces: documentation of an existing, already-hardened
  loopback surface (the Operator `:3001` HTTP server). No new runtime surface
  is introduced by Phase A.
- Tests expected: none new (the defense is tested by S71). Acceptance is a
  documentary-presence grep (plan section 4.4):
  - `grep -i "operator" docs/security/THREAT_MODEL.md` (>= 1 hit)
  - `grep -iE "operator|3001" docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`
    (>= 1 hit)
  - `grep -i "P35" docs/security/THREAT_MODEL.md
    docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`

## S1a OSS Prior Art
- Domain: security threat-cataloguing of an existing loopback HTTP control
  surface. This is documentation of a defended surface, not the design of a new
  primitive.
- Sources: not applicable. The phase neither designs a defense nor selects a
  library; it transcribes an already-implemented and S71-reviewed defense
  (`docs/shell/PATTERNS.md §P35`) into the canonical security catalogue. There
  is no OSS "right way to do it" decision to validate here — the design
  decision (loopback hardening on a write+spawn surface) was already made and
  validated in S71 Phase C. S1a was extensively run at kickoff for the S72
  code phases (D1/D2/D3, kickoff "Sources context7 + WebSearch") — not for
  Phase A.
- Finding: N/A (no new approach or library introduced by Phase A).
- Impact: none.

## S1b Dependencies, CVEs, Release Notes
- Scanned: none required. Phase A edits two Markdown files only. No dependency
  is added or bumped (the `ollama-rs 0.3.4` bump and `ExecutionTarget` land in
  Phase C, not A — plan section 6).
- Commands/sources: confirmed Phase A target list (plan section 4.2) contains
  no `Cargo.toml` / `package.json` / `pyproject.toml`.
- Finding: clean (N/A — no deps in scope).

## S2 Historical Decisions
- Commands:
  - `git log --oneline -15 -- docs/security/THREAT_MODEL.md`
    -> last edits: c92e656 (S69 preview surface), ecb25c5/f46bc66 (S68/S67
    proof-card + feed/search surfaces), ea87547 (S66 feed), 1f79c52 (S29
    section 9 per-mode), 1ff04df (S16 origin). All ADDITIVE surface entries.
  - `git log --oneline -15 -- docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`
    -> ace05b0 (S65 auth tier), ed5bbdc (S54), 9676bd9 (S22 origin). All
    additive.
- Decisions crossed:
  - P2-H-1 (`.planning/archive/v2.1/sprint71_audit_findings.md` Track H lines
    312-344, Summary line 381, Carry-Over lines 406-411): the authoritative
    finding and exit condition. The exit condition stated there matches the
    plan section 4.2 livrables EXACTLY — Operator `:3001` entry in
    LOOPBACK_ENDPOINTS_TRUST_TIERS (endpoint + trust tier + write/spawn) AND a
    CSRF/rebinding + spawn-agent threat entry in THREAT_MODEL referencing
    PATTERNS §P35. Reversion status: not a reverted decision — this is an OPEN
    carry routed to S72 Phase A, owner = S72, trigger = "before any extension
    of the Operator surface". Confirmed consistent (carry coherence verified —
    mission item 3).
  - Pre-launch protocol (`CLAUDE.md`): Phase A touches no wire format and no
    `*_VERSION`; the policy is not engaged. No conflict.
  - "Factory = outil client externe (crate sbfb-factory), hors daemon" (CLAUDE.md
    frozen decision): documenting the Operator as a TCP-loopback-only surface
    that mirrors (does not import) the daemon loopback layer is consistent with
    `auth.rs:21-30` deliberate-duplication note and §P35. No conflict.
- Finding: clean. No commit ever reverted, scope-cut, or decided AGAINST
  cataloguing the Operator surface; the only relevant decision (P2-H-1) is an
  open carry whose exit condition Phase A satisfies. No rationale-still-valid
  rejected decision is being re-opened.

## S3 Local Patterns And Threat Model
- Threats/contracts checked:
  - **Defense reality (mission item 1 — no phantom mitigation).** The mitigation
    Phase A will document is present and tested in the tree at `1803d78`:
    - Token + Host + Origin: `crates/sbfb-factory/src/auth.rs` —
      `auth_required` middleware (lines 229-262) enforces loopback `Host:`
      (403), loopback-or-absent `Origin:` (403), and `X-SBFB-Token` via
      `constant_time_eq` (401). `is_loopback_host` / `is_loopback_origin`
      reject `evil.com`, `169.254.169.254`, suffix tricks, https, and path
      injection (tests lines 268-301). This is the S71 G7 defense.
    - CORS pinned to loopback origin: `operator_server.rs:100-103`
      (`AllowOrigin::predicate(... is_loopback_origin)`) — no `allow_origin(Any)`.
    - SENSITIVE_ACTIONS SSE gate (S71 G2): `operator_server.rs:34`
      (`SENSITIVE_ACTIONS = ["shell","commit","push","PASS"]`) and the SSE
      handler `handle_chat_stream` (lines 858-879) runs the gate and returns
      `sse_gate(...)` / `requires_gate` BEFORE spawning, never on an ungated
      `bypassPermissions` path. The gate sits at lines 866-879, immediately
      before `spawn_claude_stream` at line 898 — the exact dispatch point D4
      will later swap for `ExecutionTarget::run`.
    - PATTERNS §P35 (`docs/shell/PATTERNS.md:2158-2207`) documents this
      rationale (auth / SSE gate / model / spawn safety / threat boundary D5)
      and lists the test names. The defense is real, not a phantom.
  - **Threat-model coverage gap is real (regression check).** The append-pattern
    threat-model has a clean per-surface structure (`## 10. Feed surface`,
    `## 11. Search surface`, `## 12. ProofCard surface`,
    `## 13. Preview ephemere surface`); the Operator surface has no section.
    `grep -ciE operator docs/security/THREAT_MODEL.md` -> 0. The
    LOOPBACK_ENDPOINTS_TRUST_TIERS inventory (§3, lines 48-64) lists daemon /
    coordinator endpoints but NOT the Operator `:3001` server. This is a
    documentation-completeness gap (the defense exists), not a regression of a
    previously-covered threat. Non-blocking by the S3 classification table
    ("documented future gap (not regression)"); and Phase A CLOSES it.
  - **NetworkProvider (S72) boundary anticipation.** `tasks/submit` sits inside
    `authed_routes` behind `auth_required` at tier T0 (`http.rs:306` route +
    `:432` middleware; already inventoried in LOOPBACK_ENDPOINTS_TRUST_TIERS
    §3 line 55 as T0). So the Phase A note — "the S72 network dispatch
    (NetworkProvider -> loopback daemon) stays inside the already-hardened
    loopback boundary, an outbound client, not a new inbound surface" — is
    factually correct and consistent with the existing catalogue.
  - **Tier vocabulary alignment.** LOOPBACK_ENDPOINTS_TRUST_TIERS uses a T0/T1/T2
    tier model (§2). The new Operator entry should declare a tier in that
    vocabulary (T0 loopback bearer + Host + Origin, plus the application-level
    SENSITIVE_ACTIONS gate as the spawn-path mitigation). Note for the author,
    not a blocker.
- HARDENING_ROADMAP status: no S72 pre-requirement is unmet by Phase A; the
  S71 audit explicitly recorded "HARDENING_ROADMAP.md : pas de pre-requis S71
  non livre" (sprint71_audit_findings Track H). Phase A is itself the closure
  of the carried documentation pre-requirement gating S72's SSE extension.
- Finding: clean (the only S3 observation is the documented gap Phase A is
  chartered to close — non-blocking, and resolved by the phase).

## S4 Protocol And Wire Invariants
- Wire/security files checked: none in scope. Phase A target list
  (plan section 4.2) is two Markdown docs; no `canonical.rs`, no `schemas/`,
  no `*_VERSION`, no `DOMAIN_*`, no signing domain, no serde-tagged wire struct.
- VERSION/domain/canonical status: unchanged. The kickoff confirms
  (`section 1.4`) S72 touches no wire format; the NetworkProvider (a later
  phase) only consumes the existing `TaskSubmission`/`TaskStatus`. Phase A is
  a strict no-op on the wire.
- Day 0 status: preserved. D1-D5 (kickoff section 4) concern Phase C/D/E code,
  not Phase A; Phase A does not touch them. The frozen "Factory hors daemon"
  and "pre-launch edit canonical freely / no bump" decisions are not engaged.
- Finding: clean (N/A — no wire surface in scope).

## Plan Adaptation
Not applicable (verdict is EXECUTE, no S1a APPROACH-NAIVE / LIB-EXISTS finding).

## Risks And Scope Cuts
- Blocking risks: none.
- Non-blocking risks / author notes (carry into the Phase A write, not blockers):
  1. **Doc-ref disambiguation.** "PATTERNS §P35" resolves to TWO sections:
     `docs/shell/PATTERNS.md §P35` (Factory Operator hardening — the
     load-bearing one) and `docs/rust/PATTERNS.md §P35` (an unrelated S23
     ephemeral-worker pattern). The Phase A threat entries must cite
     `docs/shell/PATTERNS.md §P35` specifically, or readers may follow the
     wrong reference. The acceptance grep `grep -i "P35"` does not distinguish
     them — the author should write the full path.
  2. **Tier vocabulary.** Declare the Operator entry in the existing T0/T1/T2
     model (LOOPBACK_ENDPOINTS_TRUST_TIERS §2), not an ad-hoc tier, so the
     inventory stays internally consistent. Recommended: T0 (loopback bearer +
     Host + Origin) with the SENSITIVE_ACTIONS gate documented as the
     spawn-path application-level mitigation; residual = hostile local process
     reading the token (the accepted node-level OS-sandbox boundary, D5 / §P35
     / §5.7 key-storage residual).
  3. **Threat-model placement.** The append pattern suggests a new
     `## 15. Operator surface (Sprint 71 ship / S72 catalogue)` section with
     one or more `### T-OPERATOR-*` entries (CSRF/DNS-rebinding, autonomous
     spawn-agent), mirroring §10-§13. This keeps the structure uniform and the
     acceptance grep satisfied.
- Scope cuts still honored (kickoff section 7 / plan section 11): Phase A adds
  zero code and zero tests; it does not touch the SSE dispatch, the
  `ExecutionTarget` enum, `ollama-rs`, or the front Operator — all of which are
  C/D/E. Phase A does not pre-empt or alter any Day 0 decision. The "no wire
  bump" pre-launch invariant is preserved.

## Action
- EXECUTE: proceed with Phase A as planned. Write the two documentary entries
  (LOOPBACK_ENDPOINTS_TRUST_TIERS Operator `:3001` inventory entry + tier;
  THREAT_MODEL Operator surface section), cross-referencing
  `docs/shell/PATTERNS.md §P35` by full path, declaring the trust tier in the
  existing T0/T1/T2 vocabulary, and noting the S72 NetworkProvider as an
  outbound client of the already-cataloged T0 `tasks/submit` endpoint. Satisfy
  the plan section 4.4 acceptance greps. P2-H-1 exit condition (sprint71 audit
  Carry-Over) is met when both catalogues reference the Operator surface and
  §P35. No commit is authorized by this artifact alone (Codex verification +
  `## Verdict: PASS` review + 9-section body still required at the phase commit).
