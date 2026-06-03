# Sprint 72 Phase B Preflight

Date: 2026-05-31
HEAD: `105c054`
Verdict: **EXECUTE**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure portable, source of truth)
  - `.planning/active/sprint72_kickoff.md` (§4 D1-D5, §5 Phase B, §6 carry, §9 R6/R7)
  - `.planning/active/sprint72_plan.md` (§5 Phase B, §10 delta)
  - `.claude/agents/*.md` (4 wrappers referencing prompts)
  - `prompts/agent/` (8 prompt files on disk)
  - `crates/sbfb-factory/src/process.rs` (PROMPT_KINDS, prompt_filename, repo_root, tests)
  - `crates/sbfb-factory/Cargo.toml` (dev-deps)
  - `docs/agent/AGENT_SYSTEM.md` (§6 Prompt Registry, table kinds)
  - `docs/agent/PROCESS.md` (prompts/agent reference)
  - `crates/nexus-shell-daemon/src/dispatch_loop.rs` (E2E test + helpers)
  - `crates/nexus-core-rs/src/task.rs` (TaskEntry/ResultEntry/Claim verify_signature)
  - `crates/nexus-core-rs/src/docs.rs` (get_many_by_prefix, blob read pattern)
  - `crates/nexus-worker-core/src/engine/runtime.rs` (result write path, docs() accessor, node privacy)
  - `crates/nexus-core-rs/src/canonical.rs` (DOMAIN_*, JCS)
  - `docs/rust/PATTERNS.md §P53` (sha256 misnomer, seed as i32)
  - `.planning/archive/v2.1/sprint70_phase_f_review.md` (P2-F-3 origin), `sprint70_verification.md:132` (1/3),
    `sprint71_verification.md:186` (2/3), `sprint71_audit_findings.md:298` (P3 carry)
  - `scripts/agent/agentctl.py:139` (prompts/agent in context paths)
  - `.claude/hooks/phase-precommit-lightcheck.sh` (hook surface for option b)
- Commands run: see each scan section (git log/rev-parse, rg/grep, ls, wc).

## Scope
- Plan source: `.planning/active/sprint72_plan.md` §5 (Phase B — Dette pair, Regle 1).
- Phase identity confirmed: plan §5 titled "Phase B — Dette pair (Regle 1) :
  P2-F-3 3/3 + carries compute"; this preflight filename is
  `sprint72_phase_b_preflight.md` (lowercase, hook-compliant). Phase A is already
  committed (`105c054`); HEAD matches. No phase mismatch.
- Target files:
  - P2-F-3: `crates/sbfb-factory/src/process.rs` (test add) + `docs/agent/AGENT_SYSTEM.md`
    (contract note). Read-only inputs: `.claude/agents/*.md`, `prompts/agent/*.md`.
  - P2-A-2: `crates/nexus-shell-daemon/src/dispatch_loop.rs` (assertion in existing
    test `dispatched_task_is_claimed_and_executed_by_worker_engine`); possibly a small
    `pub fn blobs()` accessor on `Engine` in `crates/nexus-worker-core/src/engine/runtime.rs`.
  - P3-A-3 / P3-B-1 / P3-B-2: `docs/rust/PATTERNS.md §P53` (confirm/re-doc), no code change expected.
- Deps/APIs/specs: NONE added in Phase B. (ollama-rs bump is Phase C, not B.)
- Security/protocol surfaces: P2-A-2 READS and verifies an existing signed `ResultEntry`
  (Ed25519 over `canonical_bytes(payload, DOMAIN_RESULT_V1)`). No new signing path, no
  canonical-bytes change, no domain change.
- Tests expected (plan §5.3 / §10): +1-2 Rust. P2-F-3 coupling check (1 test);
  P2-A-2 assertion added to the existing E2E (not a new test, possibly +1 small accessor).
  P3 cosmetic = re-doc only.

## S1a OSS Prior Art
- Domain: this is a **process/dette phase** (a CI-style coupling check between a
  vendor wrapper file and a portable template file + a signature assertion in an
  existing integration test). It is not a crypto primitive, a new wire format, or a
  network-exposed component. The procedure (preflight.md S3 fast-path) allows a fast
  scan for non-security, non-wire phases. S1a is therefore narrow.
- Sources / pattern: the "manifest references a file; a test asserts the referenced
  file exists" pattern is the standard mechanical anti-drift guard (e.g. doc-link
  checkers, `cargo` `include_str!` compile-time embedding, asset-existence unit tests).
  The repo itself already uses the same shape: `process.rs:738-741` computes
  `exists = root.join("prompts/agent").join(prompt_filename(k)).exists()` per kind, and
  `prompt_data()` (`process.rs:811`) returns a runtime error if a prompt file is
  missing. The bounded-test approach matches mature practice (assert-on-existence, not
  a framework).
- Finding: **APPROACH-ALIGNED**. The planned "bounded mechanical check" is the mature
  practice; no library is needed (the resolution logic already lives in `process.rs`).
- Impact: none. No PLAN-ADAPT.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `crates/sbfb-factory/Cargo.toml` (deps + dev-deps).
- Commands/sources: `grep -n "dependencies|^[a-z]" crates/sbfb-factory/Cargo.toml`.
  Dev-deps = `tempfile` only; runtime deps already include `serde_json`, `nexus-core-rs`.
- Finding: **clean**. Phase B adds NO dependency. The P2-F-3 test needs nothing beyond
  std + `serde_json` (already present) to read files and parse wrapper prose. The
  P2-A-2 assertion uses `nexus-core-rs` (already a `nexus-shell-daemon` dep) for
  `ResultEntry::verify_signature` and the existing `BlobsClient`. No CVE surface
  (the ollama-rs 0.2.6 -> 0.3.4 bump is Phase C, out of scope for B).

## S2 Historical Decisions
- Commands:
  - `git log --oneline --all -- .claude/agents prompts/agent` (reverse-commit check)
  - `rg "P2-F-3|prompt file coupling" .planning/archive/v2.1`
  - `git log --oneline -10`
- Decisions crossed:
  - **P2-F-3 origin**: `sprint70_phase_f_review.md:152` documented the finding —
    "agent wrapper files reference `prompts/agent/{phase-review,phase-auditor,preflight,
    audit-gate-checks}.md`; all 4 exist on disk; if they change without updating the
    wrappers, the wrappers could become stale ... carry-over awareness." Created as 1/3
    (`sprint70_verification.md:132`), 2/3 (`sprint71_verification.md:186`, "non escalade,
    differe S72").
  - **Reverse-commit check**: the coupling was introduced S70 (`c68e989` Phase C 8 kinds,
    `6fb95df` Phase F wrappers). The only subsequent commit touching these paths is
    `69019ed` (model 4.6->4.8) which did NOT add a guardrail. **No reversion exists.** The
    original rationale (stale wrappers if a prompt is renamed/moved) is still valid and
    still unaddressed: the existing `process.rs` test module (`resolve_kind_aliases`,
    `providers_list_is_canonical`, `repo_root_resolves`, lines 849-885) does NOT assert
    that the `PROMPT_KINDS` files exist, and `process.rs:741` reports `exists:bool` in
    JSON but never asserts it.
  - **Confirmation P2-F-3 is NOT already resolved**: no test, no hook, no failing lint
    enforces the wrapper->prompt coupling. So §G9 ("verify real state first") resolves to
    "still open, place a bounded check" — NOT "document clos". (R6 in kickoff §9 anticipated
    both outcomes; the real-state check lands on "open, bounded".)
- Finding: **clean** (confirmed open carry, no contradicted decision, no reversion to
  re-litigate). Implementing a bounded check honors the original S70 carry intent.
  Day-0 "Factory = crate externe hors daemon" is respected: the P2-F-3 test lives in
  `sbfb-factory` and only reads repo files; it does not pull worker-core/iroh.

## S3 Local Patterns And Threat Model
- Threats/contracts checked: P2-A-2 touches the **signature verification** invariant.
  `ResultEntry::verify_signature()` (`task.rs:431`) recomputes
  `canonical_bytes(&self.payload, DOMAIN_RESULT_V1)` and verifies the embedded
  `worker_pubkey` + `signature`. It is **self-contained**: the verifying key is carried
  inside the entry, so the E2E test does NOT need a separately-supplied worker public key
  (it verifies the worker self-attested its own result correctly). Adding this assertion
  STRENGTHENS coverage of the existing threat (forged/garbled result) — it is a coverage
  improvement, not a regression.
- HARDENING_ROADMAP status: N/A — Phase B introduces no new security component and no
  HARDENING pre-requirement is owed by S72 Phase B. (P2-H-1 Operator threat catalogue
  was the S71-audit pre-requirement and it was satisfied in Phase A, `105c054`.)
- Implementation note (non-blocking, see Risks): the worker-core test
  `engine_claims_and_executes_tasks_on_registered_doc` (`runtime.rs:1583-1590`) reads
  only the result entry KEY, with an explicit comment that the blob content lives in the
  node blob store. The E2E in `dispatch_loop.rs` mirrors that (asserts `results.len()==1`
  on entries only, line 234-235). To call `verify_signature()` the test must FETCH the
  blob: `blobs.get_bytes(*entry.content_hash().as_bytes())` ->
  `serde_json::from_slice::<ResultEntry>(&content)` -> `.verify_signature()`. The decode
  pattern is established (`docs.rs:603-607`, `:671-675`). `Engine.node` is private
  (`runtime.rs:143`) and there is no public `blobs()` accessor (only `docs()` at :562),
  so the assertion needs a small `pub fn blobs(&self) -> BlobsClient` on `Engine`
  (mirroring the S71 B-3 test-support `docs()` accessor), captured BEFORE the engine is
  moved into `tokio::spawn` (`dispatch_loop.rs:218`). This is bounded (~5-15 LOC), uses
  the already-imported `BlobsClient` (`runtime.rs:53`), and does not touch any signing or
  wire path.
- Finding: **clean** (no regression on any covered threat; P2-A-2 adds positive coverage).

## S4 Protocol And Wire Invariants
- Wire/security files checked: `crates/nexus-core-rs/src/canonical.rs` (read in full
  header: `DOMAIN_TASK_V1`, `DOMAIN_RESULT_V1`, `DOMAIN_CLAIM_V1`, JCS rationale),
  `crates/nexus-core-rs/src/task.rs` (TaskEntry/ResultEntry/Claim, `TASK_FORMAT_VERSION`).
- VERSION/domain/canonical status: Phase B changes **none** of them. P2-A-2 READS a
  `ResultEntry` and calls the existing `verify_signature()` — it does not alter
  `ResultPayload`, `DOMAIN_RESULT_V1`, or `TASK_FORMAT_VERSION`. No `*_VERSION` bump. No
  new tolerant multi-version decoder. No new `serde(default)`.
- Day 0 status: **preserved**. "Factory = crate externe hors daemon" (P2-F-3 test in
  `sbfb-factory`, file-reads only). "Pre-launch protocol: edit canonical freely but no
  gratuitous version bump" — Phase B bumps nothing. P3-B-1/B-2 are already documented in
  §P53 as deliberate (seed `as i32` is the ollama-rs API type, `sha256` column name is
  Sprint-55 build-task heritage holding raw text for inference) — no Day-0 conflict.
- Finding: **clean**.

## Plan Adaptation
Not applicable (verdict is EXECUTE, not PLAN-ADAPT).

## Risks And Scope Cuts
- Blocking risks: **none**.
- Non-blocking risks (carry into implementation):
  1. **P2-A-2 is slightly larger than a one-liner** (S3 note). It needs a blob fetch and
     likely a small `Engine::blobs()` test-support accessor. This is bounded and stays
     within the dette phase; it is not a refactor. Track in the commit body.
  2. **P2-F-3 coverage direction**: the 4 prompt files the wrappers reference are a SUBSET
     of `PROMPT_KINDS` resolution (verified: wrapper refs = phase-review.md, phase-auditor.md,
     preflight.md, audit-gate-checks.md; all in the 8-kind set). A test asserting every
     `PROMPT_KINDS` file exists transitively guarantees the 4 wrapper-referenced files
     exist, but does NOT catch a wrapper that references a file OUTSIDE the kind set (typo /
     renamed prompt). Recommend the test cover BOTH directions (see Action) to fully close
     the original S70 finding. Still bounded (~30-40 LOC, no new dep).
- Scope cuts still honored (kickoff §7): Phase B is the reserved dette phase
  (non-convertible to feature). It does NOT touch ollama-rs (Phase C), ExecutionTarget
  (Phase C), NetworkProvider (Phase D), the front (Phase E), or any search/fork/GPU scope
  (S73-S76). No wire format moves (PO-14, §1.4).

## Action
- **EXECUTE**: proceed with Phase B as planned, with two implementation specifics
  surfaced by the scans (both non-blocking):

  1. **P2-F-3 — recommended option (a), strengthened**: add ONE Rust test in
     `crates/sbfb-factory/src/process.rs` (the module that already owns `PROMPT_KINDS`,
     `prompt_filename`, and `repo_root`) that asserts, bidirectionally:
     (i) every `PROMPT_KINDS` entry resolves to an existing `prompts/agent/<kind>.md`;
     (ii) every `prompts/agent/*.md` path mentioned in `.claude/agents/*.md` exists on
     disk. Plus a short stability-contract note in `docs/agent/AGENT_SYSTEM.md §6`
     (the wrapper->prompt path coupling is enforced by this test; renaming a prompt
     requires updating the wrapper and the test stays green or fails fast). This closes
     P2-F-3 at 3/3 with a bounded mechanical guard — **plus jamais carry**.
     - Why (a) over (b)/(c): the resolution logic (`prompt_filename`, `repo_root`,
       `PROMPT_KINDS`) ALREADY lives in `process.rs`; a unit test there is co-located with
       the source of truth, runs on every `cargo nextest`, needs no new dep (`tempfile`
       dev-dep already present, but not even needed — repo_root reads the live tree), and
       is testable in isolation. Option (b) (extend `phase-precommit-lightcheck.sh`) buries
       the check in a 467-line bash hook that only fires at commit time and is not part of
       the test count; option (c) (doc-only contract in AGENT_SYSTEM.md) is not mechanical
       and would not fail-fast — it would re-create the same "prose can drift" gap the
       finding is about. (a) is the deepest, no-band-aid choice; the doc note from (c) is
       added on top of (a) as documentation, not as the guard.
  2. **P2-A-2**: add the signature assertion inside the existing E2E
     `dispatched_task_is_claimed_and_executed_by_worker_engine` (no new test). Capture a
     `BlobsClient` from the worker node BEFORE `engine` is moved into `tokio::spawn`
     (`dispatch_loop.rs:218`) — add a small `pub fn blobs(&self) -> BlobsClient` accessor
     on `Engine` mirroring the existing `docs()` (Sprint 71 B-3 test-support). Fetch the
     `result:` entry blob, deserialize `ResultEntry`, assert `verify_signature().is_ok()`.
  3. **P3-A-3 / P3-B-1 / P3-B-2**: re-doc/confirm only. All three are already documented
     in `docs/rust/PATTERNS.md §P53` (`sha256` column misnomer line 2743; `seed(s as i32)`
     line 2780, intentional ollama-rs API type). No code change in Phase B; confirm the
     §P53 notes still hold and reference them in the commit body. (Note: the `as i32` cast
     will naturally be re-verified by the Phase C ollama-rs 0.3.4 migration; no need to
     pre-empt it in B.)

- Commit body must record: P2-F-3 closed 3/3 via bounded `sbfb-factory` coupling test +
  AGENT_SYSTEM.md contract note; P2-A-2 signature assertion added to the existing E2E
  (with the small `Engine::blobs()` test accessor); P3 confirmed documented in §P53.
- No Codex exemption (docs-only does not exempt; this phase has code).
